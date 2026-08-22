//! admin-service 域错误类型（per RGS-DTL-019 §3 + ARC-051 COC 错误模型）
//!
//! 54.5 实化：8 公共 + 5 域特化（认证 / 授权 / 审计）

use thiserror::Error;

/// admin-service 域统一错误类型
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
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

    // ===== 域特化（per DTL-019 §3 + SEC-100 §7）=====
    #[error("invalid credentials for admin {0}")]
    InvalidCredentials(String),

    #[error("admin session expired: {0}")]
    AdminSessionExpired(String),

    #[error("audit log tamper detected: expected hash {expected}, actual {actual}")]
    AuditLogTamper { expected: String, actual: String },

    #[error("COC command requires elevated role: required {required}, actual {actual}")]
    COCRoleRequired { required: String, actual: String },

    #[error("CEM event publish failed: topic {topic} reason {reason}")]
    CEMPublishFailed { topic: String, reason: String },
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
            Error::InvalidCredentials(_) => {
                tonic::Status::new(Code::Unauthenticated, e.to_string())
            }
            Error::AdminSessionExpired(_) => {
                tonic::Status::new(Code::Unauthenticated, e.to_string())
            }
            Error::AuditLogTamper { .. } => tonic::Status::new(Code::Internal, e.to_string()),
            Error::COCRoleRequired { .. } => {
                tonic::Status::new(Code::PermissionDenied, e.to_string())
            }
            Error::CEMPublishFailed { .. } => tonic::Status::new(Code::Unavailable, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn invalid_credentials_to_unauthenticated() {
        let s: tonic::Status = Error::InvalidCredentials("root".to_string()).into();
        assert_eq!(s.code(), Code::Unauthenticated);
    }

    #[test]
    fn admin_session_expired_to_unauthenticated() {
        let s: tonic::Status = Error::AdminSessionExpired("root".to_string()).into();
        assert_eq!(s.code(), Code::Unauthenticated);
    }

    #[test]
    fn audit_log_tamper_to_internal() {
        let s: tonic::Status = Error::AuditLogTamper {
            expected: "abc".to_string(),
            actual: "xyz".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::Internal);
    }

    #[test]
    fn coc_role_required_to_permission_denied() {
        let s: tonic::Status = Error::COCRoleRequired {
            required: "super_admin".to_string(),
            actual: "auditor".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::PermissionDenied);
    }

    #[test]
    fn cem_publish_failed_to_unavailable() {
        let s: tonic::Status = Error::CEMPublishFailed {
            topic: "player.events".to_string(),
            reason: "nats timeout".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::Unavailable);
    }
}
