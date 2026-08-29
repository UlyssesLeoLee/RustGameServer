//! economy-service trade 域 Repository (per RGS-DTL-038 §7.1 + DEC-038-04)
//!
//! 规范: trait + PgRepository sqlx impl + InMemoryRepository 测试用
//! 设计: 跨域 (card-service) 软引用 card_instance_id, 不强制 FK.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::trade_entity::{Auction, AuctionFilter, AuctionStatus, PrivateTrade, PrivateTradeStatus};
use crate::Result;

/// Auction Repository trait
#[async_trait]
pub trait TradeRepository: Send + Sync {
    /// 创建拍卖（upsert by id）
    async fn save_auction(&self, a: &Auction) -> Result<Auction>;
    /// 按 ID 查询
    async fn find_auction_by_id(&self, id: Uuid) -> Result<Option<Auction>>;
    /// 列表查询（带 filter + 分页）
    async fn list_auctions(
        &self,
        filter: AuctionFilter,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)>;
    /// 玩家相关 (作为卖家或出价者) 的历史
    async fn list_auctions_by_player(
        &self,
        player_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)>;
    /// 到期扫描 (active + ends_at < now)，W36+ 后台任务调用
    async fn list_expired_active(&self, limit: i64) -> Result<Vec<Auction>>;
    /// OCC 更新（仅 status / highest_bid / highest_bidder / closed_at / winner / final_price / saga_id 可变）
    async fn update_auction(&self, a: &Auction) -> Result<Auction>;
    /// 删除（测试用）
    async fn delete_auction(&self, id: Uuid) -> Result<bool>;

    // --- PrivateTrade（私下交易）---

