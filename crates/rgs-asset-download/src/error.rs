//! DownloadError —— 客户端资源下载子系统的统一错误类型。
//!
//! 严格按 **RGS-DTL-041 §3.3** 的 13 个变体实现，**不自创**任何额外枚举值；
//! 任何新增变体必须先回写 DTL 并通过 DD Review（per RGS-SPEC-DTL-041 v0.2 §3 + §7 DoD）。
//!
//! # 硬约束绑定
//!
//! - **NFR-CDN-002（整文件校验不可绕过）**：本错误类型不暴露任何跳过/旁路
//!   `IntegrityGate` 的变体；上层调用方必须经 `integrity_gate.rs`（M-2065.5）才能
//!   完成下载。代码评审 grep `skip_integrity|bypass_integrity` 期望为空。
//! - **FR-CDN-064（断点记录不含 PII）**：错误变体的所有字段均为技术元数据
//!   （status code / byte count / ETag / DateTime / attempt 计数），不含
//!   `player_id` / `device_id` / `ip` / `mac` / `email`。
//! - **FR-CDN-074（If-Range ETag 强制）**：`ETagChanged` 变体携带 old/new ETag 供
//!   上层判定全量重传（不做 `Last-Modified` 回退）。
//!
//! # 关联规范
//!
//! - RGS-DTL-041 §3.3（13 个变体的源定义）
//! - RGS-SPEC-DTL-041 v0.2 §3（错误码契约）
//! - RGS-IMPL-PLAN-CDN-001 v0.1 §3.1 M-2063.4（本任务）

use thiserror::Error;

use crate::api::DownloadState;

