//! 错误码（per RGS-DTL-041 §6 + IMPL-PLAN §3 + SPEC §3）。
//!
//! 设计原则：
//! - **不**自创额外枚举值（SPEC §3 硬约束）；命名严格对齐 DTL §6。
//! - 错误统一在 `DownloadError` 内；`From` impl 用于跨模块传播。
//! - 错误中**不**含 PII（FR-CDN-064）：message 中只携带状态码、URL 主机名（不含 query）、chunk
//!   范围、ETag 摘要前缀；禁止包含 player_id / device_id / email / ip / mac。
//!
//! 主要错误族：
//! - `Backend*`  ：HTTP/Range 协议层
//! - `Integrity*`：整文件 hash 校验
//! - `Resume*`   ：断点 token 读写
//! - `State*`    ：状态机非法转移
//! - `Config*`   ：参数非法
//! - `Io*`       ：文件系统
//! - `Cancelled` / `Paused`  主动操作成功终止

use thiserror::Error;

use crate::state_machine::TransitionError;

/// 库内统一 `Result` 别名。
pub type DownloadResult<T> = Result<T, DownloadError>;

/// 旧名兼容别名（per `rgs_asset_download::AssetDownloadError` 测试 import）
pub type AssetDownloadError = DownloadError;

/// 错误码（与 DTL §6 一一对应；命名上不增不减）。
#[derive(Debug, Error)]
pub enum DownloadError {
    /// 后端不支持 HTTP Range（NFR-CDN-114 门禁；HEAD 探测 200 OK + 无 `Accept-Ranges`）
    #[error("backend does not support HTTP Range (NFR-CDN-114): host={host}")]
    BackendRangeUnsupported {
        /// URL 主机名（已脱敏，不含 query / path）
        host: String,
    },

    /// 后端 ETag 与本地断点不匹配（FR-CDN-074；触发全量重传）
    #[error("ETag mismatch: expected={expected}, actual={actual}")]
    BackendEtagMismatch {
        /// 期望 ETag（来自断点记录）
        expected: String,
        /// 实际 ETag（来自服务器响应）
        actual: String,
    },

    /// 后端返回 416 Range Not Satisfiable（per RFC 7233）
    #[error("range not satisfiable (416): chunk={chunk_index}, range={start}-{end}")]
    BackendRangeNotSatisfiable {
        /// 分片索引
        chunk_index: u64,
        /// 起始字节
        start: u64,
        /// 结束字节（inclusive）
        end: u64,
    },

    /// 后端返回 429 Too Many Requests（触发退避后重试；耗尽后升级）
    #[error("too many requests (429) from backend: host={host}")]
    BackendTooManyRequests {
        /// URL 主机名
        host: String,
    },

    /// 后端返回 5xx 或其他非 2xx/4xx 类错误
    #[error("backend HTTP error: status={status}, host={host}")]
    BackendHttpError {
        /// HTTP 状态码
        status: u16,
        /// URL 主机名
        host: String,
    },

    /// 整文件 SHA-256 与 manifest 期望值不符（NFR-CDN-002 硬约束）
    #[error("integrity gate failed: expected_sha256={expected}, actual_sha256={actual}")]
    IntegrityMismatch {
        /// 期望 SHA-256
        expected: String,
        /// 实际 SHA-256
        actual: String,
    },

    /// 后端声明的 `Content-Length` 与已落盘字节不一致
    #[error("integrity byte count mismatch: expected={expected}, actual={actual}")]
    IntegritySizeMismatch {
        /// 期望字节数
        expected: u64,
        /// 实际字节数
        actual: u64,
    },

    /// 断点记录找不到（resume 时）
    #[error("resume token not found: token_id={token_id}")]
    ResumeTokenNotFound {
        /// 断点 ID（UUID v4，不含 PII）
        token_id: String,
    },

    /// 断点记录解析失败（schema 损坏 / 版本不兼容）
    #[error("resume token corrupt: token_id={token_id}, reason={reason}")]
    ResumeTokenCorrupt {
        /// 断点 ID
        token_id: String,
        /// 失败原因（已脱敏）
        reason: String,
    },

    /// 断点记录已过期（超过 TTL）
    #[error("resume token expired: token_id={token_id}, expired_at={expired_at}")]
    ResumeTokenExpired {
        /// 断点 ID
        token_id: String,
        /// 过期时间（ISO-8601 字符串）
        expired_at: String,
    },

    /// 状态机非法转移（per M-2064.1 转移表）
    #[error("illegal state transition: from={from}, via={via}")]
    StateIllegalTransition {
        /// 起始状态
        from: String,
        /// 触发的动作 / 目标状态
        via: String,
    },

