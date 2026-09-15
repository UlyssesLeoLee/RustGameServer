//! TCP 二进制网关 (per 9/4 改进路线图 Phase 1 协议网关 + ULYS-2.1 P0 真实协议对齐)
//!
//! ## 范围
//! - TCP listener 127.0.0.1:7001 (per task brief)
//! - 帧格式: 与 zsyz_client_h5 客户端 SmartSocket 1:1 对齐 — `[4B length u32 BE][2B cmd u16 BE][payload TLV]`
//!   (见 `codec.rs`).
//! - 收到客户端帧 → 路由到 gRPC method (per `router.rs`) → 返回响应帧.
//!
//! ## 响应帧约定
//! 服务端响应也是 zsyz wire 格式: `[length u32][cmd u16][payload]`.
//! payload 内部约定: `[4B rcode u32 BE][...业务数据...]`.
//! 路由命中 → rcode=0, 业务数据为 `target_service.target_method` UTF-8 字符串.
//! 未注册 cmd → rcode=404, 业务数据为 `"unknown code <cmd>"` 字符串.
//!
//! ## 已知缺口
//! - gRPC client 调通需 player-service 启动 (Phase 1.5 + Phase 3 联调).
//! - 本骨架仅做路由决策 (rcode + service.method 文本), 实际 gRPC 调用 Phase 1.5.

use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

use crate::codec::{Frame, FrameError, MAX_FRAME, PROTOCOL_HEADER_LEN};
use crate::router::RouteTable;
use crate::stats::GatewayStats;

/// TCP 监听端点 (per task brief)
pub const DEFAULT_TCP_ADDR: &str = "127.0.0.1:7001";

/// 启动 TCP 监听 (主入口, main.rs 调用)
///
/// 每个连接 spawn 一个 task 处理; 流式 BytesMut 循环读帧 → 路由 → 应答.
pub async fn serve(
    addr: &str,
    routes: Arc<RouteTable>,
    stats: Arc<GatewayStats>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!(addr = %addr, "network-gateway TCP listening");

    loop {
        let (sock, peer) = listener.accept().await?;
        debug!(peer = %peer, "TCP accepted");
        stats.inc_active();
        let routes = Arc::clone(&routes);
        let stats = Arc::clone(&stats);
        tokio::spawn(async move {
            if let Err(e) = handle_conn(sock, routes, stats.clone()).await {
                warn!(peer = %peer, err = %e, "conn ended with error");
            }
            stats.dec_active();
        });
    }
}

/// 单连接处理: 循环读帧 → 路由 → 应答
async fn handle_conn(
    mut sock: TcpStream,
    routes: Arc<RouteTable>,
    stats: Arc<GatewayStats>,
) -> std::io::Result<()> {
    let mut buf = BytesMut::with_capacity(64 * 1024);
    loop {
        // 读 header (4B length + 2B cmd = 6B)
        if buf.len() < PROTOCOL_HEADER_LEN {
            let n = sock.read_buf(&mut buf).await?;
            if n == 0 {
                if buf.is_empty() {
                    return Ok(()); // 干净关闭
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "EOF in header",
                ));
            }
        }
        // 尝试解码 (可能缓冲不足)
        let frame = match Frame::decode(&mut buf) {
            Ok(Some(f)) => f,
            Ok(None) => {
                // 缓冲不够, 继续读
                let n = sock.read_buf(&mut buf).await?;
                if n == 0 {
                    if buf.is_empty() {
                        return Ok(());
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "EOF in payload",
                    ));
                }
                continue;
            }
            Err(e) => {
                warn!(err = %e, "frame decode error");
                stats.inc_failed();
                // 错误回包: 用 0 cmd 携带 rcode=400 + 错误描述
                let resp = build_error_frame(&e);
                sock.write_all(&resp).await?;
                return Ok(());
            }
        };

        stats.inc_received();
        let resp = dispatch(frame, &routes, &stats);
        sock.write_all(&resp).await?;
    }
}

/// 派发帧到 gRPC method (Phase 1 骨架: 仅路由决策, 实际 gRPC 调用 Phase 1.5)
///
/// 返回响应 wire 帧: `[length u32][cmd u16][payload]` 其中 cmd 与请求相同,
/// payload 内部: `[4B rcode u32 BE][...业务 bytes...]`.
///
/// - rcode=0: 命中路由, 业务数据 = `target_service.target_method` UTF-8.
/// - rcode=404: 未注册 cmd, 业务数据 = `"unknown code <cmd>"` UTF-8.
pub fn dispatch(frame: Frame, routes: &RouteTable, stats: &GatewayStats) -> Bytes {
    let cmd = frame.cmd;
    let payload = match routes.get(cmd as u32) {
        Some(entry) => {
            stats.inc_forwarded();
            let body = format!("{}#{}", entry.target_service, entry.target_method);
            build_response_payload(0, body.as_bytes())
        }
        None => {
            stats.inc_route_miss();
            warn!(cmd = cmd, "route miss");
            let body = format!("unknown code {}", cmd).into_bytes();
            build_response_payload(404, &body)
        }
    };
    let resp_frame = Frame {
        cmd,
        payload: Bytes::from(payload),
    };
    resp_frame.encode()
}

