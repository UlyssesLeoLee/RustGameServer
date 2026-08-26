//! Archive 操作器（归档 + 冷热分层，per AC-LCM-007 + DTL-042 §5.6 + FR-LCM-081 + RSK-LCM-005）。
//!
//! 步骤：冷热分层阈值判定（3 年热 + 10 年冷，per SPEC §8 TBD-DTL-042-01）→
//! 数据迁移到冷/热存储 → N+2 冗余（per RSK-LCM-005 缓解，2 副本）→
//// 不删数据（per FR-LCM-081；操作期**不**执行 DELETE / DROP / truncate）。
//!
//! 本 worktree（WF-1-2070）只定义 trait 签名 + 占位 operator。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::PhaseOperator;
use crate::realm_lifecycle::{
    error::Result, ApprovalRef, OperatorId, RealmId, RequestId, TraceId,
};

/// 冷热分层阈值（per SPEC §8 TBD-DTL-042-01 实测参数）。
pub const HOT_TIER_YEARS: u32 = 3;
pub const COLD_TIER_YEARS: u32 = 10;
/// N+2 存储冗余副本数（per RSK-LCM-005 缓解）。
pub const ARCHIVE_REDUNDANCY: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveTier {
    Hot,
    Cold,
}

impl ArchiveTier {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Cold => "cold",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveParams {
    pub realm_id: RealmId,
    pub last_active_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOutcome {
    pub realm_id: RealmId,
    pub tier: ArchiveTier,
    pub archived_at: DateTime<Utc>,
    pub replica_count: u32,
    /// FR-LCM-081 锚定字段：操作后 row count 必须等于操作前。
    pub row_count_preserved: bool,
}

#[async_trait]
pub trait ArchiveOperator: PhaseOperator {
    async fn execute(
        &self,
        request_id: &RequestId,
        operator_id: &OperatorId,
        approval_ref: &ApprovalRef,
        trace_id: &TraceId,
        params: ArchiveParams,
    ) -> Result<ArchiveOutcome>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopArchiveOperator;

impl PhaseOperator for NoopArchiveOperator {
    fn phase_name(&self) -> &'static str {
        "archive"
    }
}

#[async_trait]
impl ArchiveOperator for NoopArchiveOperator {
    async fn execute(
        &self,
        _request_id: &RequestId,
        _operator_id: &OperatorId,
        _approval_ref: &ApprovalRef,
        _trace_id: &TraceId,
        _params: ArchiveParams,
    ) -> Result<ArchiveOutcome> {
        Err(crate::realm_lifecycle::error::Error::Validation(
            "ArchiveOperator pending impl in WF-1-2066/2071/2074".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_cold_thresholds_match_spec() {
        // SPEC §8 TBD-DTL-042-01：3 年热 + 10 年冷
        assert_eq!(HOT_TIER_YEARS, 3);
        assert_eq!(COLD_TIER_YEARS, 10);
    }

    #[test]
    fn redundancy_is_n_plus_two() {
        // RSK-LCM-005：N+2 存储冗余
        assert_eq!(ARCHIVE_REDUNDANCY, 3);
    }

    #[test]
    fn noop_phase_name() {
        assert_eq!(NoopArchiveOperator.phase_name(), "archive");
    }
}
