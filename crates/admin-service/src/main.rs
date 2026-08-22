//! admin-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 AdminService（HealthCheck + GetAdminOp）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。

use anyhow::Context;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use shared_platform::messaging::{build_messaging_client, MessagingConfig};
use shared_platform::outbox::PgOutboxRepository;
use shared_platform::outbox_relay::{OutboxRelay, RelayConfig};
use shared_platform::producer::{Producer, ProducerConfig};
use shared_platform::tls::load_server_tls_config;

use admin_service::db;
use admin_service::repository::{
    AdminUserRepository, AuditLogRepository, PgAdminUserRepository, PgAuditLogRepository,
};
use admin_service::service::grpc_service::AdminGrpcService;
use admin_service::service::AdminServiceImpl;

/// mTLS bypass 计数（55.26 fail-closed 防线，per RGS-REV-008 AC-1 / verify-A+C）
///
/// 进程内 counter：每次 `RGS_ALLOW_INSECURE_GRPC=1` 启动导致 gRPC 走明文时 +1。
/// 监控集成（Prometheus exporter / scrape handler → `mTLS_bypassed_total`）
/// 由后续任务处理；本 PR 仅做 fail-closed 防线本身。
///
/// 注：shared-platform 已有同名 private static（`MTLS_BYPASSED_TOTAL` for client side）；
/// 因任务约束禁止改 shared-platform，本地定义与现有 client 端语义一致（per-process counter）。
static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,admin-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50055".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "admin-service", "starting service at {}, db={}", addr, database_url);

    // 55.15: 真实 DB pool + migrations（InMemory 留作测试用）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "admin-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "admin-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let users: Arc<dyn AdminUserRepository> = Arc::new(PgAdminUserRepository::new(pool.clone()));
    let audit: Arc<dyn AuditLogRepository> = Arc::new(PgAuditLogRepository::new(pool.clone()));

    tracing::info!(target: "admin-service", "admin-service started, DB pool size: {}", pool.size());

    // 55.22: 实例化 OutboxRepository（per RGS-REV-007 CH1+CH2+AH1）

    // 55.22: 实例化 OutboxRepository（per RGS-REV-007 CH1+CH2+AH1）
    let outbox_repo: Arc<PgOutboxRepository> = Arc::new(PgOutboxRepository::new(pool.clone()));

    // 55.22: 连接 NATS 并启动 outbox relay 后台轮询
    // dev/test fallback: NATS 不可用时跳过 relay，gRPC server 继续运行
    let nats_uri = env::var("NATS_URI").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    match build_messaging_client(&MessagingConfig {
        uri: nats_uri.clone(),
        name: "admin-service".to_string(),
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
            tracing::info!(target: "admin-service", "outbox relay started (NATS={})", nats_uri);
        }
        Err(e) => {
            tracing::warn!(
                target: "admin-service",
                "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required",
                e
            );
        }
    }

    // 55.13 (per verify-C CC-1): 注入 PgPool 让 audit_log 走事务化路径
    // 不调 with_pool 时 admin-service.audit_log() 会走 InMemory fallback,
    // 导致 55.13 SHA-256 hash 链 + 事务化 + UNIQUE(prev_hash) 全部失效
    let service_impl = Arc::new(
        AdminServiceImpl::new(users, audit).with_pool(pool.clone())
    );
    let grpc = AdminGrpcService::new(service_impl);

    // 55.26 fail-closed mTLS（per RGS-REV-008 AC-1 / verify-A+C）
    // 默认强制 mTLS；仅 RGS_ALLOW_INSECURE_GRPC=1 / "true" 显式 opt-out 才允许 insecure gRPC（dev/test only）。
    // 不设置 / 0 / 任意其他值 → 任何 TLS 加载失败都通过 .context() 上抛 → main 返 Err → 进程退出 1。
    let allow_insecure = env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let mut server_builder = tonic::transport::Server::builder();
    if allow_insecure {
        tracing::warn!(
            target: "admin-service",
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
        tracing::info!(target: "admin-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "admin-service", "binding gRPC server at {}", addr);
    let svc = admin_service::proto::v1::admin_service_server::AdminServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
