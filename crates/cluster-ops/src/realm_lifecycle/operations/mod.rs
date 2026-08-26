//! 6 阶段操作器（per DTL-042 §5 + SPEC-DTL-042 §3）。
//!
//! 本 worktree（WF-1-2070）只提供 trait 签名 + 占位；具体实现由
//! WF-1-2066/2071/2073 后续 worktree 补齐（per WBS L4 拆分 + DEC-008 RACI）。

pub mod archive;
pub mod merge;
pub mod new_realm;
pub mod retire;
pub mod scale;
pub mod split;

/// 阶段操作器公共 trait（per DTL §5）。
///
/// 实际方法签名与具体 Saga 步骤绑定；本 worktree 只定义 trait 形状。
pub trait PhaseOperator: Send + Sync {
    /// 操作阶段名（用于 Prometheus 标签 `feature_subtype`，per DTL §11.1）。
    fn phase_name(&self) -> &'static str;
}
