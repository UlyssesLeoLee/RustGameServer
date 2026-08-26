//! 5 类演练剧本模板（per IMPL §3.4 M-2070.2 + SPEC-DTL-042 §3）。
//!
//! ## 5 类剧本
//!
//! 1. **NewRealmPlaybook** —— 开新服（AC-LCM-001）
//! 2. **SplitPlaybook** —— 分服（AC-LCM-003）
//! 3. **MergePlaybook** —— 合服（AC-LCM-004）
//! 4. **RetirePlaybook** —— 退场（AC-LCM-006）
//! 5. **ArchivePlaybook** —— 归档（AC-LCM-007）
//!
//! 注：扩缩容（AC-LCM-002）使用 NewRealmPlaybook 的同构子模板（共享灰度 + PFAU 编排）。
//!
//! ## 硬约束
//!
//! - 5 类剧本各通过 1 次 = 演练最小集（per IMPL §7.3）
//! - 全部跑沙箱（per FR-LCM-003）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::{
    saga::steps::{SagaPhase, SagaStep, SagaStepKind},
    RealmId,
};

/// 剧本类型枚举（per IMPL §3.4 M-2070.2 + DTL §11.1 指标标签）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybookKind {
    NewRealm,
    Split,
    Merge,
    Retire,
    Archive,
}

impl PlaybookKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NewRealm => "new_realm",
            Self::Split => "split",
            Self::Merge => "merge",
            Self::Retire => "retire",
            Self::Archive => "archive",
        }
    }
}

/// 剧本 trait：5 类实现各自构造 Saga 步骤序列。
pub trait Playbook: Send + Sync {
    fn kind(&self) -> PlaybookKind;
    fn realm_id(&self) -> &RealmId;
    fn saga_steps(&self) -> Vec<SagaStep>;
    /// 演练超时（per SPEC §5 背压：步骤默认 60s，总时长 = N * 60s + 余量）。
    fn drill_timeout_secs(&self) -> u32;
}

// =====================================================================
// NewRealmPlaybook（AC-LCM-001）
// =====================================================================

/// 开新服剧本（per AC-LCM-001 + DTL-042 §5.1）。
///
/// 3 步 Saga：
///   1. InitDirectory（初始化 realm_directory 条目，灰度 0%）
///   2. WriteRunRecord（admin_db.realm_lifecycle_run 写 run）
///   3. PfauActivate（PFAU 编排到 Active）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRealmPlaybook {
    pub realm_id: RealmId,
    pub region: String,
    pub initial_node_count: u32,
}

impl Playbook for NewRealmPlaybook {
    fn kind(&self) -> PlaybookKind {
        PlaybookKind::NewRealm
    }
    fn realm_id(&self) -> &RealmId {
        &self.realm_id
    }
    fn saga_steps(&self) -> Vec<SagaStep> {
        vec![
            SagaStep::new(SagaPhase::NewRealm, SagaStepKind::InitDirectory),
            SagaStep::new(SagaPhase::NewRealm, SagaStepKind::WriteRunRecord),
            SagaStep::new(SagaPhase::NewRealm, SagaStepKind::PfauActivate),
        ]
    }
    fn drill_timeout_secs(&self) -> u32 {
        // 3 步 * 60s + 60s 余量 = 240s
        3 * SagaStep::DEFAULT_TIMEOUT_SECS + 60
    }
}

// =====================================================================
// SplitPlaybook（AC-LCM-003）
// =====================================================================

/// 分服剧本（per AC-LCM-003 + DTL-042 §5.3）。
///
/// 7 步 Saga：FreezeSource → SnapshotPlayers → CreateTargetRealm →
///              MigrateData → ShiftTraffic → PromoteDirectory → ThawSource。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPlaybook {
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub split_point_player_id: String,
    pub estimated_players: u64,
}

