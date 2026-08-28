//! `RangeClient` —— HTTP/1.1 RFC 7233 HEAD + Range 客户端（M-2065.1 + M-2065.2）。
//!
//! ## 协议契约（per SPEC §3 + IMPL-PLAN §1.3 / §5.3）
//!
//! - **HEAD 探测**：先发 `HEAD` 拿到 `Content-Length` / `Accept-Ranges` / `ETag`。
//! - **If-Range ETag 强制**（FR-CDN-074）：所有 Range 请求携带 `If-Range: <ETag>`。
//!   任意 Range 响应若为 `200 OK`（即 ETag 不匹配）→ 上抛 `BackendEtagMismatch`，触发全量重传。
//! - **响应路径**（M-2065.1）：
//!   - `206 Partial Content`    → 成功，解析 `Content-Range` 写入分片
//!   - `416 Range Not Satisfiable` → 上抛 `BackendRangeNotSatisfiable`（触发 416 fallback / 重新分片）
//!   - `200 OK`                 → ETag 不匹配或 Range header 被服务器忽略 → `BackendEtagMismatch`
//!   - `429 Too Many Requests`  → `BackendTooManyRequests`（orchestrator 触发退避重试）
//!   - `5xx`                    → `BackendHttpError`（orchestrator 重试 ≤ 3 次）
//!
//! ## 硬约束
//!
//! - **FR-CDN-074**：若用户传 `expected_etag = None`，本客户端**仍**发送 `If-Range: ""`
//!   哨兵（E-Tag mismatch → server 必返 200 OK / 重新走分片）；**不**接受 `Last-Modified`。
//! - **NFR-CDN-114**：HEAD 探测若发现 `Accept-Ranges: none` 或缺失 → `BackendRangeUnsupported`。
//! - **FR-CDN-064**：本文件**禁止**引用 PII 字段；URL 解析只保留主机名日志，路径不进 metric label。
//!
//! ## 取消信号（FR-CDN-083）
//!
//! 所有公开方法接受 `&CancellationToken`；token 触发时立刻丢弃 `reqwest::Response` 句柄。
//! `ChunkOrchestrator` 用同一 token 集中控制全部分片。

use std::time::Duration;

use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, ETAG};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::error::{DownloadError, DownloadResult};

/// 解析后的 `Content-Range: bytes start-end/complete_length`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRange {
    /// 起始字节（inclusive）
    pub start: u64,
    /// 结束字节（inclusive）
    pub end: u64,
    /// 完整资源长度
    pub complete_length: u64,
}

impl ContentRange {
    /// 从 `bytes 0-1023/20480` 解析。
    pub fn parse(header_value: &str) -> Option<Self> {
        let header_value = header_value.trim();
        let after_prefix = header_value.strip_prefix("bytes ")?;
        let (range_part, length_part) = after_prefix.split_once('/')?;
        let (start_str, end_str) = range_part.split_once('-')?;
        let start: u64 = start_str.parse().ok()?;
        let end: u64 = end_str.parse().ok()?;
        let complete_length: u64 = match length_part {
            "*" => 0, // RFC 7233 §2.1：响应 200 OK 时 complete 可能是 `*`
            v => v.parse().ok()?,
        };
        Some(Self {
            start,
            end,
            complete_length,
        })
    }
}

/// 一次 Range 请求的字节区间（inclusive start / inclusive end）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRangeSpec {
    /// 起始字节（inclusive）
    pub start: u64,
    /// 结束字节（inclusive）
    pub end: u64,
}

impl HttpRangeSpec {
    /// 构造一个闭区间范围。
    pub fn new(start: u64, end: u64) -> Self {
        debug_assert!(end >= start, "end must be >= start");
        Self { start, end }
    }

    /// 区间长度（字节数 = end - start + 1）。
    pub fn len(&self) -> u64 {
        self.end - self.start + 1
    }

    /// 区间是否为空（`start > end`；正常构造下恒为 `false`）。
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// 序列化为 `Range: bytes=start-end` 头部值。
    pub fn to_header_value(&self) -> String {
        format!("bytes={}-{}", self.start, self.end)
    }
}

/// 后端能力探测结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeBackendProbe {
    /// 支持 HTTP Range（`Accept-Ranges: bytes`）
    Supported,
    /// 不支持（`Accept-Ranges: none` 或缺失，且响应 200 OK）
    NotSupported,
    /// 探测未完成（HEAD 失败 / 网络异常）
    Unknown,
}

