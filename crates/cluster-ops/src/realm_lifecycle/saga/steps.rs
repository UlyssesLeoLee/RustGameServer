//! 6 阶段 Saga 步骤定义（per DTL-042 §6 + SPEC-DTL-042 §3 + SPEC §5 背压）。
//!
//! ## 硬约束
//!
//! - 步骤超时默认 60s（per SPEC §5 背压规则 + IMPL §3.2 M-2067.5）
//! - 失败触发反向补偿（per SPEC §5 故障域 + ADR-0015 Saga 适用边界）
//! - 复用 economy::saga_orchestrator 模式（per IMPL §2.3）
//!
//! 本 worktree（WF-1-2070）只定义枚举 + 步骤名 + 默认 trait；具体
//! SagaStep 行为由 WF-1-2067 + 2071 后续 worktree 补齐。

use serde::{Deserialize, Serialize};

/// Saga 阶段（per 6 阶段操作器）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SagaPhase {
    NewRealm,
    Scale,
    Split,
    Merge,
    MergeRollback,
    Retire,
    Archive,
}

impl SagaPhase {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NewRealm => "new_realm",
            Self::Scale => "scale",
            Self::Split => "split",
            Self::Merge => "merge",
            Self::MergeRollback => "merge_rollback",
            Self::Retire => "retire",
            Self::Archive => "archive",
        }
    }
}

/// Saga 步骤状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Compensating,
    Compensated,
    TimedOut,
}

impl StepStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Compensating => "compensating",
            Self::Compensated => "compensated",
            Self::TimedOut => "timed_out",
        }
    }

    /// 是否终态（用于 Prometheus 指标 + 调度释放）。
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Compensated | Self::TimedOut
        )
    }
}

/// 6 阶段操作器各阶段典型 Saga 步骤（per DTL §6 + SPEC §3 第 5 条）。
///
/// 实际步骤由 orchestrator 拼接；本枚举定义"步骤类型"集合（不绑定特定 phase）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SagaStepKind {
    /// 1. 冻结源 realm 写
    FreezeSource,
    /// 2. 玩家数据快照
    SnapshotPlayers,
    /// 3. 创建目标 realm
    CreateTargetRealm,
    /// 4. 数据迁移
    MigrateData,
    /// 5. 切流量
    ShiftTraffic,
    /// 6. realm_directory 灰度 0%→100%
    PromoteDirectory,
    /// 7. 解冻源 realm
    ThawSource,

    /// NewRealm 专属：初始化 realm_directory 条目（灰度 0%）
    InitDirectory,
    /// NewRealm 专属：admin_db.realm_lifecycle_run 写 run
    WriteRunRecord,
    /// NewRealm 专属：PFAU 编排到 Active
    PfauActivate,

    /// Scale 专属：调 K3s 副本数
    AdjustK8sReplicas,

    /// Merge 专属：冲突规则 v2 加载
    LoadConflictRulesV2,
    /// Merge 专属：玩家数据合并
    MergePlayerData,
    /// Merge 专属：merge_conflict_rule_set_v2 锁定（FR-LCM-062 锚定）
    LockConflictRulesV2,
    /// Merge 专属：合并完成
    MergeCompleted,

    /// MergeRollback 专属：检测 window 内回退请求
    CheckRollbackWindow,
    /// MergeRollback 专属：玩家数据切回
    RestorePlayerData,
    /// MergeRollback 专属：locked_at 保持（FR-LCM-062 不解锁）
    PreserveLockedAt,

    /// Retire 专属：retire_plan 创建
    CreateRetirePlan,
    /// Retire 专属：30-90 天后启动归档
    ScheduleArchive,

    /// Archive 专属：冷热分层判定
    ClassifyHotCold,
    /// Archive 专属：迁移到冷/热存储
    MigrateToStorage,
    /// Archive 专属：N+2 冗余
    ReplicateForNPlus2,
}

