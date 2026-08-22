//! player-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 PlayerService（HealthCheck + GetPlayer）+ tracing 初始化。
//! 54.4 PgRepository wiring 留 55.x；当前用 InMemoryRepository 让 binary 可启动。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use player_service::db;
use player_service::repository::{
    InMemoryPlayerRepository, InMemoryPlayerSessionRepository, PlayerRepository,
    PlayerSessionRepository,
};
use player_service::service::grpc_service::PlayerGrpcService;
use player_service::service::PlayerServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing 初始化
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,player-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "player-service", "starting service at {}, db={}", addr, database_url);

    // 54.4 PgRepository wiring 留 55.x：先用 InMemory 让 binary 可启动
    let players: Arc<dyn PlayerRepository> = Arc::new(InMemoryPlayerRepository::new());
    let sessions: Arc<dyn PlayerSessionRepository> = Arc::new(InMemoryPlayerSessionRepository::new());

    // 健康检查：尝试连 DB 失败不影响 binary 启动
    if let Err(e) = db::pool_from_env().await {
        tracing::warn!(target: "player-service", "DB pool init failed (using in-memory fallback): {}", e);
    }

    let service_impl = Arc::new(PlayerServiceImpl::new(players, sessions));
    let grpc = PlayerGrpcService::new(service_impl);

    // 启动 tonic server
    tracing::info!(target: "player-service", "binding gRPC server at {}", addr);
    let svc = player_service::proto::v1::player_service_server::PlayerServiceServer::new(grpc);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
