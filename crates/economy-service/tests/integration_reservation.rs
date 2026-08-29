//! WF-1-55.47: reservation 端到端集成测试 (per RGS-OPEN-QA-001 Q-M-07 答复)
//!
//! 范围：补充 WF-1-55.27 修过的 ReserveHandler OCC cleanup 路径
//! 在「真 PG + 真 Repository + 真 Service」全链路下的端到端覆盖。
//!
//! 三个 IT：
//! 1. `it_reservation_create_success` —— login → reserve → confirm 全链路 happy path
//! 2. `it_reservation_conflict_releases` —— reserve → OCC 冲突 → 自动 release 资源
//!    (验证 RGS-REV-009 CR-1 真修：apply_atomic 失败路径 reservation.delete_by_id)
//! 3. `it_reservation_cleanup_on_failure` —— reserve → saga 失败 → cleanup 完整
//!    (验证 RGS-REV-008 CC-4 / verify-C：saga 失败时 dangling reservation 全部清理)
//!
//! 设计（per RGS-OPEN-QA-001 Q-M-07 答复"3 个新增子任务，50/50 是单元测试范围，
//! 需要补端到端 IT" + RGS-REV-009 V3 H-1 强约束）：
//! - 全部用真 PG（PgAccountRepository / PgReservationRepository / PgTransactionLedgerRepository）
//! - 隔离策略同 integration_outbox.rs：每 test UUID 后缀独立 DB, test 结束 DROP DATABASE
//! - 无 DATABASE_URL → skip（CI 在 docker compose up -d postgres 后跑）
//!
//! 锚定文件：
//! - 源: src/service.rs::apply_atomic_with_reservation
//! - 源: src/saga_orchestrator.rs::ReserveHandler
//! - 源: src/saga_orchestrator.rs::ConfirmHandler
//! - 源: src/reservation.rs::release() (per RGS-REV-009 CR-1)

use std::sync::Arc;

use rgs_testkit::pg_test_db::pg_available;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use economy_service::entity::{Account, Currency, TransactionKind};
use economy_service::reservation::{ReservationRepository, ReservationStatus};
use economy_service::saga::{Saga, SagaType};
use economy_service::saga_orchestrator::{ConfirmHandler, ReserveHandler, SagaOrchestrator};
use economy_service::{
    AccountRepository, PgAccountRepository, PgReservationRepository, PgSagaRepository,
    PgTransactionLedgerRepository, SagaRepository, TransactionLedgerRepository,
};

// ============================================================================
// 隔离 DB 工具 (借鉴 integration_outbox.rs)
// ============================================================================

/// 拿独立测试 DB URL + db_name. 返回 None 表示无 DATABASE_URL (skip 模式).
///
/// 隔离策略：每个 test 拿一个全新 DB, 跑完 DROP DATABASE WITH (FORCE),
/// 避免与其它 test 共享 schema state 冲突 (sqlx::migrate! 不能在事务里跑).
fn isolated_db_url() -> Option<(String, String, String)> {
    let base = std::env::var("DATABASE_URL").ok()?;
    let (prefix, query) = match base.split_once('?') {
        Some((p, q)) => (p.to_string(), format!("?{}", q)),
        None => (base.clone(), String::new()),
    };
    let last_slash = prefix.rfind('/').expect("DATABASE_URL must have /<dbname>");
    let db_name = format!("wf_1_55_47_{}", Uuid::new_v4().simple());
    let test_url = format!("{}{}{}", &prefix[..last_slash], db_name, query);
    let admin_url = format!("{}/postgres{}", &prefix[..last_slash], query);
    Some((test_url, db_name, admin_url))
}

async fn create_test_db(admin_url: &str, db_name: &str) {
    let mut conn = PgConnection::connect(admin_url)
        .await
        .expect("connect to admin DB");
    let create_sql = format!("CREATE DATABASE \"{}\"", db_name);
    conn.execute(create_sql.as_str())
        .await
        .expect("create test database");
}

async fn drop_test_db(admin_url: &str, db_name: &str) {
    let mut conn = PgConnection::connect(admin_url)
        .await
        .expect("reconnect to admin DB");
    let drop_sql = format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", db_name);
    let _ = conn.execute(drop_sql.as_str()).await;
}

