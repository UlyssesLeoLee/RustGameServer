//! match-service 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 MatchService（HealthCheck + GetMatch + 9 v2 RPC）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。
//! 55.21 wire-up：tonic server 强制 mTLS（per RGS-REV-007 CH4 / DEC-015 P1）。
//! 55.22 wire-up：实例化 PgOutboxRepository + OutboxRelay 后台轮询（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）。
//! 55.26 fail-closed mTLS：默认强制 mTLS；RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//!                       (per RGS-REV-008 AC-1 / verify-A+C)。
//!
//! 桶 9 补完: 注入 MatchmakerServiceV2 (per RGS-DTL-038 §4.2 + §5)
//!
//! 2026-09-04 P0-1 重构：5 域 main.rs 抽公共骨架到 `shared_platform::service_bootstrap`。

use anyhow::Context;
use std::sync::Arc;

use match_service::db;
use match_service::matchmaker_v2::MatchmakerServiceV2;
use match_service::repository::{
    MatchParticipantRepository, MatchRepository, PgMatchParticipantRepository, PgMatchRepository,
};
use match_service::repository_v2::{
    PgGameSessionRepository, PgMatchmakingTicketRepository, PgMoveRepository,
};
use match_service::service::grpc_service::MatchGrpcService;
use match_service::service::MatchServiceImpl;
use shared_platform::service_bootstrap::{self, BootstrapConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 公共 bootstrap
    let cfg = BootstrapConfig::for_match(env!("CARGO_PKG_VERSION"))?;
    service_bootstrap::init_tracing(cfg.service_name);
    let _otel = service_bootstrap::init_otel_optional(&cfg);

    tracing::info!(target: "match-service", "starting service at {}, db={}", cfg.grpc_addr, cfg.database_url);

    // 2. DB pool + migrations（域特定：v1 + v2 repo 共 5 张表）
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

    // ===== v1 仓库 (5 域 matchmaker 旧业务) =====
    let matches: Arc<dyn MatchRepository> = Arc::new(PgMatchRepository::new(pool.clone()));
    let participants: Arc<dyn MatchParticipantRepository> =
        Arc::new(PgMatchParticipantRepository::new(pool.clone()));

    // ===== v2 仓库 (卡牌游戏 session/turn 业务) =====
    let v2_sessions: Arc<dyn match_service::repository_v2::GameSessionRepository> =
        Arc::new(PgGameSessionRepository::new(pool.clone()));
    let v2_moves: Arc<dyn match_service::repository_v2::MoveRepository> =
        Arc::new(PgMoveRepository::new(pool.clone()));
    let v2_tickets: Arc<dyn match_service::repository_v2::MatchmakingTicketRepository> =
        Arc::new(PgMatchmakingTicketRepository::new(pool.clone()));
    let mut matchmaker_v2 = Arc::new(MatchmakerServiceV2::new(
        v2_sessions,
        v2_moves,
        v2_tickets,
    ));

    // W36 (2026-08-30): 跨域 SaveReplay saga — 注入 replay-service gRPC 客户端
    match build_replay_client() {
        Ok(client) => {
            tracing::info!(
                target: "match-service",
                "replay-service gRPC client ready (endpoint={})",
                client_endpoint()
            );
            if let Some(mv2_mut) = Arc::get_mut(&mut matchmaker_v2) {
                mv2_mut.set_replay_client(client);
            } else {
                tracing::warn!(
                    target: "match-service",
                    "matchmaker_v2 already shared; replay client NOT injected (session 结束不触发 SaveReplay)"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                target: "match-service",
                "replay-service gRPC client init failed: {}; SaveReplay saga disabled (session 结束不触发)",
                e
            );
        }
    }

    tracing::info!(target: "match-service", "{}-service started, DB pool size: {}", cfg.service_name, pool.size());

    // 3. outbox relay 后台（公共）
    service_bootstrap::spawn_outbox_relay(pool.clone(), cfg.service_name, cfg.nats_uri.clone()).await;

    // 4. mTLS server builder（公共）
    let mut server = service_bootstrap::build_server_with_mtls(&cfg)?;

    // 5. 域 service wiring（域特定：MatchServiceImpl with v2 matchmaker）
    let service_impl = Arc::new(MatchServiceImpl::with_matchmaker_v2(
        matches,
        participants,
        matchmaker_v2,
    ));
    let grpc = MatchGrpcService::new(service_impl);
    let svc = match_service::proto::v1::match_service_server::MatchServiceServer::new(grpc);

    // 6. health + serve（公共）
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<match_service::proto::v1::match_service_server::MatchServiceServer<MatchGrpcService>>()
        .await;
    service_bootstrap::set_serving_defaults(&mut health_reporter).await;

    server
        .add_service(svc)
        .add_service(health_service)
        .serve(cfg.grpc_addr)
        .await?;
    Ok(())
}

// ============================================================================
// W36 (2026-08-30): replay-service gRPC 客户端构造辅助函数
// - mTLS fail-closed (per RGS-REV-007 CH4 / DEC-015 P1)
// ============================================================================

fn client_endpoint() -> String {
    std::env::var("REPLAY_GRPC_ENDPOINT")
        .unwrap_or_else(|_| "http://replay-service:50057".to_string())
}

fn build_replay_client() -> anyhow::Result<Arc<dyn match_service::ReplayClientTrait>> {
    use match_service::{ReplayClient, ReplayClientConfig};

    let endpoint = std::env::var("REPLAY_GRPC_ENDPOINT")
        .unwrap_or_else(|_| "http://replay-service:50057".to_string());

    let allow_insecure = std::env::var("RGS_ALLOW_INSECURE_GRPC")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let config = if allow_insecure {
        tracing::warn!(
            target: "match-service",
            "⚠ RGS_ALLOW_INSECURE_GRPC=1 — replay-service gRPC INSECURE (dev/test only)"
        );
        ReplayClientConfig::insecure(endpoint)
    } else {
        let tls_dir =
            std::env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let ca = format!("{}/ca.pem", tls_dir);
        let cert = format!("{}/replay-client.pem", tls_dir);
        let key = format!("{}/replay-client.key", tls_dir);
        if !std::path::Path::new(&ca).exists()
            || !std::path::Path::new(&cert).exists()
            || !std::path::Path::new(&key).exists()
        {
            anyhow::bail!(
                "mTLS cert missing at {tls_dir} (need ca.pem, replay-client.pem, replay-client.key). \
                 set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test"
            );
        }
        ReplayClientConfig::mtls(endpoint, "replay-service", ca, cert, key)
    };

    let client = ReplayClient::try_connect_lazy(config)
        .context("replay-service client init failed")?;
    Ok(Arc::new(client) as Arc<dyn match_service::ReplayClientTrait>)
}
