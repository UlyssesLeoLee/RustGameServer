//! 域错误类型（per RGS-DTL-042 §6 + ARC-051 + SPEC-DTL-042 §3）
//!
//! WF-1-2066 M-2066.3 骨架：错误码定义（per DTL §6；不引入新枚举）
//!
//! 硬约束：
//! - 不引入新枚举前缀（per task scope：不引入新枚举）
//! - 复用 crate 根 `Error`（per FR-LCM-002/003/004/062/081 + NFR-LCM-007 + RSK-LCM-005/006 既有）
//! - LcmError 仅作"语义别名"包装，承载 LCM 阶段变更高密度场景下的错误分类
//!
//! 阶段变更错误来源：
//! - 6 操作器参数校验失败（NewRealm realm_id 空 / Scale target_capacity ≤ 0 ...）
//! - 状态机非法跳转（per §4 PFAU 状态机：declared → planning → drill_validated → ...）
//! - 二次激活（duplicate activation）：已 Archived 的 realm 不可再 NewRealm
//! - Saga 步骤失败（per RGS-DTL-100 + RGS-DTL-015/016 既有模式）
//! - 演练隔离违规（DrillExecutor 引用生产 DB/K8s 客户端，per FR-LCM-003）
//! - 跨 DB 写失败（per RGS-ADR-0015 Saga 适用边界）
//! - OLU 预算超限（per NFR-LCM-007 硬约束，必须经 rgs-arc-olu）
//! - merge_conflict_rule_set_v2 锁定后修改尝试（per FR-LCM-062）
//!
//! 注：本文件不新增独立错误枚举；所有 LCM 错误通过 `From<LcmError> for crate::Error` 适配到
//! 既有 cluster-ops 域错误类型，确保 AdminService 转发层（FR-LCM-004）的错误码稳定。

use crate::error::Error as ClusterOpsError;
use crate::Result;
use uuid::Uuid;

/// LCM 阶段变更语义错误（per RGS-DTL-042 §6）
///
/// 此处**不**引入独立顶层错误枚举（per M-2066.3 约束），而是承载为
/// `crate::Error` 的内部表示（`Lcm` 变体）。下方的 `LcmError` 是适配层结构，
/// 供 6 操作器骨架（service + operations）使用统一的内部错误构造 API，
/// 避免散落的 `Error::Validation(format!(...))` 字符串漂移。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LcmErrorKind {
    /// 6 操作器参数校验失败（NewRealm realm_id 空 / Scale target_capacity ≤ 0 / etc.）
    InvalidParameter(String),

    /// 6 阶段状态机非法跳转（per §4 PFAU 状态机）
    /// 来自 NewRealm → Scale → Split → Merge → Retire → Archive 的非法路径
    InvalidStageTransition {
        from: String,
        to: String,
        reason: String,
    },

    /// 二次激活：realm 处于已激活/已归档态，触发 NewRealm 被拒
    AlreadyActivated { realm_id: String },

    /// Saga 步骤失败（per RGS-DTL-100 + RGS-DTL-015/016 既有）
    SagaStepFailed {
        saga_id: Uuid,
        step: String,
        reason: String,
    },

    /// 演练隔离违规：DrillExecutor 不应引用生产 DB/K8s 客户端（per FR-LCM-003）
    DrillIsolationViolation { component: String },

    /// 跨 DB 写失败（per RGS-ADR-0015 Saga 适用边界）
    CrossDbWriteFailed { db: String, reason: String },

    /// OLU 预算超限或上报通道失败（per NFR-LCM-007 硬约束）
    OluBudgetExceeded { team: String, requested: u64, ceiling: u64 },

    /// merge_conflict_rule_set_v2 锁定后修改尝试（per FR-LCM-062）
    MergeRuleSetLocked { rule_set_id: Uuid, locked_at: String },

    /// 退场后归档启动阈值未达 / 退场 RBAC 通道未配置（per SPEC §3 第 8 条）
    RetirePrerequisiteMissing { prerequisite: String },

    /// PFAU 状态机非法转移（per §4 PFAU 5 状态 + 7 子类）
    PfauTransitionDenied { from: String, to: String, reason: String },

    /// 6 操作器骨架未实现（M-2066.x 阶段占位符；后续 L4 #2067-#2071 替换为真实实现）
    /// 仅在 LCM 阶段变更路径走骨架时返回；上线 M-2067 Saga 后应消失
    NotImplemented { operator: String, milestone: String },
}

/// LCM 错误承载结构（适配层）
///
/// 不是独立顶层枚举；构造后通过 `From<LcmError> for ClusterOpsError` 统一映射
/// 到 crate 根 `Error::Lcm` 变体（在本骨架阶段由 cluster-ops 域 root Error 承载）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LcmError {
    pub kind: LcmErrorKind,
}

