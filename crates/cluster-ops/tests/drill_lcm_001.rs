//! M-2070.3: AC-LCM-001（开新服）演练（per RGS-IMPL-PLAN-LCM-001 §3.4 + SPEC-DTL-042 §7）。
//!
//! 验收项（per RGS-REQ-004 §3.7 AC-LCM-001 + DTL-042 §5.1）：
//! - 分配 realm_id（全局唯一）
//! - 初始化 realm_directory 路由条目（灰度 0%）
//! - admin_db.realm_lifecycle_run 写 run 记录
//! - PFAU 编排到 Active 状态
//!
//! 降级：沙箱不可达 → `#[ignore]` + 报告"待 SRE 接力后跑真实环境"。

#![allow(clippy::result_large_err)]

use cluster_ops::realm_lifecycle::drill::executor::{DrillExecutor, DrillStatus};
use cluster_ops::realm_lifecycle::drill::playbook::{NewRealmPlaybook, Playbook};
use cluster_ops::realm_lifecycle::drill::sandbox_k8s::{SandboxK8sClient, SandboxK8sConfig};
use cluster_ops::realm_lifecycle::drill::sandbox_pg::{SandboxPgConfig, SandboxPgPool};

/// 构造沙箱 executor（生产 URL 被拒绝，per FR-LCM-003）。
fn executor_or_skip() -> Option<DrillExecutor> {
    let pg = SandboxPgPool::new(SandboxPgConfig::new(
        "postgres://sandbox:5432/cluster_sandbox_db",
    ))
    .ok()?;
    let k8s = SandboxK8sClient::new(SandboxK8sConfig::new(None)).ok()?;
    DrillExecutor::new(pg, k8s).ok()
}

/// AC-LCM-001 演练：新服 3 步 Saga 跑通。
#[tokio::test]
async fn ac_lcm_001_new_realm_drill() {
    let Some(exec) = executor_or_skip() else {
        eprintln!("SKIP: executor init failed (FR-LCM-003 锚定：必须 sandbox URL)");
        return;
    };

    let pb = NewRealmPlaybook {
        realm_id: "rlm-drill-new-001".to_string(),
        region: "ap-east-1".to_string(),
        initial_node_count: 3,
    };

    // 5.1 验收：3 步 Saga 全 succeeded
    let steps = pb.saga_steps();
    assert_eq!(steps.len(), 3, "NewRealm must have 3 Saga steps");

    let outcome = exec.run_one(&pb).await;

    // 降级策略：沙箱不可达 → Skipped
    if outcome.status == DrillStatus::Skipped {
        eprintln!(
            "SKIP: AC-LCM-001 sandbox not available — 待 SRE 接力后跑真实环境。outcome = {:?}",
            outcome
        );
        return;
    }

    assert_eq!(
        outcome.status,
        DrillStatus::Passed,
        "AC-LCM-001 must pass when sandbox is available"
    );
    assert_eq!(outcome.steps_total, 3);
    assert_eq!(outcome.steps_succeeded, 3);
    assert_eq!(outcome.steps_failed, 0);
}

/// AC-LCM-001 验收：NewRealm 步骤序列必须包含 init_directory + write_run_record + pfau_activate。
#[tokio::test]
async fn ac_lcm_001_new_realm_saga_step_order() {
    let pb = NewRealmPlaybook {
        realm_id: "r".to_string(),
        region: "x".to_string(),
        initial_node_count: 1,
    };
    let steps = pb.saga_steps();
    use cluster_ops::realm_lifecycle::saga::steps::SagaStepKind;
    assert_eq!(steps[0].kind, SagaStepKind::InitDirectory);
    assert_eq!(steps[1].kind, SagaStepKind::WriteRunRecord);
    assert_eq!(steps[2].kind, SagaStepKind::PfauActivate);
}

/// AC-LCM-001 演练：与 Service trait 的 NewRealm 一致性。
#[tokio::test]
async fn ac_lcm_001_playbook_matches_service_new_realm_phase() {
    use cluster_ops::realm_lifecycle::service::{LifecyclePhase, NoopRealmLifecycleService, RealmLifecycleService};
    let svc = NoopRealmLifecycleService;
    let req = cluster_ops::realm_lifecycle::service::LifecycleRequest {
        request_id: "req-ac-001".to_string(),
        operator_id: "sre-1".to_string(),
        approval_ref: Some("approval-ac-001".to_string()),
        trace_id: "t-ac-001".to_string(),
        realm_id: "rlm-drill-new-001".to_string(),
        phase: LifecyclePhase::NewRealm,
    };
    // Noop 服务应返 Validation 错（具体实现由 WF-1-2066/2071 补齐；本测试仅锚定 trait 形状）
    let r = svc.new_realm(req).await;
    assert!(matches!(r, Err(cluster_ops::Error::Validation(_))));
}
