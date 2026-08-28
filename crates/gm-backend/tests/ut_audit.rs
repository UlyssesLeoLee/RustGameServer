//! TBD-08-04 audit_log handler 实装测试 + TBD-08-07 rgs-testkit 实际使用
//!
//! 7 测试:
//! 1. ban_account 写 audit_log
//! 2. grant_compensation 写 audit_log
//! 3. set_maintenance 写 audit_log
//! 4. query_audit 初始空 entries
//! 5. query_audit 3 entries 倒序
//! 6. query_audit limit 截断 + has_more
//! 7. rgs-testkit FixtureBuilder 实际使用(per TBD-08-07)

use axum_test::TestServer;
use gm_backend::{AuditLogEntry, AuditStore, GmConfig, InMemoryAuditStore};
use rgs_testkit::fixture::{self, FixtureBuilder};
use serde_json::json;
use std::sync::Arc;

fn make_test_server() -> (TestServer, Arc<InMemoryAuditStore>) {
    let cfg = GmConfig::for_test("0.0.0.0:8443", "0.0.0.0:8081", "http://admin:50055").unwrap();
    let store = InMemoryAuditStore::new();
    let store_dyn: Arc<dyn AuditStore> = Arc::new(store.clone());
    let state = gm_backend::AppState::with_audit_store(cfg, store_dyn);
    let app = gm_backend::build_router(state);
    (TestServer::new(app).unwrap(), Arc::new(store))
}

fn make_test_server_preloaded() -> (TestServer, Arc<InMemoryAuditStore>) {
    let cfg = GmConfig::for_test("0.0.0.0:8443", "0.0.0.0:8081", "http://admin:50055").unwrap();
    let store = InMemoryAuditStore::new();
    // S4 Phase 2 step 2: limit=20 (per gm.proto v0.3), 预加载 25 条以触发 has_more=true
    for i in 0..25 {
        store.append(AuditLogEntry {
            log_id: format!("pre-{i}"),
            admin_id: "test-admin".to_string(),
            action: "pre_action".to_string(),
            target_id: format!("target-{i}"),
            occurred_at_ms: 1_700_000_000_000 + i as i64,
        });
    }
    let store_dyn: Arc<dyn AuditStore> = Arc::new(store.clone());
    let state = gm_backend::AppState::with_audit_store(cfg, store_dyn);
    let app = gm_backend::build_router(state);
    (TestServer::new(app).unwrap(), Arc::new(store))
}

#[tokio::test]
async fn ban_account_writes_audit_log() {
    // W26 (2026-08-29) 桶 2a: 发送合法 body 写 audit_log
    let (server, store) = make_test_server();
    let resp = server
        .post("/api/v1/gm/ban")
        .json(&json!({"account_id": "audit-ban-target", "reason": "test", "duration_seconds": 0}))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let entries = store.list_entries(20).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "ban");
    assert_eq!(entries[0].admin_id, "system");
    assert_eq!(entries[0].target_id, "audit-ban-target"); // 真值, 不是 "stub"
}

#[tokio::test]
async fn grant_compensation_writes_audit_log() {
    // W26 (2026-08-29) 桶 2a: 发送合法 body
    let (server, store) = make_test_server();
    let resp = server
        .post("/api/v1/gm/compensation")
        .json(&json!({"account_id": "audit-comp-target", "amount": 100, "currency": "USD", "reason": "test"}))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let entries = store.list_entries(20).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "grant_compensation");
    assert_eq!(entries[0].target_id, "audit-comp-target"); // 真值
}

#[tokio::test]
async fn set_maintenance_writes_audit_log() {
    // W26 (2026-08-29) 桶 2a: 发送合法 body
    let (server, store) = make_test_server();
    let resp = server
        .post("/api/v1/gm/maintenance")
        .json(&json!({"enable": true, "scope": "cluster", "target_id": "cluster", "ttl_seconds": 0}))
        .await;
    resp.assert_status(axum::http::StatusCode::ACCEPTED);
    let entries = store.list_entries(20).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "set_maintenance");
    assert_eq!(entries[0].target_id, "cluster");
}

#[tokio::test]
async fn query_audit_initial_returns_empty_entries() {
    let (server, _) = make_test_server();
    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(body["entries"].is_array());
    assert_eq!(body["entries"].as_array().unwrap().len(), 0);
    assert_eq!(body["has_more"], false);
}

#[tokio::test]
async fn query_audit_returns_entries_in_reverse_chronological_order() {
    // 直接用 audit_store 验证
    let store = InMemoryAuditStore::new();
    store.append(AuditLogEntry {
        log_id: "1".to_string(),
        admin_id: "a".to_string(),
        action: "first".to_string(),
        target_id: "t1".to_string(),
        occurred_at_ms: 1,
    });
    store.append(AuditLogEntry {
        log_id: "2".to_string(),
        admin_id: "a".to_string(),
        action: "second".to_string(),
        target_id: "t2".to_string(),
        occurred_at_ms: 2,
    });
    store.append(AuditLogEntry {
        log_id: "3".to_string(),
        admin_id: "a".to_string(),
        action: "third".to_string(),
        target_id: "t3".to_string(),
        occurred_at_ms: 3,
    });
    let entries = store.list_entries(20).await;
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].action, "third");
    assert_eq!(entries[1].action, "second");
    assert_eq!(entries[2].action, "first");
}

#[tokio::test]
async fn query_audit_limit_truncates_and_sets_has_more() {
    let (server, _) = make_test_server_preloaded();
    let resp = server.get("/api/v1/audit/logs").await;
    resp.assert_status(axum::http::StatusCode::OK);
    let body: serde_json::Value = resp.json();
    let entries = body["entries"].as_array().unwrap();
    // S4 Phase 2 step 2: limit=20 (per gm.proto v0.3)
    assert_eq!(entries.len(), 20, "limit=20 must return 20 entries");
    assert_eq!(
        body["has_more"], true,
        "25 entries > limit 20, has_more=true"
    );
    // list_entries 反转 (新→旧), 25 条最新是 pre-24, 20 条返 pre-24..pre-5
    assert_eq!(entries[0]["log_id"], "pre-24");
    assert_eq!(entries[1]["log_id"], "pre-23");
    assert_eq!(entries[19]["log_id"], "pre-5");
}

#[test]
fn rgs_testkit_fixture_used_in_gm_backend_test() {
    // TBD-08-07 v0.2 实装: rgs-testkit FixtureBuilder 实际使用
    let admin = FixtureBuilder::new(fixture::admin_action("admin01", "ban", "player123"))
        .with_action("mute")
        .with_target("player456")
        .build();
    assert_eq!(admin.action, "mute");
    assert_eq!(admin.target_id, "player456");
}
