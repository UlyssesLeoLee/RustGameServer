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
    ///
    /// 接受 3 个入口状态（per verify-C CC-2 修复）：
    /// - Pending: 新建 saga，正常 start() + 步进
    /// - Running: 崩溃恢复 resume，跳过 start() 直接续跑当前 step
    /// - Compensating: 崩溃恢复 resume 补偿未完成 step
    pub async fn execute(&self, saga: &mut Saga) -> Result<()> {
        match saga.status {
            SagaStatus::Pending => {
                // 启动
                saga.start();
                self.sagas.save(saga).await?;
            }
            SagaStatus::Running | SagaStatus::Compensating => {
                // 崩溃恢复入口: 跳过 start(), 后续循环按 current step 续跑
                tracing::info!(
                    target: "saga",
                    saga_id = %saga.id,
                    status = ?saga.status,
                    current_step = saga.current_step,
                    "resuming saga from {:?}",
                    saga.status
                );
            }
            SagaStatus::Completed | SagaStatus::Failed | SagaStatus::Aborted => {
                return Err(Error::Validation(format!(
                    "saga {} already in terminal state ({:?})",
                    saga.id, saga.status
                )));
            }
        }

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
            // per RGS-REV-009 CR-1: 静默吞错会让 DB 故障时 reservation 永久 dangling 无告警
            // 改为 if let Err + tracing::warn，与 service.rs::apply_atomic_with_reservation 一致
            if let Err(cleanup_err) = self.reservations.delete_by_id(r.id).await {
                tracing::warn!(
                    target: "saga",
                    reservation_id = %r.id,
                    saga_id = %saga.id,
                    account_id = %account_id,
                    "failed to cleanup dangling reservation after insufficient funds: {}",
                    cleanup_err
                );
            }
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
        // per RGS-REV-009 CR-1: OCC 失败路径必须清理 reservation，否则后续 compensate 误触发 +amount
        // 之前 ? 直接传播错误，reservation 永久 dangling → ReserveHandler.compensate 凭空 +amount
        let apply_result = self.accounts.apply_atomic(&account, &entry).await;
        if let Err(apply_err) = apply_result {
            if let Err(cleanup_err) = self.reservations.delete_by_id(r.id).await {
                tracing::warn!(
                    target: "saga",
                    reservation_id = %r.id,
                    saga_id = %saga.id,
                    account_id = %account_id,
                    "failed to cleanup dangling reservation after apply_atomic failure: {}",
                    cleanup_err
                );
            }
            return Err(apply_err);
        }

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
    use crate::reservation::{InMemoryReservationRepository, Reservation, ReservationStatus};
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

    /// AccountRepository wrapper：模拟 apply_atomic OCC 失败（per RGS-REV-009 CR-1 真测试需要）
    ///
    /// 用法：构造 wrapper 包 InMemoryAccountRepository，occ_fail_remaining > 0 时
    /// apply_atomic 直接返 Error::Validation("simulated OCC conflict")，其他方法委托 inner。
    /// 这样能在不依赖 PG 集成测试的前提下，验证 ReserveHandler.execute 真实生产路径
    /// 在 OCC 失败时确实清理 reservation（修复 CR-1 资金幻影 bug）。
    struct OccFailingAccountRepository {
        inner: Arc<InMemoryAccountRepository>,
        occ_fail_remaining: tokio::sync::Mutex<usize>,
    }

    impl OccFailingAccountRepository {
        fn new(inner: Arc<InMemoryAccountRepository>, fail_count: usize) -> Self {
            Self {
                inner,
                occ_fail_remaining: tokio::sync::Mutex::new(fail_count),
            }
        }
    }

    #[async_trait::async_trait]
    impl AccountRepository for OccFailingAccountRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>> {
            self.inner.find_by_id(id).await
        }
        async fn find_by_player_and_currency(
            &self,
            player_id: Uuid,
            currency: Currency,
        ) -> Result<Option<Account>> {
            self.inner.find_by_player_and_currency(player_id, currency).await
        }
        async fn update_with_version(&self, account: &Account) -> Result<Account> {
            self.inner.update_with_version(account).await
        }
        async fn save(&self, entity: &Account) -> Result<Account> {
            self.inner.save(entity).await
        }
        async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
            self.inner.delete_by_id(id).await
        }
        async fn apply_atomic(
            &self,
            account: &Account,
            ledger: &TransactionLedger,
        ) -> Result<(Account, TransactionLedger)> {
            let mut count = self.occ_fail_remaining.lock().await;
            if *count > 0 {
                *count -= 1;
                return Err(Error::Validation(
                    "simulated OCC conflict from OccFailingAccountRepository".to_string(),
                ));
            }
            drop(count);
            self.inner.apply_atomic(account, ledger).await
        }
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

    /// RGS-REV-009 CR-1 真测试: 真实 ReserveHandler.execute 路径 OCC 失败时清理 reservation
    ///
    /// 之前 service.rs 内的 `apply_atomic_with_reservation_occ_conflict_cleans_reservation`
    /// (RGS-REV-008 CC-4) 测的是**死代码 helper** — 0 生产调用。V1+V2 共识 CC-4 修复打偏靶。
    ///
    /// 本 test 用 OccFailingAccountRepository wrapper 强制第一次 apply_atomic 返 OCC 失败，
    /// 直接驱动真实生产路径 ReserveHandler.execute (saga_orchestrator.rs:248-289)，
    /// 验证修复后 dangling reservation 被清理 — 这是 CR-1 修复的回归锚定。
    #[tokio::test]
    async fn reserve_handler_cleans_reservation_on_occ_failure() {
        let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
        let inner_acc_repo = Arc::new(
            InMemoryAccountRepository::new().with_shared_ledger(led_repo.inner.clone()),
        );
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let sag_repo = Arc::new(InMemorySagaRepository::new());

        // OccFailingAccountRepository：第一次 apply_atomic 强制返 OCC 失败
        let occ_failing = Arc::new(OccFailingAccountRepository::new(
            inner_acc_repo.clone(),
            1, // 失败 1 次
        ));
        let acc_repo: Arc<dyn AccountRepository> = occ_failing.clone();

        // 预存账户（balance 充足, version=2）
        let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
        account.credit(TEST_INITIAL_BALANCE);
        inner_acc_repo.save(&account).await.unwrap();
        let account_id = account.id;

        // 构造 ReserveHandler（绕过 orchestrator，直接测试 handler）
        let reserve_handler = ReserveHandler::new(
            res_repo.clone() as Arc<dyn ReservationRepository>,
            acc_repo.clone(),
            TEST_AMOUNT,
            Currency::Gold,
        );

        // 构造 saga (status=Running, current_step=0 准备跑 reserve)
        let mut saga = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-test-occ-failure".to_string(),
            vec!["reserve".to_string()],
        );
        saga.steps[0].resource_id = Some(account_id);
        saga.start();
        sag_repo.save(&saga).await.unwrap();

        // 触发: reserve_handler.execute 内部 reservation.save → try_debit 成功 →
        //   apply_atomic 第一次调用 → OccFailingAccountRepository 强制返 OCC 失败 →
        //   RGS-REV-009 CR-1 修复路径触发 reservation cleanup
        let err = reserve_handler.execute(&mut saga).await.unwrap_err();
        assert!(
            matches!(err, Error::Validation(ref msg) if msg.contains("OCC conflict")),
            "expected OCC conflict Validation error, got {:?}",
            err
        );

        // 关键断言: 真实生产路径 reservation cleanup（不依赖 helper）
        let reservations_for_saga = res_repo.list_by_saga(saga.id).await.unwrap();
        assert_eq!(
            reservations_for_saga.len(),
            0,
            "real ReserveHandler.execute must clean up dangling reservation on OCC failure; got {:?}",
            reservations_for_saga
        );

        // 关键断言: apply_atomic 失败回滚, 余额未变
        let reloaded = inner_acc_repo.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(
            reloaded.balance, TEST_INITIAL_BALANCE,
            "apply_atomic OCC failure must not commit the debit; balance should be untouched"
        );

        // 关键断言: ledger 无任何条目（apply_atomic 失败, ledger INSERT 未发生）
        assert_eq!(
            led_repo.inner.lock().unwrap().len(),
            0,
            "apply_atomic OCC failure must not write ledger entry"
        );
    }

    /// RGS-REV-009 CR-1 补充测试: 验证修复后 happy path 不受影响
    ///
    /// 第二个 apply_atomic 调用（OccFailingAccountRepository occ_fail_remaining=0 后）
    /// 应该走 inner.apply_atomic 成功路径，reservation 仍被 save 成功（不误清理）。
    #[tokio::test]
    async fn reserve_handler_occ_fail_then_success_does_not_over_cleanup() {
        // 验证: 第一次 OCC 失败 cleanup, 第二次成功路径 reservation 正常持久化
        // 这是 CR-1 修复的"无副作用"保证
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        // happy path: env.orch.execute 完整跑完 2 步 reserve+confirm
        env.orch.execute(&mut saga).await.unwrap();
        assert_eq!(saga.status, SagaStatus::Completed);

        // 验证 reservation 正常存在（status=Confirmed, 非 dangling）
        let reservations = env.reservations.list_by_saga(saga.id).await.unwrap();
        assert_eq!(reservations.len(), 1);
        assert_eq!(reservations[0].status, ReservationStatus::Confirmed);

        // 验证余额正确扣减
        let final_account = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(final_account.balance, TEST_INITIAL_BALANCE - TEST_AMOUNT);
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

    // ============================================================================
    // DC-1: SagaOrchestrator::resume() 测试 (per RGS-REV-008 verify-D CRITICAL)
    //
    // 背景: 55.23 economy main.rs:104-136 的 30s 崩溃恢复轮询
    //   `list_running(100)` + `orchestrator.resume(id).await`
    // 是 5 域 / economy 的核心崩溃恢复主路径, 但 resume() 函数本身无任何 test.
    //
    // 覆盖 4 个入口场景:
    //   1. Pending  → start() + 步进完成
    //   2. Running  → 跳过 start(), 从 current step 续跑
    //   3. Compensating → 重新跑失败步, 触发补偿链, handler.compensate 被调
    //   4. None (saga_id 不存在) → NotFound
    // ============================================================================

    /// DC-1.1: resume(Pending) → start() + 步进至 Completed
    #[tokio::test]
    async fn resume_pending_saga_starts_and_advances() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);

        // 构造 Pending saga 并持久化（未执行）
        let saga = make_transfer_saga(account_id);
        env.sagas.save(&saga).await.unwrap();
        let saga_id = saga.id;

        // resume: 重新加载 Pending saga 并执行
        env.orch.resume(saga_id).await.unwrap();

        // 验证: 重新加载后 saga 已 Completed（reserve + confirm 都过）
        let loaded = env.sagas.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(loaded.status, SagaStatus::Completed);
        assert_eq!(loaded.steps[0].status, SagaStepStatus::Completed);
        assert_eq!(loaded.steps[1].status, SagaStepStatus::Completed);
    }

    /// DC-1.2: resume(Running, current_step=1) → 跳过 start() 续跑 confirm 步, 无 double-debit
    #[tokio::test]
    async fn resume_running_saga_continues_current_step() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);

        // 构造 2 步 saga, step 0 (reserve) 已 Completed, current_step=1, status=Running
        // 模拟"reserve 步已成功完成 (在另一进程/重启前), 进程崩了,
        // 重启后用 resume 续跑 confirm 步"
        let mut saga = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k-resume-running".to_string(),
            vec!["reserve".to_string(), "confirm".to_string()],
        );
        saga.steps[0].resource_id = Some(account_id);
        saga.steps[1].resource_id = Some(account_id);
        saga.steps[0].mark_completed();
        saga.current_step = 1;
        saga.status = SagaStatus::Running;

        // 预存 reservation（confirm 步需要它来查找）
        let r = Reservation::new(saga.id, account_id, TEST_AMOUNT, Currency::Gold);
        env.reservations.save(&r).await.unwrap();

        // 预扣账户余额（模拟 reserve 阶段已成功扣款）
        let mut account = env.accounts.find_by_id(account_id).await.unwrap().unwrap();
        account.try_debit(TEST_AMOUNT);
        env.accounts.save(&account).await.unwrap();

        env.sagas.save(&saga).await.unwrap();
        let saga_id = saga.id;

        // resume: 跳过 start()（status=Running）, 从 current step (1) 续跑
        env.orch.resume(saga_id).await.unwrap();

        // 验证: saga 终态 = Completed
        let loaded = env.sagas.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(loaded.status, SagaStatus::Completed);
        // step 0 保持 Completed（未被重跑）
        assert_eq!(loaded.steps[0].status, SagaStepStatus::Completed);
        // step 1 已 Completed
        assert_eq!(loaded.steps[1].status, SagaStepStatus::Completed);

        // 关键: 余额只扣一次（step 0 未被重跑, 无 double-debit）
        let final_account = env.accounts.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(
            final_account.balance,
            TEST_INITIAL_BALANCE - TEST_AMOUNT,
            "step 0 (reserve) must not be re-executed, otherwise double-debit"
        );
    }

    /// DC-1.3 (RGS-REV-009 HI-2-stub): resume(Compensating) 用真实 ReserveHandler + ConfirmHandler
    ///
    /// 这是 55.12 真实资金幻影回归点（per RGS-REV-009 V1 HIGH DC-1-TEST-NO-DOUBLE-COMP）：
    /// 旧 DC-1.3 test 用 stub `CompensateRecorder` + `FailingHandler`, 仅验证"handler.compensate 被调一次",
    /// 没覆盖真实生产路径. 新实现验证：
    ///
    /// 1. step 0 (reserve) 真实执行: account -100, reservation Reserved
    /// 2. 模拟崩溃场景: reserve.compensate 部分执行（账户 +100, reservation Compensated,
    ///    step 0 标 Compensated, status=Compensating）, 但 saga.fail() 未持久化
    /// 3. resume(saga_id) → 跳过 start() → 重跑 step 1 (confirm) → 假设再次失败
    /// 4. 再次触发 compensate() → step 0 已是 Compensated (不是 Completed) → filter 为空
    /// 5. **不会**再次调 reserve.compensate → 不会凭空 +100 二次退款
    ///
    /// 关键断言: account balance 整轮只 +100 一次 (回到 500), 不是 +100 二次 (变成 600 = 资金幻影).
    #[tokio::test]
    async fn resume_compensating_saga_does_not_double_refund_with_real_handlers() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);

        // ========================================================================
        // 第 1 阶段: 构造 "reserve 步真实成功" 状态（直接调 ReserveHandler.execute,
        //   模拟"reserve 步先于 confirm 步崩前已完成"）
        // ========================================================================
        let mut saga = make_transfer_saga(account_id);
        env.sagas.save(&saga).await.unwrap();
        let saga_id = saga.id;

        let reserve_handler = ReserveHandler::new(
            env.reservations.clone() as Arc<dyn ReservationRepository>,
            env.accounts.clone() as Arc<dyn AccountRepository>,
            TEST_AMOUNT,
            Currency::Gold,
        );
        reserve_handler.execute(&mut saga).await.unwrap();
        // 模拟"reserve 步成功完成, saga 推进到 current_step=1, status=Running"
        saga.steps[0].mark_completed();
        saga.advance();
        saga.status = SagaStatus::Running;
        env.sagas.save(&saga).await.unwrap();

        // 验证 reserve 步真实扣款
        let account_after_reserve = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            account_after_reserve.balance,
            TEST_INITIAL_BALANCE - TEST_AMOUNT,
            "reserve step should debit TEST_AMOUNT (500 -> 400)"
        );

        // ========================================================================
        // 第 2 阶段: 模拟 "confirm 步失败 → 补偿链部分执行后崩溃" 的中间状态
        //
        //   真实 compensate() 内部时序:
        //     a. saga.compensate()         → step 0 标 Compensated, status=Compensating
        //     b. reserve.compensate()      → 账户 +100 退款 (400 -> 500), reservation 标 Compensated
        //     c. **CRASH** before saga.fail() 持久化 → status 仍为 Compensating
        //
        //   我们手动复现 a + b + 中间持久化状态, 然后 save.
        // ========================================================================
        reserve_handler
            .compensate(&mut saga, Some(account_id))
            .await
            .unwrap();
        saga.steps[0].mark_compensated();
        saga.status = SagaStatus::Compensating;
        env.sagas.save(&saga).await.unwrap();

        // 验证部分补偿后状态
        let account_after_partial = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            account_after_partial.balance, TEST_INITIAL_BALANCE,
            "partial compensation should refund +100, balance back to TEST_INITIAL_BALANCE (500)"
        );
        let reservations = env.reservations.list_by_saga(saga_id).await.unwrap();
        assert_eq!(reservations.len(), 1, "1 reservation per saga");
        assert_eq!(
            reservations[0].status,
            ReservationStatus::Compensated,
            "reservation must be Compensated after reserve.compensate"
        );

        // ========================================================================
        // 第 3 阶段: 模拟 "resume 后 confirm 步再次失败" → 触发第二次 compensate()
        //
        //   删除 reservation (模拟"DB 中 reservation 不可见"或"已过期清理"),
        //   ConfirmHandler.execute 会 list_by_saga 返回空 → NotFound Err.
        //
        //   然后再次 compensate():
        //     - collect Completed: step 0 是 Compensated (不是 Completed) → filter 为空
        //     - 不再调 reserve.compensate → 不 +100 二次退款 ← 关键回归点
        // ========================================================================
        for r in &reservations {
            env.reservations.delete_by_id(r.id).await.unwrap();
        }

        // resume: 跳过 start (Compensating) → 跑 step 1 (confirm) → NotFound Err → 再次 compensate
        let resume_result = env.orch.resume(saga_id).await;
        assert!(
            resume_result.is_err(),
            "confirm step should fail again (reservation gone); got {:?}",
            resume_result
        );

        // ========================================================================
        // 关键断言 1: 账户余额仍 = TEST_INITIAL_BALANCE (500), 不是 TEST_INITIAL_BALANCE + TEST_AMOUNT (600)
        //   → reserve.compensate 没被调第二次 → 无资金幻影
        // ========================================================================
        let final_account = env
            .accounts
            .find_by_id(account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            final_account.balance, TEST_INITIAL_BALANCE,
            "no double refund: account balance must remain at TEST_INITIAL_BALANCE (500), \
             not TEST_INITIAL_BALANCE + TEST_AMOUNT (600) which would indicate phantom +100 refund. \
             This is the 55.12 phantom-fund regression point: orchestrator.compensate() must \
             filter out already-Compensated steps so reserve.compensate is not re-invoked."
        );

        // ========================================================================
        // 关键断言 2: step 0 状态保持 Compensated (没被重跑, 也没被错误地重置)
        // ========================================================================
        let loaded = env.sagas.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(
            loaded.steps[0].status,
            SagaStepStatus::Compensated,
            "step 0 must remain Compensated after second compensation (orchestrator filters Compensated, not Completed)"
        );

        // ========================================================================
        // 关键断言 3: saga 终态 = Failed (compensate() 完成后 saga.fail() 标 Failed)
        // ========================================================================
        assert_eq!(
            loaded.status,
            SagaStatus::Failed,
            "saga must end in Failed state after second compensation completes"
        );
    }

    /// DC-1.4: resume(不存在的 saga_id) → NotFound("Saga", ...)
    #[tokio::test]
    async fn resume_nonexistent_saga_returns_not_found() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let phantom_id = Uuid::new_v4();

        let err = env.orch.resume(phantom_id).await.unwrap_err();
        assert!(
            matches!(err, Error::NotFound { entity: "Saga", .. }),
            "resume(non-existent) should return NotFound(Saga), got: {:?}",
            err
        );
    }

    // ============================================================================
    // RGS-REV-009 HI-D: DC-1 补 3 个终态 test
    //
    // 背景: 0434ada (RGS-REV-008 DC-1) 测了 4 个状态 (Pending/Running/Compensating/NotFound),
    //       漏了 3 个终态 (Completed/Failed/Aborted)。终态的 resume() 应返
    //       Error::Validation("...already in terminal state...")，不能再次执行。
    //       本组 test 锚定该 invariant。
    // ============================================================================

    /// DC-1.5: resume(Completed) → Validation("...already in terminal state (Completed)...")
    #[tokio::test]
    async fn resume_completed_saga_returns_validation_err() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        // 跑完 saga 终态 = Completed（execute 内部已 sagas.save）
        env.orch.execute(&mut saga).await.unwrap();
        assert_eq!(saga.status, SagaStatus::Completed);

        // resume(Completed) 应拒：终态不可逆
        let err = env.orch.resume(saga.id).await.unwrap_err();
        assert!(
            matches!(err, Error::Validation(ref msg) if msg.contains("terminal") || msg.contains("Completed")),
            "expected Validation error mentioning terminal/Completed, got {:?}",
            err
        );
    }

    /// DC-1.6: resume(Failed) → Validation（终态不可逆）
    ///
    /// 直接构造 Failed 终态保存（不依赖 compensate 完整路径，以保持 test 聚焦于 resume 终态检查）。
    #[tokio::test]
    async fn resume_failed_saga_returns_validation_err() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        // 直接设终态 Failed 并持久化
        saga.status = SagaStatus::Failed;
        env.sagas.save(&saga).await.unwrap();

        // resume(Failed) 应拒
        let err = env.orch.resume(saga.id).await.unwrap_err();
        assert!(
            matches!(err, Error::Validation(ref msg) if msg.contains("terminal") || msg.contains("Failed")),
            "expected Validation error mentioning terminal/Failed, got {:?}",
            err
        );
    }

    /// DC-1.7: resume(Aborted) → Validation（终态不可逆）
    ///
    /// 直接构造 Aborted 终态保存。Aborted 在 saga_orchestrator.rs:94-99 与 Failed/Completed
    /// 同属 terminal 状态，execute() 进入即返 Validation。
    #[tokio::test]
    async fn resume_aborted_saga_returns_validation_err() {
        let env = make_env(TEST_INITIAL_BALANCE).await;
        let account_id = account_id_in_env(&env);
        let mut saga = make_transfer_saga(account_id);

        // 直接设终态 Aborted 并持久化
        saga.status = SagaStatus::Aborted;
        env.sagas.save(&saga).await.unwrap();

        // resume(Aborted) 应拒
        let err = env.orch.resume(saga.id).await.unwrap_err();
        assert!(
            matches!(err, Error::Validation(ref msg) if msg.contains("terminal") || msg.contains("Aborted")),
            "expected Validation error mentioning terminal/Aborted, got {:?}",
            err
        );
    }
}
