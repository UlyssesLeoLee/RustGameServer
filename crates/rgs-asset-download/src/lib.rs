//! rgs-asset-download —— 客户端资源分发的断点续传与可恢复下载
//!
//! 范围：纯客户端 SDK 层（per RGS-SPEC-DTL-041 v0.2 + RGS-DTL-041）
//! - 4 平台可用：iOS 17 / Android 14 / Windows 11 / macOS 14
//! - 不绕过 `rgs-asset-update` 的 `IntegrityGate`（NFR-CDN-002 硬约束）
//! - 支持 HTTP Range 后端（自托管 MinIO + 商业 CDN 可选对照）
//! - 断点续传：进程崩溃/网络断开/手动暂停/手动取消 后，可从最近 checkpoint 恢复
//!
//! # 模块清单（per SPEC-DTL-041 §2.2 + RGS-IMPL-PLAN-CDN-001 v0.1 §2.2）
//!
//! - [`api`]           公开 API 桩（`download_asset` / `pause_download` / `cancel_download` / `get_download_state`）
//! - [`error`]         错误码（per DTL §6，不自创）
//! - [`state_machine`] `DownloadStateMachine` 8 状态 + 转移表
//! - [`range_client`]  HTTP/1.1 RFC 7233 HEAD/Range 客户端（206/416/200/429 全部响应路径）
//! - [`config`]        并发数 / LRU / 断点过期阈值（PH-3 实测填入）
//! - [`platform`]      4 平台分支（sparse file 预分配）
//!
//! # 重要阶段状态
//!
//! 本 crate 当前处于 **PH-4 实测阶段（WBS L4 #2069）**：
//! - 完整 API / 状态机 / RangeClient / IntegrityGate 实现在并行 worktree
//!   （WF-1-2063/2064/2065），本 worktree 仅承载**集成测试**与 IT 报告。
//! - 所有 IT 测试默认 `#[ignore]`，待真实 MinIO 容器可用时由 SRE 接力执行
//!   `cargo test -p rgs-asset-download --tests -- --include-ignored`。
//! - 编译期 `cargo test -p rgs-asset-download --tests --no-run` 必须 0 error
//!   （per WBS WF-1-2069 验收门槛）。

pub mod api;
pub mod config;
pub mod error;
pub mod platform;
pub mod range_client;
pub mod state_machine;

pub use api::{download_asset, get_download_state, pause_download, cancel_download, DownloadRequest, DownloadProgress, ResumeToken};
pub use error::{DownloadError, DownloadErrorKind};
pub use range_client::{RangeClient, RangeResponse, RangeRequest};
pub use state_machine::{DownloadState, DownloadStateMachine, StateTransition};
