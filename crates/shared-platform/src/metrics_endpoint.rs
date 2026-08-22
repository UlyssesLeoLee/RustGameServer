//! Prometheus metrics HTTP endpoint（per ARC-051 观测）
//!
//! 54.13 实化：scrape_metrics() 返回 (status, body) 简单 HTTP 响应
//!
//! 设计：
//! - 简化设计：返回 (status_code, content_type, body) 元组
//! - 业务方（各域 main.rs）用 hyper / axum / warp 启动 /metrics endpoint
//! - scrape_metrics() 调 metrics::encode_to_text() 返回 Prometheus text format
//!
//! 注：完整 HTTP server（bind port）由各域 main.rs 集成 hyper 启动；
//!     shared-platform 不绑死具体 HTTP 框架

use crate::metrics::encode_to_text;

/// Metrics scrape 响应
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrape_metrics_returns_text() {
        let m = crate::metrics::metrics();
        m.record_http_request("test", "ping", "200");
        m.record_http_duration("test", "ping", 0.001);
        m.set_saga_state("transfer", "running", 3);
        m.set_outbox_pending("economy", 5);
        let resp = scrape_metrics();
        assert_eq!(resp.status, 200);
        assert!(resp.content_type.starts_with("text/plain"));
        // 至少包含一个 rgs_ 指标
        assert!(resp.body.contains("rgs_"), "body: {}", resp.body);
    }
}
