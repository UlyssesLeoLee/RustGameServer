//! archive_plan 占位（per RGS-SPEC-DTL-042 §2 + FR-LCM-081）
//!
//! DDL 目标表：`archive_policy`（per SPEC §2 DDL）
//!
//! 归档**不**删除数据，**仅**迁移存储位置（per FR-LCM-081）；
//! GDPR 删除通路走 `admin_db.operation_audit` 双层审计
//! （per SPEC §3 第末段 + NFR-SE-010 合规例外）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchivePolicy {
    pub policy_id: Uuid,
    pub request_id: Uuid,
    pub target_realm_id: Uuid,
    /// 冷热分层阈值（3 年热 + 10 年冷，per SPEC §8 默认值）
    pub hot_retention_days: u32,
    pub cold_retention_days: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ArchivePolicy {
    pub fn placeholder(policy_id: Uuid, request_id: Uuid) -> Self {
        Self {
            policy_id,
            request_id,
            target_realm_id: Uuid::new_v4(),
            hot_retention_days: 3 * 365,
            cold_retention_days: 10 * 365,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_default_retention_thresholds() {
        // per SPEC §8：3 年热 + 10 年冷
        let p = ArchivePolicy::placeholder(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(p.hot_retention_days, 1095);
        assert_eq!(p.cold_retention_days, 3650);
    }
}
