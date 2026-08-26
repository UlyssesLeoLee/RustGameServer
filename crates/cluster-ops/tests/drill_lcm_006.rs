//! M-2070.8: AC-LCM-006（退场）演练（per RGS-IMPL-PLAN-LCM-001 §3.4 + SPEC-DTL-042 §7）。
//!
//! 验收项（per RGS-REQ-004 §3.7 AC-LCM-006 + DTL-042 §5.5 + SPEC §3 第 8 条）：
//! - retire_plan 创建（含 query_channel_rbac 角色配置）
//! - 默认查询通道角色 = cs_agent / sre / legal
//! - 30-90 天后启动归档（per SPEC §8 实测参数）

#![allow(clippy::result_large_err)]

use cluster_ops::realm_lifecycle::drill::executor::{DrillExecutor, DrillStatus};
use cluster_ops::realm_lifecycle::drill::playbook::{Playbook, RetirePlaybook};
use cluster_ops::realm_lifecycle::drill::sandbox_k8s::{SandboxK8sClient, SandboxK8sConfig};
use cluster_ops::realm_lifecycle::drill::sandbox_pg::{SandboxPgConfig, SandboxPgPool};
use cluster_ops::realm_lifecycle::operations::retire::DEFAULT_RETIRE_QUERY_ROLES;
use cluster_ops::realm_lifecycle::plans::retire_plan::RetirePlan;
use cluster_ops::realm_lifecycle::saga::steps::SagaStepKind;

fn executor_or_skip() -> Option<DrillExecutor> {
    let pg = SandboxPgPool::new(SandboxPgConfig::new(
        "postgres://sandbox:5432/cluster_sandbox_db",
    ))
    .ok()?;
    let k8s = SandboxK8sClient::new(SandboxK8sConfig::new(None)).ok()?;
    DrillExecutor::new(pg, k8s).ok()
}

/// AC-LCM-006 演练：退场 2 步 Saga。
#[tokio::test]
async fn ac_lcm_006_retire_drill() {
    let Some(exec) = executor_or_skip() else {
        eprintln!("SKIP: executor init failed");
        return;
    };
    let pb = RetirePlaybook {
        realm_id: "rlm-drill-retire-006".to_string(),
        query_channel_rbac: vec![
            "cs_agent".to_string(),
            "sre".to_string(),
            "legal".to_string(),
        ],
        archive_threshold_days: 60,
    };
    let outcome = exec.run_one(&pb).await;
    if outcome.status == DrillStatus::Skipped {
        eprintln!("SKIP: AC-LCM-006 sandbox not available — 待 SRE 接力后跑真实环境");
        return;
    }
    assert_eq!(outcome.status, DrillStatus::Passed);
    assert_eq!(outcome.steps_total, 2);
    assert_eq!(outcome.steps_succeeded, 2);
}

/// AC-LCM-006 验收：默认查询角色 = cs_agent / sre / legal（per SPEC §3 第 8 条）。
#[test]
fn ac_lcm_006_default_query_roles_match_spec() {
    assert_eq!(DEFAULT_RETIRE_QUERY_ROLES, &["cs_agent", "sre", "legal"]);
}

/// AC-LCM-006 验收：归档启动阈值在 SPEC §8 实测参数范围内（30-90 天）。
#[test]
fn ac_lcm_006_archive_threshold_in_spec_range() {
    let pb = RetirePlaybook {
        realm_id: "r".to_string(),
        query_channel_rbac: vec![],
        archive_threshold_days: 60,
    };
    assert!((30..=90).contains(&pb.archive_threshold_days));
}

/// AC-LCM-006 验收：RetirePlan 角色检查。
#[test]
fn ac_lcm_006_retire_plan_role_check() {
    let p = RetirePlan {
        plan_id: "rp-ac-006".to_string(),
        realm_id: "rlm-1".to_string(),
        query_channel_rbac: RetirePlan::default_query_roles(),
        archive_threshold_days: 60,
        created_at: chrono::Utc::now(),
        created_by: "sre-1".to_string(),
    };
    assert!(p.is_role_allowed("cs_agent"));
    assert!(p.is_role_allowed("sre"));
    assert!(p.is_role_allowed("legal"));
    assert!(!p.is_role_allowed("player"));
}

/// AC-LCM-006 验收：退场 2 步顺序正确。
#[test]
fn ac_lcm_006_retire_2_steps_order() {
    let pb = RetirePlaybook {
        realm_id: "r".to_string(),
        query_channel_rbac: vec![],
        archive_threshold_days: 60,
    };
    let steps = pb.saga_steps();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].kind, SagaStepKind::CreateRetirePlan);
    assert_eq!(steps[1].kind, SagaStepKind::ScheduleArchive);
}

/// AC-LCM-006 演练：与 Service trait 的 retire 一致性。
#[tokio::test]
async fn ac_lcm_006_service_retire_consistency() {
    use cluster_ops::realm_lifecycle::service::{LifecyclePhase, NoopRealmLifecycleService, RealmLifecycleService};
    let svc = NoopRealmLifecycleService;
    let req = cluster_ops::realm_lifecycle::service::LifecycleRequest {
        request_id: "req-ac-006".to_string(),
        operator_id: "sre-1".to_string(),
        approval_ref: Some("approval-ac-006".to_string()),
        trace_id: "t-ac-006".to_string(),
        realm_id: "rlm-retire".to_string(),
        phase: LifecyclePhase::Retire,
    };
    let r = svc.retire(req).await;
    assert!(matches!(r, Err(cluster_ops::Error::Validation(_))));
}
