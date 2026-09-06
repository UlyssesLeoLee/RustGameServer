//! activity-service 域错误类型

use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("activity not found: {0}")]
    ActivityNotFound(String),

    #[error("activity not open at this time: {0}")]
    ActivityNotOpen(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("player state error: {0}")]
    PlayerState(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::ActivityNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::ActivityNotOpen(_) => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
            Error::InvalidRequest(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::PlayerState(_) => tonic::Status::new(Code::FailedPrecondition, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_not_found_to_not_found() {
        let s: tonic::Status = Error::ActivityNotFound("93031".into()).into();
        assert_eq!(s.code(), tonic::Code::NotFound);
    }

    #[test]
    fn invalid_request_to_invalid_argument() {
        let s: tonic::Status = Error::InvalidRequest("day=0".into()).into();
        assert_eq!(s.code(), tonic::Code::InvalidArgument);
    }
}
