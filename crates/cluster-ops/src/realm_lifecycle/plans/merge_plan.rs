//! merge_plan 占位（per RGS-SPEC-DTL-042 §2 + FR-LCM-041 / FR-LCM-051 merge_rollback）
//!
//! DDL 目标表：`merge_conflict_rule_set_v2`（per SPEC §2 DDL；merge_rollback 走
//! 本 plan 的逆向补偿路径，不另立表）

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergePlan {
    pub plan_id: Uuid,
    pub request_id: Uuid,
    pub source_realm_ids: Vec<Uuid>,
    pub target_realm_id: Uuid,
    /// v2 规则集版本（per FR-LCM-062：locked_at 锁定后**不**允许运行时修改）
    pub conflict_rule_set_version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MergePlan {
    pub fn placeholder(plan_id: Uuid, request_id: Uuid) -> Self {
        Self {
            plan_id,
            request_id,
            source_realm_ids: vec![],
            target_realm_id: Uuid::new_v4(),
            conflict_rule_set_version: 2,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_merge_v2_default() {
        let plan_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let p = MergePlan::placeholder(plan_id, request_id);
        // per FR-LCM-062：v2 是当前规则集版本
        assert_eq!(p.conflict_rule_set_version, 2);
        assert!(p.source_realm_ids.is_empty());
    }
}
