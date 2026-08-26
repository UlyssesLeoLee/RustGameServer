//! realm_lifecycle_run entity（per M-2068.7）
//!
//! 主运行记录；按 created_at 月度范围分区（per RGS-BAS-007 §4 + RGS-SPEC-DTL-042 §3）
//! 7 个 feature_subtype 子类走 ClusterOpsService PFAU 编排（per RGS-SPEC-DTL-042 §3）
//! 唯一约束 (request_id, operator_id) 保证幂等性

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::Result;

/// feature_subtype：7 个 LCM 子类（per RGS-SPEC-DTL-042 §3 第 2 条）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureSubtype {
    /// 新建 realm
    NewRealm,
    /// 扩缩容
    Scale,
    /// 分服
    Split,
    /// 合服
    Merge,
    /// 合服回退
    MergeRollback,
    /// 退场
    Retire,
    /// 归档
    Archive,
}

impl FeatureSubtype {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatureSubtype::NewRealm => "new_realm",
            FeatureSubtype::Scale => "scale",
            FeatureSubtype::Split => "split",
            FeatureSubtype::Merge => "merge",
            FeatureSubtype::MergeRollback => "merge_rollback",
            FeatureSubtype::Retire => "retire",
            FeatureSubtype::Archive => "archive",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "new_realm" => FeatureSubtype::NewRealm,
            "scale" => FeatureSubtype::Scale,
            "split" => FeatureSubtype::Split,
            "merge" => FeatureSubtype::Merge,
            "merge_rollback" => FeatureSubtype::MergeRollback,
            "retire" => FeatureSubtype::Retire,
            "archive" => FeatureSubtype::Archive,
            _ => FeatureSubtype::NewRealm, // 兜底（应用层应避免进入此分支）
        }
    }
}

/// run 状态机：pending → in_progress → completed / failed / rolled_back
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunStatus::Pending => "pending",
            RunStatus::InProgress => "in_progress",
            RunStatus::Completed => "completed",
            RunStatus::Failed => "failed",
            RunStatus::RolledBack => "rolled_back",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "pending" => RunStatus::Pending,
            "in_progress" => RunStatus::InProgress,
            "completed" => RunStatus::Completed,
            "failed" => RunStatus::Failed,
            "rolled_back" => RunStatus::RolledBack,
            _ => RunStatus::Pending,
        }
    }
}

/// RealmLifecycleRun entity（per RGS-SPEC-DTL-042 §2 表 1/6）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealmLifecycleRun {
    pub id: Uuid,
    pub feature_subtype: FeatureSubtype,
    pub realm_id: Uuid,
    pub operator_id: Uuid,
    pub request_id: Uuid,
    pub approval_ref: Option<String>,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub trace_id: Option<String>,
}

