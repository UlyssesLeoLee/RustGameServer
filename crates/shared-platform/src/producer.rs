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
use async_nats::HeaderMap;
use opentelemetry::trace::TraceContextExt as _;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use crate::grpc_tracing::{build_traceparent, TRACEPARENT_HEADER};
use crate::retry::{backoff_duration, RetryConfig};

/// Producer 错误
#[derive(Debug, Error)]
pub enum ProducerError {
    #[error("NATS error: {0}")]
    Nats(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// 从当前 OTel Span 提取 (trace_id, span_id)（OTel 未启用时 fallback 新 UUID）
///
/// 复用逻辑：与 grpc_tracing::current_trace_ids 同源，本文件保留独立副本避免循环依赖
/// （producer 与 grpc_tracing 同级模块，模块内部可直接访问 OTel context）。
///
/// 行为：
/// - 当前 Span 已被 OTel subscriber 桥接（典型情况：业务层在 span 内调 publish）：
///   返回真实 (TraceId, SpanId)，分布式追踪贯通到 consumer 端
/// - OTel 未初始化或 Span 无 OTel context（测试 / 单进程 / OTel feature 未启用）：
///   fallback 到 (Uuid::new_v4(), Uuid::new_v4())，保持单进程调用不报错
fn current_nats_trace_ids() -> (Uuid, Uuid) {
    let span = Span::current();
    let otel_cx = span.context();
    let span_ref = otel_cx.span();
    let sc = span_ref.span_context();

    if sc.is_valid() {
        // OTel TraceId = 16 bytes → 直接拷到 UUID 16 bytes
        let trace_bytes = sc.trace_id().to_bytes();
        let mut trace_id_arr = [0u8; 16];
        trace_id_arr.copy_from_slice(&trace_bytes);
        let trace_id = Uuid::from_bytes(trace_id_arr);

        // OTel SpanId = 8 bytes → 高 8 字节填 0 + 低 8 字节真实值 → UUID 16 bytes
        let span_bytes = sc.span_id().to_bytes();
        let mut span_id_arr = [0u8; 16];
        span_id_arr[..8].copy_from_slice(&span_bytes);
        let span_id = Uuid::from_bytes(span_id_arr);

        (trace_id, span_id)
    } else {
        // OTel 未启用（fallback：单元测试 / 开发模式 / 53.12 任务未完成时）
        (Uuid::new_v4(), Uuid::new_v4())
    }
}

/// 构造包含 traceparent 的 NATS HeaderMap（per RGS-DTL-100 §7 + W3C Trace Context）
///
/// 用途：publish_with_headers 时把 traceparent 注入到 NATS message header，
///       consumer 端从 header 恢复 OTel context 形成分布式追踪链。
///
/// 行为：
/// - 当前 Span 已有 OTel context → 注入真实 trace_id / span_id（贯通）
/// - 当前 Span 无 OTel context → 注入新 UUID trace_id（fallback，单进程兼容）
fn build_traceparent_headers() -> HeaderMap {
    let (trace_id, span_id) = current_nats_trace_ids();
    let traceparent = build_traceparent(trace_id, span_id);
    let mut headers = HeaderMap::new();
    // HeaderValue::try_from 失败通常仅在 traceparent 含非法字符时，理论上不会发生
    if let Ok(value) = traceparent.parse::<async_nats::HeaderValue>() {
        headers.insert(TRACEPARENT_HEADER, value);
    }
    headers
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

    /// 发布二进制消息（带 retry + 55.45 traceparent 注入）
    ///
    /// 行为：
    /// - 每次 publish 前构造一次 HeaderMap（避免 OTel trace_id 在 retry 间漂移：
    ///   retry 仍用 producer 调用时的 trace_id，与 saga 跨域追踪语义一致）
    /// - 注入 W3C traceparent header（OTel 未启用时 no-op fallback，见 build_traceparent_headers）
    /// - 使用 async-nats 0.42 publish_with_headers（per 55.45 前置调研）
    pub async fn publish_bytes(
        &self,
        subject: &str,
        payload: Vec<u8>,
    ) -> Result<(), ProducerError> {
        tracing::debug!(
            operation = "nats_publish_entry",
            service = "shared-platform",
            component = "producer",
            subject = %subject,
            payload_bytes = payload.len(),
            "enter nats publish"
        );
        // 55.45 一次性构造 traceparent header（retry 复用同一 trace_id）
        let headers = build_traceparent_headers();
        let mut last_err: Option<ProducerError> = None;
        for attempt in 0..=self.config.retry.max_retries {
            // 55.45：使用 publish_with_headers 把 traceparent 注入 NATS message
            // 与 publish 区别：保留 HeaderMap，consumer 端可提取 traceparent 恢复 OTel context
            let fut = self.ctx.publish_with_headers(
                subject.to_string(),
                headers.clone(),
                payload.clone().into(),
            );
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

    /// 55.45 AC1：OTel 未启用时 build_traceparent_headers 仍能产出有效 traceparent header
    /// （fallback 路径使用新 UUID，验证单进程兼容）
    #[test]
    fn build_traceparent_headers_fallback_no_otel() {
        let headers = build_traceparent_headers();
        // 存在 traceparent header（即使 fallback 也写入）
        let tp = headers
            .get(TRACEPARENT_HEADER)
            .expect("traceparent header present (fallback or real)");
        let s = tp.as_str();
        // 格式：00-{32 hex}-{32 hex}-01
        let parts: Vec<&str> = s.split('-').collect();
        assert_eq!(parts.len(), 4, "traceparent 必须 4 段");
        assert_eq!(parts[0], "00", "version 固定 00");
        assert_eq!(parts[1].len(), 32, "trace_id 32 hex chars");
        assert_eq!(parts[2].len(), 32, "span_id 32 hex chars (含 zero-pad)");
        assert_eq!(parts[3], "01", "flags 固定 01");
    }

    /// 55.45 AC2：OTel 未启用时多次调用 → 每次生成不同 trace_id（确认 fallback 是按调用生成）
    #[test]
    fn build_traceparent_headers_fallback_unique_per_call() {
        let h1 = build_traceparent_headers();
        let h2 = build_traceparent_headers();
        let tp1 = h1.get(TRACEPARENT_HEADER).unwrap().as_str().to_string();
        let tp2 = h2.get(TRACEPARENT_HEADER).unwrap().as_str().to_string();
        assert_ne!(tp1, tp2, "fallback 路径每次调用应生成新 trace_id");
    }
}
