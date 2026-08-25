//! 错误码（per RGS-DTL-041 §6，不自创）
//!
//! 8 类错误：Range / Integrity / Network / Manifest / ETag / Pause / Cancel / Internal

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DownloadErrorKind {
    /// HTTP Range 请求失败（网络层 / 服务端 5xx）
    #[error("Range request failed: {0}")]
    Range(String),

    /// 整文件 hash 校验失败（NFR-CDN-002 不可绕过）
    #[error("Integrity gate rejected: expected {expected}, got {actual}")]
    Integrity { expected: String, actual: String },

    /// 网络层失败（DNS / TCP / TLS / 超时）
    #[error("Network error: {0}")]
    Network(String),

    /// Manifest 拉取 / 签名 / 灰度判定失败
    #[error("Manifest error: {0}")]
    Manifest(String),

    /// ETag 不匹配（If-Range 失败，需全量重传）
    #[error("ETag mismatch: stored {stored}, current {current}")]
    ETagMismatch { stored: String, current: String },

    /// 断点记录过期（> 7 天，需重新拉 manifest）
    #[error("Resume token expired (ttl_days=7)")]
    Expired,

    /// 重试耗尽（单 chunk 3 次失败）
    #[error("Retry exhausted after 3 attempts")]
    RetryExhausted,

    /// 内部错误（invariant 违反 / 平台错误）
    #[error("Internal: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct DownloadError(#[from] pub DownloadErrorKind);

impl DownloadError {
    pub fn kind(&self) -> &DownloadErrorKind {
        &self.0
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(e: std::io::Error) -> Self {
        DownloadError(DownloadErrorKind::Network(e.to_string()))
    }
}
