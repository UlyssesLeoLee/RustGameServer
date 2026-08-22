//! Prometheus Metrics（per RGS-ARC-051 集群运营中心观测）
//!
//! 54.13 实化：MetricsRegistry + 4 个核心指标 + 业务 helper
//!
//! 设计：
//! - 全局 MetricsRegistry 单例（lazy_static 或 OnceCell）
//! - 4 个核心指标：rgs_http_requests_total / rgs_http_request_duration_seconds /
//!   rgs_saga_state_count / rgs_outbox_pending_count
//! - 业务 helper：record_http_request / record_saga_state / record_outbox_pending
//! - 导出：encode_to_text() → Prometheus text format (scrape 端读取)

use prometheus::{
    register_counter_vec_with_registry, register_gauge_vec_with_registry,
    register_histogram_vec_with_registry, CounterVec, GaugeVec, HistogramVec, Registry,
};
use std::sync::OnceLock;
use thiserror::Error;

/// Metrics 错误
#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Prometheus encoding error: {0}")]
    Encoding(String),

    #[error("Prometheus register error: {0}")]
    Register(String),
}

/// 全局 Registry（per RGS-SPEC-CROSS-008 草案）
static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get_or_init(Registry::new)
}

/// 指标容器（每个进程一个）
pub struct Metrics {
    /// HTTP 请求计数（按 service / method / status 分桶）
    pub http_requests: CounterVec,
    /// HTTP 请求延迟（秒）
    pub http_request_duration: HistogramVec,
    /// Saga 状态计数（按 service / saga_type / status）
    pub saga_state: GaugeVec,
    /// Outbox 待发送计数
    pub outbox_pending: GaugeVec,
}

impl Metrics {
    /// 注册所有指标
    pub fn new() -> Result<Self, MetricsError> {
        let reg = registry();
        let http_requests = register_counter_vec_with_registry!(
            "rgs_http_requests_total",
            "Total HTTP/gRPC requests",
            &["service", "method", "status"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let http_request_duration = register_histogram_vec_with_registry!(
            "rgs_http_request_duration_seconds",
            "HTTP/gRPC request duration in seconds",
            &["service", "method"],
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let saga_state = register_gauge_vec_with_registry!(
            "rgs_saga_state_count",
            "Saga state count by status",
            &["saga_type", "status"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_pending = register_gauge_vec_with_registry!(
            "rgs_outbox_pending_count",
            "Outbox pending entry count by domain",
            &["domain"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        Ok(Self {
            http_requests,
            http_request_duration,
            saga_state,
            outbox_pending,
        })
    }

    /// 业务 helper：记录 HTTP 请求
    pub fn record_http_request(&self, service: &str, method: &str, status: &str) {
        self.http_requests
            .with_label_values(&[service, method, status])
            .inc();
    }

    /// 业务 helper：记录 HTTP 延迟
    pub fn record_http_duration(&self, service: &str, method: &str, duration_secs: f64) {
        self.http_request_duration
            .with_label_values(&[service, method])
            .observe(duration_secs);
    }

    /// 业务 helper：记录 Saga 状态变化
    pub fn set_saga_state(&self, saga_type: &str, status: &str, count: i64) {
        self.saga_state
            .with_label_values(&[saga_type, status])
            .set(count as f64);
    }

    /// 业务 helper：记录 Outbox 待发送数
    pub fn set_outbox_pending(&self, domain: &str, count: i64) {
        self.outbox_pending
            .with_label_values(&[domain])
            .set(count as f64);
    }
}

/// 全局 Metrics 实例（per process 共享）
static METRICS: OnceLock<Metrics> = OnceLock::new();

/// 获取全局 Metrics（lazy init）
pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(|| Metrics::new().expect("metrics init"))
}

/// 编码为 Prometheus text format
pub fn encode_to_text() -> Result<String, MetricsError> {
    let encoder = prometheus::TextEncoder::new();
    encoder
        .encode_to_string(&registry().gather())
        .map_err(|e| MetricsError::Encoding(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_metrics_init_works() {
        let m = metrics();
        m.record_http_request("test", "ping", "200");
        m.record_http_duration("test", "ping", 0.001);
        m.set_saga_state("transfer", "running", 3);
        m.set_outbox_pending("economy", 5);
    }

    #[test]
    fn global_metrics_idempotent() {
        let _ = metrics();
        let _ = metrics();
    }

    #[test]
    fn encode_to_text_works() {
        let m = metrics();
        m.record_http_request("test", "ping", "200");
        m.record_http_duration("test", "ping", 0.001);
        m.set_saga_state("transfer", "running", 3);
        m.set_outbox_pending("economy", 5);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_http_requests_total"));
        assert!(text.contains("rgs_saga_state_count"));
        assert!(text.contains("rgs_outbox_pending_count"));
    }
}
