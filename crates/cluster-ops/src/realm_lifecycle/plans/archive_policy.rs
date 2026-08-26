//! `archive_policy` 策略表（per DTL-042 §7.2 + IMPL §3.3 M-2068.3 + FR-LCM-081 + RSK-LCM-005）。
//!
//! ## 硬约束
//!
//! - 冷热分层阈值：3 年热 + 10 年冷（per SPEC §8 TBD-DTL-042-01）
//! - N+2 冗余（per RSK-LCM-005 缓解）
//! - 不删数据（per FR-LCM-081）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::{
    error::{Error, Result},
    RealmId,
};

use super::super::operations::archive::{
    ArchiveTier, ARCHIVE_REDUNDANCY, COLD_TIER_YEARS, HOT_TIER_YEARS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePolicy {
    pub policy_id: String,
    pub hot_tier_years: u32,
    pub cold_tier_years: u32,
    pub redundancy: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ArchivePolicy {
    fn default() -> Self {
        Self {
            policy_id: "default".to_string(),
            hot_tier_years: HOT_TIER_YEARS,
            cold_tier_years: COLD_TIER_YEARS,
            redundancy: ARCHIVE_REDUNDANCY,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl ArchivePolicy {
    /// 根据最后活跃时间判定冷热分层。
    pub fn classify(&self, last_active_at: DateTime<Utc>, now: DateTime<Utc>) -> ArchiveTier {
        let age_years = (now - last_active_at).num_days() / 365;
        if age_years <= self.hot_tier_years as i64 {
            ArchiveTier::Hot
        } else {
            ArchiveTier::Cold
        }
    }

    /// 验证策略不变量（per FR-LCM-081 + RSK-LCM-005）。
    pub fn validate(&self) -> Result<()> {
        if self.redundancy < 2 {
            return Err(Error::Validation(
                "archive redundancy must be >= 2 (RSK-LCM-005 N+2)".to_string(),
            ));
        }
        if self.hot_tier_years == 0 || self.cold_tier_years == 0 {
            return Err(Error::Validation(
                "archive tier thresholds must be > 0".to_string(),
            ));
        }
        Ok(())
    }

    /// FR-LCM-081 锚定：归档操作前/后 row count 校验。
    pub fn assert_row_count_preserved(before: u64, after: u64, realm: &RealmId) -> Result<()> {
        if before != after {
            return Err(Error::ArchiveDeleteForbidden {
                realm: realm.clone(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn default_policy_matches_spec() {
        // SPEC §8 TBD-DTL-042-01：3 年热 + 10 年冷
        let p = ArchivePolicy::default();
        assert_eq!(p.hot_tier_years, 3);
        assert_eq!(p.cold_tier_years, 10);
    }

    #[test]
    fn default_redundancy_is_n_plus_two() {
        let p = ArchivePolicy::default();
        assert_eq!(p.redundancy, 3);
    }

    #[test]
    fn classify_hot_within_three_years() {
        let p = ArchivePolicy::default();
        let now = Utc::now();
        let last = now - Duration::days(365);
        assert_eq!(p.classify(last, now), ArchiveTier::Hot);
    }

    #[test]
    fn classify_cold_beyond_three_years() {
        let p = ArchivePolicy::default();
        let now = Utc::now();
        let last = now - Duration::days(365 * 5);
        assert_eq!(p.classify(last, now), ArchiveTier::Cold);
    }

    #[test]
    fn validate_passes_default() {
        assert!(ArchivePolicy::default().validate().is_ok());
    }

    #[test]
    fn validate_rejects_redundancy_lt_2() {
        let mut p = ArchivePolicy::default();
        p.redundancy = 1;
        assert!(p.validate().is_err());
    }

    #[test]
    fn row_count_preserved_passes_when_equal() {
        let r = ArchivePolicy::assert_row_count_preserved(100, 100, &"rlm".to_string());
        assert!(r.is_ok());
    }

    #[test]
    fn row_count_preserved_fails_on_mismatch_fr_lcm_081() {
        let r = ArchivePolicy::assert_row_count_preserved(100, 99, &"rlm".to_string());
        assert!(matches!(r, Err(Error::ArchiveDeleteForbidden { .. })));
    }
}
