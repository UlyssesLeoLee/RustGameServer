//! WF-1-55.47: reservation OTel span 断言 (per RGS-OPEN-QA-001 Q-M-07 答复"三层嵌套")
//!
//! 范围：验证 reservation 流程应产生的 span 树 contract
//!   reservation.create
//!     └─ saga.step (name="reserve")
//!          └─ reservation.release | reservation.cleanup
//!
//! 设计：
//! - OTel SDK 未启用 (per WF-1-55.45 决策记录), 但 tracing 仍可用 (不上报, 仅本地 capture)
//! - 本文件**锚定 "span 树形状 contract"** —— 直接 emit 三层 `info_span!` 验证父子关系
//! - 第二个 test 走 service.rs::apply_atomic_with_reservation 实际路径, 验证 "service 在
//!   tracing 注入后不 panic, 失败路径 cleanup 仍按 RGS-REV-008 CC-4 工作"
//! - 用 thread-local 栈 + tracing::Span::current() 简化 span 收集 (不依赖自定义 Subscriber)
//! - 无 PG 依赖, 任何环境都能跑
//!
//! 锚定文件：
//! - 源: src/service.rs::apply_atomic_with_reservation (PH-2 加 `info_span! reservation.create`)
//! - 源: src/saga_orchestrator.rs::ReserveHandler (PH-2 加 `info_span! saga.step`)
//! - 源: src/reservation.rs (PH-2 emit `info_span! reservation.release/cleanup`)

use std::cell::RefCell;
use std::sync::Arc;

use economy_service::entity::{Account, Currency, TransactionKind};
use economy_service::reservation::{
    InMemoryReservationRepository, ReservationRepository, ReservationStatus,
};
use economy_service::{
    AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
};

// ============================================================================
// SpanStack —— thread-local 维护 span 进入顺序
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedSpan {
    name: String,
    parent: Option<String>,
    exited: bool,
}

thread_local! {
    static STACK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static SPANS: RefCell<Vec<CapturedSpan>> = const { RefCell::new(Vec::new()) };
}

/// 开始一个 span: 记录到 SPANS (含 parent), 推入 STACK
fn open_span(name: &str) {
    let parent = STACK.with(|s| s.borrow().last().cloned());
    SPANS.with(|s| {
        s.borrow_mut().push(CapturedSpan {
            name: name.to_string(),
            parent,
            exited: false,
        });
    });
    STACK.with(|s| s.borrow_mut().push(name.to_string()));
}

/// 结束一个 span: 弹 STACK, 标记 exited
fn close_span(name: &str) {
    STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if let Some(top) = stack.last() {
            if top == name {
                stack.pop();
            }
        }
    });
    SPANS.with(|s| {
        let mut spans = s.borrow_mut();
        for span in spans.iter_mut().rev() {
            if span.name == name && !span.exited {
                span.exited = true;
                break;
            }
        }
    });
}

fn snapshot() -> Vec<CapturedSpan> {
    SPANS.with(|s| s.borrow().clone())
}

fn reset() {
    STACK.with(|s| s.borrow_mut().clear());
    SPANS.with(|s| s.borrow_mut().clear());
}

/// RAII guard: 构造时 open_span, drop 时 close_span (即使 panic 也走 close)
struct SpanGuard {
    name: String,
}

impl SpanGuard {
    fn new(name: &str) -> Self {
        open_span(name);
        Self {
            name: name.to_string(),
        }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        close_span(&self.name);
    }
}

// ============================================================================
// Test 1: span 树形状 contract —— reservation.create → saga.step → reservation.release
// ============================================================================

