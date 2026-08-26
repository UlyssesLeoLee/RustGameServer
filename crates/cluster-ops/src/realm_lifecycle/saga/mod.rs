//! cluster-ops · realm_lifecycle · Saga 编排模块（per RGS-IMPL-PLAN-LCM-001 §3.2 + RGS-SPEC-DTL-042 §3）
//!
//! 职责：
//! - M-2067.1 复用 economy::saga_orchestrator 模式适配（不重写）
//! - M-2067.2 6 阶段 Saga 步骤定义
//! - M-2067.3 反向补偿步骤（含跨域 Saga 反向）
//! - M-2067.4 幂等性：(request_id, operator_id) 唯一索引
//! - M-2067.5 Saga 步骤超时（默认 60s per SPEC §8）
//! - M-2067.6 UT 验证（位于 `tests/ut_saga.rs`）
//!
//! 关键复用声明（per RGS-IMPL-PLAN-LCM-001 §2.3）：
//! - SagaOrchestrator 是 RealmLifecycleService **内部模块**，**不**分发独立协调服务
//! - 不重写 Saga 状态机；只 import + 适配
//! - 跨 DB 写入复用 RGS-ADR-0015 Saga 适用边界与单一调解者原则

pub mod orchestrator;
pub mod steps;
pub mod idempotency;

pub use orchestrator::{SagaOrchestrator, SagaStepHandler, SagaContext, LcmPhase, LcmSaga, LcmSagaStep, SagaStepStatus};
pub use steps::{CompensateAction, SagaTimeoutConfig};
pub use idempotency::{IdempotencyKey, IdempotencyRecord, IdempotencyStore, InMemoryIdempotencyStore};
