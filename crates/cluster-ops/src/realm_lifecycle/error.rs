//! realm_lifecycle 错误类型（per DTL-042 §6 + SPEC-DTL-042 §3）。
//!
//! 不引入新枚举基类；复用 `crate::Error` 通用变体 + 本模块专用变体。
//!
//! 锚定：FR-LCM-002（阶段变更全流程留痕）/ FR-LCM-003（drill 仅沙箱）/
//!       FR-LCM-062（merge_conflict_rule_set_v2 锁定后不可改）/ FR-LCM-081（归档不删数据）。

use thiserror::Error;

use super::RealmId;

/// realm_lifecycle 子模块专用错误（per DTL-042 §6 错误码 + SPEC §3）。
///
/// 不引入新枚举基类；以下变体为本模块独占，会经 `From<realm_lifecycle::Error> for crate::Error`
/// 在 service.rs 中桥接到域统一错误。
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    /// 阶段非法跳转（per DTL §4 状态机：例如 Pending → Merging）。
    #[error("illegal phase transition: from {from} to {to}")]
    IllegalPhaseTransition { from: String, to: String },

    /// 阶段二次激活（同一 request_id 重复提交，但状态机不允许二次激活）。
    #[error("phase double activation: realm {realm} request {request_id}")]
    DoubleActivation { realm: RealmId, request_id: String },

    /// Saga 步骤超时（默认 60s，per SPEC §5 背压规则）。
    #[error("saga step timeout: phase {phase} step {step} after {elapsed_ms}ms")]
    SagaStepTimeout {
        phase: String,
        step: String,
        elapsed_ms: u64,
    },

    /// Saga 步骤执行失败（per SPEC §5 故障域：Saga 步骤失败）。
    #[error("saga step failed: phase {phase} step {step} reason {reason}")]
    SagaStepFailed {
        phase: String,
        step: String,
        reason: String,
    },

    /// Saga 反向补偿失败（per SPEC §5 R9 风险缓解：业务 service gRPC 失败导致反向不完整）。
    #[error("saga rollback failed: phase {phase} step {step} reason {reason}")]
    SagaRollbackFailed {
        phase: String,
        step: String,
        reason: String,
    },

    /// merge_conflict_rule_set_v2 锁定后仍尝试修改（per FR-LCM-062）。
    #[error(
        "merge_conflict_rule_set_v2 already locked at {locked_at} (FR-LCM-062): realm {realm}"
    )]
    MergeRulesLocked {
        realm: RealmId,
        locked_at: chrono::DateTime<chrono::Utc>,
    },

    /// 归档尝试删除数据（per FR-LCM-081：不删数据，只迁移存储位置）。
    #[error("archive must not delete data (FR-LCM-081): realm {realm}")]
    ArchiveDeleteForbidden { realm: RealmId },

    /// 退场后查询通道 RBAC 不允许（per SPEC §3 第 8 条：仅 cs_agent / sre / legal）。
    #[error("retired realm query denied for role {role}: realm {realm}")]
    RetiredQueryDenied { realm: RealmId, role: String },

    /// drill 试图引用生产 PG / 生产 K8s 客户端（per FR-LCM-003：仅沙箱）。
    #[error("drill executor must not reference production PG / K8s (FR-LCM-003)")]
    DrillProductionLeak,

    /// 沙箱 PG 池不可达（演练环境未启动）。
    #[error("sandbox PG pool unavailable: {0}")]
    SandboxPgUnavailable(String),

    /// 沙箱 K8s 客户端不可达。
    #[error("sandbox K8s client unavailable: {0}")]
    SandboxK8sUnavailable(String),

    /// 阶段变更高密度期间 OLU 预算超限（per RSK-LCM-006 缓解：串行调度）。
    #[error("OLU budget exceeded for phase {phase} (RSK-LCM-006)")]
    OluBudgetExceeded { phase: String },

    /// 跨域 Saga 跨 DB 协调失败（per R1 风险 + ADR-0015 Saga 适用边界）。
    #[error("cross-DB saga coordination failed: phase {phase} db {db}")]
    CrossDbCoordinationFailed { phase: String, db: String },

    /// rgs-arc-olu 通道不可达（per NFR-LCM-007 硬约束）。
    #[error("rgs-arc-olu channel unavailable (NFR-LCM-007)")]
    OluChannelUnavailable,

    /// 计划表 query_channel_rbac 角色配置不合法（per SPEC §3 第 8 条）。
    #[error("invalid query_channel_rbac role {role} for retire plan {plan}")]
    InvalidRetireRbac { plan: String, role: String },

    /// 通用验证错误（封装到域 Error::Validation）。
    #[error("validation: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// 桥接到 cluster-ops 域统一错误。
