//! scene-service 域 Repository
//!
//! 7 域数据访问层 (per 9/1 18:30 JST Master/Transaction/Work 三分类)
//! - SceneInstanceRepository: 场景实例 (Transaction)
//! - MapUnitRepository: 地图单位 (Work + Master)
//! - SpaceRepository: 玩家空间 (Master)
//!
//! 设计原则:
//! - trait 抽象数据访问
//! - InMemoryRepository: 单测用, 验证 trait 行为一致性
//! - PgRepository: 留待 Phase 3 实装 (per L1 DoD cargo check 0 error)
//!
//! 注: 本次 scaffold 仅 InMemory, 满足 L1 (cargo check --tests) 0 error 目标。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{MapUnit, SceneInstance, SpaceInfo};
use crate::Result;

/// 分页请求 (per common.proto PageRequest)
#[derive(Debug, Clone, Copy)]
pub struct PageRequest {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// 场景实例 Repository trait
#[async_trait]
pub trait SceneInstanceRepository: Send + Sync {
    /// 创建场景实例
    async fn create(&self, instance: &SceneInstance) -> Result<SceneInstance>;
    /// 按 id 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SceneInstance>>;
    /// 保存 (insert / update)
    async fn save(&self, instance: &SceneInstance) -> Result<SceneInstance>;
    /// 按 id 删除
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 列出某玩家参与的所有场景实例
    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<SceneInstance>>;
    /// 列出某场景的所有活跃实例
    async fn list_active_by_scene(&self, scene_id: &str) -> Result<Vec<SceneInstance>>;
}

/// 地图单位 Repository trait
#[async_trait]
pub trait MapUnitRepository: Send + Sync {
    /// 创建单位
    async fn create(&self, unit: &MapUnit) -> Result<MapUnit>;
    /// 按 id 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MapUnit>>;
    /// 保存 (insert / update)
    async fn save(&self, unit: &MapUnit) -> Result<MapUnit>;
    /// 按 id 删除
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 列出某场景所有单位
    async fn list_by_scene(&self, scene_id: &str) -> Result<Vec<MapUnit>>;
    /// 按 base_id (单位类型) 列出
    async fn list_by_base_id(&self, scene_id: &str, base_id: i32) -> Result<Vec<MapUnit>>;
}

/// 玩家空间 Repository trait
#[async_trait]
pub trait SpaceRepository: Send + Sync {
    /// 创建
    async fn create(&self, info: &SpaceInfo) -> Result<SpaceInfo>;
    /// 按 player_id 查询
    async fn find_by_player(&self, player_id: Uuid) -> Result<Option<SpaceInfo>>;
    /// 保存
    async fn save(&self, info: &SpaceInfo) -> Result<SpaceInfo>;
}

// ============================================================================
// InMemory 实现 (单测用, per 5 域 InMemory 模式)
// ============================================================================

/// SceneInstance InMemory 实现
pub struct InMemorySceneInstanceRepository {
    store: Mutex<HashMap<Uuid, SceneInstance>>,
}

impl InMemorySceneInstanceRepository {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySceneInstanceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SceneInstanceRepository for InMemorySceneInstanceRepository {
    async fn create(&self, instance: &SceneInstance) -> Result<SceneInstance> {
        let mut store = self.store.lock().expect("scene store poisoned");
        store.insert(instance.id, instance.clone());
        Ok(instance.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SceneInstance>> {
        let store = self.store.lock().expect("scene store poisoned");
        Ok(store.get(&id).cloned())
    }

    async fn save(&self, instance: &SceneInstance) -> Result<SceneInstance> {
        let mut store = self.store.lock().expect("scene store poisoned");
        store.insert(instance.id, instance.clone());
        Ok(instance.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let mut store = self.store.lock().expect("scene store poisoned");
        Ok(store.remove(&id).is_some())
    }

    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<SceneInstance>> {
        let store = self.store.lock().expect("scene store poisoned");
        Ok(store
            .values()
            .filter(|i| i.owner_id == owner_id)
            .cloned()
            .collect())
    }

    async fn list_active_by_scene(&self, scene_id: &str) -> Result<Vec<SceneInstance>> {
        let store = self.store.lock().expect("scene store poisoned");
        Ok(store
            .values()
            .filter(|i| i.scene_id == scene_id)
            .cloned()
            .collect())
    }
}

/// MapUnit InMemory 实现
pub struct InMemoryMapUnitRepository {
    store: Mutex<HashMap<Uuid, MapUnit>>,
}

impl InMemoryMapUnitRepository {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryMapUnitRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MapUnitRepository for InMemoryMapUnitRepository {
    async fn create(&self, unit: &MapUnit) -> Result<MapUnit> {
        let mut store = self.store.lock().expect("unit store poisoned");
        store.insert(unit.id, unit.clone());
        Ok(unit.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<MapUnit>> {
        let store = self.store.lock().expect("unit store poisoned");
        Ok(store.get(&id).cloned())
    }

    async fn save(&self, unit: &MapUnit) -> Result<MapUnit> {
        let mut store = self.store.lock().expect("unit store poisoned");
        store.insert(unit.id, unit.clone());
        Ok(unit.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let mut store = self.store.lock().expect("unit store poisoned");
        Ok(store.remove(&id).is_some())
    }

    async fn list_by_scene(&self, scene_id: &str) -> Result<Vec<MapUnit>> {
        let store = self.store.lock().expect("unit store poisoned");
        Ok(store
            .values()
            .filter(|u| u.scene_id == scene_id)
            .cloned()
            .collect())
    }

    async fn list_by_base_id(&self, scene_id: &str, base_id: i32) -> Result<Vec<MapUnit>> {
        let store = self.store.lock().expect("unit store poisoned");
        Ok(store
            .values()
            .filter(|u| u.scene_id == scene_id && u.base_id == base_id)
            .cloned()
            .collect())
    }
}

/// SpaceInfo InMemory 实现
pub struct InMemorySpaceRepository {
    store: Mutex<HashMap<Uuid, SpaceInfo>>,
}

impl InMemorySpaceRepository {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySpaceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SpaceRepository for InMemorySpaceRepository {
    async fn create(&self, info: &SpaceInfo) -> Result<SpaceInfo> {
        let mut store = self.store.lock().expect("space store poisoned");
        store.insert(info.player_id, info.clone());
        Ok(info.clone())
    }

    async fn find_by_player(&self, player_id: Uuid) -> Result<Option<SpaceInfo>> {
        let store = self.store.lock().expect("space store poisoned");
        Ok(store.get(&player_id).cloned())
    }

    async fn save(&self, info: &SpaceInfo) -> Result<SpaceInfo> {
        let mut store = self.store.lock().expect("space store poisoned");
        store.insert(info.player_id, info.clone());
        Ok(info.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{InstanceStatus, Position, Scene, SceneInstance};

    // ---- Scene entity 业务规则测试 ----

    #[test]
    fn scene_is_level_allowed() {
        let s = Scene::new("scene-1".to_string(), "新手村".to_string(), "res-0".to_string());
        assert!(s.is_level_allowed(1));
        assert!(s.is_level_allowed(50));
    }

    #[test]
    fn scene_level_range() {
        let mut s = Scene::new("scene-1".to_string(), "新手村".to_string(), "res-0".to_string());
        s.min_level = 10;
        s.max_level = 20;
        assert!(!s.is_level_allowed(5));
        assert!(s.is_level_allowed(15));
        assert!(!s.is_level_allowed(25));
    }

    // ---- Position ----

    #[test]
    fn position_new() {
        let p = Position::new(10, 20, 1);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(p.dir, 1);
    }

    // ---- SceneInstance 业务规则测试 ----

    #[test]
    fn scene_instance_is_full() {
        let mut inst = SceneInstance::new("s".to_string(), Uuid::new_v4(), 2, 1);
        assert!(!inst.is_full());
        inst.add_player();
        assert!(inst.is_full());
        inst.add_player(); // 不应超容量
        assert_eq!(inst.player_count, 2);
    }

    #[test]
    fn scene_instance_remove_player() {
        let mut inst = SceneInstance::new("s".to_string(), Uuid::new_v4(), 5, 1);
        inst.add_player();
        inst.add_player();
        inst.add_player();
        assert_eq!(inst.player_count, 4);
        inst.remove_player();
        assert_eq!(inst.player_count, 3);
        inst.remove_player();
        inst.remove_player();
        inst.remove_player();
        inst.remove_player(); // 不应负数
        assert_eq!(inst.player_count, 0);
    }

    #[test]
    fn scene_instance_status_default_active() {
        let inst = SceneInstance::new("s".to_string(), Uuid::new_v4(), 5, 1);
        assert_eq!(inst.status, InstanceStatus::Active);
    }

    // ---- MapUnit 业务规则测试 ----

    #[test]
    fn map_unit_move_to() {
        let mut u = MapUnit::new("scene-1".to_string(), 100, "npc-1".to_string(), 0, 0);
        u.move_to(50, 60);
        assert_eq!(u.x, 50);
        assert_eq!(u.y, 60);
    }

    // ---- SpaceInfo 业务规则测试 ----

    #[test]
    fn space_info_update_sign_ok() {
        let mut s = SpaceInfo::new(Uuid::new_v4());
        s.update_sign("hello world".to_string()).unwrap();
        assert_eq!(s.sign, "hello world");
    }

    #[test]
    fn space_info_update_sign_too_long() {
        let mut s = SpaceInfo::new(Uuid::new_v4());
        let long = "a".repeat(60);
        let err = s.update_sign(long).unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    #[test]
    fn space_info_set_background() {
        let mut s = SpaceInfo::new(Uuid::new_v4());
        s.set_background("bg-7".to_string());
        assert_eq!(s.background_id, "bg-7");
    }

    #[test]
    fn space_info_add_visit() {
        let mut s = SpaceInfo::new(Uuid::new_v4());
        s.add_visit();
        s.add_visit();
        s.add_visit();
        assert_eq!(s.visits, 3);
    }

    // ---- InMemorySceneInstanceRepository ----

    #[tokio::test]
    async fn in_memory_scene_instance_create_find() {
        let repo = InMemorySceneInstanceRepository::new();
        let inst = SceneInstance::new("scene-1".to_string(), Uuid::new_v4(), 5, 1);
        let id = inst.id;
        repo.create(&inst).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.id, inst.id);
        assert_eq!(found.scene_id, "scene-1");
    }

    #[tokio::test]
    async fn in_memory_scene_instance_delete() {
        let repo = InMemorySceneInstanceRepository::new();
        let inst = SceneInstance::new("s".to_string(), Uuid::new_v4(), 5, 1);
        let id = inst.id;
        repo.create(&inst).await.unwrap();
        assert!(repo.delete_by_id(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_scene_instance_list_by_owner() {
        let repo = InMemorySceneInstanceRepository::new();
        let owner = Uuid::new_v4();
        let i1 = SceneInstance::new("s1".to_string(), owner, 5, 1);
        let i2 = SceneInstance::new("s2".to_string(), owner, 5, 1);
        let other = SceneInstance::new("s3".to_string(), Uuid::new_v4(), 5, 1);
        repo.create(&i1).await.unwrap();
        repo.create(&i2).await.unwrap();
        repo.create(&other).await.unwrap();
        let list = repo.list_by_owner(owner).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_scene_instance_list_active_by_scene() {
        let repo = InMemorySceneInstanceRepository::new();
        repo.create(&SceneInstance::new("s1".to_string(), Uuid::new_v4(), 5, 1))
            .await
            .unwrap();
        repo.create(&SceneInstance::new("s1".to_string(), Uuid::new_v4(), 5, 1))
            .await
            .unwrap();
        repo.create(&SceneInstance::new("s2".to_string(), Uuid::new_v4(), 5, 1))
            .await
            .unwrap();
        let list = repo.list_active_by_scene("s1").await.unwrap();
        assert_eq!(list.len(), 2);
    }

    // ---- InMemoryMapUnitRepository ----

    #[tokio::test]
    async fn in_memory_map_unit_create_find() {
        let repo = InMemoryMapUnitRepository::new();
        let u = MapUnit::new("scene-1".to_string(), 100, "npc-1".to_string(), 10, 20);
        let id = u.id;
        repo.create(&u).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.id, u.id);
        assert_eq!(found.name, "npc-1");
    }

    #[tokio::test]
    async fn in_memory_map_unit_list_by_scene() {
        let repo = InMemoryMapUnitRepository::new();
        let u1 = MapUnit::new("scene-1".to_string(), 100, "npc-1".to_string(), 0, 0);
        let u2 = MapUnit::new("scene-1".to_string(), 200, "npc-2".to_string(), 5, 5);
        let u3 = MapUnit::new("scene-2".to_string(), 100, "npc-3".to_string(), 0, 0);
        repo.create(&u1).await.unwrap();
        repo.create(&u2).await.unwrap();
        repo.create(&u3).await.unwrap();
        let list = repo.list_by_scene("scene-1").await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_map_unit_list_by_base_id() {
        let repo = InMemoryMapUnitRepository::new();
        let u1 = MapUnit::new("scene-1".to_string(), 100, "npc-1".to_string(), 0, 0);
        let u2 = MapUnit::new("scene-1".to_string(), 100, "npc-2".to_string(), 5, 5);
        let u3 = MapUnit::new("scene-1".to_string(), 200, "mon-1".to_string(), 0, 0);
        repo.create(&u1).await.unwrap();
        repo.create(&u2).await.unwrap();
        repo.create(&u3).await.unwrap();
        let list = repo.list_by_base_id("scene-1", 100).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_map_unit_save_update() {
        let repo = InMemoryMapUnitRepository::new();
        let mut u = MapUnit::new("scene-1".to_string(), 100, "npc-1".to_string(), 0, 0);
        repo.create(&u).await.unwrap();
        u.move_to(99, 88);
        repo.save(&u).await.unwrap();
        let found = repo.find_by_id(u.id).await.unwrap().unwrap();
        assert_eq!(found.x, 99);
        assert_eq!(found.y, 88);
    }

    #[tokio::test]
    async fn in_memory_map_unit_delete() {
        let repo = InMemoryMapUnitRepository::new();
        let u = MapUnit::new("scene-1".to_string(), 100, "npc-1".to_string(), 0, 0);
        let id = u.id;
        repo.create(&u).await.unwrap();
        assert!(repo.delete_by_id(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }

    // ---- InMemorySpaceRepository ----

    #[tokio::test]
    async fn in_memory_space_create_find() {
        let repo = InMemorySpaceRepository::new();
        let player_id = Uuid::new_v4();
        let s = SpaceInfo::new(player_id);
        repo.create(&s).await.unwrap();
        let found = repo.find_by_player(player_id).await.unwrap().unwrap();
        assert_eq!(found.player_id, player_id);
        assert_eq!(found.background_id, "default");
    }

    #[tokio::test]
    async fn in_memory_space_save_update() {
        let repo = InMemorySpaceRepository::new();
        let player_id = Uuid::new_v4();
        let mut s = SpaceInfo::new(player_id);
        repo.create(&s).await.unwrap();
        s.set_background("bg-100".to_string());
        s.update_sign("welcome".to_string()).unwrap();
        repo.save(&s).await.unwrap();
        let found = repo.find_by_player(player_id).await.unwrap().unwrap();
        assert_eq!(found.background_id, "bg-100");
        assert_eq!(found.sign, "welcome");
    }
}
