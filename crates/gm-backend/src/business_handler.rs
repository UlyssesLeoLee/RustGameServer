//! gm-backend 5 GM endpoint 业务 handler(per RGS-BAS-003 §3.1-§3.4 + gm.proto v0.3)
//!
//! ## 范围(per RGS-PLAN-WBS-token-bucket-v0.3 §2.2.1 桶 2a)
//! - 从 lib.rs 移出 5 个 stub handler(health_view / ban_account / grant_compensation
//!   / set_maintenance / query_audit)
//! - 从 HTTP body (Json) / query string (Query) 解析真值
//! - 调用 admin-service gRPC,失败降级 InMemory AuditStore
//! - 字段级校验:缺字段 → 400,业务约束违反 → 400
//!
//! ## 字段 schema(per gm.proto v0.3)
//! - BanAccount: { account_id, reason, duration_seconds? }
//! - GrantCompensation: { account_id, amount, currency, reason }
//! - SetMaintenance: { enable, scope, target_id, ttl_seconds? }
//! - QueryAuditLog: ?limit=1..=100 ?cursor= ?filter_admin= ?filter_action=
//! - HealthView: ?request_id=
//!
//! ## 关联
//! - gm.proto v0.3 commit 8ad815c (WBS v0.3)
//! - admin-service gm_handlers 实装 commit 1e25591 (S4 Phase 2 step 2)
//! - circuit_breaker wire W20 + W23
//!
//! ## 错误响应约定(per BAS-003 §3.1-§3.4 + 业务约束)
//! 400 error key 命名:
//! - ban:    missing_account_id / missing_reason / invalid_duration
//! - comp:   missing_account_id / missing_reason / invalid_amount / invalid_currency
//! - maint:  invalid_scope / missing_target_id / invalid_ttl
//! - audit:  (limit clamp, 非 400)

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

use crate::{
    admin::v1::{
        BanAccountRequest, GrantCompensationRequest, QueryAuditLogRequest, SetMaintenanceRequest,
    },
    AppState, AuditLogEntry, ServiceHealthEntry,
};

// ============================================================================
// Request body / query DTOs(per BAS-003 §3.1-§3.4 + gm.proto v0.3)
// ============================================================================

/// BanAccount 请求 body (per BAS-003 §3.1)
#[derive(Debug, Deserialize)]
pub struct BanAccountRequestBody {
    pub account_id: String,
    pub reason: String,
    /// 0 = 永久(可缺省, 默认 0)
    #[serde(default)]
    pub duration_seconds: i32,
}

/// GrantCompensation 请求 body (per BAS-003 §3.1)
#[derive(Debug, Deserialize)]
pub struct CompensationRequestBody {
    pub account_id: String,
    pub amount: i64,
    pub currency: String,
    pub reason: String,
}

/// SetMaintenance 请求 body (per BAS-003 §3.3)
#[derive(Debug, Deserialize)]
pub struct MaintenanceRequestBody {
    pub enable: bool,
    pub scope: String,
    pub target_id: String,
    /// 0 = 永久(可缺省, 默认 0)
    #[serde(default)]
    pub ttl_seconds: i32,
}

/// QueryAuditLog query string (per BAS-003 §3.4)
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
}

/// HealthView query string (可选 request_id, 默认 uuid)
#[derive(Debug, Deserialize)]
pub struct HealthViewQuery {
    #[serde(default)]
    pub request_id: Option<String>,
}

// ============================================================================
// 业务常量(per gm.proto v0.3)
// ============================================================================

/// QueryAuditLogRequest.limit 默认值(per gm.proto v0.3)
pub const DEFAULT_AUDIT_LIMIT: usize = 20;
/// QueryAuditLogRequest.limit 上限(防爆)
pub const MAX_AUDIT_LIMIT: usize = 100;
/// SetMaintenance.scope 允许值
pub const ALLOWED_MAINTENANCE_SCOPES: &[&str] = &["cluster", "domain", "single_node"];

// ============================================================================
// 校验错误响应 helper
// ============================================================================

