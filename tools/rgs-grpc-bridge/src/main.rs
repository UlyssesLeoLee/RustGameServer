//! rgs-grpc-bridge main.rs
//!
//! RGS gRPC-Web bridge: 浏览器 HTTP/1.1 + JSON (application/json) → RGS 5 域 gRPC server
//!
//! 设计 (per DTL-038 决策 + 9/10 JST Ulysses 拍板 opt2 独立 bridge binary):
//! - actix-web 4 暴露 HTTP/1.1 + JSON
//! - 内部用 tonic 0.12 client 调 backend gRPC server (mTLS or insecure)
//! - v0.1: 只 forward PlayerService (25 个 unary RPC)
//! - CORS: 允许浏览器 fetch 跨域 (H5 8788 → bridge 8080)
//! - 路径路由: /<package>.<service>/<method> 透传到对应 backend :50051
//! - JSON ↔ protobuf 转换: bridge 做透传, H5 发 JSON, bridge 编码到 protobuf 调 backend
//!
//! env 配置:
//! - RGS_BRIDGE_LISTEN: 监听地址 (default 0.0.0.0:8080)
//! - RGS_BRIDGE_PLAYER_BACKEND: player-service backend 地址 (default http://localhost:50051)
//! - RGS_BRIDGE_INSECURE: 1 = 跳过 mTLS (dev); 0 = mTLS (per RGS-REV-007 CH4 fail-closed)
//! - RGS_BRIDGE_CA_CERT / CLIENT_CERT / CLIENT_KEY: PEM 路径 (mTLS 模式)
//! - RUST_LOG: tracing filter (default info,rgs_grpc_bridge=debug)

use std::env;
use std::sync::Arc;

use actix_web::{
    http::header, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig, Identity};
use tracing_subscriber::EnvFilter;

// ============================================================================
// proto 模块: 复用 player-service 的 .proto (build.rs 编译到 OUT_DIR)
// ============================================================================

mod pb_player_proto {
    pub mod v1 {
        tonic::include_proto!("player.v1");
    }
}

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

use pb_player_proto::v1::{
    player_service_client::PlayerServiceClient, AntiAddictionCheckRequest,
    AntiAddictionCheckResponse, AvatarList, CharacterAssets, CharacterInfo, CharacterProfile,
    CreateCharacterRequest, CreateCharacterResponse, CreateDeckRequest, Deck,
    DeleteDeckRequest, DeleteDeckResponse, EnterBackgroundRequest, EnterBackgroundResponse,
    ForceDisconnectRequest, ForceDisconnectResponse, GetAvatarListRequest, GetCharacterAssetsRequest,
    GetCharacterInfoRequest, GetCharacterProfileRequest, GetPlayerProfileRequest, GetSharedDeckRequest,
    GuestModeTimeoutRequest, GuestModeTimeoutResponse, HeartbeatRequest, HeartbeatResponse,
    ListDecksRequest, ListDecksResponse, Player, RenameCharacterRequest, RenameCharacterResponse,
    ServerTimeInfo, SetAvatarRequest, SetAvatarResponse, ShareDeckRequest, ShareDeckResponse,
    UpdatePlayerProfileRequest, UpdatePlayerProfileResponse, GetServerTimeRequest,
    GetDeckRequest, UpdateDeckRequest, HeartbeatRequest as _, // avoid unused
};
use common::v1 as common_pb;
use pb_player_proto::v1::{
    AntiAddictionCheckRequest as _,
    AvatarList as _,
    CharacterAssets as _,
    CharacterInfo as _,
    CharacterProfile as _,
    CreateCharacterRequest as _,
    CreateCharacterResponse as _,
    CreateDeckRequest as _,
    Deck as _,
    DeleteDeckRequest as _,
    DeleteDeckResponse as _,
    EnterBackgroundRequest as _,
    EnterBackgroundResponse as _,
    ForceDisconnectRequest as _,
    ForceDisconnectResponse as _,
    GetAvatarListRequest as _,
    GetCharacterAssetsRequest as _,
    GetCharacterInfoRequest as _,
    GetCharacterProfileRequest as _,
    GetDeckRequest as _,
    GetPlayerProfileRequest as _,
    GetServerTimeRequest as _,
    GetSharedDeckRequest as _,
    GuestModeTimeoutRequest as _,
    GuestModeTimeoutResponse as _,
    HeartbeatRequest as _,
    HeartbeatResponse as _,
    ListDecksRequest as _,
    ListDecksResponse as _,
    LoginCharacterRequest as _,
    LoginCharacterResponse as _,
    Player as _,
    ReconnectCharacterRequest as _,
    ReconnectCharacterResponse as _,
    RenameCharacterRequest as _,
    RenameCharacterResponse as _,
    ServerTimeInfo as _,
    SetAvatarRequest as _,
    SetAvatarResponse as _,
    ShareDeckRequest as _,
    ShareDeckResponse as _,
    UpdateDeckRequest as _,
    UpdateDeckResponse as _,
    UpdatePlayerProfileRequest as _,
    UpdatePlayerProfileResponse as _,
};

