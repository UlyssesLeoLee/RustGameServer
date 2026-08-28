//! S4 Phase 2 step 2 IT: gm-backend 4 endpoint 调 admin-service gRPC
//!
//! 验证 ban_account / grant_compensation / set_maintenance / query_audit
//! 4 个 handler 调 admin-service gRPC + 失败降级 InMemory
//!
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md
//!
//! 测试策略: 用 axum-test + disable_admin_grpc=true 测 InMemory fallback (已有 integration_gm_basic),
//! 用 disable_admin_grpc=false + 不可达 admin-service 测 500ms timeout 降级
//! (无需 mock tonic server, 真实场景: admin-service 不可达时 handler 应仍能响应)

use axum_test::TestServer;
use gm_backend::{build_router, AppState, GmConfig};
use serde_json::json;
use std::time::Duration;

fn make_server_with_admin_grpc_enabled() -> TestServer {
    // 不可达 admin endpoint (1 端口), disable_admin_grpc=false
    // → connect_lazy 成功, 但 health_check 500ms timeout 返 Err
    // → handler 应降级到 InMemory + 返 202
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(), // 不可达
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let state = AppState::new(cfg);
    let app = build_router(state);
    TestServer::new(app).expect("test server should bind")
}

#[tokio::test]
async fn ban_account_with_unreachable_admin_returns_202() {
    // admin-service 不可达 → handler 仍 返 202 (降级 InMemory)
    // W26 (2026-08-29) 桶 2a: 发送合法 body
    let server = make_server_with_admin_grpc_enabled();
    let started = std::time::Instant::now();
    let resp = server
        .post("/api/v1/gm/ban")
        .json(&json!({"account_id": "u-ban", "reason": "u", "duration_seconds": 0}))
        .await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    // 500ms timeout + jitter → 总时长 < 1s
    assert!(
        elapsed < Duration::from_secs(1),
        "ban_account with unreachable admin must complete within 1s, got {elapsed:?}"
    );
}

#[tokio::test]
async fn grant_compensation_with_unreachable_admin_returns_202() {
    // W26 (2026-08-29) 桶 2a: 发送合法 body
    let server = make_server_with_admin_grpc_enabled();
    let started = std::time::Instant::now();
    let resp = server
        .post("/api/v1/gm/compensation")
        .json(&json!({"account_id": "u-comp", "amount": 10, "currency": "USD", "reason": "u"}))
        .await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    assert!(
        elapsed < Duration::from_secs(1),
        "grant_compensation with unreachable admin must complete within 1s, got {elapsed:?}"
    );
}

#[tokio::test]
async fn set_maintenance_with_unreachable_admin_returns_202_with_propagating() {
    // set_maintenance 失败降级: propagation_status 默认 PROPAGATING
    // W26 (2026-08-29) 桶 2a: 发送合法 body
    let server = make_server_with_admin_grpc_enabled();
    let started = std::time::Instant::now();
    let resp = server
        .post("/api/v1/gm/maintenance")
        .json(
            &json!({"enable": true, "scope": "cluster", "target_id": "cluster", "ttl_seconds": 0}),
        )
        .await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    assert!(
        elapsed < Duration::from_secs(1),
        "set_maintenance with unreachable admin must complete within 1s, got {elapsed:?}"
    );
    let body: serde_json::Value = resp.json();
    assert_eq!(body["op"], "maintenance");
    assert_eq!(
        body["propagation_status"], "PROPAGATING",
        "unreachable admin must default propagation_status=PROPAGATING"
    );
}

#[tokio::test]
async fn query_audit_with_unreachable_admin_returns_200_with_empty_entries() {
    // query_audit 调 admin-service gRPC 失败, 降级 InMemory (空 entries)
    let server = make_server_with_admin_grpc_enabled();
    let started = std::time::Instant::now();
    let resp = server.get("/api/v1/audit/logs").await;
    let elapsed = started.elapsed();
    resp.assert_status(axum::http::StatusCode::OK);
    assert!(
        elapsed < Duration::from_secs(1),
        "query_audit with unreachable admin must complete within 1s, got {elapsed:?}"
    );
    let body: serde_json::Value = resp.json();
    assert!(
        body["entries"].is_array(),
        "entries must be array even on fallback"
    );
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(
        entries.len(),
        0,
        "empty InMemory must return 0 entries on fallback"
    );
    assert_eq!(body["has_more"], false);
}
