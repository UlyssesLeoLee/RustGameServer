//! DownloadStateMachine —— 8 状态断点续传状态机
//!
//! 实现规格：RGS-SPEC-DTL-041 §3 + RGS-DTL-041 §5
//! 任务来源：RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.1
//!
//! ## 8 状态
//!
//! | 状态 | 含义 |
//! |---|---|
//! | `Idle` | 初始状态，token 已创建但未启动 |
//! | `Resolving` | 解析 manifest / 拉取 metadata |
//! | `Downloading` | HTTP Range 拉取中 |
//! | `Paused` | 已暂停（FR-CDN-083：in_flight 已取消）|
//! | `Completed` | 整文件校验通过（NFR-CDN-002），终态 |
//! | `Failed` | 失败（可重试或转 Expired）|
//! | `Cancelled` | 用户取消，终态 |
//! | `Expired` | 断点过期（>7 天）|
//!
//! ## 状态转移表（19 条合法转移）
//!
//! ```text
//! Idle       --ResolveStart  --> Resolving
//! Idle       --Cancel         --> Cancelled
//! Resolving  --ResolveSuccess --> Downloading
//! Resolving  --ResolveFail    --> Failed
//! Resolving  --Pause          --> Paused
//! Resolving  --Cancel         --> Cancelled
//! Downloading --Pause         --> Paused     (FR-CDN-083 取消 in_flight)
//! Downloading --Complete      --> Completed  (NFR-CDN-002 整文件校验通过)
//! Downloading --ChunkFail     --> Failed     (重试耗尽)
//! Downloading --EtagMismatch  --> Failed     (FR-CDN-074 全量重传)
//! Downloading --Cancel        --> Cancelled
//! Paused     --Resume         --> Downloading
//! Paused     --Cancel         --> Cancelled
//! Paused     --Expire         --> Expired
//! Failed     --Retry          --> Resolving
//! Failed     --Cancel         --> Cancelled
//! Failed     --Expire         --> Expired
//! Expired    --ResolveStart   --> Resolving  (重建 token)
//! Expired    --Cancel         --> Cancelled
//! Completed / Cancelled —— 终态，无出边
//! ```
//!
//! ## FR-CDN-083 集成
//!
//! `apply` 方法在状态转移为 `Paused` / `Cancelled` / `Failed` 时**必须**触发 cancel 信号：
//! - `cancel_flag: Arc<AtomicBool>` 翻为 `true`，供 in_flight task 轮询
//! - `cancel_notify: Arc<Notify>` 调 `notify_waiters()`，供 in_flight task 等待
//!
//! M-2065 `chunk_orchestrator` 通过这两个句柄监听取消。

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Notify;

// ---------------------------------------------------------------------------
// DownloadState —— 8 状态枚举
// ---------------------------------------------------------------------------

/// 8 状态枚举（per RGS-SPEC-DTL-041 §3 + RGS-DTL-041 §5）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    /// 初始状态，token 已创建但未启动
    Idle,
    /// 解析 manifest / 拉取 metadata
    Resolving,
    /// HTTP Range 拉取中
    Downloading,
    /// 已暂停（FR-CDN-083：in_flight 已取消）
    Paused,
    /// 整文件校验通过（NFR-CDN-002），终态
    Completed,
    /// 失败（可重试或转 Expired）
    Failed,
    /// 用户取消，终态
    Cancelled,
    /// 断点过期（>7 天）
    Expired,
}

impl DownloadState {
    /// 是否为终态（Completed / Cancelled）
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }

    /// 是否需要触发 FR-CDN-083 取消信号
    ///
    /// 进入 `Paused` / `Cancelled` / `Failed` 时必须取消所有 in_flight Range 请求。
    /// `Expired` 也应取消（过期 token 不应继续 in_flight）。
    pub const fn must_cancel_in_flight(self) -> bool {
        matches!(self, Self::Paused | Self::Cancelled | Self::Failed | Self::Expired)
    }

    /// 全部 8 个状态（用于穷尽测试 / 文档生成）
    pub const ALL: [DownloadState; 8] = [
        Self::Idle,
        Self::Resolving,
        Self::Downloading,
        Self::Paused,
        Self::Completed,
        Self::Failed,
        Self::Cancelled,
        Self::Expired,
    ];

    /// 状态名（lowercase，snake_case），用于 metrics label / log
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Resolving => "resolving",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }
}

impl fmt::Display for DownloadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// StateEvent —— 触发事件
// ---------------------------------------------------------------------------

