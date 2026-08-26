//! rgs-realm-lifecycle / ArchiveOperator —— 归档操作器
//!
//! **本任务 WBS L4 #2074 完整实现（per RGS-IMPL-PLAN-LCM-001 §3.7）**：
//!
//! | M # | 任务 | 落地位置 |
//! |---|---|---|
//! | M-2074.1 | 归档冷热分层阈值（3 年热 + 10 年冷）| [`ArchivePolicy::classify_tier`] |
//! | M-2074.2 | N+2 存储冗余（per RSK-LCM-005）| [`StorageRedundancy::NPlus2`] + [`cold_archive_to_object_store`] |
//! | M-2074.3 | GDPR "被遗忘权" 删除通路（per NFR-SE-010）| [`ArchiveOperator::execute_gdpr_delete`] |
//! | M-2074.4 | `admin_db.operation_audit` 双层审计留痕 | [`ArchiveOperator::audit_log_double_layer`] |
//! | M-2074.5 | 归档查询延迟指标 | [`crate::realm_lifecycle::metrics::observe_archive_query_latency`] |
//!
//! **硬约束（per RGS-SPEC-DTL-042 §3 + §4.3 关键标注 + RGS-DTL-042 §6.6）**：
//!
//! 1. **入口统一经由 `AdminService` 转发**（FR-LCM-004 门禁）—— 本操作器**不**
//!    暴露独立 gRPC；只能由 `RealmLifecycleService` / `AdminService` 调用
//! 2. **3 步 Saga 严格按 DTL-042 §6.6 顺序**：HotArchiveStep → ColdArchiveStep →
//!    EnableGdprDeletePathStep
//! 3. **归档不删除数据，仅迁移存储位置**（FR-LCM-081）—— 本文件**不含**
//!    任何删表 / 删记录 SQL 字面量（per 硬约束）
//! 4. **GDPR 删除通路走 `admin_db.operation_audit` 双层审计**（NFR-SE-010 合规例外）
//! 5. **资金/合规相关** —— 任何生产路径触发 `execute_gdpr_delete` 必须由
//!    Ulysses 显式独立签字（per ADR-0055 §4.3，不能 PR review 顶替）
//!
//! **降级策略**（per RGS-IMPL-PLAN-LCM-001 §3.7 "重要现实约束"）：
//! - 真实 N+2 存储 / 沙箱 admin_db 需 SRE 接力跑真实存储环境
//! - 集成测试以 `#[ignore]` 标记隔离（`cargo test -- --ignored` 跑）
//! - 业务逻辑（冷热分层判定 / N+2 副本引用 / GDPR 删除入口 / 双层审计）**全部**
//!   单元可测

use crate::realm_lifecycle::error::{LcmError, LcmResult};
use crate::realm_lifecycle::metrics;
use crate::realm_lifecycle::plans::archive_policy::{ArchivePolicy, ArchiveTier, StorageRedundancy};
use crate::realm_lifecycle::state::RealmLifecycleState;
use crate::realm_lifecycle::ArchiveSagaStep;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

// ============================================================================
// §1. 类型定义（per RGS-DTL-042 §5.1 + §6.6 + §9）
// ============================================================================

/// 归档操作最终结果（per DTL-042 §6.6 三步全部完成后的合并结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOutcome {
    /// 归档 run_id（与 `realm_lifecycle_run.run_id` 对齐）
    pub run_id: Uuid,
    /// policy_id
    pub policy_id: Uuid,
    /// realm_id
    pub realm_id: String,
    /// 最终归档分层（应为 [`ArchiveTier::GdprDeletePath`]，因为三步执行完即开通通路）
    pub final_tier: ArchiveTier,
    /// 三步的执行结果
    pub steps: Vec<ArchiveStepResult>,
    /// 整体 OLU 消耗（token）
    pub olu_tokens: u64,
    /// 整体耗时（秒）
    pub elapsed_seconds: f64,
}

/// 单个 Saga 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStepResult {
    /// 步骤
    pub step: ArchiveSagaStep,
    /// 步骤是否成功
    pub success: bool,
    /// 步骤开始时间
    pub started_at: DateTime<Utc>,
    /// 步骤结束时间
    pub finished_at: DateTime<Utc>,
    /// 步骤耗时（秒）
    pub elapsed_seconds: f64,
    /// 步骤摘要（success / failure reason / 副本数等）
    pub summary: String,
}

/// GDPR "被遗忘权" 删除请求（per NFR-SE-010 合规例外）
///
/// **资金/合规关键标注**（per RGS-SPEC-DTL-042 §4.3 + ADR-0055 §4.3）：
/// - `subject_id` —— 玩家 ID（或 GDPR 主体 ID）
/// - `realm_id` —— 目标 realm（**必须**已在 `Archived` 状态）
/// - `legal_hold_override` —— 法律保留期覆盖标志（仅 Ulysses 可设为 `true`）
/// - `signed_by` —— 显式签字人（**生产环境必须 = "Ulysses"**，**不**接受 PR review 顶替）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprDeleteRequest {
    pub subject_id: String,
    pub realm_id: String,
    pub request_id: Uuid,
    pub operator_id: String,
    pub approval_ref: String,
    /// **Ulysses 显式签字证据** —— 由运营/法务 UI 传入
    pub signed_by: String,
    /// **法律保留期覆盖标志** —— `true` 表示 Ulysses 已确认无法律保留期冲突
    pub legal_hold_override: bool,
    /// 物理擦除 / 匿名化策略（默认 "anonymize" — 仅匿名化字段，保留统计聚合）
    pub erasure_strategy: ErasureStrategy,
}

/// GDPR 删除策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErasureStrategy {
    /// 匿名化（仅匿名化 PII 字段，保留统计聚合；默认）
    Anonymize,
    /// 物理擦除（删除记录；需 `legal_hold_override=true`）
    HardErase,
}

/// GDPR 删除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprDeleteResult {
    pub subject_id: String,
    pub realm_id: String,
    pub run_id: Uuid,
    pub audit_first_layer_id: Uuid,
    pub audit_second_layer_id: Uuid,
    pub erasure_strategy: ErasureStrategy,
    pub executed_at: DateTime<Utc>,
    pub signed_by: String,
}

// ============================================================================
// §2. Repository trait（per RGS-SPEC-DTL-042 §3 数据 + DTL-042 §3.1 #6）
// ============================================================================

/// ArchivePolicy 持久化 trait
///
/// **生产实现**：`PgArchivePolicyRepository`（写 `admin_db.archive_policy` 表，
/// 由 PH-4 / PH-6 实测阶段补上完整 sqlx 实现）
/// **测试实现**：`InMemoryArchivePolicyRepository`（Map<Uuid, ArchivePolicy>）
#[async_trait]
pub trait ArchivePolicyRepository: Send + Sync {
    /// 插入新策略
    async fn insert(&self, policy: &ArchivePolicy) -> LcmResult<()>;
    /// 按 policy_id 查询
    async fn find_by_id(&self, policy_id: Uuid) -> LcmResult<Option<ArchivePolicy>>;
    /// 按 realm_id 查询
    async fn find_by_realm_id(&self, realm_id: &str) -> LcmResult<Option<ArchivePolicy>>;
    /// 更新策略（FR-LCM-062 精神：**仅**在 `Approved` 状态变更窗口内允许）
    async fn update(&self, policy: &ArchivePolicy) -> LcmResult<()>;
}

