//! W36 (2026-08-30) gm.proto v0.4 实际集成 IT
//!
//! 验证 gm.proto v0.4 字段真实链路 (gm-backend → admin-service gRPC):
//! 1. BanAccount force_disconnect_session 字段透传 + disconnected_sessions 填值
//! 2. GrantCompensation card_ids 字段透传 + cards_granted 填值
//! 3. SetMaintenance mode_flags 字段透传 + applied_mode_flags 填值
//! 4. QueryAuditLog audit_type 过滤 + applied_audit_type echo
//! 5. v0.3 老 client 调用 v0.4 server 仍 PASS (兼容, 0 破坏)
//!
//! 测试策略 (per 既有 it_admin_grpc_4rpc.rs 模式):
//! - disable_admin_grpc=false + 不可达 admin endpoint (1 端口)
//! - 真实 gRPC 调用 → 500ms timeout → 降级 InMemory
//! - 验证: v0.4 字段在 HTTP 请求/响应中被正确处理
//!
//! 关联: RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07
//!       docs/00-基准与治理/RGS-DTL-038 §4.3

use axum::http::StatusCode;
use axum_test::TestServer;
use gm_backend::{AppState, GmConfig};
use std::time::{Duration, Instant};

// ============================================================================
// Test helpers
// ============================================================================

/// 构造真实链路 AppState (admin gRPC 不可达)
/// 模拟: admin-service 启动中 / 跨域网络分区
/// 行为: 真实 gRPC call → 500ms timeout → 降级 InMemory
fn make_real_link_state() -> AppState {
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(), // 不可达端口
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false, // 真实 gRPC 调用
    };
    AppState::new(cfg)
}

fn make_real_link_server() -> TestServer {
    let state = make_real_link_state();
    let app = gm_backend::build_router(state);
    TestServer::new(app).expect("test server should bind")
}

// ============================================================================
// 1. BanAccount 真实调 admin-service 验证 force_disconnect / disconnected_sessions
// ============================================================================

#[tokio::test]
async fn it_ban_real_link_validates_v04_force_disconnect_field() {
    // 真实 gRPC 调用 (admin 不可达 → 500ms timeout → 降级)
    // 验证: v0.4 field force_disconnect_session=true 透传 + disconnected_sessions 填值
    let server = make_real_link_server();
    let started = Instant::now();
    let resp = server
        .post("/api/v1/gm/ban")
        .json(&serde_json::json!({
            "account_id": "p_real_force",
            "reason": "real e2e force test",
            "duration_seconds": 3600,
            "force_disconnect_session": true,
        }))
        .await;
    let elapsed = started.elapsed();

    // 500ms timeout + jitter → < 1.5s
    assert!(
        elapsed < Duration::from_millis(1500),
        "ban with unreachable admin must complete < 1.5s, got {elapsed:?}"
    );
    resp.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();

    // v0.4 字段必须存在
    assert!(
        body.as_object().unwrap().contains_key("force_disconnect_session"),
        "v0.4 force_disconnect_session must exist in response"
    );
    assert_eq!(body["force_disconnect_session"], true);
    assert!(
        body.as_object().unwrap().contains_key("disconnected_sessions"),
        "v0.4 disconnected_sessions must exist in response"
    );
    // 降级路径: disconnected_sessions = body.force_disconnect_session
    assert_eq!(body["disconnected_sessions"], true);
    // degraded 标记
    assert_eq!(body["degraded"], true, "unreachable admin must mark degraded");
    assert_eq!(body["op"], "ban");
}

// ============================================================================
// 2. GrantCompensation 真实调 admin-service 验证 card_ids
// ============================================================================

#[tokio::test]
async fn it_compensation_real_link_validates_v04_card_ids_field() {
    let server = make_real_link_server();
    let started = Instant::now();
    let resp = server
        .post("/api/v1/gm/compensation")
        .json(&serde_json::json!({
            "account_id": "p_real_cards",
            "amount": 100,
            "currency": "USD",
            "reason": "real e2e card comp",
            "card_ids": ["card_001", "card_002"],
            "pack_ids": ["pack_a"],
        }))
        .await;
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(1500),
        "compensation with unreachable admin must complete < 1.5s, got {elapsed:?}"
    );
    resp.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();

    // v0.4 字段: card_ids / pack_ids / cards_granted / packs_granted
    let card_ids = body["card_ids"].as_array().expect("card_ids must be array");
    assert_eq!(card_ids.len(), 2, "card_ids 透传");
    assert_eq!(card_ids[0], "card_001");
    assert_eq!(card_ids[1], "card_002");

    let pack_ids = body["pack_ids"].as_array().expect("pack_ids must be array");
    assert_eq!(pack_ids.len(), 1, "pack_ids 透传");
    assert_eq!(pack_ids[0], "pack_a");

    // 降级路径: cards_granted = body.card_ids.len() = 2
    assert_eq!(body["cards_granted"], 2);
    // 降级路径: packs_granted = body.pack_ids.len() = 1
    assert_eq!(body["packs_granted"], 1);
    assert_eq!(body["degraded"], true);
    assert_eq!(body["op"], "compensation");
}

