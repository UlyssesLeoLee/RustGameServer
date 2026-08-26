//! M-2071.6 UT：Feature 子类注册 100% 命中 + PFAU 5 状态合法转移
//! （per RGS-SPEC-DTL-042 §3 第 2 条 + §5 发布 + DTL-031 §1.1/§4.1/§5）

use std::collections::HashSet;

use cluster_ops::entity::{FeatureType, PfauState, SubFeature};
use cluster_ops::realm_lifecycle::feature_adapter::{
    FeatureRegistry, PfauTransition, RealmLifecycleFeatureAdapter,
};
use cluster_ops::realm_lifecycle::error::Error;
use uuid::Uuid;

// ============================================================================
// M-2071.6 验收：Feature 子类注册 100% 命中（per SPEC §3 第 2 条 + DTL-031 §5）
// ============================================================================

#[test]
fn feature_registry_contains_all_seven_sub_features() {
    let r = FeatureRegistry::with_default_seven();
    assert!(r.is_complete(), "7 子类必须全部注册");
    assert_eq!(r.list().len(), 7);
}

#[test]
fn feature_registry_seven_phase_names_distinct() {
    let r = FeatureRegistry::with_default_seven();
    let names: Vec<String> = r.list().iter().map(|x| x.phase_name.clone()).collect();
    let uniq: HashSet<String> = names.iter().cloned().collect();
    assert_eq!(uniq.len(), 7, "7 子类 phase_name 必须不重复");
}

#[test]
fn feature_registry_seven_phase_names_match_spec() {
    // per RGS-SPEC-DTL-042 §3 第 2 条：7 个子类
    let r = FeatureRegistry::with_default_seven();
    let names: HashSet<String> = r
        .list()
        .iter()
        .map(|x| x.phase_name.clone())
        .collect();
    for required in [
        "new_realm",
        "scale",
        "split",
        "merge",
        "merge_rollback",
        "retire",
        "archive",
    ] {
        assert!(
            names.contains(required),
            "missing required phase: {}",
            required
        );
    }
}

#[test]
fn feature_registry_each_uses_realm_lifecycle_feature_type() {
    // per DTL-031 §1.1：realm_lifecycle 5 大 Feature 类型之一
    let r = FeatureRegistry::with_default_seven();
    for reg in r.list() {
        assert_eq!(reg.feature_type, FeatureType::RealmLifecycle);
    }
}

#[test]
fn feature_registry_each_has_distinct_sub_feature_enum() {
    let r = FeatureRegistry::with_default_seven();
    let sfs: HashSet<SubFeature> = r.list().iter().map(|x| x.sub_feature).collect();
    assert_eq!(sfs.len(), 7, "7 SubFeature 枚举值必须不重复");
    assert!(sfs.contains(&SubFeature::NewRealm));
    assert!(sfs.contains(&SubFeature::Scale));
    assert!(sfs.contains(&SubFeature::Split));
    assert!(sfs.contains(&SubFeature::Merge));
    assert!(sfs.contains(&SubFeature::MergeRollback));
    assert!(sfs.contains(&SubFeature::Retire));
    assert!(sfs.contains(&SubFeature::Archive));
}

#[test]
fn feature_registry_initial_state_is_declared() {
    // per DTL-031 §4.1：所有 Feature 初始状态应为 declared
    let r = FeatureRegistry::with_default_seven();
    for reg in r.list() {
        assert_eq!(reg.current_state, PfauState::Declared);
        assert!(reg.enabled);
    }
}

#[test]
fn feature_registry_lookup_by_sub_feature() {
    let r = FeatureRegistry::with_default_seven();
    let found = r.find(SubFeature::MergeRollback).unwrap();
    assert_eq!(found.sub_feature, SubFeature::MergeRollback);
    assert_eq!(found.phase_name, "merge_rollback");
}

#[test]
fn feature_registry_lookup_by_phase_string() {
    let r = FeatureRegistry::with_default_seven();
    let found = r.find_by_phase("retire").unwrap();
    assert_eq!(found.sub_feature, SubFeature::Retire);
}

#[test]
fn feature_registry_lookup_unknown_phase_returns_none() {
    let r = FeatureRegistry::with_default_seven();
    assert!(r.find_by_phase("nonexistent").is_none());
}

