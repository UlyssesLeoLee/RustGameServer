//! `realm_lifecycle::plans` —— 6 张 LCM plan 表（per RGS-IMPL-PLAN-LCM-001 §2.2 + SPEC-DTL-042 §2）
//!
//! 6 张表（per DTL-042 §2 + L4 #2068 + M-2068.1~3）：
//!   - `realm_lifecycle_run`         主运行记录表（按 created_at 月度范围分区）
//!   - `new_realm_plan`             NewRealm 计划表
//!   - `split_plan`                 Split 计划表
//!   - `merge_conflict_rule_set_v2` 合服冲突规则 v2 表（locked_at 锁定后不可改，per FR-LCM-062）
//!   - `retire_plan`                退场计划表（含 query_channel_rbac 配置，per SPEC §3 第 8 条）
//!   - `archive_policy`             归档策略表（冷热分层 + N+2 冗余，per RSK-LCM-005 缓解）
//!
//! WF-1-2073 范围：
//! - `retire_plan` 完整实现（M-2073.4 RBAC 通道）
//! - 5 个占位 plan 类型（具名 entity + 占位 Repository trait，留 L4 #2068 M-2068.7 扩展）

pub mod retire_plan;

// 5 个占位 plan（具名 entity + 占位 trait），避免 L4 #2068 之前 compile 找不到模块
pub mod realm_lifecycle_run {
    //! `realm_lifecycle_run` 主运行记录表占位（per L4 #2068 M-2068.1 + M-2068.7）
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct RealmLifecycleRun {
        pub run_id: Uuid,
        pub feature_id: String,
        pub request_id: Uuid,
        pub operator_id: Uuid,
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub state: String,
    }
}

pub mod new_realm_plan {
    //! `new_realm_plan` 占位（per L4 #2068 M-2068.2 + M-2068.7）
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NewRealmPlan {
        pub plan_id: Uuid,
        pub realm_id: String,
        pub template: String,
    }
}

pub mod split_plan {
    //! `split_plan` 占位（per L4 #2068 M-2068.2 + M-2068.7）
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct SplitPlan {
        pub plan_id: Uuid,
        pub source_realm_id: String,
        pub target_realm_ids: Vec<String>,
    }
}

pub mod merge_conflict_rule_set_v2 {
    //! `merge_conflict_rule_set_v2` 占位（per L4 #2068 M-2068.2 + FR-LCM-062 锁定后不可改）
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct MergeConflictRuleV2 {
        pub rule_id: Uuid,
        pub rule_type: String,
        pub rule_body: String,
        /// 锁定时间（None = 未锁定，可改；Some(_) = 锁定后不可改，per FR-LCM-062）
        pub locked_at: Option<DateTime<Utc>>,
    }
}

pub mod archive_policy {
    //! `archive_policy` 占位（per L4 #2068 M-2068.3 + RSK-LCM-005 N+2 冗余）
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ArchivePolicy {
        pub policy_id: Uuid,
        pub hot_tier_years: u32,
        pub cold_tier_years: u32,
        /// N+2 冗余副本数（per RSK-LCM-005 缓解）
        pub replica_count: u32,
    }
}

pub use retire_plan::{QueryChannelRbac, RetireChannelRole, RetirePlan, RetirePlanConfig};
