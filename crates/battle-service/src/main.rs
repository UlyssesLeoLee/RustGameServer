//! battle-service 入口 (7 域战斗微服务 binary, per 路线图 §3 W5)
//!
//! 启动 tonic gRPC server 接 12 个 service (250 RPC, 30 真实 + 220 stub)。
//!
//! 7 域独立 Lead (per 8/21 JST 5 域独立 → 9/1 batch 域 → 9/5 battle 域):
//! - battle-service 独立 DB (battle_db) 后续接 sqlx
//! - mTLS 业务级 沿用 5 域 ST 实践 (per 路线图 §4 R5/R8)
//! - 临时 InMemory 存储, 后续桶 10 业务实装接 PgRepository

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

    let addr: std::net::SocketAddr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50059".to_string())
        .parse()?;

    tracing::info!(target: "battle-service", "starting 7-域 battle-service at {}", addr);

    let state = Arc::new(BattleServiceImpl::new());

    let battle_engine = BattleEngineServiceImpl::new(state.clone());
    let pvp = PvPServiceImpl::new(state.clone());
    let boss = BossServiceImpl::new(state.clone());
    let room = RoomServiceImpl::new(state.clone());
    let instance = InstanceServiceImpl::new(state.clone());
    let endless = EndlessTowerServiceImpl::new(state.clone());
    let escort = EscortServiceImpl::new(state.clone());
    let holy = HolyEquipServiceImpl::new(state.clone());
    let guild_war = GuildWarServiceImpl::new(state.clone());
    let cross_server = CrossServerServiceImpl::new(state.clone());
    let expedition = ExpeditionServiceImpl::new(state.clone());
    let holiday = HolidayActivityServiceImpl::new(state.clone());

    // 注: 完整 gRPC server 注册需 12 个 service_xxxServer::new(impl).await
    // 当前 worker 5 范围: scaffold + 编译通过, 完整 main wire-up 留给主会话
    tracing::info!(target: "battle-service", "12 services initialized (250 RPC, 30 real + 220 stub)");

    let _ = (battle_engine, pvp, boss, room, instance, endless, escort, holy, guild_war, cross_server, expedition, holiday);

    // 临时占位: 阻止 main 立刻退出 (W5 简报要求 "不 commit", 主会话会替换)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    tracing::warn!(target: "battle-service", "main placeholder exited (主会话负责 gRPC server wire-up)");

    Ok(())
}