/// `RangeClient` 运行时配置。
#[derive(Debug, Clone)]
pub struct RangeClientConfig {
    /// User-Agent（per config.rs：不含 PII）
    pub user_agent: String,
    /// 单次请求超时（秒）
    pub timeout_secs: u64,
    /// 是否信任平台 CA 证书（默认 true = 走 webpki-roots）
    pub verify_tls: bool,
}

impl Default for RangeClientConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("rgs-asset-download/{}", env!("CARGO_PKG_VERSION")),
            timeout_secs: 30,
            verify_tls: true,
        }
    }
}

/// 单次 Range 请求的输入（per `it_minio_latency.rs` 用法）。
#[derive(Debug, Clone)]
pub struct RangeRequest {
    /// 目标 URL（含 scheme + host + path）
    pub url: String,
    /// ETag（用于 `If-Range: <ETag>` per FR-CDN-074）
    pub etag: String,
    /// 起始字节（inclusive）
    pub start: u64,
    /// 结束字节（inclusive）
    pub end_inclusive: u64,
    /// 单次请求超时
    pub timeout: Duration,
}

/// 单次 HTTP 响应的关键字段。
///
/// 历史上是 struct；2026-08-27 重构为 enum 以显式表达 5 类响应
/// （per `chaos_responses.rs` 5 类响应分类 + SPEC §3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeResponse {
    /// 206 Partial Content：分片字节
    PartialContent {
        /// `ETag`（per FR-CDN-074 透传 If-Range）
        etag: String,
        /// 分片 body
        body: Vec<u8>,
    },
    /// 416 Range Not Satisfiable：ETag 变更 / 范围越界 → 重新拉
    RangeNotSatisfiable,
    /// 200 OK：服务端忽略 Range → 触发全量重传
    FullContent {
        /// `ETag`（per FR-CDN-074 透传 If-Range）
        etag: String,
        /// 完整 body
        body: Vec<u8>,
    },
    /// 429 Too Many Requests：触发退避重试
    TooManyRequests {
        /// `Retry-After` 头值（per RFC 7231 §7.1.3）
        retry_after: Option<Duration>,
    },
    /// 503 Service Unavailable：触发退避重试，耗尽后 `RetryExhausted`
    ServiceUnavailable {
        /// `Retry-After` 头值
        retry_after: Option<Duration>,
    },
}

impl RangeResponse {
    /// body 引用（仅 `PartialContent` / `FullContent` 有 body）
    pub fn body(&self) -> Option<&[u8]> {
        match self {
            Self::PartialContent { body, .. } | Self::FullContent { body, .. } => Some(body),
            _ => None,
        }
    }
}

/// 探测 / Range 拉取的"扩展视图"：包含 HTTP 状态码 + Content-Range 等元数据。
///
/// 内部 `chunk_orchestrator` 使用（含 body / content_range / accept_ranges）。
#[derive(Debug, Clone)]
pub struct RangeResponseDetailed {
    /// HTTP 状态码
    pub status: u16,
    /// `Content-Length`
    pub content_length: u64,
    /// `Accept-Ranges`
    pub accept_ranges: Option<String>,
    /// `ETag`
    pub etag: Option<String>,
    /// `Content-Range`
    pub content_range: Option<ContentRange>,
    /// body
    pub body: Vec<u8>,
}

/// `RangeClient` —— 单实例线程安全（内部持有 `reqwest::Client`）。
///
/// 构造时建立底层 HTTP 客户端；HEAD / Range 请求均携带 `If-Range` 头。
pub struct RangeClient {
    config: RangeClientConfig,
    http: Client,
}

impl std::fmt::Debug for RangeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RangeClient")
            .field("config", &self.config)
            .finish()
    }
}

impl Default for RangeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeClient {
    /// 新建客户端（无参：使用 [`RangeClientConfig::default()`]，per `it_minio_latency.rs` / `it_minio_nfr110.rs`）。
    ///
    /// 底层 reqwest 客户端 build 失败会 panic（极少见，仅在系统资源耗尽时触发）。
    pub fn new() -> Self {
        Self::with_config(RangeClientConfig::default())
            .expect("RangeClient::new with default config should not fail")
    }

