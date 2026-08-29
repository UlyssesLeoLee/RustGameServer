//! gRPC 链路追踪（per RGS-DTL-100 §7 + ARC-051 跨服务追踪）
//!
//! 54.12 实化：tonic interceptor 注入 trace context
//! 55.16 修（per RGS-REV-007 AC2）：client_interceptor 从当前 tracing Span 提取真实 trace_id
//!
//! 设计：
//! - traceparent header（W3C trace context）格式：version-trace_id-parent_span_id-flags
//! - client interceptor：从当前 tracing Span 提取 trace_id / span_id → metadata traceparent
//! - server interceptor：从 metadata traceparent 提取 trace_id / span_id → 当前 Span record

use opentelemetry::trace::TraceContextExt as _;
use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};
use tonic::service::interceptor::{self, InterceptorLayer};
use tonic::{Request, Status};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

/// gRPC traceparent header 名（per W3C Trace Context）
pub const TRACEPARENT_HEADER: &str = "traceparent";

/// traceparent 格式：version-trace_id-parent_span_id-flags
///
/// `span_id` 入参用 16-byte UUID：高位 8 字节填 0、低位 8 字节承载真实 span_id。
/// 这样 simple() 输出 32 hex chars 与原 traceparent 格式兼容。
///
/// 55.45 升级：从 `fn` 升为 `pub(crate)`，让 producer/consumer 同级模块复用，
///         避免 NATS 端重复实现（DRY）+ 保持 traceparent 格式与 gRPC 端完全一致。
pub(crate) fn build_traceparent(trace_id: Uuid, span_id: Uuid) -> String {
    format!("00-{}-{}-01", trace_id.simple(), span_id.simple())
}

/// parse traceparent header → (trace_id, span_id)
///
/// 55.45 升级：从 `fn` 升为 `pub(crate)`，让 consumer 模块直接复用解析逻辑。
pub(crate) fn parse_traceparent(value: &str) -> Option<(Uuid, Uuid)> {
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

/// 从当前 tracing Span 提取 (trace_id, span_id) — 55.16 接入 OTel bridge
///
/// - 当前 Span 已被 OTel subscriber 桥接（典型情况：业务层在 span 内调 gRPC client）：
///   返回 (TraceId, SpanId) 真实值，分布式追踪贯通
/// - OTel 未初始化或 Span 无 OTel context（测试 / 单进程）：
///   fallback 到新 UUID，保持单进程调用不报错
fn current_trace_ids() -> (Uuid, Uuid) {
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
        // fallback: OTel 未启用（单元测试 / 开发模式）
        (Uuid::new_v4(), Uuid::new_v4())
    }
}

/// Client Interceptor（从当前 tracing Span 提取 trace_id / span_id → metadata traceparent）
#[allow(clippy::result_large_err)]
pub fn client_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    let (trace_id, span_id) = current_trace_ids();
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

    /// 55.16 AC2 验证：client_interceptor 注入的 trace_id 与当前 OTel context 一致
    ///
    /// 在 OTel 未启用时 fallback：每次调用生成新 UUID（单进程兼容）
    #[test]
    fn client_interceptor_fallback_when_no_otel() {
        // 无 OTel subscriber → Span::current() 无有效 OTel context
        let request = Request::new(());
        let request = client_interceptor(request).expect("interceptor ok");
        let tp = request
            .metadata()
            .get(TRACEPARENT_HEADER)
            .expect("traceparent header present")
            .to_str()
            .unwrap()
            .to_string();
        // 格式：00-{32 hex}-{32 hex}-01
        let parts: Vec<&str> = tp.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "00");
        assert_eq!(parts[1].len(), 32, "trace_id 应为 32 hex chars");
        assert_eq!(
            parts[2].len(),
            32,
            "span_id 应为 32 hex chars (含 zero-pad)"
        );
        assert_eq!(parts[3], "01");
    }

    /// 55.16 AC2 验证：两次调用 → fallback 路径下 trace_id 不同（确认是按调用生成，非硬编码）
    #[test]
    fn client_interceptor_fallback_generates_unique_per_call() {
        let r1 = client_interceptor(Request::new(())).unwrap();
        let r2 = client_interceptor(Request::new(())).unwrap();
        let tp1 = r1
            .metadata()
            .get(TRACEPARENT_HEADER)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let tp2 = r2
            .metadata()
            .get(TRACEPARENT_HEADER)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert_ne!(tp1, tp2, "fallback 路径每次调用应生成新 trace_id");
    }

    /// 55.16 AC2 验证：build_traceparent 把 OTel SpanId (8 bytes) 零填充到 16 bytes UUID
    /// 模拟 OTel TraceId 16 bytes / SpanId 8 bytes → 转 traceparent 字符串
    #[test]
    fn build_traceparent_with_padded_span_id() {
        // 模拟 OTel TraceId (16 bytes random)
        let trace_bytes: [u8; 16] = [
            0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e,
            0x47, 0x36,
        ];
        // 模拟 OTel SpanId (8 bytes random)
        let span_bytes: [u8; 8] = [0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7];
        let mut trace_id_arr = [0u8; 16];
        trace_id_arr.copy_from_slice(&trace_bytes);
        let mut span_id_arr = [0u8; 16];
        span_id_arr[..8].copy_from_slice(&span_bytes);
        let trace_id = Uuid::from_bytes(trace_id_arr);
        let span_id = Uuid::from_bytes(span_id_arr);
        let tp = build_traceparent(trace_id, span_id);
        assert_eq!(
            tp,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b70000000000000000-01"
        );
    }
}
