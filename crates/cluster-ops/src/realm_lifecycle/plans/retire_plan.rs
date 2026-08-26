//! retire_plan 占位（per RGS-SPEC-DTL-042 §2 + FR-LCM-061）
//!
//! DDL 目标表：`retire_plan`（per SPEC §2 DDL）
//!
//! 退场后 RBAC 查询通道**仅**对 `query_channel_rbac` 配置的角色开放
//! （默认 `cs_agent` / `sre` / `legal`，per SPEC §3 第末段）

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetirePlan {
    pub plan_id: Uuid,
    pub request_id: Uuid,
    pub target_realm_id: Uuid,
    /// 默认 `cs_agent` / `sre` / `legal`（per SPEC §3）
    pub query_channel_rbac: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl RetirePlan {
    pub fn placeholder(plan_id: Uuid, request_id: Uuid) -> Self {
        Self {
            plan_id,
            request_id,
            target_realm_id: Uuid::new_v4(),
            query_channel_rbac: vec![
                "cs_agent".to_string(),
                "sre".to_string(),
                "legal".to_string(),
            ],
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retire_default_rbac_roles() {
        // per SPEC §3：默认 cs_agent / sre / legal
        let p = RetirePlan::placeholder(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(p.query_channel_rbac.len(), 3);
        assert!(p.query_channel_rbac.contains(&"cs_agent".to_string()));
        assert!(p.query_channel_rbac.contains(&"sre".to_string()));
        assert!(p.query_channel_rbac.contains(&"legal".to_string()));
    }
}
