//! guild-service 业务实现

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::entity::{ApplicationStatus, Guild, GuildApplication, GuildMember, GuildRole};
use crate::error::{Error, Result};

type Guilds = HashMap<Uuid, Guild>;
type Members = HashMap<Uuid, Vec<GuildMember>>;
type Apps = HashMap<Uuid, Vec<GuildApplication>>;

pub struct GuildServiceImpl {
    guilds: Arc<RwLock<Guilds>>,
    members: Arc<RwLock<Members>>,
    apps: Arc<RwLock<Apps>>,
}

impl GuildServiceImpl {
    pub fn new() -> Self {
        Self {
            guilds: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(Members::new())),
            apps: Arc::new(RwLock::new(Apps::new())),
        }
    }

    pub async fn create_guild(
        &self,
        leader_id: Uuid,
        name: &str,
        notice: &str,
        capacity: u32,
    ) -> Result<Guild> {
        if capacity == 0 || capacity > 200 {
            return Err(Error::InvalidRequest(format!("capacity {} out of range", capacity)));
        }
        let mut guilds = self.guilds.write().await;
        if guilds.values().any(|g| g.name == name) {
            return Err(Error::AlreadyInGuild(name.to_string()));
        }
        let g = Guild {
            guild_id: Uuid::new_v4(),
            name: name.to_string(),
            leader_id,
            notice: notice.to_string(),
            level: 1,
            capacity,
            created_at: chrono::Utc::now(),
        };
        guilds.insert(g.guild_id, g.clone());
        let mut members = self.members.write().await;
        members.insert(g.guild_id, vec![GuildMember::new(leader_id, "leader", GuildRole::Leader)]);
        Ok(g)
    }

    pub async fn disband_guild(&self, guild_id: Uuid, leader_id: Uuid) -> Result<()> {
        let mut guilds = self.guilds.write().await;
        let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can disband".into()));
        }
        guilds.remove(&guild_id);
        self.members.write().await.remove(&guild_id);
        Ok(())
    }

    pub async fn get_guild_info(&self, guild_id: Uuid) -> Result<Guild> {
        let guilds = self.guilds.read().await;
        guilds.get(&guild_id).cloned().ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))
    }

    pub async fn update_notice(&self, guild_id: Uuid, leader_id: Uuid, notice: &str) -> Result<()> {
        let mut guilds = self.guilds.write().await;
        let g = guilds.get_mut(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can update notice".into()));
        }
        g.notice = notice.to_string();
        Ok(())
    }

    pub async fn get_member_list(&self, guild_id: Uuid, page: u32, page_size: u32) -> Result<Vec<GuildMember>> {
        let members = self.members.read().await;
        let list = members.get(&guild_id).cloned().unwrap_or_default();
        if !self.guilds.read().await.contains_key(&guild_id) {
            return Err(Error::GuildNotFound(guild_id.to_string()));
        }
        let start = page as usize * page_size as usize;
        Ok(list.into_iter().skip(start).take(page_size as usize).collect())
    }

    pub async fn kick_member(&self, guild_id: Uuid, leader_id: Uuid, target_id: Uuid) -> Result<()> {
        let guilds = self.guilds.read().await;
        let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can kick".into()));
        }
        drop(guilds);
        let mut members = self.members.write().await;
        let m = members.get_mut(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        let before = m.len();
        m.retain(|x| x.player_id != target_id);
        if m.len() == before {
            return Err(Error::MemberNotFound(target_id.to_string()));
        }
        Ok(())
    }

    pub async fn promote_member(&self, guild_id: Uuid, leader_id: Uuid, target_id: Uuid, new_role: i32) -> Result<()> {
        let guilds = self.guilds.read().await;
        let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can promote".into()));
        }
        drop(guilds);
        let mut members = self.members.write().await;
        let m = members.get_mut(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        let mem = m.iter_mut().find(|m| m.player_id == target_id).ok_or_else(|| Error::MemberNotFound(target_id.to_string()))?;
        mem.role = GuildRole::from_i32(new_role);
        Ok(())
    }

    pub async fn leave_guild(&self, guild_id: Uuid, player_id: Uuid) -> Result<()> {
        let guilds = self.guilds.read().await;
        let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if g.leader_id == player_id {
            return Err(Error::InvalidRequest("leader must disband not leave".into()));
        }
        drop(guilds);
        let mut members = self.members.write().await;
        let m = members.get_mut(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        let before = m.len();
        m.retain(|x| x.player_id != player_id);
        if m.len() == before {
            return Err(Error::MemberNotFound(player_id.to_string()));
        }
        Ok(())
    }

    pub async fn apply_to_guild(&self, guild_id: Uuid, player_id: Uuid) -> Result<()> {
        let _ = self.get_guild_info(guild_id).await?;
        let mut apps = self.apps.write().await;
        if apps.get(&guild_id).map_or(false, |v| v.iter().any(|a| a.applicant_id == player_id && a.status == ApplicationStatus::Pending)) {
            return Err(Error::AlreadyInGuild("pending application".into()));
        }
        apps.entry(guild_id).or_default().push(GuildApplication {
            guild_id,
            applicant_id: player_id,
            applied_at: chrono::Utc::now(),
            status: ApplicationStatus::Pending,
        });
        Ok(())
    }

    pub async fn approve_application(&self, guild_id: Uuid, leader_id: Uuid, applicant_id: Uuid) -> Result<()> {
        // 1) 校验 leader 身份, clone capacity 以避免 lock 冲突
        let capacity = {
            let guilds = self.guilds.read().await;
            let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
            if g.leader_id != leader_id {
                return Err(Error::PermissionDenied("only leader can approve".into()));
            }
            g.capacity
        };
        // 2) 更新申请状态
        {
            let mut apps = self.apps.write().await;
            let a = apps.get_mut(&guild_id).ok_or_else(|| Error::InvalidRequest("no applications".into()))?
                .iter_mut().find(|a| a.applicant_id == applicant_id && a.status == ApplicationStatus::Pending)
                .ok_or_else(|| Error::InvalidRequest("no pending application".into()))?;
            a.status = ApplicationStatus::Approved;
        }
        // 3) 加入公会
        let mut members = self.members.write().await;
        let list = members.entry(guild_id).or_default();
        if list.len() as u32 >= capacity {
            return Err(Error::GuildFull(capacity));
        }
        list.push(GuildMember::new(applicant_id, "new", GuildRole::Member));
        Ok(())
    }

    pub async fn reject_application(&self, guild_id: Uuid, leader_id: Uuid, applicant_id: Uuid) -> Result<()> {
        let guilds = self.guilds.read().await;
        let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can reject".into()));
        }
        drop(guilds);
        let mut apps = self.apps.write().await;
        let a = apps.get_mut(&guild_id).ok_or_else(|| Error::InvalidRequest("no applications".into()))?
            .iter_mut().find(|a| a.applicant_id == applicant_id && a.status == ApplicationStatus::Pending)
            .ok_or_else(|| Error::InvalidRequest("no pending application".into()))?;
        a.status = ApplicationStatus::Rejected;
        Ok(())
    }

    pub async fn donate(&self, guild_id: Uuid, player_id: Uuid, _resource_type: u32, amount: u32) -> Result<u32> {
        let _ = self.get_guild_info(guild_id).await?;
        let mut members = self.members.write().await;
        let m = members.get_mut(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?
            .iter_mut().find(|m| m.player_id == player_id)
            .ok_or_else(|| Error::MemberNotFound(player_id.to_string()))?;
        m.add_contribution(amount);
        Ok(m.contribution)
    }

    pub async fn get_donation_rank(&self, guild_id: Uuid, top_n: u32) -> Result<Vec<GuildMember>> {
        let _ = self.get_guild_info(guild_id).await?;
        let members = self.members.read().await;
        let mut list: Vec<GuildMember> = members.get(&guild_id).cloned().unwrap_or_default();
        list.sort_by(|a, b| b.contribution.cmp(&a.contribution));
        Ok(list.into_iter().take(top_n as usize).collect())
    }
}

