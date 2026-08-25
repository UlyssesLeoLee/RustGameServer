//! `rgs-asset-download` —— 客户端可恢复下载 SDK
//!
//! 实施依据：RGS-IMPL-PLAN-CDN-001 v0.1（per 主对话 2026-08-25 12:13 孤儿 SPEC 治理延伸）。
//!
//! 范围（per SPEC-DTL-041 §1 + 实施计划 §1.1）：
//! - 纯客户端 SDK 层断点续传
//! - 4 平台可用（iOS 17 / Android 14 / Windows 11 / macOS 14）
//! - 不绕过 `rgs-asset-update` 的 `IntegrityGate`（NFR-CDN-002 硬约束）
//! - 支持 HTTP Range 后端（自托管 MinIO + 商业 CDN 可选对照）
//! - 不持有服务端凭证（FR-CDN-001 既有）
//! - 断点记录不含 PII（FR-CDN-064）
//!
//! 关键硬约束（per 实施计划 §1.3）：
//! - NFR-CDN-002 整文件校验不可绕过
//! - FR-CDN-074 用 `If-Range: <ETag>` 不用 `Last-Modified`
//! - FR-CDN-083 暂停时必须取消 in_flight Range 请求
//! - NFR-CDN-114 DistributionBackend 必须支持 HTTP Range
//!
//! 状态：本 crate 在 PH-3（W7-W9）逐步落地；当前 v0.1 仅含占位公开 API + 边缘实测 IT 脚手架。
//! 真正实现需等 L4 #2063 ~ #2065 任务推进。

#![deny(unsafe_code)]
#![warn(missing_docs)]

/// 公开 API 命名空间占位（per 实施计划 §2.2）：M-2063.3 落地后填充。
pub mod api {
    /// 公开 API trait 占位。**当前未实现**；等 M-2063.3 任务。
    pub trait DownloadApi {
        /// 触发一次下载。占位签名。
        fn placeholder(&self) -> &str;
    }
}

/// Range 客户端占位（per 实施计划 §2.2）：M-2065.1 落地后填充。
pub mod range_client {
    /// Range 客户端占位类型。
    pub struct RangeClientPlaceholder;
}

/// 整文件校验占位（per NFR-CDN-002）：M-2065.5 落地后填充。
pub mod integrity_gate {
    /// 整文件 hash 校验器占位。
    pub struct IntegrityGatePlaceholder;
}

/// 配置占位（per 实施计划 §2.2）：M-2063.5 落地后填充。
pub mod config {
    /// 块大小默认值 = 8MB（per 实施计划 §7.3）。
    pub const DEFAULT_CHUNK_SIZE_BYTES: u64 = 8 * 1024 * 1024;
    /// 断点记录 LRU 上限默认值 = 100MB（per 实施计划 §7.3）。
    pub const DEFAULT_LRU_MAX_BYTES: u64 = 100 * 1024 * 1024;
    /// 断点过期阈值默认值 = 7 天（per 实施计划 §7.3）。
    pub const DEFAULT_RESUME_TOKEN_TTL_DAYS: u32 = 7;
}

/// 错误码占位（per 实施计划 §2.2）：M-2063.4 落地后填充。
pub mod error {
    use thiserror::Error;

    /// 占位错误类型；M-2063.4 完整定义。
    #[derive(Debug, Error)]
    pub enum DownloadError {
        /// 占位变体。
        #[error("placeholder error: rgs-asset-download is not fully implemented yet (PH-3 W7-W9)")]
        NotImplemented,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lib_default_config_values_match_implementation_plan() {
        // 验证占位常量与 RGS-IMPL-PLAN-CDN-001 v0.1 §7.3 一致
        assert_eq!(config::DEFAULT_CHUNK_SIZE_BYTES, 8 * 1024 * 1024);
        assert_eq!(config::DEFAULT_LRU_MAX_BYTES, 100 * 1024 * 1024);
        assert_eq!(config::DEFAULT_RESUME_TOKEN_TTL_DAYS, 7);
    }

    #[test]
    fn placeholder_constructible() {
        let _ = range_client::RangeClientPlaceholder;
        let _ = integrity_gate::IntegrityGatePlaceholder;
    }

    #[test]
    fn error_not_implemented_displays() {
        let e = error::DownloadError::NotImplemented;
        let msg = e.to_string();
        assert!(msg.contains("placeholder"));
        assert!(msg.contains("PH-3"));
    }
}
