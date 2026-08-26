//! 6 阶段操作器 trait 定义（per RGS-SPEC-DTL-042 §2 + IMPL-PLAN-LCM-001 §3.5）
//!
//! 注：merge_rollback 走 merge 操作器**逆向**补偿路径而非独立 trait，因此本模块
//! 仅 6 个 trait（new_realm / scale / split / merge / retire / archive），7 子类
//! 全部走 PFAU 编排（per SPEC §3 第 2 条 + DTL-031 §1.1）。
//!
//! 每个 trait 定义：plan 构造 + execute + rollback；具体实现在各自 plan 模块内
//! （plans/{new_realm,scale,split,merge,retire,archive}_plan.rs；本任务仅占位）。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::Result;

/// 6 阶段操作器共同输入：操作者 + 幂等键 + 跟踪信息（per SPEC §3 第末段）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorInput {
    /// 幂等键（per DTL-031 §3.1）
    pub request_id: Uuid,
    /// 操作者 ID
    pub operator_id: Uuid,
    /// 高危操作的审批引用
    pub approval_ref: Option<String>,
    /// 跟踪 ID
    pub trace_id: String,
}

/// 6 阶段操作器共同输出：run_id + feature_id + 阶段名
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorOutput {
    /// 阶段 run_id（写入 realm_lifecycle_run 表）
    pub run_id: Uuid,
    /// Feature ID（per feature_registry）
    pub feature_id: String,
    /// 阶段名（snake_case：new_realm / scale / split / merge / retire / archive）
    pub phase: String,
}

// ============================================================================
// 6 阶段操作器 trait
// ============================================================================

/// 阶段 1：开新服（per FR-LCM-001）
#[async_trait]
pub trait NewRealmOperator: Send + Sync {
    async fn plan(&self, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn execute(&self, output: &OperatorOutput) -> Result<()>;
    async fn rollback(&self, output: &OperatorOutput) -> Result<()>;
}

/// 阶段 2：扩缩容（per FR-LCM-002）
#[async_trait]
pub trait ScaleOperator: Send + Sync {
    async fn plan(&self, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn execute(&self, output: &OperatorOutput) -> Result<()>;
    async fn rollback(&self, output: &OperatorOutput) -> Result<()>;
}

/// 阶段 3：分服（per FR-LCM-031）
#[async_trait]
pub trait SplitOperator: Send + Sync {
    async fn plan(&self, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn execute(&self, output: &OperatorOutput) -> Result<()>;
    async fn rollback(&self, output: &OperatorOutput) -> Result<()>;
}

/// 阶段 4：合服（per FR-LCM-041；merge_rollback 由 merge 走逆向补偿）
#[async_trait]
pub trait MergeOperator: Send + Sync {
    async fn plan(&self, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn execute(&self, output: &OperatorOutput) -> Result<()>;
    async fn rollback(&self, output: &OperatorOutput) -> Result<()>;
}

/// 阶段 5：退场（per FR-LCM-061）
#[async_trait]
pub trait RetireOperator: Send + Sync {
    async fn plan(&self, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn execute(&self, output: &OperatorOutput) -> Result<()>;
    async fn rollback(&self, output: &OperatorOutput) -> Result<()>;
}

/// 阶段 6：归档（per FR-LCM-081；**仅**迁移存储位置，**不**删除数据）
#[async_trait]
pub trait ArchiveOperator: Send + Sync {
    async fn plan(&self, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn execute(&self, output: &OperatorOutput) -> Result<()>;
    async fn rollback(&self, output: &OperatorOutput) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_input_serde_roundtrip() {
        let req = Uuid::new_v4();
        let op = Uuid::new_v4();
        let input = OperatorInput {
            request_id: req,
            operator_id: op,
            approval_ref: Some("approval-1".to_string()),
            trace_id: "trace-x".to_string(),
        };
        let s = serde_json::to_string(&input).unwrap();
        let back: OperatorInput = serde_json::from_str(&s).unwrap();
        assert_eq!(back, input);
    }
}
