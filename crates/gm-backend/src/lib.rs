//! gm-backend lib — GM 后台 APIGW 业务逻辑
//!
//! 第 8 域微服务(per Ulysses 2026-08-27 12:43 JST 明确指令),
//! per RGS-BAS-003 §2.1 设计:HTTPS 入口 + RBAC + mTLS 调 admin-service gRPC.
//!
//! ## 公开 API
//! - `GmConfig::from_env()` — 配载 + 解析
//! - `AppState` — handler 共享状态
//! - `build_router(state)` — 完整 axum Router
//! - `build_health_router(state)` — health-only Router(8081 探针用)
//! - 6 个 handler:healthz, readyz, health_view, ban_account,
//!   grant_compensation, set_maintenance, query_audit
//! - JWT middleware(per TBD-08-01 v0.2 实装)
//! - 5 GM endpoint 字段级协议(per F8 处置 v0.2)
//!
//! main.rs 只做 entry point,把所有可测部分放在这里。

use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{fmt, EnvFilter};

/// S4 Phase 2 step 1: gm-backend 作为 admin-service 的 gRPC client
/// tonic-build 生成的 `admin.v1` 路径 = `gm_backend::admin::v1::*`,
/// 引用 `common.v1` 时用 `crate::common::v1::*` (平铺 2 个 include_proto)
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

// W18 (2026-08-28): Circuit breaker (5 次失败 → 30s 断开)
// W20 (2026-08-28): wire 到 AdminGrpcClient 4 method (gm-backend 4 endpoint)
// W23 (2026-08-28): integration 5 IT 验证 wire + chaos 集成
// 关联: RGS-OPEN-QA v0.4 DDD Review 决议
pub mod circuit_breaker;

// W26 (2026-08-29) 桶 2a: 5 GM endpoint 业务实装(从 lib.rs 移出)
// - Json/Query extractor + 字段级校验
// - admin-service gRPC + 失败降级 InMemory
// - 关联: RGS-PLAN-WBS-token-bucket-v0.3 §2.2.1
pub mod business_handler;

// W26 (2026-08-29) 桶 2a: 5 it_business_*.rs 测试用 build_test_state
// - 暴露在 lib 根路径, 简化 test 文件 import
pub mod test_helpers;

// W26 (2026-08-29) 桶 2a: re-export 5 handler + 5 DTO types at lib 根路径
// - 让 it_business_*.rs 可以 `gm_backend::ban_account` 直接引用
// - 兼容外部 (admin CLI / curl / health probe) 也可用 lib:: 类型
pub use business_handler::{
    ban_account, grant_compensation, health_view, query_audit, set_maintenance,
    BanAccountRequestBody, CompensationRequestBody as GrantCompensationRequestBody,
    HealthViewQuery, MaintenanceRequestBody as SetMaintenanceRequestBody, QueryAuditLogQuery,
};

