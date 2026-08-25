//! 错误码定义（per RGS-DTL-041 §6 + RGS-SPEC-DTL-041 §3）
//!
//! 错误码严格按 DTL §6 落地，不自创额外枚举值。
//!
//! ## 错误码分类
//!
//! - `TokenNotFound` / `TokenExpired` / `TokenCorrupted` / `TokenDuplicate`：断点记录错误
//! - `IllegalStateTransition` / `AlreadyInProgress` / `AlreadyCompleted`：状态机错误
//! - `BackendError` / `RangeNotSupported` / `EtagMismatch` / `RangeRequestFailed`：HTTP 后端错误
//! - `IntegrityFailed` / `ChunkHashMismatch`：整文件校验错误（NFR-CDN-002 不可绕过）
//! - `InFlightCancelFailed`：in_flight 取消错误（FR-CDN-083）
//! - `StoreIoError` / `StoreSerializationError` / `StoreBackendError`：store 错误
//! - `InvalidConfig` / `InvalidArgument` / `InsufficientDiskSpace` / `PermissionDenied`：配置/环境错误
//! - `RetryExhausted`：重试耗尽

use thiserror::Error;

/// 资产下载错误（顶层错误枚举）
#[derive(Debug, Error)]
pub enum AssetDownloadError {
    // ---- 断点记录错误（per RGS-DTL-041 §6） ----
    /// 找不到指定 token_id 的断点记录
    #[error("resume token not found: token_id={0}")]
    TokenNotFound(String),

    /// 断点已过期（>7 天，per RGS-SPEC-DTL-041 §8 resume_token_ttl_days）
    #[error("resume token expired: token_id={0}, expired_at={1}")]
    TokenExpired(String, String),

    /// 断点记录反序列化失败（损坏）
    #[error("resume token corrupted: token_id={0}, cause={1}")]
    TokenCorrupted(String, String),

    /// 重复的 token_id（唯一索引冲突）
    #[error("resume token duplicate: token_id={0}")]
    TokenDuplicate(String),

    // ---- 状态机错误 ----
    /// 非法状态转移（per 状态机转移表）
    #[error("illegal state transition: from={from}, event={event}")]
    IllegalStateTransition {
        /// 当前状态
        from: String,
        /// 触发事件
        event: String,
    },

    /// 同一 token_id 已在进行中
    #[error("download already in progress: token_id={0}")]
    AlreadyInProgress(String),

    /// 已完成，无法再次下载
    #[error("download already completed: token_id={0}")]
    AlreadyCompleted(String),

    // ---- HTTP 后端错误 ----
    /// 后端通用错误
    #[error("backend error: status={status}, body={body}")]
    BackendError {
        /// HTTP 状态码
        status: u16,
        /// 响应体摘要
        body: String,
    },

    /// 后端不支持 HTTP Range（NFR-CDN-114 门禁）
    #[error("backend does not support HTTP Range: url={0}")]
    RangeNotSupported(String),

    /// ETag 不匹配（If-Range 触发全量重传）
    #[error("ETag mismatch: expected={expected}, actual={actual}")]
    EtagMismatch {
        /// 客户端记录的 ETag
        expected: String,
        /// 服务端返回的 ETag
        actual: String,
    },

    /// Range 请求失败（416 / 503 / 网络错误）
    #[error("Range request failed: status={status}, reason={reason}")]
    RangeRequestFailed {
        /// HTTP 状态码
        status: u16,
        /// 失败原因
        reason: String,
    },

    // ---- 整文件校验错误（NFR-CDN-002 不可绕过） ----
    /// 整文件 SHA-256 校验失败
    #[error("integrity check failed: expected={expected}, actual={actual}")]
    IntegrityFailed {
        /// manifest 给定的 hash
        expected: String,
        /// 实测 hash
        actual: String,
    },

    /// 分块 hash 不一致（完整性前置检查，仅供参考；不绕过整文件校验）
    #[error("chunk hash mismatch: chunk_index={0}, reason={1}")]
    ChunkHashMismatch(u32, String),

    // ---- in_flight 取消错误（FR-CDN-083） ----
    /// 暂停时无法取消 in_flight Range 请求
    #[error("failed to cancel in_flight request: token_id={0}, reason={1}")]
    InFlightCancelFailed(String, String),

    // ---- Store 错误 ----
    /// store IO 错误（文件系统 / SQLite IO）
    #[error("store I/O error: path={path}, cause={cause}")]
    StoreIoError {
        /// 相关路径
        path: String,
        /// 底层错误描述
        cause: String,
    },

    /// store 序列化错误（JSON / SQLite BLOB 序列化）
    #[error("store serialization error: source={0}")]
    StoreSerializationError(String),

