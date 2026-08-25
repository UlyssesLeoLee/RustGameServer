//! 公开 API 定义（per RGS-SPEC-DTL-041 §2 + §3）
//!
//! 4 个核心 API：
//! - [`AssetDownloader::download_asset`]：启动/恢复一个资产下载
//! - [`AssetDownloader::pause_download`]：暂停（FR-CDN-083 必须取消 in_flight）
//! - [`AssetDownloader::cancel_download`]：取消并删除断点记录
//! - [`AssetDownloader::get_download_state`]：查询当前状态机快照
//!
//! **M-2063.3 占位**：本模块仅定义 trait 与数据契约，不包含具体实现。
//! 具体实现（HTTP Range / 并发分片 / 整文件校验）由 WF-1-2065（M-2065.1~11）落地。
//!
//! **PII 边界**：本 trait 的入参 / 出参 / 中间状态**禁止**出现 `player_id` / `device_id` /
//! `email` / `ip_address` / `mac_address` 字段（per FR-CDN-064）。

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AssetDownloadError;
use crate::state_machine::DownloadState;

/// 下载请求（启动一次资产下载的入参）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRequest {
    /// 资产 ID（来自 `rgs-asset-update` Manifest；不含 PII）
    pub asset_id: String,
    /// 目标本地文件路径（沙箱内）
    pub file_path: PathBuf,
    /// 期望的总字节数（来自 Manifest；用于预分配 / 进度显示）
    pub expected_total_bytes: u64,
    /// 期望的 ETag（来自 Manifest；用于 `If-Range: <ETag>` 头 per FR-CDN-074）
    pub expected_etag: String,
    /// 期望的 SHA-256（来自 Manifest；用于整文件校验 per NFR-CDN-002）
    pub expected_sha256: String,
    /// 后端 URL（HTTP Range endpoint；per NFR-CDN-114 必须支持 Range）
    pub backend_url: String,
    /// 分片粒度（字节），0 表示用 config.rs 默认值（PH-3 实测填 4~16MB）
    pub chunk_size_bytes: u64,
}

/// 下载状态快照（[`AssetDownloader::get_download_state`] 出参）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadStateSnapshot {
    /// 关联的 token_id（无则为空字符串）
    pub token_id: String,
    /// 资产 ID
    pub asset_id: String,
    /// 当前状态机状态
    pub state: DownloadState,
    /// 已接收字节数
    pub bytes_received: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 已完成分片数
    pub completed_chunks: u32,
    /// 分片总数
    pub total_chunks: u32,
    /// 状态快照时间
    pub observed_at: DateTime<Utc>,
}

/// 资产下载器 trait（公开 API）
///
/// **实现约束**（per RGS-SPEC-DTL-041 §3 + RGS-DTL-041 §3）：
/// - 所有实现必须调用 [`crate::state_machine::DownloadStateMachine`] 推进状态
/// - 断点记录必须通过 [`crate::resume_token_store::ResumeTokenStore`] 持久化
/// - 整文件校验不可绕过（`download_asset` 内部必须先完成 [`crate::integrity_gate::IntegrityGate`]，
///   才能把 state 切到 `Completed`；NFR-CDN-002 硬约束）
/// - 暂停时必须取消所有 in_flight Range 请求（FR-CDN-083）
#[async_trait]
pub trait AssetDownloader: Send + Sync {
    /// 启动或恢复一个下载任务
    ///
    /// 行为契约：
    /// - 若 `token_id` 对应的断点记录存在：从 checkpoint 恢复
    /// - 若不存在：全新下载，先 `Idle -> Resolving -> Downloading`
    /// - 下载完成后必须做整文件 SHA-256 校验（NFR-CDN-002），通过后才返回 `Ok(DownloadStateSnapshot)`
    async fn download_asset(
        &self,
        request: DownloadRequest,
    ) -> Result<DownloadStateSnapshot, AssetDownloadError>;

    /// 暂停下载（FR-CDN-083：必须取消 in_flight Range 请求）
    ///
    /// 状态转移：`Downloading -> Paused`（合法）或 `Paused`（idempotent no-op）
    async fn pause_download(&self, token_id: &str) -> Result<DownloadStateSnapshot, AssetDownloadError>;

    /// 取消下载并删除断点记录
    ///
    /// 状态转移：任意 -> `Cancelled`；从 store 删除 token
    async fn cancel_download(&self, token_id: &str) -> Result<DownloadStateSnapshot, AssetDownloadError>;

    /// 查询下载状态快照
    ///
    /// 若 `token_id` 不存在返回 [`AssetDownloadError::TokenNotFound`]
    async fn get_download_state(
        &self,
        token_id: &str,
    ) -> Result<DownloadStateSnapshot, AssetDownloadError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_request_does_not_carry_pii() {
        // 确保 DownloadRequest 字段不含 PII（per FR-CDN-064）
        // 通过反射做不到；这里靠编译时 + 后续 grep 双重检查
        let req = DownloadRequest {
            asset_id: "asset-001".to_string(),
            file_path: PathBuf::from("/tmp/x.bin"),
            expected_total_bytes: 1024,
            expected_etag: "\"abc\"".to_string(),
            expected_sha256: "deadbeef".to_string(),
            backend_url: "https://cdn.example.com/x.bin".to_string(),
            chunk_size_bytes: 8 * 1024 * 1024,
        };
        // 仅基础健全性检查
        assert_eq!(req.asset_id, "asset-001");
        assert_eq!(req.expected_total_bytes, 1024);
    }

    #[test]
    fn snapshot_carries_state_and_progress() {
        let snap = DownloadStateSnapshot {
            token_id: "token-001".to_string(),
            asset_id: "asset-001".to_string(),
            state: DownloadState::Downloading,
            bytes_received: 512,
            total_bytes: 1024,
            completed_chunks: 1,
            total_chunks: 2,
            observed_at: Utc::now(),
        };
        assert_eq!(snap.state, DownloadState::Downloading);
        assert_eq!(snap.completed_chunks, 1);
    }
}