// ============================================================================
// 配置
// ============================================================================

#[derive(Debug, Clone)]
struct BridgeConfig {
    listen_addr: String,
    player_backend: String,
    insecure: bool,
    ca_cert_path: Option<String>,
    client_cert_path: Option<String>,
    client_key_path: Option<String>,
}

impl BridgeConfig {
    fn from_env() -> Self {
        let insecure = env::var("RGS_BRIDGE_INSECURE").as_deref() == Ok("1");
        Self {
            listen_addr: env::var("RGS_BRIDGE_LISTEN")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            player_backend: env::var("RGS_BRIDGE_PLAYER_BACKEND")
                .unwrap_or_else(|_| "http://localhost:50051".to_string()),
            insecure,
            ca_cert_path: env::var("RGS_BRIDGE_CA_CERT").ok(),
            client_cert_path: env::var("RGS_BRIDGE_CLIENT_CERT").ok(),
            client_key_path: env::var("RGS_BRIDGE_CLIENT_KEY").ok(),
        }
    }
}

// ============================================================================
// Backend client 封装
// ============================================================================

#[derive(Clone)]
struct BackendClient {
    cfg: BridgeConfig,
    client: Arc<Mutex<Option<PlayerServiceClient<Channel>>>>,
}

impl BackendClient {
    fn new(cfg: BridgeConfig) -> Self {
        Self {
            cfg,
            client: Arc::new(Mutex::new(None)),
        }
    }

    async fn get(&self) -> Result<PlayerServiceClient<Channel>, tonic::Status> {
        let mut guard = self.client.lock().await;
        if let Some(c) = guard.as_ref() {
            return Ok(c.clone());
        }
        tracing::info!(target: "rgs_grpc_bridge", "connecting to backend {}", self.cfg.player_backend);
        let c = connect(&self.cfg)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("backend connect: {e}")))?;
        *guard = Some(c.clone());
        Ok(c)
    }
}

async fn connect(cfg: &BridgeConfig) -> anyhow::Result<PlayerServiceClient<Channel>> {
    let mut endpoint = Channel::from_shared(cfg.player_backend.clone())?
        .http2_keep_alive_interval(std::time::Duration::from_secs(30))
        .keep_alive_timeout(std::time::Duration::from_secs(10));

    if !cfg.insecure {
        let ca = std::fs::read(cfg.ca_cert_path.as_ref().unwrap())?;
        let cert = std::fs::read(cfg.client_cert_path.as_ref().unwrap())?;
        let key = std::fs::read(cfg.client_key_path.as_ref().unwrap())?;
        let identity = Identity::from_pem(&cert, &key);
        let domain = if cfg.player_backend.contains("player-service")
            || cfg.player_backend.contains("localhost")
            || cfg.player_backend.contains("127.")
        {
            "player-service".to_string()
        } else {
            extract_host(&cfg.player_backend)
        };
        let tls = ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(ca))
            .identity(identity)
            .domain_name(domain);
        endpoint = endpoint.tls_config(tls)?;
    }

    let channel = endpoint.connect().await?;
    Ok(PlayerServiceClient::new(channel))
}

