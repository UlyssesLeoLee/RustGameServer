//! NATS JetStream Producer（per RGS-DTL-100 §5 消息总线）
//!
//! 54.10 实化：JetStream publish + 业务标识（subject 内嵌 + payload JSON envelope）
//!
//! 设计：
//! - JetStream 持久化消息（vs NATS core fire-and-forget）
//! - 业务上下文（command_id / saga_id）通过 JSON envelope 传递（简化 NATS header API）
//! - 自动 JSON 序列化 payload
//! - 失败自动重试（per RGS-SPEC-CROSS-006）

use async_nats::jetstream::Context;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::retry::{backoff_duration, RetryConfig};

/// Producer 错误
#[derive(Debug, Error)]
pub enum ProducerError {
    #[error("NATS error: {0}")]
    Nats(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Producer 配置
#[derive(Debug, Clone)]
pub struct ProducerConfig {
    /// 重试配置
    pub retry: RetryConfig,
    /// publish 超时
    pub timeout: Duration,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            retry: RetryConfig::default(),
            timeout: Duration::from_secs(5),
        }
    }
}

/// 业务消息 envelope（per RGS-DTL-100 §5.2 业务上下文）
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct MessageEnvelope<T> {
    /// command_id（幂等性 key）
    pub command_id: Uuid,
    /// saga_id
    pub saga_id: Option<Uuid>,
    /// actor_id
    pub actor_id: Option<Uuid>,
    /// trace_id（OTel 链路）
    pub trace_id: Option<String>,
    /// 业务 payload
    pub payload: T,
    /// 发布时间
    pub published_at: chrono::DateTime<chrono::Utc>,
}

impl<T> MessageEnvelope<T> {
    /// 工厂
    pub fn new(command_id: Uuid, payload: T) -> Self {
        Self {
            command_id,
            saga_id: None,
            actor_id: None,
            trace_id: None,
            payload,
            published_at: chrono::Utc::now(),
        }
    }

    /// 链式 setter
    pub fn with_saga(mut self, saga_id: Uuid) -> Self {
        self.saga_id = Some(saga_id);
        self
    }

    pub fn with_actor(mut self, actor_id: Uuid) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    pub fn with_trace(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/// NATS JetStream Producer
pub struct Producer {
    ctx: Context,
    config: ProducerConfig,
}

impl Producer {
    /// 工厂
    pub fn new(ctx: Context, config: ProducerConfig) -> Self {
        Self { ctx, config }
    }

    /// 发布 envelope 消息
    pub async fn publish<T: Serialize>(
        &self,
        subject: &str,
        envelope: &MessageEnvelope<T>,
    ) -> Result<(), ProducerError> {
        let json = serde_json::to_vec(envelope)?;
        self.publish_bytes(subject, json).await
    }

    /// 发布二进制消息（带 retry）
    pub async fn publish_bytes(
        &self,
        subject: &str,
        payload: Vec<u8>,
    ) -> Result<(), ProducerError> {
        let mut last_err: Option<ProducerError> = None;
        for attempt in 0..=self.config.retry.max_retries {
            let fut = self
                .ctx
                .publish(subject.to_string(), payload.clone().into());
            match tokio::time::timeout(self.config.timeout, fut).await {
                Ok(Ok(ack_future)) => match ack_future.await {
                    Ok(_ack) => return Ok(()),
                    Err(e) => {
                        last_err = Some(ProducerError::Nats(format!("ack error: {}", e)));
                    }
                },
                Ok(Err(e)) => {
                    last_err = Some(ProducerError::Nats(e.to_string()));
                }
                Err(_) => {
                    last_err = Some(ProducerError::Nats("publish timeout".to_string()));
                }
            }
            if attempt < self.config.retry.max_retries {
                let backoff = backoff_duration(attempt, &self.config.retry);
                tokio::time::sleep(backoff).await;
            }
        }
        Err(last_err.unwrap_or_else(|| ProducerError::Nats("unknown".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_new() {
        let cmd_id = Uuid::new_v4();
        let env = MessageEnvelope::new(cmd_id, "hello".to_string());
        assert_eq!(env.command_id, cmd_id);
        assert_eq!(env.payload, "hello");
        assert!(env.saga_id.is_none());
    }

    #[test]
    fn envelope_chaining() {
        let cmd_id = Uuid::new_v4();
        let saga_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();
        let env = MessageEnvelope::new(cmd_id, 100)
            .with_saga(saga_id)
            .with_actor(actor_id)
            .with_trace("trace-001".to_string());
        assert_eq!(env.saga_id, Some(saga_id));
        assert_eq!(env.actor_id, Some(actor_id));
        assert_eq!(env.trace_id, Some("trace-001".to_string()));
    }

    #[test]
    fn producer_config_default() {
        let cfg = ProducerConfig::default();
        assert_eq!(cfg.retry.max_retries, 3);
        assert_eq!(cfg.timeout, Duration::from_secs(5));
    }
}
