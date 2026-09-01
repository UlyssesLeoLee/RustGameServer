//! items_handler — 道具发放 (per ROPE_CS gm_platform/modules/economy 移植)
//! 2026-09-01 actix-web 重写

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{extract_claims, AppState, Claims};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantEntry {
    pub id: u64,
    pub player_id: String,
    pub item_id: String,
    pub amount: u32,
    pub admin: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantRequest {
    pub player_id: String,
    pub item_id: String,
    pub amount: i64,
}

static NEXT_GRANT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub async fn grant_item(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<GrantRequest>,
) -> HttpResponse {
    if body.player_id.trim().is_empty() || body.item_id.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "missing_fields"}));
    }
    if body.amount <= 0 || body.amount > i32::MAX as i64 {
        return HttpResponse::BadRequest().json(json!({"error": "invalid_amount"}));
    }
    let admin = extract_claims(&req).map(|c: Claims| c.sub).unwrap_or_else(|| "unknown".to_string());
    let id = NEXT_GRANT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let entry = GrantEntry {
        id,
        player_id: body.player_id.clone(),
        item_id: body.item_id.clone(),
        amount: body.amount as u32,
        admin,
        created_at: Utc::now().to_rfc3339(),
    };
    state.grants.lock().unwrap().push(entry.clone());
    HttpResponse::Ok().json(json!({"status": "granted", "grant": entry}))
}

pub async fn list_grants(state: web::Data<AppState>) -> HttpResponse {
    let grants = state.grants.lock().unwrap();
    HttpResponse::Ok().json(json!({"grants": grants.clone()}))
}
