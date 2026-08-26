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

/// Feature 类型（per RGS-DTL-031 §1.1：bounded_context / plugin / patch / config / realm_lifecycle）
///
/// RealmLifecycle 是 ARC-038 扩展 Feature 类型，作为 AD 限界上下文的扩展功能走 PFAU 编排
/// （per RGS-SPEC-DTL-042 §3 第 2 条 + NFR-LCM-007），不另起一套编排通道。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FeatureType {
    /// 独立 DB / gRPC / Helm 限界上下文（per ARC-018）
    BoundedContext,
    /// 依附宿主的编译期或沙箱特性（per ARC-021）
    Plugin,
    /// 通过编译期特性或沙箱脚本发布的补丁（per ADR-0051）
    Patch,
    /// 配置 / 特性开关在安全边界内切换（per ARC-016）
    Config,
    /// 服务器全生命周期 Feature（per ARC-038 扩展 + DTL-042 + DTL-031 §1.1）
    ///
    /// 走 5 状态 PFAU 编排；7 个 SubFeature 子类（new_realm / scale / split /
    /// merge / merge_rollback / retire / archive）。
    RealmLifecycle,
}

impl FeatureType {
    /// 字符串表示（snake_case）
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatureType::BoundedContext => "bounded_context",
            FeatureType::Plugin => "plugin",
            FeatureType::Patch => "patch",
            FeatureType::Config => "config",
            FeatureType::RealmLifecycle => "realm_lifecycle",
        }
    }
}

/// realm_lifecycle Feature 子类（per RGS-SPEC-DTL-042 §3 第 2 条）
///
/// 7 个子类必须全部注册到 FeatureRegistry（per DTL-031 §5 发布 + SPEC §5 硬约束）。
/// FeatureType::RealmLifecycle 下的 SubFeature 共同走 5 状态 PFAU 状态机
/// （declared / active / upgrade_pending / canary_in_progress / paused 等）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SubFeature {
    /// 开新服（per FR-LCM-001）
    NewRealm,
    /// 扩缩容（per FR-LCM-002）
    Scale,
    /// 分服（per FR-LCM-031）
    Split,
    /// 合服（per FR-LCM-041）
    Merge,
    /// 合服回退（per FR-LCM-051，merge_rollback 是 merge 的逆向补偿）
    MergeRollback,
    /// 退场（per FR-LCM-061）
    Retire,
    /// 归档（per FR-LCM-081，**仅**迁移存储位置，不删除数据）
    Archive,
}

impl SubFeature {
    /// 字符串表示（snake_case）
    pub fn as_str(&self) -> &'static str {
        match self {
            SubFeature::NewRealm => "new_realm",
            SubFeature::Scale => "scale",
            SubFeature::Split => "split",
            SubFeature::Merge => "merge",
            SubFeature::MergeRollback => "merge_rollback",
            SubFeature::Retire => "retire",
            SubFeature::Archive => "archive",
        }
    }

    /// 全部 7 个子类的稳定迭代顺序（per IT：Feature 子类注册 100% 命中验证）
    pub const ALL: &'static [SubFeature] = &[
        SubFeature::NewRealm,
        SubFeature::Scale,
        SubFeature::Split,
        SubFeature::Merge,
        SubFeature::MergeRollback,
        SubFeature::Retire,
        SubFeature::Archive,
    ];
}

/// PFAU 5 状态机（per RGS-DTL-031 §4.1 + SPEC-DTL-042 §3 阶段变更走 PFAU 编排）
///
/// declared → active → upgrade_pending → canary_in_progress → ...
/// RealmLifecycle 7 个 SubFeature 全部走这 5 状态；非法跳转在 feature_adapter 中
/// 作为业务错误拒绝并写审计（per DTL-031 §4.1 第 166 行硬约束）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PfauState {
    /// 已声明
    Declared,
    /// 已激活
    Active,
    /// 升级待执行
    UpgradePending,
    /// 金丝雀执行中
    CanaryInProgress,
    /// 已暂停（默认 120s 超时、健康丢失、fencing 失败、目标集合变化）
    Paused,
}

impl PfauState {
    /// 全部 5 状态迭代顺序
    pub const ALL: &'static [PfauState] = &[
        PfauState::Declared,
        PfauState::Active,
        PfauState::UpgradePending,
        PfauState::CanaryInProgress,
        PfauState::Paused,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            PfauState::Declared => "declared",
            PfauState::Active => "active",
            PfauState::UpgradePending => "upgrade_pending",
            PfauState::CanaryInProgress => "canary_in_progress",
            PfauState::Paused => "paused",
        }
    }
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

    // ===== M-2071.1 验收：FeatureType + SubFeature + PfauState =====

    #[test]
    fn feature_type_realm_lifecycle_exists() {
        // per RGS-DTL-031 §1.1：realm_lifecycle 5 大 Feature 类型之一
        assert_eq!(FeatureType::RealmLifecycle.as_str(), "realm_lifecycle");
    }

    #[test]
    fn feature_type_all_distinct() {
        let mut all: Vec<&'static str> = vec![
            FeatureType::BoundedContext.as_str(),
            FeatureType::Plugin.as_str(),
            FeatureType::Patch.as_str(),
            FeatureType::Config.as_str(),
            FeatureType::RealmLifecycle.as_str(),
        ];
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 5, "FeatureType 必须 5 个不重复变体");
    }

    #[test]
    fn sub_feature_count_is_seven() {
        // per RGS-SPEC-DTL-042 §3 第 2 条 + §5 发布：7 个子类必须全部注册
        assert_eq!(SubFeature::ALL.len(), 7);
    }

    #[test]
    fn sub_feature_seven_distinct() {
        // 7 子类不重复
        let mut all: Vec<&'static str> =
            SubFeature::ALL.iter().map(|s| s.as_str()).collect();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 7, "7 子类必须 snake_case 不重复");
    }

    #[test]
    fn sub_feature_required_snake_case_names() {
        // per RGS-SPEC-DTL-042 §3：7 子类名
        let names: Vec<&'static str> =
            SubFeature::ALL.iter().map(|s| s.as_str()).collect();
        for required in [
            "new_realm",
            "scale",
            "split",
            "merge",
            "merge_rollback",
            "retire",
            "archive",
        ] {
            assert!(
                names.contains(&required),
                "missing required sub_feature: {}",
                required
            );
        }
    }

    #[test]
    fn pfau_state_count_is_five() {
        // per RGS-DTL-031 §4.1：5 状态 PFAU 编排
        assert_eq!(PfauState::ALL.len(), 5);
    }
}
