//! match-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 MatchService（HealthCheck + GetMatch）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use match_service::db;
use match_service::repository::{
    MatchParticipantRepository, MatchRepository, PgMatchParticipantRepository, PgMatchRepository,
};
use match_service::service::grpc_service::MatchGrpcService;
use match_service::service::MatchServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,match-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50053".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "match-service", "starting service at {}, db={}", addr, database_url);

    // 55.15: 真实 DB pool + migrations（InMemory 留作测试用）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "match-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "match-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let matches: Arc<dyn MatchRepository> = Arc::new(PgMatchRepository::new(pool.clone()));
    let participants: Arc<dyn MatchParticipantRepository> =
        Arc::new(PgMatchParticipantRepository::new(pool.clone()));

    tracing::info!(target: "match-service", "match-service started, DB pool size: {}", pool.size());

    let service_impl = Arc::new(MatchServiceImpl::new(matches, participants));
    let grpc = MatchGrpcService::new(service_impl);

    tracing::info!(target: "match-service", "binding gRPC server at {}", addr);
    let svc = match_service::proto::v1::match_service_server::MatchServiceServer::new(grpc);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
