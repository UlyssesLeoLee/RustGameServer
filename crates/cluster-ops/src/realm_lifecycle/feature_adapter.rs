//! FeatureType 适配 + 7 SubFeature 注册 + PFAU 5 状态编排
//! （M-2071.1 + M-2071.2 + M-2071.3，per RGS-SPEC-DTL-042 §3 + DTL-031 §1.1/§4.1）
//!
//! 7 个 SubFeature **必须**全部注册到 FeatureRegistry 才能发布（per DTL-031 §5）；
//! 5 状态 PFAU 状态机是 RealmLifecycle Feature 类型与 4 大类型共同的底层编排
//! （per DTL-031 §4.1）。非法跳转在 `require_legal_transition` 拒绝并写审计
//! （per DTL-031 §4.1 第 166 行硬约束）。

use std::collections::HashSet;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{FeatureType, PfauState, SubFeature};
use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::operators::OperatorInput;

/// 一次 PFAU 状态转移（per DTL-031 §3.1 pfa_run_state）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PfauTransition {
    pub from: PfauState,
    pub to: PfauState,
    pub sub_feature: SubFeature,
    pub run_id: Uuid,
    pub request_id: Uuid,
    pub at: chrono::DateTime<chrono::Utc>,
}

/// 7 个 SubFeature 子类注册项（per DTL-031 §5 发布硬约束）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubFeatureRegistration {
    pub sub_feature: SubFeature,
    /// 业务阶段名（与 SubFeature::as_str() 一致）
    pub phase_name: String,
    /// 注册到 FeatureType::RealmLifecycle 名下
    pub feature_type: FeatureType,
    /// 当前 PFAU 状态（per DTL-031 §4.1 5 状态机）
    pub current_state: PfauState,
    /// 是否启用
    pub enabled: bool,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// 7 子类 Feature 注册表（per DTL-031 §5 + SPEC-DTL-042 §3 第 2 条 + §5）
///
/// FeatureRegistry 是 RealmLifecycleService 内部组件，**不**分发独立服务。
/// 7 个 SubFeature 全部注册（new_realm / scale / split / merge / merge_rollback /
/// retire / archive），缺一即视为 PFAU 编排不完整。
pub struct FeatureRegistry {
    inner: Mutex<Vec<SubFeatureRegistration>>,
}

impl FeatureRegistry {
    /// 构造并默认注册全部 7 子类（per M-2071.2 验收）
    pub fn with_default_seven() -> Self {
        let v = Self {
            inner: Mutex::new(Vec::with_capacity(SubFeature::ALL.len())),
        };
        for sf in SubFeature::ALL {
            v.register(SubFeatureRegistration {
                sub_feature: *sf,
                phase_name: sf.as_str().to_string(),
                feature_type: FeatureType::RealmLifecycle,
                current_state: PfauState::Declared,
                enabled: true,
                registered_at: chrono::Utc::now(),
            })
            .expect("default registration should not fail");
        }
        v
    }

    /// 注册单个 SubFeature（已存在则覆盖）
    pub fn register(&self, reg: SubFeatureRegistration) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        if let Some(slot) = g.iter_mut().find(|r| r.sub_feature == reg.sub_feature) {
            *slot = reg;
        } else {
            g.push(reg);
        }
        Ok(())
    }

    /// 全部注册项快照
    pub fn list(&self) -> Vec<SubFeatureRegistration> {
        self.inner.lock().unwrap().clone()
    }

    /// 7 子类是否全部注册（per M-2071.6 100% 命中验证）
    pub fn is_complete(&self) -> bool {
        let g = self.inner.lock().unwrap();
        if g.len() < SubFeature::ALL.len() {
            return false;
        }
        // 收集已注册 phase_name 到 owned Vec，再做包含性校验
        let names: Vec<String> = g.iter().map(|r| r.phase_name.clone()).collect();
        drop(g);
        let set: HashSet<String> = names.into_iter().collect();
        SubFeature::ALL
            .iter()
            .all(|sf| set.contains(sf.as_str()))
    }

    /// 按 sub_feature 查找
    pub fn find(&self, sub_feature: SubFeature) -> Option<SubFeatureRegistration> {
        self.inner
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.sub_feature == sub_feature)
            .cloned()
    }

    /// 按 phase 字符串查找（兼容 `cmd.phase` 是字符串的调用方）
    pub fn find_by_phase(&self, phase: &str) -> Option<SubFeatureRegistration> {
        self.inner
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.phase_name == phase)
            .cloned()
    }
}

