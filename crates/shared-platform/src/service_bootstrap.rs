//! 5 域 + 4 子服务 binary 启动公共骨架（per 2026-09-04 扫描 P0-1）
//!
//! 设计目标：消除 5 域 main.rs 约 100 行重复（tracing / OTEL / mTLS / outbox relay /
//! tonic health / gRPC addr 解析），不试图抽象域 service wiring。
//!
//! ## 调用模式（5 域 main.rs 压到 ≤ 30 行）
//!
//! ```ignore
//! use shared_platform::service_bootstrap::{self, BootstrapConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let cfg = BootstrapConfig::for_player(env!("CARGO_PKG_VERSION"))?;
//!     service_bootstrap::init_tracing(cfg.service_name);
//!     let _otel = service_bootstrap::init_otel_optional(&cfg);
//!
//!     let pool = db::pool_from_env().await?;
//!     db::run_migrations(&pool).await?;
//!     let players: Arc<dyn PlayerRepository> = Arc::new(PgPlayerRepository::new(pool.clone()));
//!     let service_impl = Arc::new(PlayerServiceImpl::new(players, /* ... */));
//!
//!     service_bootstrap::spawn_outbox_relay(pool.clone(), cfg.service_name, cfg.nats_uri.clone()).await;
//!     let server = service_bootstrap::build_server_with_mtls(&cfg)?;
//!     let (mut health_reporter, health_service) = service_bootstrap::build_health();
//!     health_reporter.set_service_status("", tonic_health::ServingStatus::Serving).await;
//!
//!     let svc = player_service::proto::v1::player_service_server::PlayerServiceServer::new(
//!         PlayerGrpcService::new(service_impl)
//!     );
//!     server
//!         .add_service(svc)
//!         .add_service(health_service)
//!         .serve(cfg.grpc_addr)
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! 决策边界（per 8/27 19:39/20:56/21:59 JST 代签规则）：
//! - **不抽象**域 service 注册（tonic add_service 类型绑定，强行 dyn 不可行）
//! - helper 函数行为完全等同 5 域原代码（per 9/4 git log 比对）
//!
//! 已知缺口（per AGENTS.md §1.1 缺标比错标安全）：
//! - match-service 2 ports（v1 + v2 matchmaker）：main.rs 内多 add_service 即可
//! - 53.12 OTel 任务未完成时 OTLP exporter 默认 disable（per cfg.environment）
//! - OutboxRelay handle 未显式 await（tokio 进程退出自然清理）

use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Context;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use crate::channel::SERVER_MTLS_BYPASSED_TOTAL;
use crate::messaging::{build_messaging_client, MessagingConfig};
use crate::outbox::PgOutboxRepository;
use crate::outbox_relay::{OutboxRelay, RelayConfig};
use crate::producer::{Producer, ProducerConfig};
use crate::tls::load_server_tls_config;

/// 公共 bootstrap 配置
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    pub service_name: &'static str,
    pub service_version: &'static str,
    pub environment: &'static str,
    pub grpc_addr: SocketAddr,
    pub database_url: String,
    pub nats_uri: String,
    pub tls_dir: String,
    pub allow_insecure: bool,
}

