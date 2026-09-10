// rgs-flash-mock v0.3 — 闪烁之光 mock gateway / verification harness
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3
// per 9/10 14:35 JST Ulysses 拍板 (推荐) Mavis 起骨架 (1-2h PoC)
// v0.2 升级: 接 5 域 mTLS 业务级 gRPC client (per 9/11 派工)
// v0.3 升级: 7 域 mTLS 业务级 gRPC client + card + leaderboard 真实调用 (per 9/11 派工 v0.3)
//
// 端口: 0.0.0.0:8791 (RGS_FLASH_MOCK_PORT)
// 协议: HTTP/JSON (actix-web 4, per §2.1)
// back: tonic 0.12 gRPC client to RGS 7 域 (mTLS 业务级, per 8/27 11:06 JST 硬 ban 走 env var)
// 范围: 12 类别 22 RPC stub (per §3 表)

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use rgs_flash_mock::{config, AppState, GapMatrix, GrpcClients};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

const MOCK_VERSION: &str = env!("CARGO_PKG_VERSION");

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "rgs-flash-mock",
        "version": MOCK_VERSION,
    }))
}

#[get("/ready")]
async fn ready(data: web::Data<AppState>) -> impl Responder {
    let matrix = data.matrix.lock().await;
    let grpc_status = data.clients.status_report(&data.cfg);
    let grpc_connected = grpc_status.iter().filter(|s| s.connected).count();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "version": MOCK_VERSION,
        "matrix_total": matrix.total(),
        "matrix_passed": matrix.count_by_status(rgs_flash_mock::RpcStatus::Pass),
        "matrix_partial": matrix.count_by_status(rgs_flash_mock::RpcStatus::Partial),
        "matrix_na": matrix.count_by_status(rgs_flash_mock::RpcStatus::NotApplicable),
        "grpc_connected": grpc_connected,
        "grpc_total": grpc_status.len(),
        "grpc_status": grpc_status,
    }))
}

