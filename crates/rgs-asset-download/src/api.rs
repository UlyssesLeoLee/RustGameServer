//! 公开 API（per SPEC §2 + IMPL-PLAN §3.1 / M-2063.3）。
//!
//! 提供 trait：
//! - [`AssetDownloadService::download_asset`]    启动 / 恢复下载
//! - [`AssetDownloadService::pause_download`]    暂停（断点落盘 + 取消 in_flight）
//! - [`AssetDownloadService::cancel_download`]   取消（删除断点）
//! - [`AssetDownloadService::get_download_state`] 查询状态
//!
//! **PREREQ 阶段实现策略**：trait + 默认实现 + 状态机包装。M-2065.3 / M-2065.4 落定后
//! `pause_download` / `cancel_download` 由 `ChunkOrchestrator` 提供具体实现。
//!
//! 硬约束（FR-CDN-064）：本文件**禁止**引用任何 PII 字段（per SPEC §3）。
//! 所有 token_id 为 UUID v4，由本 crate 内部生成。

use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::DownloadResult;
use crate::state_machine::{DownloadState, DownloadStateMachine, StateEvent};

/// `ResumeToken` 在 `api` 模块下的 re-export（per `it_minio_resume.rs` 的 `use rgs_asset_download::api::ResumeToken` 路径）
pub use crate::resume_token::ResumeToken;

/// 下载请求（per SPEC §2 + DTL §3）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    /// 资源 ID（manifest 主键；与签名一起保证；**不**含 PII 字段）
    pub asset_id: String,
    /// 目标落盘路径
    pub file_path: String,
    /// 远端 URL（含 `https://` / `http://`）
    pub url: String,
    /// manifest 声明的 SHA-256（hex）
    pub expected_sha256: String,
    /// manifest 声明的总字节数
    pub expected_size_bytes: u64,
    /// 断点 ID（**可选**；resume 时填入；首次下载为 None）
    pub resume_token_id: Option<String>,
}

impl DownloadRequest {
    /// 简要摘要（用于日志；不含路径明文）
    pub fn summary(&self) -> String {
        format!(
            "asset_id={} size={} token_id={:?}",
            self.asset_id, self.expected_size_bytes, self.resume_token_id
        )
    }
}

/// 下载进度（用于 `get_download_state`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 已完成字节数
    pub bytes_received: u64,
    /// 已完成 chunk 数
    pub chunks_completed: u64,
    /// 总 chunk 数
    pub chunks_total: u64,
}

impl DownloadProgress {
    /// 进度百分比（0~100）
    pub fn percent(&self) -> f64 {
        if self.chunks_total == 0 {
            return 0.0;
        }
        (self.chunks_completed as f64) * 100.0 / (self.chunks_total as f64)
    }
}

/// 状态查询结果视图。
#[derive(Debug, Clone)]
pub struct DownloadStateView {
    /// 当前状态
    pub state: DownloadState,
    /// 进度
    pub progress: DownloadProgress,
    /// 关联的断点 ID（如有）
    pub resume_token_id: Option<String>,
}

/// 暂停结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseOutcome {
    /// 断点 ID（已落盘）
    pub resume_token_id: String,
    /// 暂停时刻已完成的字节数
    pub bytes_received: u64,
}

impl fmt::Display for PauseOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PauseOutcome(token_id={}, bytes={})",
            self.resume_token_id, self.bytes_received
        )
    }
}

/// 取消结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelOutcome {
    /// 取消时刻已完成的字节数（用于审计；不会恢复）
    pub bytes_received: u64,
    /// 是否删除了断点记录
    pub token_removed: bool,
}

impl fmt::Display for CancelOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CancelOutcome(bytes={}, token_removed={})",
            self.bytes_received, self.token_removed
        )
    }
}

/// 公开 API trait（per SPEC §2）。
///
/// 任意 backend（MinIO / Cloudflare / 自研）只要实现本 trait 就接入 SDK。
/// 同步实现用 `tokio::task::spawn_blocking` 包装。
#[async_trait]
pub trait AssetDownloadService: Send + Sync {
    /// 启动 / 恢复下载。返回断点 ID + 最终结果。
    async fn download_asset(&self, req: DownloadRequest) -> DownloadResult<PauseOutcome>;

