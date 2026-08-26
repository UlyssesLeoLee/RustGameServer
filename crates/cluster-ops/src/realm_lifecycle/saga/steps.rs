//! cluster-ops · realm_lifecycle · 6 阶段 Saga 步骤 + 反向补偿（per M-2067.2/3/5）
//!
//! **M-2067.2** 6 阶段 Saga 步骤定义（含 SagaStep + CompensateAction）
//! **M-2067.3** 反向补偿步骤（含跨域 Saga 反向补偿链）
//! **M-2067.5** Saga 步骤超时（默认 60s per SPEC §8）触发反向补偿
//!
//! 设计：
//! - 7 个 step handler struct（6 阶段 + MergeRollback 子阶段）
//! - 每个 step 实现 `economy_service::saga_orchestrator::SagaStepHandler` trait
//! - 每个 step 内置反向补偿（`compensate` 方法）
//! - `CompensateAction` 枚举：标识补偿方向（reverse / rollback / cleanup）
//! - 60s 超时通过 `tokio::time::timeout` 在 orchestrator 包装（per M-2067.5）
//! - 真实业务逻辑属 WF-1-2066 / WF-1-2070 / WF-1-2071；本 worktree 只提供 Saga 编排接口

use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use async_trait::async_trait;

use economy_service::saga_orchestrator::SagaStepHandler as EconomySagaStepHandler;
pub use economy_service::saga::SagaStepStatus as LcmSagaStepStatus;

use crate::realm_lifecycle::saga::orchestrator::{LcmPhase, LcmSaga};
use crate::realm_lifecycle::service::{
    ArchiveOperator, LcmOperatorInput, MergeOperator, NewRealmOperator, RealmLifecycleService,
    RetireOperator, ScaleOperator, SplitOperator,
};

// Result 类型别名：必须用 economy_service 因为 EconomySagaStepHandler trait 要求
use economy_service::Result;

// ============================================================================
// CompensateAction 枚举（per M-2067.2）
// ============================================================================

/// 反向补偿动作类型（per RGS-IMPL-PLAN-LCM-001 §3.2 关键复用声明）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompensateAction {
    /// 反向操作（如 NewRealm 失败 → 回收已分配资源）
    Reverse,
    /// 回滚（如 Merge 锁定后撤回 → 重新合并）
    Rollback,
    /// 清理（资源释放、数据快照删除等）
    Cleanup,
    /// 跨域反向补偿链（per RGS-ADR-0015 单一调解者原则）
    /// LCM 编排器在跨域失败时调用其他域的补偿
    CrossDomainReverse,
    /// 跨域回滚
    CrossDomainRollback,
}

impl CompensateAction {
    /// 字符串名（per trace 日志）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reverse => "reverse",
            Self::Rollback => "rollback",
            Self::Cleanup => "cleanup",
            Self::CrossDomainReverse => "cross_domain_reverse",
            Self::CrossDomainRollback => "cross_domain_rollback",
        }
    }
}

// ============================================================================
// SagaTimeoutConfig（per M-2067.5）
// ============================================================================

/// Saga 步骤超时配置（per RGS-SPEC-DTL-042 §8）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SagaTimeoutConfig {
    /// 单步超时秒数（默认 60s）
    pub secs: u64,
}

impl Default for SagaTimeoutConfig {
    fn default() -> Self {
        Self { secs: 60 }
    }
}

impl SagaTimeoutConfig {
    pub const DEFAULT_SECS: u64 = 60;

    /// 工厂：自定义超时
    pub fn new(secs: u64) -> Self {
        Self { secs }
    }

    /// 转换为 Duration
    pub fn to_duration(&self) -> Duration {
        Duration::from_secs(self.secs)
    }
}

// ============================================================================
// 7 阶段 step handler（per M-2067.2 + M-2067.3）
// ============================================================================
//
// 7 个 step handler（NewRealm / Scale / Split / Merge / MergeRollback / Retire / Archive）
// 全部手工展开以提供清晰类型签名 + Saga 编排接口。
// 真实业务逻辑（resource_id 处理等）由 WF-1-2066 填充；本 worktree 仅提供 Saga 编排接口。

/// NewRealm step handler（开新服）
pub struct NewRealmStep {
    ops: Arc<dyn NewRealmOperator>,
}

impl NewRealmStep {
    pub fn new(ops: Arc<dyn NewRealmOperator>) -> Self {
        Self { ops }
    }
}

#[async_trait]
impl EconomySagaStepHandler for NewRealmStep {
    fn name(&self) -> &str {
        LcmPhase::NewRealm.as_str()
    }