/// RealmLifecycle Feature 适配器（per SPEC §2 + §3 第 2 条）
///
/// - 校验 7 SubFeature 全部注册；
/// - 校验 PFAU 5 状态机合法转移；
/// - 启动 PFAU run 并产出 `run_id`。
pub struct RealmLifecycleFeatureAdapter {
    // 适配器当前不持有额外状态；保留以备 PH-4 引入 Redis 短租约 + fencing。
}

impl Default for RealmLifecycleFeatureAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmLifecycleFeatureAdapter {
    pub fn new() -> Self {
        Self {}
    }

    /// 校验 phase 字符串在 7 SubFeature 中已注册（per FR-LCM-004 转发校验）
    pub fn require_registered(&self, phase: &str) -> std::result::Result<(), String> {
        match phase {
            "new_realm" | "scale" | "split" | "merge" | "merge_rollback" | "retire"
            | "archive" => Ok(()),
            other => {
                tracing::warn!(phase = other, "realm_lifecycle phase not registered");
                Err(other.to_string())
            }
        }
    }

    /// 校验状态机合法转移（per DTL-031 §4.1 5 状态 + 非法跳转硬约束）
    pub fn require_legal_transition(
        &self,
        from: PfauState,
        to: PfauState,
        sub_feature: SubFeature,
    ) -> Result<()> {
        use PfauState::*;
        let legal = matches!(
            (from, to),
            (Declared, Active)
                | (Active, UpgradePending)
                | (UpgradePending, CanaryInProgress)
                | (UpgradePending, Paused)
                | (CanaryInProgress, Paused)
                | (Paused, UpgradePending)
                | (Paused, Active)
        );
        if !legal {
            return Err(
                crate::realm_lifecycle::error::Error::PFAUIllegalTransition {
                    from: from.as_str(),
                    to: to.as_str(),
                    sub_feature: sub_feature.as_str(),
                },
            );
        }
        Ok(())
    }

    /// 启动 PFAU run（per M-2071.3 + DTL-031 §4.1）
    ///
    /// 返回 `run_id`（写入 `pfa_run_state` 表占位）。状态机从 `none -> Declared`。
    pub fn start_pfau_run(
        &self,
        _phase: &str,
        input: &OperatorInput,
    ) -> Result<Uuid> {
        // 占位：PH-4 引入 Redis 短租约 + fencing + pfa_run_state pg 落库
        // 步骤：1) 校验 phase 已注册（由 require_registered 提前保证）
        //      2) 校验 from==None → to==Declared 合法
        //      3) 写 pfa_run_state（占位仅返回 run_id）
        let _ = input;
        Ok(Uuid::new_v4())
    }

    /// 应用一次 PFAU 转移（per DTL-031 §3.1 pfa_run_state + §4.1 状态机）
    pub fn apply_transition(&self, t: &PfauTransition) -> Result<()> {
        self.require_legal_transition(t.from, t.to, t.sub_feature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_all_seven_sub_features() {
        let r = FeatureRegistry::with_default_seven();
        assert!(r.is_complete(), "7 子类必须全部注册");
        assert_eq!(r.list().len(), 7);
    }

    #[test]
    fn registry_each_sub_feature_uses_realm_lifecycle_feature_type() {
        let r = FeatureRegistry::with_default_seven();
        for reg in r.list() {
            assert_eq!(reg.feature_type, FeatureType::RealmLifecycle);
        }
    }

    #[test]
    fn require_registered_accepts_all_seven_phases() {
        let a = RealmLifecycleFeatureAdapter::new();
        for p in [
            "new_realm",
            "scale",
            "split",
            "merge",
            "merge_rollback",
            "retire",
            "archive",
        ] {
            assert!(a.require_registered(p).is_ok(), "phase {} should be ok", p);
        }
    }

    #[test]
    fn require_registered_rejects_unknown_phase() {
        let a = RealmLifecycleFeatureAdapter::new();
        let err = a.require_registered("not_a_phase").unwrap_err();
        assert_eq!(err, "not_a_phase");
    }

    #[test]
    fn pfau_legal_transition_examples() {
        let a = RealmLifecycleFeatureAdapter::new();
        assert!(a
            .require_legal_transition(
                PfauState::Declared,
                PfauState::Active,
                SubFeature::NewRealm
            )
            .is_ok());
        assert!(a
            .require_legal_transition(
                PfauState::Active,
                PfauState::UpgradePending,
                SubFeature::Scale
            )
            .is_ok());
    }

    #[test]
    fn pfau_illegal_transition_rejected() {
        let a = RealmLifecycleFeatureAdapter::new();
        let err = a
            .require_legal_transition(
                PfauState::Paused,
                PfauState::Declared,
                SubFeature::Merge,
            )
            .unwrap_err();
        assert!(matches!(
            err,
            crate::realm_lifecycle::error::Error::PFAUIllegalTransition { .. }
        ));
    }
}
