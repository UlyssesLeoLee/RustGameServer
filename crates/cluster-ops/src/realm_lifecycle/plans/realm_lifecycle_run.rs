//! `realm_lifecycle_run` 主运行记录表（per DTL-042 §7.1 + IMPL §3.3 M-2068.1）。
//!
//! 按 `created_at` 月度范围分区（与既有 `operation_audit` 同构，per SPEC §3 第 4 条）。
//! 本 worktree（WF-1-2070）只定义 entity 结构；DDL 在 `migrations/0020_lcm_tables.sql`
//! 由 WF-1-2068 后续 worktree 补齐。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::{OperatorId, RealmId, RequestId, SagaRunId, TraceId};

use super::super::RealmStatus;

/// `realm_lifecycle_run` 主表 row（per DTL-042 §7.1）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmLifecycleRun {
    pub run_id: String,
    pub request_id: RequestId,
    pub operator_id: OperatorId,
    pub approval_ref: Option<String>,
    pub trace_id: TraceId,
    pub realm_id: RealmId,
    pub phase: String,
    pub status: RealmStatus,
    pub saga_run_id: Option<SagaRunId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl RealmLifecycleRun {
    /// 锚定 SPEC §3 第 4 条：(request_id, operator_id) 唯一索引验证。
    pub fn idempotency_key(&self) -> (&RequestId, &OperatorId) {
        (&self.request_id, &self.operator_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_key_returns_request_and_operator() {
        let run = RealmLifecycleRun {
            run_id: "run-1".to_string(),
            request_id: "req-1".to_string(),
            operator_id: "op-1".to_string(),
            approval_ref: None,
            trace_id: "t-1".to_string(),
            realm_id: "rlm-1".to_string(),
            phase: "new_realm".to_string(),
            status: RealmStatus::Active,
            saga_run_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };
        let (req, op) = run.idempotency_key();
        assert_eq!(req, "req-1");
        assert_eq!(op, "op-1");
    }
}