/// 构造响应 payload: `[4B rcode u32 BE][body]`.
fn build_response_payload(rcode: u32, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&rcode.to_be_bytes());
    out.extend_from_slice(body);
    out
}

/// 错误帧 (协议错 / frame decode 失败): cmd=0, payload=[400 rcode][err.to_string()].
fn build_error_frame(e: &FrameError) -> Bytes {
    let payload = build_response_payload(400, e.to_string().as_bytes());
    let frame = Frame {
        cmd: 0,
        payload: Bytes::from(payload),
    };
    frame.encode()
}

/// 路由表 + 入参字节统计 (Phase 1.5 接 7 域真实 .proto 后, 这里 stub 仅路由决策).
#[allow(dead_code)]
const fn _max_frame_ref() -> usize {
    MAX_FRAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_route_hit_returns_frame() {
        let routes = RouteTable::new();
        let stats = GatewayStats::new();
        let frame = Frame {
            cmd: 10101,
            payload: Bytes::from_static(b"hello"),
        };
        let resp_bytes = dispatch(frame, &routes, &stats);
        // 解码响应 → 验证 cmd + payload[0..4]=rcode
        let mut buf = BytesMut::from(&resp_bytes[..]);
        let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(resp_frame.cmd, 10101, "响应 cmd 应回声");
        let rcode = u32::from_be_bytes([
            resp_frame.payload[0],
            resp_frame.payload[1],
            resp_frame.payload[2],
            resp_frame.payload[3],
        ]);
        assert_eq!(rcode, 0);
        let body = &resp_frame.payload[4..];
        let body_str = std::str::from_utf8(body).unwrap();
        assert_eq!(body_str, "player.v1.PlayerService#CreateCharacter");
        let snap = stats.snapshot();
        assert_eq!(snap.total_forwarded, 1);
        assert_eq!(snap.total_route_miss, 0);
    }

    #[test]
    fn dispatch_route_miss_returns_404() {
        let routes = RouteTable::new();
        let stats = GatewayStats::new();
        // 65535 = u16 max, 不在任何路由表条目里 (max cmd=10101 等)
        let frame = Frame {
            cmd: 65535,
            payload: Bytes::from_static(b""),
        };
        let resp_bytes = dispatch(frame, &routes, &stats);
        let mut buf = BytesMut::from(&resp_bytes[..]);
        let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(resp_frame.cmd, 65535);
        let rcode = u32::from_be_bytes([
            resp_frame.payload[0],
            resp_frame.payload[1],
            resp_frame.payload[2],
            resp_frame.payload[3],
        ]);
        assert_eq!(rcode, 404);
        let body = &resp_frame.payload[4..];
        let body_str = std::str::from_utf8(body).unwrap();
        assert!(body_str.contains("65535"));
        let snap = stats.snapshot();
        assert_eq!(snap.total_route_miss, 1);
    }

    #[test]
    fn dispatch_received_counter_increments() {
        let routes = RouteTable::new();
        let stats = GatewayStats::new();
        // received 在 handle_conn 中 increment, 不在 dispatch 中; 这里测 dispatch 不动 received
        let frame = Frame {
            cmd: 10101,
            payload: Bytes::from_static(b"x"),
        };
        dispatch(frame, &routes, &stats);
        let snap = stats.snapshot();
        // dispatch 只 inc forwarded / route_miss
        assert_eq!(snap.total_received, 0);
    }

    #[test]
    fn build_response_payload_format() {
        let p = build_response_payload(0, b"abc");
        // 4B rcode(=0) + "abc"
        assert_eq!(p.len(), 7);
        assert_eq!(&p[0..4], &[0, 0, 0, 0]);
        assert_eq!(&p[4..7], b"abc");
    }

    #[tokio::test]
    async fn serve_smoke_bind_then_close() {
        // 仅测 bind 成功 + 立即关闭 listener (cancel-safe)
        let routes = Arc::new(RouteTable::new());
        let stats = Arc::new(GatewayStats::new());
        // 0 端口由 OS 分配, 避免冲突
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        // 短暂 listen
        let handle = tokio::spawn(async move {
            let _ = tokio::time::timeout(
                std::time::Duration::from_millis(50),
                serve(&addr.to_string(), routes, stats),
            )
            .await;
        });
        handle.await.unwrap();
    }
}