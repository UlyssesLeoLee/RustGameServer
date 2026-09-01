//! gm-backend 跨模块集成测试 (per 9/1 14:15 JST PT-WORKER-BRIEFING §2.3)
//!
//! 跨模块场景 (3+):
//! IT1. login (auth_handler) → ban_account (business_handler) → query_audit
//!      验证 JWT 颁发 + 5 GM endpoint 降级路径 + InMemory audit 写入
//! IT2. mall CRUD 完整链路: create → list → update → delete → list (空)
//! IT3. ticket 生命周期: create → list → update_ticket_status → list
//! IT4. JWT middleware: missing token + valid token + invalid token
//! IT5. health_view + summary 聚合 + metrics 输出
//!
//! 全部用 actix-web::test 框架 + register_routes 全 15+ 端点 (降级 InMemory).

use actix_web::{test, web, App};
use gm_backend::{
    issue_jwt, register_routes, AppState, GmConfig, LoginRequest,
};

// ============================================================================
// helpers
// ============================================================================

fn test_state() -> AppState {
    let config = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://127.0.0.1:50055").unwrap();
    AppState::new(config)
}

// ============================================================================
// IT1: login → ban_account → query_audit (跨 3 模块)
// ============================================================================

#[actix_web::test]
async fn it_login_then_ban_then_query_audit_degraded_path() {
    let state = test_state();
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 1) login
    let login_req = test::TestRequest::post()
        .uri("/gm/login")
        .set_json(LoginRequest {
            username: "admin".to_string(),
            password: "adminpass".to_string(),
        })
        .to_request();
    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success());
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let token = login_body["token"].as_str().expect("token").to_string();
    assert!(!token.is_empty(), "JWT 必须颁发非空 token");

    // 2) ban_account (admin_grpc=None → degraded=true)
    let ban_req = test::TestRequest::post()
        .uri("/gm/ban_account")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(serde_json::json!({
            "account_id": "player-evil-007",
            "reason": "cheating detected",
            "duration_seconds": 86400,
            "force_disconnect_session": true,
        }))
        .to_request();
    let ban_resp = test::call_service(&app, ban_req).await;
    assert!(ban_resp.status().is_success());
    let ban_body: serde_json::Value = test::read_body_json(ban_resp).await;
    assert_eq!(ban_body["status"], "queued");
    assert_eq!(ban_body["op"], "ban");
    assert_eq!(ban_body["degraded"], true, "admin_grpc=None → degraded");
    assert_eq!(ban_body["disconnected_sessions"], true);

    // 3) query_audit — 应看到刚才的 ban entry
    let audit_req = test::TestRequest::get()
        .uri("/gm/query_audit?limit=10")
        .to_request();
    let audit_resp = test::call_service(&app, audit_req).await;
    assert!(audit_resp.status().is_success());
    let audit_body: serde_json::Value = test::read_body_json(audit_resp).await;
    assert_eq!(audit_body["source"], "in_memory_fallback");
    let entries = audit_body["entries"].as_array().expect("entries array");
    assert!(!entries.is_empty(), "InMemory 必须有至少 1 条 ban audit");
    let last = entries[0].clone();
    assert_eq!(last["action"], "ban");
    assert_eq!(last["target_id"], "player-evil-007");
    assert_eq!(last["admin_id"], "system");
}

// ============================================================================
// IT2: mall item 完整 CRUD 跨 4 handler (create/list/update/delete)
// ============================================================================

