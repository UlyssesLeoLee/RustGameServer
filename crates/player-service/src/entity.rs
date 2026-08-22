//! player-service 域 entity 定义
//!
//! 54.1 占位：1 个简单 entity 演示 chrono / uuid / serde 集成。
//! 实际 entity 实施待 WF-1-54.6 domain entity + Repository trait。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// player-service 域根 entity（54.1 占位）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Player {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
