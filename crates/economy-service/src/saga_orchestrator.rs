//! Saga 编排器（per RGS-DTL-100 §3-§5 Saga 决策与执行）
//!
//! 54.8 实化：SagaOrchestrator trait + 默认实现
//! 55.12 实化：ReserveHandler / ConfirmHandler 真实持久化（per RGS-REV-007 AC4 / DEC-015 P1）
//!
//! 设计：
//! - SagaOrchestrator 接收 saga + step 处理器列表
//! - step 处理器是 `async fn(&mut Saga, ...) -> Result<()>` 形式
//! - execute 步进：每步调对应 handler；成功 advance，失败 compensate
//! - 状态机每步都 persist 到 saga 表（崩溃可恢复）
//! - compensate 携带 step.resource_id 解决"反向补偿时 current 指针停在失败步"的问题
//!
//! 55.12 关键修复（RGS-REV-007-A 报告 C4）：
//! - 旧实现：ReserveHandler.execute 构造 Reservation 后只打 log，未调 reservations.save
//!           ConfirmHandler.execute 只打 log
//!           两个 compensate 全部 no-op
//! - 新实现：handler 构造函数注入 reservations + accounts（+ amount/currency for Reserve）
//!           execute 真正持久化 reservation、原子更新账户余额 + 写入账目
//!           compensate 真正释放 reservation、退款（如果是 Confirm 失败）

use std::sync::Arc;
use uuid::Uuid;

use crate::entity::{
    Account, AccountStatus, Currency, TransactionKind, TransactionLedger, TransactionStatus,
};
use crate::error::Error;
use crate::repository::AccountRepository;
use crate::reservation::{Reservation, ReservationRepository};
use crate::saga::{Saga, SagaRepository, SagaStatus, SagaStepStatus};
use crate::Result;

/// Saga 步进处理器 trait
///
/// 每个实现负责执行一个 step + 反向补偿
#[async_trait::async_trait]
pub trait SagaStepHandler: Send + Sync {
    /// step 名
    fn name(&self) -> &str;
    /// 执行 step
    async fn execute(&self, saga: &mut Saga) -> Result<()>;
    /// 反向补偿（per RGS-DTL-100 §4 补偿模式）
    ///
    /// `resource_id` 为被补偿 step 的 resource_id（避免依赖 saga.current()，
    /// 编排器补偿阶段 saga.current_step 指针停在失败步，与"已完成的步"不对齐）。
    async fn compensate(&self, saga: &mut Saga, resource_id: Option<Uuid>) -> Result<()>;
}

/// SagaOrchestrator
pub struct SagaOrchestrator {
    pub sagas: Arc<dyn SagaRepository>,
    /// Reservation 仓储（编排器补偿阶段可读取 saga 关联的所有 reservations 做诊断 / 审计）
    pub reservations: Arc<dyn ReservationRepository>,
    handlers: Vec<Arc<dyn SagaStepHandler>>,
}

impl SagaOrchestrator {
    pub fn new(
        sagas: Arc<dyn SagaRepository>,
        reservations: Arc<dyn ReservationRepository>,
        handlers: Vec<Arc<dyn SagaStepHandler>>,
    ) -> Self {
        Self {
            sagas,
            reservations,
            handlers,
        }
    }

