//! retire_plan entity（per M-2068.7）
//!
//! 退场计划；query_channel_rbac JSONB 配置退场后查询通道的允许角色（per FR-LCM-007）
//! 默认 ["cs_agent", "sre", "legal"]；archive_threshold_days CHECK 30-90

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::Result;

/// retire_plan 状态机
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetirePlanStatus {
    Draft,
    Validated,
    Executing,
    Done,
    Failed,
}

impl RetirePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RetirePlanStatus::Draft => "draft",
            RetirePlanStatus::Validated => "validated",
            RetirePlanStatus::Executing => "executing",
            RetirePlanStatus::Done => "done",
            RetirePlanStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "draft" => RetirePlanStatus::Draft,
            "validated" => RetirePlanStatus::Validated,
            "executing" => RetirePlanStatus::Executing,
            "done" => RetirePlanStatus::Done,
            "failed" => RetirePlanStatus::Failed,
            _ => RetirePlanStatus::Draft,
        }
    }
}

/// RetirePlan entity（per RGS-SPEC-DTL-042 §2 表 5/6）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetirePlan {
    pub id: Uuid,
    pub run_id: Uuid,
    pub target_realm_id: Uuid,
    pub archive_threshold_days: i32,
    pub query_channel_rbac: JsonValue,
    pub status: RetirePlanStatus,
    pub created_at: DateTime<Utc>,
}

impl RetirePlan {
    /// 工厂：新建 draft 状态 plan；默认 rbac = ["cs_agent","sre","legal"]
    pub fn new(
        run_id: Uuid,
        target_realm_id: Uuid,
        archive_threshold_days: i32,
    ) -> Self {
        assert!(
            (30..=90).contains(&archive_threshold_days),
            "archive_threshold_days 必须在 30-90 之间（DDL CHECK 约束）"
        );
        Self {
            id: Uuid::new_v4(),
            run_id,
            target_realm_id,
            archive_threshold_days,
            query_channel_rbac: serde_json::json!(["cs_agent", "sre", "legal"]),
            status: RetirePlanStatus::Draft,
            created_at: Utc::now(),
        }
    }

    /// 自定义 query_channel_rbac（覆盖默认三角色）
    pub fn with_query_channel_rbac(mut self, rbac: JsonValue) -> Self {
        self.query_channel_rbac = rbac;
        self
    }
}

/// PgRepository 骨架
pub struct PgRetirePlanRepository {
    pool: PgPool,
}

impl PgRetirePlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_retire_plan(row: sqlx::postgres::PgRow) -> RetirePlan {
    let status_str: String = row.get("status");
    RetirePlan {
        id: row.get("id"),
        run_id: row.get("run_id"),
        target_realm_id: row.get("target_realm_id"),
        archive_threshold_days: row.get("archive_threshold_days"),
        query_channel_rbac: row.get("query_channel_rbac"),
        status: RetirePlanStatus::parse(&status_str),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl super::RetirePlanRepository for PgRetirePlanRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RetirePlan>> {
        let row = sqlx::query(
            "SELECT id, run_id, target_realm_id, archive_threshold_days, query_channel_rbac, status, created_at \
             FROM retire_plan WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_retire_plan))
    }

    async fn find_by_run_id(&self, run_id: Uuid) -> Result<Option<RetirePlan>> {
        let row = sqlx::query(
            "SELECT id, run_id, target_realm_id, archive_threshold_days, query_channel_rbac, status, created_at \
             FROM retire_plan WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_retire_plan))
    }

    async fn save(&self, entity: &RetirePlan) -> Result<RetirePlan> {
        sqlx::query(
            "INSERT INTO retire_plan \
             (id, run_id, target_realm_id, archive_threshold_days, query_channel_rbac, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (id) DO UPDATE SET \
                target_realm_id = EXCLUDED.target_realm_id, \
                archive_threshold_days = EXCLUDED.archive_threshold_days, \
                query_channel_rbac = EXCLUDED.query_channel_rbac, \
                status = EXCLUDED.status",
        )
        .bind(entity.id)
        .bind(entity.run_id)
        .bind(entity.target_realm_id)
        .bind(entity.archive_threshold_days)
        .bind(&entity.query_channel_rbac)
        .bind(entity.status.as_str())
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retire_plan_status_roundtrip() {
        for s in [
            RetirePlanStatus::Draft,
            RetirePlanStatus::Validated,
            RetirePlanStatus::Executing,
            RetirePlanStatus::Done,
            RetirePlanStatus::Failed,
        ] {
            assert_eq!(RetirePlanStatus::parse(s.as_str()), s);
        }
    }

    #[test]
    fn retire_plan_factory_default_rbac() {
        let p = RetirePlan::new(Uuid::new_v4(), Uuid::new_v4(), 60);
        assert_eq!(p.archive_threshold_days, 60);
        assert_eq!(p.query_channel_rbac, serde_json::json!(["cs_agent", "sre", "legal"]));
    }

    #[test]
    fn retire_plan_factory_custom_rbac() {
        let p = RetirePlan::new(Uuid::new_v4(), Uuid::new_v4(), 45)
            .with_query_channel_rbac(serde_json::json!(["cs_agent", "legal"]));
        assert_eq!(p.query_channel_rbac, serde_json::json!(["cs_agent", "legal"]));
    }

    #[test]
    #[should_panic(expected = "archive_threshold_days 必须在 30-90 之间")]
    fn retire_plan_factory_rejects_threshold_out_of_range() {
        let _ = RetirePlan::new(Uuid::new_v4(), Uuid::new_v4(), 100);
    }
}