/// 下载子系统统一错误类型。
///
/// 13 个变体严格对应 RGS-DTL-041 §3.3。新增变体必须先回写 DTL 并通过 DD Review。
#[derive(Debug, Error)]
pub enum DownloadError {
    /// 状态机非法转移（per RGS-DTL-041 §3.3 + FR-CDN-051）。
    ///
    /// 由 `state_machine::transition` 在不满足 `can_transition_to` 时返回；
    /// 不属于业务可恢复错误，调用方应终止该 file_path 的下载流程。
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: DownloadState, to: DownloadState },

    /// HTTP Range 请求失败（per RGS-DTL-041 §3.3 + FR-CDN-040）。
    ///
    /// 由 `range_client::RangeClient::request` 包装 `reqwest::Error` 返回；
    /// 重试策略由调用方按 SPEC §3（3 次 / 指数退避 100ms 起步）执行。
    #[error("range request failed: {0}")]
    RangeRequestFailed(#[from] reqwest::Error),

    /// 服务端 416 Range Not Satisfiable（per RGS-DTL-041 §3.3 + FR-CDN-043）。
    ///
    /// 触发条件：客户端的 `Range` 起始字节 ≥ 服务端资源实际大小；
    /// 通常意味着本地断点记录的 `total_size` 与服务端不一致，需触发全量重传。
    #[error("server returned 416 Range Not Satisfiable: total_size={total_size}, requested_start={requested_start}")]
    RangeNotSatisfiable { total_size: u64, requested_start: u64 },

    /// 服务端 200 OK（ETag 不匹配 → 全量重传；per FR-CDN-041 + FR-CDN-074）。
    ///
    /// 触发条件：`If-Range: <ETag>` 与服务端当前 ETag 不一致；
    /// 客户端必须丢弃断点记录，从头下载。
    #[error("server returned 200 OK (ETag changed): old_etag={old_etag:?}, new_etag={new_etag:?}")]
    ETagChanged { old_etag: String, new_etag: String },

    /// 整文件 hash 校验失败（per RGS-DTL-041 §3.3 + NFR-CDN-002 硬约束）。
    ///
    /// **本变体不提供跳过校验的旁路**；调用方必须丢弃当前文件并触发重新下载。
    /// 上层 `rgs-asset-update::IntegrityGate` 不得消费此变体做"标记重试"。
    #[error("integrity check failed: expected={expected}, actual={actual}")]
    IntegrityCheckFailed { expected: String, actual: String },

    /// Manifest 签名无效（per RGS-DTL-041 §3.3 + FR-CDN-071）。
    ///
    /// 触发条件：`rgs-asset-update` 拉取的 manifest 签名校验失败；
    /// 通常意味着 CDN 响应被劫持或 manifest 被服务端回滚到旧版本。
    #[error("manifest signature invalid: {reason}")]
    ManifestSignatureInvalid { reason: String },

    /// 灰度回退（per RGS-DTL-041 §3.3 + FR-CDN-072）。
    ///
    /// 触发条件：客户端开始下载后，服务端灰度策略回退使该资源对当前用户不可访问；
    /// 调用方应保留断点记录等待灰度重新放行（不重置 token）。
    #[error("gray rollout mismatch: file is no longer accessible to this player")]
    GrayRolledBack,

    /// 断点记录过期（per RGS-DTL-041 §3.3 + FR-CDN-063，默认 7 天）。
    ///
    /// 触发条件：本地断点记录的 `last_updated_at` 距当前超过 `resume_token_ttl_days`；
    /// 调用方应丢弃断点记录并触发全新下载。
    #[error("resume token expired: last_updated_at={last_updated_at}")]
    ResumeTokenExpired { last_updated_at: chrono::DateTime<chrono::Utc> },

    /// 磁盘空间不足（per RGS-DTL-041 §3.3）。
    ///
    /// 触发条件：`preallocate` 或运行时写入返回 ENOSPC；
    /// 调用方应引导用户清理磁盘后重试。
    #[error("disk space insufficient: required={required_bytes}, available={available_bytes}")]
    DiskSpaceInsufficient { required_bytes: u64, available_bytes: u64 },

    /// 重试耗尽（per RGS-DTL-041 §3.3 + RGS-SPEC-DTL-041 v0.2 §3）。
    ///
    /// 单 chunk 重试 3 次、指数退避 100ms 起步后仍失败；调用方应终止该 file_path
    /// 的下载并上报 `rgs_asset_download_resume_failure_total`。
    #[error("retry exhausted after {attempts} attempts")]
    RetryExhausted { attempts: u32 },

    /// HTTP 429 限流（per RGS-DTL-041 §3.3 + FR-CDN-044）。
    ///
    /// 触发条件：服务端返回 `429 Too Many Requests`；
    /// 调用方应按响应头 `Retry-After` 进入背压队列（per SPEC §5 故障域）。
    #[error("HTTP 429 rate limited")]
    RateLimited,

    /// 断点记录存储层错误（per RGS-DTL-041 §3.3 + FR-CDN-061）。
    ///
    /// 触发条件：SQLite / JSON 文件读写失败、原子 rename 失败、LRU 清理失败等；
    /// 字符串描述仅供开发期排查，**不**包含 PII（per FR-CDN-064）。
    #[error("resume token store error: {0}")]
    TokenStoreError(String),

    /// 服务端不支持 Range（per RGS-DTL-041 §3.3 + NFR-CDN-114 门禁）。
    ///
    /// 触发条件：响应头 `Accept-Ranges: none` 或缺失 `Accept-Ranges`；
    /// 调用方应回滚到 `rgs-asset-update` 既有全量下载路径（per SPEC §6 Rollback）。
    #[error("server does not support Range (Accept-Ranges: none)")]
    RangeNotSupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 编译期断言：13 个变体严格对应 RGS-DTL-041 §3.3。
    ///
    /// 此测试不验证行为，只验证类型签名存在；若有人删除某个变体，编译即失败。
    #[test]
    fn error_variants_compile() {
        fn _assert_variants(
            _: DownloadError,
        ) {
            // dummy match to enumerate all variants without runtime cost
            let _ = match todo!() as DownloadError {
                DownloadError::InvalidTransition { .. } => 0,
                DownloadError::RangeRequestFailed(_) => 1,
                DownloadError::RangeNotSatisfiable { .. } => 2,
                DownloadError::ETagChanged { .. } => 3,
                DownloadError::IntegrityCheckFailed { .. } => 4,
                DownloadError::ManifestSignatureInvalid { .. } => 5,
                DownloadError::GrayRolledBack => 6,
                DownloadError::ResumeTokenExpired { .. } => 7,
                DownloadError::DiskSpaceInsufficient { .. } => 8,
                DownloadError::RetryExhausted { .. } => 9,
                DownloadError::RateLimited => 10,
                DownloadError::TokenStoreError(_) => 11,
                DownloadError::RangeNotSupported => 12,
            };
        }
    }
}
