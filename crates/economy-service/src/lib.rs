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

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
    PgAccountRepository, PgTransactionLedgerRepository, TransactionLedgerRepository,
};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
