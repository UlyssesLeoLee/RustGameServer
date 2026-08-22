//! economy-service Saga 事务系统（per RGS-DTL-100 Saga Q-003）
//!
//! 54.8 实化：Saga entity + SagaStateMachine + SagaRepository trait + Pg/InMemory impl
//!
//! 设计原则（per RGS-DTL-100 §3-§6）：
//! - Saga = 长事务，由多个 step 组成
//! - 步进式执行：每步成功才进入下一步
//! - 任一步失败 → 反向执行补偿 step（Compensation）
//! - 状态机持久化在 saga 表，支持崩溃恢复
//! - Reservation 模式：动 balance 前先 reserve，confirm 才真扣；compensate 即释放
//! - Inbox 模式：command_id 幂等，防重复处理（per RGS-DTL-100 §6 幂等性）

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::Result;

/// Saga 类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SagaType {
    /// 转账（A 账户 → B 账户）
    Transfer,
    /// 每日奖励
    DailyReward,
    /// 商城购买（货币 → 物品）
    Purchase,
}

/// Saga 整体状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SagaStatus {
    /// 待启动
    Pending,
    /// 执行中
    Running,
    /// 补偿中
    Compensating,
    /// 已完成
    Completed,
    /// 失败（补偿完成后无法挽回）
    Failed,
    /// 已中止
    Aborted,
}

/// 单步状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SagaStepStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已补偿
    Compensated,
}

/// Saga 步骤
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SagaStep {
    /// 步骤名
    pub name: String,
    /// 状态
    pub status: SagaStepStatus,
    /// 关联资源 ID（可选，如 account_id）
    pub resource_id: Option<Uuid>,
    /// 错误信息
    pub error: Option<String>,
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
}

impl SagaStep {
    /// 工厂：新建待执行步骤
    pub fn new(name: String, resource_id: Option<Uuid>) -> Self {
        Self {
            name,
            status: SagaStepStatus::Pending,
            resource_id,
            error: None,
            started_at: None,
            completed_at: None,
        }
    }

    /// 标记运行中
    pub fn mark_running(&mut self) {
        self.status = SagaStepStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// 标记完成
    pub fn mark_completed(&mut self) {
        self.status = SagaStepStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// 标记失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = SagaStepStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }

    /// 标记已补偿
    pub fn mark_compensated(&mut self) {
        self.status = SagaStepStatus::Compensated;
        self.completed_at = Some(Utc::now());
    }
}

/// Saga 实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Saga {
    /// Saga ID
    pub id: Uuid,
    /// 类型
    pub saga_type: SagaType,
    /// 触发 command_id（per RGS-DTL-100 §6 幂等性）
    pub command_id: Uuid,
    /// 业务幂等键
    pub idempotency_key: String,
    /// 当前步骤索引
    pub current_step: usize,
    /// 步骤列表
    pub steps: Vec<SagaStep>,
    /// 状态
    pub status: SagaStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
}

impl Saga {
    /// 工厂：新建 Saga（Pending）
    pub fn new(
        saga_type: SagaType,
        command_id: Uuid,
        idempotency_key: String,
        step_names: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        let steps = step_names
            .into_iter()
            .map(SagaStep::new_no_resource)
            .collect();
        Self {
            id: Uuid::new_v4(),
            saga_type,
            command_id,
            idempotency_key,
            current_step: 0,
            steps,
            status: SagaStatus::Pending,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// 当前步骤
    pub fn current(&self) -> Option<&SagaStep> {
        self.steps.get(self.current_step)
    }

    /// 当前步骤可变引用
    pub fn current_mut(&mut self) -> Option<&mut SagaStep> {
        self.steps.get_mut(self.current_step)
    }

    /// 推进到下一步
    pub fn advance(&mut self) -> bool {
        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// 开始执行
    pub fn start(&mut self) {
        self.status = SagaStatus::Running;
        if let Some(step) = self.current_mut() {
            step.mark_running();
        }
        self.updated_at = Utc::now();
    }

    /// 标记完成
    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = self.completed_at.unwrap();
    }

    /// 触发补偿（per RGS-DTL-100 §4 补偿模式）
    pub fn compensate(&mut self) {
        self.status = SagaStatus::Compensating;
        // 反向遍历已完成步骤执行补偿
        for step in self.steps.iter_mut().rev() {
            if step.status == SagaStepStatus::Completed {
                step.mark_compensated();
            }
        }
        self.updated_at = Utc::now();
    }

    /// 失败（无法挽回）
    pub fn fail(&mut self) {
        self.status = SagaStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.updated_at = self.completed_at.unwrap();
    }
}

// SagaStep 私有扩展
trait SagaStepNewNoResource {
    fn new_no_resource(name: String) -> SagaStep;
}

impl SagaStepNewNoResource for SagaStep {
    fn new_no_resource(name: String) -> SagaStep {
        SagaStep::new(name, None)
    }
}

/// Saga Repository trait
#[async_trait]
pub trait SagaRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Saga>>;
    /// 按 command_id 查（per RGS-DTL-100 §6 幂等性）
    async fn find_by_command_id(&self, command_id: Uuid) -> Result<Option<Saga>>;
    async fn save(&self, entity: &Saga) -> Result<Saga>;
    /// 列出待恢复的 Running 状态 Saga（崩溃恢复用）
    async fn list_running(&self, limit: i64) -> Result<Vec<Saga>>;
}

// ============================================================================
// PgRepository（sqlx 实现）
// ============================================================================

pub struct PgSagaRepository {
    pool: PgPool,
}

impl PgSagaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// 持久化 Saga 需要把 steps 序列化为 JSON
fn serialize_steps(steps: &[SagaStep]) -> serde_json::Value {
    serde_json::to_value(steps).unwrap_or(serde_json::json!([]))
}

fn deserialize_steps(value: serde_json::Value) -> Vec<SagaStep> {
    serde_json::from_value(value).unwrap_or_default()
}

fn saga_type_to_str(t: SagaType) -> &'static str {
    match t {
        SagaType::Transfer => "transfer",
        SagaType::DailyReward => "daily_reward",
        SagaType::Purchase => "purchase",
    }
}

fn parse_saga_type(s: &str) -> SagaType {
    match s {
        "transfer" => SagaType::Transfer,
        "purchase" => SagaType::Purchase,
        _ => SagaType::DailyReward,
    }
}

fn saga_status_to_str(s: SagaStatus) -> &'static str {
    match s {
        SagaStatus::Pending => "pending",
        SagaStatus::Running => "running",
        SagaStatus::Compensating => "compensating",
        SagaStatus::Completed => "completed",
        SagaStatus::Failed => "failed",
        SagaStatus::Aborted => "aborted",
    }
}

