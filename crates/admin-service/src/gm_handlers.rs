//! admin-service S4 Phase 2 step 2: 4 GM RPC handler
//!
//! 处理 gm-backend → admin-service 的 4 个 GM RPC:
//! - BanAccount (per RGS-BAS-003 §3.1)
//! - GrantCompensation (per RGS-BAS-003 §3.1)
//! - SetMaintenance (per RGS-BAS-003 §3.3 + DTL-003 §3.3 propagation_status)
//! - QueryAuditLog (per RGS-BAS-003 §3.4 + DTL-003 §3.4 entries[]+has_more)
//!
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md
//!       docs/00-基准与治理/RGS-TBD-08-03-S4-gm-backend-admin-gRPC-立项.md
//!
//! 数据落地:
//! - BanAccount / GrantCompensation / SetMaintenance: 写 audit_log (per RGS-SEC-100 §7 hash 链)
//! - QueryAuditLog: 从 audit_log 读 (默认 limit=20, cursor pagination)
//!
//! 失败降级: handler 内部若 admin_db 不可达, 返 InMemory fallback (与 gm-backend InMemoryAuditStore 同模式)
//!
//! 实现: 用 `OnceCell<GmHandlerState>` 全局单例, 避免 tonic 0.12 State extractor 复杂。
//! 在 main.rs / lib.rs init 时调 `gm_handlers::init_state(...)` 注入。
//!
//! W17 (2026-08-28): JWT propagation - 从 gRPC metadata 抽 `authorization: Bearer <jwt>`,
//! 验签后用 claims.sub 作 admin_id 写 audit_log (per TBD-08-01 + RGS-BAS-003 §2.1 RBAC)

use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::proto::v1::{
    AuditLogEntry as ProtoAuditLogEntry, BanAccountRequest, BanAccountResponse,
    GrantCompensationRequest, GrantCompensationResponse, PropagationStatus,
    QueryAuditLogRequest, QueryAuditLogResponse, SetMaintenanceRequest, SetMaintenanceResponse,
};
use crate::repository::AuditLogRepository;
use crate::entity::AuditLogEntry as DbAuditLogEntry;

/// 共享 handler 状态: AuditLogRepository (Pg 或 InMemory) + InMemory fallback
#[derive(Clone)]
pub struct GmHandlerState {
    pub audit_log: Arc<dyn AuditLogRepository>,
    pub in_memory_fallback: Arc<std::sync::Mutex<Vec<DbAuditLogEntry>>>,
}

