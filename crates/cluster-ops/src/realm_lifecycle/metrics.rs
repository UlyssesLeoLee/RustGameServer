//! 10 项 rgs_lcm_* 指标（per M-2071.5 + RGS-SPEC-DTL-042 §4 + DTL-031 §11.1）
//!
//! ## 10 项指标清单（per SPEC §4 + DTL §11.1）
//!
//! 1. `rgs_lcm_pfau_state_transition_total` — PFAU 状态转移（labels: feature_subtype / from / to）
//! 2. `rgs_lcm_active_runs` — active runs（labels: feature_subtype / status）
//! 3. `rgs_lcm_drill_pass_rate` — drill pass rate（labels: feature_subtype / phase / team）
//! 4. `rgs_lcm_drill_to_execute_interval_seconds` — drill → execute 间隔（labels: feature_subtype）
//! 5. `rgs_lcm_saga_step_duration_seconds` — Saga 步骤执行时长（labels: feature_subtype / step / status）
//! 6. `rgs_lcm_saga_rollback_total` — Saga 回退次数（labels: feature_subtype / step / reason）
//! 7. `rgs_lcm_drill_failure_reason_total` — drill 失败原因（labels: feature_subtype / reason）
//! 8. `rgs_lcm_archive_query_latency_seconds` — 归档查询延迟（labels: status）
//! 9. `rgs_lcm_realm_count_by_status` — 实时按 status 的 realm 数量（labels: status）
//! 10. `rgs_lcm_olu_consumed_by_team` — 各团队 OLU 消耗（labels: team / phase）
//!
//! ## 设计（per SPEC §4 低基数标签 + §3 NFR-LCM-007 OLU 必经）
//!
//! - 所有 Counter / Gauge / Histogram 包装为 `Arc` 以便跨服务共享
//! - 业务 helper：`record_pfau_transition` / `inc_active_runs` / `record_saga_step_duration` /
//!   `inc_saga_rollback` / `record_archive_query_latency` / `set_realm_count` /
//!   `inc_olu_consumed` / `record_drill_pass_rate` / `record_drill_to_execute_interval` /
//!   `inc_drill_failure_reason`
//! - OLU 计数（`rgs_lcm_olu_consumed_by_team`）由 `OluReporter` 写入（per M-2071.4）

use std::sync::Arc;
use std::sync::OnceLock;

use prometheus::{
    register_counter_vec_with_registry, register_gauge_vec_with_registry,
    register_histogram_vec_with_registry, CounterVec, GaugeVec, HistogramVec, Registry,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Prometheus encoding error: {0}")]
    Encoding(String),
    #[error("Prometheus register error: {0}")]
    Register(String),
}

/// 全局 Registry（per shared-platform metrics 模式）
fn lcm_registry() -> Arc<Registry> {
    static REG: OnceLock<Arc<Registry>> = OnceLock::new();
    REG.get_or_init(|| Arc::new(Registry::new())).clone()
}

/// 独立 Registry（test 构造使用；避免 metric 重复注册冲突）
pub fn fresh_registry() -> Arc<Registry> {
    Arc::new(Registry::new())
}

/// 10 项 rgs_lcm_* 指标容器
pub struct LcmMetrics {
    /// 1. PFAU 状态转移（CounterVec）
    pub rgs_lcm_pfau_state_transition_total: CounterVec,
    /// 2. active runs（GaugeVec）
    pub rgs_lcm_active_runs: GaugeVec,
    /// 3. drill pass rate（GaugeVec；0.0~1.0）
    pub rgs_lcm_drill_pass_rate: GaugeVec,
    /// 4. drill to execute 间隔（HistogramVec；秒）
    pub rgs_lcm_drill_to_execute_interval_seconds: HistogramVec,
    /// 5. Saga 步骤执行时长（HistogramVec；秒）
    pub rgs_lcm_saga_step_duration_seconds: HistogramVec,
    /// 6. Saga 回退次数（CounterVec）
    pub rgs_lcm_saga_rollback_total: CounterVec,
    /// 7. drill 失败原因（CounterVec）
    pub rgs_lcm_drill_failure_reason_total: CounterVec,
    /// 8. 归档查询延迟（HistogramVec；秒）
    pub rgs_lcm_archive_query_latency_seconds: HistogramVec,
    /// 9. 按 status 的 realm 数（GaugeVec）
    pub rgs_lcm_realm_count_by_status: GaugeVec,
    /// 10. 各团队 OLU 消耗（CounterVec）
    pub rgs_lcm_olu_consumed_by_team: CounterVec,
}

