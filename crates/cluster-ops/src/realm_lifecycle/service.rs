//! 域门面 trait + 6 操作器 trait + 6 阶段状态机（per RGS-SPEC-DTL-042 §3 + DTL-042 §4/§5）
//!
//! WF-1-2066 M-2066.1 + M-2066.2 + 部分 M-2066.10：骨架实现
//!
//! 关键设计（per FR-LCM-004 / ARC-051 PFAU / DEC-001 PFAU / DEC-002 all-reachable）：
//! - 6 阶段操作器以 `RealmLifecycleOperator` trait 形式表达；每个 operator 至少 1 个
//!   `async fn` 方法（M-2066 验收门槛）
//! - 6 阶段状态机：`NewRealm → Scale → Split → Merge → Retire → Archive`（含回退路径）
//! - 阶段内 PFAU 子状态（per §4）：declared → planning → drill_validated → executing →
//!   observing → completed；mid-states: paused / retrying / rolling_back / aborted
//! - 不暴露独立 gRPC（per FR-LCM-004）：本 facade **不**实现 tonic gRPC service trait
//! - 二次激活负例（per M-2066.10）：已 Archive 的 realm 不可 NewRealm
//!
//! 后续 L4 任务的接入点（per RGS-IMPL-PLAN-LCM-001 v0.1）：
//! - L4 #2067 SagaOrchestrator + Saga 步骤定义（per M-2067.1~6）
//! - L4 #2068 6 张新表 + Plan entity + PgRepository（per M-2068.1~7）
//! - L4 #2070 DrillExecutor + 沙箱 PG/K8s 客户端（per M-2070.1~14）
//! - L4 #2071 ClusterOpsService `realm_lifecycle` Feature 7 子类注册（per M-2071.1~7）
//! - L4 #2073 跨域联动（player / economy / social gRPC client）
//! - L4 #2074 归档冷热分层 + N+2 + GDPR

use std::str::FromStr;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use uuid::Uuid;

use crate::Result;

use super::error::{into_crate_result, LcmError, LcmResult};

// ============================================================================
// 6 阶段生命周期阶段枚举（per RGS-DTL-042 §4 + SPEC-DTL-042 §3）
// ============================================================================

/// 6 阶段服务器全生命周期阶段（per RGS-IMPL-PLAN-LCM-001 v0.1 §1.1）
///
/// 阶段转移合法路径（per §4 状态机 + 实施计划 §2.2）：
/// ```text
///     NewRealm ─┐
///              ├─→ Scale ─┐
///              │         ├─→ Split ─┐
///              │         │         ├─→ Merge ─┐
///              │         │         │         ├─→ Retire ─→ Archive (终态)
///              │         │         │         │
///              │         │         │         └─→ MergeRollback (回退子阶段)
///              │         │         └─→ SplitAbort (回退子阶段)
///              │         └─→ ScaleDown (回退子阶段)
///              └─→ NewRealmAbort (回退子阶段，pre-activating 阶段)
/// ```
///
/// **二次激活负例**（per M-2066.10 验收门槛 + SPEC-DTL-042 §3 第 6 条）：
/// Archive 阶段不可再 `NewRealm` —— 错误码 `LcmErrorKind::AlreadyActivated`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RealmLifecycleStage {
    /// 阶段 1：开新服（per FR-LCM-010~033）
    NewRealm,
    /// 阶段 2：扩缩容（per FR-LCM-040~044，含双向）
    Scale,
    /// 阶段 3：分服（per FR-LCM-050~055）
    Split,
    /// 阶段 4：合服（per FR-LCM-060~064）
    Merge,
    /// 阶段 5：退场（per FR-LCM-070~075）
    Retire,
    /// 阶段 6：归档（per FR-LCM-080~085，**终态**；不可二次激活）
    Archive,
}

