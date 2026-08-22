//! admin-service 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository
//! 规范：RGS-DTL-019 §3 + ARC-051 COC/CEM + RGS-SEC-100 §7 hash 链
//!
//! 注意：audit_log 表在数据库层禁 UPDATE/DELETE（per RGS-SEC-100 §7）

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{AdminRole, AdminUser, AuditLogEntry};
use crate::Result;

/// AdminUser Repository trait
#[async_trait]
pub trait AdminUserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AdminUser>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<AdminUser>>;
    async fn save(&self, entity: &AdminUser) -> Result<AdminUser>;
    async fn disable(&self, id: Uuid, at: DateTime<Utc>) -> Result<bool>;
    async fn list_active(&self) -> Result<Vec<AdminUser>>;
}

/// AuditLog Repository trait
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// 追加（per RGS-SEC-100 §7 仅 INSERT，禁止 UPDATE/DELETE）
    async fn append(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>>;
    /// 取最近 N 条（per actor_id 过滤）
    async fn list_by_actor(&self, actor_id: Uuid, limit: i64) -> Result<Vec<AuditLogEntry>>;
    /// 取最近一条（用于 hash 链续接）
    async fn latest(&self) -> Result<Option<AuditLogEntry>>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgAdminUserRepository {
    pool: PgPool,
}

impl PgAdminUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_admin_user(row: sqlx::postgres::PgRow) -> AdminUser {
    let role_str: String = row.get("role");
    AdminUser {
        id: row.get("id"),
        username: row.get("username"),
        password_hash: row.get("password_hash"),
        role: parse_role(&role_str),
        domain_scope: row.get("domain_scope"),
        created_at: row.get("created_at"),
        last_login_at: row.get("last_login_at"),
        disabled_at: row.get("disabled_at"),
    }
}

#[async_trait]
impl AdminUserRepository for PgAdminUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AdminUser>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, role, domain_scope, created_at, last_login_at, disabled_at \
             FROM admin_users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_admin_user))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<AdminUser>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, role, domain_scope, created_at, last_login_at, disabled_at \
             FROM admin_users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_admin_user))
    }

    async fn save(&self, entity: &AdminUser) -> Result<AdminUser> {
        sqlx::query(
            "INSERT INTO admin_users \
             (id, username, password_hash, role, domain_scope, created_at, last_login_at, disabled_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                password_hash = EXCLUDED.password_hash, role = EXCLUDED.role, \
                domain_scope = EXCLUDED.domain_scope, last_login_at = EXCLUDED.last_login_at, \
                disabled_at = EXCLUDED.disabled_at",
        )
        .bind(entity.id)
        .bind(&entity.username)
        .bind(&entity.password_hash)
        .bind(role_to_str(entity.role))
        .bind(&entity.domain_scope)
        .bind(entity.created_at)
        .bind(entity.last_login_at)
        .bind(entity.disabled_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn disable(&self, id: Uuid, at: DateTime<Utc>) -> Result<bool> {
        let result = sqlx::query("UPDATE admin_users SET disabled_at = $1 WHERE id = $2")
            .bind(at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_active(&self) -> Result<Vec<AdminUser>> {
        let rows = sqlx::query(
            "SELECT id, username, password_hash, role, domain_scope, created_at, last_login_at, disabled_at \
             FROM admin_users WHERE disabled_at IS NULL ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_admin_user).collect())
    }
}

pub struct PgAuditLogRepository {
    pool: PgPool,
}

impl PgAuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_audit(row: sqlx::postgres::PgRow) -> AuditLogEntry {
    AuditLogEntry {
        id: row.get("id"),
        actor_id: row.get("actor_id"),
        action: row.get("action"),
        target: row.get("target"),
        payload: row.get("payload"),
        prev_hash: row.get("prev_hash"),
        hash: row.get("hash"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn append(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry> {
        // 仅 INSERT，DB 层禁 UPDATE/DELETE（per migration trigger）
        sqlx::query(
            "INSERT INTO audit_log (id, actor_id, action, target, payload, prev_hash, hash, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(entry.id)
        .bind(entry.actor_id)
        .bind(&entry.action)
        .bind(&entry.target)
        .bind(&entry.payload)
        .bind(&entry.prev_hash)
        .bind(&entry.hash)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await?;
        Ok(entry.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>> {
        let row = sqlx::query(
            "SELECT id, actor_id, action, target, payload, prev_hash, hash, created_at \
             FROM audit_log WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_audit))
    }

    async fn list_by_actor(&self, actor_id: Uuid, limit: i64) -> Result<Vec<AuditLogEntry>> {
        let rows = sqlx::query(
            "SELECT id, actor_id, action, target, payload, prev_hash, hash, created_at \
             FROM audit_log WHERE actor_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(actor_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_audit).collect())
    }

    async fn latest(&self) -> Result<Option<AuditLogEntry>> {
        let row = sqlx::query(
            "SELECT id, actor_id, action, target, payload, prev_hash, hash, created_at \
             FROM audit_log ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_audit))
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryAdminUserRepository {
    inner: Mutex<HashMap<Uuid, AdminUser>>,
}

impl InMemoryAdminUserRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAdminUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AdminUserRepository for InMemoryAdminUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AdminUser>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_username(&self, username: &str) -> Result<Option<AdminUser>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|u| u.username == username)
            .cloned())
    }
    async fn save(&self, entity: &AdminUser) -> Result<AdminUser> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn disable(&self, id: Uuid, at: DateTime<Utc>) -> Result<bool> {
        let mut guard = self.inner.lock().unwrap();
        if let Some(u) = guard.get_mut(&id) {
            u.disabled_at = Some(at);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    async fn list_active(&self) -> Result<Vec<AdminUser>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|u| u.disabled_at.is_none())
            .cloned()
            .collect())
    }
}

pub struct InMemoryAuditLogRepository {
    inner: Mutex<HashMap<Uuid, AuditLogEntry>>,
}

impl InMemoryAuditLogRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAuditLogRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditLogRepository for InMemoryAuditLogRepository {
    async fn append(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry> {
        self.inner.lock().unwrap().insert(entry.id, entry.clone());
        Ok(entry.clone())
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_actor(&self, actor_id: Uuid, limit: i64) -> Result<Vec<AuditLogEntry>> {
        let mut all: Vec<AuditLogEntry> = self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.actor_id == actor_id)
            .cloned()
            .collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.created_at));
        all.truncate(limit as usize);
        Ok(all)
    }
    async fn latest(&self) -> Result<Option<AuditLogEntry>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .max_by_key(|e| e.created_at)
            .cloned())
    }
}

// ============================================================================
// helpers
// ============================================================================

fn role_to_str(r: AdminRole) -> &'static str {
    match r {
        AdminRole::SuperAdmin => "super_admin",
        AdminRole::DomainAdmin => "domain_admin",
        AdminRole::Auditor => "auditor",
        AdminRole::Support => "support",
    }
}

fn parse_role(s: &str) -> AdminRole {
    match s {
        "super_admin" => AdminRole::SuperAdmin,
        "domain_admin" => AdminRole::DomainAdmin,
        "auditor" => AdminRole::Auditor,
        _ => AdminRole::Support,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_admin_user_crud() {
        let repo = InMemoryAdminUserRepository::new();
        let u = AdminUser::new(
            "root".to_string(),
            "hash".to_string(),
            AdminRole::SuperAdmin,
        );
        let id = u.id;
        repo.save(&u).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.username, "root");
        assert!(found.is_active());
    }

    #[tokio::test]
    async fn in_memory_audit_log_latest() {
        let repo = InMemoryAuditLogRepository::new();
        let actor = Uuid::new_v4();
        let e1 = AuditLogEntry::new(
            actor,
            "a1".to_string(),
            "t1".to_string(),
            "p1".to_string(),
            "0".repeat(64),
        );
        let e2 = AuditLogEntry::new(
            actor,
            "a2".to_string(),
            "t2".to_string(),
            "p2".to_string(),
            e1.hash.clone(),
        );
        repo.append(&e1).await.unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        repo.append(&e2).await.unwrap();
        let latest = repo.latest().await.unwrap().unwrap();
        assert_eq!(latest.action, "a2");
    }
}