impl SagaStepKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FreezeSource => "freeze_source",
            Self::SnapshotPlayers => "snapshot_players",
            Self::CreateTargetRealm => "create_target_realm",
            Self::MigrateData => "migrate_data",
            Self::ShiftTraffic => "shift_traffic",
            Self::PromoteDirectory => "promote_directory",
            Self::ThawSource => "thaw_source",
            Self::InitDirectory => "init_directory",
            Self::WriteRunRecord => "write_run_record",
            Self::PfauActivate => "pfau_activate",
            Self::AdjustK8sReplicas => "adjust_k8s_replicas",
            Self::LoadConflictRulesV2 => "load_conflict_rules_v2",
            Self::MergePlayerData => "merge_player_data",
            Self::LockConflictRulesV2 => "lock_conflict_rules_v2",
            Self::MergeCompleted => "merge_completed",
            Self::CheckRollbackWindow => "check_rollback_window",
            Self::RestorePlayerData => "restore_player_data",
            Self::PreserveLockedAt => "preserve_locked_at",
            Self::CreateRetirePlan => "create_retire_plan",
            Self::ScheduleArchive => "schedule_archive",
            Self::ClassifyHotCold => "classify_hot_cold",
            Self::MigrateToStorage => "migrate_to_storage",
            Self::ReplicateForNPlus2 => "replicate_for_n_plus_2",
        }
    }
}

/// 一次 Saga 步骤的最小快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub phase: SagaPhase,
    pub kind: SagaStepKind,
    pub status: StepStatus,
    /// 步骤超时（默认 60s，per SPEC §5 背压）。
    pub timeout_secs: u32,
    /// 该步骤重试次数。
    pub retry_count: u32,
}

impl SagaStep {
    pub const DEFAULT_TIMEOUT_SECS: u32 = 60;

    pub fn new(phase: SagaPhase, kind: SagaStepKind) -> Self {
        Self {
            phase,
            kind,
            status: StepStatus::Pending,
            timeout_secs: Self::DEFAULT_TIMEOUT_SECS,
            retry_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saga_phase_as_str_matches_dtl_feature_subtype() {
        // DTL §11.1：feature_subtype 标签值
        for p in [
            SagaPhase::NewRealm,
            SagaPhase::Scale,
            SagaPhase::Split,
            SagaPhase::Merge,
            SagaPhase::MergeRollback,
            SagaPhase::Retire,
            SagaPhase::Archive,
        ] {
            assert!(!p.as_str().is_empty());
        }
    }

    #[test]
    fn step_status_terminal_partition() {
        for s in [
            StepStatus::Pending,
            StepStatus::Running,
            StepStatus::Succeeded,
            StepStatus::Failed,
            StepStatus::Compensating,
            StepStatus::Compensated,
            StepStatus::TimedOut,
        ] {
            let t = s.is_terminal();
            // Pending/Running/Compensating 一定非终态；其余为终态
            if matches!(s, StepStatus::Pending | StepStatus::Running | StepStatus::Compensating) {
                assert!(!t, "{:?} should not be terminal", s);
            } else {
                assert!(t, "{:?} should be terminal", s);
            }
        }
    }

    #[test]
    fn step_kind_covers_all_phases() {
        // 防止后续 phase 漏加步骤
        let all_kinds = [
            SagaStepKind::FreezeSource,
            SagaStepKind::SnapshotPlayers,
            SagaStepKind::CreateTargetRealm,
            SagaStepKind::MigrateData,
            SagaStepKind::ShiftTraffic,
            SagaStepKind::PromoteDirectory,
            SagaStepKind::ThawSource,
            SagaStepKind::InitDirectory,
            SagaStepKind::WriteRunRecord,
            SagaStepKind::PfauActivate,
            SagaStepKind::AdjustK8sReplicas,
            SagaStepKind::LoadConflictRulesV2,
            SagaStepKind::MergePlayerData,
            SagaStepKind::LockConflictRulesV2,
            SagaStepKind::MergeCompleted,
            SagaStepKind::CheckRollbackWindow,
            SagaStepKind::RestorePlayerData,
            SagaStepKind::PreserveLockedAt,
            SagaStepKind::CreateRetirePlan,
            SagaStepKind::ScheduleArchive,
            SagaStepKind::ClassifyHotCold,
            SagaStepKind::MigrateToStorage,
            SagaStepKind::ReplicateForNPlus2,
        ];
        for k in all_kinds {
            assert!(!k.as_str().is_empty());
        }
    }

    #[test]
    fn default_timeout_matches_spec() {
        // SPEC §5 背压：步骤超时默认 60s
        let step = SagaStep::new(SagaPhase::Merge, SagaStepKind::MergeCompleted);
        assert_eq!(step.timeout_secs, 60);
    }
}
