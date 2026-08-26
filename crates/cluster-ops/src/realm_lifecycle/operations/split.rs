//! Split 操作器（分服，per AC-LCM-003 + DTL-042 §5.3）。
//!
//! 步骤：玩家分布分析 → 选定 split_point → 7 步 Saga（含反向补偿）：
//!   1. 冻结源 realm 写
//!   2. 玩家数据快照
//!   3. 创建目标 realm（NewRealm 子操作）
//!   4. 数据迁移
//!   5. 切流量
//!   6. realm_directory 灰度 0%→100%
//!   7. 解冻源 realm
//!
//! 本 worktree（WF-1-2070）只定义 trait 签名 + 占位 operator。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::PhaseOperator;
use crate::realm_lifecycle::{
    error::Result, ApprovalRef, OperatorId, RealmId, RequestId, TraceId,
};

/// Split 计划参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitParams {
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub split_point_player_id: String,
    pub estimated_players: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitOutcome {
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub migrated_player_count: u64,
    pub saga_steps_completed: u32,
}

#[async_trait]
pub trait SplitOperator: PhaseOperator {
    async fn execute(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: SplitParams,
    ) -> Result<SplitOutcome>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSplitOperator;

impl PhaseOperator for NoopSplitOperator {
    fn phase_name(&self) -> &'static str {
        "split"
    }
}

#[async_trait]
impl SplitOperator for NoopSplitOperator {
    async fn execute(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: SplitParams,
    ) -> Result<SplitOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "SplitOperator pending impl in WF-1-2066/2071".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_phase_name() {
        assert_eq!(NoopSplitOperator.phase_name(), "split");
    }
}
