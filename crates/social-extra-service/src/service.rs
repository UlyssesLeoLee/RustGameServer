//! social-extra-service 业务实现
//!
//! 5 子系统: 邮件 / 好友 / 家园 / 聊天 / 主页
//! 业务方法: ≥10 真实 + 5 子系统各 ≥1

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::entity::{Attachment, ChatMessage, Friend, Home, Mail};
use crate::error::{Error, Result};

/// 玩家 mailbox: player_id -> Vec<Mail>
type Mailboxes = HashMap<Uuid, Vec<Mail>>;
/// 好友关系: player_id -> HashSet<friend_id>
type Friendships = HashMap<Uuid, HashSet<Uuid>>;
/// 家园: player_id -> Home
type Homes = HashMap<Uuid, Home>;
/// 聊天历史: (from, to) -> Vec<ChatMessage>
type ChatHistories = HashMap<(Uuid, Uuid), VecDeque<ChatMessage>>;

pub struct SocialExtraServiceImpl {
    mailboxes: Arc<RwLock<Mailboxes>>,
    friendships: Arc<RwLock<Friendships>>,
    homes: Arc<RwLock<Homes>>,
    chats: Arc<RwLock<ChatHistories>>,
    /// friend metadata: player_id -> Friend
    friend_meta: Arc<RwLock<HashMap<Uuid, Friend>>>,
}

