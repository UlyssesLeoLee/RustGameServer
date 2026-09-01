//! rgs-arc-olu 占位 crate（per RGS-SPEC-DTL-042 §3 NFR-LCM-007）
//!
//! 真实实现由 PH-4 接入：OLU 预算上报 gRPC service / 持久化 / 团队配额网关。
//! 当前最小占位：定义 OLU 上报请求/响应类型 + OluClient trait 形状。
//!
//! ## 设计意图
//!
//! - **`OluRequest` / `OluResponse`**：OLU 上报序列化结构（per SPEC §3 NFR-LCM-007 必经通道）
//! - **`OluClient` trait**：cluster-ops 等调用方通过此 trait 上报；PH-4 切到真实 gRPC client
//! - **`InMemoryOluClient`**：单线程测试 mock，记录调用 + 可配置拒绝 (per 9/1 14:15 JST 平台层派工)
//! - **`OluPhase` 6 阶段枚举**：new_realm / scale / split / merge / retire / archive
//!   （per SPEC §8 TBD-LCM-007 PH-4 实测填默认值，6 阶段必有非零 token_budget）
//! - 不依赖任何数据库 / 网络组件，保持 trait 形状稳定

use serde::{Deserialize, Serialize};

/// OLU 上报请求（per NFR-LCM-007：必经 rgs-arc-olu）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OluRequest {
    /// 阶段名（snake_case：new_realm / scale / split / merge / merge_rollback / retire / archive）
    pub phase: String,
    /// 目标 realm ID
    pub realm_id: String,
    /// 团队标识（per SPEC §4 OLU consumed by team 标签）
    pub team: String,
    /// 幂等键
    pub request_id: String,
    /// 操作者
    pub operator_id: String,
    /// 跟踪 ID
    pub trace_id: String,
    /// OLU token 预算上限
    pub token_budget: u64,
}

/// OLU 上报响应（per NFR-LCM-007）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OluResponse {
    /// 是否接受（false 时 OLU 预算超限 / 通道不可用，调用方必须 fail-closed）
    pub accepted: bool,
    /// 拒绝原因（accepted=false 时必有）
    pub reason: Option<String>,
}

impl OluResponse {
    /// 构造接受响应
    pub fn accept() -> Self {
        Self {
            accepted: true,
            reason: None,
        }
    }

    /// 构造拒绝响应（必填 reason，per NFR-LCM-007 fail-closed 必填）
    pub fn reject(reason: impl Into<String>) -> Self {
        Self {
            accepted: false,
            reason: Some(reason.into()),
        }
    }
}

/// rgs-arc-olu OLU client trait（per NFR-LCM-007 必经接口）
///
/// 调用方（cluster-ops realm_lifecycle::olu_reporter 等）通过此 trait 上报；
/// PH-4 替换为真实 gRPC impl 时本 trait 形状不变。
pub trait OluClient: Send + Sync {
    fn send(&self, req: OluRequest) -> OluResponse;
}

/// 6 阶段 OLU 枚举（per SPEC §8 TBD-LCM-007 PH-4 实测填默认值）
///
/// 7 个 SubFeature 阶段中 merge_rollback 映射到 merge 阶段，故 OLU 维度 6 阶段。
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
    /// 保证 6 阶段均 > 0 (per cluster-ops olu_reporter test 派生约束)
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

    /// 7 个 SubFeature 阶段 → 6 阶段 OLU 阶段 (merge_rollback → merge)
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

/// InMemory mock OLU client（per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello 任务）
///
/// 单元测试 + 集成测试沿用 InMemory 风格（per 8/31 5 业务域 worker 模板）：
/// - `sent` 字段记录全部 `OluRequest` 调用，供断言验证
/// - `accept` 字段控制是否一律 accept / reject（fail-closed 路径测试）
/// - `budget_limit` 字段可选，若 request.token_budget 超过则 reject
pub struct InMemoryOluClient {
    /// 全部 send 调用记录（per 9/1 14:15 JST 平台层派工要求 5+ 业务函数 + proptest）
    pub sent: std::sync::Mutex<Vec<OluRequest>>,
    /// 一律拒绝（fail-closed 路径测试）
    pub reject: bool,
    /// 拒绝时返回的 reason
    pub reject_reason: String,
    /// 超过此 token_budget 上限则 reject（None = 不限）
    pub budget_limit: Option<u64>,
}

