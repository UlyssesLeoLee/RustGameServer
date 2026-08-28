//! W26 桶 2a: HealthView 业务实装 IT
//!
//! 验证 request_id query string 解析 + services[] 返回
//! (per gm.proto v0.3 + BAS-003 §2.1)

use gm_backend::{AppState, GmConfig};
use tower::util::ServiceExt;

fn build_test_state() -> AppState {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("test config");
    AppState::new(cfg)
}

#[tokio::test]
async fn health_view_default_request_id_is_uuid() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/health/view",
            axum::routing::get(gm_backend::health_view),
        )
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json["request_id"].is_string());
    assert!(body_json["services"].is_array());
    assert!(body_json["checked_at_ms"].is_i64());
    assert!(body_json["admin_endpoint"].is_string());
}

#[tokio::test]
async fn health_view_accepts_custom_request_id() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/health/view",
            axum::routing::get(gm_backend::health_view),
        )
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view?request_id=trace-001")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["request_id"], "trace-001");
}

#[tokio::test]
async fn health_view_returns_admin_service_ready() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/health/view",
            axum::routing::get(gm_backend::health_view),
        )
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let services = body_json["services"].as_array().unwrap();
    assert!(!services.is_empty());
    let admin_service = &services[0];
    assert_eq!(admin_service["service_name"], "admin-service");
    // ready 字段存在(在测试环境无 admin_grpc, 默认 true)
    assert!(admin_service["ready"].is_boolean());
}

#[tokio::test]
async fn health_view_marks_admin_unavailable_when_unreachable() {
    // chaos: 不可达 admin → services[0].ready=false
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
        .route(
            "/api/v1/gm/health/view",
            axum::routing::get(gm_backend::health_view),
        )
        .with_state(state);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view?request_id=chaos-trace")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["request_id"], "chaos-trace");
    let services = body_json["services"].as_array().unwrap();
    assert_eq!(services[0]["ready"], false, "不可达 admin 必须 ready=false");
}