impl Playbook for SplitPlaybook {
    fn kind(&self) -> PlaybookKind {
        PlaybookKind::Split
    }
    fn realm_id(&self) -> &RealmId {
        &self.source_realm_id
    }
    fn saga_steps(&self) -> Vec<SagaStep> {
        vec![
            SagaStep::new(SagaPhase::Split, SagaStepKind::FreezeSource),
            SagaStep::new(SagaPhase::Split, SagaStepKind::SnapshotPlayers),
            SagaStep::new(SagaPhase::Split, SagaStepKind::CreateTargetRealm),
            SagaStep::new(SagaPhase::Split, SagaStepKind::MigrateData),
            SagaStep::new(SagaPhase::Split, SagaStepKind::ShiftTraffic),
            SagaStep::new(SagaPhase::Split, SagaStepKind::PromoteDirectory),
            SagaStep::new(SagaPhase::Split, SagaStepKind::ThawSource),
        ]
    }
    fn drill_timeout_secs(&self) -> u32 {
        7 * SagaStep::DEFAULT_TIMEOUT_SECS + 60
    }
}

// =====================================================================
// MergePlaybook（AC-LCM-004）
// =====================================================================

/// 合服剧本（per AC-LCM-004 + DTL-042 §5.4 + FR-LCM-062）。
///
/// 4 步 Saga：LoadConflictRulesV2 → MergePlayerData → LockConflictRulesV2 → MergeCompleted。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePlaybook {
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub conflict_rule_set_version: u32,
    pub rollback_window_days: u32,
}

impl Playbook for MergePlaybook {
    fn kind(&self) -> PlaybookKind {
        PlaybookKind::Merge
    }
    fn realm_id(&self) -> &RealmId {
        &self.source_realm_id
    }
    fn saga_steps(&self) -> Vec<SagaStep> {
        vec![
            SagaStep::new(SagaPhase::Merge, SagaStepKind::LoadConflictRulesV2),
            SagaStep::new(SagaPhase::Merge, SagaStepKind::MergePlayerData),
            SagaStep::new(SagaPhase::Merge, SagaStepKind::LockConflictRulesV2),
            SagaStep::new(SagaPhase::Merge, SagaStepKind::MergeCompleted),
        ]
    }
    fn drill_timeout_secs(&self) -> u32 {
        4 * SagaStep::DEFAULT_TIMEOUT_SECS + 60
    }
}

// =====================================================================
// RetirePlaybook（AC-LCM-006）
// =====================================================================

/// 退场剧本（per AC-LCM-006 + DTL-042 §5.5 + SPEC §3 第 8 条）。
///
/// 2 步 Saga：CreateRetirePlan → ScheduleArchive。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirePlaybook {
    pub realm_id: RealmId,
    pub query_channel_rbac: Vec<String>,
    pub archive_threshold_days: u32,
}

impl Playbook for RetirePlaybook {
    fn kind(&self) -> PlaybookKind {
        PlaybookKind::Retire
    }
    fn realm_id(&self) -> &RealmId {
        &self.realm_id
    }
    fn saga_steps(&self) -> Vec<SagaStep> {
        vec![
            SagaStep::new(SagaPhase::Retire, SagaStepKind::CreateRetirePlan),
            SagaStep::new(SagaPhase::Retire, SagaStepKind::ScheduleArchive),
        ]
    }
    fn drill_timeout_secs(&self) -> u32 {
        2 * SagaStep::DEFAULT_TIMEOUT_SECS + 60
    }
}

// =====================================================================
// ArchivePlaybook（AC-LCM-007）
// =====================================================================

/// 归档剧本（per AC-LCM-007 + DTL-042 §5.6 + FR-LCM-081 + RSK-LCM-005）。
///
/// 3 步 Saga：ClassifyHotCold → MigrateToStorage → ReplicateForNPlus2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePlaybook {
    pub realm_id: RealmId,
    pub last_active_at: DateTime<Utc>,
}

impl Playbook for ArchivePlaybook {
    fn kind(&self) -> PlaybookKind {
        PlaybookKind::Archive
    }
    fn realm_id(&self) -> &RealmId {
        &self.realm_id
    }
    fn saga_steps(&self) -> Vec<SagaStep> {
        vec![
            SagaStep::new(SagaPhase::Archive, SagaStepKind::ClassifyHotCold),
            SagaStep::new(SagaPhase::Archive, SagaStepKind::MigrateToStorage),
            SagaStep::new(SagaPhase::Archive, SagaStepKind::ReplicateForNPlus2),
        ]
    }
    fn drill_timeout_secs(&self) -> u32 {
        3 * SagaStep::DEFAULT_TIMEOUT_SECS + 60
    }
}

