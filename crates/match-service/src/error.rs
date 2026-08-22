//! match-service 域错误类型（per RGS-DTL-016 §3 错误模型）
//!
//! 54.5 实化：8 公共 + 5 域特化（Match 状态机错误）

use thiserror::Error;

/// match-service 域统一错误类型
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

    // ===== 域特化（per DTL-016 §3 对局状态机）=====
    #[error("match is full: match {match_id} has no available slots")]
    MatchFull { match_id: String },

    #[error("match already started: {0}")]
    MatchAlreadyStarted(String),

    #[error("player not in match: player {player_id} match {match_id}")]
    NotInMatch { player_id: String, match_id: String },

    #[error("invalid team assignment: {0}")]
    InvalidTeam(String),

    #[error("match slot reservation expired: {0}")]
    ReservationExpired(String),
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
            Error::MatchFull { .. } => tonic::Status::new(Code::ResourceExhausted, e.to_string()),
            Error::MatchAlreadyStarted(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::NotInMatch { .. } => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
            Error::InvalidTeam(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::ReservationExpired(_) => {
                tonic::Status::new(Code::DeadlineExceeded, e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn match_full_to_resource_exhausted() {
        let s: tonic::Status = Error::MatchFull {
            match_id: "m1".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::ResourceExhausted);
    }

    #[test]
    fn match_already_started_to_failed_precondition() {
        let s: tonic::Status = Error::MatchAlreadyStarted("m1".to_string()).into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn not_in_match_to_failed_precondition() {
        let s: tonic::Status = Error::NotInMatch {
            player_id: "p1".to_string(),
            match_id: "m1".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn invalid_team_to_invalid_argument() {
        let s: tonic::Status = Error::InvalidTeam("yellow".to_string()).into();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    #[test]
    fn reservation_expired_to_deadline_exceeded() {
        let s: tonic::Status = Error::ReservationExpired("r1".to_string()).into();
        assert_eq!(s.code(), Code::DeadlineExceeded);
    }
}
