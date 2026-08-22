//! economy-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 EconomyService（HealthCheck + GetAccount）+ tracing 初始化。
//! 54.4 PgRepository wiring 留 55.x；当前用 InMemoryRepository 让 binary 可启动。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use economy_service::repository::{
    AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
    TransactionLedgerRepository,
};
use economy_service::service::grpc_service::EconomyGrpcService;
use economy_service::service::EconomyServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,economy-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50052".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "economy-service", "starting service at {}, db={}", addr, database_url);

    let accounts: Arc<dyn AccountRepository> = Arc::new(InMemoryAccountRepository::new());
    let ledger: Arc<dyn TransactionLedgerRepository> =
        Arc::new(InMemoryTransactionLedgerRepository::new());
    let service_impl = Arc::new(EconomyServiceImpl::new(accounts, ledger));
    let grpc = EconomyGrpcService::new(service_impl);

    tracing::info!(target: "economy-service", "binding gRPC server at {}", addr);
    let svc = economy_service::proto::v1::economy_service_server::EconomyServiceServer::new(grpc);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