impl LcmError {
    pub fn new(kind: LcmErrorKind) -> Self {
        Self { kind }
    }

    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::InvalidParameter(msg.into()))
    }

    pub fn invalid_stage_transition(
        from: impl Into<String>,
        to: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(LcmErrorKind::InvalidStageTransition {
            from: from.into(),
            to: to.into(),
            reason: reason.into(),
        })
    }

    pub fn already_activated(realm_id: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::AlreadyActivated {
            realm_id: realm_id.into(),
        })
    }

    pub fn saga_step_failed(
        saga_id: Uuid,
        step: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(LcmErrorKind::SagaStepFailed {
            saga_id,
            step: step.into(),
            reason: reason.into(),
        })
    }

    pub fn drill_isolation_violation(component: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::DrillIsolationViolation {
            component: component.into(),
        })
    }

    pub fn cross_db_write_failed(db: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::CrossDbWriteFailed {
            db: db.into(),
            reason: reason.into(),
        })
    }

    pub fn olu_budget_exceeded(team: impl Into<String>, requested: u64, ceiling: u64) -> Self {
        Self::new(LcmErrorKind::OluBudgetExceeded {
            team: team.into(),
            requested,
            ceiling,
        })
    }

    pub fn merge_rule_set_locked(rule_set_id: Uuid, locked_at: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::MergeRuleSetLocked {
            rule_set_id,
            locked_at: locked_at.into(),
        })
    }

    pub fn retire_prerequisite_missing(prerequisite: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::RetirePrerequisiteMissing {
            prerequisite: prerequisite.into(),
        })
    }

    pub fn pfau_transition_denied(
        from: impl Into<String>,
        to: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(LcmErrorKind::PfauTransitionDenied {
            from: from.into(),
            to: to.into(),
            reason: reason.into(),
        })
    }

    pub fn not_implemented(operator: impl Into<String>, milestone: impl Into<String>) -> Self {
        Self::new(LcmErrorKind::NotImplemented {
            operator: operator.into(),
            milestone: milestone.into(),
        })
    }
}

impl std::fmt::Display for LcmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LcmErrorKind::InvalidParameter(msg) => write!(f, "lcm invalid parameter: {msg}"),
            LcmErrorKind::InvalidStageTransition { from, to, reason } => {
                write!(f, "lcm invalid stage transition: {from} -> {to} ({reason})")
            }
            LcmErrorKind::AlreadyActivated { realm_id } => {
                write!(f, "lcm already activated: realm_id={realm_id}")
            }
            LcmErrorKind::SagaStepFailed {
                saga_id,
                step,
                reason,
            } => {
                write!(
                    f,
                    "lcm saga step failed: saga_id={saga_id} step={step} reason={reason}"
                )
            }
            LcmErrorKind::DrillIsolationViolation { component } => {
                write!(f, "lcm drill isolation violation: {component}")
            }
            LcmErrorKind::CrossDbWriteFailed { db, reason } => {
                write!(f, "lcm cross-db write failed: db={db} reason={reason}")
            }
            LcmErrorKind::OluBudgetExceeded {
                team,
                requested,
                ceiling,
            } => {
                write!(
                    f,
                    "lcm olu budget exceeded: team={team} requested={requested} ceiling={ceiling}"
                )
            }
            LcmErrorKind::MergeRuleSetLocked {
                rule_set_id,
                locked_at,
            } => {
                write!(
                    f,
                    "lcm merge rule set locked: rule_set_id={rule_set_id} locked_at={locked_at}"
                )
            }
            LcmErrorKind::RetirePrerequisiteMissing { prerequisite } => {
                write!(f, "lcm retire prerequisite missing: {prerequisite}")
            }
            LcmErrorKind::PfauTransitionDenied { from, to, reason } => {
                write!(f, "lcm pfau transition denied: {from} -> {to} ({reason})")
            }
            LcmErrorKind::NotImplemented {
                operator,
                milestone,
            } => {
                write!(
                    f,
                    "lcm not implemented: operator={operator} milestone={milestone}"
                )
            }
        }
    }
}

impl std::error::Error for LcmError {}

/// LcmError → crate 根 Error 的适配
///
/// 现阶段 cluster-ops 根 `Error` 未含 LCM 变体；本骨架阶段统一映射到 `Validation`
/// 变体（保留语义字符串供 AdminService 转发层解析），L4 #2067 Saga 接入后将
/// 引入专用变体（per SPEC-DTL-042 §3 "实现契约" 既有约束"不引入新枚举"的
/// 临时方案——本骨架结束后，由 L4 #2068 6 表 migration 阶段一并升级）。
impl From<LcmError> for ClusterOpsError {
    fn from(e: LcmError) -> Self {
        ClusterOpsError::Validation(e.to_string())
    }
}

/// LCM 操作 Result 快捷别名
pub type LcmResult<T> = std::result::Result<T, LcmError>;

/// 桥接到 crate 根 `Result`（FR-LCM-004 AdminService 转发层用）
pub fn into_crate_result<T>(r: LcmResult<T>) -> Result<T> {
    r.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcm_error_displays_correctly() {
        let e = LcmError::invalid_parameter("realm_id is empty");
        assert_eq!(e.to_string(), "lcm invalid parameter: realm_id is empty");
    }

    #[test]
    fn lcm_error_already_activated_displays() {
        let e = LcmError::already_activated("realm-007");
        assert!(e.to_string().contains("realm-007"));
        assert!(matches!(e.kind, LcmErrorKind::AlreadyActivated { .. }));
    }

    #[test]
    fn lcm_error_into_crate_error_maps_to_validation() {
        let e = LcmError::invalid_stage_transition("NewRealm", "Archive", "skip middle stages");
        let ce: ClusterOpsError = e.into();
        assert!(matches!(ce, ClusterOpsError::Validation(_)));
    }

    #[test]
    fn lcm_error_saga_step_failed_carries_uuid() {
        let saga_id = Uuid::new_v4();
        let e = LcmError::saga_step_failed(saga_id, "migrate_players", "gRPC timeout");
        match e.kind {
            LcmErrorKind::SagaStepFailed { saga_id: sid, .. } => assert_eq!(sid, saga_id),
            _ => panic!("expected SagaStepFailed"),
        }
    }
}