    async fn execute(&self, saga: &mut LcmSaga) -> Result<()> {
        // 构造输入 + 调操作器
        let input = LcmOperatorInput {
            operator_id: parse_metadata_operator(saga)?,
            request_id: saga.command_id,
            realm_id: Uuid::nil(), // 真实 realm_id 由 orchestrator 通过 metadata 注入；本 worktree 占位
            metadata: saga.idempotency_key.clone(),
        };
        match self.ops.open(input).await {
            Ok(output) => {
                if let Some(rid) = output.resource_id {
                    saga.steps[saga.current_step].resource_id = Some(rid);
                }
                Ok(())
            }
            Err(e) => {
                // 反向补偿链触发点：NewRealm 失败 → 回收已分配资源（per M-2067.3）
                trigger_reverse_compensation(LcmPhase::NewRealm, CompensateAction::Reverse);
                Err(e.into())
            }
        }
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        resource_id: Option<Uuid>,
    ) -> Result<()> {
        // 反向：回收已开服资源
        let rid = resource_id.unwrap_or_else(Uuid::nil);
        self.ops
            .reverse(rid, "NewRealm step failed".to_string())
            .await
            .map_err(Into::into)
    }
}

/// Scale step handler（扩缩容）
pub struct ScaleStep {
    ops: Arc<dyn ScaleOperator>,
    /// 扩缩容 delta（> 0 扩容；< 0 缩容）
    pub delta: i32,
}

impl ScaleStep {
    pub fn new(ops: Arc<dyn ScaleOperator>, delta: i32) -> Self {
        Self { ops, delta }
    }
}

#[async_trait]
impl EconomySagaStepHandler for ScaleStep {
    fn name(&self) -> &str {
        LcmPhase::Scale.as_str()
    }

    async fn execute(&self, saga: &mut LcmSaga) -> Result<()> {
        let input = LcmOperatorInput {
            operator_id: parse_metadata_operator(saga)?,
            request_id: saga.command_id,
            realm_id: Uuid::nil(),
            metadata: saga.idempotency_key.clone(),
        };
        match self.ops.scale(input, self.delta).await {
            Ok(output) => {
                if let Some(rid) = output.resource_id {
                    saga.steps[saga.current_step].resource_id = Some(rid);
                }
                Ok(())
            }
            Err(e) => {
                trigger_reverse_compensation(LcmPhase::Scale, CompensateAction::Reverse);
                Err(e.into())
            }
        }
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        _resource_id: Option<Uuid>,
    ) -> Result<()> {
        // 反向：回滚到扩缩容前状态（delta 取反）
        let prior = -self.delta;
        let input = LcmOperatorInput {
            operator_id: Uuid::nil(),
            request_id: Uuid::nil(),
            realm_id: Uuid::nil(),
            metadata: String::new(),
        };
        self.ops.reverse(input.realm_id, prior).await.map_err(Into::into)
    }
}

/// Split step handler（分服）
pub struct SplitStep {
    ops: Arc<dyn SplitOperator>,
    pub target_realm_id: Uuid,
}

impl SplitStep {
    pub fn new(ops: Arc<dyn SplitOperator>, target_realm_id: Uuid) -> Self {
        Self {
            ops,
            target_realm_id,
        }
    }
}

#[async_trait]
impl EconomySagaStepHandler for SplitStep {
    fn name(&self) -> &str {
        LcmPhase::Split.as_str()
    }

    async fn execute(&self, saga: &mut LcmSaga) -> Result<()> {
        let input = LcmOperatorInput {
            operator_id: parse_metadata_operator(saga)?,
            request_id: saga.command_id,
            realm_id: self.target_realm_id,
            metadata: saga.idempotency_key.clone(),
        };
        match self.ops.split(input, self.target_realm_id).await {
            Ok(output) => {
                if let Some(rid) = output.resource_id {
                    saga.steps[saga.current_step].resource_id = Some(rid);
                }
                Ok(())
            }
            Err(e) => {
                trigger_reverse_compensation(LcmPhase::Split, CompensateAction::Reverse);
                Err(e.into())
            }
        }
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        _resource_id: Option<Uuid>,
    ) -> Result<()> {
        // 反向：合回源服
        let source_realm_id = Uuid::nil(); // 占位：真实从 metadata 解析
        self.ops
            .reverse(source_realm_id, self.target_realm_id)
            .await
            .map_err(Into::into)
    }
}

/// Merge step handler（合服）
pub struct MergeStep {
    ops: Arc<dyn MergeOperator>,
    pub target_realm_id: Uuid,
    pub source_realm_ids: Vec<Uuid>,
}

impl MergeStep {
    pub fn new(
        ops: Arc<dyn MergeOperator>,
        target_realm_id: Uuid,
        source_realm_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            ops,
            target_realm_id,
            source_realm_ids,
        }
    }
}

