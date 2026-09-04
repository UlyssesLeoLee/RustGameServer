//! rgs-flash-mock main.rs — server bootstrap + 12 大类 routes 注册
//!
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §2.2 文件结构 + §2.3 数据流
//! 跟 rgs-batch-backend 单文件起步模式一致 (v0.2+ 拆 5+ 文件)

use actix_web::{web, App, HttpServer};
use rgs_flash_mock::{config::Config, gap_matrix::{initial_rpc_records, GapMatrix}};
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. tracing init (per shared-platform::json_logging 模式)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,rgs_flash_mock=debug")),
        )
        .init();

    // 2. config 加载 (env var, 8/27 11:06 JST 凭据走 env var 永不打印)
    let cfg = Config::from_env().unwrap_or_else(|e| {
        eprintln!("config 加载失败: {}", e);
        std::process::exit(1);
    });

    tracing::info!(
        target: "rgs-flash-mock",
        "starting at {}, service={}, v0.1 stub 模式 (12 大类 22 RPC)",
        cfg.bind_addr,
        cfg.service_name,
    );

    // 3. mTLS cert 验证 (per 8/27 ST 导出 SOP + L-CAND-006 兜底)
    if let Err(e) = cfg.verify_certs() {
        tracing::warn!(
            target: "rgs-flash-mock",
            "mTLS cert 验证失败 (per 8/27 ST 导出 SOP): {}. v0.1 stub 模式不阻塞启动, v0.2+ 接 gRPC client 时必填",
            e
        );
    }

    // 4. gap matrix 初始化 + 22 RPC 注册
    let gap_matrix = Arc::new(GapMatrix::new());
    for record in initial_rpc_records() {
        gap_matrix.register(record).await;
    }
    let rpc_count = gap_matrix.report().await.total_rpcs;
    tracing::info!(
        target: "rgs-flash-mock",
        "gap matrix initialized, {} RPCs registered (12 大类 PoC)",
        rpc_count
    );

    let gap_data = web::Data::new(gap_matrix);

    // 5. actix-web server bootstrap
    let bind_addr = cfg.bind_addr.clone();
    tracing::info!(
        target: "rgs-flash-mock",
        "binding HTTP/JSON server at {} (port 8791, 0.0.0.0, 跟 rgs-batch-backend 8790 sequential)",
        bind_addr
    );

    HttpServer::new(move || {
        App::new()
            .app_data(gap_data.clone())
            // 健康检查
            .route("/health", web::get().to(rgs_flash_mock::handlers::handle_health))
            .route("/ready", web::get().to(rgs_flash_mock::handlers::handle_ready))
            // gap matrix 报告
            .route("/coverage", web::get().to(rgs_flash_mock::handlers::handle_coverage))
            // 12 大类 RPC 路由 (POST /rpc/{category}/{rpc_name})
            .route("/rpc/{category}/{rpc_name}", web::post().to(rgs_flash_mock::handlers::handle_rpc))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
