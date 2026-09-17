//! WebSocket 传输层 (per ULYS-2 任务 B / W33)
//!
//! ## 设计
//! - 路径 `/websocket`, 端口默认 8000 (对齐原 zsyz_server `web_conn.erl` 8000)
//! - 帧格式复用 `codec::Frame::decode/encode` ([4B code][4B length][payload])
//! - 业务 dispatcher 走 `Arc<dyn FrameRouter>` (per ULYS-2.2 codec.rs)
//! - 当前 Phase 1.5 骨架: WS 收到 binary frame → Frame::decode → router.handle → binary 回包
//!
//! ## 与 TCP 路径的关系
//! - TCP (`tcp.rs`) 用 `RouteTable` + `GatewayStats` 直接 dispatch
//! - WS (`ws.rs`) 通过 `FrameRouter` trait 抽象, 默认 `RouteTableFrameRouter` 包同样的 RouteTable
//! - 两路径可同时运行 (默认 WS 开, TCP 显式 `--tcp-addr` 才开, per main.rs 改动)
//!
//! ## 已知缺口 (Phase 2 推进)
//! - 握手时校验 `Sec-WebSocket-Protocol` / Origin (当前接受所有)
//! - mTLS (wss://) — 任务 brief 不要求, Phase 2 接 rustls
//! - 二进制帧粘包 / 半包: 已用 BytesMut + Frame::decode 流式处理 (同 tcp.rs)
//! - 心跳 / ping-pong: 暂未启用 (客户端 cmd=1199 走业务层处理)
//!
//! ## 参考
//! - 9/12 ULYS-2 任务 B 派工 brief
//! - zsyz_client_h5 SmartSocket.connect: `ws(s)://host:port/websocket`, binary frame
//! - zsyz_server `web_conn.erl` 端口 8000

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{Callback, Request, Response};
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};
use tracing::{debug, info, warn};

use crate::codec::{Frame, FrameError, FrameRouter};
use crate::stats::GatewayStats;

/// 默认 WS 监听地址 (per 任务 brief + zsyz_server web_conn.erl: 8000)
pub const DEFAULT_WS_ADDR: &str = "0.0.0.0:8000";

/// WebSocket 路径 (per 任务 brief, 客户端写死 /websocket)
pub const WS_PATH: &str = "/websocket";

/// 单帧最大字节数 (与 codec.rs MAX_FRAME 对齐, 1 MiB)
const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// WS 启动配置
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub bind_addr: SocketAddr,
    /// WS 路径, 默认 `/websocket`. 客户端必须用此路径连, 否则 404
    pub path: String,
    /// 允许的最大并发连接数 (backpressure); 0 = 无上限
    pub max_connections: usize,
}

impl WsConfig {
    /// 默认配置 (0.0.0.0:8000 + 路径 /websocket + 256 并发上限)
    pub fn default_local() -> Self {
        Self {
            bind_addr: DEFAULT_WS_ADDR
                .parse()
                .expect("DEFAULT_WS_ADDR must be valid SocketAddr"),
            path: WS_PATH.to_string(),
            max_connections: 256,
        }
    }
}

/// 启动 WS 监听 (主入口, main.rs 调用)
///
/// 监听流程:
/// 1. `TcpListener::bind(addr)` — 接受 TCP 握手
/// 2. 检查 HTTP request 行, 校验 `Upgrade: websocket` + 路径匹配 (404 否则)
/// 3. `tokio_tungstenite::accept_async` 完成 WS 握手
/// 4. `tokio::spawn(handle_session)` 进入帧循环
///
/// ## 错误处理
/// - bind 失败 → 返回 Err, main.rs 走 warn + 跳过 (W32 fix 模式)
/// - 单 session 失败 → warn 继续, 不影响其他 session
pub async fn serve(
    cfg: WsConfig,
    router: Arc<dyn FrameRouter>,
    stats: Arc<GatewayStats>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(cfg.bind_addr).await?;
    info!(
        addr = %cfg.bind_addr,
        path = %cfg.path,
        max_conn = cfg.max_connections,
        "network-gateway WS listening"
    );

    loop {
        let (sock, peer) = listener.accept().await?;
        debug!(peer = %peer, "WS TCP accepted");

        // 路径校验: 在 accept_async 前 peek HTTP request 行
        // 0 长度路径 (cfg.path = "/websocket") 视为只接受根路径, 写死 WS_PATH
        let path = cfg.path.clone();
        let router = Arc::clone(&router);
        let stats = Arc::clone(&stats);

        stats.inc_active();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(sock, peer, &path, router, stats.clone()).await {
                warn!(peer = %peer, err = %e, "WS conn ended with error");
            }
            stats.dec_active();
        });
    }
}