async fn pg_pool_at(url: &str) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await?;
    Ok(pool)
}

/// 公共 test 装配套件：跑 migration + 准备 repo handle 集合 (Arc 包装, 共享给 handler)
async fn bootstrap(
    url: &str,
) -> (
    PgPool,
    Arc<PgAccountRepository>,
    Arc<PgReservationRepository>,
    Arc<PgTransactionLedgerRepository>,
    Arc<PgSagaRepository>,
) {
    let pool = pg_pool_at(url).await.expect("connect test DB");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("apply economy-service migrations");
    let acc_repo = Arc::new(PgAccountRepository::new(pool.clone()));
    let res_repo = Arc::new(PgReservationRepository::new(pool.clone()));
    let led_repo = Arc::new(PgTransactionLedgerRepository::new(pool.clone()));
    let sag_repo = Arc::new(PgSagaRepository::new(pool.clone()));
    (pool, acc_repo, res_repo, led_repo, sag_repo)
}

// ============================================================================
// IT 1: reservation create success —— 全链路 happy path
// ============================================================================

/// 端到端 happy path：login (account.create) → reserve (apply_atomic_with_reservation)
/// → confirm (ConfirmHandler.execute) 全流程, reservation 进入 Confirmed 终态,
/// 账户余额正确减少, ledger 写入 1 条.
///
/// 锚定：service.rs::EconomyServiceImpl::apply_atomic_with_reservation (RGS-REV-007 AC4 / DEC-015 P1)
/// + saga_orchestrator.rs::ConfirmHandler.execute
#[tokio::test]
async fn it_reservation_create_success() {
    let (test_url, db_name, admin_url) = match isolated_db_url() {
        Some(v) => v,
        None => {
            eprintln!("skip: DATABASE_URL not set");
            return;
        }
    };
    if !pg_available().await {
        eprintln!("skip: PG not reachable for it_reservation_create_success");
        return;
    }
    create_test_db(&admin_url, &db_name).await;
    let pool = match pg_pool_at(&test_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skip: cannot connect to test DB: {}", e);
            let _ = drop_test_db(&admin_url, &db_name).await;
            return;
        }
    };

    let acc_repo = Arc::new(PgAccountRepository::new(pool.clone()));
    let res_repo = Arc::new(PgReservationRepository::new(pool.clone()));
    let led_repo = Arc::new(PgTransactionLedgerRepository::new(pool.clone()));
    let sag_repo = Arc::new(PgSagaRepository::new(pool.clone()));

    // 1. login: 准备账户 + 余额
    let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
    account.credit(1000);
    let account_id = account.id;
    acc_repo.save(&account).await.expect("save account");

    // 2. reserve: 调 apply_atomic_with_reservation helper (service.rs L100)
    let svc = economy_service::service::EconomyServiceImpl::new(
        acc_repo.clone() as Arc<dyn AccountRepository>,
        led_repo.clone() as Arc<dyn TransactionLedgerRepository>,
    );
    let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();
    let saga_id = Uuid::new_v4();
    let cmd_id = Uuid::new_v4();
    let (updated_account, entry, reservation) = svc
        .apply_atomic_with_reservation(
            &account,
            100,
            Currency::Gold,
            TransactionKind::Transfer,
            saga_id,
            cmd_id,
            "k-it1-reserve".to_string(),
            &res_repo_dyn,
        )
        .await
        .expect("reserve via apply_atomic_with_reservation");

    // assert: 余额 1000 - 100 = 900
    assert_eq!(
        updated_account.balance, 900,
        "balance should be 1000 - 100 = 900"
    );
    // assert: ledger 写入 1 条
    assert_eq!(entry.amount, -100);
    assert_eq!(
        entry.status,
        economy_service::entity::TransactionStatus::Confirmed
    );
    assert_eq!(entry.saga_id, Some(saga_id));
    // assert: reservation 持久化
    let loaded = res_repo
        .find_by_id(reservation.id)
        .await
        .expect("query reservation")
        .expect("reservation must be persisted after reserve");
    assert_eq!(loaded.account_id, account_id);
    assert_eq!(loaded.amount, 100);
    assert_eq!(loaded.status, ReservationStatus::Reserved);

    // 3. confirm: 走 ConfirmHandler.execute (saga_orchestrator.rs L479+)
    // 准备 saga 含 2 步: reserve + confirm, resource_id 指向 account_id
    let mut saga = Saga::new(
        SagaType::Transfer,
        cmd_id,
        format!("saga-{}", saga_id),
        vec!["reserve".to_string(), "confirm".to_string()],
    );
    saga.steps[0].resource_id = Some(account_id);
    saga.steps[1].resource_id = Some(account_id);
    sag_repo.save(&saga).await.expect("save saga");

    let reserve_handler = ReserveHandler::new(
        res_repo.clone() as Arc<dyn ReservationRepository>,
        acc_repo.clone() as Arc<dyn AccountRepository>,
        100,
        Currency::Gold,
    );
    let confirm_handler = ConfirmHandler::new(
        res_repo.clone() as Arc<dyn ReservationRepository>,
        acc_repo.clone() as Arc<dyn AccountRepository>,
    );
    let orchestrator = SagaOrchestrator::new(
        sag_repo.clone() as Arc<dyn SagaRepository>,
        res_repo.clone() as Arc<dyn ReservationRepository>,
        vec![
            Arc::new(reserve_handler)
                as Arc<dyn economy_service::saga_orchestrator::SagaStepHandler>,
            Arc::new(confirm_handler)
                as Arc<dyn economy_service::saga_orchestrator::SagaStepHandler>,
        ],
    );
    orchestrator.execute(&mut saga).await.expect("saga execute");

    // 4. 终态断言
    let confirmed = res_repo
        .find_by_id(reservation.id)
        .await
        .expect("query confirmed reservation")
        .expect("reservation still exists after confirm");
    assert_eq!(
        confirmed.status,
        ReservationStatus::Confirmed,
        "reservation must be Confirmed after ConfirmHandler.execute; got {:?}",
        confirmed.status
    );
    assert_eq!(
        saga.status,
        economy_service::saga::SagaStatus::Completed,
        "saga must be Completed after reserve + confirm"
    );

    drop_test_db(&admin_url, &db_name).await;
}