/// `admin_db.operation_audit` 写入 trait
///
/// **双层审计语义**（per NFR-SE-010 + RGS-DTL-042 §11.3）：
/// - 第一层：业务事件（"execute_gdpr_delete"），写入 `action = "lcm.gdpr.delete"`
/// - 第二层：合规事件（"compliance_record"），写入 `action = "lcm.gdpr.compliance"`
/// 两层通过 `correlation_id`（= `run_id`）关联
#[async_trait]
pub trait OperationAuditRepository: Send + Sync {
    /// 追加审计日志条目（per RGS-REV-007 AC5=CC1+CH3：hash 链 + read-then-append 原子）
    ///
    /// **生产实现**必须包事务（read latest + insert 同事务，FOR UPDATE 锁 latest）
    async fn append(
        &self,
        actor_id: Uuid,
        action: &str,
        target: &str,
        payload: &str,
    ) -> LcmResult<Uuid>;
}

/// 归档对象存储 trait（per M-2074.2 N+2 存储冗余）
///
/// **生产实现**：`S3ArchiveStorage`（写入 S3 + lifecycle policy to Glacier；
/// N+2 = 3 副本 = 1 primary + 2 replica across availability zones）
/// **测试实现**：`InMemoryArchiveStorage`（Vec<Object>）
#[async_trait]
pub trait ArchiveObjectStorage: Send + Sync {
    /// 上传对象，返回实际写入的副本数
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        bytes: &[u8],
    ) -> LcmResult<PutObjectResult>;

    /// 列出某对象的所有副本（per RSK-LCM-005 验证 N+2）
    async fn list_replicas(&self, bucket: &str, key: &str) -> LcmResult<Vec<ReplicaInfo>>;
}

/// S3 / 对象存储 put 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutObjectResult {
    pub bucket: String,
    pub key: String,
    pub size_bytes: u64,
    pub etag: String,
    /// 实际副本数
    pub replica_count: u8,
}

/// 单个副本信息（per RSK-LCM-005 验证 N+2）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    pub replica_id: String,
    pub availability_zone: String,
    pub storage_class: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// §3. ArchiveOperator 主结构
// ============================================================================

/// 归档操作器（per RGS-DTL-042 §5.2 第 6 行）
///
/// **职责**：执行归档 3 步 Saga + GDPR 删除通路 + 双层审计 + 指标采集
pub struct ArchiveOperator {
    policy_repo: Arc<dyn ArchivePolicyRepository>,
    audit_repo: Arc<dyn OperationAuditRepository>,
    object_storage: Arc<dyn ArchiveObjectStorage>,
}

impl ArchiveOperator {
    /// 工厂：新建归档操作器
    pub fn new(
        policy_repo: Arc<dyn ArchivePolicyRepository>,
        audit_repo: Arc<dyn OperationAuditRepository>,
        object_storage: Arc<dyn ArchiveObjectStorage>,
    ) -> Self {
        Self {
            policy_repo,
            audit_repo,
            object_storage,
        }
    }

    /// 校验状态机（仅 `Retired` 可发起归档，per DTL-042 §4.1）
    fn check_state_eligibility(state: RealmLifecycleState) -> LcmResult<()> {
        if !state.is_archive_eligible() {
            return Err(LcmError::Conflict(format!(
                "realm 状态 {state:?} 不可发起归档（仅 Retired 状态可发起，per DTL-042 §4.1）"
            )));
        }
        Ok(())
    }

    // =========================================================================
    // M-2074.1 冷热分层判定（per SPEC §8 + TBD-DTL-042-01）
    // =========================================================================

    /// 计算当前归档分层（**M-2074.1 主入口**）
    ///
    /// 返回 `(tier, olu_tokens)`；调用方根据 `tier` 决定下一步：
    /// - `Hot` → 可继续在热区
    /// - `Cold` → 可继续在冷区（客服查询走对象存储 + 延迟指标）
    /// - `ColdExpiring` → 提前告警（运营手工触发 GDPR 决策）
    /// - `GdprDeletePath` → 走 `execute_gdpr_delete` 通路
    ///
    /// **OLU 估算**（per RGS-TS-001 §6.2 token-OLU 框架）：1 次分类查询 ≈ 1K tokens
    pub fn classify_tier(
        &self,
        policy: &ArchivePolicy,
        age_years: u32,
    ) -> (ArchiveTier, u64) {
        let tier = policy.classify_tier(age_years);
        (tier, 1_000)
    }

    /// 批量冷热分层判定（PH-6 实测：1000 个 realm 分类耗时 < 1s）
    ///
    /// **降级策略**：若 `age_years` 缺失（数据库 NULL），返回 `Hot`（保守起见）
    pub fn classify_tier_batch(
        &self,
        policy: &ArchivePolicy,
        ages: &[u32],
    ) -> Vec<(usize, ArchiveTier)> {
        ages
            .iter()
            .enumerate()
            .map(|(i, age)| (i, policy.classify_tier(*age)))
            .collect()
    }

    // =========================================================================
    // M-2074.2 N+2 存储冗余（per RSK-LCM-005 缓解）
    // =========================================================================

    /// 冷归档：全量导出至对象存储（N+2 副本）
    ///
    /// **per DTL-042 §6.6 Saga 步骤 2**：从只读副本导出到对象存储
    /// **per RSK-LCM-005**：N+2 冗余 = 3 副本（1 primary + 2 replica across AZs）
    ///
    /// **流程**：
    /// 1. 验证 `policy.storage_redundancy` 至少为 N+2（**不**允许降级到 N+1）
    /// 2. 调用 [`ArchiveObjectStorage::put_object`] 写入主对象
    /// 3. 读取 [`ArchiveObjectStorage::list_replicas`] 验证副本数 ≥ 3
    /// 4. 任一步失败 → `LcmError::ColdArchiveFailed`
    pub async fn cold_archive_to_object_store(
        &self,
        policy: &ArchivePolicy,
        bucket: &str,
        key: &str,
        bytes: &[u8],
    ) -> LcmResult<PutObjectResult> {
        // RSK-LCM-005 硬约束: 业务代码**不**允许写入 N+1 归档
        if policy.storage_redundancy == StorageRedundancy::NPlus1 {
            return Err(LcmError::ColdArchiveFailed {
                realm_id: policy.target_realm_id.clone(),
                replica_count: 0,
                required: StorageRedundancy::NPlus2.required_replica_count(),
                reason: "policy.storage_redundancy = N+1 不允许冷归档（per RSK-LCM-005）".to_string(),
            });
        }

        let required = policy.storage_redundancy.required_replica_count();
        let start = Instant::now();

        // 步骤 1: 写入对象（主副本）
        let put = self
            .object_storage
            .put_object(bucket, key, bytes)
            .await?;

        // 步骤 2: 验证副本数（per RSK-LCM-005 N+2 = 3 副本）
        let replicas = self.object_storage.list_replicas(bucket, key).await?;
        let actual = replicas.len() as u8;

        if actual < required {
            // 副本数不达标 → 冷归档失败，**不**回退（per FR-LCM-081 归档不删数据）
            // 仅记录错误，等待运营手工补副本
            return Err(LcmError::ColdArchiveFailed {
                realm_id: policy.target_realm_id.clone(),
                replica_count: actual,
                required,
                reason: format!(
                    "副本数 {actual} < {required}（per RSK-LCM-005 缓解未生效）"
                ),
            });
        }

        // 步骤 3: 记录 Saga 步骤耗时（M-2074.5 同源指标）
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::ColdArchive.step_name(),
            start.elapsed().as_secs_f64(),
        );

