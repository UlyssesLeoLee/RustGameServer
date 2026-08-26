//! cluster-ops · realm_lifecycle 子模块（per RGS-IMPL-PLAN-LCM-001 §2 + RGS-SPEC-DTL-042 §2）
//!
//! 职责：服务器全生命周期管理（开新服 / 扩缩容 / 分服 / 合服 / 退场 / 归档）。
//! 6 阶段操作器 + Saga 编排 + Drill 演练 + Plans 表 + Feature 适配 + OLU 上报。
//!
//! 硬约束（per RGS-SPEC-DTL-042 §2）：
//! - 入口经 AdminService 转发；RealmLifecycleService **不**对外暴露独立接口
//! - 阶段变更作为 `realm_lifecycle::*` Feature 子类走 ClusterOpsService PFAU 编排
//! - SagaOrchestrator 是 RealmLifecycleService 内部模块，**不**分发独立协调服务
//! - 不在 `admin_db` 之外新建独立数据库
//!
//! 复用原则（per RGS-IMPL-PLAN-LCM-001 §2.3 关键复用声明）：
//! - Saga 模式：复用 `economy-service::saga_orchestrator`（per RGS-DTL-100 + RGS-DTL-015/016）
//! - 不重写 Saga 状态机；只 import + 适配
//!
//! 本 worktree（wbs/WF-1-2067）只关注 M-2067.1~6（Saga 编排层）；
//! 6 操作器业务逻辑属于 WF-1-2066 / WF-1-2070 / WF-1-2071，
//! 6 张新表 migration 属于 WF-1-2068，Drill 属于 WF-1-2070。

pub mod error;
pub mod service;
pub mod operations;
pub mod saga;

#[cfg(test)]
pub mod tests;

pub use error::{Error, Result};
pub use service::{
    ArchiveOperator, LcmOperatorInput, LcmOperatorOutput, MergeOperator, NewRealmOperator,
    RealmLifecycleService, RetireOperator, ScaleOperator, SplitOperator,
};
pub use saga::{
    CompensateAction, IdempotencyKey, IdempotencyRecord, IdempotencyStore, LcmPhase, LcmSaga,
    LcmSagaStep, SagaContext, SagaOrchestrator, SagaStepHandler, SagaStepStatus, SagaTimeoutConfig,
};
