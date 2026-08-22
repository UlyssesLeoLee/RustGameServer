//! match-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 MatchService（HealthCheck + GetMatch）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use shared_platform::messaging::{build_messaging_client, MessagingConfig};
use shared_platform::outbox::PgOutboxRepository;
use shared_platform::outbox_relay::{OutboxRelay, RelayConfig};
use shared_platform::producer::{Producer, ProducerConfig};
use shared_platform::tls::load_server_tls_config;

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

    // 55.22: 实例化 OutboxRepository（per RGS-REV-007 CH1+CH2+AH1）
    let outbox_repo: Arc<PgOutboxRepository> = Arc::new(PgOutboxRepository::new(pool.clone()));

    // 55.22: 连接 NATS 并启动 outbox relay 后台轮询
    // dev/test fallback: NATS 不可用时跳过 relay，gRPC server 继续运行
    let nats_uri = env::var("NATS_URI").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    match build_messaging_client(&MessagingConfig {
        uri: nats_uri.clone(),
        name: "match-service".to_string(),
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
            tracing::info!(target: "match-service", "outbox relay started (NATS={})", nats_uri);
        }
        Err(e) => {
            tracing::warn!(
                target: "match-service",
                "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required",
                e
            );
        }
    }

    let service_impl = Arc::new(MatchServiceImpl::new(matches, participants));
    let grpc = MatchGrpcService::new(service_impl);

    // 55.21: 加载 mTLS 配置（per RGS-REV-007 CH4：强制 client 证书校验，防中间人）
    // dev/test fallback: PEM 缺失时降级为 insecure gRPC（warn 提示）
    let tls_dir = env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
    let tls_config = match load_server_tls_config(
        &std::path::PathBuf::from(format!("{}/server.pem", tls_dir)),
        &std::path::PathBuf::from(format!("{}/server.key", tls_dir)),
        &std::path::PathBuf::from(format!("{}/ca.pem", tls_dir)),
    ) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            tracing::warn!(
                target: "match-service",
                "mTLS config load failed ({}), falling back to insecure gRPC",
                e
            );
            None
        }
    };

    let mut server_builder = tonic::transport::Server::builder();
    if let Some(tls_cfg) = tls_config {
        server_builder = server_builder
            .tls_config(tls_cfg)
            .context("tls_config")?;
        tracing::info!(target: "match-service", "mTLS ENABLED — gRPC client cert verification required");
    } else {
        tracing::warn!(target: "match-service", "⚠ mTLS DISABLED — service running insecure gRPC");
    }

    tracing::info!(target: "match-service", "binding gRPC server at {}", addr);
    let svc = match_service::proto::v1::match_service_server::MatchServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