/// 构造所有 5 类剧本（用于"演练剧本：5 类各通过 1 次"，per IMPL §7.3）。
pub fn all_playbooks() -> Vec<Box<dyn Playbook>> {
    vec![
        Box::new(NewRealmPlaybook {
            realm_id: "rlm-drill-new".to_string(),
            region: "ap-east-1".to_string(),
            initial_node_count: 3,
        }),
        Box::new(SplitPlaybook {
            source_realm_id: "rlm-drill-split-src".to_string(),
            target_realm_id: "rlm-drill-split-tgt".to_string(),
            split_point_player_id: "p-1000000".to_string(),
            estimated_players: 2_000_000,
        }),
        Box::new(MergePlaybook {
            source_realm_id: "rlm-drill-merge-src".to_string(),
            target_realm_id: "rlm-drill-merge-tgt".to_string(),
            conflict_rule_set_version: 2,
            rollback_window_days: 14,
        }),
        Box::new(RetirePlaybook {
            realm_id: "rlm-drill-retire".to_string(),
            query_channel_rbac: vec!["cs_agent".to_string(), "sre".to_string(), "legal".to_string()],
            archive_threshold_days: 60,
        }),
        Box::new(ArchivePlaybook {
            realm_id: "rlm-drill-archive".to_string(),
            last_active_at: Utc::now() - chrono::Duration::days(365 * 2),
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_lifecycle::saga::steps::SagaStepKind;

    #[test]
    fn playbook_kind_as_str_cover_all() {
        for k in [
            PlaybookKind::NewRealm,
            PlaybookKind::Split,
            PlaybookKind::Merge,
            PlaybookKind::Retire,
            PlaybookKind::Archive,
        ] {
            assert!(!k.as_str().is_empty());
        }
    }

    #[test]
    fn new_realm_playbook_has_3_steps() {
        let p = NewRealmPlaybook {
            realm_id: "r".to_string(),
            region: "x".to_string(),
            initial_node_count: 1,
        };
        let steps = p.saga_steps();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].kind, SagaStepKind::InitDirectory);
        assert_eq!(steps[2].kind, SagaStepKind::PfauActivate);
    }

    #[test]
    fn split_playbook_has_7_steps() {
        let p = SplitPlaybook {
            source_realm_id: "s".to_string(),
            target_realm_id: "t".to_string(),
            split_point_player_id: "p".to_string(),
            estimated_players: 1,
        };
        let steps = p.saga_steps();
        assert_eq!(steps.len(), 7);
        assert_eq!(steps[0].kind, SagaStepKind::FreezeSource);
        assert_eq!(steps[6].kind, SagaStepKind::ThawSource);
    }

    #[test]
    fn merge_playbook_has_4_steps() {
        let p = MergePlaybook {
            source_realm_id: "s".to_string(),
            target_realm_id: "t".to_string(),
            conflict_rule_set_version: 2,
            rollback_window_days: 14,
        };
        let steps = p.saga_steps();
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[2].kind, SagaStepKind::LockConflictRulesV2);
    }

    #[test]
    fn retire_playbook_has_2_steps() {
        let p = RetirePlaybook {
            realm_id: "r".to_string(),
            query_channel_rbac: vec![],
            archive_threshold_days: 30,
        };
        let steps = p.saga_steps();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn archive_playbook_has_3_steps() {
        let p = ArchivePlaybook {
            realm_id: "r".to_string(),
            last_active_at: Utc::now(),
        };
        let steps = p.saga_steps();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[2].kind, SagaStepKind::ReplicateForNPlus2);
    }

    #[test]
    fn all_playbooks_returns_5_kinds() {
        // IMPL §7.3 锚定：5 类剧本模板各通过 1 次
        let all = all_playbooks();
        assert_eq!(all.len(), 5);
        let kinds: std::collections::HashSet<_> =
            all.iter().map(|p| p.kind()).collect();
        assert_eq!(kinds.len(), 5);
    }
}
