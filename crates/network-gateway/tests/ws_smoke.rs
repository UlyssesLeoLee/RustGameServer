//! WS 传输层集成测试 (per ULYS-2.2 任务 brief §"验证")
//!
//! ## 范围
//! - 起 1 个 0 端口 WS listener (OS 分配, 路径 /websocket)
//! - 客户端用 tokio-tungstenite 作 client (同 crate 作 client 跟 server 配对自测)
//! - 发 zsyz 帧 `[4B length u32 BE][2B cmd u16 BE][payload]`
//! - 验证响应 (response 也应是 frame, payload 内部 `[4B rcode u32 BE][body]`)
//!
//! ## FrameRouter 默认实现 (跟 main.rs 同源)
//! - `RouteTableFrameRouter` 包 `RouteTable + tcp::dispatch`
//! - Phase 1 骨架: routing decision, 不调真实 gRPC
//!
//! ## 已知缺口
//! - 端到端 7 域 gRPC client 调通需 5 域服务启动 (Phase 1.5 + Phase 3 联调)
//! - 这里只验证 routing decision, 不验证端到端 gRPC (per task brief)

use std::sync::Arc;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use network_gateway::codec::{Frame, FrameRouter};
use network_gateway::router::RouteTable;
use network_gateway::stats::GatewayStats;
use network_gateway::tcp;
use network_gateway::ws;
use std::future::Future;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

/// FrameRouter 默认实现 (跟 main.rs 同源)
struct RouteTableFrameRouter {
    routes: Arc<RouteTable>,
    stats: Arc<GatewayStats>,
}

impl FrameRouter for RouteTableFrameRouter {
    fn handle<'a>(
        &'a self,
        frame: Frame,
    ) -> Pin<Box<dyn Future<Output = Bytes> + Send + 'a>> {
        let resp = tcp::dispatch(frame, &self.routes, &self.stats);
        Box::pin(async move { resp })
    }
}

/// 起 1 个 0 端口 WS listener, 返回 ws URL + shutdown sender + server_addr
async fn start_ws_server() -> (std::net::SocketAddr, tokio::sync::oneshot::Sender<()>) {
    let routes = Arc::new(RouteTable::with_phase15_demo());
    let stats = Arc::new(GatewayStats::new());
    let router: Arc<dyn FrameRouter> = Arc::new(RouteTableFrameRouter { routes, stats });

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // 先 probe 拿一个 OS 分配的 port
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = probe.local_addr().unwrap();
    drop(probe); // 释放 port (有 race 但 OK test)

    let cfg = ws::WsConfig {
        bind_addr: server_addr,
        path: "/websocket".to_string(),
        max_connections: 16,
    };

    tokio::spawn(async move {
        let _ = run_ws_inline(cfg, router, shutdown_rx).await;
    });

    // 给 server 一点时间起来
    tokio::time::sleep(Duration::from_millis(50)).await;

    (server_addr, shutdown_tx)
}

/// 内联 WS 循环 (跟 ws::serve 同语义)
async fn run_ws_inline(
    cfg: ws::WsConfig,
    router: Arc<dyn FrameRouter>,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.bind_addr).await?;
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                return Ok(());
            }
            accept_result = listener.accept() => {
                let (sock, peer) = match accept_result {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("WS accept err: {e}");
                        continue;
                    }
                };
                let path = cfg.path.clone();
                let router = Arc::clone(&router);
                tokio::spawn(async move {
                    if let Err(e) = handle_session(sock, &path, router).await {
                        eprintln!("WS session err peer={peer}: {e}");
                    }
                });
            }
        }
    }
}

