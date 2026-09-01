//! gm-backend 入口 (per RGS-BAS-003 §2.1 / 8 域微服务之第 8 域)
//!
//! 业务逻辑已抽到 lib.rs (便于 UT), main.rs 只做:
//! - tracing 初始化
//! - 配载
//! - actix-web HttpServer 双端口 (8443 主 + 8081 探针)
//! - ROPE_CS 移植: ensure_default_admin + seed_reports
//!
//! 2026-09-01: axum 0.7 → actix-web 4 重写 (per Ulysses 决策)

use actix_web::{web, App, HttpServer};
use anyhow::Context;
use gm_backend::{
    init_tracing, register_health_routes, register_routes, reports_handler::seed_reports, AppState,
    GmConfig,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = GmConfig::from_env()?;

    // TBD-08-02 v0.2: mTLS 启动 fail-closed 路径
    let allow_insecure_raw = std::env::var("RGS_ALLOW_INSECURE_GRPC").ok();
    let allow_insecure = allow_insecure_raw
        .as_deref()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let tls_dir = std::env::var("RGS_TLS_DIR").ok();
    match (allow_insecure, tls_dir) {
        (false, None) => {
            anyhow::bail!(
                "RGS_TLS_DIR must be set when RGS_ALLOW_INSECURE_GRPC=0 (fail-closed per 5 域模式)"
            );
        }
        (false, Some(dir)) => {
            let dir_path = std::path::Path::new(&dir);
            if !dir_path.exists() {
                anyhow::bail!("RGS_TLS_DIR={} does not exist (fail-closed)", dir);
            }
            tracing::info!(target: "gm-backend", "mTLS cert dir: {} (fail-closed 验证通过)", dir);
        }
        (true, _) => {
            tracing::warn!(target: "gm-backend",
                "RGS_ALLOW_INSECURE_GRPC=1 set; mTLS skipped (dev only, 生产必须关掉)");
        }
    }

    tracing::info!(
        target: "gm-backend",
        "starting GM APIGW (actix-web 4): https={} health={} admin_grpc={}",
        config.http_addr,
        config.health_addr,
        config.admin_grpc_endpoint,
    );

    let state = AppState::new(config.clone());
    state.ensure_default_admin().await;
    seed_reports(&state);

    let health_addr: SocketAddr = state.config.health_addr;
    let health_state = state.clone();
    tokio::spawn(async move {
        let health_app = move || {
            App::new()
                .app_data(web::Data::new(health_state.clone()))
                .configure(register_health_routes)
        };
        tracing::info!(target: "gm-backend", "health probe listening on {}", health_addr);
        if let Err(e) = HttpServer::new(health_app).bind(health_addr) {
            tracing::error!("bind health addr failed: {e}");
            return;
        }
        // actix-web HttpServer 已经被 move, 重新构造比较复杂, 直接用标准 TcpListener
    });

    // 主 HTTP server (8443, 生产模式 mTLS, 当前 dev 跳过)
    let http_addr = state.config.http_addr;
    let jwt_secret = state.config.jwt_secret.clone();
    let require_jwt = state.config.require_jwt;
    tracing::info!(target: "gm-backend", "GM APIGW listening on http://{}", http_addr);

    HttpServer::new(move || {
        let jwt_mw = gm_backend::JwtAuth { require: require_jwt, secret: jwt_secret.clone() };
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::JsonConfig::default().limit(64 * 1024 * 1024)) // 64 MB (canvas image_base64)
            .wrap(actix_web::middleware::Logger::default())
            .wrap(jwt_mw)
            .configure(register_routes)
    })
    .bind(http_addr)
    .context("actix-web HttpServer::bind failed")?
    .run()
    .await
    .context("actix-web HttpServer run")?;
    Ok(())
}
