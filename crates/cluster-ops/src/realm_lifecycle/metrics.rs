//! rgs-lcm 可观测性指标（per RGS-DTL-042 §11.1 + RGS-SPEC-DTL-042 §4）
//!
//! 10 项 `rgs_lcm_*` Prometheus 指标（per DTL §11.1）：
//! 1. `rgs_lcm_run_state_transition_total` —— 阶段变更 PFAU 状态转移次数（counter）
//! 2. `rgs_lcm_active_runs` —— 当前进行中的阶段变更实例数（gauge）
//! 3. `rgs_lcm_drill_pass_rate` —— 演练通过率（gauge，0.0~1.0）
//! 4. `rgs_lcm_drill_to_execute_duration_seconds` —— drill_validated → executing 间隔（histogram）
//! 5. `rgs_lcm_saga_step_duration_seconds` —— 单个 Saga 步骤耗时（histogram）
//! 6. `rgs_lcm_saga_rollback_total` —— Saga 回退次数（counter）
//! 7. `rgs_lcm_drill_failure_reason_total` —— 演练失败原因分布（counter）
//! 8. `rgs_lcm_archive_query_latency_seconds` —— **归档后客服查询响应时延**（histogram，本任务 M-2074.5）
//! 9. `rgs_lcm_realm_count_by_status` —— 实时各状态 realm 数（gauge）
//! 10. `rgs_lcm_olu_consumed_by_team` —— 各团队 OLU 消耗（gauge）
//!
//! **本文件实现口径**（per RGS-SPEC-DTL-042 §4 关键标注）：
//! - 业务代码**只能**调用本文件导出的 `archive_query_latency_*` / `archive_query_count`
//! - 禁止直接调用裸 `prometheus::*` / `metrics::*`（per SPEC §4 业务代码只允许
//!   走 observability façade，本文件即该 façade 在归档域的投影）
//! - 标签限定：`feature_subtype` / `from` / `to` / `team` / `phase` / `reason` / `status`
//!   等低基数标签；`realm_id` 可作低基数标签（数量级 10² 以内）

use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, GaugeVec,
    HistogramVec, Registry,
};
use std::sync::OnceLock;

/// 全局 Prometheus Registry（懒加载）
static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get_or_init(Registry::new)
}

// ===== 1. rgs_lcm_run_state_transition_total =====
static RUN_STATE_TRANSITION_TOTAL: OnceLock<CounterVec> = OnceLock::new();

fn run_state_transition_total() -> &'static CounterVec {
    RUN_STATE_TRANSITION_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "rgs_lcm_run_state_transition_total",
            "阶段变更 PFAU 状态转移次数",
            &["feature_subtype", "from", "to"]
        )
        .expect("register rgs_lcm_run_state_transition_total")
    })
}

// ===== 2. rgs_lcm_active_runs =====
static ACTIVE_RUNS: OnceLock<GaugeVec> = OnceLock::new();

fn active_runs() -> &'static GaugeVec {
    ACTIVE_RUNS.get_or_init(|| {
        register_gauge_vec!(
            "rgs_lcm_active_runs",
            "当前进行中的阶段变更实例数",
            &["feature_subtype"]
        )
        .expect("register rgs_lcm_active_runs")
    })
}

// ===== 3. rgs_lcm_drill_pass_rate =====
static DRILL_PASS_RATE: OnceLock<GaugeVec> = OnceLock::new();

fn drill_pass_rate() -> &'static GaugeVec {
    DRILL_PASS_RATE.get_or_init(|| {
        register_gauge_vec!(
            "rgs_lcm_drill_pass_rate",
            "演练通过率（0.0~1.0）",
            &["phase"]
        )
        .expect("register rgs_lcm_drill_pass_rate")
    })
}

// ===== 4. rgs_lcm_drill_to_execute_duration_seconds =====
static DRILL_TO_EXECUTE_DURATION: OnceLock<HistogramVec> = OnceLock::new();

