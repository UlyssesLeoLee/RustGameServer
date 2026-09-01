//! summary_handler — Dashboard 聚合数据 (per ROPE_CS gm_platform/server.js /gm/summary 移植)
//! 2026-09-01 actix-web 重写

use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::AppState;

/// GET /gm/summary — 一次拉取 Dashboard 全部数据 (per ROPE_CS dashboard 风格)
pub async fn summary(state: web::Data<AppState>) -> HttpResponse {
    // 直接从 InMemory 聚合 (生产应调各 5 域 gRPC)
    let grants_count = state.grants.lock().unwrap().len() as u32;
    let broadcasts_count = state.audit_store.list_entries(1000).await.iter()
        .filter(|e| e.action == "broadcast").count() as u32;
    let tickets = state.tickets.lock().unwrap();
    let tickets_open = tickets.iter().filter(|t| t.status != "resolved").count() as u32;
    let tickets_total = tickets.len() as u32;
    drop(tickets);
    let mall_count = state.mall_items.lock().unwrap().len() as u32;
    let servers = state.servers.lock().unwrap();
    let total = servers.len() as u32;
    let running = servers.iter().filter(|s| s.status == "running").count() as u32;
    drop(servers);

    HttpResponse::Ok().json(json!({
        "playerStats": {
            "total": 1000,
            "online": 487,
            "offline": 500,
            "banned": 13,
            "averageLevel": 32.5,
            "highValue": 47,
        },
        "activity": {
            "totalBroadcasts": broadcasts_count,
            "recentBroadcasts": [],
            "totalGrants": grants_count,
        },
        "support": {
            "open": tickets_open,
            "total": tickets_total,
        },
        "mall": {
            "totalItems": mall_count,
        },
        "servers": {
            "stats": { "total": total, "running": running },
            "list": state.servers.lock().unwrap().clone(),
        },
    }))
}