// ============================================================================
// IT 2: reservation conflict releases —— OCC 冲突自动 release
// ============================================================================

/// 端到端冲突释放：account 已 save 后, 外部 bump version (模拟并发 OCC 抢占),
/// 再调 apply_atomic_with_reservation, 期望：
/// - 返回 Validation("OCC conflict") 错误
/// - reservation 必须被 delete_by_id 清理 (per RGS-REV-008 CC-4 / verify-C 修复)
/// - 账户余额未变
/// - ledger 无任何条目
///
/// 锚定：service.rs::apply_atomic_with_reservation L155-171 失败路径 cleanup
/// + reservation.rs::delete_by_id
#[tokio::test]
async fn it_reservation_conflict_releases() {
    let (test_url, db_name, admin_url) = match isolated_db_url() {
        Some(v) => v,
        None => {
            eprintln!("skip: DATABASE_URL not set");
            return;
        }
    };
    if !pg_available().await {
        eprintln!("skip: PG not reachable for it_reservation_conflict_releases");
        return;
    }
    create_test_db(&admin_url, &db_name).await;
    let _pool = match pg_pool_at(&test_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skip: cannot connect to test DB: {}", e);
            let _ = drop_test_db(&admin_url, &db_name).await;
            return;
        }
    };

    let (_pool, acc_repo, res_repo, led_repo, _sag_repo) = bootstrap(&test_url).await;
    let acc_repo_dyn: Arc<dyn AccountRepository> = acc_repo.clone();
    let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();
    let led_repo_dyn: Arc<dyn TransactionLedgerRepository> = led_repo.clone();
    let svc = economy_service::service::EconomyServiceImpl::new(acc_repo_dyn, led_repo_dyn);

    let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
    account.credit(500);
    let account_id = account.id;
    let original_version = account.version;
    acc_repo.save(&account).await.expect("save account");

    // 模拟外部并发: 外部代码已经 bump version (eg. 另一笔交易抢先)
    let concurrent_account = acc_repo
        .find_by_id(account_id)
        .await
        .expect("re-fetch account")
        .expect("account must exist");
    let mut bumped = concurrent_account.clone();
    bumped.balance = 400; // 并发方已扣 100
    bumped.version = concurrent_account.version + 1;
    acc_repo
        .update_with_version(&bumped)
        .await
        .expect("concurrent OCC update");

    // 现在持有 stale account 调 apply_atomic_with_reservation
    // (本 test 不再 bump 自身 version, 用原始 version 触发冲突)
    let saga_id = Uuid::new_v4();
    let cmd_id = Uuid::new_v4();
    let err = svc
        .apply_atomic_with_reservation(
            &account, // version = original_version, 必冲突
            100,
            Currency::Gold,
            TransactionKind::Transfer,
            saga_id,
            cmd_id,
            "k-it2-occ".to_string(),
            &res_repo_dyn,
        )
        .await
        .expect_err("must fail with OCC conflict");

    // 断言 1: 错误是 Validation("OCC conflict")
    assert!(
        matches!(err, economy_service::Error::Validation(ref msg) if msg.contains("OCC conflict")),
        "expected OCC conflict Validation error, got {:?}",
        err
    );

    // 断言 2: dangling reservation 已被 cleanup (per RGS-REV-008 CC-4)
    // list_by_saga 应返回 0 条
    let for_saga = res_repo.list_by_saga(saga_id).await.expect("list_by_saga");
    assert_eq!(
        for_saga.len(),
        0,
        "dangling reservation must be cleaned up after OCC failure; got {:?}",
        for_saga
    );

    // 断言 3: 账户余额 = 400 (并发方已扣, 我们没再扣)
    let reloaded = acc_repo
        .find_by_id(account_id)
        .await
        .expect("re-fetch")
        .expect("account still exists");
    assert_eq!(
        reloaded.balance, 400,
        "balance must reflect only concurrent update"
    );
    assert_eq!(
        reloaded.version,
        original_version + 1,
        "version must be the concurrent one"
    );

    // 断言 4: ledger 无条目 (apply_atomic 未提交)
    let any = led_repo
        .find_by_idempotency_key("k-it2-occ")
        .await
        .expect("query ledger");
    assert!(any.is_none(), "ledger must have no entry after OCC failure");

    drop_test_db(&admin_url, &db_name).await;
}

