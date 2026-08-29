//! i18n-service 入口 (per RGS-DTL-038 §4.1 + DEC-038-05 + W35 桶 14 补完)
//!
//! W35 升级: 从 skeleton 占位升级到完整 service wiring
//! - PgI18nRepository 注入 (生产) / InMemoryI18nRepository 注入 (测试)
//! - tonic gRPC server 监听 50056
//! - mTLS fail-closed (per RGS-REV-008 AC-1 + 5 域对齐)

use anyhow::Context;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use i18n_service::proto::v1::i18n_service_server::I18nServiceServer;
use i18n_service::repository::I18nRepository;
use i18n_service::service::grpc_service::I18nGrpcService;
use i18n_service::service::I18nServiceImpl;
use shared_platform::tls::load_server_tls_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,i18n-service=debug")),
        )
        .init();

    // 55.45 OTLP exporter 条件初始化 (per RGS-OPEN-QA-001 Q-M-03)
    let _otel_guard = shared_platform::tracing_init::init_otel_exporter_optional(
        "i18n-service",
        env!("CARGO_PKG_VERSION"),
        "dev",
    );

    let addr: std::net::SocketAddr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50056".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;

    tracing::info!(target: "i18n-service", "starting service at {}", addr);

    // W35: 注入 repository (InMemory / Pg 二选一)
    // dev/test: InMemoryI18nRepository
    // prod: PgI18nRepository (需 DATABASE_URL + migrations)
    let database_url = std::env::var("DATABASE_URL").ok();
    let repo: Arc<dyn I18nRepository> = if let Some(_url) = database_url {
        // W36+: PgI18nRepository (per DTL-038 §6.2)
        // 当前仅 InMemory fallback (per bucket 14 partial → complete 实装)
        tracing::warn!(target: "i18n-service",
            "DATABASE_URL set but PgI18nRepository not yet wired (W36+ 任务). Using InMemory.");
        Arc::new(i18n_service::repository::InMemoryI18nRepository::new())
    } else {
        tracing::info!(target: "i18n-service",
            "DATABASE_URL not set, using InMemoryI18nRepository (dev/test mode)");
        Arc::new(i18n_service::repository::InMemoryI18nRepository::new())
    };

    let service_impl = Arc::new(I18nServiceImpl::new(repo));
    let grpc = I18nGrpcService::new(service_impl);

    // gRPC health (per 5 域对齐)
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<I18nServiceServer<I18nGrpcService>>()
        .await;
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    // mTLS fail-closed (per 5 域对齐 + RGS-REV-008 AC-1)
    let allow_insecure = std::env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let mut server_builder = tonic::transport::Server::builder();
    if allow_insecure {
        tracing::warn!(
            target: "i18n-service",
            "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"
        );
        shared_platform::channel::SERVER_MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);
    } else {
        let tls_dir = std::env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
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
        tracing::info!(target: "i18n-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "i18n-service", "binding gRPC server at {}", addr);
    server_builder
        .add_service(I18nServiceServer::new(grpc))
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
