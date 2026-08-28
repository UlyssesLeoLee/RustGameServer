//! `DownloadStateMachine` —— 8 状态机（per DTL §3 + M-2064.1 + SPEC §3）。
//!
//! ## 8 状态（per DTL §3）
//!
//! - `Idle`        初始 / 已结束清理
//! - `Resolving`   拉取 manifest（实际归 `rgs-asset-update`，本 crate 仅作状态语义占位）
//! - `Downloading` 正在拉取分片
//! - `Paused`      用户暂停（断点已落盘）
//! - `Completed`   整文件校验通过
//! - `Failed`      失败（区分 transient / permanent）
//! - `Cancelled`   用户取消
//! - `Expired`     断点过期
//!
//! ## 19 条合法转移（per RGS-IMPL-PLAN-CDN-001 §3.2 + state_machine.rs TRANSITION_TABLE）
//!
//! | from       | event                | to          |
//! |------------|----------------------|-------------|
//! | Idle       | ResolveStart         | Resolving   |
//! | Idle       | Cancel               | Cancelled   |
//! | Resolving  | ResolveSuccess       | Downloading |
//! | Resolving  | ResolveFail          | Failed      |
//! | Resolving  | Pause                | Paused      |
//! | Resolving  | Cancel               | Cancelled   |
//! | Downloading | Pause               | Paused      |
//! | Downloading | Complete            | Completed   |
//! | Downloading | ChunkFail           | Failed      |
//! | Downloading | EtagMismatch        | Failed      |
//! | Downloading | Cancel              | Cancelled   |
//! | Paused     | Resume               | Downloading |
//! | Paused     | Cancel               | Cancelled   |
//! | Paused     | Expire               | Expired     |
//! | Failed     | Retry                | Resolving   |
//! | Failed     | Cancel               | Cancelled   |
//! | Failed     | Expire               | Expired     |
//! | Expired    | ResolveStart         | Resolving   |
//! | Expired    | Cancel               | Cancelled   |
//!
//! 终态 `Completed` / `Cancelled` 无任何出边（per FR-CDN-083）。
//!
//! ## FR-CDN-083 取消信号
//!
//! 进入 `Paused` / `Cancelled` / `Failed` / `Expired` 任一终态时，`cancel_flag` (AtomicBool) 置 true，
//! `cancel_notify` (tokio::sync::Notify) 触发通知；离开 `Paused` / `Failed`（Resume / Retry）时
//! 重新置 false。`Completed` 路径**不**触发 cancel（正常完成）。
//!
//! ## 硬约束
//!
//! - **FR-CDN-064**：本文件**禁止**引用 `player_id` / `device_id` / `email` / `ip` / `mac`。
//! - **NFR-CDN-002**：不存在任何跳过整文件校验的旁路字段。

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

/// 8 状态枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    /// 初始 / 已清理
    Idle,
    /// 拉取 manifest（语义占位）
    Resolving,
    /// 正在拉取分片
    Downloading,
    /// 用户暂停（断点已落盘）
    Paused,
    /// 整文件校验通过 → 完成
    Completed,
    /// 失败
    Failed,
    /// 用户取消
    Cancelled,
    /// 断点过期
    Expired,
}

impl DownloadState {
    /// 全部 8 状态常量数组（per SPEC §3）
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

    /// snake_case 字符串表示（与 serde 序列化一致）
    pub fn as_str(&self) -> &'static str {
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

/// 11 种状态机事件（per RGS-IMPL-PLAN-CDN-001 §3.2）
///
/// 与旧 `StateTransition` enum 区分：事件是**触发**（"用户/系统做了什么"），
/// 状态是**结果**（"现在是什么"）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateEvent {
    /// 启动 manifest 解析
    ResolveStart,
    /// manifest 解析成功
    ResolveSuccess,
    /// manifest 解析失败
    ResolveFail,
    /// 暂停
    Pause,
    /// 恢复（从 Paused）
    Resume,
    /// 完成
    Complete,
    /// 分片下载失败
    ChunkFail,
    /// ETag 不匹配（per FR-CDN-074）
    EtagMismatch,
    /// 取消
    Cancel,
    /// 过期
    Expire,
    /// 重试（从 Failed）
    Retry,
}

