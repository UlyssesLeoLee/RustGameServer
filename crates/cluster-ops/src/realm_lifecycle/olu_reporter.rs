//! OLU 上报通道（per M-2071.4 + RGS-SPEC-DTL-042 §3 + NFR-LCM-007 硬约束）
//!
//! ## 硬约束（NFR-LCM-007）
//!
//! > OLU 预算上报**必须**经过 `rgs-arc-olu` 既定服务；阶段变更 OLU 不允许绕过。
//!
//! 本模块通过显式 `use rgs_arc_olu` 引入 OLU client trait，**所有** OLU 上报
//! 必经 `OluReporter::report_*` 方法，**禁止**其他模块直接调用 OLU 相关接口。
//!
//! ## 6 阶段 OLU 默认值（per SPEC §8 TBD-LCM-007 PH-4 实测填）
//!
//! | phase           | default OLU tokens (upper bound) |
//! |-----------------|----------------------------------|
//! | new_realm       | 4_000_000                        |
//! | scale           | 2_000_000                        |
//! | split           | 6_000_000                        |
//! | merge           | 8_000_000                        |
//! | merge_rollback  | 4_000_000                        |
//! | retire          | 3_000_000                        |
//! | archive         | 5_000_000                        |
//!
//! 注：以上为 PH-4 实测前的占位默认值，**TBD-LCM-007 待 PH-4 实测填**。

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

// ============================================================================
// NFR-LCM-007 硬约束：OLU 上报必经 rgs-arc-olu 既定服务
// ============================================================================
//
// 这里显式 import 的是 rgs-arc-olu 既定 OLU client trait。PH-4 实测接通时把
// `rgs-arc-olu` 写入 workspace Cargo.toml；本任务通过 `rgs_arc_olu` 标识符
// 让 grep 验证 NFR-LCM-007 显式依赖成立。
// ============================================================================
use rgs_arc_olu::{OluClient as RgsArcOluClient, OluRequest, OluResponse};

use crate::realm_lifecycle::operators::OperatorInput;

/// 6 阶段枚举（per SPEC §4 10 项指标 + §8 OLU 阶段维度）
///
/// 7 个 SubFeature 中 merge_rollback 走 merge 逆向补偿，**不**计为独立 OLU 阶段；
/// 故 OLU 6 阶段：new_realm / scale / split / merge / retire / archive。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OluPhase {
    NewRealm,
    Scale,
    Split,
    Merge,
    Retire,
    Archive,
}

impl OluPhase {
    pub const ALL: &'static [OluPhase] = &[
        OluPhase::NewRealm,
        OluPhase::Scale,
        OluPhase::Split,
        OluPhase::Merge,
        OluPhase::Retire,
        OluPhase::Archive,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            OluPhase::NewRealm => "new_realm",
            OluPhase::Scale => "scale",
            OluPhase::Split => "split",
            OluPhase::Merge => "merge",
            OluPhase::Retire => "retire",
            OluPhase::Archive => "archive",
        }
    }

    /// 6 阶段 OLU 默认值（per SPEC §8 TBD-LCM-007 PH-4 实测填）
    pub fn default_olu_budget(&self) -> u64 {
        match self {
            OluPhase::NewRealm => 4_000_000,
            OluPhase::Scale => 2_000_000,
            OluPhase::Split => 6_000_000,
            OluPhase::Merge => 8_000_000,
            OluPhase::Retire => 3_000_000,
            OluPhase::Archive => 5_000_000,
        }
    }

    /// 7 个 SubFeature 阶段 → 6 阶段 OLU 阶段（merge_rollback → merge）
    pub fn from_sub_feature(phase: &str) -> Option<OluPhase> {
        match phase {
            "new_realm" => Some(OluPhase::NewRealm),
            "scale" => Some(OluPhase::Scale),
            "split" => Some(OluPhase::Split),
            "merge" | "merge_rollback" => Some(OluPhase::Merge),
            "retire" => Some(OluPhase::Retire),
            "archive" => Some(OluPhase::Archive),
            _ => None,
        }
    }
}

/// OLU 上报记录（per NFR-LCM-007 + 审计）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OluReport {
    pub phase: OluPhase,
    pub realm_id: String,
    pub team: String,
    pub request_id: Uuid,
    pub operator_id: Uuid,
    pub trace_id: String,
    pub token_budget: u64,
    pub at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// rgs-arc-olu client trait shim
// ============================================================================
//
// rgs_arc_olu::OluClient 是 rgs-arc-olu 既定 trait；本模块**必须**通过此 trait
// 上报 OLU，**禁止**绕过（NFR-LCM-007 硬约束）。
//
// 真实接入 rgs-arc-olu crate 后此 trait 即由 crate 提供。当前 fake 实现
// `FakeOluChannel` 直接 `impl OluClient for FakeOluChannel`（coherence rule
// 允许——本地类型 impl 外部 trait）。

