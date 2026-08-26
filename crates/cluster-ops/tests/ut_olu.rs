//! M-2071.7 UT：OLU 上报 NFR-LCM-007 硬约束验证
//! （per RGS-SPEC-DTL-042 §3 + §8 + NFR-LCM-007 必经 rgs-arc-olu）

use std::sync::Arc;

use cluster_ops::realm_lifecycle::olu_reporter::{
    FakeOluChannel, OluPhase, OluReport, OluReporter,
};
use cluster_ops::realm_lifecycle::operators::OperatorInput;
use rgs_arc_olu::{OluClient, OluRequest, OluResponse};
use uuid::Uuid;

// ============================================================================
// NFR-LCM-007 硬约束：OLU 上报必经 rgs-arc-olu
// ============================================================================

#[test]
fn olu_reporter_uses_rgs_arc_olu_client_trait() {
    // 构造 + 通过 rgs_arc_olu::OluClient trait 调用
    let channel: Arc<dyn OluClient> = Arc::new(FakeOluChannel::always_ok());
    let r = OluReporter::new("platform", channel);
    assert_eq!(r.team(), "platform");
}

#[test]
fn olu_reporter_records_history_on_success() {
    let r = OluReporter::new_for_test("platform");
    let input = OperatorInput {
        request_id: Uuid::new_v4(),
        operator_id: Uuid::new_v4(),
        approval_ref: None,
        trace_id: "trace-1".to_string(),
    };
    let res = futures_lite::future::block_on(r.report_phase_start(
        "new_realm",
        "realm-1",
        &input,
    ));
    assert!(res.is_ok());
    let history = r.history();
    assert_eq!(history.len(), 1);
    let h = &history[0];
    assert_eq!(h.phase, OluPhase::NewRealm);
    assert_eq!(h.realm_id, "realm-1");
    assert_eq!(h.team, "platform");
    assert!(h.token_budget > 0);
}

// ============================================================================
// 6 阶段 OLU 默认值（per SPEC §8 TBD-LCM-007 PH-4 实测填）
// ============================================================================

#[test]
fn olu_six_phases_all_have_default_budget() {
    // per SPEC §8：6 阶段 OLU 默认值（PH-4 实测填）
    assert_eq!(OluPhase::ALL.len(), 6);
    for p in OluPhase::ALL {
        assert!(p.default_olu_budget() > 0);
    }
}

#[test]
fn olu_six_phase_default_budgets_match_spec() {
    // per SPEC §8 TBD-LCM-007 默认值（6 阶段；merge_rollback 走 merge）
    assert_eq!(OluPhase::NewRealm.default_olu_budget(), 4_000_000);
    assert_eq!(OluPhase::Scale.default_olu_budget(), 2_000_000);
    assert_eq!(OluPhase::Split.default_olu_budget(), 6_000_000);
    assert_eq!(OluPhase::Merge.default_olu_budget(), 8_000_000);
    assert_eq!(OluPhase::Retire.default_olu_budget(), 3_000_000);
    assert_eq!(OluPhase::Archive.default_olu_budget(), 5_000_000);
}

#[test]
fn olu_six_phases_as_str_match_subfeature() {
    // OLU phase 名与 SubFeature 6 阶段 phase_name 一致（merge_rollback 不计）
    assert_eq!(OluPhase::NewRealm.as_str(), "new_realm");
    assert_eq!(OluPhase::Scale.as_str(), "scale");
    assert_eq!(OluPhase::Split.as_str(), "split");
    assert_eq!(OluPhase::Merge.as_str(), "merge");
    assert_eq!(OluPhase::Retire.as_str(), "retire");
    assert_eq!(OluPhase::Archive.as_str(), "archive");
}

#[test]
fn olu_parse_phase_all_six_match() {
    for p in OluPhase::ALL {
        assert_eq!(OluReporter::parse_phase(p.as_str()), Some(*p));
    }
    assert!(OluReporter::parse_phase("unknown").is_none());
}

