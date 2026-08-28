//! 归档策略实体（per RGS-DTL-042 §3.1 #6 + §5.2 ArchiveOperator）
//!
//! **核心字段（per DTL-042 §3.1 #6）**：
//! - `hot_archive_years` —— 热归档保留年数（**默认 3 年**，per SPEC §8 / TBD-DTL-042-01）
//! - `cold_archive_years` —— 冷归档保留年数（**默认 10 年**，per SPEC §8）
//! - `storage_redundancy` —— 存储冗余等级（**默认 N+2**，per RSK-LCM-005 缓解）
//! - `gdpr_delete_path` —— GDPR "被遗忘权" 删除通路入口（per NFR-SE-010）
//! - `cross_realm_merge_history` —— 跨服合并回溯保留（per FR-LCM-085）
//!
//! **硬约束（per RGS-SPEC-DTL-042 §3 + §4.3 关键标注）**：
//! - 策略**不**允许运行时修改（FR-LCM-062 精神，archive_policy 在 retire 后即冻结）
//! - 3 年热 + 10 年冷 = 13 年总保留期（per SPEC §8 实测参数）
//! - N+2 冗余不可降级为 N+1（除非 Ulysses 显式签字，per ADR-0055 §4.3）

use crate::realm_lifecycle::error::{LcmError, LcmResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// 默认热归档保留年数（per RGS-SPEC-DTL-042 §8 / TBD-DTL-042-01）
pub const DEFAULT_HOT_RETENTION_YEARS: u32 = 3;

/// 默认冷归档保留年数（per RGS-SPEC-DTL-042 §8）
pub const DEFAULT_COLD_RETENTION_YEARS: u32 = 10;

/// N+2 存储冗余（per RSK-LCM-005 缓解默认）
pub const STORAGE_REDUNDANCY_N_PLUS_2: &str = "n_plus_2";

/// N+1 存储冗余（仅在 Ulysses 显式签字后允许降级）
pub const STORAGE_REDUNDANCY_N_PLUS_1: &str = "n_plus_1";

/// N+3 存储冗余（更高冗余等级，运营可升级到此）
pub const STORAGE_REDUNDANCY_N_PLUS_3: &str = "n_plus_3";

/// 存储冗余等级（per DTL-042 §3.1 #6 chk_storage_redundancy 约束）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StorageRedundancy {
    /// N+1（最少副本数；需 Ulysses 显式签字才能降级到此，per ADR-0055 §4.3）
    NPlus1,
    /// **N+2（默认；per RSK-LCM-005 缓解）**
    #[default]
    NPlus2,
    /// N+3（更高冗余；运营可主动升级）
    NPlus3,
}

impl fmt::Display for StorageRedundancy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StorageRedundancy::NPlus1 => STORAGE_REDUNDANCY_N_PLUS_1,
            StorageRedundancy::NPlus2 => STORAGE_REDUNDANCY_N_PLUS_2,
            StorageRedundancy::NPlus3 => STORAGE_REDUNDANCY_N_PLUS_3,
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for StorageRedundancy {
    type Err = LcmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            STORAGE_REDUNDANCY_N_PLUS_1 => Ok(StorageRedundancy::NPlus1),
            STORAGE_REDUNDANCY_N_PLUS_2 => Ok(StorageRedundancy::NPlus2),
            STORAGE_REDUNDANCY_N_PLUS_3 => Ok(StorageRedundancy::NPlus3),
            other => Err(LcmError::InvalidArchivePolicy(format!(
                "storage_redundancy 必须是 n_plus_1 / n_plus_2 / n_plus_3，实际 = {other}"
            ))),
        }
    }
}

impl StorageRedundancy {
    /// 所需副本数（含原始 + 冗余副本）
    ///
    /// 公式：replica_count = 1 + redundancy
    /// - N+1 → 2 副本
    /// - N+2 → **3 副本（默认）**
    /// - N+3 → 4 副本
    pub const fn required_replica_count(self) -> u8 {
        match self {
            StorageRedundancy::NPlus1 => 2,
            StorageRedundancy::NPlus2 => 3,
            StorageRedundancy::NPlus3 => 4,
        }
    }

    /// 是否为默认等级
    pub const fn is_default(self) -> bool {
        matches!(self, StorageRedundancy::NPlus2)
    }
}

