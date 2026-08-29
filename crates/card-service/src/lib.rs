//! card-service —— 卡牌游戏新域微服务 (per RGS-DTL-038 §4.4 + §3 DEC-038-01~09)
//!
//! 域职责 (per DEC-038-01 推荐 A): catalog (静态) + collection (动态) + 抽卡
//! - 卡组归 player-service (DEC-038-01), card-service 不承担 deck
//! - leaderboard 独立 (DEC-038-02) — 本域不含
//! - trade 归 economy (DEC-038-04) — 本域不含
//! - replay 归 cluster-ops 对象存储 (DEC-038-03) — 本域不含
//!
//! 桶 7 (proto 设计) + 桶 8 (proto 实装): 仅编译 proto + 暴露公共类型
//! 桶 10 (card catalog, 本次实装): 完整化 entity / error / repository / service / db
//!   - 5 entity: Card / CardSeries / CardInstance / DropTable / DropEntry
//!   - Pg + InMemory 双实现 repository
//!   - 10 RPC handler 完整业务逻辑 (HealthCheck / GetCard / ListCards /
//!     GetCardSeries / ListCardSeries / GetPlayerCollection / AddCardToCollection /
//!     RemoveCardFromCollection / OpenPack / [saga 占位])
//!   - 3 张表 migration (per DTL-038 §7.1 #1-3): cards / card_series / card_instances
//!   - 抽卡概率公开 (per DEC-038-06 强制, OpenPackResponse 含 DropTable snapshot)
//!   - 业务层只做数据流, 规则引擎 TODO (per DTL-038 §9.1 P2)
//!
//! DB: 独立 card_db（per ARC-008 5 独立 DB 原则, 卡牌游戏专属 6 域沿用同原则）
//! gRPC API: card/v1/card.proto（per WF-1-54.2 + WBS v0.5 桶 7 + 桶 8）

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;
pub mod db;

pub use error::{Error, Result};
pub use repository::{
    CardFilter, CardInstanceFilter, CardInstanceRepository, CardRepository, CardSeriesRepository,
    InMemoryCardInstanceRepository, InMemoryCardRepository, InMemoryCardSeriesRepository, Page,
    PageRequest, PgCardInstanceRepository, PgCardRepository, PgCardSeriesRepository,
};
pub use service::{CardService, CardServiceImpl};
