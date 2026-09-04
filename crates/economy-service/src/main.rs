//! economy-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 EconomyService（HealthCheck + GetAccount）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.23 wire-up：构造 SagaOrchestrator + ReserveHandler/ConfirmHandler，
//!                启动崩溃恢复后台任务（per RGS-REV-007 AC4 收尾 / DEC-015 P1）。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。
//!
//! 2026-09-04 P0-1 重构：5 域 main.rs 抽公共骨架到 `shared_platform::service_bootstrap`。

use std::sync::Arc;
use std::time::Duration;

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
use economy_service::trade_repository::{PgTradeRepository, TradeRepository};
use economy_service::trade_service::TradeServiceImpl;
use shared_platform::service_bootstrap::{self, BootstrapConfig};

/// 单次 Saga 预留金额（最小单位：分/钻/代币）
const SAGA_RESERVE_AMOUNT: i64 = 100;
/// Saga 恢复轮询间隔（秒）
const SAGA_RECOVER_INTERVAL_SECS: u64 = 30;
/// 单次恢复扫描上限（防列表爆炸）
const SAGA_RECOVER_BATCH: i64 = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 公共 bootstrap
    let cfg = BootstrapConfig::for_economy(env!("CARGO_PKG_VERSION"))?;
    service_bootstrap::init_tracing(cfg.service_name);
    let _otel = service_bootstrap::init_otel_optional(&cfg);

    tracing::info!(target: "economy-service", "starting service at {}, db={}", cfg.grpc_addr, cfg.database_url);

    // 2. DB pool + migrations（域特定：5 张表，4 个 repo + 1 saga repo）
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
    tracing::info!(target: "economy-service", "{}-service started, DB pool size: {}", cfg.service_name, pool.size());

    // 3. Saga 编排器 + 崩溃恢复后台任务（域特定：仅 economy 有）
    let reserve_handler: Arc<dyn economy_service::saga_orchestrator::SagaStepHandler> =
        Arc::new(ReserveHandler::new(
            reservations.clone(),
            accounts.clone(),
            SAGA_RESERVE_AMOUNT,
            Currency::Gold,
        ));
    let confirm_handler: Arc<dyn economy_service::saga_orchestrator::SagaStepHandler> =
        Arc::new(ConfirmHandler::new(reservations.clone(), accounts.clone()));
    let orchestrator = Arc::new(SagaOrchestrator::new(
        sagas.clone(),
        reservations.clone(),
        vec![reserve_handler, confirm_handler],
    ));
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
                        tracing::warn!(target: "saga", "saga list_running failed: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(SAGA_RECOVER_INTERVAL_SECS)).await;
            }
        });
    }
    tracing::info!(target: "saga", "saga orchestrator started");

    // 4. outbox relay 后台（公共）
    service_bootstrap::spawn_outbox_relay(pool.clone(), cfg.service_name, cfg.nats_uri.clone()).await;

    // 5. mTLS server builder（公共）
    let mut server = service_bootstrap::build_server_with_mtls(&cfg)?;

    // 6. 域 service wiring（域特定：EconomyServiceServer + 双 service 实现）
    let service_impl = Arc::new(EconomyServiceImpl::new(accounts.clone(), ledger.clone()));
    let trade_repo: Arc<dyn TradeRepository> = Arc::new(PgTradeRepository::new(pool.clone()));
    let trade_impl = Arc::new(TradeServiceImpl::new(
        trade_repo,
        accounts.clone() as Arc<dyn AccountRepository>,
        ledger.clone() as Arc<dyn TransactionLedgerRepository>,
    ));
    let grpc = EconomyGrpcService::new(service_impl, trade_impl);
    let svc = economy_service::proto::v1::economy_service_server::EconomyServiceServer::new(grpc);

    // 7. health + serve（公共）
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<economy_service::proto::v1::economy_service_server::EconomyServiceServer<EconomyGrpcService>>()
        .await;
    service_bootstrap::set_serving_defaults(&mut health_reporter).await;

    server
        .add_service(svc)
        .add_service(health_service)
        .serve(cfg.grpc_addr)
        .await?;
    Ok(())
}