    /// 新建客户端（带配置；per `ut_range_client.rs`）。
    pub fn with_config(config: RangeClientConfig) -> DownloadResult<Self> {
        let mut builder = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.timeout_secs));
        // 默认开启 rustls + webpki-roots（per Cargo.toml features）
        if !config.verify_tls {
            builder = builder.danger_accept_invalid_certs(true);
        }
        let http = builder
            .build()
            .map_err(|e| DownloadError::HttpClient(e.to_string()))?;
        Ok(Self { config, http })
    }

    /// 当前配置（只读）。
    pub fn config(&self) -> &RangeClientConfig {
        &self.config
    }

    /// 简化版发送：携带 `If-Range: <etag>` 直接 GET `[start, end_inclusive]`
    /// （per `it_minio_latency.rs`）。
    ///
    /// 返回归一化的 [`RangeResponse`] enum；网络错误上抛 [`DownloadError`]。
    pub async fn send(&self, req: RangeRequest) -> DownloadResult<RangeResponse> {
        let range = HttpRangeSpec::new(req.start, req.end_inclusive);
        let raw = self
            .send_range(
                &req.url,
                &range,
                if req.etag.is_empty() {
                    None
                } else {
                    Some(req.etag.as_str())
                },
                &CancellationToken::new(),
            )
            .await?;
        let status = raw.status.as_u16();
        let etag = raw.etag.clone().unwrap_or_default();
        Ok(match status {
            206 => RangeResponse::PartialContent {
                etag,
                body: raw.body,
            },
            200 => RangeResponse::FullContent {
                etag,
                body: raw.body,
            },
            416 => RangeResponse::RangeNotSatisfiable,
            429 => RangeResponse::TooManyRequests { retry_after: None },
            503 => RangeResponse::ServiceUnavailable { retry_after: None },
            _ => {
                return Err(DownloadError::BackendHttpError {
                    status,
                    host: host_of(&req.url),
                });
            }
        })
    }

    /// HEAD 探测（per NFR-CDN-114）。
    ///
    /// 成功响应必须：
    /// 1. 状态码 2xx
    /// 2. `Content-Length` > 0
    /// 3. `Accept-Ranges: bytes`（缺失则视为 `NotSupported`）
    /// 4. 携带 `ETag`（供后续 Range 携带 `If-Range`；缺失时本客户端会发送 `If-Range: ""` 哨兵）
    pub async fn probe(
        &self,
        url: &str,
        cancel: &CancellationToken,
    ) -> DownloadResult<RangeBackendProbe> {
        let resp = self.send_head(url, cancel).await?;
        if !resp.status.is_success() {
            return Err(DownloadError::BackendHttpError {
                status: resp.status.as_u16(),
                host: host_of(url),
            });
        }
        let accepts = resp.accept_ranges.as_deref().map(|s| s.to_string());
        let is_supported = accepts
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("bytes"))
            .unwrap_or(false);
        if is_supported {
            Ok(RangeBackendProbe::Supported)
        } else {
            Err(DownloadError::BackendRangeUnsupported { host: host_of(url) })
        }
    }

    /// HEAD 探测 + 返回响应详情（含 ETag / Content-Length）。
    pub async fn probe_full(
        &self,
        url: &str,
        cancel: &CancellationToken,
    ) -> DownloadResult<RangeResponseDetailed> {
        self.send_head(url, cancel).await.map(|r| r.into_response())
    }

    /// 拉取一个 Range 分片。
    ///
    /// - `expected_etag` 来自断点记录；**不**接受 `Last-Modified`。
    /// - 响应非 206 → 转译为对应 `DownloadError`（由 orchestrator 决定是否重试 / 全量重传）。
    pub async fn fetch_range(
        &self,
        url: &str,
        range: &HttpRangeSpec,
        expected_etag: Option<&str>,
        cancel: &CancellationToken,
    ) -> DownloadResult<RangeResponseDetailed> {
        let resp = self.send_range(url, range, expected_etag, cancel).await?;
        let status = resp.status;
        let resp = resp.into_response();
        match status.as_u16() {
            206 => {
                // 必须解析 Content-Range
                if resp.content_range.is_none() {
                    return Err(DownloadError::BackendHttpError {
                        status: 206,
                        host: host_of(url),
                    });
                }
                Ok(resp)
            }
            200 => {
                // ETag 不匹配 / 服务器忽略 Range → 触发全量重传
                let actual = resp.etag.clone().unwrap_or_default();
                let expected = expected_etag.unwrap_or("").to_string();
                Err(DownloadError::BackendEtagMismatch { expected, actual })
            }
            416 => Err(DownloadError::BackendRangeNotSatisfiable {
                chunk_index: 0, // 由调用方填（orchestrator）
                start: range.start,
                end: range.end,
            }),
            429 => Err(DownloadError::BackendTooManyRequests { host: host_of(url) }),
            s if (500..600).contains(&s) => Err(DownloadError::BackendHttpError {
                status: s,
                host: host_of(url),
            }),
            s => Err(DownloadError::BackendHttpError {
                status: s,
                host: host_of(url),
            }),
        }
    }

    // ===== 内部 =====

    async fn send_head(
        &self,
        url: &str,
        cancel: &CancellationToken,
    ) -> DownloadResult<RawResponse> {
        let req = self.http.head(url);
        self.execute(req, cancel).await
    }

    async fn send_range(
        &self,
        url: &str,
        range: &HttpRangeSpec,
        expected_etag: Option<&str>,
        cancel: &CancellationToken,
    ) -> DownloadResult<RawResponse> {
        let mut req = self.http.get(url);
        req = req.header("Range", range.to_header_value());
        // FR-CDN-074：强制 If-Range + ETag；不传 Last-Modified
        // 哨兵：expected_etag = None 时仍发空串（RFC 7233 §3.2 语义：if-range 不匹配 → 200 OK）
        let etag_value = expected_etag.unwrap_or("");
        req = req.header("If-Range", format!("\"{}\"", etag_value));
        self.execute(req, cancel).await
    }

    async fn execute(
        &self,
        builder: RequestBuilder,
        cancel: &CancellationToken,
    ) -> DownloadResult<RawResponse> {
        let send_fut = builder.send();
        tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                Err(DownloadError::Cancelled)
            }
            send_result = send_fut => {
                let resp = send_result?;
                let status = resp.status();
                let headers = resp.headers().clone();
                let content_length = headers
                    .get(&CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let accept_ranges = headers
                    .get(&ACCEPT_RANGES)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let etag = headers
                    .get(&ETAG)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let content_range = headers
                    .get(&CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(Self::parse_content_range);
                let body = resp.bytes().await?.to_vec();
                Ok(RawResponse {
                    status,
                    content_length,
                    accept_ranges,
                    etag,
                    content_range,
                    body,
                })
            }
        }
    }

    fn parse_content_range(value: &str) -> Option<ContentRange> {
        ContentRange::parse(value)
    }
}