// ============================================================================
// M-2071.3 验收：5 状态 PFAU 合法转移（per DTL-031 §4.1 硬约束）
// ============================================================================

#[test]
fn pfau_legal_transition_declared_to_active() {
    let a = RealmLifecycleFeatureAdapter::new();
    assert!(a
        .require_legal_transition(
            PfauState::Declared,
            PfauState::Active,
            SubFeature::NewRealm
        )
        .is_ok());
}

#[test]
fn pfau_legal_transition_active_to_upgrade_pending() {
    let a = RealmLifecycleFeatureAdapter::new();
    assert!(a
        .require_legal_transition(
            PfauState::Active,
            PfauState::UpgradePending,
            SubFeature::Scale
        )
        .is_ok());
}

#[test]
fn pfau_legal_transition_upgrade_pending_to_canary() {
    let a = RealmLifecycleFeatureAdapter::new();
    assert!(a
        .require_legal_transition(
            PfauState::UpgradePending,
            PfauState::CanaryInProgress,
            SubFeature::Split
        )
        .is_ok());
}

#[test]
fn pfau_legal_transition_paused_to_active() {
    let a = RealmLifecycleFeatureAdapter::new();
    assert!(a
        .require_legal_transition(
            PfauState::Paused,
            PfauState::Active,
            SubFeature::Retire
        )
        .is_ok());
}

#[test]
fn pfau_illegal_transition_paused_to_declared_rejected() {
    // per DTL-031 §4.1 第 166 行：非法跳转必须作为业务错误拒绝
    let a = RealmLifecycleFeatureAdapter::new();
    let err = a
        .require_legal_transition(
            PfauState::Paused,
            PfauState::Declared,
            SubFeature::Merge,
        )
        .unwrap_err();
    assert!(matches!(err, Error::PFAUIllegalTransition { .. }));
    let msg = err.to_string();
    assert!(msg.contains("paused"));
    assert!(msg.contains("declared"));
    assert!(msg.contains("merge"));
}

#[test]
fn pfau_illegal_transition_self_loop_rejected() {
    let a = RealmLifecycleFeatureAdapter::new();
    let err = a
        .require_legal_transition(
            PfauState::Active,
            PfauState::Active,
            SubFeature::Archive,
        )
        .unwrap_err();
    assert!(matches!(err, Error::PFAUIllegalTransition { .. }));
}

#[test]
fn pfau_illegal_transition_skip_states_rejected() {
    let a = RealmLifecycleFeatureAdapter::new();
    // Declared 直接跳到 CanaryInProgress 是非法的
    let err = a
        .require_legal_transition(
            PfauState::Declared,
            PfauState::CanaryInProgress,
            SubFeature::NewRealm,
        )
        .unwrap_err();
    assert!(matches!(err, Error::PFAUIllegalTransition { .. }));
}

#[test]
fn pfau_apply_transition_legal_ok() {
    let a = RealmLifecycleFeatureAdapter::new();
    let t = PfauTransition {
        from: PfauState::Declared,
        to: PfauState::Active,
        sub_feature: SubFeature::NewRealm,
        run_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        at: chrono::Utc::now(),
    };
    assert!(a.apply_transition(&t).is_ok());
}

#[test]
fn pfau_apply_transition_illegal_fails() {
    let a = RealmLifecycleFeatureAdapter::new();
    let t = PfauTransition {
        from: PfauState::Active,
        to: PfauState::Paused, // Active->Paused 不在 5 状态合法集中
        sub_feature: SubFeature::Split,
        run_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        at: chrono::Utc::now(),
    };
    assert!(a.apply_transition(&t).is_err());
}

// ============================================================================
// M-2071.2 验收：require_registered 接受 7 个 phase
// ============================================================================

#[test]
fn require_registered_accepts_seven_required_phases() {
    let a = RealmLifecycleFeatureAdapter::new();
    for p in [
        "new_realm",
        "scale",
        "split",
        "merge",
        "merge_rollback",
        "retire",
        "archive",
    ] {
        assert!(a.require_registered(p).is_ok(), "phase {} should be ok", p);
    }
}

#[test]
fn require_registered_rejects_unknown_phase() {
    let a = RealmLifecycleFeatureAdapter::new();
    let err = a.require_registered("not_a_phase").unwrap_err();
    assert_eq!(err, "not_a_phase");
}
