//! cluster-ops 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-020 §3 + ARC-051 集群运营中心）
//! - ClusterNode：跨服 Active-Active 节点
//! - FeatureFlag：PFAU 每功能原子升级（per DEC-002 all-reachable）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 节点角色（per ARC-051 跨服 Active-Active）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeRole {
    /// 主节点
    Primary,
    /// 副本节点（Active-Active）
    Replica,
    /// 候选节点（PFAU 升级中）
    Candidate,
}

/// 节点状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// 健康
    Healthy,
    /// 降级（仍可服务）
    Degraded,
    /// 不可服务
    Unhealthy,
    /// 维护中
    Maintenance,
}

/// 集群节点（per RGS-DTL-020 §3.1 + ARC-051 跨服 Active-Active）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterNode {
    /// 节点 ID
    pub id: Uuid,
    /// 主机名
    pub hostname: String,
    /// IP
    pub ip: String,
    /// 角色
    pub role: NodeRole,
    /// 状态
    pub status: NodeStatus,
    /// 最近心跳时间
    pub last_heartbeat_at: DateTime<Utc>,
    /// 节点二进制版本
    pub version: String,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 启用时间
    pub enabled_at: Option<DateTime<Utc>>,
}

impl ClusterNode {
    /// 工厂：新建节点
    pub fn new(hostname: String, ip: String, role: NodeRole, version: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            hostname,
            ip,
            role,
            status: NodeStatus::Healthy,
            last_heartbeat_at: now,
            version,
            registered_at: now,
            enabled_at: Some(now),
        }
    }

    /// 心跳刷新
    pub fn heartbeat(&mut self) {
        self.last_heartbeat_at = Utc::now();
        if matches!(self.status, NodeStatus::Unhealthy) {
            self.status = NodeStatus::Healthy;
        }
    }

    /// 标记不可服务
    pub fn mark_unhealthy(&mut self) {
        self.status = NodeStatus::Unhealthy;
    }
}

/// 功能开关作用域
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlagScope {
    /// 全局
    Global,
    /// 域（player/economy/match/social/admin）
    Domain,
    /// 节点级
    Node,
}

/// PFAU 功能开关（per RGS-ARC-051 + DEC-002 all-reachable + DEC-001 PFAU）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureFlag {
    /// 开关 key
    pub key: String,
    /// 作用域
    pub scope: FlagScope,
    /// 作用域值（域 / 节点 ID；Global 时为 "*"）
    pub scope_value: String,
    /// 是否启用
    pub enabled: bool,
    /// 当前版本（per PFAU 原子升级，递增）
    pub version: i64,
    /// 更新者（管理员 ID）
    pub updated_by: Uuid,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl FeatureFlag {
    /// 工厂：新建 flag
    pub fn new(key: String, scope: FlagScope, scope_value: String, updated_by: Uuid) -> Self {
        Self {
            key,
            scope,
            scope_value,
            enabled: false,
            version: 0,
            updated_by,
            updated_at: Utc::now(),
        }
    }

    /// 启用（version+1）
    pub fn enable(&mut self, by: Uuid) {
        self.enabled = true;
        self.version += 1;
        self.updated_by = by;
        self.updated_at = Utc::now();
    }

    /// 禁用
    pub fn disable(&mut self, by: Uuid) {
        self.enabled = false;
        self.version += 1;
        self.updated_by = by;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_heartbeat_recovers() {
        let mut n = ClusterNode::new(
            "h1".to_string(),
            "1.1.1.1".to_string(),
            NodeRole::Primary,
            "0.1.0".to_string(),
        );
        n.mark_unhealthy();
        assert_eq!(n.status, NodeStatus::Unhealthy);
        n.heartbeat();
        assert_eq!(n.status, NodeStatus::Healthy);
    }

    #[test]
    fn feature_flag_version_increments() {
        let admin = Uuid::new_v4();
        let mut f = FeatureFlag::new(
            "player.daily_reward.v2".to_string(),
            FlagScope::Domain,
            "player".to_string(),
            admin,
        );
        assert_eq!(f.version, 0);
        assert!(!f.enabled);
        f.enable(admin);
        assert_eq!(f.version, 1);
        assert!(f.enabled);
        f.disable(admin);
        assert_eq!(f.version, 2);
        assert!(!f.enabled);
    }
}
