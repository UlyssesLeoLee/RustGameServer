//! `DownloadStateMachine` —— 8 状态机（per DTL §3 + M-2064.1）。
//!
//! **本文件为 M-2065.PREREQ 占位实现**（仅满足本 worktree 编译通过 + 给出
//! WF-1-2065 所需的 `get_download_state` 入口）。完整转移表 + fuzz UT 在
//! WF-1-2064 worktree（M-2064.1 / M-2064.6）落定，**merge 时如发现冲突由 Ulysses
//! 手工合并**。
//!
//! 8 状态（per DTL §3）：
//! - `Idle`        初始 / 已结束清理
//! - `Resolving`   拉取 manifest（实际归 `rgs-asset-update`，本 crate 仅作状态语义占位）
//! - `Downloading` 正在拉取分片
//! - `Paused`      用户暂停（断点已落盘）
//! - `Completed`   整文件校验通过
//! - `Failed`      失败（区分 transient / permanent）
//! - `Cancelled`   用户取消
//! - `Expired`     断点过期
//!
//! 硬约束（FR-CDN-064）：本文件**禁止**引用 `player_id` / `device_id` / `email` / `ip` / `mac`。
//! 硬约束（NFR-CDN-002）：不存在任何跳过整文件校验的旁路字段。

use std::fmt;

use serde::{Deserialize, Serialize};

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

impl fmt::Display for DownloadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Idle => "idle",
            Self::Resolving => "resolving",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        };
        f.write_str(s)
    }
}

/// 状态机动作（外部触发）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateTransition {
    /// 开始 / 恢复
    Start,
    /// 暂停
    Pause,
    /// 取消
    Cancel,
    /// 完成
    Complete,
    /// 失败
    Fail,
    /// 过期
    Expire,
    /// 重置
    Reset,
}

impl fmt::Display for StateTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Start => "start",
            Self::Pause => "pause",
            Self::Cancel => "cancel",
            Self::Complete => "complete",
            Self::Fail => "fail",
            Self::Expire => "expire",
            Self::Reset => "reset",
        };
        f.write_str(s)
    }
}

/// 状态机非法转移错误。
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("illegal state transition: from={from}, via={via}")]
pub struct StateTransitionError {
    /// 起始状态
    pub from: DownloadState,
    /// 触发的动作
    pub via: StateTransition,
}

/// 状态机（8 状态 + 转移表 + 终态检查）。
///
/// **PREREQ 阶段实现策略**：使用查表法（`fn next_state`），把转移规则集中在
/// `fn is_legal` 一处，便于 WF-1-2064 直接替换 / 扩展。
#[derive(Debug, Clone)]
pub struct DownloadStateMachine {
    state: DownloadState,
}

impl Default for DownloadStateMachine {
    fn default() -> Self {
        Self {
            state: DownloadState::Idle,
        }
    }
}

impl DownloadStateMachine {
    /// 新建初始状态机。
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前状态。
    pub fn state(&self) -> DownloadState {
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

    /// 判断转移是否合法。
    pub fn is_legal(from: DownloadState, via: StateTransition) -> bool {
        use DownloadState as S;
        use StateTransition as T;
        match (from, via) {
            // 进入 / 推进
            (S::Idle | S::Paused | S::Expired, T::Start) => true,
            (S::Resolving, T::Start) => true,
            (S::Downloading | S::Completed | S::Failed | S::Cancelled, T::Start) => false,
            // 暂停
            (S::Downloading | S::Resolving, T::Pause) => true,
            (S::Idle | S::Paused | S::Completed | S::Failed | S::Cancelled | S::Expired, T::Pause) => {
                false
            }
            // 取消（任何非 Completed 状态都可取消）
            (S::Completed, T::Cancel) => false,
            (_, T::Cancel) => true,
            // 完成
            (S::Downloading, T::Complete) => true,
            (S::Idle | S::Resolving | S::Paused | S::Completed | S::Failed | S::Cancelled | S::Expired, T::Complete) => {
                false
            }
            // 失败（任何非 Completed 状态都可失败）
            (S::Completed, T::Fail) => false,
            (_, T::Fail) => true,
            // 过期
            (S::Paused, T::Expire) => true,
            (S::Idle | S::Resolving | S::Downloading | S::Completed | S::Failed | S::Cancelled | S::Expired, T::Expire) => {
                false
            }
            // 重置（终态 → Idle）
            (S::Completed | S::Failed | S::Cancelled | S::Expired, T::Reset) => true,
            (_, T::Reset) => false,
        }
    }

    /// 计算下一状态（不修改自身）。
    pub fn next_state(
        from: DownloadState,
        via: StateTransition,
    ) -> Result<DownloadState, StateTransitionError> {
        if !Self::is_legal(from, via) {
            return Err(StateTransitionError { from, via });
        }
        let next = match via {
            StateTransition::Start => match from {
                DownloadState::Idle => DownloadState::Resolving,
                DownloadState::Paused | DownloadState::Expired => DownloadState::Resolving,
                DownloadState::Resolving => DownloadState::Downloading,
                _ => from,
            },
            StateTransition::Pause => DownloadState::Paused,
            StateTransition::Cancel => DownloadState::Cancelled,
            StateTransition::Complete => DownloadState::Completed,
            StateTransition::Fail => DownloadState::Failed,
            StateTransition::Expire => DownloadState::Expired,
            StateTransition::Reset => DownloadState::Idle,
        };
        Ok(next)
    }

    /// 应用一次转移。
    pub fn apply(&mut self, via: StateTransition) -> Result<DownloadState, StateTransitionError> {
        let next = Self::next_state(self.state, via)?;
        self.state = next;
        Ok(next)
    }

    /// 强制覆盖状态（仅限 PREREQ / 测试使用；生产路径必须经 `apply`）。
    pub fn override_state(&mut self, new_state: DownloadState) {
        self.state = new_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_states() {
        let mut sm = DownloadStateMachine::new();
        assert_eq!(sm.state(), DownloadState::Idle);
        assert!(!sm.is_terminal());
        sm.apply(StateTransition::Start).unwrap();
        sm.apply(StateTransition::Start).unwrap();
        sm.apply(StateTransition::Complete).unwrap();
        assert_eq!(sm.state(), DownloadState::Completed);
        assert!(sm.is_terminal());
    }

    #[test]
    fn cancel_from_terminal_completed_is_illegal() {
        // SPEC §3 + 转移表：Completed → Cancel 非法
        let from = DownloadState::Completed;
        assert!(!DownloadStateMachine::is_legal(from, StateTransition::Cancel));
    }

    #[test]
    fn pause_then_start_legal() {
        let mut sm = DownloadStateMachine::new();
        sm.apply(StateTransition::Start).unwrap(); // Idle -> Resolving
        sm.apply(StateTransition::Start).unwrap(); // Resolving -> Downloading
        sm.apply(StateTransition::Pause).unwrap();
        assert_eq!(sm.state(), DownloadState::Paused);
        sm.apply(StateTransition::Start).unwrap();
        assert_eq!(sm.state(), DownloadState::Resolving);
    }

    #[test]
    fn double_start_in_downloading_illegal() {
        let mut sm = DownloadStateMachine::new();
        sm.apply(StateTransition::Start).unwrap();
        sm.apply(StateTransition::Start).unwrap();
        // 现在是 Downloading；再次 Start 非法
        assert_eq!(
            sm.apply(StateTransition::Start).unwrap_err(),
            StateTransitionError {
                from: DownloadState::Downloading,
                via: StateTransition::Start,
            }
        );
    }
}
