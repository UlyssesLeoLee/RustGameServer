//! social-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 SocialService（HealthCheck + GetGuild）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use social_service::db;
use social_service::repository::{
    GuildMemberRepository, GuildRepository, PgGuildMemberRepository, PgGuildRepository,
};
use social_service::service::grpc_service::SocialGrpcService;
use social_service::service::SocialServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,social-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50054".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "social-service", "starting service at {}, db={}", addr, database_url);

    // 55.15: 真实 DB pool + migrations（InMemory 留作测试用）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "social-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "social-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let guilds: Arc<dyn GuildRepository> = Arc::new(PgGuildRepository::new(pool.clone()));
    let members: Arc<dyn GuildMemberRepository> =
        Arc::new(PgGuildMemberRepository::new(pool.clone()));

    tracing::info!(target: "social-service", "social-service started, DB pool size: {}", pool.size());

    let service_impl = Arc::new(SocialServiceImpl::new(guilds, members));
    let grpc = SocialGrpcService::new(service_impl);

    tracing::info!(target: "social-service", "binding gRPC server at {}", addr);
    let svc = social_service::proto::v1::social_service_server::SocialServiceServer::new(grpc);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