#[get("/coverage")]
async fn coverage(data: web::Data<AppState>) -> impl Responder {
    let matrix = data.matrix.lock().await;
    HttpResponse::Ok().json(matrix.report())
}

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "rgs-flash-mock",
        "version": MOCK_VERSION,
        "description": "RGS 闪烁之光 mock gateway / verification harness (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3)",
        "endpoints": {
            "GET /": "this help",
            "GET /health": "liveness probe",
            "GET /ready": "readiness probe",
            "GET /coverage": "gap matrix coverage report",
            "POST /scene/get": "GetScene (category 1, RPC 101)",
            "POST /scene/move": "MovePlayer (category 1, RPC 102)",
            "POST /role/profile": "GetPlayerProfile (category 2, RPC 201)",
            "POST /role/upgrade_skill": "UpgradeSkill (category 2, RPC 202)",
            "POST /combat/start": "StartCombat (category 3, RPC 301)",
            "POST /combat/action": "SubmitAction (category 3, RPC 302)",
            "POST /pvp/enqueue": "EnqueuePVP (category 4, RPC 401)",
            "POST /pvp/get": "GetPVPMatch (category 4, RPC 402)",
            "POST /guild/get": "GetGuild (category 5, RPC 501)",
            "POST /guild/join": "JoinGuild (category 5, RPC 502)",
            "POST /econ/account": "GetAccount (category 6, RPC 601)",
            "POST /econ/auction": "CreateAuction (category 6, RPC 602)",
            "POST /friend/list": "GetFriendList (category 7, RPC 701)",
            "POST /friend/send": "SendMessage (category 7, RPC 702)",
            "POST /event/active": "GetActiveEvent (category 8, RPC 801)",
            "POST /event/claim": "ClaimReward (category 8, RPC 802)",
            "POST /pay/recharge": "Recharge (category 9, RPC 901)",
            "POST /pay/history": "QueryRechargeHistory (category 9, RPC 902)",
            "POST /rank/leaderboard": "GetLeaderboard (category 10, RPC 1001)",
            "POST /gm/ban": "BanAccount (category 11, RPC 1101)",
            "POST /gm/grant": "GrantCompensation (category 11, RPC 1102)",
            "POST /card/collection": "GetPlayerCollection (category 12, RPC 1201, v0.3 NEW)",
        },
        "rgs_backend": {
            "player": std::env::var("GRPC_PLAYER_ENDPOINT").unwrap_or_else(|_| "https://player-service:50051".into()),
            "economy": std::env::var("GRPC_ECONOMY_ENDPOINT").unwrap_or_else(|_| "https://economy-service:50052".into()),
            "match": std::env::var("GRPC_MATCH_ENDPOINT").unwrap_or_else(|_| "https://match-service:50053".into()),
            "social": std::env::var("GRPC_SOCIAL_ENDPOINT").unwrap_or_else(|_| "https://social-service:50054".into()),
            "admin": std::env::var("GRPC_ADMIN_ENDPOINT").unwrap_or_else(|_| "https://admin-service:50055".into()),
            "card": std::env::var("GRPC_CARD_ENDPOINT").unwrap_or_else(|_| "https://card-service:50061".into()),
            "leaderboard": std::env::var("GRPC_LEADERBOARD_ENDPOINT").unwrap_or_else(|_| "https://leaderboard-service:50062".into()),
        },
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rgs_flash_mock=info".into()),
        )
        .init();

    // 安装 ring 作为 rustls 默认 crypto provider (per shared-platform::tls 同模式)
    rgs_flash_mock::install_default_crypto_provider();

    // 加载 config (per 8/27 11:06 JST 凭据走 env var, 永不打印)
    let cfg = match config::Config::from_env() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("config load failed: {}", e);
            std::process::exit(1);
        }
    };

    // 连接 7 域 gRPC client (mTLS 业务级, 任一域失败不阻塞启动)
    let clients = GrpcClients::from_config(&cfg).await;
    let connected_count = [
        clients.player.is_some(),
        clients.economy.is_some(),
        clients.r#match.is_some(),
        clients.social.is_some(),
        clients.admin.is_some(),
        clients.card.is_some(),
        clients.leaderboard.is_some(),
    ].iter().filter(|b| **b).count();
    info!(
        version = MOCK_VERSION,
        grpc_connected = connected_count,
        grpc_total = 7,
        "rgs-flash-mock v0.3 starting: 7 域 gRPC client pool initialized (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §2.1, 5 域 + card + leaderboard)"
    );

    let bind = std::env::var("RGS_GAP_MOCK_BIND").unwrap_or_else(|_| "0.0.0.0:8791".into());
    info!(
        version = MOCK_VERSION,
        %bind,
        "rgs-flash-mock starting (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3, 12 类别 22 RPC stub + 7 域 mTLS gRPC)"
    );

    let matrix = Arc::new(Mutex::new(GapMatrix::new()));
    let state = web::Data::new(AppState {
        matrix: matrix.clone(),
        clients,
        cfg: cfg.clone(),
        started_at: chrono::Utc::now(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(root)
            .service(health)
            .service(ready)
            .service(coverage)
            .service(rgs_flash_mock::handlers::scene::get_scene)
            .service(rgs_flash_mock::handlers::scene::move_player)
            .service(rgs_flash_mock::handlers::role::get_profile)
            .service(rgs_flash_mock::handlers::role::upgrade_skill)
            .service(rgs_flash_mock::handlers::combat::start_combat)
            .service(rgs_flash_mock::handlers::combat::submit_action)
            .service(rgs_flash_mock::handlers::pvp::enqueue_pvp)
            .service(rgs_flash_mock::handlers::pvp::get_pvp_match)
            .service(rgs_flash_mock::handlers::guild::get_guild)
            .service(rgs_flash_mock::handlers::guild::join_guild)
            .service(rgs_flash_mock::handlers::econ::get_account)
            .service(rgs_flash_mock::handlers::econ::create_auction)
            .service(rgs_flash_mock::handlers::friend::list)
            .service(rgs_flash_mock::handlers::friend::send)
            .service(rgs_flash_mock::handlers::event::active)
            .service(rgs_flash_mock::handlers::event::claim)
            .service(rgs_flash_mock::handlers::pay::recharge)
            .service(rgs_flash_mock::handlers::pay::history)
            .service(rgs_flash_mock::handlers::rank::leaderboard)
            .service(rgs_flash_mock::handlers::gm::ban)
            .service(rgs_flash_mock::handlers::gm::grant)
            .service(rgs_flash_mock::handlers::card::get_collection)
    })
    .bind(&bind)?
    .run()
    .await
}
