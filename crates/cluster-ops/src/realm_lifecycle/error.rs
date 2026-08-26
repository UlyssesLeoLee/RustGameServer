//! realm_lifecycle 域特化错误（per RGS-SPEC-DTL-042 §3 + DTL-031 §4.1）
//!
//! 在 cluster-ops 域统一 `Error` 上扩展 LCM 专用变体。**不**覆盖 PFAU 编排
//! 顶层错误（NodeNotFound / PFAUVersionMismatch / PFAUAborted 等已落在
//! cluster-ops::error），本模块专注于 LCM 业务侧错误。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("LCM validation: {0}")]
    Validation(String),

    #[error("LCM PFAU illegal transition: {from} -> {to} (sub_feature={sub_feature})")]
    PFAUIllegalTransition {
        from: &'static str,
        to: &'static str,
        sub_feature: &'static str,
    },

    #[error("LCM PFAU feature not registered: sub_feature={0}")]
    PFAUFeatureNotRegistered(String),

    #[error("LCM saga step failed: step={step} reason={reason}")]
    SagaStepFailed { step: String, reason: String },

    #[error("LCM OLU report failed: phase={phase} team={team} reason={reason}")]
    OLUReportFailed {
        phase: String,
        team: String,
        reason: String,
    },

    #[error("LCM not found: {entity}")]
    NotFound { entity: &'static str },

    #[error("LCM unauthorized: {0}")]
    Unauthorized(String),

    #[error("LCM internal: {0}")]
    Internal(#[source] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Internal(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pfau_illegal_transition_message_contains_sub_feature() {
        let e = Error::PFAUIllegalTransition {
            from: "declared",
            to: "active",
            sub_feature: "merge_rollback",
        };
        let msg = e.to_string();
        assert!(msg.contains("declared"));
        assert!(msg.contains("active"));
        assert!(msg.contains("merge_rollback"));
    }

    #[test]
    fn olu_report_failed_message_contains_team() {
        let e = Error::OLUReportFailed {
            phase: "new_realm".to_string(),
            team: "platform".to_string(),
            reason: "arc-olu unavailable".to_string(),
        };
        assert!(e.to_string().contains("platform"));
        assert!(e.to_string().contains("arc-olu"));
    }
}