/// 400 响应, error key 用于客户端分支
fn bad_request(error: &str, detail: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": error, "detail": detail})),
    )
}

// ============================================================================
// 1. HealthView: services[] + checked_at_ms + admin_endpoint
// ============================================================================

/// `services[]` 5 子字段(per F8 v0.2 实装 + S4 Phase 2 step 1 admin-service gRPC)
/// 行为:
/// - admin_grpc.is_some() AND gRPC HealthCheck 500ms 内 Ok → ready=true
/// - admin_grpc.is_some() AND gRPC 失败/超时 → ready=false + tracing::warn!
/// - admin_grpc.is_none() (测试 / 连接初始化失败) → ready=true (兼容 v0.2 stub 行为)
/// W26 桶 2a: 接受可选 request_id query string, 默认 uuid
pub async fn health_view(
    State(s): State<AppState>,
    Query(q): Query<HealthViewQuery>,
) -> impl IntoResponse {
    let request_id = q.request_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let now_ms = Utc::now().timestamp_millis();
    let ready = match s.admin_grpc.as_ref() {
        Some(client) => match tokio::time::timeout(
            Duration::from_millis(500),
            client.health_check(),
        )
        .await
        {
            Ok(Ok(())) => true,
            Ok(Err(e)) => {
                tracing::warn!("admin-service health_check failed: {e}");
                false
            }
            Err(_) => {
                tracing::warn!("admin-service health_check timeout (500ms)");
                false
            }
        },
        None => true, // 测试 / 初始化失败时保持 stub 行为
    };
    let services = vec![ServiceHealthEntry {
        service_name: "admin-service".to_string(),
        ready,
        queue_depth: 0,
        db_pool_usage_ratio: 0.0,
        checked_at_ms: now_ms,
    }];
    (
        StatusCode::OK,
        Json(json!({
            "request_id": request_id,
            "services": services,
            "checked_at_ms": now_ms,
            "admin_endpoint": s.config.admin_grpc_endpoint,
        })),
    )
}

// ============================================================================
// 2. BanAccount: 解析 body, 调 admin-service gRPC, 写 audit_log
// ============================================================================

/// BanAccount handler(per gm.proto v0.3)
/// - account_id 空 → 400 (missing_account_id)
/// - reason 空 → 400 (missing_reason)
/// - duration_seconds < 0 → 400 (invalid_duration)
/// - admin_grpc 失败/超时 → 降级 InMemory + 202 (degraded, 仍记录审计)
pub async fn ban_account(
    State(s): State<AppState>,
    Json(body): Json<BanAccountRequestBody>,
) -> impl IntoResponse {
    // 字段级校验
    if body.account_id.trim().is_empty() {
        return bad_request("missing_account_id", "account_id must not be empty");
    }
    if body.reason.trim().is_empty() {
        return bad_request("missing_reason", "reason must not be empty");
    }
    if body.duration_seconds < 0 {
        return bad_request("invalid_duration", "duration_seconds must be >= 0 (0 = permanent)");
    }

    let request_id = uuid::Uuid::new_v4().to_string();
    let admin_grpc_result = match s.admin_grpc.as_ref() {
        Some(client) => {
            let req = BanAccountRequest {
                request_id: request_id.clone(),
                account_id: body.account_id.clone(),
                reason: body.reason.clone(),
                duration_seconds: body.duration_seconds,
            };
            tokio::time::timeout(Duration::from_millis(500), client.ban_account(req))
                .await
                .map_err(|_| anyhow::anyhow!("admin-service ban_account timeout"))
                .and_then(|r| r)
                .ok()
        }
        None => None,
    };
    // 无论成功/失败都写本地 audit_log (gm-backend 端 stub cache)
    s.audit_store.append(AuditLogEntry {
        log_id: request_id.clone(),
        admin_id: "system".to_string(),
        action: "ban".to_string(),
        target_id: body.account_id.clone(),
        occurred_at_ms: Utc::now().timestamp_millis(),
    });
    if admin_grpc_result.is_none() {
        tracing::warn!(
            request_id = %request_id,
            account_id = %body.account_id,
            "admin-service ban_account unavailable, local InMemory fallback used"
        );
    }
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "queued",
            "op": "ban",
            "request_id": request_id,
            "account_id": body.account_id,
            "degraded": admin_grpc_result.is_none(),
        })),
    )
}

