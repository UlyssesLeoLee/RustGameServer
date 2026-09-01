//! gm-backend actix-web 重写后最小 UT (2026-09-01)
//!
//! 覆盖:
//! - GmConfig::for_test 配置
//! - JWT issue/verify roundtrip
//! - AppState + ensure_default_admin
//! - circuit_breaker state machine (跟 web 框架无关, 跟旧版一致)
//!
//! 旧版 axum-based tests 已备份到 .git-trash/gm-backend-axum-tests-2026-09-01/
//! 待 actix-web 测试基建 (actix-web::test) 全部迁移后回来

use actix_web::{test, web, App};
use gm_backend::{
    circuit_breaker::{CircuitBreaker, CircuitState},
    health_view, issue_jwt, list_mall_items, login, register_routes, verify_jwt, AdminRecord, AppState,
    GmConfig, LoginRequest,
};
use std::time::Duration;

fn test_state() -> AppState {
    let config = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://127.0.0.1:50055").unwrap();
    AppState::new(config)
}

#[actix_web::test]
async fn gm_config_for_test_ok() {
    let cfg = GmConfig::for_test("127.0.0.1:8443", "127.0.0.1:8081", "http://admin:50055").unwrap();
    assert_eq!(cfg.http_addr.port(), 8443);
    assert!(!cfg.require_jwt);
    assert!(cfg.disable_admin_grpc);
}

#[actix_web::test]
async fn jwt_roundtrip_ok() {
    let secret = "test-secret-123";
    let token = issue_jwt(secret, "admin", vec!["GM_READ".into(), "GM_ADMIN".into()], 3600).unwrap();
    let claims = verify_jwt(secret, &token).unwrap();
    assert_eq!(claims.sub, "admin");
    assert!(claims.roles.contains(&"GM_ADMIN".to_string()));
}

#[actix_web::test]
async fn jwt_wrong_secret_fails() {
    let token = issue_jwt("secret-A", "admin", vec![], 3600).unwrap();
    assert!(verify_jwt("secret-B", &token).is_err());
}

#[tokio::test]
async fn ensure_default_admin_creates_superadmin() {
    let state = test_state();
    state.ensure_default_admin().await;
    let admins = state.admins.lock().unwrap();
    assert_eq!(admins.len(), 1);
    assert_eq!(admins[0].username, "admin");
    assert_eq!(admins[0].role, "superadmin");
    assert!(bcrypt::verify("adminpass", &admins[0].password_hash).unwrap());
}

#[actix_web::test]
async fn login_with_default_admin_returns_token() {
    let state = test_state();
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/gm/login", web::post().to(login)),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/gm/login")
        .set_json(LoginRequest {
            username: "admin".to_string(),
            password: "adminpass".to_string(),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn login_with_wrong_password_returns_401() {
    let state = test_state();
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/gm/login", web::post().to(login)),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/gm/login")
        .set_json(LoginRequest {
            username: "admin".to_string(),
            password: "WRONG".to_string(),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

#[actix_web::test]
async fn health_view_returns_services() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/gm/health_view", web::get().to(health_view)),
    )
    .await;
    let req = test::TestRequest::get().uri("/gm/health_view").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["services"].is_array());
    assert!(body["services"][0]["service_name"].as_str() == Some("admin-service"));
}

#[actix_web::test]
async fn mall_items_empty_initially() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/gm/mall/items", web::get().to(list_mall_items)),
    )
    .await;
    let req = test::TestRequest::get().uri("/gm/mall/items").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn register_routes_compiles() {
    // 验证全部 15+ 端点注册无冲突
    let state = test_state();
    let _app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;
}

#[actix_web::test]
async fn circuit_breaker_state_machine() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(50));
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.try_acquire(), "open state rejects");
    std::thread::sleep(Duration::from_millis(60));
    assert!(cb.try_acquire(), "transitions to half-open after duration");
    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[actix_web::test]
async fn admin_record_clone() {
    let a = AdminRecord {
        username: "x".to_string(),
        password_hash: "h".to_string(),
        role: "admin".to_string(),
    };
    let b = a.clone();
    assert_eq!(a.username, b.username);
}
