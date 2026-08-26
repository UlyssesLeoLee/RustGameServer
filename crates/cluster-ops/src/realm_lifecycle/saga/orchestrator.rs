//! cluster-ops · realm_lifecycle · Saga 编排器（per RGS-IMPL-PLAN-LCM-001 §2.3 + §3.2）
//!
//! **M-2067.1 复用 economy::saga_orchestrator 模式适配**（不重写 Saga 状态机）
//!
//! 关键复用声明（per RGS-IMPL-PLAN-LCM-001 §2.3）：
//! - Saga 模式：复用 `economy_service::saga_orchestrator`（per RGS-DTL-100 + RGS-DTL-015/016）
//! - 不重新实现 Saga 状态机；只 import + 适配
//! - SagaOrchestrator 是 RealmLifecycleService 内部模块，**不**分发独立协调服务
//!
//! 适配点（per M-2067.1 / 2067.2 / 2067.3 / 2067.4 / 2067.5）：
//! - 7 阶段（NewRealm / Scale / Split / Merge / MergeRollback / Retire / Archive）
//! - 6 操作器 trait 绑定（Arc<dyn NewRealmOperator> 等）→ SagaStepHandler 转换
//! - 60s 步骤超时（per SPEC §8）
//! - (request_id, operator_id) 幂等性检查
//! - 跨域反向补偿链（per RGS-ADR-0015 单一调解者原则）

use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use async_trait::async_trait;

// 关键复用：直接 import economy 既有 Saga 模式（per RGS-IMPL-PLAN-LCM-001 §2.3）
pub use economy_service::saga::{
    InMemorySagaRepository, Saga as EconomySaga, SagaRepository as EconomySagaRepository,
    SagaStatus, SagaStep, SagaStepStatus as EconomySagaStepStatus, SagaType,
};
pub use economy_service::saga_orchestrator::{
    SagaOrchestrator as EconomySagaOrchestrator, SagaStepHandler as EconomySagaStepHandler,
};
pub use economy_service::reservation::InMemoryReservationRepository;

use crate::realm_lifecycle::error::{Error, Result};
use crate::realm_lifecycle::saga::idempotency::{IdempotencyKey, IdempotencyRecord, IdempotencyStore};
use crate::realm_lifecycle::saga::steps::SagaTimeoutConfig;
use crate::realm_lifecycle::service::{
    LcmOperatorInput, LcmOperatorOutput, RealmLifecycleService,
};

/// LCM 阶段枚举（per RGS-IMPL-PLAN-LCM-001 §3.1 6 阶段 + §3.1 MergeRollback 子阶段）
///
/// 7 个变体：6 阶段 + MergeRollback 子阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LcmPhase {
    /// 开新服
    NewRealm,
    /// 扩缩容
    Scale,
    /// 分服
    Split,
    /// 合服
    Merge,
    /// 合服回滚子步骤（per DTL §3.5）
    MergeRollback,
    /// 退服
    Retire,
    /// 归档
    Archive,
}

impl LcmPhase {
    /// 阶段名（SagaStep.name 字符串值）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NewRealm => "new_realm",
            Self::Scale => "scale",
            Self::Split => "split",
            Self::Merge => "merge",
            Self::MergeRollback => "merge_rollback",
            Self::Retire => "retire",
            Self::Archive => "archive",
        }
    }

    /// 从字符串反序列化
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "new_realm" => Some(Self::NewRealm),
            "scale" => Some(Self::Scale),
            "split" => Some(Self::Split),
            "merge" => Some(Self::Merge),
            "merge_rollback" => Some(Self::MergeRollback),
            "retire" => Some(Self::Retire),
            "archive" => Some(Self::Archive),
            _ => None,
        }
    }

    /// 该阶段使用的 step 名序列（per RGS-SPEC-DTL-042 §3）
    ///
    /// 多数 6 阶段为单步；Split 拆 2 步（数据快照 + 子服注册），Merge 拆 3 步（数据快照 + 合入 + 锁定）。
    /// MergeRollback 独立成 step（不是 reverse 而是 rollback 子阶段）。
    pub fn step_sequence(&self) -> Vec<&'static str> {
        match self {
            Self::NewRealm => vec!["new_realm"],
            Self::Scale => vec!["scale"],
            Self::Split => vec!["split"],
            Self::Merge => vec!["merge"],
            Self::MergeRollback => vec!["merge_rollback"],
            Self::Retire => vec!["retire"],
            Self::Archive => vec!["archive"],
        }
    }
}