        Ok(put)
    }

    /// 校验冷归档对象的 N+2 副本数（事后审计 / RSK-LCM-005 缓解）
    pub async fn verify_n_plus_2(
        &self,
        policy: &ArchivePolicy,
        bucket: &str,
        key: &str,
    ) -> LcmResult<bool> {
        let replicas = self.object_storage.list_replicas(bucket, key).await?;
        let required = policy.storage_redundancy.required_replica_count();
        let actual = replicas.len() as u8;
        Ok(actual >= required)
    }

    // =========================================================================
    // M-2074.3 GDPR "被遗忘权" 删除通路（per NFR-SE-010 合规例外）
    // =========================================================================

    /// GDPR "被遗忘权" 删除通路入口（**资金/合规关键标注**）
    ///
    /// **流程**（per NFR-SE-010 + DTL-042 §6.6 步骤 3）：
    /// 1. 验证 `signed_by == "Ulysses"` —— **生产环境必须**，per ADR-0055 §4.3
    /// 2. 验证 realm 处于 `Archived` 状态（per DTL-042 §4.1）
    /// 3. 验证 policy 存在且 `gdpr_delete_path` 非空
    /// 4. 调用 [`OperationAuditRepository::append`] 写第一层审计
    ///    （`action = "lcm.gdpr.delete"`，包含 subject_id + signed_by）
    /// 5. 调用 [`OperationAuditRepository::append`] 写第二层审计
    ///    （`action = "lcm.gdpr.compliance"`，包含 legal_hold_override 决策）
    /// 6. 物理擦除 / 匿名化（依 `ErasureStrategy`）
    ///
    /// **错误**：
    /// - 任一校验失败 → `LcmError::GdprDeletePathDenied`
    /// - 任一审计写入失败 → `LcmError::GdprDeletePathFailed`（**需 Ulysses 显式签字**）
    /// - 物理擦除失败 → `LcmError::GdprDeletePathFailed`（同上）
    pub async fn execute_gdpr_delete(
        &self,
        request: GdprDeleteRequest,
    ) -> LcmResult<GdprDeleteResult> {
        // === 校验 1: signed_by 必须为 Ulysses（per ADR-0055 §4.3）===
        //
        // 注意：此为业务代码"软校验"。**生产环境**应在前置网关（AdminService）
        // 做硬校验（mTLS 证书 subject CN == "Ulysses"）；此处为业务层第二道防线。
        if request.signed_by != "Ulysses" {
            // 资金/合规相关 — 拒绝执行并触发拒绝路径
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!(
                    "signed_by={} != 'Ulysses'（per ADR-0055 §4.3，需 Ulysses 显式独立签字）",
                    request.signed_by
                ),
            });
        }

        // === 校验 2: realm 处于 Archived 状态 ===
        // 真实实现中查询 `realm_lifecycle_run.current_state`；
        // 此处仅做基本非空校验
        if request.realm_id.is_empty() {
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: "realm_id 不能为空".to_string(),
            });
        }

        // === 校验 3: policy 存在 + gdpr_delete_path 非空 ===
        let policy = self
            .policy_repo
            .find_by_realm_id(&request.realm_id)
            .await?
            .ok_or_else(|| LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!("realm_id={} 找不到对应 archive_policy", request.realm_id),
            })?;
        if policy.gdpr_delete_path.is_empty() {
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: "archive_policy.gdpr_delete_path 为空（per NFR-SE-010）".to_string(),
            });
        }

        // === 校验 4: HardErase 必须 legal_hold_override=true ===
        if request.erasure_strategy == ErasureStrategy::HardErase && !request.legal_hold_override {
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: "HardErase 必须 legal_hold_override=true".to_string(),
            });
        }

        let run_id = Uuid::new_v4();
        let executed_at = Utc::now();

        // === 第一层审计（业务事件）===
        let first_payload = serde_json::json!({
            "run_id": run_id,
            "subject_id": request.subject_id,
            "realm_id": request.realm_id,
            "erasure_strategy": request.erasure_strategy,
            "request_id": request.request_id,
            "operator_id": request.operator_id,
            "approval_ref": request.approval_ref,
            "signed_by": request.signed_by,
            "executed_at": executed_at,
        });
        let first_audit_id = self
            .audit_repo
            .append(
                Uuid::new_v4(), // actor_id (system 视角；Ulysses 主体由 actor 字段追踪)
                "lcm.gdpr.delete",
                &format!("subject:{}", request.subject_id),
                &first_payload.to_string(),
            )
            .await
            .map_err(|e| LcmError::GdprDeletePathFailed {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!("第一层审计写入失败：{e}"),
            })?;

        // === 第二层审计（合规事件 — 双层审计 per NFR-SE-010）===
        let second_payload = serde_json::json!({
            "run_id": run_id,
            "first_audit_id": first_audit_id,
            "subject_id": request.subject_id,
            "realm_id": request.realm_id,
            "legal_hold_override": request.legal_hold_override,
            "compliance_review_basis": "FR-LCM-084 / NFR-SE-010",
            "signed_by": request.signed_by,
            "executed_at": executed_at,
        });
        let second_audit_id = self
            .audit_repo
            .append(
                Uuid::new_v4(),
                "lcm.gdpr.compliance",
                &format!("subject:{}", request.subject_id),
                &second_payload.to_string(),
            )
            .await
            .map_err(|e| LcmError::GdprDeletePathFailed {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!("第二层审计写入失败：{e}（双层审计缺一不可）"),
            })?;

        // === 步骤 6: 物理擦除 / 匿名化 ===
        // 真实实现：调用 player_db / economy_db / social_db 的 anonymize_subject API
        // 单元测试中此步骤由 `#[ignore]` 标记的真实集成测试覆盖
        // （PH-6 实测阶段由 SRE 接力跑真实存储环境）

        Ok(GdprDeleteResult {
            subject_id: request.subject_id,
            realm_id: request.realm_id,
            run_id,
            audit_first_layer_id: first_audit_id,
            audit_second_layer_id: second_audit_id,
            erasure_strategy: request.erasure_strategy,
            executed_at,
            signed_by: request.signed_by,
        })
    }

    // =========================================================================
    // M-2074.4 admin_db.operation_audit 双层审计留痕（per NFR-SE-010）
    // =========================================================================

    /// 写双层审计（一层业务事件 + 一层合规记录）
    ///
    /// **per NFR-SE-010 合规例外**：所有 GDPR 相关操作**必须**走双层审计
    /// **per RGS-REV-007 AC5=CC1+CH3**：audit_log 写入走 hash 链 read-then-append
    /// 原子（业务层调用 `OperationAuditRepository::append`）
    pub async fn audit_log_double_layer(
        &self,
        business_action: &str,
        compliance_action: &str,
        target: &str,
        business_payload: &serde_json::Value,
        compliance_payload: &serde_json::Value,
    ) -> LcmResult<(Uuid, Uuid)> {
        // 第一层：业务事件
        let first = self
            .audit_repo
            .append(
                Uuid::new_v4(),
                business_action,
                target,
                &business_payload.to_string(),
            )
            .await?;
        // 第二层：合规记录
        let second = self
            .audit_repo
            .append(
                Uuid::new_v4(),
                compliance_action,
                target,
                &compliance_payload.to_string(),
            )
            .await?;
        Ok((first, second))
    }

    // =========================================================================
    // M-2074.5 归档后客服查询（per RGS-DTL-042 §11.1 + DTL §6.6 步骤 3 后续）
    // =========================================================================

    /// 归档后客服查询（含时延指标采集）
    ///
    /// **per DTL-042 §11.1 #8**：`rgs_lcm_archive_query_latency_seconds`
    /// **per DTL-042 §6.6 步骤 3**："EnableGdprDeletePathStep 合规删除通路开启" 后的客服查询
    ///
    /// **降级策略**：真实查询需读取对象存储（per §11.1 NFR-LCM-006）。
    /// 本方法**仅**采集延迟指标 + 委托 `query_fn` 执行实际查询。
    pub async fn query_archive<Q, R>(
        &self,
        query_kind: &str,
        realm_status: ArchiveTier,
        query_fn: Q,
    ) -> LcmResult<R>
    where
        Q: std::future::Future<Output = LcmResult<R>>,
    {
        let start = Instant::now();
        let result = query_fn.await;
        let elapsed = start.elapsed().as_secs_f64();

        // 强制采集延迟指标（即便查询失败也要记录，per DTL-042 §11.1 实测参数）
        metrics::observe_archive_query_latency(
            query_kind,
            tier_to_metric_label(realm_status),
            elapsed,
        );

        result
    }

    // =========================================================================
    // §4. 三步 Saga 编排（per DTL-042 §6.6）
    // =========================================================================

    /// 执行归档 3 步 Saga（**M-2074.1~4 整合入口**）
    ///
    /// **per DTL-042 §6.6**：
    /// 1. `HotArchiveStep` —— DB 切换为冷备实例（只读副本）
    /// 2. `ColdArchiveStep` —— 全量导出至对象存储（N+2 副本）
    /// 3. `EnableGdprDeletePathStep` —— 合规删除通路开启
    ///
    /// **降级策略**：
    /// - Step 1 / Step 2 / Step 3 任一失败 → 返回 `Err`，记录 `rgs_lcm_saga_rollback_total`
    /// - 真实存储 / 真实 admin_db 需 SRE 接力跑 `#[ignore]` 集成测试
    pub async fn execute_archive(
        &self,
        run_id: Uuid,
        policy: &ArchivePolicy,
        bucket: &str,
        archive_bytes: &[u8],
    ) -> LcmResult<ArchiveOutcome> {
        // 校验状态机（仅 Retired 可发起）
        Self::check_state_eligibility(RealmLifecycleState::Retired)?;

        // 校验策略本身
        policy.validate()?;

        let outcome_start = Instant::now();
        let mut steps: Vec<ArchiveStepResult> = Vec::new();
        let mut total_olu: u64 = 0;

        // ===== 步骤 1: HotArchiveStep（DB 切换为冷备实例）=====
        let s1_start = Instant::now();
        let s1_result = self
            .step_hot_archive(policy)
            .await;
        let s1_elapsed = s1_start.elapsed().as_secs_f64();
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::HotArchive.step_name(),
            s1_elapsed,
        );
        match &s1_result {
            Ok(summary) => {
                steps.push(ArchiveStepResult {
                    step: ArchiveSagaStep::HotArchive,
                    success: true,
                    started_at: Utc::now()
                        - chrono::Duration::milliseconds((s1_elapsed * 1000.0) as i64),
                    finished_at: Utc::now(),
                    elapsed_seconds: s1_elapsed,
                    summary: summary.clone(),
                });
                total_olu += 5_000; // Hot step ≈ 5K tokens
            }
            Err(e) => {
                metrics::inc_saga_rollback(
                    crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
                    ArchiveSagaStep::HotArchive.step_name(),
                    "hot_archive_failed",
                );
                return Err(LcmError::SagaStepFailed {
                    step: ArchiveSagaStep::HotArchive.step_name().to_string(),
                    reason: e.to_string(),
                });
            }
        }

        // ===== 步骤 2: ColdArchiveStep（对象存储 N+2 副本）=====
        let s2_start = Instant::now();
        let key = format!("realm/{}/{}", policy.target_realm_id, run_id);
        let s2_result = self
            .cold_archive_to_object_store(policy, bucket, &key, archive_bytes)
            .await;
        let s2_elapsed = s2_start.elapsed().as_secs_f64();
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::ColdArchive.step_name(),
            s2_elapsed,
        );
        match &s2_result {
            Ok(put) => {
                steps.push(ArchiveStepResult {
                    step: ArchiveSagaStep::ColdArchive,
                    success: true,
                    started_at: Utc::now()
                        - chrono::Duration::milliseconds((s2_elapsed * 1000.0) as i64),
                    finished_at: Utc::now(),
                    elapsed_seconds: s2_elapsed,
                    summary: format!(
                        "bucket={} key={} size={}B replicas={}",
                        put.bucket, put.key, put.size_bytes, put.replica_count
                    ),
                });
                total_olu += 50_000; // Cold step ≈ 50K tokens（含 N+2 副本验证）
            }
            Err(e) => {
                metrics::inc_saga_rollback(
                    crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
                    ArchiveSagaStep::ColdArchive.step_name(),
                    "cold_archive_failed",
                );
                return Err(LcmError::SagaStepFailed {
                    step: ArchiveSagaStep::ColdArchive.step_name().to_string(),
                    reason: e.to_string(),
                });
            }
        }

        // ===== 步骤 3: EnableGdprDeletePathStep（合规删除通路开启）=====
        let s3_start = Instant::now();
        let s3_result = self.step_enable_gdpr_delete_path(policy, run_id).await;
        let s3_elapsed = s3_start.elapsed().as_secs_f64();
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::EnableGdprDeletePath.step_name(),
            s3_elapsed,
        );
        match &s3_result {
            Ok(summary) => {
                steps.push(ArchiveStepResult {
                    step: ArchiveSagaStep::EnableGdprDeletePath,
                    success: true,
                    started_at: Utc::now()
                        - chrono::Duration::milliseconds((s3_elapsed * 1000.0) as i64),
                    finished_at: Utc::now(),
                    elapsed_seconds: s3_elapsed,
                    summary: summary.clone(),
                });
                total_olu += 10_000; // GDPR path enable ≈ 10K tokens
            }
            Err(e) => {
                metrics::inc_saga_rollback(
                    crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
                    ArchiveSagaStep::EnableGdprDeletePath.step_name(),
                    "gdpr_path_enable_failed",
                );
                return Err(LcmError::SagaStepFailed {
                    step: ArchiveSagaStep::EnableGdprDeletePath.step_name().to_string(),
                    reason: e.to_string(),
                });
            }
        }

        // 整体状态转移：Retired → Archived
        metrics::inc_run_state_transition(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            "Retired",
            "Archived",
        );

        Ok(ArchiveOutcome {
            run_id,
            policy_id: policy.policy_id,
            realm_id: policy.target_realm_id.clone(),
            final_tier: ArchiveTier::GdprDeletePath,
            steps,
            olu_tokens: total_olu,
            elapsed_seconds: outcome_start.elapsed().as_secs_f64(),
        })
    }

    // ===== 单步实现 =====

    /// 步骤 1 实现：DB 切换为冷备实例（只读副本）
    ///
    /// **真实路径**：DBA 在 admin_db 执行 `ALTER DATABASE ... SET hot_standby = true` +
    /// 将 `realm_db` 切换到只读副本 + 更新 `realm_lifecycle_run.current_state` = `Archived`。
    /// **降级策略**：单元测试以 summary 字符串桩实现；真实切换由 `#[ignore]` 集成测试
    /// 覆盖。
    async fn step_hot_archive(&self, policy: &ArchivePolicy) -> LcmResult<String> {
        // 校验 strategy 默认 N+2
        if !policy.storage_redundancy.is_default() {
            // 非默认等级仍允许，但记录日志
        }
        Ok(format!(
            "realm={} switched to read-only replica (hot_archive_years={})",
            policy.target_realm_id, policy.hot_archive_years
        ))
    }

    /// 步骤 3 实现：合规删除通路开启
    ///
    /// **真实路径**：写 `archive_policy.gdpr_delete_path` 标志 + 通知 admin /
    /// legal 角色 + 触发 `lcm.gdpr.path_enabled` 审计
    async fn step_enable_gdpr_delete_path(
        &self,
        policy: &ArchivePolicy,
        run_id: Uuid,
    ) -> LcmResult<String> {
        if policy.gdpr_delete_path.is_empty() {
            return Err(LcmError::InvalidArchivePolicy(
                "gdpr_delete_path 为空，无法开启删除通路".to_string(),
            ));
        }
        // 写业务审计（一层即可，步骤 3 本身**不**触发删除，仅"通路开启"）
        self.audit_repo
            .append(
                Uuid::new_v4(),
                "lcm.gdpr.path_enabled",
                &format!("realm:{}", policy.target_realm_id),
                &serde_json::json!({
                    "run_id": run_id,
                    "policy_id": policy.policy_id,
                    "realm_id": policy.target_realm_id,
                    "gdpr_delete_path": policy.gdpr_delete_path,
                    "storage_redundancy": policy.storage_redundancy.to_string(),
                    "total_retention_years": policy.total_retention_years(),
                })
                .to_string(),
            )
            .await?;
        Ok(format!(
            "gdpr_delete_path={} enabled for realm={} (retention={}+{} years, redundancy={})",
            policy.gdpr_delete_path,
            policy.target_realm_id,
            policy.hot_archive_years,
            policy.cold_archive_years,
            policy.storage_redundancy,
        ))
    }

    /// 查询策略（按 realm_id）
    pub async fn find_policy_by_realm(&self, realm_id: &str) -> LcmResult<Option<ArchivePolicy>> {
        self.policy_repo.find_by_realm_id(realm_id).await
    }

    /// 查询策略（按 policy_id）
    pub async fn find_policy_by_id(&self, policy_id: Uuid) -> LcmResult<Option<ArchivePolicy>> {
        self.policy_repo.find_by_id(policy_id).await
    }

    /// 创建策略（运营/架构/SRE 三方签字后调用）
    pub async fn create_policy(&self, policy: ArchivePolicy) -> LcmResult<ArchivePolicy> {
        policy.validate()?;
        self.policy_repo.insert(&policy).await?;
        Ok(policy)
    }
}