impl From<Error> for crate::Error {
    fn from(e: Error) -> Self {
        use crate::Error as Domain;
        match e {
            Error::Validation(s) => Domain::Validation(s),
            Error::IllegalPhaseTransition { from, to } => {
                Domain::Validation(format!("illegal phase transition {from} -> {to}"))
            }
            Error::DoubleActivation { realm, request_id } => {
                Domain::Conflict(format!("phase double activation: {realm} / {request_id}"))
            }
            Error::SagaStepTimeout {
                phase,
                step,
                elapsed_ms,
            } => Domain::Unavailable(format!(
                "saga step timeout: {phase} / {step} / {elapsed_ms}ms"
            )),
            Error::SagaStepFailed {
                phase,
                step,
                reason,
            } => Domain::Unavailable(format!(
                "saga step failed: {phase} / {step} / {reason}"
            )),
            Error::SagaRollbackFailed {
                phase,
                step,
                reason,
            } => Domain::Unavailable(format!(
                "saga rollback failed: {phase} / {step} / {reason}"
            )),
            Error::MergeRulesLocked { realm, locked_at } => {
                Domain::Conflict(format!("merge rules locked: {realm} @ {locked_at}"))
            }
            Error::ArchiveDeleteForbidden { realm } => {
                Domain::Forbidden(format!("archive must not delete: {realm}"))
            }
            Error::RetiredQueryDenied { realm, role } => {
                Domain::Forbidden(format!("retired query denied: {realm} / {role}"))
            }
            Error::DrillProductionLeak => {
                Domain::Forbidden("drill executor leaked to production".to_string())
            }
            Error::SandboxPgUnavailable(s) => Domain::Unavailable(format!("sandbox PG: {s}")),
            Error::SandboxK8sUnavailable(s) => {
                Domain::Unavailable(format!("sandbox K8s: {s}"))
            }
            Error::OluBudgetExceeded { phase } => {
                Domain::Conflict(format!("OLU budget exceeded: {phase}"))
            }
            Error::CrossDbCoordinationFailed { phase, db } => {
                Domain::Unavailable(format!("cross-DB saga: {phase} / {db}"))
            }
            Error::OluChannelUnavailable => {
                Domain::Unavailable("rgs-arc-olu channel unavailable".to_string())
            }
            Error::InvalidRetireRbac { plan, role } => {
                Domain::Validation(format!("invalid retire rbac: {plan} / {role}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_variants_display_does_not_panic() {
        // 防止 Display impl 漏写新字段导致 panic
        let _ = Error::IllegalPhaseTransition {
            from: "Pending".to_string(),
            to: "Merging".to_string(),
        }
        .to_string();
        let _ = Error::DoubleActivation {
            realm: "r-1".to_string(),
            request_id: "req-1".to_string(),
        }
        .to_string();
        let _ = Error::SagaStepTimeout {
            phase: "Merge".to_string(),
            step: "ApplyConflict".to_string(),
            elapsed_ms: 60_001,
        }
        .to_string();
        let _ = Error::MergeRulesLocked {
            realm: "r-1".to_string(),
            locked_at: chrono::Utc::now(),
        }
        .to_string();
        let _ = Error::DrillProductionLeak.to_string();
        let _ = Error::OluChannelUnavailable.to_string();
    }

    #[test]
    fn bridge_to_domain_error_preserves_semantics() {
        // 锚定 FR-LCM-062 语义：锁定冲突 → Domain::Conflict
        let e = Error::MergeRulesLocked {
            realm: "r-x".to_string(),
            locked_at: chrono::Utc::now(),
        };
        let _: crate::Error = e.into();

        // 锚定 FR-LCM-081 语义：归档删除禁止 → Domain::Forbidden
        let e = Error::ArchiveDeleteForbidden {
            realm: "r-x".to_string(),
        };
        let _: crate::Error = e.into();
    }
}
