//! `new_realm_plan` 计划表（per DTL-042 §7.2 + IMPL §3.3 M-2068.2）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::RealmId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRealmPlan {
    pub plan_id: String,
    pub realm_id: RealmId,
    pub region: String,
    pub initial_capacity: u32,
    pub initial_node_count: u32,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_realm_plan_fields_present() {
        let plan = NewRealmPlan {
            plan_id: "plan-1".to_string(),
            realm_id: "rlm-1".to_string(),
            region: "ap-east-1".to_string(),
            initial_capacity: 5000,
            initial_node_count: 3,
            created_at: Utc::now(),
            created_by: "sre-1".to_string(),
        };
        assert_eq!(plan.realm_id, "rlm-1");
        assert_eq!(plan.initial_node_count, 3);
    }
}
