//! players_handler — 玩家列表 + 统计 (per ROPE_CS gm_platform/modules/player 移植)
//! 2026-09-01 actix-web 重写, 内存 mock 数据 (生产应接 player-service gRPC)

use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::Rng;
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PlayersQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerEntry {
    pub id: String,
    pub display_name: String,
    pub status: String,
    pub level: u32,
    pub total_spent: f64,
    pub last_login: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayersResponse {
    pub players: Vec<PlayerEntry>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerStatsResponse {
    pub total: u32,
    pub online: u32,
    pub offline: u32,
    pub banned: u32,
    pub average_level: f32,
    pub high_value: u32,
}

/// 生成 mock 玩家数据 (per ROPE_CS — 无外部数据源时使用)
fn generate_mock_players(count: usize) -> Vec<PlayerEntry> {
    let mut rng = rand::thread_rng();
    let statuses = ["online", "offline", "banned"];
    (0..count).map(|i| {
        let status = statuses[rng.gen_range(0..statuses.len())];
        PlayerEntry {
            id: format!("player-{:06}", 100000 + i),
            display_name: format!("Player_{:04}", i),
            status: status.to_string(),
            level: rng.gen_range(1..80),
            total_spent: rng.gen_range(0.0..5000.0),
            last_login: (Utc::now() - chrono::Duration::seconds(rng.gen_range(0..86400 * 30))).to_rfc3339(),
        }
    }).collect()
}

pub async fn list_players(
    _state: web::Data<AppState>,
    q: web::Query<PlayersQuery>,
) -> HttpResponse {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let search = q.search.as_deref().unwrap_or("").to_lowercase();
    let status_filter = q.status.as_deref().unwrap_or("");

    // mock 1000 玩家, 过滤 + 分页
    let all = generate_mock_players(1000);
    let filtered: Vec<_> = all
        .into_iter()
        .filter(|p| {
            (search.is_empty() || p.display_name.to_lowercase().contains(&search) || p.id.contains(&search))
                && (status_filter.is_empty() || p.status == status_filter)
        })
        .collect();
    let total = filtered.len() as u32;
    let start = ((page - 1) * page_size) as usize;
    let players: Vec<_> = filtered.into_iter().skip(start).take(page_size as usize).collect();

    HttpResponse::Ok().json(PlayersResponse { players, total, page, page_size })
}

pub async fn get_player_stats(_state: web::Data<AppState>) -> HttpResponse {
    let all = generate_mock_players(1000);
    let online = all.iter().filter(|p| p.status == "online").count() as u32;
    let offline = all.iter().filter(|p| p.status == "offline").count() as u32;
    let banned = all.iter().filter(|p| p.status == "banned").count() as u32;
    let total = all.len() as u32;
    let avg_level = all.iter().map(|p| p.level).sum::<u32>() as f32 / total as f32;
    let high_value = all.iter().filter(|p| p.total_spent > 1000.0).count() as u32;
    HttpResponse::Ok().json(PlayerStatsResponse {
        total, online, offline, banned, average_level: avg_level, high_value,
    })
}
