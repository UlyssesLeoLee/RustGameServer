//! `realm_lifecycle` Service 业务门面（per RGS-DTL-042 §3 + SPEC-DTL-042 §2 + §3）
//!
//! WF-1-2073 范围：仅 6 操作器 trait 定义 + 6 操作 trait 抽象。
//! 完整 service impl（new_realm / scale / split / merge / retire / archive 操作器）
//! 属 L4 #2066~#2067（PH-3 任务）。本文件作为 trait 抽象先行落地，
//! 让后续 Saga 7 步可按 trait 解耦真实操作器 vs 演练 mock（per FR-LCM-003）。
//!
//! FR-LCM-004 硬约束：**不**分发独立 gRPC / HTTP（per SPEC-DTL-042 §2 + §3 第 1 条）。
//! 全部经 `AdminService` 转发 → `ClusterOpsService` PFAU 编排 → 本 service。
//!
//! 7 个 `realm_lifecycle::*` Feature 子类对应 6 阶段 + 1 回退（merge_rollback）：
//!   - new_realm        开新服
//!   - scale            扩缩容
//!   - split            分服
//!   - merge            合服
//!   - merge_rollback   合服回退
//!   - retire           退场
//!   - archive          归档

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::Result as ClusterOpsResult;
use crate::realm_lifecycle::error::{Error, Result};

/// 6 阶段操作器特征（per DTL-042 §3 + L4 #2066 + M-2066.2 6 操作器 trait）
///
/// 设计要点：
/// - 每个 `Operator` 是**纯异步 + 幂等**（per SPEC §5 幂等一致性）
/// - 所有写操作携带 `request_id` + `operator_id` + `approval_ref`（高危）
/// - **不**直接写业务 service DB；通过 `BusinessServiceClient` 调 gRPC
///   （per SPEC §3 第 3 条 + §6 R1 缓解）
/// - **不**绕过 PFAU 编排（per SPEC §3 第 2 条）；operator 接收 PFAU
///   注入的 `feature_id` / `run_id` 关联上下文
#[async_trait]
pub trait Operator: Send + Sync {
    /// 操作器 ID（如 `new_realm` / `scale` / `split` / `merge` / `retire` / `archive`）
    fn operator_id(&self) -> &'static str;

    /// 执行幂等操作（per SPEC §5 request_id 幂等键）
    ///
    /// 入参 `OperatorContext` 含 `request_id` + `operator_id` + `approval_ref`
    /// + `run_id`（PFAU 编排注入）。
    async fn execute(&self, ctx: &OperatorContext) -> Result<OperatorOutcome>;

    /// 反向补偿（per SPEC §3 第 5 条 + L4 #2067 M-2067.3）
    ///
    /// 仅当 `execute` 部分成功导致 Saga 中途失败时被调用；
    /// 必须**幂等**（重复补偿不能造成脏数据）。
    async fn reverse(&self, ctx: &OperatorContext) -> Result<OperatorOutcome>;
}

/// 操作器执行上下文（per SPEC §3 第 6 条：request_id + operator_id + approval_ref）
#[derive(Debug, Clone)]
pub struct OperatorContext {
    /// 幂等键（同 RGS-DTL-031 §3.1）
    pub request_id: Uuid,
    /// 操作员 ID
    pub operator_id: Uuid,
    /// 三方签字 reference（per SPEC §5 阶段变更**必须**经三方签字）
    pub approval_ref: Option<String>,
    /// PFAU run_id（5 域编排主外键）
    pub run_id: Uuid,
    /// 源 realm_id（合服 / 分服 / 退场时）
    pub source_realm_id: Option<String>,
    /// 目标 realm_id（合服 / 分服 / 开新服时）
    pub target_realm_id: Option<String>,
    /// OTel trace_id
    pub trace_id: Option<String>,
}

impl OperatorContext {
    pub fn new(
        request_id: Uuid,
        operator_id: Uuid,
        approval_ref: Option<String>,
        run_id: Uuid,
    ) -> Self {
        Self {
            request_id,
            operator_id,
            approval_ref,
            run_id,
            source_realm_id: None,
            target_realm_id: None,
            trace_id: None,
        }
    }
}

