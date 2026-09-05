//! web_conn stub (per 9/4 改进路线图 Phase 1 协议网关 + ADR-006 Option A)
//!
//! ## 范围
//! web_conn.erl 是闪烁之光 HTTP 入口 (per 9/4 MD §3 客户端入口), 端口 8000,
//! 处理轻量 HTTP 请求 (e.g. /login /register /gm 页面). RGS 协议网关在
//! Option A (rustler+BEAM) 模式下加载 web_conn.erl, 由 BEAM 内部处理 HTTP
//! 业务, 仅在需要 RGS 业务时走 NIF 调 7 域 gRPC.
//!
//! ## 本骨架 (Phase 1.5 stub)
//! - `WebConnConfig` 配置 (port / cookie / max_connections / backpressure)
//! - `start()` 函数签名 (Phase 1.5 真实实装: spawn 1 个 tokio task 监听 8000,
//!   简单 HTTP request → NIF bridge → 7 域 gRPC)
//! - 1 个 HTTP path 解析 helper
//! - 不绑真实 socket (Phase 1.5 走 hyper)
//!
//! ## 已知缺口
//! - web_conn.erl 真实源码未加载 (per 9/4 R2, 评估 rustler)
//! - 走 hyper 0.14 vs 1.0 决策待主会话拍板 (per 9/4 改进路线图风险 R2)
//!
//! ## 参考
//! - ADR-006 Option A (RGS 内嵌 BEAM via rustler)
//! - 9/4 MD §3 web_conn.erl 端口 8000 + zone 启动

use std::fmt;

/// web_conn HTTP 端口 (per 9/4 MD §3)
pub const WEB_CONN_PORT: u16 = 8000;

/// web_conn 配置 (Phase 1.5 stub)
#[derive(Debug, Clone)]
pub struct WebConnConfig {
    /// 监听端口 (per 9/4 MD §3, 默认 8000)
    pub port: u16,
    /// 监听地址 (per 9/4 改进路线图 127.0.0.1 only for local-side, ClusterIP for k8s)
    pub bind_addr: String,
    /// Erlang cookie (per 8/27 11:06 JST hard ban, 不打印)
    pub cookie: Vec<u8>,
    /// 最大并发连接数 (backpressure)
    pub max_connections: usize,
}

impl WebConnConfig {
    /// 默认配置 (端口 8000, 127.0.0.1, 64 连接上限)
    pub fn default_local() -> Self {
        Self {
            port: WEB_CONN_PORT,
            bind_addr: "127.0.0.1".to_string(),
            cookie: Vec::new(), // 凭据 0 打印, 走 env 注入
            max_connections: 64,
        }
    }
}

impl fmt::Display for WebConnConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 凭据 REDACTED 过滤 (per 8/27 11:06 JST hard ban)
        write!(
            f,
            "WebConnConfig {{ port={}, bind={}, max_conn={}, cookie=REDACTED(len={}) }}",
            self.port,
            self.bind_addr,
            self.max_connections,
            self.cookie.len()
        )
    }
}

/// 解析 HTTP request path (Phase 1.5 stub helper)
///
/// 真实 web_conn.erl 路径示例: /login /register /gm /api/login /heartbeat
/// Phase 1.5 stub: 简单 split + 路径提取
pub fn parse_http_path(req: &[u8]) -> Option<&str> {
    // 找第一行 "GET /path HTTP/1.1"
    let line_end = req.iter().position(|&b| b == b'\n')?;
    let line = std::str::from_utf8(&req[..line_end]).ok()?;
    // split: "GET /path HTTP/1.1" -> ["GET", "/path", "HTTP/1.1"]
    let mut parts = line.split_whitespace();
    let _method = parts.next()?;
    let path = parts.next()?;
    Some(path)
}

/// 启动 web_conn (Phase 1.5 stub)
///
/// 真实实装: `tokio::spawn(async move { bind 8000, accept, parse HTTP, NIF bridge })`
/// 当前: 校验配置, 返回 Ok stub, Phase 1.5 替换
pub async fn start(cfg: WebConnConfig) -> std::io::Result<()> {
    // Phase 1.5 stub: 校验 config, 立即返回 (不绑真实 socket)
    if cfg.cookie.len() > 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "cookie too long",
        ));
    }
    if cfg.port == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "port must be non-zero",
        ));
    }
    // 占位: Phase 1.5 替换为真实 listener
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_local() {
        let cfg = WebConnConfig::default_local();
        assert_eq!(cfg.port, 8000);
        assert_eq!(cfg.bind_addr, "127.0.0.1");
        assert_eq!(cfg.max_connections, 64);
    }

    #[test]
    fn display_redacts_cookie_value() {
        // 验证 Display 不打印 cookie 实际值 (per 8/27 11:06 JST hard ban)
        let mut cfg = WebConnConfig::default_local();
        cfg.cookie = b"super_secret_123".to_vec();
        let s = format!("{}", cfg);
        assert!(!s.contains("super_secret"), "Display must redact cookie value");
        assert!(s.contains("REDACTED"), "Display should mark cookie as redacted");
    }

    #[test]
    fn parse_http_get_path() {
        let req = b"GET /login HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let path = parse_http_path(req).expect("must parse");
        assert_eq!(path, "/login");
    }

    #[test]
    fn parse_http_post_path() {
        let req = b"POST /api/create_character HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        let path = parse_http_path(req).expect("must parse");
        assert_eq!(path, "/api/create_character");
    }

    #[test]
    fn parse_http_invalid_returns_none() {
        let req = b"\xFF\xFE invalid bytes";
        assert!(parse_http_path(req).is_none());
    }

    #[tokio::test]
    async fn start_stub_returns_ok_with_default() {
        let cfg = WebConnConfig::default_local();
        // Phase 1.5 stub: 立即返回 Ok, 不真绑
        assert!(start(cfg).await.is_ok());
    }

    #[tokio::test]
    async fn start_rejects_zero_port() {
        let mut cfg = WebConnConfig::default_local();
        cfg.port = 0;
        assert!(start(cfg).await.is_err());
    }
}
