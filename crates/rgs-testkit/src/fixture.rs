//! Sample data 工厂 + DB init/teardown
//!
//! - `player()`  生成 sample player
//! - `economy()` 生成 sample economy balance
//! - `saga()`    生成 sample saga 上下文
//! - `init_test_db(name)` 创建临时 DB（占位，53.3 仅声明）

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerFixture {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EconomyFixture {
    pub player_id: String,
    pub currency: i64,
    pub gold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SagaFixture {
    pub saga_id: String,
    pub saga_type: String,
    pub step: u32,
    pub state: String,
}

/// 生成 sample player（53.3 占位：固定值）
pub fn player() -> PlayerFixture {
    PlayerFixture {
        id: "player-test-001".to_string(),
        name: "Test Player".to_string(),
        level: 1,
        created_at: chrono::Utc::now(),
    }
}

/// 生成 sample economy balance
pub fn economy(player_id: &str) -> EconomyFixture {
    EconomyFixture {
        player_id: player_id.to_string(),
        currency: 1000,
        gold: 50,
    }
}

/// 生成 sample saga 上下文
pub fn saga(saga_type: &str) -> SagaFixture {
    SagaFixture {
        saga_id: format!("saga-test-{}", uuid::Uuid::new_v4()),
        saga_type: saga_type.to_string(),
        step: 0,
        state: "Pending".to_string(),
    }
}

/// 测试 DB 初始化（53.3 占位 trait；54.x 接入 testcontainers-rs）
pub async fn init_test_db(name: &str) -> anyhow::Result<String> {
    Ok(format!("postgres://test:test@localhost:5432/{}", name))
}