    /// 执行 Saga（步进式）
    pub async fn execute(&self, saga: &mut Saga) -> Result<()> {
        if saga.status != SagaStatus::Pending {
            return Err(Error::Validation(format!(
                "saga {} status is not Pending ({:?})",
                saga.id, saga.status
            )));
        }

        // 启动
        saga.start();
        self.sagas.save(saga).await?;

        // 步进
        while let Some(current) = saga.current().cloned() {
            // 先取 step name（避免 immutable / mutable borrow 冲突）
            let step_name = current.name;

            let handler = self
                .handlers
                .iter()
                .find(|h| h.name() == step_name)
                .ok_or_else(|| Error::Validation(format!("no handler for step {}", step_name)))?;

            // 标记 running（handler.execute 可能耗时）
            saga.current_mut().unwrap().mark_running();
            self.sagas.save(saga).await?;

            match handler.execute(saga).await {
                Ok(()) => {
                    saga.current_mut().unwrap().mark_completed();
                    if !saga.advance() {
                        // 所有步骤完成
                        saga.complete();
                        self.sagas.save(saga).await?;
                        return Ok(());
                    }
                    self.sagas.save(saga).await?;
                }
                Err(e) => {
                    saga.current_mut().unwrap().mark_failed(e.to_string());
                    self.sagas.save(saga).await?;
                    // 触发补偿
                    self.compensate(saga).await?;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 反向补偿
    pub async fn compensate(&self, saga: &mut Saga) -> Result<()> {
        // 先收集已完成 step 的 (name, resource_id) —— 必须在 saga.compensate() 修改
        // step.status 之前做，否则 filter 永远为空（这是 per RGS-REV-007 AC4 实化中发现的
        // 旧 bug：旧 ReserveHandler/ConfirmHandler 补偿路径全 no-op，所以 bug 未暴露）
        let completed: Vec<(String, Option<Uuid>)> = saga
            .steps
            .iter()
            .rev()
            .filter(|s| s.status == SagaStepStatus::Completed)
            .map(|s| (s.name.clone(), s.resource_id))
            .collect();

        saga.compensate();
        self.sagas.save(saga).await?;

        for (name, resource_id) in completed {
            if let Some(handler) = self.handlers.iter().find(|h| h.name() == name) {
                handler.compensate(saga, resource_id).await?;
            }
        }

        saga.fail();
        self.sagas.save(saga).await?;
        Ok(())
    }

    /// 通过 saga_id 重新加载并继续执行（崩溃恢复）
    pub async fn resume(&self, saga_id: Uuid) -> Result<()> {
        let mut saga = self
            .sagas
            .find_by_id(saga_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Saga",
                id: saga_id.to_string(),
            })?;
        self.execute(&mut saga).await
    }
}

// ============================================================================
// 真实 step handler 实化（per RGS-REV-007 AC4 / DEC-015 P1）
// ============================================================================

/// 构造 idempotency_key（handler 内部用）
fn saga_idem_key(saga_id: Uuid, suffix: &str) -> String {
    format!("saga:{}-{}", saga_id, suffix)
}

/// 寻找当前 step 的 resource_id（handler 内部 helper）
fn current_resource_id(saga: &Saga) -> Result<Uuid> {
    saga.current()
        .and_then(|s| s.resource_id)
        .ok_or_else(|| Error::Validation("current step has no resource_id".to_string()))
}

/// 读取账户，账户不存在或冻结即报错
async fn load_active_account(
    accounts: &Arc<dyn AccountRepository>,
    account_id: Uuid,
) -> Result<Account> {
    let account = accounts
        .find_by_id(account_id)
        .await?
        .ok_or_else(|| Error::NotFound {
            entity: "Account",
            id: account_id.to_string(),
        })?;
    if !matches!(account.status, AccountStatus::Active) {
        return Err(Error::AccountFrozen(account_id.to_string()));
    }
    Ok(account)
}

/// ReserveHandler：预留资金（保存 reservation + 原子扣减账户余额 + 写账目）
pub struct ReserveHandler {
    reservations: Arc<dyn ReservationRepository>,
    accounts: Arc<dyn AccountRepository>,
    /// 单次预留金额
    amount: i64,
    /// 单次预留货币
    currency: Currency,
}

impl ReserveHandler {
    pub fn new(
        reservations: Arc<dyn ReservationRepository>,
        accounts: Arc<dyn AccountRepository>,
        amount: i64,
        currency: Currency,
    ) -> Self {
        Self {
            reservations,
            accounts,
            amount,
            currency,
        }
    }
}

#[async_trait::async_trait]
impl SagaStepHandler for ReserveHandler {
    fn name(&self) -> &str {
        "reserve"
    }

    async fn execute(&self, saga: &mut Saga) -> Result<()> {
        let account_id = current_resource_id(saga)?;

        // 1. 构造 reservation 并持久化（per RGS-REV-007 AC4：旧实现只打 log）
        let r = Reservation::new(saga.id, account_id, self.amount, self.currency);
        self.reservations.save(&r).await?;

        // 2. 校验余额（OCC 前本地校验）
        let mut account = load_active_account(&self.accounts, account_id).await?;
        if !account.try_debit(self.amount) {
            // 清理 dangling reservation
            let _ = self.reservations.delete_by_id(r.id).await;
            return Err(Error::InsufficientFunds {
                account_id: account_id.to_string(),
                balance: account.balance,
                required: self.amount,
            });
        }

        // 3. 原子事务：OCC 更新账户 + 写账目（per RGS-REV-007 AC3 修复）
        let mut entry = TransactionLedger::new(
            account.id,
            -self.amount,
            self.currency,
            TransactionKind::Transfer,
            saga_idem_key(saga.id, "reserve"),
        );
        entry.saga_id = Some(saga.id);
        entry.status = TransactionStatus::Confirmed;
        self.accounts.apply_atomic(&account, &entry).await?;

        tracing::info!(
            target: "saga",
            saga_id = %saga.id,
            account_id = %account_id,
            reservation_id = %r.id,
            amount = self.amount,
            currency = ?self.currency,
            "ReserveHandler executed"
        );
        Ok(())
    }

    async fn compensate(&self, saga: &mut Saga, resource_id: Option<Uuid>) -> Result<()> {
        let account_id =
            resource_id.ok_or_else(|| Error::Validation("reserve: missing resource_id".to_string()))?;

        // 1. 找 reservation（按 saga_id + account_id 过滤）
        let reservations = self.reservations.list_by_saga(saga.id).await?;
        let mut r = reservations
            .into_iter()
            .find(|r| r.account_id == account_id)
            .ok_or_else(|| Error::NotFound {
                entity: "Reservation",
                id: format!("saga={}-acct={}", saga.id, account_id),
            })?;

        // 2. 退款（读取最新账户，OCC 更新 +amount）
        let mut account = self
            .accounts
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Account",
                id: account_id.to_string(),
            })?;
        let refund_amount = r.amount;
        let refund_currency = r.currency;
        account.credit(refund_amount);

        let mut entry = TransactionLedger::new(
            account.id,
            refund_amount,
            refund_currency,
            TransactionKind::Compensation,
            saga_idem_key(saga.id, "compensate-reserve"),
        );
        entry.saga_id = Some(saga.id);
        entry.status = TransactionStatus::Confirmed;
        self.accounts.apply_atomic(&account, &entry).await?;

        // 3. 标记 reservation 为已补偿
        r.compensate();
        self.reservations.save(&r).await?;

        tracing::info!(
            target: "saga",
            saga_id = %saga.id,
            account_id = %account_id,
            reservation_id = %r.id,
            refund_amount,
            "ReserveHandler compensated"
        );
        Ok(())
    }
}

/// ConfirmHandler：确认预留（标记 reservation 为 Confirmed）
pub struct ConfirmHandler {
    reservations: Arc<dyn ReservationRepository>,
    accounts: Arc<dyn AccountRepository>,
}

impl ConfirmHandler {
    pub fn new(
        reservations: Arc<dyn ReservationRepository>,
        accounts: Arc<dyn AccountRepository>,
    ) -> Self {
        Self {
            reservations,
            accounts,
        }
    }
}

#[async_trait::async_trait]
impl SagaStepHandler for ConfirmHandler {
    fn name(&self) -> &str {
        "confirm"
    }

