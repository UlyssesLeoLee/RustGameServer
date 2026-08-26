//! cluster-ops · realm_lifecycle 域错误类型（per RGS-SPEC-DTL-042 §5 + DTL §6）
//!
//! 设计原则（per DTL §6）：
//! - 复用 crate::Error 既有 12 个变体，不引入新顶层 Error enum
//! - LCM 域特化错误以"新变体扩展"形式加在 Error 上
//! - 反向补偿由 saga 层触发，错误向上冒泡
//!
//! 56 类错误（per RGS-SPEC-DTL-042 §5）：
//! - Validation / Conflict / NotFound / Unauthorized / Forbidden / Internal
//! - 域特化：阶段非法跳转 / 步骤超时 / 已应用（幂等）/ 跨域反向补偿失败 / 资源未释放

use thiserror::Error;

/// cluster-ops · realm_lifecycle 域统一错误类型
#[derive(Debug, Error)]
pub enum Error {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("unavailable: {0}")]
    Unavailable(String),

    #[error("internal error: {0}")]
    Internal(#[source] anyhow::Error),

    // ===== 域特化（per RGS-SPEC-DTL-042 §5 + DTL §6）=====

    /// 阶段非法跳转（如：NewRealm → Archive 跳过中间阶段）
    #[error("invalid phase transition: from {from} to {to}")]
    InvalidPhaseTransition { from: String, to: String },

    /// Saga 步骤超时（默认 60s per SPEC §8）
    #[error("saga step timeout: phase {phase} after {secs}s reason {reason}")]
    SagaStepTimeout { phase: String, secs: u64, reason: String },

    /// 幂等冲突：(request_id, operator_id) 唯一索引命中
    /// per RGS-SPEC-DTL-042 §5 幂等一致性
    #[error("already applied: request_id {request_id} operator_id {operator_id}")]
    AlreadyApplied {
        request_id: String,
        operator_id: String,
    },

    /// 跨域 Saga 反向补偿失败
    #[error("cross-domain reverse compensation failed: domain {domain} phase {phase} reason {reason}")]
    CrossDomainReverseCompensationFailed {
        domain: String,
        phase: String,
        reason: String,
    },

    /// 资源未释放（强一致性审计发现）
    #[error("resource not released: kind {kind} id {id}")]
    ResourceNotReleased { kind: String, id: String },

    /// 操作器未实现（per 硬约束：本 worktree 仅占位，业务逻辑属 WF-1-2066/2070/2071）
    #[error("operator not implemented: {0}")]
    OperatorNotImplemented(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Internal(e)
    }
}

impl From<Error> for crate::error::Error {
    fn from(e: Error) -> Self {
        use crate::error::Error as ClusterError;
        match e {
            Error::Validation(m) => ClusterError::Validation(m),
            Error::Conflict(m) => ClusterError::Conflict(m),
            Error::NotFound { entity, id } => ClusterError::NotFound { entity, id },
            Error::Unauthorized(m) => ClusterError::Unauthorized(m),
            Error::Forbidden(m) => ClusterError::Forbidden(m),
            Error::Unavailable(m) => ClusterError::Unavailable(m),
            Error::Internal(e) => ClusterError::Internal(e),
            // 域特化映射到 cluster-ops 既有错误码（不引入新 enum）
            Error::InvalidPhaseTransition { from, to } => ClusterError::Validation(format!(
                "invalid phase transition: from {} to {}",
                from, to
            )),
            Error::SagaStepTimeout { phase, secs, .. } => ClusterError::Unavailable(format!(
                "saga step timeout: phase {} after {}s",
                phase, secs
            )),
            Error::AlreadyApplied { request_id, operator_id } => ClusterError::Conflict(format!(
                "already applied: request_id {} operator_id {}",
                request_id, operator_id
            )),
            Error::CrossDomainReverseCompensationFailed { domain, phase, reason } => {
                ClusterError::Unavailable(format!(
                    "cross-domain reverse compensation failed: domain {} phase {} reason {}",
                    domain, phase, reason
                ))
            }
            Error::ResourceNotReleased { kind, id } => ClusterError::Conflict(format!(
                "resource not released: kind {} id {}",
                kind, id
            )),
            Error::OperatorNotImplemented(m) => ClusterError::Validation(m),
        }
    }
}

impl From<crate::error::Error> for Error {
    fn from(e: crate::error::Error) -> Self {
        match e {
            crate::error::Error::Validation(m) => Error::Validation(m),
            crate::error::Error::Conflict(m) => Error::Conflict(m),
            crate::error::Error::NotFound { entity, id } => Error::NotFound { entity, id },
            crate::error::Error::Unauthorized(m) => Error::Unauthorized(m),
            crate::error::Error::Forbidden(m) => Error::Forbidden(m),
            other => Error::Internal(anyhow::anyhow!("cluster-ops error: {}", other)),
        }
    }
}

impl From<tonic::Status> for Error {
    fn from(s: tonic::Status) -> Self {
        Error::Internal(anyhow::anyhow!("tonic status: {}", s))
    }
}

