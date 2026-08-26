//! Scale 操作器（扩缩容，per AC-LCM-002 + DTL-042 §5.2）。
//!
//! 步骤：判定 scale_up / scale_down → admin_db.realm_lifecycle_run 写 run →
//! sandbox_k8s 调 K3s 演练 namespace 副本数。
//!
//! 本 worktree（WF-1-2070）只定义 trait 签名 + 占位 operator。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::PhaseOperator;
use crate::realm_lifecycle::{
    error::Result, ApprovalRef, OperatorId, RealmId, RequestId, TraceId,
};

/// 扩缩容方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleDirection {
    Up,
    Down,
}

impl ScaleDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// Scale 计划参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleParams {
    pub realm_id: RealmId,
    pub direction: ScaleDirection,
    pub target_node_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleOutcome {
    pub realm_id: RealmId,
    pub from_count: u32,
    pub to_count: u32,
    pub direction: ScaleDirection,
}

#[async_trait]
pub trait ScaleOperator: PhaseOperator {
    async fn execute(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: ScaleParams,
    ) -> Result<ScaleOutcome>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopScaleOperator;

impl PhaseOperator for NoopScaleOperator {
    fn phase_name(&self) -> &'static str {
        "scale"
    }
}

#[async_trait]
impl ScaleOperator for NoopScaleOperator {
    async fn execute(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: ScaleParams,
    ) -> Result<ScaleOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "ScaleOperator pending impl in WF-1-2066/2071".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_direction_as_str() {
        assert_eq!(ScaleDirection::Up.as_str(), "up");
        assert_eq!(ScaleDirection::Down.as_str(), "down");
    }

    #[test]
    fn noop_phase_name() {
        assert_eq!(NoopScaleOperator.phase_name(), "scale");
    }
}
