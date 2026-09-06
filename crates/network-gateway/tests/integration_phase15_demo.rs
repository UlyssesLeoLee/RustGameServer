//! W7 端到端 8 域 demo 路由 roundtrip 集成测试
//!
//! ## 范围
//! 验证 W14 codegen 6 demo 路由 (player/scene/battle/economy 4 域) 全部经真实 TCP socket
//! 命中, 返回 rcode=0 + payload 含 target_service.method.
//!
//! ## W14 调整
//! W7 PHASE1_5_DEMO_ROUTES 9 条中有 5 条 (20101/10202/30101/40101/50101) 在
//! 9/4 API 清单 TSV 中不存在 (TSV 来自真实 Erlang 源, 9 域合成 code 不可用).
//! W14 demo 兼容缩为 6 条 (10101/10201/20001/20002/11000/25000), 集成测试仍可 roundtrip
//! 覆盖 4 真实域 (player/scene/battle/economy).
//!
//! ## 不动
//! - 7 域 gRPC client stub (不真调, 仅路由决策演示, per task brief "卡住的应对")
//! - 5 域 / batch / 平台 / 工具 crate 不动 (per 8/21 JST 5 域独立 Lead → 7 域)
//!
//! ## 已知缺口 (per 9/4 改进路线图 Phase 1 完整 8 SRE·d, W7 仅 stub)
//! - NIF 桥接 rustler 0.36 + BEAM (per ADR-006 Option A, Phase 1.5)
//! - 端到端 4 域 (player/scene/battle/economy) 业务调用 (per 9/4 Phase 2, 25-40 SRE·d)
//! - web_conn.erl / zone.erl 真实加载 (per 9/4 R2, 1-2 周)
//! - 完整 cookie 鉴权 (per task brief "卡住的应对", Phase 1.5 stub 推进)
//! - 1345 条非 demo 路由走 Method_<code> 占位 (Phase 2 接 4 域真实 .proto)

use std::sync::Arc;
use std::time::Duration;

use network_gateway::router::RouteTable;
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// 6 demo 路由对应 code (per W14 codegen 覆写, TSV 真实存在)
const DEMO_CODES: &[u32] = &[
    10101, // player.v1.PlayerService.CreateCharacter
    10201, // scene.v1.SceneService.EnterScene
    20001, // battle.v1.BattleService.BattlePrepare
    20002, // battle.v1.BattleService.RoundStart
    11000, // player.v1.PlayerService.GetPartnerData
    25000, // economy.v1.EconomyService.PushBaseInfo
];

/// 6 demo 路由对应 (code, name, svc, method) 元组
const DEMO_ROUTES: &[(u32, &str, &str, &str)] = &[
    (10101, "create_character", "player.v1.PlayerService", "CreateCharacter"),
    (10201, "enter_scene", "scene.v1.SceneService", "EnterScene"),
    (20001, "battle_prepare", "battle.v1.BattleService", "BattlePrepare"),
    (20002, "round_start", "battle.v1.BattleService", "RoundStart"),
    (11000, "get_partner_data", "player.v1.PlayerService", "GetPartnerData"),
    (
        25000,
        "push_base_info",
        "economy.v1.EconomyService",
        "PushBaseInfo",
    ),
];

