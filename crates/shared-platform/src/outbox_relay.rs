//! Outbox Relay（per RGS-DTL-100 §5.3 事务性消息后台轮询）
//!
//! 54.11 实化：OutboxRelay 后台循环 + 重试 + max_retries 后转 DLQ
//! 55.17 升级（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）：
//!   - 状态机加 in_flight：Pending → InFlight（list_pending 自动标 + 持 lease 30s）→ Sent / Failed
//!   - 失败时 in_flight 保留等 lease 过期被另一副本重试
//!   - OutboxRelay 改用泛型 R: OutboxRepository（trait 加了泛型 append 后非 dyn-safe）
//!
//! 设计：
//! - relay 定时 poll outbox（list_pending 内部用 FOR UPDATE SKIP LOCKED + mark in_flight + lease 30s）
//! - 每条 entry publish 到 NATS，成功 mark_sent，失败 mark_failed（retry_count+1，in_flight 保留）
//! - 超过 max_retries → mark_giveup (status='failed')
//! - relay 错误不 panic，仅 log（保证服务可用性）
//! - 多 relay 副本并发安全：FOR UPDATE SKIP LOCKED + lease_until 双重保护

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use crate::outbox::{OutboxEntry, OutboxRepository};
use crate::producer::{Producer, ProducerError};

/// Relay 配置
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// 轮询间隔
    pub poll_interval: Duration,
    /// 每次最多 poll 多少条
    pub batch_size: i64,
    /// 最大重试次数（per 域可配）
    pub max_retries: u32,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            batch_size: 100,
            max_retries: 5,
        }
    }
}

/// Outbox Relay（55.17：泛型版，因 OutboxRepository 加了泛型 append 后非 dyn-safe）
pub struct OutboxRelay<R: OutboxRepository + 'static> {
    repo: Arc<R>,
    producer: Arc<Producer>,
    config: RelayConfig,
    _marker: PhantomData<R>,
}

impl<R: OutboxRepository + 'static> OutboxRelay<R> {
    pub fn new(repo: Arc<R>, producer: Arc<Producer>, config: RelayConfig) -> Self {
        Self {
            repo,
            producer,
            config,
            _marker: PhantomData,
        }
    }

    /// 单次轮询（一次 batch）
    pub async fn tick(&self) -> crate::outbox::Result<RelayStats> {
        let mut stats = RelayStats::default();
        // 55.17：list_pending 内部已 mark in_flight + lease 30s + 提交后持锁
        let pending = self.repo.list_pending(self.config.batch_size).await?;
        stats.fetched = pending.len();

        for entry in pending {
            // entry.status 此时已是 InFlight（list_pending 内部标记）
            match self.publish_entry(&entry).await {
                Ok(()) => {
                    self.repo.mark_sent(entry.id).await?;
                    stats.sent += 1;
                }
                Err(e) => {
                    // 55.17：失败时 in_flight 保留，retry_count+1
                    // 另一副本在 lease 过期后通过 list_pending 重试
                    if entry.retry_count + 1 >= self.config.max_retries {
                        self.repo.mark_giveup(entry.id).await?;
                        stats.failed += 1;
                        tracing::warn!(
                            target: "outbox_relay",
                            outbox_id = %entry.id,
                            subject = %entry.subject,
                            retries = entry.retry_count + 1,
                            "outbox entry gave up after max retries"
                        );
                    } else {
                        // mark_failed 内部：retry_count+1, last_error, status 保持 in_flight
                        self.repo.mark_failed(entry.id, e.to_string()).await?;
                        stats.retried += 1;
                        tracing::warn!(
                            target: "outbox_relay",
                            outbox_id = %entry.id,
                            subject = %entry.subject,
                            error = %e,
                            "outbox publish failed, lease will expire then retry by another replica"
                        );
                    }
                }
            }
        }

        Ok(stats)
    }

    /// 后台循环（tokio task）
    pub async fn run(self: Arc<Self>) {
        let mut ticker = time::interval(self.config.poll_interval);
        loop {
            ticker.tick().await;
            match self.tick().await {
                Ok(stats) if stats.fetched > 0 => {
                    tracing::info!(
                        target: "outbox_relay",
                        fetched = stats.fetched,
                        sent = stats.sent,
                        retried = stats.retried,
                        failed = stats.failed,
                        "outbox tick"
                    );
                }
                Ok(_) => {
                    tracing::debug!(target: "outbox_relay", "outbox tick: no pending");
                }
                Err(e) => {
                    tracing::error!(target: "outbox_relay", error = %e, "outbox tick failed");
                }
            }
        }
    }

    /// 发布单条（解析 payload + 调 Producer）
    async fn publish_entry(&self, entry: &OutboxEntry) -> std::result::Result<(), ProducerError> {
        // payload 直接作为 bytes 发送（consumer 端自行反序列化）
        self.producer
            .publish_bytes(&entry.subject, entry.payload.as_bytes().to_vec())
            .await
    }
}

