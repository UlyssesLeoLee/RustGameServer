//! gRPC 链路追踪（per RGS-DTL-100 §7 + ARC-051 跨服务追踪）
//!
//! 54.12 实化：tonic interceptor 注入 trace context
//!
//! 设计：
//! - traceparent header（W3C trace context）格式：version-trace_id-parent_span_id-flags
//! - client interceptor：从当前 tracing Span 提取 trace_id / span_id → metadata traceparent
//! - server interceptor：从 metadata traceparent 提取 trace_id / span_id → 当前 Span record

use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};
use tonic::service::interceptor::{self, InterceptorLayer};
use tonic::{Request, Status};
use tracing::Span;
use uuid::Uuid;

/// gRPC traceparent header 名（per W3C Trace Context）
pub const TRACEPARENT_HEADER: &str = "traceparent";

/// traceparent 格式：version-trace_id-parent_span_id-flags
fn build_traceparent(trace_id: Uuid, span_id: Uuid) -> String {
    format!("00-{}-{}-01", trace_id.simple(), span_id.simple())
}

/// parse traceparent header → (trace_id, span_id)
fn parse_traceparent(value: &str) -> Option<(Uuid, Uuid)> {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 4 {
        return None;
    }
    // 接受 32-char hex (无连字符) → 加连字符转标准 UUID
    let trace_id = parse_uuid_hex(parts[1])?;
    let span_id = parse_uuid_hex(parts[2])?;
    Some((trace_id, span_id))
}

/// 32-char hex → Uuid
fn parse_uuid_hex(s: &str) -> Option<Uuid> {
    if s.len() != 32 {
        return None;
    }
    let formatted = format!(
        "{}-{}-{}-{}-{}",
        &s[0..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..32]
    );
    Uuid::parse_str(&formatted).ok()
}

/// Client Interceptor（构造 traceparent 占位 header；由 tracing-opentelemetry 在外层桥接）
#[allow(clippy::result_large_err)]
pub fn client_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    // 占位：tracing span context 通过 tracing_opentelemetry::OpenTelemetrySpanExt 桥接
    // 实际 trace_id / span_id 在调用方 span.enter() 后从 Span::current() 拿
    let span = Span::current();
    let _ = span; // 占位 — 完整 trace_id 提取需 tracing-opentelemetry 0.25 API 适配
    let trace_id = Uuid::new_v4();
    let span_id = Uuid::new_v4();
    let traceparent = build_traceparent(trace_id, span_id);
    let mut request = request;
    if let Ok(value) = MetadataValue::try_from(traceparent) {
        request
            .metadata_mut()
            .insert(MetadataKey::from_static(TRACEPARENT_HEADER), value);
    }
    Ok(request)
}

/// Server Interceptor（从 metadata 提取 traceparent + 链接到当前 Span）
#[allow(clippy::result_large_err)]
pub fn server_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    let metadata = request.metadata();
    if let Some(value) = metadata.get(TRACEPARENT_HEADER) {
        if let Ok(s) = value.to_str() {
            if let Some((trace_id, span_id)) = parse_traceparent(s) {
                let span = Span::current();
                span.record("trace_id", trace_id.to_string().as_str());
                span.record("parent_span_id", span_id.to_string().as_str());
            }
        }
    }
    Ok(request)
}

/// 构造 Client InterceptorLayer
pub fn client_interceptor_layer(
) -> InterceptorLayer<impl FnMut(Request<()>) -> Result<Request<()>, Status> + Clone> {
    interceptor::interceptor(client_interceptor)
}

/// 构造 Server InterceptorLayer
pub fn server_interceptor_layer(
) -> InterceptorLayer<impl FnMut(Request<()>) -> Result<Request<()>, Status> + Clone> {
    interceptor::interceptor(server_interceptor)
}

/// 业务层 helper：从 metadata 提取 trace_id
pub fn extract_trace_id(metadata: &MetadataMap) -> Option<Uuid> {
    metadata
        .get(TRACEPARENT_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_traceparent)
        .map(|(trace_id, _)| trace_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_traceparent_format() {
        let trace_id = Uuid::parse_str("4bf92f35-77b3-4da6-a3ce-929d0e0e4736").unwrap();
        let span_id = Uuid::parse_str("00f067aa-0ba9-02b7-0000-000000000000").unwrap();
        let tp = build_traceparent(trace_id, span_id);
        // simple() 输出 32 char (无连字符 + 全 16 字节)
        assert_eq!(
            tp,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b70000000000000000-01"
        );
    }

    #[test]
    fn parse_traceparent_ok() {
        let s = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b70000000000000000-01";
        let (trace_id, span_id) = parse_traceparent(s).unwrap();
        assert_eq!(trace_id.to_string(), "4bf92f35-77b3-4da6-a3ce-929d0e0e4736");
        assert_eq!(span_id.to_string(), "00f067aa-0ba9-02b7-0000-000000000000");
    }

    #[test]
    fn parse_traceparent_invalid() {
        assert!(parse_traceparent("not-a-traceparent").is_none());
        assert!(parse_traceparent("00-bad-uuid-01").is_none());
    }
}
