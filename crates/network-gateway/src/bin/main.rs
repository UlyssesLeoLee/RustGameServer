//! network-gateway 二进制入口 (per W6 Phase 1 协议网关 骨架)
//!
//! ## 工作流
//! 1. 加载默认路由表 (1 条 demo: 10101 → player-service.CreateCharacter)
//! 2. 启动 admin gRPC server (0.0.0.0:50090, per task 默认 8790 留待 Phase 1.5 调整)
//! 3. 启动 TCP listener 127.0.0.1:7001, 接 4B code + 4B length + payload
//! 4. tokio::join! 并发跑两个 server, 任一退出即整体退出
//!
//! ## 已知缺口 (per 9/4 改进路线图 Phase 1 完整 8 SRE·d, 本骨架仅 0.5)
//! - mTLS 业务级 (per 5 域 ST 实践 commit `401ac5c`, Phase 4)
//! - Prometheus metrics (per Phase 4)
//! - 实际 player-service gRPC client 调用 (Phase 1.5 + Phase 3 联调)

use std::sync::Arc;

use network_gateway::router::RouteTable;
use network_gateway::server::GatewayAdminService;
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use network_gateway::{web_conn, zone};
use tonic::transport::Server;
use tracing::{info, warn};

/// admin gRPC 端口 (per task brief, Phase 1 骨架默认值)
pub const ADMIN_GRPC_ADDR: &str = "127.0.0.1:50090";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing 初始化 (per RGS 5 域规范)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // W7 扩展: 用 8 域 demo 路由表 (per 9/4 改进路线图 Phase 2 7 域)
    let routes = Arc::new(RouteTable::with_phase15_demo());
    let stats = Arc::new(GatewayStats::new());
    let admin = GatewayAdminService::new(Arc::clone(&routes), Arc::clone(&stats));

    let admin_addr: std::net::SocketAddr = ADMIN_GRPC_ADDR.parse()?;
    let tcp_addr = tcp::DEFAULT_TCP_ADDR.to_string();

    info!(
        admin_grpc = %admin_addr,
        tcp = %tcp_addr,
        routes = routes.len(),
        "network-gateway starting (W7 Phase 1.5 stub)"
    );

    // 并发跑 admin gRPC + TCP 监听 + web_conn stub + zone stub
    // Phase 1.5 真实实装: 启 BEAM 跑 web_conn.erl + zone 启动
    let admin_task = tokio::spawn(async move {
        let svc = admin.into_server();
        if let Err(e) = Server::builder().add_service(svc).serve(admin_addr).await {
            warn!(err = %e, "admin gRPC exited");
        }
    });

    let tcp_task = tokio::spawn({
        let routes = Arc::clone(&routes);
        let stats = Arc::clone(&stats);
        async move {
            if let Err(e) = tcp::serve(&tcp_addr, routes, stats).await {
                warn!(err = %e, "TCP listener exited");
            }
        }
    });

    // W7 扩展: web_conn (8000) + zone (per 9/4 MD §3 拓扑) stub 启动
    let web_conn_task = tokio::spawn(async move {
        let cfg = web_conn::WebConnConfig::default_local();
        if let Err(e) = web_conn::start(cfg).await {
            warn!(err = %e, "web_conn stub exited");
        }
    });

    let zone_task = tokio::spawn(async move {
        // Phase 1.5 stub: 仅 center 节点起 (port 8000), zone 节点 Phase 4 k3s StatefulSet 推进
        let cfg = zone::ZoneConfig::default_center("sszg_center_6");
        if let Err(e) = zone::start(cfg).await {
            warn!(err = %e, "zone stub exited");
        }
    });

    // 任一退出即整体退出 (per main.rs 退出语义)
    tokio::select! {
        r = admin_task => { warn!("admin task finished: {:?}", r); }
        r = tcp_task => { warn!("tcp task finished: {:?}", r); }
        r = web_conn_task => { warn!("web_conn task finished: {:?}", r); }
        r = zone_task => { warn!("zone task finished: {:?}", r); }
    }
    Ok(())
}