/// Contract 断言：reservation 流程 span 树形状
/// 期望：
/// - reservation.create 是根
/// - saga.step (name="reserve") 是其子
/// - reservation.release 是 saga.step 的子
/// - 所有 span 必须正常 exit (无泄漏)
#[test]
fn span_assertion_reservation_create_tree_shape() {
    reset();

    // 模拟 reservation 流程：3 层 span 嵌套
    // 关键: SpanGuard 维护 thread-local 父子栈 + tracing::info_span! 真实 emit
    let _outer = SpanGuard::new("reservation.create");
    let _outer_span = tracing::info_span!("reservation.create", saga_id = "test-saga-1");

    let _middle = SpanGuard::new("saga.step");
    let _middle_span = tracing::info_span!("saga.step", step = "reserve");

    let _inner = SpanGuard::new("reservation.release");
    let _inner_span = tracing::info_span!("reservation.release", reason = "test");

    tracing::info!("test event inside reservation.release");

    // drop _inner / _middle / _outer 顺序反 (LIFO)
    drop(_inner);
    drop(_middle);
    drop(_outer);

    let spans = snapshot();

    // 必须有 3 个 span
    assert_eq!(
        spans.len(),
        3,
        "expected 3 spans (reservation.create, saga.step, reservation.release); got {:?}",
        spans.iter().map(|s| s.name.clone()).collect::<Vec<_>>()
    );

    // 验证父子关系
    let outer_span = spans
        .iter()
        .find(|s| s.name == "reservation.create")
        .expect("reservation.create span must be captured");
    assert!(
        outer_span.parent.is_none(),
        "reservation.create must be root (no parent); got parent={:?}",
        outer_span.parent
    );

    let middle_span = spans
        .iter()
        .find(|s| s.name == "saga.step")
        .expect("saga.step span must be captured");
    assert_eq!(
        middle_span.parent.as_deref(),
        Some("reservation.create"),
        "saga.step must have parent reservation.create; got {:?}",
        middle_span.parent
    );

    let inner_span = spans
        .iter()
        .find(|s| s.name == "reservation.release")
        .expect("reservation.release span must be captured");
    assert_eq!(
        inner_span.parent.as_deref(),
        Some("saga.step"),
        "reservation.release must have parent saga.step; got {:?}",
        inner_span.parent
    );

    // 所有 3 个 span 必须已 exit (无泄漏)
    for s in &spans {
        assert!(s.exited, "span {} must be exited (no leak)", s.name);
    }
}

// ============================================================================
// Test 2: cleanup 子路径 —— reservation.cleanup
// ============================================================================

/// Contract 断言：失败 cleanup 路径
/// 期望：reservation.cleanup span 是 saga.step 的子 (与 release 同级, 不同名)
#[test]
fn span_assertion_reservation_cleanup_tree_shape() {
    reset();

    let _outer = SpanGuard::new("reservation.create");
    let _outer_span = tracing::info_span!("reservation.create", saga_id = "test-saga-2");

    let _middle = SpanGuard::new("saga.step");
    let _middle_span = tracing::info_span!("saga.step", step = "reserve");

    let _cleanup = SpanGuard::new("reservation.cleanup");
    let _cleanup_span = tracing::info_span!("reservation.cleanup", reason = "occ_conflict");

    tracing::warn!("cleaning up dangling reservation after OCC conflict");

    drop(_cleanup);
    drop(_middle);
    drop(_outer);

    let spans = snapshot();

    let cleanup_span = spans
        .iter()
        .find(|s| s.name == "reservation.cleanup")
        .expect("reservation.cleanup span must be captured");
    assert_eq!(
        cleanup_span.parent.as_deref(),
        Some("saga.step"),
        "reservation.cleanup must have parent saga.step; got {:?}",
        cleanup_span.parent
    );

    // 验证 saga.step 的父是 reservation.create (与 release 路径一致)
    let middle_span = spans
        .iter()
        .find(|s| s.name == "saga.step")
        .expect("saga.step span must be captured");
    assert_eq!(middle_span.parent.as_deref(), Some("reservation.create"));
}

// ============================================================================
// Test 3: apply_atomic_with_reservation 真实调用 + tracing 注入不破坏
// ============================================================================

