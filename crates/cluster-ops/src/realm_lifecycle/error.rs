//! `realm_lifecycle` 错误码（per RGS-DTL-042 §6 + SPEC-DTL-042 §3）
//!
//! 错误分类（per FR-LCM-002/003/004/062/081 + NFR-LCM-007 既有约束）：
//! - 通用：NotFound / Validation / Conflict / Unauthorized / Forbidden / Unavailable / Internal
//! - 域特化：
//!   - `SagaStepFailed`        跨域 Saga 步骤失败（per §3 第 5 条反向补偿触发）
//!   - `SagaRollbackFailed`    反向补偿失败（per §3 第 5 条，运营需要人工介入）
//!   - `BusinessServiceCallFailed`  业务 service gRPC 调用失败（per §3 第 3 条 + §6 R1）
//!   - `DrillSandboxOnly`      DrillExecutor 拒绝生产 DB 访问（per FR-LCM-003）
//!   - `RetireChannelDenied`   退场查询通道 RBAC 拒绝（per §3 第 8 条 + §6 R5）
//!   - `OluBudgetExceeded`     OLU 预算超限（per NFR-LCM-007 + RSK-LCM-006 缓解）
//!   - `MergeRuleLocked`       合服冲突规则 v2 已锁定不可改（per FR-LCM-062）
//!
//! 不引入新枚举（DTL §6 既有错误码的全集化扩展）。

use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    // ===== 通用 =====
    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("unavailable: {0}")]
    Unavailable(String),

    #[error("internal error: {0}")]
    Internal(#[source] anyhow::Error),

    #[error("gRPC transport error: {0}")]
    Transport(#[source] Box<tonic::Status>),

    // ===== 域特化（per DTL-042 §6 + SPEC-DTL-042 §3）=====
    #[error("saga step failed at step_id={step_id} saga={saga_id}: {reason}")]
    SagaStepFailed {
        step_id: String,
        saga_id: String,
        reason: String,
    },

    #[error("saga reverse compensation failed at step_id={step_id} saga={saga_id}: {reason}")]
    SagaRollbackFailed {
        step_id: String,
        saga_id: String,
        reason: String,
    },

    #[error("business service gRPC call failed service={service} op={op}: {reason}")]
    BusinessServiceCallFailed {
        service: String,
        op: String,
        reason: String,
    },

    #[error("DrillExecutor only allows sandbox environment (per FR-LCM-003); rejected production target: {0}")]
    DrillSandboxOnly(String),

    #[error("retire query channel RBAC denied (per SPEC-DTL-042 §3 第 8 条); required roles={required:?} actual={actual}")]
    RetireChannelDenied {
        required: Vec<String>,
        actual: String,
    },

    #[error("OLU budget exceeded team={team} consumed={consumed} limit={limit}")]
    OluBudgetExceeded {
        team: String,
        consumed: u64,
        limit: u64,
    },

    #[error("merge conflict rule v2 is locked (per FR-LCM-062); rule_id={0} cannot be modified")]
    MergeRuleLocked(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Internal(e)
    }
}

impl From<tonic::Status> for Error {
    fn from(s: tonic::Status) -> Self {
        Error::Transport(Box::new(s))
    }
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::Validation(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::NotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::Conflict(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::Unauthorized(_) => tonic::Status::new(Code::Unauthenticated, e.to_string()),
            Error::Forbidden(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::Unavailable(_) => tonic::Status::new(Code::Unavailable, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
            Error::Transport(s) => *s,
            // 域特化
            Error::SagaStepFailed { .. } => tonic::Status::new(Code::Aborted, e.to_string()),
            Error::SagaRollbackFailed { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::BusinessServiceCallFailed { .. } => {
                tonic::Status::new(Code::Unavailable, e.to_string())
            }
            Error::DrillSandboxOnly(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::RetireChannelDenied { .. } => {
                tonic::Status::new(Code::PermissionDenied, e.to_string())
            }
            Error::OluBudgetExceeded { .. } => {
                tonic::Status::new(Code::ResourceExhausted, e.to_string())
            }
            Error::MergeRuleLocked(_) => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn saga_step_failed_to_aborted() {
        let s: tonic::Status = Error::SagaStepFailed {
            step_id: "s1".into(),
            saga_id: "sg1".into(),
            reason: "x".into(),
        }
        .into();
        assert_eq!(s.code(), Code::Aborted);
    }

    #[test]
    fn retire_channel_denied_to_permission_denied() {
        let s: tonic::Status = Error::RetireChannelDenied {
            required: vec!["cs_agent".into()],
            actual: "player".into(),
        }
        .into();
        assert_eq!(s.code(), Code::PermissionDenied);
    }

    #[test]
    fn olu_budget_exceeded_to_resource_exhausted() {
        let s: tonic::Status = Error::OluBudgetExceeded {
            team: "admin".into(),
            consumed: 100,
            limit: 50,
        }
        .into();
        assert_eq!(s.code(), Code::ResourceExhausted);
    }
}
