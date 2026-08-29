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
//!
//! v2 卡牌游戏 session/turn 抽象 (per RGS-DTL-038 §4.2 + §5):
//! - `entity_v2` — GameSession/Move/Board + 状态机
//! - `matchmaker_v2` — 9 RPC 业务实装
//! - `repository_v2` — 3 个 Repository trait + Pg + InMemory

pub mod entity;
pub mod error;
pub mod matchmaker;
pub mod repository;
pub mod service;

// v2 卡牌游戏适配 (per RGS-DTL-038 §4.2 + §5 + §7.1)
pub mod entity_v2;
pub mod matchmaker_v2;
pub mod repository_v2;

pub use error::{Error, Result};
pub use repository::{
    InMemoryMatchParticipantRepository, InMemoryMatchRepository, MatchParticipantRepository,
    MatchRepository, PgMatchParticipantRepository, PgMatchRepository,
};

// v2 导出 (桶 9 补完: service.rs 通过这些 module 接入 matchmaker_v2)
pub use entity_v2::{
    Board, BoardUnit, GameMode, GameSession, MatchmakingTicket, Move, MoveType, SessionPlayer,
    SessionStatus,
};
pub use matchmaker_v2::{
    CreateMatchResult, EnqueueResult, EventBus, JoinMatchResult, LeaveMatchResult, MatchEvent,
    MatchState, MatchmakingStatus, MatchmakerServiceV2, SubmitMoveResult, TicketStatus,
};
pub use repository_v2::{
    GameSessionRepository, InMemoryGameSessionRepository, InMemoryMatchmakingTicketRepository,
    InMemoryMoveRepository, MatchmakingTicketRepository, MoveRepository, PgGameSessionRepository,
    PgMatchmakingTicketRepository, PgMoveRepository,
};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