    async fn execute(&self, saga: &mut Saga) -> Result<()> {
        let account_id = current_resource_id(saga)?;

        // 找 reservation
        let reservations = self.reservations.list_by_saga(saga.id).await?;
        let mut r = reservations
            .into_iter()
            .find(|r| r.account_id == account_id)
            .ok_or_else(|| Error::NotFound {
                entity: "Reservation",
                id: format!("saga={}-acct={}", saga.id, account_id),
            })?;

        // 标记 confirmed 并持久化
        r.confirm();
        self.reservations.save(&r).await?;

        tracing::info!(
            target: "saga",
            saga_id = %saga.id,
            account_id = %account_id,
            reservation_id = %r.id,
            "ConfirmHandler executed"
        );
        Ok(())
    }

    async fn compensate(&self, saga: &mut Saga, resource_id: Option<Uuid>) -> Result<()> {
        // Confirm 失败也需退款（因为 Reserve 阶段已实际扣款）
        let account_id = resource_id
            .ok_or_else(|| Error::Validation("confirm: missing resource_id".to_string()))?;

        let reservations = self.reservations.list_by_saga(saga.id).await?;
        let mut r = reservations
            .into_iter()
            .find(|r| r.account_id == account_id)
            .ok_or_else(|| Error::NotFound {
                entity: "Reservation",
                id: format!("saga={}-acct={}", saga.id, account_id),
            })?;

        // 退款
        let mut account = self
            .accounts
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Account",
                id: account_id.to_string(),
            })?;
        let refund_amount = r.amount;
        let refund_currency = r.currency;
        account.credit(refund_amount);

