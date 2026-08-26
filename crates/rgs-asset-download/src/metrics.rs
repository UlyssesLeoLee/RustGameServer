//! 10 项 `rgs_asset_download_*` 指标（M-2065.8 + SPEC §4）。
//!
//! ## 10 项指标（per DTL §10 + IMPL-PLAN §3.3 + SPEC §4）
//!
//! 1. `rgs_asset_download_state_transition_total{from,to}`  Counter
//! 2. `rgs_asset_download_active_count{status}`           Gauge
//! 3. `rgs_asset_download_bytes_received_total`           Counter
//! 4. `rgs_asset_download_resume_success_total`            Counter
//! 5. `rgs_asset_download_resume_failure_total{reason}`    Counter
//! 6. `rgs_asset_download_chunk_retry_total{reason}`       Counter
//! 7. `rgs_asset_download_duration_seconds`               Histogram
//! 8. `rgs_asset_download_throughput_bytes_per_second`     Gauge
//! 9. `rgs_asset_download_integrity_failure_total{reason}` Counter
//! 10. `rgs_asset_download_resume_token_store_bytes`       Gauge
//!
//! ## 标签硬约束（SPEC §4 / FR-CDN-064）
//!
//! - 仅 `from` / `to` / `file_path` / `status` / `reason` 等**低基数**标签
//! - **不**使用 `player_id` / `device_id` / `ip` / `mac` 作为 metric label
//! - `file_path` 限制为 basename 或 hash 摘要（防止路径展开到 PII）
//!
//! ## Registry 隔离
//!
//! 本 crate 用**独立** prometheus::Registry（不与 `shared-platform` 共享），避免
//! `rgs-asset-download` 测试被 sqlx/OTel 拖入编译路径。

use prometheus::{
    register_counter_vec_with_registry, register_gauge_vec_with_registry,
    register_histogram_vec_with_registry, CounterVec, GaugeVec, HistogramVec, Registry, TextEncoder,
};
use std::sync::OnceLock;

/// 整文件校验结果（metrics 标签值；稳定字符串）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityOutcome {
    /// 期望 == 实际
    Match,
    /// 不匹配
    Mismatch,
    /// 跳过（理论不存在；保留以备 NFR-CDN-002 监控告警）
    Skipped,
}

impl IntegrityOutcome {
    /// 标签值（用于 prometheus）。
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::Skipped => "skipped",
        }
    }
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get_or_init(Registry::new)
}

/// 10 项指标容器。
pub struct AssetDownloadMetrics {
    /// 1. 状态机转移次数
    pub state_transition_total: CounterVec,
    /// 2. 当前活跃下载数（按 status 分桶）
    pub active_count: GaugeVec,
    /// 3. 累计下载字节数
    pub bytes_received_total: CounterVec,
    /// 4. 断点续传成功次数
    pub resume_success_total: CounterVec,
    /// 5. 断点续传失败次数（按 reason）
    pub resume_failure_total: CounterVec,
    /// 6. 分片重试次数（按 reason）
    pub chunk_retry_total: CounterVec,
    /// 7. 单次下载耗时（秒）
    pub duration_seconds: HistogramVec,
    /// 8. 吞吐（bytes / sec，按 file_path 限流）
    pub throughput_bytes_per_second: GaugeVec,
    /// 9. 整文件校验失败次数（按 reason）
    pub integrity_failure_total: CounterVec,
    /// 10. 断点记录 store 大小（字节）
    pub resume_token_store_bytes: GaugeVec,
}

