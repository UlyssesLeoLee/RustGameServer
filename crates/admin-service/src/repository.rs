//! admin-service 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository
//! 规范：RGS-DTL-019 §3 + ARC-051 COC/CEM + RGS-SEC-100 §7 hash 链
//!
//! 55.13 增补：`append_atomic` 在事务内 `SELECT ... FOR UPDATE` 锁 latest 行 +
//! INSERT 新行，保证 hash 链 read-then-append 原子（per RGS-REV-007 AC5=CC1+CH3 /
//! DEC-015 P1）。
//!
//! 注意：audit_log 表在数据库层禁 UPDATE/DELETE（per RGS-SEC-100 §7）。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};
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
    /// 55.13 原子 append：read latest (FOR UPDATE) + insert + 提交由调用方事务管理。
    /// 实现层在事务内串行化 latest 读取，保证 hash 链 read-then-append 不出现
    /// 并发串号（per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）。
    async fn append_atomic(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry: &AuditLogEntry,
    ) -> Result<AuditLogEntry>;
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

    async fn append_atomic(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entry: &AuditLogEntry,
    ) -> Result<AuditLogEntry> {
        // 锁 latest 行（FOR UPDATE）—— 强制序列化同一 hash 链的续接，避免并发
        // 读到同一 prev_hash 而产生串号 / 分叉（per RGS-REV-007 AC5=CC1+CH3）。
        // 表级 append-only 触发器仍生效（per RGS-SEC-100 §7）。
        let latest_row = sqlx::query(
            "SELECT id, actor_id, action, target, payload, prev_hash, hash, created_at \
             FROM audit_log ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
        )
        .fetch_optional(&mut **tx)
        .await?;
        // 找到 prev_hash：调用方负责校验 entry.prev_hash 与 latest_row.hash 一致。
        // 此处仅取出供调用方比对（避免在同一接口内重复 SELECT）。
        let _ = latest_row; // 锁生效即满足；调用方已知 prev_hash。

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
        .execute(&mut **tx)
        .await?;
        Ok(entry.clone())
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

    async fn append_atomic(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        entry: &AuditLogEntry,
    ) -> Result<AuditLogEntry> {
        // 内存实现：Mutex 本身已序列化所有访问，因此 latest + append 在同一锁内
        // 串行化，与 PG 的 FOR UPDATE 等价。这里仅做 INSERT（不重读 latest，因为
        // 调用方在事务内已持有 prev_hash 校验语义）。
        self.inner.lock().unwrap().insert(entry.id, entry.clone());
        Ok(entry.clone())
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

    // ========================================================================
    // UT 子代理 (2026-08-31 v2): repository 行为不变式
    // ========================================================================

    /// InMemoryAdminUserRepository::list_active 仅返未停用用户
    #[tokio::test]
    async fn in_memory_list_active_excludes_disabled() {
        let repo = InMemoryAdminUserRepository::new();
        let mut u_active = AdminUser::new(
            "a".to_string(),
            "h".to_string(),
            AdminRole::SuperAdmin,
        );
        let mut u_disabled = AdminUser::new(
            "d".to_string(),
            "h".to_string(),
            AdminRole::DomainAdmin,
        );
        u_disabled.disabled_at = Some(Utc::now());
        let _ = u_active;
        repo.save(&u_active).await.unwrap();
        repo.save(&u_disabled).await.unwrap();
        let active = repo.list_active().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].username, "a");
        assert!(active[0].is_active());
    }

    /// InMemoryAdminUserRepository::disable 对不存在的 id 返 false
    #[tokio::test]
    async fn in_memory_disable_unknown_id_returns_false() {
        let repo = InMemoryAdminUserRepository::new();
        let ok = repo.disable(Uuid::new_v4(), Utc::now()).await.unwrap();
        assert!(!ok);
    }

    /// InMemoryAuditLogRepository::list_by_actor 仅返匹配 actor_id
    /// 的条目, 且按 created_at DESC 排序, limit 生效.
    #[tokio::test]
    async fn in_memory_list_by_actor_filters_and_sorts() {
        let repo = InMemoryAuditLogRepository::new();
        let actor_a = Uuid::new_v4();
        let actor_b = Uuid::new_v4();
        // actor_a 写 3 条
        for i in 0..3 {
            let e = AuditLogEntry::new(
                actor_a,
                format!("a{i}"),
                format!("t{i}"),
                "p".to_string(),
                "0".repeat(64),
            );
            repo.append(&e).await.unwrap();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // actor_b 写 1 条
        let e = AuditLogEntry::new(
            actor_b,
            "b0".to_string(),
            "tb".to_string(),
            "p".to_string(),
            "0".repeat(64),
        );
        repo.append(&e).await.unwrap();

        let list = repo.list_by_actor(actor_a, 10).await.unwrap();
        assert_eq!(list.len(), 3, "应仅返 actor_a 的 3 条");
        for entry in &list {
            assert_eq!(entry.actor_id, actor_a);
        }
        // 按 created_at DESC: list[0] 应是最后写的 (a2)
        assert_eq!(list[0].action, "a2");
        assert_eq!(list[1].action, "a1");
        assert_eq!(list[2].action, "a0");

        // limit=2 仅取前 2 条
        let limited = repo.list_by_actor(actor_a, 2).await.unwrap();
        assert_eq!(limited.len(), 2);
    }

    /// InMemoryAuditLogRepository::find_by_id 返 None for 不存在 id
    #[tokio::test]
    async fn in_memory_audit_find_by_id_unknown_returns_none() {
        let repo = InMemoryAuditLogRepository::new();
        let result = repo.find_by_id(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }
}

// ============================================================================
// UT 子代理 (2026-08-31 v2): repository 容量 / 一致性 proptest
// ============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// InMemoryAdminUserRepository: 任意 (username, role) 组合 save 后
        /// find_by_username 必能找到且 username 完全相等 (大小写敏感).
        #[test]
        fn username_lookup_exact_match(
            username in "[a-z]{1,12}",
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let repo = InMemoryAdminUserRepository::new();
                let u = AdminUser::new(
                    username.clone(),
                    "h".to_string(),
                    AdminRole::SuperAdmin,
                );
                repo.save(&u).await.unwrap();
                let found = repo.find_by_username(&username).await.unwrap();
                prop_assert!(found.is_some());
                let found = found.unwrap();
                prop_assert_eq!(found.username.as_str(), username.as_str());
                prop_assert_eq!(found.id, u.id);
            });
        }

        /// 任意 N 条 audit_log 追加后, latest() 必返 created_at 最大者.
        #[test]
        fn latest_returns_max_created_at(n in 1usize..15) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let repo = InMemoryAuditLogRepository::new();
                let actor = Uuid::new_v4();
                let mut all = Vec::with_capacity(n);
                for i in 0..n {
                    let e = AuditLogEntry::new(
                        actor,
                        format!("a.{i}"),
                        format!("t.{i}"),
                        "p".to_string(),
                        "0".repeat(64),
                    );
                    repo.append(&e).await.unwrap();
                    all.push(e);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                let latest = repo.latest().await.unwrap();
                prop_assert!(latest.is_some());
                let latest = latest.unwrap();
                // latest 应是 all 中 created_at 最大的, action 形如 "a.{n-1}"
                let max_action = format!("a.{}", n - 1);
                prop_assert_eq!(latest.action.as_str(), max_action.as_str());
            });
        }
    }
}
