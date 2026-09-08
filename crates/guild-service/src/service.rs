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

// ===== W8 L18 scaffold start: 副本/技能/远航/公会战 4 个共享存储 =====

/// 副本运行态: (guild_id, dungeon_id) -> (boss_hp_remaining, started_at_ms, defeated)
type GuildDungeons = HashMap<(Uuid, String), GuildDungeonState>;
#[derive(Debug, Clone)]
pub struct GuildDungeonState {
    pub boss_hp_remaining: i32,
    pub started_at_ms: i64,
    pub defeated: bool,
    pub claimed_by: Vec<Uuid>,
}

/// 技能捐献累计: (guild_id, skill_id) -> (level, total_donated, last_donated_ms)
type GuildSkillDons = HashMap<(Uuid, String), GuildSkillDonState>;
#[derive(Debug, Clone)]
pub struct GuildSkillDonState {
    pub level: i32,
    pub total_donated: i32,
    pub last_donated_ms: i64,
    pub bonus_claimed_by: Vec<Uuid>,
}

/// 远航历史: (guild_id, ship_id) -> (route_id, arrived_at_ms, reward_gold, claimed_by)
type ShippingHistories = HashMap<(Uuid, String), ShippingHistoryState>;
#[derive(Debug, Clone)]
pub struct ShippingHistoryState {
    pub route_id: String,
    pub arrived_at_ms: i64,
    pub reward_gold: i32,
    pub claimed_by: Vec<Uuid>,
}

/// 公会战结果: war_id -> (winner_guild_id, ended_at_ms)
type GuildWarResults = HashMap<String, GuildWarResultState>;
#[derive(Debug, Clone)]
pub struct GuildWarResultState {
    pub winner_guild_id: String,
    pub ended_at_ms: i64,
    pub scores: HashMap<String, i32>,
}

pub struct GuildServiceImpl {
    guilds: Arc<RwLock<Guilds>>,
    members: Arc<RwLock<Members>>,
    apps: Arc<RwLock<Apps>>,
    // W8 L18 scaffold
    dungeons: Arc<RwLock<GuildDungeons>>,
    skill_dons: Arc<RwLock<GuildSkillDons>>,
    shippings: Arc<RwLock<ShippingHistories>>,
    war_results: Arc<RwLock<GuildWarResults>>,
}

