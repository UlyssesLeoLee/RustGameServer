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

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // span helper 是 RGS-DTL-100 §7 OTel 桥接入口, 4 个 helper 各加 1 单测

    #[test]
    fn saga_orchestrator_span_creates() {
        let _ = saga_orchestrator_span("saga-id-123", "transfer");
    }

    #[test]
    fn service_call_span_creates() {
        let _ = service_call_span("player-service", "RegisterPlayer");
    }

    #[test]
    fn outbox_relay_span_creates() {
        let _ = outbox_relay_span(100);
        let _ = outbox_relay_span(0); // 边界: batch_size=0
    }

    #[test]
    fn grpc_handler_span_creates() {
        let _ = grpc_handler_span("/player.v1.PlayerService/Register");
    }

    #[test]
    fn all_spans_can_be_entered_and_exited() {
        // span 必须能 in_scope 执行闭包不 panic
        saga_step_span("transfer", "reserve").in_scope(|| {
            repository_span("Player", "find").in_scope(|| {
                service_call_span("economy", "credit").in_scope(|| {
                    // 嵌套 3 层不 panic 即通过
                });
            });
        });
    }
}
