//! economy-service 域错误类型（per RGS-DTL-015 §3 错误模型 + RGS-DTL-100 Saga 错误）
//!
//! 54.5 实化：8 公共 + 6 域特化（OCC + Saga + 货币）

use thiserror::Error;

/// economy-service 域统一错误类型
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    // ===== 公共变体 =====
    #[error("database error: {0}")]
    Database(#[source] Box<sqlx::Error>),

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("validation error: {0}")]
    Validation(String),

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

    // ===== 域特化（per DTL-015 §3 + DTL-100 Saga）=====
    #[error("insufficient funds: account {account_id} balance {balance} < required {required}")]
    InsufficientFunds {
        account_id: String,
        balance: i64,
        required: i64,
    },

    #[error("OCC conflict: account {account_id} version {version} is stale")]
    OCCConflict { account_id: String, version: i64 },

    #[error("currency not found: {0}")]
    CurrencyNotFound(String),

    #[error("account is frozen: {0}")]
    AccountFrozen(String),

    #[error("idempotency key conflict: {0}")]
    IdempotencyConflict(String),

    #[error("saga failed: {saga_id} step {step} reason {reason}")]
    SagaFailed {
        saga_id: String,
        step: String,
        reason: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::Database(Box::new(e))
    }
}

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
            Error::Database(_) => tonic::Status::new(Code::Internal, e.to_string()),
            Error::NotFound { .. } => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::Validation(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::Conflict(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::Unauthorized(_) => tonic::Status::new(Code::Unauthenticated, e.to_string()),
            Error::Forbidden(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::Unavailable(_) => tonic::Status::new(Code::Unavailable, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
            Error::Transport(s) => *s,
            // 域特化
            Error::InsufficientFunds { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::OCCConflict { .. } => tonic::Status::new(Code::Aborted, e.to_string()),
            Error::CurrencyNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::AccountFrozen(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::IdempotencyConflict(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::SagaFailed { .. } => tonic::Status::new(Code::Aborted, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn insufficient_funds_to_failed_precondition() {
        let e = Error::InsufficientFunds {
            account_id: "acc-1".to_string(),
            balance: 50,
            required: 100,
        };
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn occ_conflict_to_aborted() {
        let s: tonic::Status = Error::OCCConflict {
            account_id: "acc-1".to_string(),
            version: 0,
        }
        .into();
        assert_eq!(s.code(), Code::Aborted);
    }

    #[test]
    fn saga_failed_to_aborted() {
        let s: tonic::Status = Error::SagaFailed {
            saga_id: "s-1".to_string(),
            step: "reserve".to_string(),
            reason: "timeout".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::Aborted);
    }

    #[test]
    fn currency_not_found_to_not_found() {
        let s: tonic::Status = Error::CurrencyNotFound("soul".to_string()).into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn account_frozen_to_permission_denied() {
        let s: tonic::Status = Error::AccountFrozen("acc-1".to_string()).into();
        assert_eq!(s.code(), Code::PermissionDenied);
    }

    #[test]
    fn idempotency_conflict_to_already_exists() {
        let s: tonic::Status = Error::IdempotencyConflict("key-1".to_string()).into();
        assert_eq!(s.code(), Code::AlreadyExists);
    }
}
