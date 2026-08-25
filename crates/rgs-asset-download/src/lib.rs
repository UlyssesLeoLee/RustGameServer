//! `rgs-asset-download` —— 客户端资源分发的断点续传与可恢复下载
//!
//! 实现规格：RGS-SPEC-DTL-041 v0.2
//! 实施计划：RGS-IMPL-PLAN-CDN-001 v0.1
//!
//! ## 范围
//!
//! 纯客户端 SDK 层；不向 `rgs-asset-update` 反向依赖（per RGS-SPEC-DTL-041 §3）。
//!
//! ## 模块（per RGS-IMPL-PLAN-CDN-001 §2.2 + L4 #2063/#2064/#2065 拆解）
//!
//! - [`api`]：公开 API trait（`download_asset` / `pause_download` / `cancel_download` / `get_download_state`）
//! - [`error`]：错误码（per RGS-DTL-041 §6，不自创）
//! - [`state_machine`]：8 状态 DownloadStateMachine + 转移表（M-2064.1）
//! - [`resume_token`]：ResumeToken 13 字段结构（M-2064.2；不含 PII per FR-CDN-064）
//! - [`resume_token_store`]：ResumeTokenStore trait + JsonFile / Sqlite 实现（M-2064.3~5）
//!
//! ## 后续 M 任务（不在本 worktree 范围）
//!
//! - `range_client`：HTTP/1.1 RFC 7233 HEAD + Range（M-2065.1~2）
//! - `chunk_orchestrator`：并发分片调度 + 暂停/取消信号（M-2065.3~4）
//! - `integrity_gate`：整文件 SHA-256（M-2065.5）
//! - `platform/*`：4 平台 sparse file（M-2065.6~7）
//! - `metrics`：10 项 `rgs_asset_download_*`（M-2065.8）
//! - `config`：并发数 / LRU / 过期阈值（PH-3 实测填入）

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod api;
pub mod error;
pub mod resume_token;
pub mod resume_token_store;
pub mod state_machine;

// 公开 API re-export（per RGS-SPEC-DTL-041 §2 + §3）
pub use api::{AssetDownloader, DownloadRequest, DownloadStateSnapshot};
pub use error::AssetDownloadError;
pub use resume_token::{ResumeToken, ResumeTokenError, TOKEN_SCHEMA_VERSION};
pub use resume_token_store::{
    JsonFileResumeTokenStore, ResumeTokenStore, SqliteResumeTokenStore, DEFAULT_LRU_MAX_BYTES,
};
pub use state_machine::{
    allowed_events, allowed_transitions, next_state, DownloadState, DownloadStateMachine,
    StateEvent, StateTransition, TransitionError,
};