/// 启动一个临时 TCP listener (127.0.0.1:0 = OS 分配端口)
async fn spawn_test_server() -> (String, tokio::task::JoinHandle<()>) {
    // W14: 走 RouteTable::new() (1351 全加载) 验证 9 demo 路由
    let routes = Arc::new(RouteTable::new());
    let stats = Arc::new(GatewayStats::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ok");
    let addr = listener.local_addr().expect("local_addr ok");
    drop(listener); // 释放端口给 serve
    let addr_s = addr.to_string();
    let handle = tokio::spawn(async move {
        let _ = tokio::time::timeout(
            Duration::from_secs(5),
            tcp::serve(&addr_s, routes, stats),
        )
        .await;
    });
    // 等 server 起来
    tokio::time::sleep(Duration::from_millis(50)).await;
    (addr.to_string(), handle)
}

/// TCP 客户端发一帧, 收一帧
async fn send_recv_frame(addr: &str, code: u32, payload: &[u8]) -> (u32, Vec<u8>) {
    let mut stream = TcpStream::connect(addr).await.expect("connect ok");
    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.extend_from_slice(&code.to_be_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    stream.write_all(&frame).await.expect("write ok");
    let mut header = [0u8; 8];
    stream.read_exact(&mut header).await.expect("read header");
    let rcode = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let mut body = vec![0u8; length];
    stream.read_exact(&mut body).await.expect("read body");
    stream.shutdown().await.ok();
    (rcode, body)
}

#[tokio::test]
async fn phase15_six_routes_route_roundtrip() {
    // 启动 1 个临时 server, 跑 6 demo 路由全命中
    let (addr, _handle) = spawn_test_server().await;

    for (code, name, svc, method) in DEMO_ROUTES {
        let payload = format!("w14-demo-{}-{}", name, code);
        let (rcode, body) = send_recv_frame(&addr, *code, payload.as_bytes()).await;
        assert_eq!(rcode, 0, "code {} ({}) 应该命中路由", code, name);
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains(svc) && body_str.contains(method),
            "code {} 响应含 {}.{} 实际 = {}",
            code,
            svc,
            method,
            body_str
        );
    }
}

#[tokio::test]
async fn phase15_route_miss_returns_404() {
    // 验证未注册 code 返回 404
    let (addr, _handle) = spawn_test_server().await;
    let (rcode, body) = send_recv_frame(&addr, 99999, b"unknown").await;
    assert_eq!(rcode, 404);
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("99999") || body_str.contains("unknown"));
}

#[tokio::test]
async fn phase15_six_routes_have_four_addrs() {
    // 6 demo 路由 → 4 唯一 gRPC 地址 (player/scene/battle/economy)
    // player 2 路由 (10101/11000) → 50051
    // scene 1 (10201) → 50053
    // battle 2 (20001/20002) → 50054
    // economy 1 (25000) → 50052
    let rt = RouteTable::new();
    let mut unique_addrs = std::collections::HashSet::new();
    for code in DEMO_CODES {
        if let Some(entry) = rt.get(*code) {
            unique_addrs.insert(entry.target_addr.clone());
        }
    }
    assert_eq!(
        unique_addrs.len(),
        4,
        "6 demo 跨 4 真实域 → 4 唯一 gRPC 地址"
    );
}

#[tokio::test]
async fn phase15_login_role_battle_endtoend_stub() {
    // 端到端 stub: 客户端 → RGS 协议网关 → 路由决策
    // 真实 Phase 3 联调: 调 player-service.CreateCharacter / scene-service.EnterScene /
    //                    battle-service.BattlePrepare
    // 当前: 仅验证路由层 OK, Phase 1.5 接 NIF + 4 域 gRPC client
    let (addr, _handle) = spawn_test_server().await;

    // 1. 创建角色 (10101 → player.v1.PlayerService.CreateCharacter)
    let (r1, _) = send_recv_frame(&addr, 10101, b"player-1").await;
    assert_eq!(r1, 0, "step1 创建角色路由命中");

    // 2. 进入场景 (10201 → scene.v1.SceneService.EnterScene)
    let (r2, _) = send_recv_frame(&addr, 10201, b"scene-1").await;
    assert_eq!(r2, 0, "step2 进入场景路由命中");

    // 3. 战斗准备 (20001 → battle.v1.BattleService.BattlePrepare)
    let (r3, _) = send_recv_frame(&addr, 20001, b"battle-1").await;
    assert_eq!(r3, 0, "step3 战斗准备路由命中");

    // 4. 回合开始 (20002 → battle.v1.BattleService.RoundStart)
    let (r4, _) = send_recv_frame(&addr, 20002, b"battle-1-end").await;
    assert_eq!(r4, 0, "step4 回合开始路由命中");

    // 5. 推送基础信息 (25000 → economy.v1.EconomyService.PushBaseInfo)
    let (r5, _) = send_recv_frame(&addr, 25000, b"reward-100").await;
    assert_eq!(r5, 0, "step5 推送基础信息路由命中 (战斗结算)");
}

#[tokio::test]
async fn phase15_full_1351_loaded_in_table() {
    // W14 codegen: RouteTable::new() 加载 1351 条 (per 9/4 TSV)
    let rt = RouteTable::new();
    assert_eq!(rt.len(), 1351, "W14 1351 全加载");
    // 6 demo code 全部命中
    for code in DEMO_CODES {
        assert!(rt.get(*code).is_some(), "demo code {} must hit", code);
    }
}