#[actix_web::test]
async fn it_mall_item_full_crud_lifecycle() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 1) create
    let create_req = test::TestRequest::post()
        .uri("/gm/mall/items")
        .set_json(serde_json::json!({
            "name": "Gold Sword",
            "price": 99.99,
            "currency": "GOLD",
            "category": "weapon",
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    assert_eq!(create_body["status"], "created");
    let item_id = create_body["item"]["id"].as_u64().expect("item.id");
    assert_eq!(create_body["item"]["name"], "Gold Sword");
    assert_eq!(create_body["item"]["price"], 99.99);

    // 2) list — 1 条
    let list_req = test::TestRequest::get().uri("/gm/mall/items").to_request();
    let list_resp = test::call_service(&app, list_req).await;
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    assert_eq!(list_body["items"].as_array().unwrap().len(), 1);

    // 3) update price
    let update_req = test::TestRequest::put()
        .uri(&format!("/gm/mall/items/{item_id}"))
        .set_json(serde_json::json!({ "price": 149.99, "enabled": true }))
        .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert!(update_resp.status().is_success());
    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["item"]["price"], 149.99);

    // 4) delete
    let del_req = test::TestRequest::delete()
        .uri(&format!("/gm/mall/items/{item_id}"))
        .to_request();
    let del_resp = test::call_service(&app, del_req).await;
    assert!(del_resp.status().is_success());

    // 5) list again — 0 条
    let list2_req = test::TestRequest::get().uri("/gm/mall/items").to_request();
    let list2_resp = test::call_service(&app, list2_req).await;
    let list2_body: serde_json::Value = test::read_body_json(list2_resp).await;
    assert_eq!(list2_body["items"].as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn it_mall_item_create_rejects_invalid_price() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // price < 0
    let req = test::TestRequest::post()
        .uri("/gm/mall/items")
        .set_json(serde_json::json!({
            "name": "Bad Item",
            "price": -10.0,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400, "负价必须返 400");

    // empty name
    let req2 = test::TestRequest::post()
        .uri("/gm/mall/items")
        .set_json(serde_json::json!({
            "name": "  ",
            "price": 10.0,
        }))
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status().as_u16(), 400, "空名必须返 400");
}

// ============================================================================
// IT3: ticket 生命周期 (create → list → update status → list)
// ============================================================================

#[actix_web::test]
async fn it_ticket_full_lifecycle() {
    let state = test_state();
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 1) create ticket
    let create_req = test::TestRequest::post()
        .uri("/gm/support")
        .set_json(serde_json::json!({
            "player_id": "player-100200",
            "message": "Lost my sword after gacha pull",
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    assert_eq!(create_body["status"], "received");
    let ticket_id = create_body["ticket"]["id"].as_u64().expect("ticket.id");
    assert_eq!(create_body["ticket"]["status"], "open");

    // 2) list — 1 条
    let list_req = test::TestRequest::get().uri("/gm/support/tickets").to_request();
    let list_resp = test::call_service(&app, list_req).await;
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    assert_eq!(list_body["tickets"].as_array().unwrap().len(), 1);

    // 3) update → pending
    let update_req = test::TestRequest::patch()
        .uri(&format!("/gm/support/tickets/{ticket_id}"))
        .set_json(serde_json::json!({ "status": "pending" }))
        .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert!(update_resp.status().is_success());
    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["ticket"]["status"], "pending");

    // 4) update → resolved
    let update2_req = test::TestRequest::patch()
        .uri(&format!("/gm/support/tickets/{ticket_id}"))
        .set_json(serde_json::json!({ "status": "resolved" }))
        .to_request();
    let update2_resp = test::call_service(&app, update2_req).await;
    let update2_body: serde_json::Value = test::read_body_json(update2_resp).await;
    assert_eq!(update2_body["ticket"]["status"], "resolved");
}

#[actix_web::test]
async fn it_ticket_status_invalid_returns_400() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 先创建一个 ticket
    let create_req = test::TestRequest::post()
        .uri("/gm/support")
        .set_json(serde_json::json!({
            "player_id": "p1",
            "message": "test",
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let tid = create_body["ticket"]["id"].as_u64().unwrap();

    // 尝试非法状态
    let bad_req = test::TestRequest::patch()
        .uri(&format!("/gm/support/tickets/{tid}"))
        .set_json(serde_json::json!({ "status": "deleted" }))
        .to_request();
    let bad_resp = test::call_service(&app, bad_req).await;
    assert_eq!(bad_resp.status().as_u16(), 400, "非法 status 必须返 400");
}

// ============================================================================
// IT4: JWT middleware (require_jwt=true 时 missing/invalid token → 401)
// ============================================================================

#[actix_web::test]
async fn it_jwt_middleware_missing_token_returns_401() {
    // 构造一个 require_jwt=true 的 AppState
    let mut cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://127.0.0.1:50055").unwrap();
    cfg.require_jwt = true;
    cfg.jwt_secret = "test-secret-xyz".to_string();
    let state = AppState::new(cfg);
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(gm_backend::JwtAuth {
                require: true,
                secret: "test-secret-xyz".to_string(),
            })
            .configure(register_routes),
    )
    .await;

    // missing token
    let req = test::TestRequest::post()
        .uri("/gm/ban_account")
        .set_json(serde_json::json!({
            "account_id": "p1",
            "reason": "test",
            "duration_seconds": 60,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401, "missing token 必须 401");
}

#[actix_web::test]
async fn it_jwt_middleware_invalid_token_returns_401() {
    let mut cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://127.0.0.1:50055").unwrap();
    cfg.require_jwt = true;
    cfg.jwt_secret = "test-secret-xyz".to_string();
    let state = AppState::new(cfg);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(gm_backend::JwtAuth {
                require: true,
                secret: "test-secret-xyz".to_string(),
            })
            .configure(register_routes),
    )
    .await;

    // garbage token
    let req = test::TestRequest::post()
        .uri("/gm/ban_account")
        .insert_header(("Authorization", "Bearer garbage.token.value"))
        .set_json(serde_json::json!({
            "account_id": "p1",
            "reason": "test",
            "duration_seconds": 60,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401, "invalid token 必须 401");
}

#[actix_web::test]
async fn it_jwt_middleware_valid_token_passes_through() {
    let mut cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://127.0.0.1:50055").unwrap();
    cfg.require_jwt = true;
    cfg.jwt_secret = "test-secret-xyz".to_string();
    let state = AppState::new(cfg);
    let token = issue_jwt("test-secret-xyz", "ops", vec!["GM_READ".into()], 60).unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(gm_backend::JwtAuth {
                require: true,
                secret: "test-secret-xyz".to_string(),
            })
            .configure(register_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/gm/ban_account")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(serde_json::json!({
            "account_id": "p-valid",
            "reason": "test valid",
            "duration_seconds": 30,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "valid token 必须通过 JWT middleware");
}

// ============================================================================
// IT5: health_view + summary + metrics 聚合 (3 端点)
// ============================================================================

#[actix_web::test]
async fn it_health_view_summary_metrics_aggregation() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 1) health_view
    let h_req = test::TestRequest::get().uri("/gm/health_view").to_request();
    let h_resp = test::call_service(&app, h_req).await;
    assert!(h_resp.status().is_success());
    let h_body: serde_json::Value = test::read_body_json(h_resp).await;
    assert!(h_body["services"].is_array());
    assert_eq!(h_body["services"][0]["service_name"], "admin-service");
    // admin_grpc=None → ready=true (no fallback)
    assert_eq!(h_body["services"][0]["ready"], true);

    // 2) summary
    let s_req = test::TestRequest::get().uri("/gm/summary").to_request();
    let s_resp = test::call_service(&app, s_req).await;
    assert!(s_resp.status().is_success());
    let s_body: serde_json::Value = test::read_body_json(s_resp).await;
    assert_eq!(s_body["playerStats"]["total"], 1000);
    assert_eq!(s_body["servers"]["stats"]["total"], 5); // 初始 5 假 server
    assert_eq!(s_body["servers"]["stats"]["running"], 4); // 1 stopped (social-1)

    // 3) metrics (Prometheus text)
    let m_req = test::TestRequest::get().uri("/gm/metrics").to_request();
    let m_resp = test::call_service(&app, m_req).await;
    assert!(m_resp.status().is_success());
    let m_body = test::read_body(m_resp).await;
    let m_text = std::str::from_utf8(&m_body).expect("utf-8");
    assert!(m_text.contains("online_connections"));
    assert!(m_text.contains("running_servers 4"));
    assert!(m_text.contains("total_servers 5"));
}

// ============================================================================
// IT6: broadcast → list_broadcasts (audit_store 写入 + 反查)
// 2026-09-01 22:30 JST Phase D D6 修复: broadcast 端点已写 audit,
// list_broadcasts 现在应返 1 条 (per WBS v0.2 桶 10 Phase D D6, commit 84edf26)
// ============================================================================

#[actix_web::test]
async fn it_broadcast_writes_to_audit_store() {
    let state = test_state();
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 1) broadcast
    let req = test::TestRequest::post()
        .uri("/gm/broadcast")
        .set_json(serde_json::json!({
            "message": "Server maintenance at 03:00 UTC",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "sent");
    assert_eq!(body["broadcast"]["message"], "Server maintenance at 03:00 UTC");

    // 2) list_broadcasts — 现在从 audit_store 反查 action=="broadcast" 的条目
    //    (per WBS v0.2 桶 10 Phase D D6 修复) — broadcast 端点已写 audit
    //    list 应返 1 条
    let list_req = test::TestRequest::get().uri("/gm/broadcasts").to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert!(list_resp.status().is_success());
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let arr = list_body["broadcasts"].as_array().expect("broadcasts array");
    assert_eq!(arr.len(), 1, "D6 修复后 list_broadcasts 应返 1 条");
    assert_eq!(arr[0]["message"], "Server maintenance at 03:00 UTC");
}

// ============================================================================
// IT6b: list_broadcasts 空状态 (无 broadcast 时返 0)
// 2026-09-01 22:30 JST Phase D D6 新增: 边界用例
// ============================================================================

#[actix_web::test]
async fn it_list_broadcasts_empty_state() {
    let state = test_state();
    state.ensure_default_admin().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    let list_req = test::TestRequest::get().uri("/gm/broadcasts").to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert!(list_resp.status().is_success());
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let arr = list_body["broadcasts"].as_array().expect("broadcasts array");
    assert_eq!(arr.len(), 0, "无 broadcast 时 list_broadcasts 返 0 条");
}

// ============================================================================
// IT7: server start/stop state transitions
// ============================================================================

#[actix_web::test]
async fn it_server_start_stop_state_transitions() {
    let state = test_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(register_routes),
    )
    .await;

    // 1) list 初始
    let list_req = test::TestRequest::get().uri("/gm/servers").to_request();
    let list_resp = test::call_service(&app, list_req).await;
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let servers = list_body["servers"].as_array().expect("servers");
    assert_eq!(servers.len(), 5);
    // 找 social-1 (初始 stopped)
    let social1 = servers.iter().find(|s| s["id"] == "social-1").expect("social-1");
    assert_eq!(social1["status"], "stopped");

    // 2) start social-1
    let start_req = test::TestRequest::post().uri("/gm/servers/social-1/start").to_request();
    let start_resp = test::call_service(&app, start_req).await;
    let start_body: serde_json::Value = test::read_body_json(start_resp).await;
    assert_eq!(start_body["status"], "started");
    assert_eq!(start_body["server"]["status"], "running");
    assert!(start_body["server"]["online_players"].as_u64().unwrap() >= 50);

    // 3) stop social-1
    let stop_req = test::TestRequest::post().uri("/gm/servers/social-1/stop").to_request();
    let stop_resp = test::call_service(&app, stop_req).await;
    let stop_body: serde_json::Value = test::read_body_json(stop_resp).await;
    assert_eq!(stop_body["status"], "stopped");
    assert_eq!(stop_body["server"]["status"], "stopped");
    assert_eq!(stop_body["server"]["online_players"], 0);

    // 4) start 不存在的 server
    let bad_req = test::TestRequest::post().uri("/gm/servers/nonexistent/start").to_request();
    let bad_resp = test::call_service(&app, bad_req).await;
    assert_eq!(bad_resp.status().as_u16(), 404, "未知 server 必须 404");
}