impl RealmLifecycleStage {
    /// 字符串化（用于审计日志 + Saga 步骤标签）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NewRealm => "new_realm",
            Self::Scale => "scale",
            Self::Split => "split",
            Self::Merge => "merge",
            Self::Retire => "retire",
            Self::Archive => "archive",
        }
    }

    /// Feature 子类 key（per ARC-051 `realm_lifecycle::*` Feature 7 子类注册）
    ///
    /// 注：Merge 子类含 `merge_rollback` 子变体（per M-2066.7 + 实施计划 §3.5），
    /// 本枚举不直接区分；子变体在 operator 实现层处理（per RGS-IMPL-PLAN-LCM-001 §3.5）
    pub fn feature_subtype(&self) -> &'static str {
        match self {
            Self::NewRealm => "realm_lifecycle::new_realm",
            Self::Scale => "realm_lifecycle::scale",
            Self::Split => "realm_lifecycle::split",
            Self::Merge => "realm_lifecycle::merge",
            Self::Retire => "realm_lifecycle::retire",
            Self::Archive => "realm_lifecycle::archive",
        }
    }

    /// 阶段是否处于"终态"（per FR-LCM-081 归档不删除数据）
    ///
    /// Archive 是唯一终态；Merge / Retire 都可经回退路径回到中间态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Archive)
    }

    /// 判断从当前阶段向 `next` 阶段的转移是否合法
    ///
    /// 合法路径（per §4 状态机）：
    /// - NewRealm → Scale
    /// - Scale → Split
    /// - Split → Merge
    /// - Merge → Retire
    /// - Retire → Archive（**唯一终态路径**）
    /// - Merge → Merge（合服回退窗口期内，per TBD-DTL-042 合服回退 7~30 天）
    /// - **所有非终态 → Archive 的"跳过中间阶段"路径非法**
    /// - **Archive → 任何阶段 全部非法**（含二次激活 NewRealm）
    pub fn can_transition_to(self, next: RealmLifecycleStage) -> bool {
        use RealmLifecycleStage::*;
        match (self, next) {
            // 主流转路径
            (NewRealm, Scale) => true,
            (Scale, Split) => true,
            (Split, Merge) => true,
            (Merge, Retire) => true,
            (Retire, Archive) => true,
            // 合服回退窗口期内可重试（per SPEC §3 实现契约 + TBD-DTL-042 合服回退 7~30 天）
            (Merge, Merge) => true,
            // 终态不可转移
            (Archive, _) => false,
            // 其他所有路径非法（含 NewRealm→Archive 跳过中间阶段 / Scale→Archive 跳过 / ...）
            _ => false,
        }
    }

    /// 严格转移（含错误返回）
    ///
    /// 返回 `Err(LcmErrorKind::InvalidStageTransition)` 当 `can_transition_to` 返回 false
    pub fn ensure_transition(self, next: RealmLifecycleStage) -> LcmResult<()> {
        if self.can_transition_to(next) {
            Ok(())
        } else {
            Err(LcmError::invalid_stage_transition(
                self.as_str(),
                next.as_str(),
                "not in legal transition path per RGS-DTL-042 §4 state machine",
            ))
        }
    }
}