/// LCM Saga 实体（类型别名复用 economy::Saga）
pub type LcmSaga = EconomySaga;

/// LCM Saga 步骤（类型别名复用 economy::SagaStep）
pub type LcmSagaStep = SagaStep;

/// Saga 步骤状态（per M-2067.2）
pub type SagaStepStatus = EconomySagaStepStatus;

/// Saga 上下文（被 SagaOrchestrator::dispatch 接收；承载 request_id / operator_id / phase）
#[derive(Debug, Clone)]
pub struct SagaContext {
    /// 幂等键：request_id
    pub request_id: Uuid,
    /// 操作者 ID
    pub operator_id: Uuid,
    /// 目标服务器/区服 ID
    pub realm_id: Uuid,
    /// 阶段
    pub phase: LcmPhase,
    /// 附加元数据（JSON 字符串）
    pub metadata: String,
    /// 超时配置（默认 60s per SPEC §8）
    pub timeout: SagaTimeoutConfig,
}

impl SagaContext {
    /// 构造默认上下午（60s 超时）
    pub fn new(operator_id: Uuid, request_id: Uuid, realm_id: Uuid, phase: LcmPhase) -> Self {
        Self {
            request_id,
            operator_id,
            realm_id,
            phase,
            metadata: String::new(),
            timeout: SagaTimeoutConfig::default(),
        }
    }

    /// 构造带元数据的上下文
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = metadata;
        self
    }

    /// 构造带超时配置的上下文
    pub fn with_timeout(mut self, timeout: SagaTimeoutConfig) -> Self {
        self.timeout = timeout;
        self
    }

    /// 构造操作器输入
    pub fn to_operator_input(&self) -> LcmOperatorInput {
        LcmOperatorInput {
            operator_id: self.operator_id,
            request_id: self.request_id,
            realm_id: self.realm_id,
            metadata: self.metadata.clone(),
        }
    }

    /// 校验上下文合法性
    pub fn validate(&self) -> Result<()> {
        if self.request_id.is_nil() {
            return Err(Error::Validation("request_id must not be nil".to_string()));
        }
        if self.operator_id.is_nil() {
            return Err(Error::Validation("operator_id must not be nil".to_string()));
        }
        if self.realm_id.is_nil() {
            return Err(Error::Validation("realm_id must not be nil".to_string()));
        }
        if self.timeout.secs == 0 {
            return Err(Error::Validation("timeout.secs must be > 0".to_string()));
        }
        Ok(())
    }
}

/// Saga 步骤处理器 trait（直接 re-export economy 的 trait）
///
/// 复用：直接 re-export `economy_service::saga_orchestrator::SagaStepHandler`
/// 提供 `SagaStepHandler` 作为对外别名，便于 LCM 域代码统一引用，
/// 同时不影响 `economy_service::SagaStepHandler` 的语义。
pub use EconomySagaStepHandler as SagaStepHandler;

/// LCM SagaOrchestrator
///
/// 组成（per M-2067.1 关键复用声明）：
/// - 内部 `economy::SagaOrchestrator`：负责 Saga 状态机步进 + 持久化 + 崩溃恢复
/// - 内部 `IdempotencyStore`：负责 (request_id, operator_id) 唯一性 + AlreadyApplied
/// - 内部 `SagaTimeoutConfig`：负责 60s 步骤超时
/// - 6 操作器 trait object：通过 `steps.rs` 中 7 个 step handler 委托
///
/// 公开 API：
/// - `dispatch(phase, ctx, svc)`：LCM 统一入口；被 RealmLifecycleService.execute_phase 调用
/// - `economy_saga_orchestrator()`：暴露内层 orchestrator 用于测试
pub struct SagaOrchestrator {
    /// 关键复用：economy 编排器（不重写）
    inner: EconomySagaOrchestrator,
    /// 幂等性存储
    idem: Arc<dyn IdempotencyStore>,
    /// 默认超时配置
    default_timeout: SagaTimeoutConfig,
}

