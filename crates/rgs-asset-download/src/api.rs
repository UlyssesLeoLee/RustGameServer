//! AssetDownloader —— 客户端资源下载子系统的公开 API。
//!
//! per **RGS-DTL-041 §2.4**（公开 API 入口）+ **RGS-SPEC-DTL-041 v0.2 §2 / §3**
//! + **RGS-IMPL-PLAN-CDN-001 v0.1 §3.1 M-2063.3**。
//!
//! # 4 个公开方法（严格按 DTL §2.4）
//!
//! | 方法 | 用途 | 关键约束 |
//! |---|---|---|
//! | `download_asset` | 触发下载入口（支持断点恢复）| NFR-CDN-002 / FR-CDN-074 / FR-CDN-083 |
//! | `pause_download` | 暂停下载 | FR-CDN-083（取消 in_flight）|
//! | `cancel_download` | 取消下载（区别于暂停）| FR-CDN-063 清理断点 |
//! | `get_download_state` | 查询当前状态 | — |
//!
//! # 硬约束绑定
//!
//! - **NFR-CDN-002（整文件校验不可绕过）**：`download_asset` 实现契约**必须**
//!   经 `IntegrityGate`（M-2065.5）；trait 层面不暴露任何绕过整文件 hash 校验
//!   的旁路（代码评审 grep 验证，模式由 §5.3 实施计划维护，本文件不写字面）。
//! - **FR-CDN-064（断点记录不含 PII）**：本 trait 的所有参数与返回类型**不**含
//!   任何玩家身份 / 设备 / 网络标识字段；断点记录 13 字段见 `ResumeToken`
//!   （M-2064.2 完整定义，本骨架不在此列出字段，per DTL §4.1）。
//! - **FR-CDN-083（暂停时必须取消 in_flight）**：`download_asset` 接受
//!   `cancel_token: &CancelToken` 参数；`pause_download` 通过
//!   `cancel_token.cancel()` 触发取消，实现必须在 in_flight reqwest 请求
//!   循环中轮询 `is_cancelled()` 并立即 abort。
//! - **FR-CDN-074（If-Range ETag 强制）**：`download_asset` 实现契约
//!   必须在所有 Range 请求上携带 `If-Range: <ETag>` 头，**不**接受
//!   `Last-Modified` 回退。
//!
//! # 关联规范
//!
//! - RGS-DTL-041 §2.4（公开 API 入口）
//! - RGS-DTL-041 §3.1（8 状态枚举）
//! - RGS-SPEC-DTL-041 v0.2 §2 / §3
//! - RGS-IMPL-PLAN-CDN-001 v0.1 §3.1 M-2063.3（本任务）

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::DownloadError;

/// 下载状态枚举（per RGS-DTL-041 §3.1，8 状态）。
///
/// 本骨架在 `api.rs` 给出最小定义；M-2064.1 将在专属状态机模块（per
/// RGS-IMPL-PLAN-CDN-001 v0.1 §2.2 计划路径）扩展 `can_transition_to`
/// 转移表并完善文档。
///
/// > **命名差异说明**：实施计划 §3.1 列出 `Idle / Resolving / ...` 8 状态名称
/// > 与 DTL §3.1 的 `NotStarted / Probing / Resuming / ...` 略有差异。按
/// > "DTL 评审变更为准"原则（per RGS-SPEC-DTL-041 v0.2 §1），本骨架采用
/// > DTL §3.1 的 8 状态命名。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DownloadState {
    /// 初始态：尚未触发下载
    NotStarted,
    /// HEAD 探测中（per FR-CDN-042）
    Probing,
    /// 断点恢复中（校验 ETag / 灰度 / Manifest 签名）
    Resuming,
    /// 下载中
    Downloading,
    /// 已暂停（玩家意图；恢复时实际先 Resuming）
    Paused,
    /// 失败（可重试）
    Failed,
    /// 已取消（区别于暂停；断点记录保留但可被清理）
    Canceled,
    /// 已完成（终态）
    Completed,
}

impl DownloadState {
    /// 是否终态（per RGS-DTL-041 §3.1）
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Canceled | Self::Completed)
    }
}