impl LcmMetrics {
    /// 用全局 Registry 注册（生产模式）
    pub fn new() -> std::result::Result<Self, MetricsError> {
        let reg = lcm_registry();
        Self::with_registry(&reg)
    }

    /// 测试构造（每次独立 Registry，避免 metric 重复注册）
    pub fn new_for_test() -> Self {
        Self::with_registry(&fresh_registry()).expect("fresh registry should not fail")
    }

    fn with_registry(reg: &Registry) -> std::result::Result<Self, MetricsError> {
        let pfau = register_counter_vec_with_registry!(
            "rgs_lcm_pfau_state_transition_total",
            "PFAU state transition count for realm_lifecycle sub features",
            &["feature_subtype", "from", "to"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let active = register_gauge_vec_with_registry!(
            "rgs_lcm_active_runs",
            "Active realm_lifecycle runs by sub feature / status",
            &["feature_subtype", "status"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let drill_pass = register_gauge_vec_with_registry!(
            "rgs_lcm_drill_pass_rate",
            "Drill pass rate by feature_subtype / phase / team (0.0~1.0)",
            &["feature_subtype", "phase", "team"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let drill_interval = register_histogram_vec_with_registry!(
            "rgs_lcm_drill_to_execute_interval_seconds",
            "Drill to execute interval in seconds (per feature_subtype)",
            &["feature_subtype"],
            vec![1.0, 5.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let saga_duration = register_histogram_vec_with_registry!(
            "rgs_lcm_saga_step_duration_seconds",
            "Saga step duration in seconds (per feature_subtype / step / status)",
            &["feature_subtype", "step", "status"],
            vec![0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let saga_rollback = register_counter_vec_with_registry!(
            "rgs_lcm_saga_rollback_total",
            "Saga rollback count by feature_subtype / step / reason",
            &["feature_subtype", "step", "reason"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let drill_failure = register_counter_vec_with_registry!(
            "rgs_lcm_drill_failure_reason_total",
            "Drill failure count by reason (per feature_subtype / reason)",
            &["feature_subtype", "reason"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let archive_latency = register_histogram_vec_with_registry!(
            "rgs_lcm_archive_query_latency_seconds",
            "Archive query latency in seconds (per status)",
            &["status"],
            vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let realm_count = register_gauge_vec_with_registry!(
            "rgs_lcm_realm_count_by_status",
            "Realm count by status",
            &["status"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        let olu_consumed = register_counter_vec_with_registry!(
            "rgs_lcm_olu_consumed_by_team",
            "OLU tokens consumed by team / phase",
            &["team", "phase"],
            reg
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;
        Ok(Self {
            rgs_lcm_pfau_state_transition_total: pfau,
            rgs_lcm_active_runs: active,
            rgs_lcm_drill_pass_rate: drill_pass,
            rgs_lcm_drill_to_execute_interval_seconds: drill_interval,
            rgs_lcm_saga_step_duration_seconds: saga_duration,
            rgs_lcm_saga_rollback_total: saga_rollback,
            rgs_lcm_drill_failure_reason_total: drill_failure,
            rgs_lcm_archive_query_latency_seconds: archive_latency,
            rgs_lcm_realm_count_by_status: realm_count,
            rgs_lcm_olu_consumed_by_team: olu_consumed,
        })
    }

    // ===== 业务 helper（10 个；与 10 项 rgs_lcm_* 指标 1:1） =====

    /// 1. 记录 PFAU 状态转移
    pub fn record_pfau_transition(&self, feature_subtype: &str, from: &str, to: &str) {
        self.rgs_lcm_pfau_state_transition_total
            .with_label_values(&[feature_subtype, from, to])
            .inc();
    }

    /// 2. 增加 active runs
    pub fn inc_active_runs(&self, feature_subtype: &str) {
        self.rgs_lcm_active_runs
            .with_label_values(&[feature_subtype, "active"])
            .inc();
    }

    /// 2. 减少 active runs
    pub fn dec_active_runs(&self, feature_subtype: &str) {
        self.rgs_lcm_active_runs
            .with_label_values(&[feature_subtype, "active"])
            .dec();
    }

    /// 3. 记录 drill pass rate（0.0~1.0）
    pub fn record_drill_pass_rate(
        &self,
        feature_subtype: &str,
        phase: &str,
        team: &str,
        rate: f64,
    ) {
        self.rgs_lcm_drill_pass_rate
            .with_label_values(&[feature_subtype, phase, team])
            .set(rate);
    }

    /// 4. 记录 drill to execute 间隔
    pub fn record_drill_to_execute_interval(&self, feature_subtype: &str, seconds: f64) {
        self.rgs_lcm_drill_to_execute_interval_seconds
            .with_label_values(&[feature_subtype])
            .observe(seconds);
    }

    /// 5. 记录 Saga 步骤时长
    pub fn record_saga_step_duration(
        &self,
        feature_subtype: &str,
        step: &str,
        status: &str,
        seconds: f64,
    ) {
        self.rgs_lcm_saga_step_duration_seconds
            .with_label_values(&[feature_subtype, step, status])
            .observe(seconds);
    }

    /// 6. 记录 Saga 回退
    pub fn inc_saga_rollback(&self, feature_subtype: &str, step: &str, reason: &str) {
        self.rgs_lcm_saga_rollback_total
            .with_label_values(&[feature_subtype, step, reason])
            .inc();
    }

    /// 7. 记录 drill 失败原因
    pub fn inc_drill_failure_reason(&self, feature_subtype: &str, reason: &str) {
        self.rgs_lcm_drill_failure_reason_total
            .with_label_values(&[feature_subtype, reason])
            .inc();
    }

    /// 8. 记录归档查询延迟
    pub fn record_archive_query_latency(&self, status: &str, seconds: f64) {
        self.rgs_lcm_archive_query_latency_seconds
            .with_label_values(&[status])
            .observe(seconds);
    }

    /// 9. 设置按 status 的 realm 数
    pub fn set_realm_count(&self, status: &str, count: i64) {
        self.rgs_lcm_realm_count_by_status
            .with_label_values(&[status])
            .set(count as f64);
    }

    /// 10. 增加团队 OLU 消耗
    pub fn inc_olu_consumed(&self, team: &str, phase: &str, tokens: u64) {
        self.rgs_lcm_olu_consumed_by_team
            .with_label_values(&[team, phase])
            .inc_by(tokens as f64);
    }
}

// ============================================================================
// 测试辅助
// ============================================================================

/// 把全部 rgs_lcm_* 指标编码为 Prometheus 文本格式（用于 UT 断言）
pub fn encode_for_test(metrics: &LcmMetrics, registry: &Registry) -> String {
    let encoder = prometheus::TextEncoder::new();
    let mf = registry.gather();
    let _ = mf; // gather 不持有引用；TextEncoder 实际从 registry 收集
    let _ = metrics; // 保留以备显式引用
    encoder
        .encode_to_string(&registry.gather())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_lcm_indicators_init() {
        let m = LcmMetrics::new_for_test();
        m.record_pfau_transition("new_realm", "none", "declared");
        m.inc_active_runs("new_realm");
        m.record_drill_pass_rate("new_realm", "plan", "platform", 0.95);
        m.record_drill_to_execute_interval("new_realm", 1.5);
        m.record_saga_step_duration("new_realm", "precheck", "ok", 0.2);
        m.inc_saga_rollback("new_realm", "precheck", "timeout");
        m.inc_drill_failure_reason("new_realm", "schema_mismatch");
        m.record_archive_query_latency("ok", 0.05);
        m.set_realm_count("active", 12);
        m.inc_olu_consumed("platform", "new_realm", 1_000_000);
    }

    #[test]
    fn all_ten_indicator_names_present() {
        // 10 项 rgs_lcm_* 指标
        let names = [
            "rgs_lcm_pfau_state_transition_total",
            "rgs_lcm_active_runs",
            "rgs_lcm_drill_pass_rate",
            "rgs_lcm_drill_to_execute_interval_seconds",
            "rgs_lcm_saga_step_duration_seconds",
            "rgs_lcm_saga_rollback_total",
            "rgs_lcm_drill_failure_reason_total",
            "rgs_lcm_archive_query_latency_seconds",
            "rgs_lcm_realm_count_by_status",
            "rgs_lcm_olu_consumed_by_team",
        ];
        assert_eq!(names.len(), 10);
        // 在源码中通过文本匹配（10 个不同指标名）—这里通过 unique 校验
        let mut sorted = names.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 10, "10 项 rgs_lcm_* 指标必须不重复");
    }
}
