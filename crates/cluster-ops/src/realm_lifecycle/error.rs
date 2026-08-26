//! rgs-realm-lifecycle 错误类型
//!
//! **职责**（per RGS-DTL-042 §1.1 + RGS-SPEC-DTL-042 §5）：
//! - Saga 步骤失败 / PFAU 失联 / admin_db 写失败 / 业务 service gRPC 失败
//! - 演练环境故障 / 灰度回滚 / OLU 预算超限 / 跨 DB 长事务阻塞
//! - **GDPR 删除通路失败**（per NFR-SE-010 合规例外 — 资金/合规相关硬约束）
//!
//! **与既有 `cluster_ops::Error` 的关系**：
//! - 本模块**不**与 `Error` 平级（避免业务层双错误类型歧义）
//! - 跨域调用（PH-5 业务 service gRPC 集成时）通过 `From<LcmError> for cluster_ops::Error`
//!   透传为 `cluster_ops::Error::Internal`
//!
//! **资金/合规相关标注**（per RGS-SPEC-DTL-042 §4.3 关键标注）：
//! - `LcmError::GdprDeletePathFailed` 涉合规 — 任何生产路径触发此错误需 Ulysses 显式签字

use thiserror::Error;

/// LCM 错误分类（per RGS-DTL-042 §10 故障降级 + RGS-SPEC-DTL-042 §5 故障表）
#[derive(Debug, Error)]
pub enum LcmError {
    // ===== Saga / PFAU 编排 =====
    /// Saga 步骤失败（per RGS-DTL-042 §10 + ADR-0015 适用边界）
    #[error("saga step {step} failed: {reason}")]
    SagaStepFailed { step: String, reason: String },

    /// PFAU 编排失联（per ClusterOpsService PFAU 监督者）
    #[error("PFAU orchestrator unreachable: {0}")]
    PfauUnreachable(String),

    /// 演练未通过（per FR-LCM-003 门禁：drill_validated → executing 前置条件）
    #[error("drill not passed: {0}")]
    DrillNotPassed(String),

    /// 灰度回滚触发
    #[error("gray rollback triggered: {0}")]
    GrayRollback(String),

    // ===== 数据治理 =====
    /// OLU 预算超限（per NFR-LCM-007 硬约束）
    #[error("OLU budget exceeded: team={team} phase={phase} consumed={consumed} limit={limit}")]
    OluBudgetExceeded {
        team: String,
        phase: String,
        consumed: u64,
        limit: u64,
    },

    /// admin_db 写失败
    #[error("admin_db write failed: {0}")]
    AdminDbWriteFailed(String),

    /// 业务 service gRPC 失败（PH-5 集成时使用）
    #[error("business service gRPC failed: service={service} method={method} reason={reason}")]
    BusinessServiceFailed {
        service: String,
        method: String,
        reason: String,
    },

    /// 跨 DB 长事务阻塞
    #[error("cross-DB long transaction blocked: {0}")]
    CrossDbBlocked(String),

    // ===== 归档（per WBS L4 #2074） =====
    /// 归档策略无效（3 年热 + 10 年冷 + N+2 校验失败）
    #[error("invalid archive policy: {0}")]
    InvalidArchivePolicy(String),

    /// 热归档失败（per DTL-042 §6.6 Saga 步骤 1）
    #[error("hot archive failed: realm={realm_id} reason={reason}")]
    HotArchiveFailed { realm_id: String, reason: String },

    /// 冷归档失败（N+2 副本数不达标，per RSK-LCM-005 缓解）
    #[error("cold archive failed: realm={realm_id} replica_count={replica_count} required={required} reason={reason}")]
    ColdArchiveFailed {
        realm_id: String,
        replica_count: u8,
        required: u8,
        reason: String,
    },

    /// GDPR 删除通路失败（**资金/合规相关硬约束** per SPEC §4.3）
    ///
    /// 此错误由 `ArchiveOperator::execute_gdpr_delete` 抛出，触发时**必须**：
    /// 1. 写 `admin_db.operation_audit` 第一层 + 错误描述审计（第二层）
    /// 2. 通知运营 / 法务 / 合规
    /// 3. Ulysses 显式签字（per ADR-0055 §4.3 — 不能 PR review 顶替）
    #[error("GDPR delete path failed: subject_id={subject_id} realm={realm_id} reason={reason}")]
    GdprDeletePathFailed {
        subject_id: String,
        realm_id: String,
        reason: String,
    },

