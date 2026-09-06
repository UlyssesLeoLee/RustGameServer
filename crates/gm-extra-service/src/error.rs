use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("player not found: {0}")]
    PlayerNotFound(String),

    #[error("gm permission denied: {0}")]
    PermissionDenied(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::PlayerNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::PermissionDenied(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::InvalidRequest(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permission_denied_code() {
        let s: tonic::Status = Error::PermissionDenied("x".into()).into();
        assert_eq!(s.code(), tonic::Code::PermissionDenied);
    }
}
