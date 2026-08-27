//! gm-backend lib — GM 后台 APIGW 业务逻辑
//!
//! 第 8 域微服务(per Ulysses 2026-08-27 12:43 JST 明确指令),
//! per RGS-BAS-003 §2.1 设计:HTTPS 入口 + RBAC + mTLS 调 admin-service gRPC.
//!
//! 公开 API:
//! - `GmConfig::from_env()` — 配载 + 解析
//! - `AppState` — handler 共享状态
//! - `build_router(state)` — 完整 axum Router
//! - `build_health_router(state)` — health-only Router(8081 探针用)
//! - 6 个 handler:healthz, readyz, health_view, ban_account,
//!   grant_compensation, set_maintenance, query_audit
//!
//! main.rs 只做 entry point,把所有可测部分放在这里。

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmConfig {
    pub http_addr: SocketAddr,
    pub health_addr: SocketAddr,
    pub admin_grpc_endpoint: String,
    pub jwt_secret: String,
}

impl GmConfig {
    /// 从环境变量加载 + 解析配置
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
        Ok(Self {
            http_addr,
            health_addr,
            admin_grpc_endpoint,
            jwt_secret,
        })
    }

    /// 测试用 builder(避开 env,UT 友好)
    pub fn for_test(http: &str, health: &str, admin: &str) -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: http.parse()?,
            health_addr: health.parse()?,
            admin_grpc_endpoint: admin.to_string(),
            jwt_secret: "test-secret".to_string(),
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GmConfig>,
}

impl AppState {
    pub fn new(config: GmConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

/// 初始化 tracing(供 main 调,test 跳过以免污染日志)
pub fn init_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,gm-backend=debug")),
        )
        .init();
}

/// 主 router(8443,完整 APIGW)
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/gm/health/view", get(health_view))
        .route("/api/v1/gm/ban", post(ban_account))
        .route("/api/v1/gm/compensation", post(grant_compensation))
        .route("/api/v1/gm/maintenance", post(set_maintenance))
        .route("/api/v1/audit/logs", get(query_audit))
        .with_state(state)
}

/// health-only router(8081,k8s 探针专用)
pub fn build_health_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state)
}

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ok","service":"gm-backend"})))
}

pub async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ready","service":"gm-backend"})))
}

pub async fn health_view(State(s): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "service": "gm-backend",
            "admin_endpoint": s.config.admin_grpc_endpoint,
            "mode": "stub-ok",
        })),
    )
}

pub async fn ban_account() -> impl IntoResponse {
    (
        StatusCode::ACCEPTED,
        Json(json!({"status":"queued","op":"ban"})),
    )
}

pub async fn grant_compensation() -> impl IntoResponse {
    (
        StatusCode::ACCEPTED,
        Json(json!({"status":"queued","op":"compensation"})),
    )
}

pub async fn set_maintenance() -> impl IntoResponse {
    (
        StatusCode::ACCEPTED,
        Json(json!({"status":"queued","op":"maintenance"})),
    )
}

pub async fn query_audit() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({"items":[],"next":"stub"})),
    )
}