// ============================================================================
// 3. GrantCompensation: amount > 0, currency 3-4 字符
// ============================================================================

/// GrantCompensation handler(per gm.proto v0.3)
/// - account_id / reason 空 → 400
/// - amount ≤ 0 → 400 (invalid_amount)
/// - currency 长度 ∉ [3, 4] → 400 (invalid_currency)
/// - admin_grpc 失败 → 降级 InMemory + 202
pub async fn grant_compensation(
    State(s): State<AppState>,
    Json(body): Json<CompensationRequestBody>,
) -> impl IntoResponse {
    // 字段级校验
    if body.account_id.trim().is_empty() {
        return bad_request("missing_account_id", "account_id must not be empty");
    }
    if body.reason.trim().is_empty() {
        return bad_request("missing_reason", "reason must not be empty");
    }
    if body.amount <= 0 {
        return bad_request("invalid_amount", "amount must be > 0");
    }
    if body.currency.len() < 3 || body.currency.len() > 4 {
        return bad_request(
            "invalid_currency",
            "currency length must be 3 or 4 (e.g. USD, CNY, GOLD)",
        );
    }

    let request_id = uuid::Uuid::new_v4().to_string();
    let admin_grpc_result = match s.admin_grpc.as_ref() {
        Some(client) => {
            let req = GrantCompensationRequest {
                request_id: request_id.clone(),
                account_id: body.account_id.clone(),
                amount: body.amount,
                currency: body.currency.clone(),
                reason: body.reason.clone(),
            };
            tokio::time::timeout(Duration::from_millis(500), client.grant_compensation(req))
                .await
                .map_err(|_| anyhow::anyhow!("admin-service grant_compensation timeout"))
                .and_then(|r| r)
                .ok()
        }
        None => None,
    };
    s.audit_store.append(AuditLogEntry {
        log_id: request_id.clone(),
        admin_id: "system".to_string(),
        action: "grant_compensation".to_string(),
        target_id: body.account_id.clone(),
        occurred_at_ms: Utc::now().timestamp_millis(),
    });
    if admin_grpc_result.is_none() {
        tracing::warn!(
            request_id = %request_id,
            account_id = %body.account_id,
            "admin-service grant_compensation unavailable, local InMemory fallback used"
        );
    }
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "queued",
            "op": "compensation",
            "request_id": request_id,
            "account_id": body.account_id,
            "amount": body.amount,
            "currency": body.currency,
            "degraded": admin_grpc_result.is_none(),
        })),
    )
}

// ============================================================================
// 4. SetMaintenance: scope ∈ {cluster, domain, single_node}
// ============================================================================

