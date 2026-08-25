//! 6 阶段状态机 + 非法跳转 + 二次激活负例（per M-2066.10 验收门槛）
//!
//! WF-1-2066 M-2066.10 验证 —— 集成测试入口（per 任务验收命令
//! `cargo test -p rgs-cluster-ops --test ut_state_machine`）
//!
//! 注：本文件与 `src/realm_lifecycle/tests/ut_state_machine.rs` 内容对齐
//! （同一份测试的"双位置"形式）：
//! - `src/.../tests/ut_state_machine.rs`  → 任务规定的源文件路径（per 验收门槛
//!   `Test-Path crates/cluster-ops/src/realm_lifecycle/tests/ut_state_machine.rs`）
//! - `tests/ut_state_machine.rs`           → 满足 `cargo test --test ut_state_machine`
//!   命令格式（Cargo 集成测试发现机制要求 tests/ 目录）
//!
//! 覆盖（per RGS-DTL-042 §4 + SPEC-DTL-042 §3 第 6 条）：
//! 1. 6 阶段状态机主路径（NewRealm → Scale → Split → Merge → Retire → Archive）
//! 2. 非法跳转负例（NewRealm → Archive 跳过 / Scale → NewRealm 倒退 / Merge → Split 倒退）
//! 3. 二次激活负例（Archive → NewRealm 显式返回 `AlreadyActivated`）
//! 4. 合服回退窗口期（Merge → Merge 自转移合法，per TBD-DTL-042 7~30 天）
//! 5. 终态唯一性（Archive 是唯一终态，per FR-LCM-081）
//! 6. 6 操作器 trait 全部实现 + 各自至少 1 个 `async fn` 方法

use std::str::FromStr;
use std::sync::Arc;

use cluster_ops::realm_lifecycle::error::LcmErrorKind;
use cluster_ops::realm_lifecycle::operations::{
    ArchiveOperator, MergeOperator, MergeRollbackOperator, NewRealmOperator, RetireOperator,
    ScaleOperator, SplitOperator,
};
use cluster_ops::realm_lifecycle::{
    RealmLifecycleOperator, RealmLifecycleService, RealmLifecycleServiceImpl, RealmLifecycleStage,
    RealmLifecycleStateMachine,
};
use uuid::Uuid;

// ============================================================================
// 1. 6 阶段状态机主路径测试
// ============================================================================

#[test]
fn ut_state_machine_full_lifecycle_walks_through_6_stages() {
    let sm = RealmLifecycleStateMachine::new("realm-001");
    let req = Uuid::new_v4();
    assert_eq!(sm.current(), RealmLifecycleStage::NewRealm);

    sm.transition(RealmLifecycleStage::Scale, req).unwrap();
    assert_eq!(sm.current(), RealmLifecycleStage::Scale);

    sm.transition(RealmLifecycleStage::Split, req).unwrap();
    assert_eq!(sm.current(), RealmLifecycleStage::Split);

    sm.transition(RealmLifecycleStage::Merge, req).unwrap();
    assert_eq!(sm.current(), RealmLifecycleStage::Merge);

    sm.transition(RealmLifecycleStage::Retire, req).unwrap();
    assert_eq!(sm.current(), RealmLifecycleStage::Retire);

    sm.transition(RealmLifecycleStage::Archive, req).unwrap();
    assert_eq!(sm.current(), RealmLifecycleStage::Archive);
}

#[test]
fn ut_state_machine_restore_preserves_stage() {
    let sm = RealmLifecycleStateMachine::restore("realm-002", RealmLifecycleStage::Split);
    assert_eq!(sm.current(), RealmLifecycleStage::Split);
    assert_eq!(sm.realm_id(), "realm-002");
}

#[test]
fn ut_state_machine_merge_self_transition_is_legal() {
    let sm = RealmLifecycleStateMachine::restore("realm-merge", RealmLifecycleStage::Merge);
    let req = Uuid::new_v4();
    let result = sm.transition(RealmLifecycleStage::Merge, req);
    assert!(result.is_ok());
    assert_eq!(sm.current(), RealmLifecycleStage::Merge);
}

// ============================================================================
// 2. 非法跳转负例测试
// ============================================================================

