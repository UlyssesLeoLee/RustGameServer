//! rgs-arc-olu 集成测试（per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello）
//!
//! 3 集成场景 (F001~F003),模拟 5 业务域 + cluster-ops tool 调用 rgs-arc-olu:
//! - F001: 5 业务域 (player / economy / match / social / admin) + cluster-ops 6 个 tool
//!         通过 OluClient trait 上报 6 阶段,InMemory mock 记录全部调用
//! - F002: OLU fail-closed 路径,always_reject 时 6 tool 全部失败,无 fallback
//! - F003: 6 tool 调用幂等 (同一 request_id 不重复上报 per NFR-LCM-007 幂等键约束)

use rgs_arc_olu::{
    request_for_phase, InMemoryOluClient, OluClient, OluPhase, OluRequest, OluResponse,
};

/// 6 域 tool 标识（per RGS 5 域架构 + cluster-ops 平台）
const SIX_DOMAINS: &[&str] = &[
    "player",
    "economy",
    "match",
    "social",
    "admin",
    "cluster-ops",
];

/// F001: 5 业务域 + cluster-ops 6 tool 通过 OluClient 上报 6 阶段 OLU
/// 验证: InMemory mock 收到 6 次 send,每次 phase 来自不同域的不同阶段
#[test]
fn f001_six_domains_six_phases_via_olu_client() {
    let client = InMemoryOluClient::always_accept();

    // 6 域各报 1 阶段（每域选不同 OLU 阶段）
    let mappings: &[(&str, OluPhase)] = &[
        ("player", OluPhase::NewRealm),
        ("economy", OluPhase::Scale),
        ("match", OluPhase::Split),
        ("social", OluPhase::Merge),
        ("admin", OluPhase::Retire),
        ("cluster-ops", OluPhase::Archive),
    ];

    for (domain, phase) in mappings {
        let realm = format!("{}.realm", domain);
        let req = request_for_phase(*phase, &realm, domain);
        let resp = client.send(req);
        assert!(resp.accepted, "6 域 {} OLU 上报必须被接受", domain);
    }

    let sent = client.take_sent();
    assert_eq!(sent.len(), 6, "InMemoryOluClient 应记录 6 次 send");
    for (i, (domain, phase)) in mappings.iter().enumerate() {
        assert_eq!(sent[i].team, *domain, "team 字段应等于调用方域");
        assert_eq!(sent[i].phase, phase.as_str(), "phase 字段应等于 OluPhase::as_str");
        assert_eq!(sent[i].token_budget, phase.default_olu_budget());
    }
}

/// F002: NFR-LCM-007 fail-closed 路径 - rgs-arc-olu always_reject 时 6 域 tool 全部 fail
#[test]
fn f002_fail_closed_when_olu_unavailable() {
    let client = InMemoryOluClient::always_reject("rgs-arc-olu gRPC down");
    let mut all_rejected = true;
    for domain in SIX_DOMAINS {
        let req = request_for_phase(OluPhase::NewRealm, "realm", domain);
        let resp = client.send(req);
        if resp.accepted {
            all_rejected = false;
        }
    }
    assert!(
        all_rejected,
        "rgs-arc-olu always_reject 时 6 域 tool 必须全部 fail-closed"
    );
}

/// F003: 幂等键 (request_id) 跨多次 send 可用于去重 (per NFR-LCM-007 幂等键约束)
/// 这里只验证 rgs-arc-olu 接收的 OluRequest 字段完整性,实际去重由 PH-4 真实 gRPC impl 提供
#[test]
fn f003_request_id_idempotency_field_preserved() {
    let client = InMemoryOluClient::always_accept();
    let realm = "realm-42";
    // 同一 request_id 重发 3 次
    let fixed_id = "req-fixed-001";
    for _ in 0..3 {
        let mut req = request_for_phase(OluPhase::Merge, realm, "social");
        req.request_id = fixed_id.to_string();
        let resp = client.send(req);
        assert!(resp.accepted);
    }
    let sent = client.take_sent();
    assert_eq!(sent.len(), 3);
    for s in &sent {
        assert_eq!(s.request_id, fixed_id, "request_id 幂等键必须保留");
        assert_eq!(s.phase, "merge");
        assert_eq!(s.realm_id, realm);
    }
}

/// F004: OluRequest JSON 序列化后字段名 (snake_case) 不变 (per 9/1 派工 5 域 tool 调用约定)
#[test]
fn f004_olu_request_json_uses_snake_case_field_names() {
    let req = OluRequest {
        phase: "new_realm".to_string(),
        realm_id: "r1".to_string(),
        team: "platform".to_string(),
        request_id: "rq-1".to_string(),
        operator_id: "op-1".to_string(),
        trace_id: "trace-1".to_string(),
        token_budget: 4_000_000,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    // 7 个字段都必须 snake_case 出现在 JSON 中
    for f in &[
        "\"phase\":",
        "\"realm_id\":",
        "\"team\":",
        "\"request_id\":",
        "\"operator_id\":",
        "\"trace_id\":",
        "\"token_budget\":",
    ] {
        assert!(json.contains(f), "JSON should contain {}: {}", f, json);
    }
}

/// F005: OluResponse 序列化后 accepted 字段 + reason Optional 正确表达
#[test]
fn f005_olu_response_json_optional_reason() {
    let accept_json = serde_json::to_string(&OluResponse::accept()).unwrap();
    assert!(accept_json.contains("\"accepted\":true"));
    // accept 时 reason 字段为 null
    assert!(accept_json.contains("\"reason\":null"));

    let reject_json = serde_json::to_string(&OluResponse::reject("quota exceeded")).unwrap();
    assert!(reject_json.contains("\"accepted\":false"));
    assert!(reject_json.contains("\"reason\":\"quota exceeded\""));
}
