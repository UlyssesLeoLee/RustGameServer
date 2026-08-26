//! 6 张计划表占位（per DTL-042 §7 + IMPL §2.2）。
//!
//! 实际 entity + PgRepository 由 WF-1-2068 后续 worktree 补齐。
//! 本 worktree（WF-1-2070）只定义 entity 结构 + re-export。

pub mod archive_policy;
pub mod merge_conflict_rule_set_v2;
pub mod new_realm_plan;
pub mod realm_lifecycle_run;
pub mod retire_plan;
pub mod split_plan;