#[test]
fn ut_state_machine_rejects_skip_middle_stages_to_archive() {
    let sm = RealmLifecycleStateMachine::new("realm-skip-1");
    let req = Uuid::new_v4();
    let err = sm
        .transition(RealmLifecycleStage::Archive, req)
        .unwrap_err();
    match err.kind {
        LcmErrorKind::InvalidStageTransition { from, to, .. } => {
            assert_eq!(from, "new_realm");
            assert_eq!(to, "archive");
        }
        other => panic!("expected InvalidStageTransition, got {other:?}"),
    }
}

#[test]
fn ut_state_machine_rejects_backward_transitions() {
    let cases = [
        (RealmLifecycleStage::Scale, RealmLifecycleStage::NewRealm),
        (RealmLifecycleStage::Merge, RealmLifecycleStage::Split),
        (RealmLifecycleStage::Retire, RealmLifecycleStage::Merge),
    ];
    for (from, to) in cases {
        let sm = RealmLifecycleStateMachine::restore("realm-backward", from);
        let req = Uuid::new_v4();
        let err = sm.transition(to, req).unwrap_err();
        assert!(
            matches!(err.kind, LcmErrorKind::InvalidStageTransition { .. }),
            "transition {:?} → {:?} should be rejected",
            from,
            to
        );
    }
}

#[test]
fn ut_state_machine_rejects_archive_terminal_to_anything() {
    let sm = RealmLifecycleStateMachine::restore("realm-final", RealmLifecycleStage::Archive);
    let req = Uuid::new_v4();
    for next in [
        RealmLifecycleStage::Scale,
        RealmLifecycleStage::Split,
        RealmLifecycleStage::Merge,
        RealmLifecycleStage::Retire,
    ] {
        let err = sm.transition(next, req).unwrap_err();
        assert!(matches!(err.kind, LcmErrorKind::InvalidStageTransition { .. }));
    }
}

// ============================================================================
// 3. 二次激活负例测试（per M-2066.10 验收门槛 + SPEC §3 第 6 条）
// ============================================================================

#[test]
fn ut_state_machine_duplicate_activation_returns_already_activated() {
    let sm = RealmLifecycleStateMachine::restore("realm-007", RealmLifecycleStage::Archive);
    let req = Uuid::new_v4();
    let err = sm
        .transition(RealmLifecycleStage::NewRealm, req)
        .unwrap_err();
    match err.kind {
        LcmErrorKind::AlreadyActivated { realm_id } => {
            assert_eq!(realm_id, "realm-007");
        }
        other => panic!("expected AlreadyActivated, got {other:?}"),
    }
    assert_eq!(sm.current(), RealmLifecycleStage::Archive);
}

#[test]
fn ut_state_machine_archive_is_only_terminal_stage() {
    assert!(RealmLifecycleStage::Archive.is_terminal());
    assert!(!RealmLifecycleStage::NewRealm.is_terminal());
    assert!(!RealmLifecycleStage::Scale.is_terminal());
    assert!(!RealmLifecycleStage::Split.is_terminal());
    assert!(!RealmLifecycleStage::Merge.is_terminal());
    assert!(!RealmLifecycleStage::Retire.is_terminal());
}

// ============================================================================
// 4. FromStr 解析 + 6 阶段覆盖
// ============================================================================

#[test]
fn ut_state_machine_from_str_round_trip() {
    let cases = [
        ("new_realm", RealmLifecycleStage::NewRealm),
        ("scale", RealmLifecycleStage::Scale),
        ("split", RealmLifecycleStage::Split),
        ("merge", RealmLifecycleStage::Merge),
        ("retire", RealmLifecycleStage::Retire),
        ("archive", RealmLifecycleStage::Archive),
    ];
    for (s, expected) in cases {
        let stage = RealmLifecycleStage::from_str(s).unwrap();
        assert_eq!(stage, expected);
        assert_eq!(stage.as_str(), s);
    }
}

// ============================================================================
// 5. 6 操作器 trait 实现 + 二次激活负例综合场景
// ============================================================================

