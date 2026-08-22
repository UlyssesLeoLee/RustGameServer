//! Subject 命名空间（per RGS-ARC-051 CEM 中心事件管理 + RGS-DTL-100 §5 消息总线）
//!
//! 54.10 实化：Subject 命名规范 + 构造器 + 解析器
//!
//! 命名规则（per RGS-SPEC-CROSS-005 草案）：
//! - 全局事件：rgs.<domain>.<event_type>.<version>
//! - Saga 事件：rgs.saga.<saga_type>.<event>
//! - CEM 路由：rgs.cem.<event_type>
//! - DLQ：rgs.dlq.<source_subject>
//!
//! 示例：
//! - rgs.player.registered.v1
//! - rgs.economy.credited.v1
//! - rgs.saga.transfer.step_completed
//! - rgs.cem.feature_flag_updated

use thiserror::Error;

/// Subject 域
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectDomain {
    /// 域事件（player / economy / match / social / admin / cluster_ops）
    Domain,
    /// Saga 事件
    Saga,
    /// CEM 中心事件
    Cem,
    /// DLQ 死信
    Dlq,
}

/// Subject 解析错误
#[derive(Debug, Error)]
pub enum SubjectError {
    #[error("invalid subject: {0}")]
    InvalidFormat(String),

    #[error("unknown domain prefix: {0}")]
    UnknownDomain(String),
}

/// Subject 命名构造器
pub struct SubjectBuilder;

impl SubjectBuilder {
    /// 构造域事件 subject：`rgs.<domain>.<event_type>.<version>`
    pub fn domain_event(domain: &str, event_type: &str, version: u32) -> String {
        format!("rgs.{}.{}.v{}", domain, event_type, version)
    }

    /// 构造 Saga 事件 subject：`rgs.saga.<saga_type>.<event>`
    pub fn saga_event(saga_type: &str, event: &str) -> String {
        format!("rgs.saga.{}.{}", saga_type, event)
    }

    /// 构造 CEM 事件 subject：`rgs.cem.<event_type>`
    pub fn cem_event(event_type: &str) -> String {
        format!("rgs.cem.{}", event_type)
    }

    /// 构造 DLQ subject：`rgs.dlq.<source>`
    pub fn dlq(source: &str) -> String {
        format!("rgs.dlq.{}", source)
    }
}

/// Subject 解析器
pub fn parse(subject: &str) -> Result<(SubjectDomain, String), SubjectError> {
    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() < 3 || parts[0] != "rgs" {
        return Err(SubjectError::InvalidFormat(subject.to_string()));
    }
    let domain = match parts[1] {
        "saga" => SubjectDomain::Saga,
        "cem" => SubjectDomain::Cem,
        "dlq" => SubjectDomain::Dlq,
        // 域事件
        "player" | "economy" | "match" | "social" | "admin" | "cluster_ops" => {
            SubjectDomain::Domain
        }
        other => return Err(SubjectError::UnknownDomain(other.to_string())),
    };
    Ok((domain, parts[2..].join(".")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_event_format() {
        let s = SubjectBuilder::domain_event("player", "registered", 1);
        assert_eq!(s, "rgs.player.registered.v1");
    }

    #[test]
    fn saga_event_format() {
        let s = SubjectBuilder::saga_event("transfer", "step_completed");
        assert_eq!(s, "rgs.saga.transfer.step_completed");
    }

    #[test]
    fn cem_event_format() {
        let s = SubjectBuilder::cem_event("feature_flag_updated");
        assert_eq!(s, "rgs.cem.feature_flag_updated");
    }

    #[test]
    fn dlq_format() {
        let s = SubjectBuilder::dlq("rgs.player.registered.v1");
        assert_eq!(s, "rgs.dlq.rgs.player.registered.v1");
    }

    #[test]
    fn parse_domain_event() {
        let (d, rest) = parse("rgs.player.registered.v1").unwrap();
        assert_eq!(d, SubjectDomain::Domain);
        // parts[2..] = ["registered", "v1"] joined with "."
        assert_eq!(rest, "registered.v1");
    }

    #[test]
    fn parse_saga_event() {
        let (d, rest) = parse("rgs.saga.transfer.step_completed").unwrap();
        assert_eq!(d, SubjectDomain::Saga);
        assert_eq!(rest, "transfer.step_completed");
    }

    #[test]
    fn parse_invalid() {
        assert!(parse("not.rgs.subject").is_err());
        assert!(parse("rgs.unknown.event").is_err());
    }
}
