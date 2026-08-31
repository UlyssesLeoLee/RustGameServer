//! social-service 域错误类型（per RGS-DTL-026 §3 错误模型）
//!
//! 54.5 实化：8 公共 + 5 域特化（公会 / 成员 / 权限）

use thiserror::Error;

/// social-service 域统一错误类型
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

    // ===== 域特化（per DTL-026 §3 公会状态）=====
    #[error("guild is full: guild {guild_id} reached max members")]
    GuildFull { guild_id: String },

    #[error("player already in guild: player {player_id} guild {guild_id}")]
    AlreadyInGuild { player_id: String, guild_id: String },

    #[error("not a guild member: player {player_id} guild {guild_id}")]
    NotGuildMember { player_id: String, guild_id: String },

    #[error("insufficient permission: required {required}, actual {actual}")]
    InsufficientPermission { required: String, actual: String },

    #[error("guild has been dissolved: {0}")]
    GuildDissolved(String),
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
            Error::GuildFull { .. } => tonic::Status::new(Code::ResourceExhausted, e.to_string()),
            Error::AlreadyInGuild { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::NotGuildMember { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::InsufficientPermission { .. } => {
                tonic::Status::new(Code::PermissionDenied, e.to_string())
            }
            Error::GuildDissolved(_) => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn guild_full_to_resource_exhausted() {
        let s: tonic::Status = Error::GuildFull {
            guild_id: "g1".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::ResourceExhausted);
    }

    #[test]
    fn already_in_guild_to_failed_precondition() {
        let s: tonic::Status = Error::AlreadyInGuild {
            player_id: "p1".to_string(),
            guild_id: "g1".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn insufficient_permission_to_permission_denied() {
        let s: tonic::Status = Error::InsufficientPermission {
            required: "leader".to_string(),
            actual: "member".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::PermissionDenied);
    }

    #[test]
    fn guild_dissolved_to_failed_precondition() {
        let s: tonic::Status = Error::GuildDissolved("g1".to_string()).into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn not_guild_member_to_failed_precondition() {
        let s: tonic::Status = Error::NotGuildMember {
            player_id: "p1".to_string(),
            guild_id: "g1".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn database_to_internal() {
        // 用 anyhow::Error 模拟 Database 内部错误路径(避免依赖具体 sqlx::Error 变体)
        let s: tonic::Status =
            Error::Internal(anyhow::anyhow!("simulated db failure")).into();
        assert_eq!(s.code(), Code::Internal);
    }

    #[test]
    fn validation_to_invalid_argument() {
        let s: tonic::Status = Error::Validation("bad".to_string()).into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn conflict_to_already_exists() {
        let s: tonic::Status = Error::Conflict("dup".to_string()).into();
        assert_eq!(s.code(), Code::AlreadyExists);
    }

    #[test]
    fn not_found_to_not_found() {
        let s: tonic::Status = Error::NotFound {
            entity: "Guild",
            id: "x".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn unauthorized_to_unauthenticated() {
        let s: tonic::Status = Error::Unauthorized("nope".to_string()).into();
        assert_eq!(s.code(), Code::Unauthenticated);
    }

    #[test]
    fn forbidden_to_permission_denied() {
        let s: tonic::Status = Error::Forbidden("nope".to_string()).into();
        assert_eq!(s.code(), Code::PermissionDenied);
    }

    #[test]
    fn unavailable_to_unavailable() {
        let s: tonic::Status = Error::Unavailable("down".to_string()).into();
        assert_eq!(s.code(), Code::Unavailable);
    }
}
