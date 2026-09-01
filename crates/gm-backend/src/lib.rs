//! gm-backend lib — GM 后台 APIGW 业务逻辑
//!
//! 第 8 域微服务 (per Ulysses 2026-08-27 12:43 JST 明确指令),
//! per RGS-BAS-003 §2.1 设计: HTTPS 入口 + RBAC + mTLS 调 admin-service gRPC.
//!
//! ## 2026-09-01 actix-web 重写 (per Ulysses 决策)
//! 替换原 axum 0.7 → actix-web 4.0, 保留 5 现有 GM endpoint + JWT + admin gRPC client +
//! circuit breaker, 补全 ROPE_CS 9 端点 + 4 业务模块 + SSE 实时事件流.
//!
//! ## 公开 API
//! - `GmConfig::from_env()` — 配载 + 解析
//! - `AppState` — handler 共享状态 (含 8 个 ROPE_CS 移植的内存模块)
//! - `register_routes(cfg)` — 把全部 15+ 端点 + SSE 注册到 actix-web ServiceConfig
//! - `register_health_routes(cfg)` — 探针路由 (8081, 走 /healthz /readyz)
//! - 业务 handler: health_view, ban_account, grant_compensation, set_maintenance,
//!   query_audit, login, list_players, broadcast, list_anchors, send_canvas_command,
//!   list_servers, start_server, stop_server, list_mall_items, create_mall_item, ...
//! - JWT middleware: `JwtAuth` (actix-web Transform)
//! - bcrypt password hash (ROPE_CS 同等功能)
//!
//! main.rs 只做 entry point, 把所有可测部分放在这里.

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    web, Error as ActixError, HttpMessage, HttpRequest, HttpResponse,
};
use anyhow::{Context, Result};
use chrono::Utc;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{fmt, EnvFilter};

// ============================================================================
// proto include
// ============================================================================

#[allow(clippy::result_large_err)]
pub mod admin {
    pub mod v1 {
        tonic::include_proto!("admin.v1");
    }
}
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
pub mod proto {
    #![allow(clippy::all)]
    pub mod v1 {
        tonic::include_proto!("gm.v1");
    }
}

pub mod circuit_breaker;

// 5 GM 业务 handler (per gm.proto v0.4)
pub mod business_handler;

// 补全 4 端点 + SSE handler (per ROPE_CS 移植)
pub mod auth_handler;
pub mod players_handler;
pub mod broadcast_handler;
pub mod canvas_handler;
pub mod servers_handler;
pub mod mall_handler;
pub mod items_handler;
pub mod support_handler;
pub mod reports_handler;
pub mod summary_handler;

pub mod test_helpers;

// re-export
pub use business_handler::{
    ban_account, grant_compensation, health_view, query_audit, set_maintenance,
    BanAccountRequestBody, CompensationRequestBody as GrantCompensationRequestBody,
    HealthViewQuery, MaintenanceRequestBody as SetMaintenanceRequestBody, QueryAuditLogQuery,
};
pub use auth_handler::{login, LoginRequest, LoginResponse, AdminRecord};
pub use players_handler::{list_players, get_player_stats, PlayersQuery, PlayersResponse, PlayerStatsResponse};
pub use broadcast_handler::{broadcast, list_broadcasts, BroadcastRequest, BroadcastEntry, sse_events};
pub use canvas_handler::{list_anchors, send_canvas_command, CanvasCommandRequest, AnchorOption};
pub use servers_handler::{list_servers, get_server_stats, start_server, stop_server, metrics, ServerEntry, ServerStats};
pub use mall_handler::{list_mall_items, create_mall_item, update_mall_item, delete_mall_item, MallItem};
pub use items_handler::{grant_item, list_grants, GrantRequest, GrantEntry};
pub use support_handler::{create_ticket, list_tickets, update_ticket_status, TicketEntry, CreateTicketRequest, UpdateTicketStatusRequest};
pub use reports_handler::{list_reports, ReportEntry};
pub use summary_handler::summary;

