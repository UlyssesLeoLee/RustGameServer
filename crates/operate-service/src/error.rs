use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("gift pack not found: {0}")]
    GiftPackNotFound(String),

    #[error("recharge package not found: {0}")]
    RechargeNotFound(String),

    #[error("purchase limit reached for pack {0}")]
    LimitReached(u32),

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
            Error::GiftPackNotFound(_) | Error::RechargeNotFound(_) => {
                tonic::Status::new(Code::NotFound, e.to_string())
            }
            Error::LimitReached(_) => tonic::Status::new(Code::ResourceExhausted, e.to_string()),
            Error::InvalidRequest(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gift_not_found_to_not_found() {
        let s: tonic::Status = Error::GiftPackNotFound("g".into()).into();
        assert_eq!(s.code(), tonic::Code::NotFound);
    }
}
