//! Retire 操作器骨架（per RGS-DTL-042 §5.2 + FR-LCM-070~075）
//!
//! WF-1-2066 M-2066.8 骨架
//!
//! 范围（per FR-LCM-070~075）：
//! - 退场（retire）：只读维护模式 + 玩家迁移 + 灰度下线
//! - 涉及业务：玩家数据迁出 + 工会/好友关系保持 + 资金冻结
//! - Saga 4 步执行（per RGS-DTL-042 §6 退场 4 步）
//!   1. 退场前置检查（per SPEC §3 第 8 条：retire_plan.query_channel_rbac 配置）
//!   2. 只读维护模式切换
//!   3. 玩家数据迁出 + 资金冻结
//!   4. 灰度路由下线
//! - 退场后 RBAC 查询通道**仅**对 `retire_plan.query_channel_rbac` 配置角色开放
//!   （默认 `cs_agent` / `sre` / `legal`，per SPEC §3 第 8 条 + M-2073.4 退场后 RBAC）
//!
//! 骨架阶段：仅 trait + RBAC 通道占位字段
//! 等待 L4 #2067 Saga + L4 #2068 6 表 + L4 #2073 跨域 gRPC + M-2073.4 RBAC

use async_trait::async_trait;
use uuid::Uuid;

use super::{not_implemented_skeleton, validate_request};
use crate::realm_lifecycle::error::LcmResult;
use crate::realm_lifecycle::service::{RealmLifecycleOperator, RealmLifecycleStage};

/// 退场 RBAC 角色（per SPEC §3 第 8 条 + M-2073.4 退场后 RBAC）
///
/// 退场后查询通道**仅**对这些角色开放；其他角色查询被拒
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetireRbacRole {
    /// 客服 agent
    CsAgent,
    /// SRE
    Sre,
    /// 法务
    Legal,
}

impl RetireRbacRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CsAgent => "cs_agent",
            Self::Sre => "sre",
            Self::Legal => "legal",
        }
    }
}

/// 退场计划骨架（per M-2068.3 retire_plan DDL + M-2073.4 RBAC 配置列）
#[derive(Debug, Default, Clone)]
pub struct RetirePlan {
    pub target_realm_id: String,
    /// 退场后查询通道 RBAC（per SPEC §3 第 8 条）
    pub query_channel_rbac: Vec<RetireRbacRole>,
    /// 退场后归档启动阈值（per TBD-DTL-042 30~90 天；M-2070.8 实测填）
    pub archive_threshold_days: u32,
}

/// Retire 操作器（per FR-LCM-070~075）
#[derive(Debug, Default, Clone)]
pub struct RetireOperator;

impl RetireOperator {
    pub fn new() -> Self {
        Self
    }

    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for RetireOperator {
    fn name(&self) -> &'static str {
        "retire"
    }

    fn stage(&self) -> RealmLifecycleStage {
        RealmLifecycleStage::Retire
    }

    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        _operator_id: Uuid,
        _approval_ref: Option<&str>,
    ) -> LcmResult<Uuid> {
        validate_request(request_id, realm_id)?;
        // 骨架阶段占位
        // 后续 L4 #2067 Saga 接入后将替换为：
        //   1. 加载 retire_plan + 校验 query_channel_rbac（per SPEC §3 第 8 条）
        //   2. 构建 Retire Saga 4 步
        //   3. 调 SagaOrchestrator.execute
        //   4. 跨域 gRPC：rgs-player-service（玩家迁出） + rgs-economy-service（资金冻结）
        Err(not_implemented_skeleton(
            "RetireOperator",
            "M-2067.2 Retire Saga 4 步 + M-2068.3 retire_plan DDL + M-2073.4 退场后 RBAC 通道",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retire_execute_returns_not_implemented() {
        let op = RetireOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "realm-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("RetireOperator"));
    }

    #[test]
    fn retire_operator_metadata() {
        let op = RetireOperator::new_for_skeleton();
        assert_eq!(op.name(), "retire");
        assert_eq!(op.stage(), RealmLifecycleStage::Retire);
    }

    #[test]
    fn retire_rbac_default_roles() {
        let plan = RetirePlan {
            target_realm_id: "realm-001".to_string(),
            query_channel_rbac: vec![
                RetireRbacRole::CsAgent,
                RetireRbacRole::Sre,
                RetireRbacRole::Legal,
            ],
            archive_threshold_days: 60,
        };
        assert_eq!(plan.query_channel_rbac.len(), 3);
        assert_eq!(plan.archive_threshold_days, 60);
    }

    #[test]
    fn retire_rbac_role_str_round_trip() {
        for (role, s) in [
            (RetireRbacRole::CsAgent, "cs_agent"),
            (RetireRbacRole::Sre, "sre"),
            (RetireRbacRole::Legal, "legal"),
        ] {
            assert_eq!(role.as_str(), s);
        }
    }
}
