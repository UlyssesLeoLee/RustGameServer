//! 服务器全生命周期管理（LCM）— realm_lifecycle 子模块入口
//!
//! 规范：RGS-SPEC-DTL-042 §2 + ARC-051 Feature 扩展
//! 入口统一经由 `AdminService` 转发（FR-LCM-004 硬约束）
//! 阶段变更作为 `realm_lifecycle::*` Feature 子类走 ClusterOpsService PFAU 编排
//!
//! ## 子模块
//!
//! - [`plans`]：6 张 Plan 表 entity + PgRepository 骨架（per M-2068.7）
//! - 后续 L4 任务（WF-1-2066 / WF-1-2067）将补充：
//!   - `operates/`：6 阶段操作器（NewRealm / Scale / Split / Merge / Retire / Archive）
//!   - `saga.rs`：SagaOrchestrator
//!   - `drill.rs`：DrillExecutor（沙箱 PG + K8s 客户端）
//!   - `feature_adapter.rs`：ClusterOpsService PFAU 集成
//!   - `olu_reporter.rs`：OLU 预算上报（per NFR-LCM-007）
//!   - `metrics.rs`：10 项 `rgs_lcm_*` 指标
//!
//! ## 硬约束（继承自 RGS-SPEC-DTL-042 §3）
//!
//! - **FR-LCM-001**：6 张表全部在 admin_db；本子模块不新建独立数据库
//! - **FR-LCM-003**：DrillExecutor **仅**在沙箱 PG 池 + 沙箱 K8s 客户端跑
//! - **FR-LCM-004**：入口统一经由 AdminService 转发；不暴露独立接口
//! - **NFR-LCM-007**：OLU 预算上报**必须**经 rgs-arc-olu 既定服务
//! - **NFR-SE-010**：GDPR 删除通路 admin_db.audit_log 双层审计

pub mod plans;

// 后续 L4 任务会扩展以下子模块（per RGS-SPEC-DTL-042 §2 实现单元）：
// pub mod operates;
// pub mod saga;
// pub mod drill;
// pub mod feature_adapter;
// pub mod olu_reporter;
// pub mod metrics;

// 在 realm_lifecycle 命名空间下重新导出 plans 模块的公共项，
// 方便调用方写 `realm_lifecycle::RealmLifecycleRun` 而非 `realm_lifecycle::plans::RealmLifecycleRun`。
pub use plans::*;
