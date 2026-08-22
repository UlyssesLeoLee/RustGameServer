//! cluster-ops 域错误类型（per RGS-DTL-020 §3 + ARC-051 + DEC-001/DEC-002）
//!
//! 54.5 实化：8 公共 + 5 域特化（节点 / PFAU / 跨服）

use thiserror::Error;

/// cluster-ops 域统一错误类型
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

    // ===== 域特化（per DTL-020 §3 + ARC-051 PFAU）=====
    #[error("node not found: hostname {0}")]
    NodeNotFound(String),

    #[error("node is in maintenance: hostname {0}")]
    NodeMaintenance(String),

    #[error("PFAU version mismatch: expected {expected}, actual {actual}")]
    PFAUVersionMismatch { expected: i64, actual: i64 },

    #[error("PFAU upgrade aborted: feature {feature} reason {reason}")]
    PFAUAborted { feature: String, reason: String },

    #[error("cross-server routing failed: from {from} to {to} reason {reason}")]
    CrossServerRoutingFailed {
        from: String,
        to: String,
        reason: String,
    },
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
            Error::NodeNotFound(_) => tonic::Status::new(Code::NotFound, e.to_string()),
            Error::NodeMaintenance(_) => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::PFAUVersionMismatch { .. } => {
                tonic::Status::new(Code::FailedPrecondition, e.to_string())
            }
            Error::PFAUAborted { .. } => tonic::Status::new(Code::Aborted, e.to_string()),
            Error::CrossServerRoutingFailed { .. } => {
                tonic::Status::new(Code::Unavailable, e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn node_not_found_to_not_found() {
        let s: tonic::Status = Error::NodeNotFound("h1".to_string()).into();
        assert_eq!(s.code(), Code::NotFound);
    }

    #[test]
    fn node_maintenance_to_failed_precondition() {
        let s: tonic::Status = Error::NodeMaintenance("h1".to_string()).into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn pfau_version_mismatch_to_failed_precondition() {
        let s: tonic::Status = Error::PFAUVersionMismatch {
            expected: 5,
            actual: 3,
        }
        .into();
        assert_eq!(s.code(), Code::FailedPrecondition);
    }

    #[test]
    fn pfau_aborted_to_aborted() {
        let s: tonic::Status = Error::PFAUAborted {
            feature: "x".to_string(),
            reason: "timeout".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::Aborted);
    }

    #[test]
    fn cross_server_routing_failed_to_unavailable() {
        let s: tonic::Status = Error::CrossServerRoutingFailed {
            from: "node-a".to_string(),
            to: "node-b".to_string(),
            reason: "no route".to_string(),
        }
        .into();
        assert_eq!(s.code(), Code::Unavailable);
    }
}
