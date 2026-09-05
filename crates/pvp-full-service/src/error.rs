use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("unknown pvp mode: {0}")]
    UnknownMode(String),

    #[error("daily limit reached for mode {0}")]
    DailyLimitReached(String),

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
            Error::UnknownMode(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::DailyLimitReached(_) => tonic::Status::new(Code::ResourceExhausted, e.to_string()),
            Error::InvalidRequest(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_mode_to_invalid_argument() {
        let s: tonic::Status = Error::UnknownMode("x".into()).into();
        assert_eq!(s.code(), tonic::Code::InvalidArgument);
    }
}