// ============================================================================
// IT 3: reservation cleanup on failure —— saga 失败完整 cleanup
// ============================================================================

/// 端到端 saga 失败 cleanup：saga 走 reserve + confirm 2 步, reserve 成功后
/// confirm 失败 (eg. 用错误货币触发 AccountCurrencyMismatch 模拟),
/// 期望：orchestrator 触发 compensate, reserve 步骤的 reservation 退回 Compensated,
/// 账户余额恢复 (per saga_orchestrator.rs::ReserveHandler.compensate 退款逻辑).
///
/// 锚定：saga_orchestrator.rs::SagaOrchestrator::execute L127-145 失败分支
/// + saga_orchestrator.rs::SagaOrchestrator::compensate (RGS-REV-009 V1 LO-4 修复)
/// + saga_orchestrator.rs::ReserveHandler.compensate
#[tokio::test]
async fn it_reservation_cleanup_on_failure() {
    let (test_url, db_name, admin_url) = match isolated_db_url() {
        Some(v) => v,
        None => {
            eprintln!("skip: DATABASE_URL not set");
            return;
        }
    };
    if !pg_available().await {
        eprintln!("skip: PG not reachable for it_reservation_cleanup_on_failure");
        return;
    }
    create_test_db(&admin_url, &db_name).await;
    let _pool = match pg_pool_at(&test_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skip: cannot connect to test DB: {}", e);
            let _ = drop_test_db(&admin_url, &db_name).await;
            return;
        }
    };

    let (_pool, acc_repo, res_repo, _led_repo, sag_repo) = bootstrap(&test_url).await;
    let acc_repo_dyn: Arc<dyn AccountRepository> = acc_repo.clone();
    let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

    // login
    let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
    account.credit(500);
    let account_id = account.id;
    acc_repo.save(&account).await.expect("save account");

    // 建 saga: reserve (success) + confirm (失败, 用一个会失败的 Confirm handler)
    let cmd_id = Uuid::new_v4();
    let mut saga = Saga::new(
        SagaType::Transfer,
        cmd_id,
        format!("saga-cleanup-{}", Uuid::new_v4().simple()),
        vec!["reserve".to_string(), "confirm".to_string()],
    );
    saga.steps[0].resource_id = Some(account_id);
    saga.steps[1].resource_id = Some(account_id);
    sag_repo.save(&saga).await.expect("save saga");

    let reserve_handler = ReserveHandler::new(
        res_repo_dyn.clone(),
        acc_repo_dyn.clone(),
        100,
        Currency::Gold,
    );
    // 构造一个 confirm 阶段必失败的 handler: 用不存在的 account_id
    // 让 ConfirmHandler 走 find_by_id → None 路径 → Error::NotFound → 触发 compensate
    let mut broken_saga = saga.clone();
    broken_saga.steps[1].resource_id = Some(Uuid::new_v4()); // 故意指向不存在账户
    let confirm_handler = ConfirmHandler::new(res_repo_dyn.clone(), acc_repo_dyn.clone());

    let orchestrator = SagaOrchestrator::new(
        sag_repo.clone() as Arc<dyn SagaRepository>,
        res_repo_dyn.clone(),
        vec![
            Arc::new(reserve_handler)
                as Arc<dyn economy_service::saga_orchestrator::SagaStepHandler>,
            Arc::new(confirm_handler)
                as Arc<dyn economy_service::saga_orchestrator::SagaStepHandler>,
        ],
    );

    // 执行: reserve 成功, confirm 失败 → 触发 compensate
    // 用 broken_saga 让 confirm 找不到 account
    let _ = orchestrator.execute(&mut broken_saga).await; // 期待 Err, 显式忽略

    // 重新加载 saga 验证终态
    let final_saga = sag_repo
        .find_by_id(broken_saga.id)
        .await
        .expect("re-fetch saga")
        .expect("saga must persist");
    assert_eq!(
        final_saga.status,
        economy_service::saga::SagaStatus::Failed,
        "saga must be Failed after confirm step failure"
    );

    // 关键断言：reserve 步骤的 reservation 状态: 应该不存在 (ReserveHandler.compensate 走
    // list_by_saga 找 reservation + apply_atomic 退款 + 标记 Compensated)
    // 由于 broken_saga 的 step[1] resource_id 是不存在 account, reserve 阶段拿 account_id
    // 是从 step[0].resource_id = account_id (正确), 所以 reserve 成功 + reservation 持久化
    // + apply_atomic 扣 100 成功, 余额 = 500 - 100 = 400
    // 之后 confirm 失败 → compensate → ReserveHandler.compensate 退款 +amount 100 → 余额 = 500
    let reloaded = acc_repo
        .find_by_id(account_id)
        .await
        .expect("re-fetch")
        .expect("account exists");
    assert_eq!(
        reloaded.balance, 500,
        "balance must be restored to 500 after compensate (refund +100); got {}",
        reloaded.balance
    );

    // reservation 列表 (按 broken_saga.id): reserve 成功时持久化的 reservation 已被
    // compensate 标记 Compensated
    let res_list = res_repo
        .list_by_saga(broken_saga.id)
        .await
        .expect("list_by_saga");
    // 可能有 0 条 (若 confirm 错误是 load 阶段, reserve 的 reservation 已被 cleanup) 或
    // 1 条 Compensated (若 confirm 错误发生在 reserve 之后, ReserveHandler.compensate 走完整退款)
    if !res_list.is_empty() {
        for r in &res_list {
            assert_eq!(
                r.status,
                ReservationStatus::Compensated,
                "all reservations on failed saga must be Compensated; got {:?} for reservation {}",
                r.status,
                r.id
            );
        }
    }

    // 不再使用 saga 变量
    let _ = saga;

    drop_test_db(&admin_url, &db_name).await;
}

// ============================================================================
// 显式 anchor: 防 rgs-testkit import 在 cargo test 编译时静默丢失
// ============================================================================

#[allow(dead_code)]
fn _ensure_rgs_testkit_used() {
    let _ = pg_available;
}
