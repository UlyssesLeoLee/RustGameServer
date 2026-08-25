//! HTTP Range 客户端（per SPEC-DTL-041 v0.2 §3 + RFC 7233）
//!
//! 全部响应路径：206 Partial Content / 416 Range Not Satisfiable / 200 OK / 429 / 503
//! 强制 `If-Range: <ETag>`（FR-CDN-074，**不**接受 Last-Modified）
//!
//! 本模块为 IT 测试所需的最小可编译 stub；完整 RangeClient（HEAD/Range 流式
//! 接收 + ETag 验证 + retry/backoff）在 WF-1-2065 worktree 实现。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Range 请求（HTTP/1.1 RFC 7233）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeRequest {
    pub url: String,
    pub etag: String,
    pub start: u64,
    pub end_inclusive: u64,
    pub timeout: Duration,
}

/// Range 响应
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeResponse {
    /// 206 Partial Content —— Range 命中
    PartialContent { etag: String, body: Vec<u8> },
    /// 416 Range Not Satisfiable —— 范围越界（重置或 ETag 变更触发全量重传）
    RangeNotSatisfiable,
    /// 200 OK —— 整文件（ETag 变更导致服务端忽略 Range）
    FullContent { etag: String, body: Vec<u8> },
    /// 429 Too Many Requests —— 背压
    TooManyRequests { retry_after: Option<Duration> },
    /// 503 Service Unavailable —— 服务端临时不可用
    ServiceUnavailable { retry_after: Option<Duration> },
}

/// Range 客户端桩
#[derive(Debug, Clone, Default)]
pub struct RangeClient {
    pub max_retries: u32,
    pub initial_backoff: Duration,
}

impl RangeClient {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
        }
    }

    /// 同步发送 Range 请求的占位实现（IT 测试不会实际调用，由 wiremock 接管）
    pub async fn send(&self, _req: RangeRequest) -> Result<RangeResponse, crate::error::DownloadError> {
        // 真实实现由 WF-1-2065 worktree 交付；本 worktree 仅提供 trait
        // 形态让 IT 编译通过。返回 Network error 提示调用方用 wiremock mock。
        Err(crate::error::DownloadError(
            crate::error::DownloadErrorKind::Range(
                "RangeClient::send 是 IT 测试 stub —— 真实实现在 WF-1-2065 worktree".into(),
            ),
        ))
    }

    /// HEAD 请求占位
    pub async fn head(
        &self,
        _url: &str,
        _timeout: Duration,
    ) -> Result<(String, u64), crate::error::DownloadError> {
        Err(crate::error::DownloadError(
            crate::error::DownloadErrorKind::Range(
                "RangeClient::head 是 IT 测试 stub".into(),
            ),
        ))
    }
}
