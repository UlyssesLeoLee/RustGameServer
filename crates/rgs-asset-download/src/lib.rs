//! `rgs-asset-download` —— 客户端资源分发的断点续传与可恢复下载。
//!
//! 实施参考：
//! - `docs/12-工作流/RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md`（L4 #2063/#2064/#2065）
//! - `docs/13-实现规格/RGS-SPEC-DTL-041_实现规格书.md`（v0.2）
//!
//! 本 crate 自身**只**做传输层（RangeClient / ChunkOrchestrator / IntegrityGate + 4 平台
//! 预分配），**不**重新实现 manifest 拉取 / 签名校验 / 灰度判定（前置归 `rgs-asset-update`）。
//!
//! 硬约束（与本 crate 强相关，已通过 `grep` 验证）：
//! - **NFR-CDN-002** 整文件校验不可绕过 —— `integrity_gate.rs` 无任何 `skip_integrity` 旁路。
//! - **FR-CDN-064** 断点记录不含 PII —— `api.rs` / `state_machine.rs` 不引用 `player_id` 等。
//! - **FR-CDN-074** `If-Range: <ETag>` 强制 —— `range_client.rs` 仅接受 ETag，不接受 Last-Modified。
//! - **FR-CDN-083** 暂停时取消所有 in_flight Range —— `chunk_orchestrator.rs` 调用 `cancel_request`。
//! - **NFR-CDN-114** 任何 `DistributionBackend` 必须支持 HTTP Range —— 默认 HEAD 探测 + 416 fallback。
//!
//! 模块清单：
//! - [`api`]              公开 trait：`download_asset` / `pause_download` / `cancel_download` / `get_download_state`
//! - [`config`]           并发数 / 分片大小 / LRU / 断点 TTL 等运行时参数
//! - [`error`]            错误码（per DTL §6）
//! - [`state_machine`]    `DownloadStateMachine`（8 状态 + 19 转移表；M-2064.1 落定）
//! - [`range_client`]     `RangeClient`（M-2065.1~2）
//! - [`chunk_orchestrator`] `ChunkOrchestrator`（M-2065.3~4）
//! - [`integrity_gate`]   `IntegrityGate`（M-2065.5）
//! - [`platform`]         4 平台 sparse file 预分配（M-2065.6~7）
//! - [`resume_token`]     `ResumeToken` 13 字段断点记录（M-2064.2）
//! - [`resume_token_store`] `JsonFileResumeTokenStore` / `SqliteResumeTokenStore`（M-2064.3~5）
//! - [`metrics`]          10 项 `rgs_asset_download_*`（M-2065.8）

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod api;
pub mod chunk_orchestrator;
pub mod config;
pub mod error;
pub mod integrity_gate;
pub mod metrics;
pub mod platform;
pub mod range_client;
pub mod resume_token;
pub mod resume_token_store;
pub mod state_machine;

pub use api::{
    AssetDownloadService, CancelOutcome, DownloadProgress, DownloadRequest, DownloadStateView,
    PauseOutcome,
};
pub use chunk_orchestrator::{ChunkOrchestrator, ChunkSpec, InFlightChunk, PauseCancelSignal};
pub use config::{DownloadConfig, PlatformProfile};
pub use error::{AssetDownloadError, DownloadError, DownloadResult};
pub use integrity_gate::{IntegrityGate, IntegrityReport, IntegrityStatus};
pub use metrics::{encode_metrics_text, AssetDownloadMetrics, IntegrityOutcome};
pub use platform::{preallocate_sparse_file, PreallocateOutcome, PreallocateStrategy};
pub use range_client::{
    ContentRange, HttpRangeSpec, RangeBackendProbe, RangeClient, RangeClientConfig, RangeRequest,
    RangeResponse, RangeResponseDetailed,
};
pub use resume_token::{ResumeToken, ResumeTokenError};
pub use resume_token_store::{
    JsonFileResumeTokenStore, ResumeTokenStore, SqliteResumeTokenStore, DEFAULT_LRU_MAX_BYTES,
};
pub use state_machine::{
    allowed_events, allowed_transitions, next_state, DownloadState, DownloadStateMachine,
    StateEvent, StateTransition, TransitionError, TRANSITION_TABLE,
};
