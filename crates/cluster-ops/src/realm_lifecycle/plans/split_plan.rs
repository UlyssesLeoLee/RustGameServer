//! split_plan 占位（per RGS-SPEC-DTL-042 §2 + FR-LCM-031）
//!
//! DDL 目标表：`split_plan`（per SPEC §2 DDL）

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SplitPlan {
    pub plan_id: Uuid,
    pub request_id: Uuid,
    pub source_realm_id: Uuid,
    pub target_realm_ids: Vec<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SplitPlan {
    pub fn placeholder(plan_id: Uuid, request_id: Uuid) -> Self {
        Self {
            plan_id,
            request_id,
            source_realm_id: Uuid::new_v4(),
            target_realm_ids: vec![],
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_split_empty_targets_by_default() {
        let plan_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let p = SplitPlan::placeholder(plan_id, request_id);
        assert!(p.target_realm_ids.is_empty());
    }
}