impl StateEvent {
    /// snake_case 字符串表示
    pub fn as_str(&self) -> &'static str {
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

/// 一次状态转移的记录（from / event / to 三元组）。
///
/// 用于审计 / 日志 / metrics 标签，不参与 `apply` / `next_state` 的主流程。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransition {
    /// 起始状态
    pub from: DownloadState,
    /// 触发的事件
    pub event: StateEvent,
    /// 目标状态
    pub to: DownloadState,
}

impl StateTransition {
    /// 构造一次状态转移记录。
    pub fn new(from: DownloadState, event: StateEvent, to: DownloadState) -> Self {
        Self { from, event, to }
    }
}

/// 状态机非法转移错误。
///
/// - `from`：起始状态
/// - `event`：触发的非法事件
/// - `allowed`：从 `from` 状态出发的合法事件列表（用于提示可恢复路径）
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("illegal state transition: from={from}, event={event}, allowed={allowed:?}")]
pub struct TransitionError {
    /// 起始状态
    pub from: DownloadState,
    /// 触发的事件
    pub event: StateEvent,
    /// 合法事件列表
    pub allowed: Vec<StateEvent>,
}

// ---------------------------------------------------------------------------
// 转移表（per SPEC §3 + IMPL-PLAN §3.2 + tests/ut_state_machine.rs LEGAL_TRANSITIONS）
// ---------------------------------------------------------------------------

/// `(from, event, to)` 三元组构成的转移表。
pub const TRANSITION_TABLE: &[(DownloadState, StateEvent, DownloadState)] = &[
    // Idle
    (DownloadState::Idle, StateEvent::ResolveStart, DownloadState::Resolving),
    (DownloadState::Idle, StateEvent::Cancel, DownloadState::Cancelled),
    // Resolving
    (
        DownloadState::Resolving,
        StateEvent::ResolveSuccess,
        DownloadState::Downloading,
    ),
    (
        DownloadState::Resolving,
        StateEvent::ResolveFail,
        DownloadState::Failed,
    ),
    (DownloadState::Resolving, StateEvent::Pause, DownloadState::Paused),
    (DownloadState::Resolving, StateEvent::Cancel, DownloadState::Cancelled),
    // Downloading
    (
        DownloadState::Downloading,
        StateEvent::Pause,
        DownloadState::Paused,
    ),
    (
        DownloadState::Downloading,
        StateEvent::Complete,
        DownloadState::Completed,
    ),
    (
        DownloadState::Downloading,
        StateEvent::ChunkFail,
        DownloadState::Failed,
    ),
    (
        DownloadState::Downloading,
        StateEvent::EtagMismatch,
        DownloadState::Failed,
    ),
    (
        DownloadState::Downloading,
        StateEvent::Cancel,
        DownloadState::Cancelled,
    ),
    // Paused
    (DownloadState::Paused, StateEvent::Resume, DownloadState::Downloading),
    (DownloadState::Paused, StateEvent::Cancel, DownloadState::Cancelled),
    (DownloadState::Paused, StateEvent::Expire, DownloadState::Expired),
    // Failed
    (DownloadState::Failed, StateEvent::Retry, DownloadState::Resolving),
    (DownloadState::Failed, StateEvent::Cancel, DownloadState::Cancelled),
    (DownloadState::Failed, StateEvent::Expire, DownloadState::Expired),
    // Expired
    (
        DownloadState::Expired,
        StateEvent::ResolveStart,
        DownloadState::Resolving,
    ),
    (DownloadState::Expired, StateEvent::Cancel, DownloadState::Cancelled),
];

/// 静态查询：从 `from` 状态出发的合法事件列表。
pub fn allowed_events(from: DownloadState) -> Vec<StateEvent> {
    let mut out: Vec<StateEvent> = TRANSITION_TABLE
        .iter()
        .filter(|(s, _, _)| *s == from)
        .map(|(_, e, _)| *e)
        .collect();
    out.sort_by_key(|e| e.as_str());
    out
}

