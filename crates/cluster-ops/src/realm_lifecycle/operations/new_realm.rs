//! NewRealm 操作器骨架（per RGS-DTL-042 §5.3 + FR-LCM-010~033）
//!
//! WF-1-2066 M-2066.4 骨架
//!
//! 范围（per FR-LCM-010~033）：
//! - 开新服（new_realm）：资源申请 + ARC-018 落地创建 + 灰度可关停
//! - 涉及业务：新服元数据 + 初始玩家数据 + 灰度路由配置
//! - Saga 步骤（per RGS-DTL-100 + RGS-DTL-015/016 既有模式，待 L4 #2067 接入）：
//!   1. 资源配额申请（向 platform 域提交 request）
//!   2. 元数据写入（new_realm_plan + realm_lifecycle_run 落 admin_db）
//!   3. 灰度路由表新增（rgs-realm-directory 选服路由）
//!   4. 演练验证（DrillExecutor 沙箱环境跑通 + drill_validated 状态）
//!   5. 标记 NewRealm 完成
//!
//! 骨架阶段（per M-2066.4）：
//! - 仅实现 `RealmLifecycleOperator` trait
//! - `execute()` 返回 `NotImplemented` 占位错误
//! - 等待 L4 #2067 Saga 接入 + L4 #2068 6 表 migration + L4 #2070 DrillExecutor

use async_trait::async_trait;
use uuid::Uuid;

use super::{not_implemented_skeleton, validate_request};
use crate::realm_lifecycle::error::LcmResult;
use crate::realm_lifecycle::service::{RealmLifecycleOperator, RealmLifecycleStage};

/// NewRealm 操作器（per FR-LCM-010~033）
///
/// 骨架实现：仅承载 trait 接口 + 元信息；业务逻辑在 L4 #2067/#2068/#2070 接入
#[derive(Debug, Default, Clone)]
pub struct NewRealmOperator {
    /// 灰度策略（per ARC-018 灰度可关停，骨架阶段仅占位）
    pub canary_percentage: Option<u8>,
}

impl NewRealmOperator {
    /// 工厂：新建 NewRealm 操作器
    pub fn new() -> Self {
        Self::default()
    }

    /// 工厂：骨架阶段最小可用实例（per 6 操作器统一注册）
    ///
    /// 与 `new()` 等价；为 M-2066 阶段统一接入点提供稳定接口
    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for NewRealmOperator {
    fn name(&self) -> &'static str {
        "new_realm"
    }

    fn stage(&self) -> RealmLifecycleStage {
        RealmLifecycleStage::NewRealm
    }

    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        _operator_id: Uuid,
        _approval_ref: Option<&str>,
    ) -> LcmResult<Uuid> {
        // 骨架阶段参数校验（per FR-LCM-002 阶段变更全流程留痕 + RGS-DTL-031 §3.1 幂等性）
        validate_request(request_id, realm_id)?;
        // 骨架阶段占位：业务逻辑未实现
        // 后续 L4 #2067 SagaOrchestrator 接入后将替换为：
        //   1. 构建 5 步 NewRealm Saga
        //   2. 调 SagaOrchestrator.execute
        //   3. 返回 saga.run_id（即 realm_lifecycle_run.run_id）
        Err(not_implemented_skeleton(
            "NewRealmOperator",
            "M-2067.2 NewRealm Saga 5 步 + M-2068.2 new_realm_plan 表 + M-2070.2 DrillExecutor",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_realm_execute_returns_not_implemented() {
        let op = NewRealmOperator::new_for_skeleton();
        let req = Uuid::new_v4();
        let operator = Uuid::new_v4();
        let err = op.execute(req, "realm-new-001", operator, None).await.unwrap_err();
        let display = err.to_string();
        assert!(display.contains("NewRealmOperator"));
        assert!(display.contains("M-2067"));
    }

    #[tokio::test]
    async fn new_realm_execute_rejects_empty_realm_id() {
        let op = NewRealmOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(matches!(
            err.kind,
            crate::realm_lifecycle::error::LcmErrorKind::InvalidParameter(_)
        ));
    }

    #[tokio::test]
    async fn new_realm_execute_rejects_nil_request_id() {
        let op = NewRealmOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::nil(), "realm-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(matches!(
            err.kind,
            crate::realm_lifecycle::error::LcmErrorKind::InvalidParameter(_)
        ));
    }

    #[test]
    fn new_realm_operator_metadata() {
        let op = NewRealmOperator::new_for_skeleton();
        assert_eq!(op.name(), "new_realm");
        assert_eq!(op.stage(), RealmLifecycleStage::NewRealm);
    }
}
