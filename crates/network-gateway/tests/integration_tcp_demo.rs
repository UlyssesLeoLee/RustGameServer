//! 集成测试: TCP 接 [游戏A] 真实 wire 协议 → 路由到 5 域 demo service
//!
//! ## 范围 (per ULYS-2.1 P0 + 9/4 改进路线图 Phase 1 协议网关)
//! - 起 1 个 0 端口 TCP listener (OS 分配)
//! - 客户端发 [游戏A] 帧 `[4B length u32 BE][2B cmd u16 BE][payload]`
//! - 服务端 dispatch 到路由表, 返回 [游戏A] 帧, payload 内部: `[4B rcode u32 BE][...业务 bytes...]`
//! - 验证 rcode=0 + payload 内容 + stats 计数
//!
//! ## 与旧版差异 (per ULYS-2.1)
//! - 旧: `[4B code u32][4B length u32][payload]` (stub, 与客户端 1:1 不一致)
//! - 新: `[4B length u32 BE][2B cmd u16 BE][payload]` ([游戏A]_client_h5 SmartSocket 真实协议)
//! - 旧 cmd 字段名 `code: u32` → 新 `cmd: u16`
//! - 旧响应 `[4B rcode][4B length][payload]` → 新响应直接是 frame payload, 内部 `[4B rcode][body]`
//!
//! ## 已知缺口
//! - 实际 gRPC client 调通需 player-service 启动 (Phase 1.5 + Phase 3 联调)
//! - 这里只验证 routing decision, 不验证端到端 gRPC (per task brief "即使调不通也算")

use std::sync::Arc;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use network_gateway::codec::{Frame, PROTOCOL_HEADER_LEN};
use network_gateway::router::RouteTable;
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::test]
async fn tcp_demo_route_10101_to_player_create_character() {
    let routes = Arc::new(RouteTable::new());
    let stats = Arc::new(GatewayStats::new());

    // 验证 dispatch (Phase 1 骨架: routing decision)
    // cmd=10101 → player.v1.PlayerService#CreateCharacter (per W14 codegen)
    let frame = Frame {
        cmd: 10101,
        payload: Bytes::from_static(b"hello"),
    };
    let resp_bytes = tcp::dispatch(frame, &routes, &stats);
    assert!(resp_bytes.len() > PROTOCOL_HEADER_LEN);

    // 解析响应 frame: [4B length][2B cmd][payload]
    let mut buf = BytesMut::from(&resp_bytes[..]);
    let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
    assert_eq!(resp_frame.cmd, 10101, "响应 cmd 应回声");

    // payload 内部: [4B rcode u32 BE][...业务 bytes...]
    let payload = &resp_frame.payload;
    assert!(payload.len() >= 4);
    let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(rcode, 0, "路由 10101 应成功 (默认路由表 demo)");
    let body = &payload[4..];
    let body_str = std::str::from_utf8(body).unwrap();
    assert_eq!(body_str, "player.v1.PlayerService#CreateCharacter");
}

#[tokio::test]
async fn tcp_demo_route_miss_returns_404() {
    let routes = RouteTable::new();
    let stats = GatewayStats::new();
    // cmd=1351 远超默认表的最大 code
    let frame = Frame {
        cmd: 1351,
        payload: Bytes::from_static(b"x"),
    };
    let resp_bytes = tcp::dispatch(frame, &routes, &stats);
    let mut buf = BytesMut::from(&resp_bytes[..]);
    let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
    assert_eq!(resp_frame.cmd, 1351);
    let payload = &resp_frame.payload;
    let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(rcode, 404, "未注册 cmd 应返回 404");
    let snap = stats.snapshot();
    assert_eq!(snap.total_route_miss, 1);
}

#[tokio::test]
async fn tcp_serve_client_roundtrip() {
    // 真实 TCP 端到端: 启动 serve, 客户端发帧, 验证响应
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

    // 客户端发 [游戏A] 帧: cmd=10101, payload="hello" (5B)
    // wire: [4B length=2+5=7][2B cmd=0x2775][5B payload="hello"]
    let mut client = TcpStream::connect(&addr).await.expect("connect ok");
    let req_frame = Frame {
        cmd: 10101,
        payload: Bytes::from_static(b"hello"),
    };
    let wire = req_frame.encode();
    client.write_all(&wire).await.expect("write ok");
    client.flush().await.ok();

    // 读响应 (4B length + 2B cmd + 4B rcode + body 长度未知, 一次读到 EOF)
    let mut buf = Vec::new();
    let mut tmp = [0u8; 256];
    loop {
        match tokio::time::timeout(Duration::from_secs(1), client.read(&mut tmp)).await {
            Ok(Ok(0)) => break, // EOF
            Ok(Ok(n)) => buf.extend_from_slice(&tmp[..n]),
            Ok(Err(e)) => panic!("read err: {e}"),
            Err(_) => break, // 1s timeout, 视作收完
        }
    }
    assert!(buf.len() >= PROTOCOL_HEADER_LEN + 4, "至少 6+4=10 字节响应, got {}", buf.len());

    // 解析响应 frame
    let mut resp_buf = BytesMut::from(&buf[..]);
    let resp_frame = Frame::decode(&mut resp_buf).unwrap().expect("响应帧解析成功");
    assert_eq!(resp_frame.cmd, 10101, "响应 cmd 应回声");

    let payload = &resp_frame.payload;
    let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(rcode, 0, "rcode 应为 0 (route hit)");
    let body = &payload[4..];
    let body_str = std::str::from_utf8(body).unwrap();
    assert_eq!(body_str, "player.v1.PlayerService#CreateCharacter");

    drop(client);
    tokio::time::sleep(Duration::from_millis(50)).await;

    let snap = stats.snapshot();
    assert_eq!(snap.total_received, 1, "received 计数 = 1");
    assert_eq!(snap.total_forwarded, 1, "forwarded 计数 = 1");
    assert_eq!(snap.total_route_miss, 0);

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
    // cmd=65535 (u16 max), payload 空 → length=2 (仅含 cmd)
    let req_frame = Frame {
        cmd: 65535,
        payload: Bytes::new(),
    };
    let wire = req_frame.encode();
    client.write_all(&wire).await.expect("write ok");
    client.flush().await.ok();

    let mut buf = Vec::new();
    let mut tmp = [0u8; 256];
    loop {
        match tokio::time::timeout(Duration::from_secs(1), client.read(&mut tmp)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => buf.extend_from_slice(&tmp[..n]),
            Ok(Err(e)) => panic!("read err: {e}"),
            Err(_) => break,
        }
    }
    let mut resp_buf = BytesMut::from(&buf[..]);
    let resp_frame = Frame::decode(&mut resp_buf).unwrap().expect("响应帧解析成功");
    let payload = &resp_frame.payload;
    let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(rcode, 404, "未注册 cmd 应返回 404");

    drop(client);
    tokio::time::sleep(Duration::from_millis(50)).await;

    let snap = stats.snapshot();
    assert_eq!(snap.total_route_miss, 1);

    serve_task.abort();
}
