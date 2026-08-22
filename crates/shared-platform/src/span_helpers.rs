//! 业务 span helper（per RGS-DTL-100 §7 + ARC-051）
//!
//! 54.12 实化：常用业务 span 构造（saga step / repository operation / service call）

use tracing::{span, Level, Span};

/// Saga step span
pub fn saga_step_span(saga_type: &str, step_name: &str) -> Span {
    span!(
        Level::INFO,
        "saga.step",
        saga_type = %saga_type,
        step.name = %step_name,
    )
}

/// Saga orchestrator span
pub fn saga_orchestrator_span(saga_id: &str, saga_type: &str) -> Span {
    span!(
        Level::INFO,
        "saga.orchestrate",
        saga.id = %saga_id,
        saga.type = %saga_type,
    )
}

/// Repository 操作 span
pub fn repository_span(entity: &str, op: &str) -> Span {
    span!(
        Level::DEBUG,
        "repository.op",
        entity = %entity,
        operation = %op,
    )
}

/// Service 调用 span
pub fn service_call_span(service: &str, method: &str) -> Span {
    span!(
        Level::INFO,
        "service.call",
        service = %service,
        method = %method,
    )
}

/// Outbox relay span
pub fn outbox_relay_span(batch_size: usize) -> Span {
    span!(Level::INFO, "outbox.relay", batch.size = batch_size,)
}

/// gRPC handler span
pub fn grpc_handler_span(method: &str) -> Span {
    span!(
        Level::INFO,
        "grpc.handler",
        rpc.method = %method,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saga_step_span_creates() {
        let _ = saga_step_span("transfer", "reserve");
    }

    #[test]
    fn repository_span_creates() {
        let _ = repository_span("Player", "find_by_id");
    }
}