impl InMemoryOluClient {
    /// 默认 accept mock
    pub fn always_accept() -> Self {
        Self {
            sent: std::sync::Mutex::new(Vec::new()),
            reject: false,
            reject_reason: String::new(),
            budget_limit: None,
        }
    }

    /// 一律 reject mock（per NFR-LCM-007 fail-closed）
    pub fn always_reject(reason: impl Into<String>) -> Self {
        let r = reason.into();
        Self {
            sent: std::sync::Mutex::new(Vec::new()),
            reject: true,
            reject_reason: r,
            budget_limit: None,
        }
    }

    /// 带 budget_limit 的 mock：超限 reject（per PH-4 团队配额网关占位）
    pub fn with_budget_limit(limit: u64) -> Self {
        Self {
            sent: std::sync::Mutex::new(Vec::new()),
            reject: false,
            reject_reason: String::new(),
            budget_limit: Some(limit),
        }
    }

    /// 取出全部 send 调用记录（供测试断言）
    pub fn take_sent(&self) -> Vec<OluRequest> {
        std::mem::take(&mut *self.sent.lock().unwrap())
    }
}

impl OluClient for InMemoryOluClient {
    fn send(&self, req: OluRequest) -> OluResponse {
        self.sent.lock().unwrap().push(req.clone());
        if self.reject {
            return OluResponse::reject(self.reject_reason.clone());
        }
        if let Some(limit) = self.budget_limit {
            if req.token_budget > limit {
                return OluResponse::reject(format!(
                    "token_budget {} exceeds limit {}",
                    req.token_budget, limit
                ));
            }
        }
        OluResponse::accept()
    }
}

