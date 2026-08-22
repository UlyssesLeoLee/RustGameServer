//! economy-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 EconomyService（HealthCheck + GetAccount）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use economy_service::db;
use economy_service::repository::{
    AccountRepository, PgAccountRepository, PgTransactionLedgerRepository,
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

    // 55.15: 真实 DB pool + migrations（InMemory 留作测试用）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "economy-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "economy-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let accounts: Arc<dyn AccountRepository> = Arc::new(PgAccountRepository::new(pool.clone()));
    let ledger: Arc<dyn TransactionLedgerRepository> =
        Arc::new(PgTransactionLedgerRepository::new(pool.clone()));

    tracing::info!(target: "economy-service", "economy-service started, DB pool size: {}", pool.size());

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
