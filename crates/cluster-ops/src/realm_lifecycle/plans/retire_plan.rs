//! `retire_plan` 计划表（per DTL-042 §7.2 + IMPL §3.3 M-2068.3 + SPEC §3 第 8 条）。
//!
//! ## 硬约束
//!
//! `query_channel_rbac` 角色配置默认 `["cs_agent", "sre", "legal"]`；
//! 退场后查询通道**仅**对配置角色开放（其他角色 → `Error::RetiredQueryDenied`）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::{
    error::{Error, Result},
    RealmId,
};

use super::super::operations::retire::DEFAULT_RETIRE_QUERY_ROLES;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirePlan {
    pub plan_id: String,
    pub realm_id: RealmId,
    pub query_channel_rbac: Vec<String>,
    /// 退场后归档启动阈值（天，per SPEC §8 实测参数 30-90 天）。
    pub archive_threshold_days: u32,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl RetirePlan {
    /// SPEC §3 第 8 条锚定：默认角色 = cs_agent / sre / legal。
    pub fn default_query_roles() -> Vec<String> {
        DEFAULT_RETIRE_QUERY_ROLES
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// SPEC §3 第 8 条锚定：判定角色是否被允许访问退场后查询通道。
    pub fn is_role_allowed(&self, role: &str) -> bool {
        self.query_channel_rbac.iter().any(|r| r == role)
    }

    /// 验证 query_channel_rbac 配置。
    pub fn validate_rbac(&self) -> Result<()> {
        for role in &self.query_channel_rbac {
            if role.is_empty() {
                return Err(Error::InvalidRetireRbac {
                    plan: self.plan_id.clone(),
                    role: "<empty>".to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RetirePlan {
        RetirePlan {
            plan_id: "rp-1".to_string(),
            realm_id: "rlm-1".to_string(),
            query_channel_rbac: RetirePlan::default_query_roles(),
            archive_threshold_days: 60,
            created_at: Utc::now(),
            created_by: "sre-1".to_string(),
        }
    }

    #[test]
    fn default_roles_match_spec() {
        let roles = RetirePlan::default_query_roles();
        assert_eq!(roles, vec!["cs_agent", "sre", "legal"]);
    }

    #[test]
    fn allowed_role_passes() {
        let p = sample();
        assert!(p.is_role_allowed("cs_agent"));
        assert!(p.is_role_allowed("sre"));
        assert!(p.is_role_allowed("legal"));
    }

    #[test]
    fn non_allowed_role_rejected() {
        let p = sample();
        assert!(!p.is_role_allowed("player"));
        assert!(!p.is_role_allowed("anonymous"));
    }

    #[test]
    fn validate_rbac_passes_for_valid_plan() {
        assert!(sample().validate_rbac().is_ok());
    }

    #[test]
    fn validate_rbac_rejects_empty_role() {
        let mut p = sample();
        p.query_channel_rbac.push(String::new());
        let r = p.validate_rbac();
        assert!(matches!(r, Err(Error::InvalidRetireRbac { .. })));
    }
}