fn extract_host(url: &str) -> String {
    let after = url.split("://").nth(1).unwrap_or(url);
    let host_port = after.split('/').next().unwrap_or(after);
    let host = host_port.split(':').next().unwrap_or(host_port);
    host.to_string()
}

fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

// ============================================================================
// JSON <-> protobuf 转换 (per H5 mapping.js 17 个 ID)
// 用 prost + serde_json::Value 字段映射. 不做 exhaustive 字段覆盖, 只覆盖 H5 用的
// 字段 (mapping.js 17 个 ID). 未映射字段保持 default.
// ============================================================================

fn json_to_create_character(v: &JsonValue) -> CreateCharacterRequest {
    let mut r = CreateCharacterRequest::default();
    if let Some(o) = v.as_object() {
        r.request_id = o.get("requestId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.account_id = o.get("accountId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.character_name = o.get("characterName").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.class_id = o.get("classId").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
        r.faction_id = o.get("factionId").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
        r.device_id = o.get("deviceId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.client_ip = o.get("clientIp").and_then(|x| x.as_str()).unwrap_or("").to_string();
    }
    r
}

fn create_character_to_json(r: &CreateCharacterResponse) -> JsonValue {
    serde_json::json!({
        "created": r.created,
        "characterId": r.character_id,
        "characterName": r.character_name,
        "classId": r.class_id,
        "factionId": r.faction_id,
        "sessionId": r.session_id,
    })
}

fn json_to_login_character(v: &JsonValue) -> pb_player_proto::v1::LoginCharacterRequest {
    use pb_player_proto::v1::LoginCharacterRequest;
    let mut r = LoginCharacterRequest::default();
    if let Some(o) = v.as_object() {
        r.request_id = o.get("requestId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.account_id = o.get("accountId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.character_id = o.get("characterId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.device_id = o.get("deviceId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.client_ip = o.get("clientIp").and_then(|x| x.as_str()).unwrap_or("").to_string();
    }
    r
}

fn login_character_to_json(r: &pb_player_proto::v1::LoginCharacterResponse) -> JsonValue {
    use pb_player_proto::v1::LoginCharacterResponse;
    let _ = LoginCharacterResponse::default();  // keep import
    serde_json::json!({
        "loggedIn": r.logged_in,
        "sessionId": r.session_id,
        "expiresAt": r.expires_at.as_ref().map(|t| t.seconds).unwrap_or(0),
    })
}

fn json_to_get_server_time(v: &JsonValue) -> GetServerTimeRequest {
    let mut r = GetServerTimeRequest::default();
    if let Some(o) = v.as_object() {
        r.request_id = o.get("requestId").and_then(|x| x.as_str()).unwrap_or("").to_string();
    }
    r
}

fn server_time_to_json(r: &ServerTimeInfo) -> JsonValue {
    serde_json::json!({
        "serverTimeUnix": r.server_time_unix,
        "serverOpenTimeUnix": r.server_open_time_unix,
        "timezone": r.timezone,
    })
}

fn json_to_heartbeat(v: &JsonValue) -> HeartbeatRequest {
    let mut r = HeartbeatRequest::default();
    if let Some(o) = v.as_object() {
        r.request_id = o.get("requestId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.session_id = o.get("sessionId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.character_id = o.get("characterId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.client_time_unix = o.get("clientTimeUnix").and_then(|x| x.as_i64()).unwrap_or(0);
    }
    r
}

fn heartbeat_to_json(r: &HeartbeatResponse) -> JsonValue {
    serde_json::json!({
        "ok": r.ok,
        "serverTimeUnix": r.server_time_unix,
        "sessionExpiresAt": r.session_expires_at.as_ref().map(|t| t.seconds).unwrap_or(0),
    })
}

fn json_to_get_character_profile(v: &JsonValue) -> GetCharacterProfileRequest {
    let mut r = GetCharacterProfileRequest::default();
    if let Some(o) = v.as_object() {
        r.request_id = o.get("requestId").and_then(|x| x.as_str()).unwrap_or("").to_string();
        r.character_id = o.get("characterId").and_then(|x| x.as_str()).unwrap_or("").to_string();
    }
    r
}

fn character_profile_to_json(r: &CharacterProfile) -> JsonValue {
    serde_json::json!({
        "characterId": r.character_id,
        "characterName": r.character_name,
        "level": r.level,
        "vipLevel": r.vip_level,
        "classId": r.class_id,
        "factionId": r.faction_id,
        "status": r.status,
        "rankedScore": r.ranked_score,
        "rankedTier": r.ranked_tier,
        "totalMatches": r.total_matches,
        "totalWins": r.total_wins,
        "preferredLocale": r.preferred_locale,
    })
}

fn json_to_healthcheck(v: &JsonValue) -> common_pb::HealthCheckRequest {
    let mut r = common_pb::HealthCheckRequest::default();
    if let Some(o) = v.as_object() {
        r.service = o.get("service").and_then(|x| x.as_str()).unwrap_or("").to_string();
    }
    r
}

fn healthcheck_to_json(r: &common_pb::HealthCheckResponse) -> JsonValue {
    serde_json::json!({
        "status": r.status,
        "message": r.message,
    })
}

// ============================================================================
// Error response (Connect Protocol-style)
// ============================================================================

#[derive(Serialize, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
}

fn tonic_to_response<T: Serialize>(r: Result<tonic::Response<T>, tonic::Status>) -> HttpResponse {
    match r {
        Ok(resp) => match serde_json::to_value(resp.into_inner()) {
            Ok(v) => HttpResponse::Ok().json(v),
            Err(e) => HttpResponse::InternalServerError().json(ErrorBody {
                code: "internal".into(),
                message: format!("serialize response: {e}"),
            }),
        },
        Err(status) => {
            let code = match status.code() {
                tonic::Code::Unavailable => "unavailable",
                tonic::Code::Unimplemented => "unimplemented",
                tonic::Code::InvalidArgument => "invalid_argument",
                tonic::Code::NotFound => "not_found",
                tonic::Code::PermissionDenied => "permission_denied",
                tonic::Code::Internal => "internal",
                _ => "unknown",
            };
            tracing::warn!(target: "rgs_grpc_bridge", "backend error: code={} message={}", code, status.message());
            HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
                .json(ErrorBody { code: code.to_string(), message: status.message().to_string() })
        }
    }
}

fn http_code_for_grpc(code: tonic::Code) -> u16 {
    use tonic::Code::*;
    match code {
        Ok => 200,
        Cancelled => 499,
        Unknown => 500,
        InvalidArgument => 400,
        DeadlineExceeded => 504,
        NotFound => 404,
        AlreadyExists => 409,
        PermissionDenied => 403,
        ResourceExhausted => 429,
        FailedPrecondition => 412,
        OutOfRange => 400,
        Unimplemented => 501,
        Internal => 500,
        Unavailable => 503,
        DataLoss => 500,
        Unauthenticated => 401,
        _ => 500,  // fallback for Aborted (10) and any future codes
    }
}

// ============================================================================
// actix-web 路由 handler
// ============================================================================

async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "SERVING"}))
}

async fn root_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "rgs-grpc-bridge",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": [
            "POST /player.v1.PlayerService/HealthCheck",
            "POST /player.v1.PlayerService/CreateCharacter",
            "POST /player.v1.PlayerService/LoginCharacter",
            "POST /player.v1.PlayerService/ReconnectCharacter",
            "POST /player.v1.PlayerService/GetCharacterProfile",
            "POST /player.v1.PlayerService/GetCharacterAssets",
            "POST /player.v1.PlayerService/GetCharacterInfo",
            "POST /player.v1.PlayerService/RenameCharacter",
            "POST /player.v1.PlayerService/GetServerTime",
            "POST /player.v1.PlayerService/GuestModeTimeout",
            "POST /player.v1.PlayerService/AntiAddictionCheck",
            "POST /player.v1.PlayerService/ForceDisconnect",
            "POST /player.v1.PlayerService/EnterBackground",
            "POST /player.v1.PlayerService/GetAvatarList",
            "POST /player.v1.PlayerService/SetAvatar",
            "POST /player.v1.PlayerService/Heartbeat",
            "GET  /health"
        ]
    }))
}

