//! player-service 域错误类型（per RGS-DTL-018 §3 错误模型）
//!
//! 54.5 实化：
//! - 8 个公共变体（Database / NotFound / Validation / Conflict / Unauthorized /
//!   Forbidden / Unavailable / Internal / Transport）
//! - 5 个域特化变体（NicknameTaken / SessionExpired / AlreadyLoggedIn /
//!   InvalidDevice / AccountDisabled）
//! - `From<Error> for tonic::Status` gRPC status code 映射
//! - 公共 + 域特化共 13 变体 + Status 映射 unit test
//!
//! 设计原则（per RGS-SPEC-CROSS-003 错误模型 CROSS 规范草案）：
//! - 域 Error 统一封装，业务层只跟 Result<T, Error> 打交道
//! - gRPC boundary 才转 tonic::Status（per gRPC status code 规范）
//! - sqlx::Error / anyhow::Error 显式 From impl，不使用 `#[from]`（控制 enum 大小）

use thiserror::Error;

/// player-service 域统一错误类型
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    // ===== 公共变体（per RGS-SPEC-CROSS-003 草案 8 个）=====
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

    // ===== 域特化变体（per DTL-018 §3.1-§3.2 玩家域错误）=====
    #[error("nickname already taken: {0}")]
    NicknameTaken(String),

    #[error("player session expired or invalid")]
    SessionExpired,

    #[error("player already logged in on another device")]
    AlreadyLoggedIn,

    #[error("invalid device fingerprint: {0}")]
    InvalidDevice(String),

    #[error("player account disabled: {0}")]
    AccountDisabled(String),
}

/// 域统一 Result 类型
pub type Result<T> = std::result::Result<T, Error>;

// ===== sqlx Error / anyhow Error 显式 From impl =====

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

// ===== tonic::Status 映射（per gRPC status code 规范）=====

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
            Error::NicknameTaken(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::SessionExpired => tonic::Status::new(Code::Unauthenticated, e.to_string()),
            Error::AlreadyLoggedIn => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
            Error::InvalidDevice(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::AccountDisabled(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn not_found_to_status() {
        let e = Error::NotFound {
            entity: "Player",
            id: "123".to_string(),
        };
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::NotFound);
        assert!(s.message().contains("Player"));
    }

    #[test]
    fn validation_to_status() {
        let e = Error::Validation("name empty".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn nickname_taken_to_status() {
        let e = Error::NicknameTaken("alice".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::AlreadyExists);
    }

    #[test]
    fn session_expired_to_status() {
        let s: tonic::Status = Error::SessionExpired.into();
        assert_eq!(s.code(), Code::Unauthenticated);
    }

    #[test]
    fn already_logged_in_to_status() {
        let s: tonic::Status = Error::AlreadyLoggedIn.into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn transport_passthrough() {
        let original = tonic::Status::new(Code::DeadlineExceeded, "timeout");
        let e: Error = original.clone().into();
        let back: tonic::Status = e.into();
        assert_eq!(back.code(), Code::DeadlineExceeded);
        assert_eq!(back.message(), "timeout");
    }

    #[test]
    fn sqlx_error_wraps_as_database() {
        let sqlx_err = sqlx::Error::PoolClosed;
        let e: Error = sqlx_err.into();
        assert!(matches!(e, Error::Database(_)));
    }

    #[test]
    fn forbidden_account_disabled() {
        let s: tonic::Status = Error::AccountDisabled("banned".to_string()).into();
        assert_eq!(s.code(), Code::PermissionDenied);
    }
}