/// 简化版 WS session (路径校验 + 帧循环)
async fn handle_session(
    sock: TcpStream,
    expected_path: &str,
    router: Arc<dyn FrameRouter>,
) -> std::io::Result<()> {
    use tokio_tungstenite::tungstenite::handshake::server::{Callback, Request, Response};
    use tokio_tungstenite::tungstenite::http::Response as HttpResponse;
    use tokio_tungstenite::accept_hdr_async;

    struct PathCheck {
        expected: String,
    }
    impl Callback for PathCheck {
        fn on_request(
            self,
            req: &Request,
            response: Response,
        ) -> Result<Response, HttpResponse<Option<String>>> {
            if req.uri().path() == self.expected {
                Ok(response)
            } else {
                Err(HttpResponse::builder()
                    .status(404)
                    .body(Some("not found".to_string()))
                    .unwrap())
            }
        }
    }

    let cb = PathCheck {
        expected: expected_path.to_string(),
    };
    let mut ws = match accept_hdr_async(sock, cb).await {
        Ok(ws) => ws,
        Err(_) => return Ok(()), // 路径错或握手失败, 静默丢弃
    };

    while let Some(msg) = ws.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };
        if let Message::Binary(bin) = msg {
            let mut buf = BytesMut::from(&bin[..]);
            while let Ok(Some(frame)) = Frame::decode(&mut buf) {
                let resp = router.handle(frame).await;
                if ws.send(Message::Binary(resp.to_vec())).await.is_err() {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

/// 连接 WS server, 返回 client stream
async fn connect_ws(
    server_addr: std::net::SocketAddr,
    path: &str,
) -> Result<WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Error> {
    let url = format!("ws://{}{}", server_addr, path);
    let req = url.into_client_request().unwrap();
    // 先 TCP connect (连真 port), 再 wrap WS
    let tcp = TcpStream::connect(server_addr).await.expect("TCP connect");
    let (ws, _resp) = tokio_tungstenite::client_async(req, tcp).await?;
    Ok(ws)
}

#[tokio::test]
async fn ws_handshake_and_route_roundtrip() {
    let (server_addr, shutdown_tx) = start_ws_server().await;

    let mut ws_client = connect_ws(server_addr, "/websocket")
        .await
        .expect("WS handshake ok");

    // 发 cmd=10101 (默认路由表 demo 命中, 期望 rcode=0)
    let frame = Frame {
        cmd: 10101,
        payload: Bytes::from_static(b"hello"),
    };
    let wire = frame.encode();
    ws_client
        .send(Message::Binary(wire.to_vec()))
        .await
        .expect("WS send ok");

    // 读响应
    let resp_msg = tokio::time::timeout(Duration::from_secs(2), ws_client.next())
        .await
        .expect("response timeout")
        .expect("stream ok")
        .expect("frame ok");

    let resp_bytes = match resp_msg {
        Message::Binary(b) => b,
        other => panic!("expected binary, got {other:?}"),
    };

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
    assert!(
        body_str.contains("PlayerService"),
        "body 应含 PlayerService (per W14 codegen), got: {body_str}"
    );

    // 主动 close
    let _ = ws_client.send(Message::Close(None)).await;
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn ws_route_miss_returns_404() {
    let (server_addr, shutdown_tx) = start_ws_server().await;

    let mut ws_client = connect_ws(server_addr, "/websocket")
        .await
        .expect("WS handshake ok");

    // cmd=55555 不在默认路由表 → 404
    let frame = Frame {
        cmd: 55555,
        payload: Bytes::from_static(b""),
    };
    let wire = frame.encode();
    ws_client
        .send(Message::Binary(wire.to_vec()))
        .await
        .expect("WS send ok");

    let resp_msg = tokio::time::timeout(Duration::from_secs(2), ws_client.next())
        .await
        .expect("response timeout")
        .expect("stream ok")
        .expect("frame ok");

    let resp_bytes = match resp_msg {
        Message::Binary(b) => b,
        other => panic!("expected binary, got {other:?}"),
    };

    let mut buf = BytesMut::from(&resp_bytes[..]);
    let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
    let payload = &resp_frame.payload;
    let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(rcode, 404, "未注册 cmd 应返回 404");

    let _ = ws_client.send(Message::Close(None)).await;
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn ws_wrong_path_returns_404_http() {
    let (server_addr, shutdown_tx) = start_ws_server().await;

    // 改 path 到 /wrong → server callback 应返回 Err 404
    let result = connect_ws(server_addr, "/wrong").await;
    assert!(
        result.is_err(),
        "路径 /wrong 应握手失败 (server 返回 404), got: {result:?}"
    );

    let _ = shutdown_tx.send(());
}