/// 测试 helper：根据 OluPhase 构造标准 OluRequest
pub fn request_for_phase(phase: OluPhase, realm_id: &str, team: &str) -> OluRequest {
    OluRequest {
        phase: phase.as_str().to_string(),
        realm_id: realm_id.to_string(),
        team: team.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        operator_id: uuid::Uuid::new_v4().to_string(),
        trace_id: format!("trace-{}", realm_id),
        token_budget: phase.default_olu_budget(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn olu_request_serde_roundtrip() {
        let r = OluRequest {
            phase: "new_realm".to_string(),
            realm_id: "realm-1".to_string(),
            team: "platform".to_string(),
            request_id: "req-1".to_string(),
            operator_id: "op-1".to_string(),
            trace_id: "trace-1".to_string(),
            token_budget: 4_000_000,
        };
        let s = serde_json::to_string(&r).unwrap();
        let back: OluRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn olu_response_default_accept() {
        let r = OluResponse::accept();
        assert!(r.accepted);
        assert!(r.reason.is_none());
    }

    #[test]
    fn olu_response_reject_has_reason() {
        let r = OluResponse::reject("quota exceeded");
        assert!(!r.accepted);
        assert_eq!(r.reason.as_deref(), Some("quota exceeded"));
    }

    #[test]
    fn olu_phase_all_six_have_positive_budget() {
        // 6 阶段均 > 0 (per cluster-ops olu_reporter test 派生约束)
        for p in OluPhase::ALL {
            assert!(p.default_olu_budget() > 0, "{:?} OLU default must be > 0", p);
            assert!(!p.as_str().is_empty(), "{:?} as_str must be non-empty", p);
        }
    }

    #[test]
    fn olu_phase_sub_feature_mapping_seven() {
        // 7 个 SubFeature 阶段 (含 merge_rollback) → 6 阶段 OLU
        assert_eq!(OluPhase::from_sub_feature("new_realm"), Some(OluPhase::NewRealm));
        assert_eq!(OluPhase::from_sub_feature("scale"), Some(OluPhase::Scale));
        assert_eq!(OluPhase::from_sub_feature("split"), Some(OluPhase::Split));
        assert_eq!(OluPhase::from_sub_feature("merge"), Some(OluPhase::Merge));
        // merge_rollback 映射到 merge 阶段（per SPEC §4 7 个 SubFeature）
        assert_eq!(OluPhase::from_sub_feature("merge_rollback"), Some(OluPhase::Merge));
        assert_eq!(OluPhase::from_sub_feature("retire"), Some(OluPhase::Retire));
        assert_eq!(OluPhase::from_sub_feature("archive"), Some(OluPhase::Archive));
        assert!(OluPhase::from_sub_feature("unknown_phase").is_none());
    }

    #[test]
    fn in_memory_olu_client_records_sends() {
        let client = InMemoryOluClient::always_accept();
        let req = request_for_phase(OluPhase::NewRealm, "r1", "platform");
        let resp = client.send(req.clone());
        assert!(resp.accepted);
        let sent = client.take_sent();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], req);
    }

    #[test]
    fn in_memory_olu_client_rejects_when_configured() {
        let client = InMemoryOluClient::always_reject("rgs-arc-olu unavailable");
        let req = request_for_phase(OluPhase::Scale, "r1", "platform");
        let resp = client.send(req);
        assert!(!resp.accepted);
        assert_eq!(resp.reason.as_deref(), Some("rgs-arc-olu unavailable"));
    }

    #[test]
    fn in_memory_olu_client_budget_limit_enforced() {
        let client = InMemoryOluClient::with_budget_limit(1_000_000);
        // NewRealm default 4M > 1M, 应该被 reject
        let big = request_for_phase(OluPhase::NewRealm, "r1", "platform");
        let resp = client.send(big);
        assert!(!resp.accepted);
        assert!(resp.reason.unwrap_or_default().contains("exceeds limit"));

        // Scale default 2M > 1M, reject
        let medium = request_for_phase(OluPhase::Scale, "r1", "platform");
        let resp = client.send(medium);
        assert!(!resp.accepted);
    }

    // ====== proptest: OLU request serde round-trip (per 9/1 14:15 JST 派工要求 proptest) ======
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn arb_phase() -> impl Strategy<Value = OluPhase> {
            prop_oneof![
                Just(OluPhase::NewRealm),
                Just(OluPhase::Scale),
                Just(OluPhase::Split),
                Just(OluPhase::Merge),
                Just(OluPhase::Retire),
                Just(OluPhase::Archive),
            ]
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(128))]

            /// 6 阶段 OLU request serde roundtrip 保留全部字段（per 9/1 14:15 JST proptest 要求）
            #[test]
            fn olu_request_serde_roundtrip_preserves_fields(
                phase in arb_phase(),
                realm in "[a-z]{1,16}",
                team in "[a-z]{1,16}",
                request_id in "[a-zA-Z0-9-]{8,32}",
                operator_id in "[a-zA-Z0-9-]{8,32}",
                trace_id in "[a-zA-Z0-9-]{8,32}",
                token_budget in 1u64..1_000_000_000u64,
            ) {
                let r = OluRequest {
                    phase: phase.as_str().to_string(),
                    realm_id: realm.clone(),
                    team: team.clone(),
                    request_id: request_id.clone(),
                    operator_id: operator_id.clone(),
                    trace_id: trace_id.clone(),
                    token_budget,
                };
                let s = serde_json::to_string(&r).expect("serialize");
                let back: OluRequest = serde_json::from_str(&s).expect("deserialize");
                prop_assert_eq!(back, r);
            }

            /// 6 阶段 as_str 永远非空 + 不含大写 (per SPEC §4 snake_case 约定)
            #[test]
            fn olu_phase_as_str_invariant(phase in arb_phase()) {
                let s = phase.as_str();
                prop_assert!(!s.is_empty());
                prop_assert!(!s.chars().any(|c| c.is_ascii_uppercase()));
            }
        }
    }
}
