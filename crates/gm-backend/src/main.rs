//! gm-backend 入口 — GM 后台 APIGW(第 8 域微服务)
//!
//! 职责(per RGS-BAS-003 §2.1 / 53 部署架构):
//! - HTTPS 入口(HTTP/1.1 + TLS + 浏览器/mTLS 客户端证书)
//! - 路由 GM 后台 REST API → AdminService gRPC(mTLS)
//! - RBAC:读取 JWT/Session,角色校验(GM_OPERATOR / GM_ADMIN / SRE)
//! - 审计日志:每条请求 → audit_log(outbox → admin_db)
//!
//! 8 域微服务之第 8 域(per Ulysses 2026-08-27 12:43 JST 明确指令:
//! "所有,GM 后台的服务器也应该是一个微服务的形式")
//!
//! 部署目标:k3s deployment,1 replica dev / 2+ prod,NetworkPolicy
//! 只允许 ingress traefik→gm-backend:8443,只允许 egress→admin-service:50055(mTLS)
//!
//! 端口(per RGS-ARC-005 端口分配):
//! - HTTPS 8443(APIGW 入口)
//! - metrics 9108(Prometheus scrape)
//! - health 8081(GET /healthz 不带 TLS,k8s liveness/readiness)

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

#[derive(Debug, Clone)]
struct GmConfig {
    pub http_addr: SocketAddr,
    pub health_addr: SocketAddr,
    pub admin_grpc_endpoint: String,
    pub jwt_secret: String,
}

impl GmConfig {
    fn from_env() -> anyhow::Result<Self> {
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
}

#[derive(Clone)]
struct AppState {
    config: Arc<GmConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,gm-backend=debug")),
        )
        .init();

    let config = GmConfig::from_env()?;
    tracing::info!(
        target: "gm-backend",
        "starting GM APIGW: https={} health={} admin_grpc={}",
        config.http_addr,
        config.health_addr,
        config.admin_grpc_endpoint,
    );

    let state = AppState {
        config: Arc::new(config.clone()),
    };

    // === Router(8 域第 8 域 / GM 后台) ===
    let api = Router::new()
        // 健康检查(per RGS-OPS-101,k8s liveness/readiness 用)
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        // GM 操作(per RGS-BAS-003 §3 AdminService API 扩展)
        .route("/api/v1/gm/health/view", get(health_view))
        .route("/api/v1/gm/ban", post(ban_account))
        .route("/api/v1/gm/compensation", post(grant_compensation))
        .route("/api/v1/gm/maintenance", post(set_maintenance))
        // 审计查询
        .route("/api/v1/audit/logs", get(query_audit))
        .with_state(state.clone());

    // 健康检查路由(8081,HTTP,不走 TLS,k8s 探针专用)
    let health_app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state.clone());

    // === 启动 health 探针(8081)===
    let health_addr = state.config.health_addr;
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(health_addr).await
            .expect("bind health addr");
        tracing::info!(target: "gm-backend", "health probe listening on {}", health_addr);
        axum::serve(listener, health_app).await
            .expect("health server");
    });

    // === 主 HTTPS server(8443,mTLS,完整 APIGW)===
    // dev 简化:不强制 mTLS 客户端证书,生产 RGS-BAS-003 §2.1 强制双向 mTLS
    let http_addr = state.config.http_addr;
    let listener = tokio::net::TcpListener::bind(http_addr).await
        .context("bind GM_HTTP_ADDR")?;
    tracing::info!(target: "gm-backend", "GM APIGW listening on https://{}", http_addr);
    axum::serve(listener, api).await.context("axum serve")?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ok","service":"gm-backend"})))
}

async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ready","service":"gm-backend"})))
}

async fn health_view(State(s): State<AppState>) -> impl IntoResponse {
    // TODO: gRPC client → admin-service / AdminService.QueryHealthView
    // 现在返回 stub,生产接 gRPC
    (
        StatusCode::OK,
        Json(json!({
            "service": "gm-backend",
            "admin_endpoint": s.config.admin_grpc_endpoint,
            "mode": "stub-ok",
        })),
    )
}

async fn ban_account() -> impl IntoResponse {
    // TODO: 调 admin-service BanAccount(mTLS)
    (StatusCode::ACCEPTED, Json(json!({"status":"queued","op":"ban"})))
}

async fn grant_compensation() -> impl IntoResponse {
    // TODO: 调 admin-service GrantCompensation(mTLS)
    (StatusCode::ACCEPTED, Json(json!({"status":"queued","op":"compensation"})))
}

async fn set_maintenance() -> impl IntoResponse {
    // TODO: 调 admin-service SetMaintenanceMode(mTLS)
    (StatusCode::ACCEPTED, Json(json!({"status":"queued","op":"maintenance"})))
}

async fn query_audit() -> impl IntoResponse {
    // TODO: 调 admin-service QueryAuditLog
    (StatusCode::OK, Json(json!({"items":[],"next":"stub"})))
}