impl AssetDownloadMetrics {
    /// 注册所有 10 项指标。
    pub fn new() -> Result<Self, MetricsError> {
        let reg = registry();
        let state_transition_total = register_counter_vec_with_registry!(
            "rgs_asset_download_state_transition_total",
            "State machine transition count (per from/to)",
            &["from", "to"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let active_count = register_gauge_vec_with_registry!(
            "rgs_asset_download_active_count",
            "Currently active downloads by status",
            &["status"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let bytes_received_total = register_counter_vec_with_registry!(
            "rgs_asset_download_bytes_received_total",
            "Total bytes received from backend",
            &["file_path"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let resume_success_total = register_counter_vec_with_registry!(
            "rgs_asset_download_resume_success_total",
            "Resume success count",
            &["file_path"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let resume_failure_total = register_counter_vec_with_registry!(
            "rgs_asset_download_resume_failure_total",
            "Resume failure count (by reason)",
            &["reason"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let chunk_retry_total = register_counter_vec_with_registry!(
            "rgs_asset_download_chunk_retry_total",
            "Chunk retry count (by reason)",
            &["reason"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let duration_seconds = register_histogram_vec_with_registry!(
            "rgs_asset_download_duration_seconds",
            "Per-download duration in seconds",
            &["outcome"],
            vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let throughput_bytes_per_second = register_gauge_vec_with_registry!(
            "rgs_asset_download_throughput_bytes_per_second",
            "Per-file throughput (bytes/sec)",
            &["file_path"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let integrity_failure_total = register_counter_vec_with_registry!(
            "rgs_asset_download_integrity_failure_total",
            "Integrity gate failure count (by reason)",
            &["reason"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let resume_token_store_bytes = register_gauge_vec_with_registry!(
            "rgs_asset_download_resume_token_store_bytes",
            "Resume token store size in bytes",
            &["backend"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        Ok(Self {
            state_transition_total,
            active_count,
            bytes_received_total,
            resume_success_total,
            resume_failure_total,
            chunk_retry_total,
            duration_seconds,
            throughput_bytes_per_second,
            integrity_failure_total,
            resume_token_store_bytes,
        })
    }

    // ===== 业务 helper =====

    /// 记录一次状态机转移
    pub fn record_state_transition(&self, from: &str, to: &str) {
        self.state_transition_total
            .with_label_values(&[from, to])
            .inc();
    }

    /// 设置活跃下载数
    pub fn set_active_count(&self, status: &str, count: i64) {
        self.active_count.with_label_values(&[status]).set(count as f64);
    }

    /// 累加已收字节
    pub fn add_bytes_received(&self, file_path: &str, bytes: u64) {
        self.bytes_received_total
            .with_label_values(&[file_path])
            .inc_by(bytes as f64);
    }

    /// 记录断点恢复成功 / 失败
    pub fn record_resume_success(&self, file_path: &str) {
        self.resume_success_total
            .with_label_values(&[file_path])
            .inc();
    }

    /// 记录断点恢复失败（按原因）
    pub fn record_resume_failure(&self, reason: &str) {
        self.resume_failure_total.with_label_values(&[reason]).inc();
    }

    /// 记录分片重试
    pub fn record_chunk_retry(&self, reason: &str) {
        self.chunk_retry_total.with_label_values(&[reason]).inc();
    }

    /// 记录下载耗时
    pub fn record_duration(&self, outcome: &str, seconds: f64) {
        self.duration_seconds
            .with_label_values(&[outcome])
            .observe(seconds);
    }

    /// 设置吞吐
    pub fn set_throughput(&self, file_path: &str, bytes_per_sec: f64) {
        self.throughput_bytes_per_second
            .with_label_values(&[file_path])
            .set(bytes_per_sec);
    }

    /// 记录整文件校验失败
    pub fn record_integrity_outcome(&self, outcome: IntegrityOutcome, reason: &str) {
        if matches!(outcome, IntegrityOutcome::Mismatch) {
            self.integrity_failure_total
                .with_label_values(&[reason])
                .inc();
        }
    }

    /// 设置 store 大小
    pub fn set_resume_token_store_bytes(&self, backend: &str, bytes: i64) {
        self.resume_token_store_bytes
            .with_label_values(&[backend])
            .set(bytes as f64);
    }
}

static METRICS: OnceLock<AssetDownloadMetrics> = OnceLock::new();

/// 获取全局 metrics（lazy init）。
pub fn metrics() -> &'static AssetDownloadMetrics {
    METRICS.get_or_init(|| AssetDownloadMetrics::new().expect("metrics init"))
}

/// 编码为 Prometheus text format（供 scrape 端读取）。
pub fn encode_metrics_text() -> Result<String, MetricsError> {
    let encoder = TextEncoder::new();
    encoder
        .encode_to_string(&registry().gather())
        .map_err(|e| MetricsError::Encoding(e.to_string()))
}

/// Metrics 错误。
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Prometheus encoding error: {0}")]
    Encoding(String),
    #[error("Prometheus register error: {0}")]
    Register(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_init_and_record() {
        let m = metrics();
        m.record_state_transition("idle", "resolving");
        m.record_state_transition("resolving", "downloading");
        m.set_active_count("downloading", 1);
        m.add_bytes_received("asset-001", 4096);
        m.record_resume_success("asset-001");
        m.record_resume_failure("expired");
        m.record_chunk_retry("429");
        m.record_duration("completed", 1.234);
        m.set_throughput("asset-001", 524288.0);
        m.record_integrity_outcome(IntegrityOutcome::Mismatch, "size_mismatch");
        m.set_resume_token_store_bytes("sqlite", 12345);
    }

    #[test]
    fn integrity_outcome_label_is_stable() {
        assert_eq!(IntegrityOutcome::Match.as_label(), "match");
        assert_eq!(IntegrityOutcome::Mismatch.as_label(), "mismatch");
        assert_eq!(IntegrityOutcome::Skipped.as_label(), "skipped");
    }

    #[test]
    fn encode_metrics_text_contains_all_10_names() {
        // 先对所有指标都至少观测一次（否则 GaugeVec / HistogramVec 不会出现在输出中）
        let m = metrics();
        m.record_state_transition("idle", "resolving");
        m.set_active_count("downloading", 1);
        m.add_bytes_received("asset-001", 1);
        m.record_resume_success("asset-001");
        m.record_resume_failure("expired");
        m.record_chunk_retry("429");
        m.record_duration("completed", 0.1);
        m.set_throughput("asset-001", 1.0);
        m.record_integrity_outcome(IntegrityOutcome::Mismatch, "mismatch");
        m.set_resume_token_store_bytes("sqlite", 1);
        let text = encode_metrics_text().unwrap();
        // 10 项指标名都应出现在输出中
        for name in &[
            "rgs_asset_download_state_transition_total",
            "rgs_asset_download_active_count",
            "rgs_asset_download_bytes_received_total",
            "rgs_asset_download_resume_success_total",
            "rgs_asset_download_resume_failure_total",
            "rgs_asset_download_chunk_retry_total",
            "rgs_asset_download_duration_seconds",
            "rgs_asset_download_throughput_bytes_per_second",
            "rgs_asset_download_integrity_failure_total",
            "rgs_asset_download_resume_token_store_bytes",
        ] {
            assert!(text.contains(name), "missing metric: {name}");
        }
    }
}
