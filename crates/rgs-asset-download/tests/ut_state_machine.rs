//! UT：DownloadStateMachine 8 状态转移 + 非法转移负例
//!
//! 实现规格：RGS-SPEC-DTL-041 §6
//! 任务来源：RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.6
//!
//! 覆盖：
//! - 19 条合法转移（全部 8 状态 × 全部 11 事件的有效边）
//! - 非法转移负例（每状态至少 1 个非法事件）
//! - FR-CDN-083：进入 Paused / Cancelled / Failed / Expired 时 cancel 信号必须触发
//! - 终态：Completed / Cancelled 无出边
//! - 状态恢复：从任意非终态经过 Resume / Retry 可回到 Downloading / Resolving

use std::sync::atomic::Ordering;

use rgs_asset_download::{
    DownloadState, DownloadStateMachine, StateEvent, StateTransition, TransitionError,
    allowed_events, allowed_transitions, next_state,
};

// ---------------------------------------------------------------------------
// 19 条合法转移（per RGS-IMPL-PLAN-CDN-001 §3.2 + state_machine.rs TRANSITION_TABLE）
// ---------------------------------------------------------------------------

const LEGAL_TRANSITIONS: &[(DownloadState, StateEvent, DownloadState)] = &[
    // Idle
    (DownloadState::Idle, StateEvent::ResolveStart, DownloadState::Resolving),
    (DownloadState::Idle, StateEvent::Cancel, DownloadState::Cancelled),
    // Resolving
    (DownloadState::Resolving, StateEvent::ResolveSuccess, DownloadState::Downloading),
    (DownloadState::Resolving, StateEvent::ResolveFail, DownloadState::Failed),
    (DownloadState::Resolving, StateEvent::Pause, DownloadState::Paused),
    (DownloadState::Resolving, StateEvent::Cancel, DownloadState::Cancelled),
    // Downloading
    (DownloadState::Downloading, StateEvent::Pause, DownloadState::Paused),
    (DownloadState::Downloading, StateEvent::Complete, DownloadState::Completed),
    (DownloadState::Downloading, StateEvent::ChunkFail, DownloadState::Failed),
    (DownloadState::Downloading, StateEvent::EtagMismatch, DownloadState::Failed),
    (DownloadState::Downloading, StateEvent::Cancel, DownloadState::Cancelled),
    // Paused
    (DownloadState::Paused, StateEvent::Resume, DownloadState::Downloading),
    (DownloadState::Paused, StateEvent::Cancel, DownloadState::Cancelled),
    (DownloadState::Paused, StateEvent::Expire, DownloadState::Expired),
    // Failed
    (DownloadState::Failed, StateEvent::Retry, DownloadState::Resolving),
    (DownloadState::Failed, StateEvent::Cancel, DownloadState::Cancelled),
    (DownloadState::Failed, StateEvent::Expire, DownloadState::Expired),
    // Expired
    (DownloadState::Expired, StateEvent::ResolveStart, DownloadState::Resolving),
    (DownloadState::Expired, StateEvent::Cancel, DownloadState::Cancelled),
];

#[test]
fn transition_table_has_exactly_19_legal_entries() {
    assert_eq!(
        LEGAL_TRANSITIONS.len(),
        19,
        "transition table size drifted; please update this test + state_machine.rs"
    );
}

#[test]
fn every_legal_transition_lands_on_expected_state() {
    for (from, event, expected_to) in LEGAL_TRANSITIONS {
        let mut sm = DownloadStateMachine::with_state(*from);
        let got = sm.apply(*event).expect("legal transition must succeed");
        assert_eq!(
            got, *expected_to,
            "transition (from={from:?}, event={event:?}) should land on {expected_to:?}, got {got:?}"
        );
        assert_eq!(sm.current(), *expected_to, "current state should match");
    }
}

#[test]
fn next_state_lookup_table_matches_legal_transitions() {
    for (from, event, to) in LEGAL_TRANSITIONS {
        assert_eq!(
            next_state(*from, *event),
            Some(*to),
            "next_state({from:?}, {event:?}) mismatch"
        );
    }
}