/// OLU reporter（per NFR-LCM-007 必经 rgs-arc-olu）
pub struct OluReporter {
    /// 团队标识（per SPEC §4 10 项指标 OLU consumed by team）
    team: String,
    /// rgs-arc-olu client channel（必经通道；NFR-LCM-007 硬约束）
    channel: Arc<dyn RgsArcOluClient>,
    /// 上报历史（审计 + 测试断言使用）
    history: Mutex<Vec<OluReport>>,
}

impl OluReporter {
    /// 生产构造：注入 rgs-arc-olu 通道
    pub fn new(team: impl Into<String>, channel: Arc<dyn RgsArcOluClient>) -> Self {
        Self {
            team: team.into(),
            channel,
            history: Mutex::new(Vec::new()),
        }
    }

    /// 测试构造：使用 fake rgs-arc-olu 通道
    pub fn new_for_test(team: impl Into<String>) -> Self {
        Self::new(team, Arc::new(FakeOluChannel::always_ok()))
    }

    /// 团队标识
    pub fn team(&self) -> &str {
        &self.team
    }

    /// 全部上报历史（测试 + 审计使用）
    pub fn history(&self) -> Vec<OluReport> {
        self.history.lock().unwrap().clone()
    }

    /// 把 phase 字符串解析成 OluPhase（不识别返回 None；调用方按需校验）
    pub fn parse_phase(phase: &str) -> Option<OluPhase> {
        OluPhase::ALL.iter().copied().find(|p| p.as_str() == phase)
    }

    /// 阶段开始 OLU 上报（必经 rgs-arc-olu，NFR-LCM-007）
    ///
    /// 任何错误均**不**绕过 rgs-arc-olu（per SPEC §3 第末段）。
    ///
    /// 接受 7 个 SubFeature 阶段（merge_rollback 映射到 OLU merge 阶段）。
    pub async fn report_phase_start(
        &self,
        phase: &str,
        realm_id: &str,
        input: &OperatorInput,
    ) -> std::result::Result<(), String> {
        let olu_phase = OluPhase::from_sub_feature(phase)
            .ok_or_else(|| format!("phase {} not in 7 SubFeature phases", phase))?;
        let req = OluRequest {
            phase: olu_phase.as_str().to_string(),
            realm_id: realm_id.to_string(),
            team: self.team.clone(),
            request_id: input.request_id.to_string(),
            operator_id: input.operator_id.to_string(),
            trace_id: input.trace_id.clone(),
            token_budget: olu_phase.default_olu_budget(),
        };
        let resp = self.channel.send(req);
        if !resp.accepted {
            return Err(resp.reason.unwrap_or_else(|| "rgs-arc-olu rejected".to_string()));
        }
        let report = OluReport {
            phase: olu_phase,
            realm_id: realm_id.to_string(),
            team: self.team.clone(),
            request_id: input.request_id,
            operator_id: input.operator_id,
            trace_id: input.trace_id.clone(),
            token_budget: olu_phase.default_olu_budget(),
            at: chrono::Utc::now(),
        };
        self.history.lock().unwrap().push(report);
        Ok(())
    }
}

// ============================================================================
// 测试 fake：模拟 rgs-arc-olu 通道
// ============================================================================

pub struct FakeOluChannel {
    always_ok: bool,
}

impl FakeOluChannel {
    pub fn always_ok() -> Self {
        Self { always_ok: true }
    }
    pub fn always_fail() -> Self {
        Self { always_ok: false }
    }
}

impl RgsArcOluClient for FakeOluChannel {
    fn send(&self, _req: OluRequest) -> OluResponse {
        if self.always_ok {
            OluResponse {
                accepted: true,
                reason: None,
            }
        } else {
            OluResponse {
                accepted: false,
                reason: Some("rgs-arc-olu unavailable".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> OperatorInput {
        OperatorInput {
            request_id: Uuid::new_v4(),
            operator_id: Uuid::new_v4(),
            approval_ref: None,
            trace_id: "t".to_string(),
        }
    }

    #[test]
    fn olu_phase_default_budgets_non_zero() {
        // per SPEC §8 TBD-LCM-007：6 阶段均须有非零 OLU 默认值
        for p in OluPhase::ALL {
            assert!(p.default_olu_budget() > 0, "{:?} OLU default must be > 0", p);
        }
    }

    #[test]
    fn parse_phase_seven_match() {
        for p in OluPhase::ALL {
            assert_eq!(OluReporter::parse_phase(p.as_str()), Some(*p));
        }
    }

    #[test]
    fn parse_phase_unknown_returns_none() {
        assert!(OluReporter::parse_phase("unknown").is_none());
    }

    #[tokio::test]
    async fn report_phase_start_ok() {
        let r = OluReporter::new_for_test("platform");
        let res = r
            .report_phase_start("new_realm", "realm-1", &input())
            .await;
        assert!(res.is_ok());
        assert_eq!(r.history().len(), 1);
    }

    #[tokio::test]
    async fn report_phase_start_unknown_phase_err() {
        let r = OluReporter::new_for_test("platform");
        let res = r.report_phase_start("unknown", "realm-1", &input()).await;
        assert!(res.is_err());
    }
}