// ============================================================================
// 配置
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmConfig {
    pub http_addr: SocketAddr,
    pub health_addr: SocketAddr,
    pub admin_grpc_endpoint: String,
    pub jwt_secret: String,
    /// 是否要求 JWT 验证(per 5 域 RGS_ALLOW_INSECURE_GRPC=0 默认)
    /// dev 环境可设 GM_REQUIRE_JWT=0 跳过(per TBD-08-01)
    pub require_jwt: bool,
    /// S4 Phase 2 step 1: 是否禁用 admin-service gRPC 注入(测试用)
    /// dev/UT 环境设 true, AppState.admin_grpc 永远 None, HealthView 走 stub 行为
    /// 生产环境 (k3s) 设 false, AppState::new 尝试 connect admin-service
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

    /// 测试用 builder(避开 env,UT 友好, 默认 disable_admin_grpc=true)
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

/// Audit store trait — per TBD-08-04 v0.2 抽象
///
/// v0.2 默认实现 InMemoryAuditStore(测试用 + 5 域 outbox 暂未接通场景)
/// v0.3 实装 PgAuditStore 走 admin_db.audit_log 表(per TBD-08-04 延后)
pub trait AuditStore: Send + Sync + 'static {
    fn append(&self, entry: AuditLogEntry);
    fn list_entries(
        &self,
        limit: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<AuditLogEntry>> + Send + '_>>;
}

/// In-memory 默认实现
#[derive(Default, Clone)]
pub struct InMemoryAuditStore {
    entries: Arc<std::sync::Mutex<Vec<AuditLogEntry>>>,
}

impl InMemoryAuditStore {
    pub fn new() -> Self {
        Self::default()
    }
    /// 追加 entry(供 ban_account / grant_compensation 等 stub handler 调用)
    pub fn append(&self, entry: AuditLogEntry) {
        self.entries.lock().unwrap().push(entry);
    }
}

impl AuditStore for InMemoryAuditStore {
    fn append(&self, entry: AuditLogEntry) {
        self.entries.lock().unwrap().push(entry);
    }
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

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GmConfig>,
    /// v0.2 抽象:audit_log 存储(默认 InMemory,可替换为 PgAuditStore)
    pub audit_store: Arc<dyn AuditStore>,
    /// S4 Phase 2 step 1: admin-service gRPC client(失败时为 None, 降级)
    pub admin_grpc: Option<Arc<AdminGrpcClient>>,
}

impl AppState {
    /// 构造 AppState + 尝试连接 admin-service gRPC(失败不 panic,设 None 降级)
    pub fn new(config: GmConfig) -> Self {
        let audit_store: Arc<dyn AuditStore> = Arc::new(InMemoryAuditStore::new());
        Self::with_audit_store(config, audit_store)
    }

    /// 测试用 + 注入自定义 AuditStore + admin_grpc client
    pub fn with_audit_store(config: GmConfig, audit_store: Arc<dyn AuditStore>) -> Self {
        let config = Arc::new(config);
        let admin_grpc = if config.disable_admin_grpc {
            None
        } else {
            let client = AdminGrpcClient::try_connect(&config).ok().map(Arc::new);
            if client.is_none() {
                tracing::warn!(
                    "admin-service gRPC not reachable at {} (fail-open: HealthView will mark admin ready=false)",
                    config.admin_grpc_endpoint
                );
            }
            client
        };
        Self {
            config,
            audit_store,
            admin_grpc,
        }
    }
}

/// S4 Phase 2 step 1: admin-service gRPC client wrapper
/// - 包装 tonic Channel + AdminServiceClient
/// - 构造时 try_connect:失败返回 Err(供 AppState 降级为 None)
/// - 提供 `health_check(timeout)` 调 admin-service HealthCheck
pub struct AdminGrpcClient {
    client: crate::admin::v1::admin_service_client::AdminServiceClient<tonic::transport::Channel>,
    /// W18 (2026-08-28): Circuit breaker (5 次失败 → 30s 断开)
    /// W20 (2026-08-28): wire 到 4 method (gm-backend 4 endpoint)
    /// W23 (2026-08-28): wire 到 5 method (gm-backend 4 endpoint + 业务 1)
    breaker: std::sync::Arc<crate::circuit_breaker::CircuitBreaker>,
}

impl AdminGrpcClient {
    /// 尝试连接 admin-service(失败返 Err,AppState 设 None 降级)
    /// 用 `Endpoint::connect_lazy()` 构造 Channel(实际连接在第一次 RPC 时发生),
    /// 符合 fail-open 语义:AppState::with_audit_store 不会因 admin-service 未就绪而 panic
    ///
    /// W23 (2026-08-28): 加 CircuitBreaker 默认 (5 失败 → 30s 断开)
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

    /// 调 admin-service HealthCheck, 500ms timeout, 失败返 Err
    /// W23: 走 CircuitBreaker (共享 5 method 失败计数)
    pub async fn health_check(&self) -> Result<()> {
        use crate::common::v1::HealthCheckRequest;
        if !self.breaker.try_acquire() {
            anyhow::bail!("circuit breaker OPEN, skipping admin-service health_check");
        }
        let mut client = self.client.clone();
        let req = HealthCheckRequest {
            service: "gm-backend".to_string(),
        };
        let result = client
            .health_check(req)
            .await
            .context("admin-service health_check RPC failed");
        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }
        result?;
        Ok(())
    }

    // S4 Phase 2 step 2: 4 GM RPC client methods
    // 每个方法 500ms timeout, 失败返 Err(让 handler 降级到 InMemory fallback)

    pub async fn ban_account(
        &self,
        req: crate::admin::v1::BanAccountRequest,
    ) -> Result<crate::admin::v1::BanAccountResponse> {
        if !self.breaker.try_acquire() {
            anyhow::bail!("circuit breaker OPEN, skipping admin-service ban_account");
        }
        let mut client = self.client.clone();
        let result = client
            .ban_account(req)
            .await
            .context("admin-service ban_account RPC failed");
        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }
        Ok(result?.into_inner())
    }

    pub async fn grant_compensation(
        &self,
        req: crate::admin::v1::GrantCompensationRequest,
    ) -> Result<crate::admin::v1::GrantCompensationResponse> {
        if !self.breaker.try_acquire() {
            anyhow::bail!("circuit breaker OPEN, skipping admin-service grant_compensation");
        }
        let mut client = self.client.clone();
        let result = client
            .grant_compensation(req)
            .await
            .context("admin-service grant_compensation RPC failed");
        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }
        Ok(result?.into_inner())
    }

    pub async fn set_maintenance(
        &self,
        req: crate::admin::v1::SetMaintenanceRequest,
    ) -> Result<crate::admin::v1::SetMaintenanceResponse> {
        if !self.breaker.try_acquire() {
            anyhow::bail!("circuit breaker OPEN, skipping admin-service set_maintenance");
        }
        let mut client = self.client.clone();
        let result = client
            .set_maintenance(req)
            .await
            .context("admin-service set_maintenance RPC failed");
        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }
        Ok(result?.into_inner())
    }

    pub async fn query_audit_log(
        &self,
        req: crate::admin::v1::QueryAuditLogRequest,
    ) -> Result<crate::admin::v1::QueryAuditLogResponse> {
        if !self.breaker.try_acquire() {
            anyhow::bail!("circuit breaker OPEN, skipping admin-service query_audit_log");
        }
        let mut client = self.client.clone();
        let result = client
            .query_audit_log(req)
            .await
            .context("admin-service query_audit_log RPC failed");
        match &result {
            Ok(_) => self.breaker.record_success(),
            Err(_) => self.breaker.record_failure(),
        }
        Ok(result?.into_inner())
    }
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
// JWT middleware(per TBD-08-01 v0.2 实装)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub roles: Vec<String>,
}

