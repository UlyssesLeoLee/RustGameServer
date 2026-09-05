//! operate-service 域 entity

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiftPack {
    pub pack_id: u32,
    pub name: String,
    pub price_cents: u32,
    pub original_price_cents: u32,
    pub limit_per_player: u32,
}

impl GiftPack {
    pub fn new(pack_id: u32, name: &str, price_cents: u32, original_price_cents: u32, limit: u32) -> Self {
        Self { pack_id, name: name.into(), price_cents, original_price_cents, limit_per_player: limit }
    }

    pub fn discount_pct(&self) -> u32 {
        if self.original_price_cents == 0 {
            return 0;
        }
        let saved = self.original_price_cents.saturating_sub(self.price_cents);
        (saved * 100) / self.original_price_cents
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RechargePackage {
    pub package_id: u32,
    pub name: String,
    pub amount_cents: u32,
    pub bonus_cents: u32,
    pub currency_amount: u32,
}

impl RechargePackage {
    pub fn total_currency(&self) -> u32 {
        self.currency_amount + self.bonus_cents
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushSchedule {
    pub push_id: Uuid,
    pub title: String,
    pub content: String,
    pub target_segment: u32,
    pub fire_at_ms: i64,
    pub fired: bool,
    pub cancelled: bool,
}

impl PushSchedule {
    pub fn new(title: &str, content: &str, segment: u32, fire_at_ms: i64) -> Self {
        Self {
            push_id: Uuid::new_v4(),
            title: title.into(),
            content: content.into(),
            target_segment: segment,
            fire_at_ms,
            fired: false,
            cancelled: false,
        }
    }

    pub fn is_pending_at(&self, now_ms: i64) -> bool {
        !self.fired && !self.cancelled && self.fire_at_ms > now_ms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reward {
    pub item_id: u32,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gift_pack_discount_50pct() {
        let p = GiftPack::new(1, "p", 50, 100, 1);
        assert_eq!(p.discount_pct(), 50);
    }

    #[test]
    fn gift_pack_no_discount() {
        let p = GiftPack::new(1, "p", 100, 100, 1);
        assert_eq!(p.discount_pct(), 0);
    }

    #[test]
    fn gift_pack_zero_original() {
        let p = GiftPack::new(1, "p", 0, 0, 1);
        assert_eq!(p.discount_pct(), 0);
    }

    #[test]
    fn recharge_total_with_bonus() {
        let p = RechargePackage { package_id: 1, name: "x".into(), amount_cents: 100, bonus_cents: 50, currency_amount: 100 };
        assert_eq!(p.total_currency(), 150);
    }

    #[test]
    fn push_is_pending_before_fire() {
        let p = PushSchedule::new("t", "c", 1, 1000);
        assert!(p.is_pending_at(500));
        assert!(!p.is_pending_at(1500));
    }
}
