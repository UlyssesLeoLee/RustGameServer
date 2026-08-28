//! W26 桶 2a: SetMaintenance 业务实装 IT
//!
//! 验证 scope 范围 cluster/domain/single_node + propagation_status
//! (per gm.proto v0.3 + BAS-003 §3.3)

use gm_backend::{AppState, GmConfig};
use tower::util::ServiceExt;

fn build_test_state() -> AppState {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("test config");
    AppState::new(cfg)
}

#[tokio::test]
async fn maintenance_parses_body_with_valid_scope() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/maintenance",
            axum::routing::post(gm_backend::set_maintenance),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "enable": true,
        "scope": "domain",
        "target_id": "domain_economy_01",
        "ttl_seconds": 600,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/maintenance")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["scope"], "domain");
    assert_eq!(body_json["target_id"], "domain_economy_01");
    assert_eq!(body_json["enable"], true);
    assert!(body_json["propagation_status"].is_string());
}

#[tokio::test]
async fn maintenance_rejects_invalid_scope() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/maintenance",
            axum::routing::post(gm_backend::set_maintenance),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "enable": true,
        "scope": "invalid_scope",
        "target_id": "x",
        "ttl_seconds": 0,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/maintenance")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"], "invalid_scope");
}

#[tokio::test]
async fn maintenance_accepts_single_node_scope() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/maintenance",
            axum::routing::post(gm_backend::set_maintenance),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "enable": false,
        "scope": "single_node",
        "target_id": "node-3",
        "ttl_seconds": 60,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/maintenance")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::ACCEPTED);
}

#[tokio::test]
async fn maintenance_rejects_negative_ttl() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/maintenance",
            axum::routing::post(gm_backend::set_maintenance),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "enable": true,
        "scope": "cluster",
        "target_id": "cluster",
        "ttl_seconds": -1,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/maintenance")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"], "invalid_ttl");
}