/// 签发 JWT(供测试 + admin 端用,生产应该 admin-service 签发)
pub fn issue_jwt(secret: &str, sub: &str, roles: Vec<String>, ttl_seconds: i64) -> Result<String> {
    let exp = (Utc::now().timestamp() + ttl_seconds) as usize;
    let claims = Claims {
        sub: sub.to_string(),
        exp,
        roles,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("encode jwt")
}

/// 验证 JWT 签名 + 过期
pub fn verify_jwt(secret: &str, token: &str) -> Result<Claims> {
    let validation = Validation::default();
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .context("decode jwt")?;
    Ok(data.claims)
}

/// JWT middleware:从 Authorization: Bearer <token> 取 token,验证签名
/// per TBD-08-01 v0.2 实装(per RGS-BAS-003 §2.1 RBAC 链路)
pub async fn jwt_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    if !state.config.require_jwt {
        return next.run(req).await;
    }
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    let token = match auth_header {
        Some(s) if s.starts_with("Bearer ") => &s[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":"missing_bearer_token"})),
            )
                .into_response();
        }
    };
    match verify_jwt(&state.config.jwt_secret, token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"invalid_token","detail":e.to_string()})),
        )
            .into_response(),
    }
}

// ============================================================================
// 5 GM endpoint 字段级协议(per F8 处置 v0.2)
// ============================================================================

/// `SetMaintenanceModeResponse` 新增 `propagation_status` (PROPAGATING / CONVERGED)
/// per RGS-BAS-003 §3.3 + DTL-003 §3.3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropagationStatus {
    #[serde(rename = "PROPAGATING")]
    Propagating,
    #[serde(rename = "CONVERGED")]
    Converged,
}

/// `QueryHealthViewResponse` = `services[]` (ServiceHealthEntry)
/// per RGS-BAS-003 §3.4 + DTL-003 §3.4
/// 注:`db_pool_usage_ratio: f32` 不能 derive Eq(NaN 不等),只用 PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceHealthEntry {
    pub service_name: String,
    pub ready: bool,
    pub queue_depth: u32,
    pub db_pool_usage_ratio: f32,
    pub checked_at_ms: i64,
}

/// `QueryAuditLogResponse` = `entries[]` + `has_more`
/// per RGS-BAS-003 §3.4 + DTL-003 §3.4
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    pub log_id: String,
    pub admin_id: String,
    pub action: String,
    pub target_id: String,
    pub occurred_at_ms: i64,
}

// 注: 5 GM endpoint RequestBody/Query 类型已移至 `business_handler` 模块
// (per W26 桶 2a: lib.rs 只留 router 配置 + 类型 + 基础 service 抽象)

// ============================================================================
// Router 构建
// ============================================================================

pub fn build_router(state: AppState) -> Router {
    use crate::business_handler::{
        ban_account, grant_compensation, health_view, query_audit, set_maintenance,
    };
    let api = Router::new()
        .route("/api/v1/gm/health/view", get(health_view))
        .route("/api/v1/gm/ban", post(ban_account))
        .route("/api/v1/gm/compensation", post(grant_compensation))
        .route("/api/v1/gm/maintenance", post(set_maintenance))
        .route("/api/v1/audit/logs", get(query_audit))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_middleware,
        ));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(api)
        .with_state(state)
}

/// health-only router(8081,k8s 探针专用,**不挂 JWT,免探针误拒**)
pub fn build_health_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state)
}

// ============================================================================
// 2 health handler(per BAS-003 §2.1 探针专用)
// ============================================================================

pub async fn healthz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({"status":"ok","service":"gm-backend"})),
    )
}

pub async fn readyz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({"status":"ready","service":"gm-backend"})),
    )
}

// 注: 5 GM endpoint 业务 handler(health_view, ban_account, grant_compensation,
// set_maintenance, query_audit) 已移至 `business_handler` 模块
// (per W26 桶 2a + RGS-PLAN-WBS-token-bucket-v0.3 §2.2.1)
// lib.rs 只保留 router 配置 + 类型 + 基础 service 抽象
