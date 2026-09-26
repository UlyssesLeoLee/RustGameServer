//! Prometheus metrics HTTP endpoint（per ARC-051 观测 + ULYS-100 P2-#2 6 域 outbox）
//!
//! 54.13 实化：scrape_metrics() 返回 (status, body) 简单 HTTP 响应
//! ULYS-100 扩展：bind_metrics_server() 启动 tokio TcpListener 监听 `/metrics` 端点
//!
//! 设计：
//! - 简化设计 (54.13)：返回 (status_code, content_type, body) 元组
//! - 业务方（各域 main.rs）调 `bind_metrics_server(addr)` 后台 tokio 任务
//! - HTTP/1.1 解析用 tokio + 手写 minimal parser (避免引入 hyper/axum 额外依赖)
//! - 仅支持 GET / 路径（健康检查 200 OK）和 GET /metrics 路径（Prometheus exposition format）
//! - 任何其他路径返回 404
//!
//! **不**绑死具体 HTTP 框架 (gm-backend 用 actix-web, 6 域服务用 tonic gRPC, 都可调 bind_metrics_server)

use crate::metrics::encode_to_text;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Metrics scrape 响应（per 54.13 API）
pub struct MetricsResponse {
    /// HTTP status code（200 OK）
    pub status: u16,
    /// Content-Type
    pub content_type: &'static str,
    /// Body
    pub body: String,
}

/// 抓取 /metrics（供 HTTP handler 调用）
pub fn scrape_metrics() -> MetricsResponse {
    match encode_to_text() {
        Ok(body) => MetricsResponse {
            status: 200,
            content_type: "text/plain; version=0.0.4",
            body,
        },
        Err(e) => MetricsResponse {
            status: 500,
            content_type: "text/plain; version=0.0.4",
            body: format!("# metrics encode error: {}", e),
        },
    }
}

/// 启动 metrics HTTP server 后台任务（per ULYS-100 P2-#2 §4 Stage 2）
///
/// **用途**: 6 域 outbox 服务各自 `main.rs` 在启动 outbox relay + metrics reporter 之后调
/// `tokio::spawn(bind_metrics_server(addr))`。
///
/// **协议**: 极简 HTTP/1.1 server, 只支持:
/// - `GET /`           → 200 OK + "ok\n"（健康检查）
/// - `GET /metrics`    → 200 OK + Prometheus text format
/// - `GET /healthz`    → 200 OK + "ok\n"（k8s liveness probe 习惯）
/// - 其他              → 404 Not Found
///
/// **依赖**: 仅 tokio, 不引入 hyper/axum（保持 shared-platform 依赖最小）。
pub async fn bind_metrics_server(addr: SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(
        target: "metrics_endpoint",
        addr = %addr,
        "metrics HTTP server listening on http://{}/metrics",
        addr
    );
    loop {
        let (mut socket, peer) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(target: "metrics_endpoint", error = %e, "accept failed");
                continue;
            }
        };
        tokio::spawn(async move {
            if let Err(e) = handle_request(&mut socket).await {
                tracing::debug!(
                    target: "metrics_endpoint",
                    peer = %peer,
                    error = %e,
                    "request handler error"
                );
            }
        });
    }
}

/// 处理单次 HTTP 请求（极简 parser，仅支持 GET）
async fn handle_request(socket: &mut tokio::net::TcpStream) -> std::io::Result<()> {
    // 读 request 头部 (max 8KB, 避免恶意长 header)
    let mut buf = vec![0u8; 8192];
    let mut total = 0;
    loop {
        let n = socket.read(&mut buf[total..]).await?;
        if n == 0 {
            return Ok(()); // EOF, peer closed
        }
        total += n;
        // 找到 header 终止 (\r\n\r\n) 或 buffer 满
        if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") || total >= buf.len() {
            break;
        }
    }
    let req_str = match std::str::from_utf8(&buf[..total]) {
        Ok(s) => s,
        Err(_) => {
            return write_response(socket, 400, "text/plain", "Bad Request\n").await;
        }
    };

    // 解析 request line: "GET /path HTTP/1.1"
    let mut parts = req_str.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    if method != "GET" {
        return write_response(socket, 405, "text/plain", "Method Not Allowed\n").await;
    }

    // 去掉 query string
    let path_only = path.split('?').next().unwrap_or(path);

    match path_only {
        "/" | "/healthz" => write_response(socket, 200, "text/plain", "ok\n").await,
        "/metrics" => {
            let resp = scrape_metrics();
            write_response(socket, resp.status, resp.content_type, &resp.body).await
        }
        _ => write_response(socket, 404, "text/plain", "Not Found\n").await,
    }
}

/// 写一个完整的 HTTP/1.1 响应
async fn write_response(
    socket: &mut tokio::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n",
        status = status,
        reason = reason,
        content_type = content_type,
        len = body.len()
    );
    socket.write_all(header.as_bytes()).await?;
    socket.write_all(body.as_bytes()).await?;
    socket.shutdown().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrape_metrics_returns_text() {
        let m = crate::metrics::metrics();
        m.record_http_request("test", "ping", "200");
        m.record_http_duration("test", "ping", 0.001);
        m.set_saga_state("transfer", "running", 3);
        m.set_outbox_pending("economy", "economy.transfer", 5);
        let resp = scrape_metrics();
        assert_eq!(resp.status, 200);
        assert!(resp.content_type.starts_with("text/plain"));
        // 至少包含一个 rgs_ 指标
        assert!(resp.body.contains("rgs_"), "body: {}", resp.body);
    }

    /// ULYS-100 集成 smoke test: 启动 server, 验证 /metrics + /healthz 都返回 200
    #[tokio::test]
    async fn bind_metrics_server_serves_metrics_and_health() {
        // 绑 loopback 任意端口
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener); // 释放; bind_metrics_server 重新绑

        // 后台启动 server
        let server_handle = tokio::spawn(bind_metrics_server(bound));

        // 客户端 GET /metrics
        let mut stream = tokio::net::TcpStream::connect(bound).await.unwrap();
        stream
            .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let mut buf = vec![0u8; 8192];
        let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
            .await
            .expect("timeout")
            .expect("read");
        let resp = String::from_utf8_lossy(&buf[..n]).to_string();
        assert!(resp.starts_with("HTTP/1.1 200"), "got: {}", resp);
        assert!(resp.contains("Content-Type: text/plain"));
        assert!(
            resp.contains("rgs_"),
            "metrics body missing rgs_*: {}",
            resp
        );

        // 客户端 GET /healthz
        let mut stream = tokio::net::TcpStream::connect(bound).await.unwrap();
        stream
            .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
            .await
            .expect("timeout")
            .expect("read");
        let resp = String::from_utf8_lossy(&buf[..n]).to_string();
        assert!(resp.starts_with("HTTP/1.1 200"), "got: {}", resp);
        assert!(resp.contains("ok"), "healthz body: {}", resp);

        // 客户端 GET /unknown → 404
        let mut stream = tokio::net::TcpStream::connect(bound).await.unwrap();
        stream
            .write_all(b"GET /unknown HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
            .await
            .expect("timeout")
            .expect("read");
        let resp = String::from_utf8_lossy(&buf[..n]).to_string();
        assert!(resp.starts_with("HTTP/1.1 404"), "got: {}", resp);

        // 关闭 server
        server_handle.abort();
    }
}
