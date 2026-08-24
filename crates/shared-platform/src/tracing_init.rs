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

/// 55.45 OTLP exporter 条件初始化（env-gated, 53.12 任务未完成时 = no-op）
///
/// 设计：
/// - 环境变量 `OTEL_SDK_DISABLED=true` → 跳过 OTel 初始化（默认行为，53.12 任务未完成时）
/// - 环境变量 `OTEL_SDK_DISABLED=false` / 未设置 → 实际初始化 OTLP exporter
/// - 端点：`OTEL_EXPORTER_OTLP_ENDPOINT` env，默认 `http://otel-collector:4317`
/// - 采样率：`OTEL_TRACES_SAMPLER_ARG` env，默认 0.10（10%，per Q-M-03 答复）
/// - Resource attributes：service.name / service.version / deployment.environment
/// - Batch span processor（install_batch → opentelemetry_sdk::runtime::Tokio）
/// - 返回 Drop guard 用于 graceful shutdown（drop 时 flush 残余 span）
///
/// 容错：
/// - 初始化失败 → 记 warn，返回空 guard（不 panic，OTel 不可用不影响业务）
/// - 端点不可达 → 仍可调用（OTel SDK 内部 retry + buffer）
pub struct OtelExporterGuard {
    /// 是否实际初始化（false = no-op guard）
    enabled: bool,
}

impl Drop for OtelExporterGuard {
    fn drop(&mut self) {
        if self.enabled {
            // 53.12 任务完成 + feature flag 启用时，shutdown 真正 flush 残余 span
            opentelemetry::global::shutdown_tracer_provider();
        }
    }
}

/// 55.45 条件初始化 OTLP exporter
///
/// 用法（per 5 域 main.rs）：
/// ```ignore
/// let _otel_guard = shared_platform::tracing_init::init_otel_exporter_optional(
///     "player-service",
///     "0.1.0",
///     "dev",
/// );
/// ```
pub fn init_otel_exporter_optional(
    service_name: &str,
    service_version: &str,
    deployment_env: &str,
) -> OtelExporterGuard {
    // 53.12 任务未完成时默认 disabled
    if std::env::var("OTEL_SDK_DISABLED")
        .is_ok_and(|v| v == "true" || v == "1" || v.eq_ignore_ascii_case("yes"))
    {
        tracing::info!(
            target: "otel",
            service = service_name,
            "OTEL_SDK_DISABLED=true — OTel exporter NOT initialized (53.12 任务未完成)"
        );
        return OtelExporterGuard { enabled: false };
    }

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://otel-collector:4317".to_string());
    let sample_ratio: f64 = std::env::var("OTEL_TRACES_SAMPLER_ARG")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| (0.0..=1.0).contains(v))
        .unwrap_or(0.10);

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", service_version.to_string()),
        KeyValue::new("deployment.environment", deployment_env.to_string()),
    ]);

    // 53.12 任务未完成：OTel SDK 链路未启用（tracing-opentelemetry bridge 缺失）。
    // 本函数仅"占位 + env 解析 + 校验 + log 告警"，等 53.12 完成后
    // 改回 opentelemetry_otlp::new_pipeline().install_batch 即可生效。
    //
    // 55.45 临时：直接返回 noop guard（避免 OTLP exporter 启动但 bridge 未挂导致 span 静默丢失）
    tracing::info!(
        target: "otel",
        service = service_name,
        endpoint = %endpoint,
        sample_ratio = sample_ratio,
        "OTel exporter env detected (endpoint={}, sample={}) — awaiting 53.12 OTel SDK 启用",
        endpoint,
        sample_ratio
    );

    // 53.12 完成后：把以下行注释去掉即可真正启用
    //
    // match opentelemetry_otlp::new_pipeline()
    //     .tracing()
    //     .with_exporter(
    //         opentelemetry_otlp::new_exporter()
    //             .tonic()
    //             .with_endpoint(&endpoint),
    //     )
    //     .with_trace_config(
    //         sdktrace::Config::default()
    //             .with_resource(resource)
    //             .with_sampler(sdktrace::Sampler::TraceIdRatioBased(sample_ratio)),
    //     )
    //     .install_batch(opentelemetry_sdk::runtime::Tokio)
    // {
    //     Ok(_) => OtelExporterGuard { enabled: true },
    //     Err(e) => {
    //         tracing::warn!(...);
    //         OtelExporterGuard { enabled: false }
    //     }
    // }

    let _ = (resource, endpoint, sample_ratio); // suppress unused warnings
    OtelExporterGuard { enabled: false }
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
