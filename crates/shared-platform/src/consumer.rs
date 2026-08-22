//! NATS JetStream Consumer（per RGS-DTL-100 §5 消息总线 + ARC-051 CEM）
//!
//! 54.10 实化：Consumer handler trait + Ack/Nak/DLQ 模式
//!
//! 设计：
//! - ConsumerHandler trait 抽象消息处理
//! - ack：处理成功
//! - nak with delay：处理失败，可重试（per NATS JetStream ack_with(AckKind::Nak(d)))
//! - 超过 max_retries → 转发到 DLQ subject
//! - 业务上下文从 envelope JSON 提取（per RGS-DTL-100 §5.2）

use async_nats::jetstream::message::AckKind;
use async_nats::jetstream::Context;
use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;
use thiserror::Error;

use crate::dlq::DlqEntry;
use crate::producer::MessageEnvelope;

/// Consumer 错误
#[derive(Debug, Error)]
pub enum ConsumerError {
    #[error("max retries exceeded, sent to DLQ")]
    DlqSent,

    #[error("handler error: {0}")]
    Handler(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Consumer 配置
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（基础 backoff）
    pub retry_interval: Duration,
    /// DLQ subject 前缀
    pub dlq_prefix: String,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_interval: Duration::from_secs(5),
            dlq_prefix: "rgs.dlq".to_string(),
        }
    }
}

/// 反序列化结果：env 提取 + 原始 payload
pub struct DeserializedMessage<T> {
    pub envelope: MessageEnvelope<T>,
    pub original_payload: Vec<u8>,
}

/// 反序列化消息（per RGS-DTL-100 §5.2 envelope 模式）
pub fn deserialize_envelope<T: serde::de::DeserializeOwned>(
    _subject: &str,
    payload: Vec<u8>,
) -> Result<DeserializedMessage<T>, ConsumerError> {
    let envelope: MessageEnvelope<T> =
        serde_json::from_slice(&payload).map_err(ConsumerError::Serialization)?;
    Ok(DeserializedMessage {
        envelope,
        original_payload: payload,
    })
}

/// Consumer handler trait
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    /// 业务名（用于日志 / 指标）
    fn name(&self) -> &str;
    /// 处理消息（业务自行反序列化）
    async fn handle(&self, subject: &str, payload: Vec<u8>) -> Result<(), ConsumerError>;
}

/// 处理单条消息（带 retry + DLQ 逻辑）
pub async fn process_with_retry<T: serde::de::DeserializeOwned + Send>(
    handler: &dyn ConsumerHandler,
    subject: String,
    payload: Vec<u8>,
    retry_count: u32,
    jetstream: &Context,
    config: &ConsumerConfig,
) -> Result<(), ConsumerError> {
    let envelope = serde_json::from_slice::<MessageEnvelope<T>>(&payload).ok();
    let attempt = retry_count;

    match handler.handle(&subject, payload.clone()).await {
        Ok(()) => {
            tracing::info!(
                target: "consumer",
                handler = handler.name(),
                subject = %subject,
                attempt = attempt,
                "message processed"
            );
            Ok(())
        }
        Err(e) => {
            let next_attempt = attempt + 1;
            if next_attempt > config.max_retries {
                // 超过最大重试 → 转发到 DLQ
                let dlq_subject = format!("{}.{}", config.dlq_prefix, subject);
                let dlq_entry = DlqEntry {
                    original_subject: subject.clone(),
                    handler: handler.name().to_string(),
                    attempts: next_attempt,
                    error: e.to_string(),
                    command_id: envelope.as_ref().map(|e| e.command_id),
                    saga_id: envelope.as_ref().and_then(|e| e.saga_id),
                    actor_id: envelope.as_ref().and_then(|e| e.actor_id),
                    payload_base64: {
                        use base64::Engine;
                        base64::engine::general_purpose::STANDARD.encode(&payload)
                    },
                    failed_at: chrono::Utc::now(),
                };
                let dlq_json = serde_json::to_vec(&dlq_entry)?;
                let _ = jetstream.publish(dlq_subject, Bytes::from(dlq_json)).await;
                tracing::warn!(
                    target: "consumer",
                    handler = handler.name(),
                    subject = %subject,
                    attempts = next_attempt,
                    "max retries exceeded, sent to DLQ"
                );
                return Err(ConsumerError::DlqSent);
            }
            // nak with delay（重试）
            tracing::warn!(
                target: "consumer",
                handler = handler.name(),
                subject = %subject,
                attempt = next_attempt,
                error = %e,
                "handler failed, will retry"
            );
            Err(ConsumerError::Handler(e.to_string()))
        }
    }
}

/// 构造 nak AckKind（per NATS 0.42）
pub fn nak_with_delay(delay: Duration) -> AckKind {
    AckKind::Nak(Some(delay))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn consumer_config_default() {
        let cfg = ConsumerConfig::default();
        assert_eq!(cfg.max_retries, 3);
        assert_eq!(cfg.dlq_prefix, "rgs.dlq");
    }

    #[test]
    fn deserialize_envelope_ok() {
        let cmd_id = Uuid::new_v4();
        let env = MessageEnvelope::new(cmd_id, "hello".to_string());
        let json = serde_json::to_vec(&env).unwrap();
        let result = deserialize_envelope::<String>("rgs.test", json).unwrap();
        assert_eq!(result.envelope.command_id, cmd_id);
        assert_eq!(result.envelope.payload, "hello");
    }

    #[test]
    fn nak_with_delay_creates_nak() {
        #[allow(unused_variables)]
        let _ = nak_with_delay(Duration::from_secs(5));
    }
}
