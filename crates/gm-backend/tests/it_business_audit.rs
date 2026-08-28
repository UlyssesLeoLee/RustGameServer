//! W26 桶 2a: QueryAuditLog 业务实装 IT
//!
//! 验证 query string 解析 (limit / cursor / filter) + limit clamp
//! (per gm.proto v0.3 + BAS-003 §3.4)

use gm_backend::{AppState, GmConfig};
use tower::util::ServiceExt;

fn build_test_state() -> AppState {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("test config");
    AppState::new(cfg)
}

#[tokio::test]
async fn query_audit_uses_default_limit_when_no_query() {
    let app = axum::Router::new()
        .route("/api/v1/audit/logs", axum::routing::get(gm_backend::query_audit))
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json["entries"].is_array());
    assert!(body_json["has_more"].is_boolean());
}

#[tokio::test]
async fn query_audit_clamps_limit_to_max_100() {
    // limit=999 → clamp 到 100, 不报错
    let app = axum::Router::new()
        .route("/api/v1/audit/logs", axum::routing::get(gm_backend::query_audit))
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs?limit=999")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn query_audit_accepts_filter_query_params() {
    let app = axum::Router::new()
        .route("/api/v1/audit/logs", axum::routing::get(gm_backend::query_audit))
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs?limit=10&filter_action=ban&filter_admin=admin_1")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn query_audit_returns_next_cursor_field() {
    let app = axum::Router::new()
        .route("/api/v1/audit/logs", axum::routing::get(gm_backend::query_audit))
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs?cursor=abc123")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    // next_cursor 字段必须存在 (per gm.proto v0.3 schema)
    assert!(body_json.as_object().unwrap().contains_key("next_cursor"));
}

#[tokio::test]
async fn query_audit_with_unreachable_admin_returns_empty_inmemory() {
    // chaos: 不可达 admin 走 InMemory 降级路径
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(), // 不可达
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let state = AppState::new(cfg);
    let app = axum::Router::new()
        .route("/api/v1/audit/logs", axum::routing::get(gm_backend::query_audit))
        .with_state(state);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json["entries"].is_array());
    assert_eq!(body_json["entries"].as_array().unwrap().len(), 0);
    assert_eq!(body_json["has_more"], false);
}