        let mut entry = TransactionLedger::new(
            account.id,
            refund_amount,
            refund_currency,
            TransactionKind::Compensation,
            saga_idem_key(saga.id, "compensate-confirm"),
        );
        entry.saga_id = Some(saga.id);
        entry.status = TransactionStatus::Confirmed;
        self.accounts.apply_atomic(&account, &entry).await?;

        // 标记 reservation 为已补偿
        r.compensate();
        self.reservations.save(&r).await?;

        tracing::info!(
            target: "saga",
            saga_id = %saga.id,
            account_id = %account_id,
            reservation_id = %r.id,
            refund_amount,
            "ConfirmHandler compensated"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Account;
    use crate::repository::{
        InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
    };
    use crate::reservation::{InMemoryReservationRepository, ReservationStatus};
    use crate::saga::{InMemorySagaRepository, Saga, SagaType};
    use std::sync::Mutex;

    const TEST_AMOUNT: i64 = 100;
    const TEST_INITIAL_BALANCE: i64 = 500;

    /// 构造带共享 ledger 的 in-memory 依赖四件套（per RGS-REV-007 AC3 修复）
    struct TestEnv {
        orch: SagaOrchestrator,
        #[allow(dead_code)]
        sagas: Arc<InMemorySagaRepository>,
        accounts: Arc<InMemoryAccountRepository>,
        reservations: Arc<InMemoryReservationRepository>,
        ledger: Arc<InMemoryTransactionLedgerRepository>,
    }

    async fn make_env(initial_balance: i64) -> TestEnv {
        let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
        let acc_repo = Arc::new(
            InMemoryAccountRepository::new().with_shared_ledger(led_repo.inner.clone()),
        );
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let sag_repo = Arc::new(InMemorySagaRepository::new());

        let reserve = ReserveHandler::new(
            res_repo.clone() as Arc<dyn ReservationRepository>,
            acc_repo.clone() as Arc<dyn AccountRepository>,
            TEST_AMOUNT,
            Currency::Gold,
        );
        let confirm = ConfirmHandler::new(
            res_repo.clone() as Arc<dyn ReservationRepository>,
            acc_repo.clone() as Arc<dyn AccountRepository>,
        );

        let orch = SagaOrchestrator::new(
            sag_repo.clone() as Arc<dyn SagaRepository>,
            res_repo.clone() as Arc<dyn ReservationRepository>,
            vec![Arc::new(reserve), Arc::new(confirm)],
        );

        // 预存账户
        let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
        account.credit(initial_balance);
        acc_repo.save(&account).await.unwrap();

        TestEnv {
            orch,
            sagas: sag_repo,
            accounts: acc_repo,
            reservations: res_repo,
            ledger: led_repo,
        }
    }

    /// 失败 step handler —— 用于触发补偿路径测试
    struct FailingHandler {
        name: String,
    }

    #[async_trait::async_trait]
    impl SagaStepHandler for FailingHandler {
        fn name(&self) -> &str {
            &self.name
        }
        async fn execute(&self, _saga: &mut Saga) -> Result<()> {
            Err(Error::Validation("simulated step failure".to_string()))
        }
        async fn compensate(&self, _saga: &mut Saga, _rid: Option<Uuid>) -> Result<()> {
            Ok(())
        }
    }

    fn make_transfer_saga(account_id: Uuid) -> Saga {
        let mut saga = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-transfer-001".to_string(),
            vec!["reserve".to_string(), "confirm".to_string()],
        );
        saga.steps[0].resource_id = Some(account_id);
        saga.steps[1].resource_id = Some(account_id);
        saga
    }

    fn account_id_in_env(env: &TestEnv) -> Uuid {
        let map = env.accounts.inner.lock().unwrap();
        *map.keys().next().expect("env must have one account")
    }

    #[tokio::test]
    async fn execute_simple_saga_completes() {
        // 兼容旧测试：原 execute_simple_saga_completes 仍工作
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        env.orch.execute(&mut saga).await.unwrap();
        assert_eq!(saga.status, SagaStatus::Completed);
        assert_eq!(saga.steps[0].status, SagaStepStatus::Completed);
        assert_eq!(saga.steps[1].status, SagaStepStatus::Completed);
    }

