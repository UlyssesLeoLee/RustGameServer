//! W11 (2026-08-28) IT: gm-backend 业务路由 v2 (4 endpoint 真实 body 解析)
//!
//! 验证 /api/v2/gm/ban, /api/v2/gm/compensation, /api/v2/gm/maintenance, /api/v2/audit/logs
//! 4 个 endpoint 解析真实 request body, 调 admin-service gRPC, 失败降级 InMemory
//!
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md

use axum_test::TestServer;
use gm_backend::{build_router, AppState, GmConfig};
use serde_json::json;

fn make_server_admin_disabled() -> TestServer {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin:50055").unwrap();
    let state = AppState::new(cfg);
    TestServer::new(build_router(state)).expect("test server")
}

fn make_server_unreachable_admin() -> TestServer {
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let state = AppState::new(cfg);
    TestServer::new(build_router(state)).expect("test server")
}

#[tokio::test]
async fn v2_ban_account_uses_real_body_fields() {
    // v2 路由解析真实 body, 不再 stub
    let server = make_server_admin_disabled();
    let resp = server
        .post("/api/v2/gm/ban")
        .json(&json!({
            "account_id": "player-v2-test",
            "reason": "test-ban-v2",
            "duration_seconds": 7200
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "ban");
    // admin_grpc=None (test) → status=queued
    assert!(body["status"].is_string());
    assert!(body["accepted_at_ms"].is_number());
}

#[tokio::test]
async fn v2_grant_compensation_uses_real_body_fields() {
    let server = make_server_admin_disabled();
    let resp = server
        .post("/api/v2/gm/compensation")
        .json(&json!({
            "account_id": "player-comp",
            "amount": 1000,
            "currency": "gold",
            "reason": "test-comp-v2"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "compensation");
    assert!(body["accepted_at_ms"].is_number());
}

#[tokio::test]
async fn v2_set_maintenance_returns_propagation_status() {
    let server = make_server_admin_disabled();
    let resp = server
        .post("/api/v2/gm/maintenance")
        .json(&json!({
            "enable": true,
            "scope": "cluster",
            "target_id": "cluster-v2",
            "ttl_seconds": 3600
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "maintenance");
    // admin_grpc=None → 默认 PROPAGATING
    assert_eq!(body["propagation_status"], "PROPAGATING");
}

#[tokio::test]
async fn v2_query_audit_log_returns_empty_inmemory() {
    let server = make_server_admin_disabled();
    let resp = server
        .post("/api/v2/audit/logs")
        .json(&json!({
            "limit": 5,
            "cursor": "",
            "filter_admin": "",
            "filter_action": ""
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "audit");
    assert!(body["entries"].is_array());
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 0);
    assert_eq!(body["has_more"], false);
}

#[tokio::test]
async fn v2_ban_account_with_unreachable_admin_completes_within_1s() {
    let server = make_server_unreachable_admin();
    let started = std::time::Instant::now();
    let resp = server
        .post("/api/v2/gm/ban")
        .json(&json!({
            "account_id": "player-fallback",
            "reason": "fallback-test",
            "duration_seconds": 0
        }))
        .await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "v2 ban_account with unreachable admin must complete within 1s, got {elapsed:?}"
    );
}

#[tokio::test]
async fn v1_routes_still_work_for_backward_compat() {
    // v1 路由保留, 旧测试不应 break
    let server = make_server_admin_disabled();
    let resp = server.post("/api/v1/gm/ban").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let resp = server.post("/api/v1/gm/compensation").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let resp = server.post("/api/v1/gm/maintenance").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status(axum::http::StatusCode::OK);
}
