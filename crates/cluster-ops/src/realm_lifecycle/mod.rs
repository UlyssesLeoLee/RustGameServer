//! `realm_lifecycle` —— 服务器全生命周期管理子模块。
//!
//! 规范：RGS-IMPL-PLAN-LCM-001 v0.1（per 主对话 2026-08-25 12:13 孤儿 SPEC 治理延伸）。
//! 源 DTL：RGS-DTL-042；实现规格：RGS-SPEC-DTL-042 v0.2。
//! 关联：ARC-038 + ARC-051 `realm_lifecycle` Feature + WBS L4 #2066/#2067/#2068/#2070/#2071/#2073/#2074。
//!
//! ## 6 阶段操作器
//!
//! - **NewRealm**：开新服（分配 realm_id + 初始化目录 + 灰度 0%）
//! - **Scale**：扩缩容（同 region 内增减节点；与 NewRealm 共用部分逻辑）
//! - **Split**：分服（高负载服拆分；data migration via Saga）
//! - **Merge**：合服（低负载服合并；merge_conflict_rule_set_v2 锁定后不可改）
//! - **Retire**：退场（realm 关闭 + RBAC 查询通道仅 cs_agent/sre/legal）
//! - **Archive**：归档（冷热分层 + N+2 冗余；不删数据）
//!
//! ## 硬约束（per FR-LCM-003 / FR-LCM-004 / FR-LCM-062 / FR-LCM-081）
//!
//! - `DrillExecutor` 仅跑沙箱环境（sandbox_pg + sandbox_k8s）
//! - `RealmLifecycleService` 不对外暴露独立接口（仅经 AdminService 转发）
//! - `merge_conflict_rule_set_v2` 锁定后不可改
//! - 归档不删除数据，仅迁移存储位置
//!
//! ## 当前状态（per SPEC-DTL-042 §1）
//!
//! 本子模块是 WF-1-2066/2067/2068/2070/2071/2073/2074 等并行 worktree 的目标。
//! 本 worktree（WF-1-2070）只交付：
//! - 6 操作器 trait 签名（M-2070.PREREQ 给 drill 测试引用）
//! - DrillExecutor 框架 + sandbox_pg + sandbox_k8s + 5 类 playbook 模板
//! - 10 项 AC + 3 项 NFR + 2 项 RSK + 6 类故障注入测试代码
//!
//! 其他 worktree 后续会实现具体操作器 + Saga 步骤 + Feature 子类注册 + OLU 上报。

#![allow(clippy::result_large_err)]

pub mod drill;
pub mod error;
pub mod operations;
pub mod plans;
pub mod saga;
pub mod service;

pub use error::{Error, Result};
pub use operations::{
    archive::ArchiveOperator, merge::MergeOperator, new_realm::NewRealmOperator,
    retire::RetireOperator, scale::ScaleOperator, split::SplitOperator,
};
pub use plans::{
    archive_policy::ArchivePolicy, merge_conflict_rule_set_v2::MergeConflictRuleSetV2,
    new_realm_plan::NewRealmPlan, realm_lifecycle_run::RealmLifecycleRun,
    retire_plan::RetirePlan, split_plan::SplitPlan,
};
pub use saga::steps::{SagaPhase, SagaStepKind};
pub use service::{
    LifecyclePhase, LifecycleRequest, LifecycleResponse, RealmLifecycleService,
};

// =====================================================================
// Shared LCM context types (per SPEC-DTL-042 §3 + DTL-042 §3)
// =====================================================================

/// LCM 操作统一请求 ID（per SPEC §3 第 7 条：request_id 幂等键）。
pub type RequestId = String;

/// 操作人 ID（operator）。
pub type OperatorId = String;

/// 批准引用（高危操作必填；运营 + 架构 + SRE 三方签字之一）。
pub type ApprovalRef = Option<String>;

/// Trace ID（OTel 关联）。
pub type TraceId = String;

/// Realm ID（业务实体；低基数标签，per SPEC §4）。
pub type RealmId = String;

/// Saga run ID（关联 saga_orchestrator）。
pub type SagaRunId = String;

/// LCM 阶段状态（per DTL-042 §4 状态机；6 阶段主状态 + 终态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RealmStatus {
    /// 待开服（初始）
    Pending,
    /// Active（运行中）
    Active,
    /// Scaling（扩缩容中）
    Scaling,
    /// Splitting（分服中）
    Splitting,
    /// Merging（合服中）
    Merging,
    /// Retired（已退场）
    Retired,
    /// Archived（已归档）
    Archived,
}

impl RealmStatus {
    /// 转换为 Prometheus 标签字符串（per DTL §11.1）。
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Scaling => "scaling",
            Self::Splitting => "splitting",
            Self::Merging => "merging",
            Self::Retired => "retired",
            Self::Archived => "archived",
        }
    }

    /// 终态判断（用于 RSK-LCM-006 串行调度释放）。
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Archived)
    }

    /// 中间态判断（用于 RSK-LCM-006 串行调度保持）。
    pub const fn is_in_flight(&self) -> bool {
        matches!(
            self,
            Self::Scaling | Self::Splitting | Self::Merging
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_status_as_str_cover_all_variants() {
        // 防止新增 variant 漏改 as_str（per DTL §11.1 指标标签完整）
        let _ = RealmStatus::Pending.as_str();
        let _ = RealmStatus::Active.as_str();
        let _ = RealmStatus::Scaling.as_str();
        let _ = RealmStatus::Splitting.as_str();
        let _ = RealmStatus::Merging.as_str();
        let _ = RealmStatus::Retired.as_str();
        let _ = RealmStatus::Archived.as_str();
    }

    #[test]
    fn terminal_and_in_flight_partition() {
        // 任何 RealmStatus 必须恰好是 terminal / in_flight / idle 之一
        for s in [
            RealmStatus::Pending,
            RealmStatus::Active,
            RealmStatus::Scaling,
            RealmStatus::Splitting,
            RealmStatus::Merging,
            RealmStatus::Retired,
            RealmStatus::Archived,
        ] {
            let t = s.is_terminal();
            let f = s.is_in_flight();
            assert!(
                t ^ f || (!t && !f),
                "RealmStatus::{:?} partition broken: t={t} f={f}",
                s
            );
        }
    }
}