#[async_trait]
impl EconomySagaStepHandler for MergeStep {
    fn name(&self) -> &str {
        LcmPhase::Merge.as_str()
    }

    async fn execute(&self, saga: &mut LcmSaga) -> Result<()> {
        let input = LcmOperatorInput {
            operator_id: parse_metadata_operator(saga)?,
            request_id: saga.command_id,
            realm_id: self.target_realm_id,
            metadata: saga.idempotency_key.clone(),
        };
        match self
            .ops
            .merge(input, self.target_realm_id, self.source_realm_ids.clone())
            .await
        {
            Ok(output) => {
                if let Some(rid) = output.resource_id {
                    saga.steps[saga.current_step].resource_id = Some(rid);
                }
                Ok(())
            }
            Err(e) => {
                // 跨域反向补偿链（per RGS-ADR-0015）：合服失败可能涉及 player / social / economy 域
                trigger_cross_domain_reverse(LcmPhase::Merge, &["player", "social", "economy"]);
                Err(e.into())
            }
        }
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        _resource_id: Option<Uuid>,
    ) -> Result<()> {
        self.ops
            .reverse(self.target_realm_id, self.source_realm_ids.clone())
            .await
            .map_err(Into::into)
    }
}

/// MergeRollback step handler（合服回滚子步骤，独立 step 不是 reverse）
pub struct MergeRollbackStep {
    ops: Arc<dyn MergeOperator>,
    pub target_realm_id: Uuid,
    pub locked_at_ms: i64,
}

impl MergeRollbackStep {
    pub fn new(ops: Arc<dyn MergeOperator>, target_realm_id: Uuid, locked_at_ms: i64) -> Self {
        Self {
            ops,
            target_realm_id,
            locked_at_ms,
        }
    }
}

#[async_trait]
impl EconomySagaStepHandler for MergeRollbackStep {
    fn name(&self) -> &str {
        LcmPhase::MergeRollback.as_str()
    }

    async fn execute(&self, _saga: &mut LcmSaga) -> Result<()> {
        // MergeRollback 不走 saga execute 路径（由 Merge.rollback 直接调用）
        // 这里仅占位；真实路径：合服窗口期内（7-30 天）管理员发起 rollback
        self.ops
            .rollback(self.target_realm_id, self.locked_at_ms)
            .await
            .map_err(Into::into)
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        _resource_id: Option<Uuid>,
    ) -> Result<()> {
        // 跨域回滚（per M-2067.3 跨域 Saga 反向）
        trigger_cross_domain_reverse(LcmPhase::MergeRollback, &["player", "social", "economy"]);
        Ok(())
    }
}

/// Retire step handler（退服）
pub struct RetireStep {
    ops: Arc<dyn RetireOperator>,
}

impl RetireStep {
    pub fn new(ops: Arc<dyn RetireOperator>) -> Self {
        Self { ops }
    }
}

#[async_trait]
impl EconomySagaStepHandler for RetireStep {
    fn name(&self) -> &str {
        LcmPhase::Retire.as_str()
    }

    async fn execute(&self, saga: &mut LcmSaga) -> Result<()> {
        let input = LcmOperatorInput {
            operator_id: parse_metadata_operator(saga)?,
            request_id: saga.command_id,
            realm_id: Uuid::nil(),
            metadata: saga.idempotency_key.clone(),
        };
        match self.ops.retire(input).await {
            Ok(output) => {
                if let Some(rid) = output.resource_id {
                    saga.steps[saga.current_step].resource_id = Some(rid);
                }
                Ok(())
            }
            Err(e) => {
                trigger_reverse_compensation(LcmPhase::Retire, CompensateAction::Reverse);
                Err(e.into())
            }
        }
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        resource_id: Option<Uuid>,
    ) -> Result<()> {
        // 反向：恢复已退服
        self.ops
            .reverse(resource_id.unwrap_or_else(Uuid::nil))
            .await
            .map_err(Into::into)
    }
}

/// Archive step handler（归档）
pub struct ArchiveStep {
    ops: Arc<dyn ArchiveOperator>,
}

impl ArchiveStep {
    pub fn new(ops: Arc<dyn ArchiveOperator>) -> Self {
        Self { ops }
    }
}

#[async_trait]
impl EconomySagaStepHandler for ArchiveStep {
    fn name(&self) -> &str {
        LcmPhase::Archive.as_str()
    }

