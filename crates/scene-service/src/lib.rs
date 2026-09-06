#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! scene-service —— 7 域场景微服务业务骨架。
//!
//! 域职责：场景切换、地图单位、移动事件、棋盘小游戏、场景任务、伙伴、剧情、阵法、副本、buff、空间签名。
//! 规范：闪烁之光借鉴 148 RPC（per 9/4 MD §2 + 9/5 改进路线图 Phase 2）。
//! 7 域独立 Lead（per 9/1 18:00 JST batch 域扩展 + 8/21 JST 5 域独立 Lead 原则）。
//! DB：独立 scene_db（per ARC-008 5 独立 DB 原则 + 7 域独立 Lead 扩展）。
//! gRPC API：scene/v1/scene.proto（148 RPC）。
//!
//! 设计原则：
//! - 数据驱动 + 反"一活动一模块"反例（per 9/4 MD §4）
//! - 不照搬闪烁之光 holiday_* 9 套重复，1 套框架 + 配置
//! - 移动同步用 (x, y) 简单坐标，Phase 3 完善 Geo hash

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    InMemoryMapUnitRepository, InMemorySceneInstanceRepository, InMemorySpaceRepository,
    MapUnitRepository, PageRequest, SceneInstanceRepository, SpaceRepository,
};

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
