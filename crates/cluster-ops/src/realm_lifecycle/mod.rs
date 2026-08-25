//! 服务器全生命周期管理（LCM）子模块（per RGS-DTL-042 + SPEC-DTL-042 + ARC-051）
//!
//! WF-1-2066 M-2066.1 骨架：6 阶段操作器 + Saga + Drill + Plans + Feature 适配 + OLU 上报的子模块入口
//!
//! 入口单一（per FR-LCM-004）：
//! - 本模块**不**对外暴露独立 gRPC / HTTP
//! - 全部经 `rgs-admin-service` 的 `AdminService` 转发
//! - 阶段变更作为 `realm_lifecycle::*` Feature 子类走 ClusterOpsService PFAU 编排
//!
//! 范围（per RGS-IMPL-PLAN-LCM-001 v0.1 §1.1 + SPEC-DTL-042 §1）：
//! - 6 阶段操作器（开新服 / 扩缩容 / 分服 / 合服 / 退场 / 归档）
//! - SagaOrchestrator（per RGS-DTL-100 + RGS-DTL-015/016 既有模式复用）
//! - DrillExecutor（仅沙箱环境 per FR-LCM-003）
//! - 6 张新表 + Plan entity + PgRepository
//! - `rgs-arc-olu` 通道（per NFR-LCM-007 硬约束）
//!
//! 6 阶段状态机（per §4 + M-2066.10）：
//! ```text
//!     NewRealm → Scale → Split → Merge → Retire → Archive (终态)
//!     Archive 不可二次激活 NewRealm（负例）
//! ```
//!
//! 后续 L4 任务的子模块拆分（per 实施计划 §2.2）：
//! - `saga/`      → M-2067.x Saga 编排（WF-1-2067）
//! - `drill/`     → M-2070.x 演练执行器（WF-1-2070）
//! - `plans/`     → M-2068.x 6 张新表 + Plan entity（WF-1-2068）
//! - `feature_adapter.rs`  → M-2071.x Feature 7 子类注册（WF-1-2071）
//! - `olu_reporter.rs`     → M-2071.x OLU 上报（WF-1-2071）
//! - `metrics.rs`          → M-2071.x 10 项 rgs_lcm_* 指标（WF-1-2071）
//! - `realm_directory.rs`  → rgs-realm-directory 选服路由表（per SPEC §2）
//! - `config.rs`           → TBD-LCM-007 6 阶段 OLU 估算默认值（PH-4 实测填）

// 公开子模块（per FR-LCM-004：仅业务模块，不暴露独立 gRPC）
pub mod error;
pub mod operations;
pub mod service;

// 6 阶段状态机 UT 子模块（per M-2066.10；cfg(test) 仅测试期编译）
#[cfg(test)]
mod tests;

// 公开子模块（仅面向 cluster-ops 内部 crate，不在 lib.rs re-export）
// 注：plans / saga / drill / feature_adapter / olu_reporter / metrics 由后续 L4 任务追加
// （per RGS-IMPL-PLAN-LCM-001 §2.2 拟新增结构）

// 公开类型 re-export（per M-2066 验收门槛 + ARC-051 realm_lifecycle Feature 类型）
pub use error::{into_crate_result, LcmError, LcmErrorKind, LcmResult};
pub use service::{
    RealmLifecycleOperator, RealmLifecycleService, RealmLifecycleServiceImpl, RealmLifecycleStage,
    RealmLifecycleStateMachine,
};

// 操作器 re-export（per 6 操作器统一访问）
pub use operations::{
    ArchiveOperator, MergeOperator, MergeRollbackOperator, NewRealmOperator, RetireOperator,
    ScaleOperator,
};
