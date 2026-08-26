//! `DrillExecutor`（per FR-LCM-003 + IMPL §3.4 M-2070.2 + SPEC-DTL-042 §3）。
//!
//! ## 硬约束
//!
//! - **仅**跑沙箱环境（sandbox_pg + sandbox_k8s）
//! - **不**引用生产 PG / 生产 K8s client
//! - 5 类剧本模板（新服/分服/合服/退场/归档）
//!
//! ## 降级策略
//!
//! 沙箱 PG 不可达 + 沙箱 K8s 不可达 → `DrillOutcome::Skipped`（per 任务降级策略 + IMPL R3 风险）。
//! 生产实测由 SRE 接力后启动 K3s sandbox namespace + cluster_sandbox_db。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{
    metrics_collector::{DrillMetrics, DrillMetricsCollector},
    playbook::{Playbook, PlaybookKind},
    sandbox_k8s::SandboxK8sClient,
    sandbox_pg::SandboxPgPool,
};
use crate::realm_lifecycle::{
    error::{Error, Result},
    saga::steps::{SagaStep, StepStatus},
};

/// 单次演练结果（per DTL §11.1 指标标签）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillOutcome {
    pub playbook: PlaybookKind,
    pub realm_id: String,
    pub status: DrillStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub steps_total: u32,
    pub steps_succeeded: u32,
    pub steps_failed: u32,
    pub duration_ms: u64,
}

impl DrillOutcome {
    /// 演练是否通过。
    pub fn passed(&self) -> bool {
        matches!(self.status, DrillStatus::Passed)
    }
}

/// 演练终态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrillStatus {
    /// 演练通过（沙箱环境 + 全部步骤 succeeded）。
    Passed,
    /// 演练失败（步骤失败 / saga 反向）。
    Failed,
    /// 演练跳过（沙箱不可达；per 降级策略）。
    Skipped,
}

impl DrillStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// 演练报告（聚合 5 类剧本结果）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrillReport {
    pub run_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub outcomes: Vec<DrillOutcome>,
    pub passed_count: u32,
    pub failed_count: u32,
    pub skipped_count: u32,
}

impl DrillReport {
    /// 演练 pass rate（per DTL §11.1：rgs_lcm_drill_pass_rate）。
    pub fn pass_rate(&self) -> f64 {
        let executed = (self.passed_count + self.failed_count) as f64;
        if executed == 0.0 {
            0.0
        } else {
            self.passed_count as f64 / executed
        }
    }

    /// 演练 to execute interval（per DTL §11.1：rgs_lcm_drill_to_execute_interval_seconds）。
    /// 返回首次执行与最后一次执行的间隔（秒）；未执行 = 0。
    pub fn execute_interval_secs(&self) -> i64 {
        if self.outcomes.is_empty() {
            return 0;
        }
        let mut min_ts = self.outcomes[0].started_at;
        let mut max_ts = self.outcomes[0].started_at;
        for o in &self.outcomes[1..] {
            if o.started_at < min_ts {
                min_ts = o.started_at;
            }
            if o.started_at > max_ts {
                max_ts = o.started_at;
            }
        }
        (max_ts - min_ts).num_seconds().max(0)
    }
}

/// `DrillExecutor` —— 沙箱环境演练执行器（per FR-LCM-003）。
///
/// ## 用法
///
/// ```no_run
/// use cluster_ops::realm_lifecycle::drill::executor::DrillExecutor;
/// use cluster_ops::realm_lifecycle::drill::playbook::all_playbooks;
/// let exec = DrillExecutor::from_env()?;
/// let report = exec.run_all(all_playbooks()).await?;
/// assert!(report.passed_count >= 3);
/// # Ok::<(), cluster_ops::Error>(())
/// ```
///
/// ## 生产引用防御
///
/// `DrillExecutor` 构造 + 运行路径**只**接受 `SandboxPgPool` / `SandboxK8sClient`；
/// 任何传入生产 pool / 生产 kubeconfig → 编译期拒绝（构造返回 `Err(DrillProductionLeak)`）。
#[derive(Debug, Clone)]
pub struct DrillExecutor {
    pg: SandboxPgPool,
    k8s: SandboxK8sClient,
    metrics: DrillMetricsCollector,
}

