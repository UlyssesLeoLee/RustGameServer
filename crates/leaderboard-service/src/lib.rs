#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! leaderboard-service —— 卡牌游戏 4 类排行榜微服务业务骨架。
//!
//! 域职责：天梯 / 休闲 / 集换价值 3 类榜单 CRUD + 玩家位次查询。
//! 规范：RGS-REQ-038 §FR-007 + RGS-DTL-038 §3 (DEC-038-02 推荐 A 新域)。
//! DB：独立 leaderboard_db (per ARC-008 5 独立 DB 原则的卡牌游戏扩展)。
//! gRPC API：leaderboard/v1/leaderboard.proto (4 RPC + 1 内部 AddEntry)。
//!
//! 桶 12 实化：1 个核心 entity (LeaderboardEntry) + Repository trait +
//! PgRepository (sqlx impl) + InMemoryRepository (测用) + 4 RPC 业务方法 +
//! gRPC 桥接 + 内部 AddEntry。

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    InMemoryLeaderboardRepository, LeaderboardRepository, PgLeaderboardRepository,
};
pub use service::{LeaderboardDomainService, LeaderboardServiceImpl};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
