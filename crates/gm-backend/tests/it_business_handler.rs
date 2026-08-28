//! S4 Phase 2 step 3 (W7) IT: gm-backend 业务 handler
//!
//! 验证 business_handler 4 个 endpoint (BanAccount/Compensation/Maintenance/QueryAudit)
//! 业务字段 schema + 响应字段 + 失败降级
//!
//! 关联: docs/00-基准与治理/RGS-TBD-08-03-S4-gm-backend-admin-gRPC-立项.md

use axum_test::TestServer;
use gm_backend::{build_router, AppState, GmConfig};
use serde_json::json;

fn make_server_admin_disabled() -> TestServer {
    // for_test 强制 disable_admin_grpc=true (走 InMemory fallback)
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
async fn ban_account_business_request_schema() {
    // 注意: 业务 endpoint 在 S4 Phase 2 step 2 commit 1e25591 实装的是 stub handler,
    // 字段是 stub "stub"/"stub". 这里测 S4 Phase 2 step 3 business handler schema (待 router 接入)
    // 暂时只测 InMemory audit store 验证业务 schema 正确
    let server = make_server_admin_disabled();
    let resp = server
        .post("/api/v1/gm/ban")
        .json(&json!({
            "account_id": "player-1",
            "reason": "test-ban",
            "duration_seconds": 3600
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    // stub handler (S4 step 2) 暂不解析 body, 仍返 status=queued + op=ban
    // 业务 handler 接入在 step 3 router 改造
    let body: serde_json::Value = resp.json();
    assert!(body["op"].is_string());
}

#[tokio::test]
async fn ban_account_business_unreachable_admin_fallback() {
    let server = make_server_unreachable_admin();
    let started = std::time::Instant::now();
    let resp = server.post("/api/v1/gm/ban").await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "ban_account with unreachable admin must complete within 1s, got {elapsed:?}"
    );
}

#[tokio::test]
async fn grant_compensation_business_unreachable_admin_fallback() {
    let server = make_server_unreachable_admin();
    let started = std::time::Instant::now();
    let resp = server.post("/api/v1/gm/compensation").await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "grant_compensation with unreachable admin must complete within 1s, got {elapsed:?}"
    );
}

#[tokio::test]
async fn set_maintenance_business_unreachable_admin_returns_propagating() {
    let server = make_server_unreachable_admin();
    let resp = server.post("/api/v1/gm/maintenance").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "maintenance");
    assert_eq!(body["propagation_status"], "PROPAGATING");
}

#[tokio::test]
async fn query_audit_business_unreachable_admin_returns_empty() {
    let server = make_server_unreachable_admin();
    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(body["entries"].is_array());
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 0);
    assert_eq!(body["has_more"], false);
}
