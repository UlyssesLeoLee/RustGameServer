//! 5 GM endpoint 业务 handler (per RGS-BAS-003 §3.1-§3.4 + gm.proto v0.4)
//! 2026-09-01 actix-web 重写 (per Ulysses 决策), 保留原 axum 5 endpoint 行为不变.

use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

use crate::{
    admin::v1::{AuditType, BanAccountRequest, GrantCompensationRequest, QueryAuditLogRequest, SetMaintenanceRequest},
    AppState, AuditLogEntry, ServiceHealthEntry,
};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BanAccountRequestBody {
    pub account_id: String,
    pub reason: String,
    #[serde(default)]
    pub duration_seconds: i32,
    #[serde(default)]
    pub force_disconnect_session: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompensationRequestBody {
    pub account_id: String,
    pub amount: i64,
    pub currency: String,
    pub reason: String,
    #[serde(default)]
    pub card_ids: Vec<String>,
    #[serde(default)]
    pub pack_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceRequestBody {
    pub enable: bool,
    pub scope: String,
    pub target_id: String,
    #[serde(default)]
    pub ttl_seconds: i32,
    #[serde(default)]
    pub mode_flags: u32,
}

#[derive(Debug, Deserialize)]
pub struct QueryAuditLogQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub filter_admin: Option<String>,
    #[serde(default)]
    pub filter_action: Option<String>,
    #[serde(default)]
    pub audit_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HealthViewQuery {
    #[serde(default)]
    pub request_id: Option<String>,
}

pub const DEFAULT_AUDIT_LIMIT: usize = 20;
pub const MAX_AUDIT_LIMIT: usize = 100;
pub const ALLOWED_MAINTENANCE_SCOPES: &[&str] = &["cluster", "domain", "single_node"];

pub fn parse_audit_type(s: &str) -> Option<i32> {
    match s.to_ascii_lowercase().as_str() {
        "all" => Some(AuditType::All as i32),
        "trade" => Some(AuditType::Trade as i32),
        "gacha" => Some(AuditType::Gacha as i32),
        "match" => Some(AuditType::Match as i32),
        "compensation" => Some(AuditType::Compensation as i32),
        _ => None,
    }
}

fn bad_request(error: &str, detail: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({"error": error, "detail": detail}))
}

// ============================================================================
// 1. HealthView
// ============================================================================

pub async fn health_view(
    state: web::Data<AppState>,
    q: web::Query<HealthViewQuery>,
) -> HttpResponse {
    let request_id = q.request_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let now_ms = Utc::now().timestamp_millis();
    let ready = match state.admin_grpc.as_ref() {
        Some(client) => {
            match tokio::time::timeout(Duration::from_millis(500), client.health_check()).await {
                Ok(Ok(())) => true,
                Ok(Err(e)) => { tracing::warn!("admin-service health_check failed: {e}"); false }
                Err(_) => { tracing::warn!("admin-service health_check timeout"); false }
            }
        }
        None => true,
    };
    let services = vec![ServiceHealthEntry {
        service_name: "admin-service".to_string(),
        ready,
        queue_depth: 0,
        db_pool_usage_ratio: 0.0,
        checked_at_ms: now_ms,
    }];
    HttpResponse::Ok().json(json!({
        "request_id": request_id,
        "services": services,
        "checked_at_ms": now_ms,
        "admin_endpoint": state.config.admin_grpc_endpoint,
    }))
}

// ============================================================================
// 2. BanAccount
// ============================================================================

pub async fn ban_account(
    state: web::Data<AppState>,
    body: web::Json<BanAccountRequestBody>,
) -> HttpResponse {
    if body.account_id.trim().is_empty() {
        return bad_request("missing_account_id", "account_id must not be empty");
    }
    if body.reason.trim().is_empty() {
        return bad_request("missing_reason", "reason must not be empty");
    }
    if body.duration_seconds < 0 {
        return bad_request("invalid_duration", "duration_seconds must be >= 0");
    }

    let request_id = uuid::Uuid::new_v4().to_string();
    let admin_grpc_result = match state.admin_grpc.as_ref() {
        Some(client) => {
            let req = BanAccountRequest {
                request_id: request_id.clone(),
                account_id: body.account_id.clone(),
                reason: body.reason.clone(),
                duration_seconds: body.duration_seconds,
                force_disconnect_session: body.force_disconnect_session,
            };
            tokio::time::timeout(Duration::from_millis(500), client.ban_account(req))
                .await
                .map_err(|_| anyhow::anyhow!("timeout"))
                .and_then(|r| r)
                .ok()
        }
        None => None,
    };
    state.audit_store.append(AuditLogEntry {
        log_id: request_id.clone(),
        admin_id: "system".to_string(),
        action: "ban".to_string(),
        target_id: body.account_id.clone(),
        occurred_at_ms: Utc::now().timestamp_millis(),
    });
    if admin_grpc_result.is_none() {
        tracing::warn!(request_id = %request_id, account_id = %body.account_id, "ban_account fallback");
    }
    let disconnected = admin_grpc_result
        .as_ref()
        .map(|r| r.disconnected_sessions)
        .unwrap_or(body.force_disconnect_session);
    HttpResponse::Accepted().json(json!({
        "status": "queued",
        "op": "ban",
        "request_id": request_id,
        "account_id": body.account_id,
        "degraded": admin_grpc_result.is_none(),
        "force_disconnect_session": body.force_disconnect_session,
        "disconnected_sessions": disconnected,
    }))
}

// ============================================================================
// 3. GrantCompensation
// ============================================================================

pub async fn grant_compensation(
    state: web::Data<AppState>,
    body: web::Json<CompensationRequestBody>,
) -> HttpResponse {
    if body.account_id.trim().is_empty() { return bad_request("missing_account_id", ""); }
    if body.reason.trim().is_empty() { return bad_request("missing_reason", ""); }
    if body.amount <= 0 { return bad_request("invalid_amount", "amount must be > 0"); }
    if body.currency.len() < 3 || body.currency.len() > 4 {
        return bad_request("invalid_currency", "currency length must be 3 or 4");
    }

    let request_id = uuid::Uuid::new_v4().to_string();
    let admin_grpc_result = match state.admin_grpc.as_ref() {
        Some(client) => {
            let req = GrantCompensationRequest {
                request_id: request_id.clone(),
                account_id: body.account_id.clone(),
                amount: body.amount,
                currency: body.currency.clone(),
                reason: body.reason.clone(),
                card_ids: body.card_ids.clone(),
                pack_ids: body.pack_ids.clone(),
            };
            tokio::time::timeout(Duration::from_millis(500), client.grant_compensation(req))
                .await
                .map_err(|_| anyhow::anyhow!("timeout"))
                .and_then(|r| r)
                .ok()
        }
        None => None,
    };
    state.audit_store.append(AuditLogEntry {
        log_id: request_id.clone(),
        admin_id: "system".to_string(),
        action: "grant_compensation".to_string(),
        target_id: body.account_id.clone(),
        occurred_at_ms: Utc::now().timestamp_millis(),
    });
    let cards_granted = admin_grpc_result.as_ref().map(|r| r.cards_granted).unwrap_or(body.card_ids.len() as u32);
    let packs_granted = admin_grpc_result.as_ref().map(|r| r.packs_granted).unwrap_or(body.pack_ids.len() as u32);
    HttpResponse::Accepted().json(json!({
        "status": "queued",
        "op": "compensation",
        "request_id": request_id,
        "account_id": body.account_id,
        "amount": body.amount,
        "currency": body.currency,
        "degraded": admin_grpc_result.is_none(),
        "card_ids": body.card_ids,
        "pack_ids": body.pack_ids,
        "cards_granted": cards_granted,
        "packs_granted": packs_granted,
    }))
}

// ============================================================================
// 4. SetMaintenance
// ============================================================================

pub async fn set_maintenance(
    state: web::Data<AppState>,
    body: web::Json<MaintenanceRequestBody>,
) -> HttpResponse {
    if !ALLOWED_MAINTENANCE_SCOPES.contains(&body.scope.as_str()) {
        return bad_request("invalid_scope", "scope must be cluster|domain|single_node");
    }
    if body.target_id.trim().is_empty() {
        return bad_request("missing_target_id", "");
    }
    if body.ttl_seconds < 0 {
        return bad_request("invalid_ttl", "ttl_seconds must be >= 0");
    }

    let request_id = uuid::Uuid::new_v4().to_string();
    let (propagation_status, applied_mode_flags) = match state.admin_grpc.as_ref() {
        Some(client) => {
            let req = SetMaintenanceRequest {
                request_id: request_id.clone(),
                enable: body.enable,
                scope: body.scope.clone(),
                target_id: body.target_id.clone(),
                ttl_seconds: body.ttl_seconds,
                mode_flags: body.mode_flags,
            };
            match tokio::time::timeout(Duration::from_millis(500), client.set_maintenance(req)).await {
                Ok(Ok(resp)) => (
                    match resp.propagation_status { 1 => "PROPAGATING", 2 => "CONVERGED", _ => "PROPAGATING" }.to_string(),
                    resp.applied_mode_flags,
                ),
                Ok(Err(e)) => { tracing::warn!("set_maintenance failed: {e}"); ("PROPAGATING".to_string(), body.mode_flags) }
                Err(_) => { tracing::warn!("set_maintenance timeout"); ("PROPAGATING".to_string(), body.mode_flags) }
            }
        }
        None => ("PROPAGATING".to_string(), body.mode_flags),
    };
    state.audit_store.append(AuditLogEntry {
        log_id: request_id.clone(),
        admin_id: "system".to_string(),
        action: "set_maintenance".to_string(),
        target_id: body.target_id.clone(),
        occurred_at_ms: Utc::now().timestamp_millis(),
    });
    HttpResponse::Accepted().json(json!({
        "status": "queued",
        "op": "maintenance",
        "request_id": request_id,
        "scope": body.scope,
        "target_id": body.target_id,
        "enable": body.enable,
        "propagation_status": propagation_status,
        "mode_flags": body.mode_flags,
        "applied_mode_flags": applied_mode_flags,
    }))
}

// ============================================================================
// 5. QueryAuditLog
// ============================================================================

pub async fn query_audit(
    state: web::Data<AppState>,
    q: web::Query<QueryAuditLogQuery>,
) -> HttpResponse {
    let request_id = uuid::Uuid::new_v4().to_string();
    let limit = q.limit.unwrap_or(DEFAULT_AUDIT_LIMIT).clamp(1, MAX_AUDIT_LIMIT);
    let cursor = q.cursor.clone().unwrap_or_default();
    let filter_admin = q.filter_admin.clone().unwrap_or_default();
    let filter_action = q.filter_action.clone().unwrap_or_default();
    let audit_type = q.audit_type.as_deref().and_then(parse_audit_type).unwrap_or(AuditType::All as i32);

    let admin_entries: Option<Vec<crate::admin::v1::AuditLogEntry>> = match state.admin_grpc.as_ref() {
        Some(client) => {
            let req = QueryAuditLogRequest {
                request_id: request_id.clone(),
                limit: limit as i32,
                cursor: cursor.clone(),
                filter_admin: filter_admin.clone(),
                filter_action: filter_action.clone(),
                audit_type,
            };
            match tokio::time::timeout(Duration::from_millis(500), client.query_audit_log(req)).await {
                Ok(Ok(resp)) => Some(resp.entries),
                Ok(Err(e)) => { tracing::warn!("query_audit_log failed: {e}"); None }
                Err(_) => { tracing::warn!("query_audit_log timeout"); None }
            }
        }
        None => None,
    };

    if let Some(entries) = admin_entries {
        let out: Vec<_> = entries.into_iter().map(|e| json!({
            "log_id": e.log_id,
            "admin_id": e.admin_id,
            "action": e.action,
            "target_id": e.target_id,
            "occurred_at_ms": e.occurred_at_ms,
        })).collect();
        return HttpResponse::Ok().json(json!({
            "request_id": request_id,
            "entries": out,
            "source": "admin_service",
            "limit": limit,
        }));
    }

    // 降级 InMemory
    let entries = state.audit_store.list_entries(limit).await;
    let out: Vec<_> = entries.into_iter().map(|e| json!({
        "log_id": e.log_id,
        "admin_id": e.admin_id,
        "action": e.action,
        "target_id": e.target_id,
        "occurred_at_ms": e.occurred_at_ms,
    })).collect();
    HttpResponse::Ok().json(json!({
        "request_id": request_id,
        "entries": out,
        "source": "in_memory_fallback",
        "limit": limit,
    }))
}