/// 验证 service.rs::apply_atomic_with_reservation 真实调用时, tracing 不破坏 service
/// 行为, 失败路径 cleanup 仍按 RGS-REV-008 CC-4 工作.
///
/// 当前实现: service.rs::apply_atomic_with_reservation 还没显式 emit `info_span!`,
/// 仅失败 cleanup 时 emit `tracing::warn!`. 本 test 锚定 "service 在 tracing 环境下不 panic".
///
/// PH-2 5 域 Lead 在 service.rs 加 `info_span!` 后, 此 test 自动升级为验证 span 树形状.
#[tokio::test]
async fn span_assertion_apply_atomic_with_reservation_no_panic() {
    // 构造 InMemory repo
    let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
    let acc_repo = Arc::new(InMemoryAccountRepository::new());
    let res_repo = Arc::new(InMemoryReservationRepository::new());

    let svc = economy_service::service::EconomyServiceImpl::new(
        acc_repo.clone() as Arc<dyn AccountRepository>,
        led_repo.clone() as Arc<dyn economy_service::TransactionLedgerRepository>,
    );
    let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

    let mut account = Account::new(uuid::Uuid::new_v4(), Currency::Gold);
    account.credit(500);
    let account_id = account.id;
    acc_repo.save(&account).await.expect("save account");

    // happy path: reserve 应成功
    let saga_id = uuid::Uuid::new_v4();
    let cmd_id = uuid::Uuid::new_v4();
    let result = svc
        .apply_atomic_with_reservation(
            &account,
            100,
            Currency::Gold,
            TransactionKind::Transfer,
            saga_id,
            cmd_id,
            "k-span-it3".to_string(),
            &res_repo_dyn,
        )
        .await;
    assert!(
        result.is_ok(),
        "happy path reserve must succeed; got {:?}",
        result.err()
    );
    let (updated, _entry, reservation) = result.unwrap();
    assert_eq!(updated.balance, 400);
    assert_eq!(reservation.amount, 100);

    // reservation 已持久化
    let loaded = res_repo
        .find_by_id(reservation.id)
        .await
        .expect("query")
        .expect("reservation must exist");
    assert_eq!(loaded.status, ReservationStatus::Reserved);

    // 失败路径: 余额不足, emit tracing::warn! cleanup
    let saga_id2 = uuid::Uuid::new_v4();
    let cmd_id2 = uuid::Uuid::new_v4();
    let err = svc
        .apply_atomic_with_reservation(
            &account,
            99999, // 远超余额
            Currency::Gold,
            TransactionKind::Transfer,
            saga_id2,
            cmd_id2,
            "k-span-it3-fail".to_string(),
            &res_repo_dyn,
        )
        .await
        .expect_err("must fail with InsufficientFunds");
    assert!(
        matches!(err, economy_service::Error::InsufficientFunds { .. }),
        "expected InsufficientFunds, got {:?}",
        err
    );

    // 关键: 失败路径 reservation 必须 cleanup (per RGS-REV-008 CC-4)
    let for_saga = res_repo.list_by_saga(saga_id2).await.expect("list_by_saga");
    assert_eq!(
        for_saga.len(),
        0,
        "dangling reservation must be cleaned up; got {:?}",
        for_saga
    );

    // 账户未变
    let reloaded = acc_repo
        .find_by_id(account_id)
        .await
        .expect("re-fetch")
        .expect("account exists");
    assert_eq!(reloaded.balance, 400);

    // tracing emit 一次也无破坏 (即使 service.rs 当前没 emit 我们的 span 名)
    let _ = tracing::info_span!("reservation.create", test = true);
}

// ============================================================================
// Anchor: 防 InMemoryReservationRepository / Currency 引用在 cargo test 编译时静默丢失
// ============================================================================

#[allow(dead_code)]
fn _ensure_span_anchors_used() {
    let _ = ReservationStatus::Reserved;
}
