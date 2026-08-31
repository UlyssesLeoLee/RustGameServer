//! RGS-UT 2026-08-31 JST — economy 域 IT (最高规格) / 子桶 1
//!
//! `integration_reservation_under_concurrent_saga`
//!
//! 场景: 同一账户同一货币并发 2 个 saga 预约, 模拟"资源争抢"边界
//!       1 个成功 + 1 个失败 (OCC 冲突 / 余额不足路径)
//!
//! 设计目标 (per RGS-UT 2026-08-31 13:55 JST 指令 + IT-AGENT-BRIEFING §3.2 #1):
//! - 用 `tokio::join!` 真并发跑 2 个 saga orchestrator
//! - 共享同一 `InMemoryAccountRepository` / `InMemoryReservationRepository` /
//!   `InMemoryTransactionLedgerRepository` (per RGS-REV-007 AC3 共享 ledger HashMap)
//! - 不连真 DB, 不起真实 gRPC server (per IT-AGENT-BRIEFING §4)
//!
//! 覆盖 4 个 case:
//! 1. `concurrent_two_sagas_one_account_one_wins`:
//!    账户 balance=100, 2 个 saga 各 reserve 80. 预期 1 成功 1 OCC 冲突.
//!    最终 balance=20, 1 个 reservation 保留 (Confirmed/Reserved), 1 个已 cleanup.
//! 2. `concurrent_two_sagas_same_account_both_lose_balance`:
//!    账户 balance=50, 2 个 saga 各 reserve 80. 预期 2 个都因 InsufficientFunds 失败.
//!    最终 balance=50, 0 个 reservation (失败路径上 dangling reservation 被清理).
//! 3. `concurrent_two_sagas_distinct_accounts_both_win`:
//!    2 个不同账户各 balance=100, 各自 reserve 80. 预期 2 个都成功.
//!    最终 2 个 balance 各 20, 2 个 reservation.
//! 4. `concurrent_two_sagas_same_player_distinct_currency`:
//!    同一 player 2 个 currency 账户, 各 balance=100, 各自 reserve 80.
//!    预期 2 个都成功 (currency 维度独立, 不冲突).
//!
//! 锚定文件:
//! - 源: src/saga_orchestrator.rs (ReserveHandler.apply_atomic OCC + 失败清理路径)
//! - 源: src/reservation.rs (Reservation::release 语义)
//! - 设计: per RGS-DTL-100 §3.2 资源争抢 + RGS-REV-009 V1 LO-4 / CR-1 retry 真修
//!
//! mTLS 验证: 业务层与传输层解耦, Mock 客户端不涉及 TLS, 真实 gRPC 客户端在
//!            CardGrpcClient::new() 处强制 mTLS (per BAS-003 fail-closed).

use std::sync::Arc;
use uuid::Uuid;

use economy_service::entity::{Account, Currency};
use economy_service::repository::{
    AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
    TransactionLedgerRepository,
};
use economy_service::reservation::{InMemoryReservationRepository, ReservationRepository};
use economy_service::saga::{InMemorySagaRepository, Saga, SagaRepository, SagaType};
use economy_service::saga_orchestrator::{ReserveHandler, SagaOrchestrator, SagaStepHandler};

// ============================================================================
// 测试装配套件
// ============================================================================

struct ConcurrencyEnv {
    accounts: Arc<InMemoryAccountRepository>,
    reservations: Arc<InMemoryReservationRepository>,
    ledger: Arc<InMemoryTransactionLedgerRepository>,
    sagas: Arc<InMemorySagaRepository>,
}

fn bootstrap() -> ConcurrencyEnv {
    let led = Arc::new(InMemoryTransactionLedgerRepository::new());
    let acc = Arc::new(
        InMemoryAccountRepository::new().with_shared_ledger(led.inner.clone()),
    );
    let res = Arc::new(InMemoryReservationRepository::new());
    let sag = Arc::new(InMemorySagaRepository::new());
    ConcurrencyEnv {
        accounts: acc,
        reservations: res,
        ledger: led,
        sagas: sag,
    }
}

fn build_orchestrator(env: &ConcurrencyEnv) -> SagaOrchestrator {
    let reserve = ReserveHandler::new(
        env.reservations.clone() as Arc<dyn ReservationRepository>,
        env.accounts.clone() as Arc<dyn AccountRepository>,
        80, // 每 saga reserve 80
        Currency::Gold,
    );
    SagaOrchestrator::new(
        env.sagas.clone() as Arc<dyn SagaRepository>,
        env.reservations.clone() as Arc<dyn ReservationRepository>,
        vec![Arc::new(reserve) as Arc<dyn SagaStepHandler>],
    )
}

fn make_saga(account_id: Uuid, idem_key: &str) -> Saga {
    let mut s = Saga::new(
        SagaType::Transfer,
        Uuid::new_v4(),
        idem_key.to_string(),
        vec!["reserve".to_string()],
    );
    s.steps[0].resource_id = Some(account_id);
    s
}