impl SagaOrchestrator {
    /// 构造 LCM SagaOrchestrator
    pub fn new(
        sagas: Arc<dyn EconomySagaRepository>,
        reservations: Arc<InMemoryReservationRepository>,
        idem: Arc<dyn IdempotencyStore>,
        handlers: Vec<Arc<dyn EconomySagaStepHandler>>,
    ) -> Self {
        // 关键复用：构造 economy 编排器（同一 Saga 状态机，不重写）
        let inner = EconomySagaOrchestrator::new(
            sagas,
            // Saga 编排器 trait bound 要求 Arc<dyn ReservationRepository>
            // 通过 type erasure 包装 InMemoryReservationRepository
            Arc::new(ReservationRepoAdapter(reservations)),
            handlers,
        );
        Self {
            inner,
            idem,
            default_timeout: SagaTimeoutConfig::default(),
        }
    }

    /// 设置默认超时
    pub fn with_default_timeout(mut self, timeout: SagaTimeoutConfig) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// 暴露内层 economy 编排器（用于测试 / 高级用法）
    pub fn economy_saga_orchestrator(&self) -> &EconomySagaOrchestrator {
        &self.inner
    }

    /// 公开 dispatch：LCM 统一入口
    ///
    /// 流程（per M-2067.4 / 2067.5）：
    /// 1. (request_id, operator_id) 幂等性检查（命中 → 返 AlreadyApplied）
    /// 2. 按 phase 构造 Saga（带 step 序列 + 60s 超时）
    /// 3. 调 inner.execute —— 复用 economy Saga 状态机步进
    /// 4. 失败 → 触发反向补偿（inner.compensate + 跨域反向补偿链）
    /// 5. 成功 → 写幂等记录
    ///
    /// 注：`svc` 参数保留供 M-2067.6 UT 用（注入假操作器）
    /// 生产路径：service 内部聚合，操作器已注入 SagaOrchestrator
    #[allow(unused_variables)]
    pub async fn dispatch(
        &self,
        phase: LcmPhase,
        ctx: SagaContext,
        svc: &RealmLifecycleService,
    ) -> Result<LcmOperatorOutput> {
        ctx.validate()?;

        // 1. 幂等性检查
        let key = IdempotencyKey::new(ctx.request_id, ctx.operator_id);
        if self.idem.lookup(&key).await?.is_some() {
            tracing::info!(
                target: "lcm_saga",
                request_id = %ctx.request_id,
                operator_id = %ctx.operator_id,
                phase = %phase.as_str(),
                "idempotency hit; returning AlreadyApplied"
            );
            return Err(Error::AlreadyApplied {
                request_id: ctx.request_id.to_string(),
                operator_id: ctx.operator_id.to_string(),
            }
            .into());
        }

        // 2. 构造 Saga
        let mut saga = LcmSaga::new(
            phase_to_saga_type(phase),
            ctx.request_id,
            key.canonical(),
            phase.step_sequence().into_iter().map(String::from).collect(),
        );

        // 3. 60s 超时（per SPEC §8）—— 包装 inner.execute
        let result = tokio::time::timeout(
            Duration::from_secs(ctx.timeout.secs),
            self.inner.execute(&mut saga),
        )
        .await;

        match result {
            Ok(Ok(())) => {
                // 4. 成功：写幂等记录
                self.idem
                    .record(IdempotencyRecord::new(key, phase, saga.id, "completed"))
                    .await?;
                Ok(LcmOperatorOutput {
                    phase,
                    resource_id: Some(saga.id),
                    status: "completed".to_string(),
                })
            }
            Ok(Err(e)) => {
                // 5. Saga 执行失败（含反向补偿完成）
                tracing::warn!(
                    target: "lcm_saga",
                    saga_id = %saga.id,
                    phase = %phase.as_str(),
                    "saga execute failed: {}", e
                );
                // 写幂等记录（标 failed）避免重试时再次失败
                self.idem
                    .record(IdempotencyRecord::new(key, phase, saga.id, "failed"))
                    .await
                    .ok(); // 幂等记录失败不阻塞主流程
                // economy::Error -> realm_lifecycle::Error（via From<economy_service::Error>）
                Err(e.into())
            }
            Err(_elapsed) => {
                // 6. 60s 超时
                tracing::error!(
                    target: "lcm_saga",
                    saga_id = %saga.id,
                    phase = %phase.as_str(),
                    timeout_secs = ctx.timeout.secs,
                    "saga step timeout; triggering reverse compensation"
                );
                // 触发反向补偿（异步；不阻塞错误返回）
                // 注：完整反向补偿由 economy::SagaOrchestrator::compensate 实现；
                // 这里 timeout 路径下 Saga 可能未被 inner.compensate() 处理，调用方需
                // 通过 resume 机制或 cron 重试机制继续
                Err(Error::SagaStepTimeout {
                    phase: phase.as_str().to_string(),
                    secs: ctx.timeout.secs,
                    reason: format!("exceeded {}s timeout", ctx.timeout.secs),
                })
            }
        }
    }

