//! Merge + MergeRollback 操作器骨架（per RGS-DTL-042 §5.2 + FR-LCM-060~064）
//!
//! WF-1-2066 M-2066.7 骨架
//!
//! 范围（per FR-LCM-060~064）：
//! - 合服（merge）：将多个 realm 玩家/数据/资源合并到单一 target realm
//! - `MergeConflictRuleSet` v2：3 类规则（per DTL-042 §3 merge_conflict_rule_set_v2）
//!   1. 命名空间冲突（玩家名 / 工会名 / 邮件主题）
//!   2. 资产冲突（货币余额取较大 / 物品 ID 重映射）
//!   3. 关系冲突（好友/工会合并策略）
//! - Saga 5 步执行（per RGS-DTL-042 §6 合服 5 步）
//!   1. 冲突规则集加载 + 校验（FR-LCM-062：locked_at 后不可改）
//!   2. 源 realm 写冻结
//!   3. 数据迁移 + 冲突解决
//!   4. 跨服关系保持
//!   5. 灰度路由切换 + 写冻结解除
//! - **合服回退子操作**（per FR-LCM-062 验证路径）：合服完成后 7~30 天回退窗口期内可触发
//!
//! 骨架阶段：仅 trait + 冲突规则集占位字段
//! 等待 L4 #2067 Saga 接入 + L4 #2068 6 表 + L4 #2073 跨域 gRPC

use async_trait::async_trait;
use uuid::Uuid;

use super::{not_implemented_skeleton, validate_request};
use crate::realm_lifecycle::error::LcmResult;
use crate::realm_lifecycle::service::{RealmLifecycleOperator, RealmLifecycleStage};

/// 冲突规则集版本（per FR-LCM-062 锁定后不可改）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeRuleSetVersion {
    /// v1：基础规则（命名空间 + 资产）
    #[default]
    V1,
    /// v2：扩展规则（命名空间 + 资产 + 关系；per DTL-042 §3 升级点）
    V2,
}

/// 冲突规则集骨架（per M-2068.2 merge_conflict_rule_set_v2 DDL）
///
/// 骨架阶段仅承载字段；实际 PgRepository 由 L4 #2068 接入
#[derive(Debug, Default, Clone)]
pub struct MergeConflictRuleSet {
    pub rule_set_id: Uuid,
    pub version: MergeRuleSetVersion,
    /// 锁定时间戳（per FR-LCM-062：锁定后运行时不可改）
    pub locked_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Merge 操作器（per FR-LCM-060~064）
#[derive(Debug, Default, Clone)]
pub struct MergeOperator;

impl MergeOperator {
    pub fn new() -> Self {
        Self
    }

    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for MergeOperator {
    fn name(&self) -> &'static str {
        "merge"
    }

    fn stage(&self) -> RealmLifecycleStage {
        RealmLifecycleStage::Merge
    }

    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        _operator_id: Uuid,
        _approval_ref: Option<&str>,
    ) -> LcmResult<Uuid> {
        // realm_id 在 merge 语义下为 target_realm_id
        validate_request(request_id, realm_id)?;
        // 骨架阶段占位
        // 后续 L4 #2067 Saga 接入后将替换为：
        //   1. 加载 merge_conflict_rule_set_v2（per FR-LCM-062 锁定校验）
        //   2. 构建 Merge Saga 5 步
        //   3. 调 SagaOrchestrator.execute（含反向补偿）
        Err(not_implemented_skeleton(
            "MergeOperator",
            "M-2067.2 Merge Saga 5 步 + M-2068.2 merge_conflict_rule_set_v2 DDL + M-2073 跨域 gRPC",
        ))
    }
}

/// MergeRollback 操作器（per FR-LCM-062 验证路径）
///
/// 仅当 merge 处于 7~30 天回退窗口期（TBD-DTL-042 实测填）时可调用
#[derive(Debug, Default, Clone)]
pub struct MergeRollbackOperator;

impl MergeRollbackOperator {
    pub fn new() -> Self {
        Self
    }

    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for MergeRollbackOperator {
    fn name(&self) -> &'static str {
        "merge_rollback"
    }

    fn stage(&self) -> RealmLifecycleStage {
        // 回退子操作仍归属 Merge 阶段
        RealmLifecycleStage::Merge
    }

    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        _operator_id: Uuid,
        _approval_ref: Option<&str>,
    ) -> LcmResult<Uuid> {
        // realm_id 在 merge_rollback 语义下为原 target_realm_id（用于反查源 realm_ids）
        validate_request(request_id, realm_id)?;
        // 骨架阶段占位
        // 后续 L4 #2067 Saga 接入后将替换为：
        //   1. 校验 merge_run_id 在 7~30 天回退窗口期内
        //   2. 加载原始 merge Saga 步骤 + 触发反向补偿（per RGS-DTL-100 §4 补偿模式）
        //   3. 灰度路由回切
        Err(not_implemented_skeleton(
            "MergeRollbackOperator",
            "M-2067.3 Merge Saga 反向补偿 + M-2070.7 AC-LCM-005 演练 1 次 (FR-LCM-062 验证)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn merge_execute_returns_not_implemented() {
        let op = MergeOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "realm-target-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("MergeOperator"));
    }

    #[tokio::test]
    async fn merge_rollback_execute_returns_not_implemented() {
        let op = MergeRollbackOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "realm-target-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("MergeRollbackOperator"));
    }

    #[test]
    fn merge_operator_metadata() {
        let op = MergeOperator::new_for_skeleton();
        assert_eq!(op.name(), "merge");
        assert_eq!(op.stage(), RealmLifecycleStage::Merge);
    }

    #[test]
    fn merge_rollback_operator_metadata() {
        let op = MergeRollbackOperator::new_for_skeleton();
        assert_eq!(op.name(), "merge_rollback");
        assert_eq!(op.stage(), RealmLifecycleStage::Merge);
    }

    #[test]
    fn rule_set_version_v2_locked_semantics() {
        let v1 = MergeRuleSetVersion::V1;
        let v2 = MergeRuleSetVersion::V2;
        assert_ne!(v1, v2);
        // FR-LCM-062: locked_at 非空即锁定
        let mut rs = MergeConflictRuleSet {
            rule_set_id: Uuid::new_v4(),
            version: v2,
            locked_at: None,
        };
        assert!(rs.locked_at.is_none());
        rs.locked_at = Some(chrono::Utc::now());
        assert!(rs.locked_at.is_some());
    }
}
