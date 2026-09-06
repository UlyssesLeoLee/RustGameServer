//! gm-extra-service 业务实现

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::entity::{AuditEntry, BanRecord, BanScope};
use crate::error::{Error, Result};

const MAX_AUDIT_LOG: usize = 1000;

type Bans = HashMap<String, Vec<BanRecord>>;
type Audit = VecDeque<AuditEntry>;

pub struct GmExtraServiceImpl {
    bans: Arc<RwLock<Bans>>,
    audit: Arc<RwLock<Audit>>,
    world_level: Arc<RwLock<u32>>,
    /// 模拟 GM 角色表
    gm_roles: HashMap<String, GmRole>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GmRole {
    Viewer,
    Operator,
    SuperAdmin,
}

impl GmRole {
    pub fn can_ban(&self) -> bool {
        matches!(self, GmRole::SuperAdmin)
    }
    pub fn can_kick(&self) -> bool {
        matches!(self, GmRole::Operator | GmRole::SuperAdmin)
    }
    pub fn can_set_world_level(&self) -> bool {
        matches!(self, GmRole::SuperAdmin)
    }
}

impl GmExtraServiceImpl {
    pub fn new() -> Self {
        let mut gm_roles = HashMap::new();
        gm_roles.insert("gm_admin".to_string(), GmRole::SuperAdmin);
        gm_roles.insert("gm_op".to_string(), GmRole::Operator);
        Self {
            bans: Arc::new(RwLock::new(HashMap::new())),
            audit: Arc::new(RwLock::new(VecDeque::new())),
            world_level: Arc::new(RwLock::new(1)),
            gm_roles,
        }
    }

    fn check_gm(&self, gm_id: &str, required: GmRole) -> Result<()> {
        let role = self.gm_roles.get(gm_id).copied().unwrap_or(GmRole::Viewer);
        if (role as u8) >= (required as u8) {
            Ok(())
        } else {
            Err(Error::PermissionDenied(format!("gm {} lacks role {:?}", gm_id, required)))
        }
    }

    async fn record_audit(&self, gm_id: &str, command: &str, target_id: &str, result: &str) {
        let mut audit = self.audit.write().await;
        audit.push_back(AuditEntry::new(gm_id, command, target_id, result));
        while audit.len() > MAX_AUDIT_LOG {
            audit.pop_front();
        }
    }

    pub async fn ban_account(&self, gm_id: &str, player_id: &str, reason: &str, duration_secs: i64) -> Result<String> {
        self.check_gm(gm_id, GmRole::SuperAdmin)?;
        if duration_secs <= 0 {
            return Err(Error::InvalidRequest("duration must be > 0".into()));
        }
        let rec = BanRecord::new(gm_id, player_id, reason, BanScope::All, duration_secs);
        let bid = rec.ban_id.to_string();
        self.bans.write().await.entry(player_id.to_string()).or_default().push(rec);
        self.record_audit(gm_id, "ban_account", player_id, "ok").await;
        Ok(bid)
    }

    pub async fn unban_account(&self, gm_id: &str, player_id: &str) -> Result<()> {
        self.check_gm(gm_id, GmRole::SuperAdmin)?;
        let mut bans = self.bans.write().await;
        let list = bans.get_mut(player_id).ok_or_else(|| Error::PlayerNotFound(player_id.into()))?;
        let count = list.iter().filter(|b| b.active).count();
        if count == 0 {
            return Err(Error::PlayerNotFound(player_id.into()));
        }
        for b in list.iter_mut() {
            b.active = false;
        }
        self.record_audit(gm_id, "unban_account", player_id, "ok").await;
        Ok(())
    }

    pub async fn mute_player(&self, gm_id: &str, player_id: &str, channel: &str, duration_secs: i64) -> Result<()> {
        self.check_gm(gm_id, GmRole::Operator)?;
        if duration_secs <= 0 {
            return Err(Error::InvalidRequest("duration must be > 0".into()));
        }
        self.bans.write().await.entry(player_id.to_string()).or_default()
            .push(BanRecord::new(gm_id, player_id, &format!("mute {}", channel), BanScope::Chat, duration_secs));
        self.record_audit(gm_id, "mute_player", player_id, "ok").await;
        Ok(())
    }

    pub async fn unmute_player(&self, gm_id: &str, player_id: &str, channel: &str) -> Result<()> {
        self.check_gm(gm_id, GmRole::Operator)?;
        self.record_audit(gm_id, "unmute_player", player_id, channel).await;
        Ok(())
    }