/// 复用：economy::Error → realm_lifecycle::Error（per M-2067.1 关键复用声明）
///
/// 关键：Saga 编排器调 economy::SagaOrchestrator::execute 返 economy::Error，
/// 需要映射到 LCM 域错误（避免泄漏 economy 域错误码到 LCM API 边界）。
impl From<economy_service::Error> for Error {
    fn from(e: economy_service::Error) -> Self {
        use economy_service::Error as EconomyError;
        match e {
            EconomyError::Database(_) => Error::Internal(anyhow::anyhow!("db: {}", e)),
            EconomyError::NotFound { entity, id } => Error::NotFound { entity, id },
            EconomyError::Validation(m) => Error::Validation(m),
            EconomyError::Conflict(m) => Error::Conflict(m),
            EconomyError::Unauthorized(m) => Error::Unauthorized(m),
            EconomyError::Forbidden(m) => Error::Forbidden(m),
            EconomyError::Unavailable(m) => Error::Unavailable(m),
            EconomyError::Internal(_) => Error::Internal(anyhow::anyhow!("internal: {}", e)),
            EconomyError::Transport(_) => Error::Internal(anyhow::anyhow!("transport: {}", e)),
            EconomyError::InsufficientFunds { .. } => {
                Error::Validation(format!("insufficient funds: {}", e))
            }
            EconomyError::OCCConflict { .. } => Error::Conflict(format!("OCC: {}", e)),
            EconomyError::CurrencyNotFound(c) => Error::NotFound {
                entity: "Currency",
                id: c,
            },
            EconomyError::AccountFrozen(_) => Error::Forbidden(e.to_string()),
            EconomyError::IdempotencyConflict(_) => {
                // 注意：此处 economy 侧的 IdempotencyConflict 转换为 LCM AlreadyApplied
                // 幂等键语义一致
                Error::AlreadyApplied {
                    request_id: "?".to_string(),
                    operator_id: "?".to_string(),
                }
            }
            EconomyError::SagaFailed { step, reason, .. } => Error::SagaStepTimeout {
                phase: step,
                secs: 0,
                reason,
            },
        }
    }
}

/// 反向：realm_lifecycle::Error → economy::Error
///
/// 用途：step handler 实现 EconomySagaStepHandler trait 时，
/// 操作器返 realm_lifecycle::Error，需要转 economy::Error。
impl From<Error> for economy_service::Error {
    fn from(e: Error) -> Self {
        use economy_service::Error as EconomyError;
        match e {
            Error::Validation(m) => EconomyError::Validation(m),
            Error::Conflict(m) => EconomyError::Conflict(m),
            Error::NotFound { entity, id } => EconomyError::NotFound { entity, id },
            Error::Unauthorized(m) => EconomyError::Unauthorized(m),
            Error::Forbidden(m) => EconomyError::Forbidden(m),
            Error::Unavailable(m) => EconomyError::Unavailable(m),
            Error::Internal(_) => EconomyError::Internal(anyhow::anyhow!("lcm: {}", e)),
            // 域特化 → economy 域变体映射
            Error::InvalidPhaseTransition { from, to } => EconomyError::Validation(format!(
                "invalid phase transition: from {} to {}",
                from, to
            )),
            Error::SagaStepTimeout { phase, secs, reason } => EconomyError::SagaFailed {
                saga_id: String::new(),
                step: phase,
                reason: format!("timeout after {}s: {}", secs, reason),
            },
            Error::AlreadyApplied { request_id, operator_id } => {
                EconomyError::IdempotencyConflict(format!(
                    "request_id={} operator_id={}",
                    request_id, operator_id
                ))
            }
            Error::CrossDomainReverseCompensationFailed { domain, phase, reason } => {
                EconomyError::SagaFailed {
                    saga_id: String::new(),
                    step: phase,
                    reason: format!("cross-domain reverse failed: domain={} reason={}", domain, reason),
                }
            }
            Error::ResourceNotReleased { kind, id } => {
                EconomyError::Validation(format!("resource not released: kind={} id={}", kind, id))
            }
            Error::OperatorNotImplemented(m) => EconomyError::Validation(m),
        }
    }
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::Validation(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::Conflict(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::NotFound { .. } => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::Unauthorized(_) => tonic::Status::new(Code::Unauthenticated, e.to_string()),
            Error::Forbidden(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::Unavailable(_) => tonic::Status::new(Code::Unavailable, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
            Error::InvalidPhaseTransition { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::SagaStepTimeout { .. } => {
                tonic::Status::new(Code::DeadlineExceeded, e.to_string())
            }
            Error::AlreadyApplied { .. } => {
                tonic::Status::new(Code::AlreadyExists, e.to_string())
            }
            Error::CrossDomainReverseCompensationFailed { .. } => {
                tonic::Status::new(Code::Unavailable, e.to_string())
            }
            Error::ResourceNotReleased { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::OperatorNotImplemented(_) => {
                tonic::Status::new(Code::Unimplemented, e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_phase_transition_to_failed_precondition() {
        let s: tonic::Status = Error::InvalidPhaseTransition {
            from: "NewRealm".to_string(),
            to: "Archive".to_string(),
        }
        .into();
        assert_eq!(s.code(), tonic::Code::FailedPrecondition);
    }

    #[test]
    fn saga_step_timeout_to_deadline_exceeded() {
        let s: tonic::Status = Error::SagaStepTimeout {
            phase: "Scale".to_string(),
            secs: 60,
            reason: "step exceeded 60s".to_string(),
        }
        .into();
        assert_eq!(s.code(), tonic::Code::DeadlineExceeded);
    }

    #[test]
    fn already_applied_to_already_exists() {
        let s: tonic::Status = Error::AlreadyApplied {
            request_id: "r1".to_string(),
            operator_id: "o1".to_string(),
        }
        .into();
        assert_eq!(s.code(), tonic::Code::AlreadyExists);
    }

    #[test]
    fn from_cluster_error_roundtrip() {
        let original = Error::Validation("x".to_string());
        let cluster: crate::error::Error = original.into();
        let back: Error = cluster.into();
        assert!(matches!(back, Error::Validation(_)));
    }
}
