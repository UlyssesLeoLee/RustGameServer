//! scale_plan 占位（per RGS-SPEC-DTL-042 §2 + FR-LCM-002）
//!
//! DDL 目标表：`scale_plan`（per SPEC §2 DDL 命名约定）

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalePlan {
    pub plan_id: Uuid,
    pub request_id: Uuid,
    pub target_realm_id: Uuid,
    /// 目标 shard 数（扩 = +N / 缩 = -N）
    pub delta_shards: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ScalePlan {
    pub fn placeholder(plan_id: Uuid, request_id: Uuid) -> Self {
        Self {
            plan_id,
            request_id,
            target_realm_id: Uuid::new_v4(),
            delta_shards: 0,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_scale_has_distinct_ids() {
        let plan_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let p = ScalePlan::placeholder(plan_id, request_id);
        assert_eq!(p.plan_id, plan_id);
        assert_eq!(p.request_id, request_id);
        assert_eq!(p.delta_shards, 0);
    }
}
