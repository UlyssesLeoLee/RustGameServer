//! replay-service 域错误类型 (per RGS-DTL-038 §3 DEC-038-03 + 桶 13)
//!
//! 6 公共变体 + 4 域特化:
//! - NotFound: replay 不存在
//! - Validation: 参数非法 (空 player_a / 负 duration_secs / 0 < chunk_size < 1024 ...)
//! - Conflict: object_key 冲突 (相同 replay_id 重复保存)
//! - Storage: 对象存储错误 (put / get / delete 失败)
//! - StreamFailed: 流式读取中断

use thiserror::Error;

/// replay-service 域统一错误类型
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

    #[error("internal error: {0}")]
    Internal(#[source] anyhow::Error),

    #[error("gRPC transport error: {0}")]
    Transport(#[source] Box<tonic::Status>),

    // ===== 域特化 (per DEC-038-03 replay 域错误) =====
    /// 对象存储错误 (put / get / delete / list 失败)
    #[error("storage error: {0}")]
    Storage(String),

    /// 流式读取失败 (replay 不存在 / chunk 越界 / I/O 中断)
    #[error("stream failed: {0}")]
    StreamFailed(String),

    /// replay 已过期
    #[error("replay expired: {0}")]
    Expired(String),

    /// ReplayMeta 不存在 (查不到)
    #[error("replay not found: {0}")]
    ReplayNotFound(String),
}

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
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
            Error::Transport(s) => *s,
            // 域特化
            Error::Storage(_) => tonic::Status::new(Code::Unavailable, e.to_string()),
            Error::StreamFailed(_) => tonic::Status::new(Code::Aborted, e.to_string()),
            Error::Expired(_) => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
            Error::ReplayNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
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
            entity: "Replay",
            id: "rp-1".to_string(),
        };
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn validation_to_status() {
        let s: tonic::Status = Error::Validation("player_a empty".to_string()).into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn conflict_to_status() {
        let s: tonic::Status = Error::Conflict("dup replay_id".to_string()).into();
        assert_eq!(s.code(), Code::AlreadyExists);
    }

    #[test]
    fn storage_to_unavailable() {
        let s: tonic::Status = Error::Storage("put failed".to_string()).into();
        assert_eq!(s.code(), Code::Unavailable);
    }

    #[test]
    fn stream_failed_to_aborted() {
        let s: tonic::Status = Error::StreamFailed("io broken".to_string()).into();
        assert_eq!(s.code(), Code::Aborted);
    }

    #[test]
    fn expired_to_failed_precondition() {
        let s: tonic::Status = Error::Expired("rp-1".to_string()).into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn replay_not_found_to_not_found() {
        let s: tonic::Status = Error::ReplayNotFound("rp-1".to_string()).into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn database_to_internal() {
        let s: tonic::Status = Error::Database(Box::new(sqlx::Error::PoolClosed)).into();
        assert_eq!(s.code(), Code::Internal);
    }

    #[test]
    fn transport_passthrough() {
        let original = tonic::Status::new(Code::DeadlineExceeded, "timeout");
        let e: Error = original.clone().into();
        let back: tonic::Status = e.into();
        assert_eq!(back.code(), Code::DeadlineExceeded);
        assert_eq!(back.message(), "timeout");
    }
}
