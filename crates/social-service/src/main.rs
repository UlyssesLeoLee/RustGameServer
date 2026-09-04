//! social-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 SocialService（HealthCheck + GetGuild）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。
//!
//! 2026-09-04 P0-1 重构：5 域 main.rs 抽公共骨架到 `shared_platform::service_bootstrap`。

use std::sync::Arc;

use shared_platform::service_bootstrap::{self, BootstrapConfig};
use social_service::db;
use social_service::repository::{
    GuildMemberRepository, GuildRepository, PgGuildMemberRepository, PgGuildRepository,
};
use social_service::service::grpc_service::SocialGrpcService;
use social_service::service::SocialServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 公共 bootstrap
    let cfg = BootstrapConfig::for_social(env!("CARGO_PKG_VERSION"))?;
    service_bootstrap::init_tracing(cfg.service_name);
    let _otel = service_bootstrap::init_otel_optional(&cfg);

    tracing::info!(target: "social-service", "starting service at {}, db={}", cfg.grpc_addr, cfg.database_url);

    // 2. DB pool + migrations（域特定：2 张表 guilds + guild_members）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "social-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "social-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }
    let guilds: Arc<dyn GuildRepository> = Arc::new(PgGuildRepository::new(pool.clone()));
    let members: Arc<dyn GuildMemberRepository> =
        Arc::new(PgGuildMemberRepository::new(pool.clone()));
    tracing::info!(target: "social-service", "{}-service started, DB pool size: {}", cfg.service_name, pool.size());

    // 3. outbox relay 后台（公共）
    service_bootstrap::spawn_outbox_relay(pool.clone(), cfg.service_name, cfg.nats_uri.clone()).await;

    // 4. mTLS server builder（公共）
    let mut server = service_bootstrap::build_server_with_mtls(&cfg)?;

    // 5. 域 service wiring（域特定：SocialServiceServer）
    let service_impl = Arc::new(SocialServiceImpl::new(guilds, members));
    let grpc = SocialGrpcService::new(service_impl);
    let svc = social_service::proto::v1::social_service_server::SocialServiceServer::new(grpc);

    // 6. health + serve（公共）
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<social_service::proto::v1::social_service_server::SocialServiceServer<SocialGrpcService>>()
        .await;
    service_bootstrap::set_serving_defaults(&mut health_reporter).await;

    server
        .add_service(svc)
        .add_service(health_service)
        .serve(cfg.grpc_addr)
        .await?;
    Ok(())
}
