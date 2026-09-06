use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("mail not found: {0}")]
    MailNotFound(String),

    #[error("friend not found: {0}")]
    FriendNotFound(String),

    #[error("already friends: {0}")]
    AlreadyFriends(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("not authorized: {0}")]
    NotAuthorized(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::MailNotFound(_) | Error::FriendNotFound(_) => {
                tonic::Status::new(Code::NotFound, e.to_string())
            }
            Error::AlreadyFriends(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::InvalidRequest(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::NotAuthorized(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mail_not_found_to_not_found() {
        let s: tonic::Status = Error::MailNotFound("m1".into()).into();
        assert_eq!(s.code(), tonic::Code::NotFound);
    }
    #[test]
    fn invalid_request_to_invalid_argument() {
        let s: tonic::Status = Error::InvalidRequest("x".into()).into();
        assert_eq!(s.code(), tonic::Code::InvalidArgument);
    }
}
