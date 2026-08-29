//! match-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 MatchService（HealthCheck + GetMatch）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。

use anyhow::Context;
use std::env;
use std::sync::atomic::Ordering;
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

// mTLS bypass 计数（55.26 fail-closed 防线，per RGS-REV-008 AC-1 / verify-A+C / RGS-REV-009 HI-1）
//
// 进程内 counter：每次 `RGS_ALLOW_INSECURE_GRPC=1` 启动导致 gRPC 走明文时 +1。
// 监控集成（Prometheus exporter / scrape handler → `mTLS_bypassed_total`）
// 由后续任务处理；本 PR 仅做 fail-closed 防线本身。
//
// RGS-REV-009 HI-1：server 端 mTLS bypass 计数已迁移到 shared-platform
// `SERVER_MTLS_BYPASSED_TOTAL`（与 client 端 `MTLS_BYPASSED_TOTAL` 对称），
// 通过 `server_mtls_bypassed_total()` getter 读取。

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,match-service=debug")),
        )
        .init();

    // 55.45 OTLP exporter 条件初始化（per RGS-OPEN-QA-001 Q-M-03 + WBS WF-1-55.45 §3.3）
    // 默认 OTEL_SDK_DISABLED=true（53.12 任务未完成），即不真正启用
    // 53.12 完成后：去掉 OTEL_SDK_DISABLED env → 实际初始化
    let _otel_guard = shared_platform::tracing_init::init_otel_exporter_optional(
        "match-service",
        env!("CARGO_PKG_VERSION"),
        "dev",
    );

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

    // grpc.health.v1.Health 服务（k8s exec 探针 + mTLS，per RGS-OPS-101）
    // DB pool/migrations 已在此之前成功（失败已 exit(1)），此时注册即代表"可服务"。
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<match_service::proto::v1::match_service_server::MatchServiceServer<MatchGrpcService>>()
        .await;
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    // 55.26 fail-closed mTLS（per RGS-REV-008 AC-1 / verify-A+C）
    // 默认强制 mTLS；仅 RGS_ALLOW_INSECURE_GRPC=1 / "true" 显式 opt-out 才允许 insecure gRPC（dev/test only）。
    // 不设置 / 0 / 任意其他值 → 任何 TLS 加载失败都通过 .context() 上抛 → main 返 Err → 进程退出 1。
    let allow_insecure = env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let mut server_builder = tonic::transport::Server::builder();
    if allow_insecure {
        tracing::warn!(
            target: "match-service",
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
        tracing::info!(target: "match-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "match-service", "binding gRPC server at {}", addr);
    let svc = match_service::proto::v1::match_service_server::MatchServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