// ============================================================================
// 配置
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmConfig {
    pub http_addr: SocketAddr,
    pub health_addr: SocketAddr,
    pub admin_grpc_endpoint: String,
    pub jwt_secret: String,
    pub require_jwt: bool,
    pub disable_admin_grpc: bool,
}

impl GmConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let http_addr: SocketAddr = std::env::var("GM_HTTP_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8443".to_string())
            .parse()
            .context("invalid GM_HTTP_ADDR")?;
        let health_addr: SocketAddr = std::env::var("GM_HEALTH_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8081".to_string())
            .parse()
            .context("invalid GM_HEALTH_ADDR")?;
        let admin_grpc_endpoint = std::env::var("ADMIN_GRPC_ENDPOINT")
            .unwrap_or_else(|_| "https://admin-service:50055".to_string());
        let jwt_secret = std::env::var("GM_JWT_SECRET")
            .unwrap_or_else(|_| "dev-only-do-not-use-in-prod".to_string());
        let require_jwt = std::env::var("GM_REQUIRE_JWT")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        let disable_admin_grpc = std::env::var("GM_DISABLE_ADMIN_GRPC")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        Ok(Self {
            http_addr,
            health_addr,
            admin_grpc_endpoint,
            jwt_secret,
            require_jwt,
            disable_admin_grpc,
        })
    }

    pub fn for_test(http: &str, health: &str, admin: &str) -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: http.parse()?,
            health_addr: health.parse()?,
            admin_grpc_endpoint: admin.to_string(),
            jwt_secret: "test-secret".to_string(),
            require_jwt: false,
            disable_admin_grpc: true,
        })
    }
}

// ============================================================================
// Audit store trait + InMemory
// ============================================================================

pub trait AuditStore: Send + Sync + 'static {
    fn append(&self, entry: AuditLogEntry);
    fn list_entries(
        &self,
        limit: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<AuditLogEntry>> + Send + '_>>;
}

#[derive(Default, Clone)]
pub struct InMemoryAuditStore {
    entries: Arc<std::sync::Mutex<Vec<AuditLogEntry>>>,
}

impl InMemoryAuditStore {
    pub fn new() -> Self { Self::default() }
    pub fn append(&self, entry: AuditLogEntry) { self.entries.lock().unwrap().push(entry); }
}

impl AuditStore for InMemoryAuditStore {
    fn append(&self, entry: AuditLogEntry) { self.entries.lock().unwrap().push(entry); }
    fn list_entries(
        &self,
        limit: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<AuditLogEntry>> + Send + '_>> {
        Box::pin(async move {
            let guard = self.entries.lock().unwrap();
            guard.iter().rev().take(limit).cloned().collect()
        })
    }
}

// ============================================================================
// AppState — 含 8 个 ROPE_CS 移植的内存模块
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GmConfig>,
    pub audit_store: Arc<dyn AuditStore>,
    pub admin_grpc: Option<Arc<AdminGrpcClient>>,
    /// ROPE_CS 移植: SSE 实时事件总线
    pub broadcast_tx: tokio::sync::broadcast::Sender<BroadcastEntry>,
    /// ROPE_CS 移植: admin 列表 (init 时 ensure_default_admin 创建 superadmin)
    pub admins: Arc<std::sync::Mutex<Vec<AdminRecord>>>,
    /// ROPE_CS 移植: mall items
    pub mall_items: Arc<std::sync::Mutex<Vec<MallItem>>>,
    /// ROPE_CS 移植: grants
    pub grants: Arc<std::sync::Mutex<Vec<GrantEntry>>>,
    /// ROPE_CS 移植: tickets
    pub tickets: Arc<std::sync::Mutex<Vec<TicketEntry>>>,
    /// ROPE_CS 移植: reports
    pub reports: Arc<std::sync::Mutex<Vec<ReportEntry>>>,
    /// ROPE_CS 移植: servers 列表 + state (5 假 server 初始化)
    pub servers: Arc<std::sync::Mutex<Vec<ServerEntry>>>,
}