#[test]
fn ut_all_six_operators_implement_trait() {
    let ops: Vec<(&'static str, Arc<dyn RealmLifecycleOperator>)> = vec![
        ("new_realm", Arc::new(NewRealmOperator::new_for_skeleton())),
        ("scale", Arc::new(ScaleOperator::new_for_skeleton())),
        ("split", Arc::new(SplitOperator::new_for_skeleton())),
        ("merge", Arc::new(MergeOperator::new_for_skeleton())),
        (
            "merge_rollback",
            Arc::new(MergeRollbackOperator::new_for_skeleton()),
        ),
        ("retire", Arc::new(RetireOperator::new_for_skeleton())),
        ("archive", Arc::new(ArchiveOperator::new_for_skeleton())),
    ];
    assert_eq!(ops.len(), 7);
    for (name, op) in ops {
        assert!(!op.name().is_empty());
        assert!(op.name() == name);
    }
}

#[tokio::test]
async fn ut_each_operator_has_at_least_one_async_method_execute() {
    let req = Uuid::new_v4();
    let realm_id = "realm-ut-001";
    let operator_id = Uuid::new_v4();

    let new_realm = NewRealmOperator::new_for_skeleton();
    let scale = ScaleOperator::new_for_skeleton();
    let split = SplitOperator::new_for_skeleton();
    let merge = MergeOperator::new_for_skeleton();
    let merge_rollback = MergeRollbackOperator::new_for_skeleton();
    let retire = RetireOperator::new_for_skeleton();
    let archive = ArchiveOperator::new_for_skeleton();

    for (op, expected_name) in [
        (&new_realm as &dyn RealmLifecycleOperator, "new_realm"),
        (&scale, "scale"),
        (&split, "split"),
        (&merge, "merge"),
        (&merge_rollback, "merge_rollback"),
        (&retire, "retire"),
        (&archive, "archive"),
    ] {
        assert_eq!(op.name(), expected_name);
        let err = op
            .execute(req, realm_id, operator_id, None)
            .await
            .expect_err("skeleton operator should return NotImplemented");
        assert!(
            matches!(err.kind, LcmErrorKind::NotImplemented { .. }),
            "{}: expected NotImplemented, got {:?}",
            expected_name,
            err.kind
        );
    }
}

#[test]
fn ut_full_lifecycle_with_duplicate_activation_blocked() {
    let sm = RealmLifecycleStateMachine::new("realm-full-001");
    let req = Uuid::new_v4();
    for next in [
        RealmLifecycleStage::Scale,
        RealmLifecycleStage::Split,
        RealmLifecycleStage::Merge,
        RealmLifecycleStage::Retire,
        RealmLifecycleStage::Archive,
    ] {
        sm.transition(next, req).unwrap();
    }
    assert_eq!(sm.current(), RealmLifecycleStage::Archive);

    let err = sm
        .transition(RealmLifecycleStage::NewRealm, req)
        .unwrap_err();
    assert!(matches!(err.kind, LcmErrorKind::AlreadyActivated { .. }));
    assert_eq!(sm.current(), RealmLifecycleStage::Archive);
}

// ============================================================================
// 6. RealmLifecycleService 门面 6 阶段方法
// ============================================================================

fn build_service() -> RealmLifecycleServiceImpl {
    RealmLifecycleServiceImpl::new(
        Arc::new(NewRealmOperator::new_for_skeleton()),
        Arc::new(ScaleOperator::new_for_skeleton()),
        Arc::new(SplitOperator::new_for_skeleton()),
        Arc::new(MergeOperator::new_for_skeleton()),
        Arc::new(RetireOperator::new_for_skeleton()),
        Arc::new(ArchiveOperator::new_for_skeleton()),
    )
}

#[tokio::test]
async fn ut_service_health_check_returns_true() {
    let svc = build_service();
    assert!(svc.health_check().await.unwrap());
}

#[tokio::test]
async fn ut_service_scale_rejects_zero_target_capacity() {
    let svc = build_service();
    let err = svc
        .scale(Uuid::new_v4(), "realm-001", 0, Uuid::new_v4(), None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("target_capacity must be > 0"));
}

#[tokio::test]
async fn ut_service_split_rejects_empty_target_realm_ids() {
    let svc = build_service();
    let err = svc
        .split(Uuid::new_v4(), "realm-001", &[], Uuid::new_v4(), None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("target_realm_ids must not be empty"));
}

#[tokio::test]
async fn ut_service_merge_rejects_single_source_realm() {
    let svc = build_service();
    let err = svc
        .merge(
            Uuid::new_v4(),
            &["realm-src-1".to_string()],
            "realm-tgt",
            Uuid::new_v4(),
            None,
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("at least 2 source_realm_ids"));
}