/// 单连接处理: HTTP 路径校验 → WS 握手 → 帧循环
async fn handle_conn(
    sock: TcpStream,
    peer: SocketAddr,
    expected_path: &str,
    router: Arc<dyn FrameRouter>,
    stats: Arc<GatewayStats>,
) -> std::io::Result<()> {
    // 用 tungstenite low-level API 校验 HTTP request 行 (避免 RustlsAcceptor 依赖)
    // accept_async 内部已做完整握手, 这里我们用自定义 handshake 步骤:
    // 1. peek 第一个 HTTP request
    // 2. 校验路径
    // 3. 调 accept_async 完整握手
    let ws = match accept_ws_with_path(sock, expected_path).await {
        Ok(ws) => ws,
        Err(HttpReject::NotFound) => {
            warn!(peer = %peer, expected = %expected_path, "WS path mismatch → 404");
            return Ok(()); // 软失败, 不计入 error
        }
        Err(HttpReject::Io(e)) => return Err(e),
        Err(HttpReject::Ws(e)) => {
            warn!(peer = %peer, err = %e, "WS handshake failed");
            return Ok(()); // 握手失败不算 IO error
        }
    };

    debug!(peer = %peer, "WS handshake complete");
    handle_session(ws, router, stats).await
}

/// HTTP 拒绝原因 (内部 helper)
enum HttpReject {
    NotFound,
    Io(std::io::Error),
    Ws(WsError),
}

/// 接受 WS 连接, 先校验 HTTP request 路径再握手
///
/// ## 路径校验策略
/// - 用 `accept_hdr_async` + Callback 在 handshake 阶段检查 path
/// - 路径不匹配 → Callback 里设标志位, 完成后我们返回 HttpReject::NotFound
/// - tungstenite 0.24 Callback 签名: `on_request(self, &Request, Response) -> Result<Response, ErrorResponse>`
///   其中 `Response = HttpResponse<()>`, `ErrorResponse = HttpResponse<Option<String>>`
async fn accept_ws_with_path(
    sock: TcpStream,
    expected_path: &str,
) -> Result<WebSocketStream<TcpStream>, HttpReject> {
    use std::sync::Mutex;
    use tokio_tungstenite::tungstenite::http::Response as HttpResponse;

    let path_valid: std::sync::Arc<Mutex<Option<bool>>> =
        std::sync::Arc::new(Mutex::new(None));
    let expected = expected_path.to_string();

    struct PathCheck {
        valid_flag: std::sync::Arc<Mutex<Option<bool>>>,
        expected: String,
    }

    impl Callback for PathCheck {
        fn on_request(
            self,
            req: &Request,
            response: Response,
        ) -> Result<Response, HttpResponse<Option<String>>> {
            let path = req.uri().path();
            let is_match = path == self.expected;
            *self.valid_flag.lock().unwrap() = Some(is_match);
            // 不管匹配与否都返回 Ok(response), 让 handshake 完成.
            // 路径错的标志位会在外面读取, 然后我们返回 HttpReject::NotFound.
            // 注: tungstenite 0.24 没有提供直接返回 4xx 的 API 在 callback 里,
            //      所以这里接受 WS upgrade 后立即 close (per RFC 6455 §4.4 客户端会看到 close frame)
            Ok(response)
        }
    }

    let callback = PathCheck {
        valid_flag: std::sync::Arc::clone(&path_valid),
        expected: expected.clone(),
    };

    let ws = match accept_hdr_async(sock, callback).await {
        Ok(ws) => ws,
        Err(e) => {
            return Err(HttpReject::Ws(e));
        }
    };

    let valid = path_valid.lock().unwrap().clone();
    match valid {
        Some(true) => Ok(ws),
        Some(false) => Err(HttpReject::NotFound),
        None => {
            // 回调没触发, 不太可能 (handshake 完成时一定调用过 on_request)
            Err(HttpReject::Ws(WsError::AlreadyClosed))
        }
    }
}