impl RealmLifecycleRun {
    /// 工厂：新建 pending 状态的 run
    pub fn new(
        feature_subtype: FeatureSubtype,
        realm_id: Uuid,
        operator_id: Uuid,
        request_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            feature_subtype,
            realm_id,
            operator_id,
            request_id,
            approval_ref: None,
            status: RunStatus::Pending,
            started_at: now,
            completed_at: None,
            created_at: now,
            trace_id: None,
        }
    }

    /// 标记高危操作已审批（FR-LCM-002 approval_ref 必填）
    pub fn attach_approval(&mut self, approval_ref: String) {
        self.approval_ref = Some(approval_ref);
    }

    /// 进入 in_progress
    pub fn start(&mut self) {
        self.status = RunStatus::InProgress;
        self.started_at = Utc::now();
    }

    /// 完成
    pub fn complete(&mut self) {
        self.status = RunStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// 失败
    pub fn fail(&mut self) {
        self.status = RunStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    /// 回退（rollback 阶段）
    pub fn rollback(&mut self) {
        self.status = RunStatus::RolledBack;
        self.completed_at = Some(Utc::now());
    }
}

/// PgRepository：sqlx 骨架（per M-2068.7 最小可用）
pub struct PgRealmLifecycleRunRepository {
    pool: PgPool,
}

impl PgRealmLifecycleRunRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_run(row: sqlx::postgres::PgRow) -> RealmLifecycleRun {
    let subtype_str: String = row.get("feature_subtype");
    let status_str: String = row.get("status");
    RealmLifecycleRun {
        id: row.get("id"),
        feature_subtype: FeatureSubtype::parse(&subtype_str),
        realm_id: row.get("realm_id"),
        operator_id: row.get("operator_id"),
        request_id: row.get("request_id"),
        approval_ref: row.get("approval_ref"),
        status: RunStatus::parse(&status_str),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        trace_id: row.get("trace_id"),
    }
}

#[async_trait]
impl super::RealmLifecycleRunRepository for PgRealmLifecycleRunRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RealmLifecycleRun>> {
        let row = sqlx::query(
            "SELECT id, feature_subtype, realm_id, operator_id, request_id, approval_ref, \
             status, started_at, completed_at, created_at, trace_id \
             FROM realm_lifecycle_run WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_run))
    }

    async fn find_by_request_operator(
        &self,
        request_id: Uuid,
        operator_id: Uuid,
    ) -> Result<Option<RealmLifecycleRun>> {
        let row = sqlx::query(
            "SELECT id, feature_subtype, realm_id, operator_id, request_id, approval_ref, \
             status, started_at, completed_at, created_at, trace_id \
             FROM realm_lifecycle_run WHERE request_id = $1 AND operator_id = $2",
        )
        .bind(request_id)
        .bind(operator_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_run))
    }

    async fn save(&self, entity: &RealmLifecycleRun) -> Result<RealmLifecycleRun> {
        sqlx::query(
            "INSERT INTO realm_lifecycle_run \
             (id, feature_subtype, realm_id, operator_id, request_id, approval_ref, \
              status, started_at, completed_at, created_at, trace_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (id) DO UPDATE SET \
                feature_subtype = EXCLUDED.feature_subtype, \
                realm_id = EXCLUDED.realm_id, \
                status = EXCLUDED.status, \
                approval_ref = EXCLUDED.approval_ref, \
                started_at = EXCLUDED.started_at, \
                completed_at = EXCLUDED.completed_at, \
                trace_id = EXCLUDED.trace_id",
        )
        .bind(entity.id)
        .bind(entity.feature_subtype.as_str())
        .bind(entity.realm_id)
        .bind(entity.operator_id)
        .bind(entity.request_id)
        .bind(&entity.approval_ref)
        .bind(entity.status.as_str())
        .bind(entity.started_at)
        .bind(entity.completed_at)
        .bind(entity.created_at)
        .bind(&entity.trace_id)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_subtype_roundtrip() {
        for s in [
            FeatureSubtype::NewRealm,
            FeatureSubtype::Scale,
            FeatureSubtype::Split,
            FeatureSubtype::Merge,
            FeatureSubtype::MergeRollback,
            FeatureSubtype::Retire,
            FeatureSubtype::Archive,
        ] {
            assert_eq!(FeatureSubtype::parse(s.as_str()), s);
        }
    }

    #[test]
    fn run_status_roundtrip() {
        for s in [
            RunStatus::Pending,
            RunStatus::InProgress,
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::RolledBack,
        ] {
            assert_eq!(RunStatus::parse(s.as_str()), s);
        }
    }

    #[test]
    fn run_state_machine() {
        let mut r = RealmLifecycleRun::new(
            FeatureSubtype::NewRealm,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        assert_eq!(r.status, RunStatus::Pending);
        r.start();
        assert_eq!(r.status, RunStatus::InProgress);
        r.complete();
        assert_eq!(r.status, RunStatus::Completed);
        assert!(r.completed_at.is_some());
    }

    #[test]
    fn run_attach_approval() {
        let mut r = RealmLifecycleRun::new(
            FeatureSubtype::Retire,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        assert!(r.approval_ref.is_none());
        r.attach_approval("TKT-12345".to_string());
        assert_eq!(r.approval_ref.as_deref(), Some("TKT-12345"));
    }
}
