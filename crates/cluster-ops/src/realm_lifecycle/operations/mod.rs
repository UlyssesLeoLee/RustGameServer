//! 6 阶段操作器子模块入口（per RGS-DTL-042 §5 + RGS-IMPL-PLAN-LCM-001 §2.2）
//!
//! WF-1-2066 M-2066.4 ~ M-2066.9 操作器骨架
//!
//! 6 操作器对应 6 阶段 + 1 个合服回退子操作：
//! - `new_realm`  → NewRealmOperator    (FR-LCM-010~033)
//! - `scale`      → ScaleOperator       (FR-LCM-040~044，扩缩容双向)
//! - `split`      → SplitOperator       (FR-LCM-050~055)
//! - `merge`      → MergeOperator       (FR-LCM-060~064) + MergeRollbackOperator
//! - `retire`     → RetireOperator      (FR-LCM-070~075)
//! - `archive`    → ArchiveOperator     (FR-LCM-080~085，含冷热分层占位)
//!
//! 骨架阶段（per M-2066.4~9 任务范围）：
//! - 6 操作器**仅**实现 `RealmLifecycleOperator` trait 接口（`name` / `stage` / `execute`）
//! - **不**实现业务逻辑（DB 持久化 / Saga 编排 / 演练 / 跨域 gRPC）
//! - `execute()` 统一返回 `LcmErrorKind::NotImplemented` 占位结果
//!   等待 L4 #2067 (Saga) / #2068 (6 表) / #2070 (Drill) / #2071 (Feature) / #2073 (跨域) 接入
//!
//! 公共 trait 定义见 `super::RealmLifecycleOperator`（service.rs）

use uuid::Uuid;

use super::error::{LcmError, LcmErrorKind, LcmResult};

pub mod archive;
pub mod merge;
pub mod new_realm;
pub mod retire;
pub mod scale;
pub mod split;

pub use archive::ArchiveOperator;
pub use merge::{MergeOperator, MergeRollbackOperator};
pub use new_realm::NewRealmOperator;
pub use retire::RetireOperator;
pub use scale::ScaleOperator;
pub use split::SplitOperator;

// ============================================================================
// 共享骨架工具：6 操作器都需要的参数校验 + 占位错误生成
// ============================================================================

/// 共享参数校验（per FR-LCM-002 阶段变更全流程留痕 + RGS-DTL-031 §3.1 幂等性）
///
/// 注：本骨架阶段**不**真正调 admin_db；仅做参数合法性校验。
/// 实际持久化由 L4 #2068 6 表 migration 接入后完成。
pub(crate) fn validate_request(request_id: Uuid, realm_id: &str) -> LcmResult<()> {
    if request_id.is_nil() {
        return Err(LcmError::invalid_parameter(
            "request_id must be a non-nil UUID (per RGS-DTL-031 §3.1 idempotency)",
        ));
    }
    if realm_id.is_empty() {
        return Err(LcmError::invalid_parameter(
            "realm_id must not be empty (per RGS-DTL-042 §3 implementation contract)",
        ));
    }
    if realm_id.len() > 64 {
        return Err(LcmError::invalid_parameter(format!(
            "realm_id length {} exceeds 64-char limit (per RGS-SPEC-CROSS-005 §2.2)",
            realm_id.len()
        )));
    }
    Ok(())
}

/// 骨架阶段占位：6 操作器统一返回 `NotImplemented` 错误
///
/// 仅在 L4 #2066 阶段变更路径走骨架时返回；上线 M-2067 Saga 后应消失
/// （per RGS-IMPL-PLAN-LCM-001 §3.1 L4 #2066 阶段产物）。
pub(crate) fn not_implemented_skeleton(
    operator: &'static str,
    next_milestone: &'static str,
) -> LcmError {
    LcmError::new(LcmErrorKind::NotImplemented {
        operator: operator.to_string(),
        milestone: next_milestone.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_request_rejects_nil_uuid() {
        let err = validate_request(Uuid::nil(), "realm-001").unwrap_err();
        assert!(matches!(err.kind, LcmErrorKind::InvalidParameter(_)));
    }

    #[test]
    fn validate_request_rejects_empty_realm_id() {
        let err = validate_request(Uuid::new_v4(), "").unwrap_err();
        assert!(matches!(err.kind, LcmErrorKind::InvalidParameter(_)));
    }

    #[test]
    fn validate_request_rejects_overlong_realm_id() {
        let long_id = "a".repeat(65);
        let err = validate_request(Uuid::new_v4(), &long_id).unwrap_err();
        assert!(matches!(err.kind, LcmErrorKind::InvalidParameter(_)));
    }

    #[test]
    fn validate_request_accepts_well_formed_inputs() {
        let r = validate_request(Uuid::new_v4(), "realm-001");
        assert!(r.is_ok());
    }
}
