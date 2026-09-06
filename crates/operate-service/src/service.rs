//! operate-service 业务实现

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::entity::{GiftPack, PushSchedule, RechargePackage, Reward};
use crate::error::{Error, Result};

type GiftDb = HashMap<u32, GiftPack>;
type RechargeDb = HashMap<u32, RechargePackage>;
type Pushes = HashMap<String, PushSchedule>;
type PurchaseCount = HashMap<(String, u32), u32>;
type PlayerBalance = HashMap<String, u32>;

pub struct OperateServiceImpl {
    gifts: Arc<RwLock<GiftDb>>,
    recharges: Arc<RwLock<RechargeDb>>,
    pushes: Arc<RwLock<Pushes>>,
    purchase_count: Arc<RwLock<PurchaseCount>>,
    balance: Arc<RwLock<PlayerBalance>>,
}

impl OperateServiceImpl {
    pub fn new() -> Self {
        let mut gifts = HashMap::new();
        gifts.insert(1, GiftPack::new(1, "新手礼包", 100, 200, 1));
        gifts.insert(2, GiftPack::new(2, "周卡", 500, 700, 1));
        gifts.insert(3, GiftPack::new(3, "月卡", 2000, 2500, 1));

        let mut recharges = HashMap::new();
        recharges.insert(1, RechargePackage { package_id: 1, name: "6元".into(), amount_cents: 600, bonus_cents: 0, currency_amount: 60 });
        recharges.insert(2, RechargePackage { package_id: 2, name: "30元".into(), amount_cents: 3000, bonus_cents: 300, currency_amount: 300 });
        recharges.insert(3, RechargePackage { package_id: 3, name: "98元".into(), amount_cents: 9800, bonus_cents: 1500, currency_amount: 980 });
        recharges.insert(4, RechargePackage { package_id: 4, name: "328元".into(), amount_cents: 32800, bonus_cents: 6500, currency_amount: 3280 });

        Self {
            gifts: Arc::new(RwLock::new(gifts)),
            recharges: Arc::new(RwLock::new(recharges)),
            pushes: Arc::new(RwLock::new(HashMap::new())),
            purchase_count: Arc::new(RwLock::new(HashMap::new())),
            balance: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn list_gift_packs(&self, _category: u32) -> Vec<GiftPack> {
        self.gifts.read().await.values().cloned().collect()
    }

    pub async fn purchase_gift_pack(&self, player_id: &str, pack_id: u32) -> Result<Vec<Reward>> {
        let gifts = self.gifts.read().await;
        let pack = gifts.get(&pack_id).cloned().ok_or_else(|| Error::GiftPackNotFound(pack_id.to_string()))?;
        drop(gifts);
        let mut count = self.purchase_count.write().await;
        let key = (player_id.to_string(), pack_id);
        let cur = count.get(&key).copied().unwrap_or(0);
        if cur >= pack.limit_per_player {
            return Err(Error::LimitReached(pack_id));
        }
        count.insert(key, cur + 1);
        Ok(vec![
            Reward { item_id: 1001, count: 10 },
            Reward { item_id: 1002, count: 1 },
        ])
    }

    pub async fn get_recharge_packages(&self) -> Vec<RechargePackage> {
        self.recharges.read().await.values().cloned().collect()
    }

    pub async fn complete_recharge(&self, player_id: &str, package_id: u32, receipt: &str) -> Result<u32> {
        if receipt.is_empty() {
            return Err(Error::InvalidRequest("receipt required".into()));
        }
        let recharges = self.recharges.read().await;
        let p = recharges.get(&package_id).cloned().ok_or_else(|| Error::RechargeNotFound(package_id.to_string()))?;
        drop(recharges);
        let mut bal = self.balance.write().await;
        let cur = bal.get(player_id).copied().unwrap_or(0);
        let new_bal = cur + p.total_currency();
        bal.insert(player_id.to_string(), new_bal);
        Ok(new_bal)
    }

    pub async fn schedule_push(&self, title: &str, content: &str, segment: u32, fire_at_ms: i64) -> Result<String> {
        if title.is_empty() || content.is_empty() {
            return Err(Error::InvalidRequest("title/content required".into()));
        }
        let p = PushSchedule::new(title, content, segment, fire_at_ms);
        let id = p.push_id.to_string();
        self.pushes.write().await.insert(id.clone(), p);
        Ok(id)
    }

    pub async fn cancel_push(&self, push_id: &str) -> Result<()> {
        let mut pushes = self.pushes.write().await;
        let p = pushes.get_mut(push_id).ok_or_else(|| Error::InvalidRequest("push not found".into()))?;
        if p.fired {
            return Err(Error::InvalidRequest("already fired".into()));
        }
        p.cancelled = true;
        Ok(())
    }

    pub async fn get_balance(&self, player_id: &str) -> u32 {
        self.balance.read().await.get(player_id).copied().unwrap_or(0)
    }
}

impl Default for OperateServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_gift_packs_seeded() {
        let svc = OperateServiceImpl::new();
        let packs = svc.list_gift_packs(0).await;
        assert_eq!(packs.len(), 3);
    }

    #[tokio::test]
    async fn purchase_gift_pack() {
        let svc = OperateServiceImpl::new();
        let rewards = svc.purchase_gift_pack("p1", 1).await.unwrap();
        assert_eq!(rewards.len(), 2);
    }

    #[tokio::test]
    async fn purchase_unknown_fails() {
        let svc = OperateServiceImpl::new();
        let r = svc.purchase_gift_pack("p1", 999).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn purchase_limit_enforced() {
        let svc = OperateServiceImpl::new();
        svc.purchase_gift_pack("p1", 1).await.unwrap();
        let r = svc.purchase_gift_pack("p1", 1).await;
        assert!(matches!(r, Err(Error::LimitReached(_))));
    }

    #[tokio::test]
    async fn get_recharge_packages_seeded() {
        let svc = OperateServiceImpl::new();
        let pkgs = svc.get_recharge_packages().await;
        assert_eq!(pkgs.len(), 4);
    }

    #[tokio::test]
    async fn complete_recharge_adds_balance() {
        let svc = OperateServiceImpl::new();
        let bal = svc.complete_recharge("p1", 4, "valid_receipt").await.unwrap();
        // 3280 + 650 bonus
        assert_eq!(bal, 9780);
    }

    #[tokio::test]
    async fn complete_recharge_empty_receipt_fails() {
        let svc = OperateServiceImpl::new();
        let r = svc.complete_recharge("p1", 1, "").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn complete_recharge_unknown_package_fails() {
        let svc = OperateServiceImpl::new();
        let r = svc.complete_recharge("p1", 999, "x").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn schedule_push() {
        let svc = OperateServiceImpl::new();
        let id = svc.schedule_push("t", "c", 1, 1000).await.unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn schedule_push_empty_title_fails() {
        let svc = OperateServiceImpl::new();
        let r = svc.schedule_push("", "c", 1, 1000).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn cancel_push() {
        let svc = OperateServiceImpl::new();
        let id = svc.schedule_push("t", "c", 1, 1000).await.unwrap();
        svc.cancel_push(&id).await.unwrap();
    }

    #[tokio::test]
    async fn cancel_unknown_push_fails() {
        let svc = OperateServiceImpl::new();
        let r = svc.cancel_push("bogus").await;
        assert!(r.is_err());
    }
}
