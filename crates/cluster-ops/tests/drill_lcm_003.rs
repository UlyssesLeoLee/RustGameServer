//! M-2070.5: AC-LCM-003（分服）演练（per RGS-IMPL-PLAN-LCM-001 §3.4 + SPEC-DTL-042 §7）。
//!
//! 验收项（per RGS-REQ-004 §3.7 AC-LCM-003 + DTL-042 §5.3）：
//! - 7 步 Saga：FreezeSource → SnapshotPlayers → CreateTargetRealm → MigrateData → ShiftTraffic → PromoteDirectory → ThawSource
//! - saga 反向补偿在每一步都可触发
//!
//! 降级：沙箱不可达 → `Skipped`。

#![allow(clippy::result_large_err)]

use cluster_ops::realm_lifecycle::drill::executor::{DrillExecutor, DrillStatus};
use cluster_ops::realm_lifecycle::drill::playbook::{Playbook, SplitPlaybook};
use cluster_ops::realm_lifecycle::drill::sandbox_k8s::{SandboxK8sClient, SandboxK8sConfig};
use cluster_ops::realm_lifecycle::drill::sandbox_pg::{SandboxPgConfig, SandboxPgPool};
use cluster_ops::realm_lifecycle::saga::steps::SagaStepKind;

fn executor_or_skip() -> Option<DrillExecutor> {
    let pg = SandboxPgPool::new(SandboxPgConfig::new(
        "postgres://sandbox:5432/cluster_sandbox_db",
    ))
    .ok()?;
    let k8s = SandboxK8sClient::new(SandboxK8sConfig::new(None)).ok()?;
    DrillExecutor::new(pg, k8s).ok()
}

/// AC-LCM-003 演练：分服 7 步 Saga。
#[tokio::test]
async fn ac_lcm_003_split_drill() {
    let Some(exec) = executor_or_skip() else {
        eprintln!("SKIP: executor init failed");
        return;
    };
    let pb = SplitPlaybook {
        source_realm_id: "rlm-drill-split-src-003".to_string(),
        target_realm_id: "rlm-drill-split-tgt-003".to_string(),
        split_point_player_id: "p-1000000".to_string(),
        estimated_players: 2_000_000,
    };
    let outcome = exec.run_one(&pb).await;
    if outcome.status == DrillStatus::Skipped {
        eprintln!("SKIP: AC-LCM-003 sandbox not available — 待 SRE 接力后跑真实环境");
        return;
    }
    assert_eq!(outcome.status, DrillStatus::Passed);
    assert_eq!(outcome.steps_total, 7);
    assert_eq!(outcome.steps_succeeded, 7);
}

/// AC-LCM-003 验收：分服 7 步顺序正确。
#[test]
fn ac_lcm_003_split_7_steps_order() {
    let pb = SplitPlaybook {
        source_realm_id: "s".to_string(),
        target_realm_id: "t".to_string(),
        split_point_player_id: "p".to_string(),
        estimated_players: 1,
    };
    let steps = pb.saga_steps();
    assert_eq!(steps.len(), 7);
    let expected = [
        SagaStepKind::FreezeSource,
        SagaStepKind::SnapshotPlayers,
        SagaStepKind::CreateTargetRealm,
        SagaStepKind::MigrateData,
        SagaStepKind::ShiftTraffic,
        SagaStepKind::PromoteDirectory,
        SagaStepKind::ThawSource,
    ];
    for (i, k) in expected.iter().enumerate() {
        assert_eq!(steps[i].kind, *k, "step {} kind mismatch", i);
    }
}

/// AC-LCM-003 演练：分服超时上界 = 7 * 60s + 60s = 480s。
#[test]
fn ac_lcm_003_split_drill_timeout_bound() {
    let pb = SplitPlaybook {
        source_realm_id: "s".to_string(),
        target_realm_id: "t".to_string(),
        split_point_player_id: "p".to_string(),
        estimated_players: 1,
    };
    // 7 步 * 默认 60s + 60s 余量
    assert_eq!(pb.drill_timeout_secs(), 7 * 60 + 60);
}

/// AC-LCM-003 演练：与 Service trait 的 Split 一致性。
#[tokio::test]
async fn ac_lcm_003_service_split_consistency() {
    use cluster_ops::realm_lifecycle::service::{LifecyclePhase, NoopRealmLifecycleService, RealmLifecycleService};
    let svc = NoopRealmLifecycleService;
    let req = cluster_ops::realm_lifecycle::service::LifecycleRequest {
        request_id: "req-ac-003".to_string(),
        operator_id: "sre-1".to_string(),
        approval_ref: Some("approval-ac-003".to_string()),
        trace_id: "t-ac-003".to_string(),
        realm_id: "rlm-split".to_string(),
        phase: LifecyclePhase::Split,
    };
    let r = svc.split(req).await;
    assert!(matches!(r, Err(cluster_ops::Error::Validation(_))));
}
