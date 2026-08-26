//! `RealmLifecycleService` 门面 trait（per SPEC-DTL-042 §3 + DTL-042 §5）。
//!
//! ## 硬约束（per FR-LCM-004）
//!
//! `RealmLifecycleService` **不**对外暴露独立接口；仅经 `AdminService` 转发。
//! `lib.rs` 不得 `re-export` tonic::include_proto 入口。
//!
//! ## 6 操作器签名
//!
//! 实际实现由后续 worktree（WF-1-2066/2067/2068/2071/2073）补齐；
//! 本 worktree（WF-1-2070）只提供 trait 签名 + 默认实现占位 + drill 测试可引用。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    ApprovalRef, OperatorId, RealmId, RealmStatus, RequestId, SagaRunId, TraceId,
};

// =====================================================================
// DTO
// =====================================================================

/// LCM 操作统一请求（per SPEC §3 第 7 条：request_id + operator_id + approval_ref + trace_id）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRequest {
    pub request_id: RequestId,
    pub operator_id: OperatorId,
    pub approval_ref: ApprovalRef,
    pub trace_id: TraceId,
    pub realm_id: RealmId,
    /// 操作阶段（由 AdminService 转发时填入；drill 也透传）。
    pub phase: LifecyclePhase,
}

/// LCM 操作统一响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleResponse {
    pub run_id: String,
    pub saga_run_id: Option<SagaRunId>,
    pub status: RealmStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 6 阶段枚举（per DTL-042 §4 状态机）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecyclePhase {
    NewRealm,
    Scale,
    Split,
    Merge,
    MergeRollback,
    Retire,
    Archive,
}

impl LifecyclePhase {
    /// 字符串化（用于 Prometheus 标签 `feature_subtype`，per DTL §11.1）。
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NewRealm => "new_realm",
            Self::Scale => "scale",
            Self::Split => "split",
            Self::Merge => "merge",
            Self::MergeRollback => "merge_rollback",
            Self::Retire => "retire",
            Self::Archive => "archive",
        }
    }
}

// =====================================================================
// RealmLifecycleService trait（per FR-LCM-004：仅 AdminService 转发）
// =====================================================================

/// 6 阶段操作器门面 trait。
///
/// 实际实现由 `ClusterOpsService` 内部组合 `operations::*` + `saga::orchestrator` 适配。
/// 本 trait **不**对外暴露 gRPC；任何对 `RealmLifecycleService::*` 方法的调用必须经
/// `AdminService` 转发到 `ClusterOpsService` PFAU 编排。
#[async_trait]
pub trait RealmLifecycleService: Send + Sync {
    /// 开新服（AC-LCM-001）。
    async fn new_realm(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;

    /// 扩缩容（AC-LCM-002；scale_up / scale_down 双向）。
    async fn scale(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;

    /// 分服（AC-LCM-003）。
    async fn split(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;

    /// 合服（AC-LCM-004）。
    async fn merge(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;

    /// 合服回退（AC-LCM-005；FR-LCM-062 锁定后触发回退窗口期 7-30 天）。
    async fn merge_rollback(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;

    /// 退场（AC-LCM-006；触发 RBAC 查询通道限制 + 30-90 天后启动归档）。
    async fn retire(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;

    /// 归档（AC-LCM-007；冷热分层 + N+2 冗余；不删数据）。
    async fn archive(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse>;
}

// =====================================================================
// InMemoryService 占位实现（供 UT + drill test 引用；实际实现由后续 worktree）
// =====================================================================

/// `NoopRealmLifecycleService` —— 默认占位实现。
///
/// 所有方法返回 `Error::Validation` 标记"未实现"；drill 测试**不**调用这些方法，
/// drill 走自己的 `DrillExecutor` + `sandbox_*` 路径（per FR-LCM-003）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRealmLifecycleService;

#[async_trait]
impl RealmLifecycleService for NoopRealmLifecycleService {
    async fn new_realm(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("new_realm", &req)
    }
    async fn scale(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("scale", &req)
    }
    async fn split(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("split", &req)
    }
    async fn merge(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("merge", &req)
    }
    async fn merge_rollback(
        &self,
        req: LifecycleRequest,
    ) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("merge_rollback", &req)
    }
    async fn retire(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("retire", &req)
    }
    async fn archive(&self, req: LifecycleRequest) -> crate::Result<LifecycleResponse> {
        unimplemented_marker("archive", &req)
    }
}

fn unimplemented_marker(
    op: &str,
    _req: &LifecycleRequest,
) -> crate::Result<LifecycleResponse> {
    Err(crate::Error::Validation(format!(
        "RealmLifecycleService::{op} pending impl in WF-1-2066/2071/2073"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_as_str_matches_dtl_feature_subtype() {
        // DTL §11.1：feature_subtype 标签值
        assert_eq!(LifecyclePhase::NewRealm.as_str(), "new_realm");
        assert_eq!(LifecyclePhase::Scale.as_str(), "scale");
        assert_eq!(LifecyclePhase::Split.as_str(), "split");
        assert_eq!(LifecyclePhase::Merge.as_str(), "merge");
        assert_eq!(LifecyclePhase::MergeRollback.as_str(), "merge_rollback");
        assert_eq!(LifecyclePhase::Retire.as_str(), "retire");
        assert_eq!(LifecyclePhase::Archive.as_str(), "archive");
    }

    #[tokio::test]
    async fn noop_service_returns_validation_error() {
        let svc = NoopRealmLifecycleService;
        let req = LifecycleRequest {
            request_id: "r-1".to_string(),
            operator_id: "op-1".to_string(),
            approval_ref: None,
            trace_id: "t-1".to_string(),
            realm_id: "rlm-1".to_string(),
            phase: LifecyclePhase::NewRealm,
        };
        for r in [
            svc.new_realm(req.clone()).await,
            svc.scale(req.clone()).await,
            svc.split(req.clone()).await,
            svc.merge(req.clone()).await,
            svc.merge_rollback(req.clone()).await,
            svc.retire(req.clone()).await,
            svc.archive(req.clone()).await,
        ] {
            assert!(matches!(r, Err(crate::Error::Validation(_))));
        }
    }
}
