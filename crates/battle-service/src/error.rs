//! battle-service 域错误类型 (per RGS-SPEC-CROSS-003 错误模型)
//!
//! 设计原则 (与 5 域对齐):
//! - 域 Error 统一封装, 业务层只跟 Result<T, Error> 打交道
//! - gRPC boundary 才转 tonic::Status
//! - sqlx::Error / anyhow::Error 显式 From impl, 不使用 `#[from]` (控制 enum 大小)
//!
//! 业务错误码 (per 闪烁之光借鉴 + 战斗域):
//! - NotFound: 战斗/房间/BOSS/副本/护送/圣器/公会战 不存在
//! - Validation: 参数非法 (空 battle_id / 数值超界 ...)
//! - Conflict: 战斗状态机非法转移 / 资源不足 / 次数耗尽
//! - Unimplemented: 220 个 stub RPC 暂时未实装 (per W5 简报)

use thiserror::Error;

/// battle-service 域统一错误类型
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

    // ===== 域特化变体 (per 战斗域) =====
    #[error("battle not found: {0}")]
    BattleNotFound(String),

    #[error("battle invalid state transition: from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("battle resource insufficient: {0}")]
    ResourceInsufficient(String),

    #[error("challenge count exhausted: {0}")]
    ChallengeExhausted(String),

    #[error("activity not in window: activity_id={0}")]
    ActivityNotInWindow(String),

    #[error("pvp mode not configured: {0}")]
    PvpModeNotConfigured(String),

    #[error("unimplemented: {0}")]
    Unimplemented(String),
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
            Error::BattleNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::InvalidStateTransition { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::ResourceInsufficient(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::ChallengeExhausted(_) => {
                tonic::Status::new(Code::ResourceExhausted, e.to_string())
            }
            Error::ActivityNotInWindow(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::PvpModeNotConfigured(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::Unimplemented(_) => tonic::Status::new(Code::Unimplemented, e.to_string()),
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
            entity: "Battle",
            id: "b1".to_string(),
        };
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn validation_to_status() {
        let e = Error::Validation("empty battle_id".to_string());
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
    fn battle_not_found_to_status() {
        let e = Error::BattleNotFound("b1".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn invalid_state_transition_to_status() {
        let e = Error::InvalidStateTransition {
            from: "Init".to_string(),
            to: "End".to_string(),
        };
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn resource_insufficient_to_status() {
        let e = Error::ResourceInsufficient("stamina 0".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn challenge_exhausted_to_status() {
        let e = Error::ChallengeExhausted("pvp 0/5".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::ResourceExhausted);
    }

    #[test]
    fn activity_not_in_window_to_status() {
        let e = Error::ActivityNotInWindow("93031".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn unimplemented_to_status() {
        let e = Error::Unimplemented("BattleShareReplay".to_string());
        let s: tonic::Status = e.into();
        assert_eq!(s.code(), Code::Unimplemented);
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