/// SetMaintenance handler(per gm.proto v0.3 + DTL-003 §3.3 propagation_status)
/// - scope ∉ 3 选 1 → 400 (invalid_scope)
/// - target_id 空 → 400
/// - ttl_seconds < 0 → 400
/// - 响应保留 propagation_status 字段(PROPAGATING / CONVERGED)
pub async fn set_maintenance(
    State(s): State<AppState>,
    Json(body): Json<MaintenanceRequestBody>,
) -> impl IntoResponse {
    // 字段级校验
    if !ALLOWED_MAINTENANCE_SCOPES.contains(&body.scope.as_str()) {
        return bad_request(
            "invalid_scope",
            "scope must be cluster|domain|single_node",
        );
    }
    if body.target_id.trim().is_empty() {
        return bad_request("missing_target_id", "target_id must not be empty");
    }
    if body.ttl_seconds < 0 {
        return bad_request("invalid_ttl", "ttl_seconds must be >= 0 (0 = permanent)");
    }

    let request_id = uuid::Uuid::new_v4().to_string();
    let propagation_status = match s.admin_grpc.as_ref() {
        Some(client) => {
            let req = SetMaintenanceRequest {
                request_id: request_id.clone(),
                enable: body.enable,
                scope: body.scope.clone(),
                target_id: body.target_id.clone(),
                ttl_seconds: body.ttl_seconds,
            };
            match tokio::time::timeout(Duration::from_millis(500), client.set_maintenance(req))
                .await
            {
                Ok(Ok(resp)) => match resp.propagation_status {
                    1 => "PROPAGATING",
                    2 => "CONVERGED",
                    _ => "PROPAGATING",
                }
                .to_string(),
                Ok(Err(e)) => {
                    tracing::warn!("admin-service set_maintenance failed: {e}");
                    "PROPAGATING".to_string()
                }
                Err(_) => {
                    tracing::warn!("admin-service set_maintenance timeout");
                    "PROPAGATING".to_string()
                }
            }
        }
        None => "PROPAGATING".to_string(),
    };
    s.audit_store.append(AuditLogEntry {
        log_id: request_id.clone(),
        admin_id: "system".to_string(),
        action: "set_maintenance".to_string(),
        target_id: body.target_id.clone(),
        occurred_at_ms: Utc::now().timestamp_millis(),
    });
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "queued",
            "op": "maintenance",
            "request_id": request_id,
            "scope": body.scope,
            "target_id": body.target_id,
            "enable": body.enable,
            "propagation_status": propagation_status,
        })),
    )
}

// ============================================================================
// 5. QueryAuditLog: ?limit=1..=100 + cursor/filter_admin/filter_action
// ============================================================================

/// QueryAuditLog handler(per gm.proto v0.3 + DTL-003 §3.4 entries[] + has_more + next_cursor)
/// - limit 0 → 默认 20;limit > 100 → clamp 到 100(防爆, 不返 400)
/// - 优先返回 admin-service gRPC 真实 entries,失败降级 InMemory AuditStore
pub async fn query_audit(
    State(s): State<AppState>,
    Query(q): Query<QueryAuditLogQuery>,
) -> impl IntoResponse {
    let request_id = uuid::Uuid::new_v4().to_string();
    let limit = q.limit.unwrap_or(DEFAULT_AUDIT_LIMIT).clamp(1, MAX_AUDIT_LIMIT);
    let cursor = q.cursor.unwrap_or_default();
    let filter_admin = q.filter_admin.unwrap_or_default();
    let filter_action = q.filter_action.unwrap_or_default();

    // 尝试调 admin-service gRPC
    let admin_entries: Option<Vec<crate::admin::v1::AuditLogEntry>> = match s.admin_grpc.as_ref() {
        Some(client) => {
            let req = QueryAuditLogRequest {
                request_id: request_id.clone(),
                limit: limit as i32,
                cursor: cursor.clone(),
                filter_admin: filter_admin.clone(),
                filter_action: filter_action.clone(),
            };
            match tokio::time::timeout(Duration::from_millis(500), client.query_audit_log(req))
                .await
            {
                Ok(Ok(resp)) => Some(resp.entries),
                _ => None,
            }
        }
        None => None,
    };
    // 优先用 admin-service 返回, 失败降级本地 InMemory
    if let Some(entries) = admin_entries {
        let proto_entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                json!({
                    "log_id": e.log_id,
                    "admin_id": e.admin_id,
                    "action": e.action,
                    "target_id": e.target_id,
                    "occurred_at_ms": e.occurred_at_ms,
                })
            })
            .collect();
        return (
            StatusCode::OK,
            Json(json!({
                "request_id": request_id,
                "entries": proto_entries,
                "has_more": proto_entries.len() >= limit,
                "next_cursor": null,
            })),
        );
    }
    // 降级路径
    let entries = s.audit_store.list_entries(limit).await;
    let has_more = entries.len() >= limit;
    let proto_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            json!({
                "log_id": e.log_id,
                "admin_id": e.admin_id,
                "action": e.action,
                "target_id": e.target_id,
                "occurred_at_ms": e.occurred_at_ms,
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(json!({
            "request_id": request_id,
            "entries": proto_entries,
            "has_more": has_more,
            "next_cursor": null,
        })),
    )
}
