//! match-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 MatchService（HealthCheck + GetMatch）+ tracing 初始化。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use match_service::repository::{
    InMemoryMatchParticipantRepository, InMemoryMatchRepository, MatchParticipantRepository,
    MatchRepository,
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

    let matches: Arc<dyn MatchRepository> = Arc::new(InMemoryMatchRepository::new());
    let participants: Arc<dyn MatchParticipantRepository> = Arc::new(InMemoryMatchParticipantRepository::new());
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
