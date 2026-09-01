//! LCM step execution schema 草案 (per BAS-001 v0.2 §6.6.2 拍板)
//!
//! 归类: **Work** 表 (per §6.6.2 admin Lead 拍板 opt1, 2026-09-01 22:25 JST)
//! 保留期: 24h cleanup (per brief B8 "决策 保留期 24h vs 7d vs 30d" → 24h)
//! 范围: admin_db, 物理位置 `lcm_step_execution` (per RGS-ARC-008 5 独立 DB 原则)
//!
//! 关联:
//! - 决策记录: docs/00-基準与治理/lcm/RGS-LCM-STEP-EXECUTION-DECISION_v0.1.md
//! - DDL 落地: crates/admin-service/migrations/0005_lcm_step_execution.sql
//! - 横展开母规范: docs/00-基準与治理/RGS-DB-BAS-001_数据库表设计三分类横展开基本设计书_v0.2.md
//!
//! ## PH-2 待实装
//!
//! 1. `LcmStepExecutionRepository` trait (insert / list_by_run_id / cleanup_expired)
//! 2. `PgLcmStepExecutionRepository` sqlx impl
//! 3. `InMemoryLcmStepExecutionRepository` (test only)
//! 4. cleanup cron (per BAS-001 §6.3 14-§7 cleanup SOP)
//! 5. admin_backend gRPC integration (GetStepExecution / ListStepExecutions RPC)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LCM step 状态机 (5 态 per RGS-ARC-051 + LCM 业务约定)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LcmStepStatus {
    /// 等待执行 (phase 已开始但 step 未轮转到)
    Pending,
    /// 执行中 (step 已轮转, 正在调用外部系统)
    InProgress,
    /// 成功完成
    Succeeded,
    /// 失败 (attempt_count > max_attempts 后由上层决定 retry / 告警 / 暂停)
    Failed,
    /// 跳过 (上游 phase 失败导致 step 不再执行, 显式标记而非 NULL)
    Skipped,
}

impl LcmStepStatus {
    /// 返回状态机字面量 (用于 sqlx CHECK 约束 / log)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// LCM step execution 内存模型 (Work 表, 24h cleanup)
///
/// 字段对应 DDL `0005_lcm_step_execution.sql` 4 字段 (id / run_id / step_seq /
/// step_name / status / started_at / completed_at / attempt_count / last_error /
/// step_metadata / expires_at / created_at) + UNIQUE(run_id, step_seq).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LcmStepExecution {
    pub id: Uuid,
    /// 关联 LCM run (FK → realm_lifecycle_run.id, ON DELETE CASCADE)
    pub run_id: Uuid,
    /// 步骤序号 (在 phase 内, 1-based)
    pub step_seq: i32,
    /// 步骤名 (e.g. "provision" / "configure" / "smoke_test" / "route53_update")
    pub step_name: String,
    pub status: LcmStepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    /// 重试次数 (per step, 区别于 run-level retry)
    pub attempt_count: i32,
    pub last_error: Option<String>,
    /// step 私有元数据 (JSONB; 跨 step 状态共享通过此字段, per brief B8 "跨 step 状态共享用 step_metadata JSONB")
    pub step_metadata: Option<serde_json::Value>,
    /// 过期时间 (cleanup cron 在此时间后删除; 默认 = created_at + 24h)
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl LcmStepExecution {
    /// 工厂: 新建 pending step (phase 启动时调用)
    pub fn new_pending(
        run_id: Uuid,
        step_seq: i32,
        step_name: impl Into<String>,
        ttl_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            run_id,
            step_seq,
            step_name: step_name.into(),
            status: LcmStepStatus::Pending,
            started_at: None,
            completed_at: None,
            attempt_count: 0,
            last_error: None,
            step_metadata: None,
            expires_at: now + chrono::Duration::seconds(ttl_seconds),
            created_at: now,
        }
    }

    /// 标记 step 进入 in_progress 状态
    pub fn mark_in_progress(&mut self) {
        self.status = LcmStepStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.attempt_count += 1;
    }

