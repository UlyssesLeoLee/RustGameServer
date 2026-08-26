//! M-2070.9: AC-LCM-007（归档冷热分层）演练（per RGS-IMPL-PLAN-LCM-001 §3.4 + SPEC-DTL-042 §7）。
//!
//! 验收项（per RGS-REQ-004 §3.7 AC-LCM-007 + DTL-042 §5.6 + FR-LCM-081 + RSK-LCM-005）：
//! - 3 步 Saga：ClassifyHotCold → MigrateToStorage → ReplicateForNPlus2
//! - 冷热分层阈值：3 年热 + 10 年冷（per SPEC §8 TBD-DTL-042-01）
//! - N+2 存储冗余（per RSK-LCM-005 缓解）
//! - 归档**不**删除数据（per FR-LCM-081 硬约束）

#![allow(clippy::result_large_err)]

use cluster_ops::realm_lifecycle::drill::executor::{DrillExecutor, DrillStatus};
use cluster_ops::realm_lifecycle::drill::playbook::{ArchivePlaybook, Playbook};
use cluster_ops::realm_lifecycle::drill::sandbox_k8s::{SandboxK8sClient, SandboxK8sConfig};
use cluster_ops::realm_lifecycle::drill::sandbox_pg::{SandboxPgConfig, SandboxPgPool};
use cluster_ops::realm_lifecycle::operations::archive::{
    ArchiveTier, ARCHIVE_REDUNDANCY, COLD_TIER_YEARS, HOT_TIER_YEARS,
};
use cluster_ops::realm_lifecycle::plans::archive_policy::ArchivePolicy;
use cluster_ops::realm_lifecycle::saga::steps::SagaStepKind;

fn executor_or_skip() -> Option<DrillExecutor> {
    let pg = SandboxPgPool::new(SandboxPgConfig::new(
        "postgres://sandbox:5432/cluster_sandbox_db",
    ))
    .ok()?;
    let k8s = SandboxK8sClient::new(SandboxK8sConfig::new(None)).ok()?;
    DrillExecutor::new(pg, k8s).ok()
}

/// AC-LCM-007 演练：归档 3 步 Saga。
#[tokio::test]
async fn ac_lcm_007_archive_drill() {
    let Some(exec) = executor_or_skip() else {
        eprintln!("SKIP: executor init failed");
        return;
    };
    let pb = ArchivePlaybook {
        realm_id: "rlm-drill-archive-007".to_string(),
        last_active_at: chrono::Utc::now() - chrono::Duration::days(365 * 2),
    };
    let outcome = exec.run_one(&pb).await;
    if outcome.status == DrillStatus::Skipped {
        eprintln!("SKIP: AC-LCM-007 sandbox not available — 待 SRE 接力后跑真实环境");
        return;
    }
    assert_eq!(outcome.status, DrillStatus::Passed);
    assert_eq!(outcome.steps_total, 3);
    assert_eq!(outcome.steps_succeeded, 3);
}

/// AC-LCM-007 验收：冷热分层阈值（3 年热 + 10 年冷，per SPEC §8 TBD-DTL-042-01）。
#[test]
fn ac_lcm_007_hot_cold_thresholds_match_spec() {
    assert_eq!(HOT_TIER_YEARS, 3);
    assert_eq!(COLD_TIER_YEARS, 10);
}

/// AC-LCM-007 验收：N+2 冗余（per RSK-LCM-005 缓解）。
#[test]
fn ac_lcm_007_redundancy_is_n_plus_two() {
    assert_eq!(ARCHIVE_REDUNDANCY, 3);
}

/// AC-LCM-007 验收：FR-LCM-081 锚定 —— row count 前后必须相等。
#[test]
fn ac_lcm_007_archive_must_not_delete_data_fr_lcm_081() {
    use cluster_ops::realm_lifecycle::error::Error;
    // row count 前后相等 → ok
    let r = ArchivePolicy::assert_row_count_preserved(1000, 1000, &"rlm-1".to_string());
    assert!(r.is_ok());
    // row count 减少 → ArchiveDeleteForbidden
    let r = ArchivePolicy::assert_row_count_preserved(1000, 999, &"rlm-1".to_string());
    assert!(matches!(r, Err(Error::ArchiveDeleteForbidden { .. })));
}

/// AC-LCM-007 验收：归档 3 步顺序正确。
#[test]
fn ac_lcm_007_archive_3_steps_order() {
    let pb = ArchivePlaybook {
        realm_id: "r".to_string(),
        last_active_at: chrono::Utc::now(),
    };
    let steps = pb.saga_steps();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].kind, SagaStepKind::ClassifyHotCold);
    assert_eq!(steps[1].kind, SagaStepKind::MigrateToStorage);
    assert_eq!(steps[2].kind, SagaStepKind::ReplicateForNPlus2);
}

/// AC-LCM-007 验收：冷热分层判定。
#[test]
fn ac_lcm_007_classify_hot_cold() {
    let p = ArchivePolicy::default();
    let now = chrono::Utc::now();
    use chrono::Duration;
    // 1 年内 → 热
    assert_eq!(p.classify(now - Duration::days(365), now), ArchiveTier::Hot);
    // 5 年 → 冷
    assert_eq!(
        p.classify(now - Duration::days(365 * 5), now),
        ArchiveTier::Cold
    );
}

/// AC-LCM-007 演练：与 Service trait 的 archive 一致性。
#[tokio::test]
async fn ac_lcm_007_service_archive_consistency() {
    use cluster_ops::realm_lifecycle::service::{LifecyclePhase, NoopRealmLifecycleService, RealmLifecycleService};
    let svc = NoopRealmLifecycleService;
    let req = cluster_ops::realm_lifecycle::service::LifecycleRequest {
        request_id: "req-ac-007".to_string(),
        operator_id: "sre-1".to_string(),
        approval_ref: Some("approval-ac-007".to_string()),
        trace_id: "t-ac-007".to_string(),
        realm_id: "rlm-archive".to_string(),
        phase: LifecyclePhase::Archive,
    };
    let r = svc.archive(req).await;
    assert!(matches!(r, Err(cluster_ops::Error::Validation(_))));
}