/// 状态机事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateEvent {
    /// 解析启动（Idle / Expired -> Resolving）
    ResolveStart,
    /// 解析成功（Resolving -> Downloading）
    ResolveSuccess,
    /// 解析失败（Resolving -> Failed）
    ResolveFail,
    /// 暂停（Downloading / Resolving -> Paused；FR-CDN-083 触发 cancel）
    Pause,
    /// 恢复（Paused -> Downloading）
    Resume,
    /// 整文件校验通过（Downloading -> Completed；NFR-CDN-002）
    Complete,
    /// chunk 失败，重试耗尽（Downloading -> Failed）
    ChunkFail,
    /// ETag 不匹配（Downloading -> Failed；FR-CDN-074 全量重传）
    EtagMismatch,
    /// 用户取消（任意非终态 -> Cancelled）
    Cancel,
    /// 断点过期（Paused / Failed -> Expired）
    Expire,
    /// 重试（Failed -> Resolving）
    Retry,
}

impl StateEvent {
    /// 事件名（snake_case），用于 metrics label / log
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResolveStart => "resolve_start",
            Self::ResolveSuccess => "resolve_success",
            Self::ResolveFail => "resolve_fail",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Complete => "complete",
            Self::ChunkFail => "chunk_fail",
            Self::EtagMismatch => "etag_mismatch",
            Self::Cancel => "cancel",
            Self::Expire => "expire",
            Self::Retry => "retry",
        }
    }
}

impl fmt::Display for StateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// StateTransition —— 转移描述
// ---------------------------------------------------------------------------

/// 单次状态转移的描述（用于 metrics / 审计日志）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateTransition {
    /// 转移前状态
    pub from: DownloadState,
    /// 触发事件
    pub event: StateEvent,
    /// 转移后状态
    pub to: DownloadState,
}

impl StateTransition {
    /// 构造一条状态转移记录
    pub const fn new(from: DownloadState, event: StateEvent, to: DownloadState) -> Self {
        Self { from, event, to }
    }
}

// ---------------------------------------------------------------------------
// TransitionError —— 非法转移错误
// ---------------------------------------------------------------------------

/// 非法状态转移错误
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("illegal state transition: from={from}, event={event} (allowed: {allowed:?})")]
pub struct TransitionError {
    /// 当前状态
    pub from: DownloadState,
    /// 触发事件
    pub event: StateEvent,
    /// 该状态允许的事件列表（用于错误信息）
    pub allowed: Vec<StateEvent>,
}

impl TransitionError {
    /// 构造非法转移错误
    pub fn new(from: DownloadState, event: StateEvent) -> Self {
        Self {
            from,
            event,
            allowed: allowed_events(from),
        }
    }
}

// ---------------------------------------------------------------------------
// 转移表（核心：合法 from -> 事件 -> to）
// ---------------------------------------------------------------------------

/// 8 状态 × 11 事件 → 转移表
///
/// 返回 `(from, event)` 的目标状态；若不存在返回 `None`。
const TRANSITION_TABLE: &[(DownloadState, StateEvent, DownloadState)] = &[
    // --- Idle ---
    (DownloadState::Idle, StateEvent::ResolveStart, DownloadState::Resolving),
    (DownloadState::Idle, StateEvent::Cancel, DownloadState::Cancelled),
    // --- Resolving ---
    (DownloadState::Resolving, StateEvent::ResolveSuccess, DownloadState::Downloading),
    (DownloadState::Resolving, StateEvent::ResolveFail, DownloadState::Failed),
    (DownloadState::Resolving, StateEvent::Pause, DownloadState::Paused),
    (DownloadState::Resolving, StateEvent::Cancel, DownloadState::Cancelled),
    // --- Downloading ---
    (DownloadState::Downloading, StateEvent::Pause, DownloadState::Paused),
    (DownloadState::Downloading, StateEvent::Complete, DownloadState::Completed),
    (DownloadState::Downloading, StateEvent::ChunkFail, DownloadState::Failed),
    (DownloadState::Downloading, StateEvent::EtagMismatch, DownloadState::Failed),
    (DownloadState::Downloading, StateEvent::Cancel, DownloadState::Cancelled),
    // --- Paused ---
    (DownloadState::Paused, StateEvent::Resume, DownloadState::Downloading),
    (DownloadState::Paused, StateEvent::Cancel, DownloadState::Cancelled),
    (DownloadState::Paused, StateEvent::Expire, DownloadState::Expired),
    // --- Failed ---
    (DownloadState::Failed, StateEvent::Retry, DownloadState::Resolving),
    (DownloadState::Failed, StateEvent::Cancel, DownloadState::Cancelled),
    (DownloadState::Failed, StateEvent::Expire, DownloadState::Expired),
    // --- Expired ---
    (DownloadState::Expired, StateEvent::ResolveStart, DownloadState::Resolving),
    (DownloadState::Expired, StateEvent::Cancel, DownloadState::Cancelled),
    // --- Cancelled / Completed：无出边（终态）---
];

