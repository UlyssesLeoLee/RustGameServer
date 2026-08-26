//! `merge_conflict_rule_set_v2` 表（per DTL-042 §7.2 + IMPL §3.3 M-2068.2 + FR-LCM-062）。
//!
//! ## 硬约束（per FR-LCM-062）
//!
//! `locked_at` 锁定后**不**允许运行时修改。
//! `check_locked` 必须在所有写路径前置检查；锁定后写 → `Error::MergeRulesLocked`。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::{
    error::{Error, Result},
    RealmId,
};

/// v2 冲突规则集合（per SPEC §6 56 条 UT 拆分中"冲突规则 v2"项）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflictRuleSetV2 {
    pub rule_set_id: String,
    pub version: u32,
    pub rules: Vec<ConflictRule>,
    /// FR-LCM-062 锚定字段：`locked_at = Some(_)` 后**不**允许运行时修改。
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 单条冲突规则（v2 新增 3 类规则之一）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRule {
    pub rule_id: String,
    pub rule_kind: ConflictRuleKind,
    pub priority: i32,
    pub description: String,
}

/// v2 冲突规则类型（v2 新增 3 类）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictRuleKind {
    /// 玩家名冲突：源服玩家名 + 服区后缀
    PlayerNameWithRealmSuffix,
    /// 公会名冲突：源服公会 + 服区后缀
    GuildNameWithRealmSuffix,
    /// 工会战积分：取 MAX（避免双计）
    GuildWarScoreMax,
}

impl ConflictRuleKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PlayerNameWithRealmSuffix => "player_name_with_realm_suffix",
            Self::GuildNameWithRealmSuffix => "guild_name_with_realm_suffix",
            Self::GuildWarScoreMax => "guild_war_score_max",
        }
    }
}

impl MergeConflictRuleSetV2 {
    /// FR-LCM-062 锚定：检查锁定状态。
    ///
    /// 锁定后调用方**不**应尝试修改。
    pub fn check_locked(&self) -> Result<()> {
        if self.locked_at.is_some() {
            Err(Error::MergeRulesLocked {
                realm: RealmId::from("<rule_set>"),
                locked_at: self.locked_at.unwrap_or_else(Utc::now),
            })
        } else {
            Ok(())
        }
    }

    /// 锁定操作（仅在未锁定时可调用）。
    pub fn lock(&mut self, operator_id: &str) -> Result<()> {
        self.check_locked()?;
        self.locked_at = Some(Utc::now());
        self.locked_by = Some(operator_id.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MergeConflictRuleSetV2 {
        MergeConflictRuleSetV2 {
            rule_set_id: "rs-1".to_string(),
            version: 2,
            rules: vec![ConflictRule {
                rule_id: "r-1".to_string(),
                rule_kind: ConflictRuleKind::PlayerNameWithRealmSuffix,
                priority: 100,
                description: "player name with realm suffix".to_string(),
            }],
            locked_at: None,
            locked_by: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn unlocked_check_passes() {
        let rs = sample();
        assert!(rs.check_locked().is_ok());
    }

    #[test]
    fn locked_check_fails_fr_lcm_062() {
        // FR-LCM-062 锚定：锁定后 check_locked 必须返回 MergeRulesLocked
        let mut rs = sample();
        rs.lock("sre-1").unwrap();
        let r = rs.check_locked();
        assert!(matches!(r, Err(Error::MergeRulesLocked { .. })));
    }

    #[test]
    fn double_lock_fails() {
        let mut rs = sample();
        rs.lock("sre-1").unwrap();
        // 二次 lock 应失败
        let r = rs.lock("sre-2");
        assert!(matches!(r, Err(Error::MergeRulesLocked { .. })));
    }

    #[test]
    fn rule_kind_as_str_covers_v2_three_kinds() {
        // SPEC §6：冲突规则 v2 新增 3 类
        assert_eq!(
            ConflictRuleKind::PlayerNameWithRealmSuffix.as_str(),
            "player_name_with_realm_suffix"
        );
        assert_eq!(
            ConflictRuleKind::GuildNameWithRealmSuffix.as_str(),
            "guild_name_with_realm_suffix"
        );
        assert_eq!(
            ConflictRuleKind::GuildWarScoreMax.as_str(),
            "guild_war_score_max"
        );
    }
}