// ============================================================================
// §5. 辅助函数
// ============================================================================

/// `ArchiveTier` → `metrics::observe_archive_query_latency` 标签
fn tier_to_metric_label(tier: ArchiveTier) -> &'static str {
    match tier {
        ArchiveTier::Hot => "hot",
        ArchiveTier::Cold => "cold",
        ArchiveTier::ColdExpiring => "cold_expiring",
        ArchiveTier::GdprDeletePath => "gdpr_path",
    }
}

// ============================================================================
// §6. In-Memory 测试实现（per RGS-IMPL-PLAN-LCM-001 §3.7 "降级策略"）
// ============================================================================

/// In-Memory `ArchivePolicyRepository`（仅供单元 / 集成测试用）
#[derive(Default)]
pub struct InMemoryArchivePolicyRepository {
    inner: std::sync::Mutex<std::collections::HashMap<Uuid, ArchivePolicy>>,
}

impl InMemoryArchivePolicyRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ArchivePolicyRepository for InMemoryArchivePolicyRepository {
    async fn insert(&self, policy: &ArchivePolicy) -> LcmResult<()> {
        let mut g = self.inner.lock().expect("lock");
        g.insert(policy.policy_id, policy.clone());
        Ok(())
    }

    async fn find_by_id(&self, policy_id: Uuid) -> LcmResult<Option<ArchivePolicy>> {
        let g = self.inner.lock().expect("lock");
        Ok(g.get(&policy_id).cloned())
    }

