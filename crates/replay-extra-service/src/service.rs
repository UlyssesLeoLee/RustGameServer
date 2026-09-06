//! replay-extra-service 业务实现

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::entity::{Comment, ReplayRecord, VideoRecord};
use crate::error::{Error, Result};

type Replays = HashMap<Uuid, ReplayRecord>;
type Videos = HashMap<Uuid, VideoRecord>;
type Comments = HashMap<String, Vec<Comment>>;

pub struct ReplayExtraServiceImpl {
    replays: Arc<RwLock<Replays>>,
    videos: Arc<RwLock<Videos>>,
    comments: Arc<RwLock<Comments>>,
    /// share_url -> replay_id
    shares: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl ReplayExtraServiceImpl {
    pub fn new() -> Self {
        Self {
            replays: Arc::new(RwLock::new(HashMap::new())),
            videos: Arc::new(RwLock::new(HashMap::new())),
            comments: Arc::new(RwLock::new(HashMap::new())),
            shares: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save_replay(
        &self,
        player_id: &str,
        title: &str,
        duration_secs: u32,
        size_bytes: u32,
    ) -> Result<ReplayRecord> {
        if duration_secs == 0 {
            return Err(Error::InvalidRequest("duration must be > 0".into()));
        }
        let r = ReplayRecord::new(player_id, title, duration_secs, size_bytes);
        self.replays.write().await.insert(r.replay_id, r.clone());
        Ok(r)
    }

    pub async fn get_replay(&self, replay_id: Uuid) -> Result<ReplayRecord> {
        let mut replays = self.replays.write().await;
        let r = replays.get_mut(&replay_id).ok_or_else(|| Error::ReplayNotFound(replay_id.to_string()))?;
        r.add_view();
        Ok(r.clone())
    }

    pub async fn delete_replay(&self, replay_id: Uuid, player_id: &str) -> Result<()> {
        let mut replays = self.replays.write().await;
        let r = replays.get(&replay_id).ok_or_else(|| Error::ReplayNotFound(replay_id.to_string()))?;
        if r.player_id != player_id {
            return Err(Error::NotAuthorized("not owner".into()));
        }
        replays.remove(&replay_id);
        Ok(())
    }

    pub async fn share_replay(&self, replay_id: Uuid, player_id: &str, _channel: &str) -> Result<String> {
        let replays = self.replays.read().await;
        let r = replays.get(&replay_id).ok_or_else(|| Error::ReplayNotFound(replay_id.to_string()))?;
        if r.player_id != player_id {
            return Err(Error::NotAuthorized("not owner".into()));
        }
        drop(replays);
        let url = format!("https://rgs.example/share/{}", Uuid::new_v4());
        self.shares.write().await.insert(url.clone(), replay_id);
        Ok(url)
    }

    pub async fn get_shared_replay(&self, share_url: &str) -> Result<(ReplayRecord, u32)> {
        let shares = self.shares.read().await;
        let rid = shares.get(share_url).copied().ok_or_else(|| Error::ReplayNotFound(share_url.into()))?;
        drop(shares);
        let r = self.get_replay(rid).await?;
        let vc = r.view_count;
        Ok((r, vc))
    }

    pub async fn upload_video(
        &self,
        player_id: &str,
        title: &str,
        duration_secs: u32,
        size_bytes: u32,
    ) -> Result<VideoRecord> {
        if duration_secs == 0 {
            return Err(Error::InvalidRequest("duration must be > 0".into()));
        }
        let v = VideoRecord::new(player_id, title, duration_secs, size_bytes);
        self.videos.write().await.insert(v.video_id, v.clone());
        Ok(v)
    }

    pub async fn get_video(&self, video_id: Uuid) -> Result<VideoRecord> {
        let mut videos = self.videos.write().await;
        let v = videos.get_mut(&video_id).ok_or_else(|| Error::VideoNotFound(video_id.to_string()))?;
        v.view_count = v.view_count.saturating_add(1);
        Ok(v.clone())
    }

    pub async fn post_comment(&self, target_id: &str, player_id: &str, content: &str, target_type: u32) -> Result<Comment> {
        if content.is_empty() {
            return Err(Error::InvalidRequest("empty content".into()));
        }
        let c = Comment {
            comment_id: Uuid::new_v4(),
            target_id: format!("{}:{}", target_type, target_id),
            player_id: player_id.to_string(),
            content: content.to_string(),
            posted_at: chrono::Utc::now(),
        };
        let key = c.target_id.clone();
        self.comments.write().await.entry(key).or_default().push(c.clone());
        Ok(c)
    }

    pub async fn get_comments(&self, target_id: &str, target_type: u32, page: u32, page_size: u32) -> Vec<Comment> {
        let key = format!("{}:{}", target_type, target_id);
        let comments = self.comments.read().await;
        let all = comments.get(&key).cloned().unwrap_or_default();
        let start = page as usize * page_size as usize;
        all.into_iter().skip(start).take(page_size as usize).collect()
    }
}

impl Default for ReplayExtraServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn save_and_get_replay() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.save_replay("p1", "title", 60, 1024).await.unwrap();
        let g = svc.get_replay(r.replay_id).await.unwrap();
        assert_eq!(g.view_count, 1);
    }

    #[tokio::test]
    async fn save_zero_duration_fails() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.save_replay("p1", "t", 0, 1024).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn get_unknown_replay_fails() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.get_replay(Uuid::new_v4()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn delete_replay() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.save_replay("p1", "t", 60, 1024).await.unwrap();
        svc.delete_replay(r.replay_id, "p1").await.unwrap();
    }

    #[tokio::test]
    async fn delete_other_player_replay_forbidden() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.save_replay("p1", "t", 60, 1024).await.unwrap();
        let res = svc.delete_replay(r.replay_id, "p2").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn share_replay() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.save_replay("p1", "t", 60, 1024).await.unwrap();
        let url = svc.share_replay(r.replay_id, "p1", "wx").await.unwrap();
        let (g, _) = svc.get_shared_replay(&url).await.unwrap();
        assert_eq!(g.replay_id, r.replay_id);
    }

    #[tokio::test]
    async fn upload_and_get_video() {
        let svc = ReplayExtraServiceImpl::new();
        let v = svc.upload_video("p1", "t", 120, 4096).await.unwrap();
        let g = svc.get_video(v.video_id).await.unwrap();
        assert_eq!(g.view_count, 1);
    }

    #[tokio::test]
    async fn upload_zero_duration_fails() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.upload_video("p1", "t", 0, 1024).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn post_and_get_comments() {
        let svc = ReplayExtraServiceImpl::new();
        svc.post_comment("t1", "p1", "great", 1).await.unwrap();
        svc.post_comment("t1", "p2", "+1", 1).await.unwrap();
        let list = svc.get_comments("t1", 1, 0, 10).await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn post_empty_comment_fails() {
        let svc = ReplayExtraServiceImpl::new();
        let r = svc.post_comment("t1", "p1", "", 1).await;
        assert!(r.is_err());
    }
}