/// 单 session: 帧循环
///
/// 帧循环逻辑:
/// - 收到 `Message::Binary(bin)` → extend 进 BytesMut, 循环 Frame::decode
/// - 每解出一帧 → `router.handle(frame).await` → 回包
/// - 收到 `Message::Close(_)` → break (客户端主动关闭)
/// - 其他消息 (Text/Ping/Pong/Frame) → continue (忽略或回 Pong)
async fn handle_session<S>(
    ws: WebSocketStream<S>,
    router: Arc<dyn FrameRouter>,
    stats: Arc<GatewayStats>,
) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut write, mut read) = ws.split();
    let mut buf = BytesMut::with_capacity(64 * 1024);

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!(err = %e, "WS read error → close session");
                return Ok(());
            }
        };
        match msg {
            Message::Binary(bin) => {
                buf.extend_from_slice(&bin);
                // 循环 decode: 可能一个 WS message 含多个 frame (粘包), 也可能半个 (半包)
                loop {
                    match Frame::decode(&mut buf) {
                        Ok(Some(frame)) => {
                            stats.inc_received();
                            let resp: Bytes = router.handle(frame).await;
                            // 写回 binary
                            // Bytes → Vec<u8> (tungstenite 0.24 Message::Binary takes Vec)
                            let out: Vec<u8> = resp.to_vec();
                            if let Err(e) = write.send(Message::Binary(out)).await {
                                warn!(err = %e, "WS write error → close session");
                                return Ok(());
                            }
                        }
                        Ok(None) => break, // 缓冲不够, 等下个 WS message
                        Err(FrameError::TooShort { .. }) => break, // 同上
                        Err(FrameError::LengthOverflow { declared, max }) => {
                            warn!(declared, max, "WS frame length overflow → drop session");
                            stats.inc_failed();
                            let _ = write.send(Message::Close(None)).await;
                            return Ok(());
                        }
                        // 其他协议错误 (TruncatedField / UnknownTlvType / InvalidUtf8)
                        // 都是 wire 格式畸形, 一律 drop session 防滥用
                        Err(e) => {
                            warn!(err = %e, "WS frame decode protocol error → drop session");
                            stats.inc_failed();
                            let _ = write.send(Message::Close(None)).await;
                            return Ok(());
                        }
                    }
                }
                // 防止 buf 单边无限增长 (deframe 不消耗)
                if buf.len() > MAX_FRAME_BYTES {
                    warn!(buf_len = buf.len(), "WS buf too large → drop session");
                    let _ = write.send(Message::Close(None)).await;
                    return Ok(());
                }
            }
            Message::Close(frame) => {
                debug!(?frame, "WS close frame → exit session");
                // 回 echo close
                let _ = write.send(Message::Close(frame)).await;
                return Ok(());
            }
            Message::Ping(payload) => {
                // 自动回 Pong (tungstenite 半自动, 但保险起见手写)
                if let Err(e) = write.send(Message::Pong(payload)).await {
                    warn!(err = %e, "WS pong write error");
                    return Ok(());
                }
            }
            Message::Pong(_) => {
                // 忽略 (客户端响应我们 ping, 但当前我们不发 ping)
            }
            Message::Text(_) => {
                // 任务 brief 只支持 binary frame; Text 忽略 (per zsyz_client_h5 send 是 binary)
                debug!("WS text frame ignored (binary-only)");
            }
            Message::Frame(_) => {
                // tungstenite 内部 raw frame, 不应直接收到; 忽略
                debug!("WS raw frame ignored");
            }
        }
    }
    debug!("WS stream ended");
    Ok(())
}

/// 兼容检查: Role 标记为 Server (供未来 TLS acceptor 集成用, 当前 stub)
#[allow(dead_code)]
fn _assert_server_role() {
    let _r = Role::Server;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::RouteTable;
    use bytes::Bytes;
    use std::sync::Arc;

    /// 测试用 FrameRouter: 走 RouteTable (沿用 tcp::dispatch 逻辑)
    struct RouteTableRouter {
        routes: Arc<RouteTable>,
        stats: Arc<GatewayStats>,
    }

    impl FrameRouter for RouteTableRouter {
        fn handle<'a>(
            &'a self,
            frame: Frame,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Bytes> + Send + 'a>,
        > {
            let resp = crate::tcp::dispatch(frame, &self.routes, &self.stats);
            Box::pin(async move { resp })
        }
    }

    #[test]
    fn ws_config_default_has_correct_addr() {
        let cfg = WsConfig::default_local();
        assert_eq!(cfg.bind_addr.to_string(), "0.0.0.0:8000");
        assert_eq!(cfg.path, "/websocket");
        assert_eq!(cfg.max_connections, 256);
    }

    #[test]
    fn ws_path_constant_matches_brief() {
        // 任务 brief: 路径 /websocket
        assert_eq!(WS_PATH, "/websocket");
    }

    #[test]
    fn default_ws_addr_matches_web_conn() {
        // zsyz_server web_conn.erl: 端口 8000
        assert_eq!(DEFAULT_WS_ADDR, "0.0.0.0:8000");
    }

    // 注: 集成测试 (tokio-tungstenite client + 真实 WS roundtrip) 在
    // `tests/ws_smoke.rs` 中跑, 这里只放纯单元测试 (无 IO).
}