/// 在 `(from, event)` 上查表
pub fn next_state(from: DownloadState, event: StateEvent) -> Option<DownloadState> {
    TRANSITION_TABLE
        .iter()
        .find(|(f, e, _)| *f == from && *e == event)
        .map(|(_, _, t)| *t)
}

/// 给定状态返回所有允许的事件
pub fn allowed_events(from: DownloadState) -> Vec<StateEvent> {
    TRANSITION_TABLE
        .iter()
        .filter(|(f, _, _)| *f == from)
        .map(|(_, e, _)| *e)
        .collect()
}

/// 给定状态返回所有合法 `(event, to)` 对
pub fn allowed_transitions(from: DownloadState) -> Vec<(StateEvent, DownloadState)> {
    TRANSITION_TABLE
        .iter()
        .filter(|(f, _, _)| *f == from)
        .map(|(_, e, t)| (*e, *t))
        .collect()
}

// ---------------------------------------------------------------------------
// DownloadStateMachine —— 线程安全状态机实例
// ---------------------------------------------------------------------------

/// 线程安全的状态机实例
///
/// 设计：
/// - 状态本身用 `Mutex<DownloadState>` 保护（转移是 O(1) 操作，开销可忽略）
/// - FR-CDN-083 cancel 信号用 `Arc<AtomicBool>` 翻位 + `Arc<Notify>` 唤醒
/// - 通过 `Arc` 共享，让 in_flight 任务能持有 cancel 句柄
#[derive(Debug)]
pub struct DownloadStateMachine {
    state: Mutex<DownloadState>,
    cancel_flag: Arc<AtomicBool>,
    cancel_notify: Arc<Notify>,
}

impl Default for DownloadStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadStateMachine {
    /// 构造一个从 `Idle` 起始的状态机
    pub fn new() -> Self {
        Self::with_state(DownloadState::Idle)
    }

    /// 构造一个指定起始状态的状态机（用于恢复 checkpoint / 测试）
    ///
    /// 若初始状态本身需要取消 in_flight（`must_cancel_in_flight() == true`），
    /// 则 `cancel_flag` 初始化为 `true`（与"经过一次有效转移进入该状态"等价）。
    pub fn with_state(initial: DownloadState) -> Self {
        Self {
            state: Mutex::new(initial),
            cancel_flag: Arc::new(AtomicBool::new(initial.must_cancel_in_flight())),
            cancel_notify: Arc::new(Notify::new()),
        }
    }

