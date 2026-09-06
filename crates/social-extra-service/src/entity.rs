//! social-extra-service 域 entity
//!
//! 5 子系统: Mail, Friend, Home, ChatMessage, Profile

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Attachment {
    pub item_id: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mail {
    pub mail_id: Uuid,
    pub from: String,
    pub to: String,
    pub title: String,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub sent_at_ms: i64,
    pub read: bool,
    pub claimed: bool,
}

impl Mail {
    pub fn new(from: &str, to: &str, title: &str, body: &str, attachments: Vec<Attachment>) -> Self {
        Self {
            mail_id: Uuid::new_v4(),
            from: from.to_string(),
            to: to.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            attachments,
            sent_at_ms: Utc::now().timestamp_millis(),
            read: false,
            claimed: false,
        }
    }

    pub fn has_unclaimed_attachment(&self) -> bool {
        !self.attachments.is_empty() && !self.claimed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Friend {
    pub player_id: Uuid,
    pub display_name: String,
    pub level: u32,
    pub last_online_ms: i64,
}

impl Friend {
    pub fn new(player_id: Uuid, display_name: &str, level: u32) -> Self {
        Self {
            player_id,
            display_name: display_name.to_string(),
            level,
            last_online_ms: Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Home {
    pub player_id: Uuid,
    pub theme_id: u32,
    pub slots: Vec<u32>, // 10 slots
    pub visit_count: u32,
}

impl Home {
    pub fn new(player_id: Uuid) -> Self {
        Self {
            player_id,
            theme_id: 1,
            slots: vec![0; 10],
            visit_count: 0,
        }
    }

    pub fn decorate(&mut self, slot: usize, item_id: u32) -> bool {
        if slot < self.slots.len() {
            self.slots[slot] = item_id;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub message_id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub content: String,
    pub sent_at_ms: i64,
    pub channel: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mail_factory_creates_unread() {
        let m = Mail::new("sys", "p1", "Welcome", "Hi", vec![Attachment { item_id: 100, count: 1 }]);
        assert!(!m.read);
        assert!(!m.claimed);
        assert!(m.has_unclaimed_attachment());
    }

    #[test]
    fn mail_no_attachment_is_not_unclaimed() {
        let m = Mail::new("sys", "p1", "T", "B", vec![]);
        assert!(!m.has_unclaimed_attachment());
    }

    #[test]
    fn friend_factory_sets_level() {
        let f = Friend::new(Uuid::new_v4(), "alice", 50);
        assert_eq!(f.level, 50);
    }

    #[test]
    fn home_decorate_valid_slot() {
        let mut h = Home::new(Uuid::new_v4());
        assert!(h.decorate(0, 1001));
        assert_eq!(h.slots[0], 1001);
    }

    #[test]
    fn home_decorate_invalid_slot_rejected() {
        let mut h = Home::new(Uuid::new_v4());
        assert!(!h.decorate(100, 1001));
    }

    #[test]
    fn home_starts_at_theme_1() {
        let h = Home::new(Uuid::new_v4());
        assert_eq!(h.theme_id, 1);
        assert_eq!(h.slots.len(), 10);
    }
}