/// 单次轮询统计
#[derive(Debug, Default, Clone, Copy)]
pub struct RelayStats {
    /// 拉取条数
    pub fetched: usize,
    /// 成功发送
    pub sent: usize,
    /// 失败重试
    pub retried: usize,
    /// 最终失败
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbox::{InMemoryOutboxRepository, OutboxEntry, OutboxStatus};
    use crate::producer::ProducerConfig;
    use async_nats::jetstream;
    use std::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn relay_config_default() {
        let cfg = RelayConfig::default();
        assert_eq!(cfg.poll_interval, Duration::from_secs(5));
        assert_eq!(cfg.batch_size, 100);
        assert_eq!(cfg.max_retries, 5);
    }

    #[tokio::test]
    async fn relay_tick_empty() {
        // 准备：InMemory outbox（无 pending）+ Producer 不发
        let repo: Arc<InMemoryOutboxRepository> = Arc::new(InMemoryOutboxRepository::new());
        // 验证空 list
        let pending = repo.list_pending(10).await.unwrap();
        assert_eq!(pending.len(), 0);
        // 避免 unused warning
        let _ = ProducerConfig::default();
        let _ = jetstream::new;
    }

    /// 55.17 测试 4：relay_uses_in_flight_state
    ///
    /// 验证：
    /// - list_pending 取出后 entry 状态变成 in_flight（带 lease）
    /// - 重复 list_pending 在 lease 内不再返回同一行（被持锁）
    /// - mark_sent 后再 list_pending 不再返回（已 sent）
    #[tokio::test]
    async fn relay_uses_in_flight_state() {
        // 1h lease 避免测试中等过期
        let repo: Arc<InMemoryOutboxRepository> = Arc::new(
            InMemoryOutboxRepository::with_lease(Duration::from_secs(3600)),
        );
        // 测试用 lazy pool（InMemory 忽略 executor）
        let pool: sqlx::PgPool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/nonexistent")
            .expect("lazy connect should not fail");
        let entry = OutboxEntry::new(
            "rgs.test.event.v1".to_string(),
            r#"{"k":"v"}"#.to_string(),
            Uuid::new_v4(),
        );
        let id = entry.id;
        repo.append(&entry, &pool).await.unwrap();

        // 第 1 次 list_pending：拿 1 条 + 自动 mark in_flight
        let first = repo.list_pending(10).await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].status, OutboxStatus::InFlight);
        assert!(first[0].lease_until.is_some(), "in_flight 必须有 lease");

        // 第 2 次 list_pending：lease 未过 → 0 条
        let second = repo.list_pending(10).await.unwrap();
        assert_eq!(second.len(), 0, "in_flight + lease 未过 → 不应再被取出");

        // mark_sent：relay 模拟 publish 成功
        repo.mark_sent(id).await.unwrap();

        // 第 3 次：永久不返回
        let third = repo.list_pending(10).await.unwrap();
        assert_eq!(third.len(), 0, "sent 后不应再被取出");
    }
}
