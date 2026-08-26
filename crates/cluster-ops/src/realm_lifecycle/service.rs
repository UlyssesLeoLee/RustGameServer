//! RealmLifecycleService 主入口（per RGS-SPEC-DTL-042 §2 + §3）
//!
//! **不**对外暴露独立接口（FR-LCM-004 硬约束）；所有调用经 `AdminService` 转发。
//! 本服务**只**作为 RealmLifecycleService 内部业务编排入口存在，调用者必须
//! 在 AdminService 完成 RBAC + 幂等 + 审计后才能委派到此服务。

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::feature_adapter::{FeatureRegistry, RealmLifecycleFeatureAdapter};
use crate::realm_lifecycle::metrics::LcmMetrics;
use crate::realm_lifecycle::olu_reporter::OluReporter;
use crate::realm_lifecycle::operators::OperatorInput;

/// 阶段变更命令（per SPEC §2 RealmLifecycleService）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhaseChangeCommand {
    /// 阶段名（snake_case：new_realm / scale / split / merge / merge_rollback / retire / archive）
    pub phase: String,
    /// 目标 realm ID
    pub target_realm_id: Uuid,
    /// 操作者上下文
    pub input: OperatorInput,
}

/// 阶段变更结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhaseChangeResult {
    pub run_id: Uuid,
    pub phase: String,
    pub feature_id: String,
    pub pfau_state: String,
}

/// RealmLifecycleService trait（per SPEC §3 RealmLifecycleService 业务方法）
#[async_trait]
pub trait RealmLifecycleServiceTrait: Send + Sync {
    /// 启动一次阶段变更（走 PFAU 5 状态机 + 7 SubFeature 编排 + OLU 上报）
    async fn start_phase_change(&self, cmd: PhaseChangeCommand) -> Result<PhaseChangeResult>;

    /// 查询阶段变更状态
    async fn get_phase_change(&self, run_id: Uuid) -> Result<PhaseChangeResult>;
}

/// RealmLifecycleServiceImpl（per SPEC §2 + IMPL-PLAN-LCM-001 §3.5）
pub struct RealmLifecycleService {
    feature_adapter: Arc<RealmLifecycleFeatureAdapter>,
    olu_reporter: Arc<OluReporter>,
    metrics: Arc<LcmMetrics>,
    registry: Arc<FeatureRegistry>,
}

impl RealmLifecycleService {
    pub fn new(
        feature_adapter: Arc<RealmLifecycleFeatureAdapter>,
        olu_reporter: Arc<OluReporter>,
        metrics: Arc<LcmMetrics>,
        registry: Arc<FeatureRegistry>,
    ) -> Self {
        Self {
            feature_adapter,
            olu_reporter,
            metrics,
            registry,
        }
    }

    /// 直接访问 Feature 注册表（per M-2071.6 验证需要）
    pub fn registry(&self) -> &Arc<FeatureRegistry> {
        &self.registry
    }

    /// 直接访问 OLU reporter（per M-2071.7 验证需要）
    pub fn olu_reporter(&self) -> &Arc<OluReporter> {
        &self.olu_reporter
    }

    /// 直接访问 metrics（per M-2071.5 + 验证需要）
    pub fn metrics(&self) -> &Arc<LcmMetrics> {
        &self.metrics
    }
}

#[async_trait]
impl RealmLifecycleServiceTrait for RealmLifecycleService {
    async fn start_phase_change(&self, cmd: PhaseChangeCommand) -> Result<PhaseChangeResult> {
        // 1. FeatureType::RealmLifecycle 注册校验（per M-2071.2 + DTL-031 §5 发布）
        self.feature_adapter
            .require_registered(&cmd.phase)
            .map_err(crate::realm_lifecycle::error::Error::PFAUFeatureNotRegistered)?;

        // 2. PFAU 5 状态机启动（per M-2071.3）
        let run_id = self
            .feature_adapter
            .start_pfau_run(&cmd.phase, &cmd.input)?;

        // 3. OLU 上报（per M-2071.4 + NFR-LCM-007 硬约束：必经 rgs-arc-olu）
        self.olu_reporter
            .report_phase_start(&cmd.phase, &cmd.target_realm_id.to_string(), &cmd.input)
            .await
            .map_err(|reason| {
                crate::realm_lifecycle::error::Error::OLUReportFailed {
                    phase: cmd.phase.clone(),
                    team: "platform".to_string(),
                    reason,
                }
            })?;

        // 4. metrics 记录（per M-2071.5）
        self.metrics
            .record_pfau_transition(&cmd.phase, "none", "declared");
        self.metrics.inc_active_runs(&cmd.phase);

        Ok(PhaseChangeResult {
            run_id,
            phase: cmd.phase.clone(),
            feature_id: format!("realm_lifecycle.{}", cmd.phase),
            pfau_state: "declared".to_string(),
        })
    }

    async fn get_phase_change(&self, _run_id: Uuid) -> Result<PhaseChangeResult> {
        // 占位：PH-4 接入 realm_lifecycle_run 表查询
        Err(crate::realm_lifecycle::error::Error::NotFound {
            entity: "realm_lifecycle_run",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{FeatureType, PfauState, SubFeature};
    use crate::realm_lifecycle::feature_adapter::FeatureRegistry;
    use crate::realm_lifecycle::metrics::LcmMetrics;
    use crate::realm_lifecycle::olu_reporter::OluReporter;

    fn build_service() -> RealmLifecycleService {
        let adapter = Arc::new(RealmLifecycleFeatureAdapter::new());
        let olu = Arc::new(OluReporter::new_for_test("test"));
        let metrics = Arc::new(LcmMetrics::new_for_test());
        let registry = Arc::new(FeatureRegistry::with_default_seven());
        let _ = adapter;
        let _ = FeatureType::RealmLifecycle;
        let _ = SubFeature::ALL;
        let _ = PfauState::ALL;
        RealmLifecycleService::new(adapter, olu, metrics, registry)
    }

    fn cmd(phase: &str) -> PhaseChangeCommand {
        PhaseChangeCommand {
            phase: phase.to_string(),
            target_realm_id: Uuid::new_v4(),
            input: OperatorInput {
                request_id: Uuid::new_v4(),
                operator_id: Uuid::new_v4(),
                approval_ref: None,
                trace_id: "trace-test".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn start_phase_change_new_realm_ok() {
        let s = build_service();
        let r = s.start_phase_change(cmd("new_realm")).await.unwrap();
        assert_eq!(r.phase, "new_realm");
        assert_eq!(r.pfau_state, "declared");
    }

    #[tokio::test]
    async fn start_phase_change_unknown_phase_fails() {
        let s = build_service();
        let err = s
            .start_phase_change(cmd("not_a_phase"))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::realm_lifecycle::error::Error::PFAUFeatureNotRegistered(_)
        ));
    }
}
