//! archive_policy entity（per M-2068.7 + FR-LCM-081 + NFR-SE-010）
//!
//! 归档策略：仅迁移存储位置，**不**删除数据
//! N+2 冗余（per RSK-LCM-005 缓解）
//! 3 年热 + 10 年冷（per RGS-SPEC-DTL-042 §8 Gate 证据）
//!
//! 硬约束：
//! - **不**含数据销毁路径（per NFR-SE-010 双层审计，仅追加）
//! - GDPR 删除通路走 `admin_db.audit_log`，**不**走本表
//! - 本 entity 不暴露任何数据销毁方法

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::Result;

/// ArchivePolicy entity（per RGS-SPEC-DTL-042 §2 表 6/6）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchivePolicy {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub hot_storage_tier: String,
    pub cold_storage_tier: String,
    pub hot_retention_years: i32,
    pub cold_retention_years: i32,
    pub n_plus_2_redundancy: bool,
    pub created_at: DateTime<Utc>,
}

impl ArchivePolicy {
    /// 工厂：新建归档策略
    /// - hot_retention_years >= 3
    /// - cold_retention_years >= 10
    /// - n_plus_2_redundancy 默认 true
    pub fn new(
        realm_id: Uuid,
        hot_storage_tier: String,
        cold_storage_tier: String,
        hot_retention_years: i32,
        cold_retention_years: i32,
    ) -> Self {
        assert!(hot_retention_years >= 3, "hot_retention_years 必须 >= 3（DDL CHECK 约束）");
        assert!(cold_retention_years >= 10, "cold_retention_years 必须 >= 10（DDL CHECK 约束）");
        Self {
            id: Uuid::new_v4(),
            realm_id,
            hot_storage_tier,
            cold_storage_tier,
            hot_retention_years,
            cold_retention_years,
            n_plus_2_redundancy: true,
            created_at: Utc::now(),
        }
    }

    /// 切换 N+2 冗余开关
    pub fn set_n_plus_2(&mut self, enabled: bool) {
        self.n_plus_2_redundancy = enabled;
    }
}

/// PgRepository 骨架
///
/// **本 Repository 不提供任何数据销毁方法**（per FR-LCM-081 + NFR-SE-010）
/// 归档迁移通过 save() 的 UPSERT 完成（update_storage_tier 等由业务层调用 save）
pub struct PgArchivePolicyRepository {
    pool: PgPool,
}

impl PgArchivePolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_archive_policy(row: sqlx::postgres::PgRow) -> ArchivePolicy {
    ArchivePolicy {
        id: row.get("id"),
        realm_id: row.get("realm_id"),
        hot_storage_tier: row.get("hot_storage_tier"),
        cold_storage_tier: row.get("cold_storage_tier"),
        hot_retention_years: row.get("hot_retention_years"),
        cold_retention_years: row.get("cold_retention_years"),
        n_plus_2_redundancy: row.get("n_plus_2_redundancy"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl super::ArchivePolicyRepository for PgArchivePolicyRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ArchivePolicy>> {
        let row = sqlx::query(
            "SELECT id, realm_id, hot_storage_tier, cold_storage_tier, \
             hot_retention_years, cold_retention_years, n_plus_2_redundancy, created_at \
             FROM archive_policy WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_archive_policy))
    }

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Option<ArchivePolicy>> {
        let row = sqlx::query(
            "SELECT id, realm_id, hot_storage_tier, cold_storage_tier, \
             hot_retention_years, cold_retention_years, n_plus_2_redundancy, created_at \
             FROM archive_policy WHERE realm_id = $1",
        )
        .bind(realm_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_archive_policy))
    }

    async fn save(&self, entity: &ArchivePolicy) -> Result<ArchivePolicy> {
        sqlx::query(
            "INSERT INTO archive_policy \
             (id, realm_id, hot_storage_tier, cold_storage_tier, \
              hot_retention_years, cold_retention_years, n_plus_2_redundancy, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                hot_storage_tier = EXCLUDED.hot_storage_tier, \
                cold_storage_tier = EXCLUDED.cold_storage_tier, \
                hot_retention_years = EXCLUDED.hot_retention_years, \
                cold_retention_years = EXCLUDED.cold_retention_years, \
                n_plus_2_redundancy = EXCLUDED.n_plus_2_redundancy",
        )
        .bind(entity.id)
        .bind(entity.realm_id)
        .bind(&entity.hot_storage_tier)
        .bind(&entity.cold_storage_tier)
        .bind(entity.hot_retention_years)
        .bind(entity.cold_retention_years)
        .bind(entity.n_plus_2_redundancy)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }
    // 故意不提供任何数据销毁方法 —— per FR-LCM-081 + NFR-SE-010
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_policy_factory_defaults() {
        let p = ArchivePolicy::new(
            Uuid::new_v4(),
            "nvme".to_string(),
            "object_storage".to_string(),
            3,
            10,
        );
        assert_eq!(p.hot_retention_years, 3);
        assert_eq!(p.cold_retention_years, 10);
        assert!(p.n_plus_2_redundancy);
    }

    #[test]
    fn archive_policy_set_n_plus_2() {
        let mut p = ArchivePolicy::new(
            Uuid::new_v4(),
            "ssd".to_string(),
            "tape".to_string(),
            5,
            15,
        );
        p.set_n_plus_2(false);
        assert!(!p.n_plus_2_redundancy);
    }

    #[test]
    #[should_panic(expected = "hot_retention_years 必须 >= 3")]
    fn archive_policy_rejects_hot_lt_3() {
        let _ = ArchivePolicy::new(
            Uuid::new_v4(),
            "nvme".to_string(),
            "object_storage".to_string(),
            2,
            10,
        );
    }

    #[test]
    #[should_panic(expected = "cold_retention_years 必须 >= 10")]
    fn archive_policy_rejects_cold_lt_10() {
        let _ = ArchivePolicy::new(
            Uuid::new_v4(),
            "nvme".to_string(),
            "object_storage".to_string(),
            3,
            9,
        );
    }
}
