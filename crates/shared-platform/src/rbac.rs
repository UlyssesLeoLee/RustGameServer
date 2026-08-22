//! RBAC 角色基础访问控制（per RGS-DTL-019 §3 + DEC-005 5 域 Lead + ARC-051 COC）
//!
//! 54.15 实化：Subject / Role / Permission / Authorizer trait + CheckResult
//!
//! 设计：
//! - Subject 抽象主体（管理员 ID / 玩家 ID / 系统）
//! - Role 角色（SuperAdmin / DomainAdmin / Auditor / Support / Player 等）
//! - Permission 权限（resource:action 形式）
//! - Authorizer trait：抽象授权检查（业务方 impl）
//! - CheckResult：Allow / Deny { reason }
//!
//! 用法：
//! ```no_run
//! use shared_platform::rbac::*;
//! let authorizer = SimpleAuthorizer::new();
//! let result = authorizer.check(&subject, "player:ban", "player:123");
//! match result { CheckResult::Allow => ..., CheckResult::Deny { reason } => ... }
//! ```

use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// RBAC 错误
#[derive(Debug, Error)]
pub enum RbacError {
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("unknown role: {0}")]
    UnknownRole(String),
}

/// 主体（管理员 / 玩家 / 系统）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    /// 主体 ID
    pub id: Uuid,
    /// 主体类型（admin / player / system）
    pub subject_type: SubjectType,
    /// 角色列表
    pub roles: Vec<Role>,
    /// 域 scope（None = 全域 / Some("player") = 仅 player 域）
    pub domain_scope: Option<String>,
}

/// 主体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectType {
    Admin,
    Player,
    System,
}

/// 角色（per RGS-DTL-019 §3.1 + DEC-005 5 域 Lead）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// 超级管理员（全权）
    SuperAdmin,
    /// 域管理员（域 scope 限定）
    DomainAdmin,
    /// 审计员（只读）
    Auditor,
    /// 客服
    Support,
    /// 玩家
    Player,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::SuperAdmin => "super_admin",
            Role::DomainAdmin => "domain_admin",
            Role::Auditor => "auditor",
            Role::Support => "support",
            Role::Player => "player",
        }
    }
}

impl std::str::FromStr for Role {
    type Err = RbacError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "super_admin" => Ok(Role::SuperAdmin),
            "domain_admin" => Ok(Role::DomainAdmin),
            "auditor" => Ok(Role::Auditor),
            "support" => Ok(Role::Support),
            "player" => Ok(Role::Player),
            other => Err(RbacError::UnknownRole(other.to_string())),
        }
    }
}

/// 权限检查结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// 允许
    Allow,
    /// 拒绝 + 原因
    Deny { reason: String },
}

impl CheckResult {
    pub fn is_allow(&self) -> bool {
        matches!(self, CheckResult::Allow)
    }

    pub fn deny_if(reason: impl Into<String>) -> Self {
        CheckResult::Deny {
            reason: reason.into(),
        }
    }
}

/// Authorizer trait（业务方 impl）
pub trait Authorizer: Send + Sync {
    /// 检查 subject 是否有 permission 在 resource 上
    fn check(&self, subject: &Subject, permission: &str, resource: &str) -> CheckResult;
}

/// 简单 Authorizer 实现：role → permission set 静态映射
pub struct SimpleAuthorizer {
    /// role → permissions 映射
    role_permissions: HashMap<Role, Vec<&'static str>>,
}

impl SimpleAuthorizer {
    /// 工厂：默认 role permission 映射（per DTL-019 + DEC-005）
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();
        // SuperAdmin: 全部
        role_permissions.insert(Role::SuperAdmin, vec!["*:*"]);
        // DomainAdmin: 域内全部 (scope 检查在 check 内做)
        role_permissions.insert(Role::DomainAdmin, vec!["*:*"]);
        // Auditor: 只读
        role_permissions.insert(Role::Auditor, vec!["*:read"]);
        // Support: 玩家查询
        role_permissions.insert(Role::Support, vec!["player:read", "guild:read"]);
        // Player: 自查
        role_permissions.insert(Role::Player, vec!["player:self"]);

        Self { role_permissions }
    }
}