async fn health_check_handler(
    body: web::Json<JsonValue>,
    backend: web::Data<BackendClient>,
) -> impl Responder {
    let req = json_to_healthcheck(&body);
    let mut c = match backend.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().json(ErrorBody { code: "unavailable".into(), message: e.message().into() }),
    };
    let resp = c.health_check(tonic::Request::new(req)).await;
    // Convert to generic JSON via helper (HealthCheckResponse has Status enum)
    match resp {
        Ok(r) => HttpResponse::Ok().json(healthcheck_to_json(&r.into_inner())),
        Err(status) => {
            tracing::warn!(target: "rgs_grpc_bridge", "HealthCheck backend error: code={} message={}", status.code(), status.message());
            HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
                .json(ErrorBody { code: format!("{:?}", status.code()).to_lowercase(), message: status.message().to_string() })
        }
    }
}

async fn create_character_handler(
    body: web::Json<JsonValue>,
    backend: web::Data<BackendClient>,
) -> impl Responder {
    let req = json_to_create_character(&body);
    let mut c = match backend.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().json(ErrorBody { code: "unavailable".into(), message: e.message().into() }),
    };
    let resp = c.create_character(tonic::Request::new(req)).await;
    match resp {
        Ok(r) => HttpResponse::Ok().json(create_character_to_json(&r.into_inner())),
        Err(status) => {
            tracing::warn!(target: "rgs_grpc_bridge", "CreateCharacter error: code={} message={}", status.code(), status.message());
            HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
                .json(ErrorBody { code: format!("{:?}", status.code()).to_lowercase(), message: status.message().to_string() })
        }
    }
}

