//! shared-platform —— 公共服务库
//!
//! 域职责：
//! - tracing 初始化（OpenTelemetry OTLP 导出，per WF-1-54.12）
//! - JSON 结构化日志（per WF-1-54.14）
//! - gRPC 链路追踪（traceparent 注入，per WF-1-54.12）
//! - 业务 span helper（saga / repository / service，per WF-1-54.12）
//! - Prometheus metrics 导出 + scrape endpoint（per WF-1-54.13）
//! - RBAC 角色基础访问控制（per WF-1-54.15）
//! - config 加载（基于 envy / figment + .env 读取 per RGS-SEC-100 §7）
//! - error 类型（thiserror + 各域 error 集中转换）
//! - mTLS 工具（rustls + rcgen 证书生成 per WF-1-53.11）
//! - sqlx 公共 connection pool helper
//! - gRPC client 工厂（mTLS + retry + timeout，per WF-1-54.9）
//! - NATS JetStream 消息总线（per WF-1-54.10）
//! - Outbox pattern 事务性消息（per WF-1-54.11）
//!
//! 规范：RGS-SPEC-CROSS-001~008 横向规范
//!       RGS-IMPL-001 §3 / RGS-IMPL-003 §3 工具链
//!
//! 不持有 DB（per ARC-008 5 独立 DB 原则）。
//!
//! 53.2 占位 → 54.9 client → 54.10 messaging → 54.11 outbox → 54.12 observability → 54.13 metrics → 54.14 json logging → 54.15 RBAC.

pub mod cert_rotation;
pub mod channel;
pub mod client;
pub mod consumer;
pub mod dlq;
pub mod grpc_tracing;
pub mod json_logging;
pub mod messaging;
pub mod metrics;
pub mod metrics_endpoint;
pub mod outbox;
pub mod outbox_relay;
pub mod producer;
pub mod rbac;
pub mod retry;
pub mod span_helpers;
pub mod subject;
pub mod tls;
pub mod tracing_init;

pub use channel::{
    build_channel, mtls_bypassed_total, retry_backoff, ChannelError, RpcChannelConfig,
};
pub use client::{
    build_insecure_channel, build_secure_channel, build_secure_channel_with_tls, ServiceId,
};
pub use consumer::{
    deserialize_envelope, nak_with_delay, process_with_retry, ConsumerConfig, ConsumerError,
    ConsumerHandler, DeserializedMessage,
};
pub use dlq::DlqEntry;
pub use grpc_tracing::{
    client_interceptor, client_interceptor_layer, extract_trace_id, server_interceptor,
    server_interceptor_layer, TRACEPARENT_HEADER,
};
pub use json_logging::{
    init_json_logging, with_actor, with_request_id, with_saga_id, JsonLogError,
};
pub use messaging::{build_messaging_client, MessagingConfig, MessagingError};
pub use metrics::{encode_to_text, metrics, Metrics, MetricsError};
pub use metrics_endpoint::{scrape_metrics, MetricsResponse};
pub use outbox::{
    InMemoryOutboxRepository, OutboxEntry, OutboxError, OutboxRepository, OutboxStatus,
    PgOutboxRepository, MIGRATION_TEMPLATE,
};
pub use outbox_relay::{OutboxRelay, RelayConfig, RelayStats};
pub use producer::{MessageEnvelope, Producer, ProducerConfig, ProducerError};
pub use rbac::{
    enforce, Authorizer, CheckResult, RbacError, Role, SimpleAuthorizer, Subject, SubjectType,
};
pub use retry::{backoff_duration, is_retryable, RetryConfig};
pub use span_helpers::{
    grpc_handler_span, outbox_relay_span, repository_span, saga_orchestrator_span, saga_step_span,
    service_call_span,
};
pub use subject::{parse, SubjectBuilder, SubjectDomain, SubjectError};
pub use tls::{
    install_default_crypto_provider, load_client_tls, load_server_identity, load_server_tls_config,
    ClientTlsConfigInput, TlsError,
};
pub use tracing_init::{
    init_otel_exporter_optional, init_tracing, init_tracing_with_otel, shutdown_tracing,
    OtelConfig, OtelExporterGuard, TracingError,
};
