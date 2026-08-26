//! `split_plan` 计划表（per DTL-042 §7.2 + IMPL §3.3 M-2068.2）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::RealmId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPlan {
    pub plan_id: String,
    pub source_realm_id: RealmId,
    pub target_realm_id: RealmId,
    pub split_point_player_id: String,
    pub estimated_players: u64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_plan_fields_present() {
        let plan = SplitPlan {
            plan_id: "sp-1".to_string(),
            source_realm_id: "src".to_string(),
            target_realm_id: "tgt".to_string(),
            split_point_player_id: "p-1000000".to_string(),
            estimated_players: 2_000_000,
            created_at: Utc::now(),
            created_by: "sre-1".to_string(),
        };
        assert_eq!(plan.source_realm_id, "src");
        assert_eq!(plan.target_realm_id, "tgt");
    }
}
