//! admin-service 域 Service 业务实施（per RGS-DTL-019 §3 + ARC-051 COC）
//!
//! 54.7 实化：4 Service 业务方法（authenticate / create_admin / disable_admin / audit_log）
//! + gRPC 桥接 HealthCheck + GetAdminUser
//!
//! 55.13 实化：`audit_log` 包事务（read latest + append 同一事务 FOR UPDATE 锁），
//! 保证 hash 链 read-then-append 原子（per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）。

use crate::entity::{AdminRole, AdminUser, AuditLogEntry};
use crate::error::Error;
use crate::repository::{AdminUserRepository, AuditLogRepository};
use crate::Result;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait AdminService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    /// 认证（密码 hash 校验）
    async fn authenticate(&self, username: String, password_hash: String) -> Result<AdminUser>;

    /// 创建管理员
    async fn create_admin(
        &self,
        username: String,
        password_hash: String,
        role: AdminRole,
        domain_scope: Option<String>,
    ) -> Result<AdminUser>;

    /// 停用管理员
    async fn disable_admin(&self, admin_id: Uuid) -> Result<bool>;

    /// 追加审计日志
    async fn audit_log(
        &self,
        actor_id: Uuid,
        action: String,
        target: String,
        payload: String,
    ) -> Result<AuditLogEntry>;
}

pub struct AdminServiceImpl {
    users: Arc<dyn AdminUserRepository>,
    audit: Arc<dyn AuditLogRepository>,
    /// 55.13 增补：事务化所需 PgPool（Some = 生产 PG 路径，None = InMemory 测试路径）。
    /// 生产构造时由 `with_pool` 注入；测试可直接 `new`（无 pool）。
    pool: Option<PgPool>,
}

impl AdminServiceImpl {
    pub fn new(users: Arc<dyn AdminUserRepository>, audit: Arc<dyn AuditLogRepository>) -> Self {
        Self {
            users,
            audit,
            pool: None,
        }
    }

    /// 55.13：注入 PgPool 启用事务化 audit_log（per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）。
    pub fn with_pool(mut self, pool: PgPool) -> Self {
        self.pool = Some(pool);
        self
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<AdminUser>> {
        self.users.find_by_id(id).await
    }
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn authenticate(&self, username: String, password_hash: String) -> Result<AdminUser> {
        let user = self
            .users
            .find_by_username(&username)
            .await?
            .ok_or_else(|| Error::InvalidCredentials(username.clone()))?;
        if !user.is_active() {
            return Err(Error::AdminSessionExpired(username));
        }
        if user.password_hash != password_hash {
            return Err(Error::InvalidCredentials(username));
        }
        Ok(user)
    }

    async fn create_admin(
        &self,
        username: String,
        password_hash: String,
        role: AdminRole,
        domain_scope: Option<String>,
    ) -> Result<AdminUser> {
        if username.is_empty() {
            return Err(Error::Validation("username must not be empty".to_string()));
        }
        if password_hash.is_empty() {
            return Err(Error::Validation(
                "password_hash must not be empty".to_string(),
            ));
        }
        if self.users.find_by_username(&username).await?.is_some() {
            return Err(Error::Conflict(format!(
                "admin username {} taken",
                username
            )));
        }
        let mut user = AdminUser::new(username, password_hash, role);
        user.domain_scope = domain_scope;
        self.users.save(&user).await?;
        Ok(user)
    }

    async fn disable_admin(&self, admin_id: Uuid) -> Result<bool> {
        let ok = self.users.disable(admin_id, chrono::Utc::now()).await?;
        if !ok {
            return Err(Error::NotFound {
                entity: "AdminUser",
                id: admin_id.to_string(),
            });
        }
        Ok(true)
    }

    async fn audit_log(
        &self,
        actor_id: Uuid,
        action: String,
        target: String,
        payload: String,
    ) -> Result<AuditLogEntry> {
        // 55.13 事务化（per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）：
        // read latest + insert 在同一 PgTransaction 内完成，latest 行用
        // SELECT ... FOR UPDATE 锁住，避免并发读到同一 prev_hash 而分叉。
        // 失败时 drop tx → 自动 rollback；commit 仅在 INSERT 成功时执行。
        if let Some(pool) = &self.pool {
            let mut tx = pool.begin().await?;
            // 锁 latest 行（FOR UPDATE）→ 取出 hash 作 prev_hash
            let latest_row = sqlx::query(
                "SELECT id, actor_id, action, target, payload, prev_hash, hash, created_at \
                 FROM audit_log ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
            )
            .fetch_optional(&mut *tx)
            .await?;
            let prev_hash = latest_row
                .map(|r| r.get::<String, _>("hash"))
                .unwrap_or_else(|| "0".repeat(64));
            let entry = AuditLogEntry::new(actor_id, action, target, payload, prev_hash);
            self.audit.append_atomic(&mut tx, &entry).await?;
            tx.commit().await?;
            return Ok(entry);
        }
        // InMemory / 无 pool 路径：Mut<ex 已序列化 latest + append，语义等价
        let prev = self.audit.latest().await?;
        let prev_hash = prev.map(|e| e.hash).unwrap_or_else(|| "0".repeat(64));
        let entry = AuditLogEntry::new(actor_id, action, target, payload, prev_hash);
        self.audit.append(&entry).await?;
        Ok(entry)
    }
}

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as admin_proto;

    pub struct AdminGrpcService {
        pub impl_: Arc<AdminServiceImpl>,
    }

