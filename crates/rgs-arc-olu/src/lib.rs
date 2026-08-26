//! rgs-arc-olu 占位 crate（per RGS-SPEC-DTL-042 §3 NFR-LCM-007）
//!
//! 真实实现由 PH-4 接入：OLU 预算上报 gRPC service / 持久化 / 团队配额网关。
//! 当前最小占位：定义 OLU 上报请求/响应类型 + OluClient trait 形状。
//!
//! ## 设计意图
//!
//! - **`OluRequest` / `OluResponse`**：OLU 上报序列化结构（per SPEC §3 NFR-LCM-007 必经通道）
//! - **`OluClient` trait**：cluster-ops 等调用方通过此 trait 上报；PH-4 切到真实 gRPC client
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

/// rgs-arc-olu OLU client trait（per NFR-LCM-007 必经接口）
///
/// 调用方（cluster-ops realm_lifecycle::olu_reporter 等）通过此 trait 上报；
/// PH-4 替换为真实 gRPC impl 时本 trait 形状不变。
pub trait OluClient: Send + Sync {
    fn send(&self, req: OluRequest) -> OluResponse;
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
        let r = OluResponse {
            accepted: true,
            reason: None,
        };
        assert!(r.accepted);
        assert!(r.reason.is_none());
    }
}
