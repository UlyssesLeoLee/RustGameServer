//! 排队层 — NATS JetStream 后端
//!
//! 关键点（per 任务 §1 + handoff §1/§2/§3）：
//! - **复用** `shared_platform::messaging::build_messaging_client`，不要自己引独立版本
//! - subject 构造走 `shared_platform::subject::SubjectBuilder::domain_event`
//! - stream 覆盖 4 域 subject filter = `rgs.*.overflow.v1`
//! - 超 `max_pending` → `QueueError::QueueFull`
//!
//! **多副本并发启动** stream create 竞态处理：依赖 NATS 自身的 stream create idempotency
//! —— `jetstream` 0.42 的 `get_or_create_stream` 在 stream 已存在且配置兼容时**不报错**。
//! 配置不兼容时返回错误，由上层决定是否降级到 `LogOnlyQueue` 兜底（本期不实化）。

use crate::config::OverflowConfig;
use crate::domain::Domain;
use async_nats::jetstream;
use async_nats::jetstream::stream::{Config as StreamConfig, StorageType};
use async_nats::jetstream::stream::RetentionPolicy;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared_platform::messaging::{build_messaging_client, MessagingConfig};
use shared_platform::subject::SubjectBuilder;
use std::sync::Arc;
use thiserror::Error;

/// 队列错误
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("queue full: max_pending={0}")]
    QueueFull(u64),
    #[error("NATS connect error: {0}")]
    Connect(String),
    #[error("stream config error: {0}")]
    StreamConfig(String),
    #[error("publish error: {0}")]
    Publish(String),
    #[error("ack token encode error: {0}")]
    Encode(String),
}

/// 入队后返回的 ack token（客户端可用来查询 / 取消）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AckToken {
    /// 域
    pub domain: String,
    /// NATS sequence（流上的位置）
    pub sequence: u64,
    /// 入队时间（RFC3339）
    pub enqueued_at: String,
}

impl AckToken {
    pub fn encode(&self) -> Result<String, QueueError> {
        serde_json::to_string(self).map_err(|e| QueueError::Encode(e.to_string()))
    }
    pub fn decode(s: &str) -> Result<Self, QueueError> {
        serde_json::from_str(s).map_err(|e| QueueError::Encode(e.to_string()))
    }
}

/// 入队 payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverflowPayload {
    /// 业务调用方传的 op name（用于排障 — 例如 `PlayerService::Register`）
    pub op: String,
    /// 业务请求 ID（UUID4 字符串，调用方生成）
    pub request_id: String,
    /// 域
    pub domain: String,
    /// in-flight 占用（告警 / 监控用）
    pub in_flight: u32,
    /// 硬上限
    pub hard_cap: u32,
    /// 软阈值
    pub soft_cap: u32,
    /// Pod 名（hostname fallback）
    pub pod: String,
    /// service name
    pub service: String,
    /// 业务 JSON（任意 — 业务调用方序列化后传入）
    pub business_json: Option<String>,
    /// 首次发生时间
    pub first_at: String,
    /// 当前 / 末次发生时间
    pub last_at: String,
    /// 5min reject 计数
    pub reject_count_5min: u64,
}

/// 队列后端 trait
#[async_trait]
pub trait QueueBackend: Send + Sync {
    /// 入队一条超限请求；超 `max_pending` → `QueueError::QueueFull`
    async fn enqueue(
        &self,
        domain: Domain,
        payload: &OverflowPayload,
    ) -> Result<AckToken, QueueError>;
}

/// NATS JetStream 后端
pub struct NatsJsQueueBackend {
    js_ctx: jetstream::Context,
    /// 客户端名（keepalive，drop 时连接关闭）
    _client: async_nats::Client,
    /// 流名
    stream: String,
    /// 单 stream 覆盖 4 域 subject 模板（publish 时用 SubjectBuilder 解析为实际 subject）
    /// = `rgs.*.overflow.v1`
    subject_filter: String,
    /// max_msgs（per 任务 §1：超此返回 QueueFull）
    max_pending: u64,
    /// 当前 stream 上的消息数（approximate — JS 自维护）
    current_msgs: Arc<std::sync::atomic::AtomicU64>,
}

