//! admin-service 域 Service 业务实施（per RGS-DTL-019 §3 + ARC-051 COC）
//!
//! 54.7 实化：4 Service 业务方法（authenticate / create_admin / disable_admin / audit_log）
//! + gRPC 桥接 HealthCheck + GetAdminUser

use crate::entity::{AdminRole, AdminUser, AuditLogEntry};
use crate::error::Error;
use crate::repository::{AdminUserRepository, AuditLogRepository};
use crate::Result;

use async_trait::async_trait;
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
}

impl AdminServiceImpl {
    pub fn new(users: Arc<dyn AdminUserRepository>, audit: Arc<dyn AuditLogRepository>) -> Self {
        Self { users, audit }
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
        // 取最新 hash 作 prev_hash
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
}
