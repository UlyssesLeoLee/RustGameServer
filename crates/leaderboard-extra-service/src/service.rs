//! leaderboard-extra-service 业务实现
//!
//! 5 业务方法 + 5 UT:
//! - 图鉴 (GetCollection / UnlockCard / GetCardDetail / GetCollectionProgress)
//! - 排行榜扩展 (GetServerRank)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::entity::{CardDetail, CardEntry, RankRow, Rarity};
use crate::error::{Error, Result};

type PlayerCollection = HashMap<Uuid, HashMap<u32, CardEntry>>;
type CardDb = HashMap<u32, CardDetail>;
type ServerRank = HashMap<u32, Vec<RankRow>>;

pub struct LeaderboardExtraServiceImpl {
    collections: Arc<RwLock<PlayerCollection>>,
    card_db: Arc<RwLock<CardDb>>,
    server_rank: Arc<RwLock<ServerRank>>,
}

impl LeaderboardExtraServiceImpl {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
            card_db: Arc::new(RwLock::new(HashMap::new())),
            server_rank: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn seed_card(&self, detail: CardDetail) {
        let mut db = self.card_db.write().await;
        db.insert(detail.card_id, detail);
    }

    pub async fn seed_rank(&self, board_id: u32, rows: Vec<RankRow>) {
        let mut rank = self.server_rank.write().await;
        rank.insert(board_id, rows);
    }

    pub async fn get_collection(&self, player_id: Uuid, _category: u32) -> Vec<CardEntry> {
        let col = self.collections.read().await;
        col.get(&player_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn unlock_card(&self, player_id: Uuid, card_id: u32, count: u32) -> Result<u32> {
        let db = self.card_db.read().await;
        if !db.contains_key(&card_id) {
            return Err(Error::CardNotFound(card_id.to_string()));
        }
        drop(db);
        let mut col = self.collections.write().await;
        let m = col.entry(player_id).or_default();
        let entry = m.entry(card_id).or_insert_with(|| {
            let mut e = CardEntry::new(card_id, Rarity::Common);
            e.is_new = true;
            e
        });
        entry.add(count);
        Ok(entry.count)
    }

    pub async fn get_card_detail(&self, card_id: u32) -> Result<CardDetail> {
        let db = self.card_db.read().await;
        db.get(&card_id).cloned().ok_or_else(|| Error::CardNotFound(card_id.to_string()))
    }

    pub async fn get_collection_progress(&self, player_id: Uuid) -> (u32, u32, f32) {
        let db = self.card_db.read().await;
        let col = self.collections.read().await;
        let total = db.len() as u32;
        let unlocked = col.get(&player_id).map(|m| m.len() as u32).unwrap_or(0);
        let pct = if total == 0 { 0.0 } else { (unlocked as f32) / (total as f32) * 100.0 };
        (total, unlocked, pct)
    }

    pub async fn get_server_rank(&self, board_id: u32, top_n: u32) -> Vec<RankRow> {
        let rank = self.server_rank.read().await;
        rank.get(&board_id)
            .map(|v| v.iter().take(top_n as usize).cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for LeaderboardExtraServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail(id: u32) -> CardDetail {
        CardDetail { card_id: id, name: format!("Card{}", id), rarity: Rarity::Common, description: "x".into(), max_count: 99 }
    }

    #[tokio::test]
    async fn get_empty_collection() {
        let svc = LeaderboardExtraServiceImpl::new();
        let list = svc.get_collection(Uuid::new_v4(), 0).await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn unlock_unknown_card_fails() {
        let svc = LeaderboardExtraServiceImpl::new();
        let r = svc.unlock_card(Uuid::new_v4(), 999, 1).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn unlock_then_get() {
        let svc = LeaderboardExtraServiceImpl::new();
        svc.seed_card(detail(1)).await;
        svc.seed_card(detail(2)).await;
        let p = Uuid::new_v4();
        let c = svc.unlock_card(p, 1, 3).await.unwrap();
        assert_eq!(c, 3);
        let list = svc.get_collection(p, 0).await;
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn get_card_detail() {
        let svc = LeaderboardExtraServiceImpl::new();
        svc.seed_card(detail(42)).await;
        let d = svc.get_card_detail(42).await.unwrap();
        assert_eq!(d.name, "Card42");
    }

    #[tokio::test]
    async fn get_card_detail_unknown_fails() {
        let svc = LeaderboardExtraServiceImpl::new();
        let r = svc.get_card_detail(999).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn collection_progress_pct() {
        let svc = LeaderboardExtraServiceImpl::new();
        svc.seed_card(detail(1)).await;
        svc.seed_card(detail(2)).await;
        svc.seed_card(detail(3)).await;
        svc.seed_card(detail(4)).await;
        let p = Uuid::new_v4();
        svc.unlock_card(p, 1, 1).await.unwrap();
        svc.unlock_card(p, 2, 1).await.unwrap();
        let (total, unlocked, pct) = svc.get_collection_progress(p).await;
        assert_eq!(total, 4);
        assert_eq!(unlocked, 2);
        assert!((pct - 50.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn server_rank_empty() {
        let svc = LeaderboardExtraServiceImpl::new();
        let rank = svc.get_server_rank(1, 10).await;
        assert!(rank.is_empty());
    }

    #[tokio::test]
    async fn server_rank_top_n() {
        let svc = LeaderboardExtraServiceImpl::new();
        let rows: Vec<RankRow> = (0..5).map(|i| RankRow { rank: i, player_id: format!("p{}", i), display_name: format!("d{}", i), score: 1000 - i as i64 }).collect();
        svc.seed_rank(1, rows).await;
        let r = svc.get_server_rank(1, 3).await;
        assert_eq!(r.len(), 3);
    }
}
