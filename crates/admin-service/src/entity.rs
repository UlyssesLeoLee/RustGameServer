//! admin-service 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-019 §3 + ARC-051 COC/CEM）
//! - AdminUser：管理员账号（per COC RBAC）
//! - AuditLogEntry：审计日志（per RGS-SEC-100 §7 hash 链 + UPDATE/DELETE 触发器禁）
//!
//! 55.13 实化：audit_log hash 升级 FNV-1a → SHA-256 + 长度前缀防 separator 碰撞
//! （per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// 管理员角色（per ARC-051 COC RBAC）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    /// 超级管理员
    SuperAdmin,
    /// 域管理员（player/economy/match/social/admin）
    DomainAdmin,
    /// 审计员（只读）
    Auditor,
    /// 客服
    Support,
}

/// 管理员账号（per RGS-DTL-019 §3.1 COC RBAC）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminUser {
    /// 管理员 ID
    pub id: Uuid,
    /// 用户名（唯一）
    pub username: String,
    /// 密码哈希（argon2id）
    pub password_hash: String,
    /// 角色
    pub role: AdminRole,
    /// 关联域（None = 全域 / 跨域管理员）
    pub domain_scope: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近登录时间
    pub last_login_at: Option<DateTime<Utc>>,
    /// 停用时间（None = 启用中）
    pub disabled_at: Option<DateTime<Utc>>,
}

impl AdminUser {
    /// 工厂：新建管理员
    pub fn new(username: String, password_hash: String, role: AdminRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash,
            role,
            domain_scope: None,
            created_at: Utc::now(),
            last_login_at: None,
            disabled_at: None,
        }
    }

    /// 是否启用
    pub fn is_active(&self) -> bool {
        self.disabled_at.is_none()
    }

    /// 是否有权操作指定域
    pub fn can_admin_domain(&self, domain: &str) -> bool {
        match self.role {
            AdminRole::SuperAdmin => true,
            AdminRole::DomainAdmin => self
                .domain_scope
                .as_deref()
                .map(|d| d == domain)
                .unwrap_or(false),
            AdminRole::Auditor => false,
            AdminRole::Support => false,
        }
    }
}

/// 审计日志条目（per RGS-SEC-100 §7 hash 链防篡改）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    /// 日志 ID
    pub id: Uuid,
    /// 操作者 ID（管理员 / 系统）
    pub actor_id: Uuid,
    /// 操作（per ARC-051 COC 命令命名空间：player.ban / economy.grant 等）
    pub action: String,
    /// 操作目标（如 player_id / guild_id）
    pub target: String,
    /// payload JSON
    pub payload: String,
    /// 前一条 hash（hash 链）
    pub prev_hash: String,
    /// 当前条目 hash = sha256(actor_id + action + target + payload + prev_hash + created_at)
    pub hash: String,
    /// 时间
    pub created_at: DateTime<Utc>,
}

impl AuditLogEntry {
    /// 工厂：新建审计日志（prev_hash 由调用方提供 = 上条 hash；首条 prev_hash = "0" * 64）
    pub fn new(
        actor_id: Uuid,
        action: String,
        target: String,
        payload: String,
        prev_hash: String,
    ) -> Self {
        let now = Utc::now();
        let hash = compute_hash(actor_id, &action, &target, &payload, &prev_hash, now);
        Self {
            id: Uuid::new_v4(),
            actor_id,
            action,
            target,
            payload,
            prev_hash,
            hash,
            created_at: now,
        }
    }
}