// ============================================================================
// NFR-LCM-007 上报失败 / 通道不可用：fail-closed
// ============================================================================

#[test]
fn olu_reporter_fails_closed_when_channel_rejects() {
    // rgs-arc-olu 拒绝时 OluReporter 必须 fail-closed（NFR-LCM-007 硬约束）
    let channel: Arc<dyn OluClient> = Arc::new(FakeOluChannel::always_fail());
    let r = OluReporter::new("platform", channel);
    let input = OperatorInput {
        request_id: Uuid::new_v4(),
        operator_id: Uuid::new_v4(),
        approval_ref: None,
        trace_id: "trace-2".to_string(),
    };
    let res = futures_lite::future::block_on(r.report_phase_start(
        "merge",
        "realm-x",
        &input,
    ));
    assert!(res.is_err());
    // 拒绝时**不**记录 history（审计正确性）
    assert!(r.history().is_empty());
}

#[test]
fn olu_reporter_rejects_unknown_phase() {
    let r = OluReporter::new_for_test("platform");
    let input = OperatorInput {
        request_id: Uuid::new_v4(),
        operator_id: Uuid::new_v4(),
        approval_ref: None,
        trace_id: "trace-3".to_string(),
    };
    let res = futures_lite::future::block_on(r.report_phase_start(
        "not_a_phase",
        "realm-y",
        &input,
    ));
    assert!(res.is_err());
}

// ============================================================================
// 7 子类必须全部能 OLU 上报（per M-2071.4 + SPEC §8）
// ============================================================================

#[test]
fn olu_reporter_accepts_all_seven_required_phases() {
    let r = OluReporter::new_for_test("platform");
    for phase in [
        "new_realm",
        "scale",
        "split",
        "merge",
        "merge_rollback",
        "retire",
        "archive",
    ] {
        let input = OperatorInput {
            request_id: Uuid::new_v4(),
            operator_id: Uuid::new_v4(),
            approval_ref: None,
            trace_id: format!("trace-{}", phase),
        };
        let res = futures_lite::future::block_on(r.report_phase_start(
            phase,
            "realm-z",
            &input,
        ));
        assert!(res.is_ok(), "phase {} should report ok", phase);
    }
    assert_eq!(r.history().len(), 7);
}

// ============================================================================
// NFR-LCM-007：rgs-arc-olu trait 形状契约
// ============================================================================

#[test]
fn olu_request_response_contain_required_fields() {
    // OluRequest / OluResponse 必含字段（rgs-arc-olu 既定契约）
    let req = OluRequest {
        phase: "new_realm".to_string(),
        realm_id: "realm-1".to_string(),
        team: "platform".to_string(),
        request_id: "req-1".to_string(),
        operator_id: "op-1".to_string(),
        trace_id: "trace-1".to_string(),
        token_budget: 4_000_000,
    };
    assert_eq!(req.phase, "new_realm");
    assert_eq!(req.token_budget, 4_000_000);

    let resp = OluResponse {
        accepted: true,
        reason: None,
    };
    assert!(resp.accepted);
    assert!(resp.reason.is_none());
}

// ============================================================================
// 报告历史：审计 + 7 子类各一次
// ============================================================================

#[test]
fn olu_report_history_contains_correct_team_and_phase() {
    let r = OluReporter::new_for_test("sre");
    let input = OperatorInput {
        request_id: Uuid::new_v4(),
        operator_id: Uuid::new_v4(),
        approval_ref: Some("approval-1".to_string()),
        trace_id: "trace-sre".to_string(),
    };
    futures_lite::future::block_on(r.report_phase_start("retire", "realm-r", &input))
        .unwrap();
    let h: OluReport = r.history().into_iter().next().unwrap();
    assert_eq!(h.team, "sre");
    assert_eq!(h.phase, OluPhase::Retire);
    assert_eq!(h.realm_id, "realm-r");
}