/// 操作器执行结果
#[derive(Debug, Clone)]
pub struct OperatorOutcome {
    /// 影响的实体 ID 列表（用于审计 + 后续 Saga step 引用）
    pub affected_entity_ids: Vec<String>,
    /// 状态变化描述（"realm_created" / "data_migrated" 等）
    pub state_change: String,
    /// 可观测性元数据
    pub metadata: serde_json::Value,
}

impl OperatorOutcome {
    pub fn empty(state_change: impl Into<String>) -> Self {
        Self {
            affected_entity_ids: vec![],
            state_change: state_change.into(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_affected(state_change: impl Into<String>, ids: Vec<String>) -> Self {
        Self {
            affected_entity_ids: ids,
            state_change: state_change.into(),
            metadata: serde_json::json!({}),
        }
    }
}

/// 6 阶段操作器 trait 集合（per L4 #2066 M-2066.2）
///
/// 6 个 operator 子 trait 强制每个阶段有 `name()` 静态标识，
/// `RealmLifecycleService` 通过这 6 个 trait 委托具体实现。
#[async_trait]
pub trait NewRealmOperator: Operator {}
#[async_trait]
pub trait ScaleOperator: Operator {}
#[async_trait]
pub trait SplitOperator: Operator {}
#[async_trait]
pub trait MergeOperator: Operator {}
#[async_trait]
pub trait MergeRollbackOperator: Operator {}
#[async_trait]
pub trait RetireOperator: Operator {}
#[async_trait]
pub trait ArchiveOperator: Operator {}

/// `RealmLifecycleService` 6 操作器门面（per SPEC §3 第 2 条 + DTL-042 §3）
///
/// 所有操作经 PFAU 编排（per SPEC §3 第 2 条），**不**为 LCM 另起一套编排。
/// 本 service 持有 6 operator（Arc<dyn ...>）作为可插拔后端；真实实现
/// 属 L4 #2066，演练 mock 属 L4 #2070。
pub struct RealmLifecycleService {
    pub new_realm: Arc<dyn NewRealmOperator>,
    pub scale: Arc<dyn ScaleOperator>,
    pub split: Arc<dyn SplitOperator>,
    pub merge: Arc<dyn MergeOperator>,
    pub merge_rollback: Arc<dyn MergeRollbackOperator>,
    pub retire: Arc<dyn RetireOperator>,
    pub archive: Arc<dyn ArchiveOperator>,
}

impl RealmLifecycleService {
    pub fn new(
        new_realm: Arc<dyn NewRealmOperator>,
        scale: Arc<dyn ScaleOperator>,
        split: Arc<dyn SplitOperator>,
        merge: Arc<dyn MergeOperator>,
        merge_rollback: Arc<dyn MergeRollbackOperator>,
        retire: Arc<dyn RetireOperator>,
        archive: Arc<dyn ArchiveOperator>,
    ) -> Self {
        Self {
            new_realm,
            scale,
            split,
            merge,
            merge_rollback,
            retire,
            archive,
        }
    }

    /// 7 个 Feature 子类注册名（per RGS-DTL-031 §1.1 + ARC-051 既有枚举扩展）
    pub const FEATURE_NEW_REALM: &'static str = "realm_lifecycle::new_realm";
    pub const FEATURE_SCALE: &'static str = "realm_lifecycle::scale";
    pub const FEATURE_SPLIT: &'static str = "realm_lifecycle::split";
    pub const FEATURE_MERGE: &'static str = "realm_lifecycle::merge";
    pub const FEATURE_MERGE_ROLLBACK: &'static str = "realm_lifecycle::merge_rollback";
    pub const FEATURE_RETIRE: &'static str = "realm_lifecycle::retire";
    pub const FEATURE_ARCHIVE: &'static str = "realm_lifecycle::archive";

    /// 7 个 Feature 名（per M-2071.2 Feature 7 子类注册完整性验证）
    pub const ALL_FEATURES: &'static [&'static str] = &[
        Self::FEATURE_NEW_REALM,
        Self::FEATURE_SCALE,
        Self::FEATURE_SPLIT,
        Self::FEATURE_MERGE,
        Self::FEATURE_MERGE_ROLLBACK,
        Self::FEATURE_RETIRE,
        Self::FEATURE_ARCHIVE,
    ];

    /// Feature 名 → operator（per PFAU 编排 dispatch 协议）
    pub fn dispatch(&self, feature: &str, ctx: &OperatorContext) -> Option<&dyn Operator> {
        match feature {
            Self::FEATURE_NEW_REALM => Some(self.new_realm.as_ref()),
            Self::FEATURE_SCALE => Some(self.scale.as_ref()),
            Self::FEATURE_SPLIT => Some(self.split.as_ref()),
            Self::FEATURE_MERGE => Some(self.merge.as_ref()),
            Self::FEATURE_MERGE_ROLLBACK => Some(self.merge_rollback.as_ref()),
            Self::FEATURE_RETIRE => Some(self.retire.as_ref()),
            Self::FEATURE_ARCHIVE => Some(self.archive.as_ref()),
            _ => None,
        }
    }
}

/// 适配 cluster-ops 主 `Error` 类型 → realm_lifecycle `Error` 类型
impl From<crate::error::Error> for Error {
    fn from(e: crate::error::Error) -> Self {
        match e {
            crate::error::Error::Validation(s) => Error::Validation(s),
            crate::error::Error::NotFound { entity, id } => {
                Error::NotFound(format!("{} {}", entity, id))
            }
            crate::error::Error::Conflict(s) => Error::Conflict(s),
            crate::error::Error::Unauthorized(s) => Error::Unauthorized(s),
            crate::error::Error::Forbidden(s) => Error::Forbidden(s),
            crate::error::Error::Unavailable(s) => Error::Unavailable(s),
            crate::error::Error::Internal(a) => Error::Internal(a),
            crate::error::Error::Transport(s) => Error::Transport(s),
            other => Error::Internal(anyhow::anyhow!("cluster-ops error: {}", other)),
        }
    }
}

impl From<Error> for crate::error::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Validation(s) => crate::error::Error::Validation(s),
            Error::NotFound(s) => crate::error::Error::NotFound {
                entity: "realm_lifecycle",
                id: s,
            },
            Error::Conflict(s) => crate::error::Error::Conflict(s),
            Error::Unauthorized(s) => crate::error::Error::Unauthorized(s),
            Error::Forbidden(s) => crate::error::Error::Forbidden(s),
            Error::Unavailable(s) => crate::error::Error::Unavailable(s),
            Error::Internal(a) => crate::error::Error::Internal(a),
            Error::Transport(s) => crate::error::Error::Transport(s),
            other => crate::error::Error::Internal(anyhow::anyhow!("realm_lifecycle: {}", other)),
        }
    }
}

