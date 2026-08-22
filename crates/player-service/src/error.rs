//! player-service 域错误类型（per RGS-DTL-018 §3 错误模型）
//!
//! 53.3 rgs-testkit 公共错误 + 54.1 域特化错误。

use thiserror::Error;

/// player-service 域统一错误类型
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[source] Box<sqlx::Error>),

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::Status),
}

/// 域统一 Result 类型
pub type Result<T> = std::result::Result<T, Error>;

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::Database(Box::new(e))
    }
}
