//! scene-service 域错误类型
//!
//! 7 域统一错误模型（per RGS-SPEC-CROSS-003 草案 + 8 域扩展）
//! - 公共变体：Database / NotFound / Validation / Conflict / Unauthorized / Forbidden / Unavailable / Internal
//! - 域特化变体：SceneNotExist / UnitNotFound / PositionOutOfBounds / AlreadyInScene / MoveRejected / QuestAlreadyAccepted / PartnerNotOwned

use thiserror::Error;

/// scene-service 域统一错误类型
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
    Internal(String),

    // ===== 域特化变体 (per scene-service 业务) =====
    #[error("scene not exist: {0}")]
    SceneNotExist(String),

    #[error("unit not found: {0}")]
    UnitNotFound(String),

    #[error("position out of bounds: ({0}, {1})")]
    PositionOutOfBounds(i32, i32),

    #[error("already in scene: {0}")]
    AlreadyInScene(String),

    #[error("move rejected: {0}")]
    MoveRejected(String),

    #[error("quest already accepted: {0}")]
    QuestAlreadyAccepted(String),

    #[error("partner not owned: {0}")]
    PartnerNotOwned(String),
}

/// scene-service 域 Result
pub type Result<T> = std::result::Result<T, Error>;

/// gRPC Status 映射（per gRPC status code 规范）
impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::Database(_) | Error::Internal(_) | Error::Unavailable(_) => {
                tonic::Status::new(Code::Internal, e.to_string())
            }
            Error::NotFound { .. }
            | Error::SceneNotExist(_)
            | Error::UnitNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::Validation(_) | Error::PositionOutOfBounds(_, _) => {
                tonic::Status::new(Code::InvalidArgument, e.to_string())
            }
            Error::Conflict(_)
            | Error::AlreadyInScene(_)
            | Error::QuestAlreadyAccepted(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::Unauthorized(_) => tonic::Status::new(Code::Unauthenticated, e.to_string()),
            Error::Forbidden(_) | Error::MoveRejected(_) | Error::PartnerNotOwned(_) => {
                tonic::Status::new(Code::PermissionDenied, e.to_string())
            }
        }
    }
}
