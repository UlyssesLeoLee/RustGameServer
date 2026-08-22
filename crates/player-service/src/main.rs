//! player-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 PlayerService（HealthCheck + GetPlayer）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use player_service::db;
use player_service::repository::{
    PgPlayerRepository, PgPlayerSessionRepository, PlayerRepository, PlayerSessionRepository,
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

    // 55.15: 真实 DB pool + migrations（InMemory 留作测试用）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "player-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "player-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let players: Arc<dyn PlayerRepository> = Arc::new(PgPlayerRepository::new(pool.clone()));
    let sessions: Arc<dyn PlayerSessionRepository> =
        Arc::new(PgPlayerSessionRepository::new(pool.clone()));

    tracing::info!(target: "player-service", "player-service started, DB pool size: {}", pool.size());

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
