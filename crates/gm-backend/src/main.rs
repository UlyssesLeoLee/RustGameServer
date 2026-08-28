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

    // TBD-08-02 v0.2: mTLS 启动 fail-closed 路径(per 5 域 RGS_ALLOW_INSECURE_GRPC=0 模式)
    // - 生产模式 (RGS_ALLOW_INSECURE_GRPC=0): RGS_TLS_DIR 必须存在 + 含 server cert/key
    // - dev 模式 (RGS_ALLOW_INSECURE_GRPC=1): 跳过 mTLS 验证,log warning
    let allow_insecure_raw = std::env::var("RGS_ALLOW_INSECURE_GRPC").ok();
    // bool parse 只接受 "true"/"false",5 域 0/1 兼容
    let allow_insecure = allow_insecure_raw
        .as_deref()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let tls_dir = std::env::var("RGS_TLS_DIR").ok();
    match (allow_insecure, tls_dir) {
        (false, None) => {
            anyhow::bail!(
                "RGS_TLS_DIR must be set when RGS_ALLOW_INSECURE_GRPC=0 (fail-closed per 5 域模式) [raw={:?}]",
                allow_insecure_raw
            );
        }
        (false, Some(dir)) => {
            let dir_path = std::path::Path::new(&dir);
            if !dir_path.exists() {
                anyhow::bail!("RGS_TLS_DIR={} does not exist (fail-closed)", dir);
            }
            tracing::info!(target: "gm-backend", "mTLS cert dir: {} (fail-closed 验证通过)", dir);
            // 实际 mTLS 集成留给 v0.3 (per TBD-08-02)
        }
        (true, _) => {
            tracing::warn!(target: "gm-backend",
                "RGS_ALLOW_INSECURE_GRPC=1 set; mTLS skipped (dev only, 生产必须关掉)");
        }
    }

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
