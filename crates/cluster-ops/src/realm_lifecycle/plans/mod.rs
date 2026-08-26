//! 5 张 LCM 配置 Plan 表（per RGS-DTL-042 §3.1）
//!
//! - [`archive_policy`] —— 归档策略（3 年热 + 10 年冷 + N+2 存储冗余 + GDPR 删除通路入口）
//! - 其它 4 张（新服 / 分服 / 合服冲突 v2 / 退场）由其他 L4 任务覆盖
//!
//! **本任务范围（per WBS L4 #2074 / RGS-IMPL-PLAN-LCM-001 §3.7）**：
//! - 仅实现 [`archive_policy`]
//! - 其它 Plan 实体保留 stub 引用以便 lib.rs 通过编译

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod archive_policy;

// ===== 其它 Plan 实体占位（per RGS-DTL-042 §3.1） =====
// 由其它 WBS L4 任务（#2067 / #2068 / #2071）覆盖；本任务不实现。

/// 新服计划占位（per DTL-042 §3.1 #2）
pub mod new_realm_plan {
    use uuid::Uuid;

    /// 新服计划标识（占位，完整字段由 #2067 任务实现）
    #[derive(Debug, Clone)]
    pub struct NewRealmPlanId(pub Uuid);
}

/// 分服计划占位（per DTL-042 §3.1 #3）
pub mod split_plan {
    use uuid::Uuid;

    /// 分服计划标识（占位，完整字段由 #2068 任务实现）
    #[derive(Debug, Clone)]
    pub struct SplitPlanId(pub Uuid);
}

/// 合服冲突规则 v2 占位（per DTL-042 §3.1 #4）
pub mod merge_conflict_rule_set {
    use uuid::Uuid;

    /// 合服冲突规则集标识（占位，完整字段由 #2071 任务实现）
    #[derive(Debug, Clone)]
    pub struct MergeRuleSetId(pub Uuid);
}

/// 退场计划占位（per DTL-042 §3.1 #5）
pub mod retire_plan {
    use uuid::Uuid;

    /// 退场计划标识（占位，完整字段由其它任务实现）
    #[derive(Debug, Clone)]
    pub struct RetirePlanId(pub Uuid);
}
