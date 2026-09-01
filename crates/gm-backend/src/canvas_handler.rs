//! canvas_handler — Canvas 画布指令 (per ROPE_CS gm_platform/modules/canvas 移植)
//! 2026-09-01 actix-web 重写, mock 转发 (生产应接 match-service / game-server)

use actix_web::{web, HttpResponse};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct CanvasCommandRequest {
    pub layer: String,
    pub anchor: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub image_base64: Option<String>,
    #[serde(default)]
    pub clear: Option<bool>,
}

pub const ALLOWED_ANCHORS: &[&str] = &[
    "top_left", "top_center", "top_right",
    "center_left", "center", "center_right",
    "bottom_left", "bottom_center", "bottom_right",
];

pub async fn list_anchors(_state: web::Data<AppState>) -> HttpResponse {
    let anchors: Vec<AnchorOption> = ALLOWED_ANCHORS.iter().map(|a| AnchorOption {
        value: a.to_string(),
        label: a.replace('_', " "),
    }).collect();
    HttpResponse::Ok().json(json!({"anchors": anchors}))
}

pub async fn send_canvas_command(
    _state: web::Data<AppState>,
    body: web::Json<CanvasCommandRequest>,
) -> HttpResponse {
    if !ALLOWED_ANCHORS.contains(&body.anchor.as_str()) {
        return HttpResponse::BadRequest().json(json!({
            "error": "invalid_anchor",
            "message": format!("anchor must be one of {:?}", ALLOWED_ANCHORS)
        }));
    }
    if body.width == 0 || body.height == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "invalid_dimensions"}));
    }
    // 校验 base64 长度 (简单 sanity check)
    if let Some(b64) = &body.image_base64 {
        if b64.len() > 10 * 1024 * 1024 {
            return HttpResponse::BadRequest().json(json!({"error": "image_too_large", "max_bytes": 10485760}));
        }
        // 验证可解析
        let _ = base64::engine::general_purpose::STANDARD.decode(b64)
            .map_err(|_| actix_web::error::ErrorBadRequest("invalid_base64"));
    }
    // mock 转发成功
    HttpResponse::Ok().json(json!({
        "status": "queued",
        "command": {
            "layer": body.layer,
            "anchor": body.anchor,
            "width": body.width,
            "height": body.height,
            "clear": body.clear.unwrap_or(false),
        }
    }))
}
