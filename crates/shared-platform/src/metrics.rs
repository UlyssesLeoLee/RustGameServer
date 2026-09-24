//! Prometheus Metrics（per RGS-ARC-051 集群运营中心观测 + ULYS-100 P2-#2 outbox）
//!
//! 54.13 实化：MetricsRegistry + 4 个核心指标 + 业务 helper
//! ULYS-100 扩展：8 个 outbox 专项指标 + 业务 helper
//!   (per docs/00-基准与治理/ULYS-56-follow-up-drafts/06_Outbox监控指标草案.md §1.1+§1.2)
//!
//! 设计：
//! - 全局 MetricsRegistry 单例（OnceCell）
//! - HTTP / Saga 核心指标：rgs_http_requests_total / rgs_http_request_duration_seconds / rgs_saga_state_count
//! - Outbox 8 指标：
//!   * rgs_outbox_pending_count{service, aggregate_type}         Gauge   条
//!   * rgs_outbox_inflight_count{service, aggregate_type}        Gauge   条
//!   * rgs_outbox_inflight_lease_lag_seconds{service}            Gauge   秒（最大 lease 剩余时间）
//!   * rgs_outbox_failed_count{service, aggregate_type}          Gauge   条
//!   * rgs_outbox_relay_poll_cycle_duration_seconds{service}     Histogram 秒
//!   * rgs_outbox_relay_publish_total{service, aggregate_type, result}  Counter
//!   * rgs_outbox_event_age_seconds{service, aggregate_type}     Histogram 秒（created_at→sent_at）
//!   * rgs_outbox_batch_size{service}                            Histogram 条
//!   * rgs_outbox_oldest_pending_created_at_seconds{service}    Gauge   unix_ts（供 recording rule 派生 age）
//! - 业务 helper：record_http_request / record_saga_state / record_outbox_publish /
//!                observe_outbox_event_age / observe_outbox_poll_cycle / observe_outbox_batch_size /
//!                set_outbox_gauge (Pending/Inflight/Failed) / set_outbox_lease_lag / set_outbox_oldest_pending_created_at
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

/// Histogram 默认桶（per 06_Outbox监控指标草案.md §2 阈值分布）
///
/// - HTTP: 1ms..10s（既有 54.13 默认）
/// - Outbox poll cycle / event age / batch size: 同区间复用
const DEFAULT_BUCKETS: &[f64] = &[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];

/// Batch size 桶（per 草案 §1.2 批量范围）
const BATCH_SIZE_BUCKETS: &[f64] = &[1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];

/// Event age 桶（草案 §1.2 端到端延迟，秒）
const EVENT_AGE_BUCKETS: &[f64] = &[
    0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 300.0,
];

/// 指标容器（每个进程一个）
pub struct Metrics {
    /// HTTP 请求计数（按 service / method / status 分桶）
    pub http_requests: CounterVec,
    /// HTTP 请求延迟（秒）
    pub http_request_duration: HistogramVec,
    /// Saga 状态计数（按 service / saga_type / status）
    pub saga_state: GaugeVec,

    // ============ ULYS-100 outbox 指标 ============
    /// 当前 `status = 'Pending'` 的 outbox 行数（service × aggregate_type）
    pub outbox_pending: GaugeVec,
    /// 当前 `status = 'InFlight'` 且 lease 仍有效的 outbox 行数
    pub outbox_inflight: GaugeVec,
    /// InFlight 行 lease 剩余时间最大值（秒；接近 30s = 接近过期）
    pub outbox_inflight_lease_lag: GaugeVec,
    /// 当前 `status = 'Failed'` 的 outbox 行数
    pub outbox_failed: GaugeVec,
    /// 单次轮询周期耗时（list_pending + publish + mark_sent/_failed 全过程）
    pub outbox_relay_poll_cycle_duration: HistogramVec,
    /// Relay 发布次数（result = success / failure / timeout）
    pub outbox_relay_publish: CounterVec,
    /// outbox 端到端延迟（created_at → sent_at），仅成功 publish 后 observe
    pub outbox_event_age: HistogramVec,
    /// 单次轮询批量大小（list_pending LIMIT N 返回的条数）
    pub outbox_batch_size: HistogramVec,
    /// 最旧 Pending 行的 created_at unix 秒（供 recording rule 派生 `time() - oldest_pending_age_seconds`）
    pub outbox_oldest_pending_created_at: GaugeVec,
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
            DEFAULT_BUCKETS.to_vec(),
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

