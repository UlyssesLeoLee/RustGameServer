//! W36 (2026-08-30) gm.proto v0.4 实际集成 UT
//!
//! 验证 gm.proto v0.4 新增字段 (per DTL-038 §4.3 + DEC-038-07):
//! - BanAccount.force_disconnect_session / disconnected_sessions
//! - GrantCompensation.card_ids / pack_ids / cards_granted / packs_granted
//! - SetMaintenance.mode_flags / applied_mode_flags
//! - QueryAuditLog.audit_type / applied_audit_type
//!
//! 15 UT 覆盖:
//! 1. ban_account_force_disconnect_session_true
//! 2. ban_account_force_disconnect_session_false
//! 3. ban_account_disconnected_sessions_echoed_from_admin_response
//! 4. grant_compensation_card_ids_passes_through
//! 5. grant_compensation_packs_granted_echo
//! 6. set_maintenance_mode_flags_passes_through
//! 7. query_audit_log_audit_type_filter_trade
//! 8. query_audit_log_applied_audit_type_echo
//! 9. audit_log_entry_audit_type_field_in_response
//! 10. v03_compat_ban_no_new_fields
//! 11. v03_compat_compensation_no_new_fields
//! 12. v03_compat_maintenance_no_mode_flags
//! 13. v03_compat_audit_no_audit_type_query
//! 14. parse_audit_type_returns_enum_for_valid_strings
//! 15. parse_audit_type_returns_none_for_invalid_strings
//!
//! 关联: RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07

use axum::http::StatusCode;
use axum_test::TestServer;
use gm_backend::business_handler::{audit_type_to_str, parse_audit_type};
use gm_backend::{AppState, GmConfig};
use tower::util::ServiceExt;

// v0.4 AuditType 枚举值(per gm.proto v0.4 + admin.proto)
// 1=All, 2=Trade, 3=Gacha, 4=Match, 5=Compensation
const AUDIT_TYPE_ALL: i32 = 1;
const AUDIT_TYPE_TRADE: i32 = 2;
const AUDIT_TYPE_GACHA: i32 = 3;
const AUDIT_TYPE_MATCH: i32 = 4;
const AUDIT_TYPE_COMPENSATION: i32 = 5;

// ============================================================================
// Test helpers
// ============================================================================

fn build_test_state() -> AppState {
    // 走 InMemory 降级路径(无 admin gRPC 注入),验证 v0.4 字段填值
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("test config");
    AppState::new(cfg)
}

// ============================================================================
// 1. BanAccount: force_disconnect_session=true
// ============================================================================

#[tokio::test]
async fn ban_account_force_disconnect_session_true() {
    // v0.4 增量: force_disconnect_session=true → 响应 disconnected_sessions 应 echo true
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/ban",
            axum::routing::post(gm_backend::ban_account),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player_force",
        "reason": "force disconnect test",
        "duration_seconds": 3600,
        "force_disconnect_session": true,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["force_disconnect_session"], true);
    assert_eq!(
        body_json["disconnected_sessions"], true,
        "disconnected_sessions must echo force_disconnect_session=true (降级路径)"
    );
}

// ============================================================================
// 2. BanAccount: force_disconnect_session=false
// ============================================================================

#[tokio::test]
async fn ban_account_force_disconnect_session_false() {
    // v0.4 增量: force_disconnect_session=false → 响应 disconnected_sessions 应 echo false
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/ban",
            axum::routing::post(gm_backend::ban_account),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "player_no_force",
        "reason": "soft ban test",
        "duration_seconds": 600,
        "force_disconnect_session": false,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["force_disconnect_session"], false);
    assert_eq!(body_json["disconnected_sessions"], false);
}

// ============================================================================
// 3. BanAccount: disconnected_sessions 字段填值(从 admin-service 响应)
// ============================================================================

#[tokio::test]
async fn ban_account_disconnected_sessions_echoed_from_admin_response() {
    // 降级路径(InMemory)下 disconnected_sessions = body.force_disconnect_session
    // 注: 真实 admin-service 调通时, disconnected_sessions = resp.disconnected_sessions
    //     (per admin.gm_handlers ban_account → 写 audit_log + 返 default false)
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/ban",
            axum::routing::post(gm_backend::ban_account),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "p_echo",
        "reason": "echo test",
        "force_disconnect_session": true,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    // 字段必须存在(v0.4 schema)
    assert!(
        body_json.as_object().unwrap().contains_key("disconnected_sessions"),
        "BanAccount response must include v0.4 disconnected_sessions field"
    );
    assert_eq!(body_json["disconnected_sessions"], true);
}

// ============================================================================
// 4. GrantCompensation: card_ids 字段透传
// ============================================================================

