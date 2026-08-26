//! `realm_lifecycle` —— 服务器全生命周期管理子模块
//!
//! # 规范
//!
//! - RGS-IMPL-PLAN-LCM-001 v0.1 §2.2（结构）+ §3.6（WF-1-2073 范围）
//! - RGS-SPEC-DTL-042 v0.2 §2（实现单元）+ §3（实现契约）
//!
//! # 子模块
//!
//! - [`service`]        6 阶段操作器 trait 抽象（per L4 #2066 M-2066.2）
//! - [`error`]          LCM 错误码（per DTL-042 §6）
//! - [`saga`]           跨域 Saga 7 步 + 3 业务 service gRPC client（per L4 #2073）
//! - [`plans`]          6 张 LCM plan 表（per L4 #2068；retire_plan 完整实现 + 5 占位）
//!
//! # FR-LCM-004 硬约束
//!
//! 本子模块**不**对外暴露独立 gRPC / HTTP（per FR-LCM-004 硬约束）。
//! 全部经 `AdminService` 转发 + `ClusterOpsService` PFAU 编排。

pub mod error;
pub mod plans;
pub mod saga;
pub mod service;

pub use error::{Error, Result};
pub use plans::{
    retire_plan::{
        InMemoryRetirePlanConfig, QueryChannelRbac, RetireChannelRole, RetirePlan,
        RetirePlanConfig,
    },
    realm_lifecycle_run::RealmLifecycleRun,
    new_realm_plan::NewRealmPlan,
    split_plan::SplitPlan,
    merge_conflict_rule_set_v2::MergeConflictRuleV2,
    archive_policy::ArchivePolicy,
};
pub use saga::{
    BusinessServiceClient, CrossDomainSaga, InMemoryBusinessServiceClient, SagaContext,
    SagaStepError, SagaStepKind, SagaStepOutcome, SagaStepResult, TonicBusinessServiceClient,
    SAGA_STEP_KINDS, SAGA_STEP_ORDER,
};
pub use service::{
    NewRealmOperator, Operator, OperatorContext, OperatorOutcome, RealmLifecycleService,
    ScaleOperator, SplitOperator, MergeOperator, MergeRollbackOperator, RetireOperator,
    ArchiveOperator,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证子模块 re-export 完整（per FR-LCM-004 + L4 #2066/2067/2068/2073 集成）
    #[test]
    fn submodules_reexported() {
        // error
        let _: Error = Error::NotFound("test".to_string());
        // plans
        let _ = QueryChannelRbac::default();
        // saga
        assert_eq!(SAGA_STEP_ORDER.len(), 7);
        assert_eq!(SAGA_STEP_KINDS.len(), 7);
        // service
        assert_eq!(RealmLifecycleService::ALL_FEATURES.len(), 7);
    }
}