    async fn execute(&self, saga: &mut LcmSaga) -> Result<()> {
        let input = LcmOperatorInput {
            operator_id: parse_metadata_operator(saga)?,
            request_id: saga.command_id,
            realm_id: Uuid::nil(),
            metadata: saga.idempotency_key.clone(),
        };
        match self.ops.archive(input).await {
            Ok(output) => {
                if let Some(rid) = output.resource_id {
                    saga.steps[saga.current_step].resource_id = Some(rid);
                }
                Ok(())
            }
            Err(e) => {
                trigger_reverse_compensation(LcmPhase::Archive, CompensateAction::Reverse);
                Err(e.into())
            }
        }
    }

    async fn compensate(
        &self,
        _saga: &mut LcmSaga,
        resource_id: Option<Uuid>,
    ) -> Result<()> {
        // 反向：从归档恢复
        self.ops
            .reverse(resource_id.unwrap_or_else(Uuid::nil))
            .await
            .map_err(Into::into)
    }
}

// ============================================================================
// 跨域反向补偿辅助函数（per M-2067.3 跨域 Saga 反向补偿链）
// ============================================================================

/// 触发反向补偿（占位：真实路径属 WF-1-2073 跨域 gRPC 集成）
fn trigger_reverse_compensation(phase: LcmPhase, action: CompensateAction) {
    tracing::info!(
        target: "lcm_saga",
        phase = phase.as_str(),
        action = action.as_str(),
        "triggering reverse compensation chain (placeholder)"
    );
}

/// 触发跨域反向补偿链（per RGS-ADR-0015 单一调解者原则）
///
/// 真实路径属 WF-1-2073：跨域 gRPC client 集成（player / economy / social）
/// 本 worktree 仅占位 + 日志
fn trigger_cross_domain_reverse(phase: LcmPhase, domains: &[&str]) {
    tracing::info!(
        target: "lcm_saga",
        phase = phase.as_str(),
        domains = ?domains,
        "triggering cross-domain reverse compensation chain (placeholder)"
    );
}

/// 从 saga 元数据解析 operator_id（占位 helper）
fn parse_metadata_operator(saga: &LcmSaga) -> Result<Uuid> {
    // 真实路径：从 saga.idempotency_key 解析；本 worktree 返 nil
    let _ = saga;
    Ok(Uuid::nil())
}

// ============================================================================
// 工厂：构建完整 7 step handler 集合
// ============================================================================

/// 工厂：构造 7 阶段 step handler Vec（per phase 选 step handler）
///
/// service 字段必须已设置所有 6 操作器；本函数按 phase 选对应 handler
pub fn build_step_handlers(
    service: &RealmLifecycleService,
    phase: LcmPhase,
    target_realm_id: Option<Uuid>,
    source_realm_ids: Vec<Uuid>,
    locked_at_ms: Option<i64>,
    delta: Option<i32>,
) -> Vec<Arc<dyn EconomySagaStepHandler>> {
    match phase {
        LcmPhase::NewRealm => vec![Arc::new(NewRealmStep::new(service.new_realm.clone()))],
        LcmPhase::Scale => vec![Arc::new(ScaleStep::new(
            service.scale.clone(),
            delta.unwrap_or(1),
        ))],
        LcmPhase::Split => vec![Arc::new(SplitStep::new(
            service.split.clone(),
            target_realm_id.unwrap_or_else(Uuid::new_v4),
        ))],
        LcmPhase::Merge => vec![Arc::new(MergeStep::new(
            service.merge.clone(),
            target_realm_id.unwrap_or_else(Uuid::new_v4),
            source_realm_ids,
        ))],
        LcmPhase::MergeRollback => vec![Arc::new(MergeRollbackStep::new(
            service.merge.clone(),
            target_realm_id.unwrap_or_else(Uuid::new_v4),
            locked_at_ms.unwrap_or(0),
        ))],
        LcmPhase::Retire => vec![Arc::new(RetireStep::new(service.retire.clone()))],
        LcmPhase::Archive => vec![Arc::new(ArchiveStep::new(service.archive.clone()))],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compensate_action_str() {
        assert_eq!(CompensateAction::Reverse.as_str(), "reverse");
        assert_eq!(CompensateAction::Rollback.as_str(), "rollback");
        assert_eq!(CompensateAction::Cleanup.as_str(), "cleanup");
        assert_eq!(CompensateAction::CrossDomainReverse.as_str(), "cross_domain_reverse");
        assert_eq!(CompensateAction::CrossDomainRollback.as_str(), "cross_domain_rollback");
    }

    #[test]
    fn timeout_config_default() {
        let t = SagaTimeoutConfig::default();
        assert_eq!(t.secs, 60);
    }

    #[test]
    fn timeout_config_custom() {
        let t = SagaTimeoutConfig::new(30);
        assert_eq!(t.secs, 30);
    }

    #[test]
    fn timeout_config_to_duration() {
        let t = SagaTimeoutConfig::new(45);
        assert_eq!(t.to_duration(), Duration::from_secs(45));
    }
}
