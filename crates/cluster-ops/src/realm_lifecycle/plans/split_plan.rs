//! split_plan entity（per M-2068.7）
//!
//! 分服计划；target_realm_count >= 2（DDL CHECK 约束）
//! split_strategy：hash / range / manual

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::Result;

/// split_plan 状态机
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SplitPlanStatus {
    Draft,
    Validated,
    Executing,
    Done,
    Failed,
}

impl SplitPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SplitPlanStatus::Draft => "draft",
            SplitPlanStatus::Validated => "validated",
            SplitPlanStatus::Executing => "executing",
            SplitPlanStatus::Done => "done",
            SplitPlanStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "draft" => SplitPlanStatus::Draft,
            "validated" => SplitPlanStatus::Validated,
            "executing" => SplitPlanStatus::Executing,
            "done" => SplitPlanStatus::Done,
            "failed" => SplitPlanStatus::Failed,
            _ => SplitPlanStatus::Draft,
        }
    }
}

/// 分服策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SplitStrategy {
    /// 按 hash 拆分（按 player_id 哈希）
    Hash,
    /// 按区间拆分
    Range,
    /// 手工指定映射
    Manual,
}

impl SplitStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SplitStrategy::Hash => "hash",
            SplitStrategy::Range => "range",
            SplitStrategy::Manual => "manual",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "hash" => SplitStrategy::Hash,
            "range" => SplitStrategy::Range,
            "manual" => SplitStrategy::Manual,
            _ => SplitStrategy::Hash,
        }
    }
}

/// SplitPlan entity（per RGS-SPEC-DTL-042 §2 表 3/6）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SplitPlan {
    pub id: Uuid,
    pub run_id: Uuid,
    pub source_realm_id: Uuid,
    pub target_realm_count: i32,
    pub split_strategy: SplitStrategy,
    pub status: SplitPlanStatus,
    pub created_at: DateTime<Utc>,
}

impl SplitPlan {
    /// 工厂：新建 draft 状态 plan；target_realm_count >= 2 由 DDL CHECK 约束
    pub fn new(
        run_id: Uuid,
        source_realm_id: Uuid,
        target_realm_count: i32,
        split_strategy: SplitStrategy,
    ) -> Self {
        assert!(target_realm_count >= 2, "target_realm_count 必须 >= 2（DDL CHECK 约束）");
        Self {
            id: Uuid::new_v4(),
            run_id,
            source_realm_id,
            target_realm_count,
            split_strategy,
            status: SplitPlanStatus::Draft,
            created_at: Utc::now(),
        }
    }
}

/// PgRepository 骨架
pub struct PgSplitPlanRepository {
    pool: PgPool,
}

impl PgSplitPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_split_plan(row: sqlx::postgres::PgRow) -> SplitPlan {
    let status_str: String = row.get("status");
    let strategy_str: String = row.get("split_strategy");
    SplitPlan {
        id: row.get("id"),
        run_id: row.get("run_id"),
        source_realm_id: row.get("source_realm_id"),
        target_realm_count: row.get("target_realm_count"),
        split_strategy: SplitStrategy::parse(&strategy_str),
        status: SplitPlanStatus::parse(&status_str),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl super::SplitPlanRepository for PgSplitPlanRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SplitPlan>> {
        let row = sqlx::query(
            "SELECT id, run_id, source_realm_id, target_realm_count, split_strategy, status, created_at \
             FROM split_plan WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_split_plan))
    }

    async fn find_by_run_id(&self, run_id: Uuid) -> Result<Option<SplitPlan>> {
        let row = sqlx::query(
            "SELECT id, run_id, source_realm_id, target_realm_count, split_strategy, status, created_at \
             FROM split_plan WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_split_plan))
    }

    async fn save(&self, entity: &SplitPlan) -> Result<SplitPlan> {
        sqlx::query(
            "INSERT INTO split_plan \
             (id, run_id, source_realm_id, target_realm_count, split_strategy, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (id) DO UPDATE SET \
                source_realm_id = EXCLUDED.source_realm_id, \
                target_realm_count = EXCLUDED.target_realm_count, \
                split_strategy = EXCLUDED.split_strategy, \
                status = EXCLUDED.status",
        )
        .bind(entity.id)
        .bind(entity.run_id)
        .bind(entity.source_realm_id)
        .bind(entity.target_realm_count)
        .bind(entity.split_strategy.as_str())
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
    fn split_plan_status_roundtrip() {
        for s in [
            SplitPlanStatus::Draft,
            SplitPlanStatus::Validated,
            SplitPlanStatus::Executing,
            SplitPlanStatus::Done,
            SplitPlanStatus::Failed,
        ] {
            assert_eq!(SplitPlanStatus::parse(s.as_str()), s);
        }
    }

    #[test]
    fn split_strategy_roundtrip() {
        for s in [SplitStrategy::Hash, SplitStrategy::Range, SplitStrategy::Manual] {
            assert_eq!(SplitStrategy::parse(s.as_str()), s);
        }
    }

    #[test]
    #[should_panic(expected = "target_realm_count 必须 >= 2")]
    fn split_plan_factory_rejects_count_lt_2() {
        let _ = SplitPlan::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            SplitStrategy::Hash,
        );
    }

    #[test]
    fn split_plan_factory_accepts_count_2() {
        let p = SplitPlan::new(Uuid::new_v4(), Uuid::new_v4(), 2, SplitStrategy::Hash);
        assert_eq!(p.target_realm_count, 2);
    }
}