impl GuildServiceImpl {
    pub fn new() -> Self {
        Self {
            guilds: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(Members::new())),
            apps: Arc::new(RwLock::new(Apps::new())),
            dungeons: Arc::new(RwLock::new(GuildDungeons::new())),
            skill_dons: Arc::new(RwLock::new(GuildSkillDons::new())),
            shippings: Arc::new(RwLock::new(ShippingHistories::new())),
            war_results: Arc::new(RwLock::new(GuildWarResults::new())),
        }
    }

    // ===== 原有 14 RPC (保留) =====

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
        let capacity = {
            let guilds = self.guilds.read().await;
            let g = guilds.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
            if g.leader_id != leader_id {
                return Err(Error::PermissionDenied("only leader can approve".into()));
            }
            g.capacity
        };
        {
            let mut apps = self.apps.write().await;
            let a = apps.get_mut(&guild_id).ok_or_else(|| Error::InvalidRequest("no applications".into()))?
                .iter_mut().find(|a| a.applicant_id == applicant_id && a.status == ApplicationStatus::Pending)
                .ok_or_else(|| Error::InvalidRequest("no pending application".into()))?;
            a.status = ApplicationStatus::Approved;
        }
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

    // ===== W8 L18 scaffold: 副本 (4) =====

    /// 静态副本配置表: dungeon_id -> (name, difficulty, boss_hp, recommended_level, duration_sec)
    fn dungeon_config(dungeon_id: &str) -> Option<(&'static str, i32, i32, i32, i32)> {
        match dungeon_id {
            "d_amber" => Some(("琥珀洞窟", 1, 1000, 5, 600)),
            "d_obsidian" => Some(("黑曜深渊", 2, 5000, 15, 1200)),
            "d_dragon" => Some(("龙巢", 3, 20000, 30, 1800)),
            _ => None,
        }
    }

    pub async fn get_guild_dungeon_info(&self, guild_id: Uuid, dungeon_id: &str) -> Result<(String, String, i32, i32, i32)> {
        let _ = self.get_guild_info(guild_id).await?;
        let cfg = Self::dungeon_config(dungeon_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown dungeon {}", dungeon_id)))?;
        Ok((dungeon_id.to_string(), cfg.0.to_string(), cfg.1, cfg.2, cfg.3))
    }

    pub async fn start_guild_dungeon(&self, guild_id: Uuid, dungeon_id: &str, leader_id: Uuid) -> Result<(String, i64, i32, i32)> {
        let g = self.get_guild_info(guild_id).await?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can start dungeon".into()));
        }
        let cfg = Self::dungeon_config(dungeon_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown dungeon {}", dungeon_id)))?;
        let now = chrono::Utc::now().timestamp_millis();
        let mut dungeons = self.dungeons.write().await;
        dungeons.insert(
            (guild_id, dungeon_id.to_string()),
            GuildDungeonState {
                boss_hp_remaining: cfg.2,
                started_at_ms: now,
                defeated: false,
                claimed_by: Vec::new(),
            },
        );
        Ok((dungeon_id.to_string(), now, cfg.2, cfg.3))
    }

    pub async fn report_guild_dungeon_progress(&self, guild_id: Uuid, dungeon_id: &str, boss_hp_remaining: i32) -> Result<(i32, bool, i64)> {
        let _ = self.get_guild_info(guild_id).await?;
        let mut dungeons = self.dungeons.write().await;
        let s = dungeons.get_mut(&(guild_id, dungeon_id.to_string()))
            .ok_or_else(|| Error::InvalidRequest("dungeon not started".into()))?;
        s.boss_hp_remaining = boss_hp_remaining.max(0);
        if boss_hp_remaining <= 0 {
            s.defeated = true;
        }
        let now = chrono::Utc::now().timestamp_millis();
        Ok((s.boss_hp_remaining, s.defeated, now))
    }

    pub async fn claim_guild_dungeon_reward(&self, guild_id: Uuid, dungeon_id: &str, player_id: Uuid) -> Result<(i32, i32, i32, bool)> {
        let _ = self.get_guild_info(guild_id).await?;
        let cfg = Self::dungeon_config(dungeon_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown dungeon {}", dungeon_id)))?;
        let mut dungeons = self.dungeons.write().await;
        let s = dungeons.get_mut(&(guild_id, dungeon_id.to_string()))
            .ok_or_else(|| Error::InvalidRequest("dungeon not started".into()))?;
        if !s.defeated {
            return Err(Error::InvalidRequest("dungeon not defeated".into()));
        }
        if s.claimed_by.contains(&player_id) {
            return Err(Error::AlreadyInGuild("already claimed".into()));
        }
        s.claimed_by.push(player_id);
        let gold = cfg.2;
        let exp = cfg.1 * 100;
        let item_count = cfg.1 * 2;
        Ok((gold, exp, item_count, true))
    }

    // ===== W8 L18 scaffold: 技能 (4) =====

    /// 静态技能配置: skill_id -> (name, max_level, base_cost_gold, base_cost_contribution)
    fn skill_config(skill_id: &str) -> Option<(&'static str, i32, i32, i32)> {
        match skill_id {
            "s_attack" => Some(("攻击强化", 10, 1000, 500)),
            "s_defense" => Some(("防御强化", 10, 1000, 500)),
            "s_luck" => Some(("幸运强化", 5, 5000, 2000)),
            _ => None,
        }
    }

    pub async fn get_guild_skills(&self, guild_id: Uuid) -> Result<Vec<(String, String, i32, i32, i32, i32)>> {
        let _ = self.get_guild_info(guild_id).await?;
        let dons = self.skill_dons.read().await;
        let mut out = Vec::new();
        for (skill_id, cfg) in [("s_attack", Self::skill_config("s_attack").unwrap()),
                               ("s_defense", Self::skill_config("s_defense").unwrap()),
                               ("s_luck", Self::skill_config("s_luck").unwrap())] {
            let level = dons.get(&(guild_id, skill_id.to_string())).map(|s| s.level).unwrap_or(0);
            let cost_gold = cfg.2 + level * cfg.2;
            let cost_contrib = cfg.3 + level * cfg.3;
            out.push((skill_id.to_string(), cfg.0.to_string(), level, cfg.1, cost_gold, cost_contrib));
        }
        Ok(out)
    }

    pub async fn upgrade_guild_skill(&self, guild_id: Uuid, skill_id: &str, leader_id: Uuid) -> Result<(String, i32, i32, i32)> {
        let g = self.get_guild_info(guild_id).await?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can upgrade skill".into()));
        }
        let cfg = Self::skill_config(skill_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown skill {}", skill_id)))?;
        let mut dons = self.skill_dons.write().await;
        let entry = dons.entry((guild_id, skill_id.to_string())).or_insert(GuildSkillDonState {
            level: 0,
            total_donated: 0,
            last_donated_ms: 0,
            bonus_claimed_by: Vec::new(),
        });
        if entry.level >= cfg.1 {
            return Err(Error::InvalidRequest("skill already max level".into()));
        }
        entry.level += 1;
        let new_level = entry.level;
        let cost_gold = cfg.2 + (new_level - 1) * cfg.2;
        let cost_contrib = cfg.3 + (new_level - 1) * cfg.3;
        Ok((skill_id.to_string(), new_level, cost_gold, cost_contrib))
    }

    pub async fn donate_to_guild_skill(&self, guild_id: Uuid, skill_id: &str, amount: i32, player_id: Uuid) -> Result<(String, i32, i32)> {
        let _ = self.get_guild_info(guild_id).await?;
        if amount <= 0 {
            return Err(Error::InvalidRequest("amount must be positive".into()));
        }
        let _ = Self::skill_config(skill_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown skill {}", skill_id)))?;
        // player must be a member
        let members = self.members.read().await;
        let mlist = members.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if !mlist.iter().any(|m| m.player_id == player_id) {
            return Err(Error::MemberNotFound(player_id.to_string()));
        }
        let mut dons = self.skill_dons.write().await;
        let entry = dons.entry((guild_id, skill_id.to_string())).or_insert(GuildSkillDonState {
            level: 0,
            total_donated: 0,
            last_donated_ms: 0,
            bonus_claimed_by: Vec::new(),
        });
        entry.total_donated = entry.total_donated.saturating_add(amount);
        entry.level = (entry.total_donated / 1000).min(10);
        entry.last_donated_ms = chrono::Utc::now().timestamp_millis();
        let new_level = entry.level;
        let total = entry.total_donated;
        Ok((skill_id.to_string(), new_level, total))
    }

    pub async fn claim_guild_skill_bonus(&self, guild_id: Uuid, skill_id: &str, player_id: Uuid) -> Result<(i32, i32, bool)> {
        let _ = self.get_guild_info(guild_id).await?;
        let cfg = Self::skill_config(skill_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown skill {}", skill_id)))?;
        let mut dons = self.skill_dons.write().await;
        let s = dons.get_mut(&(guild_id, skill_id.to_string()))
            .ok_or_else(|| Error::InvalidRequest("skill has no donations yet".into()))?;
        if s.level < 1 {
            return Err(Error::InvalidRequest("skill not leveled".into()));
        }
        if s.bonus_claimed_by.contains(&player_id) {
            return Err(Error::AlreadyInGuild("bonus already claimed".into()));
        }
        s.bonus_claimed_by.push(player_id);
        let gold = s.level * cfg.2;
        let exp = s.level * 100;
        Ok((gold, exp, true))
    }

    // ===== W8 L18 scaffold: 远航 (4) =====

    fn route_config(route_id: &str) -> Option<(&'static str, i32, i32, i32)> {
        match route_id {
            "r_near" => Some(("近海航线", 100, 600, 500)),
            "r_mid" => Some(("远海航线", 500, 1800, 2000)),
            "r_far" => Some(("深海航线", 2000, 3600, 10000)),
            _ => None,
        }
    }

    pub async fn get_shipping_routes(&self) -> Vec<(String, String, i32, i32, i32)> {
        let mut out = Vec::new();
        for (id, cfg) in [("r_near", Self::route_config("r_near").unwrap()),
                          ("r_mid", Self::route_config("r_mid").unwrap()),
                          ("r_far", Self::route_config("r_far").unwrap())] {
            out.push((id.to_string(), cfg.0.to_string(), cfg.1, cfg.2, cfg.3));
        }
        out
    }

    pub async fn start_shipping(&self, guild_id: Uuid, route_id: &str, leader_id: Uuid) -> Result<(String, i64, i32)> {
        let g = self.get_guild_info(guild_id).await?;
        if g.leader_id != leader_id {
            return Err(Error::PermissionDenied("only leader can start shipping".into()));
        }
        let cfg = Self::route_config(route_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown route {}", route_id)))?;
        let ship_id = format!("ship-{}", Uuid::new_v4());
        let arrival_ms = chrono::Utc::now().timestamp_millis() + (cfg.2 as i64) * 1000;
        let mut shippings = self.shippings.write().await;
        shippings.insert(
            (guild_id, ship_id.clone()),
            ShippingHistoryState {
                route_id: route_id.to_string(),
                arrived_at_ms: arrival_ms,
                reward_gold: cfg.3,
                claimed_by: Vec::new(),
            },
        );
        Ok((ship_id, arrival_ms, cfg.2))
    }

    pub async fn claim_shipping_reward(&self, guild_id: Uuid, ship_id: &str, player_id: Uuid) -> Result<(i32, i32, bool)> {
        let _ = self.get_guild_info(guild_id).await?;
        let mut shippings = self.shippings.write().await;
        let s = shippings.get_mut(&(guild_id, ship_id.to_string()))
            .ok_or_else(|| Error::InvalidRequest("ship not found".into()))?;
        if s.claimed_by.contains(&player_id) {
            return Err(Error::AlreadyInGuild("shipping reward already claimed".into()));
        }
        s.claimed_by.push(player_id);
        let reward = s.reward_gold / (s.claimed_by.len() as i32).max(1);
        let item_count = 1;
        Ok((reward, item_count, true))
    }

    pub async fn list_shipping_history(&self, guild_id: Uuid, top_n: u32) -> Result<Vec<(String, String, i32, i64)>> {
        let _ = self.get_guild_info(guild_id).await?;
        let shippings = self.shippings.read().await;
        let mut rows: Vec<(String, String, i32, i64)> = shippings
            .iter()
            .filter(|((g, _), _)| *g == guild_id)
            .map(|((_, ship_id), s)| (ship_id.clone(), s.route_id.clone(), s.reward_gold, s.arrived_at_ms))
            .collect();
        rows.sort_by(|a, b| b.3.cmp(&a.3));
        Ok(rows.into_iter().take(top_n as usize).collect())
    }

    // ===== W8 L18 scaffold: 公会战 (4) =====

    fn war_schedule() -> Vec<(String, String, i64, i32, &'static str)> {
        let now = chrono::Utc::now().timestamp_millis();
        vec![
            ("w_001".to_string(), "黎明之战".to_string(), now + 3_600_000, 8, "scheduled"),
            ("w_002".to_string(), "黄昏之战".to_string(), now + 7_200_000, 16, "scheduled"),
            ("w_003".to_string(), "永夜之战".to_string(), now + 86_400_000, 32, "scheduled"),
        ]
    }

    pub async fn get_guild_war_schedule(&self) -> Vec<(String, String, i64, i32, String)> {
        Self::war_schedule().into_iter().map(|(a, b, c, d, e)| (a, b, c, d, e.to_string())).collect()
    }

    pub async fn join_guild_war(&self, guild_id: Uuid, war_id: &str, player_id: Uuid) -> Result<(String, i32, i64)> {
        let _ = self.get_guild_info(guild_id).await?;
        let schedule = Self::war_schedule();
        let war = schedule.iter().find(|(wid, _, _, _, _)| wid == war_id)
            .ok_or_else(|| Error::InvalidRequest(format!("unknown war {}", war_id)))?;
        let members = self.members.read().await;
        let mlist = members.get(&guild_id).ok_or_else(|| Error::GuildNotFound(guild_id.to_string()))?;
        if !mlist.iter().any(|m| m.player_id == player_id) {
            return Err(Error::MemberNotFound(player_id.to_string()));
        }
        let team_id = format!("{}-{}", war_id, &guild_id.to_string()[..8]);
        let team_size = mlist.len() as i32;
        let joined_at_ms = chrono::Utc::now().timestamp_millis();
        // war used to silence unused warning
        let _ = war;
        Ok((team_id, team_size, joined_at_ms))
    }

    pub async fn report_guild_war_result(&self, war_id: &str, winner_guild_id: &str) -> Result<(bool, String, String, i64)> {
        let schedule = Self::war_schedule();
        if !schedule.iter().any(|(wid, _, _, _, _)| wid == war_id) {
            return Err(Error::InvalidRequest(format!("unknown war {}", war_id)));
        }
        let now = chrono::Utc::now().timestamp_millis();
        let mut results = self.war_results.write().await;
        let prev_score = results.get(war_id).map(|s| s.scores.get(winner_guild_id).copied().unwrap_or(0)).unwrap_or(0);
        let entry = results.entry(war_id.to_string()).or_insert_with(|| GuildWarResultState {
            winner_guild_id: winner_guild_id.to_string(),
            ended_at_ms: now,
            scores: HashMap::new(),
        });
        entry.winner_guild_id = winner_guild_id.to_string();
        entry.ended_at_ms = now;
        entry.scores.insert(winner_guild_id.to_string(), prev_score + 1);
        Ok((true, war_id.to_string(), winner_guild_id.to_string(), now))
    }

    pub async fn get_guild_war_leaderboard(&self, war_id: &str, top_n: u32) -> Result<Vec<(i32, String, String, i32)>> {
        let schedule = Self::war_schedule();
        if !schedule.iter().any(|(wid, _, _, _, _)| wid == war_id) {
            return Err(Error::InvalidRequest(format!("unknown war {}", war_id)));
        }
        let results = self.war_results.read().await;
        let mut rows: Vec<(String, String, i32)> = match results.get(war_id) {
            Some(s) => s.scores.iter().map(|(g, sc)| (g.clone(), g.clone(), *sc)).collect(),
            None => Vec::new(),
        };
        rows.sort_by(|a, b| b.2.cmp(&a.2));
        Ok(rows.into_iter().take(top_n as usize).enumerate()
            .map(|(i, (_, g, sc))| ((i + 1) as i32, g.clone(), g.clone(), sc))
            .collect())
    }
}

impl Default for GuildServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 原有 14 RPC 测试 (保留) =====

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

    // ===== W8 L18 scaffold tests (≥30) =====

    // --- Dungeon 4 ---
    #[tokio::test]
    async fn w8_dungeon_info_known() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        let r = svc.get_guild_dungeon_info(g.guild_id, "d_amber").await.unwrap();
        assert_eq!(r.0, "d_amber");
        assert_eq!(r.2, 1); // difficulty
        assert!(r.3 > 0); // boss_hp
    }

    #[tokio::test]
    async fn w8_dungeon_info_unknown_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        let r = svc.get_guild_dungeon_info(g.guild_id, "d_bogus").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_dungeon_start_and_report_defeat() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        let (_id, _ts, boss_hp, _dur) = svc.start_guild_dungeon(g.guild_id, "d_amber", leader).await.unwrap();
        assert!(boss_hp > 0);
        let (hp, defeated, _ts) = svc.report_guild_dungeon_progress(g.guild_id, "d_amber", 0).await.unwrap();
        assert_eq!(hp, 0);
        assert!(defeated);
    }

    #[tokio::test]
    async fn w8_dungeon_start_non_leader_forbidden() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let other = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        let r = svc.start_guild_dungeon(g.guild_id, "d_amber", other).await;
        assert!(matches!(r, Err(Error::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn w8_dungeon_report_before_start_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        let r = svc.report_guild_dungeon_progress(g.guild_id, "d_amber", 100).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_dungeon_claim_after_defeat() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        svc.start_guild_dungeon(g.guild_id, "d_amber", leader).await.unwrap();
        svc.report_guild_dungeon_progress(g.guild_id, "d_amber", 0).await.unwrap();
        let player = Uuid::new_v4();
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(player, "p", GuildRole::Member));
        let (gold, _exp, _ic, claimed) = svc.claim_guild_dungeon_reward(g.guild_id, "d_amber", player).await.unwrap();
        assert!(claimed);
        assert!(gold > 0);
    }

    #[tokio::test]
    async fn w8_dungeon_claim_before_defeat_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        svc.start_guild_dungeon(g.guild_id, "d_amber", leader).await.unwrap();
        let player = Uuid::new_v4();
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(player, "p", GuildRole::Member));
        let r = svc.claim_guild_dungeon_reward(g.guild_id, "d_amber", player).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_dungeon_double_claim_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "D", "", 50).await.unwrap();
        svc.start_guild_dungeon(g.guild_id, "d_amber", leader).await.unwrap();
        svc.report_guild_dungeon_progress(g.guild_id, "d_amber", 0).await.unwrap();
        let player = Uuid::new_v4();
        svc.members.write().await.get_mut(&g.guild_id).unwrap().push(GuildMember::new(player, "p", GuildRole::Member));
        svc.claim_guild_dungeon_reward(g.guild_id, "d_amber", player).await.unwrap();
        let r = svc.claim_guild_dungeon_reward(g.guild_id, "d_amber", player).await;
        assert!(r.is_err());
    }

    // --- Skill 5 ---
    #[tokio::test]
    async fn w8_skill_get_all_default_zero() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let skills = svc.get_guild_skills(g.guild_id).await.unwrap();
        assert_eq!(skills.len(), 3);
        assert!(skills.iter().all(|s| s.2 == 0));
    }

    #[tokio::test]
    async fn w8_skill_upgrade_increments_level() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let (_id, lv, _g, _c) = svc.upgrade_guild_skill(g.guild_id, "s_attack", leader).await.unwrap();
        assert_eq!(lv, 1);
    }

    #[tokio::test]
    async fn w8_skill_upgrade_non_leader_forbidden() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let other = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let r = svc.upgrade_guild_skill(g.guild_id, "s_attack", other).await;
        assert!(matches!(r, Err(Error::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn w8_skill_upgrade_unknown_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let r = svc.upgrade_guild_skill(g.guild_id, "s_bogus", leader).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_skill_donate_increases_level() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let (_id, lv, total) = svc.donate_to_guild_skill(g.guild_id, "s_attack", 2500, leader).await.unwrap();
        assert_eq!(total, 2500);
        assert_eq!(lv, 2);
    }

    #[tokio::test]
    async fn w8_skill_donate_non_member_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let outsider = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let r = svc.donate_to_guild_skill(g.guild_id, "s_attack", 100, outsider).await;
        assert!(matches!(r, Err(Error::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn w8_skill_donate_zero_or_negative_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let r = svc.donate_to_guild_skill(g.guild_id, "s_attack", 0, leader).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_skill_claim_bonus_after_upgrade() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        svc.upgrade_guild_skill(g.guild_id, "s_attack", leader).await.unwrap();
        let (gold, _exp, claimed) = svc.claim_guild_skill_bonus(g.guild_id, "s_attack", leader).await.unwrap();
        assert!(claimed);
        assert!(gold > 0);
    }

    #[tokio::test]
    async fn w8_skill_claim_bonus_before_level_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        let r = svc.claim_guild_skill_bonus(g.guild_id, "s_attack", leader).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_skill_claim_bonus_double_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "S", "", 50).await.unwrap();
        svc.upgrade_guild_skill(g.guild_id, "s_attack", leader).await.unwrap();
        svc.claim_guild_skill_bonus(g.guild_id, "s_attack", leader).await.unwrap();
        let r = svc.claim_guild_skill_bonus(g.guild_id, "s_attack", leader).await;
        assert!(r.is_err());
    }

    // --- Shipping 5 ---
    #[tokio::test]
    async fn w8_shipping_routes_listed() {
        let svc = GuildServiceImpl::new();
        let routes = svc.get_shipping_routes().await;
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    async fn w8_shipping_start_creates_ship() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "SH", "", 50).await.unwrap();
        let (ship_id, arrival, _dur) = svc.start_shipping(g.guild_id, "r_near", leader).await.unwrap();
        assert!(ship_id.starts_with("ship-"));
        assert!(arrival > chrono::Utc::now().timestamp_millis());
    }

    #[tokio::test]
    async fn w8_shipping_start_non_leader_forbidden() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let other = Uuid::new_v4();
        let g = svc.create_guild(leader, "SH", "", 50).await.unwrap();
        let r = svc.start_shipping(g.guild_id, "r_near", other).await;
        assert!(matches!(r, Err(Error::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn w8_shipping_claim_after_start() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "SH", "", 50).await.unwrap();
        let (ship_id, _, _) = svc.start_shipping(g.guild_id, "r_near", leader).await.unwrap();
        let (gold, _ic, claimed) = svc.claim_shipping_reward(g.guild_id, &ship_id, leader).await.unwrap();
        assert!(claimed);
        assert!(gold > 0);
    }

    #[tokio::test]
    async fn w8_shipping_claim_unknown_ship_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "SH", "", 50).await.unwrap();
        let r = svc.claim_shipping_reward(g.guild_id, "ship-bogus", leader).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_shipping_history_sorted() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "SH", "", 50).await.unwrap();
        let (s1, _, _) = svc.start_shipping(g.guild_id, "r_near", leader).await.unwrap();
        let (s2, _, _) = svc.start_shipping(g.guild_id, "r_mid", leader).await.unwrap();
        let rows = svc.list_shipping_history(g.guild_id, 5).await.unwrap();
        assert_eq!(rows.len(), 2);
        let ids: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
        assert!(ids.contains(&s1));
        assert!(ids.contains(&s2));
    }

    // --- War 6 ---
    #[tokio::test]
    async fn w8_war_schedule_non_empty() {
        let svc = GuildServiceImpl::new();
        let sched = svc.get_guild_war_schedule().await;
        assert!(sched.len() >= 3);
    }

    #[tokio::test]
    async fn w8_war_join_member() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "W", "", 50).await.unwrap();
        let (team_id, size, _ts) = svc.join_guild_war(g.guild_id, "w_001", leader).await.unwrap();
        assert!(team_id.contains("w_001"));
        assert_eq!(size, 1);
    }

    #[tokio::test]
    async fn w8_war_join_non_member_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let outsider = Uuid::new_v4();
        let g = svc.create_guild(leader, "W", "", 50).await.unwrap();
        let r = svc.join_guild_war(g.guild_id, "w_001", outsider).await;
        assert!(matches!(r, Err(Error::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn w8_war_join_unknown_war_fails() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "W", "", 50).await.unwrap();
        let r = svc.join_guild_war(g.guild_id, "w_bogus", leader).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_war_report_result() {
        let svc = GuildServiceImpl::new();
        let leader = Uuid::new_v4();
        let g = svc.create_guild(leader, "W", "", 50).await.unwrap();
        let (ok, war_id, winner, _ts) = svc.report_guild_war_result("w_001", &g.guild_id.to_string()).await.unwrap();
        assert!(ok);
        assert_eq!(war_id, "w_001");
        assert_eq!(winner, g.guild_id.to_string());
    }

    #[tokio::test]
    async fn w8_war_report_unknown_war_fails() {
        let svc = GuildServiceImpl::new();
        let r = svc.report_guild_war_result("w_bogus", "g1").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn w8_war_leaderboard_after_report() {
        let svc = GuildServiceImpl::new();
        let leader1 = Uuid::new_v4();
        let leader2 = Uuid::new_v4();
        let g1 = svc.create_guild(leader1, "A", "", 50).await.unwrap();
        let g2 = svc.create_guild(leader2, "B", "", 50).await.unwrap();
        svc.report_guild_war_result("w_001", &g1.guild_id.to_string()).await.unwrap();
        svc.report_guild_war_result("w_001", &g2.guild_id.to_string()).await.unwrap();
        let rows = svc.get_guild_war_leaderboard("w_001", 5).await.unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn w8_war_leaderboard_unknown_war_fails() {
        let svc = GuildServiceImpl::new();
        let r = svc.get_guild_war_leaderboard("w_bogus", 5).await;
        assert!(r.is_err());
    }
}