/// 归档分层（hot / cold / gdpr_delete_path）
///
/// 用于判断某 realm 在当前时间点处于哪一层：
/// - `Hot` —— 热归档层（在线 / 可读，查询延迟 < 1s）
/// - `Cold` —— 冷归档层（对象存储，查询延迟 < 10s）
/// - `GdprDeletePath` —— GDPR "被遗忘权" 删除通路已启用
///   （合规查询可能涉及物理擦除/匿名化操作）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveTier {
    /// 热归档层（`hot_archive_years` 内）
    Hot,
    /// 冷归档层（`hot_archive_years` 之后 ~ `cold_archive_years`）
    Cold,
    /// 超过 `cold_archive_years` 即将到期（提前告警）
    ColdExpiring,
    /// GDPR "被遗忘权" 删除通路已开启
    GdprDeletePath,
}

/// 归档策略（per RGS-DTL-042 §3.1 #6）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePolicy {
    /// 策略 ID（PK）
    pub policy_id: Uuid,
    /// 目标 realm_id
    pub target_realm_id: String,
    /// 关联退场计划 ID（FK → retire_plan.plan_id）
    pub retire_plan_id: Uuid,
    /// 热归档保留年数（**默认 3**）
    pub hot_archive_years: u32,
    /// 冷归档保留年数（**默认 10**）
    pub cold_archive_years: u32,
    /// 存储冗余等级（**默认 N+2**）
    pub storage_redundancy: StorageRedundancy,
    /// GDPR "被遗忘权" 删除通路入口标识（per NFR-SE-010 双层审计）
    pub gdpr_delete_path: String,
    /// 跨服合并回溯保留（per FR-LCM-085）
    pub cross_realm_merge_history: bool,
    /// 审批人
    pub approved_by: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl ArchivePolicy {
    /// 工厂：新建归档策略（默认值 = 3 年热 + 10 年冷 + N+2）
    ///
    /// **调用方**：PFAU `executing` 阶段之前，运营 + 架构 + SRE 三方签字后调用
    /// （per RGS-SPEC-DTL-042 §5 认证授权）。
    pub fn new(
        target_realm_id: String,
        retire_plan_id: Uuid,
        gdpr_delete_path: String,
        approved_by: String,
    ) -> Self {
        Self {
            policy_id: Uuid::new_v4(),
            target_realm_id,
            retire_plan_id,
            hot_archive_years: DEFAULT_HOT_RETENTION_YEARS,
            cold_archive_years: DEFAULT_COLD_RETENTION_YEARS,
            storage_redundancy: StorageRedundancy::default(),
            gdpr_delete_path,
            cross_realm_merge_history: true,
            approved_by,
            created_at: Utc::now(),
        }
    }

    /// 工厂：自定义参数（用于运营在审批时调整保留期）
    pub fn with_retention(
        target_realm_id: String,
        retire_plan_id: Uuid,
        hot_archive_years: u32,
        cold_archive_years: u32,
        storage_redundancy: StorageRedundancy,
        gdpr_delete_path: String,
        approved_by: String,
    ) -> LcmResult<Self> {
        if hot_archive_years == 0 {
            return Err(LcmError::InvalidArchivePolicy(
                "hot_archive_years 必须 >= 1".to_string(),
            ));
        }
        if cold_archive_years == 0 {
            return Err(LcmError::InvalidArchivePolicy(
                "cold_archive_years 必须 >= 1".to_string(),
            ));
        }
        if hot_archive_years > 20 {
            return Err(LcmError::InvalidArchivePolicy(format!(
                "hot_archive_years={hot_archive_years} 超过 20 年上限（运营/法务审批才能突破）"
            )));
        }
        if cold_archive_years > 50 {
            return Err(LcmError::InvalidArchivePolicy(format!(
                "cold_archive_years={cold_archive_years} 超过 50 年上限（运营/法务审批才能突破）"
            )));
        }
        if storage_redundancy == StorageRedundancy::NPlus1 {
            // 降级到 N+1 必须 Ulysses 显式签字（per ADR-0055 §4.3 资金/合规关键标注）
            // 本方法**不**接受 `signed_by: Option<String>` 形参，意味着**不**允许
            // 通过业务代码直接降级 — 降级必须由外部 SRE 工具发起且经 UI 显式签字
            return Err(LcmError::InvalidArchivePolicy(
                "N+1 降级必须 Ulysses 显式签字（per ADR-0055 §4.3），业务代码不允许"
                    .to_string(),
            ));
        }
        Ok(Self {
            policy_id: Uuid::new_v4(),
            target_realm_id,
            retire_plan_id,
            hot_archive_years,
            cold_archive_years,
            storage_redundancy,
            gdpr_delete_path,
            cross_realm_merge_history: true,
            approved_by,
            created_at: Utc::now(),
        })
    }

    /// 计算当前归档分层
    ///
    /// **算法**（per DTL-042 §5.2 + SPEC §8）：
    /// 1. 计算 `archive_age_years = (now - retired_at).years`（**注意**：本方法不持有
    ///    `retired_at`；调用方需传入）
    /// 2. 若 `age_years < hot_archive_years` → `Hot`
    /// 3. 若 `hot_archive_years <= age_years < cold_archive_years` → `Cold`
    /// 4. 若 `age_years >= cold_archive_years - 1` → `ColdExpiring`（提前告警）
    /// 5. 超过 `cold_archive_years` 后由运营手工触发 GDPR 删除通路
    pub fn classify_tier(&self, age_years: u32) -> ArchiveTier {
        if age_years < self.hot_archive_years {
            ArchiveTier::Hot
        } else if age_years < self.cold_archive_years.saturating_sub(1) {
            ArchiveTier::Cold
        } else if age_years < self.cold_archive_years {
            ArchiveTier::ColdExpiring
        } else {
            // 超过 cold_archive_years：调用方需走 GDPR 删除通路决策
            ArchiveTier::GdprDeletePath
        }
    }

    /// N+2 副本是否满足（per RSK-LCM-005 缓解）
    ///
    /// 真实路径会读对象存储的副本清单（`get_object_replicas`），
    /// 本方法为**判定入口**，调用方传入实测副本数。
    pub fn replica_count_satisfied(&self, actual_replica_count: u8) -> bool {
        actual_replica_count >= self.storage_redundancy.required_replica_count()
    }

    /// 策略是否合法（3 年热 + 10 年冷 + N+2 默认配置，per SPEC §8 + DTL-042 §3.1 #6）
    pub fn validate(&self) -> LcmResult<()> {
        if self.hot_archive_years == 0 || self.cold_archive_years == 0 {
            return Err(LcmError::InvalidArchivePolicy(
                "hot/cold archive years 必须 >= 1".to_string(),
            ));
        }
        if self.gdpr_delete_path.is_empty() {
            return Err(LcmError::InvalidArchivePolicy(
                "gdpr_delete_path 不能为空（per NFR-SE-010 双层审计）".to_string(),
            ));
        }
        if self.approved_by.is_empty() {
            return Err(LcmError::InvalidArchivePolicy(
                "approved_by 不能为空（三方签字）".to_string(),
            ));
        }
        Ok(())
    }

    /// 总保留期年数 = hot + cold
    pub const fn total_retention_years(&self) -> u32 {
        self.hot_archive_years + self.cold_archive_years
    }
}