fn drill_to_execute_duration() -> &'static HistogramVec {
    DRILL_TO_EXECUTE_DURATION.get_or_init(|| {
        register_histogram_vec!(
            "rgs_lcm_drill_to_execute_duration_seconds",
            "drill_validated → executing 间隔秒数",
            &["feature_subtype"],
            vec![1.0, 5.0, 30.0, 60.0, 300.0, 900.0, 3600.0]
        )
        .expect("register rgs_lcm_drill_to_execute_duration_seconds")
    })
}

// ===== 5. rgs_lcm_saga_step_duration_seconds =====
static SAGA_STEP_DURATION: OnceLock<HistogramVec> = OnceLock::new();

fn saga_step_duration() -> &'static HistogramVec {
    SAGA_STEP_DURATION.get_or_init(|| {
        register_histogram_vec!(
            "rgs_lcm_saga_step_duration_seconds",
            "单个 Saga 步骤耗时秒数",
            &["feature_subtype", "step"],
            vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0]
        )
        .expect("register rgs_lcm_saga_step_duration_seconds")
    })
}

// ===== 6. rgs_lcm_saga_rollback_total =====
static SAGA_ROLLBACK_TOTAL: OnceLock<CounterVec> = OnceLock::new();

fn saga_rollback_total() -> &'static CounterVec {
    SAGA_ROLLBACK_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "rgs_lcm_saga_rollback_total",
            "Saga 回退次数",
            &["feature_subtype", "step", "reason"]
        )
        .expect("register rgs_lcm_saga_rollback_total")
    })
}

// ===== 7. rgs_lcm_drill_failure_reason_total =====
static DRILL_FAILURE_REASON_TOTAL: OnceLock<CounterVec> = OnceLock::new();

fn drill_failure_reason_total() -> &'static CounterVec {
    DRILL_FAILURE_REASON_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "rgs_lcm_drill_failure_reason_total",
            "演练失败原因分布",
            &["phase", "reason"]
        )
        .expect("register rgs_lcm_drill_failure_reason_total")
    })
}

// ===== 8. rgs_lcm_archive_query_latency_seconds (本任务 M-2074.5) =====
static ARCHIVE_QUERY_LATENCY: OnceLock<HistogramVec> = OnceLock::new();

fn archive_query_latency() -> &'static HistogramVec {
    ARCHIVE_QUERY_LATENCY.get_or_init(|| {
        register_histogram_vec!(
            "rgs_lcm_archive_query_latency_seconds",
            "归档后客服查询响应时延秒数（per RGS-DTL-042 §11.1 / M-2074.5）",
            &["query_kind", "realm_status"],
            vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0]
        )
        .expect("register rgs_lcm_archive_query_latency_seconds")
    })
}

// ===== 9. rgs_lcm_realm_count_by_status =====
static REALM_COUNT_BY_STATUS: OnceLock<GaugeVec> = OnceLock::new();

fn realm_count_by_status() -> &'static GaugeVec {
    REALM_COUNT_BY_STATUS.get_or_init(|| {
        register_gauge_vec!(
            "rgs_lcm_realm_count_by_status",
            "实时各状态 realm 数",
            &["status"]
        )
        .expect("register rgs_lcm_realm_count_by_status")
    })
}

// ===== 10. rgs_lcm_olu_consumed_by_team =====
static OLU_CONSUMED_BY_TEAM: OnceLock<GaugeVec> = OnceLock::new();

fn olu_consumed_by_team() -> &'static GaugeVec {
    OLU_CONSUMED_BY_TEAM.get_or_init(|| {
        register_gauge_vec!(
            "rgs_lcm_olu_consumed_by_team",
            "各团队 OLU 消耗（per RGS-TS-001 §6.2 token-OLU 框架）",
            &["team", "phase"]
        )
        .expect("register rgs_lcm_olu_consumed_by_team")
    })
}