    pub async fn kick_player(&self, gm_id: &str, player_id: &str, reason: &str) -> Result<()> {
        self.check_gm(gm_id, GmRole::Operator)?;
        if reason.is_empty() {
            return Err(Error::InvalidRequest("reason required".into()));
        }
        self.record_audit(gm_id, "kick_player", player_id, reason).await;
        Ok(())
    }

    pub async fn set_world_level(&self, gm_id: &str, new_level: u32) -> Result<()> {
        self.check_gm(gm_id, GmRole::SuperAdmin)?;
        if new_level == 0 || new_level > 200 {
            return Err(Error::InvalidRequest(format!("level {} out of range", new_level)));
        }
        *self.world_level.write().await = new_level;
        self.record_audit(gm_id, "set_world_level", "", &new_level.to_string()).await;
        Ok(())
    }

    pub async fn get_world_level(&self) -> u32 {
        *self.world_level.read().await
    }

    pub async fn broadcast(&self, gm_id: &str, content: &str, channel: u32, repeat: u32) -> Result<()> {
        self.check_gm(gm_id, GmRole::Operator)?;
        if content.is_empty() {
            return Err(Error::InvalidRequest("content required".into()));
        }
        self.record_audit(gm_id, "broadcast", &format!("ch{}x{}", channel, repeat), "ok").await;
        Ok(())
    }

    pub async fn get_audit_log(&self, _gm_id: &str, page: u32, page_size: u32) -> Vec<AuditEntry> {
        let audit = self.audit.read().await;
        let start = page as usize * page_size as usize;
        audit.iter().rev().skip(start).take(page_size as usize).cloned().collect()
    }
}

impl Default for GmExtraServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ban_requires_superadmin() {
        let svc = GmExtraServiceImpl::new();
        let r = svc.ban_account("gm_op", "p1", "x", 3600).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn ban_account_succeeds() {
        let svc = GmExtraServiceImpl::new();
        let bid = svc.ban_account("gm_admin", "p1", "x", 3600).await.unwrap();
        assert!(!bid.is_empty());
    }

    #[tokio::test]
    async fn ban_duration_zero_fails() {
        let svc = GmExtraServiceImpl::new();
        let r = svc.ban_account("gm_admin", "p1", "x", 0).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn unban_requires_superadmin() {
        let svc = GmExtraServiceImpl::new();
        svc.ban_account("gm_admin", "p1", "x", 3600).await.unwrap();
        let r = svc.unban_account("gm_op", "p1").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn unban_account() {
        let svc = GmExtraServiceImpl::new();
        svc.ban_account("gm_admin", "p1", "x", 3600).await.unwrap();
        svc.unban_account("gm_admin", "p1").await.unwrap();
    }

    #[tokio::test]
    async fn mute_player() {
        let svc = GmExtraServiceImpl::new();
        svc.mute_player("gm_op", "p1", "world", 60).await.unwrap();
    }

    #[tokio::test]
    async fn mute_zero_duration_fails() {
        let svc = GmExtraServiceImpl::new();
        let r = svc.mute_player("gm_op", "p1", "world", 0).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn kick_requires_reason() {
        let svc = GmExtraServiceImpl::new();
        let r = svc.kick_player("gm_op", "p1", "").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn set_world_level() {
        let svc = GmExtraServiceImpl::new();
        svc.set_world_level("gm_admin", 50).await.unwrap();
        assert_eq!(svc.get_world_level().await, 50);
    }

    #[tokio::test]
    async fn set_world_level_out_of_range() {
        let svc = GmExtraServiceImpl::new();
        let r = svc.set_world_level("gm_admin", 999).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn broadcast_empty_fails() {
        let svc = GmExtraServiceImpl::new();
        let r = svc.broadcast("gm_op", "", 1, 1).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn broadcast_succeeds() {
        let svc = GmExtraServiceImpl::new();
        svc.broadcast("gm_op", "Hello", 1, 1).await.unwrap();
    }

    #[tokio::test]
    async fn audit_log_records_actions() {
        let svc = GmExtraServiceImpl::new();
        svc.ban_account("gm_admin", "p1", "x", 3600).await.unwrap();
        let log = svc.get_audit_log("gm_admin", 0, 10).await;
        assert!(!log.is_empty());
    }
}