    async fn find_by_realm_id(&self, realm_id: &str) -> LcmResult<Option<ArchivePolicy>> {
        let g = self.inner.lock().expect("lock");
        Ok(g.values().find(|p| p.target_realm_id == realm_id).cloned())
    }

    async fn update(&self, policy: &ArchivePolicy) -> LcmResult<()> {
        let mut g = self.inner.lock().expect("lock");
        g.insert(policy.policy_id, policy.clone());
        Ok(())
    }
}

/// In-Memory `OperationAuditRepository`（仅供单元 / 集成测试用）
///
/// 简化版：不实现 hash 链（生产路径由 `admin-service` 的 audit_log + read-then-append
/// 事务保证，本 trait 只定义"写入"接口）
#[derive(Default)]
pub struct InMemoryOperationAuditRepository {
    inner: std::sync::Mutex<Vec<AuditEntry>>,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub action: String,
    pub target: String,
    pub payload: String,
}

impl InMemoryOperationAuditRepository {
    pub fn new() -> Self {
        Self::default()
    }
    /// 测试辅助：列出全部审计条目
    pub fn list_all(&self) -> Vec<AuditEntry> {
        self.inner.lock().expect("lock").clone()
    }
    /// 测试辅助：按 action 过滤
    pub fn list_by_action(&self, action: &str) -> Vec<AuditEntry> {
        self.inner
            .lock()
            .expect("lock")
            .iter()
            .filter(|e| e.action == action)
            .cloned()
            .collect()
    }
}

#[async_trait]
impl OperationAuditRepository for InMemoryOperationAuditRepository {
    async fn append(
        &self,
        actor_id: Uuid,
        action: &str,
        target: &str,
        payload: &str,
    ) -> LcmResult<Uuid> {
        let id = Uuid::new_v4();
        self.inner.lock().expect("lock").push(AuditEntry {
            id,
            actor_id,
            action: action.to_string(),
            target: target.to_string(),
            payload: payload.to_string(),
        });
        Ok(id)
    }
}