    /// 读取当前状态
    pub fn current(&self) -> DownloadState {
        *self.state.lock().expect("state mutex poisoned")
    }

    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        self.current().is_terminal()
    }

    /// 应用一个事件，推进状态机
    ///
    /// 成功：返回新状态 + 触发（若需要）cancel 信号
    /// 失败：返回 `TransitionError`（不修改状态、不触发 cancel）
    pub fn apply(&self, event: StateEvent) -> Result<DownloadState, TransitionError> {
        let mut guard = self.state.lock().expect("state mutex poisoned");
        let from = *guard;
        let to = next_state(from, event).ok_or_else(|| TransitionError::new(from, event))?;

        *guard = to;

        // FR-CDN-083：进入 Paused / Cancelled / Failed / Expired 时必须取消 in_flight
        if to.must_cancel_in_flight() {
            self.cancel_flag.store(true, Ordering::SeqCst);
            self.cancel_notify.notify_waiters();
        }

        // Completed / Cancelled 之后进入终态：保留 cancel_flag 状态（重置时机由 store 决定）
        // 若从 Failed/Paused 走回 Downloading/Resolving，应重置 cancel_flag
        if matches!(to, DownloadState::Downloading | DownloadState::Resolving) {
            self.cancel_flag.store(false, Ordering::SeqCst);
        }

        Ok(to)
    }

    /// 取消标志（`Arc<AtomicBool>`），in_flight 任务可轮询
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel_flag)
    }

    /// 取消通知句柄（`Arc<Notify>`），in_flight 任务可 `.notified().await`
    pub fn cancel_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.cancel_notify)
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_states_are_defined() {
        // 强制覆盖 8 个状态，防止新增/遗漏
        let _states: [DownloadState; 8] = DownloadState::ALL;
        assert_eq!(DownloadState::ALL.len(), 8);
        // 命名
        assert_eq!(DownloadState::Idle.as_str(), "idle");
        assert_eq!(DownloadState::Resolving.as_str(), "resolving");
        assert_eq!(DownloadState::Downloading.as_str(), "downloading");
        assert_eq!(DownloadState::Paused.as_str(), "paused");
        assert_eq!(DownloadState::Completed.as_str(), "completed");
        assert_eq!(DownloadState::Failed.as_str(), "failed");
        assert_eq!(DownloadState::Cancelled.as_str(), "cancelled");
        assert_eq!(DownloadState::Expired.as_str(), "expired");
    }

    #[test]
    fn only_completed_and_cancelled_are_terminal() {
        for s in DownloadState::ALL {
            let expected = matches!(s, DownloadState::Completed | DownloadState::Cancelled);
            assert_eq!(s.is_terminal(), expected, "state={s:?}");
        }
    }

    #[test]
    fn must_cancel_in_flight_matches_spec() {
        // FR-CDN-083：Paused / Cancelled / Failed / Expired 都需要取消
        assert!(DownloadState::Paused.must_cancel_in_flight());
        assert!(DownloadState::Cancelled.must_cancel_in_flight());
        assert!(DownloadState::Failed.must_cancel_in_flight());
        assert!(DownloadState::Expired.must_cancel_in_flight());
        // Idle / Resolving / Downloading / Completed 不触发取消
        assert!(!DownloadState::Idle.must_cancel_in_flight());
        assert!(!DownloadState::Resolving.must_cancel_in_flight());
        assert!(!DownloadState::Downloading.must_cancel_in_flight());
        assert!(!DownloadState::Completed.must_cancel_in_flight());
    }

    #[test]
    fn legal_transition_downloading_to_paused() {
        // FR-CDN-083 关键路径
        let sm = DownloadStateMachine::new();
        sm.apply(StateEvent::ResolveStart).unwrap();
        sm.apply(StateEvent::ResolveSuccess).unwrap();
        assert_eq!(sm.current(), DownloadState::Downloading);
        // 关键转移
        let new_state = sm.apply(StateEvent::Pause).unwrap();
        assert_eq!(new_state, DownloadState::Paused);
        // cancel flag 已置位
        assert!(sm.cancel_flag().load(Ordering::SeqCst));
    }

    #[test]
    fn terminal_state_rejects_all_events() {
        let sm = DownloadStateMachine::with_state(DownloadState::Completed);
        for ev in [
            StateEvent::ResolveStart,
            StateEvent::Pause,
            StateEvent::Resume,
            StateEvent::Complete,
            StateEvent::ChunkFail,
            StateEvent::Cancel,
            StateEvent::Expire,
            StateEvent::Retry,
        ] {
            assert!(sm.apply(ev).is_err(), "Completed should reject {ev:?}");
        }
        let sm = DownloadStateMachine::with_state(DownloadState::Cancelled);
        for ev in [
            StateEvent::ResolveStart,
            StateEvent::Pause,
            StateEvent::Resume,
            StateEvent::Cancel,
        ] {
            assert!(sm.apply(ev).is_err(), "Cancelled should reject {ev:?}");
        }
    }

    #[test]
    fn illegal_transition_idle_to_downloading_is_rejected() {
        let sm = DownloadStateMachine::new();
        // Idle 不能直接 Complete（必须先 ResolveStart）
        let err = sm.apply(StateEvent::Complete).unwrap_err();
        assert_eq!(err.from, DownloadState::Idle);
        assert_eq!(err.event, StateEvent::Complete);
    }

    #[test]
    fn cancel_from_any_non_terminal_state_lands_on_cancelled() {
        for s in [
            DownloadState::Idle,
            DownloadState::Resolving,
            DownloadState::Downloading,
            DownloadState::Paused,
            DownloadState::Failed,
            DownloadState::Expired,
        ] {
            let sm = DownloadStateMachine::with_state(s);
            let to = sm.apply(StateEvent::Cancel).unwrap();
            assert_eq!(to, DownloadState::Cancelled, "from={s:?}");
        }
    }
}