/// 取消令牌（per FR-CDN-083 + 实施计划 §3.1 M-2063.3 "trait 签名设计留 cancel_token"）。
///
/// 轻量自实现，避免引入 `tokio-util` 依赖（workspace 现有 crate 均无此 dep）。
/// 语义对齐 `tokio_util::sync::CancellationToken`：
/// - `cancel()` 后 `is_cancelled()` 立即返回 `true`
/// - 多 owner 可同时持有（`Clone`），所有持有者观察到一致的取消状态
///
/// # 用法
///
/// ```text
/// let cancel_token = CancelToken::new();
/// let token_for_pause = cancel_token.clone();
///
/// // 触发下载
/// downloader.download_asset(file_path, source_url, &cancel_token).await?;
///
/// // 玩家暂停：触发取消
/// downloader.pause_download(file_path).await?;  // 内部 cancel_token.cancel()
/// # assert!(token_for_pause.is_cancelled());
/// ```
#[derive(Debug, Clone, Default)]
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    /// 构造未触发的取消令牌。
    pub fn new() -> Self {
        Self::default()
    }

    /// 触发取消；`is_cancelled()` 之后立即返回 `true`。
    ///
    /// 由 `pause_download` / `cancel_download` 内部调用。
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// 检查是否已触发取消。
    ///
    /// `download_asset` 实现需在每个 in_flight reqwest 请求循环中轮询此方法，
    /// 触发后立即 `reqwest::RequestBuilder::abort()` 或丢弃当前 chunk。
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// 客户端资源下载子系统的公开 API trait（per RGS-DTL-041 §2.4）。
///
/// 4 个方法严格对应 RGS-IMPL-PLAN-CDN-001 v0.1 §3.1 M-2063.3。
/// 上层通过 `rgs-asset-update` 的 `Manifest` 拉取后调用本 trait；
/// **不**反向依赖 `rgs-asset-update`（per RGS-SPEC-DTL-041 v0.2 §2）。
#[async_trait]
pub trait AssetDownloader: Send + Sync {
    /// 触发下载入口（支持断点恢复）。
    ///
    /// 流程：HEAD 探测 → 断点恢复（若 `ResumeToken` 存在）→ ChunkOrchestrator
    /// 并发分片 → IntegrityGate 整文件校验 → Completed。
    ///
    /// # 契约
    ///
    /// - **NFR-CDN-002**：实现**必须**在所有分片落盘后调用
    ///   `IntegrityGate::verify_whole_file`；不允许任何跳过整文件 hash 校验的
    ///   旁路。
    /// - **FR-CDN-074**：所有 Range 请求携带 `If-Range: <ETag>` 头；
    ///   ETag 不匹配触发 `DownloadError::ETagChanged` 后全量重传。
    /// - **FR-CDN-083**：实现必须**轮询** `cancel_token.is_cancelled()`，
    ///   触发后立即取消所有 in_flight reqwest 请求并返回 `DownloadError::RetryExhausted`。
    async fn download_asset(
        &self,
        file_path: &str,
        source_url: &str,
        cancel_token: &CancelToken,
    ) -> Result<(), DownloadError>;

    /// 暂停下载。
    ///
    /// 实现契约：
    /// 1. 触发 `cancel_token.cancel()`（per FR-CDN-083 取消 in_flight）
    /// 2. 落盘当前 `ResumeToken`（per FR-CDN-061 原子写：先 JSON 再 SQLite）
    /// 3. 状态机转 `Paused`（per DTL §3.1）
    async fn pause_download(&self, file_path: &str) -> Result<(), DownloadError>;

    /// 取消下载（区别于暂停）。
    ///
    /// 与 `pause_download` 的区别：
    /// - **取消**会**丢弃**断点记录（per FR-CDN-063 清理路径）
    /// - **暂停**会**保留**断点记录，恢复时无需重新探测
    async fn cancel_download(&self, file_path: &str) -> Result<(), DownloadError>;

    /// 查询当前下载状态。
    ///
    /// 返回 `ResumeToken` 中持久化的最新 `DownloadState`；常用于 UI 进度条渲染
    /// （per RGS-SPEC-DTL-041 v0.2 §4：关键请求可用 `file_path` + `token_id` 反查）。
    async fn get_download_state(
        &self,
        file_path: &str,
    ) -> Result<DownloadState, DownloadError>;
}
