//! network-gateway 二进制入口 (per W6 Phase 1 协议网关 骨架)
//!
//! ## W32 fix (per 改进路线图 §1 Phase 4 + worker30-3domain-pod-diag-report §3.3)
//!
//! 9/6 19:19 JST 旧 binary 是 W7 Phase 1.5 stub: `tokio::select!` 4 task
//! 选最先 return, web_conn/zone stub 0ms 返 Ok(()) → main 立刻 exit 0 →
//! k8s "Completed" Exit Code 0 → CrashLoopBackOff 74 次 (41h).
//!
//! W32 修法:
//! 1. `tokio::select!` 改 `tokio::join!` — main 等 admin+TCP 两个长跑 task,
//!    web_conn/zone stub 立即返 Ok 不影响 (join! 等全部完成, 但 admin+TCP 永不返)
//! 2. ADMIN_GRPC_ADDR 改 0.0.0.0:50090 (pod 内部 127.0.0.1 也能用, 但
//!    接受外部 svc 流量 + 跟 yaml env RGS_NETWORK_GATEWAY_ADMIN_GRPC_ADDR 对齐)
//! 3. tcp_addr 改 0.0.0.0:7001 (同上对齐 RGS_NETWORK_GATEWAY_TCP_ADDR env)
//! 4. 加 tracing 阶段 marker (W32 fix: 改 select! → join!, 0ms exit 修)
//!
//! 不动: 5 域 / batch / battle / scene / cluster_ops / 8 子系统 / 平台 / 工具
//!       (per W32 任务 brief "不动" §).
//!
//! 已知缺口 (per 缺标比错标, AGENTS §1.1):
//! - 实际 player-service gRPC client 调用 (Phase 1.5 + Phase 3 联调, W7)
//! - mTLS 业务级 (per 5 域 ST 实践, Phase 4)
//! - EPMD 4369 + dist_proto + 1351 codegen + cookie 鉴权 (Phase 1 完整实装)
//! - web_conn 真实 HTTP 监听 (当前 stub 立即 Ok, Phase 1.5)
//! - zone 真实 BEAM 启动 (当前 stub 立即 Ok, Phase 1.5 + ADR-006 Option A rustler)

use std::sync::Arc;

use network_gateway::router::RouteTable;
use network_gateway::server::GatewayAdminService;
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use network_gateway::{web_conn, zone};
use tonic::transport::Server;
use tracing::{info, warn};

/// admin gRPC 监听地址 (W32 fix: 0.0.0.0 让 pod 内部 + 外部 svc 都能连)
/// 跟 yaml env `RGS_NETWORK_GATEWAY_ADMIN_GRPC_ADDR=0.0.0.0:50090` 对齐
pub const ADMIN_GRPC_ADDR: &str = "0.0.0.0:50090";

/// TCP 二进制监听地址 (W32 fix: 同上 0.0.0.0)
/// 跟 yaml env `RGS_NETWORK_GATEWAY_TCP_ADDR=0.0.0.0:7001` 对齐
pub const TCP_BINARY_ADDR: &str = "0.0.0.0:7001";

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

    // W32 fix: 优先读 env var (per yaml 配 RGS_NETWORK_GATEWAY_*_ADDR),
    // fallback 编译时常量 0.0.0.0:port
    let admin_addr: std::net::SocketAddr = std::env::var("RGS_NETWORK_GATEWAY_ADMIN_GRPC_ADDR")
        .unwrap_or_else(|_| ADMIN_GRPC_ADDR.to_string())
        .parse()
        .unwrap_or_else(|_| ADMIN_GRPC_ADDR.parse().expect("valid ADMIN_GRPC_ADDR"));
    let tcp_addr = std::env::var("RGS_NETWORK_GATEWAY_TCP_ADDR")
        .unwrap_or_else(|_| TCP_BINARY_ADDR.to_string());

    info!(
        admin_grpc = %admin_addr,
        tcp = %tcp_addr,
        routes = routes.len(),
        "network-gateway starting (W32 fix: tokio::join! + 0.0.0.0)"
    );

    // 并发跑 admin gRPC + TCP 监听 + web_conn stub + zone stub
    // W32 fix: 用 tokio::join! 替代 tokio::select!, main 等 admin+TCP 两个
    // long-running task. web_conn/zone stub 立即返 Ok(()) 不影响 (join! 仍等全部).
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
    // W32 fix: 这些 stub 立即返 Ok(()) 是 Phase 1.5 占位预期行为, join! 容忍
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

    // W32 fix: 用 tokio::join! 替代 tokio::select!
    // join! 等全部 task 完成后返 Result tuple; admin+TCP 永不返, 所以 main 永不 exit
    // (除非 admin 或 TCP error 退出, 那时整个 binary 退出, k8s restart 也合理)
    let (admin_r, tcp_r, web_conn_r, zone_r) =
        tokio::join!(admin_task, tcp_task, web_conn_task, zone_task);
    warn!(
        admin = ?admin_r,
        tcp = ?tcp_r,
        web_conn = ?web_conn_r,
        zone = ?zone_r,
        "network-gateway all tasks completed (rare, only on error)"
    );
    Ok(())
}
