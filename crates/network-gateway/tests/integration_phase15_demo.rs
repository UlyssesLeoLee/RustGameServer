//! W7 端到端 8 域 demo 路由 roundtrip 集成测试
//!
//! ## 范围
//! 验证 8 域 (player/economy/scene/battle/batch/admin/cluster_ops + 5 域) demo 路由
//! 全部经真实 TCP socket 命中, 返回 rcode=0 + payload 含 target_service.method.
//!
//! ## 不动
//! - 7 域 gRPC client stub (不真调, 仅路由决策演示, per task brief "卡住的应对")
//! - 5 域 / batch / 平台 / 工具 crate 不动 (per 8/21 JST 5 域独立 Lead → 7 域)
//!
//! ## 已知缺口 (per 9/4 改进路线图 Phase 1 完整 8 SRE·d, W7 仅 stub)
//! - NIF 桥接 rustler 0.36 + BEAM (per ADR-006 Option A, Phase 1.5)
//! - 端到端 7 域业务调用 (per 9/4 Phase 2, 25-40 SRE·d)
//! - web_conn.erl / zone.erl 真实加载 (per 9/4 R2, 1-2 周)
//! - 完整 cookie 鉴权 (per task brief "卡住的应对", Phase 1.5 stub 推进)
//! - 1351 条全路由 codegen (per task brief "100/1351 核心", Phase 1.5 补完)

use std::sync::Arc;
use std::time::Duration;

use network_gateway::router::{RouteTable, PHASE1_5_DEMO_ROUTES};
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// 启动一个临时 TCP listener (127.0.0.1:0 = OS 分配端口)
async fn spawn_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let routes = Arc::new(RouteTable::with_phase15_demo());
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
async fn phase15_eight_domains_route_roundtrip() {
    // 启动 1 个临时 server, 跑 8 域 demo 全命中
    let (addr, _handle) = spawn_test_server().await;

    for (code, name, svc, method, _addr) in PHASE1_5_DEMO_ROUTES {
        let payload = format!("w7-demo-{}-{}", name, code);
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
async fn phase15_eight_domains_have_seven_addrs() {
    // 8 域 (7 域 + cluster_ops) → 7 唯一 gRPC 地址 (port 50051-50057)
    // scene 2 路由 + battle 2 路由 → 共享同地址
    // 9 demo 路由 → 7 唯一地址
    let addrs: std::collections::HashSet<_> = PHASE1_5_DEMO_ROUTES
        .iter()
        .map(|(_, _, _, _, a)| *a)
        .collect();
    assert_eq!(
        addrs.len(),
        7,
        "8 域 → 7 唯一 gRPC 地址 (scene/battle 各 2 路由同址)"
    );
}

#[tokio::test]
async fn phase15_login_role_battle_endtoend_stub() {
    // 端到端 stub: 客户端 → RGS 协议网关 → 路由决策
    // 真实 Phase 3 联调: 调 player-service.CreateCharacter / scene-service.EnterScene /
    //                    battle-service.StartPve
    // 当前: 仅验证路由层 OK, Phase 1.5 接 NIF + 7 域 gRPC client
    let (addr, _handle) = spawn_test_server().await;

    // 1. 创建角色 (10101 → player.v1.PlayerService.CreateCharacter)
    let (r1, _) = send_recv_frame(&addr, 10101, b"player-1").await;
    assert_eq!(r1, 0, "step1 创建角色路由命中");

    // 2. 进入场景 (10201 → scene.v1.SceneService.EnterScene)
    let (r2, _) = send_recv_frame(&addr, 10201, b"scene-1").await;
    assert_eq!(r2, 0, "step2 进入场景路由命中");

    // 3. 战斗开始 (20001 → battle.v1.BattleService.StartPve)
    let (r3, _) = send_recv_frame(&addr, 20001, b"battle-1").await;
    assert_eq!(r3, 0, "step3 战斗开始路由命中");

    // 4. 战斗结束 (20002 → battle.v1.BattleService.EndPve)
    let (r4, _) = send_recv_frame(&addr, 20002, b"battle-1-end").await;
    assert_eq!(r4, 0, "step4 战斗结束路由命中");

    // 5. 加货币 (20101 → economy.v1.EconomyService.AddCurrency, 战斗结算)
    let (r5, _) = send_recv_frame(&addr, 20101, b"reward-100").await;
    assert_eq!(r5, 0, "step5 加货币路由命中 (战斗结算)");
}
