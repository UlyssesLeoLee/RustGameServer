//! W26 桶 2a: GrantCompensation 业务实装 IT
//!
//! 验证 amount > 0 / currency 长度 3-4 / 缺字段校验
//! (per gm.proto v0.3 + BAS-003 §3.1)

use gm_backend::{AppState, GmConfig};
use tower::util::ServiceExt;

fn build_test_state() -> AppState {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("test config");
    AppState::new(cfg)
}

#[tokio::test]
async fn compensation_parses_body_with_valid_amount() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player_comp_001",
        "amount": 100,
        "currency": "USD",
        "reason": "compensation for outage",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["account_id"], "player_comp_001");
    assert_eq!(body_json["amount"], 100);
    assert_eq!(body_json["currency"], "USD");
}

#[tokio::test]
async fn compensation_rejects_zero_amount() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player",
        "amount": 0,
        "currency": "USD",
        "reason": "test",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"], "invalid_amount");
}

#[tokio::test]
async fn compensation_rejects_invalid_currency_length() {
    // currency="US" (2 字符) → 应 400 invalid_currency
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player",
        "amount": 50,
        "currency": "US",
        "reason": "test",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"], "invalid_currency");
}

#[tokio::test]
async fn compensation_accepts_4_char_currency() {
    // currency="GOLD" (4 字符) → 应 202
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player_gold",
        "amount": 200,
        "currency": "GOLD",
        "reason": "gold reward",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::ACCEPTED);
}

#[tokio::test]
async fn compensation_rejects_empty_account_id() {
    // account_id="" → 应 400 missing_account_id
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "",
        "amount": 50,
        "currency": "USD",
        "reason": "test",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"], "missing_account_id");
}