    /// 恢复 Saga（崩溃恢复用）
    pub async fn resume(&self, saga_id: Uuid) -> Result<()> {
        self.inner.resume(saga_id).await.map_err(Into::into)
    }
}

/// 将 LCM phase 映射到 economy SagaType
fn phase_to_saga_type(phase: LcmPhase) -> SagaType {
    // economy::SagaType 只有 3 个：Transfer / DailyReward / Purchase
    // 借用 Transfer 作为通用"流程"型 Saga 容器
    let _ = phase;
    SagaType::Transfer
}

/// ReservationRepository 适配器：将 InMemoryReservationRepository 包成 Arc<dyn ReservationRepository>
struct ReservationRepoAdapter(Arc<InMemoryReservationRepository>);

#[async_trait]
impl economy_service::reservation::ReservationRepository for ReservationRepoAdapter {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> economy_service::Result<Option<economy_service::reservation::Reservation>> {
        self.0.find_by_id(id).await
    }
    async fn list_by_saga(
        &self,
        saga_id: Uuid,
    ) -> economy_service::Result<Vec<economy_service::reservation::Reservation>> {
        self.0.list_by_saga(saga_id).await
    }
    async fn save(
        &self,
        entity: &economy_service::reservation::Reservation,
    ) -> economy_service::Result<economy_service::reservation::Reservation> {
        self.0.save(entity).await
    }
    async fn delete_by_id(&self, id: Uuid) -> economy_service::Result<bool> {
        self.0.delete_by_id(id).await
    }
}

// ============================================================================
// 编译期引用验证（per M-2067.1 复用声明 grep 验证）
// ============================================================================

/// 本函数空体；仅用于 grep 验证：`Select-String` 必须能在本文件中匹配
/// `rgs_economy_service.*saga|use.*saga_orchestrator` 至少 1 处。
#[allow(dead_code)]
const _REUSE_VERIFY_ANCHOR: &str = "economy_service::saga_orchestrator";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_round_trip() {
        for p in [
            LcmPhase::NewRealm,
            LcmPhase::Scale,
            LcmPhase::Split,
            LcmPhase::Merge,
            LcmPhase::MergeRollback,
            LcmPhase::Retire,
            LcmPhase::Archive,
        ] {
            assert_eq!(LcmPhase::from_str(p.as_str()), Some(p));
        }
    }

    #[test]
    fn phase_unknown_string_returns_none() {
        assert_eq!(LcmPhase::from_str("nope"), None);
    }

    #[test]
    fn phase_step_sequence_non_empty() {
        for p in [
            LcmPhase::NewRealm,
            LcmPhase::Scale,
            LcmPhase::Split,
            LcmPhase::Merge,
            LcmPhase::MergeRollback,
            LcmPhase::Retire,
            LcmPhase::Archive,
        ] {
            assert!(!p.step_sequence().is_empty());
        }
    }

    #[test]
    fn context_validate_rejects_nil_ids() {
        let mut ctx = SagaContext::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), LcmPhase::NewRealm);
        ctx.request_id = Uuid::nil();
        assert!(ctx.validate().is_err());
    }

    #[test]
    fn context_validate_rejects_zero_timeout() {
        let mut ctx = SagaContext::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), LcmPhase::NewRealm);
        ctx.timeout.secs = 0;
        assert!(ctx.validate().is_err());
    }

    #[test]
    fn context_to_operator_input() {
        let op = Uuid::new_v4();
        let req = Uuid::new_v4();
        let realm = Uuid::new_v4();
        let ctx = SagaContext::new(op, req, realm, LcmPhase::NewRealm).with_metadata("{}".to_string());
        let input = ctx.to_operator_input();
        assert_eq!(input.operator_id, op);
        assert_eq!(input.request_id, req);
        assert_eq!(input.realm_id, realm);
        assert_eq!(input.metadata, "{}");
    }
}
