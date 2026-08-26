//! Saga 占位（per RGS-SPEC-DTL-042 §2 + §3 第 3 条 + IMPL-PLAN-LCM-001 §3.5）
//!
//! SagaOrchestrator 是 RealmLifecycleService 内部模块，**不**分发独立协调服务；
//! 跨 DB 写入复用 RGS-ADR-0015 Saga 适用边界与单一调解者原则。
//!
//! 本任务仅完成占位 + trait 形状；具体步骤实现在 `steps.rs` 与各 `plans/` 子模块。

pub mod steps;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::operators::{OperatorInput, OperatorOutput};

/// Saga 步骤 ID（per SPEC §2 SagaOrchestrator 步骤）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SagaStepKind {
    Plan,
    Precheck,
    Execute,
    Validate,
    Commit,
    Compensate,
}

/// Saga 步骤记录（运行态）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SagaStepRecord {
    pub step: SagaStepKind,
    pub saga_run_id: Uuid,
    pub request_id: Uuid,
    pub phase: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// SagaOrchestrator trait（per SPEC §2）
///
/// 调用方：RealmLifecycleService；实现方：6 阶段操作器各自的 Saga 编排。
/// 步骤默认 60s 超时（per SPEC §5 背压）；超时触发反向补偿。
#[async_trait]
pub trait SagaOrchestrator: Send + Sync {
    /// 启动一次 Saga run，返回 saga_run_id
    async fn start(&self, phase: &str, input: &OperatorInput) -> Result<Uuid>;

    /// 推进到下一阶段
    async fn advance(
        &self,
        saga_run_id: Uuid,
        current: SagaStepKind,
        output: &OperatorOutput,
    ) -> Result<SagaStepRecord>;

    /// 反向补偿
    async fn compensate(&self, saga_run_id: Uuid, reason: &str) -> Result<()>;

    /// 查询 Saga run 状态
    async fn status(&self, saga_run_id: Uuid) -> Result<Vec<SagaStepRecord>>;
}

/// InMemory 占位实现（PH-4 前使用；具体 pg 落库 + Redis 短租约 + fencing 在
/// SagaOrchestrator pg 实现内完成）
pub struct InMemorySagaOrchestrator {
    // 简单 key-value
    inner: std::sync::Mutex<Vec<SagaStepRecord>>,
}

impl InMemorySagaOrchestrator {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemorySagaOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SagaOrchestrator for InMemorySagaOrchestrator {
    async fn start(&self, phase: &str, input: &OperatorInput) -> Result<Uuid> {
        let run_id = Uuid::new_v4();
        let rec = SagaStepRecord {
            step: SagaStepKind::Plan,
            saga_run_id: run_id,
            request_id: input.request_id,
            phase: phase.to_string(),
            started_at: chrono::Utc::now(),
            finished_at: None,
        };
        self.inner.lock().unwrap().push(rec);
        Ok(run_id)
    }

    async fn advance(
        &self,
        saga_run_id: Uuid,
        current: SagaStepKind,
        output: &OperatorOutput,
    ) -> Result<SagaStepRecord> {
        let now = chrono::Utc::now();
        let rec = SagaStepRecord {
            step: current,
            saga_run_id,
            request_id: Uuid::nil(),
            phase: output.phase.clone(),
            started_at: now,
            finished_at: Some(now),
        };
        self.inner.lock().unwrap().push(rec.clone());
        Ok(rec)
    }

    async fn compensate(&self, _saga_run_id: Uuid, _reason: &str) -> Result<()> {
        // 占位：仅记日志；具体反向步骤由各 plan 模块实现
        Ok(())
    }

    async fn status(&self, saga_run_id: Uuid) -> Result<Vec<SagaStepRecord>> {
        let guard = self.inner.lock().unwrap();
        Ok(guard
            .iter()
            .filter(|r| r.saga_run_id == saga_run_id)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_lifecycle::operators::OperatorInput;

    fn input() -> OperatorInput {
        OperatorInput {
            request_id: Uuid::new_v4(),
            operator_id: Uuid::new_v4(),
            approval_ref: None,
            trace_id: "t1".to_string(),
        }
    }

    #[tokio::test]
    async fn in_memory_saga_start_and_advance() {
        let s = InMemorySagaOrchestrator::new();
        let run_id = s.start("new_realm", &input()).await.unwrap();
        let out = OperatorOutput {
            run_id,
            feature_id: "f1".to_string(),
            phase: "new_realm".to_string(),
        };
        s.advance(run_id, SagaStepKind::Precheck, &out).await.unwrap();
        let st = s.status(run_id).await.unwrap();
        assert_eq!(st.len(), 2);
    }
}
