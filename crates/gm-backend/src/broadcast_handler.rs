//! broadcast_handler — 公告 + SSE 实时事件流 (per ROPE_CS gm_platform/modules/activity 移植)
//! 2026-09-01 actix-web 重写 + actix-web-lab SSE
//!
//! ## 2026-09-01 22:30 JST Phase D D6 修复
//! - `broadcast` handler 写入 audit_store (action="broadcast") 以便 `list_broadcasts` 反查
//! - 解决 DDD Review §7.2 P1 GM backend `list_broadcasts` 已知 gap
//! - IT6 同步更新: 期望 list 返 1 条 (修复前返 0 条)

use actix_web::{web, HttpRequest, HttpResponse};
use async_stream::stream;
use bytes::Bytes;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{extract_claims, verify_jwt, AppState, AuditLogEntry, Claims};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEntry {
    pub id: u64,
    pub message: String,
    pub admin: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct BroadcastRequest {
    pub message: String,
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub async fn broadcast(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<BroadcastRequest>,
) -> HttpResponse {
    if body.message.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "missing_message"}));
    }
    let admin = extract_claims(&req)
        .map(|c: Claims| c.sub)
        .unwrap_or_else(|| "unknown".to_string());
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let occurred_at_ms = Utc::now().timestamp_millis();
    let entry = BroadcastEntry {
        id,
        message: body.message.clone(),
        admin: admin.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    // 推送给所有 SSE 订阅者
    let _ = state.broadcast_tx.send(entry.clone());
    // 2026-09-01 22:30 JST Phase D D6: 写入 audit_store 以便 list_broadcasts 反查
    // 解决 DDD Review §7.2 P1 GM backend list_broadcasts 已知 gap (per WT-10-brief §D6)
    state.audit_store.append(AuditLogEntry {
        log_id: format!("broadcast-{}", id),
        admin_id: admin,
        action: "broadcast".to_string(),
        target_id: body.message.clone(),
        occurred_at_ms,
    });
    HttpResponse::Ok().json(json!({"status": "sent", "broadcast": entry}))
}

pub async fn list_broadcasts(state: web::Data<AppState>) -> HttpResponse {
    // 2026-09-01 22:30 JST Phase D D6 修复: 实际从 audit_store 反查
    // (per broadcast_handler.rs L60-72) — broadcast 现在写 audit action="broadcast"
    // (per WBS v0.2 桶 10 Phase D D6, commit 84edf26)
    let entries = state.audit_store.list_entries(50).await;
    let broadcasts: Vec<BroadcastEntry> = entries
        .into_iter()
        .filter(|e| e.action == "broadcast")
        .map(|e| BroadcastEntry {
            id: e.occurred_at_ms as u64,
            message: e.target_id,
            admin: e.admin_id,
            created_at: chrono::DateTime::<Utc>::from_timestamp_millis(e.occurred_at_ms)
                .map(|d| d.to_rfc3339())
                .unwrap_or_default(),
        })
        .collect();
    HttpResponse::Ok().json(json!({"broadcasts": broadcasts}))
}

// ============================================================================
// SSE 实时事件流 (per ROPE_CS /gm/events)
// ============================================================================

pub async fn sse_events(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    // 手动验证 token (因为 SSE 不走 JWT middleware scope)
    let token = req.query_string()
        .split('&')
        .find_map(|kv| kv.strip_prefix("token=").map(|s| s.to_string()))
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        });
    if state.config.require_jwt {
        let token = match token {
            Some(t) => t,
            None => return HttpResponse::Unauthorized().json(json!({"error": "missing_token"})),
        };
        if verify_jwt(&state.config.jwt_secret, &token).is_err() {
            return HttpResponse::Unauthorized().json(json!({"error": "invalid_token"}));
        }
    }
    let mut rx = state.broadcast_tx.subscribe();
    let sse_stream = stream! {
        // 初始 retry hint
        yield Ok::<_, std::convert::Infallible>(Bytes::from("retry: 10000\n\n"));
        loop {
            match rx.recv().await {
                Ok(entry) => {
                    let payload = serde_json::to_string(&entry).unwrap_or_default();
                    yield Ok(Bytes::from(format!("data: {}\n\n", payload)));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    };
    HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .insert_header(("cache-control", "no-cache"))
        .insert_header(("connection", "keep-alive"))
        .streaming(sse_stream)
}