    /// GDPR 删除通路被拒绝（subject_id 不在已归档 realm / RBAC 不通过 / 法律保留期未过）
    #[error("GDPR delete path denied: subject_id={subject_id} realm={realm_id} reason={reason}")]
    GdprDeletePathDenied {
        subject_id: String,
        realm_id: String,
        reason: String,
    },

    // ===== 通用 =====
    /// 数据库错误
    #[error("database error: {0}")]
    Database(#[source] Box<sqlx::Error>),

    /// 资源未找到
    #[error("not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    /// 验证错误
    #[error("validation error: {0}")]
    Validation(String),

    /// 冲突（幂等键重复 / state 非法跳转）
    #[error("conflict: {0}")]
    Conflict(String),

    /// 未授权
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// 服务不可用（per RGS-DTL-042 §10 故障降级）
    #[error("unavailable: {0}")]
    Unavailable(String),

    /// 内部错误
    #[error("internal error: {0}")]
    Internal(#[source] anyhow::Error),
}

/// LCM 统一 Result 类型
pub type LcmResult<T> = std::result::Result<T, LcmError>;

impl From<sqlx::Error> for LcmError {
    fn from(e: sqlx::Error) -> Self {
        LcmError::Database(Box::new(e))
    }
}

impl From<anyhow::Error> for LcmError {
    fn from(e: anyhow::Error) -> Self {
        LcmError::Internal(e)
    }
}

impl From<serde_json::Error> for LcmError {
    fn from(e: serde_json::Error) -> Self {
        LcmError::Internal(anyhow::anyhow!("serde_json error: {}", e))
    }
}

impl LcmError {
    /// 是否为资金/合规相关错误（影响是否需要 Ulysses 显式签字）
    ///
    /// per RGS-SPEC-DTL-042 §4.3 关键标注：
    /// - GDPR 删除通路相关错误必须显式独立签字
    /// - OLU 预算超限属于强治理（仍走标准审批）
    pub fn is_compliance_critical(&self) -> bool {
        matches!(
            self,
            LcmError::GdprDeletePathFailed { .. } | LcmError::GdprDeletePathDenied { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdpr_failed_is_compliance_critical() {
        // SPEC §4.3 关键标注: GDPR 相关错误必须 Ulysses 显式签字
        let e = LcmError::GdprDeletePathFailed {
            subject_id: "player-1".to_string(),
            realm_id: "r-1".to_string(),
            reason: "audit chain broken".to_string(),
        };
        assert!(e.is_compliance_critical());
    }

    #[test]
    fn gdpr_denied_is_compliance_critical() {
        let e = LcmError::GdprDeletePathDenied {
            subject_id: "player-1".to_string(),
            realm_id: "r-1".to_string(),
            reason: "subject not in archived realm".to_string(),
        };
        assert!(e.is_compliance_critical());
    }

    #[test]
    fn saga_failure_is_not_compliance_critical() {
        // 业务 Saga 失败走标准 PFAU 编排，**不**触发显式签字
        let e = LcmError::SagaStepFailed {
            step: "HotArchiveStep".to_string(),
            reason: "DB switch timeout".to_string(),
        };
        assert!(!e.is_compliance_critical());
    }

    #[test]
    fn olu_budget_exceeded_is_not_compliance_critical() {
        // OLU 预算超限走标准 SRE 审批（per NFR-LCM-007）
        let e = LcmError::OluBudgetExceeded {
            team: "admin".to_string(),
            phase: "archive".to_string(),
            consumed: 21_000_000,
            limit: 20_000_000,
        };
        assert!(!e.is_compliance_critical());
    }

    #[test]
    fn cold_archive_n_plus_2_validation_error() {
        // RSK-LCM-005: replica_count < required 触发 ColdArchiveFailed
        let e = LcmError::ColdArchiveFailed {
            realm_id: "r-1".to_string(),
            replica_count: 2,
            required: 3, // N+2 of N=1 → required=3
            reason: "replica 3 unreachable".to_string(),
        };
        let msg = e.to_string();
        assert!(msg.contains("replica_count=2"));
        assert!(msg.contains("required=3"));
    }
}
