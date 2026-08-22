//! Tracing 初始化 + OTel exporter（per RGS-DTL-100 §7 + ARC-051 COC 观测）
//!
//! 54.12 实化：init_tracing 统一 tracing + OTel bridge + 控制台 fallback
//!
//! 设计：
//! - tracing-subscriber EnvFilter 控制日志级别
//! - tracing-opentelemetry bridge：tracing → OTel span
//! - opentelemetry-otlp + OtlpPipeline → OTel Collector
//! - service.name / service.version / deployment.environment resource attrs
//!
//! **互斥约束**：init_tracing 与 init_json_logging（json_logging 模块）互斥 —
//! tracing_subscriber 全局只能一个 subscriber。生产用 init_tracing（OTel），
//! 开发/测试用 init_json_logging（JSON）。二选一，不可同时调用。

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{self as sdktrace};
use opentelemetry_sdk::Resource;
use thiserror::Error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Tracing init 错误
#[derive(Debug, Error)]
pub enum TracingError {
    #[error("OTel pipeline error: {0}")]
    OTelPipeline(String),

    #[error("subscriber init error: {0}")]
    SubscriberInit(String),
}

/// OTel 配置
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// OTel Collector endpoint（如 http://otel-collector:4317）
    pub endpoint: String,
    /// 采样率 0.0-1.0（默认 1.0 = 全采样）
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

/// 初始化 tracing + OTel
pub fn init_tracing(
    service_name: &str,
    service_version: &str,
    deployment_env: &str,
) -> Result<(), TracingError> {
    init_tracing_with_otel(
        service_name,
        service_version,
        deployment_env,
        &OtelConfig::default(),
    )
}

/// 初始化 tracing + OTel（自定义 OTel 配置）
pub fn init_tracing_with_otel(
    service_name: &str,
    service_version: &str,
    deployment_env: &str,
    otel_cfg: &OtelConfig,
) -> Result<(), TracingError> {
    // 1. 构造 resource
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", service_version.to_string()),
        KeyValue::new("deployment.environment", deployment_env.to_string()),
    ]);

    // 2. 构造 OTel TracerProvider（OtlpPipeline API per opentelemetry-otlp 0.17）
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&otel_cfg.endpoint),
        )
        .with_trace_config(
            sdktrace::Config::default()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::TraceIdRatioBased(otel_cfg.sample_ratio)),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| TracingError::OTelPipeline(e.to_string()))?;

    let tracer = tracer_provider.tracer(service_name.to_string());

    // 3. tracing-opentelemetry layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // 4. EnvFilter + fmt + otel layer
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,hyper=warn,h2=warn"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_layer)
        .try_init()
        .map_err(|e| TracingError::SubscriberInit(e.to_string()))?;

    tracing::info!(
        target: "tracing_init",
        service = service_name,
        version = service_version,
        env = deployment_env,
        otel_endpoint = %otel_cfg.endpoint,
        "tracing initialized"
    );

    Ok(())
}

/// 关闭 OTel provider（graceful shutdown，flush 残余 span）
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otel_config_default() {
        let cfg = OtelConfig::default();
        assert_eq!(cfg.endpoint, "http://localhost:4317");
        assert_eq!(cfg.sample_ratio, 1.0);
    }
}
