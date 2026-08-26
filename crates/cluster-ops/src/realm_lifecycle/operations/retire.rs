//! Retire 操作器（退场，per AC-LCM-006 + DTL-042 §5.5 + SPEC §3 第 8 条）。
//!
//! 步骤：retire_plan 创建（含 query_channel_rbac 角色配置，默认 cs_agent/sre/legal）→
//! 30-90 天后启动归档（per SPEC §8 实测参数）。
//!
//! 本 worktree（WF-1-2070）只定义 trait 签名 + 占位 operator。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::PhaseOperator;
use crate::realm_lifecycle::{
    error::Result, ApprovalRef, OperatorId, RealmId, RequestId, TraceId,
};

/// 退场后 RBAC 查询通道允许的角色（per SPEC §3 第 8 条）。
///
/// 默认 `["cs_agent", "sre", "legal"]`；其他角色访问应被拒绝（per
/// `RetiredQueryDenied` 错误）。
pub const DEFAULT_RETIRE_QUERY_ROLES: &[&str] = &["cs_agent", "sre", "legal"];

/// Retire 计划参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetireParams {
    pub realm_id: RealmId,
    pub query_channel_rbac: Vec<String>,
    /// 退场后归档启动阈值（天，per SPEC §8 实测参数 30-90 天）。
    pub archive_threshold_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetireOutcome {
    pub realm_id: RealmId,
    pub retired_at: DateTime<Utc>,
    pub query_channel_rbac: Vec<String>,
    pub archive_scheduled_at: DateTime<Utc>,
}

#[async_trait]
pub trait RetireOperator: PhaseOperator {
    async fn execute(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: RetireParams,
    ) -> Result<RetireOutcome>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRetireOperator;

impl PhaseOperator for NoopRetireOperator {
    fn phase_name(&self) -> &'static str {
        "retire"
    }
}

#[async_trait]
impl RetireOperator for NoopRetireOperator {
    async fn execute(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: RetireParams,
    ) -> Result<RetireOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "RetireOperator pending impl in WF-1-2066/2071/2073".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_retire_query_roles_match_spec() {
        // SPEC §3 第 8 条：默认 cs_agent / sre / legal
        assert_eq!(DEFAULT_RETIRE_QUERY_ROLES, &["cs_agent", "sre", "legal"]);
    }

    #[test]
    fn noop_phase_name() {
        assert_eq!(NoopRetireOperator.phase_name(), "retire");
    }
}
