//! W19 (2026-08-28) Chaos test: admin-service 503 / 不可达时 gm-backend 仍能服务
//!
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md §3.1
//!
//! 失败降级: handler 内部若 admin-service 不可达, 返 InMemory fallback
//! 不让 8081 health probe 误判 gm-backend 自己挂

use axum_test::TestServer;
use gm_backend::{build_router, AppState, GmConfig};
use std::time::Duration;

fn make_server_unreachable_admin() -> TestServer {
    // admin-service 不可达 (端口 1)
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
async fn chaos_health_endpoint_survives_admin_outage() {
    // 关键: 8081 /healthz 探针不应因 admin-service 不可达而 fail
    // (这是 W19 关键: gm-backend 自身健康, 业务降级 admin 不可达)
    let server = make_server_unreachable_admin();
    let resp = server.get("/healthz").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "gm-backend");
}

#[tokio::test]
async fn chaos_ready_endpoint_survives_admin_outage() {
    let server = make_server_unreachable_admin();
    let resp = server.get("/readyz").await;
    resp.assert_status(axum::http::StatusCode::OK);
}

#[tokio::test]
async fn chaos_health_view_marks_admin_unavailable() {
    // HealthView 调 admin gRPC 失败 → services[0].ready=false
    // (这是 chaos 行为: 业务降级, 但 gm-backend 自身仍 200)
    let server = make_server_unreachable_admin();
    let resp = server.get("/api/v1/gm/health/view").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    let services = body["services"].as_array().unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0]["service_name"], "admin-service");
    assert_eq!(
        services[0]["ready"], false,
        "admin-service should be marked unavailable during chaos"
    );
}

#[tokio::test]
async fn chaos_ban_account_returns_202_insteadof_503() {
    // 即使 admin-service 完全不可达, gm-backend ban 仍 202 + 降级 InMemory
    // (per S4 Phase 2 step 2 设计: 业务不中断, audit 写本地)
    let server = make_server_unreachable_admin();
    let started = std::time::Instant::now();
    let resp = server.post("/api/v1/gm/ban").await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    // 500ms timeout, 总时长 < 1s
    assert!(
        elapsed < Duration::from_secs(1),
        "chaos ban_account must complete within 1s, got {elapsed:?}"
    );
}

#[tokio::test]
async fn chaos_grant_compensation_returns_202_insteadof_503() {
    let server = make_server_unreachable_admin();
    let resp = server.post("/api/v1/gm/compensation").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
}

#[tokio::test]
async fn chaos_set_maintenance_returns_202_with_propagating() {
    let server = make_server_unreachable_admin();
    let resp = server.post("/api/v1/gm/maintenance").await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "maintenance");
    assert_eq!(
        body["propagation_status"], "PROPAGATING",
        "fallback propagation_status must be PROPAGATING (not CONVERGED)"
    );
}

#[tokio::test]
async fn chaos_query_audit_returns_200_with_empty_inmemory() {
    let server = make_server_unreachable_admin();
    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(body["entries"].is_array());
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 0);
}

#[tokio::test]
async fn chaos_5_serial_requests_all_complete_under_5s() {
    // 5 sequential ban requests, 全部 5s 内完成 (admin 不可达)
    // TestServer 不能 clone, 改串行跑
    use serde_json::json;
    let server = make_server_unreachable_admin();
    let started = std::time::Instant::now();
    for i in 0..5 {
        let resp = server
            .post("/api/v1/gm/ban")
            .json(&json!({"account_id": format!("chaos-{i}"), "reason": "chaos", "duration_seconds": 0}))
            .await;
        resp.assert_status(axum::http::StatusCode::ACCEPTED);
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(5),
        "5 serial ban requests must complete within 5s, got {elapsed:?}"
    );
}