    #[tokio::test]
    async fn reserve_handler_persists_reservation() {
        // AC4 验证：reservations.save 被调，reservation 真在表里
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        env.orch.execute(&mut saga).await.unwrap();

        // 1 个 reservation
        let reservations = env
            .reservations
            .list_by_saga(saga.id)
            .await
            .unwrap();
        assert_eq!(reservations.len(), 1, "exactly 1 reservation");
        let r = &reservations[0];
        assert_eq!(r.saga_id, saga.id);
        assert_eq!(r.account_id, account_id);
        assert_eq!(r.amount, TEST_AMOUNT);
        assert_eq!(r.currency, Currency::Gold);
        assert_eq!(r.status, ReservationStatus::Confirmed); // confirm 步骤也跑过
    }

    #[tokio::test]
    async fn reserve_handler_debits_account_atomically() {
        // AC3 兼容：debit 成功时 balance 和 ledger 同步更新
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        env.orch.execute(&mut saga).await.unwrap();

        // balance = initial - amount
        let account = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.balance, TEST_INITIAL_BALANCE - TEST_AMOUNT);

        // ledger 写入（apply_atomic 原子性）：reserve + compensate-confirm 也算 → 至少 1 条
        let led_count = env.ledger.inner.lock().unwrap().len();
        assert!(led_count >= 1, "ledger should have entries");
    }

    #[tokio::test]
    async fn confirm_handler_marks_reservation_consumed() {
        // AC4 验证：ConfirmHandler 把 reservation 标为 Confirmed
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);

        // 只跑 reserve 步骤
        let mut reserve_only = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-reserve-only".to_string(),
            vec!["reserve".to_string()],
        );
        reserve_only.steps[0].resource_id = Some(account_id);
        env.orch.execute(&mut reserve_only).await.unwrap();

        // reservation 应是 Reserved（未走 confirm）
        let reservations = env
            .reservations
            .list_by_saga(reserve_only.id)
            .await
            .unwrap();
        assert_eq!(reservations.len(), 1);
        assert_eq!(reservations[0].status, ReservationStatus::Reserved);

        // 现在跑 confirm（构造只含 confirm 步骤的 saga，复用 saga_id）
        let mut confirm_only = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-confirm-only".to_string(),
            vec!["confirm".to_string()],
        );
        // 复用上一个 saga 的 reservation：通过直接调 ConfirmHandler
        let confirm = ConfirmHandler::new(
            env.reservations.clone() as Arc<dyn ReservationRepository>,
            env.accounts.clone() as Arc<dyn AccountRepository>,
        );
        // 把 reservation 的 saga_id 改成 confirm_only.id 以便 handler 找到
        let mut r = reservations.into_iter().next().unwrap();
        r.saga_id = confirm_only.id;
        r.status = ReservationStatus::Reserved;
        env.reservations.save(&r).await.unwrap();
        confirm_only.steps[0].resource_id = Some(account_id);

        confirm.execute(&mut confirm_only).await.unwrap();

        let after = env
            .reservations
            .find_by_id(r.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.status, ReservationStatus::Confirmed);
    }

    #[tokio::test]
    async fn compensate_releases_reservation_and_refunds() {
        // AC4 验证：补偿路径真释放 reservation + 退款
        let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
        let acc_repo = Arc::new(
            InMemoryAccountRepository::new().with_shared_ledger(led_repo.inner.clone()),
        );
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let sag_repo = Arc::new(InMemorySagaRepository::new());

        // 预存账户
        let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
        account.credit(TEST_INITIAL_BALANCE);
        let account_id = account.id;
        acc_repo.save(&account).await.unwrap();

        // 编排器：reserve → fail_step，failing handler 会触发 reserve 补偿
        let reserve = ReserveHandler::new(
            res_repo.clone() as Arc<dyn ReservationRepository>,
            acc_repo.clone() as Arc<dyn AccountRepository>,
            TEST_AMOUNT,
            Currency::Gold,
        );
        let fail = FailingHandler {
            name: "fail_step".to_string(),
        };
        let orch = SagaOrchestrator::new(
            sag_repo.clone() as Arc<dyn SagaRepository>,
            res_repo.clone() as Arc<dyn ReservationRepository>,
            vec![Arc::new(reserve), Arc::new(fail)],
        );

        let mut saga = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-compensate".to_string(),
            vec!["reserve".to_string(), "fail_step".to_string()],
        );
        saga.steps[0].resource_id = Some(account_id);
        saga.steps[1].resource_id = Some(account_id);

        let err = orch.execute(&mut saga).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)));

        // saga 终态 = Failed
        assert_eq!(saga.status, SagaStatus::Failed);
        // 失败步标记 Failed；reserve 步被补偿（saga.compensate() 会把它标为 Compensated）
        assert_eq!(saga.steps[0].status, SagaStepStatus::Compensated);
        assert_eq!(saga.steps[1].status, SagaStepStatus::Failed);

        // reservation 被标为 Compensated
        let reservations = res_repo.list_by_saga(saga.id).await.unwrap();
        assert_eq!(reservations.len(), 1);
        assert_eq!(reservations[0].status, ReservationStatus::Compensated);

        // 账户被退款（balance 恢复 initial）
        let after = acc_repo.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(after.balance, TEST_INITIAL_BALANCE);
    }

    #[tokio::test]
    async fn saga_full_lifecycle_with_real_handlers() {
        // AC4 端到端：完整 Transfer Saga（reserve + confirm）+ 验证最终态
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        env.orch.execute(&mut saga).await.unwrap();

        // saga 终态 = Completed
        assert_eq!(saga.status, SagaStatus::Completed);
        assert_eq!(saga.steps[0].status, SagaStepStatus::Completed);
        assert_eq!(saga.steps[1].status, SagaStepStatus::Completed);

        // 账户余额 = initial - amount（reserve 阶段扣了，confirm 阶段不重复扣）
        let account = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.balance, TEST_INITIAL_BALANCE - TEST_AMOUNT);

        // reservation = Confirmed
        let reservations = env
            .reservations
            .list_by_saga(saga.id)
            .await
            .unwrap();
        assert_eq!(reservations.len(), 1);
        assert_eq!(reservations[0].status, ReservationStatus::Confirmed);

        // ledger 至少 1 条（reserve 的 Transfer 账目）
        let led_count = env.ledger.inner.lock().unwrap().len();
        assert!(led_count >= 1, "ledger should have at least 1 entry");
    }

    #[tokio::test]
    async fn reserve_handler_rejects_insufficient_funds() {
        // Reserve 校验：余额不足时返回 InsufficientFunds 且 reservation 被清理
        let env = make_env(TEST_AMOUNT - 1).await; // 余额 99 < amount 100
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        let err = env.orch.execute(&mut saga).await.unwrap_err();
        assert!(matches!(err, Error::InsufficientFunds { .. }));

        // reservation 被清理（防止 dangling）
        let reservations = env
            .reservations
            .list_by_saga(saga.id)
            .await
            .unwrap();
        assert_eq!(reservations.len(), 0, "dangling reservation should be deleted");

        // 余额未变
        let account = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.balance, TEST_AMOUNT - 1);
    }

    #[tokio::test]
    async fn orchestrator_compensate_passes_resource_id_to_handler() {
        // 验证：orchestrator.compensate 把每个已完成 step 的 resource_id 传给 handler
        // 通过自定义 step 记录收到的 resource_id 来验证
        struct RecordingHandler {
            name: String,
            last_resource_id: Arc<Mutex<Option<Option<Uuid>>>>,
        }
        #[async_trait::async_trait]
        impl SagaStepHandler for RecordingHandler {
            fn name(&self) -> &str {
                &self.name
            }
            async fn execute(&self, _saga: &mut Saga) -> Result<()> {
                Ok(())
            }
            async fn compensate(&self, _saga: &mut Saga, rid: Option<Uuid>) -> Result<()> {
                *self.last_resource_id.lock().unwrap() = Some(rid);
                Ok(())
            }
        }

        let sag_repo = Arc::new(InMemorySagaRepository::new());
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let rec = Arc::new(RecordingHandler {
            name: "rec".to_string(),
            last_resource_id: Arc::new(Mutex::new(None)),
        });
        let last = rec.last_resource_id.clone();
        let orch = SagaOrchestrator::new(
            sag_repo.clone() as Arc<dyn SagaRepository>,
            res_repo.clone() as Arc<dyn ReservationRepository>,
            vec![rec],
        );

        let mut saga = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-record".to_string(),
            vec!["rec".to_string()],
        );
        let target = Uuid::new_v4();
        saga.steps[0].resource_id = Some(target);
        // 手动 start + mark completed
        saga.start();
        sag_repo.save(&saga).await.unwrap();
        saga.steps[0].mark_completed();

        orch.compensate(&mut saga).await.unwrap();

        let recorded = *last.lock().unwrap();
        assert_eq!(recorded, Some(Some(target)), "compensate must pass step's resource_id");
    }
}