#[test]
fn state_transition_record_carries_three_fields() {
    let t = StateTransition::new(
        DownloadState::Downloading,
        StateEvent::Pause,
        DownloadState::Paused,
    );
    assert_eq!(t.from, DownloadState::Downloading);
    assert_eq!(t.event, StateEvent::Pause);
    assert_eq!(t.to, DownloadState::Paused);
}

#[test]
fn allowed_events_returns_exact_set() {
    // Idle 仅允许 ResolveStart / Cancel
    let mut idle_events = allowed_events(DownloadState::Idle);
    idle_events.sort_by_key(|e| e.as_str());
    assert_eq!(
        idle_events,
        vec![StateEvent::Cancel, StateEvent::ResolveStart]
    );
    // Cancelled 无出边
    assert!(allowed_events(DownloadState::Cancelled).is_empty());
    // Completed 无出边
    assert!(allowed_events(DownloadState::Completed).is_empty());
}

#[test]
fn allowed_transitions_count_per_state() {
    // 各状态的出边数
    let cases = [
        (DownloadState::Idle, 2),
        (DownloadState::Resolving, 4),
        (DownloadState::Downloading, 5),
        (DownloadState::Paused, 3),
        (DownloadState::Failed, 3),
        (DownloadState::Expired, 2),
        (DownloadState::Cancelled, 0),
        (DownloadState::Completed, 0),
    ];
    for (state, expected) in cases {
        let got = allowed_transitions(state).len();
        assert_eq!(got, expected, "state={state:?}");
    }
}

// ---------------------------------------------------------------------------
// 非法转移负例（每状态至少 1 个）
// ---------------------------------------------------------------------------

#[test]
fn illegal_transitions_are_rejected_with_descriptive_error() {
    // Idle -> Complete（必须先 ResolveStart）
    let mut sm = DownloadStateMachine::new();
    let err = sm.apply(StateEvent::Complete).unwrap_err();
    let TransitionError { from, event, allowed } = err;
    assert_eq!(from, DownloadState::Idle);
    assert_eq!(event, StateEvent::Complete);
    assert!(allowed.contains(&StateEvent::ResolveStart));
    assert!(allowed.contains(&StateEvent::Cancel));
    // 非法转移不改状态
    assert_eq!(sm.current(), DownloadState::Idle);
}

#[test]
fn illegal_transition_paused_to_completed_directly_is_rejected() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Paused);
    // Paused 不能直接 Complete（必须先 Resume）
    let err = sm.apply(StateEvent::Complete).unwrap_err();
    assert_eq!(err.from, DownloadState::Paused);
    assert_eq!(err.event, StateEvent::Complete);
    assert_eq!(sm.current(), DownloadState::Paused);
}

#[test]
fn illegal_transition_resolving_to_completed_directly_is_rejected() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Resolving);
    let err = sm.apply(StateEvent::Complete).unwrap_err();
    assert_eq!(err.from, DownloadState::Resolving);
    assert_eq!(err.event, StateEvent::Complete);
}

#[test]
fn illegal_transition_failed_to_completed_is_rejected() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Failed);
    let err = sm.apply(StateEvent::Complete).unwrap_err();
    assert_eq!(err.from, DownloadState::Failed);
    assert!(sm.current() == DownloadState::Failed);
}

#[test]
fn illegal_transition_expired_to_paused_is_rejected() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Expired);
    // Expired 不能直接 Pause（必须先 ResolveStart）
    let err = sm.apply(StateEvent::Pause).unwrap_err();
    assert_eq!(err.from, DownloadState::Expired);
}

#[test]
fn terminal_states_reject_all_events() {
    // Completed 拒绝所有事件
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
        let mut sm = DownloadStateMachine::with_state(DownloadState::Completed);
        assert!(
            sm.apply(ev).is_err(),
            "Completed should reject event {ev:?}"
        );
    }
    // Cancelled 拒绝所有事件
    for ev in [
        StateEvent::ResolveStart,
        StateEvent::Pause,
        StateEvent::Resume,
        StateEvent::Complete,
        StateEvent::Cancel,
        StateEvent::Expire,
        StateEvent::Retry,
    ] {
        let mut sm = DownloadStateMachine::with_state(DownloadState::Cancelled);
        assert!(
            sm.apply(ev).is_err(),
            "Cancelled should reject event {ev:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// FR-CDN-083：进入 Paused / Cancelled / Failed / Expired 必须触发 cancel 信号
// ---------------------------------------------------------------------------

#[test]
fn fr_cdn_083_cancel_flag_set_on_paused() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    // 转移前 cancel flag 是 false
    assert!(!sm.cancel_flag().load(Ordering::SeqCst));
    sm.apply(StateEvent::Pause).unwrap();
    assert!(sm.cancel_flag().load(Ordering::SeqCst), "FR-CDN-083 violation");
}

#[test]
fn fr_cdn_083_cancel_flag_set_on_cancelled_from_downloading() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Cancel).unwrap();
    assert!(sm.cancel_flag().load(Ordering::SeqCst));
}

