//! replay-service 入口 (per RGS-DTL-038 §3 DEC-038-03 + 桶 13)
//!
//! ## 状态
//! 桶 13 实装 (per 2026-08-29 19:42 JST 父 session 任务):
//! - tonic gRPC server 接 ReplayService (4 RPC + HealthCheck)
//! - PgReplayRepository (per ARC-008 → replay_db)
//! - LocalFsBackend 对象存储 (mock cluster-ops)
//! - mTLS (per RGS-REV-007 CH4 / DEC-015 P1) — 默认强制, RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//! - 1 张表 migration (replays, per DTL-038 §7.1 #7)
//!
//! ## 集成
//! - match-service session 结束自动调 SaveReplay (TODO 推 W36+)
//! - 客户端: 回放拉 / 流
//!
//! ## 端口
//! GRPC_ADDR default: 0.0.0.0:50058 (per 5 域 + 卡牌游戏 6 域端口分配)

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

use shared_platform::tls::load_server_tls_config;
use shared_platform::tracing_init::init_otel_exporter_optional;

use replay_service::db;
use replay_service::proto::v1::replay_service_server::ReplayServiceServer;
use replay_service::repository::{PgReplayRepository, ReplayRepository};
use replay_service::service::grpc_service::ReplayGrpcService;
use replay_service::service::ReplayServiceImpl;
use replay_service::storage::{LocalFsBackend, StorageBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    // tracing 初始化 (per 5 域对齐)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,replay-service=debug")),
        )
        .init();

    // OTLP exporter 条件初始化 (per WBS WF-1-55.45 §3.3, 默认禁用)
    let _otel_guard = init_otel_exporter_optional(
        "replay-service",
        env!("CARGO_PKG_VERSION"),
        "dev",
    );

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50058".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL env required (per ARC-008 replay_db)")?;
    let storage_root = env::var("RGS_REPLAY_STORAGE_DIR")
        .unwrap_or_else(|_| "/var/lib/rgs/replays".to_string());

    tracing::info!(
        target: "replay-service",
        "starting service at {}, db={}, storage={}",
        addr,
        database_url,
        storage_root
    );

    // DB pool + migrations
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "replay-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "replay-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let repo: Arc<dyn ReplayRepository> = Arc::new(PgReplayRepository::new(pool.clone()));
    let storage: Arc<dyn StorageBackend> = Arc::new(LocalFsBackend::new(&storage_root));
    tracing::info!(
        target: "replay-service",
        "replay-service started, DB pool size: {}, storage root: {}",
        pool.size(),
        storage_root
    );

    let service_impl = Arc::new(ReplayServiceImpl::new(repo, storage));
    let grpc = ReplayGrpcService::new(service_impl);

    // grpc.health.v1.Health (k8s exec 探针)
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ReplayServiceServer<ReplayGrpcService>>()
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
            target: "replay-service",
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
        tracing::info!(target: "replay-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    // 启动 tonic server
    tracing::info!(target: "replay-service", "binding gRPC server at {}", addr);
    let svc = ReplayServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
