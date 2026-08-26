//! 6 Plan 占位（per RGS-SPEC-DTL-042 §2 + IMPL-PLAN-LCM-001 §3.5）
//!
//! 6 张 DDL 目标表（全部在 admin_db）：
//! - new_realm_plan
//! - scale_plan
//! - split_plan
//! - merge_plan
//! - retire_plan
//! - archive_policy
//!
//! 本任务仅占位 6 个 Plan 子模块；具体 DDL 在 `migrations/0020_lcm_tables.sql`。
//!
//! merge_rollback 走 merge_plan 的逆向补偿路径（per FR-LCM-051），不另立 Plan。

pub mod archive_plan;
pub mod merge_plan;
pub mod new_realm_plan;
pub mod retire_plan;
pub mod scale_plan;
pub mod split_plan;