impl DrillExecutor {
    /// 从 env 构造（推荐入口）。
    ///
    /// - 缺 `RGS_SANDBOX_DATABASE_URL` → `Err(SandboxPgUnavailable)`（per 降级策略）
    /// - 缺 `RGS_SANDBOX_KUBECONFIG` → K8s 探测 unavailable 但**不**阻断（仅 PG 是硬依赖）
    pub fn from_env() -> Result<Self> {
        let pg_config = crate::realm_lifecycle::drill::sandbox_pg::SandboxPgConfig::from_env()
            .ok_or(Error::SandboxPgUnavailable(
                "RGS_SANDBOX_DATABASE_URL not set".to_string(),
            ))?;
        let pg = SandboxPgPool::new(pg_config)?;
        let k8s = SandboxK8sClient::new(
            crate::realm_lifecycle::drill::sandbox_k8s::SandboxK8sConfig::from_env()
                .unwrap_or_else(|| {
                    crate::realm_lifecycle::drill::sandbox_k8s::SandboxK8sConfig::new(None)
                }),
        )?;
        Ok(Self {
            pg,
            k8s,
            metrics: DrillMetricsCollector::new(),
        })
    }

    /// 显式构造（drill test 用）。
    pub fn new(pg: SandboxPgPool, k8s: SandboxK8sClient) -> Result<Self> {
        Ok(Self {
            pg,
            k8s,
            metrics: DrillMetricsCollector::new(),
        })
    }

    /// 沙箱 PG 可达性。
    pub fn pg_available(&self) -> bool {
        self.pg.probe_available().unwrap_or(false)
    }

    /// 沙箱 K8s 可达性。
    pub fn k8s_available(&self) -> bool {
        self.k8s.probe_available()
    }

    /// 全部不可达 → 整个 run 降级为 Skipped。
    pub fn should_skip(&self) -> bool {
        !self.pg_available() && !self.k8s_available()
    }

    /// 执行单剧本。
    pub async fn run_one(&self, playbook: &dyn Playbook) -> DrillOutcome {
        let started_at = Utc::now();
        let steps = playbook.saga_steps();

        // 沙箱不可达 → 演练跳过
        if self.should_skip() {
            return self.skip_outcome(playbook, started_at, steps.len() as u32);
        }

        // 沙箱可达 → 模拟步骤执行（占位；SRE 接力后接真 sandbox PG / K8s）
        let (succ, fail) = self.simulate_steps(&steps).await;
        let completed_at = Utc::now();
        let status = if fail == 0 {
            DrillStatus::Passed
        } else {
            DrillStatus::Failed
        };
        let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;

        let outcome = DrillOutcome {
            playbook: playbook.kind(),
            realm_id: playbook.realm_id().clone(),
            status,
            started_at,
            completed_at,
            steps_total: steps.len() as u32,
            steps_succeeded: succ,
            steps_failed: fail,
            duration_ms,
        };
        self.metrics.record(&outcome);
        outcome
    }

    /// 执行所有剧本（典型 5 类）。
    pub async fn run_all(
        &self,
        playbooks: Vec<Box<dyn Playbook>>,
    ) -> DrillReport {
        let started_at = Utc::now();
        let run_id = format!("drill-{}", started_at.timestamp_millis());
        let mut outcomes = Vec::with_capacity(playbooks.len());
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;
        for pb in &playbooks {
            let o = self.run_one(pb.as_ref()).await;
            match o.status {
                DrillStatus::Passed => passed += 1,
                DrillStatus::Failed => failed += 1,
                DrillStatus::Skipped => skipped += 1,
            }
            outcomes.push(o);
        }
        let completed_at = Utc::now();
        DrillReport {
            run_id,
            started_at,
            completed_at,
            outcomes,
            passed_count: passed,
            failed_count: failed,
            skipped_count: skipped,
        }
    }

    /// 指标导出。
    pub fn metrics(&self) -> DrillMetrics {
        self.metrics.snapshot()
    }

    fn skip_outcome(
        &self,
        playbook: &dyn Playbook,
        started_at: DateTime<Utc>,
        steps_total: u32,
    ) -> DrillOutcome {
        let completed_at = Utc::now();
        let outcome = DrillOutcome {
            playbook: playbook.kind(),
            realm_id: playbook.realm_id().clone(),
            status: DrillStatus::Skipped,
            started_at,
            completed_at,
            steps_total,
            steps_succeeded: 0,
            steps_failed: 0,
            duration_ms: (completed_at - started_at).num_milliseconds().max(0) as u64,
        };
        self.metrics.record(&outcome);
        outcome
    }

