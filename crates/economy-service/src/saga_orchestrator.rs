//! Saga 编排器（per RGS-DTL-100 §3-§5 Saga 决策与执行）
//!
//! 54.8 实化：SagaOrchestrator trait + 默认实现
//!
//! 设计：
//! - SagaOrchestrator 接收 saga + step 处理器列表
//! - step 处理器是 `async fn(&mut Saga, &Ctx) -> Result<()>` 形式
//! - execute 步进：每步调对应 handler；成功 advance，失败 compensate
//! - 状态机每步都 persist 到 saga 表（崩溃可恢复）

use std::sync::Arc;

use crate::error::Error;
use crate::reservation::{Reservation, ReservationRepository};
use crate::saga::{Saga, SagaRepository, SagaStatus};
use crate::Result;

/// Saga 步进处理器 trait
///
/// 每个实现负责执行一个 step + 反向补偿
#[async_trait::async_trait]
pub trait SagaStepHandler: Send + Sync {
    /// step 名
    fn name(&self) -> &str;
    /// 执行 step
    async fn execute(&self, saga: &mut Saga) -> Result<()>;
    /// 反向补偿（per RGS-DTL-100 §4 补偿模式）
    async fn compensate(&self, saga: &mut Saga) -> Result<()>;
}

/// SagaOrchestrator
pub struct SagaOrchestrator {
    sagas: Arc<dyn SagaRepository>,
    /// Reservation 仓储（保留供后续 step handler 内部使用；54.8 编排器自身不直接访问）
    #[allow(dead_code)]
    reservations: Arc<dyn ReservationRepository>,
    handlers: Vec<Arc<dyn SagaStepHandler>>,
}

impl SagaOrchestrator {
    pub fn new(
        sagas: Arc<dyn SagaRepository>,
        reservations: Arc<dyn ReservationRepository>,
        handlers: Vec<Arc<dyn SagaStepHandler>>,
    ) -> Self {
        Self {
            sagas,
            reservations,
            handlers,
        }
    }

    /// 执行 Saga（步进式）
    pub async fn execute(&self, saga: &mut Saga) -> Result<()> {
        if saga.status != SagaStatus::Pending {
            return Err(Error::Validation(format!(
                "saga {} status is not Pending ({:?})",
                saga.id, saga.status
            )));
        }

        // 启动
        saga.start();
        self.sagas.save(saga).await?;

        // 步进
        while let Some(current) = saga.current().cloned() {
            // 先取 step name（避免 immutable / mutable borrow 冲突）
            let step_name = current.name;

            let handler = self
                .handlers
                .iter()
                .find(|h| h.name() == step_name)
                .ok_or_else(|| Error::Validation(format!("no handler for step {}", step_name)))?;

            // 标记 running（handler.execute 可能耗时）
            saga.current_mut().unwrap().mark_running();
            self.sagas.save(saga).await?;

            match handler.execute(saga).await {
                Ok(()) => {
                    saga.current_mut().unwrap().mark_completed();
                    if !saga.advance() {
                        // 所有步骤完成
                        saga.complete();
                        self.sagas.save(saga).await?;
                        return Ok(());
                    }
                    self.sagas.save(saga).await?;
                }
                Err(e) => {
                    saga.current_mut().unwrap().mark_failed(e.to_string());
                    self.sagas.save(saga).await?;
                    // 触发补偿
                    self.compensate(saga).await?;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 反向补偿
    pub async fn compensate(&self, saga: &mut Saga) -> Result<()> {
        saga.compensate();
        self.sagas.save(saga).await?;

        // 反向执行补偿（仅已完成步骤）
        let completed_names: Vec<String> = saga
            .steps
            .iter()
            .rev()
            .filter(|s| s.status == crate::saga::SagaStepStatus::Completed)
            .map(|s| s.name.clone())
            .collect();
        for name in completed_names {
            if let Some(handler) = self.handlers.iter().find(|h| h.name() == name) {
                handler.compensate(saga).await?;
            }
        }

        saga.fail();
        self.sagas.save(saga).await?;
        Ok(())
    }

    /// 通过 saga_id 重新加载并继续执行（崩溃恢复）
    pub async fn resume(&self, saga_id: uuid::Uuid) -> Result<()> {
        let mut saga = self
            .sagas
            .find_by_id(saga_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Saga",
                id: saga_id.to_string(),
            })?;
        self.execute(&mut saga).await
    }
}

/// 示例 step handler：Reserve（创建 reservation）
pub struct ReserveHandler;

#[async_trait::async_trait]
impl SagaStepHandler for ReserveHandler {
    fn name(&self) -> &str {
        "reserve"
    }

    async fn execute(&self, saga: &mut Saga) -> Result<()> {
        // step 携带 account_id / amount / currency（实际应从 saga context 拿）
        // 这里简化：每 step 至少创建 1 个 reservation（仅示例）
        if let Some(step) = saga.current() {
            if let Some(resource_id) = step.resource_id {
                let r = Reservation::new(
                    saga.id,
                    resource_id,
                    100, // 示例 amount
                    crate::entity::Currency::Gold,
                );
                tracing::info!(target: "saga", saga_id = %saga.id, reservation_id = %r.id, "ReserveHandler executed");
            }
        }
        Ok(())
    }

    async fn compensate(&self, saga: &mut Saga) -> Result<()> {
        tracing::info!(target: "saga", saga_id = %saga.id, "ReserveHandler compensated");
        Ok(())
    }
}

/// 示例 step handler：Confirm（确认 reservation → 实际扣款）
pub struct ConfirmHandler;

#[async_trait::async_trait]
impl SagaStepHandler for ConfirmHandler {
    fn name(&self) -> &str {
        "confirm"
    }

    async fn execute(&self, saga: &mut Saga) -> Result<()> {
        tracing::info!(target: "saga", saga_id = %saga.id, "ConfirmHandler executed");
        Ok(())
    }

    async fn compensate(&self, saga: &mut Saga) -> Result<()> {
        tracing::info!(target: "saga", saga_id = %saga.id, "ConfirmHandler compensated");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reservation::InMemoryReservationRepository;
    use crate::saga::{InMemorySagaRepository, SagaType};
    use uuid::Uuid;

    async fn make_orchestrator() -> SagaOrchestrator {
        let sagas: Arc<dyn SagaRepository> = Arc::new(InMemorySagaRepository::new());
        let reservations: Arc<dyn ReservationRepository> =
            Arc::new(InMemoryReservationRepository::new());
        SagaOrchestrator::new(
            sagas,
            reservations,
            vec![Arc::new(ReserveHandler), Arc::new(ConfirmHandler)],
        )
    }

    #[tokio::test]
    async fn execute_simple_saga_completes() {
        let orch = make_orchestrator().await;
        let mut saga = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k1".to_string(),
            vec!["reserve".to_string(), "confirm".to_string()],
        );
        // 给 step 1 加 resource_id
        saga.steps[0].resource_id = Some(Uuid::new_v4());

        orch.execute(&mut saga).await.unwrap();
        assert_eq!(saga.status, SagaStatus::Completed);
        assert_eq!(saga.steps[0].status, crate::saga::SagaStepStatus::Completed);
        assert_eq!(saga.steps[1].status, crate::saga::SagaStepStatus::Completed);
    }
}
