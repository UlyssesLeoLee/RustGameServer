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

use crate::entity::{AdminUser, AuditLogEntry as DbAuditLogEntry};
use crate::error::Error;
use crate::proto::v1::{
    AuditLogEntry as ProtoAuditLogEntry, BanAccountRequest, BanAccountResponse,
    GrantCompensationRequest, GrantCompensationResponse, PropagationStatus, QueryAuditLogRequest,
    QueryAuditLogResponse, SetMaintenanceRequest, SetMaintenanceResponse,
};
use crate::repository::{AdminUserRepository, AuditLogRepository};

/// 共享 handler 状态: AuditLogRepository (Pg 或 InMemory) + AdminUserRepository
/// (Q1 RBAC: 需根据 sub 查 admin, 校验 can_admin_domain) + InMemory fallback
#[derive(Clone)]
pub struct GmHandlerState {
    pub audit_log: Arc<dyn AuditLogRepository>,
    pub users: Arc<dyn AdminUserRepository>,
    pub in_memory_fallback: Arc<std::sync::Mutex<Vec<DbAuditLogEntry>>>,
}

impl GmHandlerState {
    pub fn new(
        audit_log: Arc<dyn AuditLogRepository>,
        users: Arc<dyn AdminUserRepository>,
    ) -> Self {
        Self {
            audit_log,
            users,
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
    // Q1 RBAC: handler 入口先校验 actor 有权执行 player.ban (per v0.2 §Q1)
    let admin = extract_admin_user_from_jwt(&request).await?;
    require_coc_role(&admin, "player.ban").map_err(Into::<tonic::Status>::into)?;

    let req = request.into_inner();
    let accepted_at_ms = Utc::now().timestamp_millis();

    let actor_id = admin.id;
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
        // v0.4 (per RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07): 默认 false (未强制踢出对战)
        disconnected_sessions: false,
    }))
}

// ============================================================================
// GrantCompensation
// ============================================================================

pub async fn grant_compensation(
    request: Request<GrantCompensationRequest>,
) -> Result<Response<GrantCompensationResponse>, Status> {
    // Q1 RBAC: handler 入口先校验 actor 有权执行 economy.grant (per v0.2 §Q1)
    let admin = extract_admin_user_from_jwt(&request).await?;
    require_coc_role(&admin, "economy.grant").map_err(Into::<tonic::Status>::into)?;

    let req = request.into_inner();
    let accepted_at_ms = Utc::now().timestamp_millis();

    let actor_id = admin.id;
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
        // v0.4 (per RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07): 默认 0 (未发卡牌/卡包)
        cards_granted: 0,
        packs_granted: 0,
    }))
}

// ============================================================================
// SetMaintenance
// ============================================================================

pub async fn set_maintenance(
    request: Request<SetMaintenanceRequest>,
) -> Result<Response<SetMaintenanceResponse>, Status> {
    // Q1 RBAC: handler 入口先校验 actor 有权执行 cluster.maintenance (per v0.2 §Q1)
    // cluster 域仅 SuperAdmin 有权 (per AdminUser::can_admin_domain: DomainAdmin
    // 只匹配 domain_scope=cluster, 但默认 GM 操作使用 SuperAdmin 路径).
    let admin = extract_admin_user_from_jwt(&request).await?;
    require_coc_role(&admin, "cluster.maintenance").map_err(Into::<tonic::Status>::into)?;

    let req = request.into_inner();
    let accepted_at_ms = Utc::now().timestamp_millis();

    let actor_id = admin.id;
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
        // v0.4 (per RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07): 默认 0 (无维护模式)
        applied_mode_flags: 0,
    }))
}

// ============================================================================
// QueryAuditLog
// ============================================================================

pub async fn query_audit_log(
    request: Request<QueryAuditLogRequest>,
) -> Result<Response<QueryAuditLogResponse>, Status> {
    let req = request.into_inner();
    let limit = if req.limit <= 0 {
        20
    } else {
        req.limit as usize
    };

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
            // v0.4 (per RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07): 默认 All (本地 store 不分类)
            audit_type: crate::proto::v1::AuditType::All as i32,
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
        // v0.4 (per RGS-DDD-CARD-9DEC-2026-08-29 DEC-038-07): 默认 Unspecified (待 filter 解析)
        applied_audit_type: crate::proto::v1::AuditType::Unspecified as i32,
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

/// Q1 RBAC: GM 指令 action → 目标域映射 (per RGS-ARC-051 §COC command namespace)
///
/// 与 `integration_gm_command_permission_chain.rs` IT 中既有定义保持一致 (per
/// v0.2 §Q1 "IT 为主, UT 补 role_matrix" 决策). handler 入口处按此映射做
/// can_admin_domain 校验.
pub fn action_target_domain(action: &str) -> &'static str {
    match action {
        "player.ban" | "player.unban" | "player.mute" | "player.promote" => "player",
        "economy.grant" | "economy.adjust" => "economy",
        "match.kick" | "match.end" => "match",
        "guild.dissolve" | "guild.promote" => "social",
        "cluster.maintenance" | "cluster.shutdown" => "cluster",
        _ => "unknown",
    }
}

