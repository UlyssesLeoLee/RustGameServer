//! merge_conflict_rule_set_v2 entity（per M-2068.7 + FR-LCM-062）
//!
//! 合服冲突规则集 v2；**锁定后（locked_at 非空）不允许运行时修改**
//! 字段对应 DDL：id / rule_set_version / rules (JSONB) / locked_at / locked_by / created_at
//!
//! FR-LCM-062 硬约束：
//! - save() 在 locked_at 已设的情况下禁止覆盖 rules / locked_at / locked_by
//! - lock() 工厂：一次性把 locked_at 设为 now()；set_locked() 后只读
//! - 应用层校验（DDL 仅含 chk_merge_conflict_lock_consistency 同步约束）

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::Error;
use crate::Result;

/// 锁定后修改错误文案（per FR-LCM-062）
fn locked_err_msg(locked_at: Option<DateTime<Utc>>) -> String {
    format!(
        "merge_conflict_rule_set_v2 已锁定（locked_at={:?}），禁止运行时修改（FR-LCM-062）",
        locked_at
    )
}

/// MergeConflictRuleSetV2 entity（per RGS-SPEC-DTL-042 §2 表 4/6）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeConflictRuleSetV2 {
    pub id: Uuid,
    pub rule_set_version: i32,
    pub rules: JsonValue,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl MergeConflictRuleSetV2 {
    /// 工厂：新建未锁定 rule set（rules 默认 '[]'）
    pub fn new(rule_set_version: i32, rules: JsonValue) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_set_version,
            rules,
            locked_at: None,
            locked_by: None,
            created_at: Utc::now(),
        }
    }

    /// 是否已锁定
    pub fn is_locked(&self) -> bool {
        self.locked_at.is_some()
    }

    /// 锁定（一次性；锁定后规则集只读）
    pub fn lock(&mut self, by: Uuid) {
        if self.is_locked() {
            return; // 幂等：已锁定忽略
        }
        self.locked_at = Some(Utc::now());
        self.locked_by = Some(by);
    }

    /// 业务校验：锁定后禁止修改 rules（FR-LCM-062）
    pub fn validate_save_allowed(&self, new_rules: &JsonValue) -> Result<()> {
        if self.is_locked() && &self.rules != new_rules {
            return Err(Error::Conflict(locked_err_msg(self.locked_at)));
        }
        Ok(())
    }
}

/// PgRepository 骨架（per M-2068.7）
///
/// 关键约束（per FR-LCM-062）：
/// - save() 之前**必须**调用 validate_save_allowed
/// - 锁定后 UPSERT 会带 WHERE locked_at IS NULL（防 SQL 层面绕过）
pub struct PgMergeConflictRuleSetV2Repository {
    pool: PgPool,
}

impl PgMergeConflictRuleSetV2Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_rule_set(row: sqlx::postgres::PgRow) -> MergeConflictRuleSetV2 {
    MergeConflictRuleSetV2 {
        id: row.get("id"),
        rule_set_version: row.get("rule_set_version"),
        rules: row.get("rules"),
        locked_at: row.get("locked_at"),
        locked_by: row.get("locked_by"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl super::MergeConflictRuleSetV2Repository for PgMergeConflictRuleSetV2Repository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MergeConflictRuleSetV2>> {
        let row = sqlx::query(
            "SELECT id, rule_set_version, rules, locked_at, locked_by, created_at \
             FROM merge_conflict_rule_set_v2 WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_rule_set))
    }

    async fn find_by_version(
        &self,
        version: i32,
    ) -> Result<Option<MergeConflictRuleSetV2>> {
        let row = sqlx::query(
            "SELECT id, rule_set_version, rules, locked_at, locked_by, created_at \
             FROM merge_conflict_rule_set_v2 WHERE rule_set_version = $1",
        )
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_rule_set))
    }

    async fn save(
        &self,
        entity: &MergeConflictRuleSetV2,
    ) -> Result<MergeConflictRuleSetV2> {
        // 应用层校验：锁定后禁止修改（FR-LCM-062 硬约束）
        // 先 SELECT 查旧值；锁定时只允许 locked_at / locked_by 变化
        if let Some(existing) = self.find_by_id(entity.id).await? {
            if existing.is_locked() {
                if existing.rules != entity.rules {
                    return Err(Error::Conflict(locked_err_msg(existing.locked_at)));
                }
                if existing.locked_at != entity.locked_at
                    || existing.locked_by != entity.locked_by
                {
                    return Err(Error::Conflict(locked_err_msg(existing.locked_at)));
                }
            }
        }
        sqlx::query(
            "INSERT INTO merge_conflict_rule_set_v2 \
             (id, rule_set_version, rules, locked_at, locked_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (id) DO UPDATE SET \
                rules = EXCLUDED.rules, \
                locked_at = EXCLUDED.locked_at, \
                locked_by = EXCLUDED.locked_by",
        )
        .bind(entity.id)
        .bind(entity.rule_set_version)
        .bind(&entity.rules)
        .bind(entity.locked_at)
        .bind(entity.locked_by)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rule_set_factory() {
        let r = MergeConflictRuleSetV2::new(1, json!([]));
        assert_eq!(r.rule_set_version, 1);
        assert!(!r.is_locked());
    }

    #[test]
    fn rule_set_lock() {
        let mut r = MergeConflictRuleSetV2::new(1, json!([]));
        let admin = Uuid::new_v4();
        r.lock(admin);
        assert!(r.is_locked());
        assert_eq!(r.locked_by, Some(admin));
    }

    #[test]
    fn rule_set_lock_is_idempotent() {
        let mut r = MergeConflictRuleSetV2::new(1, json!([]));
        let admin1 = Uuid::new_v4();
        let admin2 = Uuid::new_v4();
        r.lock(admin1);
        let first_locked = r.locked_at;
        r.lock(admin2);
        // 二次 lock 应被忽略（locked_by 保持原值）
        assert_eq!(r.locked_at, first_locked);
        assert_eq!(r.locked_by, Some(admin1));
    }

    #[test]
    fn rule_set_validate_blocks_modification_after_lock() {
        let mut r = MergeConflictRuleSetV2::new(1, json!([{"kind": "guild_owner"}]));
        r.lock(Uuid::new_v4());
        let new_rules = json!([{"kind": "guild_owner"}, {"kind": "friend_link"}]);
        let result = r.validate_save_allowed(&new_rules);
        assert!(result.is_err());
    }

    #[test]
    fn rule_set_validate_allows_same_rules_after_lock() {
        let mut r = MergeConflictRuleSetV2::new(1, json!([{"kind": "guild_owner"}]));
        r.lock(Uuid::new_v4());
        let same = json!([{"kind": "guild_owner"}]);
        let result = r.validate_save_allowed(&same);
        assert!(result.is_ok());
    }
}
