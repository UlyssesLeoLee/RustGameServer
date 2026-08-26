//! Saga 步骤占位（per RGS-SPEC-DTL-042 §2 + IMPL-PLAN-LCM-001 §3.5）
//!
//! 7 步 Saga 的反向补偿由各 plan 模块实现；本文件仅做 trait 形状占位。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::operators::{OperatorInput, OperatorOutput};

/// 7 步 Saga 中具体一步定义（per SPEC §2 SagaOrchestrator）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SagaStepSpec {
    pub id: String,
    pub phase: String,
    /// 默认 60s（per SPEC §5 背压）
    pub timeout_seconds: u32,
    /// 反向步骤 ID（None 表示无补偿）
    pub compensation: Option<String>,
}

/// 步骤执行器 trait（per SPEC §2 Saga 步骤执行）
#[async_trait]
pub trait SagaStepExecutor: Send + Sync {
    async fn execute(&self, step: &SagaStepSpec, input: &OperatorInput) -> Result<OperatorOutput>;
    async fn compensate(&self, step: &SagaStepSpec, output: &OperatorOutput) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saga_step_spec_default_timeout() {
        let s = SagaStepSpec {
            id: "precheck".to_string(),
            phase: "new_realm".to_string(),
            timeout_seconds: 60,
            compensation: Some("rollback_precheck".to_string()),
        };
        assert_eq!(s.timeout_seconds, 60);
        assert!(s.compensation.is_some());
    }

    #[test]
    fn saga_step_spec_uniqueness_helper() {
        // 占位：将来用 step id 在 steps 容器内查重
    }
}