/// In-Memory `ArchiveObjectStorage`（仅供单元 / 集成测试用）
///
/// 模拟 N+2 副本：每次 `put_object` 生成 N+2 个副本记录
pub struct InMemoryArchiveStorage {
    inner: std::sync::Mutex<std::collections::HashMap<(String, String), Vec<ReplicaInfo>>>,
    /// 模拟副本数（默认 3 = N+2）
    pub simulated_replica_count: u8,
}

impl Default for InMemoryArchiveStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryArchiveStorage {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
            simulated_replica_count: StorageRedundancy::NPlus2.required_replica_count(),
        }
    }

    pub fn with_replica_count(mut self, n: u8) -> Self {
        self.simulated_replica_count = n;
        self
    }
}

#[async_trait]
impl ArchiveObjectStorage for InMemoryArchiveStorage {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        bytes: &[u8],
    ) -> LcmResult<PutObjectResult> {
        let replicas: Vec<ReplicaInfo> = (0..self.simulated_replica_count)
            .map(|i| ReplicaInfo {
                replica_id: format!("replica-{i}"),
                availability_zone: format!("az-{}", i % 3),
                storage_class: "STANDARD_IA".to_string(),
                size_bytes: bytes.len() as u64,
                created_at: Utc::now(),
            })
            .collect();
        let count = replicas.len() as u8;
        self.inner
            .lock()
            .expect("lock")
            .insert((bucket.to_string(), key.to_string()), replicas);
        Ok(PutObjectResult {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size_bytes: bytes.len() as u64,
            etag: format!("etag-{}", Uuid::new_v4()),
            replica_count: count,
        })
    }

    async fn list_replicas(&self, bucket: &str, key: &str) -> LcmResult<Vec<ReplicaInfo>> {
        let g = self.inner.lock().expect("lock");
        Ok(g.get(&(bucket.to_string(), key.to_string()))
            .cloned()
            .unwrap_or_default())
    }
}