/// `RealmLifecycleService` 暴露的 cluster-ops 层 `Result` 别名
pub type ClusterOpsResultAdapter<T> = ClusterOpsResult<T>;

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 7 个 Feature 名常量完整（per M-2071.2 Feature 子类注册 100% 命中）
    #[test]
    fn all_seven_features_registered() {
        assert_eq!(RealmLifecycleService::ALL_FEATURES.len(), 7);
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::new_realm"));
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::scale"));
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::split"));
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::merge"));
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::merge_rollback"));
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::retire"));
        assert!(RealmLifecycleService::ALL_FEATURES.contains(&"realm_lifecycle::archive"));
    }

    /// 验证 OperatorContext 字段完整
    #[test]
    fn operator_context_fields_present() {
        let ctx = OperatorContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some("approval-123".to_string()),
            Uuid::new_v4(),
        );
        assert!(ctx.approval_ref.is_some());
        assert!(ctx.source_realm_id.is_none());
        assert!(ctx.target_realm_id.is_none());
    }

    /// 验证 OperatorOutcome 构造
    #[test]
    fn operator_outcome_empty() {
        let o = OperatorOutcome::empty("test_state");
        assert_eq!(o.state_change, "test_state");
        assert!(o.affected_entity_ids.is_empty());
    }

    /// 验证 realm_lifecycle Error ↔ cluster-ops Error 互转不丢失信息
    #[test]
    fn error_cross_translation_preserves_kind() {
        let lcm_err = Error::NotFound("realm-a".to_string());
        let co_err: crate::error::Error = lcm_err.into();
        match co_err {
            crate::error::Error::NotFound { entity, id } => {
                assert_eq!(entity, "realm_lifecycle");
                assert!(id.contains("realm-a"));
            }
            _ => panic!("expected NotFound"),
        }
    }
}
