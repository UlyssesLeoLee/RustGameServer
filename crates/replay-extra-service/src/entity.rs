//! replay-extra-service 域 entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayRecord {
    pub replay_id: Uuid,
    pub player_id: String,
    pub title: String,
    pub duration_secs: u32,
    pub created_at: DateTime<Utc>,
    pub view_count: u32,
    pub size_bytes: u32,
}

impl ReplayRecord {
    pub fn new(player_id: &str, title: &str, duration_secs: u32, size_bytes: u32) -> Self {
        Self {
            replay_id: Uuid::new_v4(),
            player_id: player_id.to_string(),
            title: title.to_string(),
            duration_secs,
            created_at: Utc::now(),
            view_count: 0,
            size_bytes,
        }
    }

    pub fn add_view(&mut self) {
        self.view_count = self.view_count.saturating_add(1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoRecord {
    pub video_id: Uuid,
    pub player_id: String,
    pub title: String,
    pub duration_secs: u32,
    pub view_count: u32,
    pub size_bytes: u32,
}

impl VideoRecord {
    pub fn new(player_id: &str, title: &str, duration_secs: u32, size_bytes: u32) -> Self {
        Self {
            video_id: Uuid::new_v4(),
            player_id: player_id.to_string(),
            title: title.to_string(),
            duration_secs,
            view_count: 0,
            size_bytes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Comment {
    pub comment_id: Uuid,
    pub target_id: String,
    pub player_id: String,
    pub content: String,
    pub posted_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_factory_default_view() {
        let r = ReplayRecord::new("p1", "test", 60, 1024);
        assert_eq!(r.view_count, 0);
        assert_eq!(r.duration_secs, 60);
    }

    #[test]
    fn replay_add_view_increments() {
        let mut r = ReplayRecord::new("p1", "t", 60, 1024);
        r.add_view();
        r.add_view();
        r.add_view();
        assert_eq!(r.view_count, 3);
    }

    #[test]
    fn replay_view_saturates() {
        let mut r = ReplayRecord::new("p1", "t", 60, 1024);
        r.view_count = u32::MAX;
        r.add_view();
        assert_eq!(r.view_count, u32::MAX);
    }

    #[test]
    fn video_factory() {
        let v = VideoRecord::new("p1", "t", 120, 4096);
        assert_eq!(v.size_bytes, 4096);
    }
}