#[test]
fn fr_cdn_083_cancel_flag_set_on_failed() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::ChunkFail).unwrap();
    assert!(sm.cancel_flag().load(Ordering::SeqCst));
}

#[test]
fn fr_cdn_083_cancel_flag_set_on_expired() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Paused);
    sm.apply(StateEvent::Expire).unwrap();
    assert!(sm.cancel_flag().load(Ordering::SeqCst));
}

#[test]
fn fr_cdn_083_cancel_flag_reset_when_resuming_from_paused() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Paused);
    // 初始 Pause 状态：cancel flag 应为 true
    assert!(sm.cancel_flag().load(Ordering::SeqCst));
    // Resume → Downloading：cancel flag 应被重置
    sm.apply(StateEvent::Resume).unwrap();
    assert!(!sm.cancel_flag().load(Ordering::SeqCst));
}

#[test]
fn fr_cdn_083_cancel_flag_reset_when_retrying_from_failed() {
    let mut sm = DownloadStateMachine::with_state(DownloadState::Failed);
    assert!(sm.cancel_flag().load(Ordering::SeqCst));
    sm.apply(StateEvent::Retry).unwrap();
    // Retry → Resolving：cancel flag 应被重置
    assert!(!sm.cancel_flag().load(Ordering::SeqCst));
}

#[test]
fn fr_cdn_083_cancel_flag_not_set_on_completed() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Complete).unwrap();
    // Completed 不触发 cancel
    assert!(!sm.cancel_flag().load(Ordering::SeqCst));
}

#[tokio::test]
async fn fr_cdn_083_cancel_notify_signal_present() {
    use std::sync::Arc;
    use std::time::Duration;
    let mut sm = DownloadStateMachine::with_state(DownloadState::Downloading);
    let notify = sm.cancel_notify();
    // 在 notify 触发前先 spawn waiter（避免 race：Notify 单次触发，已触发后再 .notified() 不会唤醒）
    let waiter = {
        let notify = Arc::clone(&notify);
        tokio::spawn(async move {
            notify.notified().await;
        })
    };
    // 给 waiter 时间注册 future
    tokio::time::sleep(Duration::from_millis(20)).await;
    // 触发 Pause
    sm.apply(StateEvent::Pause).unwrap();
    // waiter 应该在 1s 内完成
    let result = tokio::time::timeout(Duration::from_secs(1), waiter).await;
    assert!(
        result.is_ok(),
        "cancel_notify should fire after Pause (got: {:?})",
        result
    );
    let join = result.unwrap();
    assert!(join.is_ok(), "waiter task panicked: {:?}", join);
}

// ---------------------------------------------------------------------------
// 端到端场景：正常下载 / 暂停恢复 / 失败重试 / 取消 / 过期
// ---------------------------------------------------------------------------

#[test]
fn scenario_happy_path_idle_to_completed() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Complete).unwrap();
    assert_eq!(sm.current(), DownloadState::Completed);
    assert!(sm.is_terminal());
}

#[test]
fn scenario_pause_resume_cycle() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Pause).unwrap();
    assert_eq!(sm.current(), DownloadState::Paused);
    sm.apply(StateEvent::Resume).unwrap();
    assert_eq!(sm.current(), DownloadState::Downloading);
    sm.apply(StateEvent::Complete).unwrap();
    assert_eq!(sm.current(), DownloadState::Completed);
}

