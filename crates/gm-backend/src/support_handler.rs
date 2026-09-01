//! support_handler — 客服工单 (per ROPE_CS gm_platform/modules/support 移植)
//! 2026-09-01 actix-web 重写

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{extract_claims, AppState, Claims};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketEntry {
    pub id: u64,
    pub player_id: String,
    pub message: String,
    pub admin: String,
    pub status: String, // "open" / "pending" / "resolved"
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub player_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketStatusRequest {
    pub status: String,
}

static NEXT_TICKET_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub async fn create_ticket(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateTicketRequest>,
) -> HttpResponse {
    if body.player_id.trim().is_empty() || body.message.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "missing_fields"}));
    }
    let admin = extract_claims(&req).map(|c: Claims| c.sub).unwrap_or_else(|| "unknown".to_string());
    let id = NEXT_TICKET_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let now = Utc::now().to_rfc3339();
    let ticket = TicketEntry {
        id,
        player_id: body.player_id.clone(),
        message: body.message.clone(),
        admin,
        status: "open".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    state.tickets.lock().unwrap().push(ticket.clone());
    HttpResponse::Ok().json(json!({"status": "received", "ticket": ticket}))
}

pub async fn list_tickets(state: web::Data<AppState>) -> HttpResponse {
    let tickets = state.tickets.lock().unwrap();
    HttpResponse::Ok().json(json!({"tickets": tickets.clone()}))
}

pub async fn update_ticket_status(
    state: web::Data<AppState>,
    path: web::Path<u64>,
    body: web::Json<UpdateTicketStatusRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let mut tickets = state.tickets.lock().unwrap();
    let ticket = match tickets.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return HttpResponse::NotFound().json(json!({"error": "not_found"})),
    };
    let allowed = ["open", "pending", "resolved"];
    if !allowed.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(json!({"error": "invalid_status", "allowed": allowed}));
    }
    ticket.status = body.status.clone();
    ticket.updated_at = Utc::now().to_rfc3339();
    HttpResponse::Ok().json(json!({"ticket": ticket.clone()}))
}