/// Q1 RBAC 入口 helper: 校验 admin 是否有权执行指定 action.
///
/// 校验顺序 (per ARC-051 + RGS-SEC-100 §7):
/// 1. 启用状态 (`is_active`) → 不启用返 `AdminSessionExpired` (session 已失效)
/// 2. 域权限 (`can_admin_domain(target_domain(action))`) → 不足返 `COCRoleRequired`
///
/// 返回 `Result<(), Error>`: 调用方决定如何映射为 `tonic::Status`.
pub fn require_coc_role(admin: &AdminUser, action: &str) -> Result<(), Error> {
    if !admin.is_active() {
        return Err(Error::AdminSessionExpired(admin.username.clone()));
    }
    let required_domain = action_target_domain(action);
    if !admin.can_admin_domain(required_domain) {
        return Err(Error::COCRoleRequired {
            required: format!("role with '{}' scope", required_domain),
            actual: format!("{:?}", admin.role),
        });
    }
    Ok(())
}

/// Q1 RBAC: 从 gRPC metadata 抽 JWT, 验签后查 AdminUser 实体.
///
/// **fail-closed**: 任何一步失败 (无 metadata / 无 Bearer / JWT 验签失败 /
/// sub 非 UUID / admin 不存在) 均返 `Err(tonic::Status::unauthenticated(...))`,
/// 不降级到 "system" — 这是 Q1 关键修复点 (per v0.2 §Q1).
///
/// Returns `Result<AdminUser, tonic::Status>`.
pub async fn extract_admin_user_from_jwt<T>(
    request: &Request<T>,
) -> Result<AdminUser, Status> {
    let auth_value = request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_value {
        Some(s) if s.starts_with("Bearer ") => &s[7..],
        _ => {
            return Err(Status::unauthenticated(
                "missing or malformed authorization metadata",
            ));
        }
    };

    // JWT secret 简化: dev 环境固定 secret, prod 应从 ADMIN_JWT_SECRET env
    let secret = std::env::var("ADMIN_JWT_SECRET")
        .unwrap_or_else(|_| "dev-only-do-not-use-in-prod".to_string());

    let claims = match decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(e) => {
            return Err(Status::unauthenticated(format!("invalid jwt: {e}")));
        }
    };

    // sub 必须是 UUID (admin.id 类型约束)
    let admin_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        Status::unauthenticated(format!("invalid sub in jwt: {}", claims.sub))
    })?;

    // 查 AdminUser; 不存在视为 unauthenticated
    let admin = state()
        .users
        .find_by_id(admin_id)
        .await
        .map_err(|e| Status::internal(format!("admin lookup failed: {e}")))?
        .ok_or_else(|| Status::unauthenticated(format!("admin {} not found", admin_id)))?;

    Ok(admin)
}

/// 兼容层: 保留旧签名以供未来调用; 但 Q1 改造后三个 handler 不再使用本函数.
#[allow(dead_code)]
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

