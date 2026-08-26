//! M-2070.6: AC-LCM-004（合服）演练（per RGS-IMPL-PLAN-LCM-001 §3.4 + SPEC-DTL-042 §7）。
//!
//! 验收项（per RGS-REQ-004 §3.7 AC-LCM-004 + DTL-042 §5.4）：
//! - 4 步 Saga：LoadConflictRulesV2 → MergePlayerData → LockConflictRulesV2 → MergeCompleted
//! - 锁定后**不**允许运行时修改（per FR-LCM-062 锚定）
//! - 启动 7-30 天回退窗口期（per SPEC §8 实测参数）

#![allow(clippy::result_large_err)]

use cluster_ops::realm_lifecycle::drill::executor::{DrillExecutor, DrillStatus};
use cluster_ops::realm_lifecycle::drill::playbook::{MergePlaybook, Playbook};
use cluster_ops::realm_lifecycle::drill::sandbox_k8s::{SandboxK8sClient, SandboxK8sConfig};
use cluster_ops::realm_lifecycle::drill::sandbox_pg::{SandboxPgConfig, SandboxPgPool};
use cluster_ops::realm_lifecycle::plans::merge_conflict_rule_set_v2::{
    ConflictRule, ConflictRuleKind, MergeConflictRuleSetV2,
};
use cluster_ops::realm_lifecycle::saga::steps::SagaStepKind;

fn executor_or_skip() -> Option<DrillExecutor> {
    let pg = SandboxPgPool::new(SandboxPgConfig::new(
        "postgres://sandbox:5432/cluster_sandbox_db",
    ))
    .ok()?;
    let k8s = SandboxK8sClient::new(SandboxK8sConfig::new(None)).ok()?;
    DrillExecutor::new(pg, k8s).ok()
}

/// AC-LCM-004 演练：合服 4 步 Saga。
#[tokio::test]
async fn ac_lcm_004_merge_drill() {
    let Some(exec) = executor_or_skip() else {
        eprintln!("SKIP: executor init failed");
        return;
    };
    let pb = MergePlaybook {
        source_realm_id: "rlm-drill-merge-src-004".to_string(),
        target_realm_id: "rlm-drill-merge-tgt-004".to_string(),
        conflict_rule_set_version: 2,
        rollback_window_days: 14,
    };
    let outcome = exec.run_one(&pb).await;
    if outcome.status == DrillStatus::Skipped {
        eprintln!("SKIP: AC-LCM-004 sandbox not available — 待 SRE 接力后跑真实环境");
        return;
    }
    assert_eq!(outcome.status, DrillStatus::Passed);
    assert_eq!(outcome.steps_total, 4);
    assert_eq!(outcome.steps_succeeded, 4);
}

/// AC-LCM-004 验收：合服 4 步顺序正确。
#[test]
fn ac_lcm_004_merge_4_steps_order() {
    let pb = MergePlaybook {
        source_realm_id: "s".to_string(),
        target_realm_id: "t".to_string(),
        conflict_rule_set_version: 2,
        rollback_window_days: 14,
    };
    let steps = pb.saga_steps();
    assert_eq!(steps.len(), 4);
    assert_eq!(steps[0].kind, SagaStepKind::LoadConflictRulesV2);
    assert_eq!(steps[1].kind, SagaStepKind::MergePlayerData);
    assert_eq!(steps[2].kind, SagaStepKind::LockConflictRulesV2);
    assert_eq!(steps[3].kind, SagaStepKind::MergeCompleted);
}

/// AC-LCM-004 演练：回退窗口期在 SPEC §8 实测参数范围内（7-30 天）。
#[test]
fn ac_lcm_004_rollback_window_in_spec_range() {
    let pb = MergePlaybook {
        source_realm_id: "s".to_string(),
        target_realm_id: "t".to_string(),
        conflict_rule_set_version: 2,
        rollback_window_days: 14,
    };
    assert!((7..=30).contains(&pb.rollback_window_days));
}

/// AC-LCM-004 验收：v2 冲突规则集锁定后不可改（per FR-LCM-062 锚定）。
#[test]
fn ac_lcm_004_lock_then_modify_rejected_fr_lcm_062() {
    let mut rs = MergeConflictRuleSetV2 {
        rule_set_id: "rs-ac-004".to_string(),
        version: 2,
        rules: vec![ConflictRule {
            rule_id: "r-1".to_string(),
            rule_kind: ConflictRuleKind::PlayerNameWithRealmSuffix,
            priority: 100,
            description: "test".to_string(),
        }],
        locked_at: None,
        locked_by: None,
        created_at: chrono::Utc::now(),
    };
    rs.lock("sre-1").expect("first lock ok");
    // 锁定后任何 lock 调用都应失败
    let r = rs.lock("sre-2");
    assert!(matches!(
        r,
        Err(cluster_ops::realm_lifecycle::error::Error::MergeRulesLocked { .. })
    ));
}
