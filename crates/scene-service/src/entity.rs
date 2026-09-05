//! scene-service 域 entity 定义
//!
//! 7 域 scene-service 业务核心 entity (per 9/4 MD §2 + 9/5 改进路线图 Phase 2)
//! - Scene: 静态场景元数据 (master data)
//! - SceneInstance: 玩家实例化的场景 (transaction)
//! - MapUnit: 地图上的单位 (master + work)
//! - SpaceInfo: 玩家空间/签名 (master)
//!
//! 设计原则: Master / Transaction / Work 三分类 (per 9/1 18:30 JST 横展)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 坐标 (per 闪烁之光 proto_102.erl)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub dir: i32,
}

impl Position {
    /// 工厂: 新建坐标
    pub fn new(x: i32, y: i32, dir: i32) -> Self {
        Self { x, y, dir }
    }
}

/// 场景元数据 (Master data, per 9/1 18:30 JST DB 三分类)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub description: String,
    pub map_resource_id: String,
    pub max_players: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub scene_type: String,
    pub created_at: DateTime<Utc>,
}

impl Scene {
    /// 工厂: 新建场景
    pub fn new(id: String, name: String, map_resource_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: String::new(),
            map_resource_id,
            max_players: 100,
            min_level: 1,
            max_level: 999,
            scene_type: "normal".to_string(),
            created_at: now,
        }
    }

    /// 业务规则: 等级是否在范围内
    pub fn is_level_allowed(&self, level: i32) -> bool {
        level >= self.min_level && level <= self.max_level
    }
}

/// 场景实例 (Transaction, per 9/1 18:30 JST)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneInstance {
    pub id: Uuid,
    pub scene_id: String,
    pub owner_id: Uuid,
    pub player_count: i32,
    pub capacity: i32,
    pub status: InstanceStatus,
    pub server_node_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    Active,
    Loading,
    Closing,
    Closed,
}

impl SceneInstance {
    /// 工厂: 新建场景实例
    pub fn new(scene_id: String, owner_id: Uuid, capacity: i32, server_node_id: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            scene_id,
            owner_id,
            player_count: 1,
            capacity,
            status: InstanceStatus::Active,
            server_node_id,
            created_at: now,
        }
    }

    /// 业务规则: 是否已满
    pub fn is_full(&self) -> bool {
        self.player_count >= self.capacity
    }

    /// 业务规则: 加入玩家
    pub fn add_player(&mut self) {
        if !self.is_full() {
            self.player_count += 1;
        }
    }

    /// 业务规则: 离开玩家
    pub fn remove_player(&mut self) {
        if self.player_count > 0 {
            self.player_count -= 1;
        }
    }
}

/// 地图单位 (per 闪烁之光 proto_102.erl map_unit)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapUnit {
    pub id: Uuid,
    pub battle_id: i32,
    pub base_id: i32,
    pub name: String,
    pub status: i32,
    pub speed: i32,
    pub x: i32,
    pub y: i32,
    pub level: i32,
    pub scene_id: String,
}

impl MapUnit {
    /// 工厂: 新建单位
    pub fn new(scene_id: String, base_id: i32, name: String, x: i32, y: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            battle_id: 0,
            base_id,
            name,
            status: 0,
            speed: 100,
            x,
            y,
            level: 1,
            scene_id,
        }
    }

    /// 业务规则: 移动
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

/// 玩家空间 (per 闪烁之光 proto_103.erl 空间背景/签名)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpaceInfo {
    pub player_id: Uuid,
    pub background_id: String,
    pub sign: String,
    pub visits: i32,
    pub likes: i32,
}

impl SpaceInfo {
    /// 工厂: 新建空间
    pub fn new(player_id: Uuid) -> Self {
        Self {
            player_id,
            background_id: "default".to_string(),
            sign: String::new(),
            visits: 0,
            likes: 0,
        }
    }

    /// 业务规则: 更新签名 (业务校验 ≤ 50 字符)
    pub fn update_sign(&mut self, sign: String) -> crate::Result<()> {
        if sign.chars().count() > 50 {
            return Err(crate::Error::Validation(
                "sign must be <= 50 characters".to_string(),
            ));
        }
        self.sign = sign;
        Ok(())
    }

    /// 业务规则: 设置背景
    pub fn set_background(&mut self, background_id: String) {
        self.background_id = background_id;
    }

    /// 业务规则: 增加访问
    pub fn add_visit(&mut self) {
        self.visits += 1;
    }
}
