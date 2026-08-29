//! economy-service trade 域 entity 定义 (per RGS-DTL-038 §4.4 + DEC-038-04)
//!
//! 卡牌 8 桶 / 子桶 1: trade 域 (auction + private trade) 实体.
//! 规范: RGS-DTL-038 §7.1 #8 auctions + 9 DEC 全 A 拍板 (DEC-038-04 trade 归 economy-service v2).
//!
//! 跨域考量: card_instance_id 是跨服务软引用 (card-service 持有 card_instances),
//!          不强制 FK. 卡牌实际转移由 §6.3 ExecuteAuction saga 跨域编排完成 (W36+).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 拍卖状态（per DTL-038 §7.1 #8 + proto AuctionStatus）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuctionStatus {
    /// 进行中
    Active,
    /// 已成交
    Sold,
    /// 已撤单
    Cancelled,
    /// 已过期
    Expired,
}

impl AuctionStatus {
    /// proto / SQL 双向转换
    pub fn as_i32(self) -> i32 {
        match self {
            AuctionStatus::Active => 1,
            AuctionStatus::Sold => 2,
            AuctionStatus::Cancelled => 3,
            AuctionStatus::Expired => 4,
        }
    }
    pub fn from_i32(v: i32) -> Self {
        match v {
            2 => AuctionStatus::Sold,
            3 => AuctionStatus::Cancelled,
            4 => AuctionStatus::Expired,
            _ => AuctionStatus::Active,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            AuctionStatus::Active => "active",
            AuctionStatus::Sold => "sold",
            AuctionStatus::Cancelled => "cancelled",
            AuctionStatus::Expired => "expired",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "sold" => AuctionStatus::Sold,
            "cancelled" => AuctionStatus::Cancelled,
            "expired" => AuctionStatus::Expired,
            _ => AuctionStatus::Active,
        }
    }
}

/// 拍卖过滤器（per proto AuctionFilter）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuctionFilter {
    /// 仅进行中
    Active,
    /// 已结束 (sold / cancelled / expired)
    Closed,
    /// 全部
    All,
}

impl AuctionFilter {
    pub fn as_i32(self) -> i32 {
        match self {
            AuctionFilter::Active => 1,
            AuctionFilter::Closed => 2,
            AuctionFilter::All => 3,
        }
    }
    pub fn from_i32(v: i32) -> Self {
        match v {
            2 => AuctionFilter::Closed,
            3 => AuctionFilter::All,
            _ => AuctionFilter::Active,
        }
    }
}

/// 拍卖 entity（per DTL-038 §7.1 #8）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Auction {
    pub auction_id: Uuid,
    /// 卖家 player_id (跨服务边界, 用 String)
    pub seller_id: String,
    /// 卡牌静态 ID (catalog 引用, card.card_id)
    pub card_id: String,
    /// 卡牌实例 ID (card_instances.instance_id 跨域软引用)
    pub card_instance_id: String,
    /// 起拍价 / 一口价 (最小单位: 分 / 钻 / 代币)
    pub min_price: i64,
    /// 货币类型 (1=soft 2=hard 3=card_value, per common.proto CurrencyType)
    pub currency_type: i32,
    /// 当前最高价 (0 = 无人出价)
    pub highest_bid: i64,
    /// 当前最高出价者 player_id ("" = 无人)
    pub highest_bidder: String,
    pub status: AuctionStatus,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    /// 成交时买家 ID
    pub winner_id: Option<String>,
    /// 成交价
    pub final_price: i64,
    /// 跨域 saga 关联 (ExecuteAuction saga_id, 崩溃恢复用)
    pub saga_id: Option<Uuid>,
}

impl Auction {
    /// 工厂：新建拍卖（默认 Active / 24h 后到期）
    pub fn new(
        seller_id: String,
        card_id: String,
        card_instance_id: String,
        min_price: i64,
        currency_type: i32,
        duration_secs: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            auction_id: Uuid::new_v4(),
            seller_id,
            card_id,
            card_instance_id,
            min_price,
            currency_type,
            highest_bid: 0,
            highest_bidder: String::new(),
            status: AuctionStatus::Active,
            started_at: now,
            ends_at: now + chrono::Duration::seconds(duration_secs),
            closed_at: None,
            winner_id: None,
            final_price: 0,
            saga_id: None,
        }
    }

    /// 业务规则：拍卖是否活跃
    pub fn is_active(&self) -> bool {
        self.status == AuctionStatus::Active && Utc::now() < self.ends_at
    }

    /// 业务规则：是否到期
    pub fn is_expired(&self) -> bool {
        self.status == AuctionStatus::Active && Utc::now() >= self.ends_at
    }

    /// 业务规则：出价是否合法（>= min_price, > 当前 highest_bid）
    pub fn is_valid_bid(&self, amount: i64, bidder_id: &str) -> Result<(), &'static str> {
        if !self.is_active() {
            return Err("auction not active");
        }
        if bidder_id == self.seller_id {
            return Err("seller cannot bid on own auction");
        }
        if amount < self.min_price {
            return Err("bid below min_price");
        }
        if amount <= self.highest_bid {
            return Err("bid not higher than current highest");
        }
        Ok(())
    }
}

