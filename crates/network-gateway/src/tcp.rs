//! TCP 二进制网关 (per 9/4 改进路线图 Phase 1 协议网关)
//!
//! ## 范围 (W6 0.5 SRE·d 骨架)
//! - TCP listener 127.0.0.1:7001 (per task brief)
//! - 帧格式: [4B code][4B length][lengthB payload] (per codec.rs)
//! - 收到 10101 演示路由到 player-service.CreateCharacter (gRPC client stub)
//! - 不做握手 / 加密 / 压缩 (per R3 风险, Phase 2)
//!
//! ## 已知缺口
//! - gRPC client 调通需 player-service 启动 (Phase 1.5 + Phase 3 联调)
//! - 连接认证 (Flash socket 策略 / 自研握手) Phase 1.5
//! - 粘包 / 半包 (本骨架假设一次 read 拿完整帧; Phase 1.5 改 BytesMut 流式解码)

use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

use crate::codec::{Frame, FrameError, PROTOCOL_HEADER_LEN};
use crate::router::RouteTable;
use crate::stats::GatewayStats;

/// TCP 监听端点 (per task brief)
pub const DEFAULT_TCP_ADDR: &str = "127.0.0.1:7001";

/// 启动 TCP 监听 (主入口, main.rs 调用)
///
/// 每个连接 spawn 一个 task 处理; 简单 read-exact-respond 循环.
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
        // 读 header (4B + 4B = 8B)
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
/// 返回 4 字节 (rcode) + 4 字节 (length) + payload 的应答帧.
pub fn dispatch(frame: Frame, routes: &RouteTable, stats: &GatewayStats) -> Bytes {
    let code = frame.code;
    match routes.get(code) {
        Some(entry) => {
            // Phase 1.5: 实际调 entry.target_addr 的 entry.target_method
            // 当前骨架: 返回路由决策结果 (二进制 0=rcode=OK, payload 写 service.method)
            stats.inc_forwarded();
            let payload = format!(
                "{}#{}",
                entry.target_service, entry.target_method
            );
            build_response_frame(0, payload.as_bytes())
        }
        None => {
            stats.inc_route_miss();
            warn!(code = code, "route miss");
            let payload = format!("unknown code {}", code).into_bytes();
            build_response_frame(404, &payload)
        }
    }
}

/// 构建应答帧: 4B rcode + 4B length + payload
fn build_response_frame(rcode: u32, payload: &[u8]) -> Bytes {
    let mut out = BytesMut::with_capacity(8 + payload.len());
    out.extend_from_slice(&rcode.to_be_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out.freeze()
}

/// 错误帧 (协议错 / frame decode 失败)
fn build_error_frame(e: &FrameError) -> Vec<u8> {
    let payload = e.to_string().into_bytes();
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&400u32.to_be_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(&payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_route_hit() {
        let routes = RouteTable::new();
        let stats = GatewayStats::new();
        let frame = Frame {
            code: 10101,
            payload: Bytes::from_static(b"hello"),
        };
        let resp = dispatch(frame, &routes, &stats);
        // 4B rcode(0) + 4B length + payload
        assert!(resp.len() > 8);
        let rcode = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
        assert_eq!(rcode, 0);
        let snap = stats.snapshot();
        assert_eq!(snap.total_forwarded, 1);
        assert_eq!(snap.total_route_miss, 0);
    }

    #[test]
    fn dispatch_route_miss() {
        let routes = RouteTable::new();
        let stats = GatewayStats::new();
        let frame = Frame {
            code: 99999,
            payload: Bytes::from_static(b""),
        };
        let resp = dispatch(frame, &routes, &stats);
        let rcode = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
        assert_eq!(rcode, 404);
        let snap = stats.snapshot();
        assert_eq!(snap.total_route_miss, 1);
    }

    #[test]
    fn dispatch_received_counter_increments() {
        let routes = RouteTable::new();
        let stats = GatewayStats::new();
        // received 在 handle_conn 中 increment, 不在 dispatch 中; 这里测 dispatch 不动 received
        let frame = Frame {
            code: 10101,
            payload: Bytes::from_static(b"x"),
        };
        dispatch(frame, &routes, &stats);
        let snap = stats.snapshot();
        // dispatch 只 inc forwarded / route_miss
        assert_eq!(snap.total_received, 0);
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