/// 静态查询：从 `from` 状态出发的合法 `(event, to)` 列表。
pub fn allowed_transitions(from: DownloadState) -> Vec<(StateEvent, DownloadState)> {
    let mut out: Vec<(StateEvent, DownloadState)> = TRANSITION_TABLE
        .iter()
        .filter(|(s, _, _)| *s == from)
        .map(|(_, e, t)| (*e, *t))
        .collect();
    out.sort_by_key(|(e, _)| e.as_str());
    out
}

/// 静态查询：根据 `(from, event)` 计算下一状态；非法返回 `None`。
pub fn next_state(from: DownloadState, event: StateEvent) -> Option<DownloadState> {
    TRANSITION_TABLE
        .iter()
        .find(|(s, e, _)| *s == from && *e == event)
        .map(|(_, _, t)| *t)
}

/// 状态机（8 状态 + 19 转移表 + 终态检查 + 取消信号）。
///
/// 内部包含：
/// - `state`：当前状态
/// - `cancel_flag`：FR-CDN-083 取消标志（Arc<AtomicBool>）
/// - `cancel_notify`：FR-CDN-083 取消异步通知（Arc<Notify>）
#[derive(Debug)]
pub struct DownloadStateMachine {
    state: DownloadState,
    cancel_flag: Arc<AtomicBool>,
    cancel_notify: Arc<Notify>,
}

impl Default for DownloadStateMachine {
    fn default() -> Self {
        Self {
            state: DownloadState::Idle,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            cancel_notify: Arc::new(Notify::new()),
        }
    }
}

impl Clone for DownloadStateMachine {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            cancel_flag: Arc::clone(&self.cancel_flag),
            cancel_notify: Arc::clone(&self.cancel_notify),
        }
    }
}

impl DownloadStateMachine {
    /// 新建初始状态机（Idle）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 以指定状态构造（用于测试 / 序列化恢复）。
    pub fn with_state(state: DownloadState) -> Self {
        let s = Self {
            state,
            ..Self::default()
        };
        // Paused / Failed / Expired / Cancelled 终态（部分）应有 cancel 信号
        match state {
            DownloadState::Paused
            | DownloadState::Failed
            | DownloadState::Expired
            | DownloadState::Cancelled => {
                s.cancel_flag.store(true, Ordering::SeqCst);
            }
            _ => {}
        }
        s
    }

    /// 当前状态。
    pub fn state(&self) -> DownloadState {
        self.state
    }

    /// 当前状态（alias for `state()`，匹配 `ut_state_machine.rs` 命名）。
    pub fn current(&self) -> DownloadState {
        self.state
    }

