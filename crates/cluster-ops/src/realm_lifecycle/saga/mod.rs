//! Saga 子模块（per DTL-042 §6 + SPEC-DTL-042 §3 + ADR-0015 Saga 适用边界）。
//!
//! 实际 SagaOrchestrator 实现由 WF-1-2067 后续 worktree 补齐（复用
//! `rgs-economy-service::saga_orchestrator` 的 `apply_atomic_with_reservation` 模式）。
//!
//! 本 worktree（WF-1-2070）只定义公共类型 + 6 阶段 SagaStep 枚举。

pub mod steps;

pub use steps::{SagaPhase, SagaStep, SagaStepKind, StepStatus};
