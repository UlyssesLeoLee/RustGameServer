// rgs-shim v0.3.0 — production-grade Rust rewrite (per 9/9 14:20 JST Ulysses 拍板)
// 完整对接 RGS + 100% 正常 + 性能更优 (Rust + tokio) + 无移植隐患 (同 RGS 5 binary 语言)
// async TCP listener 9001, big-endian binary frame, cmd dispatch, 5 域 RGS gRPC via rgs-proxy

mod frame;
mod handlers;
mod handlers_social;
mod handlers_w2;
mod registry;
mod rgs;
mod registry_stubs;

use registry::Registry;
use rgs::RgsClient;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

const SHIM_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rgs_shim=info".into()),
        )
        .init();

    let shim_port: u16 = std::env::var("SHIM_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9001);
    let rgs_proxy = std::env::var("RGS_PROXY")
        .unwrap_or_else(|_| "http://127.0.0.1:8084".to_string());

    let rgs = Arc::new(RgsClient::new(rgs_proxy.clone()));
    let registry = Arc::new(Registry::new());

    let listener = TcpListener::bind(("0.0.0.0", shim_port)).await?;
    info!(
        version = SHIM_VERSION,
        port = shim_port,
        %rgs_proxy,
        cmds = registry.list().len(),
        "rgs-shim listening"
    );

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let rgs = rgs.clone();
                let registry = registry.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, addr, rgs, registry).await {
                        warn!(%addr, error = %e, "connection error");
                    }
                });
            }
            Err(e) => error!(error = %e, "accept error"),
        }
    }
}

async fn handle_connection(
    socket: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    rgs: Arc<RgsClient>,
    registry: Arc<Registry>,
) -> anyhow::Result<()> {
    info!(%addr, "+ client");
    let (mut reader, writer) = tokio::io::split(socket);
    let writer = Arc::new(Mutex::new(writer));
    let mut frame_count: u32 = 0;
    let mut buf = vec![0u8; 4096];

    loop {
        let n = match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                warn!(%addr, error = %e, "read error");
                break;
            }
        };

        // 解析所有完整帧
        let mut idx = 0;
        while idx + 6 <= n {
            let len = u32::from_be_bytes([buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3]]) as usize;
            let total = 4 + len;
            if n < idx + total {
                break;
            }
            // 解析 cmd + payload
            let cmd = u16::from_be_bytes([buf[idx + 4], buf[idx + 5]]);
            let payload_len = len - 2;
            let payload = buf[idx + 6..idx + 6 + payload_len].to_vec();
            frame_count += 1;
            info!(%addr, frame = frame_count, cmd, payload_len, "frame");

            // dispatch (owned payload + cloned rgs Arc 进 future)
            let rgs = rgs.clone();
            let registry = registry.clone();
            let writer = writer.clone();
            tokio::spawn(async move {
                let resp = registry.dispatch(cmd, payload, rgs).await;
                let mut out = Vec::with_capacity(6 + resp.payload.len());
                let resp_len = (2 + resp.payload.len()) as u32;
                out.extend_from_slice(&resp_len.to_be_bytes());
                out.extend_from_slice(&resp.cmd.to_be_bytes());
                out.extend_from_slice(&resp.payload);
                let mut w = writer.lock().await;
                if let Err(e) = w.write_all(&out).await {
                    warn!(cmd = resp.cmd, error = %e, "write error");
                } else {
                    info!(%addr, cmd = resp.cmd, payload_len = resp.payload.len(), "→");
                }
            });
            idx += total;
        }
    }
    info!(%addr, frame_count, "- client");
    Ok(())
}
