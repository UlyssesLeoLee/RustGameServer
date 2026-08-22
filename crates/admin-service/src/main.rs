//! admin-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 AdminService（HealthCheck + GetAdminOp）+ tracing 初始化。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use admin_service::repository::{
    AdminUserRepository, AuditLogRepository, InMemoryAdminUserRepository, InMemoryAuditLogRepository,
};
use admin_service::service::grpc_service::AdminGrpcService;
use admin_service::service::AdminServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,admin-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50055".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "admin-service", "starting service at {}, db={}", addr, database_url);

    let users: Arc<dyn AdminUserRepository> = Arc::new(InMemoryAdminUserRepository::new());
    let audit: Arc<dyn AuditLogRepository> = Arc::new(InMemoryAuditLogRepository::new());
    let service_impl = Arc::new(AdminServiceImpl::new(users, audit));
    let grpc = AdminGrpcService::new(service_impl);

    tracing::info!(target: "admin-service", "binding gRPC server at {}", addr);
    let svc = admin_service::proto::v1::admin_service_server::AdminServiceServer::new(grpc);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
