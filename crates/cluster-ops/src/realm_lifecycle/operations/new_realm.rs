//! NewRealm 操作器（开新服，per AC-LCM-001 + DTL-042 §5.1）。
//!
//! 步骤：分配 realm_id → 初始化 realm_directory 路由条目（灰度 0%）→
//! admin_db.realm_lifecycle_run 写 run 记录 → PFAU 编排到 Active 状态。
//!
//! 本 worktree（WF-1-2070）只定义 trait 签名 + 占位 operator。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::PhaseOperator;
use crate::realm_lifecycle::{
    error::Result, ApprovalRef, OperatorId, RealmId, RequestId, TraceId,
};

/// NewRealm 计划参数（per DTL-042 §5.1 + SPEC §3）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRealmParams {
    pub realm_id: RealmId,
    pub region: String,
    pub initial_capacity: u32,
    pub initial_node_count: u32,
}

/// NewRealm 操作结果（drill 用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRealmOutcome {
    pub realm_id: RealmId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub directory_entry_registered: bool,
    pub pfau_state: String,
}

#[async_trait]
pub trait NewRealmOperator: PhaseOperator {
    /// 执行开新服；返回 outcome。
    ///
    /// drill 实现走 `DrillExecutor` → `sandbox_pg` + `sandbox_k8s`。
    /// 生产实现由 WF-1-2066/2071 后续 worktree 提供。
    async fn execute(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: NewRealmParams,
    ) -> Result<NewRealmOutcome>;
}

/// `NoopNewRealmOperator` —— 占位实现，标记"未实现"（不 panic）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopNewRealmOperator;

impl PhaseOperator for NoopNewRealmOperator {
    fn phase_name(&self) -> &'static str {
        "new_realm"
    }
}

#[async_trait]
impl NewRealmOperator for NoopNewRealmOperator {
    async fn execute(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: NewRealmParams,
    ) -> Result<NewRealmOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "NewRealmOperator pending impl in WF-1-2066/2071".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_phase_name_matches_dtl_feature_subtype() {
        let op = NoopNewRealmOperator;
        assert_eq!(op.phase_name(), "new_realm");
    }
}
