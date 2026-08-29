//! card-service 域错误类型 (per RGS-SPEC-CROSS-003 错误模型 CROSS 规范草案)
//!
//! 桶 10 实化: 任务书要求 3 错误码 (NotFound / Validation / Conflict),
//! 沿用 player-service 错误模型 8 公共变体 + 域特化 4 变体.
//!
//! 设计原则：
//! - 域 Error 统一封装, 业务层只跟 Result<T, Error> 打交道
//! - gRPC boundary 才转 tonic::Status (per gRPC status code 规范)
//! - sqlx::Error / anyhow::Error 显式 From impl, 不使用 `#[from]` (控制 enum 大小)
//!
//! 业务错误码 (per DTL-038 §4.4 + 任务书):
//! - NotFound: card / card_series / card_instance 不存在
//! - Validation: 参数非法 (prob sum > 1.0 / empty card_id / pack_size = 0 ...)
//! - Conflict: card_series status 非 Ok (不可抽) / card_instance locked

use thiserror::Error;

/// card-service 域统一错误类型
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    // ===== 公共变体 (per RGS-SPEC-CROSS-003 草案 8 个) =====
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

    // ===== 域特化变体 (per DTL-038 §4.4 卡牌域错误) =====
    /// 卡牌不存在
    #[error("card not found: {0}")]
    CardNotFound(String),

    /// 卡包 / 系列不存在
    #[error("card_series not found: {0}")]
    CardSeriesNotFound(String),

    /// 卡牌实例不存在
    #[error("card_instance not found: {0}")]
    CardInstanceNotFound(String),

    /// 卡包状态不可抽 (per DEC-038-06 仅 Ok 状态可抽)
    #[error("card_series not packable: {0}")]
    CardSeriesNotPackable(String),
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

// ===== tonic::Status 映射 (per gRPC status code 规范) =====

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
            Error::CardNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::CardSeriesNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::CardInstanceNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::CardSeriesNotPackable(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
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
            entity: "Card",
            id: "card_001".to_string(),
        };
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::NotFound);
        assert!(s.message().contains("Card"));
    }

    #[test]
    fn validation_to_status() {
        let e = Error::Validation("pack_size = 0".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn conflict_to_status() {
        let e = Error::Conflict("duplicate".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::AlreadyExists);
    }

    #[test]
    fn card_not_found_to_status() {
        let e = Error::CardNotFound("card_001".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn card_series_not_packable_to_status() {
        let e = Error::CardSeriesNotPackable("series_x".to_string());
        let s: tonic::Status = e.into();
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
}
