#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! economy-service —— 5 域经济微服务业务骨架。
//!
//! 域职责：货币、物品、商店、跨服转账、Reservation & Compensation (Q-003 Saga 关键能力)。
//! 规范：RGS-REQ-015 / RGS-BAS-015 / RGS-DTL-015 / RGS-SPEC-DTL-015 / RGS-DTL-100 Saga。
//! DB：独立 economy_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：economy/v1/economy.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.6 实化：entity 2 个 + Repository trait + PgRepository sqlx impl + InMemoryRepository 测用。
//! 54.8 实化：Saga 事务系统（saga / reservation / inbox + orchestrator）。

pub mod entity;
pub mod error;
pub mod inbox;
pub mod repository;
pub mod reservation;
pub mod saga;
pub mod saga_orchestrator;
pub mod service;
// 卡牌 8 桶 / 子桶 1: trade 域 (per RGS-DTL-038 §4.4 + DEC-038-04 + 9 DEC 全 A 拍板)
pub mod trade_entity;
pub mod trade_repository;
pub mod trade_service;
// W36 跨域 1/3 步收尾: trade 跨域 saga (per RGS-DTL-038 §6 + DEC-038-04)
pub mod trade_saga;
pub mod trade_saga_clients;

pub use error::{Error, Result};
pub use inbox::{InboxEntry, InboxRepository, InboxStatus, PgInboxRepository};
pub use repository::{
    AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
    PgAccountRepository, PgTransactionLedgerRepository, TransactionLedgerRepository,
};
pub use reservation::{
    InMemoryReservationRepository, PgReservationRepository, Reservation, ReservationRepository,
    ReservationStatus,
};
pub use saga::{
    InMemorySagaRepository, PgSagaRepository, Saga, SagaRepository, SagaStatus, SagaStep,
    SagaStepStatus, SagaType,
};
pub use saga_orchestrator::{ConfirmHandler, ReserveHandler, SagaOrchestrator, SagaStepHandler};
pub use trade_entity::{Auction, AuctionFilter, AuctionStatus, PrivateTrade, PrivateTradeStatus};
pub use trade_repository::{InMemoryTradeRepository, PgTradeRepository, TradeRepository};
pub use trade_service::{ExecuteTradeServiceImpl, TradeService, TradeServiceImpl};
// W36 跨域 saga 导出 (per RGS-DTL-038 §6)
pub use trade_saga::{
    BidAuctionInput, BidAuctionOutput, BidAuctionSaga, ExecuteAuctionInput, ExecuteAuctionOutput,
    ExecuteAuctionSaga, OpenPackInput, OpenPackOutput, OpenPackSaga,
};
pub use trade_saga_clients::{
    AuctionLockState, CardClient, CardGrpcClient, CardSource, MockCardClient, MockTradeClient,
    TradeClient,
};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
