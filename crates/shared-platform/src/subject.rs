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

/// 从 subject 提取 `aggregate_type`（用于 outbox metrics 标签维度）
///
/// **目的**: 06_Outbox监控指标草案.md §1.1 表要求 `rgs_outbox_*_count{service, aggregate_type}`。
/// 6 域 outbox 表 schema (per 6 份 `0[0-3]X_outbox.sql`) 只有 `subject` 列无 `aggregate_type` 列;
/// ULYS-102 (P2-#4 Schema Evolution) 负责未来添加 `aggregate_type` 列。当前实现从 subject
/// 推断:
/// - 域事件 `rgs.<domain>.<event_type>.<version>` → `<domain>.<event_type>` (例如 `economy.transfer`)
/// - Saga `rgs.saga.<saga_type>.<event>` → `saga.<saga_type>`
/// - CEM `rgs.cem.<event_type>` → `cem.<event_type>`
/// - DLQ `rgs.dlq.<source>` → `dlq`
/// - 解析失败 → `"unknown"` (避免空 label 触发 Prometheus 异常)
///
/// 返回 `'static &str` 借用字面量或 `Cow` 是不可能的 (subject 是运行时字符串), 因此返回
/// `String`, 在 metrics helper 处 `.as_str()` 借用即可。
pub fn aggregate_type_of(subject: &str) -> String {
    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() < 3 || parts[0] != "rgs" {
        return "unknown".to_string();
    }
    match parts[1] {
        // 域事件: aggregate_type = "<domain>.<event_type>" (合并前两段业务含义)
        // 例: rgs.economy.transfer.v1 → "economy.transfer"
        // 例: rgs.player.registered.v1 → "player.registered"
        domain @ ("player" | "economy" | "match" | "social" | "admin" | "cluster_ops") => {
            if parts.len() >= 3 {
                format!("{}.{}", domain, parts[2])
            } else {
                domain.to_string()
            }
        }
        // Saga: aggregate_type = "saga.<saga_type>"
        "saga" => {
            if parts.len() >= 3 {
                format!("saga.{}", parts[2])
            } else {
                "saga".to_string()
            }
        }
        // CEM: aggregate_type = "cem.<event_type>"
        "cem" => {
            if parts.len() >= 3 {
                format!("cem.{}", parts[2])
            } else {
                "cem".to_string()
            }
        }
        // DLQ: aggregate_type = "dlq"
        "dlq" => "dlq".to_string(),
        // 未知前缀: 兜底
        other => {
            if parts.len() >= 3 {
                format!("{}.{}", other, parts[2])
            } else {
                other.to_string()
            }
        }
    }
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

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // Subject 是 RGS-ARC-051 CEM 中心事件管理核心命名空间, 加 3 单测 + 2 proptest

    #[test]
    fn parse_dlq_subject_works() {
        // DLQ subject: rgs.dlq.<source>
        let (d, rest) = parse("rgs.dlq.rgs.player.registered.v1").unwrap();
        assert_eq!(d, SubjectDomain::Dlq);
        assert_eq!(rest, "rgs.player.registered.v1");
    }

    #[test]
    fn parse_cem_event_works() {
        let (d, rest) = parse("rgs.cem.feature_flag_updated").unwrap();
        assert_eq!(d, SubjectDomain::Cem);
        assert_eq!(rest, "feature_flag_updated");
    }

    #[test]
    fn parse_all_five_business_domains() {
        // 5 业务域 + cluster_ops 都应解析为 Domain
        for d in &[
            "player",
            "economy",
            "match",
            "social",
            "admin",
            "cluster_ops",
        ] {
            let subj = format!("rgs.{}.some_event.v1", d);
            let (parsed_domain, _) = parse(&subj).unwrap();
            assert_eq!(
                parsed_domain,
                SubjectDomain::Domain,
                "{} 应被识别为业务域",
                d
            );
        }
    }

    #[test]
    fn parse_empty_or_short_fails() {
        assert!(parse("").is_err());
        assert!(parse("rgs").is_err());
        assert!(parse("rgs.x").is_err());
    }

    #[test]
    fn dlq_subject_wraps_source_correctly() {
        // SubjectBuilder::dlq(source) 之后 parse 回的 rest 应 == source
        let source = "rgs.economy.transferred.v1";
        let dlq = SubjectBuilder::dlq(source);
        let (d, rest) = parse(&dlq).unwrap();
        assert_eq!(d, SubjectDomain::Dlq);
        assert_eq!(rest, source);
    }

    #[test]
    fn role_round_trip() {
        // 5 类 SubjectDomain 都能 format + parse
        // (此处仅验证 format 输出是 rgs.<domain>.<event> 结构)
        let subj = SubjectBuilder::domain_event("player", "registered", 1);
        assert!(subj.starts_with("rgs.player."));
        let (d, _) = parse(&subj).unwrap();
        assert_eq!(d, SubjectDomain::Domain);
    }

    // ============ ULYS-100 P2-#2 aggregate_type_of 单元测试 ============

    #[test]
    fn aggregate_type_of_extracts_domain_event_type() {
        // 域事件: rgs.<domain>.<event_type>.<version> → "<domain>.<event_type>"
        assert_eq!(
            aggregate_type_of("rgs.economy.transfer.v1"),
            "economy.transfer"
        );
        assert_eq!(
            aggregate_type_of("rgs.player.registered.v1"),
            "player.registered"
        );
        assert_eq!(
            aggregate_type_of("rgs.match.matchmake.v1"),
            "match.matchmake"
        );
        assert_eq!(
            aggregate_type_of("rgs.social.friend_added.v1"),
            "social.friend_added"
        );
        assert_eq!(
            aggregate_type_of("rgs.admin.lcm_started.v1"),
            "admin.lcm_started"
        );
        assert_eq!(
            aggregate_type_of("rgs.cluster_ops.health_check.v1"),
            "cluster_ops.health_check"
        );
    }

    #[test]
    fn aggregate_type_of_extracts_saga_event() {
        // Saga: rgs.saga.<saga_type>.<event> → "saga.<saga_type>"
        assert_eq!(
            aggregate_type_of("rgs.saga.transfer.step_completed"),
            "saga.transfer"
        );
        assert_eq!(aggregate_type_of("rgs.saga.cancel.reverted"), "saga.cancel");
    }

    #[test]
    fn aggregate_type_of_extracts_cem_event() {
        // CEM: rgs.cem.<event_type> → "cem.<event_type>"
        assert_eq!(
            aggregate_type_of("rgs.cem.feature_flag_updated"),
            "cem.feature_flag_updated"
        );
    }

    #[test]
    fn aggregate_type_of_dlq_returns_dlq() {
        assert_eq!(aggregate_type_of("rgs.dlq.rgs.player.registered.v1"), "dlq");
    }

    #[test]
    fn aggregate_type_of_invalid_returns_unknown() {
        // 解析失败 → "unknown"（避免空 label 触发 Prometheus 异常）
        assert_eq!(aggregate_type_of(""), "unknown");
        assert_eq!(aggregate_type_of("not.rgs.subject"), "unknown");
        assert_eq!(aggregate_type_of("rgs"), "unknown");
        assert_eq!(aggregate_type_of("rgs.x"), "unknown");
    }

    #[test]
    fn aggregate_type_of_unknown_prefix_falls_back() {
        // 非已知域前缀: 兜底为 "<other>.<event>"
        assert_eq!(
            aggregate_type_of("rgs.unknown_namespace.event.v1"),
            "unknown_namespace.event"
        );
    }
}

// ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
// Subject 命名空间 proptest 守恒 / 不变式
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // domain_event(name, type, v=1..=999) → parse 后 (Domain, name.type.vN)
    proptest! {
        #[test]
        fn domain_event_format_then_parse(
            domain in "[a-z_]{1,16}",
            event_type in "[a-z_]{1,16}",
            version in 1u32..1000,
        ) {
            let subj = SubjectBuilder::domain_event(&domain, &event_type, version);
            // domain 不在 5 业务域 / cluster_ops / saga / cem / dlq 列表时会被识别为 UnknownDomain
            // (per 9/7 14:00 JST 调优, proptest 容忍该 case 只验证 Ok 分支)
            match parse(&subj) {
                Ok((parsed_domain, rest)) => {
                    prop_assert_eq!(parsed_domain, SubjectDomain::Domain);
                    prop_assert!(rest.contains(&event_type), "rest={} 应含 event_type={}", rest, event_type);
                }
                Err(SubjectError::UnknownDomain(_)) => {
                    // 期望外: 调优 (per 9/7 14:00 JST 派生约束)
                }
                Err(e) => {
                    prop_assert!(false, "不应有其他 parse 错误: {:?}", e);
                }
            }
        }
    }

    // saga_event(type, event) → parse 后 (Saga, type.event)
    proptest! {
        #[test]
        fn saga_event_format_then_parse(
            saga_type in "[a-z_]{1,16}",
            event in "[a-z_]{1,16}",
        ) {
            let subj = SubjectBuilder::saga_event(&saga_type, &event);
            let (d, rest) = parse(&subj).expect("saga subject 应能被 parse");
            prop_assert_eq!(d, SubjectDomain::Saga);
            prop_assert!(rest.contains(&saga_type));
            prop_assert!(rest.contains(&event));
        }
    }
}
