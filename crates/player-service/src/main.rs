//! player-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 PlayerService（HealthCheck + GetPlayer）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。
//!
//! 2026-09-04 P0-1 重构：5 域 main.rs 抽公共骨架到 `shared_platform::service_bootstrap`，
//! 消除 ~100 行重复。main.rs 主体压到 ~30 行（仅保留域 service wiring + DB pool 实例化）。

use std::sync::Arc;

use player_service::db;
use player_service::repository::{
    DeckRepository, PgDeckRepository, PgPlayerRepository, PgPlayerSessionRepository,
    PlayerRepository, PlayerSessionRepository,
};
use player_service::service::grpc_service::PlayerGrpcService;
use player_service::service::PlayerServiceImpl;
use shared_platform::service_bootstrap::{self, BootstrapConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 公共 bootstrap（tracing / OTEL / DB / outbox / mTLS / health 默认值）
    let cfg = BootstrapConfig::for_player(env!("CARGO_PKG_VERSION"))?;
    service_bootstrap::init_tracing(cfg.service_name);
    let _otel = service_bootstrap::init_otel_optional(&cfg);

    tracing::info!(target: "player-service", "starting service at {}, db={}", cfg.grpc_addr, cfg.database_url);

    // 2. DB pool + migrations（域特定：player 有 3 张表，要每个 repo 实例化）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "player-service", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "player-service", "DB migrations failed: {}", e);
        std::process::exit(1);
    }
    let players: Arc<dyn PlayerRepository> = Arc::new(PgPlayerRepository::new(pool.clone()));
    let sessions: Arc<dyn PlayerSessionRepository> =
        Arc::new(PgPlayerSessionRepository::new(pool.clone()));
    let decks: Arc<dyn DeckRepository> = Arc::new(PgDeckRepository::new(pool.clone()));
    tracing::info!(target: "player-service", "{}-service started, DB pool size: {}", cfg.service_name, pool.size());

    // 3. outbox relay 后台（公共）
    service_bootstrap::spawn_outbox_relay(pool.clone(), cfg.service_name, cfg.nats_uri.clone()).await;

    // 4. mTLS server builder（公共）
    let mut server = service_bootstrap::build_server_with_mtls(&cfg)?;

    // 5. 域 service wiring（域特定：PlayerServiceServer）
    let service_impl = Arc::new(PlayerServiceImpl::new(players, sessions, decks));
    let grpc = PlayerGrpcService::new(service_impl);
    let svc = player_service::proto::v1::player_service_server::PlayerServiceServer::new(grpc);

    // 6. health + serve（公共）
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<player_service::proto::v1::player_service_server::PlayerServiceServer<PlayerGrpcService>>()
        .await;
    service_bootstrap::set_serving_defaults(&mut health_reporter).await;

    server
        .add_service(svc)
        .add_service(health_service)
        .serve(cfg.grpc_addr)
        .await?;
    Ok(())
}
