//! card-service —— 卡牌游戏新域微服务骨架 (per RGS-DTL-038 §4.4 + §3 DEC-038-01~09)
//!
//! 域职责 (per DEC-038-01 推荐 A): catalog (静态) + collection (动态) + 抽卡
//! - 卡组归 player-service (DEC-038-01), card-service 不承担 deck
//! - leaderboard 独立 (DEC-038-02) — 本域不含
//! - trade 归 economy (DEC-038-04) — 本域不含
//! - replay 归 cluster-ops 对象存储 (DEC-038-03) — 本域不含
//!
//! 桶 7 (proto 设计) 阶段: 仅编译 proto + 暴露公共类型。
//! 后续桶 (per RGS-DTL-038 §8 WBS):
//! - 桶 10 (card catalog): 加 db / service / repository / 8 张表 migration
//! - 桶 9 (session): match.proto v2 实装, 跨域调用本域
//! - 桶 13 (replay): match → replay, 间接触发本域 collection
//! - 桶 14 (trade+gm): 跨域调用本域 AddCardToCollection / RemoveCardFromCollection (saga)
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

// 桶 10 起按 5 域模板补: pub mod entity; pub mod error; pub mod repository; pub mod service; pub mod db;