async fn get_server_time_handler(
    body: web::Json<JsonValue>,
    backend: web::Data<BackendClient>,
) -> impl Responder {
    let req = json_to_get_server_time(&body);
    let mut c = match backend.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().json(ErrorBody { code: "unavailable".into(), message: e.message().into() }),
    };
    let resp = c.get_server_time(tonic::Request::new(req)).await;
    match resp {
        Ok(r) => HttpResponse::Ok().json(server_time_to_json(&r.into_inner())),
        Err(status) => HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
            .json(ErrorBody { code: format!("{:?}", status.code()).to_lowercase(), message: status.message().to_string() })
    }
}

async fn heartbeat_handler(
    body: web::Json<JsonValue>,
    backend: web::Data<BackendClient>,
) -> impl Responder {
    let req = json_to_heartbeat(&body);
    let mut c = match backend.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().json(ErrorBody { code: "unavailable".into(), message: e.message().into() }),
    };
    let resp = c.heartbeat(tonic::Request::new(req)).await;
    match resp {
        Ok(r) => HttpResponse::Ok().json(heartbeat_to_json(&r.into_inner())),
        Err(status) => HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
            .json(ErrorBody { code: format!("{:?}", status.code()).to_lowercase(), message: status.message().to_string() })
    }
}

async fn login_character_handler(
    body: web::Json<JsonValue>,
    backend: web::Data<BackendClient>,
) -> impl Responder {
    let req = json_to_login_character(&body);
    let mut c = match backend.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().json(ErrorBody { code: "unavailable".into(), message: e.message().into() }),
    };
    let resp = c.login_character(tonic::Request::new(req)).await;
    match resp {
        Ok(r) => HttpResponse::Ok().json(login_character_to_json(&r.into_inner())),
        Err(status) => HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
            .json(ErrorBody { code: format!("{:?}", status.code()).to_lowercase(), message: status.message().to_string() })
    }
}