async fn fund(acc: &InMemoryAccountRepository, player: Uuid, amount: i64) -> Uuid {
    let mut a = Account::new(player, Currency::Gold);
    a.credit(amount);
    let id = a.id;
    acc.save(&a).await.unwrap();
    id
}

// ============================================================================
// IT 1: 2 saga 争抢同一账户 → 1 成功 1 失败 (OCC 边界)
// ============================================================================

/// 场景: account.balance=100, 2 saga 各 reserve 80 并发跑
/// 预期: 1 success + 1 Err (OCC conflict)
/// 终态: balance=20, 1 reservation Reserved/Confirmed, 1 已 delete
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_two_sagas_one_account_one_wins() {
    let env = bootstrap();
    let player = Uuid::new_v4();
    let acc_id = fund(&env.accounts, player, 100).await;
    let orch = build_orchestrator(&env);

    let mut saga1 = make_saga(acc_id, "k-conc-a-1");
    let mut saga2 = make_saga(acc_id, "k-conc-a-2");

    // 并发跑 2 个 saga: 1 成功 1 OCC 冲突 (per RGS-REV-009 V1 LO-4)
    let (r1, r2) = tokio::join!(
        orch.execute(&mut saga1),
        orch.execute(&mut saga2),
    );

    let oks = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
    let errs = [&r1, &r2].iter().filter(|r| r.is_err()).count();
    assert_eq!(oks + errs, 2, "must produce exactly 2 results");
    assert_eq!(oks, 1, "exactly 1 saga should win");
    assert_eq!(errs, 1, "exactly 1 saga should lose (OCC or InsufficientFunds)");

    // 终态余额: 100 - 80 = 20 (成功那一个)
    let final_acc = env
        .accounts
        .find_by_id(acc_id)
        .await
        .unwrap()
        .expect("account exists");
    assert_eq!(
        final_acc.balance, 20,
        "after one successful reserve of 80, balance must be 20"
    );

    // reservations: 1 个 (winner) + 失败路径被 cleanup
    let winner_reservations = env.reservations.list_by_saga(saga1.id).await.unwrap();
    let loser_reservations = env.reservations.list_by_saga(saga2.id).await.unwrap();
    let total = winner_reservations.len() + loser_reservations.len();
    assert_eq!(
        total, 1,
        "exactly 1 reservation must remain (loser's reservation cleaned up on failure), got {} (s1={}, s2={})",
        total,
        winner_reservations.len(),
        loser_reservations.len()
    );

    // ledger: 1 条 spend (winner)
    let winner_saga = if r1.is_ok() { saga1.id } else { saga2.id };
    let ledger_entries = env.ledger.list_by_saga(winner_saga).await.unwrap();
    assert_eq!(
        ledger_entries.len(),
        1,
        "winner's ledger has 1 spend entry"
    );
    assert_eq!(ledger_entries[0].amount, -80, "spend amount is 80");
}

// ============================================================================
// IT 2: 余额严重不足 → 2 saga 都 InsufficientFunds 失败
// ============================================================================

/// 场景: account.balance=50, 2 saga 各 reserve 80
/// 预期: 2 个都失败 (InsufficientFunds 走 try_debit 路径, 不进 apply_atomic)
/// 终态: balance=50, 0 reservation (失败路径 dangling reservation 全部清理)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_two_sagas_same_account_both_lose_balance() {
    let env = bootstrap();
    let player = Uuid::new_v4();
    let acc_id = fund(&env.accounts, player, 50).await;
    let orch = build_orchestrator(&env);

    let mut saga1 = make_saga(acc_id, "k-conc-b-1");
    let mut saga2 = make_saga(acc_id, "k-conc-b-2");

    let (r1, r2) = tokio::join!(
        orch.execute(&mut saga1),
        orch.execute(&mut saga2),
    );

    assert!(r1.is_err(), "saga1 must fail (InsufficientFunds)");
    assert!(r2.is_err(), "saga2 must fail (InsufficientFunds)");

    // 终态余额: 50 (没有扣款)
    let final_acc = env
        .accounts
        .find_by_id(acc_id)
        .await
        .unwrap()
        .expect("account exists");
    assert_eq!(
        final_acc.balance, 50,
        "balance unchanged after InsufficientFunds failures"
    );

    // reservations: 失败路径上 dangling reservation 全部被清理 (per RGS-REV-009 CR-1)
    let s1_res = env.reservations.list_by_saga(saga1.id).await.unwrap();
    let s2_res = env.reservations.list_by_saga(saga2.id).await.unwrap();
    assert_eq!(
        s1_res.len() + s2_res.len(),
        0,
        "all dangling reservations must be cleaned up, got s1={} s2={}",
        s1_res.len(),
        s2_res.len()
    );

    // ledger: 0 条 (没有实际扣款)
    let s1_led = env.ledger.list_by_saga(saga1.id).await.unwrap();
    let s2_led = env.ledger.list_by_saga(saga2.id).await.unwrap();
    assert_eq!(s1_led.len(), 0);
    assert_eq!(s2_led.len(), 0);
}