impl NatsJsQueueBackend {
    /// 启动：build_messaging_client + get_or_create_stream
    ///
    /// 失败 → `QueueError::Connect` / `StreamConfig`
    pub async fn connect(cfg: &OverflowConfig) -> Result<Self, QueueError> {
        let (client, js_ctx) = build_messaging_client(&MessagingConfig {
            uri: cfg.nats_uri.clone(),
            name: "rgs-overflow-alert".to_string(),
        })
        .await
        .map_err(|e| QueueError::Connect(e.to_string()))?;

        // 4 域共享一个 stream；subject filter 通配 4 域
        let subject_filter = "rgs.*.overflow.v1".to_string();
        let stream_cfg = StreamConfig {
            name: cfg.stream_name.clone(),
            subjects: vec![subject_filter.clone()],
            storage: StorageType::File,
            retention: RetentionPolicy::Limits,
            max_messages: cfg.max_pending as i64,
            max_bytes: 1024 * 1024 * 1024, // 1 GiB per stream（足够；每条 payload 几 KB）
            ..Default::default()
        };
        js_ctx
            .get_or_create_stream(stream_cfg)
            .await
            .map_err(|e| QueueError::StreamConfig(e.to_string()))?;

        // current_msgs initial 0（approximate；publish 时 +1，consumer 不可见时不影响 enqueue 软检查）
        Ok(Self {
            js_ctx,
            _client: client,
            stream: cfg.stream_name.clone(),
            subject_filter,
            max_pending: cfg.max_pending,
            current_msgs: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// 当前 stream 上的 pending 消息数（approximate）
    pub fn current_pending(&self) -> u64 {
        self.current_msgs
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// stream 名
    pub fn stream_name(&self) -> &str {
        &self.stream
    }

    /// subject filter（debug / health endpoint 用）
    pub fn subject_filter(&self) -> &str {
        &self.subject_filter
    }

    /// 给定域的 NATS subject（per shared_platform::subject 规范）
    pub fn subject_for(domain: Domain) -> String {
        SubjectBuilder::domain_event(domain.as_str(), "overflow", 1)
    }
}

#[async_trait]
impl QueueBackend for NatsJsQueueBackend {
    async fn enqueue(
        &self,
        domain: Domain,
        payload: &OverflowPayload,
    ) -> Result<AckToken, QueueError> {
        // 软检查：当前 pending 已 ≥ max_pending → 立刻返回 QueueFull
        if self.current_pending() >= self.max_pending {
            return Err(QueueError::QueueFull(self.max_pending));
        }
        let subject = Self::subject_for(domain);
        let body = serde_json::to_vec(payload).map_err(|e| QueueError::Publish(e.to_string()))?;
        let ack_fut = self.js_ctx.publish(subject, body.into()).await;
        let ack = match ack_fut {
            Ok(ack) => ack,
            Err(e) => {
                let s = e.to_string();
                // async-nats 0.42 PublishErrorKind: 检查字符串包含 "max messages" 兜底识别
                if s.contains("max messages") || s.contains("max bytes") {
                    return Err(QueueError::QueueFull(self.max_pending));
                }
                return Err(QueueError::Publish(s));
            }
        };
        let ack = ack
            .await
            .map_err(|e| QueueError::Publish(e.to_string()))?;
        let seq = ack.sequence;
        // approximate update
        self.current_msgs
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(AckToken {
            domain: domain.as_str().to_string(),
            sequence: seq,
            enqueued_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

/// 内存后端（**仅 dev / test 用**）—— 集成测试 e2e 1000 并发验证 Pass/Queue/Reject 比例
pub struct InMemoryQueueBackend {
    inner: Arc<std::sync::Mutex<Vec<(Domain, OverflowPayload)>>>,
    max_pending: u64,
}

impl InMemoryQueueBackend {
    pub fn new(max_pending: u64) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(Vec::new())),
            max_pending,
        }
    }
    pub fn snapshot(&self) -> Vec<(Domain, OverflowPayload)> {
        self.inner.lock().expect("mem queue mutex").clone()
    }
    pub fn len(&self) -> usize {
        self.inner.lock().expect("mem queue mutex").len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl QueueBackend for InMemoryQueueBackend {
    async fn enqueue(
        &self,
        domain: Domain,
        payload: &OverflowPayload,
    ) -> Result<AckToken, QueueError> {
        let mut g = self.inner.lock().expect("mem queue mutex");
        if g.len() as u64 >= self.max_pending {
            return Err(QueueError::QueueFull(self.max_pending));
        }
        g.push((domain, payload.clone()));
        Ok(AckToken {
            domain: domain.as_str().to_string(),
            sequence: g.len() as u64,
            enqueued_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OverflowConfig;

    fn sample_payload(domain: Domain) -> OverflowPayload {
        OverflowPayload {
            op: "Test::Op".to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            domain: domain.as_str().to_string(),
            in_flight: 0,
            hard_cap: 10,
            soft_cap: 8,
            pod: "test-pod".to_string(),
            service: "test-service".to_string(),
            business_json: Some(r#"{"k":"v"}"#.to_string()),
            first_at: chrono::Utc::now().to_rfc3339(),
            last_at: chrono::Utc::now().to_rfc3339(),
            reject_count_5min: 0,
        }
    }

    #[tokio::test]
    async fn in_memory_queue_enqueues_and_caps() {
        let q = InMemoryQueueBackend::new(3);
        for _ in 0..3 {
            q.enqueue(Domain::Player, &sample_payload(Domain::Player))
                .await
                .unwrap();
        }
        let err = q
            .enqueue(Domain::Player, &sample_payload(Domain::Player))
            .await
            .unwrap_err();
        assert!(matches!(err, QueueError::QueueFull(3)));
    }

    #[test]
    fn subject_format_matches_shared_platform() {
        // 锚定 shared_platform::subject::SubjectBuilder::domain_event
        assert_eq!(
            NatsJsQueueBackend::subject_for(Domain::Player),
            "rgs.player.overflow.v1"
        );
        assert_eq!(
            NatsJsQueueBackend::subject_for(Domain::Economy),
            "rgs.economy.overflow.v1"
        );
        assert_eq!(
            NatsJsQueueBackend::subject_for(Domain::Match),
            "rgs.match.overflow.v1"
        );
        assert_eq!(
            NatsJsQueueBackend::subject_for(Domain::Social),
            "rgs.social.overflow.v1"
        );
    }

    #[test]
    fn ack_token_round_trip() {
        let t = AckToken {
            domain: "player".to_string(),
            sequence: 42,
            enqueued_at: chrono::Utc::now().to_rfc3339(),
        };
        let s = t.encode().unwrap();
        let d = AckToken::decode(&s).unwrap();
        assert_eq!(d, t);
    }

    #[test]
    fn nats_js_backend_subject_filter_covers_4_domains() {
        // 行为约束：subject filter = "rgs.*.overflow.v1" 应覆盖 4 域实际 subject
        for d in Domain::ALL {
            let s = NatsJsQueueBackend::subject_for(d);
            assert!(
                s.starts_with("rgs.") && s.ends_with(".overflow.v1"),
                "subject {} does not match expected pattern",
                s
            );
        }
    }

    #[test]
    fn config_streams_name_is_rgs_overflow() {
        // 锚定任务 §1：stream = RGS_OVERFLOW
        std::env::remove_var("NATS_OVERFLOW_STREAM");
        let cfg = OverflowConfig::from_env().unwrap();
        assert_eq!(cfg.stream_name, "RGS_OVERFLOW");
    }
}
