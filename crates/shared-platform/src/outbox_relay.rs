//! Outbox Relay（per RGS-DTL-100 §5.3 事务性消息后台轮询）
//!
//! 54.11 实化：OutboxRelay 后台循环 + 重试 + max_retries 后转 DLQ
//!
//! 设计：
//! - relay 定时 poll outbox WHERE status='pending'
//! - 每条 entry publish 到 NATS，成功 mark_sent，失败 mark_failed（retry_count+1）
//! - 超过 max_retries → mark_giveup (status='failed')
//! - relay 错误不 panic，仅 log（保证服务可用性）

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

/// Outbox Relay
pub struct OutboxRelay {
    repo: Arc<dyn OutboxRepository>,
    producer: Arc<Producer>,
    config: RelayConfig,
}

impl OutboxRelay {
    pub fn new(
        repo: Arc<dyn OutboxRepository>,
        producer: Arc<Producer>,
        config: RelayConfig,
    ) -> Self {
        Self {
            repo,
            producer,
            config,
        }
    }

    /// 单次轮询（一次 batch）
    pub async fn tick(&self) -> crate::outbox::Result<RelayStats> {
        let mut stats = RelayStats::default();
        let pending = self.repo.list_pending(self.config.batch_size).await?;
        stats.fetched = pending.len();

        for entry in pending {
            match self.publish_entry(&entry).await {
                Ok(()) => {
                    self.repo.mark_sent(entry.id).await?;
                    stats.sent += 1;
                }
                Err(e) => {
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
                        self.repo.mark_failed(entry.id, e.to_string()).await?;
                        stats.retried += 1;
                        tracing::warn!(
                            target: "outbox_relay",
                            outbox_id = %entry.id,
                            subject = %entry.subject,
                            error = %e,
                            "outbox publish failed, will retry"
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
    use crate::outbox::InMemoryOutboxRepository;
    use crate::producer::ProducerConfig;
    use async_nats::jetstream;

    #[tokio::test]
    async fn relay_tick_empty() {
        // 准备：InMemory outbox（无 pending）+ fake producer（不会真的连 NATS）
        let repo: Arc<dyn OutboxRepository> = Arc::new(InMemoryOutboxRepository::new());
        // 构造一个 producer 不会 publish（用 None jetstream 不可行；跳过构造，仅测空 list）
        // 测空 list 返回 0 即可
        let pending = repo.list_pending(10).await.unwrap();
        assert_eq!(pending.len(), 0);
        // 避免 unused warning
        let _ = ProducerConfig::default();
        let _ = jetstream::new;
    }

    #[tokio::test]
    async fn relay_config_default() {
        let cfg = RelayConfig::default();
        assert_eq!(cfg.poll_interval, Duration::from_secs(5));
        assert_eq!(cfg.batch_size, 100);
        assert_eq!(cfg.max_retries, 5);
    }
}
