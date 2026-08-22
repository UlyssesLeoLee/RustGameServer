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

use anyhow::Context;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use shared_platform::messaging::{build_messaging_client, MessagingConfig};
use shared_platform::outbox::PgOutboxRepository;
use shared_platform::outbox_relay::{OutboxRelay, RelayConfig};
use shared_platform::producer::{Producer, ProducerConfig};
use shared_platform::tls::load_server_tls_config;

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

/// mTLS bypass 计数（55.26 fail-closed 防线，per RGS-REV-008 AC-1 / verify-A+C）
///
/// 进程内 counter：每次 `RGS_ALLOW_INSECURE_GRPC=1` 启动导致 gRPC 走明文时 +1。
/// 监控集成（Prometheus exporter / scrape handler → `mTLS_bypassed_total`）
/// 由后续任务处理；本 PR 仅做 fail-closed 防线本身。
///
/// 注：shared-platform 已有同名 private static（`MTLS_BYPASSED_TOTAL` for client side）；
/// 因任务约束禁止改 shared-platform，本地定义与现有 client 端语义一致（per-process counter）。
static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);

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

    // 55.22: 实例化 OutboxRepository（per RGS-REV-007 CH1+CH2+AH1）
    let outbox_repo: Arc<PgOutboxRepository> = Arc::new(PgOutboxRepository::new(pool.clone()));

    // 55.22: 连接 NATS 并启动 outbox relay 后台轮询
    // dev/test fallback: NATS 不可用时跳过 relay，gRPC server 继续运行
    let nats_uri = env::var("NATS_URI").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    match build_messaging_client(&MessagingConfig {
        uri: nats_uri.clone(),
        name: "economy-service".to_string(),
    })
    .await
    {
        Ok((nats_client, js_ctx)) => {
            let producer = Arc::new(Producer::new(js_ctx, ProducerConfig::default()));
            let relay = OutboxRelay::new(outbox_repo, producer, RelayConfig::default());
            tokio::spawn(async move {
                // 保持 NATS Client 存活（async_nats::Client 内部共享 Arc，但需 owner 存在以维持连接）
                let _nats_keepalive = nats_client;
                Arc::new(relay).run().await;
            });
            tracing::info!(target: "economy-service", "outbox relay started (NATS={})", nats_uri);
        }
        Err(e) => {
            tracing::warn!(
                target: "economy-service",
                "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required",
                e
            );
        }
    }

    let service_impl = Arc::new(EconomyServiceImpl::new(accounts, ledger));
    let grpc = EconomyGrpcService::new(service_impl);

    // 55.26 fail-closed mTLS（per RGS-REV-008 AC-1 / verify-A+C）
    // 默认强制 mTLS；仅 RGS_ALLOW_INSECURE_GRPC=1 / "true" 显式 opt-out 才允许 insecure gRPC（dev/test only）。
    // 不设置 / 0 / 任意其他值 → 任何 TLS 加载失败都通过 .context() 上抛 → main 返 Err → 进程退出 1。
    let allow_insecure = env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let mut server_builder = tonic::transport::Server::builder();
    if allow_insecure {
        tracing::warn!(
            target: "economy-service",
            "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"
        );
        MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);
    } else {
        let tls_dir = env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let tls_config = load_server_tls_config(
            &std::path::PathBuf::from(format!("{}/server.pem", tls_dir)),
            &std::path::PathBuf::from(format!("{}/server.key", tls_dir)),
            &std::path::PathBuf::from(format!("{}/ca.pem", tls_dir)),
        )
        .context("mTLS config load failed (set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test)")?;
        server_builder = server_builder
            .tls_config(tls_config)
            .context("tls_config")?;
        tracing::info!(target: "economy-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "economy-service", "binding gRPC server at {}", addr);
    let svc = economy_service::proto::v1::economy_service_server::EconomyServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
