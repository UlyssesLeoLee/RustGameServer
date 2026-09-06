//! player-service 域 Service 业务实施（per RGS-DTL-018 §3 + DTL-038 §4.3 卡牌 v2）
//!
//! 54.7 实化：
//! - 4 Service 业务方法（register / heartbeat / update_profile / disable_player）
//! - ServiceImpl 接 PlayerRepository + PlayerSessionRepository（Arc<dyn>）
//! - PlayerServiceImpl 直接暴露 find_by_id（gRPC GetPlayer 用，绕开 trait）
//! - gRPC 桥接：impl player_proto::player_service_server::PlayerService for PlayerGrpcService
//!   接 HealthCheck + GetPlayer（per 54.2 proto 定义）
//!
//! 桶 11 增量（per DTL-038 §4.3 + §7.1 + FR-001/FR-002 + DEC-038-01）：
//! - 7 业务方法（create_deck / get_deck / update_deck / delete_deck / list_decks / share_deck / get_shared_deck）
//! - ServiceImpl 加 DeckRepository（Arc<dyn>）
//! - 业务层校验占位（30-60 张, 同卡 ≤ 2 张; 规则引擎未实装, 留 TODO）
//! - saga 占位（per DTL-038 §6 抽卡 / 交易 saga 不在本桶; deck 业务无 saga 需求, 仅需 outbox 通知）

use crate::entity::{
    Character, CharacterAssetsSnapshot, Deck, DeckSlot, DeckStatus, Player, PlayerProfile,
    PlayerSession, PlayerStatus,
};
use crate::error::Error;
use crate::repository::{
    CharacterRepository, DeckRepository, PageRequest, PlayerRepository, PlayerSessionRepository,
};
use crate::Result;
use chrono::Utc;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// player-service 域 Service trait（业务层，gRPC 桥接在 grpc_service 模块）
#[async_trait]
pub trait PlayerService: Send + Sync {
    /// 健康检查
    async fn health_check(&self) -> Result<bool>;

    /// 注册新玩家（unique 昵称检查）
    async fn register(&self, name: String) -> Result<Player>;

    /// 心跳（滑动 session 过期）
    async fn heartbeat(&self, session_id: Uuid) -> Result<PlayerSession>;

    /// 更新档案（等级 / vip）
    async fn update_profile(
        &self,
        player_id: Uuid,
        level: Option<i32>,
        vip_level: Option<i32>,
    ) -> Result<Player>;

    /// 封禁 / 停用
    async fn disable_player(&self, player_id: Uuid, reason: String) -> Result<Player>;

    // ----- v2 卡牌游戏业务方法 (per DTL-038 §4.3 + FR-001/FR-002, 桶 11 增量) -----

    /// 读取卡牌游戏玩家档案 (per FR-001)
    async fn get_player_profile(&self, player_id: Uuid) -> Result<PlayerProfile>;

    /// 更新卡牌游戏玩家档案 (per FR-001)
    async fn update_player_profile(&self, profile: PlayerProfile) -> Result<PlayerProfile>;

    /// 创建卡组 (per FR-002)
    async fn create_deck(&self, owner_id: Uuid, name: String, mode: i32) -> Result<Deck>;

    /// 读取单个卡组
    async fn get_deck(&self, deck_id: Uuid) -> Result<Deck>;

    /// 更新卡组（仅 owner 可改）
    async fn update_deck(
        &self,
        deck_id: Uuid,
        owner_id: Uuid,
        name: Option<String>,
        slots: Option<Vec<DeckSlot>>,
    ) -> Result<Deck>;

    /// 删除卡组（仅 owner 可删）
    async fn delete_deck(&self, deck_id: Uuid, owner_id: Uuid) -> Result<bool>;

    /// 分页列出某玩家所有卡组
    async fn list_decks(&self, owner_id: Uuid, page_req: PageRequest) -> Result<(Vec<Deck>, i64)>;

    /// 开启/取消分享
    async fn share_deck(
        &self,
        deck_id: Uuid,
        owner_id: Uuid,
        make_public: bool,
    ) -> Result<Deck>;

    /// 通过 share_code 拉取公开卡组
    async fn get_shared_deck(&self, share_code: String) -> Result<Deck>;

    // ========================================================================
    // 闪烁之光 100% 兼容 Phase 2 — 账号+角色 15 RPC (per 9/5 11:50 JST 4 拍板)
    // 上游参考: 闪烁之光 proto_101.erl (101xx) + proto_103.erl (103xx)
    // 桶 12 增量: 不破坏既有 11 个方法, 全部 15 个为新加, 5 真实逻辑 + 10 stub
    // ========================================================================

    // ----- 角色生命周期 (10101-10103) — 5 真实逻辑 -----

    /// 10101 创建角色 (per proto_101.erl)
    async fn create_character(
        &self,
        account_id: Uuid,
        name: String,
        class_id: i32,
        faction_id: i32,
        device_id: String,
        client_ip: String,
    ) -> Result<(Character, PlayerSession)>;

    /// 10102 登录指定角色 (per proto_101.erl)
    async fn login_character(
        &self,
        account_id: Uuid,
        character_id: Uuid,
        device_id: String,
        client_ip: String,
    ) -> Result<(Character, PlayerSession)>;

    /// 10103 角色重连 (per proto_101.erl)
    async fn reconnect_character(
        &self,
        session_id: Uuid,
        client_ip: String,
    ) -> Result<(Character, PlayerSession)>;

    /// 10301 获取角色基础信息 (per proto_103.erl)
    async fn get_character_profile(&self, character_id: Uuid) -> Result<PlayerProfile>;

    /// 10302 角色资产信息 (per proto_103.erl, 跨域 economy 占位)
    async fn get_character_assets(
        &self,
        character_id: Uuid,
    ) -> Result<CharacterAssetsSnapshot>;

    // ----- 10315 查看角色信息 — stub -----

    async fn get_character_info(&self, character_id: Uuid) -> Result<Character>;

    // ----- 10343 个人改名 — stub -----

    async fn rename_character(&self, character_id: Uuid, new_name: String) -> Result<Character>;

    // ----- 10380 注册时间与开服时间 — stub -----

    async fn get_server_time(&self) -> Result<(i64, i64, String)>; // (now, server_open, tz)

    // ----- 10394 服务端通知游客模式超时 — stub -----

    async fn guest_mode_timeout(
        &self,
        character_id: Uuid,
        timeout_seconds: i32,
    ) -> Result<(bool, chrono::DateTime<Utc>)>;

    // ----- 10395 客户端通知查验身份认证（防沉迷） — stub -----

    async fn anti_addiction_check(
        &self,
        character_id: Uuid,
        is_adult: bool,
    ) -> Result<(bool, i32)>; // (is_adult_confirmed, max_play_minutes)

    // ----- 10396 强制关闭客户端 — stub -----

    async fn force_disconnect(
        &self,
        character_id: Uuid,
        reason: String,
    ) -> Result<(bool, Option<Uuid>)>; // (issued, session_id)

    // ----- 10397 客户端进入后台 — stub -----

    async fn enter_background(
        &self,
        character_id: Uuid,
        session_id: Uuid,
    ) -> Result<(bool, chrono::DateTime<Utc>)>;

    // ----- 10325 头像列表 — stub -----

    async fn get_avatar_list(&self, character_id: Uuid) -> Result<Vec<AvatarStub>>;

    // ----- 10327 设置头像 — stub -----

    async fn set_avatar(&self, character_id: Uuid, avatar_id: i32) -> Result<i32>;

    // ----- 11000 心跳 (RGS 新增) — stub -----

    async fn heartbeat_rpc(
        &self,
        session_id: Uuid,
        character_id: Option<Uuid>,
        client_time_unix: i64,
    ) -> Result<(bool, i64, chrono::DateTime<Utc>)>; // (ok, server_time, expires_at)
}

/// 头像 stub (per 10325 头像列表)
/// 真实图片资源由 CDN 服务承载, player-service 仅保留 ID + 拥有状态.
#[derive(Debug, Clone)]
pub struct AvatarStub {
    pub avatar_id: i32,
    pub name: String,
    pub url: String,
    pub owned: bool,
    pub unlock_by_default: bool,
}

/// player-service 默认实现
pub struct PlayerServiceImpl {
    players: Arc<dyn PlayerRepository>,
    sessions: Arc<dyn PlayerSessionRepository>,
    decks: Arc<dyn DeckRepository>,
    characters: Arc<dyn CharacterRepository>,
}

impl PlayerServiceImpl {
    /// 5 参构造: 完整接入 PlayerRepository + PlayerSessionRepository + DeckRepository + CharacterRepository
    pub fn new(
        players: Arc<dyn PlayerRepository>,
        sessions: Arc<dyn PlayerSessionRepository>,
        decks: Arc<dyn DeckRepository>,
        characters: Arc<dyn CharacterRepository>,
    ) -> Self {
        Self {
            players,
            sessions,
            decks,
            characters,
        }
    }

    /// gRPC GetPlayer 用：直接通过 Repository 查（绕开 trait）
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Player>> {
        tracing::debug!(
            operation = "db_query_before",
            service = "player-service",
            method = "find_by_id",
            player_id = %id,
            "query player by id"
        );
        let res = self.players.find_by_id(id).await;
        tracing::debug!(
            operation = "db_query_after",
            service = "player-service",
            method = "find_by_id",
            player_id = %id,
            found = res.as_ref().map(|o| o.is_some()).unwrap_or(false),
            "query player by id done"
        );
        res
    }