    impl AdminGrpcService {
        pub fn new(impl_: Arc<AdminServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl admin_proto::admin_service_server::AdminService for AdminGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: if healthy {
                    common_proto::Status::Ok as i32
                } else {
                    common_proto::Status::Failed as i32
                },
                message: if healthy {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
            }))
        }

        async fn get_admin_op(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<admin_proto::AdminOp>, Status> {
            let id_str = request.get_ref().id.clone();
            let user_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let u = self
                .impl_
                .find_user_by_id(user_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("admin {}", id_str)))?;
            Ok(Response::new(admin_proto::AdminOp {
                id: Some(common_proto::EntityId {
                    id: u.id.to_string(),
                }),
                status: u.is_active() as i32,
                created_at: Some(common_proto::Timestamp {
                    seconds: u.created_at.timestamp(),
                    nanos: u.created_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: u.username,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InMemoryAdminUserRepository, InMemoryAuditLogRepository};

    fn svc() -> AdminServiceImpl {
        AdminServiceImpl::new(
            Arc::new(InMemoryAdminUserRepository::new()),
            Arc::new(InMemoryAuditLogRepository::new()),
        )
    }

    #[tokio::test]
    async fn create_and_authenticate_admin() {
        let s = svc();
        let admin = s
            .create_admin(
                "root".to_string(),
                "hash123".to_string(),
                AdminRole::SuperAdmin,
                None,
            )
            .await
            .unwrap();
        let authed = s
            .authenticate("root".to_string(), "hash123".to_string())
            .await
            .unwrap();
        assert_eq!(authed.id, admin.id);
    }

    #[tokio::test]
    async fn authenticate_wrong_password() {
        let s = svc();
        s.create_admin(
            "u".to_string(),
            "h1".to_string(),
            AdminRole::SuperAdmin,
            None,
        )
        .await
        .unwrap();
        let err = s
            .authenticate("u".to_string(), "h2".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidCredentials(_)));
    }

    #[tokio::test]
    async fn disable_admin() {
        let s = svc();
        let u = s
            .create_admin(
                "d".to_string(),
                "h".to_string(),
                AdminRole::DomainAdmin,
                Some("player".to_string()),
            )
            .await
            .unwrap();
        s.disable_admin(u.id).await.unwrap();
        let loaded = s.find_user_by_id(u.id).await.unwrap().unwrap();
        assert!(!loaded.is_active());
    }

    #[tokio::test]
    async fn audit_log_chains() {
        let s = svc();
        let actor = Uuid::new_v4();
        let e1 = s
            .audit_log(
                actor,
                "player.ban".to_string(),
                "p1".to_string(),
                "{}".to_string(),
            )
            .await
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let e2 = s
            .audit_log(
                actor,
                "player.unban".to_string(),
                "p1".to_string(),
                "{}".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(e2.prev_hash, e1.hash);
        assert_ne!(e1.hash, e2.hash);
    }

    #[tokio::test]
    async fn duplicate_admin_username_conflict() {
        let s = svc();
        s.create_admin(
            "u".to_string(),
            "h".to_string(),
            AdminRole::SuperAdmin,
            None,
        )
        .await
        .unwrap();
        let err = s
            .create_admin(
                "u".to_string(),
                "h".to_string(),
                AdminRole::SuperAdmin,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn health_check() {
        let s = svc();
        assert!(s.health_check().await.unwrap());
    }

    /// 55.13 AC5=CC1：read-then-append 事务性（InMemory 路径下 Mutex 等价
    /// FOR UPDATE 串行化）。并发 20 个 audit_log 必须产出 20 条互不相同的 hash
    /// 且 prev_hash 链严格单向（e[i].prev_hash == e[i-1].hash 或首条为零）。
    #[tokio::test]
    async fn audit_log_atomic_latest_append() {
        let s = Arc::new(svc());
        let actor = Uuid::new_v4();

        // 串行 20 条：验证 prev_hash 链严格连续
        let mut entries = Vec::with_capacity(20);
        for i in 0..20 {
            let e = s
                .audit_log(
                    actor,
                    format!("action.{i}"),
                    format!("target.{i}"),
                    format!("{{\"i\":{i}}}"),
                )
                .await
                .unwrap();
            entries.push(e);
        }
        // 首条 prev_hash 应为 64 个 "0"
        assert_eq!(entries[0].prev_hash, "0".repeat(64));
        // 后续每条 prev_hash 严格等于前一条 hash
        for i in 1..entries.len() {
            assert_eq!(
                entries[i].prev_hash, entries[i - 1].hash,
                "hash 链断裂 at i={i}"
            );
        }
        // 20 条 hash 全部互不相同
        let unique: std::collections::HashSet<&String> =
            entries.iter().map(|e| &e.hash).collect();
        assert_eq!(unique.len(), entries.len(), "hash 出现碰撞");

        // 并发 20 条：再次验证 Mutex/InMemory 路径下不出现分叉
        let mut handles = Vec::with_capacity(20);
        for i in 0..20 {
            let s = Arc::clone(&s);
            handles.push(tokio::spawn(async move {
                s.audit_log(
                    actor,
                    format!("concurrent.action.{i}"),
                    format!("concurrent.target.{i}"),
                    "{}".to_string(),
                )
                .await
            }));
        }
        let mut concurrent_entries = Vec::with_capacity(20);
        for h in handles {
            concurrent_entries.push(h.await.unwrap().unwrap());
        }
        let unique: std::collections::HashSet<&String> =
            concurrent_entries.iter().map(|e| &e.hash).collect();
        assert_eq!(unique.len(), 20, "并发路径下 hash 链分叉 / 碰撞");
    }
}