// ============================================================================
// 3. SetMaintenance 验证 mode_flags / applied_mode_flags 透传
// ============================================================================

#[tokio::test]
async fn it_maintenance_real_link_validates_v04_mode_flags_field() {
    let server = make_real_link_server();
    let started = Instant::now();
    // 0b0011 = 3: bit0 ladder_freeze + bit1 trade_freeze
    let resp = server
        .post("/api/v1/gm/maintenance")
        .json(&serde_json::json!({
            "enable": true,
            "scope": "cluster",
            "target_id": "cluster-prod",
            "ttl_seconds": 1800,
            "mode_flags": 3,
        }))
        .await;
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(1500),
        "maintenance with unreachable admin must complete < 1.5s, got {elapsed:?}"
    );
    resp.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();

    // v0.4 字段
    assert_eq!(body["mode_flags"], 3, "mode_flags 透传");
    assert!(
        body.as_object().unwrap().contains_key("applied_mode_flags"),
        "v0.4 applied_mode_flags must exist in response"
    );
    // 降级路径: applied_mode_flags = body.mode_flags
    assert_eq!(body["applied_mode_flags"], 3);
    // propagation_status 应为 PROPAGATING 或 CONVERGED
    let ps = body["propagation_status"].as_str().expect("propagation_status");
    assert!(
        ps == "PROPAGATING" || ps == "CONVERGED",
        "propagation_status must be PROPAGATING or CONVERGED, got {ps}"
    );
    assert_eq!(body["op"], "maintenance");
}

// ============================================================================
// 4. QueryAuditLog 验证 audit_type 过滤 + applied_audit_type echo
// ============================================================================

#[tokio::test]
async fn it_query_audit_real_link_validates_v04_audit_type_filter() {
    let server = make_real_link_server();
    let started = Instant::now();
    let resp = server
        .get("/api/v1/audit/logs?audit_type=compensation&limit=10")
        .await;
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(1500),
        "query_audit with unreachable admin must complete < 1.5s, got {elapsed:?}"
    );
    resp.assert_status(StatusCode::OK);
    let body: serde_json::Value = resp.json();

    // v0.4 字段
    assert_eq!(body["audit_type"], "compensation", "audit_type 透传");
    assert_eq!(
        body["applied_audit_type"], "compensation",
        "applied_audit_type echo"
    );
    // 降级路径 InMemory → entries 应为空 (无预加载)
    let entries = body["entries"].as_array().expect("entries must be array");
    assert_eq!(entries.len(), 0);
    // v0.4: entries[].audit_type 字段 (空 entries 也有 schema)
    assert_eq!(body["has_more"], false);
}

// ============================================================================
// 5. v0.3 老 client 调用 v0.4 server 仍 PASS (兼容, 0 破坏)
// ============================================================================

#[tokio::test]
async fn it_v03_compat_old_client_calls_v04_server_still_passes() {
    // v0.3 老 client: 不传任何 v0.4 字段
    // v0.4 server: 应正常处理 (向后兼容)
    let server = make_real_link_server();

    // 1. 老 ban 请求(无 force_disconnect_session) → 202
    let resp = server
        .post("/api/v1/gm/ban")
        .json(&serde_json::json!({
            "account_id": "p_v03_real",
            "reason": "v03 compat e2e",
            "duration_seconds": 0,
        }))
        .await;
    resp.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["force_disconnect_session"], false);
    assert_eq!(body["disconnected_sessions"], false);

    // 2. 老 grant 请求(无 card_ids/pack_ids) → 202
    let resp = server
        .post("/api/v1/gm/compensation")
        .json(&serde_json::json!({
            "account_id": "p_v03_real_comp",
            "amount": 50,
            "currency": "USD",
            "reason": "v03 compat e2e",
        }))
        .await;
    resp.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["cards_granted"], 0);
    assert_eq!(body["packs_granted"], 0);
    assert!(body["card_ids"].is_array());
    assert_eq!(body["card_ids"].as_array().unwrap().len(), 0);

    // 3. 老 maintenance 请求(无 mode_flags) → 202
    let resp = server
        .post("/api/v1/gm/maintenance")
        .json(&serde_json::json!({
            "enable": true,
            "scope": "single_node",
            "target_id": "node-1",
            "ttl_seconds": 60,
        }))
        .await;
    resp.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["mode_flags"], 0);
    assert_eq!(body["applied_mode_flags"], 0);

    // 4. 老 audit 请求(无 audit_type) → 200 + applied_audit_type="all"
    let resp = server.get("/api/v1/audit/logs?limit=5").await;
    resp.assert_status(StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["audit_type"], "all", "v0.3 兼容: 默认 all");
    assert_eq!(body["applied_audit_type"], "all");
}