#[tokio::test]
async fn grant_compensation_card_ids_passes_through() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "p_cards",
        "amount": 100,
        "currency": "USD",
        "reason": "card comp",
        "card_ids": ["card_001", "card_002", "card_003"],
        "pack_ids": [],
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 2048).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let card_ids = body_json["card_ids"].as_array().expect("card_ids must be array");
    assert_eq!(card_ids.len(), 3);
    assert_eq!(card_ids[0], "card_001");
    assert_eq!(card_ids[1], "card_002");
    assert_eq!(card_ids[2], "card_003");
    // 降级路径 cards_granted = body.card_ids.len() = 3
    assert_eq!(body_json["cards_granted"], 3);
}

// ============================================================================
// 5. GrantCompensation: packs_granted 字段填值
// ============================================================================

#[tokio::test]
async fn grant_compensation_packs_granted_echo() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "p_packs",
        "amount": 50,
        "currency": "GOLD",
        "reason": "pack comp",
        "card_ids": [],
        "pack_ids": ["pack_a", "pack_b"],
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 2048).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["packs_granted"], 2, "降级路径 packs_granted = body.pack_ids.len()");
    let pack_ids = body_json["pack_ids"].as_array().expect("pack_ids must be array");
    assert_eq!(pack_ids.len(), 2);
    assert_eq!(pack_ids[0], "pack_a");
    assert_eq!(pack_ids[1], "pack_b");
    // cards_granted 应为 0(card_ids 空)
    assert_eq!(body_json["cards_granted"], 0);
}

// ============================================================================
// 6. SetMaintenance: mode_flags 透传
// ============================================================================