        // ----- ULYS-100 P2-#2 outbox 8 + 1 指标 -----
        // 注：pending / inflight / failed 沿用 06_草案 §1.1 标签集 {service, aggregate_type}
        //     lease_lag / oldest_pending 只按 service 聚合（per §1.1 单 label 形式）
        //     relay_publish_total 含 result 维度
        let outbox_pending = register_gauge_vec_with_registry!(
            "rgs_outbox_pending_count",
            "Outbox pending entry count (status=pending) by service and aggregate_type",
            &["service", "aggregate_type"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_inflight = register_gauge_vec_with_registry!(
            "rgs_outbox_inflight_count",
            "Outbox in-flight entry count (status=in_flight AND lease_until>NOW()) by service and aggregate_type",
            &["service", "aggregate_type"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_inflight_lease_lag = register_gauge_vec_with_registry!(
            "rgs_outbox_inflight_lease_lag_seconds",
            "Max remaining lease seconds across in-flight outbox rows by service (approaching 30s = nearing reclaim)",
            &["service"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_failed = register_gauge_vec_with_registry!(
            "rgs_outbox_failed_count",
            "Outbox failed entry count (status=failed, retry_count >= max) by service and aggregate_type",
            &["service", "aggregate_type"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_relay_poll_cycle_duration = register_histogram_vec_with_registry!(
            "rgs_outbox_relay_poll_cycle_duration_seconds",
            "Outbox relay single poll cycle duration (list_pending + publish batch + mark_sent/_failed)",
            &["service"],
            DEFAULT_BUCKETS.to_vec(),
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_relay_publish = register_counter_vec_with_registry!(
            "rgs_outbox_relay_publish_total",
            "Outbox relay publish attempts (result=success|failure|timeout)",
            &["service", "aggregate_type", "result"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_event_age = register_histogram_vec_with_registry!(
            "rgs_outbox_event_age_seconds",
            "Outbox end-to-end latency from created_at to sent_at (per published entry)",
            &["service", "aggregate_type"],
            EVENT_AGE_BUCKETS.to_vec(),
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_batch_size = register_histogram_vec_with_registry!(
            "rgs_outbox_batch_size",
            "Outbox relay single poll batch size (number of entries fetched per tick)",
            &["service"],
            BATCH_SIZE_BUCKETS.to_vec(),
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let outbox_oldest_pending_created_at = register_gauge_vec_with_registry!(
            "rgs_outbox_oldest_pending_created_at_seconds",
            "Unix timestamp of oldest pending outbox entry (per service); use recording rule time()-gauge to derive age_seconds",
            &["service"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        Ok(Self {
            http_requests,
            http_request_duration,
            saga_state,
            outbox_pending,
            outbox_inflight,
            outbox_inflight_lease_lag,
            outbox_failed,
            outbox_relay_poll_cycle_duration,
            outbox_relay_publish,
            outbox_event_age,
            outbox_batch_size,
            outbox_oldest_pending_created_at,
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

    // ============ ULYS-100 outbox 业务 helper ============

    /// 设置当前 Pending 计数（per service × aggregate_type）
    pub fn set_outbox_pending(&self, service: &str, aggregate_type: &str, count: i64) {
        self.outbox_pending
            .with_label_values(&[service, aggregate_type])
            .set(count as f64);
    }

    /// 设置当前 InFlight 计数
    pub fn set_outbox_inflight(&self, service: &str, aggregate_type: &str, count: i64) {
        self.outbox_inflight
            .with_label_values(&[service, aggregate_type])
            .set(count as f64);
    }

    /// 设置 InFlight 行 lease 剩余秒数最大值（per service 单 label 聚合）
    pub fn set_outbox_inflight_lease_lag(&self, service: &str, max_lag_secs: f64) {
        self.outbox_inflight_lease_lag
            .with_label_values(&[service])
            .set(max_lag_secs);
    }

    /// 设置当前 Failed 计数
    pub fn set_outbox_failed(&self, service: &str, aggregate_type: &str, count: i64) {
        self.outbox_failed
            .with_label_values(&[service, aggregate_type])
            .set(count as f64);
    }

    /// 观察 relay 单次轮询周期耗时（秒）
    pub fn observe_outbox_poll_cycle_duration(&self, service: &str, secs: f64) {
        self.outbox_relay_poll_cycle_duration
            .with_label_values(&[service])
            .observe(secs);
    }

    /// 记录 relay 发布尝试计数
    ///
    /// `result` 必须是 `"success"` / `"failure"` / `"timeout"` 之一
    /// （per 06_草案 §1.1 + 派生 recording rule `rgs_outbox_publish_failure_rate_5m`）
    pub fn record_outbox_publish(&self, service: &str, aggregate_type: &str, result: &str) {
        self.outbox_relay_publish
            .with_label_values(&[service, aggregate_type, result])
            .inc();
    }

    /// 观察 outbox 端到端延迟（成功 publish 后由 relay 调用；created_at→sent_at）
    pub fn observe_outbox_event_age(&self, service: &str, aggregate_type: &str, secs: f64) {
        self.outbox_event_age
            .with_label_values(&[service, aggregate_type])
            .observe(secs);
    }

    /// 观察单次轮询批量大小
    pub fn observe_outbox_batch_size(&self, service: &str, count: usize) {
        self.outbox_batch_size
            .with_label_values(&[service])
            .observe(count as f64);
    }

    /// 设置最旧 Pending 行 created_at unix 秒（per service；供 recording rule 派生 age）
    pub fn set_outbox_oldest_pending_created_at(&self, service: &str, unix_secs: f64) {
        self.outbox_oldest_pending_created_at
            .with_label_values(&[service])
            .set(unix_secs);
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
        m.set_outbox_pending("economy", "economy.transfer", 5);
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
        m.set_outbox_pending("economy", "economy.transfer", 5);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_http_requests_total"));
        assert!(text.contains("rgs_saga_state_count"));
        assert!(text.contains("rgs_outbox_pending_count"));
    }

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // Prometheus 指标是 ARC-051 集群运营中心入口, 加 4 单测

    #[test]
    fn record_http_request_increments_counter() {
        // 同一 label 多次 inc, Prometheus 输出应包含样例
        let m = metrics();
        for _ in 0..3 {
            m.record_http_request("unit-test-svc", "GET", "200");
        }
        let text = encode_to_text().unwrap();
        assert!(text.contains("unit-test-svc"));
        assert!(text.contains("rgs_http_requests_total"));
    }

    #[test]
    fn set_saga_state_visible_in_text_format() {
        let m = metrics();
        m.set_saga_state("test-saga", "completed", 42);
        let text = encode_to_text().unwrap();
        assert!(text.contains("test-saga"));
        assert!(text.contains("rgs_saga_state_count"));
    }

    #[test]
    fn set_outbox_pending_negative_value_works() {
        // Gauge 支持负数 (虽然业务上不会, 但 Prometheus 允许)
        let m = metrics();
        m.set_outbox_pending("test-domain", "test-domain.unknown", -1);
        let text = encode_to_text().unwrap();
        assert!(text.contains("test-domain"));
    }

    #[test]
    fn record_http_duration_records_histogram() {
        let m = metrics();
        m.record_http_duration("test-dur-svc", "Slow", 0.123);
        m.record_http_duration("test-dur-svc", "Fast", 0.001);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_http_request_duration_seconds"));
        assert!(text.contains("test-dur-svc"));
    }

    // ============ ULYS-100 P2-#2 outbox metrics tests ============

    #[test]
    fn outbox_pending_gauge_emits_service_and_aggregate_type_labels() {
        let m = metrics();
        m.set_outbox_pending("economy", "economy.transfer", 42);
        m.set_outbox_pending("economy", "economy.credit", 7);
        m.set_outbox_pending("player", "player.registered", 100);
        let text = encode_to_text().unwrap();
        // service × aggregate_type 标签都在
        assert!(text.contains("rgs_outbox_pending_count"));
        assert!(text.contains("service=\"economy\""));
        assert!(text.contains("aggregate_type=\"economy.transfer\""));
        assert!(text.contains("service=\"player\""));
    }

    #[test]
    fn outbox_inflight_count_and_lease_lag_gauge_work() {
        let m = metrics();
        m.set_outbox_inflight("admin", "admin.lcm", 15);
        m.set_outbox_inflight_lease_lag("admin", 25.5);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_inflight_count"));
        assert!(text.contains("service=\"admin\""));
        assert!(text.contains("aggregate_type=\"admin.lcm\""));
        assert!(text.contains("rgs_outbox_inflight_lease_lag_seconds"));
    }

    #[test]
    fn outbox_failed_gauge_emits_correctly() {
        let m = metrics();
        m.set_outbox_failed("match", "match.matchmake", 3);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_failed_count"));
        assert!(text.contains("service=\"match\""));
        assert!(text.contains("aggregate_type=\"match.matchmake\""));
    }

    #[test]
    fn outbox_relay_publish_total_records_all_three_results() {
        let m = metrics();
        m.record_outbox_publish("economy", "economy.transfer", "success");
        m.record_outbox_publish("economy", "economy.transfer", "failure");
        m.record_outbox_publish("economy", "economy.transfer", "timeout");
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_relay_publish_total"));
        assert!(text.contains("result=\"success\""));
        assert!(text.contains("result=\"failure\""));
        assert!(text.contains("result=\"timeout\""));
    }

    #[test]
    fn outbox_event_age_histogram_observes() {
        let m = metrics();
        m.observe_outbox_event_age("social", "social.friend", 0.5);
        m.observe_outbox_event_age("social", "social.friend", 2.5);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_event_age_seconds"));
        assert!(text.contains("rgs_outbox_event_age_seconds_bucket"));
    }

    #[test]
    fn outbox_batch_size_histogram_observes() {
        let m = metrics();
        m.observe_outbox_batch_size("player", 0);
        m.observe_outbox_batch_size("player", 50);
        m.observe_outbox_batch_size("player", 200);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_batch_size"));
        assert!(text.contains("rgs_outbox_batch_size_bucket"));
    }

    #[test]
    fn outbox_poll_cycle_duration_histogram_observes() {
        let m = metrics();
        m.observe_outbox_poll_cycle_duration("cluster_ops", 0.05);
        m.observe_outbox_poll_cycle_duration("cluster_ops", 1.5);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_relay_poll_cycle_duration_seconds"));
        assert!(text.contains("rgs_outbox_relay_poll_cycle_duration_seconds_bucket"));
    }

    #[test]
    fn outbox_oldest_pending_created_at_gauge_works() {
        let m = metrics();
        m.set_outbox_oldest_pending_created_at("admin", 1_700_000_000.0);
        let text = encode_to_text().unwrap();
        assert!(text.contains("rgs_outbox_oldest_pending_created_at_seconds"));
        assert!(text.contains("1700000000"));
    }
}
