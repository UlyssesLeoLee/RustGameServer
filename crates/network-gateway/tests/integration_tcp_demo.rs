//! 集成测试: TCP 接 4 字节 [code][length] + payload → 路由到 player-service.CreateCharacter
//!
//! ## 范围 (per W6 task "1 真实演示")
//! - 起 1 个 0 端口 TCP listener (OS 分配)
//! - 客户端连, 发 [code=10101][length=5][payload="hello"]
//! - 服务端 dispatch 到路由表, 返回 [rcode=0][length=37][payload="player.v1.PlayerService#CreateCharacter"]
//! - 验证 rcode=0 + payload 内容 + stats 计数
//!
//! ## 已知缺口
//! - 实际 gRPC client 调通需 player-service 启动 (Phase 1.5 + Phase 3 联调)
//! - 这里只验证 routing decision, 不验证端到端 gRPC (per task brief "即使调不通也算")

use std::sync::Arc;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use network_gateway::router::RouteTable;
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::test]
async fn tcp_demo_route_10101_to_player_create_character() {
    let routes = Arc::new(RouteTable::new());
    let stats = Arc::new(GatewayStats::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener); // 释放; serve() 会重新 bind 同 port? 否, 我们用 serve() 替 listener
                    // 实际: serve 内部 TcpListener::bind, 端口冲突风险 → 让 serve 改用 0 端口
                    // 简化: 这里直接调 dispatch + 测 TCP 写
    let _ = addr; // suppress unused

    // 验证 dispatch (Phase 1 骨架: routing decision)
    let frame = network_gateway::codec::Frame {
        code: 10101,
        payload: Bytes::from_static(b"hello"),
    };
    let resp = tcp::dispatch(frame, &routes, &stats);
    assert!(resp.len() > 8);
    let rcode = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
    assert_eq!(rcode, 0, "路由 10101 应成功 (默认路由表 demo)");
    let length = u32::from_be_bytes([resp[4], resp[5], resp[6], resp[7]]) as usize;
    let payload = &resp[8..8 + length];
    let payload_str = std::str::from_utf8(payload).unwrap();
    assert_eq!(payload_str, "player.v1.PlayerService#CreateCharacter");
}

#[tokio::test]
async fn tcp_demo_route_miss_returns_404() {
    let routes = RouteTable::new();
    let stats = GatewayStats::new();
    let frame = network_gateway::codec::Frame {
        code: 1351, // 远超默认表的协议码
        payload: Bytes::from_static(b"x"),
    };
    let resp = tcp::dispatch(frame, &routes, &stats);
    let rcode = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
    assert_eq!(rcode, 404, "未注册 code 应返回 404");
    let snap = stats.snapshot();
    assert_eq!(snap.total_route_miss, 1);
}

#[tokio::test]
async fn tcp_serve_client_roundtrip() {
    // 真实 TCP 端到端: 启动 serve, 客户端发帧, 验证响应
    let routes = Arc::new(RouteTable::new());
    let stats = Arc::new(GatewayStats::new());

    // 用 0 端口拿一个空闲端口
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    let addr = format!("127.0.0.1:{}", port);

    // 启动 serve
    let routes_for_serve = Arc::clone(&routes);
    let stats_for_serve = Arc::clone(&stats);
    let serve_addr = addr.clone();
    let serve_task = tokio::spawn(async move {
        // serve 用 60ms timeout 退出 (本测试只需要 roundtrip)
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            tcp::serve(&serve_addr, routes_for_serve, stats_for_serve),
        )
        .await;
    });

    // 给 serve 50ms 启动
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 客户端发 [code=10101][length=5][payload="hello"]
    let mut client = TcpStream::connect(&addr).await.expect("connect ok");
    let mut out = BytesMut::with_capacity(13);
    out.extend_from_slice(&10101u32.to_be_bytes());
    out.extend_from_slice(&5u32.to_be_bytes());
    out.extend_from_slice(b"hello");
    client.write_all(&out).await.expect("write ok");
    client.flush().await.ok();

    // 读响应
    let mut buf = [0u8; 128];
    let n = tokio::time::timeout(Duration::from_secs(2), client.read(&mut buf))
        .await
        .expect("read not timeout")
        .expect("read ok");
    assert!(n >= 8, "至少 8 字节响应头, got {}", n);
    let rcode = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    assert_eq!(rcode, 0, "rcode 应为 0 (route hit)");
    let length = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    let payload = &buf[8..8 + length];
    let payload_str = std::str::from_utf8(payload).unwrap();
    assert_eq!(payload_str, "player.v1.PlayerService#CreateCharacter");

    // 关闭连接
    drop(client);

    // 给服务端一点时间处理 close
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 验证 stats
    let snap = stats.snapshot();
    assert_eq!(snap.total_received, 1, "received 计数 = 1");
    assert_eq!(snap.total_forwarded, 1, "forwarded 计数 = 1");
    assert_eq!(snap.total_route_miss, 0);

    // 取消 serve
    serve_task.abort();
}

#[tokio::test]
async fn tcp_serve_route_miss_increments_stat() {
    let routes = Arc::new(RouteTable::new());
    let stats = Arc::new(GatewayStats::new());

    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    let addr = format!("127.0.0.1:{}", port);

    let routes_for_serve = Arc::clone(&routes);
    let stats_for_serve = Arc::clone(&stats);
    let serve_addr = addr.clone();
    let serve_task = tokio::spawn(async move {
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            tcp::serve(&serve_addr, routes_for_serve, stats_for_serve),
        )
        .await;
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = TcpStream::connect(&addr).await.expect("connect ok");
    let mut out = BytesMut::with_capacity(12);
    out.extend_from_slice(&99999u32.to_be_bytes()); // 未注册
    out.extend_from_slice(&0u32.to_be_bytes());
    client.write_all(&out).await.expect("write ok");
    client.flush().await.ok();

    let mut buf = [0u8; 128];
    let _n = tokio::time::timeout(Duration::from_secs(2), client.read(&mut buf))
        .await
        .expect("read not timeout")
        .expect("read ok");
    let rcode = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    assert_eq!(rcode, 404, "未注册 code 应返回 404");

    drop(client);
    tokio::time::sleep(Duration::from_millis(50)).await;

    let snap = stats.snapshot();
    assert_eq!(snap.total_route_miss, 1);

    serve_task.abort();
}
