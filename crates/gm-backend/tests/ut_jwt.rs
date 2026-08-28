//! TBD-08-01 JWT middleware 实装 + 测试
//!
//! per 2026-08-28 ut 实施 + RGS-BAS-003 §2.1 RBAC 链路:
//! - JWT 签发 (issue_jwt) + 验证 (verify_jwt)
//! - axum middleware:从 Authorization: Bearer <token> 提取
//! - require_jwt = false (dev 模式跳过,默认;生产设为 true)
//!
//! 5 测试:
//! 1. issue_jwt + verify_jwt roundtrip OK
//! 2. verify_jwt 错误 secret 失败
//! 3. verify_jwt 过期 token 失败
//! 4. middleware require_jwt=false 跳过验证,200 OK
//! 5. middleware require_jwt=true 无 token 401
//! 6. middleware require_jwt=true 有效 token 200 OK
//! 7. middleware require_jwt=true 错误 token 401

use axum::body::Body;
use axum::http::{Request, StatusCode};
use gm_backend::{issue_jwt, verify_jwt, AppState, GmConfig};
use std::time::Duration;
use tower::ServiceExt;

fn test_state(require_jwt: bool) -> AppState {
    let cfg = GmConfig {
        http_addr: "0.0.0.0:8443".parse().unwrap(),
        health_addr: "0.0.0.0:8081".parse().unwrap(),
        admin_grpc_endpoint: "http://admin:50055".to_string(),
        jwt_secret: "test-secret".to_string(),
        require_jwt,
    };
    AppState::new(cfg)
}

#[test]
fn issue_and_verify_jwt_roundtrip() {
    let token = issue_jwt("test-secret", "admin01", vec!["gm".to_string()], 3600)
        .expect("issue_jwt ok");
    let claims = verify_jwt("test-secret", &token).expect("verify_jwt ok");
    assert_eq!(claims.sub, "admin01");
    assert_eq!(claims.roles, vec!["gm"]);
}

#[test]
fn verify_jwt_wrong_secret_fails() {
    let token = issue_jwt("secret-a", "admin01", vec!["gm".to_string()], 3600).unwrap();
    let r = verify_jwt("secret-b", &token);
    assert!(r.is_err(), "verify with wrong secret must fail");
}

#[test]
fn verify_jwt_expired_token_fails() {
    // TTL = -120s (超过 jsonwebtoken 默认 leeway=60s)
    let token = issue_jwt("test-secret", "admin01", vec!["gm".to_string()], -120).unwrap();
    let r = verify_jwt("test-secret", &token);
    assert!(r.is_err(), "expired token must fail");
}

#[tokio::test]
async fn middleware_require_jwt_false_skips_validation() {
    let state = test_state(false);
    let app = gm_backend::build_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn middleware_require_jwt_true_no_token_returns_401() {
    let state = test_state(true);
    let app = gm_backend::build_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn middleware_require_jwt_true_valid_token_returns_200() {
    let state = test_state(true);
    let token = issue_jwt(
        "test-secret",
        "admin01",
        vec!["gm".to_string()],
        3600,
    )
    .unwrap();
    let app = gm_backend::build_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn middleware_require_jwt_true_invalid_token_returns_401() {
    let state = test_state(true);
    let app = gm_backend::build_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/gm/health/view")
        .header("Authorization", "Bearer not-a-jwt")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn middleware_does_not_apply_to_healthz() {
    // /healthz 必须在 build_health_router 上,不走 JWT(探针免拒)
    // 这里用 build_router 测 /healthz 也应该不挂 JWT(因为 middleware 只挂在 /api/v1/*)
    let state = test_state(true);
    let app = gm_backend::build_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[allow(dead_code)]
fn _unused(_: Duration) {}