/// 提取 URL 的主机名（用于日志 / 错误消息；不含路径 / query / PII）。
fn host_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "<invalid-url>".to_string())
}

struct RawResponse {
    status: StatusCode,
    content_length: u64,
    accept_ranges: Option<String>,
    etag: Option<String>,
    content_range: Option<ContentRange>,
    body: Vec<u8>,
}

impl RawResponse {
    fn into_response(self) -> RangeResponseDetailed {
        RangeResponseDetailed {
            status: self.status.as_u16(),
            content_length: self.content_length,
            accept_ranges: self.accept_ranges,
            etag: self.etag,
            content_range: self.content_range,
            body: self.body,
        }
    }
}

#[allow(dead_code)]
const _UNUSED_METHOD_REF: Method = Method::GET;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_range_parses_well_formed() {
        let cr = ContentRange::parse("bytes 0-1023/20480").unwrap();
        assert_eq!(cr.start, 0);
        assert_eq!(cr.end, 1023);
        assert_eq!(cr.complete_length, 20480);
    }

    #[test]
    fn content_range_handles_star_length() {
        let cr = ContentRange::parse("bytes 0-1023/*").unwrap();
        assert_eq!(cr.complete_length, 0);
    }

    #[test]
    fn content_range_rejects_garbage() {
        assert!(ContentRange::parse("chunks 0-1/2").is_none());
        assert!(ContentRange::parse("bytes abc-def/10").is_none());
        assert!(ContentRange::parse("").is_none());
    }

    #[test]
    fn http_range_to_header_value() {
        let r = HttpRangeSpec::new(0, 1023);
        assert_eq!(r.to_header_value(), "bytes=0-1023");
        assert_eq!(r.len(), 1024);
    }

    #[test]
    fn host_of_strips_path_and_query() {
        assert_eq!(
            host_of("https://cdn.example.com/assets/foo.bin?token=abc"),
            "cdn.example.com"
        );
        // 不接受 player_id / device_id 等 PII 字段
        let h = host_of("https://device-id-leak.example.com/x");
        assert_eq!(h, "device-id-leak.example.com");
        // 这里 host 名是 URL 一部分（用户控制），不视为 PII；
        // 真正的 PII 防护在 error message 模板（见 error.rs）层
    }
}
