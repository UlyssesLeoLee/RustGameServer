//! cluster-ops · realm_lifecycle · Saga 编排器集成测试（per M-2067.6 + `cargo test --test ut_saga`）
//!
//! 镜像 `src/realm_lifecycle/tests/ut_saga.rs` 的核心断言；通过 `pub` re-export 引用模块测试。
//! 设计：模块测试与集成测试并存，模块测试覆盖细节（type-level），集成测试覆盖 binary boundary。

use cluster_ops::realm_lifecycle::saga::orchestrator::LcmPhase;
use cluster_ops::realm_lifecycle::saga::steps::CompensateAction;

// 简单的烟雾测试：验证 6 阶段 + 1 子阶段 + 5 补偿动作 + 1 上下文字段
#[test]
fn smoke_test_6_phases_plus_1() {
    // 6 阶段 + MergeRollback 子阶段 = 7 phase
    let phases = [
        LcmPhase::NewRealm,
        LcmPhase::Scale,
        LcmPhase::Split,
        LcmPhase::Merge,
        LcmPhase::MergeRollback,
        LcmPhase::Retire,
        LcmPhase::Archive,
    ];
    assert_eq!(phases.len(), 7);
    for p in phases {
        let s = p.as_str();
        assert!(!s.is_empty());
        // 字符串与反向 round-trip 一致
        let back = LcmPhase::from_str(s);
        assert_eq!(back, Some(p));
    }
}

#[test]
fn smoke_test_5_compensate_actions() {
    let actions = [
        CompensateAction::Reverse,
        CompensateAction::Rollback,
        CompensateAction::Cleanup,
        CompensateAction::CrossDomainReverse,
        CompensateAction::CrossDomainRollback,
    ];
    let names = [
        "reverse",
        "rollback",
        "cleanup",
        "cross_domain_reverse",
        "cross_domain_rollback",
    ];
    for (a, n) in actions.iter().zip(names.iter()) {
        assert_eq!(a.as_str(), *n);
    }
}

#[test]
fn smoke_test_timeout_default_60s() {
    use cluster_ops::realm_lifecycle::saga::steps::SagaTimeoutConfig;
    let cfg = SagaTimeoutConfig::default();
    assert_eq!(cfg.secs, 60);
}

#[test]
fn smoke_test_idempotency_key_canonical() {
    use cluster_ops::realm_lifecycle::saga::idempotency::IdempotencyKey;
    use uuid::Uuid;
    let key = IdempotencyKey::new(Uuid::nil(), Uuid::nil());
    let canonical = key.canonical();
    assert!(canonical.starts_with("lcm:"));
}

#[test]
fn smoke_test_orchestrator_construction() {
    use std::sync::Arc;
    use economy_service::reservation::InMemoryReservationRepository;
    use economy_service::saga::InMemorySagaRepository;
    use cluster_ops::realm_lifecycle::saga::idempotency::InMemoryIdempotencyStore;
    use cluster_ops::realm_lifecycle::saga::orchestrator::SagaOrchestrator;

    let sagas: Arc<dyn economy_service::saga::SagaRepository> = Arc::new(InMemorySagaRepository::new());
    let reservations = Arc::new(InMemoryReservationRepository::new());
    let idem: Arc<dyn cluster_ops::realm_lifecycle::saga::idempotency::IdempotencyStore> =
        Arc::new(InMemoryIdempotencyStore::new());
    let handlers: Vec<Arc<dyn economy_service::saga_orchestrator::SagaStepHandler>> = vec![];

    let _orchestrator = SagaOrchestrator::new(sagas, reservations, idem, handlers);
    // 构造成功即验证类型 + 6 trait + 模块链路 OK
}