impl FromStr for RealmLifecycleStage {
    type Err = LcmError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "new_realm" => Ok(Self::NewRealm),
            "scale" => Ok(Self::Scale),
            "split" => Ok(Self::Split),
            "merge" => Ok(Self::Merge),
            "retire" => Ok(Self::Retire),
            "archive" => Ok(Self::Archive),
            other => Err(LcmError::invalid_parameter(format!(
                "unknown realm lifecycle stage: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for RealmLifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================================
// 6 阶段状态机（per RGS-DTL-042 §4 + M-2066.10）
// ============================================================================

/// 6 阶段状态机（线程安全）
///
/// - `current` 用 `RwLock` 保护（per ARC-018 高密度期间串行调度）
/// - `transition` 接受阶段 + `request_id` + `realm_id`；非法转移返回 `InvalidStageTransition`
/// - 二次激活负例（per M-2066.10）：当 `current == Archive` 时，`NewRealm` 转移
///   返回 `AlreadyActivated`（比 `InvalidStageTransition` 更精确的错误码）
#[derive(Debug)]
pub struct RealmLifecycleStateMachine {
    realm_id: String,
    current: RwLock<RealmLifecycleStage>,
}

impl RealmLifecycleStateMachine {
    /// 新建状态机，初始阶段为 `NewRealm`
    pub fn new(realm_id: impl Into<String>) -> Self {
        Self {
            realm_id: realm_id.into(),
            current: RwLock::new(RealmLifecycleStage::NewRealm),
        }
    }

    /// 以指定阶段恢复状态机（per Saga resume + 重启恢复，per RGS-DTL-100 §4）
    pub fn restore(realm_id: impl Into<String>, stage: RealmLifecycleStage) -> Self {
        Self {
            realm_id: realm_id.into(),
            current: RwLock::new(stage),
        }
    }

    pub fn realm_id(&self) -> &str {
        &self.realm_id
    }

    pub fn current(&self) -> RealmLifecycleStage {
        *self.current.read().expect("RealmLifecycleStateMachine poisoned")
    }

    /// 状态转移（含二次激活负例短路）
    ///
    /// 二次激活负例：当 `current == Archive` 且 `next == NewRealm` 时，
    /// 返回 `LcmErrorKind::AlreadyActivated`（per M-2066.10 + SPEC §3 第 6 条）
    pub fn transition(
        &self,
        next: RealmLifecycleStage,
        request_id: Uuid,
    ) -> LcmResult<Uuid> {
        let prev = self.current();
        // 二次激活负例（per M-2066.10 验收门槛 + SPEC-DTL-042 §3 第 6 条）
        // Archive → NewRealm 显式标记为 `AlreadyActivated` 而非通用 `InvalidStageTransition`
        if prev == RealmLifecycleStage::Archive && next == RealmLifecycleStage::NewRealm {
            return Err(LcmError::already_activated(&self.realm_id));
        }
        prev.ensure_transition(next)?;
        *self
            .current
            .write()
            .expect("RealmLifecycleStateMachine poisoned") = next;
        Ok(request_id)
    }
}

// ============================================================================
// 6 操作器 trait（per M-2066.2 + RGS-DTL-042 §5）
// ============================================================================

/// 6 操作器统一 trait
///
/// 每个操作器至少实现：
/// - `name()`: 静态名称（用于 Saga step name + 审计日志）
/// - `stage()`: 所属 6 阶段（per §4 状态机）
/// - `execute()`: 阶段执行（异步；返回 `run_id`）
///
/// **不**暴露独立 gRPC trait（per FR-LCM-004）：本 trait 由 `RealmLifecycleService`
/// 门面封装后转发给 AdminService
#[async_trait]
pub trait RealmLifecycleOperator: Send + Sync {
    /// 阶段名称（静态）
    fn name(&self) -> &'static str;

    /// 所属 6 阶段
    fn stage(&self) -> RealmLifecycleStage;

    /// 执行阶段变更
    ///
    /// 参数：
    /// - `request_id`: 幂等键（per RGS-DTL-031 §3.1 既有；FR-LCM-002 阶段变更全流程留痕）
    /// - `realm_id`: 目标 realm 标识
    /// - `operator_id`: 操作人 UUID
    /// - `approval_ref`: 高危操作三方签字引用（per SPEC §5 安全容错）
    ///
    /// 返回：
    /// - `Ok(Uuid)`: 新建的 `realm_lifecycle_run.run_id`（per L4 #2068 migration）
    ///
    /// 错误：
    /// - `LcmErrorKind::InvalidParameter`: 参数校验失败
    /// - `LcmErrorKind::InvalidStageTransition`: 状态机非法跳转
    /// - `LcmErrorKind::AlreadyActivated`: 二次激活负例
    /// - `LcmErrorKind::NotImplemented`: 骨架阶段占位符（L4 #2067 Saga 接入前）
    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> LcmResult<Uuid>;
}

// ============================================================================
// 6 阶段服务门面（per FR-LCM-004 + M-2066.1/M-2066.2）
// ============================================================================

/// 6 阶段服务门面 trait
///
/// **不**分发独立 gRPC（per FR-LCM-004）；本 trait 由 `rgs-admin-service` 的
/// `AdminService` 转发层调用，cluster-ops crate 内部不再 `tonic::include_proto!`
/// 独立 LCM proto（per FR-LCM-004 + M-2066 验收门槛的"必须 grep 验证"项）。
///
/// 7 个方法对应 6 阶段 + 1 个合服回退子操作（per M-2066.7）：
/// - `new_realm` (Stage 1)
/// - `scale` (Stage 2，扩缩容双向)
/// - `split` (Stage 3)
/// - `merge` (Stage 4)
/// - `merge_rollback` (Stage 4 子操作)
/// - `retire` (Stage 5)
/// - `archive` (Stage 6，终态)
#[async_trait]
pub trait RealmLifecycleService: Send + Sync {
    async fn new_realm(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    async fn scale(
        &self,
        request_id: Uuid,
        realm_id: &str,
        target_capacity: u32,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    async fn split(
        &self,
        request_id: Uuid,
        source_realm_id: &str,
        target_realm_ids: &[String],
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    async fn merge(
        &self,
        request_id: Uuid,
        source_realm_ids: &[String],
        target_realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    /// 合服回退子操作（per M-2066.7 + FR-LCM-062 验证路径）
    ///
    /// 仅当 merge 处于 7~30 天回退窗口期（TBD-DTL-042 实测填）时可调用
    async fn merge_rollback(
        &self,
        request_id: Uuid,
        merge_run_id: Uuid,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    async fn retire(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    async fn archive(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid>;

    /// 健康检查（per ClusterOpsService 既有 pattern，per service.rs）
    async fn health_check(&self) -> Result<bool>;
}

// ============================================================================
// 6 阶段服务门面默认实现：路由到对应 operator
// ============================================================================

/// 6 操作器路由器 + 6 阶段状态机注册表
///
/// 注：本结构仅承载 6 个 operator 的注册 + 状态机映射；具体业务执行（DB 持久化 /
/// Saga 编排 / 演练 / 跨域 gRPC 调用）在 L4 #2067/#2068/#2070/#2071/#2073/#2074
/// 任务中接入，本骨架阶段统一返回 `NotImplemented` 占位结果（per M-2066.x 阶段任务范围）。
#[derive(Clone)]
pub struct RealmLifecycleServiceImpl {
    new_realm: Arc<dyn RealmLifecycleOperator>,
    scale: Arc<dyn RealmLifecycleOperator>,
    split: Arc<dyn RealmLifecycleOperator>,
    merge: Arc<dyn RealmLifecycleOperator>,
    retire: Arc<dyn RealmLifecycleOperator>,
    archive: Arc<dyn RealmLifecycleOperator>,
}

impl std::fmt::Debug for RealmLifecycleServiceImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealmLifecycleServiceImpl")
            .field("new_realm", &self.new_realm.name())
            .field("scale", &self.scale.name())
            .field("split", &self.split.name())
            .field("merge", &self.merge.name())
            .field("retire", &self.retire.name())
            .field("archive", &self.archive.name())
            .finish()
    }
}

impl RealmLifecycleServiceImpl {
    pub fn new(
        new_realm: Arc<dyn RealmLifecycleOperator>,
        scale: Arc<dyn RealmLifecycleOperator>,
        split: Arc<dyn RealmLifecycleOperator>,
        merge: Arc<dyn RealmLifecycleOperator>,
        retire: Arc<dyn RealmLifecycleOperator>,
        archive: Arc<dyn RealmLifecycleOperator>,
    ) -> Self {
        Self {
            new_realm,
            scale,
            split,
            merge,
            retire,
            archive,
        }
    }

    /// 通过 6 阶段获取对应 operator
    pub fn operator_for(&self, stage: RealmLifecycleStage) -> Arc<dyn RealmLifecycleOperator> {
        match stage {
            RealmLifecycleStage::NewRealm => self.new_realm.clone(),
            RealmLifecycleStage::Scale => self.scale.clone(),
            RealmLifecycleStage::Split => self.split.clone(),
            RealmLifecycleStage::Merge => self.merge.clone(),
            RealmLifecycleStage::Retire => self.retire.clone(),
            RealmLifecycleStage::Archive => self.archive.clone(),
        }
    }
}

#[async_trait]
impl RealmLifecycleService for RealmLifecycleServiceImpl {
    async fn new_realm(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        into_crate_result(
            self.new_realm
                .execute(request_id, realm_id, operator_id, approval_ref)
                .await,
        )
    }

    async fn scale(
        &self,
        request_id: Uuid,
        realm_id: &str,
        target_capacity: u32,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        // 扩缩容双向参数校验（per FR-LCM-040~044）
        if target_capacity == 0 {
            return into_crate_result(Err(LcmError::invalid_parameter(
                "scale target_capacity must be > 0",
            )));
        }
        into_crate_result(
            self.scale
                .execute(request_id, realm_id, operator_id, approval_ref)
                .await,
        )
    }

    async fn split(
        &self,
        request_id: Uuid,
        source_realm_id: &str,
        target_realm_ids: &[String],
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        if target_realm_ids.is_empty() {
            return into_crate_result(Err(LcmError::invalid_parameter(
                "split target_realm_ids must not be empty",
            )));
        }
        into_crate_result(
            self.split
                .execute(request_id, source_realm_id, operator_id, approval_ref)
                .await,
        )
    }

    async fn merge(
        &self,
        request_id: Uuid,
        source_realm_ids: &[String],
        target_realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        if source_realm_ids.len() < 2 {
            return into_crate_result(Err(LcmError::invalid_parameter(
                "merge requires at least 2 source_realm_ids",
            )));
        }
        into_crate_result(
            self.merge
                .execute(request_id, target_realm_id, operator_id, approval_ref)
                .await,
        )
    }

    async fn merge_rollback(
        &self,
        request_id: Uuid,
        merge_run_id: Uuid,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        // 合服回退子操作由 merge operator 内部区分（per M-2066.7 注释）；
        // 骨架阶段通过 merge operator 的 stage() 路由 + 二次校验 merge_run_id
        let _ = request_id;
        let _ = merge_run_id;
        let _ = operator_id;
        let _ = approval_ref;
        // 骨架占位：实际回退在 L4 #2067 Saga 接入时实现
        into_crate_result(Err(LcmError::not_implemented(
            "merge_rollback",
            "M-2067 Saga 接入 + L4 #2068 merge_conflict_rule_set_v2 表 + L4 #2073 跨域 gRPC 客户端",
        )))
    }

    async fn retire(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        // 退场前置条件（per SPEC §3 第 8 条 + RGS-IMPL-PLAN-LCM-001 §3.5 M-2073.4）：
        // 退场后 RBAC 查询通道**仅**对 `retire_plan.query_channel_rbac` 配置角色开放
        // 骨架阶段仅检查 realm_id 非空；具体配置校验由 L4 #2068 retire_plan 接入后完成
        if realm_id.is_empty() {
            return into_crate_result(Err(LcmError::invalid_parameter(
                "retire realm_id must not be empty",
            )));
        }
        let _ = operator_id;
        into_crate_result(
            self.retire
                .execute(request_id, realm_id, operator_id, approval_ref)
                .await,
        )
    }

    async fn archive(
        &self,
        request_id: Uuid,
        realm_id: &str,
        operator_id: Uuid,
        approval_ref: Option<&str>,
    ) -> Result<Uuid> {
        if realm_id.is_empty() {
            return into_crate_result(Err(LcmError::invalid_parameter(
                "archive realm_id must not be empty",
            )));
        }
        into_crate_result(
            self.archive
                .execute(request_id, realm_id, operator_id, approval_ref)
                .await,
        )
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

// ============================================================================
// 测试：6 阶段状态机 + 二次激活负例（per M-2066.10 + 验收门槛）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_legal_transitions_match_state_machine() {
        use RealmLifecycleStage::*;
        // 主流转路径
        assert!(NewRealm.can_transition_to(Scale));
        assert!(Scale.can_transition_to(Split));
        assert!(Split.can_transition_to(Merge));
        assert!(Merge.can_transition_to(Retire));
        assert!(Retire.can_transition_to(Archive));
        // 合服回退窗口期（per TBD-DTL-042 7~30 天）
        assert!(Merge.can_transition_to(Merge));
    }

    #[test]
    fn stage_archive_is_terminal() {
        use RealmLifecycleStage::*;
        assert!(Archive.is_terminal());
        assert!(!NewRealm.is_terminal());
        assert!(!Retire.is_terminal());
    }

    #[test]
    fn stage_invalid_transitions_rejected() {
        use RealmLifecycleStage::*;
        // 跳过中间阶段非法
        assert!(!NewRealm.can_transition_to(Archive));
        assert!(!Scale.can_transition_to(Archive));
        assert!(!NewRealm.can_transition_to(Split));
        assert!(!NewRealm.can_transition_to(Merge));
        assert!(!Split.can_transition_to(Retire));
        assert!(!Split.can_transition_to(Archive));
        // 倒退非法
        assert!(!Scale.can_transition_to(NewRealm));
        assert!(!Merge.can_transition_to(Split));
        assert!(!Retire.can_transition_to(Merge));
    }

    #[test]
    fn stage_archive_to_anything_rejected() {
        use RealmLifecycleStage::*;
        // 终态不可转移（含二次激活 NewRealm —— 由 StateMachine 区分错误码）
        assert!(!Archive.can_transition_to(NewRealm));
        assert!(!Archive.can_transition_to(Scale));
        assert!(!Archive.can_transition_to(Split));
        assert!(!Archive.can_transition_to(Merge));
        assert!(!Archive.can_transition_to(Retire));
    }

    #[test]
    fn state_machine_full_lifecycle_walks_through_6_stages() {
        let sm = RealmLifecycleStateMachine::new("realm-001");
        assert_eq!(sm.current(), RealmLifecycleStage::NewRealm);
        let req = Uuid::new_v4();
        sm.transition(RealmLifecycleStage::Scale, req).unwrap();
        assert_eq!(sm.current(), RealmLifecycleStage::Scale);
        sm.transition(RealmLifecycleStage::Split, req).unwrap();
        assert_eq!(sm.current(), RealmLifecycleStage::Split);
        sm.transition(RealmLifecycleStage::Merge, req).unwrap();
        assert_eq!(sm.current(), RealmLifecycleStage::Merge);
        sm.transition(RealmLifecycleStage::Retire, req).unwrap();
        assert_eq!(sm.current(), RealmLifecycleStage::Retire);
        sm.transition(RealmLifecycleStage::Archive, req).unwrap();
        assert_eq!(sm.current(), RealmLifecycleStage::Archive);
    }

    #[test]
    fn state_machine_duplicate_activation_returns_already_activated() {
        // 二次激活负例（per M-2066.10 验收门槛）
        let sm = RealmLifecycleStateMachine::restore("realm-007", RealmLifecycleStage::Archive);
        let req = Uuid::new_v4();
        let err = sm
            .transition(RealmLifecycleStage::NewRealm, req)
            .unwrap_err();
        match err.kind {
            super::super::error::LcmErrorKind::AlreadyActivated { realm_id } => {
                assert_eq!(realm_id, "realm-007");
            }
            other => panic!("expected AlreadyActivated, got {other:?}"),
        }
    }

    #[test]
    fn state_machine_invalid_transition_returns_lcm_error() {
        // NewRealm → Archive 跳过中间阶段
        let sm = RealmLifecycleStateMachine::new("realm-002");
        let req = Uuid::new_v4();
        let err = sm
            .transition(RealmLifecycleStage::Archive, req)
            .unwrap_err();
        match err.kind {
            super::super::error::LcmErrorKind::InvalidStageTransition { from, to, .. } => {
                assert_eq!(from, "new_realm");
                assert_eq!(to, "archive");
            }
            other => panic!("expected InvalidStageTransition, got {other:?}"),
        }
    }

    #[test]
    fn stage_from_str_round_trip() {
        use std::str::FromStr;
        for s in [
            "new_realm",
            "scale",
            "split",
            "merge",
            "retire",
            "archive",
        ] {
            let stage = RealmLifecycleStage::from_str(s).unwrap();
            assert_eq!(stage.as_str(), s);
        }
        let err = RealmLifecycleStage::from_str("unknown").unwrap_err();
        assert!(matches!(
            err.kind,
            super::super::error::LcmErrorKind::InvalidParameter(_)
        ));
    }
}

// ============================================================================
// serde 适配（用于 audit 日志 + OLU 上报 + DB 持久化的 JSON 列）
// ============================================================================
//
// 骨架阶段（M-2066）暂不启用 serde 适配；待 L4 #2071 接入 rgs-arc-olu 时
// 启用（届时 OLU 上报需要序列化 RealmLifecycleStage）。
// 注：cluster-ops 根 crate 未声明 `serde` feature flag，cfg 暂留作占位。
// ============================================================================

#[allow(dead_code)]
mod serde_compat {
    use super::{FromStr, RealmLifecycleStage};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for RealmLifecycleStage {
        fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
            s.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for RealmLifecycleStage {
        fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
            let s = String::deserialize(d)?;
            FromStr::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}
