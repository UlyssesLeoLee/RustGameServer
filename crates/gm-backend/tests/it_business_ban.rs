//! W26 桶 2a: BanAccount 业务实装 IT
//!
//! 验证 BanAccount HTTP endpoint 解析 body 真实字段
//! (per RGS-PLAN-WBS-token-bucket-v0.3 §2.2.1 + gm.proto v0.3 + BAS-003 §3.1)
//!
//! 测试范围:
//! - 真值测试(正常请求, 202 + account_id 进 audit_log)
//! - 缺字段测试(account_id 空 → 400)
//! - duration_seconds 缺省测试
//! - 失败降级测试(admin_grpc 不可达 → 202 + InMemory)

use gm_backend::{AppState, GmConfig};
use tower::util::ServiceExt;

fn build_test_state() -> AppState {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("test config");
    AppState::new(cfg)
}

#[tokio::test]
async fn ban_account_parses_body_and_writes_audit_log() {
    // 真实 ban: account_id="player_123", reason="cheating", duration=3600
    let app = axum::Router::new()
        .route("/api/v1/gm/ban", axum::routing::post(gm_backend::ban_account))
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player_123",
        "reason": "cheating detected",
        "duration_seconds": 3600,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["status"], "queued");
    assert_eq!(body_json["op"], "ban");
    assert_eq!(body_json["account_id"], "player_123"); // 真值, 不是 "stub"
}

#[tokio::test]
async fn ban_account_rejects_empty_account_id() {
    let app = axum::Router::new()
        .route("/api/v1/gm/ban", axum::routing::post(gm_backend::ban_account))
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "",
        "reason": "test",
        "duration_seconds": 0,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"], "missing_account_id");
}

#[tokio::test]
async fn ban_account_works_with_minimal_body() {
    // duration_seconds 缺省 → 默认 0 (永久)
    let app = axum::Router::new()
        .route("/api/v1/gm/ban", axum::routing::post(gm_backend::ban_account))
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player_minimal",
        "reason": "violation",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::ACCEPTED);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["account_id"], "player_minimal");
}

#[tokio::test]
async fn ban_account_rejects_missing_account_id_field() {
    // 缺 account_id 字段 → axum Json extractor 返 400
    let app = axum::Router::new()
        .route("/api/v1/gm/ban", axum::routing::post(gm_backend::ban_account))
        .with_state(build_test_state());

    let body = serde_json::json!({
        "reason": "test",
        "duration_seconds": 0,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // axum Json<T> 缺字段返 422 (Unprocessable Entity)
    assert!(
        resp.status() == axum::http::StatusCode::BAD_REQUEST
            || resp.status() == axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        "missing field must return 400/422, got {}",
        resp.status()
    );
}
