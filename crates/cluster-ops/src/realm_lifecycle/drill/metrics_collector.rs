//! Drill 指标采集（per DTL §11.1 + IMPL §3.4 M-2070.14）。
//!
//! 采集 2 项指标：
//! - `rgs_lcm_drill_pass_rate` —— drill 演练 pass rate
//! - `rgs_lcm_drill_to_execute_interval_seconds` —— drill to execute interval

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::executor::{DrillOutcome, DrillStatus};

/// 指标快照（per DTL §11.1：低基数标签 feature_subtype / status）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillMetrics {
    pub pass_rate: f64,
    pub execute_interval_seconds: i64,
    pub total_runs: u64,
    pub passed_runs: u64,
    pub failed_runs: u64,
    pub skipped_runs: u64,
    /// per playbook kind pass rate（per DTL §11.1 feature_subtype 标签）。
    pub per_kind_pass_rate: std::collections::HashMap<String, f64>,
}

impl Default for DrillMetrics {
    fn default() -> Self {
        Self {
            pass_rate: 0.0,
            execute_interval_seconds: 0,
            total_runs: 0,
            passed_runs: 0,
            failed_runs: 0,
            skipped_runs: 0,
            per_kind_pass_rate: std::collections::HashMap::new(),
        }
    }
}

/// 指标采集器（线程安全）。
#[derive(Debug, Clone, Default)]
pub struct DrillMetricsCollector {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    outcomes: Vec<DrillOutcome>,
}

impl DrillMetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录单次演练结果。
    pub fn record(&self, outcome: &DrillOutcome) {
        let mut inner = self.inner.lock().expect("DrillMetricsCollector mutex poisoned");
        inner.outcomes.push(outcome.clone());
    }

    /// 导出指标快照。
    pub fn snapshot(&self) -> DrillMetrics {
        let inner = self.inner.lock().expect("DrillMetricsCollector mutex poisoned");
        let total = inner.outcomes.len() as u64;
        let passed = inner
            .outcomes
            .iter()
            .filter(|o| o.status == DrillStatus::Passed)
            .count() as u64;
        let failed = inner
            .outcomes
            .iter()
            .filter(|o| o.status == DrillStatus::Failed)
            .count() as u64;
        let skipped = inner
            .outcomes
            .iter()
            .filter(|o| o.status == DrillStatus::Skipped)
            .count() as u64;

        // pass rate = passed / (passed + failed)；skipped 不计入分母
        let executed = (passed + failed) as f64;
        let pass_rate = if executed == 0.0 {
            0.0
        } else {
            passed as f64 / executed
        };

        // to execute interval：首次与最后一次 started_at 间隔（秒）
        let interval = if inner.outcomes.is_empty() {
            0
        } else {
            let mut min_ts = inner.outcomes[0].started_at;
            let mut max_ts = inner.outcomes[0].started_at;
            for o in &inner.outcomes[1..] {
                if o.started_at < min_ts {
                    min_ts = o.started_at;
                }
                if o.started_at > max_ts {
                    max_ts = o.started_at;
                }
            }
            (max_ts - min_ts).num_seconds().max(0)
        };

        // per kind pass rate
        let mut per_kind: std::collections::HashMap<String, (u64, u64)> =
            std::collections::HashMap::new();
        for o in &inner.outcomes {
            let key = o.playbook.as_str().to_string();
            let entry = per_kind.entry(key).or_insert((0, 0));
            entry.0 += 1;
            if o.status == DrillStatus::Passed {
                entry.1 += 1;
            }
        }
        let per_kind_pass_rate = per_kind
            .into_iter()
            .map(|(k, (t, p))| {
                let rate = if t == 0 { 0.0 } else { p as f64 / t as f64 };
                (k, rate)
            })
            .collect();

        DrillMetrics {
            pass_rate,
            execute_interval_seconds: interval,
            total_runs: total,
            passed_runs: passed,
            failed_runs: failed,
            skipped_runs: skipped,
            per_kind_pass_rate,
        }
    }

    /// 清空（drill test 重置用）。
    pub fn reset(&self) {
        let mut inner = self.inner.lock().expect("DrillMetricsCollector mutex poisoned");
        inner.outcomes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_lifecycle::drill::playbook::PlaybookKind;
    use chrono::Utc;

    fn outcome(kind: PlaybookKind, status: DrillStatus) -> DrillOutcome {
        DrillOutcome {
            playbook: kind,
            realm_id: "r".to_string(),
            status,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            steps_total: 3,
            steps_succeeded: 3,
            steps_failed: 0,
            duration_ms: 10,
        }
    }

    #[test]
    fn empty_collector_snapshot_is_zero() {
        let c = DrillMetricsCollector::new();
        let m = c.snapshot();
        assert_eq!(m.total_runs, 0);
        assert_eq!(m.pass_rate, 0.0);
        assert_eq!(m.execute_interval_seconds, 0);
    }

    #[test]
    fn pass_rate_ignores_skipped() {
        let c = DrillMetricsCollector::new();
        c.record(&outcome(PlaybookKind::NewRealm, DrillStatus::Passed));
        c.record(&outcome(PlaybookKind::NewRealm, DrillStatus::Passed));
        c.record(&outcome(PlaybookKind::NewRealm, DrillStatus::Skipped));
        let m = c.snapshot();
        // 2 passed / (2 passed + 0 failed) = 1.0
        assert!((m.pass_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(m.passed_runs, 2);
        assert_eq!(m.skipped_runs, 1);
    }

    #[test]
    fn pass_rate_mixed() {
        let c = DrillMetricsCollector::new();
        c.record(&outcome(PlaybookKind::Merge, DrillStatus::Passed));
        c.record(&outcome(PlaybookKind::Merge, DrillStatus::Failed));
        c.record(&outcome(PlaybookKind::Merge, DrillStatus::Failed));
        let m = c.snapshot();
        // 1 / 3 ≈ 0.333
        assert!((m.pass_rate - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(m.passed_runs, 1);
        assert_eq!(m.failed_runs, 2);
    }

    #[test]
    fn per_kind_pass_rate() {
        let c = DrillMetricsCollector::new();
        c.record(&outcome(PlaybookKind::NewRealm, DrillStatus::Passed));
        c.record(&outcome(PlaybookKind::Merge, DrillStatus::Failed));
        c.record(&outcome(PlaybookKind::Merge, DrillStatus::Passed));
        let m = c.snapshot();
        assert_eq!(m.per_kind_pass_rate.get("new_realm"), Some(&1.0));
        assert_eq!(m.per_kind_pass_rate.get("merge"), Some(&0.5));
    }

    #[test]
    fn reset_clears_outcomes() {
        let c = DrillMetricsCollector::new();
        c.record(&outcome(PlaybookKind::NewRealm, DrillStatus::Passed));
        c.reset();
        let m = c.snapshot();
        assert_eq!(m.total_runs, 0);
    }
}
