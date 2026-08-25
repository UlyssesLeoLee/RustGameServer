//! `DownloadStateMachine` 8 状态 + 转移表（per SPEC §3 + DTL §3.4）
//!
//! 8 状态：Idle / Resolving / Downloading / Paused / Completed / Failed / Cancelled / Expired
//! 转移合法性：详见 SPEC-DTL-041 v0.2 §6 + DTL §3.4 状态机图

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DownloadState {
    Idle,
    Resolving,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

impl DownloadState {
    /// 全部 8 个状态
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

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Resolving => "Resolving",
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::Expired => "Expired",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: DownloadState,
    pub to: DownloadState,
}

/// 状态机：8 状态 + 合法转移表
#[derive(Debug, Default, Clone)]
pub struct DownloadStateMachine {
    current: Option<DownloadState>,
}

impl DownloadStateMachine {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn current(&self) -> Option<DownloadState> {
        self.current
    }

    /// 转移：若 from 与 current 不一致返回 false；to 本身不查合法性
    /// （由调用方按 SPEC §3 合法转移表保证）
    pub fn transition(&mut self, to: DownloadState) -> bool {
        let Some(from) = self.current else {
            // 起始：仅 Idle 合法
            if matches!(to, DownloadState::Idle) {
                self.current = Some(to);
                return true;
            }
            return false;
        };
        if Self::is_legal(from, to) {
            self.current = Some(to);
            true
        } else {
            false
        }
    }

    /// 合法转移表（per SPEC-DTL-041 v0.2 §6）
    pub fn is_legal(from: DownloadState, to: DownloadState) -> bool {
        use DownloadState::*;
        matches!(
            (from, to),
            (Idle, Resolving)
                | (Resolving, Downloading)
                | (Downloading, Paused)
                | (Downloading, Completed)
                | (Downloading, Failed)
                | (Paused, Downloading)
                | (Paused, Cancelled)
                | (Paused, Expired)
                | (Failed, Resolving)
                | (Cancelled, Idle)
                | (Expired, Idle)
        )
    }
}
