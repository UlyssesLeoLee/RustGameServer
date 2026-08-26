//! rgs-realm-lifecycle 7 阶段状态机（per RGS-DTL-042 §4.1）
//!
//! 状态枚举：NotYet / Active / Scaling / Splitting / Merging / Retired / Archived
//!
//! 合法转移（per DTL-042 §4.1 表格）：
//! - `(NotYet, Active)` —— 开新服
//! - `(Active, Scaling) | (Scaling, Active)` —— 扩缩容
//! - `(Active, Splitting) | (Splitting, Active)` —— 分服
//! - `(Active, Merging) | (Merging, Active)` —— 合服
//! - `(Active | Splitting | Merging, Retired)` —— 退场
//! - `(Retired, Active)` —— 二次激活
//! - `(Retired, Archived)` —— **归档（本任务 L4 #2074 目标状态）**

use serde::{Deserialize, Serialize};

/// 7 阶段状态（per RGS-DTL-042 §4.1）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealmLifecycleState {
    /// 逻辑服尚未创建
    NotYet,
    /// 运营中
    Active,
    /// 扩缩容中
    Scaling,
    /// 分服中
    Splitting,
    /// 合服中
    Merging,
    /// 已下线（数据保留）
    Retired,
    /// 已归档（**本任务 L4 #2074 终态**）
    Archived,
}

/// 阶段变更 Feature 子类（per ARC-051 `realm_lifecycle` Feature 7 子类）
///
/// 实际注册到 `FeatureRegistry` 在 PH-4 / ClusterOpsService 集成时落地（per
/// RGS-DTL-042 §8 + RGS-SPEC-DTL-042 §3 "realm_lifecycle::* Feature 子类"）。
pub mod feature_subtypes {
    /// 开新服
    pub const NEW_REALM: &str = "realm_lifecycle::new_realm";
    /// 扩缩容
    pub const SCALE: &str = "realm_lifecycle::scale";
    /// 分服
    pub const SPLIT: &str = "realm_lifecycle::split";
    /// 合服
    pub const MERGE: &str = "realm_lifecycle::merge";
    /// 合服回退
    pub const MERGE_ROLLBACK: &str = "realm_lifecycle::merge_rollback";
    /// 退场
    pub const RETIRE: &str = "realm_lifecycle::retire";
    /// **归档（本任务 L4 #2074 目标 Feature 子类）**
    pub const ARCHIVE: &str = "realm_lifecycle::archive";
}

/// 重新导出常用 Feature 子类常量（避免调用方写 `feature_subtypes::ARCHIVE`）
pub use feature_subtypes::ARCHIVE as FEATURE_SUBTYPE_ARCHIVE;

impl RealmLifecycleState {
    /// 判断 (self → next) 是否为合法状态转移
    ///
    /// 严格按 DTL-042 §4.1 表格实现（任何表格外组合返回 `false`）。
    pub fn can_transition_to(self, next: RealmLifecycleState) -> bool {
        tracing::debug!(
            operation = "pfau_state_transition_check",
            service = "cluster-ops",
            method = "can_transition_to",
            from = ?self,
            to = ?next,
            "check pfau state transition"
        );
        use RealmLifecycleState::*;
        match (self, next) {
            // 开新服
            (NotYet, Active) => true,
            // 扩缩容（进入 / 退出）
            (Active, Scaling) | (Scaling, Active) => true,
            // 分服
            (Active, Splitting) | (Splitting, Active) => true,
            // 合服
            (Active, Merging) | (Merging, Active) => true,
            // 退场（从 Active / Splitting / Merging 三态均可触发）
            (Active, Retired) | (Splitting, Retired) | (Merging, Retired) => true,
            // 二次激活（退场后 30 天内）
            (Retired, Active) => true,
            // **归档（仅 Retired → Archived 合法）**
            (Retired, Archived) => true,
            // 其他
            _ => false,
        }
    }

    /// 是否终态
    pub fn is_terminal(self) -> bool {
        tracing::debug!(
            operation = "pfau_state_query",
            service = "cluster-ops",
            method = "is_terminal",
            state = ?self,
            "query if pfau state is terminal"
        );
        // 当前实现下 Archived 为终态（per FR-LCM-081 归档后仅迁移存储位置，
        // 不进入二次状态变更）；如需"二次激活归档服"需先经合规审批并扩 DTL。
        matches!(self, RealmLifecycleState::Archived)
    }

    /// 是否可发起归档前置转换（仅 Retired 状态可发起，per DTL-042 §4.1 表格）
    pub fn is_archive_eligible(self) -> bool {
        tracing::debug!(
            operation = "pfau_state_query",
            service = "cluster-ops",
            method = "is_archive_eligible",
            state = ?self,
            "query if pfau state is archive-eligible"
        );
        matches!(self, RealmLifecycleState::Retired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RealmLifecycleState::*;

    #[test]
    fn legal_transitions_match_dtl_042() {
        // DTL-042 §4.1 表格正例
        assert!(NotYet.can_transition_to(Active));
        assert!(Active.can_transition_to(Scaling));
        assert!(Scaling.can_transition_to(Active));
        assert!(Active.can_transition_to(Splitting));
        assert!(Splitting.can_transition_to(Active));
        assert!(Active.can_transition_to(Merging));
        assert!(Merging.can_transition_to(Active));
        assert!(Active.can_transition_to(Retired));
        assert!(Splitting.can_transition_to(Retired));
        assert!(Merging.can_transition_to(Retired));
        assert!(Retired.can_transition_to(Active));
        // 归档
        assert!(Retired.can_transition_to(Archived));
    }

    #[test]
    fn illegal_transitions_rejected() {
        // 非法跳转必须返回 false
        assert!(!NotYet.can_transition_to(Retired));
        assert!(!NotYet.can_transition_to(Archived));
        assert!(!Active.can_transition_to(Archived));
        assert!(!Archived.can_transition_to(Active));
        assert!(!Archived.can_transition_to(Retired));
        assert!(!Scaling.can_transition_to(Retired));
        assert!(!Scaling.can_transition_to(Archived));
    }

    #[test]
    fn archived_is_terminal() {
        assert!(Archived.is_terminal());
        assert!(!Active.is_terminal());
        assert!(!Retired.is_terminal());
    }

    #[test]
    fn archive_eligible_only_from_retired() {
        assert!(Retired.is_archive_eligible());
        assert!(!Active.is_archive_eligible());
        assert!(!NotYet.is_archive_eligible());
        assert!(!Archived.is_archive_eligible());
    }
}
