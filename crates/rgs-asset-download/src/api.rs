//! 公开 API 桩（per SPEC-DTL-041 v0.2 §3 + RGS-IMPL-PLAN-CDN-001 v0.1 §2.2）
//!
//! `download_asset` / `pause_download` / `cancel_download` / `get_download_state`
//!
//! 本模块为 IT 测试所需的最小可编译 stub；实质实现由 WF-1-2063 ~ WF-1-2065
//! worktree 交付。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::state_machine::DownloadState;

/// 公开下载请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub asset_id: String,
    pub manifest_url: String,
    pub backend_url: String,
    pub dest_path: PathBuf,
    pub chunk_size: u32,
}

/// 公开下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub state: DownloadState,
    pub bytes_received: u64,
    pub total_size: u64,
    pub chunks_completed: u32,
    pub etag: String,
}

/// 断点记录（13 字段 per SPEC §6：实际持久化在 SqliteResumeTokenStore，
/// 本 struct 为 API 视图；缺字段时由 worker 补全）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeToken {
    pub token_id: String,
    pub asset_id: String,
    pub file_path: PathBuf,
    pub total_size: u64,
    pub chunk_size: u32,
    pub completed_chunks: Vec<u32>,
    pub etag: String,
    pub backend_url: String,
    pub created_at_unix: u64,
    pub last_resume_at_unix: u64,
    pub resume_count: u32,
    pub sha256_expected: String,
    pub app_session_id: String,
}

/// 启动下载（IT 测试 stub）
pub async fn download_asset(_req: DownloadRequest) -> Result<DownloadProgress, crate::error::DownloadError> {
    Err(crate::error::DownloadError(
        crate::error::DownloadErrorKind::Internal(
            "download_asset 是 IT 测试 stub —— 真实实现在 WF-1-2063~2065".into(),
        ),
    ))
}

/// 暂停下载（IT 测试 stub）
pub async fn pause_download(_token_id: &str) -> Result<(), crate::error::DownloadError> {
    Err(crate::error::DownloadError(
        crate::error::DownloadErrorKind::Internal("pause_download stub".into()),
    ))
}

/// 取消下载（IT 测试 stub）
pub async fn cancel_download(_token_id: &str) -> Result<(), crate::error::DownloadError> {
    Err(crate::error::DownloadError(
        crate::error::DownloadErrorKind::Internal("cancel_download stub".into()),
    ))
}

/// 查询状态（IT 测试 stub）
pub async fn get_download_state(_token_id: &str) -> Result<DownloadProgress, crate::error::DownloadError> {
    Err(crate::error::DownloadError(
        crate::error::DownloadErrorKind::Internal("get_download_state stub".into()),
    ))
}

/// 工具：mock backoff（IT 测试用，避免 sleep 真实等待）
pub fn mock_backoff(base: Duration) -> Duration {
    base
}