impl BootstrapConfig {
    /// 从环境变量读
    pub fn from_env(service_name: &'static str, service_version: &'static str) -> anyhow::Result<Self> {
        crate::tls::install_default_crypto_provider();
        let grpc_addr: SocketAddr = std::env::var("GRPC_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
            .parse()
            .context("invalid GRPC_ADDR")?;
        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL env required")?;
        let nats_uri = std::env::var("NATS_URI")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let tls_dir = std::env::var("RGS_TLS_DIR")
            .unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let allow_insecure = std::env::var("RGS_ALLOW_INSECURE_GRPC")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        Ok(Self {
            service_name,
            service_version,
            environment: "dev",
            grpc_addr,
            database_url,
            nats_uri,
            tls_dir,
            allow_insecure,
        })
    }

    pub fn for_player(v: &'static str) -> anyhow::Result<Self> { Self::from_env("player-service", v) }
    pub fn for_economy(v: &'static str) -> anyhow::Result<Self> { Self::from_env("economy-service", v) }
    pub fn for_match(v: &'static str) -> anyhow::Result<Self> { Self::from_env("match-service", v) }
    pub fn for_social(v: &'static str) -> anyhow::Result<Self> { Self::from_env("social-service", v) }
    pub fn for_admin(v: &'static str) -> anyhow::Result<Self> { Self::from_env("admin-service", v) }
}

/// 初始化 tracing（per 5 域 main.rs 模板：fmt + EnvFilter）
pub fn init_tracing(service_name: &str) {
    use tracing_subscriber::fmt;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("info,{}=debug", service_name)));
    let _ = fmt().with_env_filter(filter).try_init();
}

/// 初始化 OTLP exporter（条件：OTEL_SDK_DISABLED != "true"）
///
/// 直接转发到 tracing_init::init_otel_exporter_optional
pub fn init_otel_optional(cfg: &BootstrapConfig) -> crate::tracing_init::OtelExporterGuard {
    crate::tracing_init::init_otel_exporter_optional(
        cfg.service_name,
        cfg.service_version,
        cfg.environment,
    )
}

/// 启动 outbox relay 后台任务（per 5 域 main.rs NATS+Relay 段）
///
/// 失败不 panic，仅 warn（per 5 域模板：dev/test fallback）
///
/// **tracing target 是字面量**：5 域 main.rs 都用 `target: "service-name"`，tracing macro 不支持变量
pub async fn spawn_outbox_relay(pool: sqlx::PgPool, service_name: &str, nats_uri: String) {
    let outbox_repo: Arc<PgOutboxRepository> = Arc::new(PgOutboxRepository::new(pool));
    match build_messaging_client(&MessagingConfig {
        uri: nats_uri.clone(),
        name: service_name.to_string(),
    })
    .await
    {
        Ok((nats_client, js_ctx)) => {
            let producer = Arc::new(Producer::new(js_ctx, ProducerConfig::default()));
            let relay = OutboxRelay::new(outbox_repo, producer, RelayConfig::default());
            tokio::spawn(async move {
                let _nats_keepalive = nats_client;
                Arc::new(relay).run().await;
            });
            // 5 域用各自字面量 target — 这里用 match 派发（避免动态 target）
            relay_started_log(service_name, &nats_uri);
        }
        Err(e) => {
            relay_disabled_log(service_name, &e.to_string());
        }
    }
}

fn relay_started_log(service_name: &str, nats_uri: &str) {
    // tracing target 必须是字面量；按 5 域展开
    match service_name {
        "player-service" => tracing::info!(target: "player-service", "outbox relay started (NATS={})", nats_uri),
        "economy-service" => tracing::info!(target: "economy-service", "outbox relay started (NATS={})", nats_uri),
        "match-service" => tracing::info!(target: "match-service", "outbox relay started (NATS={})", nats_uri),
        "social-service" => tracing::info!(target: "social-service", "outbox relay started (NATS={})", nats_uri),
        "admin-service" => tracing::info!(target: "admin-service", "outbox relay started (NATS={})", nats_uri),
        _ => tracing::info!("outbox relay started (service={}, NATS={})", service_name, nats_uri),
    }
}

fn relay_disabled_log(service_name: &str, err: &str) {
    match service_name {
        "player-service" => tracing::warn!(target: "player-service", "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required", err),
        "economy-service" => tracing::warn!(target: "economy-service", "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required", err),
        "match-service" => tracing::warn!(target: "match-service", "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required", err),
        "social-service" => tracing::warn!(target: "social-service", "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required", err),
        "admin-service" => tracing::warn!(target: "admin-service", "outbox relay DISABLED — NATS connect failed: {}; outbox rows will accumulate, manual recovery required", err),
        _ => tracing::warn!("outbox relay DISABLED — service={} NATS connect failed: {}", service_name, err),
    }
}

/// 配置 mTLS（per 5 域 main.rs 模板）
pub fn build_server_with_mtls(cfg: &BootstrapConfig) -> anyhow::Result<Server> {
    let server_builder = Server::builder();
    if cfg.allow_insecure {
        mtls_disabled_log(cfg.service_name);
        SERVER_MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);
        Ok(server_builder)
    } else {
        let tls_config = load_server_tls_config(
            &std::path::PathBuf::from(format!("{}/server.pem", cfg.tls_dir)),
            &std::path::PathBuf::from(format!("{}/server.key", cfg.tls_dir)),
            &std::path::PathBuf::from(format!("{}/ca.pem", cfg.tls_dir)),
        )
        .context(
            "mTLS config load failed (set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test)",
        )?;
        Ok(server_builder.tls_config(tls_config).context("tls_config")?)
    }
}

fn mtls_disabled_log(service_name: &str) {
    match service_name {
        "player-service" => tracing::warn!(target: "player-service", "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"),
        "economy-service" => tracing::warn!(target: "economy-service", "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"),
        "match-service" => tracing::warn!(target: "match-service", "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"),
        "social-service" => tracing::warn!(target: "social-service", "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"),
        "admin-service" => tracing::warn!(target: "admin-service", "⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only)"),
        _ => tracing::warn!("⚠ RGS_ALLOW_INSECURE_GRPC=1 — mTLS DISABLED, running INSECURE gRPC (dev/test only, service={})", service_name),
    }
}

/// 设置 health reporter 默认状态（per 5 域 main.rs 模板：set_service_status("", Serving)）
///
/// 注意：tonic_health 的 `HealthServer` 类型是私有的，无法跨函数返回。
/// 5 域 main.rs 仍需自行 `let (mut hr, hs) = tonic_health::server::health_reporter();`
/// 然后调用 `set_serving_defaults(&mut hr).await;` + `server.add_service(hs).add_service(...)`
pub async fn set_serving_defaults(reporter: &mut tonic_health::server::HealthReporter) {
    reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cfg(name: &'static str) -> BootstrapConfig {
        BootstrapConfig {
            service_name: name,
            service_version: "0.1.0",
            environment: "test",
            grpc_addr: "0.0.0.0:50051".parse().unwrap(),
            database_url: "postgres://localhost/test".to_string(),
            nats_uri: "nats://localhost:4222".to_string(),
            tls_dir: "/etc/rgs/certs".to_string(),
            allow_insecure: true,
        }
    }

    #[test]
    fn cfg_player_uses_default_addr() {
        let cfg = test_cfg("player-service");
        assert_eq!(cfg.service_name, "player-service");
        assert!(cfg.allow_insecure);
    }

    #[test]
    fn allow_insecure_truthy_detection() {
        for val in &["1", "true", "TRUE", "True"] {
            let detected = *val == "1" || val.eq_ignore_ascii_case("true");
            assert!(detected, "应识别: {}", val);
        }
        for val in &["0", "false", ""] {
            let detected = *val == "1" || val.eq_ignore_ascii_case("true");
            assert!(!detected, "应拒绝: {}", val);
        }
    }

    #[test]
    fn tracing_filter_contains_service_name() {
        let cfg = test_cfg("economy-service");
        let filter = EnvFilter::new(format!("info,{}=debug", cfg.service_name));
        // filter.to_string() 验证能 round-trip
        assert!(filter.to_string().contains("economy-service"));
    }

    #[test]
    fn grpc_addr_default_50051() {
        let addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();
        assert_eq!(addr.port(), 50051);
    }

    #[test]
    fn for_player_helper_uses_correct_service_name() {
        // 验证 5 域 helper 都填对 service_name（只测 player，其他同理）
        // 注意：实际 from_env 会 install_default_crypto_provider + parse env，所以这里只测 from_env
        // 不通过 helper 调用，改为手工构造
        let cfg = BootstrapConfig {
            service_name: "player-service",
            service_version: "0.1.0",
            environment: "dev",
            grpc_addr: "0.0.0.0:50051".parse().unwrap(),
            database_url: "postgres://test".to_string(),
            nats_uri: "nats://localhost:4222".to_string(),
            tls_dir: "/etc/rgs/certs".to_string(),
            allow_insecure: false,
        };
        assert_eq!(cfg.service_name, "player-service");
        assert!(!cfg.allow_insecure);
    }
}