#[test]
fn scenario_retry_after_failure() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::ChunkFail).unwrap();
    assert_eq!(sm.current(), DownloadState::Failed);
    sm.apply(StateEvent::Retry).unwrap();
    assert_eq!(sm.current(), DownloadState::Resolving);
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Complete).unwrap();
    assert_eq!(sm.current(), DownloadState::Completed);
}

#[test]
fn scenario_etag_mismatch_triggers_full_retransmit() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    // ETag 不匹配（per FR-CDN-074）→ Failed（不是 Cancelled）
    sm.apply(StateEvent::EtagMismatch).unwrap();
    assert_eq!(sm.current(), DownloadState::Failed);
    // 重试 → Resolving → Downloading → Complete
    sm.apply(StateEvent::Retry).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Complete).unwrap();
    assert_eq!(sm.current(), DownloadState::Completed);
}

#[test]
fn scenario_expire_after_pause() {
    let mut sm = DownloadStateMachine::new();
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Pause).unwrap();
    // 7 天后过期
    sm.apply(StateEvent::Expire).unwrap();
    assert_eq!(sm.current(), DownloadState::Expired);
    // 重新启动
    sm.apply(StateEvent::ResolveStart).unwrap();
    sm.apply(StateEvent::ResolveSuccess).unwrap();
    sm.apply(StateEvent::Complete).unwrap();
    assert_eq!(sm.current(), DownloadState::Completed);
}

#[test]
fn scenario_cancel_from_every_non_terminal_state() {
    let non_terminals = [
        DownloadState::Idle,
        DownloadState::Resolving,
        DownloadState::Downloading,
        DownloadState::Paused,
        DownloadState::Failed,
        DownloadState::Expired,
    ];
    for state in non_terminals {
        let mut sm = DownloadStateMachine::with_state(state);
        let to = sm.apply(StateEvent::Cancel).expect("cancel must succeed");
        assert_eq!(to, DownloadState::Cancelled, "from {state:?}");
        assert!(sm.is_terminal());
    }
}

// ---------------------------------------------------------------------------
// Display & 字符串化
// ---------------------------------------------------------------------------

#[test]
fn state_display_returns_snake_case_str() {
    assert_eq!(DownloadState::Idle.to_string(), "idle");
    assert_eq!(DownloadState::Resolving.to_string(), "resolving");
    assert_eq!(DownloadState::Downloading.to_string(), "downloading");
    assert_eq!(DownloadState::Paused.to_string(), "paused");
    assert_eq!(DownloadState::Completed.to_string(), "completed");
    assert_eq!(DownloadState::Failed.to_string(), "failed");
    assert_eq!(DownloadState::Cancelled.to_string(), "cancelled");
    assert_eq!(DownloadState::Expired.to_string(), "expired");
}

#[test]
fn event_display_returns_snake_case_str() {
    assert_eq!(StateEvent::ResolveStart.to_string(), "resolve_start");
    assert_eq!(StateEvent::ResolveSuccess.to_string(), "resolve_success");
    assert_eq!(StateEvent::ResolveFail.to_string(), "resolve_fail");
    assert_eq!(StateEvent::Pause.to_string(), "pause");
    assert_eq!(StateEvent::Resume.to_string(), "resume");
    assert_eq!(StateEvent::Complete.to_string(), "complete");
    assert_eq!(StateEvent::ChunkFail.to_string(), "chunk_fail");
    assert_eq!(StateEvent::EtagMismatch.to_string(), "etag_mismatch");
    assert_eq!(StateEvent::Cancel.to_string(), "cancel");
    assert_eq!(StateEvent::Expire.to_string(), "expire");
    assert_eq!(StateEvent::Retry.to_string(), "retry");
}

#[test]
fn state_serde_uses_snake_case() {
    let s = serde_json::to_string(&DownloadState::Downloading).unwrap();
    assert_eq!(s, "\"downloading\"");
    // 非法字符串应解析失败
    let res: Result<DownloadState, _> = serde_json::from_str("\"in_flight_download\"");
    assert!(res.is_err(), "invalid state name should fail to parse");
}

#[test]
fn state_all_constant_contains_eight_unique() {
    use std::collections::HashSet;
    let set: HashSet<DownloadState> = DownloadState::ALL.iter().copied().collect();
    assert_eq!(set.len(), 8);
}
