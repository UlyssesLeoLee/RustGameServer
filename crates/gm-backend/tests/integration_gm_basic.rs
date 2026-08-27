//! gm-backend 集成测试 — Router + 5 个 GM endpoint + 2 个 health endpoint
//!
//! 用 axum-test 启动 in-process server,验证:
//! - /healthz 返回 200 + service=gm-backend
//! - /readyz 返回 200 + status=ready
//! - /api/v1/gm/health/view 返回 admin_endpoint 配置值
//! - /api/v1/gm/ban /compensation /maintenance 返回 202 + queued
//! - /api/v1/audit/logs 返回 200 + 空 items
//!
//! 跨 endpoint 验证 config 隔离(每个测试独立 AppState,无共享)

use axum_test::TestServer;
use gm_backend::{build_health_router, build_router, AppState, GmConfig};

fn make_test_server() -> TestServer {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055").unwrap();
    let state = AppState::new(cfg);
    let app = build_router(state);
    TestServer::new(app).expect("test server should bind")
}

fn make_health_server() -> TestServer {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055").unwrap();
    let state = AppState::new(cfg);
    let app = build_health_router(state);
    TestServer::new(app).expect("health server should bind")
}

#[tokio::test]
async fn healthz_returns_ok_with_service_name() {
    let server = make_test_server();
    let resp = server.get("/healthz").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "gm-backend");
}

#[tokio::test]
async fn readyz_returns_ready() {
    let server = make_test_server();
    let resp = server.get("/readyz").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "ready");
    assert_eq!(body["service"], "gm-backend");
}

#[tokio::test]
async fn health_router_also_exposes_healthz() {
    // 验证 build_health_router 独立 router(8081 探针专用)
    let server = make_health_server();
    let resp = server.get("/healthz").await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn health_view_returns_admin_endpoint_from_config() {
    let server = make_test_server();
    let resp = server.get("/api/v1/gm/health/view").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["service"], "gm-backend");
    assert_eq!(body["admin_endpoint"], "http://admin-staging:50055");
    assert_eq!(body["mode"], "stub-ok");
}

#[tokio::test]
async fn ban_account_returns_202_queued() {
    let server = make_test_server();
    let resp = server.post("/api/v1/gm/ban").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "queued");
    assert_eq!(body["op"], "ban");
}

#[tokio::test]
async fn grant_compensation_returns_202_queued() {
    let server = make_test_server();
    let resp = server.post("/api/v1/gm/compensation").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "queued");
    assert_eq!(body["op"], "compensation");
}

#[tokio::test]
async fn set_maintenance_returns_202_queued() {
    let server = make_test_server();
    let resp = server.post("/api/v1/gm/maintenance").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "queued");
    assert_eq!(body["op"], "maintenance");
}

#[tokio::test]
async fn query_audit_returns_empty_items_stub() {
    let server = make_test_server();
    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["items"].is_array());
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["next"], "stub");
}

#[tokio::test]
async fn health_router_does_not_expose_gm_endpoints() {
    // 8081 health-only router 不应暴露 /api/v1/gm/*
    // 安全边界:探针端口不应有业务 endpoint
    let server = make_health_server();
    let resp = server.get("/api/v1/gm/health/view").await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn main_router_does_not_accept_post_on_get_endpoints() {
    // 405 路由不匹配
    let server = make_test_server();
    let resp = server.post("/healthz").await;
    resp.assert_status(axum::http::StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn main_router_does_not_accept_get_on_post_endpoints() {
    let server = make_test_server();
    let resp = server.get("/api/v1/gm/ban").await;
    resp.assert_status(axum::http::StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let server = make_test_server();
    let resp = server.get("/api/v1/gm/nonexistent").await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
