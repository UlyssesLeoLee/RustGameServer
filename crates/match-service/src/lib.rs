#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! match-service —— 5 域匹配微服务业务骨架。
//!
//! 域职责：房间匹配、对战撮合、Match Slot Reservation、不可逆比赛结算。
//! 规范：RGS-REQ-016 / RGS-BAS-016 / RGS-DTL-016 / RGS-SPEC-DTL-016。
//! DB：独立 match_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：match/v1/match.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.6 实化：entity 2 个 + Repository trait + PgRepository sqlx impl + InMemoryRepository 测用。
//! 注意：`Match` 是 Rust 关键字，外部用 `r#Match` 引用。

pub mod entity;
pub mod error;
pub mod matchmaker;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    InMemoryMatchParticipantRepository, InMemoryMatchRepository, MatchParticipantRepository,
    MatchRepository, PgMatchParticipantRepository, PgMatchRepository,
};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