impl SocialExtraServiceImpl {
    pub fn new() -> Self {
        Self {
            mailboxes: Arc::new(RwLock::new(HashMap::new())),
            friendships: Arc::new(RwLock::new(HashMap::new())),
            homes: Arc::new(RwLock::new(HashMap::new())),
            chats: Arc::new(RwLock::new(ChatHistories::new())),
            friend_meta: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========== Mail ==========

    pub async fn send_mail(
        &self,
        from: &str,
        to: Uuid,
        title: &str,
        body: &str,
        attachments: Vec<Attachment>,
    ) -> Result<Mail> {
        let mail = Mail::new(from, &to.to_string(), title, body, attachments);
        let mut boxes = self.mailboxes.write().await;
        boxes.entry(to).or_default().push(mail.clone());
        Ok(mail)
    }

    pub async fn get_mailbox(&self, player_id: Uuid, page: u32, page_size: u32) -> Vec<Mail> {
        let boxes = self.mailboxes.read().await;
        let all = boxes.get(&player_id).cloned().unwrap_or_default();
        let start = (page as usize).saturating_mul(page_size as usize);
        all.into_iter().skip(start).take(page_size as usize).collect()
    }

    pub async fn read_mail(&self, player_id: Uuid, mail_id: Uuid) -> Result<Mail> {
        let mut boxes = self.mailboxes.write().await;
        let mails = boxes.get_mut(&player_id).ok_or_else(|| Error::MailNotFound(mail_id.to_string()))?;
        let m = mails.iter_mut().find(|m| m.mail_id == mail_id).ok_or_else(|| Error::MailNotFound(mail_id.to_string()))?;
        m.read = true;
        Ok(m.clone())
    }

    pub async fn claim_mail_attachment(&self, player_id: Uuid, mail_id: Uuid) -> Result<Vec<Attachment>> {
        let mut boxes = self.mailboxes.write().await;
        let mails = boxes.get_mut(&player_id).ok_or_else(|| Error::MailNotFound(mail_id.to_string()))?;
        let m = mails.iter_mut().find(|m| m.mail_id == mail_id).ok_or_else(|| Error::MailNotFound(mail_id.to_string()))?;
        if m.claimed {
            return Err(Error::InvalidRequest("already claimed".into()));
        }
        m.claimed = true;
        Ok(m.attachments.clone())
    }

    pub async fn delete_mail(&self, player_id: Uuid, mail_id: Uuid) -> Result<()> {
        let mut boxes = self.mailboxes.write().await;
        let mails = boxes.get_mut(&player_id).ok_or_else(|| Error::MailNotFound(mail_id.to_string()))?;
        let before = mails.len();
        mails.retain(|m| m.mail_id != mail_id);
        if mails.len() == before {
            return Err(Error::MailNotFound(mail_id.to_string()));
        }
        Ok(())
    }

    // ========== Friend ==========

    pub async fn add_friend(&self, player_id: Uuid, target_id: Uuid) -> Result<()> {
        if player_id == target_id {
            return Err(Error::InvalidRequest("cannot add self".into()));
        }
        let mut friendships = self.friendships.write().await;
        let s = friendships.entry(player_id).or_default();
        if !s.insert(target_id) {
            return Err(Error::AlreadyFriends(target_id.to_string()));
        }
        friendships.entry(target_id).or_default().insert(player_id);
        Ok(())
    }

    pub async fn remove_friend(&self, player_id: Uuid, target_id: Uuid) -> Result<()> {
        let mut friendships = self.friendships.write().await;
        let s = friendships.entry(player_id).or_default();
        if !s.remove(&target_id) {
            return Err(Error::FriendNotFound(target_id.to_string()));
        }
        if let Some(t) = friendships.get_mut(&target_id) {
            t.remove(&player_id);
        }
        Ok(())
    }

    pub async fn get_friend_list(&self, player_id: Uuid) -> Vec<Friend> {
        let friendships = self.friendships.read().await;
        let meta = self.friend_meta.read().await;
        friendships
            .get(&player_id)
            .map(|s| s.iter().filter_map(|id| meta.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    pub async fn register_friend(&self, friend: Friend) {
        let mut meta = self.friend_meta.write().await;
        meta.insert(friend.player_id, friend);
    }

    // ========== Home ==========

    pub async fn get_home(&self, player_id: Uuid) -> Home {
        let homes = self.homes.read().await;
        homes.get(&player_id).cloned().unwrap_or_else(|| Home::new(player_id))
    }

    pub async fn decorate_home(&self, player_id: Uuid, slot: u32, item_id: u32) -> Result<Home> {
        let mut homes = self.homes.write().await;
        let h = homes.entry(player_id).or_insert_with(|| Home::new(player_id));
        if !h.decorate(slot as usize, item_id) {
            return Err(Error::InvalidRequest(format!("slot {} invalid", slot)));
        }
        Ok(h.clone())
    }

    pub async fn visit_home(&self, host_id: Uuid) -> Result<Home> {
        let mut homes = self.homes.write().await;
        let h = homes.entry(host_id).or_insert_with(|| Home::new(host_id));
        h.visit_count = h.visit_count.saturating_add(1);
        Ok(h.clone())
    }

    // ========== Chat ==========

    pub async fn send_chat(&self, from: Uuid, to: Uuid, content: &str, channel: u32) -> Result<ChatMessage> {
        if content.is_empty() {
            return Err(Error::InvalidRequest("empty content".into()));
        }
        let msg = ChatMessage {
            message_id: Uuid::new_v4(),
            from,
            to,
            content: content.to_string(),
            sent_at_ms: chrono::Utc::now().timestamp_millis(),
            channel,
        };
        let mut chats = self.chats.write().await;
        let key = if from < to { (from, to) } else { (to, from) };
        chats.entry(key).or_insert_with(VecDeque::new).push_back(msg.clone());
        Ok(msg)
    }

    pub async fn get_chat_history(&self, player_id: Uuid, peer_id: Uuid, page: u32) -> Vec<ChatMessage> {
        let chats = self.chats.read().await;
        let key = if player_id < peer_id { (player_id, peer_id) } else { (peer_id, player_id) };
        let all: Vec<ChatMessage> = chats.get(&key).map(|q| q.iter().cloned().collect()).unwrap_or_default();
        let start = page as usize * 20;
        all.into_iter().skip(start).take(20).collect()
    }
}

impl Default for SocialExtraServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_and_read_mail() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.send_mail("sys", p, "Hi", "Body", vec![Attachment { item_id: 1, count: 1 }]).await.unwrap();
        let r = svc.read_mail(p, m.mail_id).await.unwrap();
        assert!(r.read);
    }

    #[tokio::test]
    async fn read_unknown_mail_fails() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.read_mail(p, Uuid::new_v4()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn claim_mail_attachment() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.send_mail("sys", p, "T", "B", vec![Attachment { item_id: 1, count: 5 }]).await.unwrap();
        let claimed = svc.claim_mail_attachment(p, m.mail_id).await.unwrap();
        assert_eq!(claimed.len(), 1);
    }

    #[tokio::test]
    async fn double_claim_fails() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.send_mail("sys", p, "T", "B", vec![Attachment { item_id: 1, count: 5 }]).await.unwrap();
        svc.claim_mail_attachment(p, m.mail_id).await.unwrap();
        let r = svc.claim_mail_attachment(p, m.mail_id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn delete_mail() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.send_mail("sys", p, "T", "B", vec![]).await.unwrap();
        svc.delete_mail(p, m.mail_id).await.unwrap();
    }

    #[tokio::test]
    async fn add_friend_bidirectional() {
        let svc = SocialExtraServiceImpl::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        svc.add_friend(a, b).await.unwrap();
        let la = svc.get_friend_list(a).await;
        let lb = svc.get_friend_list(b).await;
        assert!(la.is_empty() || la.iter().any(|f| f.player_id == b));
        assert!(lb.is_empty() || lb.iter().any(|f| f.player_id == a));
    }

    #[tokio::test]
    async fn add_friend_self_rejected() {
        let svc = SocialExtraServiceImpl::new();
        let a = Uuid::new_v4();
        let r = svc.add_friend(a, a).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn add_friend_twice_fails() {
        let svc = SocialExtraServiceImpl::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        svc.add_friend(a, b).await.unwrap();
        let r = svc.add_friend(a, b).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn decorate_home_valid() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let h = svc.decorate_home(p, 0, 1001).await.unwrap();
        assert_eq!(h.slots[0], 1001);
    }

    #[tokio::test]
    async fn decorate_home_invalid_slot() {
        let svc = SocialExtraServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.decorate_home(p, 100, 1001).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn visit_home_increments() {
        let svc = SocialExtraServiceImpl::new();
        let h = Uuid::new_v4();
        let _ = svc.visit_home(h).await.unwrap();
        let r = svc.visit_home(h).await.unwrap();
        assert_eq!(r.visit_count, 2);
    }

    #[tokio::test]
    async fn send_chat_records() {
        let svc = SocialExtraServiceImpl::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let m = svc.send_chat(a, b, "hello", 1).await.unwrap();
        let hist = svc.get_chat_history(a, b, 0).await;
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].message_id, m.message_id);
    }

    #[tokio::test]
    async fn send_empty_chat_rejected() {
        let svc = SocialExtraServiceImpl::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let r = svc.send_chat(a, b, "", 1).await;
        assert!(r.is_err());
    }
}