    /// 业务层卡组 slots 校验（per DTL-038 §4.3 + §9.1 P2 规则引擎占位）
    ///
    /// 当前桶 11 仅占位: 返回空 errors. 规则引擎（30-60 张, 同卡 ≤ 2 张, 稀有度上限等）
    /// 由后续 game-logic crate 实装. 桶 11 任务书明确"不实装规则引擎".
    pub fn validate_deck_slots(_slots: &[DeckSlot]) -> Vec<String> {
        // TODO(per DTL-038 §9.1 P2): 实装规则引擎
        //   - 总卡数 30-60 (per 业务规则)
        //   - 同卡 count ≤ 2 (per 业务规则)
        //   - 稀有度上限 (per 业务规则)
        //   - 跨系列平衡 (per 业务规则)
        Vec::new()
    }
}

#[async_trait]
impl PlayerService for PlayerServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn register(&self, name: String) -> Result<Player> {
        if name.trim().is_empty() {
            return Err(Error::Validation("name must not be empty".to_string()));
        }
        if name.len() > 64 {
            return Err(Error::Validation("name too long (max 64)".to_string()));
        }
        // unique 昵称检查
        if self.players.find_by_name(&name).await?.is_some() {
            return Err(Error::NicknameTaken(name));
        }
        let player = Player::new(name);
        self.players.save(&player).await?;
        Ok(player)
    }

    async fn heartbeat(&self, session_id: Uuid) -> Result<PlayerSession> {
        let mut session = self
            .sessions
            .find_by_id(session_id)
            .await?
            .ok_or(Error::SessionExpired)?;
        if session.is_expired() {
            return Err(Error::SessionExpired);
        }
        session.heartbeat();
        self.sessions.save(&session).await?;
        Ok(session)
    }

    async fn update_profile(
        &self,
        player_id: Uuid,
        level: Option<i32>,
        vip_level: Option<i32>,
    ) -> Result<Player> {
        let mut player =
            self.players
                .find_by_id(player_id)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "Player",
                    id: player_id.to_string(),
                })?;

        if !is_active_for_update(&player) {
            return Err(Error::AccountDisabled(player.name));
        }
        if let Some(l) = level {
            if !(1..=999).contains(&l) {
                return Err(Error::Validation(format!("level {} out of range 1-999", l)));
            }
            player.level = l;
        }
        if let Some(v) = vip_level {
            if !(0..=20).contains(&v) {
                return Err(Error::Validation(format!(
                    "vip_level {} out of range 0-20",
                    v
                )));
            }
            player.vip_level = v;
        }
        player.updated_at = chrono::Utc::now();
        self.players.save(&player).await?;
        Ok(player)
    }

    async fn disable_player(&self, player_id: Uuid, reason: String) -> Result<Player> {
        let mut player =
            self.players
                .find_by_id(player_id)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "Player",
                    id: player_id.to_string(),
                })?;
        player.status = PlayerStatus::Disabled;
        player.updated_at = chrono::Utc::now();
        self.players.save(&player).await?;
        tracing::info!(target: "player-service", player_id = %player_id, reason = %reason, "player disabled");
        Ok(player)
    }

    // ----- v2 卡牌游戏 RPC handler 实现 (per DTL-038 §4.3, 桶 11 增量) -----

    async fn get_player_profile(&self, player_id: Uuid) -> Result<PlayerProfile> {
        // 桶 11 占位: profile 业务表 (player_profiles) 尚未实装,
        // 返默认档案 + 从 player 域读取 (确保 player 存在)
        let _ = self
            .players
            .find_by_id(player_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Player",
                id: player_id.to_string(),
            })?;
        // TODO(DTL-038 §7.2): player_profiles 表实装后, 走 DB 查询
        Ok(PlayerProfile::new(player_id))
    }

    async fn update_player_profile(&self, profile: PlayerProfile) -> Result<PlayerProfile> {
        // 验证 player 存在
        self.players.find_by_id(profile.player_id).await?.ok_or_else(|| Error::NotFound {
            entity: "Player",
            id: profile.player_id.to_string(),
        })?;
        // TODO(DTL-038 §7.2): player_profiles 表实装后, 持久化 + 审计
        tracing::info!(
            target: "player-service",
            player_id = %profile.player_id,
            ranked_score = profile.ranked_score,
            total_matches = profile.total_matches,
            "player profile updated (placeholder)"
        );
        Ok(profile)
    }

    async fn create_deck(&self, owner_id: Uuid, name: String, mode: i32) -> Result<Deck> {
        // 参数校验
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::Validation("deck name must not be empty".to_string()));
        }
        if name.len() > 64 {
            return Err(Error::Validation("deck name too long (max 64)".to_string()));
        }
        if crate::entity::GameMode::from_i32(mode).is_none() {
            return Err(Error::Validation(format!("invalid mode: {}", mode)));
        }
        // 验证 player 存在
        self.players.find_by_id(owner_id).await?.ok_or_else(|| Error::NotFound {
            entity: "Player",
            id: owner_id.to_string(),
        })?;
        // 业务规则占位 (per DTL-038 §9.1 P2 规则引擎 TODO)
        let _validation_errors = Self::validate_deck_slots(&[]);

        let deck = Deck::new(owner_id, name, mode);
        let saved = self.decks.create(&deck).await?;
        tracing::info!(
            target: "player-service",
            deck_id = %saved.id,
            owner_id = %owner_id,
            mode = mode,
            "deck created"
        );
        Ok(saved)
    }

    async fn get_deck(&self, deck_id: Uuid) -> Result<Deck> {
        self.decks
            .find_by_id(deck_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Deck",
                id: deck_id.to_string(),
            })
    }

    async fn update_deck(
        &self,
        deck_id: Uuid,
        owner_id: Uuid,
        name: Option<String>,
        slots: Option<Vec<DeckSlot>>,
    ) -> Result<Deck> {
        let mut deck = self
            .decks
            .find_by_id(deck_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Deck",
                id: deck_id.to_string(),
            })?;
        // 权限校验: 仅 owner 可改
        if deck.owner_id != owner_id {
            return Err(Error::Forbidden(format!(
                "deck {} not owned by player {}",
                deck_id, owner_id
            )));
        }
        if let Some(n) = name {
            let n = n.trim().to_string();
            if n.is_empty() {
                return Err(Error::Validation("deck name must not be empty".to_string()));
            }
            if n.len() > 64 {
                return Err(Error::Validation("deck name too long (max 64)".to_string()));
            }
            deck.name = n;
        }
        if let Some(s) = slots {
            // 业务规则占位 (per DTL-038 §9.1 P2 规则引擎 TODO)
            let _validation_errors = Self::validate_deck_slots(&s);
            deck.slots = s;
        }
        deck.updated_at = chrono::Utc::now();
        let saved = self.decks.update(&deck).await?;
        tracing::info!(
            target: "player-service",
            deck_id = %deck_id,
            owner_id = %owner_id,
            "deck updated"
        );
        Ok(saved)
    }

    async fn delete_deck(&self, deck_id: Uuid, owner_id: Uuid) -> Result<bool> {
        let deck = self
            .decks
            .find_by_id(deck_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Deck",
                id: deck_id.to_string(),
            })?;
        if deck.owner_id != owner_id {
            return Err(Error::Forbidden(format!(
                "deck {} not owned by player {}",
                deck_id, owner_id
            )));
        }
        let deleted = self.decks.delete_by_id(deck_id).await?;
        tracing::info!(
            target: "player-service",
            deck_id = %deck_id,
            owner_id = %owner_id,
            deleted = deleted,
            "deck deleted"
        );
        Ok(deleted)
    }

    async fn list_decks(&self, owner_id: Uuid, page_req: PageRequest) -> Result<(Vec<Deck>, i64)> {
        let page = self.decks.list_by_owner(owner_id, page_req).await?;
        Ok((page.items, page.total))
    }

    async fn share_deck(
        &self,
        deck_id: Uuid,
        owner_id: Uuid,
        make_public: bool,
    ) -> Result<Deck> {
        let mut deck = self
            .decks
            .find_by_id(deck_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Deck",
                id: deck_id.to_string(),
            })?;
        if deck.owner_id != owner_id {
            return Err(Error::Forbidden(format!(
                "deck {} not owned by player {}",
                deck_id, owner_id
            )));
        }
        if make_public {
            // 开启分享: 生成 share_code (UUIDv4 string, 确保唯一)
            deck.is_public = true;
            if deck.share_code.is_none() {
                deck.share_code = Some(Uuid::new_v4().to_string());
            }
        } else {
            // 取消分享
            deck.is_public = false;
            deck.share_code = None;
        }
        deck.updated_at = chrono::Utc::now();
        let saved = self.decks.update(&deck).await?;
        tracing::info!(
            target: "player-service",
            deck_id = %deck_id,
            owner_id = %owner_id,
            is_public = saved.is_public,
            share_code = saved.share_code.as_deref().unwrap_or("-"),
            "deck share state updated"
        );
        Ok(saved)
    }

    async fn get_shared_deck(&self, share_code: String) -> Result<Deck> {
        if share_code.trim().is_empty() {
            return Err(Error::Validation("share_code must not be empty".to_string()));
        }
        self.decks
            .find_by_share_code(&share_code)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Deck",
                id: format!("share_code={}", share_code),
            })
    }

    // ========================================================================
    // 桶 12 增量: 闪烁之光 账号+角色 15 RPC impl (per 9/5 11:50 JST 4 拍板)
    // 5 真实逻辑: create_character / login_character / reconnect_character /
    //             get_character_profile / get_character_assets
    // 10 stub (per §卡住的应对): 返回 Unimplemented 风格 (per 业务逻辑占位)
    // ========================================================================

    async fn create_character(
        &self,
        account_id: Uuid,
        name: String,
        class_id: i32,
        faction_id: i32,
        device_id: String,
        client_ip: String,
    ) -> Result<(Character, PlayerSession)> {
        // 1. 参数校验
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::Validation("character name must not be empty".to_string()));
        }
        if name.len() > 64 {
            return Err(Error::Validation("character name too long (max 64)".to_string()));
        }
        if !(1..=5).contains(&class_id) {
            return Err(Error::Validation(format!(
                "class_id {} out of range 1-5",
                class_id
            )));
        }
        if !(1..=3).contains(&faction_id) {
            return Err(Error::Validation(format!(
                "faction_id {} out of range 1-3",
                faction_id
            )));
        }
        if device_id.trim().is_empty() {
            return Err(Error::Validation("device_id must not be empty".to_string()));
        }
        if client_ip.trim().is_empty() {
            return Err(Error::Validation("client_ip must not be empty".to_string()));
        }
        // 2. 校验账号存在 (per RGS 抽象: account = player)
        let player = self
            .players
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Player",
                id: account_id.to_string(),
            })?;
        if !is_player_active_for_character(&player) {
            return Err(Error::AccountDisabled(player.name));
        }
        // 3. unique 角色名检查
        if self.characters.find_by_name(&name).await?.is_some() {
            return Err(Error::NicknameTaken(name));
        }
        // 4. 1 账号 1 角色 (v0.1) — 防重复创建
        if self.characters.find_by_account_id(account_id).await?.is_some() {
            return Err(Error::Conflict(format!(
                "account {} already has a character",
                account_id
            )));
        }
        // 5. 创建角色
        let character = Character::new(account_id, name.clone(), class_id, faction_id);
        let saved_char = self.characters.create(&character).await?;
        // 6. 创建会话 (创建后默认登录)
        let session = PlayerSession::new(account_id, device_id, client_ip);
        let saved_session = self.sessions.save(&session).await?;
        // 7. 更新 last_login_at
        let mut updated_char = saved_char.clone();
        updated_char.last_login_at = Some(chrono::Utc::now());
        updated_char.updated_at = chrono::Utc::now();
        let final_char = self.characters.update(&updated_char).await?;
        tracing::info!(
            target: "player-service",
            character_id = %final_char.id,
            account_id = %account_id,
            name = %name,
            class_id = class_id,
            faction_id = faction_id,
            session_id = %saved_session.id,
            "character created and logged in"
        );
        Ok((final_char, saved_session))
    }

    async fn login_character(
        &self,
        account_id: Uuid,
        character_id: Uuid,
        device_id: String,
        client_ip: String,
    ) -> Result<(Character, PlayerSession)> {
        // 1. 参数校验
        if device_id.trim().is_empty() {
            return Err(Error::Validation("device_id must not be empty".to_string()));
        }
        if client_ip.trim().is_empty() {
            return Err(Error::Validation("client_ip must not be empty".to_string()));
        }
        // 2. 校验账号
        let player = self
            .players
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Player",
                id: account_id.to_string(),
            })?;
        if !is_player_active_for_character(&player) {
            return Err(Error::AccountDisabled(player.name));
        }
        // 3. 校验角色 + 归属
        let mut character = self
            .characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        if character.account_id != account_id {
            return Err(Error::Forbidden(format!(
                "character {} not owned by account {}",
                character_id, account_id
            )));
        }
        if !character.is_active() {
            return Err(Error::AccountDisabled(character.name));
        }
        // 4. 创建新 session
        let session = PlayerSession::new(account_id, device_id, client_ip);
        let saved_session = self.sessions.save(&session).await?;
        // 5. 更新 last_login_at + in_background=false
        character.last_login_at = Some(chrono::Utc::now());
        character.in_background = false;
        character.updated_at = chrono::Utc::now();
        let final_char = self.characters.update(&character).await?;
        tracing::info!(
            target: "player-service",
            character_id = %character_id,
            account_id = %account_id,
            session_id = %saved_session.id,
            "character logged in"
        );
        Ok((final_char, saved_session))
    }

    async fn reconnect_character(
        &self,
        session_id: Uuid,
        client_ip: String,
    ) -> Result<(Character, PlayerSession)> {
        // 1. 校验 session
        let mut session = self
            .sessions
            .find_by_id(session_id)
            .await?
            .ok_or(Error::SessionExpired)?;
        if session.is_expired() {
            return Err(Error::SessionExpired);
        }
        // 2. 校验 client_ip (跨设备防御: 不一致则警告, 不阻止)
        if !client_ip.trim().is_empty() && client_ip != session.ip {
            tracing::warn!(
                target: "player-service",
                session_id = %session_id,
                old_ip = %session.ip,
                new_ip = %client_ip,
                "reconnect IP changed (suspicious)"
            );
        }
        // 3. 找角色: 通过 session.player_id 查 character (v0.1 1 账号 1 角色)
        let mut character = self
            .characters
            .find_by_account_id(session.player_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: format!("account={}", session.player_id),
            })?;
        // 4. 滑动 session + 角色心跳
        session.heartbeat();
        let saved_session = self.sessions.save(&session).await?;
        character.in_background = false;
        character.updated_at = chrono::Utc::now();
        let final_char = self.characters.update(&character).await?;
        tracing::info!(
            target: "player-service",
            session_id = %session_id,
            character_id = %final_char.id,
            new_expires_at = %saved_session.expires_at,
            "character reconnected"
        );
        Ok((final_char, saved_session))
    }

    async fn get_character_profile(&self, character_id: Uuid) -> Result<PlayerProfile> {
        // 校验角色存在
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        // 桶 12 占位: profile 业务表 (player_profiles) 尚未实装,
        // 返默认档案 + 从 character 域读取
        Ok(PlayerProfile::new(character_id))
    }

    async fn get_character_assets(
        &self,
        character_id: Uuid,
    ) -> Result<CharacterAssetsSnapshot> {
        // 校验角色存在
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        // 桶 12 占位: 跨域 economy-service 余额实装后, 走 gRPC 调用.
        // 当前返空 entries, 字段为 0 余额.
        Ok(CharacterAssetsSnapshot {
            character_id,
            entries: vec![],
            queried_at: chrono::Utc::now(),
        })
    }

    // ----- 10315 查看角色信息 — stub 占位 -----

    async fn get_character_info(&self, character_id: Uuid) -> Result<Character> {
        // stub 占位: 当前仅返回 character entity (基础信息).
        // v0.2 评估: 含 VIP 等级 / 防沉迷状态 / 头像 URL 等.
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })
    }

    // ----- 10343 个人改名 — stub 占位 -----

    async fn rename_character(&self, character_id: Uuid, new_name: String) -> Result<Character> {
        // stub 占位: 仅做基础校验 + 简单 update.
        // 真实业务: 走改名卡 / 改名费用 / 冷却时间 (per 闪烁之光 10343).
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            return Err(Error::Validation("new_name must not be empty".to_string()));
        }
        if new_name.len() > 64 {
            return Err(Error::Validation("new_name too long (max 64)".to_string()));
        }
        if self.characters.find_by_name(&new_name).await?.is_some() {
            return Err(Error::NicknameTaken(new_name));
        }
        let mut character = self
            .characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        character.name = new_name;
        character.updated_at = chrono::Utc::now();
        self.characters.update(&character).await
    }

    // ----- 10380 注册时间与开服时间 — stub 占位 -----

    async fn get_server_time(&self) -> Result<(i64, i64, String)> {
        // stub 占位: 当前返回 now + 固定开服时间 2026-01-01 + Asia/Tokyo.
        // 真实业务: 开服时间由 admin-service 配置, TZ 由 ops 配置.
        let now = chrono::Utc::now().timestamp();
        let server_open = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00+00:00")
            .unwrap()
            .timestamp();
        Ok((now, server_open, "Asia/Tokyo".to_string()))
    }

    // ----- 10394 服务端通知游客模式超时 — stub 占位 -----

    async fn guest_mode_timeout(
        &self,
        character_id: Uuid,
        timeout_seconds: i32,
    ) -> Result<(bool, chrono::DateTime<Utc>)> {
        // stub 占位: 仅校验 + 计算 deadline.
        // 真实业务: 游客模式配额管理 (per 闪烁之光 10394).
        if !(60..=86400).contains(&timeout_seconds) {
            return Err(Error::Validation(format!(
                "timeout_seconds {} out of range 60-86400",
                timeout_seconds
            )));
        }
        // 校验角色存在
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        let deadline = chrono::Utc::now() + chrono::Duration::seconds(timeout_seconds as i64);
        Ok((true, deadline))
    }

    // ----- 10395 防沉迷查验 — stub 占位 -----

    async fn anti_addiction_check(
        &self,
        character_id: Uuid,
        is_adult: bool,
    ) -> Result<(bool, i32)> {
        // stub 占位: 简化为客户端自报. 真实业务: 服务端做身份证 hash 验证
        // (per 国家新闻出版署 防沉迷规定, 桶 12 v0.2 实装).
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        let max_play_minutes = if is_adult { 0 } else { 90 }; // 18+ 不限, 未成年 90min/日
        Ok((is_adult, max_play_minutes))
    }

    // ----- 10396 强制关闭客户端 — stub 占位 -----

    async fn force_disconnect(
        &self,
        character_id: Uuid,
        reason: String,
    ) -> Result<(bool, Option<Uuid>)> {
        // stub 占位: 仅返回 issued=true + 标 None (无活跃 session).
        // 真实业务: 走 admin 域 RBAC 校验 + 找活跃 session + 推 NATS 消息.
        if reason.trim().is_empty() {
            return Err(Error::Validation("reason must not be empty".to_string()));
        }
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        tracing::warn!(
            target: "player-service",
            character_id = %character_id,
            reason = %reason,
            "force disconnect issued (stub, no session_id)"
        );
        Ok((true, None))
    }

    // ----- 10397 客户端进入后台 — stub 占位 -----

    async fn enter_background(
        &self,
        character_id: Uuid,
        session_id: Uuid,
    ) -> Result<(bool, chrono::DateTime<Utc>)> {
        // stub 占位: 仅校验 + 标 in_background.
        let session = self
            .sessions
            .find_by_id(session_id)
            .await?
            .ok_or(Error::SessionExpired)?;
        if session.player_id != character_id && self.characters.find_by_id(character_id).await?.map(|c| c.account_id) != Some(session.player_id) {
            return Err(Error::Forbidden(format!(
                "session {} not bound to character {}",
                session_id, character_id
            )));
        }
        let mut character = self
            .characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        character.in_background = true;
        character.updated_at = chrono::Utc::now();
        self.characters.update(&character).await?;
        Ok((true, character.updated_at))
    }

    // ----- 10325 头像列表 — stub 占位 -----

    async fn get_avatar_list(&self, character_id: Uuid) -> Result<Vec<AvatarStub>> {
        // stub 占位: 硬编码 3 个默认头像.
        // 真实业务: 走 master data (avatar_service / shared-platform).
        self.characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        Ok(vec![
            AvatarStub {
                avatar_id: 1,
                name: "默认头像".to_string(),
                url: "https://cdn.rgs.example.com/avatars/1.png".to_string(),
                owned: true,
                unlock_by_default: true,
            },
            AvatarStub {
                avatar_id: 2,
                name: "战士".to_string(),
                url: "https://cdn.rgs.example.com/avatars/2.png".to_string(),
                owned: true,
                unlock_by_default: true,
            },
            AvatarStub {
                avatar_id: 3,
                name: "法师".to_string(),
                url: "https://cdn.rgs.example.com/avatars/3.png".to_string(),
                owned: false,
                unlock_by_default: false,
            },
        ])
    }

    // ----- 10327 设置头像 — stub 占位 -----

    async fn set_avatar(&self, character_id: Uuid, avatar_id: i32) -> Result<i32> {
        // stub 占位: 校验 avatar_id ∈ [1,3] (硬编码 3 个头像) + 更新 character.
        if !(1..=3).contains(&avatar_id) {
            return Err(Error::Validation(format!(
                "avatar_id {} out of range 1-3",
                avatar_id
            )));
        }
        let mut character = self
            .characters
            .find_by_id(character_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Character",
                id: character_id.to_string(),
            })?;
        character.current_avatar_id = avatar_id;
        character.updated_at = chrono::Utc::now();
        self.characters.update(&character).await?;
        Ok(avatar_id)
    }

    // ----- 11000 心跳 (RGS 新增) — stub 占位 -----

    async fn heartbeat_rpc(
        &self,
        session_id: Uuid,
        character_id: Option<Uuid>,
        _client_time_unix: i64,
    ) -> Result<(bool, i64, chrono::DateTime<Utc>)> {
        // stub 占位: 滑动 session + 返回 server time.
        // 真实业务: 检测时钟漂移 (> 30s 警告) + 推 NATS 心跳.
        let mut session = self
            .sessions
            .find_by_id(session_id)
            .await?
            .ok_or(Error::SessionExpired)?;
        if session.is_expired() {
            return Err(Error::SessionExpired);
        }
        session.heartbeat();
        let saved_session = self.sessions.save(&session).await?;
        // 校验 character_id (optional)
        if let Some(cid) = character_id {
            let character = self
                .characters
                .find_by_id(cid)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "Character",
                    id: cid.to_string(),
                })?;
            if character.account_id != session.player_id {
                return Err(Error::Forbidden(format!(
                    "character {} not bound to session {}",
                    cid, session_id
                )));
            }
        }
        Ok((true, chrono::Utc::now().timestamp(), saved_session.expires_at))
    }
}