/// sha256 hash 链（per RGS-SEC-100 §7）
///
/// 55.13 升级：FNV-1a 64-bit → SHA-256（per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）。
/// 每段前先写 `len.to_le_bytes()` 再写数据，长度前缀保证域边界明确，构造攻击
/// （`action="a|" target="b"` vs `action="a" target="b|"`）无法产生同一 hash。
fn compute_hash(
    actor_id: Uuid,
    action: &str,
    target: &str,
    payload: &str,
    prev_hash: &str,
    created_at: DateTime<Utc>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(actor_id.as_bytes());
    hasher.update((action.len() as u64).to_le_bytes());
    hasher.update(action.as_bytes());
    hasher.update((target.len() as u64).to_le_bytes());
    hasher.update(target.as_bytes());
    hasher.update((payload.len() as u64).to_le_bytes());
    hasher.update(payload.as_bytes());
    hasher.update(prev_hash.as_bytes());
    hasher.update(created_at.timestamp_millis().to_le_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_user_can_admin_domain() {
        let super_admin = AdminUser::new(
            "root".to_string(),
            "hash".to_string(),
            AdminRole::SuperAdmin,
        );
        assert!(super_admin.can_admin_domain("player"));
        assert!(super_admin.can_admin_domain("economy"));

        let mut domain_admin =
            AdminUser::new("pm".to_string(), "hash".to_string(), AdminRole::DomainAdmin);
        domain_admin.domain_scope = Some("player".to_string());
        assert!(domain_admin.can_admin_domain("player"));
        assert!(!domain_admin.can_admin_domain("economy"));

        let auditor = AdminUser::new("a".to_string(), "hash".to_string(), AdminRole::Auditor);
        assert!(!auditor.can_admin_domain("player"));
    }

    #[test]
    fn audit_log_hash_chain_changes() {
        let actor = Uuid::new_v4();
        let e1 = AuditLogEntry::new(
            actor,
            "player.ban".to_string(),
            "player-1".to_string(),
            "{}".to_string(),
            "0".repeat(64),
        );
        let e2 = AuditLogEntry::new(
            actor,
            "player.unban".to_string(),
            "player-1".to_string(),
            "{}".to_string(),
            e1.hash.clone(),
        );
        assert_ne!(e1.hash, e2.hash);
        assert_eq!(e2.prev_hash, e1.hash);
    }

    /// 55.13 AC5=CC1：1000 随机输入验证 hash 全部不同（防碰撞基本性质）。
    /// SHA-256 在 32 字节输出上生日攻击需 2^128 次，1000 条全不同概率上确近乎 1。
    #[test]
    fn compute_hash_collision_resistance_basic() {
        use std::collections::HashSet;
        let actor = Uuid::nil();
        let fixed_ts = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let mut seen: HashSet<String> = HashSet::with_capacity(1024);
        for i in 0..1000u32 {
            // 在 action / target / payload 三个域内变化以制造不同输入
            let h = compute_hash(
                actor,
                &format!("action.{i}"),
                &format!("target.{i}"),
                &format!("{{\"i\":{i}}}"),
                "0".repeat(64).as_str(),
                fixed_ts,
            );
            assert_eq!(h.len(), 64, "SHA-256 hex 应为 64 字符");
            assert!(seen.insert(h.clone()), "碰撞 i={i}: hash={h}");
        }
        assert_eq!(seen.len(), 1000);
    }

    /// 55.13 AC5=CH3：构造 action/target 拼接碰撞（FNV-1a 旧版缺陷）。
    /// `action="a|" target="b"` vs `action="a" target="b|"` —— 旧版 `"|"` 分隔
    /// 会产生同一字符串，因此旧 FNV-1a 给出同一 hash。SHA-256 + 长度前缀下两
    /// 个输入 hash 必不同。
    #[test]
    fn compute_hash_separator_independence() {
        let actor = Uuid::nil();
        let fixed_ts = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let h1 = compute_hash(actor, "a|", "b", "p", "0".repeat(64).as_str(), fixed_ts);
        let h2 = compute_hash(actor, "a", "|b", "p", "0".repeat(64).as_str(), fixed_ts);
        assert_ne!(
            h1, h2,
            "action/target 边界变化必须产生不同 hash（55.13 AC5=CH3 separator 碰撞防御）"
        );
    }
}
