//! social-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 SocialService（HealthCheck + GetGuild）+ tracing 初始化。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use social_service::repository::{
    GuildMemberRepository, GuildRepository, InMemoryGuildMemberRepository, InMemoryGuildRepository,
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

    let guilds: Arc<dyn GuildRepository> = Arc::new(InMemoryGuildRepository::new());
    let members: Arc<dyn GuildMemberRepository> = Arc::new(InMemoryGuildMemberRepository::new());
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