/// 私下交易状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivateTradeStatus {
    /// 已提议
    Proposed,
    /// 已接受
    Accepted,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

impl PrivateTradeStatus {
    pub fn as_i32(self) -> i32 {
        match self {
            PrivateTradeStatus::Proposed => 1,
            PrivateTradeStatus::Accepted => 2,
            PrivateTradeStatus::Completed => 3,
            PrivateTradeStatus::Cancelled => 4,
        }
    }
    pub fn from_i32(v: i32) -> Self {
        match v {
            2 => PrivateTradeStatus::Accepted,
            3 => PrivateTradeStatus::Completed,
            4 => PrivateTradeStatus::Cancelled,
            _ => PrivateTradeStatus::Proposed,
        }
    }
}

/// 私下交易 entity（per DEC-038-04: 私下交易也归 economy-service v2）
///
/// 业务语义: 玩家 A 向玩家 B 提出私下交易, 双方各自出 (货币 + 1 卡牌实例),
/// 双方 accept 后跨域执行 (card-service 协同, per §6.3 saga step 3/4).
///
/// 范围: W36+ 实装跨域 saga 完整逻辑 (TODO), 当前 schema + 简单 propose/cancel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivateTrade {
    pub trade_id: Uuid,
    pub proposer_id: String,
    pub counterparty_id: String,
    pub status: PrivateTradeStatus,
    pub proposer_currency_amount: i64,
    pub proposer_currency_type: Option<i32>,
    pub proposer_card_instance_id: Option<String>,
    pub counterparty_currency_amount: i64,
    pub counterparty_currency_type: Option<i32>,
    pub counterparty_card_instance_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub saga_id: Option<Uuid>,
}

impl PrivateTrade {
    /// 工厂：新建私下交易提议
    pub fn new(
        proposer_id: String,
        counterparty_id: String,
        proposer_currency_amount: i64,
        proposer_currency_type: Option<i32>,
        proposer_card_instance_id: Option<String>,
        counterparty_currency_amount: i64,
        counterparty_currency_type: Option<i32>,
        counterparty_card_instance_id: Option<String>,
    ) -> Self {
        Self {
            trade_id: Uuid::new_v4(),
            proposer_id,
            counterparty_id,
            status: PrivateTradeStatus::Proposed,
            proposer_currency_amount,
            proposer_currency_type,
            proposer_card_instance_id,
            counterparty_currency_amount,
            counterparty_currency_type,
            counterparty_card_instance_id,
            created_at: Utc::now(),
            closed_at: None,
            saga_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auction_new_defaults() {
        let a = Auction::new(
            "seller-1".to_string(),
            "card-1".to_string(),
            "inst-1".to_string(),
            100,
            1,
            3600,
        );
        assert_eq!(a.status, AuctionStatus::Active);
        assert_eq!(a.highest_bid, 0);
        assert_eq!(a.highest_bidder, "");
        assert!(a.is_active());
    }

    #[test]
    fn auction_valid_bid() {
        let a = Auction::new(
            "seller-1".to_string(),
            "card-1".to_string(),
            "inst-1".to_string(),
            100,
            1,
            3600,
        );
        assert!(a.is_valid_bid(150, "bidder-1").is_ok());
        // 低于起拍
        assert!(a.is_valid_bid(50, "bidder-1").is_err());
        // 卖家自己出价
        assert!(a.is_valid_bid(200, "seller-1").is_err());
    }

    #[test]
    fn auction_status_round_trip() {
        for s in [
            AuctionStatus::Active,
            AuctionStatus::Sold,
            AuctionStatus::Cancelled,
            AuctionStatus::Expired,
        ] {
            assert_eq!(AuctionStatus::from_i32(s.as_i32()), s);
            assert_eq!(AuctionStatus::from_str(s.as_str()), s);
        }
    }

    #[test]
    fn private_trade_new() {
        let t = PrivateTrade::new(
            "a".to_string(),
            "b".to_string(),
            100,
            Some(1),
            Some("card-a".to_string()),
            200,
            Some(1),
            Some("card-b".to_string()),
        );
        assert_eq!(t.status, PrivateTradeStatus::Proposed);
        assert_eq!(t.proposer_currency_amount, 100);
    }
}