    /// store 后端错误（SQLite 返回非 IO / 非序列化的错误）
    #[error("store backend error: source={0}")]
    StoreBackendError(String),

    // ---- 配置 / 环境错误 ----
    /// 配置非法（如 chunk_size=0）
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// 非法参数
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// 磁盘空间不足
    #[error("insufficient disk space: required={required_bytes}, available={available_bytes}")]
    InsufficientDiskSpace {
        /// 所需字节
        required_bytes: u64,
        /// 可用字节
        available_bytes: u64,
    },

    /// 权限不足（如 Windows SeManageVolumePrivilege 缺失）
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    // ---- 重试 ----
    /// 重试耗尽（per SPEC §3 单 chunk 3 次重试 + 指数退避）
    #[error("retry exhausted: token_id={token_id}, attempts={attempts}")]
    RetryExhausted {
        /// 相关 token_id
        token_id: String,
        /// 实际尝试次数
        attempts: u32,
    },
}

/// 错误码分类（用于 metrics label 等场景；非错误变体本身）
pub mod error_code {
    /// 错误码常量表（与 DTL §6 对齐；用于 metrics / log 标签）
    pub const TOKEN_NOT_FOUND: &str = "TOKEN_NOT_FOUND";
    /// token 过期
    pub const TOKEN_EXPIRED: &str = "TOKEN_EXPIRED";
    /// token 损坏
    pub const TOKEN_CORRUPTED: &str = "TOKEN_CORRUPTED";
    /// token 重复
    pub const TOKEN_DUPLICATE: &str = "TOKEN_DUPLICATE";
    /// 非法状态转移
    pub const ILLEGAL_STATE_TRANSITION: &str = "ILLEGAL_STATE_TRANSITION";
    /// 已在进行中
    pub const ALREADY_IN_PROGRESS: &str = "ALREADY_IN_PROGRESS";
    /// 已完成
    pub const ALREADY_COMPLETED: &str = "ALREADY_COMPLETED";
    /// 后端错误
    pub const BACKEND_ERROR: &str = "BACKEND_ERROR";
    /// Range 不支持
    pub const RANGE_NOT_SUPPORTED: &str = "RANGE_NOT_SUPPORTED";
    /// ETag 不匹配
    pub const ETAG_MISMATCH: &str = "ETAG_MISMATCH";
    /// Range 请求失败
    pub const RANGE_REQUEST_FAILED: &str = "RANGE_REQUEST_FAILED";
    /// 整文件校验失败（NFR-CDN-002 不可绕过）
    pub const INTEGRITY_FAILED: &str = "INTEGRITY_FAILED";
    /// chunk hash 不一致
    pub const CHUNK_HASH_MISMATCH: &str = "CHUNK_HASH_MISMATCH";
    /// in_flight 取消失败
    pub const IN_FLIGHT_CANCEL_FAILED: &str = "IN_FLIGHT_CANCEL_FAILED";
    /// store IO 错误
    pub const STORE_IO_ERROR: &str = "STORE_IO_ERROR";
    /// store 序列化错误
    pub const STORE_SERIALIZATION_ERROR: &str = "STORE_SERIALIZATION_ERROR";
    /// store 后端错误
    pub const STORE_BACKEND_ERROR: &str = "STORE_BACKEND_ERROR";
    /// 配置非法
    pub const INVALID_CONFIG: &str = "INVALID_CONFIG";
    /// 非法参数
    pub const INVALID_ARGUMENT: &str = "INVALID_ARGUMENT";
    /// 磁盘空间不足
    pub const INSUFFICIENT_DISK_SPACE: &str = "INSUFFICIENT_DISK_SPACE";
    /// 权限不足
    pub const PERMISSION_DENIED: &str = "PERMISSION_DENIED";
    /// 重试耗尽
    pub const RETRY_EXHAUSTED: &str = "RETRY_EXHAUSTED";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_does_not_leak_pii() {
        // 确保错误信息不含 PII 字段名（per FR-CDN-064）
        let err = AssetDownloadError::TokenNotFound("token-abc".to_string());
        let s = err.to_string();
        assert!(!s.contains("player_id"));
        assert!(!s.contains("device_id"));
        assert!(!s.contains("email"));
        assert!(!s.contains("ip_address"));
        assert!(!s.contains("mac_address"));
    }

    #[test]
    fn error_code_constants_match_dtl_section_6() {
        // DTL §6 错误码总览 - 抽样验证
        assert_eq!(error_code::TOKEN_NOT_FOUND, "TOKEN_NOT_FOUND");
        assert_eq!(error_code::INTEGRITY_FAILED, "INTEGRITY_FAILED");
        assert_eq!(error_code::IN_FLIGHT_CANCEL_FAILED, "IN_FLIGHT_CANCEL_FAILED");
    }
}
