//! card-service 入口 (桶 10 card catalog 阶段)
//!
//! 桶 10 阶段 (per RGS-DTL-038 §4.4 + §3 DEC-038-01~09):
//! - tonic gRPC server 接 CardService (HealthCheck + GetCard + ... + OpenPack)
//! - PgCardRepository / PgCardSeriesRepository / PgCardInstanceRepository (per ARC-008 → card_db)
//! - mTLS (per RGS-REV-007 CH4 / DEC-015 P1) — 桶 10 占位: 强制 mTLS, 不做完整 wire-up
//! - 3 张表 migration (per DTL-038 §7.1 #1-3): cards / card_series / card_instances

use anyhow::Context;
use std::env;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

use shared_platform::tls::load_server_tls_config;

use card_service::db;
use card_service::repository::{
    CardInstanceRepository, CardRepository, CardSeriesRepository, PgCardInstanceRepository,
    PgCardRepository, PgCardSeriesRepository,
};
use card_service::service::grpc_service::CardGrpcService;
use card_service::service::CardServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    // tracing 初始化 (与 5 域模板对齐)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,card-service=debug")),
        )
        .init();

    // 55.45 OTLP exporter 条件初始化 (per RGS-OPEN-QA-001 Q-M-03 + WBS WF-1-55.45 §3.3)
    // 默认 OTEL_SDK_DISABLED=true (53.12 任务未完成), 即不真正启用
    let _otel_guard = shared_platform::tracing_init::init_otel_exporter_optional(
        "card-service",
        env!("CARGO_PKG_VERSION"),
        "dev",
    );

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50061".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required (per ARC-008 card_db)")?;

    tracing::info!(target: "card-service", "starting service at {}, db={}", addr, database_url);

    // DB pool + migrations
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "card-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "card-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let cards: Arc<dyn CardRepository> = Arc::new(PgCardRepository::new(pool.clone()));
    let series: Arc<dyn CardSeriesRepository> = Arc::new(PgCardSeriesRepository::new(pool.clone()));
    let instances: Arc<dyn CardInstanceRepository> =
        Arc::new(PgCardInstanceRepository::new(pool.clone()));

    let service_impl = Arc::new(CardServiceImpl::new(cards, series, instances));
    let grpc = CardGrpcService::new(service_impl);

    // grpc.health.v1.Health (k8s exec 探针)
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<card_service::proto::v1::card_service_server::CardServiceServer<CardGrpcService>>()
        .await;
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    // mTLS (per RGS-REV-007 CH4 / DEC-015 P1)
    // 桶 10 模板: 默认强制 mTLS, RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out (dev/test only)
    let allow_insecure = env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let mut server_builder = tonic::transport::Server::builder();
    if allow_insecure {
        tracing::warn!(
            target: "card-service",
            "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"
        );
        shared_platform::channel::SERVER_MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);
    } else {
        let tls_dir = env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let tls_config = load_server_tls_config(
            &std::path::PathBuf::from(format!("{}/server.pem", tls_dir)),
            &std::path::PathBuf::from(format!("{}/server.key", tls_dir)),
            &std::path::PathBuf::from(format!("{}/ca.pem", tls_dir)),
        )
        .context(
            "mTLS config load failed (set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test)",
        )?;
        server_builder = server_builder
            .tls_config(tls_config)
            .context("tls_config")?;
        tracing::info!(target: "card-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    // 启动 tonic server
    tracing::info!(target: "card-service", "binding gRPC server at {}", addr);
    let svc = card_service::proto::v1::card_service_server::CardServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