impl AppState {
    pub fn new(config: GmConfig) -> Self {
        let audit_store: Arc<dyn AuditStore> = Arc::new(InMemoryAuditStore::new());
        Self::with_audit_store(config, audit_store)
    }

    pub fn with_audit_store(config: GmConfig, audit_store: Arc<dyn AuditStore>) -> Self {
        let config = Arc::new(config);
        let admin_grpc = if config.disable_admin_grpc {
            None
        } else {
            let client = AdminGrpcClient::try_connect(&config).ok().map(Arc::new);
            if client.is_none() {
                tracing::warn!(
                    "admin-service gRPC not reachable at {} (fail-open)",
                    config.admin_grpc_endpoint
                );
            }
            client
        };
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);
        let admins = Arc::new(std::sync::Mutex::new(Vec::<AdminRecord>::new()));
        let mall_items = Arc::new(std::sync::Mutex::new(Vec::<MallItem>::new()));
        let grants = Arc::new(std::sync::Mutex::new(Vec::<GrantEntry>::new()));
        let tickets = Arc::new(std::sync::Mutex::new(Vec::<TicketEntry>::new()));
        let reports = Arc::new(std::sync::Mutex::new(Vec::<ReportEntry>::new()));
        let servers = Arc::new(std::sync::Mutex::new(vec![
            ServerEntry { id: "player-1".into(), name: "Player Shard 1".into(), region: Some("ap-east-1".into()), status: "running".into(), online_players: 1284, last_updated: Some(Utc::now().to_rfc3339()) },
            ServerEntry { id: "player-2".into(), name: "Player Shard 2".into(), region: Some("ap-east-1".into()), status: "running".into(), online_players: 982, last_updated: Some(Utc::now().to_rfc3339()) },
            ServerEntry { id: "match-1".into(), name: "Match Service 1".into(), region: Some("us-west-2".into()), status: "running".into(), online_players: 421, last_updated: Some(Utc::now().to_rfc3339()) },
            ServerEntry { id: "social-1".into(), name: "Social Shard 1".into(), region: Some("eu-central-1".into()), status: "stopped".into(), online_players: 0, last_updated: Some(Utc::now().to_rfc3339()) },
            ServerEntry { id: "economy-1".into(), name: "Economy Shard 1".into(), region: Some("ap-east-1".into()), status: "running".into(), online_players: 0, last_updated: Some(Utc::now().to_rfc3339()) },
        ]));
        Self {
            config,
            audit_store,
            admin_grpc,
            broadcast_tx,
            admins,
            mall_items,
            grants,
            tickets,
            reports,
            servers,
        }
    }

    /// ROPE_CS 移植: ensure_default_admin — 创建默认 superadmin (admin/adminpass)
    pub async fn ensure_default_admin(&self) {
        let mut admins = self.admins.lock().unwrap();
        if !admins.iter().any(|a| a.username == "admin") {
            let hashed = bcrypt::hash("adminpass", 12).unwrap_or_default();
            admins.push(AdminRecord {
                username: "admin".to_string(),
                password_hash: hashed,
                role: "superadmin".to_string(),
            });
            tracing::info!("default superadmin 'admin' created (password: adminpass)");
        }
    }
}

// ============================================================================
// AdminGrpcClient (gRPC client 跟 web 框架无关, 直接保留)
// ============================================================================

pub struct AdminGrpcClient {
    client: crate::admin::v1::admin_service_client::AdminServiceClient<tonic::transport::Channel>,
    breaker: std::sync::Arc<crate::circuit_breaker::CircuitBreaker>,
}

impl AdminGrpcClient {
    pub fn try_connect(config: &GmConfig) -> Result<Self> {
        let endpoint = tonic::transport::Endpoint::from_shared(config.admin_grpc_endpoint.clone())
            .context("invalid admin_grpc_endpoint")?
            .timeout(Duration::from_millis(500))
            .connect_timeout(Duration::from_millis(500));
        let channel = endpoint.connect_lazy();
        let client = crate::admin::v1::admin_service_client::AdminServiceClient::new(channel);
        Ok(Self {
            client,
            breaker: std::sync::Arc::new(crate::circuit_breaker::CircuitBreaker::default()),
        })
    }

