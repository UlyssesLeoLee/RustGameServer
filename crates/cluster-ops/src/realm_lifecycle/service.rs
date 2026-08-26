//! cluster-ops · realm_lifecycle · RealmLifecycleService 业务接口（per RGS-SPEC-DTL-042 §2.1）
//!
//! 硬约束（per RGS-SPEC-DTL-042 §2.1）：
//! - RealmLifecycleService **不**对外暴露独立接口（FR-LCM-004）
//! - 6 阶段操作器 trait 由 RealmLifecycleService 内部聚合
//! - 每个 operator 至少 1 个 `async fn`（per 验收门槛）
//! - 业务逻辑**不**在本 worktree 实现（属 WF-1-2066 / WF-1-2070 / WF-1-2071）
//!
//! 设计：
//! - 6 个独立 trait，per OperatorType，便于 PFAU Feature 集成
//! - RealmLifecycleService 聚合 6 个 Arc<dyn ...Operator> 引用
//! - execute_phase 统一入口（内部调 SagaOrchestrator）

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::error::Result;
use super::saga::{LcmPhase, SagaContext, SagaOrchestrator};

/// LCM 操作输入（per 6 阶段抽象）
#[derive(Debug, Clone)]
pub struct LcmOperatorInput {
    /// 操作者 ID（管理员）
    pub operator_id: Uuid,
    /// 请求 ID（幂等键）
    pub request_id: Uuid,
    /// 目标服务器/区服 ID
    pub realm_id: Uuid,
    /// 附加元数据（JSON 字符串；各操作器按需解析）
    pub metadata: String,
}

impl LcmOperatorInput {
    pub fn new(operator_id: Uuid, request_id: Uuid, realm_id: Uuid) -> Self {
        Self {
            operator_id,
            request_id,
            realm_id,
            metadata: String::new(),
        }
    }

    /// 构造带 metadata 的输入
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = metadata;
        self
    }
}

/// LCM 操作结果（per 6 阶段抽象）
#[derive(Debug, Clone)]
pub struct LcmOperatorOutput {
    /// 阶段
    pub phase: LcmPhase,
    /// 关联资源 ID（如新服 ID / Saga ID）
    pub resource_id: Option<Uuid>,
    /// 执行状态描述
    pub status: String,
}

// ============================================================================
// 6 操作器 trait 签名（per RGS-SPEC-DTL-042 §3 第 3 条 + WBS WF-1-2066 §3.1）
// ============================================================================

/// NewRealm 操作器（开新服）
#[async_trait]
pub trait NewRealmOperator: Send + Sync {
    /// 执行开新服
    async fn open(&self, input: LcmOperatorInput) -> Result<LcmOperatorOutput>;
    /// 反向：回收已开服资源（被 SagaOrchestrator 触发）
    async fn reverse(&self, resource_id: Uuid, reason: String) -> Result<()>;
}

/// Scale 操作器（扩缩容；含双向）
#[async_trait]
pub trait ScaleOperator: Send + Sync {
    /// 执行扩缩容（`delta > 0` = 扩容；`delta < 0` = 缩容）
    async fn scale(&self, input: LcmOperatorInput, delta: i32) -> Result<LcmOperatorOutput>;
    /// 反向：回滚到扩缩容前状态
    async fn reverse(&self, resource_id: Uuid, prior_replicas: i32) -> Result<()>;
}

/// Split 操作器（分服）
#[async_trait]
pub trait SplitOperator: Send + Sync {
    /// 执行分服（从源服拆出新区服）
    async fn split(&self, input: LcmOperatorInput, target_realm_id: Uuid) -> Result<LcmOperatorOutput>;
    /// 反向：合回源服（清理已分出的子服）
    async fn reverse(&self, source_realm_id: Uuid, child_realm_id: Uuid) -> Result<()>;
}

/// Merge 操作器（合服）+ MergeRollback 子操作
#[async_trait]
pub trait MergeOperator: Send + Sync {
    /// 执行合服（多服合入目标服）
    async fn merge(
        &self,
        input: LcmOperatorInput,
        target_realm_id: Uuid,
        source_realm_ids: Vec<Uuid>,
    ) -> Result<LcmOperatorOutput>;
    /// 反向：合服回退（per DTL §3.5 合服回退窗口期 7~30 天）
    async fn reverse(
        &self,
        target_realm_id: Uuid,
        source_realm_ids: Vec<Uuid>,
    ) -> Result<()>;
    /// MergeRollback 子操作：合服锁定后撤回（不同于 reverse，可触发重新合并）
    async fn rollback(&self, target_realm_id: Uuid, locked_at_ms: i64) -> Result<()>;
}

/// Retire 操作器（退服）
#[async_trait]
pub trait RetireOperator: Send + Sync {
    /// 执行退服
    async fn retire(&self, input: LcmOperatorInput) -> Result<LcmOperatorOutput>;
    /// 反向：恢复已退服
    async fn reverse(&self, resource_id: Uuid) -> Result<()>;
}

