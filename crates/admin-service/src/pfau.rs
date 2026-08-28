//! Admin 域 PFAU 批次状态机(per RGS-DTL-031 §4.2)
//!
//! ## 状态枚举(per DTL-031 §4.2)
//! - Declared
//! - CanaryInProgress
//! - CanaryConfirmed (当前批次全部目标节点 ACK)
//! - Observing (观察窗口)
//! - Paused (超时、健康丢失、fencing 失败、目标集合变化)
//! - Retrying (人工选择重试)
//! - RollingBack (人工选择回滚,或自动回滚)
//! - Aborted (人工终止)
//! - Completed (全部批次完成,更新 current_version)

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PfauState {
    Declared,
    CanaryInProgress,
    CanaryConfirmed,
    Observing,
    Paused,
    Retrying,
    RollingBack,
    Aborted,
    Completed,
}

impl fmt::Display for PfauState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Declared => "declared",
            Self::CanaryInProgress => "canary_in_progress",
            Self::CanaryConfirmed => "canary_confirmed",
            Self::Observing => "observing",
            Self::Paused => "paused",
            Self::Retrying => "retrying",
            Self::RollingBack => "rolling_back",
            Self::Aborted => "aborted",
            Self::Completed => "completed",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PfauError {
    InvalidTransition { from: PfauState, to: PfauState },
}

/// 合法转移表(per DTL-031 §4.2 文本图)
///
/// declared -> canary_in_progress
/// declared -> canary_confirmed
/// canary_in_progress -> canary_confirmed
/// canary_in_progress -> paused
/// canary_confirmed -> observing
/// canary_confirmed -> paused
/// observing -> canary_in_progress (还有下一批)
/// observing -> completed (全部批次完成)
/// observing -> paused
/// paused -> retrying
/// paused -> rolling_back
/// paused -> aborted
/// retrying -> canary_in_progress
/// rolling_back -> aborted
/// rolling_back -> completed
pub fn can_transition(from: PfauState, to: PfauState) -> bool {
    use PfauState::*;
    matches!(
        (from, to),
        (Declared, CanaryInProgress)
            | (Declared, CanaryConfirmed)
            | (CanaryInProgress, CanaryConfirmed)
            | (CanaryInProgress, Paused)
            | (CanaryConfirmed, Observing)
            | (CanaryConfirmed, Paused)
            | (Observing, CanaryInProgress)
            | (Observing, Completed)
            | (Observing, Paused)
            | (Paused, Retrying)
            | (Paused, RollingBack)
            | (Paused, Aborted)
            | (Retrying, CanaryInProgress)
            | (RollingBack, Aborted)
            | (RollingBack, Completed)
    )
}

/// 尝试转移,失败返回 PfauError
pub fn try_transition(from: PfauState, to: PfauState) -> Result<PfauState, PfauError> {
    if can_transition(from, to) {
        Ok(to)
    } else {
        Err(PfauError::InvalidTransition { from, to })
    }
}

/// 当前批次的 ACK 计数
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CanaryAck {
    pub total_nodes: u32,
    pub acked_nodes: u32,
}

impl CanaryAck {
    pub fn new(total: u32) -> Self {
        Self { total_nodes: total, acked_nodes: 0 }
    }
    pub fn record_ack(&mut self) {
        self.acked_nodes = (self.acked_nodes + 1).min(self.total_nodes);
    }
    /// all-reachable 规则(per DTL-031 §4.3):必须全部 ACK 才进 CanaryConfirmed
    pub fn is_all_acked(&self) -> bool {
        self.total_nodes > 0 && self.acked_nodes >= self.total_nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PfauState::*;

    #[test]
    fn declared_can_advance_to_canary_in_progress() {
        assert!(can_transition(Declared, CanaryInProgress));
    }

    #[test]
    fn declared_cannot_jump_to_completed() {
        assert!(!can_transition(Declared, Completed));
    }

    #[test]
    fn canary_in_progress_can_pause() {
        assert!(can_transition(CanaryInProgress, Paused));
    }

    #[test]
    fn observing_can_return_to_canary_in_progress() {
        // 还有下一批
        assert!(can_transition(Observing, CanaryInProgress));
    }

    #[test]
    fn observing_can_complete() {
        assert!(can_transition(Observing, Completed));
    }

    #[test]
    fn paused_can_retry_rollback_abort() {
        assert!(can_transition(Paused, Retrying));
        assert!(can_transition(Paused, RollingBack));
        assert!(can_transition(Paused, Aborted));
    }

    #[test]
    fn rolling_back_can_complete() {
        assert!(can_transition(RollingBack, Completed));
    }

    #[test]
    fn completed_is_terminal() {
        // Completed 不能再转移
        for to in [Declared, CanaryInProgress, CanaryConfirmed, Observing, Paused, Retrying, RollingBack, Aborted, Completed] {
            assert!(!can_transition(Completed, to), "Completed -> {} should be invalid", to);
        }
    }

    #[test]
    fn aborted_is_terminal() {
        for to in [Declared, CanaryInProgress, CanaryConfirmed, Observing, Paused, Retrying, RollingBack, Aborted, Completed] {
            assert!(!can_transition(Aborted, to), "Aborted -> {} should be invalid", to);
        }
    }

    #[test]
    fn canary_ack_all_acked_only_when_total_acked() {
        let mut ack = CanaryAck::new(5);
        assert!(!ack.is_all_acked());
        for _ in 0..4 {
            ack.record_ack();
            assert!(!ack.is_all_acked());
        }
        ack.record_ack();
        assert!(ack.is_all_acked());
    }

    #[test]
    fn canary_ack_zero_total_is_not_all_acked() {
        // 边界:total=0 时不能算 all-reached(避免空批误判)
        let ack = CanaryAck::new(0);
        assert!(!ack.is_all_acked());
    }
}
