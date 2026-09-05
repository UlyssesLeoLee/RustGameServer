#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! player-service —— 5 域玩家微服务业务骨架。
//!
//! 域职责：账号生命周期、角色档案、好友关系、跨服身份 (active-active 模式)。
//! 规范：RGS-REQ-018 / RGS-BAS-018 / RGS-DTL-018 / RGS-SPEC-DTL-018。
//! DB：独立 player_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：player/v1/player.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.1 骨架：4 子模块（error / service / repository / entity）；
//! 54.6 实化：entity 2 个 + Repository trait + PgRepository sqlx impl + InMemoryRepository 测用。

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    CharacterRepository, DeckRepository, InMemoryCharacterRepository, InMemoryDeckRepository,
    InMemoryPlayerRepository, InMemoryPlayerSessionRepository, Page, PageRequest,
    PgCharacterRepository, PgDeckRepository, PgPlayerRepository, PgPlayerSessionRepository,
    PlayerRepository, PlayerSessionRepository,
};

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod db;
