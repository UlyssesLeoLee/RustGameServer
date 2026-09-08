//! battle-service 入口 (7 域战斗微服务 binary, per 路线图 §3 W5)
//!
//! ## W32 fix (per 改进路线图 §1 Phase 4 + worker30-3domain-pod-diag-report §2.3)
//!
//! 9/6 19:18 JST 旧 binary 是 W20 §1.2 Phase 1.5 placeholder: 启动 1 秒后
//! `std::process::exit(0)`, k8s "Completed" Exit Code 0 → CrashLoopBackOff
//! 73 次 (41h).
//!
//! W32 修法: 补 tonic gRPC server 真接 + tonic_health 健康检查
//! (per 5 域 / scene-service 范式, per W30 §7.2 选项 A 补 binary 简化版).
//!
//! 范围:
//! - tonic transport Server 绑 GRPC_ADDR (default 0.0.0.0:50058)
//! - 注册 tonic_health reporter → 12 service stub health check 都 OK
//! - mTLS 业务级 (per 5 域 ST 实践, RGS_TLS_DIR=/etc/rgs/certs)
//! - 12 service wire-up 留主会话 (per W5 §3 30 真实 + 220 stub 待 Phase 2 业务实装)
//! - 业务级 0 RPC 注册: proto-defined services 暂空, 仅 health endpoint,
//!   probe 改 tcpSocket 5 域对齐 (per W30 §4.3 方案 A)
//!
//! 已知缺口 (per 缺标比错标, AGENTS §1.1):
//! - 12 battle services 250 RPC 业务实装 留 Phase 2 W5 follow-up
//! - 镜像 COPY grpc_health_probe 二进制仍未做 (yaml probe 改 tcpSocket 规避)
//! - DB connection + sqlx 实装 留 Phase 2 W5 follow-up
//!
//! 不动: 5 域 / batch / scene-service / cluster_ops / 8 子系统 / 平台 / 工具
//!       (per W32 任务 brief "不动" §).

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use battle_service::service::{
    BattleEngineServiceImpl, BattleServiceImpl, BossServiceImpl, CrossServerServiceImpl,
    EndlessTowerServiceImpl, EscortServiceImpl, ExpeditionServiceImpl, GuildWarServiceImpl,
    HolidayActivityServiceImpl, HolyEquipServiceImpl, InstanceServiceImpl, PvPServiceImpl,
    RoomServiceImpl,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,battle-service=debug")),
        )
        .init();

    // GRPC_ADDR: yaml 配 0.0.0.0:50058, 默认同样
    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50058".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;

    tracing::info!(target: "battle-service", "starting 7-域 battle-service at {} (W32 fix: tonic server with health)", addr);

    // 实例化 12 service (W5 30 真实 RPC 业务 + 220 stub, 留 main wire-up 用)
    let state = Arc::new(BattleServiceImpl::new());
    let _battle_engine = BattleEngineServiceImpl::new(state.clone());
    let _pvp = PvPServiceImpl::new(state.clone());
    let _boss = BossServiceImpl::new(state.clone());
    let _room = RoomServiceImpl::new(state.clone());
    let _instance = InstanceServiceImpl::new(state.clone());
    let _endless = EndlessTowerServiceImpl::new(state.clone());
    let _escort = EscortServiceImpl::new(state.clone());
    let _holy = HolyEquipServiceImpl::new(state.clone());
    let _guild_war = GuildWarServiceImpl::new(state.clone());
    let _cross_server = CrossServerServiceImpl::new(state.clone());
    let _expedition = ExpeditionServiceImpl::new(state.clone());
    let _holiday = HolidayActivityServiceImpl::new(state.clone());

    tracing::info!(target: "battle-service", "12 services initialized (250 RPC, 30 real + 220 stub; W32 fix: not yet registered as tonic service)");

    // tonic health (per 5 域 / scene-service 范式, RGS-OPS-101)
    // 12 service 暂未 wire-up, 但 health reporter 用空 service 字符串即可 (default = "")
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    // mTLS 业务级 (per 5 域 ST 实践, RGS_TLS_DIR=/etc/rgs/certs)
    // 镜像内 cert mount 路径, ca-sbn.pem 是 3 NEW 域 mini-CA (per W20 §1.4)
    let mut server_builder = tonic::transport::Server::builder();
    if env::var("RGS_ALLOW_INSECURE_GRPC").is_ok() {
        tracing::warn!(
            target: "battle-service",
            "RGS_ALLOW_INSECURE_GRPC set — mTLS DISABLED, running INSECURE gRPC (dev/test only)"
        );
    } else {
        let tls_dir = env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let tls_config = shared_platform::tls::load_server_tls_config(
            &std::path::PathBuf::from(format!("{}/server.pem", tls_dir)),
            &std::path::PathBuf::from(format!("{}/server.key", tls_dir)),
            &std::path::PathBuf::from(format!("{}/ca-sbn.pem", tls_dir)),
        )
        .context(
            "mTLS config load failed (set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test)",
        )?;
        server_builder = server_builder
            .tls_config(tls_config)
            .context("tls_config")?;
        tracing::info!(target: "battle-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "battle-service", "binding gRPC server at {}", addr);
    server_builder
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