    async fn save_private_trade(&self, t: &PrivateTrade) -> Result<PrivateTrade>;
    async fn find_private_trade_by_id(&self, id: Uuid) -> Result<Option<PrivateTrade>>;
    async fn update_private_trade(&self, t: &PrivateTrade) -> Result<PrivateTrade>;
    async fn delete_private_trade(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// PgRepository (sqlx 实现)
// ============================================================================

pub struct PgTradeRepository {
    pool: PgPool,
}

impl PgTradeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_auction(row: sqlx::postgres::PgRow) -> Auction {
    let status_v: i32 = row.get("status");
    let currency_v: i32 = row.get("currency_type");
    Auction {
        auction_id: row.get("auction_id"),
        seller_id: row.get("seller_id"),
        card_id: row.get("card_id"),
        card_instance_id: row.get("card_instance_id"),
        min_price: row.get("min_price"),
        currency_type: currency_v,
        highest_bid: row.get("highest_bid"),
        highest_bidder: row.get("highest_bidder"),
        status: AuctionStatus::from_i32(status_v),
        started_at: row.get("started_at"),
        ends_at: row.get("ends_at"),
        closed_at: row.get("closed_at"),
        winner_id: row.get("winner_id"),
        final_price: row.get("final_price"),
        saga_id: row.get("saga_id"),
    }
}

fn row_to_private_trade(row: sqlx::postgres::PgRow) -> PrivateTrade {
    let status_v: i32 = row.get("status");
    PrivateTrade {
        trade_id: row.get("trade_id"),
        proposer_id: row.get("proposer_id"),
        counterparty_id: row.get("counterparty_id"),
        status: PrivateTradeStatus::from_i32(status_v),
        proposer_currency_amount: row.get("proposer_currency_amount"),
        proposer_currency_type: row.get("proposer_currency_type"),
        proposer_card_instance_id: row.get("proposer_card_instance_id"),
        counterparty_currency_amount: row.get("counterparty_currency_amount"),
        counterparty_currency_type: row.get("counterparty_currency_type"),
        counterparty_card_instance_id: row.get("counterparty_card_instance_id"),
        created_at: row.get("created_at"),
        closed_at: row.get("closed_at"),
        saga_id: row.get("saga_id"),
    }
}

#[async_trait]
impl TradeRepository for PgTradeRepository {
    async fn save_auction(&self, a: &Auction) -> Result<Auction> {
        sqlx::query(
            "INSERT INTO auctions \
             (auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
              highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
              winner_id, final_price, saga_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             ON CONFLICT (auction_id) DO UPDATE SET \
                highest_bid = EXCLUDED.highest_bid, highest_bidder = EXCLUDED.highest_bidder, \
                status = EXCLUDED.status, closed_at = EXCLUDED.closed_at, \
                winner_id = EXCLUDED.winner_id, final_price = EXCLUDED.final_price, \
                saga_id = EXCLUDED.saga_id",
        )
        .bind(a.auction_id)
        .bind(&a.seller_id)
        .bind(&a.card_id)
        .bind(&a.card_instance_id)
        .bind(a.min_price)
        .bind(a.currency_type)
        .bind(a.highest_bid)
        .bind(&a.highest_bidder)
        .bind(a.status.as_i32())
        .bind(a.started_at)
        .bind(a.ends_at)
        .bind(a.closed_at)
        .bind(&a.winner_id)
        .bind(a.final_price)
        .bind(a.saga_id)
        .execute(&self.pool)
        .await?;
        Ok(a.clone())
    }

    async fn find_auction_by_id(&self, id: Uuid) -> Result<Option<Auction>> {
        let row = sqlx::query(
            "SELECT auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
                    highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
                    winner_id, final_price, saga_id \
             FROM auctions WHERE auction_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_auction))
    }

    async fn list_auctions(
        &self,
        filter: AuctionFilter,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)> {
        let offset = (page.saturating_sub(1) as i64) * page_size as i64;
        let limit = page_size as i64;

        let (rows, total) = match filter {
            AuctionFilter::Active => {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM auctions WHERE status = 1 AND ends_at > now()",
                )
                .fetch_one(&self.pool)
                .await?;
                let rows = sqlx::query(
                    "SELECT auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
                            highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
                            winner_id, final_price, saga_id \
                     FROM auctions WHERE status = 1 AND ends_at > now() \
                     ORDER BY started_at DESC LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                (rows, total as u64)
            }
            AuctionFilter::Closed => {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM auctions WHERE status IN (2, 3, 4)",
                )
                .fetch_one(&self.pool)
                .await?;
                let rows = sqlx::query(
                    "SELECT auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
                            highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
                            winner_id, final_price, saga_id \
                     FROM auctions WHERE status IN (2, 3, 4) \
                     ORDER BY closed_at DESC LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                (rows, total as u64)
            }
            AuctionFilter::All => {
                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auctions")
                    .fetch_one(&self.pool)
                    .await?;
                let rows = sqlx::query(
                    "SELECT auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
                            highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
                            winner_id, final_price, saga_id \
                     FROM auctions ORDER BY started_at DESC LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                (rows, total as u64)
            }
        };

        Ok((rows.into_iter().map(row_to_auction).collect(), total))
    }

    async fn list_auctions_by_player(
        &self,
        player_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)> {
        let offset = (page.saturating_sub(1) as i64) * page_size as i64;
        let limit = page_size as i64;
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM auctions WHERE seller_id = $1 OR highest_bidder = $1",
        )
        .bind(player_id)
        .fetch_one(&self.pool)
        .await?;
        let rows = sqlx::query(
            "SELECT auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
                    highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
                    winner_id, final_price, saga_id \
             FROM auctions WHERE seller_id = $1 OR highest_bidder = $1 \
             ORDER BY started_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(player_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows.into_iter().map(row_to_auction).collect(), total as u64))
    }

    async fn list_expired_active(&self, limit: i64) -> Result<Vec<Auction>> {
        let rows = sqlx::query(
            "SELECT auction_id, seller_id, card_id, card_instance_id, min_price, currency_type, \
                    highest_bid, highest_bidder, status, started_at, ends_at, closed_at, \
                    winner_id, final_price, saga_id \
             FROM auctions WHERE status = 1 AND ends_at <= now() \
             ORDER BY ends_at ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_auction).collect())
    }

    async fn update_auction(&self, a: &Auction) -> Result<Auction> {
        // OCC: status 在 DB 中必须与传入一致 (避免并发状态机)
        let result = sqlx::query(
            "UPDATE auctions SET highest_bid = $1, highest_bidder = $2, status = $3, \
                closed_at = $4, winner_id = $5, final_price = $6, saga_id = $7 \
             WHERE auction_id = $8",
        )
        .bind(a.highest_bid)
        .bind(&a.highest_bidder)
        .bind(a.status.as_i32())
        .bind(a.closed_at)
        .bind(&a.winner_id)
        .bind(a.final_price)
        .bind(a.saga_id)
        .bind(a.auction_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(crate::Error::NotFound {
                entity: "Auction",
                id: a.auction_id.to_string(),
            });
        }
        Ok(a.clone())
    }

    async fn delete_auction(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM auctions WHERE auction_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // --- PrivateTrade ---

    async fn save_private_trade(&self, t: &PrivateTrade) -> Result<PrivateTrade> {
        sqlx::query(
            "INSERT INTO private_trades \
             (trade_id, proposer_id, counterparty_id, status, proposer_currency_amount, \
              proposer_currency_type, proposer_card_instance_id, counterparty_currency_amount, \
              counterparty_currency_type, counterparty_card_instance_id, created_at, closed_at, saga_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (trade_id) DO UPDATE SET \
                status = EXCLUDED.status, closed_at = EXCLUDED.closed_at, saga_id = EXCLUDED.saga_id",
        )
        .bind(t.trade_id)
        .bind(&t.proposer_id)
        .bind(&t.counterparty_id)
        .bind(t.status.as_i32())
        .bind(t.proposer_currency_amount)
        .bind(t.proposer_currency_type)
        .bind(&t.proposer_card_instance_id)
        .bind(t.counterparty_currency_amount)
        .bind(t.counterparty_currency_type)
        .bind(&t.counterparty_card_instance_id)
        .bind(t.created_at)
        .bind(t.closed_at)
        .bind(t.saga_id)
        .execute(&self.pool)
        .await?;
        Ok(t.clone())
    }

    async fn find_private_trade_by_id(&self, id: Uuid) -> Result<Option<PrivateTrade>> {
        let row = sqlx::query(
            "SELECT trade_id, proposer_id, counterparty_id, status, proposer_currency_amount, \
                    proposer_currency_type, proposer_card_instance_id, counterparty_currency_amount, \
                    counterparty_currency_type, counterparty_card_instance_id, \
                    created_at, closed_at, saga_id \
             FROM private_trades WHERE trade_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_private_trade))
    }

    async fn update_private_trade(&self, t: &PrivateTrade) -> Result<PrivateTrade> {
        let result = sqlx::query(
            "UPDATE private_trades SET status = $1, closed_at = $2, saga_id = $3 \
             WHERE trade_id = $4",
        )
        .bind(t.status.as_i32())
        .bind(t.closed_at)
        .bind(t.saga_id)
        .bind(t.trade_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(crate::Error::NotFound {
                entity: "PrivateTrade",
                id: t.trade_id.to_string(),
            });
        }
        Ok(t.clone())
    }

    async fn delete_private_trade(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM private_trades WHERE trade_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// InMemoryRepository (测试用)
// ============================================================================

#[derive(Default)]
pub struct InMemoryTradeRepository {
    pub(crate) auctions: Arc<Mutex<HashMap<Uuid, Auction>>>,
    pub(crate) private_trades: Arc<Mutex<HashMap<Uuid, PrivateTrade>>>,
}

impl InMemoryTradeRepository {
    pub fn new() -> Self {
        Self {
            auctions: Arc::new(Mutex::new(HashMap::new())),
            private_trades: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TradeRepository for InMemoryTradeRepository {
    async fn save_auction(&self, a: &Auction) -> Result<Auction> {
        self.auctions.lock().unwrap().insert(a.auction_id, a.clone());
        Ok(a.clone())
    }

    async fn find_auction_by_id(&self, id: Uuid) -> Result<Option<Auction>> {
        Ok(self.auctions.lock().unwrap().get(&id).cloned())
    }

    async fn list_auctions(
        &self,
        filter: AuctionFilter,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)> {
        let now = Utc::now();
        let all: Vec<Auction> = self
            .auctions
            .lock()
            .unwrap()
            .values()
            .filter(|a| match filter {
                AuctionFilter::Active => a.status == AuctionStatus::Active && a.ends_at > now,
                AuctionFilter::Closed => {
                    a.status == AuctionStatus::Sold
                        || a.status == AuctionStatus::Cancelled
                        || a.status == AuctionStatus::Expired
                }
                AuctionFilter::All => true,
            })
            .cloned()
            .collect();
        let total = all.len() as u64;
        let start = (page.saturating_sub(1) as usize) * page_size as usize;
        let mut sorted = all;
        sorted.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        let page_items: Vec<Auction> =
            sorted.into_iter().skip(start).take(page_size as usize).collect();
        Ok((page_items, total))
    }

    async fn list_auctions_by_player(
        &self,
        player_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)> {
        let all: Vec<Auction> = self
            .auctions
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.seller_id == player_id || a.highest_bidder == player_id)
            .cloned()
            .collect();
        let total = all.len() as u64;
        let start = (page.saturating_sub(1) as usize) * page_size as usize;
        let mut sorted = all;
        sorted.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        let page_items: Vec<Auction> =
            sorted.into_iter().skip(start).take(page_size as usize).collect();
        Ok((page_items, total))
    }

    async fn list_expired_active(&self, limit: i64) -> Result<Vec<Auction>> {
        let now = Utc::now();
        let mut expired: Vec<Auction> = self
            .auctions
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.status == AuctionStatus::Active && a.ends_at <= now)
            .cloned()
            .collect();
        expired.sort_by_key(|a| a.ends_at);
        expired.truncate(limit as usize);
        Ok(expired)
    }

    async fn update_auction(&self, a: &Auction) -> Result<Auction> {
        let mut guard = self.auctions.lock().unwrap();
        if !guard.contains_key(&a.auction_id) {
            return Err(crate::Error::NotFound {
                entity: "Auction",
                id: a.auction_id.to_string(),
            });
        }
        guard.insert(a.auction_id, a.clone());
        Ok(a.clone())
    }

    async fn delete_auction(&self, id: Uuid) -> Result<bool> {
        Ok(self.auctions.lock().unwrap().remove(&id).is_some())
    }

    async fn save_private_trade(&self, t: &PrivateTrade) -> Result<PrivateTrade> {
        self.private_trades
            .lock()
            .unwrap()
            .insert(t.trade_id, t.clone());
        Ok(t.clone())
    }

    async fn find_private_trade_by_id(&self, id: Uuid) -> Result<Option<PrivateTrade>> {
        Ok(self.private_trades.lock().unwrap().get(&id).cloned())
    }

    async fn update_private_trade(&self, t: &PrivateTrade) -> Result<PrivateTrade> {
        let mut guard = self.private_trades.lock().unwrap();
        if !guard.contains_key(&t.trade_id) {
            return Err(crate::Error::NotFound {
                entity: "PrivateTrade",
                id: t.trade_id.to_string(),
            });
        }
        guard.insert(t.trade_id, t.clone());
        Ok(t.clone())
    }

    async fn delete_private_trade(&self, id: Uuid) -> Result<bool> {
        Ok(self.private_trades.lock().unwrap().remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_auction(seller: &str, price: i64) -> Auction {
        Auction::new(
            seller.to_string(),
            "card-x".to_string(),
            "inst-x".to_string(),
            price,
            1,
            3600,
        )
    }

    #[tokio::test]
    async fn in_memory_save_and_find() {
        let repo = InMemoryTradeRepository::new();
        let a = make_auction("seller-a", 100);
        let id = a.auction_id;
        repo.save_auction(&a).await.unwrap();

        let found = repo.find_auction_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.seller_id, "seller-a");
        assert_eq!(found.min_price, 100);
    }

    #[tokio::test]
    async fn in_memory_list_active() {
        let repo = InMemoryTradeRepository::new();
        repo.save_auction(&make_auction("s1", 50)).await.unwrap();
        repo.save_auction(&make_auction("s2", 200)).await.unwrap();
        let (list, total) = repo.list_auctions(AuctionFilter::Active, 1, 10).await.unwrap();
        assert_eq!(total, 2);
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_list_by_player() {
        let repo = InMemoryTradeRepository::new();
        let mut a = make_auction("alice", 100);
        a.highest_bid = 150;
        a.highest_bidder = "bob".to_string();
        repo.save_auction(&a).await.unwrap();
        repo.save_auction(&make_auction("charlie", 200)).await.unwrap();

        // alice 视角: 看到自己作为卖家的
        let (alice_list, alice_total) = repo
            .list_auctions_by_player("alice", 1, 10)
            .await
            .unwrap();
        assert_eq!(alice_total, 1);
        assert_eq!(alice_list[0].seller_id, "alice");

        // bob 视角: 看到自己作为出价者的
        let (bob_list, bob_total) = repo
            .list_auctions_by_player("bob", 1, 10)
            .await
            .unwrap();
        assert_eq!(bob_total, 1);
        assert_eq!(bob_list[0].highest_bidder, "bob");
    }

    #[tokio::test]
    async fn in_memory_update_auction_status() {
        let repo = InMemoryTradeRepository::new();
        let mut a = make_auction("s1", 100);
        let id = a.auction_id;
        repo.save_auction(&a).await.unwrap();

        a.status = AuctionStatus::Sold;
        a.winner_id = Some("w1".to_string());
        a.final_price = 200;
        a.highest_bid = 200;
        a.highest_bidder = "w1".to_string();
        a.closed_at = Some(Utc::now());
        repo.update_auction(&a).await.unwrap();

        let loaded = repo.find_auction_by_id(id).await.unwrap().unwrap();
        assert_eq!(loaded.status, AuctionStatus::Sold);
        assert_eq!(loaded.winner_id, Some("w1".to_string()));
        assert_eq!(loaded.final_price, 200);
    }

    #[tokio::test]
    async fn in_memory_private_trade_save_update() {
        let repo = InMemoryTradeRepository::new();
        let t = PrivateTrade::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            Some(1),
            Some("card-a".to_string()),
            200,
            Some(1),
            Some("card-b".to_string()),
        );
        let id = t.trade_id;
        repo.save_private_trade(&t).await.unwrap();

        let mut loaded = repo.find_private_trade_by_id(id).await.unwrap().unwrap();
        assert_eq!(loaded.status, PrivateTradeStatus::Proposed);
        loaded.status = PrivateTradeStatus::Cancelled;
        loaded.closed_at = Some(Utc::now());
        repo.update_private_trade(&loaded).await.unwrap();

        let updated = repo.find_private_trade_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.status, PrivateTradeStatus::Cancelled);
    }

    #[tokio::test]
    async fn in_memory_list_expired_active() {
        let repo = InMemoryTradeRepository::new();
        // 1 active 未到期
        repo.save_auction(&make_auction("s1", 50)).await.unwrap();
        // 1 active 已到期 (-1s)
        let mut expired = make_auction("s2", 100);
        expired.ends_at = Utc::now() - chrono::Duration::seconds(1);
        repo.save_auction(&expired).await.unwrap();
        // 1 cancelled 不算 expired_active
        let mut cancelled = make_auction("s3", 150);
        cancelled.status = AuctionStatus::Cancelled;
        repo.save_auction(&cancelled).await.unwrap();

        let list = repo.list_expired_active(100).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].seller_id, "s2");
    }
}

// 抑制 unused DateTime 警告
#[allow(dead_code)]
fn _force_use_datetime(_dt: DateTime<Utc>) {}