    /// 标记 step 成功
    pub fn mark_succeeded(&mut self) {
        self.status = LcmStepStatus::Succeeded;
        self.completed_at = Some(Utc::now());
    }

    /// 标记 step 失败 (带 error)
    pub fn mark_failed(&mut self, err: impl Into<String>) {
        self.status = LcmStepStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.last_error = Some(err.into());
    }

    /// 标记 step 跳过 (上游 phase 失败, 不再执行)
    pub fn mark_skipped(&mut self, reason: impl Into<String>) {
        self.status = LcmStepStatus::Skipped;
        self.completed_at = Some(Utc::now());
        self.last_error = Some(reason.into());
    }

    /// 是否终态 (per 状态机 5 态, succeeded / failed / skipped 为终态)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            LcmStepStatus::Succeeded | LcmStepStatus::Failed | LcmStepStatus::Skipped
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcm_step_status_as_str_round_trip() {
        for s in [
            LcmStepStatus::Pending,
            LcmStepStatus::InProgress,
            LcmStepStatus::Succeeded,
            LcmStepStatus::Failed,
            LcmStepStatus::Skipped,
        ] {
            let s_str = s.as_str();
            // 5 态字面量均与 DDL CHECK 约束一致
            assert!(matches!(
                s_str,
                "pending" | "in_progress" | "succeeded" | "failed" | "skipped"
            ));
        }
    }

    #[test]
    fn lcm_step_execution_factory_default_ttl_24h() {
        let run_id = Uuid::new_v4();
        let s = LcmStepExecution::new_pending(run_id, 1, "provision", 24 * 3600);
        assert_eq!(s.run_id, run_id);
        assert_eq!(s.step_seq, 1);
        assert_eq!(s.step_name, "provision");
        assert_eq!(s.status, LcmStepStatus::Pending);
        assert_eq!(s.attempt_count, 0);
        assert!(s.started_at.is_none());
        assert!(s.completed_at.is_none());
        // expires_at 距 created_at 应为 24h
        let diff = s.expires_at - s.created_at;
        assert_eq!(diff.num_seconds(), 24 * 3600);
    }

    #[test]
    fn lcm_step_execution_state_machine_lifecycle() {
        let mut s = LcmStepExecution::new_pending(Uuid::new_v4(), 1, "configure", 3600);
        assert!(!s.is_terminal());

        s.mark_in_progress();
        assert_eq!(s.status, LcmStepStatus::InProgress);
        assert!(s.started_at.is_some());
        assert_eq!(s.attempt_count, 1);

        s.mark_succeeded();
        assert!(s.is_terminal());
        assert!(s.completed_at.is_some());
    }

    #[test]
    fn lcm_step_execution_failure_records_error() {
        let mut s = LcmStepExecution::new_pending(Uuid::new_v4(), 2, "smoke_test", 3600);
        s.mark_in_progress();
        s.mark_failed("connection refused");
        assert_eq!(s.status, LcmStepStatus::Failed);
        assert_eq!(s.last_error.as_deref(), Some("connection refused"));
        assert!(s.is_terminal());
    }

    #[test]
    fn lcm_step_execution_skipped_for_upstream_failure() {
        let mut s = LcmStepExecution::new_pending(Uuid::new_v4(), 3, "route53_update", 3600);
        s.mark_skipped("upstream phase failed");
        assert_eq!(s.status, LcmStepStatus::Skipped);
        assert!(s.is_terminal());
    }

    #[test]
    fn lcm_step_execution_attempt_count_increments_on_retry() {
        let mut s = LcmStepExecution::new_pending(Uuid::new_v4(), 1, "provision", 3600);
        s.mark_in_progress();
        assert_eq!(s.attempt_count, 1);
        s.mark_failed("timeout");
        s.mark_in_progress();
        assert_eq!(s.attempt_count, 2, "retry 时 attempt_count 应累加");
    }
}
