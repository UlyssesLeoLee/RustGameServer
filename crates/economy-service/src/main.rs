//! economy-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 EconomyService（HealthCheck + GetAccount）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.23 wire-up：构造 SagaOrchestrator + ReserveHandler/ConfirmHandler，
//!                启动崩溃恢复后台任务（per RGS-REV-007 AC4 收尾 / DEC-015 P1）。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use economy_service::db;
use economy_service::entity::Currency;
use economy_service::repository::{
    AccountRepository, PgAccountRepository, PgTransactionLedgerRepository,
    TransactionLedgerRepository,
};
use economy_service::reservation::{PgReservationRepository, ReservationRepository};
use economy_service::saga::{PgSagaRepository, SagaRepository};
use economy_service::saga_orchestrator::{ConfirmHandler, ReserveHandler, SagaOrchestrator};
use economy_service::service::grpc_service::EconomyGrpcService;
use economy_service::service::EconomyServiceImpl;

/// 单次 Saga 预留金额（最小单位：分/钻/代币）
///
/// 与 55.12 saga_orchestrator.rs 测试默认值对齐（per TEST_AMOUNT）
const SAGA_RESERVE_AMOUNT: i64 = 100;

/// Saga 恢复轮询间隔（秒）
const SAGA_RECOVER_INTERVAL_SECS: u64 = 30;

/// 单次恢复扫描上限（防列表爆炸）
const SAGA_RECOVER_BATCH: i64 = 100;

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
    let reservations: Arc<dyn ReservationRepository> =
        Arc::new(PgReservationRepository::new(pool.clone()));
    let sagas: Arc<dyn SagaRepository> = Arc::new(PgSagaRepository::new(pool.clone()));

    tracing::info!(target: "economy-service", "economy-service started, DB pool size: {}", pool.size());

    // 55.23: 构造 SagaOrchestrator + ReserveHandler/ConfirmHandler（per RGS-REV-007 AC4 收尾）
    let reserve_handler: Arc<dyn economy_service::saga_orchestrator::SagaStepHandler> =
        Arc::new(ReserveHandler::new(
            reservations.clone(),
            accounts.clone(),
            SAGA_RESERVE_AMOUNT,
            Currency::Gold,
        ));
    let confirm_handler: Arc<dyn economy_service::saga_orchestrator::SagaStepHandler> =
        Arc::new(ConfirmHandler::new(
            reservations.clone(),
            accounts.clone(),
        ));
    let orchestrator = Arc::new(SagaOrchestrator::new(
        sagas.clone(),
        reservations.clone(),
        vec![reserve_handler, confirm_handler],
    ));

    // 55.23: 启动崩溃恢复后台任务（per RGS-DTL-100 §3 Saga 决策与执行）
    // 周期扫描 sagas.status IN ('running', 'compensating') 并 resume
    {
        let orch = orchestrator.clone();
        let sagas_for_recover = sagas.clone();
        tokio::spawn(async move {
            loop {
                match sagas_for_recover.list_running(SAGA_RECOVER_BATCH).await {
                    Ok(running) => {
                        for saga in running {
                            let id = saga.id;
                            if let Err(e) = orch.resume(id).await {
                                tracing::warn!(
                                    target: "saga",
                                    saga_id = %id,
                                    "saga resume failed: {}",
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            target: "saga",
                            "saga list_running failed: {}",
                            e
                        );
                    }
                }
                tokio::time::sleep(Duration::from_secs(SAGA_RECOVER_INTERVAL_SECS)).await;
            }
        });
    }

    tracing::info!(target: "saga", "saga orchestrator started");

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
