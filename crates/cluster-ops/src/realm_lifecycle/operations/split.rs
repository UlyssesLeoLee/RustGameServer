//! Split 操作器骨架（per RGS-DTL-042 §5.2 + FR-LCM-050~055）
//!
//! WF-1-2066 M-2066.6 骨架
//!
//! 范围（per FR-LCM-050~055）：
//! - 分服（split）：将单个 realm 的玩家/数据/资源拆分到多个新 realm
//! - `SplitPlan` 落地：源 realm + 多个目标 realm + 拆分规则
//! - Saga 6 步执行（per RGS-DTL-100 §3 + DTL-042 §6 拆分 6 步）
//!   1. 拆分规则验证（数据均匀性 + 业务约束）
//!   2. 源 realm 写冻结（per FR-LCM-051 写冻结窗口）
//!   3. 目标 realm 资源创建（K8s namespace + DB schema + 缓存预热）
//!   4. 数据迁移（按拆分规则批量迁移 player_db / economy_db / social_db）
//!   5. 跨服关系重建（好友/工会/邮件 per L4 #2073 跨域联动）
//!   6. 灰度路由切换 + 写冻结解除
//!
//! 骨架阶段：仅 trait + SplitPlan 字段占位
//! 等待 L4 #2067 Saga 接入 + L4 #2068 6 表 + L4 #2073 跨域 gRPC

use async_trait::async_trait;
use uuid::Uuid;

use super::{not_implemented_skeleton, validate_request};
use crate::realm_lifecycle::error::LcmResult;
use crate::realm_lifecycle::service::{RealmLifecycleOperator, RealmLifecycleStage};

/// Split 计划骨架（per M-2068.2 split_plan DDL）
///
/// 骨架阶段仅承载字段；实际 PgRepository 由 L4 #2068 接入
#[derive(Debug, Default, Clone)]
pub struct SplitPlan {
    /// 源 realm_id
    pub source_realm_id: String,
    /// 目标 realm_id 列表
    pub target_realm_ids: Vec<String>,
    /// 拆分规则 key（per merge_conflict_rule_set_v2 复用机制）
    pub split_rule_key: Option<String>,
}

/// Split 操作器（per FR-LCM-050~055）
///
/// 骨架实现：仅承载 trait 接口 + SplitPlan 占位
#[derive(Debug, Default, Clone)]
pub struct SplitOperator;

impl SplitOperator {
    /// 工厂：新建 Split 操作器
    pub fn new() -> Self {
        Self
    }

    /// 工厂：骨架阶段最小可用实例
    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for SplitOperator {
    fn name(&self) -> &'static str {
        "split"
    }

    fn stage(&self) -> RealmLifecycleStage {
        RealmLifecycleStage::Split
    }

    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        _operator_id: Uuid,
        _approval_ref: Option<&str>,
    ) -> LcmResult<Uuid> {
        // realm_id 在 split 语义下为 source_realm_id
        validate_request(request_id, realm_id)?;
        // 骨架阶段占位
        // 后续 L4 #2067 Saga 接入后将替换为：
        //   1. 加载 SplitPlan（per L4 #2068 split_plan 表）
        //   2. 验证 split_rule_key 一致性
        //   3. 构建 Split Saga 6 步
        //   4. 调 SagaOrchestrator.execute（含反向补偿步骤）
        //   5. 跨域 gRPC：rgs-player-service / rgs-economy-service / rgs-social-service
        Err(not_implemented_skeleton(
            "SplitOperator",
            "M-2067.2 Split Saga 6 步 + M-2068.2 split_plan DDL + M-2073 跨域 gRPC 客户端",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn split_execute_returns_not_implemented() {
        let op = SplitOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "realm-source-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("SplitOperator"));
        assert!(err.to_string().contains("M-2067"));
    }

    #[test]
    fn split_operator_metadata() {
        let op = SplitOperator::new_for_skeleton();
        assert_eq!(op.name(), "split");
        assert_eq!(op.stage(), RealmLifecycleStage::Split);
    }

    #[test]
    fn split_plan_carries_source_and_targets() {
        let plan = SplitPlan {
            source_realm_id: "realm-source".to_string(),
            target_realm_ids: vec!["realm-tgt-1".to_string(), "realm-tgt-2".to_string()],
            split_rule_key: Some("even_split_v1".to_string()),
        };
        assert_eq!(plan.source_realm_id, "realm-source");
        assert_eq!(plan.target_realm_ids.len(), 2);
    }
}
