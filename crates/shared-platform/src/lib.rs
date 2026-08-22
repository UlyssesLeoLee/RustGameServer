//! shared-platform —— 公共服务库
//!
//! 域职责：
//! - tracing 初始化（OpenTelemetry OTLP 导出）
//! - config 加载（基于 envy / figment + .env 读取 per RGS-SEC-100 §7）
//! - error 类型（thiserror + 各域 error 集中转换）
//! - mTLS 工具（rustls + rcgen 证书生成 per WF-1-53.11）
//! - sqlx 公共 connection pool helper
//!
//! 规范：RGS-SPEC-CROSS-001~007 横向规范
//!       RGS-IMPL-001 §3 / RGS-IMPL-003 §3 工具链
//!
//! 不持有 DB（per ARC-008 5 独立 DB 原则）。
//!
//! 53.2 占位 → 54.9 启用 client 工具层。
//!
//! 模块清单（54.9）：
//! - channel：RpcChannel 工厂（mTLS + 超时 + retry）
//! - tls：mTLS 证书加载（rustls + pem）
//! - retry：exponential backoff + gRPC status code 分类
//! - client：6 域 ServiceId + 统一 channel 构造

pub mod channel;
pub mod client;
pub mod retry;
pub mod tls;

pub use channel::{build_channel, retry_backoff, ChannelError, RpcChannelConfig};
pub use client::{build_service_channel, ServiceId};
pub use retry::{backoff_duration, is_retryable, RetryConfig};
pub use tls::{load_client_tls, load_server_identity, ClientTlsConfigInput, TlsError};