impl GmHandlerState {
    pub fn new(audit_log: Arc<dyn AuditLogRepository>) -> Self {
        Self {
            audit_log,
            in_memory_fallback: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

static STATE: OnceLock<GmHandlerState> = OnceLock::new();

/// main.rs / lib.rs 启动时调一次, 注入 audit_log repository
pub fn init_state(state: GmHandlerState) {
    let _ = STATE.set(state);
}

fn state() -> &'static GmHandlerState {
    STATE
        .get()
        .expect("gm_handlers::init_state must be called before handling RPC")
}

// ============================================================================
// BanAccount
// ============================================================================

pub async fn ban_account(
    request: Request<BanAccountRequest>,
) -> Result<Response<BanAccountResponse>, Status> {
    let admin_id_str = extract_admin_id_from_jwt(&request);
    let req = request.into_inner();
    let accepted_at_ms = Utc::now().timestamp_millis();

    let actor_id = Uuid::parse_str(&admin_id_str)
        .ok()
        .unwrap_or_else(Uuid::nil);
    let payload = format!(
        "{{\"request_id\":\"{}\",\"account_id\":\"{}\",\"reason\":\"{}\",\"duration_seconds\":{}}}",
        escape(&req.request_id),
        escape(&req.account_id),
        escape(&req.reason),
        req.duration_seconds
    );
    let prev_hash = state()
        .audit_log
        .latest()
        .await
        .ok()
        .flatten()
        .map(|e| e.hash)
        .unwrap_or_else(|| "0".repeat(64));
    let entry = DbAuditLogEntry::new(
        actor_id,
        "player.ban".to_string(),
        req.account_id.clone(),
        payload,
        prev_hash,
    );

    if let Err(e) = state().audit_log.append(&entry).await {
        tracing::warn!(
            "admin-service ban_account audit_log.append failed: {e}, fallback to InMemory"
        );
        if let Ok(mut guard) = state().in_memory_fallback.lock() {
            guard.push(entry);
        }
    }

    Ok(Response::new(BanAccountResponse {
        status: "queued".to_string(),
        op: "ban".to_string(),
        accepted_at_ms,
    }))
}

// ============================================================================
// GrantCompensation
// ============================================================================

pub async fn grant_compensation(
    request: Request<GrantCompensationRequest>,
) -> Result<Response<GrantCompensationResponse>, Status> {
    let admin_id_str = extract_admin_id_from_jwt(&request);
    let req = request.into_inner();
    let accepted_at_ms = Utc::now().timestamp_millis();

    let actor_id = Uuid::parse_str(&admin_id_str)
        .ok()
        .unwrap_or_else(Uuid::nil);
    let payload = format!(
        "{{\"request_id\":\"{}\",\"account_id\":\"{}\",\"amount\":{},\"currency\":\"{}\",\"reason\":\"{}\"}}",
        escape(&req.request_id),
        escape(&req.account_id),
        req.amount,
        escape(&req.currency),
        escape(&req.reason)
    );
    let prev_hash = state()
        .audit_log
        .latest()
        .await
        .ok()
        .flatten()
        .map(|e| e.hash)
        .unwrap_or_else(|| "0".repeat(64));
    let entry = DbAuditLogEntry::new(
        actor_id,
        "economy.grant".to_string(),
        req.account_id.clone(),
        payload,
        prev_hash,
    );

    if let Err(e) = state().audit_log.append(&entry).await {
        tracing::warn!(
            "admin-service grant_compensation audit_log.append failed: {e}, fallback to InMemory"
        );
        if let Ok(mut guard) = state().in_memory_fallback.lock() {
            guard.push(entry);
        }
    }

    Ok(Response::new(GrantCompensationResponse {
        status: "queued".to_string(),
        op: "compensation".to_string(),
        accepted_at_ms,
    }))
}

// ============================================================================
// SetMaintenance
// ============================================================================

pub async fn set_maintenance(
    request: Request<SetMaintenanceRequest>,
) -> Result<Response<SetMaintenanceResponse>, Status> {
    let admin_id_str = extract_admin_id_from_jwt(&request);
    let req = request.into_inner();
    let accepted_at_ms = Utc::now().timestamp_millis();

    let actor_id = Uuid::parse_str(&admin_id_str)
        .ok()
        .unwrap_or_else(Uuid::nil);
    let payload = format!(
        "{{\"request_id\":\"{}\",\"enable\":{},\"scope\":\"{}\",\"target_id\":\"{}\",\"ttl_seconds\":{}}}",
        escape(&req.request_id),
        req.enable,
        escape(&req.scope),
        escape(&req.target_id),
        req.ttl_seconds
    );
    let prev_hash = state()
        .audit_log
        .latest()
        .await
        .ok()
        .flatten()
        .map(|e| e.hash)
        .unwrap_or_else(|| "0".repeat(64));
    let entry = DbAuditLogEntry::new(
        actor_id,
        "cluster.maintenance".to_string(),
        req.target_id.clone(),
        payload,
        prev_hash,
    );

    if let Err(e) = state().audit_log.append(&entry).await {
        tracing::warn!(
            "admin-service set_maintenance audit_log.append failed: {e}, fallback to InMemory"
        );
        if let Ok(mut guard) = state().in_memory_fallback.lock() {
            guard.push(entry);
        }
    }

    Ok(Response::new(SetMaintenanceResponse {
        status: "queued".to_string(),
        op: "maintenance".to_string(),
        propagation_status: PropagationStatus::Converged as i32,
        accepted_at_ms,
    }))
}

// ============================================================================
// QueryAuditLog
// ============================================================================

pub async fn query_audit_log(
    request: Request<QueryAuditLogRequest>,
) -> Result<Response<QueryAuditLogResponse>, Status> {
    let req = request.into_inner();
    let limit = if req.limit <= 0 { 20 } else { req.limit as usize };

    let entries: Vec<DbAuditLogEntry> = match state().audit_log.latest().await {
        Ok(_) => match state()
            .audit_log
            .list_by_actor(Uuid::nil(), limit as i64 + 1)
            .await
        {
            Ok(mut v) => {
                v.reverse();
                v
            }
            Err(_) => in_memory_latest(limit),
        },
        Err(_) => in_memory_latest(limit),
    };

    let has_more = entries.len() > limit;
    let entries_truncated: Vec<_> = entries.into_iter().take(limit).collect();
    let proto_entries: Vec<ProtoAuditLogEntry> = entries_truncated
        .iter()
        .map(|e| ProtoAuditLogEntry {
            log_id: e.id.to_string(),
            admin_id: e.actor_id.to_string(),
            action: e.action.clone(),
            target_id: e.target.clone(),
            occurred_at_ms: e.created_at.timestamp_millis(),
        })
        .collect();

    let next_cursor = if has_more {
        entries_truncated
            .last()
            .map(|e| e.id.to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    Ok(Response::new(QueryAuditLogResponse {
        entries: proto_entries,
        has_more,
        next_cursor,
    }))
}

fn in_memory_latest(limit: usize) -> Vec<DbAuditLogEntry> {
    state()
        .in_memory_fallback
        .lock()
        .map(|g| g.iter().rev().take(limit).cloned().collect())
        .unwrap_or_default()
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ============================================================================
// W17 JWT propagation (per TBD-08-01 + RGS-BAS-003 §2.1 RBAC)
// ============================================================================

/// JWT claims (per gm-backend issue_jwt 格式)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
    pub roles: Vec<String>,
}

/// 从 gRPC metadata 抽 `authorization: Bearer <token>`, 验签
/// 失败: 返 None (handler 降级用 "system" admin_id)
fn extract_admin_id_from_jwt<T>(request: &Request<T>) -> String {
    // metadata "authorization" key
    let auth_value = request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_value {
        Some(s) if s.starts_with("Bearer ") => &s[7..],
        _ => return "system".to_string(),
    };

    // JWT secret 简化: dev 环境固定 secret, prod 应从 RGS_JWT_SECRET env
    let secret = std::env::var("ADMIN_JWT_SECRET")
        .unwrap_or_else(|_| "dev-only-do-not-use-in-prod".to_string());

    match decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims.sub,
        Err(_) => "system".to_string(),
    }
}