    pub async fn health_check(&self) -> Result<()> {
        use crate::common::v1::HealthCheckRequest;
        if !self.breaker.try_acquire() { anyhow::bail!("circuit breaker OPEN"); }
        let mut client = self.client.clone();
        let req = HealthCheckRequest { service: "gm-backend".to_string() };
        let result = client.health_check(req).await.context("admin-service health_check RPC failed");
        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }
        result?;
        Ok(())
    }

    pub async fn ban_account(
        &self,
        req: crate::admin::v1::BanAccountRequest,
    ) -> Result<crate::admin::v1::BanAccountResponse> {
        if !self.breaker.try_acquire() { anyhow::bail!("circuit breaker OPEN"); }
        let mut client = self.client.clone();
        let result = client.ban_account(req).await.context("admin-service ban_account RPC failed");
        match &result { Ok(_) => self.breaker.record_success(), Err(_) => self.breaker.record_failure() }
        Ok(result?.into_inner())
    }

    pub async fn grant_compensation(
        &self,
        req: crate::admin::v1::GrantCompensationRequest,
    ) -> Result<crate::admin::v1::GrantCompensationResponse> {
        if !self.breaker.try_acquire() { anyhow::bail!("circuit breaker OPEN"); }
        let mut client = self.client.clone();
        let result = client.grant_compensation(req).await.context("admin-service grant_compensation RPC failed");
        match &result { Ok(_) => self.breaker.record_success(), Err(_) => self.breaker.record_failure() }
        Ok(result?.into_inner())
    }

    pub async fn set_maintenance(
        &self,
        req: crate::admin::v1::SetMaintenanceRequest,
    ) -> Result<crate::admin::v1::SetMaintenanceResponse> {
        if !self.breaker.try_acquire() { anyhow::bail!("circuit breaker OPEN"); }
        let mut client = self.client.clone();
        let result = client.set_maintenance(req).await.context("admin-service set_maintenance RPC failed");
        match &result { Ok(_) => self.breaker.record_success(), Err(_) => self.breaker.record_failure() }
        Ok(result?.into_inner())
    }

    pub async fn query_audit_log(
        &self,
        req: crate::admin::v1::QueryAuditLogRequest,
    ) -> Result<crate::admin::v1::QueryAuditLogResponse> {
        if !self.breaker.try_acquire() { anyhow::bail!("circuit breaker OPEN"); }
        let mut client = self.client.clone();
        let result = client.query_audit_log(req).await.context("admin-service query_audit_log RPC failed");
        match &result { Ok(_) => self.breaker.record_success(), Err(_) => self.breaker.record_failure() }
        Ok(result?.into_inner())
    }
}

// ============================================================================
// 共享类型
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub log_id: String,
    pub admin_id: String,
    pub action: String,
    pub target_id: String,
    pub occurred_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthEntry {
    pub service_name: String,
    pub ready: bool,
    pub queue_depth: u32,
    pub db_pool_usage_ratio: f32,
    pub checked_at_ms: i64,
}

// ============================================================================
// JWT
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub roles: Vec<String>,
}

pub fn issue_jwt(secret: &str, sub: &str, roles: Vec<String>, ttl_seconds: i64) -> Result<String> {
    let exp = (Utc::now().timestamp() + ttl_seconds) as usize;
    let claims = Claims { sub: sub.to_string(), exp, roles };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .context("encode jwt")
}

pub fn verify_jwt(secret: &str, token: &str) -> Result<Claims> {
    let validation = Validation::default();
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .context("decode jwt")?;
    Ok(data.claims)
}

pub fn init_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,gm-backend=debug")),
        )
        .init();
}

// ============================================================================
// JWT middleware (actix-web Transform)
// ============================================================================

pub struct JwtAuth {
    pub require: bool,
    pub secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            require: self.require,
            secret: self.secret.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    require: bool,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let require = self.require;
        let secret = self.secret.clone();

