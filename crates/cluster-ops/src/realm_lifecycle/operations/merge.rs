//! Merge + MergeRollback 操作器（合服 + 回退，per AC-LCM-004/005 + DTL-042 §5.4 + FR-LCM-062）。
//!
//! 步骤（合服）：冲突规则 v2 加载 → 玩家数据合并 → merge_conflict_rule_set_v2 锁定 →
//!                7 步 Saga → 合并完成 → 启动回退窗口期（7-30 天）。
//! 步骤（回退）：检测 window 内回退请求 → 玩家数据切回 → 冲突规则解锁失败（已锁不可改）→
//!                merge_conflict_rule_set_v2.locked_at 保持（FR-LCM-062）。
//!
//! 本 worktree（WF-1-2070）只定义 trait 签名 + 占位 operator。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::PhaseOperator;
use crate::realm_lifecycle::{
    error::Result, ApprovalRef, OperatorId, RealmId, RequestId, SagaRunId, TraceId,
};

/// Merge 计划参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeParams {
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub conflict_rule_set_version: u32,
    /// 回退窗口期（天，per SPEC §8 实测参数 7-30 天）。
    pub rollback_window_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOutcome {
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub merged_player_count: u64,
    pub conflict_rule_set_locked_at: DateTime<Utc>,
    pub rollback_window_until: DateTime<Utc>,
}

/// MergeRollback 参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRollbackParams {
    pub saga_run_id: SagaRunId,
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRollbackOutcome {
    pub saga_run_id: SagaRunId,
    pub rollback_completed_at: DateTime<Utc>,
    pub players_restored: u64,
    /// FR-LCM-062 语义：merge_conflict_rule_set_v2 锁定后不可改；
    /// 回退时 locked_at 保持不变。
    pub conflict_rule_set_locked_at: DateTime<Utc>,
}

#[async_trait]
pub trait MergeOperator: PhaseOperator {
    async fn execute(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: MergeParams,
    ) -> Result<MergeOutcome>;

    async fn rollback(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: MergeRollbackParams,
    ) -> Result<MergeRollbackOutcome>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopMergeOperator;

impl PhaseOperator for NoopMergeOperator {
    fn phase_name(&self) -> &'static str {
        "merge"
    }
}

#[async_trait]
impl MergeOperator for NoopMergeOperator {
    async fn execute(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: MergeParams,
    ) -> Result<MergeOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "MergeOperator::execute pending impl in WF-1-2066/2071".to_string(),
        ))
    }

    async fn rollback(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: MergeRollbackParams,
    ) -> Result<MergeRollbackOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "MergeOperator::rollback pending impl in WF-1-2066/2071".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_phase_name() {
        assert_eq!(NoopMergeOperator.phase_name(), "merge");
    }
}
