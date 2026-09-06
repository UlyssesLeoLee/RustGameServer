//! gm-extra-service 域 entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BanScope {
    Login,
    Chat,
    Trade,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BanRecord {
    pub ban_id: Uuid,
    pub gm_id: String,
    pub player_id: String,
    pub reason: String,
    pub scope: BanScope,
    pub starts_at_ms: i64,
    pub ends_at_ms: i64,
    pub active: bool,
}

impl BanRecord {
    pub fn new(gm_id: &str, player_id: &str, reason: &str, scope: BanScope, duration_secs: i64) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            ban_id: Uuid::new_v4(),
            gm_id: gm_id.to_string(),
            player_id: player_id.to_string(),
            reason: reason.to_string(),
            scope,
            starts_at_ms: now,
            ends_at_ms: now + duration_secs * 1000,
            active: true,
        }
    }

    pub fn is_active_at(&self, now_ms: i64) -> bool {
        self.active && now_ms >= self.starts_at_ms && now_ms < self.ends_at_ms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEntry {
    pub entry_id: Uuid,
    pub gm_id: String,
    pub command: String,
    pub target_id: String,
    pub timestamp: DateTime<Utc>,
    pub result: String,
}

impl AuditEntry {
    pub fn new(gm_id: &str, command: &str, target_id: &str, result: &str) -> Self {
        Self {
            entry_id: Uuid::new_v4(),
            gm_id: gm_id.to_string(),
            command: command.to_string(),
            target_id: target_id.to_string(),
            timestamp: Utc::now(),
            result: result.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ban_factory_active_within_window() {
        let b = BanRecord::new("gm1", "p1", "test", BanScope::Login, 3600);
        let now = b.starts_at_ms + 1000;
        assert!(b.is_active_at(now));
    }

    #[test]
    fn ban_expires() {
        let b = BanRecord::new("gm1", "p1", "test", BanScope::Login, 60);
        let after = b.ends_at_ms + 1;
        assert!(!b.is_active_at(after));
    }

    #[test]
    fn ban_inactive_when_deactivated() {
        let mut b = BanRecord::new("gm1", "p1", "test", BanScope::Login, 3600);
        b.active = false;
        assert!(!b.is_active_at(b.starts_at_ms + 1000));
    }

    #[test]
    fn audit_entry_factory() {
        let e = AuditEntry::new("gm1", "ban", "p1", "ok");
        assert_eq!(e.command, "ban");
    }
}