        Box::pin(async move {
            if !require {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }
            let token = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            match token {
                Some(t) => match verify_jwt(&secret, &t) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        let res = service.call(req).await?;
                        Ok(res.map_into_left_body())
                    }
                    Err(e) => {
                        tracing::debug!("jwt verify failed: {e}");
                        let (req_parts, _payload) = req.into_parts();
                        let resp = HttpResponse::Unauthorized()
                            .json(json!({"error": "invalid_token"}));
                        Ok(ServiceResponse::new(req_parts, resp).map_into_right_body())
                    }
                },
                None => {
                    let (req_parts, _payload) = req.into_parts();
                    let resp = HttpResponse::Unauthorized()
                        .json(json!({"error": "missing_bearer_token"}));
                    Ok(ServiceResponse::new(req_parts, resp).map_into_right_body())
                }
            }
        })
    }
}

pub fn extract_claims(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

// ============================================================================
// Routes 注册函数 (actix-web 标准 ServiceConfig 模式)
// ============================================================================

/// 把全部 15+ 端点 + SSE 注册到 actix-web ServiceConfig
/// main.rs 用: App::new().app_data(state).wrap(JwtAuth{...}).configure(register_routes)
pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/gm")
            // 公开端点
            .route("/login", web::post().to(login))
            // 鉴权后端点 — 中间件全局 (注: SSE 在 /gm/events 单独注册, 不走这个 scope)
            .service(
                web::scope("")
                    // ping
                    .route("/ping", web::get().to(ping))
                    // 5 GM endpoint
                    .route("/health_view", web::get().to(health_view))
                    .route("/ban_account", web::post().to(ban_account))
                    .route("/grant_compensation", web::post().to(grant_compensation))
                    .route("/set_maintenance", web::post().to(set_maintenance))
                    .route("/query_audit", web::get().to(query_audit))
                    // admin 管理
                    .route("/admins", web::post().to(auth_handler::create_admin))
                    .route("/admins", web::get().to(auth_handler::list_admins))
                    // 4 补全端点 (ROPE_CS 移植)
                    .route("/players", web::get().to(list_players))
                    .route("/broadcast", web::post().to(broadcast))
                    .route("/broadcasts", web::get().to(list_broadcasts))
                    .route("/canvas/anchors", web::get().to(list_anchors))
                    .route("/canvas/send", web::post().to(send_canvas_command))
                    // 4 业务模块 (ROPE_CS 移植)
                    .route("/servers", web::get().to(list_servers))
                    .route("/servers/{id}/start", web::post().to(start_server))
                    .route("/servers/{id}/stop", web::post().to(stop_server))
                    .route("/metrics", web::get().to(metrics))
                    .route("/mall/items", web::get().to(list_mall_items))
                    .route("/mall/items", web::post().to(create_mall_item))
                    .route("/mall/items/{id}", web::put().to(update_mall_item))
                    .route("/mall/items/{id}", web::delete().to(delete_mall_item))
                    .route("/items/grant", web::post().to(grant_item))
                    .route("/items/grants", web::get().to(list_grants))
                    .route("/support", web::post().to(create_ticket))
                    .route("/support/tickets", web::get().to(list_tickets))
                    .route("/support/tickets/{id}", web::patch().to(update_ticket_status))
                    .route("/reports", web::get().to(list_reports))
                    // 1 聚合 (Dashboard 数据源)
                    .route("/summary", web::get().to(summary))
                    // 1 SSE 实时事件流 — 在 events.rs 内部手动验证 JWT
                    .route("/events", web::get().to(sse_events)),
            ),
    );
}

/// health 探针路由 (k8s exec 探针, 8081)
pub fn register_health_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz))
        .route("/readyz", web::get().to(readyz));
}

async fn ping() -> HttpResponse { HttpResponse::Ok().json(json!({"status": "ok"})) }
async fn healthz() -> HttpResponse { HttpResponse::Ok().json(json!({"status": "healthy"})) }
async fn readyz() -> HttpResponse { HttpResponse::Ok().json(json!({"status": "ready"})) }