impl Default for ArchivePolicy {
    fn default() -> Self {
        // 仅供测试 / 占位；业务代码**不**应使用 Default
        Self::new(
            "test-realm".to_string(),
            Uuid::nil(),
            "admin_db.operation_audit".to_string(),
            "system-default".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_retention_thresholds_match_spec() {
        // SPEC-DTL-042 §8: 归档冷热分层阈值 = 3 年热 + 10 年冷
        assert_eq!(DEFAULT_HOT_RETENTION_YEARS, 3);
        assert_eq!(DEFAULT_COLD_RETENTION_YEARS, 10);
    }

    #[test]
    fn storage_redundancy_n_plus_2_is_default() {
        // RSK-LCM-005: N+2 为默认冗余等级
        let policy = ArchivePolicy::default();
        assert_eq!(policy.storage_redundancy, StorageRedundancy::NPlus2);
        assert_eq!(policy.storage_redundancy.required_replica_count(), 3);
    }

    #[test]
    fn storage_redundancy_required_replica_count_matches_dtl() {
        // DTL-042 §3.1 #6 chk_storage_redundancy:
        //   n_plus_1 = 2 副本, n_plus_2 = 3 副本, n_plus_3 = 4 副本
        assert_eq!(StorageRedundancy::NPlus1.required_replica_count(), 2);
        assert_eq!(StorageRedundancy::NPlus2.required_replica_count(), 3);
        assert_eq!(StorageRedundancy::NPlus3.required_replica_count(), 4);
    }

    #[test]
    fn classify_tier_hot_within_3_years() {
        // SPEC §8: 热归档 3 年内
        let p = ArchivePolicy::default();
        assert_eq!(p.classify_tier(0), ArchiveTier::Hot);
        assert_eq!(p.classify_tier(1), ArchiveTier::Hot);
        assert_eq!(p.classify_tier(2), ArchiveTier::Hot);
    }

    #[test]
    fn classify_tier_cold_between_3_and_9_years() {
        // SPEC §8: 冷归档 3 ~ 10 年（expiring 在最后 1 年）
        let p = ArchivePolicy::default();
        assert_eq!(p.classify_tier(3), ArchiveTier::Cold);
        assert_eq!(p.classify_tier(5), ArchiveTier::Cold);
        assert_eq!(p.classify_tier(8), ArchiveTier::Cold);
    }

    #[test]
    fn classify_tier_cold_expiring_at_year_9() {
        // 最后 1 年（cold_archive_years - 1 = 9）标记为 ColdExpiring
        let p = ArchivePolicy::default();
        assert_eq!(p.classify_tier(9), ArchiveTier::ColdExpiring);
    }

    #[test]
    fn classify_tier_gdpr_delete_path_at_year_10() {
        // 超过 cold_archive_years → 触发 GDPR 删除通路决策
        let p = ArchivePolicy::default();
        assert_eq!(p.classify_tier(10), ArchiveTier::GdprDeletePath);
        assert_eq!(p.classify_tier(15), ArchiveTier::GdprDeletePath);
    }

    #[test]
    fn n_plus_1_downgrade_rejected_without_explicit_sign() {
        // ADR-0055 §4.3: N+1 降级必须 Ulysses 显式签字，业务代码**不**允许
        let result = ArchivePolicy::with_retention(
            "r-1".to_string(),
            Uuid::new_v4(),
            3,
            10,
            StorageRedundancy::NPlus1,
            "admin_db.operation_audit".to_string(),
            "ops".to_string(),
        );
        assert!(matches!(result, Err(LcmError::InvalidArchivePolicy(_))));
    }

    #[test]
    fn n_plus_2_with_custom_retention_accepted() {
        // 标准 3 + 10 = 13 年总保留期（per SPEC §8）
        let p = ArchivePolicy::with_retention(
            "r-1".to_string(),
            Uuid::new_v4(),
            3,
            10,
            StorageRedundancy::NPlus2,
            "admin_db.operation_audit".to_string(),
            "ops+arch+sre".to_string(),
        )
        .expect("N+2 with 3+10 retention must be accepted");
        assert_eq!(p.hot_archive_years, 3);
        assert_eq!(p.cold_archive_years, 10);
        assert_eq!(p.total_retention_years(), 13);
    }

    #[test]
    fn n_plus_3_upgrade_accepted() {
        // 升级到 N+3 不需 Ulysses 签字（更高冗余）
        let p = ArchivePolicy::with_retention(
            "r-1".to_string(),
            Uuid::new_v4(),
            3,
            10,
            StorageRedundancy::NPlus3,
            "admin_db.operation_audit".to_string(),
            "ops".to_string(),
        )
        .expect("N+3 upgrade must be accepted");
        assert_eq!(p.storage_redundancy, StorageRedundancy::NPlus3);
        assert_eq!(p.storage_redundancy.required_replica_count(), 4);
    }

    #[test]
    fn zero_retention_rejected() {
        // hot/cold archive_years = 0 不允许
        let r = ArchivePolicy::with_retention(
            "r-1".to_string(),
            Uuid::new_v4(),
            0,
            10,
            StorageRedundancy::NPlus2,
            "admin_db.operation_audit".to_string(),
            "ops".to_string(),
        );
        assert!(matches!(r, Err(LcmError::InvalidArchivePolicy(_))));
    }

    #[test]
    fn empty_gdpr_path_rejected() {
        // gdpr_delete_path 不能为空（per NFR-SE-010）
        let p = ArchivePolicy {
            gdpr_delete_path: "".to_string(),
            ..ArchivePolicy::default()
        };
        let v = p.validate();
        assert!(matches!(v, Err(LcmError::InvalidArchivePolicy(_))));
    }

    #[test]
    fn replica_count_satisfied_helper() {
        let p = ArchivePolicy::default();
        // N+2 = 3 副本要求
        assert!(p.replica_count_satisfied(3));
        assert!(p.replica_count_satisfied(4));
        assert!(!p.replica_count_satisfied(2));
        assert!(!p.replica_count_satisfied(1));
    }

    #[test]
    fn storage_redundancy_parse_roundtrip() {
        for r in [
            StorageRedundancy::NPlus1,
            StorageRedundancy::NPlus2,
            StorageRedundancy::NPlus3,
        ] {
            let s = r.to_string();
            let parsed: StorageRedundancy = s.parse().expect("parse");
            assert_eq!(parsed, r);
        }
    }

    #[test]
    fn storage_redundancy_invalid_string_rejected() {
        let r: Result<StorageRedundancy, _> = "n_plus_99".parse();
        assert!(matches!(r, Err(LcmError::InvalidArchivePolicy(_))));
    }
}
