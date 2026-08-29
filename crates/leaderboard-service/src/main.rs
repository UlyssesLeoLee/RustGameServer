//! leaderboard-service 入口 (per RGS-REQ-038 §FR-007 + RGS-DTL-038 §3)
//!
//! 启动 tonic gRPC server 接 LeaderboardService (4 RPC + 内部 AddEntry) + tracing 初始化。
//! 桶 12 实施：DB pool init + migrations + 4 RPC + mTLS (per RGS-REV-007 CH4 / DEC-015 P1)。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use shared_platform::tls::load_server_tls_config;
use shared_platform::tracing_init::init_otel_exporter_optional;

use leaderboard_service::db;
use leaderboard_service::proto::v1::leaderboard_service_server::LeaderboardServiceServer;
use leaderboard_service::repository::{LeaderboardRepository, PgLeaderboardRepository};
use leaderboard_service::service::grpc_service::LeaderboardGrpcService;
use leaderboard_service::service::LeaderboardServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,leaderboard-service=debug")),
        )
        .init();

    // OTLP exporter 条件初始化 (per WBS WF-1-55.45 §3.3, 默认禁用)
    let _otel_guard = init_otel_exporter_optional(
        "leaderboard-service",
        env!("CARGO_PKG_VERSION"),
        "dev",
    );

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50057".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "leaderboard-service", "starting service at {}, db={}", addr, database_url);

    // DB pool + migrations
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "leaderboard-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "leaderboard-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let repo: Arc<dyn LeaderboardRepository> = Arc::new(PgLeaderboardRepository::new(pool.clone()));
    tracing::info!(target: "leaderboard-service", "leaderboard-service started, DB pool size: {}", pool.size());

    let service_impl = Arc::new(LeaderboardServiceImpl::new(repo));
    let grpc = LeaderboardGrpcService::new(service_impl);

    // grpc.health.v1.Health 服务 (k8s exec 探针)
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<LeaderboardServiceServer<LeaderboardGrpcService>>()
        .await;
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    // mTLS fail-closed (per RGS-REV-008 AC-1)
    let allow_insecure = env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let mut server_builder = tonic::transport::Server::builder();
    if allow_insecure {
        tracing::warn!(
            target: "leaderboard-service",
            "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"
        );
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
        tracing::info!(target: "leaderboard-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "leaderboard-service", "binding gRPC server at {}", addr);
    let svc = LeaderboardServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
