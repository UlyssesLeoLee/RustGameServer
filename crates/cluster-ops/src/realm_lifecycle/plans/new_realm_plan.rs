//! new_realm_plan entity（per M-2068.7）
//!
//! 新建 realm 计划；同 DB 内 FK 到 realm_lifecycle_run
//! 字段对应 DDL：id / run_id / target_region / target_player_count / target_tps / status / created_at

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::Result;

/// new_realm_plan 状态机：draft → validated → executing → done / failed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NewRealmPlanStatus {
    Draft,
    Validated,
    Executing,
    Done,
    Failed,
}

impl NewRealmPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NewRealmPlanStatus::Draft => "draft",
            NewRealmPlanStatus::Validated => "validated",
            NewRealmPlanStatus::Executing => "executing",
            NewRealmPlanStatus::Done => "done",
            NewRealmPlanStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "draft" => NewRealmPlanStatus::Draft,
            "validated" => NewRealmPlanStatus::Validated,
            "executing" => NewRealmPlanStatus::Executing,
            "done" => NewRealmPlanStatus::Done,
            "failed" => NewRealmPlanStatus::Failed,
            _ => NewRealmPlanStatus::Draft,
        }
    }
}

/// NewRealmPlan entity（per RGS-SPEC-DTL-042 §2 表 2/6）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewRealmPlan {
    pub id: Uuid,
    pub run_id: Uuid,
    pub target_region: String,
    pub target_player_count: i32,
    pub target_tps: i32,
    pub status: NewRealmPlanStatus,
    pub created_at: DateTime<Utc>,
}

impl NewRealmPlan {
    /// 工厂：新建 draft 状态 plan
    pub fn new(
        run_id: Uuid,
        target_region: String,
        target_player_count: i32,
        target_tps: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            run_id,
            target_region,
            target_player_count,
            target_tps,
            status: NewRealmPlanStatus::Draft,
            created_at: Utc::now(),
        }
    }
}

/// PgRepository 骨架
pub struct PgNewRealmPlanRepository {
    pool: PgPool,
}

impl PgNewRealmPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_new_realm_plan(row: sqlx::postgres::PgRow) -> NewRealmPlan {
    let status_str: String = row.get("status");
    NewRealmPlan {
        id: row.get("id"),
        run_id: row.get("run_id"),
        target_region: row.get("target_region"),
        target_player_count: row.get("target_player_count"),
        target_tps: row.get("target_tps"),
        status: NewRealmPlanStatus::parse(&status_str),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl super::NewRealmPlanRepository for PgNewRealmPlanRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewRealmPlan>> {
        let row = sqlx::query(
            "SELECT id, run_id, target_region, target_player_count, target_tps, status, created_at \
             FROM new_realm_plan WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_new_realm_plan))
    }

    async fn find_by_run_id(&self, run_id: Uuid) -> Result<Option<NewRealmPlan>> {
        let row = sqlx::query(
            "SELECT id, run_id, target_region, target_player_count, target_tps, status, created_at \
             FROM new_realm_plan WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_new_realm_plan))
    }

    async fn save(&self, entity: &NewRealmPlan) -> Result<NewRealmPlan> {
        sqlx::query(
            "INSERT INTO new_realm_plan \
             (id, run_id, target_region, target_player_count, target_tps, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (id) DO UPDATE SET \
                target_region = EXCLUDED.target_region, \
                target_player_count = EXCLUDED.target_player_count, \
                target_tps = EXCLUDED.target_tps, \
                status = EXCLUDED.status",
        )
        .bind(entity.id)
        .bind(entity.run_id)
        .bind(&entity.target_region)
        .bind(entity.target_player_count)
        .bind(entity.target_tps)
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
    fn new_realm_plan_status_roundtrip() {
        for s in [
            NewRealmPlanStatus::Draft,
            NewRealmPlanStatus::Validated,
            NewRealmPlanStatus::Executing,
            NewRealmPlanStatus::Done,
            NewRealmPlanStatus::Failed,
        ] {
            assert_eq!(NewRealmPlanStatus::parse(s.as_str()), s);
        }
    }

    #[test]
    fn new_realm_plan_factory() {
        let run = Uuid::new_v4();
        let p = NewRealmPlan::new(run, "ap-northeast-1".to_string(), 10000, 5000);
        assert_eq!(p.run_id, run);
        assert_eq!(p.target_region, "ap-northeast-1");
        assert_eq!(p.target_player_count, 10000);
        assert_eq!(p.target_tps, 5000);
        assert_eq!(p.status, NewRealmPlanStatus::Draft);
    }
}