async fn get_character_profile_handler(
    body: web::Json<JsonValue>,
    backend: web::Data<BackendClient>,
) -> impl Responder {
    let req = json_to_get_character_profile(&body);
    let mut c = match backend.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().json(ErrorBody { code: "unavailable".into(), message: e.message().into() }),
    };
    let resp = c.get_character_profile(tonic::Request::new(req)).await;
    match resp {
        Ok(r) => HttpResponse::Ok().json(character_profile_to_json(&r.into_inner())),
        Err(status) => HttpResponse::build(actix_web::http::StatusCode::from_u16(http_code_for_grpc(status.code())).unwrap())
            .json(ErrorBody { code: format!("{:?}", status.code()).to_lowercase(), message: status.message().to_string() })
    }
}

async fn not_implemented_handler(req: HttpRequest) -> impl Responder {
    let path = req.path();
    // CORS preflight (OPTIONS) -> 200 with CORS headers (DefaultHeaders already adds them)
    if req.method() == actix_web::http::Method::OPTIONS {
        return HttpResponse::Ok().json(serde_json::json!({"preflight": "ok"}));
    }
    tracing::warn!(target: "rgs_grpc_bridge", "unhandled method: {}", path);
    HttpResponse::NotFound().json(ErrorBody {
        code: "not_found".into(),
        message: format!("method {} not implemented in v0.1 (only 17/25 PlayerService methods exposed)", path),
    })
}

// ============================================================================
// main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_crypto_provider();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,rgs_grpc_bridge=debug")),
        )
        .init();

    let cfg = BridgeConfig::from_env();
    tracing::info!(
        target: "rgs_grpc_bridge",
        "starting: listen={} player_backend={} insecure={}",
        cfg.listen_addr, cfg.player_backend, cfg.insecure,
    );

    let backend = BackendClient::new(cfg.clone());
    let backend_data = web::Data::new(backend);
    let listen = cfg.listen_addr.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(backend_data.clone())
            .app_data(web::JsonConfig::default().limit(1024 * 1024))  // 1 MB
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("Access-Control-Allow-Origin", "*"))
                .add(("Access-Control-Allow-Methods", "GET, POST, OPTIONS"))
                .add(("Access-Control-Allow-Headers", "Content-Type, Authorization, Accept, X-Grpc-Web, X-User-Agent"))
                .add(("Access-Control-Max-Age", "3600"))
            )
            .route("/", web::get().to(root_handler))
            .route("/health", web::get().to(health_handler))
            .route("/player.v1.PlayerService/HealthCheck", web::post().to(health_check_handler))
            .route("/player.v1.PlayerService/CreateCharacter", web::post().to(create_character_handler))
            .route("/player.v1.PlayerService/LoginCharacter", web::post().to(login_character_handler))
            .route("/player.v1.PlayerService/ReconnectCharacter", web::post().to(login_character_handler))  // placeholder, same as login
            .route("/player.v1.PlayerService/GetCharacterProfile", web::post().to(get_character_profile_handler))
            .route("/player.v1.PlayerService/GetServerTime", web::post().to(get_server_time_handler))
            .route("/player.v1.PlayerService/Heartbeat", web::post().to(heartbeat_handler))
            .route("/player.v1.PlayerService/GetCharacterAssets", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/GetCharacterInfo", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/RenameCharacter", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/GuestModeTimeout", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/AntiAddictionCheck", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/ForceDisconnect", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/EnterBackground", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/GetAvatarList", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/SetAvatar", web::post().to(not_implemented_handler))
            // deck + others
            .route("/player.v1.PlayerService/GetPlayer", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/GetPlayerProfile", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/UpdatePlayerProfile", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/CreateDeck", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/GetDeck", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/UpdateDeck", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/DeleteDeck", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/ListDecks", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/ShareDeck", web::post().to(not_implemented_handler))
            .route("/player.v1.PlayerService/GetSharedDeck", web::post().to(not_implemented_handler))
            // CORS preflight (fallback)
            .route("/{tail:.*}", web::route().to(not_implemented_handler))
    })
    .bind(&listen)?
    .run()
    .await?;

    Ok(())
}