// ============================================================================
// IT 3: 不同账户不冲突 → 2 saga 都成功
// ============================================================================

/// 场景: 2 个不同账户各 balance=100, 各自 reserve 80
/// 预期: 2 个都成功 (资源独立, 无争抢)
/// 终态: 2 个 balance 各 20, 2 个 reservation
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_two_sagas_distinct_accounts_both_win() {
    let env = bootstrap();
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let acc1 = fund(&env.accounts, player1, 100).await;
    let acc2 = fund(&env.accounts, player2, 100).await;
    let orch = build_orchestrator(&env);

    let mut saga1 = make_saga(acc1, "k-conc-c-1");
    let mut saga2 = make_saga(acc2, "k-conc-c-2");

    let (r1, r2) = tokio::join!(
        orch.execute(&mut saga1),
        orch.execute(&mut saga2),
    );

    assert!(r1.is_ok(), "saga1 must succeed (independent account)");
    assert!(r2.is_ok(), "saga2 must succeed (independent account)");

    // 终态余额: 各 20
    let a1 = env.accounts.find_by_id(acc1).await.unwrap().unwrap();
    let a2 = env.accounts.find_by_id(acc2).await.unwrap().unwrap();
    assert_eq!(a1.balance, 20, "acc1: 100 - 80 = 20");
    assert_eq!(a2.balance, 20, "acc2: 100 - 80 = 20");

    // reservations: 各 1 个
    let s1_res = env.reservations.list_by_saga(saga1.id).await.unwrap();
    let s2_res = env.reservations.list_by_saga(saga2.id).await.unwrap();
    assert_eq!(s1_res.len(), 1);
    assert_eq!(s2_res.len(), 1);
}

// ============================================================================
// IT 4: 同 player 不同 currency 不冲突 (currency 维度隔离)
// ============================================================================

/// 场景: 同一 player, Gold 账户 balance=100 + Diamond 账户 balance=100, 各自 reserve 80
/// 预期: 2 个都成功 (currency 维度独立, OCC 不会冲突)
/// 终态: Gold=20, Diamond=20, 2 个 reservation
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_two_sagas_same_player_distinct_currency() {
    let env = bootstrap();
    let player = Uuid::new_v4();
    let mut gold_acc = Account::new(player, Currency::Gold);
    gold_acc.credit(100);
    let gold_id = gold_acc.id;
    env.accounts.save(&gold_acc).await.unwrap();

    let mut diamond_acc = Account::new(player, Currency::Diamond);
    diamond_acc.credit(100);
    let diamond_id = diamond_acc.id;
    env.accounts.save(&diamond_acc).await.unwrap();

    let reserve_gold = ReserveHandler::new(
        env.reservations.clone() as Arc<dyn ReservationRepository>,
        env.accounts.clone() as Arc<dyn AccountRepository>,
        80,
        Currency::Gold,
    );
    let reserve_diamond = ReserveHandler::new(
        env.reservations.clone() as Arc<dyn ReservationRepository>,
        env.accounts.clone() as Arc<dyn AccountRepository>,
        80,
        Currency::Diamond,
    );
    let orch = SagaOrchestrator::new(
        env.sagas.clone() as Arc<dyn SagaRepository>,
        env.reservations.clone() as Arc<dyn ReservationRepository>,
        vec![
            Arc::new(reserve_gold) as Arc<dyn SagaStepHandler>,
            Arc::new(reserve_diamond) as Arc<dyn SagaStepHandler>,
        ],
    );

    let mut saga_gold = Saga::new(
        SagaType::Transfer,
        Uuid::new_v4(),
        "k-conc-d-gold".to_string(),
        vec!["reserve".to_string()],
    );
    saga_gold.steps[0].resource_id = Some(gold_id);
    let mut saga_diamond = Saga::new(
        SagaType::Transfer,
        Uuid::new_v4(),
        "k-conc-d-diamond".to_string(),
        vec!["reserve".to_string()],
    );
    saga_diamond.steps[0].resource_id = Some(diamond_id);

    let (rg, rd) = tokio::join!(
        orch.execute(&mut saga_gold),
        orch.execute(&mut saga_diamond),
    );

    assert!(rg.is_ok(), "Gold saga must succeed");
    assert!(rd.is_ok(), "Diamond saga must succeed");

    let final_gold = env.accounts.find_by_id(gold_id).await.unwrap().unwrap();
    let final_diamond = env.accounts.find_by_id(diamond_id).await.unwrap().unwrap();
    assert_eq!(final_gold.balance, 20, "Gold: 100 - 80 = 20");
    assert_eq!(final_diamond.balance, 20, "Diamond: 100 - 80 = 20");
}