    /// 状态机内部错误（被取消、暂停等）
    #[error("state machine invariant violated: detail={detail}")]
    StateInvariant {
        /// 详情
        detail: String,
    },

    /// 配置非法
    #[error("invalid config: field={field}, reason={reason}")]
    ConfigInvalid {
        /// 字段名
        field: String,
        /// 原因
        reason: String,
    },

    /// 单 chunk 重试耗尽（per SPEC §3：默认 3 次）
    #[error("retry exhausted: chunk={chunk_index}, attempts={attempts}")]
    RetryExhausted {
        /// 分片索引
        chunk_index: u64,
        /// 实际尝试次数
        attempts: u32,
    },

    /// 主动取消（FR-CDN-083）
    #[error("download cancelled by user signal")]
    Cancelled,

    /// 主动暂停（per SPEC §3）
    #[error("download paused by user signal")]
    Paused,

    /// IO 错误（落盘 / sparse 预分配 / SQLite 等）
    #[error("io error: path={path}, kind={kind}")]
    Io {
        /// 文件路径（已脱敏：不含 player_id / device_id）
        path: String,
        /// 错误种类
        kind: String,
    },

    /// 平台预分配不支持（unreachable on supported targets）
    #[error("platform preallocate unsupported: target_os={target_os}")]
    PlatformPreallocateUnsupported {
        /// 目标 OS
        target_os: String,
    },

    /// reqwest / rustls 底层错误
    #[error("http client error: {0}")]
    HttpClient(String),

    /// URL 解析失败
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    /// Store IO 错误（per `AssetDownloadError::StoreIoError` 测试 import）
    #[error("store io error: path={path}, cause={cause}")]
    StoreIoError {
        /// 失败路径
        path: String,
        /// 底层原因
        cause: String,
    },

    /// Store 后端错误（SQLite / generic backend）
    #[error("store backend error: {0}")]
    StoreBackendError(String),

    /// Store 序列化错误
    #[error("store serialization error: {0}")]
    StoreSerializationError(String),
}

impl DownloadError {
    /// 错误分类（用于 metrics 标签 / 日志分级）。
    pub fn category(&self) -> &'static str {
        match self {
            Self::BackendRangeUnsupported { .. }
            | Self::BackendEtagMismatch { .. }
            | Self::BackendRangeNotSatisfiable { .. }
            | Self::BackendTooManyRequests { .. }
            | Self::BackendHttpError { .. } => "backend",
            Self::IntegrityMismatch { .. } | Self::IntegritySizeMismatch { .. } => "integrity",
            Self::ResumeTokenNotFound { .. }
            | Self::ResumeTokenCorrupt { .. }
            | Self::ResumeTokenExpired { .. } => "resume_token",
            Self::StateIllegalTransition { .. } | Self::StateInvariant { .. } => "state_machine",
            Self::ConfigInvalid { .. } => "config",
            Self::RetryExhausted { .. } => "retry",
            Self::Cancelled => "cancelled",
            Self::Paused => "paused",
            Self::Io { .. } | Self::PlatformPreallocateUnsupported { .. } => "io",
            Self::HttpClient(_) | Self::InvalidUrl(_) => "client",
            Self::StoreIoError { .. }
            | Self::StoreBackendError(_)
            | Self::StoreSerializationError(_) => "store",
        }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpClient(e.to_string())
    }
}

impl From<url::ParseError> for DownloadError {
    fn from(e: url::ParseError) -> Self {
        Self::InvalidUrl(e.to_string())
    }
}

impl From<TransitionError> for DownloadError {
    fn from(e: TransitionError) -> Self {
        Self::StateIllegalTransition {
            from: e.from.to_string(),
            via: e.event.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_categories_are_stable() {
        // 类别字符串在 metrics 标签中使用，必须稳定
        assert_eq!(
            DownloadError::Cancelled.category(),
            "cancelled"
        );
        assert_eq!(
            DownloadError::IntegrityMismatch {
                expected: "a".into(),
                actual: "b".into(),
            }
            .category(),
            "integrity"
        );
    }

    #[test]
    fn error_message_contains_no_pii_field_names() {
        // 错误消息模板不应引用 PII 字段名（FR-CDN-064 防御性）
        let e = DownloadError::BackendHttpError {
            status: 503,
            host: "cdn.example.com".into(),
        };
        let s = e.to_string();
        assert!(!s.contains("player_id"));
        assert!(!s.contains("device_id"));
        assert!(!s.contains("email"));
    }
}
