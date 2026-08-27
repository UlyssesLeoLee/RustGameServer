//! gm-backend 入口(per RGS-BAS-003 §2.1 / 8 域微服务之第 8 域)
//!
//! 业务逻辑已抽到 lib.rs(便于 UT),main.rs 只做:
//! - tracing 初始化
//! - 配载
//! - axum serve 双端口(8443 主 + 8081 探针)

use anyhow::Context;
use gm_backend::{build_health_router, build_router, init_tracing, AppState, GmConfig};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = GmConfig::from_env()?;
    tracing::info!(
        target: "gm-backend",
        "starting GM APIGW: https={} health={} admin_grpc={}",
        config.http_addr,
        config.health_addr,
        config.admin_grpc_endpoint,
    );

    let state = AppState::new(config.clone());

    // health 探针(8081,HTTP,不走 TLS,k8s 探针专用)
    let health_addr: SocketAddr = state.config.health_addr;
    let health_app = build_health_router(state.clone());
    tokio::spawn(async move {
        let listener = TcpListener::bind(health_addr).await
            .expect("bind health addr");
        tracing::info!(target: "gm-backend", "health probe listening on {}", health_addr);
        axum::serve(listener, health_app).await
            .expect("health server");
    });

    // 主 HTTPS server(8443,生产模式 mTLS,当前 dev 跳过)
    let http_addr = state.config.http_addr;
    let listener = TcpListener::bind(http_addr).await
        .context("bind GM_HTTP_ADDR")?;
    let api = build_router(state);
    tracing::info!(target: "gm-backend", "GM APIGW listening on https://{}", http_addr);
    axum::serve(listener, api).await.context("axum serve")?;
    Ok(())
}