impl Default for GuildServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_guild_default() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "Test", "notice", 50).await.unwrap();
        assert_eq!(g.name, "Test");
        assert_eq!(g.leader_id, leader);
    }

    #[tokio::test]
    async fn create_duplicate_name_fails() {
        let svc = GuildServiceImpl::new();
        svc.create_guild(Uuid::new_v4(), "T", "", 50).await.unwrap();
        let r = svc.create_guild(Uuid::new_v4(), "T", "", 50).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn disband_guild() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        svc.disband_guild(g.guild_id, leader).await.unwrap();
    }

    #[tokio::test]
    async fn disband_non_leader_forbidden() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let other = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        let r = svc.disband_guild(g.guild_id, other).await;
        assert!(matches!(r, Err(Error::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn get_guild_info_unknown_fails() {
        let svc = GuildServiceImpl::new();
        let r = svc.get_guild_info(Uuid::new_v4()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn update_notice() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "old", 50).await.unwrap();
        svc.update_notice(g.guild_id, leader, "new").await.unwrap();
        let info = svc.get_guild_info(g.guild_id).await.unwrap();
        assert_eq!(info.notice, "new");
    }

    #[tokio::test]
    async fn member_list_after_create_has_leader() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        let list = svc.get_member_list(g.guild_id, 0, 10).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].player_id, leader);
    }

    #[tokio::test]
    async fn promote_member() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let other = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        // 手动加 other
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(other, "x", GuildRole::Member));
        svc.promote_member(g.guild_id, leader, other, 2).await.unwrap();
        let list = svc.get_member_list(g.guild_id, 0, 10).await.unwrap();
        let m = list.iter().find(|m| m.player_id == other).unwrap();
        assert_eq!(m.role, GuildRole::ViceLeader);
    }

    #[tokio::test]
    async fn leave_guild_removes_member() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let other = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(other, "x", GuildRole::Member));
        svc.leave_guild(g.guild_id, other).await.unwrap();
    }

    #[tokio::test]
    async fn leader_cannot_leave() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        let r = svc.leave_guild(g.guild_id, leader).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn apply_and_approve() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let applicant = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        svc.apply_to_guild(g.guild_id, applicant).await.unwrap();
        svc.approve_application(g.guild_id, leader, applicant).await.unwrap();
        let list = svc.get_member_list(g.guild_id, 0, 10).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn reject_application() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let applicant = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        svc.apply_to_guild(g.guild_id, applicant).await.unwrap();
        svc.reject_application(g.guild_id, leader, applicant).await.unwrap();
    }

    #[tokio::test]
    async fn donate_adds_contribution() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        let total = svc.donate(g.guild_id, leader, 1, 100).await.unwrap();
        assert_eq!(total, 100);
    }

    #[tokio::test]
    async fn donation_rank_sorted() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let g = svc.create_guild(leader, "T", "", 50).await.unwrap();
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(a, "a", GuildRole::Member));
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(b, "b", GuildRole::Member));
        svc.donate(g.guild_id, leader, 1, 50).await.unwrap();
        svc.donate(g.guild_id, a, 1, 200).await.unwrap();
        svc.donate(g.guild_id, b, 1, 100).await.unwrap();
        let rank = svc.get_donation_rank(g.guild_id, 3).await.unwrap();
        assert_eq!(rank[0].player_id, a);
        assert_eq!(rank[1].player_id, b);
        assert_eq!(rank[2].player_id, leader);
    }
}