    /// 是否处于终态（Completed / Failed / Cancelled / Expired）。
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Completed
                | DownloadState::Failed
                | DownloadState::Cancelled
                | DownloadState::Expired
        )
    }

    /// FR-CDN-083 取消标志引用。外部可读、检测取消信号。
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel_flag)
    }

    /// FR-CDN-083 取消异步通知。await 此 `Notify` 即可等待 cancel 触发。
    pub fn cancel_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.cancel_notify)
    }

    /// 应用一次事件（事件驱动 API，per ut_state_machine.rs）。
    ///
    /// 成功时返回新状态；非法时返回 `TransitionError`。
    pub fn apply(&mut self, event: StateEvent) -> Result<DownloadState, TransitionError> {
        let next = next_state(self.state, event).ok_or_else(|| TransitionError {
            from: self.state,
            event,
            allowed: allowed_events(self.state),
        })?;
        self.apply_internal(next, event);
        Ok(next)
    }

    /// 应用一次事件（alias for `apply()`，保留以兼容不同命名）。
    pub fn transition_event(&mut self, event: StateEvent) -> Result<DownloadState, TransitionError> {
        self.apply(event)
    }

    /// 强制转移（chaos / IT 用：直接设置目标状态，返回是否"合法"）。
    ///
    /// 合法定义：从 `current` 状态出发，存在任意事件能到达 `target`。
    /// 非法时状态不变，返回 false。
    pub fn transition(&mut self, target: DownloadState) -> bool {
        if self.state == target {
            return true;
        }
        if let Some((event, _)) = allowed_transitions(self.state)
            .into_iter()
            .find(|(_, t)| *t == target)
        {
            self.apply_internal(target, event);
            true
        } else {
            false
        }
    }

    /// 内部：完成一次状态切换并触发 / 重置 cancel 信号。
    fn apply_internal(&mut self, next: DownloadState, _via: StateEvent) {
        self.state = next;
        match next {
            // 进入会触发 cancel 通知的"准终态"
            DownloadState::Paused
            | DownloadState::Cancelled
            | DownloadState::Failed
            | DownloadState::Expired => {
                self.cancel_flag.store(true, Ordering::SeqCst);
                self.cancel_notify.notify_waiters();
            }
            // Completed 不触发 cancel（正常完成）
            DownloadState::Completed => {
                // cancel flag 保持 false
            }
            // 离开 Paused / Failed → Resolving / Downloading：重置 cancel
            DownloadState::Resolving | DownloadState::Downloading | DownloadState::Idle => {
                self.cancel_flag.store(false, Ordering::SeqCst);
            }
        }
    }

    /// 强制覆盖状态（仅限 PREREQ / 测试使用；生产路径必须经 `apply` / `transition`）。
    pub fn override_state(&mut self, new_state: DownloadState) {
        self.state = new_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_machine_is_idle() {
        let sm = DownloadStateMachine::new();
        assert_eq!(sm.state(), DownloadState::Idle);
        assert_eq!(sm.current(), DownloadState::Idle);
        assert!(!sm.is_terminal());
        assert!(!sm.cancel_flag().load(Ordering::SeqCst));
    }

    #[test]
    fn transition_table_size_is_19() {
        assert_eq!(TRANSITION_TABLE.len(), 19);
        assert_eq!(DownloadState::ALL.len(), 8);
    }

    #[test]
    fn idle_to_resolving_to_completed_happy_path() {
        let mut sm = DownloadStateMachine::new();
        assert_eq!(
            sm.apply(StateEvent::ResolveStart).unwrap(),
            DownloadState::Resolving
        );
        assert_eq!(
            sm.apply(StateEvent::ResolveSuccess).unwrap(),
            DownloadState::Downloading
        );
        assert_eq!(
            sm.apply(StateEvent::Complete).unwrap(),
            DownloadState::Completed
        );
        assert!(sm.is_terminal());
        // Completed 不触发 cancel
        assert!(!sm.cancel_flag().load(Ordering::SeqCst));
    }

    #[test]
    fn illegal_transition_returns_descriptive_error() {
        let mut sm = DownloadStateMachine::new();
        let err = sm.apply(StateEvent::Complete).unwrap_err();
        assert_eq!(err.from, DownloadState::Idle);
        assert_eq!(err.event, StateEvent::Complete);
        assert!(err.allowed.contains(&StateEvent::ResolveStart));
        assert!(err.allowed.contains(&StateEvent::Cancel));
    }

    #[test]
    fn terminal_states_reject_all_events() {
        for terminal in [DownloadState::Completed, DownloadState::Cancelled] {
            let mut sm = DownloadStateMachine::with_state(terminal);
            for ev in [
                StateEvent::ResolveStart,
                StateEvent::ResolveSuccess,
                StateEvent::ResolveFail,
                StateEvent::Pause,
                StateEvent::Resume,
                StateEvent::Complete,
                StateEvent::ChunkFail,
                StateEvent::EtagMismatch,
                StateEvent::Cancel,
                StateEvent::Expire,
                StateEvent::Retry,
            ] {
                assert!(sm.apply(ev).is_err(), "{terminal:?} should reject {ev:?}");
            }
        }
    }

    #[test]
    fn fr_cdn_083_cancel_set_on_paused_reset_on_resume() {
        let mut sm = DownloadStateMachine::new();
        sm.apply(StateEvent::ResolveStart).unwrap();
        sm.apply(StateEvent::ResolveSuccess).unwrap();
        sm.apply(StateEvent::Pause).unwrap();
        assert!(sm.cancel_flag().load(Ordering::SeqCst));
        sm.apply(StateEvent::Resume).unwrap();
        assert!(!sm.cancel_flag().load(Ordering::SeqCst));
    }
}