/// 注册全部 10 项 LCM 指标（启动时调用一次）
///
/// **调用方**：`cluster-ops` 启动流程（per RGS-SPEC-DTL-042 §6 测试规格）
///
/// **实现说明**：prometheus crate 的 `register_*_vec!` 宏**同时**注册到全局默认
/// `prometheus::gather()` 收集器，因此本函数**不**需要二次注册到 `registry()` —
/// 触达每个 `OnceLock` 即触发懒注册 + 全局注册。
pub fn register_all_metrics() {
    // 触达每个 OnceLock 触发懒注册
    let _ = run_state_transition_total();
    let _ = active_runs();
    let _ = drill_pass_rate();
    let _ = drill_to_execute_duration();
    let _ = saga_step_duration();
    let _ = saga_rollback_total();
    let _ = drill_failure_reason_total();
    let _ = archive_query_latency();
    let _ = realm_count_by_status();
    let _ = olu_consumed_by_team();
    // 静默使用 registry() 避免 unused 警告（生产可挂自定义 Registry）
    let _r = registry();
}

// ===== 业务层 façade 函数（per SPEC §4 只允许业务代码调这些） =====

/// 记录归档后客服查询响应时延（**M-2074.5 入口**）
///
/// - `query_kind` —— 查询类型（如 `gdpr_subject_lookup` / `cs_query_archive` / `audit_lookup`）
/// - `realm_status` —— realm 当前归档状态（`hot` / `cold` / `gdpr_path`）
/// - `latency_seconds` —— 实测时延
pub fn observe_archive_query_latency(query_kind: &str, realm_status: &str, latency_seconds: f64) {
    archive_query_latency()
        .with_label_values(&[query_kind, realm_status])
        .observe(latency_seconds);
}

/// 记录 PFAU 状态转移
pub fn inc_run_state_transition(feature_subtype: &str, from: &str, to: &str) {
    run_state_transition_total()
        .with_label_values(&[feature_subtype, from, to])
        .inc();
}

/// 记录 Saga 步骤耗时
pub fn observe_saga_step_duration(feature_subtype: &str, step: &str, seconds: f64) {
    saga_step_duration()
        .with_label_values(&[feature_subtype, step])
        .observe(seconds);
}

/// 记录 Saga 回退
pub fn inc_saga_rollback(feature_subtype: &str, step: &str, reason: &str) {
    saga_rollback_total()
        .with_label_values(&[feature_subtype, step, reason])
        .inc();
}

/// 设置当前 realm 数（按状态）
pub fn set_realm_count(status: &str, count: f64) {
    realm_count_by_status()
        .with_label_values(&[status])
        .set(count);
}

/// 设置 OLU 消耗（按团队 + 阶段）
pub fn set_olu_consumed(team: &str, phase: &str, tokens: f64) {
    olu_consumed_by_team()
        .with_label_values(&[team, phase])
        .set(tokens);
}

/// 拉取所有指标的当前快照（测试 / 报告用）
pub fn gather_metrics_text() -> Result<String, prometheus::Error> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buf = Vec::new();
    encoder.encode(&metric_families, &mut buf)?;
    String::from_utf8(buf).map_err(|e| prometheus::Error::Msg(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_all_metrics_idempotent() {
        // 重复调用 register_all_metrics 不应 panic
        register_all_metrics();
        register_all_metrics();
    }

    #[test]
    fn archive_query_latency_records_value() {
        observe_archive_query_latency("gdpr_subject_lookup", "cold", 0.123);
        let text = gather_metrics_text().expect("gather");
        assert!(text.contains("rgs_lcm_archive_query_latency_seconds"));
        assert!(text.contains("gdpr_subject_lookup"));
    }

    #[test]
    fn run_state_transition_increments() {
        inc_run_state_transition("realm_lifecycle::archive", "Retired", "Archived");
        let text = gather_metrics_text().expect("gather");
        assert!(text.contains("rgs_lcm_run_state_transition_total"));
    }

    #[test]
    fn saga_step_duration_records_observation() {
        observe_saga_step_duration("realm_lifecycle::archive", "HotArchiveStep", 0.5);
        let text = gather_metrics_text().expect("gather");
        assert!(text.contains("rgs_lcm_saga_step_duration_seconds"));
    }
}