fn is_active_for_update(p: &Player) -> bool {
    !matches!(p.status, PlayerStatus::Banned | PlayerStatus::Disabled)
}

/// 桶 12 helper: 校验 player (账号) 是否可创建/登录角色.
fn is_player_active_for_character(p: &Player) -> bool {
    is_active_for_update(p)
}

// ============================================================================
// gRPC 桥接（per 54.2 proto：HealthCheck + GetPlayer）
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as player_proto;

    /// 业务 Service 包装成 gRPC service
    pub struct PlayerGrpcService {
        pub impl_: Arc<PlayerServiceImpl>,
    }

    impl PlayerGrpcService {
        pub fn new(impl_: Arc<PlayerServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl player_proto::player_service_server::PlayerService for PlayerGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "player-service",
                method = "HealthCheck",
                "enter grpc handler"
            );
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let (status_enum, msg) = if healthy {
                (common_proto::Status::Ok, "ok".to_string())
            } else {
                (common_proto::Status::Failed, "degraded".to_string())
            };
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: status_enum as i32,
                message: msg,
            }))
        }

        async fn get_player(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<player_proto::Player>, Status> {
            let id_str = request.get_ref().id.clone();
            let player_id_parsed = Uuid::parse_str(&id_str).ok();
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "player-service",
                method = "GetPlayer",
                player_id = %player_id_parsed.as_ref().map(|u| u.to_string()).unwrap_or_else(|| id_str.clone()),
                "enter grpc handler"
            );
            let player_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let player = self
                .impl_
                .find_by_id(player_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("player {}", id_str)))?;
            Ok(Response::new(player_proto::Player {
                id: Some(common_proto::EntityId {
                    id: player.id.to_string(),
                }),
                status: player.status as i32,
                created_at: Some(common_proto::Timestamp {
                    seconds: player.created_at.timestamp(),
                    nanos: player.created_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: player.name,
            }))
        }

        // ----- v2 gRPC 桥接 (per DTL-038 §4.3, 桶 11 增量) -----

        async fn get_player_profile(
            &self,
            request: Request<player_proto::GetPlayerProfileRequest>,
        ) -> std::result::Result<Response<player_proto::PlayerProfile>, Status> {
            let req = request.get_ref();
            let player_id = Uuid::parse_str(&req.player_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.player_id)))?;
            let profile = self
                .impl_
                .get_player_profile(player_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::PlayerProfile {
                player_id: profile.player_id.to_string(),
                ranked_score: profile.ranked_score,
                ranked_tier: profile.ranked_tier,
                total_matches: profile.total_matches,
                total_wins: profile.total_wins,
                collection_count: profile.collection_count,
                currencies: vec![],
                preferred_locale: profile.preferred_locale,
            }))
        }

        async fn update_player_profile(
            &self,
            request: Request<player_proto::UpdatePlayerProfileRequest>,
        ) -> std::result::Result<Response<player_proto::UpdatePlayerProfileResponse>, Status> {
            let req = request.get_ref();
            let player_id = Uuid::parse_str(&req.player_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.player_id)))?;
            // proto3 optional 字段: profile 是 Option<PlayerProfile>
            let proto_profile = req
                .profile
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("profile is required"))?;
            let profile = PlayerProfile {
                player_id,
                ranked_score: proto_profile.ranked_score,
                ranked_tier: proto_profile.ranked_tier.clone(),
                total_matches: proto_profile.total_matches,
                total_wins: proto_profile.total_wins,
                collection_count: proto_profile.collection_count,
                preferred_locale: proto_profile.preferred_locale.clone(),
            };
            self.impl_
                .update_player_profile(profile)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::UpdatePlayerProfileResponse { updated: true }))
        }

        async fn create_deck(
            &self,
            request: Request<player_proto::CreateDeckRequest>,
        ) -> std::result::Result<Response<player_proto::Deck>, Status> {
            let req = request.get_ref();
            let owner_id = Uuid::parse_str(&req.owner_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.owner_id)))?;
            let deck = self
                .impl_
                .create_deck(owner_id, req.name.clone(), req.mode)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(deck_to_proto(&deck)))
        }

        async fn get_deck(
            &self,
            request: Request<player_proto::GetDeckRequest>,
        ) -> std::result::Result<Response<player_proto::Deck>, Status> {
            let req = request.get_ref();
            let deck_id = Uuid::parse_str(&req.deck_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.deck_id)))?;
            let deck = self
                .impl_
                .get_deck(deck_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(deck_to_proto(&deck)))
        }

        async fn update_deck(
            &self,
            request: Request<player_proto::UpdateDeckRequest>,
        ) -> std::result::Result<Response<player_proto::UpdateDeckResponse>, Status> {
            let req = request.get_ref();
            let deck_id = Uuid::parse_str(&req.deck_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.deck_id)))?;
            let owner_id = Uuid::parse_str(&req.owner_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.owner_id)))?;
            // slots 转换: 空 = 不改
            let slots_opt: Option<Vec<DeckSlot>> = if req.slots.is_empty() {
                None
            } else {
                Some(
                    req.slots
                        .iter()
                        .map(|s| DeckSlot {
                            card_id: s.card_id.clone(),
                            count: s.count,
                        })
                        .collect(),
                )
            };
            // name 转换: 空 = 不改
            let name_opt: Option<String> = if req.name.is_empty() {
                None
            } else {
                Some(req.name.clone())
            };
            // 调用 service
            let _updated = self
                .impl_
                .update_deck(deck_id, owner_id, name_opt, slots_opt)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            // 校验 errors 占位: 当前桶 11 不实装规则引擎, 永远空
            Ok(Response::new(player_proto::UpdateDeckResponse {
                updated: true,
                validation_errors: vec![],
            }))
        }

        async fn delete_deck(
            &self,
            request: Request<player_proto::DeleteDeckRequest>,
        ) -> std::result::Result<Response<player_proto::DeleteDeckResponse>, Status> {
            let req = request.get_ref();
            let deck_id = Uuid::parse_str(&req.deck_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.deck_id)))?;
            let owner_id = Uuid::parse_str(&req.owner_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.owner_id)))?;
            let deleted = self
                .impl_
                .delete_deck(deck_id, owner_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::DeleteDeckResponse { deleted }))
        }

        async fn list_decks(
            &self,
            request: Request<player_proto::ListDecksRequest>,
        ) -> std::result::Result<Response<player_proto::ListDecksResponse>, Status> {
            let req = request.get_ref();
            let owner_id = Uuid::parse_str(&req.owner_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.owner_id)))?;
            let page_req = PageRequest {
                page: req.page.as_ref().map(|p| p.page).unwrap_or(1),
                page_size: req.page.as_ref().map(|p| p.page_size).unwrap_or(20),
            };
            let (decks, total) = self
                .impl_
                .list_decks(owner_id, page_req)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let proto_decks: Vec<player_proto::Deck> =
                decks.iter().map(deck_to_proto).collect();
            let has_next = proto_decks.len() as i64 + ((req.page.as_ref().map(|p| (p.page as i64 - 1) * p.page_size as i64).unwrap_or(0))) < total;
            Ok(Response::new(player_proto::ListDecksResponse {
                decks: proto_decks,
                page: Some(common_proto::PageResponse {
                    total: total as u32,
                    has_next,
                    next_cursor: String::new(),
                }),
            }))
        }

        async fn share_deck(
            &self,
            request: Request<player_proto::ShareDeckRequest>,
        ) -> std::result::Result<Response<player_proto::ShareDeckResponse>, Status> {
            let req = request.get_ref();
            let deck_id = Uuid::parse_str(&req.deck_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.deck_id)))?;
            let owner_id = Uuid::parse_str(&req.owner_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.owner_id)))?;
            let deck = self
                .impl_
                .share_deck(deck_id, owner_id, req.make_public)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            // share_url 占位: per service 拼接 (后续可改为 from config)
            let share_url = deck
                .share_code
                .as_ref()
                .map(|c| format!("https://rgs.example.com/decks/shared/{}", c))
                .unwrap_or_default();
            Ok(Response::new(player_proto::ShareDeckResponse {
                share_code: deck.share_code.clone().unwrap_or_default(),
                share_url,
            }))
        }

        async fn get_shared_deck(
            &self,
            request: Request<player_proto::GetSharedDeckRequest>,
        ) -> std::result::Result<Response<player_proto::Deck>, Status> {
            let req = request.get_ref();
            // 优先 share_code 路径, 否则 friend_id+friend_deck_id 路径
            if !req.share_code.is_empty() {
                let deck = self
                    .impl_
                    .get_shared_deck(req.share_code.clone())
                    .await
                    .map_err(Into::<tonic::Status>::into)?;
                Ok(Response::new(deck_to_proto(&deck)))
            } else if !req.friend_deck_id.is_empty() {
                // friend_deck_id 路径: 当 share_code 路径不可用时, 通过 friend 私有 deck id 直查
                // per DTL-038 §4.3 GetSharedDeckRequest 兼容好友 ID 拉取
                let deck_id = Uuid::parse_str(&req.friend_deck_id).map_err(|_| {
                    Status::invalid_argument(format!("invalid uuid: {}", req.friend_deck_id))
                })?;
                let deck = self
                    .impl_
                    .get_deck(deck_id)
                    .await
                    .map_err(Into::<tonic::Status>::into)?;
                Ok(Response::new(deck_to_proto(&deck)))
            } else {
                Err(Status::invalid_argument(
                    "either share_code or friend_deck_id required",
                ))
            }
        }

        // ========================================================================
        // 桶 12 增量: 闪烁之光 账号+角色 15 RPC gRPC 桥接 (per 9/5 11:50 JST 4 拍板)
        // ========================================================================

        async fn create_character(
            &self,
            request: Request<player_proto::CreateCharacterRequest>,
        ) -> std::result::Result<Response<player_proto::CreateCharacterResponse>, Status> {
            let req = request.get_ref();
            let account_id = Uuid::parse_str(&req.account_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.account_id)))?;
            let (character, session) = self
                .impl_
                .create_character(
                    account_id,
                    req.character_name.clone(),
                    req.class_id,
                    req.faction_id,
                    req.device_id.clone(),
                    req.client_ip.clone(),
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::CreateCharacterResponse {
                created: true,
                character_id: character.id.to_string(),
                character_name: character.name.clone(),
                class_id: character.class_id,
                faction_id: character.faction_id,
                created_at: Some(common_proto::Timestamp {
                    seconds: character.created_at.timestamp(),
                    nanos: character.created_at.timestamp_subsec_nanos() as i32,
                }),
                session_id: session.id.to_string(),
            }))
        }

        async fn login_character(
            &self,
            request: Request<player_proto::LoginCharacterRequest>,
        ) -> std::result::Result<Response<player_proto::LoginCharacterResponse>, Status> {
            let req = request.get_ref();
            let account_id = Uuid::parse_str(&req.account_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.account_id)))?;
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let (character, session) = self
                .impl_
                .login_character(
                    account_id,
                    character_id,
                    req.device_id.clone(),
                    req.client_ip.clone(),
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let info = character_to_character_info(&character);
            Ok(Response::new(player_proto::LoginCharacterResponse {
                logged_in: true,
                session_id: session.id.to_string(),
                expires_at: Some(common_proto::Timestamp {
                    seconds: session.expires_at.timestamp(),
                    nanos: session.expires_at.timestamp_subsec_nanos() as i32,
                }),
                info: Some(info),
            }))
        }

        async fn reconnect_character(
            &self,
            request: Request<player_proto::ReconnectCharacterRequest>,
        ) -> std::result::Result<Response<player_proto::ReconnectCharacterResponse>, Status> {
            let req = request.get_ref();
            let session_id = Uuid::parse_str(&req.session_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.session_id)))?;
            let (character, session) = self
                .impl_
                .reconnect_character(session_id, req.client_ip.clone())
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::ReconnectCharacterResponse {
                reconnected: true,
                character_id: character.id.to_string(),
                character_name: character.name.clone(),
                new_expires_at: Some(common_proto::Timestamp {
                    seconds: session.expires_at.timestamp(),
                    nanos: session.expires_at.timestamp_subsec_nanos() as i32,
                }),
            }))
        }

        async fn get_character_profile(
            &self,
            request: Request<player_proto::GetCharacterProfileRequest>,
        ) -> std::result::Result<Response<player_proto::CharacterProfile>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let profile = self
                .impl_
                .get_character_profile(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let character = self
                .impl_
                .get_character_info(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::CharacterProfile {
                character_id: profile.player_id.to_string(),
                character_name: character.name.clone(),
                level: character.level,
                vip_level: character.vip_level,
                class_id: character.class_id,
                faction_id: character.faction_id,
                status: character.status as i32,
                ranked_score: profile.ranked_score as i32,
                ranked_tier: profile.ranked_tier,
                total_matches: profile.total_matches,
                total_wins: profile.total_wins,
                preferred_locale: profile.preferred_locale,
                created_at: Some(common_proto::Timestamp {
                    seconds: character.created_at.timestamp(),
                    nanos: character.created_at.timestamp_subsec_nanos() as i32,
                }),
                last_login_at: character.last_login_at.map(|dt| common_proto::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }),
            }))
        }

        async fn get_character_assets(
            &self,
            request: Request<player_proto::GetCharacterAssetsRequest>,
        ) -> std::result::Result<Response<player_proto::CharacterAssets>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let snap = self
                .impl_
                .get_character_assets(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let entries: Vec<player_proto::AssetEntry> = snap
                .entries
                .iter()
                .map(|e| player_proto::AssetEntry {
                    code: e.code.clone(),
                    amount: e.amount,
                    updated_at: Some(common_proto::Timestamp {
                        seconds: e.updated_at.timestamp(),
                        nanos: e.updated_at.timestamp_subsec_nanos() as i32,
                    }),
                })
                .collect();
            Ok(Response::new(player_proto::CharacterAssets {
                character_id: snap.character_id.to_string(),
                entries,
                queried_at: Some(common_proto::Timestamp {
                    seconds: snap.queried_at.timestamp(),
                    nanos: snap.queried_at.timestamp_subsec_nanos() as i32,
                }),
            }))
        }

        async fn get_character_info(
            &self,
            request: Request<player_proto::GetCharacterInfoRequest>,
        ) -> std::result::Result<Response<player_proto::CharacterInfo>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let character = self
                .impl_
                .get_character_info(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(character_to_character_info(&character)))
        }

        async fn rename_character(
            &self,
            request: Request<player_proto::RenameCharacterRequest>,
        ) -> std::result::Result<Response<player_proto::RenameCharacterResponse>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let old_name = self
                .impl_
                .get_character_info(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .name;
            let character = self
                .impl_
                .rename_character(character_id, req.new_name.clone())
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::RenameCharacterResponse {
                renamed: true,
                old_name,
                new_name: character.name,
            }))
        }

        async fn get_server_time(
            &self,
            _request: Request<player_proto::GetServerTimeRequest>,
        ) -> std::result::Result<Response<player_proto::ServerTimeInfo>, Status> {
            let (now, server_open, tz) = self
                .impl_
                .get_server_time()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::ServerTimeInfo {
                server_time_unix: now,
                server_open_time_unix: server_open,
                timezone: tz,
            }))
        }

        async fn guest_mode_timeout(
            &self,
            request: Request<player_proto::GuestModeTimeoutRequest>,
        ) -> std::result::Result<Response<player_proto::GuestModeTimeoutResponse>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let (ack, deadline) = self
                .impl_
                .guest_mode_timeout(character_id, req.timeout_seconds)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::GuestModeTimeoutResponse {
                acknowledged: ack,
                deadline_at: Some(common_proto::Timestamp {
                    seconds: deadline.timestamp(),
                    nanos: deadline.timestamp_subsec_nanos() as i32,
                }),
            }))
        }

        async fn anti_addiction_check(
            &self,
            request: Request<player_proto::AntiAddictionCheckRequest>,
        ) -> std::result::Result<Response<player_proto::AntiAddictionCheckResponse>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let (is_adult, max_min) = self
                .impl_
                .anti_addiction_check(character_id, req.is_adult)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::AntiAddictionCheckResponse {
                acknowledged: true,
                is_adult,
                max_play_minutes: max_min,
            }))
        }

        async fn force_disconnect(
            &self,
            request: Request<player_proto::ForceDisconnectRequest>,
        ) -> std::result::Result<Response<player_proto::ForceDisconnectResponse>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let (issued, session_id) = self
                .impl_
                .force_disconnect(character_id, req.reason.clone())
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::ForceDisconnectResponse {
                disconnect_issued: issued,
                session_id: session_id.map(|u| u.to_string()).unwrap_or_default(),
            }))
        }

        async fn enter_background(
            &self,
            request: Request<player_proto::EnterBackgroundRequest>,
        ) -> std::result::Result<Response<player_proto::EnterBackgroundResponse>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let session_id = Uuid::parse_str(&req.session_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.session_id)))?;
            let (in_bg, since) = self
                .impl_
                .enter_background(character_id, session_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::EnterBackgroundResponse {
                in_background: in_bg,
                background_since: Some(common_proto::Timestamp {
                    seconds: since.timestamp(),
                    nanos: since.timestamp_subsec_nanos() as i32,
                }),
            }))
        }

        async fn get_avatar_list(
            &self,
            request: Request<player_proto::GetAvatarListRequest>,
        ) -> std::result::Result<Response<player_proto::AvatarList>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let character = self
                .impl_
                .get_character_info(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let avatars = self
                .impl_
                .get_avatar_list(character_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let avatar_infos: Vec<player_proto::AvatarInfo> = avatars
                .iter()
                .map(|a| player_proto::AvatarInfo {
                    avatar_id: a.avatar_id,
                    name: a.name.clone(),
                    url: a.url.clone(),
                    owned: a.owned,
                    unlock_by_default: a.unlock_by_default,
                })
                .collect();
            let total_owned = avatar_infos.iter().filter(|a| a.owned).count() as i32;
            Ok(Response::new(player_proto::AvatarList {
                avatars: avatar_infos,
                current_avatar_id: character.current_avatar_id,
                total_owned,
            }))
        }

        async fn set_avatar(
            &self,
            request: Request<player_proto::SetAvatarRequest>,
        ) -> std::result::Result<Response<player_proto::SetAvatarResponse>, Status> {
            let req = request.get_ref();
            let character_id = Uuid::parse_str(&req.character_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.character_id)))?;
            let new_id = self
                .impl_
                .set_avatar(character_id, req.avatar_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::SetAvatarResponse {
                set: true,
                current_avatar_id: new_id,
            }))
        }

        async fn heartbeat(
            &self,
            request: Request<player_proto::HeartbeatRequest>,
        ) -> std::result::Result<Response<player_proto::HeartbeatResponse>, Status> {
            let req = request.get_ref();
            let session_id = Uuid::parse_str(&req.session_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.session_id)))?;
            let character_id = if !req.character_id.is_empty() {
                Some(
                    Uuid::parse_str(&req.character_id).map_err(|_| {
                        Status::invalid_argument(format!("invalid uuid: {}", req.character_id))
                    })?,
                )
            } else {
                None
            };
            let (ok, server_time, expires_at) = self
                .impl_
                .heartbeat_rpc(session_id, character_id, req.client_time_unix)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(player_proto::HeartbeatResponse {
                ok,
                server_time_unix: server_time,
                session_expires_at: Some(common_proto::Timestamp {
                    seconds: expires_at.timestamp(),
                    nanos: expires_at.timestamp_subsec_nanos() as i32,
                }),
            }))
        }
    }

    /// entity Deck → proto Deck 转换 helper (free function, 不依赖 self)
    pub fn deck_to_proto(d: &Deck) -> player_proto::Deck {
        player_proto::Deck {
            deck_id: d.id.to_string(),
            owner_id: d.owner_id.to_string(),
            name: d.name.clone(),
            mode: d.mode,
            slots: d
                .slots
                .iter()
                .map(|s| player_proto::DeckSlot {
                    card_id: s.card_id.clone(),
                    count: s.count,
                })
                .collect(),
            status: match d.status {
                DeckStatus::Active => common_proto::Status::Ok as i32,
                DeckStatus::Archived => common_proto::Status::Cancelled as i32,
                DeckStatus::Draft => common_proto::Status::Pending as i32,
            },
            created_at: Some(common_proto::Timestamp {
                seconds: d.created_at.timestamp(),
                nanos: d.created_at.timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(common_proto::Timestamp {
                seconds: d.updated_at.timestamp(),
                nanos: d.updated_at.timestamp_subsec_nanos() as i32,
            }),
            is_public: d.is_public,
            share_code: d.share_code.clone().unwrap_or_default(),
            like_count: d.like_count,
        }
    }

    /// entity Character → proto CharacterInfo 转换 helper (桶 12 增量)
    pub fn character_to_character_info(c: &Character) -> player_proto::CharacterInfo {
        player_proto::CharacterInfo {
            character_id: c.id.to_string(),
            character_name: c.name.clone(),
            level: c.level,
            vip_level: c.vip_level,
            class_id: c.class_id,
            faction_id: c.faction_id,
            status: c.status as i32,
            current_avatar_id: c.current_avatar_id,
            signature: c.signature.clone(),
            created_at: Some(common_proto::Timestamp {
                seconds: c.created_at.timestamp(),
                nanos: c.created_at.timestamp_subsec_nanos() as i32,
            }),
            last_login_at: c.last_login_at.map(|dt| common_proto::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryCharacterRepository;
    use crate::repository::InMemoryDeckRepository;
    use crate::repository::InMemoryPlayerRepository;
    use crate::repository::InMemoryPlayerSessionRepository;

    async fn make_service() -> (
        PlayerServiceImpl,
        Arc<InMemoryPlayerRepository>,
        Arc<InMemoryPlayerSessionRepository>,
        Arc<InMemoryDeckRepository>,
        Arc<InMemoryCharacterRepository>,
    ) {
        let players = Arc::new(InMemoryPlayerRepository::new());
        let sessions = Arc::new(InMemoryPlayerSessionRepository::new());
        let decks = Arc::new(InMemoryDeckRepository::new());
        let characters = Arc::new(InMemoryCharacterRepository::new());
        let svc = PlayerServiceImpl::new(
            players.clone() as Arc<dyn PlayerRepository>,
            sessions.clone() as Arc<dyn PlayerSessionRepository>,
            decks.clone() as Arc<dyn DeckRepository>,
            characters.clone() as Arc<dyn CharacterRepository>,
        );
        (svc, players, sessions, decks, characters)
    }

    #[tokio::test]
    async fn register_creates_player() {
        let (svc, _, _, _, _) = make_service().await;
        let p = svc.register("alice".to_string()).await.unwrap();
        assert_eq!(p.name, "alice");
        assert_eq!(p.level, 1);
    }

    #[tokio::test]
    async fn register_duplicate_nickname_fails() {
        let (svc, _, _, _, _) = make_service().await;
        svc.register("bob".to_string()).await.unwrap();
        let err = svc.register("bob".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::NicknameTaken(_)));
    }

    #[tokio::test]
    async fn register_empty_name_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc.register("".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn update_profile_changes_level() {
        let (svc, _, _, _, _) = make_service().await;
        let p = svc.register("carol".to_string()).await.unwrap();
        let updated = svc.update_profile(p.id, Some(50), Some(1)).await.unwrap();
        assert_eq!(updated.level, 50);
        assert_eq!(updated.vip_level, 1);
    }

    #[tokio::test]
    async fn update_profile_level_out_of_range() {
        let (svc, _, _, _, _) = make_service().await;
        let p = svc.register("dave".to_string()).await.unwrap();
        let err = svc
            .update_profile(p.id, Some(9999), None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn disable_player() {
        let (svc, _, _, _, _) = make_service().await;
        let p = svc.register("eve".to_string()).await.unwrap();
        let disabled = svc
            .disable_player(p.id, "test ban".to_string())
            .await
            .unwrap();
        assert_eq!(disabled.status, PlayerStatus::Disabled);
    }

    #[tokio::test]
    async fn heartbeat_slides_session() {
        let (_, _, sessions, decks, characters) = make_service().await;
        let player_id = Uuid::new_v4();
        let session = PlayerSession::new(player_id, "dev-1".to_string(), "127.0.0.1".to_string());
        sessions.save(&session).await.unwrap();

        let svc = PlayerServiceImpl::new(
            Arc::new(InMemoryPlayerRepository::new()),
            sessions.clone() as Arc<dyn PlayerSessionRepository>,
            decks.clone() as Arc<dyn DeckRepository>,
            characters.clone() as Arc<dyn CharacterRepository>,
        );
        let updated = svc.heartbeat(session.id).await.unwrap();
        assert!(updated.expires_at > session.expires_at);
    }

    #[tokio::test]
    async fn find_by_id_returns_player() {
        let (svc, _, _, _, _) = make_service().await;
        let p = svc.register("frank".to_string()).await.unwrap();
        let found = svc.find_by_id(p.id).await.unwrap().unwrap();
        assert_eq!(found.name, "frank");
    }

    #[tokio::test]
    async fn health_check_returns_true() {
        let (svc, _, _, _, _) = make_service().await;
        assert!(svc.health_check().await.unwrap());
    }

    // ----- v2 卡牌游戏 service UT (per DTL-038 §4.3, 桶 11 增量) -----

    #[tokio::test]
    async fn create_deck_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("alice".to_string()).await.unwrap();
        let d = svc
            .create_deck(owner.id, "aggressive".to_string(), 1)
            .await
            .unwrap();
        assert_eq!(d.owner_id, owner.id);
        assert_eq!(d.name, "aggressive");
        assert_eq!(d.mode, 1);
        assert_eq!(d.status, DeckStatus::Draft);
        assert!(!d.is_public);
        assert!(d.share_code.is_none());
    }

    #[tokio::test]
    async fn create_deck_empty_name_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("bob".to_string()).await.unwrap();
        let err = svc
            .create_deck(owner.id, "".to_string(), 1)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_deck_invalid_mode_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("carol".to_string()).await.unwrap();
        let err = svc
            .create_deck(owner.id, "deck".to_string(), 99)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn get_deck_returns_deck() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("dave".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "control".to_string(), 2)
            .await
            .unwrap();
        let found = svc.get_deck(created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "control");
    }

    #[tokio::test]
    async fn update_deck_replaces_slots() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("eve".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "combo".to_string(), 1)
            .await
            .unwrap();
        let new_slots = vec![
            DeckSlot::new("card-A".to_string(), 2),
            DeckSlot::new("card-B".to_string(), 1),
        ];
        let updated = svc
            .update_deck(
                created.id,
                owner.id,
                Some("combo-v2".to_string()),
                Some(new_slots.clone()),
            )
            .await
            .unwrap();
        assert_eq!(updated.name, "combo-v2");
        assert_eq!(updated.slots.len(), 2);
        assert_eq!(updated.slots[0].card_id, "card-A");
    }

    #[tokio::test]
    async fn update_deck_not_owner_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("frank".to_string()).await.unwrap();
        let other = svc.register("other".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "deck".to_string(), 1)
            .await
            .unwrap();
        let err = svc
            .update_deck(created.id, other.id, Some("hijack".to_string()), None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn delete_deck_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("grace".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "deck".to_string(), 1)
            .await
            .unwrap();
        assert!(svc.delete_deck(created.id, owner.id).await.unwrap());
        let err = svc.get_deck(created.id).await.unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn delete_deck_not_owner_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("henry".to_string()).await.unwrap();
        let other = svc.register("ivan".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "deck".to_string(), 1)
            .await
            .unwrap();
        let err = svc.delete_deck(created.id, other.id).await.unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn list_decks_paginated() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("jack".to_string()).await.unwrap();
        for i in 0..5 {
            svc.create_deck(owner.id, format!("deck-{}", i), 1)
                .await
                .unwrap();
        }
        let (items, total) = svc
            .list_decks(
                owner.id,
                PageRequest {
                    page: 1,
                    page_size: 3,
                },
            )
            .await
            .unwrap();
        assert_eq!(total, 5);
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn share_deck_make_public_generates_share_code() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("kate".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "deck".to_string(), 1)
            .await
            .unwrap();
        assert!(!created.is_public);
        let shared = svc
            .share_deck(created.id, owner.id, true)
            .await
            .unwrap();
        assert!(shared.is_public);
        assert!(shared.share_code.is_some());
        let code = shared.share_code.clone().unwrap();
        // 校验 UUIDv4 格式
        assert!(Uuid::parse_str(&code).is_ok());
    }

    #[tokio::test]
    async fn share_deck_unpublic_clears_share_code() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("liam".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "deck".to_string(), 1)
            .await
            .unwrap();
        let shared = svc
            .share_deck(created.id, owner.id, true)
            .await
            .unwrap();
        assert!(shared.share_code.is_some());
        let unshared = svc
            .share_deck(created.id, owner.id, false)
            .await
            .unwrap();
        assert!(!unshared.is_public);
        assert!(unshared.share_code.is_none());
    }

    #[tokio::test]
    async fn get_shared_deck_by_code() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("mia".to_string()).await.unwrap();
        let created = svc
            .create_deck(owner.id, "deck".to_string(), 1)
            .await
            .unwrap();
        let shared = svc
            .share_deck(created.id, owner.id, true)
            .await
            .unwrap();
        let code = shared.share_code.clone().unwrap();
        let pulled = svc.get_shared_deck(code.clone()).await.unwrap();
        assert_eq!(pulled.id, created.id);
        assert!(pulled.is_public);
    }

    #[tokio::test]
    async fn get_shared_deck_unknown_code_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc
            .get_shared_deck("nonexistent-code".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn get_shared_deck_empty_code_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc
            .get_shared_deck("".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn get_player_profile_default() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("nia".to_string()).await.unwrap();
        let profile = svc.get_player_profile(owner.id).await.unwrap();
        assert_eq!(profile.player_id, owner.id);
        assert_eq!(profile.ranked_tier, "Bronze");
        assert_eq!(profile.total_matches, 0);
    }

    #[tokio::test]
    async fn update_player_profile_persists_fields() {
        let (svc, _, _, _, _) = make_service().await;
        let owner = svc.register("oscar".to_string()).await.unwrap();
        let updated = svc
            .update_player_profile(PlayerProfile {
                player_id: owner.id,
                ranked_score: 1500,
                ranked_tier: "Gold".to_string(),
                total_matches: 100,
                total_wins: 60,
                collection_count: 50,
                preferred_locale: "ja-JP".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(updated.ranked_score, 1500);
        assert_eq!(updated.ranked_tier, "Gold");
        assert_eq!(updated.preferred_locale, "ja-JP");
    }

    #[tokio::test]
    async fn validate_deck_slots_returns_empty_per_bucket_11() {
        // 桶 11 不实装规则引擎: 永远返回空
        let slots = vec![
            DeckSlot::new("card-1".to_string(), 1),
            DeckSlot::new("card-2".to_string(), 1),
        ];
        let errs = PlayerServiceImpl::validate_deck_slots(&slots);
        assert!(errs.is_empty());
    }

    // ========================================================================
    // 桶 12 增量: 闪烁之光 账号+角色 15 RPC UT (per 9/5 11:50 JST 4 拍板)
    // 5 真实逻辑 (Create/Login/Reconnect/Profile/Assets) + 10 stub
    // ========================================================================

    // ----- 10101 CreateCharacter -----

    #[tokio::test]
    async fn create_character_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("acct-owner".to_string()).await.unwrap();
        let (character, session) = svc
            .create_character(
                account.id,
                "alice".to_string(),
                1, // class=warrior
                1, // faction=order
                "dev-1".to_string(),
                "127.0.0.1".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(character.account_id, account.id);
        assert_eq!(character.name, "alice");
        assert_eq!(character.class_id, 1);
        assert_eq!(character.faction_id, 1);
        assert_eq!(character.level, 1);
        assert!(character.is_active());
        assert!(character.last_login_at.is_some());
        assert_eq!(session.player_id, account.id);
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn create_character_empty_name_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("acct".to_string()).await.unwrap();
        let err = svc
            .create_character(account.id, "".to_string(), 1, 1, "d".to_string(), "1.1.1.1".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_character_invalid_class_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("acct".to_string()).await.unwrap();
        let err = svc
            .create_character(account.id, "bob".to_string(), 99, 1, "d".to_string(), "1.1.1.1".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_character_duplicate_name_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("acct".to_string()).await.unwrap();
        svc.create_character(
            account.id,
            "carol".to_string(),
            1,
            1,
            "d".to_string(),
            "1.1.1.1".to_string(),
        )
        .await
        .unwrap();
        // 第二次同名 → NicknameTaken
        let err = svc
            .create_character(account.id, "carol".to_string(), 1, 1, "d".to_string(), "1.1.1.1".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NicknameTaken(_)));
    }

    // ----- 10102 LoginCharacter -----

    #[tokio::test]
    async fn login_character_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("login-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "dave".to_string(),
                2,
                1,
                "dev".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (char2, session) = svc
            .login_character(
                account.id,
                char1.id,
                "dev-2".to_string(),
                "2.2.2.2".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(char2.id, char1.id);
        assert_eq!(char2.account_id, account.id);
        assert!(char2.last_login_at.is_some());
        assert_eq!(session.player_id, account.id);
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn login_character_wrong_account_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let a1 = svc.register("a1".to_string()).await.unwrap();
        let a2 = svc.register("a2".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                a1.id,
                "eve".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let err = svc
            .login_character(a2.id, char1.id, "d".to_string(), "1.1.1.1".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    // ----- 10103 ReconnectCharacter -----

    #[tokio::test]
    async fn reconnect_character_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("recon-acct".to_string()).await.unwrap();
        let (char1, session) = svc
            .create_character(
                account.id,
                "frank".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (char2, new_session) = svc
            .reconnect_character(session.id, "1.1.1.1".to_string())
            .await
            .unwrap();
        assert_eq!(char2.id, char1.id);
        assert_eq!(new_session.id, session.id);
        assert!(new_session.expires_at >= session.expires_at);
    }

    #[tokio::test]
    async fn reconnect_character_expired_session_fails() {
        let (svc, _, sessions, _, _) = make_service().await;
        let account_id = Uuid::new_v4();
        let mut session = PlayerSession::new(account_id, "d".to_string(), "1.1.1.1".to_string());
        session.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
        sessions.save(&session).await.unwrap();
        let err = svc
            .reconnect_character(session.id, "1.1.1.1".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::SessionExpired));
    }

    // ----- 10301 GetCharacterProfile -----

    #[tokio::test]
    async fn get_character_profile_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("prof-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "grace".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let profile = svc.get_character_profile(char1.id).await.unwrap();
        assert_eq!(profile.player_id, char1.id);
        assert_eq!(profile.ranked_tier, "Bronze");
    }

    #[tokio::test]
    async fn get_character_profile_not_found_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc
            .get_character_profile(Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    // ----- 10302 GetCharacterAssets -----

    #[tokio::test]
    async fn get_character_assets_empty_per_bucket_12() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("assets-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "henry".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let snap = svc.get_character_assets(char1.id).await.unwrap();
        assert_eq!(snap.character_id, char1.id);
        assert!(snap.entries.is_empty()); // 桶 12 占位, 0 余额
    }

    // ----- 10315 GetCharacterInfo (stub) -----

    #[tokio::test]
    async fn get_character_info_returns_character() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("info-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "ivy".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let info = svc.get_character_info(char1.id).await.unwrap();
        assert_eq!(info.id, char1.id);
    }

    // ----- 10343 RenameCharacter (stub 兼真实) -----

    #[tokio::test]
    async fn rename_character_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("rename-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "jack".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let renamed = svc
            .rename_character(char1.id, "jack-renamed".to_string())
            .await
            .unwrap();
        assert_eq!(renamed.name, "jack-renamed");
    }

    #[tokio::test]
    async fn rename_character_empty_name_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("rename2".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "kate".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let err = svc
            .rename_character(char1.id, "".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // ----- 10380 GetServerTime (stub) -----

    #[tokio::test]
    async fn get_server_time_returns_trio() {
        let (svc, _, _, _, _) = make_service().await;
        let (now, server_open, tz) = svc.get_server_time().await.unwrap();
        assert!(now > 0);
        assert!(server_open > 0);
        assert_eq!(tz, "Asia/Tokyo");
        assert!(now > server_open);
    }

    // ----- 10394 GuestModeTimeout (stub) -----

    #[tokio::test]
    async fn guest_mode_timeout_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("guest-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "liam".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (ok, deadline) = svc.guest_mode_timeout(char1.id, 600).await.unwrap();
        assert!(ok);
        assert!(deadline > chrono::Utc::now());
    }

    #[tokio::test]
    async fn guest_mode_timeout_out_of_range_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc.guest_mode_timeout(Uuid::new_v4(), 30).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // ----- 10395 AntiAddictionCheck (stub) -----

    #[tokio::test]
    async fn anti_addiction_check_adult() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("anti-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "mia".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (is_adult, max_min) = svc
            .anti_addiction_check(char1.id, true)
            .await
            .unwrap();
        assert!(is_adult);
        assert_eq!(max_min, 0); // 18+ 不限
    }

    #[tokio::test]
    async fn anti_addiction_check_minor_90min() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("anti2".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "nia".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (is_adult, max_min) = svc
            .anti_addiction_check(char1.id, false)
            .await
            .unwrap();
        assert!(!is_adult);
        assert_eq!(max_min, 90);
    }

    // ----- 10396 ForceDisconnect (stub) -----

    #[tokio::test]
    async fn force_disconnect_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("force-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "oscar".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (issued, session_id) = svc
            .force_disconnect(char1.id, "违规发言".to_string())
            .await
            .unwrap();
        assert!(issued);
        assert!(session_id.is_none()); // stub: 无活跃 session
    }

    #[tokio::test]
    async fn force_disconnect_empty_reason_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc
            .force_disconnect(Uuid::new_v4(), "".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // ----- 10397 EnterBackground (stub) -----

    #[tokio::test]
    async fn enter_background_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("bg-acct".to_string()).await.unwrap();
        let (char1, session) = svc
            .create_character(
                account.id,
                "peter".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (ok, since) = svc
            .enter_background(char1.id, session.id)
            .await
            .unwrap();
        assert!(ok);
        assert!(since <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn enter_background_invalid_session_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc
            .enter_background(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::SessionExpired));
    }

    // ----- 10325 GetAvatarList (stub) -----

    #[tokio::test]
    async fn get_avatar_list_default_three() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("avatar-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "quinn".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let avatars = svc.get_avatar_list(char1.id).await.unwrap();
        assert_eq!(avatars.len(), 3);
        assert!(avatars.iter().any(|a| a.unlock_by_default && a.owned));
    }

    // ----- 10327 SetAvatar (stub) -----

    #[tokio::test]
    async fn set_avatar_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("setavatar-acct".to_string()).await.unwrap();
        let (char1, _) = svc
            .create_character(
                account.id,
                "rachel".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let new_avatar = svc.set_avatar(char1.id, 2).await.unwrap();
        assert_eq!(new_avatar, 2);
    }

    #[tokio::test]
    async fn set_avatar_out_of_range_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let err = svc
            .set_avatar(Uuid::new_v4(), 99)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // ----- 11000 Heartbeat (RGS 新增, stub) -----

    #[tokio::test]
    async fn heartbeat_rpc_happy_path() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("hb-acct".to_string()).await.unwrap();
        let (char1, session) = svc
            .create_character(
                account.id,
                "sam".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let (ok, server_time, expires_at) = svc
            .heartbeat_rpc(session.id, Some(char1.id), 1234567890)
            .await
            .unwrap();
        assert!(ok);
        assert!(server_time > 0);
        assert!(expires_at > chrono::Utc::now());
    }

    #[tokio::test]
    async fn heartbeat_rpc_expired_session_fails() {
        let (svc, _, sessions, _, _) = make_service().await;
        let account_id = Uuid::new_v4();
        let mut session = PlayerSession::new(account_id, "d".to_string(), "1.1.1.1".to_string());
        session.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
        sessions.save(&session).await.unwrap();
        let err = svc
            .heartbeat_rpc(session.id, None, 0)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::SessionExpired));
    }

    #[tokio::test]
    async fn heartbeat_rpc_wrong_character_fails() {
        let (svc, _, _, _, _) = make_service().await;
        let account = svc.register("hb2-acct".to_string()).await.unwrap();
        let (_char1, session) = svc
            .create_character(
                account.id,
                "tina".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        // 错误 character_id (存在但 account_id 不匹配)
        let other_account = svc.register("hb2-other".to_string()).await.unwrap();
        let (other_char, _) = svc
            .create_character(
                other_account.id,
                "tina-other".to_string(),
                1,
                1,
                "d".to_string(),
                "1.1.1.1".to_string(),
            )
            .await
            .unwrap();
        let err = svc
            .heartbeat_rpc(session.id, Some(other_char.id), 0)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
