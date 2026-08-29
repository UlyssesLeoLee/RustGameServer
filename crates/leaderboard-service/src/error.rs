//! leaderboard-service 域错误类型 (per RGS-DTL-038 §3 + §3.1 错误模型)
//!
//! 6 公共 + 3 域特化:
//! - InvalidPage (分页参数非法)
//! - PlayerNotRanked (玩家未入榜)
//! - InvalidLeaderboardSpec (榜单类型 / 周期 / season_id 组合非法)

use thiserror::Error;

/// leaderboard-service 域统一错误类型
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

    // ===== 域特化 (per DTL-038 §3 榜单域错误) =====
    #[error("invalid page request: {0}")]
    InvalidPage(String),

    #[error("player not ranked: {0}")]
    PlayerNotRanked(String),

    #[error("invalid leaderboard spec: {0}")]
    InvalidLeaderboardSpec(String),
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
            Error::InvalidPage(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::PlayerNotRanked(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::InvalidLeaderboardSpec(_) => {
                tonic::Status::new(Code::InvalidArgument, e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn invalid_page_to_invalid_argument() {
        let s: tonic::Status = Error::InvalidPage("page=0".to_string()).into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn player_not_ranked_to_not_found() {
        let s: tonic::Status = Error::PlayerNotRanked("p-1".to_string()).into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn invalid_leaderboard_spec_to_invalid_argument() {
        let s: tonic::Status =
            Error::InvalidLeaderboardSpec("ranked + all_time 暂不支持".to_string()).into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn database_to_internal() {
        let s: tonic::Status = Error::Database(Box::new(sqlx::Error::PoolClosed)).into();
        assert_eq!(s.code(), Code::Internal);
    }
}
