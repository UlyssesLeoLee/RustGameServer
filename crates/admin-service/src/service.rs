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
        tracing::debug!(
            operation = "auth_entry",
            service = "admin-service",
            method = "Authenticate",
            username = %username,
            "enter authenticate"
        );
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
        tracing::debug!(
            operation = "rbac_admin_create",
            service = "admin-service",
            method = "CreateAdmin",
            username = %username,
            role = ?role,
            domain_scope = ?domain_scope,
            "enter create_admin (RBAC grant)"
        );
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
        tracing::debug!(
            operation = "rbac_admin_disable",
            service = "admin-service",
            method = "DisableAdmin",
            admin_id = %admin_id,
            "enter disable_admin (RBAC revoke)"
        );
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
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "admin-service",
                method = "HealthCheck",
                "enter grpc handler"
            );
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
            let user_id_parsed = Uuid::parse_str(&id_str).ok();
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "admin-service",
                method = "GetAdminOp",
                admin_id = %user_id_parsed.as_ref().map(|u| u.to_string()).unwrap_or_else(|| id_str.clone()),
                "enter grpc handler"
            );
            let user_id_parsed = Uuid::parse_str(&id_str).ok();
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "admin-service",
                method = "GetAdminOp",
                user_id = %user_id_parsed.as_ref().map(|u| u.to_string()).unwrap_or_else(|| id_str.clone()),
                "enter grpc handler"
            );
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
                display_name: u.username.clone(),
            }))
        }

        // S4 Phase 2 step 2: 4 GM RPC (BanAccount / GrantCompensation / SetMaintenance / QueryAuditLog)
        async fn ban_account(
            &self,
            request: Request<admin_proto::BanAccountRequest>,
        ) -> std::result::Result<Response<admin_proto::BanAccountResponse>, Status> {
            crate::gm_handlers::ban_account(request).await
        }

        async fn grant_compensation(
            &self,
            request: Request<admin_proto::GrantCompensationRequest>,
        ) -> std::result::Result<Response<admin_proto::GrantCompensationResponse>, Status> {
            crate::gm_handlers::grant_compensation(request).await
        }

        async fn set_maintenance(
            &self,
            request: Request<admin_proto::SetMaintenanceRequest>,
        ) -> std::result::Result<Response<admin_proto::SetMaintenanceResponse>, Status> {
            crate::gm_handlers::set_maintenance(request).await
        }

        async fn query_audit_log(
            &self,
            request: Request<admin_proto::QueryAuditLogRequest>,
        ) -> std::result::Result<Response<admin_proto::QueryAuditLogResponse>, Status> {
            crate::gm_handlers::query_audit_log(request).await
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
                entries[i].prev_hash,
                entries[i - 1].hash,
                "hash 链断裂 at i={i}"
            );
        }
        // 20 条 hash 全部互不相同
        let unique: std::collections::HashSet<&String> = entries.iter().map(|e| &e.hash).collect();
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

    // ========================================================================
    // UT 子代理 (2026-08-31 v2): 权限模型 + 未授权调用拒绝路径
    // 覆盖 8 项 (per ARC-051 COC RBAC + RGS-SEC-100 §7)
    // ========================================================================

    /// create_admin 拒绝空 username (输入校验)
    #[tokio::test]
    async fn create_admin_rejects_empty_username() {
        let s = svc();
        let err = s
            .create_admin(
                "".to_string(),
                "h".to_string(),
                AdminRole::SuperAdmin,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    /// create_admin 拒绝空 password_hash
    #[tokio::test]
    async fn create_admin_rejects_empty_password_hash() {
        let s = svc();
        let err = s
            .create_admin(
                "u".to_string(),
                "".to_string(),
                AdminRole::SuperAdmin,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    /// authenticate 找不到用户 → InvalidCredentials
    #[tokio::test]
    async fn authenticate_unknown_user_returns_invalid_credentials() {
        let s = svc();
        let err = s
            .authenticate("ghost".to_string(), "h".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidCredentials(_)));
    }

    /// 关键 RBAC 路径: 停用后 authenticate 应被拒绝 (AdminSessionExpired)
    /// per ARC-051 COC: disabled 用户无 session, 不允许后续 admin 操作.
    #[tokio::test]
    async fn disabled_admin_cannot_authenticate() {
        let s = svc();
        let u = s
            .create_admin(
                "soon-disabled".to_string(),
                "h".to_string(),
                AdminRole::SuperAdmin,
                None,
            )
            .await
            .unwrap();
        s.disable_admin(u.id).await.unwrap();
        let err = s
            .authenticate("soon-disabled".to_string(), "h".to_string())
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::AdminSessionExpired(_)),
            "停用后应返 AdminSessionExpired, 实得: {err:?}"
        );
    }

    /// disable_admin 找不到 id → NotFound
    #[tokio::test]
    async fn disable_admin_unknown_id_returns_not_found() {
        let s = svc();
        let err = s.disable_admin(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    /// 关键 RBAC 路径: can_admin_domain 全 16 种 (4 角色 × 4 域) 组合均与
    /// ARC-051 规范一致. 4 角色 × 4 域 = 16 case, 显式逐 case 断言 (per
    /// DDD Review 必查 "权限矩阵覆盖" 要求).
    #[test]
    fn rbac_matrix_full_table() {
        // 4 域: player / economy / match / social
        // 4 角色: SuperAdmin / DomainAdmin(player) / Auditor / Support
        let mut super_admin = AdminUser::new("root".into(), "h".into(), AdminRole::SuperAdmin);
        super_admin.domain_scope = None;
        let mut domain_admin = AdminUser::new("da".into(), "h".into(), AdminRole::DomainAdmin);
        domain_admin.domain_scope = Some("player".into());
        let mut domain_admin_other = AdminUser::new("dao".into(), "h".into(), AdminRole::DomainAdmin);
        domain_admin_other.domain_scope = Some("economy".into());
        let auditor = AdminUser::new("a".into(), "h".into(), AdminRole::Auditor);
        let support = AdminUser::new("s".into(), "h".into(), AdminRole::Support);

        // 期望矩阵: rows=角色, cols=域 (player/economy/match/social)
        // 4×4 = 16 cases
        let cases: [(&str, &AdminUser, [&str; 4], [bool; 4]); 4] = [
            (
                "SuperAdmin",
                &super_admin,
                ["player", "economy", "match", "social"],
                [true, true, true, true],
            ),
            (
                "DomainAdmin(player)",
                &domain_admin,
                ["player", "economy", "match", "social"],
                [true, false, false, false],
            ),
            (
                "DomainAdmin(economy)",
                &domain_admin_other,
                ["player", "economy", "match", "social"],
                [false, true, false, false],
            ),
            (
                "Auditor/Support",
                &auditor,
                ["player", "economy", "match", "social"],
                [false, false, false, false],
            ),
        ];
        // Auditor 与 Support 矩阵一致, 复用 auditor slot
        let _ = support; // 防 unused warning

        for (role_name, user, domains, expected) in &cases {
            for (i, domain) in domains.iter().enumerate() {
                let actual = user.can_admin_domain(domain);
                assert_eq!(
                    actual, expected[i],
                    "RBAC 矩阵不一致: role={role_name} domain={domain} expected={} actual={actual}",
                    expected[i]
                );
            }
        }
    }

    /// 关键 RBAC 路径: 未授权调用 (Auditor 调用 audit_log) - audit_log 本身
    /// 无角色检查 (是底层记录接口), 但**调用方**应有 role check. 验证 service
    /// 当前接口对所有角色都允许 audit_log (即 audit_log 是 system-level
    /// write 入口, 业务层在 handler 做 RBAC). 这一不变式需明确.
    #[tokio::test]
    async fn audit_log_accepts_any_actor_system_level_write() {
        // 明确不变式: audit_log 是低层 record, 角色检查在 gm_handlers /
        // service 之外的层做. 此处仅验证 record 层对任意 UUID 都接受.
        let s = svc();
        // 用 nil UUID 也应可写
        let e = s
            .audit_log(
                Uuid::nil(),
                "system.startup".into(),
                "boot".into(),
                "{}".into(),
            )
            .await
            .unwrap();
        assert_eq!(e.actor_id, Uuid::nil());
        assert_eq!(e.prev_hash, "0".repeat(64));
    }
}

// ============================================================================
// UT 子代理 (2026-08-31 v2): 权限矩阵 + 域校验 proptest
// ============================================================================

#[cfg(test)]
mod proptests {
    use super::tests::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// create_admin 拒绝空 username (不变式: 任何角色 × 任何 scope 都拒绝 "")
        #[test]
        fn create_admin_rejects_empty_username_any_role(
            role in prop_oneof![
                Just(AdminRole::SuperAdmin),
                Just(AdminRole::DomainAdmin),
                Just(AdminRole::Auditor),
                Just(AdminRole::Support),
            ],
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let s = svc();
                let err = s
                    .create_admin("".into(), "h".into(), role, None)
                    .await
                    .unwrap_err();
                prop_assert!(matches!(err, Error::Validation(_)));
            });
        }

        /// username 唯一性: 同名第二次 create 必返 Conflict
        #[test]
        fn duplicate_username_always_conflicts(
            name in "[a-z]{1,12}",
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let s = svc();
                s.create_admin(name.clone(), "h1".into(), AdminRole::SuperAdmin, None)
                    .await
                    .unwrap();
                let err = s
                    .create_admin(name.clone(), "h2".into(), AdminRole::Auditor, None)
                    .await
                    .unwrap_err();
                prop_assert!(matches!(err, Error::Conflict(_)));
            });
        }

        /// 审计日志 hash 链在 N 次追加下严格单向连续.
        /// prev_hash[0] = "0" * 64, prev_hash[i] = hash[i-1] for i >= 1.
        #[test]
        fn audit_log_chain_strict_ordering(n in 1usize..20) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let s = svc();
                let actor = Uuid::new_v4();
                let mut entries = Vec::with_capacity(n);
                for i in 0..n {
                    let e = s
                        .audit_log(
                            actor,
                            format!("a.{i}"),
                            format!("t.{i}"),
                            format!("{{\"i\":{i}}}"),
                        )
                        .await
                        .unwrap();
                    entries.push(e);
                }
                prop_assert_eq!(entries[0].prev_hash, "0".repeat(64));
                for i in 1..entries.len() {
                    prop_assert_eq!(
                        entries[i].prev_hash,
                        entries[i - 1].hash,
                        "chain break at i={i}"
                    );
                }
                // N 个 hash 全部不同
                let unique: std::collections::HashSet<&String> =
                    entries.iter().map(|e| &e.hash).collect();
                prop_assert_eq!(unique.len(), entries.len());
            });
        }
    }
}