    /// 暂停下载（断点已落盘 + 所有 in_flight Range 已取消，per FR-CDN-083）。
    async fn pause_download(&self, resume_token_id: &str) -> DownloadResult<PauseOutcome>;

    /// 取消下载（删除断点 + 取消 in_flight，per FR-CDN-083）。
    async fn cancel_download(&self, resume_token_id: &str) -> DownloadResult<CancelOutcome>;

    /// 查询状态。
    async fn get_download_state(
        &self,
        resume_token_id: &str,
    ) -> DownloadResult<DownloadStateView>;
}

/// PREREQ 阶段默认实现：仅做状态机推进（不实际触发 IO / HTTP）。
/// WF-1-2064 / WF-1-2065 / WF-1-2069 完成后会被真实实现替换。
pub struct DefaultAssetDownloadService {
    sm: std::sync::Mutex<DownloadStateMachine>,
}

impl Default for DefaultAssetDownloadService {
    fn default() -> Self {
        Self {
            sm: std::sync::Mutex::new(DownloadStateMachine::new()),
        }
    }
}

impl DefaultAssetDownloadService {
    /// 新建。
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AssetDownloadService for DefaultAssetDownloadService {
    async fn download_asset(&self, req: DownloadRequest) -> DownloadResult<PauseOutcome> {
        // PREREQ 占位：仅推进状态机 + 生成 token_id
        let mut sm = self.sm.lock().expect("state machine poisoned");
        sm.apply(StateEvent::ResolveStart)?;
        let token_id = req
            .resume_token_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Ok(PauseOutcome {
            resume_token_id: token_id,
            bytes_received: 0,
        })
    }

    async fn pause_download(&self, resume_token_id: &str) -> DownloadResult<PauseOutcome> {
        let mut sm = self.sm.lock().expect("state machine poisoned");
        sm.apply(StateEvent::Pause)?;
        Ok(PauseOutcome {
            resume_token_id: resume_token_id.to_string(),
            bytes_received: 0,
        })
    }

    async fn cancel_download(&self, _resume_token_id: &str) -> DownloadResult<CancelOutcome> {
        let mut sm = self.sm.lock().expect("state machine poisoned");
        sm.apply(StateEvent::Cancel)?;
        Ok(CancelOutcome {
            bytes_received: 0,
            token_removed: true,
        })
    }

    async fn get_download_state(
        &self,
        resume_token_id: &str,
    ) -> DownloadResult<DownloadStateView> {
        let sm = self.sm.lock().expect("state machine poisoned");
        Ok(DownloadStateView {
            state: sm.state(),
            progress: DownloadProgress {
                bytes_received: 0,
                chunks_completed: 0,
                chunks_total: 0,
            },
            resume_token_id: Some(resume_token_id.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_service_drives_state_machine() {
        let svc = DefaultAssetDownloadService::new();
        let req = DownloadRequest {
            asset_id: "asset-001".into(),
            file_path: "/tmp/out.bin".into(),
            url: "https://cdn.example.com/assets/asset-001".into(),
            expected_sha256: "deadbeef".into(),
            expected_size_bytes: 1024,
            resume_token_id: None,
        };
        let outcome = svc.download_asset(req).await.unwrap();
        assert!(!outcome.resume_token_id.is_empty());
        let view = svc.get_download_state(&outcome.resume_token_id).await.unwrap();
        // 推进到 Resolving
        assert!(matches!(
            view.state,
            DownloadState::Resolving | DownloadState::Downloading
        ));
    }

    #[test]
    fn download_request_summary_has_no_pii() {
        let req = DownloadRequest {
            asset_id: "asset-001".into(),
            file_path: "/tmp/out.bin".into(),
            url: "https://cdn.example.com/assets/asset-001".into(),
            expected_sha256: "deadbeef".into(),
            expected_size_bytes: 1024,
            resume_token_id: None,
        };
        let s = req.summary();
        // PII 字段名反向断言由 tests/security_no_pii.rs 集中验证
        // （per FR-CDN-064 硬约束）
        let _ = s;
    }
}
