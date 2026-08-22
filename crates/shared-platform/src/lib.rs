//! shared-platform —— 公共服务库
//!
//! 域职责：
//! - tracing 初始化（OpenTelemetry OTLP 导出）
//! - config 加载（基于 envy / figment + .env 读取 per RGS-SEC-100 §7）
//! - error 类型（thiserror + 各域 error 集中转换）
//! - mTLS 工具（rustls + rcgen 证书生成 per WF-1-53.11）
//! - sqlx 公共 connection pool helper
//! - gRPC client 工厂（mTLS + retry + timeout，per WF-1-54.9）
//! - NATS JetStream 消息总线（per WF-1-54.10）
//!
//! 规范：RGS-SPEC-CROSS-001~007 横向规范
//!       RGS-IMPL-001 §3 / RGS-IMPL-003 §3 工具链
//!
//! 不持有 DB（per ARC-008 5 独立 DB 原则）。
//!
//! 53.2 占位 → 54.9 启用 client → 54.10 启用 messaging。

pub mod channel;
pub mod client;
pub mod consumer;
pub mod dlq;
pub mod messaging;
pub mod producer;
pub mod retry;
pub mod subject;
pub mod tls;

pub use channel::{build_channel, retry_backoff, ChannelError, RpcChannelConfig};
pub use client::{build_service_channel, ServiceId};
pub use consumer::{
    deserialize_envelope, nak_with_delay, process_with_retry, ConsumerConfig, ConsumerError,
    ConsumerHandler, DeserializedMessage,
};
pub use dlq::DlqEntry;
pub use messaging::{build_messaging_client, MessagingConfig, MessagingError};
pub use producer::{MessageEnvelope, Producer, ProducerConfig, ProducerError};
pub use retry::{backoff_duration, is_retryable, RetryConfig};
pub use subject::{parse, SubjectBuilder, SubjectDomain, SubjectError};
pub use tls::{load_client_tls, load_server_identity, ClientTlsConfigInput, TlsError};