/// Archive 操作器（归档 + 冷热分层占位）
#[async_trait]
pub trait ArchiveOperator: Send + Sync {
    /// 执行归档
    async fn archive(&self, input: LcmOperatorInput) -> Result<LcmOperatorOutput>;
    /// 反向：从归档恢复（演练 / 客服查询）
    async fn reverse(&self, resource_id: Uuid) -> Result<()>;
}

// ============================================================================
// RealmLifecycleService 聚合（per RGS-SPEC-DTL-042 §2.1 + §2.3）
// ============================================================================

/// RealmLifecycleService 聚合（**内部模块，不分发独立接口**）
///
/// 6 操作器 trait 实现由 WF-1-2066 / WF-1-2070 / WF-1-2071 填充；
/// 本 worktree 只持有 trait object 占位 + SagaOrchestrator 编排入口。
pub struct RealmLifecycleService {
    pub new_realm: Arc<dyn NewRealmOperator>,
    pub scale: Arc<dyn ScaleOperator>,
    pub split: Arc<dyn SplitOperator>,
    pub merge: Arc<dyn MergeOperator>,
    pub retire: Arc<dyn RetireOperator>,
    pub archive: Arc<dyn ArchiveOperator>,
    pub saga: Arc<SagaOrchestrator>,
}

impl RealmLifecycleService {
    pub fn new(
        new_realm: Arc<dyn NewRealmOperator>,
        scale: Arc<dyn ScaleOperator>,
        split: Arc<dyn SplitOperator>,
        merge: Arc<dyn MergeOperator>,
        retire: Arc<dyn RetireOperator>,
        archive: Arc<dyn ArchiveOperator>,
        saga: Arc<SagaOrchestrator>,
    ) -> Self {
        Self {
            new_realm,
            scale,
            split,
            merge,
            retire,
            archive,
            saga,
        }
    }

    /// 统一 LCM 入口（被 AdminService 转发调用）
    ///
    /// 注：业务执行由 SagaOrchestrator 编排 6 操作器；本方法只做参数校验 + 委托
    /// 真实路径见 saga::orchestrator::SagaOrchestrator::execute
    pub async fn execute_phase(
        &self,
        phase: LcmPhase,
        ctx: SagaContext,
    ) -> Result<LcmOperatorOutput> {
        ctx.validate()?;
        self.saga.dispatch(phase, ctx, self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_input_constructs() {
        let op = Uuid::new_v4();
        let req = Uuid::new_v4();
        let realm = Uuid::new_v4();
        let input = LcmOperatorInput::new(op, req, realm);
        assert_eq!(input.operator_id, op);
        assert_eq!(input.request_id, req);
        assert_eq!(input.realm_id, realm);
        assert!(input.metadata.is_empty());
    }

    #[test]
    fn operator_input_with_metadata() {
        let input = LcmOperatorInput::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
            .with_metadata("{}".to_string());
        assert_eq!(input.metadata, "{}");
    }

    /// 空操作器 stub（仅占位以满足 trait 签名约束）
    pub struct EmptyOperator;

    #[async_trait]
    impl NewRealmOperator for EmptyOperator {
        async fn open(&self, _input: LcmOperatorInput) -> Result<LcmOperatorOutput> {
            unimplemented!()
        }
        async fn reverse(&self, _resource_id: Uuid, _reason: String) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl ScaleOperator for EmptyOperator {
        async fn scale(&self, _input: LcmOperatorInput, _delta: i32) -> Result<LcmOperatorOutput> {
            unimplemented!()
        }
        async fn reverse(&self, _resource_id: Uuid, _prior_replicas: i32) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl SplitOperator for EmptyOperator {
        async fn split(
            &self,
            _input: LcmOperatorInput,
            _target_realm_id: Uuid,
        ) -> Result<LcmOperatorOutput> {
            unimplemented!()
        }
        async fn reverse(&self, _source_realm_id: Uuid, _child_realm_id: Uuid) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl MergeOperator for EmptyOperator {
        async fn merge(
            &self,
            _input: LcmOperatorInput,
            _target_realm_id: Uuid,
            _source_realm_ids: Vec<Uuid>,
        ) -> Result<LcmOperatorOutput> {
            unimplemented!()
        }
        async fn reverse(
            &self,
            _target_realm_id: Uuid,
            _source_realm_ids: Vec<Uuid>,
        ) -> Result<()> {
            unimplemented!()
        }
        async fn rollback(&self, _target_realm_id: Uuid, _locked_at_ms: i64) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl RetireOperator for EmptyOperator {
        async fn retire(&self, _input: LcmOperatorInput) -> Result<LcmOperatorOutput> {
            unimplemented!()
        }
        async fn reverse(&self, _resource_id: Uuid) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl ArchiveOperator for EmptyOperator {
        async fn archive(&self, _input: LcmOperatorInput) -> Result<LcmOperatorOutput> {
            unimplemented!()
        }
        async fn reverse(&self, _resource_id: Uuid) -> Result<()> {
            unimplemented!()
        }
    }
}
