//! rgs-arc-olu 单元测试（per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello）
//!
//! 6 测试 (E001~E006),验证 OluClient trait 契约:
//! - E001: InMemoryOluClient 接受路径 (always_accept)
//! - E002: InMemoryOluClient 拒绝路径 (always_reject, fail-closed per NFR-LCM-007)
//! - E003: InMemoryOluClient 记录 send 调用 (sent 字段)
//! - E004: InMemoryOluClient 记录多条 send 调用
//! - E005: budget_limit 超限拒绝路径
//! - E006: OluResponse accept/reject 构造正确性 + reason 必填

use rgs_arc_olu::{
    request_for_phase, InMemoryOluClient, OluClient, OluPhase, OluRequest, OluResponse,
};

/// E001: InMemoryOluClient 默认 accept 路径
#[test]
fn e001_in_memory_always_accept_returns_accepted() {
    let client = InMemoryOluClient::always_accept();
    let req = request_for_phase(OluPhase::NewRealm, "realm-1", "platform");
    let resp = client.send(req);
    assert!(resp.accepted, "always_accept should return accepted=true");
    assert!(resp.reason.is_none(), "accepted response has no reason");
}

/// E002: InMemoryOluClient always_reject 路径 (per NFR-LCM-007 fail-closed)
#[test]
fn e002_in_memory_always_reject_returns_rejected_with_reason() {
    let client = InMemoryOluClient::always_reject("rgs-arc-olu service down");
    let req = request_for_phase(OluPhase::Split, "realm-7", "platform");
    let resp = client.send(req);
    assert!(!resp.accepted, "always_reject should return accepted=false");
    assert_eq!(
        resp.reason.as_deref(),
        Some("rgs-arc-olu service down"),
        "reason must match the configured reject_reason"
    );
}

/// E003: InMemoryOluClient 记录 send 调用 (sent 字段)
#[test]
fn e003_in_memory_records_send_calls_in_order() {
    let client = InMemoryOluClient::always_accept();
    let r1 = request_for_phase(OluPhase::NewRealm, "realm-A", "platform");
    let r2 = request_for_phase(OluPhase::Scale, "realm-B", "player");
    client.send(r1.clone());
    client.send(r2.clone());
    let sent = client.take_sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0], r1);
    assert_eq!(sent[1], r2);
}

/// E004: InMemoryOluClient take_sent 清空 sent
#[test]
fn e004_in_memory_take_sent_clears_recorded() {
    let client = InMemoryOluClient::always_accept();
    let r1 = request_for_phase(OluPhase::Merge, "realm-X", "economy");
    client.send(r1);
    let first = client.take_sent();
    assert_eq!(first.len(), 1);
    let second = client.take_sent();
    assert!(second.is_empty(), "take_sent should clear the buffer");
}

/// E005: budget_limit 超限 reject (per PH-4 团队配额网关占位)
#[test]
fn e005_in_memory_budget_limit_rejects_overlimit() {
    let client = InMemoryOluClient::with_budget_limit(1_000_000);
    // NewRealm default = 4M, > 1M, should reject
    let big = request_for_phase(OluPhase::NewRealm, "r1", "platform");
    let resp = client.send(big);
    assert!(!resp.accepted);
    let reason = resp.reason.unwrap_or_default();
    assert!(reason.contains("exceeds limit"), "reason should mention limit");
    assert!(reason.contains("1000000"), "reason should include the limit value");
}

/// E006: OluResponse accept/reject 构造正确性
#[test]
fn e006_olu_response_constructors_match_acceptance_state() {
    let accept = OluResponse::accept();
    assert!(accept.accepted);
    assert!(accept.reason.is_none());

    let reject = OluResponse::reject("token exhausted");
    assert!(!reject.accepted);
    assert_eq!(reject.reason.as_deref(), Some("token exhausted"));
}

/// E007 (bonus): request_for_phase 工厂函数填默认值
#[test]
fn e007_request_for_phase_factory_fills_defaults() {
    let r: OluRequest = request_for_phase(OluPhase::Split, "r-Z", "match");
    assert_eq!(r.phase, "split");
    assert_eq!(r.realm_id, "r-Z");
    assert_eq!(r.team, "match");
    assert!(!r.request_id.is_empty(), "request_id should be auto-generated");
    assert!(!r.operator_id.is_empty(), "operator_id should be auto-generated");
    assert!(!r.trace_id.is_empty(), "trace_id should be derived from realm_id");
    assert_eq!(
        r.token_budget,
        OluPhase::Split.default_olu_budget(),
        "token_budget should match OluPhase default"
    );
}
