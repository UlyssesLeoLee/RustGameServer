//! i18n-service 域错误类型 (per RGS-DTL-038 §4.1 + DEC-038-05)
//!
//! 6 公共 + 2 域特化:
//! - LocaleNotFound (请求的 locale 未启用)
//! - KeyNotFound (key 在所有 fallback locale 都缺失)

use thiserror::Error;

/// i18n-service 域统一错误类型
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

    #[error("internal error: {0}")]
    Internal(#[source] anyhow::Error),

    #[error("gRPC transport error: {0}")]
    Transport(#[source] Box<tonic::Status>),

    // ===== 域特化 (per DTL-038 §4.1 i18n 域错误) =====
    #[error("locale not found: {0}")]
    LocaleNotFound(String),

    #[error("i18n key not found: {0}")]
    KeyNotFound(String),
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
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
            Error::Transport(s) => *s,
            // 域特化
            Error::LocaleNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::KeyNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn locale_not_found_to_not_found() {
        let s: tonic::Status = Error::LocaleNotFound("zh_cn".to_string()).into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn key_not_found_to_not_found() {
        let s: tonic::Status = Error::KeyNotFound("card.foo.bar".to_string()).into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn database_to_internal() {
        let s: tonic::Status = Error::Database(Box::new(sqlx::Error::PoolClosed)).into();
        assert_eq!(s.code(), Code::Internal);
    }
}