impl Default for SimpleAuthorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Authorizer for SimpleAuthorizer {
    fn check(&self, subject: &Subject, permission: &str, resource: &str) -> CheckResult {
        for role in &subject.roles {
            // DomainAdmin scope check 优先（即使有 *:* 也要 scope 限定）
            if matches!(role, Role::DomainAdmin) {
                if let Some(scope) = &subject.domain_scope {
                    if !resource.starts_with(scope) {
                        return CheckResult::deny_if(format!(
                            "domain_admin scope {} not match resource {}",
                            scope, resource
                        ));
                    }
                }
            }

            if let Some(perms) = self.role_permissions.get(role) {
                for p in perms {
                    // *:* 全权（已在上面做了 scope 检查）
                    if *p == "*:*" {
                        return CheckResult::Allow;
                    }
                    // 通配符匹配 (e.g. "*:read" match "player:read")
                    if permission_matches(p, permission) {
                        // Player 只能操作 self
                        if matches!(role, Role::Player) && *p == "player:self" {
                            let subject_id_str = subject.id.to_string();
                            if resource != subject_id_str {
                                return CheckResult::deny_if("player can only access self");
                            }
                        }
                        return CheckResult::Allow;
                    }
                }
            }
        }
        CheckResult::deny_if(format!(
            "no role grants permission {} on resource {}",
            permission, resource
        ))
    }
}

/// 简单通配符匹配（支持 * 通配符）
fn permission_matches(pattern: &str, perm: &str) -> bool {
    if pattern == perm {
        return true;
    }
    // pattern "*:read" match "player:read" → split by ':', compare
    let p_parts: Vec<&str> = pattern.split(':').collect();
    let perm_parts: Vec<&str> = perm.split(':').collect();
    if p_parts.len() != perm_parts.len() {
        return false;
    }
    p_parts
        .iter()
        .zip(perm_parts.iter())
        .all(|(a, b)| *a == "*" || a == b)
}

/// 业务层 helper：检查后 deny 立即返回 Err
pub fn enforce(
    authorizer: &dyn Authorizer,
    subject: &Subject,
    permission: &str,
    resource: &str,
) -> Result<(), RbacError> {
    match authorizer.check(subject, permission, resource) {
        CheckResult::Allow => Ok(()),
        CheckResult::Deny { reason } => Err(RbacError::PermissionDenied(reason)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn super_admin() -> Subject {
        Subject {
            id: Uuid::new_v4(),
            subject_type: SubjectType::Admin,
            roles: vec![Role::SuperAdmin],
            domain_scope: None,
        }
    }

    fn player_admin() -> Subject {
        let mut s = super_admin();
        s.roles = vec![Role::DomainAdmin];
        s.domain_scope = Some("player".to_string());
        s
    }

    fn player() -> Subject {
        Subject {
            id: Uuid::new_v4(),
            subject_type: SubjectType::Player,
            roles: vec![Role::Player],
            domain_scope: None,
        }
    }

    #[test]
    fn super_admin_allows_all() {
        let a = SimpleAuthorizer::new();
        assert!(a
            .check(&super_admin(), "player:ban", "player:123")
            .is_allow());
        assert!(a
            .check(&super_admin(), "economy:transfer", "economy:1")
            .is_allow());
    }

    #[test]
    fn domain_admin_scoped() {
        let a = SimpleAuthorizer::new();
        assert!(a
            .check(&player_admin(), "player:ban", "player:123")
            .is_allow());
        // economy 不在 scope 内 → deny
        assert!(!a
            .check(&player_admin(), "economy:grant", "economy:1")
            .is_allow());
    }

    #[test]
    fn player_self_only() {
        let p = player();
        let a = SimpleAuthorizer::new();
        // 自己 → 允许
        assert!(a.check(&p, "player:self", &p.id.to_string()).is_allow());
        // 别人 → 拒绝
        assert!(!a.check(&p, "player:self", "other-player-id").is_allow());
    }

    #[test]
    fn auditor_read_only() {
        let mut a_sub = super_admin();
        a_sub.roles = vec![Role::Auditor];
        let a = SimpleAuthorizer::new();
        assert!(a.check(&a_sub, "player:read", "player:1").is_allow());
        assert!(!a.check(&a_sub, "player:ban", "player:1").is_allow());
    }

    #[test]
    fn permission_wildcard_match() {
        assert!(permission_matches("*:read", "player:read"));
        assert!(permission_matches("*:read", "economy:read"));
        assert!(!permission_matches("*:read", "player:ban"));
    }

    #[test]
    fn enforce_returns_err_on_deny() {
        let a = SimpleAuthorizer::new();
        let p = player();
        let result = enforce(&a, &p, "player:self", "other-id");
        assert!(matches!(result, Err(RbacError::PermissionDenied(_))));
    }

    #[test]
    fn role_from_str() {
        assert_eq!("super_admin".parse::<Role>().unwrap(), Role::SuperAdmin);
        assert!("unknown".parse::<Role>().is_err());
    }
}
