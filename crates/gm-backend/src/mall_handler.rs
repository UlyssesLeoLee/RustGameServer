//! mall_handler — 商品 CRUD (per ROPE_CS gm_platform/modules/economy 移植)
//! 2026-09-01 actix-web 重写, 内存 mock

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MallItem {
    pub id: u64,
    pub name: String,
    pub price: f64,
    pub currency: String,
    pub category: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateMallItemRequest {
    pub name: String,
    pub price: f64,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMallItemRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

static NEXT_ITEM_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub async fn list_mall_items(state: web::Data<AppState>) -> HttpResponse {
    let items = state.mall_items.lock().unwrap();
    HttpResponse::Ok().json(json!({"items": items.clone()}))
}

pub async fn create_mall_item(
    state: web::Data<AppState>,
    body: web::Json<CreateMallItemRequest>,
) -> HttpResponse {
    if body.name.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "missing_name"}));
    }
    if body.price < 0.0 {
        return HttpResponse::BadRequest().json(json!({"error": "invalid_price"}));
    }
    let id = NEXT_ITEM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let item = MallItem {
        id,
        name: body.name.clone(),
        price: body.price,
        currency: body.currency.clone().unwrap_or_else(|| "GOLD".to_string()),
        category: body.category.clone(),
        enabled: true,
    };
    state.mall_items.lock().unwrap().push(item.clone());
    HttpResponse::Ok().json(json!({"status": "created", "item": item}))
}

pub async fn update_mall_item(
    state: web::Data<AppState>,
    path: web::Path<u64>,
    body: web::Json<UpdateMallItemRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let mut items = state.mall_items.lock().unwrap();
    let item = match items.iter_mut().find(|i| i.id == id) {
        Some(i) => i,
        None => return HttpResponse::NotFound().json(json!({"error": "not_found"})),
    };
    if let Some(name) = &body.name { item.name = name.clone(); }
    if let Some(price) = body.price { item.price = price; }
    if let Some(category) = &body.category { item.category = Some(category.clone()); }
    if let Some(enabled) = body.enabled { item.enabled = enabled; }
    HttpResponse::Ok().json(json!({"status": "updated", "item": item.clone()}))
}

pub async fn delete_mall_item(
    state: web::Data<AppState>,
    path: web::Path<u64>,
) -> HttpResponse {
    let id = path.into_inner();
    let mut items = state.mall_items.lock().unwrap();
    let pos = items.iter().position(|i| i.id == id);
    match pos {
        Some(p) => { items.remove(p); HttpResponse::Ok().json(json!({"status": "deleted"})) }
        None => HttpResponse::NotFound().json(json!({"error": "not_found"})),
    }
}