// ============================================================================
// UT 子代理 (2026-08-31 v3 P1 fix Q1): gm_handlers role_matrix
// 3 roles × 3 actions = 9 cases (per v0.2 §Q1 "UT 补 role_matrix" 决策)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::AdminRole;
    use crate::error::Error;

    /// helper: 建启用 + 指定 role + scope 的 AdminUser
    fn admin(role: AdminRole, scope: Option<&str>) -> AdminUser {
        let mut u = AdminUser::new("u".to_string(), "h".to_string(), role);
        u.domain_scope = scope.map(|s| s.to_string());
        u
    }

    // --- SuperAdmin: 3 actions 全 ok (per ARC-051 COC) ---
    #[test]
    fn rbac_super_admin_can_ban_player() {
        let a = admin(AdminRole::SuperAdmin, None);
        assert!(require_coc_role(&a, "player.ban").is_ok());
    }
    #[test]
    fn rbac_super_admin_can_grant_economy() {
        let a = admin(AdminRole::SuperAdmin, None);
        assert!(require_coc_role(&a, "economy.grant").is_ok());
    }
    #[test]
    fn rbac_super_admin_can_set_cluster_maintenance() {
        let a = admin(AdminRole::SuperAdmin, None);
        assert!(require_coc_role(&a, "cluster.maintenance").is_ok());
    }

    // --- DomainAdmin(player): 仅 player.ban ok ---
    #[test]
    fn rbac_domain_admin_player_can_ban_player() {
        let a = admin(AdminRole::DomainAdmin, Some("player"));
        assert!(require_coc_role(&a, "player.ban").is_ok());
    }
    #[test]
    fn rbac_domain_admin_player_cannot_grant_economy() {
        let a = admin(AdminRole::DomainAdmin, Some("player"));
        let err = require_coc_role(&a, "economy.grant").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }), "got: {err:?}");
    }
    #[test]
    fn rbac_domain_admin_player_cannot_set_cluster_maintenance() {
        let a = admin(AdminRole::DomainAdmin, Some("player"));
        let err = require_coc_role(&a, "cluster.maintenance").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }

    // --- DomainAdmin(economy): 仅 economy.grant ok ---
    #[test]
    fn rbac_domain_admin_economy_cannot_ban_player() {
        let a = admin(AdminRole::DomainAdmin, Some("economy"));
        let err = require_coc_role(&a, "player.ban").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_domain_admin_economy_can_grant_economy() {
        let a = admin(AdminRole::DomainAdmin, Some("economy"));
        assert!(require_coc_role(&a, "economy.grant").is_ok());
    }
    #[test]
    fn rbac_domain_admin_economy_cannot_set_cluster_maintenance() {
        let a = admin(AdminRole::DomainAdmin, Some("economy"));
        let err = require_coc_role(&a, "cluster.maintenance").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }

    // --- DomainAdmin(cluster): 仅 cluster.maintenance ok ---
    #[test]
    fn rbac_domain_admin_cluster_cannot_ban_player() {
        let a = admin(AdminRole::DomainAdmin, Some("cluster"));
        let err = require_coc_role(&a, "player.ban").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_domain_admin_cluster_cannot_grant_economy() {
        let a = admin(AdminRole::DomainAdmin, Some("cluster"));
        let err = require_coc_role(&a, "economy.grant").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_domain_admin_cluster_can_set_cluster_maintenance() {
        let a = admin(AdminRole::DomainAdmin, Some("cluster"));
        assert!(require_coc_role(&a, "cluster.maintenance").is_ok());
    }

    // --- Auditor: 全部拒 (无域权限) ---
    #[test]
    fn rbac_auditor_cannot_ban_player() {
        let a = admin(AdminRole::Auditor, None);
        let err = require_coc_role(&a, "player.ban").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_auditor_cannot_grant_economy() {
        let a = admin(AdminRole::Auditor, None);
        let err = require_coc_role(&a, "economy.grant").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_auditor_cannot_set_cluster_maintenance() {
        let a = admin(AdminRole::Auditor, None);
        let err = require_coc_role(&a, "cluster.maintenance").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }

    // --- Support: 全部拒 (无域权限) ---
    #[test]
    fn rbac_support_cannot_ban_player() {
        let a = admin(AdminRole::Support, None);
        let err = require_coc_role(&a, "player.ban").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_support_cannot_grant_economy() {
        let a = admin(AdminRole::Support, None);
        let err = require_coc_role(&a, "economy.grant").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }
    #[test]
    fn rbac_support_cannot_set_cluster_maintenance() {
        let a = admin(AdminRole::Support, None);
        let err = require_coc_role(&a, "cluster.maintenance").unwrap_err();
        assert!(matches!(err, Error::COCRoleRequired { .. }));
    }

    // --- 已停用 admin: session expired 优先于 RBAC ---
    #[test]
    fn rbac_disabled_admin_returns_session_expired() {
        let mut a = admin(AdminRole::SuperAdmin, None);
        a.disabled_at = Some(Utc::now());
        let err = require_coc_role(&a, "player.ban").unwrap_err();
        assert!(
            matches!(err, Error::AdminSessionExpired(_)),
            "已停用 SuperAdmin 应被 session expired 拒, got {err:?}"
        );
    }

    // --- action_target_domain 映射正确性 (per ARC-051 §COC) ---
    #[test]
    fn action_target_domain_mapping() {
        assert_eq!(action_target_domain("player.ban"), "player");
        assert_eq!(action_target_domain("player.unban"), "player");
        assert_eq!(action_target_domain("economy.grant"), "economy");
        assert_eq!(action_target_domain("economy.adjust"), "economy");
        assert_eq!(action_target_domain("match.kick"), "match");
        assert_eq!(action_target_domain("guild.dissolve"), "social");
        assert_eq!(action_target_domain("cluster.maintenance"), "cluster");
        assert_eq!(action_target_domain("unknown.action"), "unknown");
    }

    // --- 未知 action: SuperAdmin 因 can_admin_domain == true 一律 ok,
    //     其他角色因 "unknown" 不匹配 domain_scope 而被拒 ---
    // 注: 实际 handler 调用 require_coc_role 时传的是具体已知 action
    // (player.ban / economy.grant / cluster.maintenance), 不会出现 unknown;
    // 此测试仅锁定 action_target_domain 与 can_admin_domain 的交互语义.
    #[test]
    fn rbac_unknown_action_super_admin_passes_others_rejected() {
        // SuperAdmin: can_admin_domain = true → 通过 (per ARC-051 全域)
        let sa = admin(AdminRole::SuperAdmin, None);
        assert!(
            require_coc_role(&sa, "mystery.action").is_ok(),
            "SuperAdmin 对任何 action 应通过 (per ARC-051)"
        );
        // 其他角色: "unknown" 不匹配任何 domain_scope → COCRoleRequired
        for role in [
            AdminRole::DomainAdmin,
            AdminRole::Auditor,
            AdminRole::Support,
        ] {
            let a = admin(role, None);
            let err = require_coc_role(&a, "mystery.action").unwrap_err();
            assert!(matches!(err, Error::COCRoleRequired { .. }));
        }
    }
}
