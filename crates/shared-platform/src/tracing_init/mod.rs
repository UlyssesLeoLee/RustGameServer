//! Tracing 初始化公共类型 + 重导出（per P2-9 拆分，2026-09-05）
//!
//! 拆为 3 模块：
//! - [`super::stdout`] (stdout subscriber + EnvFilter + fmt layer，公共 stdout 路径)
//! - [`super::otel`] (OTel OTLP exporter + guard + 条件初始化)
//! - 本文件: 公共类型 `TracingError` + `OtelConfig` + 重导出
//!
//! **设计原则**：impl stdout 路径 + OTel 路径可独立使用，但全局 tracing subscriber
//! 互斥（per DTL-100 §7）。生产用 OTel 路径，dev/test 用 stdout 路径。
//!
//! **拆分原因**：原 tracing_init.rs 231 行混 3 职责（types / stdout / otel），
//! 53.12 OTel SDK 启用时 otel.rs 会增长。拆开后各模块独立演进。

use thiserror::Error;

pub mod otel;
pub mod stdout;

// 重导出公共 API (保持原 lib.rs `pub use tracing_init::{...}` 兼容)
pub use otel::{
    init_otel_exporter_optional, init_tracing, init_tracing_with_otel, shutdown_tracing,
    OtelExporterGuard,
};
pub use stdout::init_stdout_tracing;

/// Tracing init 错误
#[derive(Debug, Error)]
pub enum TracingError {
    #[error("OTel pipeline error: {0}")]
    OTelPipeline(String),

    #[error("subscriber init error: {0}")]
    SubscriberInit(String),
}

/// OTel 公共配置
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// OTel Collector endpoint（如 `http://otel-collector:4317`）
    pub endpoint: String,
    /// 采样率 0.0-1.0，默认 1.0 = 全量采样
    pub sample_ratio: f64,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            sample_ratio: 1.0,
        }
    }
}
