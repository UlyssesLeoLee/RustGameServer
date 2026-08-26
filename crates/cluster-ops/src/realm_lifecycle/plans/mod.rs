//! 服务器全生命周期管理（LCM）— 6 张 Plan 表 entity + PgRepository 骨架
//!
//! 规范：RGS-SPEC-DTL-042 §2 + §3 第 5 条 + RGS-IMPL-001 §3 既有模式
//! DDL：`crates/cluster-ops/migrations/0020_lcm_tables.sql`
//! M 任务：M-2068.7
//!
//! ## 模块结构
//!
//! - [`realm_lifecycle_run`]：主运行记录（按 created_at 月度范围分区）
//! - [`new_realm_plan`]：新服计划
//! - [`split_plan`]：分服计划
//! - [`merge_conflict_rule_set_v2`]：合服冲突规则集 v2（FR-LCM-062 锁定后不可改）
//! - [`retire_plan`]：退场计划（query_channel_rbac 配置）
//! - [`archive_policy`]：归档策略（N+2 冗余，**不**含删除路径，NFR-SE-010）
//!
//! ## 硬约束（继承自 RGS-SPEC-DTL-042 §3）
//!
//! - **FR-LCM-001**：6 张表全部在 admin_db；本模块不新建独立数据库
//! - **FR-LCM-002**：阶段变更全流程留痕既有 `admin_db.audit_log`，本模块不绕过也不复制
//! - **FR-LCM-062**：`merge_conflict_rule_set_v2` 在 `locked_at` 锁定后不允许运行时修改
//! - **FR-LCM-081**：归档**不**删除数据；`archive_policy` 不含 DELETE 路径
//! - **NFR-SE-010**：GDPR 删除通路在 `admin_db.audit_log` 双层审计
//!
//! ## PgRepository 骨架（per M-2068.7）
//!
//! 本模块提供 6 个 Repository trait + 6 个 PgRepository 骨架（`find_by_id` + `save` 两条核心方法）。
//! 列表 / 删除 / 状态机推进等业务方法在后续 L4 任务（WF-1-2066 / WF-1-2067）补全。

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

pub mod archive_policy;
pub mod merge_conflict_rule_set_v2;
pub mod new_realm_plan;
pub mod realm_lifecycle_run;
pub mod retire_plan;
pub mod split_plan;

pub use archive_policy::{ArchivePolicy, PgArchivePolicyRepository};
pub use merge_conflict_rule_set_v2::{
    MergeConflictRuleSetV2, PgMergeConflictRuleSetV2Repository,
};
pub use new_realm_plan::{NewRealmPlan, NewRealmPlanStatus, PgNewRealmPlanRepository};
pub use realm_lifecycle_run::{
    FeatureSubtype, PgRealmLifecycleRunRepository, RealmLifecycleRun, RunStatus,
};
pub use retire_plan::{PgRetirePlanRepository, RetirePlan, RetirePlanStatus};
pub use split_plan::{PgSplitPlanRepository, SplitPlan, SplitPlanStatus, SplitStrategy};

/// RealmLifecycleRun Repository trait（per M-2068.7）
#[async_trait]
pub trait RealmLifecycleRunRepository: Send + Sync {
    /// 按 ID 查
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RealmLifecycleRun>>;
    /// 按 request_id + operator_id 查（幂等性）
    async fn find_by_request_operator(
        &self,
        request_id: Uuid,
        operator_id: Uuid,
    ) -> Result<Option<RealmLifecycleRun>>;
    /// 写入（INSERT ON CONFLICT）
    async fn save(&self, entity: &RealmLifecycleRun) -> Result<RealmLifecycleRun>;
}

/// NewRealmPlan Repository trait
#[async_trait]
pub trait NewRealmPlanRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewRealmPlan>>;
    async fn find_by_run_id(&self, run_id: Uuid) -> Result<Option<NewRealmPlan>>;
    async fn save(&self, entity: &NewRealmPlan) -> Result<NewRealmPlan>;
}

/// SplitPlan Repository trait
#[async_trait]
pub trait SplitPlanRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SplitPlan>>;
    async fn find_by_run_id(&self, run_id: Uuid) -> Result<Option<SplitPlan>>;
    async fn save(&self, entity: &SplitPlan) -> Result<SplitPlan>;
}

/// MergeConflictRuleSetV2 Repository trait（per FR-LCM-062 锁定后不可改）
#[async_trait]
pub trait MergeConflictRuleSetV2Repository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MergeConflictRuleSetV2>>;
    /// 按 rule_set_version 查（uq_merge_conflict_rule_set_version 唯一）
    async fn find_by_version(
        &self,
        version: i32,
    ) -> Result<Option<MergeConflictRuleSetV2>>;
    /// 写入；**锁定后**（locked_at IS NOT NULL）禁止修改（FR-LCM-062 必须在 save 前做业务校验）
    async fn save(&self, entity: &MergeConflictRuleSetV2) -> Result<MergeConflictRuleSetV2>;
}

/// RetirePlan Repository trait
#[async_trait]
pub trait RetirePlanRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RetirePlan>>;
    async fn find_by_run_id(&self, run_id: Uuid) -> Result<Option<RetirePlan>>;
    async fn save(&self, entity: &RetirePlan) -> Result<RetirePlan>;
}

/// ArchivePolicy Repository trait（per FR-LCM-081 + NFR-SE-010 不含删除）
#[async_trait]
pub trait ArchivePolicyRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ArchivePolicy>>;
    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Option<ArchivePolicy>>;
    /// 写入 / upsert
    async fn save(&self, entity: &ArchivePolicy) -> Result<ArchivePolicy>;
}

/// 公共辅助：把 6 个 PgRepository 绑到同一个 PgPool（admin_db pool，per FR-LCM-001）
pub struct LcmRepositories {
    pub realm_lifecycle_run: PgRealmLifecycleRunRepository,
    pub new_realm_plan: PgNewRealmPlanRepository,
    pub split_plan: PgSplitPlanRepository,
    pub merge_conflict_rule_set_v2: PgMergeConflictRuleSetV2Repository,
    pub retire_plan: PgRetirePlanRepository,
    pub archive_policy: PgArchivePolicyRepository,
}

impl LcmRepositories {
    /// 工厂：从 admin_db PgPool 创建 6 个 Repository
    pub fn new(pool: PgPool) -> Self {
        Self {
            realm_lifecycle_run: PgRealmLifecycleRunRepository::new(pool.clone()),
            new_realm_plan: PgNewRealmPlanRepository::new(pool.clone()),
            split_plan: PgSplitPlanRepository::new(pool.clone()),
            merge_conflict_rule_set_v2: PgMergeConflictRuleSetV2Repository::new(pool.clone()),
            retire_plan: PgRetirePlanRepository::new(pool.clone()),
            archive_policy: PgArchivePolicyRepository::new(pool),
        }
    }
}