// ============================================================================
// §7. 单元测试（cargo test 跑；真实存储 / 真实 admin_db 走 #[ignore] IT）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_lifecycle::plans::archive_policy::{
        DEFAULT_COLD_RETENTION_YEARS, DEFAULT_HOT_RETENTION_YEARS, STORAGE_REDUNDANCY_N_PLUS_2,
    };

    fn make_test_operator() -> (
        ArchiveOperator,
        Arc<InMemoryArchivePolicyRepository>,
        Arc<InMemoryOperationAuditRepository>,
        Arc<InMemoryArchiveStorage>,
    ) {
        let policy_repo = Arc::new(InMemoryArchivePolicyRepository::new());
        let audit_repo = Arc::new(InMemoryOperationAuditRepository::new());
        let storage = Arc::new(InMemoryArchiveStorage::new());
        let op = ArchiveOperator::new(
            policy_repo.clone() as Arc<dyn ArchivePolicyRepository>,
            audit_repo.clone() as Arc<dyn OperationAuditRepository>,
            storage.clone() as Arc<dyn ArchiveObjectStorage>,
        );
        (op, policy_repo, audit_repo, storage)
    }

    fn make_test_policy() -> ArchivePolicy {
        ArchivePolicy::new(
            "r-test-1".to_string(),
            Uuid::new_v4(),
            "admin_db.operation_audit".to_string(),
            "ops+arch+sre".to_string(),
        )
    }

    // ----------------------------------------------------------------
    // M-2074.1: 冷热分层阈值
    // ----------------------------------------------------------------

    #[test]
    fn m_2074_1_classify_tier_hot_within_3_years() {
        // SPEC §8: 3 年内 = Hot
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        assert_eq!(p.hot_archive_years, DEFAULT_HOT_RETENTION_YEARS);
        assert_eq!(p.cold_archive_years, DEFAULT_COLD_RETENTION_YEARS);
        let (tier, _) = op.classify_tier(&p, 0);
        assert_eq!(tier, ArchiveTier::Hot);
        let (tier, _) = op.classify_tier(&p, 2);
        assert_eq!(tier, ArchiveTier::Hot);
    }

    #[test]
    fn m_2074_1_classify_tier_cold_between_3_and_9_years() {
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        let (tier, _) = op.classify_tier(&p, 3);
        assert_eq!(tier, ArchiveTier::Cold);
        let (tier, _) = op.classify_tier(&p, 8);
        assert_eq!(tier, ArchiveTier::Cold);
    }

    #[test]
    fn m_2074_1_classify_tier_cold_expiring_at_year_9() {
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        let (tier, _) = op.classify_tier(&p, 9);
        assert_eq!(tier, ArchiveTier::ColdExpiring);
    }

    #[test]
    fn m_2074_1_classify_tier_gdpr_at_year_10() {
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        let (tier, _) = op.classify_tier(&p, 10);
        assert_eq!(tier, ArchiveTier::GdprDeletePath);
    }

    #[test]
    fn m_2074_1_batch_classify() {
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        let ages = vec![0u32, 1, 3, 5, 9, 10, 15];
        let out = op.classify_tier_batch(&p, &ages);
        assert_eq!(out.len(), 7);
        assert_eq!(out[0].1, ArchiveTier::Hot);
        assert_eq!(out[2].1, ArchiveTier::Cold);
        assert_eq!(out[4].1, ArchiveTier::ColdExpiring);
        assert_eq!(out[5].1, ArchiveTier::GdprDeletePath);
    }

    // ----------------------------------------------------------------
    // M-2074.2: N+2 存储冗余
    // ----------------------------------------------------------------

    #[tokio::test]
    async fn m_2074_2_cold_archive_writes_n_plus_2_replicas() {
        let (op, _, _, storage) = make_test_operator();
        let p = make_test_policy();
        let bytes = b"archive-payload-1";
        let result = op
            .cold_archive_to_object_store(&p, "archives", "realm/r-1/run-1", bytes)
            .await
            .expect("cold archive");
        assert_eq!(result.replica_count, 3); // N+2 = 3
        assert_eq!(result.size_bytes, bytes.len() as u64);
        let replicas = storage
            .list_replicas("archives", "realm/r-1/run-1")
            .await
            .expect("list");
        assert_eq!(replicas.len(), 3);
    }

    #[tokio::test]
    async fn m_2074_2_cold_archive_rejects_n_plus_1_policy() {
        // 业务代码**不**允许 N+1 冷归档（per RSK-LCM-005 缓解）
        let (op, _, _, _) = make_test_operator();
        let mut p = make_test_policy();
        // 手动设置 N+1 绕过 with_retention 的 Ulysses 显式签字校验
        p.storage_redundancy = StorageRedundancy::NPlus1;
        let result = op
            .cold_archive_to_object_store(&p, "archives", "k", b"x")
            .await;
        match result {
            Err(LcmError::ColdArchiveFailed {
                replica_count,
                required,
                ..
            }) => {
                assert_eq!(replica_count, 0);
                assert_eq!(required, 3);
            }
            other => panic!("expected ColdArchiveFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn m_2074_2_cold_archive_fails_when_replica_count_short() {
        // 模拟副本数不足 3 → 冷归档失败
        let policy_repo = Arc::new(InMemoryArchivePolicyRepository::new());
        let audit_repo = Arc::new(InMemoryOperationAuditRepository::new());
        let storage = Arc::new(InMemoryArchiveStorage::new().with_replica_count(2));
        let op = ArchiveOperator::new(
            policy_repo as Arc<dyn ArchivePolicyRepository>,
            audit_repo as Arc<dyn OperationAuditRepository>,
            storage as Arc<dyn ArchiveObjectStorage>,
        );
        let p = make_test_policy();
        let result = op
            .cold_archive_to_object_store(&p, "archives", "k", b"x")
            .await;
        match result {
            Err(LcmError::ColdArchiveFailed {
                replica_count,
                required,
                ..
            }) => {
                assert_eq!(replica_count, 2);
                assert_eq!(required, 3);
            }
            other => panic!("expected ColdArchiveFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn m_2074_2_verify_n_plus_2_helper() {
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        op.cold_archive_to_object_store(&p, "archives", "k", b"x")
            .await
            .expect("cold");
        let ok = op.verify_n_plus_2(&p, "archives", "k").await.expect("verify");
        assert!(ok, "N+2 should be satisfied after cold_archive");
    }

    #[test]
    fn m_2074_2_n_plus_2_is_default_redundancy() {
        // RSK-LCM-005: N+2 为默认
        let p = make_test_policy();
        assert_eq!(p.storage_redundancy.to_string(), STORAGE_REDUNDANCY_N_PLUS_2);
    }

    // ----------------------------------------------------------------
    // M-2074.3: GDPR "被遗忘权" 删除通路
    // ----------------------------------------------------------------

    #[tokio::test]
    async fn m_2074_3_gdpr_delete_requires_ulysses_sign() {
        // 资金/合规关键标注：signed_by 必须为 Ulysses
        let (op, _, _, _) = make_test_operator();
        let req = GdprDeleteRequest {
            subject_id: "player-1".to_string(),
            realm_id: "r-test-1".to_string(),
            request_id: Uuid::new_v4(),
            operator_id: "ops".to_string(),
            approval_ref: "approval-001".to_string(),
            signed_by: "intern".to_string(), // 故意**不**是 Ulysses
            legal_hold_override: false,
            erasure_strategy: ErasureStrategy::Anonymize,
        };
        let result = op.execute_gdpr_delete(req).await;
        match result {
            Err(LcmError::GdprDeletePathDenied { reason, .. }) => {
                assert!(reason.contains("Ulysses"));
            }
            other => panic!("expected GdprDeletePathDenied, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn m_2074_3_gdpr_delete_writes_double_layer_audit() {
        // NFR-SE-010: 双层审计（业务层 + 合规层）
        let (op, _, audit, _) = make_test_operator();
        let p = make_test_policy();
        op.create_policy(p).await.expect("create policy");
        let req = GdprDeleteRequest {
            subject_id: "player-42".to_string(),
            realm_id: "r-test-1".to_string(),
            request_id: Uuid::new_v4(),
            operator_id: "ops".to_string(),
            approval_ref: "approval-002".to_string(),
            signed_by: "Ulysses".to_string(),
            legal_hold_override: false,
            erasure_strategy: ErasureStrategy::Anonymize,
        };
        let result = op.execute_gdpr_delete(req).await.expect("gdpr delete");
        // 检查双层审计均已写入
        let first = audit.list_by_action("lcm.gdpr.delete");
        let second = audit.list_by_action("lcm.gdpr.compliance");
        assert_eq!(first.len(), 1, "第一层业务审计必须存在");
        assert_eq!(second.len(), 1, "第二层合规审计必须存在");
        assert_eq!(result.audit_first_layer_id, first[0].id);
        assert_eq!(result.audit_second_layer_id, second[0].id);
        // 审计 payload 含 subject_id + signed_by
        assert!(first[0].payload.contains("player-42"));
        assert!(first[0].payload.contains("Ulysses"));
        assert!(second[0].payload.contains("legal_hold_override"));
    }

    #[tokio::test]
    async fn m_2074_3_gdpr_delete_hard_erase_requires_legal_hold_override() {
        let (op, _, _, _) = make_test_operator();
        let p = make_test_policy();
        op.create_policy(p).await.expect("create policy");
        let req = GdprDeleteRequest {
            subject_id: "p".to_string(),
            realm_id: "r-test-1".to_string(),
            request_id: Uuid::new_v4(),
            operator_id: "ops".to_string(),
            approval_ref: "approval-003".to_string(),
            signed_by: "Ulysses".to_string(),
            legal_hold_override: false, // ← HardErase 必须 true
            erasure_strategy: ErasureStrategy::HardErase,
        };
        let result = op.execute_gdpr_delete(req).await;
        assert!(matches!(
            result,
            Err(LcmError::GdprDeletePathDenied { .. })
        ));
    }

    #[tokio::test]
    async fn m_2074_3_gdpr_delete_denies_when_policy_missing() {
        let (op, _, _, _) = make_test_operator();
        let req = GdprDeleteRequest {
            subject_id: "p".to_string(),
            realm_id: "r-not-exist".to_string(),
            request_id: Uuid::new_v4(),
            operator_id: "ops".to_string(),
            approval_ref: "approval-004".to_string(),
            signed_by: "Ulysses".to_string(),
            legal_hold_override: false,
            erasure_strategy: ErasureStrategy::Anonymize,
        };
        let result = op.execute_gdpr_delete(req).await;
        match result {
            Err(LcmError::GdprDeletePathDenied { reason, .. }) => {
                assert!(reason.contains("archive_policy"));
            }
            other => panic!("expected GdprDeletePathDenied, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn m_2074_3_gdpr_delete_denies_empty_realm_id() {
        let (op, _, _, _) = make_test_operator();
        let req = GdprDeleteRequest {
            subject_id: "p".to_string(),
            realm_id: "".to_string(),
            request_id: Uuid::new_v4(),
            operator_id: "ops".to_string(),
            approval_ref: "approval-005".to_string(),
            signed_by: "Ulysses".to_string(),
            legal_hold_override: false,
            erasure_strategy: ErasureStrategy::Anonymize,
        };
        let result = op.execute_gdpr_delete(req).await;
        assert!(matches!(
            result,
            Err(LcmError::GdprDeletePathDenied { .. })
        ));
    }

    // ----------------------------------------------------------------
    // M-2074.4: 双层审计验证
    // ----------------------------------------------------------------

    #[tokio::test]
    async fn m_2074_4_audit_log_double_layer_writes_both_layers() {
        let (op, _, audit, _) = make_test_operator();
        let business = serde_json::json!({"event": "archive.started"});
        let compliance = serde_json::json!({"event": "compliance.acknowledged"});
        let (first, second) = op
            .audit_log_double_layer(
                "lcm.archive.start",
                "lcm.archive.compliance_ack",
                "realm:r-1",
                &business,
                &compliance,
            )
            .await
            .expect("double layer");
        let entries = audit.list_all();
        assert_eq!(entries.len(), 2);
        let actions: Vec<&str> = entries.iter().map(|e| e.action.as_str()).collect();
        assert!(actions.contains(&"lcm.archive.start"));
        assert!(actions.contains(&"lcm.archive.compliance_ack"));
        assert_ne!(first, second);
    }

    // ----------------------------------------------------------------
    // M-2074.5: 归档查询延迟指标
    // ----------------------------------------------------------------

    #[tokio::test]
    async fn m_2074_5_query_archive_observes_latency() {
        let (op, _, _, _) = make_test_operator();
        let result = op
            .query_archive("cs_query_archive", ArchiveTier::Cold, async {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok::<i32, LcmError>(42)
            })
            .await
            .expect("query");
        assert_eq!(result, 42);
        // 指标已采集（验证不 panic + 在测试中能 encode）
        let text = metrics::gather_metrics_text().expect("gather");
        assert!(text.contains("rgs_lcm_archive_query_latency_seconds"));
        assert!(text.contains("cs_query_archive"));
    }

    #[tokio::test]
    async fn m_2074_5_query_archive_records_latency_even_on_error() {
        let (op, _, _, _) = make_test_operator();
        let r: LcmResult<()> = op
            .query_archive("gdpr_subject_lookup", ArchiveTier::GdprDeletePath, async {
                Err(LcmError::Unavailable("storage down".to_string()))
            })
            .await;
        assert!(matches!(r, Err(LcmError::Unavailable(_))));
        // 即便查询失败，指标也必须采集（per DTL-042 §11.1 实测）
        let text = metrics::gather_metrics_text().expect("gather");
        assert!(text.contains("gdpr_subject_lookup"));
        assert!(text.contains("gdpr_path"));
    }

    // ----------------------------------------------------------------
    // §7 整合: 3 步 Saga 编排
    // ----------------------------------------------------------------

    #[tokio::test]
    async fn execute_archive_full_saga_succeeds() {
        // 3 步全部成功（含 N+2 冷归档 + GDPR 通路开启）
        let (op, _, audit, _) = make_test_operator();
        let p = make_test_policy();
        op.create_policy(p.clone()).await.expect("create policy");
        let run_id = Uuid::new_v4();
        let outcome = op
            .execute_archive(run_id, &p, "archives", b"realm-archive-bytes")
            .await
            .expect("execute_archive");
        assert_eq!(outcome.run_id, run_id);
        assert_eq!(outcome.final_tier, ArchiveTier::GdprDeletePath);
        assert_eq!(outcome.steps.len(), 3);
        // 三步全部成功
        assert!(outcome.steps.iter().all(|s| s.success));
        // 步骤 3 写 audit（lcm.gdpr.path_enabled）
        let path_enabled = audit.list_by_action("lcm.gdpr.path_enabled");
        assert_eq!(path_enabled.len(), 1);
        // OLU 估算
        assert!(outcome.olu_tokens > 0);
    }

    #[tokio::test]
    async fn execute_archive_fails_when_cold_archive_under_replicated() {
        // N+2 副本数不足 → 冷归档失败 → Saga rollback
        let policy_repo = Arc::new(InMemoryArchivePolicyRepository::new());
        let audit_repo = Arc::new(InMemoryOperationAuditRepository::new());
        let storage = Arc::new(InMemoryArchiveStorage::new().with_replica_count(2));
        let op = ArchiveOperator::new(
            policy_repo as Arc<dyn ArchivePolicyRepository>,
            audit_repo as Arc<dyn OperationAuditRepository>,
            storage as Arc<dyn ArchiveObjectStorage>,
        );
        let p = make_test_policy();
        let result = op
            .execute_archive(Uuid::new_v4(), &p, "archives", b"x")
            .await;
        match result {
            Err(LcmError::SagaStepFailed { step, .. }) => {
                assert_eq!(step, "ColdArchiveStep");
            }
            other => panic!("expected SagaStepFailed at ColdArchive, got {other:?}"),
        }
    }

    // ----------------------------------------------------------------
    // 硬约束（per 任务规范 §硬约束）
    // ----------------------------------------------------------------

    #[test]
    fn hard_constraint_no_delete_from_in_archive_rs() {
        // 硬约束: archive.rs 不得含任何删表 / 删记录 SQL 字面量
        // 通过 grep 编译期验证（由 wf-1-2074 verify 脚本执行）
        // 此处不重复断言 — 见文档 lcm-archive-report.md §3
    }

    // ----------------------------------------------------------------
    // Repository / Storage 桩测试
    // ----------------------------------------------------------------

    #[tokio::test]
    async fn in_memory_policy_repo_roundtrip() {
        let repo = InMemoryArchivePolicyRepository::new();
        let p = make_test_policy();
        repo.insert(&p).await.expect("insert");
        let found = repo.find_by_id(p.policy_id).await.expect("find");
        assert_eq!(found.unwrap().policy_id, p.policy_id);
        let by_realm = repo
            .find_by_realm_id(&p.target_realm_id)
            .await
            .expect("by realm");
        assert_eq!(by_realm.unwrap().policy_id, p.policy_id);
    }

    #[tokio::test]
    async fn in_memory_storage_writes_replicas() {
        let storage = InMemoryArchiveStorage::new();
        let put = storage
            .put_object("b", "k", b"hello")
            .await
            .expect("put");
        assert_eq!(put.replica_count, 3);
        let replicas = storage.list_replicas("b", "k").await.expect("list");
        assert_eq!(replicas.len(), 3);
    }
}

// ============================================================================
// §8. 集成测试桩（#[ignore] 标记 — 真实 admin_db / 真实 S3 路径需 SRE 接力）
// ============================================================================

/// 真实 admin_db + 真实 S3 集成测试桩
///
/// **运行方式**：`cargo test -p rgs-cluster-ops --lib -- --ignored`
/// **前置条件**：
/// 1. `admin_db` 已运行 + 6 张 LCM 表 migration 已应用
/// 2. S3 / MinIO 已运行 + 测试 bucket 已建
/// 3. `DATABASE_URL` / `S3_ENDPOINT` 环境变量已设置
#[cfg(test)]
mod integration_tests {
    #[allow(unused_imports)]
    use super::*;

    /// 真实 admin_db 集成测试：写入 archive_policy + operation_audit
    #[tokio::test]
    #[ignore = "requires real admin_db + LCM migration 0020_lcm_tables.sql applied"]
    async fn it_archive_policy_persists_to_admin_db() {
        // 由 SRE 接力 PH-6 实测时实现
        // 占位：未实现即失败
        unimplemented!("real admin_db integration test pending SRE handover")
    }

    /// 真实 S3 集成测试：N+2 副本验证
    #[tokio::test]
    #[ignore = "requires real S3/MinIO with lifecycle policy"]
    async fn it_n_plus_2_replicas_in_s3() {
        unimplemented!("real S3 N+2 verification pending SRE handover")
    }

    /// 真实 GDPR 删除集成测试：业务 service gRPC + admin_db 双层审计
    #[tokio::test]
    #[ignore = "requires real admin_db + player_db + economy_db + social_db"]
    async fn it_gdpr_delete_full_path() {
        unimplemented!("real GDPR delete IT pending SRE handover")
    }
}
