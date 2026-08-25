//! rgs-asset-download —— 客户端资源分发的断点续传与可恢复下载。
//!
//! per **RGS-DTL-041**（源详细设计）+ **RGS-SPEC-DTL-041 v0.2**（实现规格）
//! + **RGS-IMPL-PLAN-CDN-001 v0.1 §3.1**（L4 #2063 crate 骨架）。
//!
//! # 公开 API（per RGS-DTL-041 §2.4）
//!
//! - [`AssetDownloader`] —— 4 方法 trait：`download_asset` / `pause_download`
//!   / `cancel_download` / `get_download_state`
//! - [`DownloadError`] —— 13 变体错误类型（per DTL §3.3，**不自创**）
//! - [`DownloadState`] —— 8 状态枚举（per DTL §3.1）
//! - [`CancelToken`] —— FR-CDN-083 取消令牌（自实现，避免新增 `tokio-util` 依赖）
//! - [`DownloadConfig`] —— 运行时配置（chunk size / LRU / TTL 保守默认值）
//!
//! # 硬约束绑定
//!
//! | 编号 | 内容 | 落地位置 |
//! |---|---|---|
//! | **NFR-CDN-002** | 整文件校验不可绕过 | `IntegrityGate`（M-2065.5 接入）；代码评审 grep 验证 |
//! | **FR-CDN-064** | 断点记录不含 PII | `ResumeToken` 13 字段（per DTL §4.1）；代码评审 grep 验证 |
//! | **FR-CDN-074** | `If-Range: <ETag>` 强制 | `RangeClient`（M-2065.1 + M-2065.2）|
//! | **FR-CDN-083** | 暂停时必须取消 in_flight | [`CancelToken`] + [`AssetDownloader::download_asset`] 签名 |
//! | **NFR-CDN-114** | DistributionBackend 必须支持 Range | `RangeNotSupported` 错误（per DTL §3.3）|
//!
//! # M 任务进度（本 L4 #2063）
//!
//! | M # | 状态 | 文件 |
//! |---|---|---|
//! | M-2063.1 | ✅ done（本骨架）| `Cargo.toml` |
//! | M-2063.2 | ✅ done（本骨架）| `lib.rs` + `platform/{mod,unix,windows,android,ios}.rs` |
//! | M-2063.3 | ✅ done（本骨架）| `api.rs` |
//! | M-2063.4 | ✅ done（本骨架）| `error.rs` |
//! | M-2063.5 | ✅ done（本骨架）| `config.rs` |
//!
//! 后续 M-2064.x / M-2065.x 任务由对应 L4 worker 子代理在本骨架上增量实现。
//!
//! # 当前状态
//!
//! **骨架版本（0.1.0）**：仅含公开 API 契约 + 错误码 + 平台模块 stub + 配置占位。
//! **不**含：状态机实现 / ResumeTokenStore / RangeClient / ChunkOrchestrator /
//! IntegrityGate —— 这些由 M-2064.x / M-2065.x 任务在本骨架上增量实现。
//!
//! per RGS-SPEC-DTL-041 v0.2 §7 DoD 第 7 条：当前无完整实现时应保持
//! "待实现/待评审"状态，**不**得标记生产完成。

pub mod api;
pub mod config;
pub mod error;
pub mod platform;

pub use api::{AssetDownloader, CancelToken, DownloadState};
pub use config::DownloadConfig;
pub use error::DownloadError;

// — 单元测试夹具（占位；M-2064 / M-2065 接入真实测试）—

/// 编译期断言：`DownloadError` 的 13 变体严格对应 RGS-DTL-041 §3.3。
///
/// 此断言在 lib.rs 顶层执行；任何破坏 §3.3 错误码契约的改动会立即编译失败。
#[allow(dead_code)]
const _: fn() = || {
    // dummy closure body ensures compile-time evaluation
    let _ = || {
        // 13 个变体的穷举：与 src/error.rs 末位 tests 中的枚举顺序保持一致
        let _: DownloadError = DownloadError::InvalidTransition {
            from: DownloadState::NotStarted,
            to: DownloadState::Downloading,
        };
        let _: DownloadError = DownloadError::RangeRequestFailed(
            // 占位 reqwest::Error 由 M-2064.x 替换
            reqwest::Client::new().get("http://127.0.0.1:0/").build().unwrap_err(),
        );
        let _: DownloadError = DownloadError::RangeNotSatisfiable {
            total_size: 0,
            requested_start: 0,
        };
        let _: DownloadError = DownloadError::ETagChanged {
            old_etag: String::new(),
            new_etag: String::new(),
        };
        let _: DownloadError = DownloadError::IntegrityCheckFailed {
            expected: String::new(),
            actual: String::new(),
        };
        let _: DownloadError = DownloadError::ManifestSignatureInvalid {
            reason: String::new(),
        };
        let _: DownloadError = DownloadError::GrayRolledBack;
        let _: DownloadError = DownloadError::ResumeTokenExpired {
            last_updated_at: chrono::Utc::now(),
        };
        let _: DownloadError = DownloadError::DiskSpaceInsufficient {
            required_bytes: 0,
            available_bytes: 0,
        };
        let _: DownloadError = DownloadError::RetryExhausted { attempts: 0 };
        let _: DownloadError = DownloadError::RateLimited;
        let _: DownloadError = DownloadError::TokenStoreError(String::new());
        let _: DownloadError = DownloadError::RangeNotSupported;
    };
};
