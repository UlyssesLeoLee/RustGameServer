//! shared-platform IT: subject + dlq + json_logging 协同（per PT-WORKER-BRIEFING.md §2）
//!
//! 覆盖 3 个跨模块场景：
//! 1. it_subject_dlq_payload_round_trip
//! 2. it_subject_dlq_serde_preserves_naming
//! 3. it_subject_domain_event_with_saga_id_span

use serde_json;
use shared_platform::dlq::DlqEntry;
use shared_platform::subject::{parse, SubjectBuilder, SubjectDomain};
use uuid::Uuid;

/// 场景 1: DLQ subject 命名 + payload 编码必须闭环
/// (per RGS-SPEC-CROSS-005: rgs.dlq.<source>)
#[test]
fn it_subject_dlq_payload_round_trip() {
    let original_subject = "rgs.economy.transferred.v1";
    let dlq_subject = SubjectBuilder::dlq(original_subject);

    // (a) DLQ subject 格式正确
    assert_eq!(dlq_subject, "rgs.dlq.rgs.economy.transferred.v1");

    // (b) parse DLQ subject → (Dlq, source)
    let (domain, source) = parse(&dlq_subject).expect("DLQ subject 应能 parse");
    assert_eq!(domain, SubjectDomain::Dlq);
    assert_eq!(source, original_subject);

    // (c) DlqEntry 的 payload 编码闭环
    let entry = DlqEntry::new(
        dlq_subject.clone(),
        "EconomyHandler".to_string(),
        5,
        "DB pool exhausted".to_string(),
        Some(Uuid::nil()),
        Some(Uuid::nil()),
        Some(Uuid::nil()),
        b"saga-step-failed".to_vec(),
    );
    assert_eq!(entry.decode_payload(), b"saga-step-failed");
}

/// 场景 2: DLQ entry serde JSON 往返, subject 命名不被破坏
/// (DLQ 落库 / 跨进程传输依赖 JSON 完整性)
#[test]
fn it_subject_dlq_serde_preserves_naming() {
    let original = "rgs.player.registered.v1";
    let dlq = SubjectBuilder::dlq(original);

    let entry = DlqEntry::new(
        dlq.clone(),
        "PlayerHandler".to_string(),
        3,
        "transient error".to_string(),
        None,
        None,
        None,
        b"player-payload".to_vec(),
    );

    let json = serde_json::to_string(&entry).expect("DLQ entry 必须可序列化");
    let decoded: DlqEntry = serde_json::from_str(&json).expect("DLQ entry 必须可反序列化");

    // subject 命名 + payload 都应保真
    assert_eq!(decoded.original_subject, dlq);
    assert_eq!(decoded.decode_payload(), b"player-payload");
    // JSON 中应包含 rgs.dlq 前缀
    assert!(json.contains("rgs.dlq."), "JSON 应含 DLQ 命名空间: {}", json);
}

/// 场景 3: domain event + json_logging span 协同
/// (per RGS-SPEC-CROSS-005: 业务事件 subject + saga_id/request_id 跨域追踪)
#[test]
fn it_subject_domain_event_with_saga_id_span() {
    use shared_platform::json_logging::{with_request_id, with_saga_id};

    let saga_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();
    let domain = "player";
    let event_type = "registered";
    let version = 1;

    // (a) subject 命名
    let subject = SubjectBuilder::domain_event(domain, event_type, version);
    assert_eq!(subject, "rgs.player.registered.v1");
    let (parsed_domain, rest) = parse(&subject).unwrap();
    assert_eq!(parsed_domain, SubjectDomain::Domain);
    assert_eq!(rest, "registered.v1");

    // (b) json_logging span 嵌套 — 在 saga span 内嵌套 request span
    let result = with_saga_id(saga_id, "player.registered", || {
        with_request_id(request_id, || {
            // 业务逻辑位置: 应能获取 event subject
            assert_eq!(subject, "rgs.player.registered.v1");
            true
        })
    });
    assert!(result, "嵌套 span 必须能执行");
}

/// 场景 4 (附加): 5 业务域事件命名都能被 parse + 重构 (回归用)
#[test]
fn it_subject_all_five_business_domains_parseable() {
    for domain in &["player", "economy", "match", "social", "admin"] {
        let subject = SubjectBuilder::domain_event(domain, "test_event", 1);
        let (d, rest) = parse(&subject).expect(&format!("{} 域 subject 应能 parse", domain));
        assert_eq!(d, SubjectDomain::Domain, "{} 应是 Domain", domain);
        assert!(rest.contains("test_event"), "rest 应含 event_type");
    }
}

/// 场景 5 (附加): saga event 与 DLQ 串联
/// (per RGS-SPEC-CROSS-005: saga 超 max_retries → 业务 subject 转 DLQ subject)
#[test]
fn it_subject_saga_to_dlq_chaining() {
    let saga_subject = SubjectBuilder::saga_event("transfer", "step_failed");
    assert_eq!(saga_subject, "rgs.saga.transfer.step_failed");

    let (domain, rest) = parse(&saga_subject).unwrap();
    assert_eq!(domain, SubjectDomain::Saga);
    assert_eq!(rest, "transfer.step_failed");

    // 模拟"超 max_retries → 转 DLQ"
    let dlq_subject = SubjectBuilder::dlq(&saga_subject);
    let (dlq_domain, dlq_source) = parse(&dlq_subject).unwrap();
    assert_eq!(dlq_domain, SubjectDomain::Dlq);
    assert_eq!(dlq_source, saga_subject, "DLQ source 应等于 saga subject");
}
