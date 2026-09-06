use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum Error {
    #[error("guild not found: {0}")]
    GuildNotFound(String),

    #[error("member not found: {0}")]
    MemberNotFound(String),

    #[error("guild full: capacity {0}")]
    GuildFull(u32),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("already in guild: {0}")]
    AlreadyInGuild(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        use tonic::Code;
        match e {
            Error::GuildNotFound(_) | Error::MemberNotFound(_) => {
                tonic::Status::new(Code::NotFound, e.to_string())
            }
            Error::GuildFull(_) => tonic::Status::new(Code::ResourceExhausted, e.to_string()),
            Error::PermissionDenied(_) => tonic::Status::new(Code::PermissionDenied, e.to_string()),
            Error::InvalidRequest(_) => tonic::Status::new(Code::InvalidArgument, e.to_string()),
            Error::AlreadyInGuild(_) => tonic::Status::new(Code::AlreadyExists, e.to_string()),
            Error::Internal(_) => tonic::Status::new(Code::Internal, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guild_full_to_resource_exhausted() {
        let s: tonic::Status = Error::GuildFull(50).into();
        assert_eq!(s.code(), tonic::Code::ResourceExhausted);
    }
    #[test]
    fn permission_denied_to_permission_denied() {
        let s: tonic::Status = Error::PermissionDenied("x".into()).into();
        assert_eq!(s.code(), tonic::Code::PermissionDenied);
    }
}
