//! servers_handler — 服务管控 (per ROPE_CS gm_platform/modules/system 移植)
//! 2026-09-01 actix-web 重写, 内存 mock (生产应接 cluster-ops gRPC)

use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub name: String,
    pub region: Option<String>,
    pub status: String,
    pub online_players: u32,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub total: u32,
    pub running: u32,
}

pub async fn list_servers(state: web::Data<AppState>) -> HttpResponse {
    let servers = state.servers.lock().unwrap();
    HttpResponse::Ok().json(json!({"servers": servers.clone()}))
}

pub async fn get_server_stats(state: web::Data<AppState>) -> HttpResponse {
    let servers = state.servers.lock().unwrap();
    let total = servers.len() as u32;
    let running = servers.iter().filter(|s| s.status == "running").count() as u32;
    HttpResponse::Ok().json(ServerStats { total, running })
}

pub async fn start_server(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let id = path.into_inner();
    let mut servers = state.servers.lock().unwrap();
    let server = match servers.iter_mut().find(|s| s.id == id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().json(json!({"error": "not_found"})),
    };
    server.status = "running".to_string();
    server.online_players = rand::thread_rng().gen_range(50..500);
    server.last_updated = Some(Utc::now().to_rfc3339());
    HttpResponse::Ok().json(json!({"status": "started", "server": server.clone()}))
}

pub async fn stop_server(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let id = path.into_inner();
    let mut servers = state.servers.lock().unwrap();
    let server = match servers.iter_mut().find(|s| s.id == id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().json(json!({"error": "not_found"})),
    };
    server.status = "stopped".to_string();
    server.online_players = 0;
    server.last_updated = Some(Utc::now().to_rfc3339());
    HttpResponse::Ok().json(json!({"status": "stopped", "server": server.clone()}))
}

/// Prometheus 风格 /gm/metrics
pub async fn metrics(state: web::Data<AppState>) -> HttpResponse {
    let servers = state.servers.lock().unwrap();
    let total_connections: u32 = servers.iter().map(|s| s.online_players).sum();
    let total = servers.len() as u32;
    let running = servers.iter().filter(|s| s.status == "running").count() as u32;
    let body = format!(
        "online_connections {}\nrunning_servers {}\ntotal_servers {}\n",
        total_connections, running, total
    );
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(body)
}