fn parse_saga_status(s: &str) -> SagaStatus {
    match s {
        "pending" => SagaStatus::Pending,
        "running" => SagaStatus::Running,
        "compensating" => SagaStatus::Compensating,
        "completed" => SagaStatus::Completed,
        "aborted" => SagaStatus::Aborted,
        _ => SagaStatus::Failed,
    }
}

#[async_trait]
impl SagaRepository for PgSagaRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Saga>> {
        let row = sqlx::query(
            "SELECT id, saga_type, command_id, idempotency_key, current_step, steps, status, created_at, updated_at, completed_at \
             FROM sagas WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            let steps_json: serde_json::Value = r.get("steps");
            Saga {
                id: r.get("id"),
                saga_type: parse_saga_type(&r.get::<String, _>("saga_type")),
                command_id: r.get("command_id"),
                idempotency_key: r.get("idempotency_key"),
                current_step: r.get::<i32, _>("current_step") as usize,
                steps: deserialize_steps(steps_json),
                status: parse_saga_status(&r.get::<String, _>("status")),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }
        }))
    }

    async fn find_by_command_id(&self, command_id: Uuid) -> Result<Option<Saga>> {
        let row = sqlx::query(
            "SELECT id, saga_type, command_id, idempotency_key, current_step, steps, status, created_at, updated_at, completed_at \
             FROM sagas WHERE command_id = $1",
        )
        .bind(command_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            let steps_json: serde_json::Value = r.get("steps");
            Saga {
                id: r.get("id"),
                saga_type: parse_saga_type(&r.get::<String, _>("saga_type")),
                command_id: r.get("command_id"),
                idempotency_key: r.get("idempotency_key"),
                current_step: r.get::<i32, _>("current_step") as usize,
                steps: deserialize_steps(steps_json),
                status: parse_saga_status(&r.get::<String, _>("status")),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }
        }))
    }

    async fn save(&self, entity: &Saga) -> Result<Saga> {
        let steps_json = serialize_steps(&entity.steps);
        sqlx::query(
            "INSERT INTO sagas \
             (id, saga_type, command_id, idempotency_key, current_step, steps, status, created_at, updated_at, completed_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET \
                current_step = EXCLUDED.current_step, steps = EXCLUDED.steps, \
                status = EXCLUDED.status, updated_at = EXCLUDED.updated_at, \
                completed_at = EXCLUDED.completed_at",
        )
        .bind(entity.id)
        .bind(saga_type_to_str(entity.saga_type))
        .bind(entity.command_id)
        .bind(&entity.idempotency_key)
        .bind(entity.current_step as i32)
        .bind(steps_json)
        .bind(saga_status_to_str(entity.status))
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .bind(entity.completed_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn list_running(&self, limit: i64) -> Result<Vec<Saga>> {
        let rows = sqlx::query(
            "SELECT id, saga_type, command_id, idempotency_key, current_step, steps, status, created_at, updated_at, completed_at \
             FROM sagas WHERE status IN ('running', 'compensating') ORDER BY updated_at LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let steps_json: serde_json::Value = r.get("steps");
                Saga {
                    id: r.get("id"),
                    saga_type: parse_saga_type(&r.get::<String, _>("saga_type")),
                    command_id: r.get("command_id"),
                    idempotency_key: r.get("idempotency_key"),
                    current_step: r.get::<i32, _>("current_step") as usize,
                    steps: deserialize_steps(steps_json),
                    status: parse_saga_status(&r.get::<String, _>("status")),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    completed_at: r.get("completed_at"),
                }
            })
            .collect())
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemorySagaRepository {
    inner: Mutex<HashMap<Uuid, Saga>>,
}

impl InMemorySagaRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySagaRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SagaRepository for InMemorySagaRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Saga>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_command_id(&self, command_id: Uuid) -> Result<Option<Saga>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|s| s.command_id == command_id)
            .cloned())
    }
    async fn save(&self, entity: &Saga) -> Result<Saga> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn list_running(&self, limit: i64) -> Result<Vec<Saga>> {
        let mut running: Vec<Saga> = self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|s| matches!(s.status, SagaStatus::Running | SagaStatus::Compensating))
            .cloned()
            .collect();
        running.sort_by_key(|s| s.updated_at);
        running.truncate(limit as usize);
        Ok(running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saga_lifecycle() {
        let mut s = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k1".to_string(),
            vec![
                "reserve".to_string(),
                "transfer".to_string(),
                "confirm".to_string(),
            ],
        );
        assert_eq!(s.status, SagaStatus::Pending);
        s.start();
        assert_eq!(s.status, SagaStatus::Running);
        assert_eq!(s.current().unwrap().name, "reserve");
        assert_eq!(s.current().unwrap().status, SagaStepStatus::Running);

        s.current_mut().unwrap().mark_completed();
        s.advance();
        s.current_mut().unwrap().mark_failed("network".to_string());
        s.compensate();
        assert_eq!(s.status, SagaStatus::Compensating);
        // 第一个 step 已补偿
        assert_eq!(s.steps[0].status, SagaStepStatus::Compensated);
        // 第二个 step 失败但没补偿
        assert_eq!(s.steps[1].status, SagaStepStatus::Failed);
    }

    #[tokio::test]
    async fn in_memory_saga_save_and_find() {
        let repo = InMemorySagaRepository::new();
        let s = Saga::new(
            SagaType::DailyReward,
            Uuid::new_v4(),
            "k1".to_string(),
            vec!["check".to_string(), "grant".to_string()],
        );
        let id = s.id;
        repo.save(&s).await.unwrap();
        let loaded = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(loaded.saga_type, SagaType::DailyReward);
    }

    #[tokio::test]
    async fn in_memory_saga_find_by_command_id() {
        let repo = InMemorySagaRepository::new();
        let cmd_id = Uuid::new_v4();
        let s = Saga::new(
            SagaType::Transfer,
            cmd_id,
            "k1".to_string(),
            vec!["a".to_string()],
        );
        repo.save(&s).await.unwrap();
        let found = repo.find_by_command_id(cmd_id).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn in_memory_saga_list_running() {
        let repo = InMemorySagaRepository::new();
        let mut s1 = Saga::new(
            SagaType::Purchase,
            Uuid::new_v4(),
            "k1".to_string(),
            vec!["x".to_string()],
        );
        s1.start();
        repo.save(&s1).await.unwrap();

        let mut s2 = Saga::new(
            SagaType::Transfer,
            Uuid::new_v4(),
            "k2".to_string(),
            vec!["y".to_string()],
        );
        s2.start();
        s2.complete();
        repo.save(&s2).await.unwrap();

        let running = repo.list_running(10).await.unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, s1.id);
    }
}
