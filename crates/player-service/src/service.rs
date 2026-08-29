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

use crate::entity::{Deck, DeckSlot, DeckStatus, Player, PlayerProfile, PlayerSession, PlayerStatus};
use crate::error::Error;
use crate::repository::{DeckRepository, PageRequest, PlayerRepository, PlayerSessionRepository};
use crate::Result;

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
}

/// player-service 默认实现
pub struct PlayerServiceImpl {
    players: Arc<dyn PlayerRepository>,
    sessions: Arc<dyn PlayerSessionRepository>,
    decks: Arc<dyn DeckRepository>,
}

impl PlayerServiceImpl {
    /// 4 参构造: 完整接入 PlayerRepository + PlayerSessionRepository + DeckRepository
    pub fn new(
        players: Arc<dyn PlayerRepository>,
        sessions: Arc<dyn PlayerSessionRepository>,
        decks: Arc<dyn DeckRepository>,
    ) -> Self {
        Self {
            players,
            sessions,
            decks,
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
}

fn is_active_for_update(p: &Player) -> bool {
    !matches!(p.status, PlayerStatus::Banned | PlayerStatus::Disabled)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryDeckRepository;
    use crate::repository::InMemoryPlayerRepository;
    use crate::repository::InMemoryPlayerSessionRepository;

    async fn make_service() -> (
        PlayerServiceImpl,
        Arc<InMemoryPlayerRepository>,
        Arc<InMemoryPlayerSessionRepository>,
        Arc<InMemoryDeckRepository>,
    ) {
        let players = Arc::new(InMemoryPlayerRepository::new());
        let sessions = Arc::new(InMemoryPlayerSessionRepository::new());
        let decks = Arc::new(InMemoryDeckRepository::new());
        let svc = PlayerServiceImpl::new(
            players.clone() as Arc<dyn PlayerRepository>,
            sessions.clone() as Arc<dyn PlayerSessionRepository>,
            decks.clone() as Arc<dyn DeckRepository>,
        );
        (svc, players, sessions, decks)
    }

    #[tokio::test]
    async fn register_creates_player() {
        let (svc, _, _, _) = make_service().await;
        let p = svc.register("alice".to_string()).await.unwrap();
        assert_eq!(p.name, "alice");
        assert_eq!(p.level, 1);
    }

    #[tokio::test]
    async fn register_duplicate_nickname_fails() {
        let (svc, _, _, _) = make_service().await;
        svc.register("bob".to_string()).await.unwrap();
        let err = svc.register("bob".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::NicknameTaken(_)));
    }

    #[tokio::test]
    async fn register_empty_name_fails() {
        let (svc, _, _, _) = make_service().await;
        let err = svc.register("".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn update_profile_changes_level() {
        let (svc, _, _, _) = make_service().await;
        let p = svc.register("carol".to_string()).await.unwrap();
        let updated = svc.update_profile(p.id, Some(50), Some(1)).await.unwrap();
        assert_eq!(updated.level, 50);
        assert_eq!(updated.vip_level, 1);
    }

    #[tokio::test]
    async fn update_profile_level_out_of_range() {
        let (svc, _, _, _) = make_service().await;
        let p = svc.register("dave".to_string()).await.unwrap();
        let err = svc
            .update_profile(p.id, Some(9999), None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn disable_player() {
        let (svc, _, _, _) = make_service().await;
        let p = svc.register("eve".to_string()).await.unwrap();
        let disabled = svc
            .disable_player(p.id, "test ban".to_string())
            .await
            .unwrap();
        assert_eq!(disabled.status, PlayerStatus::Disabled);
    }

    #[tokio::test]
    async fn heartbeat_slides_session() {
        let (_, _, sessions, decks) = make_service().await;
        let player_id = Uuid::new_v4();
        let session = PlayerSession::new(player_id, "dev-1".to_string(), "127.0.0.1".to_string());
        sessions.save(&session).await.unwrap();

        let svc = PlayerServiceImpl::new(
            Arc::new(InMemoryPlayerRepository::new()),
            sessions.clone() as Arc<dyn PlayerSessionRepository>,
            decks.clone() as Arc<dyn DeckRepository>,
        );
        let updated = svc.heartbeat(session.id).await.unwrap();
        assert!(updated.expires_at > session.expires_at);
    }

    #[tokio::test]
    async fn find_by_id_returns_player() {
        let (svc, _, _, _) = make_service().await;
        let p = svc.register("frank".to_string()).await.unwrap();
        let found = svc.find_by_id(p.id).await.unwrap().unwrap();
        assert_eq!(found.name, "frank");
    }

    #[tokio::test]
    async fn health_check_returns_true() {
        let (svc, _, _, _) = make_service().await;
        assert!(svc.health_check().await.unwrap());
    }

    // ----- v2 卡牌游戏 service UT (per DTL-038 §4.3, 桶 11 增量) -----

    #[tokio::test]
    async fn create_deck_happy_path() {
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
        let owner = svc.register("bob".to_string()).await.unwrap();
        let err = svc
            .create_deck(owner.id, "".to_string(), 1)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_deck_invalid_mode_fails() {
        let (svc, _, _, _) = make_service().await;
        let owner = svc.register("carol".to_string()).await.unwrap();
        let err = svc
            .create_deck(owner.id, "deck".to_string(), 99)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn get_deck_returns_deck() {
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
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
        let (svc, _, _, _) = make_service().await;
        let err = svc
            .get_shared_deck("nonexistent-code".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn get_shared_deck_empty_code_fails() {
        let (svc, _, _, _) = make_service().await;
        let err = svc
            .get_shared_deck("".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn get_player_profile_default() {
        let (svc, _, _, _) = make_service().await;
        let owner = svc.register("nia".to_string()).await.unwrap();
        let profile = svc.get_player_profile(owner.id).await.unwrap();
        assert_eq!(profile.player_id, owner.id);
        assert_eq!(profile.ranked_tier, "Bronze");
        assert_eq!(profile.total_matches, 0);
    }

    #[tokio::test]
    async fn update_player_profile_persists_fields() {
        let (svc, _, _, _) = make_service().await;
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
}
