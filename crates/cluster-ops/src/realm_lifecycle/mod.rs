//! rgs-realm-lifecycle —— 服务器全生命周期管理（LCM）子模块
//!
//! **定位**（per RGS-DTL-042 §1 + RGS-SPEC-DTL-042 §2）：
//! - AD 限界上下文扩展（**不**新建独立 crate / DB / 限界上下文）
//! - 与 `ClusterOpsService` 同处 `rgs-cluster-ops` crate 内
//! - 扩 ARC-051 `realm_lifecycle` Feature 类型（7 个子类）
//! - 6 阶段操作器（new_realm / scale / split / merge / merge_rollback / retire / archive）
//! - SagaOrchestrator（分服 6 步 / 合服 5 步 / 退场 4 步 / **归档 3 步**）
//!
//! **本文件落位（per WBS L4 #2074 / RGS-IMPL-PLAN-LCM-001 v0.1 §3.7）**：
//! - M-2074.1 冷热分层阈值（3 年热 + 10 年冷，TBD-DTL-042-01）
//! - M-2074.2 N+2 存储冗余（per RSK-LCM-005 缓解）
//! - M-2074.3 GDPR "被遗忘权" 删除通路（双层审计 per NFR-SE-010）
//! - M-2074.4 `admin_db.operation_audit` 双层审计留痕
//! - M-2074.5 归档查询延迟指标 `rgs_lcm_archive_query_latency_seconds`
//!
//! **硬约束（per SPEC §3 + §4.3 关键标注）**：
//! - 入口统一经由 `AdminService` 转发（FR-LCM-004 门禁）
//! - 归档**不**删除数据，**仅**迁移存储位置（FR-LCM-081）
//! - GDPR 删除通路走 `admin_db.operation_audit` 双层审计（NFR-SE-010 合规例外）
//! - 资金/合规相关决策 → A 角色 Ulysses 显式独立签字（per §4.3 关键标注）
//!
//! **非职责**（由既有模块负责）：
//! - RBAC / 审计 / 限流（既有 `AdminService`）
//! - PFAU 状态机推进（既有 `ClusterOpsService`）
//! - 业务 DB 改写（既有 player / economy / social service）
//!
//! ## 目录
//!
//! - [`error`] —— `LcmError` 统一错误类型 + 审计 + 预算分类
//! - [`state`] —— 7 阶段状态机（`RealmLifecycleState`）
//! - [`metrics`] —— 10 项 `rgs_lcm_*` Prometheus 指标注册
//! - [`plans::archive_policy`] —— 归档策略实体（3 年热 + 10 年冷 + N+2）
//! - [`operations::archive`] —— `ArchiveOperator`（热归档 → 冷归档 → GDPR 删除通路）

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod drill;
pub mod error;
pub mod metrics;
pub mod operations;
pub mod plans;
pub mod saga;
pub mod state;

pub use error::{LcmError, LcmResult};
pub use operations::archive::{ArchiveOutcome, ArchiveOperator, GdprDeleteRequest, GdprDeleteResult};
pub use plans::archive_policy::{
    ArchivePolicy, ArchiveTier, DEFAULT_COLD_RETENTION_YEARS, DEFAULT_HOT_RETENTION_YEARS,
    STORAGE_REDUNDANCY_N_PLUS_2,
};
pub use state::{RealmLifecycleState, FEATURE_SUBTYPE_ARCHIVE};

/// 归档操作阶段（per RGS-DTL-042 §6.6 Saga 3 步）
///
/// 1. `HotArchiveStep` —— DB 切换为冷备实例（只读副本）
/// 2. `ColdArchiveStep` —— 全量导出至对象存储（N+2 副本）
/// 3. `EnableGdprDeletePathStep` —— 合规删除通路开启
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveSagaStep {
    /// 步骤 1：热归档（DB → 只读副本）
    HotArchive,
    /// 步骤 2：冷归档（只读副本 → 对象存储 N+2 副本）
    ColdArchive,
    /// 步骤 3：GDPR "被遗忘权" 删除通路开启
    EnableGdprDeletePath,
}

impl ArchiveSagaStep {
    /// 步骤编号（per DTL-042 §6.6 顺序）
    pub const fn step_index(self) -> u8 {
        match self {
            ArchiveSagaStep::HotArchive => 1,
            ArchiveSagaStep::ColdArchive => 2,
            ArchiveSagaStep::EnableGdprDeletePath => 3,
        }
    }

    /// 步骤名（per RGS-DTL-042 §6.6 命名）
    pub const fn step_name(self) -> &'static str {
        match self {
            ArchiveSagaStep::HotArchive => "HotArchiveStep",
            ArchiveSagaStep::ColdArchive => "ColdArchiveStep",
            ArchiveSagaStep::EnableGdprDeletePath => "EnableGdprDeletePathStep",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_saga_step_index_matches_dtl_042() {
        // DTL-042 §6.6: 1.HotArchiveStep, 2.ColdArchiveStep, 3.EnableGdprDeletePathStep
        assert_eq!(ArchiveSagaStep::HotArchive.step_index(), 1);
        assert_eq!(ArchiveSagaStep::ColdArchive.step_index(), 2);
        assert_eq!(ArchiveSagaStep::EnableGdprDeletePath.step_index(), 3);
    }

    #[test]
    fn archive_saga_step_name_matches_dtl_042() {
        // DTL-042 §6.6 命名严格匹配（运营 / SRE 排查 log 关键字）
        assert_eq!(ArchiveSagaStep::HotArchive.step_name(), "HotArchiveStep");
        assert_eq!(
            ArchiveSagaStep::ColdArchive.step_name(),
            "ColdArchiveStep"
        );
        assert_eq!(
            ArchiveSagaStep::EnableGdprDeletePath.step_name(),
            "EnableGdprDeletePathStep"
        );
    }

    #[test]
    fn feature_subtype_archive_matches_arc_051() {
        // ARC-051: realm_lifecycle::archive 子类
        assert_eq!(FEATURE_SUBTYPE_ARCHIVE, "realm_lifecycle::archive");
    }

    #[test]
    fn default_retention_thresholds_match_spec() {
        // SPEC-DTL-042 §8: 归档冷热分层阈值 = 3 年热 + 10 年冷
        assert_eq!(DEFAULT_HOT_RETENTION_YEARS, 3);
        assert_eq!(DEFAULT_COLD_RETENTION_YEARS, 10);
    }

    #[test]
    fn storage_redundancy_default_is_n_plus_2() {
        // RSK-LCM-005 缓解: N+2 存储冗余为默认
        assert_eq!(STORAGE_REDUNDANCY_N_PLUS_2, "n_plus_2");
    }
}