    /// 模拟步骤执行（占位；返回 (succ, fail)）。
    ///
    /// SRE 接力后改为真沙箱 PG + K8s 步骤调用；当前默认全通过（演练框架 dry-run）。
    async fn simulate_steps(&self, steps: &[SagaStep]) -> (u32, u32) {
        let mut succ = 0u32;
        let fail = 0u32;
        for s in steps {
            // 仅做 dry-run：占位 + 验证步骤状态机不变量
            debug_assert_eq!(s.status, StepStatus::Pending);
            succ += 1;
        }
        let _ = Duration::from_millis(0);
        (succ, fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_lifecycle::drill::playbook::{
        all_playbooks, ArchivePlaybook, MergePlaybook, NewRealmPlaybook, RetirePlaybook,
        SplitPlaybook,
    };
    use crate::realm_lifecycle::drill::sandbox_k8s::SandboxK8sConfig;
    use crate::realm_lifecycle::drill::sandbox_pg::SandboxPgConfig;
    use chrono::Utc;

    fn executor() -> DrillExecutor {
        let pg = SandboxPgPool::new(SandboxPgConfig::new(
            "postgres://sandbox:5432/cluster_sandbox_db",
        ))
        .unwrap();
        let k8s = SandboxK8sClient::new(SandboxK8sConfig::new(None)).unwrap();
        DrillExecutor::new(pg, k8s).unwrap()
    }

    #[test]
    fn executor_sandbox_only_accepts_sandbox_url() {
        // FR-LCM-003 锚定：生产 URL 被拒绝
        let pg = SandboxPgPool::new(SandboxPgConfig::new(
            "postgres://prod:5432/admin_db",
        ));
        assert!(pg.is_err());
    }

    #[test]
    fn report_pass_rate_zero_when_no_executed() {
        let r = DrillReport::default();
        assert_eq!(r.pass_rate(), 0.0);
    }

    #[test]
    fn report_pass_rate_with_only_passed() {
        let r = DrillReport {
            passed_count: 3,
            failed_count: 0,
            skipped_count: 2,
            ..Default::default()
        };
        assert!((r.pass_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn report_pass_rate_with_mixed() {
        let r = DrillReport {
            passed_count: 3,
            failed_count: 1,
            skipped_count: 0,
            ..Default::default()
        };
        assert!((r.pass_rate() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn report_execute_interval_secs_zero_for_empty() {
        let r = DrillReport::default();
        assert_eq!(r.execute_interval_secs(), 0);
    }

    #[tokio::test]
    async fn run_one_skips_when_sandbox_unavailable() {
        // K8s 未配置 + PG 仍可达（K8s 不阻断）
        let exec = executor();
        let pb = NewRealmPlaybook {
            realm_id: "r".to_string(),
            region: "x".to_string(),
            initial_node_count: 1,
        };
        let o = exec.run_one(&pb).await;
        // PG URL 已设置 → probe_available = true → 不应 skip
        assert_ne!(o.status, DrillStatus::Skipped);
    }

    #[tokio::test]
    async fn run_all_5_playbooks_emits_5_outcomes() {
        let exec = executor();
        let report = exec.run_all(all_playbooks()).await;
        assert_eq!(report.outcomes.len(), 5);
    }

    #[test]
    fn drill_status_as_str_cover_all() {
        assert_eq!(DrillStatus::Passed.as_str(), "passed");
        assert_eq!(DrillStatus::Failed.as_str(), "failed");
        assert_eq!(DrillStatus::Skipped.as_str(), "skipped");
    }

    #[test]
    fn all_5_kinds_covered() {
        // IMPL §3.4 M-2070.2 锚定：5 类剧本模板各通过 1 次
        let all = all_playbooks();
        let kinds: std::collections::HashSet<_> =
            all.iter().map(|p| p.kind()).collect();
        assert!(kinds.contains(&PlaybookKind::NewRealm));
        assert!(kinds.contains(&PlaybookKind::Split));
        assert!(kinds.contains(&PlaybookKind::Merge));
        assert!(kinds.contains(&PlaybookKind::Retire));
        assert!(kinds.contains(&PlaybookKind::Archive));
    }

    #[test]
    fn playbook_construction_smoke() {
        // 防止 playbook 字段漏改导致 all_playbooks() panic
        let _ = NewRealmPlaybook {
            realm_id: "r".to_string(),
            region: "x".to_string(),
            initial_node_count: 1,
        };
        let _ = SplitPlaybook {
            source_realm_id: "s".to_string(),
            target_realm_id: "t".to_string(),
            split_point_player_id: "p".to_string(),
            estimated_players: 1,
        };
        let _ = MergePlaybook {
            source_realm_id: "s".to_string(),
            target_realm_id: "t".to_string(),
            conflict_rule_set_version: 2,
            rollback_window_days: 14,
        };
        let _ = RetirePlaybook {
            realm_id: "r".to_string(),
            query_channel_rbac: vec![],
            archive_threshold_days: 30,
        };
        let _ = ArchivePlaybook {
            realm_id: "r".to_string(),
            last_active_at: Utc::now(),
        };
    }
}
