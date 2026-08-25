//! Scale 操作器骨架（per RGS-DTL-042 §5.2 + FR-LCM-040~044）
//!
//! WF-1-2066 M-2066.5 骨架
//!
//! 范围（per FR-LCM-040~044）：
//! - 扩缩容双向（scale up / scale down）
//! - 节点级 HPA（水平扩缩，K8s HPA + 节点池）+ 资源配额调整
//! - 与 NewRealmOperator 共用部分逻辑（资源配额）
//!
//! 骨架阶段：仅 trait + 扩缩方向占位字段
//! 等待 L4 #2067 Saga 接入 + L4 #2068 6 表 migration + L4 #2070 DrillExecutor

use async_trait::async_trait;
use uuid::Uuid;

use super::{not_implemented_skeleton, validate_request};
use crate::realm_lifecycle::error::LcmResult;
use crate::realm_lifecycle::service::{RealmLifecycleOperator, RealmLifecycleStage};

/// 扩缩容方向（per FR-LCM-040~044 双向）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleDirection {
    /// 扩容（scale up）
    Up,
    /// 缩容（scale down）
    Down,
}

/// Scale 操作器（per FR-LCM-040~044，扩缩容双向）
///
/// 骨架实现：仅承载 trait 接口 + 扩缩方向枚举；业务逻辑在 L4 #2067/#2068/#2070 接入
#[derive(Debug, Default, Clone)]
pub struct ScaleOperator {
    /// 默认扩缩容方向（per M-2066.5 双向；骨架阶段占位）
    pub default_direction: Option<ScaleDirection>,
}

impl ScaleOperator {
    /// 工厂：新建 Scale 操作器
    pub fn new() -> Self {
        Self::default()
    }

    /// 工厂：骨架阶段最小可用实例
    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for ScaleOperator {
    fn name(&self) -> &'static str {
        "scale"
    }

    fn stage(&self) -> RealmLifecycleStage {
        RealmLifecycleStage::Scale
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
        //   1. 解析 scale direction（up/down）+ target_capacity
        //   2. 构建 Scale Saga（3 步：HPA 配置调整 + 节点池扩容 + 演练验证）
        //   3. 调 SagaOrchestrator.execute
        Err(not_implemented_skeleton(
            "ScaleOperator",
            "M-2067.2 Scale Saga 3 步 (HPA + 节点池) + M-2068.2 split_plan 表 + M-2070.2 DrillExecutor",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scale_execute_returns_not_implemented() {
        let op = ScaleOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "realm-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("ScaleOperator"));
    }

    #[test]
    fn scale_operator_metadata() {
        let op = ScaleOperator::new_for_skeleton();
        assert_eq!(op.name(), "scale");
        assert_eq!(op.stage(), RealmLifecycleStage::Scale);
    }

    #[test]
    fn scale_direction_equality() {
        assert_eq!(ScaleDirection::Up, ScaleDirection::Up);
        assert_ne!(ScaleDirection::Up, ScaleDirection::Down);
    }
}
