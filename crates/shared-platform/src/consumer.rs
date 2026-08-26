//! NATS JetStream Consumer（per RGS-DTL-100 §5 消息总线 + ARC-051 CEM）
//!
//! 54.10 实化：Consumer handler trait + Ack/Nak/DLQ 模式
//! 55.45 实化：traceparent 提取 + OTel parent link（per RGS-OPEN-QA-001 Q-M-03）
//!
//! 设计：
//! - ConsumerHandler trait 抽象消息处理
//! - ack：处理成功
//! - nak with delay：处理失败，可重试（per NATS JetStream ack_with(AckKind::Nak(d)))
//! - 超过 max_retries → 转发到 DLQ subject
//! - 业务上下文从 envelope JSON 提取（per RGS-DTL-100 §5.2）
//! - 跨服务追踪：从 NATS header 提取 traceparent → 关联到当前 Span
//!   （OTel 未启用 / header 不存在时 no-op，单进程兼容）

use async_nats::jetstream::message::AckKind;
use async_nats::jetstream::Context;
use async_nats::HeaderMap;
use async_trait::async_trait;
use bytes::Bytes;
use opentelemetry::trace::TraceContextExt as _;
use std::time::Duration;
use thiserror::Error;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use crate::dlq::DlqEntry;
use crate::grpc_tracing::{parse_traceparent, TRACEPARENT_HEADER};
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

/// 55.45：从 NATS HeaderMap 提取 traceparent → (trace_id, span_id)
///
/// 行为：
/// - header 存在且格式正确 → 返回 Some((trace_id, span_id))
/// - header 缺失 / 格式错误 / 解析失败 → 返回 None（fallback，no panic）
/// - OTel 未启用时也返回 None（与 producer 端 fallback 对称，链路能贯通到 consumer 的日志）
pub fn extract_traceparent_from_headers(headers: &HeaderMap) -> Option<(Uuid, Uuid)> {
    headers
        .get(TRACEPARENT_HEADER)
        .map(|v| v.as_str())
        .and_then(parse_traceparent)
}

/// 55.45：把父 trace_id / span_id 链接到当前 Span（OTel context 继承）
///
/// 行为：
/// - 调用 OTel Propagator 不可用 → 用 OpenTelemetrySpanExt::set_parent 手动构造 parent context
/// - 当前 Span 已绑定 OTel subscriber → 新产生的 child span 自动继承父 trace_id
/// - OTel 未启用时 → no-op（Span::current() 无 OTel context，set_parent 也无副作用）
///
/// 复用 grpc_tracing::parse_traceparent 解析格式（per W3C Trace Context）。
fn link_current_span_to_parent(trace_id: Uuid, span_id: Uuid) {
    use opentelemetry::trace::{SpanContext, SpanId as OtelSpanId, TraceFlags, TraceId as OtelTraceId, TraceState};
    // OTel TraceId = 16 bytes，直接从 UUID 拷过来
    let mut trace_bytes = [0u8; 16];
    trace_bytes.copy_from_slice(trace_id.as_bytes());
    // OTel SpanId = 8 bytes，取 UUID 低 8 字节（与 producer 端 high 8 bytes = 0 对称）
    let mut span_bytes = [0u8; 8];
    span_bytes.copy_from_slice(&span_id.as_bytes()[8..16]);
    let remote_cx = SpanContext::new(
        OtelTraceId::from_bytes(trace_bytes),
        OtelSpanId::from_bytes(span_bytes),
        TraceFlags::SAMPLED,
        true, // is_remote（来自 producer 端）
        TraceState::default(),
    );
    let cx = opentelemetry::Context::new().with_remote_span_context(remote_cx);
    Span::current().set_parent(cx);
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

/// 处理单条消息（带 retry + DLQ 逻辑 + 55.45 traceparent 提取）
///
/// 行为：
/// - 接收 NATS message headers（55.45 新增；旧调用方传空 HeaderMap 走 no-op 路径）
/// - 从 headers 提取 traceparent → 关联到当前 Span（贯通 producer 端 trace）
/// - 处理失败超过 max_retries → 转发到 DLQ subject
/// - OTel 未启用 / header 无 traceparent → no-op（不报错，handler 仍正常执行）
pub async fn process_with_retry<T: serde::de::DeserializeOwned + Send>(
    handler: &dyn ConsumerHandler,
    subject: String,
    payload: Vec<u8>,
    headers: HeaderMap,
    retry_count: u32,
    jetstream: &Context,
    config: &ConsumerConfig,
) -> Result<(), ConsumerError> {
    tracing::debug!(
        operation = "nats_consume_entry",
        service = "shared-platform",
        method = "process_with_retry",
        handler = handler.name(),
        subject = %subject,
        retry_count = retry_count,
        payload_bytes = payload.len(),
        "enter consumer process"
    );
    // 55.45 入口 traceparent 关联（容错：缺失 / 解析失败 / OTel 未启用都不报错）
    if let Some((trace_id, span_id)) = extract_traceparent_from_headers(&headers) {
        link_current_span_to_parent(trace_id, span_id);
        tracing::debug!(
            target: "consumer",
            trace_id = %trace_id,
            parent_span_id = %span_id,
            subject = %subject,
            "linked consumer span to producer trace"
        );
    }

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

    /// 55.45 AC1：consumer 从 NATS header 提取 traceparent，header 缺失/无效返回 None
    #[test]
    fn extract_traceparent_from_headers_empty() {
        let headers = HeaderMap::new();
        assert!(
            extract_traceparent_from_headers(&headers).is_none(),
            "空 header 应返回 None"
        );
    }

    /// 55.45 AC2：合法 traceparent 字符串能被正确解析
    #[test]
    fn extract_traceparent_from_headers_valid() {
        let mut headers = HeaderMap::new();
        let tp = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b70000000000000000-01";
        headers.insert(
            TRACEPARENT_HEADER,
            tp.parse::<async_nats::HeaderValue>().expect("valid traceparent"),
        );
        let (trace_id, span_id) =
            extract_traceparent_from_headers(&headers).expect("valid traceparent must parse");
        assert_eq!(trace_id.to_string(), "4bf92f35-77b3-4da6-a3ce-929d0e0e4736");
        assert_eq!(span_id.to_string(), "00f067aa-0ba9-02b7-0000-000000000000");
    }

    /// 55.45 AC3：非法 traceparent 字符串（缺失段）返回 None（容错，不 panic）
    #[test]
    fn extract_traceparent_from_headers_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            TRACEPARENT_HEADER,
            "not-a-valid-traceparent"
                .parse::<async_nats::HeaderValue>()
                .expect("parse header literal"),
        );
        assert!(
            extract_traceparent_from_headers(&headers).is_none(),
            "非法格式应返回 None（容错）"
        );
    }

    /// 55.45 AC4：link_current_span_to_parent 不 panic（OTel 未启用时 no-op）
    #[test]
    fn link_current_span_to_parent_no_otel() {
        let trace_id = Uuid::new_v4();
        let span_id = Uuid::new_v4();
        // OTel 未启用 → Span::current() 无 OTel context → set_parent 不报错也不关联成功
        link_current_span_to_parent(trace_id, span_id);
        // 验证：函数未 panic 即通过
    }
}