#[tokio::test]
async fn set_maintenance_mode_flags_passes_through() {
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/maintenance",
            axum::routing::post(gm_backend::set_maintenance),
        )
        .with_state(build_test_state());

    // 0b1011 = 11: bit0 ladder_freeze + bit1 trade_freeze + bit3 match_freeze
    let body = serde_json::json!({
        "enable": true,
        "scope": "domain",
        "target_id": "domain_economy_01",
        "ttl_seconds": 600,
        "mode_flags": 11,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/maintenance")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["mode_flags"], 11);
    // 降级路径 applied_mode_flags = body.mode_flags
    assert_eq!(body_json["applied_mode_flags"], 11);
    assert!(body_json["propagation_status"].is_string());
}

// ============================================================================
// 7. QueryAuditLog: audit_type 过滤
// ============================================================================

#[tokio::test]
async fn query_audit_log_audit_type_filter_trade() {
    // ?audit_type=trade → 透传给 admin-service, applied_audit_type echo "trade"
    let app = axum::Router::new()
        .route(
            "/api/v1/audit/logs",
            axum::routing::get(gm_backend::query_audit),
        )
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs?audit_type=trade&limit=5")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["audit_type"], "trade");
    assert_eq!(body_json["applied_audit_type"], "trade");
}

// ============================================================================
// 8. QueryAuditLog: applied_audit_type echo
// ============================================================================

#[tokio::test]
async fn query_audit_log_applied_audit_type_echo() {
    // ?audit_type=gacha → echo "gacha"
    let app = axum::Router::new()
        .route(
            "/api/v1/audit/logs",
            axum::routing::get(gm_backend::query_audit),
        )
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs?audit_type=gacha")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["audit_type"], "gacha");
    assert_eq!(body_json["applied_audit_type"], "gacha");
    assert!(body_json["entries"].is_array());
}

// ============================================================================
// 9. AuditLogEntry: audit_type 字段在响应中
// ============================================================================

#[tokio::test]
async fn audit_log_entry_audit_type_field_in_response() {
    // 预加载 1 条 audit entry, 调 query 验证响应 entries[].audit_type 字段存在
    let cfg = GmConfig::for_test("0.0.0.0:0", "0.0.0.0:0", "http://admin:50055").unwrap();
    let store = gm_backend::InMemoryAuditStore::new();
    store.append(gm_backend::AuditLogEntry {
        log_id: "v04-1".to_string(),
        admin_id: "test-admin".to_string(),
        action: "ban".to_string(),
        target_id: "v04-target".to_string(),
        occurred_at_ms: 1_700_000_000_000,
    });
    let store_dyn: std::sync::Arc<dyn gm_backend::AuditStore> = std::sync::Arc::new(store);
    let state = AppState::with_audit_store(cfg, store_dyn);
    let app = gm_backend::build_router(state);
    let server = TestServer::new(app).expect("test server");

    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status(StatusCode::OK);
    let body: serde_json::Value = resp.json();
    let entries = body["entries"].as_array().expect("entries must be array");
    assert_eq!(entries.len(), 1);
    // v0.4 字段: 降级路径 audit_type = "all"
    assert!(
        entries[0].as_object().unwrap().contains_key("audit_type"),
        "AuditLogEntry response must include v0.4 audit_type field"
    );
    assert_eq!(entries[0]["audit_type"], "all");
    assert_eq!(body["audit_type"], "all");
    assert_eq!(body["applied_audit_type"], "all");
}

// ============================================================================
// 10. v0.3 兼容: 老 BanAccount 请求无 force_disconnect_session
// ============================================================================

#[tokio::test]
async fn v03_compat_ban_no_new_fields() {
    // v0.3 老请求: 不传 force_disconnect_session → 默认 false (0 破坏)
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/ban",
            axum::routing::post(gm_backend::ban_account),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "p_v03_ban",
        "reason": "v03 compat test",
        "duration_seconds": 60,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/ban")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["force_disconnect_session"], false);
    assert_eq!(body_json["disconnected_sessions"], false);
}

// ============================================================================
// 11. v0.3 兼容: 老 GrantCompensation 请求无 card_ids/pack_ids
// ============================================================================

#[tokio::test]
async fn v03_compat_compensation_no_new_fields() {
    // v0.3 老请求: 不传 card_ids/pack_ids → 默认空 vec (0 破坏)
    let app = axum::Router::new()
        .route(
            "/api/v1/gm/compensation",
            axum::routing::post(gm_backend::grant_compensation),
        )
        .with_state(build_test_state());

    let body = serde_json::json!({
        "account_id": "p_v03_comp",
        "amount": 50,
        "currency": "USD",
        "reason": "v03 compat",
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/compensation")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 2048).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    // 字段必须存在(空 array)
    assert!(body_json["card_ids"].is_array());
    assert_eq!(body_json["card_ids"].as_array().unwrap().len(), 0);
    assert!(body_json["pack_ids"].is_array());
    assert_eq!(body_json["pack_ids"].as_array().unwrap().len(), 0);
    assert_eq!(body_json["cards_granted"], 0);
    assert_eq!(body_json["packs_granted"], 0);
}

// ============================================================================
// 12. v0.3 兼容: 老 SetMaintenance 请求无 mode_flags
// ============================================================================

#[tokio::test]
async fn v03_compat_maintenance_no_mode_flags() {
    // v0.3 老请求: 不传 mode_flags → 默认 0 (0 破坏)
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
        "ttl_seconds": 0,
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/gm/maintenance")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["mode_flags"], 0);
    assert_eq!(body_json["applied_mode_flags"], 0);
}

// ============================================================================
// 13. v0.3 兼容: 老 QueryAuditLog 请求无 audit_type
// ============================================================================

#[tokio::test]
async fn v03_compat_audit_no_audit_type_query() {
    // v0.3 老请求: 不传 audit_type → 默认 all (兼容)
    let app = axum::Router::new()
        .route(
            "/api/v1/audit/logs",
            axum::routing::get(gm_backend::query_audit),
        )
        .with_state(build_test_state());

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/audit/logs?limit=10")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["audit_type"], "all");
    assert_eq!(body_json["applied_audit_type"], "all");
    assert!(body_json["entries"].is_array());
}

// ============================================================================
// 14. parse_audit_type: 有效字符串 → enum
// ============================================================================

#[test]
fn parse_audit_type_returns_enum_for_valid_strings() {
    assert_eq!(parse_audit_type("all"), Some(AUDIT_TYPE_ALL));
    assert_eq!(parse_audit_type("trade"), Some(AUDIT_TYPE_TRADE));
    assert_eq!(parse_audit_type("gacha"), Some(AUDIT_TYPE_GACHA));
    assert_eq!(parse_audit_type("match"), Some(AUDIT_TYPE_MATCH));
    assert_eq!(
        parse_audit_type("compensation"),
        Some(AUDIT_TYPE_COMPENSATION)
    );
    // 大小写不敏感
    assert_eq!(parse_audit_type("TRADE"), Some(AUDIT_TYPE_TRADE));
    assert_eq!(parse_audit_type("Gacha"), Some(AUDIT_TYPE_GACHA));
}

// ============================================================================
// 15. parse_audit_type: 无效字符串 → None
// ============================================================================

#[test]
fn parse_audit_type_returns_none_for_invalid_strings() {
    assert_eq!(parse_audit_type("invalid"), None);
    assert_eq!(parse_audit_type(""), None);
    assert_eq!(parse_audit_type("TRADE_FAKE"), None);
    // 验证 audit_type_to_str 反向映射一致性
    assert_eq!(audit_type_to_str(AUDIT_TYPE_ALL), "all");
    assert_eq!(audit_type_to_str(AUDIT_TYPE_TRADE), "trade");
    assert_eq!(audit_type_to_str(AUDIT_TYPE_GACHA), "gacha");
    assert_eq!(audit_type_to_str(AUDIT_TYPE_MATCH), "match");
    assert_eq!(audit_type_to_str(AUDIT_TYPE_COMPENSATION), "compensation");
    // 未定义值 fallback "all"
    assert_eq!(audit_type_to_str(999), "all");
    assert_eq!(audit_type_to_str(0), "all");
}
