//! leaderboard-extra-service 域 entity
//!
//! CardEntry / CardDetail / RankRow

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Common = 1,
    Rare = 2,
    Epic = 3,
    Legendary = 4,
    Mythic = 5,
}

impl Rarity {
    pub fn from_i32(v: i32) -> Self {
        match v {
            5 => Rarity::Mythic,
            4 => Rarity::Legendary,
            3 => Rarity::Epic,
            2 => Rarity::Rare,
            _ => Rarity::Common,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardEntry {
    pub card_id: u32,
    pub count: u32,
    pub is_new: bool,
    pub rarity: Rarity,
}

impl CardEntry {
    pub fn new(card_id: u32, rarity: Rarity) -> Self {
        Self { card_id, count: 0, is_new: true, rarity }
    }

    pub fn add(&mut self, by: u32) -> bool {
        let was_new = self.count == 0;
        self.count = self.count.saturating_add(by);
        if was_new && self.count > 0 {
            self.is_new = false;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardDetail {
    pub card_id: u32,
    pub name: String,
    pub rarity: Rarity,
    pub description: String,
    pub max_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankRow {
    pub rank: u32,
    pub player_id: String,
    pub display_name: String,
    pub score: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rarity_from_i32() {
        assert_eq!(Rarity::from_i32(5), Rarity::Mythic);
        assert_eq!(Rarity::from_i32(0), Rarity::Common);
        assert_eq!(Rarity::from_i32(99), Rarity::Common);
    }

    #[test]
    fn card_entry_starts_unlocked_zero() {
        let c = CardEntry::new(1, Rarity::Common);
        assert_eq!(c.count, 0);
        assert!(c.is_new);
    }

    #[test]
    fn card_add_returns_first_unlock() {
        let mut c = CardEntry::new(1, Rarity::Common);
        assert!(c.add(1)); // 首次解锁
        assert!(!c.is_new);
        assert!(!c.add(1)); // 重复添加
    }

    #[test]
    fn card_add_saturates() {
        let mut c = CardEntry::new(1, Rarity::Common);
        c.add(u32::MAX);
        c.add(100);
        assert_eq!(c.count, u32::MAX);
    }
}
