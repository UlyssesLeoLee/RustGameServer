//! player-service 域 Service 业务实施（per RGS-DTL-018 §3）
//!
//! 54.7 实化：
//! - 4 Service 业务方法（register / heartbeat / update_profile / disable_player）
//! - ServiceImpl 接 PlayerRepository + PlayerSessionRepository（Arc<dyn>）
//! - PlayerServiceImpl 直接暴露 find_by_id（gRPC GetPlayer 用，绕开 trait）
//! - gRPC 桥接：impl player_proto::player_service_server::PlayerService for PlayerGrpcService
//!   接 HealthCheck + GetPlayer（per 54.2 proto 定义）

use crate::entity::{Player, PlayerSession, PlayerStatus};
use crate::error::Error;
use crate::repository::{PlayerRepository, PlayerSessionRepository};
use crate::Result;

use async_trait::async_trait;
use chrono::Utc;
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
}

/// player-service 默认实现
pub struct PlayerServiceImpl {
    players: Arc<dyn PlayerRepository>,
    sessions: Arc<dyn PlayerSessionRepository>,
}

impl PlayerServiceImpl {
    pub fn new(
        players: Arc<dyn PlayerRepository>,
        sessions: Arc<dyn PlayerSessionRepository>,
    ) -> Self {
        Self { players, sessions }
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

        // W15 (2026-08-28): BanAccount gRPC RPC, 供 admin-service GM 调
        // 链路: gm-backend → admin-service BanAccount → player-service BanAccount
        async fn ban_account(
            &self,
            request: Request<player_proto::BanAccountRequest>,
        ) -> std::result::Result<Response<player_proto::BanAccountResponse>, Status> {
            let req = request.into_inner();
            let banned_at_ms = Utc::now().timestamp_millis();

            // 先按 uuid 查, 失败按 name 查
            let player_id_opt = Uuid::parse_str(&req.account_id).ok();
            let player_id = if let Some(pid) = player_id_opt {
                pid
            } else {
                // 按 name 查 (业务 schema: account_id = name)
                match self.impl_.players.find_by_name(&req.account_id).await {
                    Ok(Some(p)) => p.id,
                    Ok(None) => {
                        return Ok(Response::new(player_proto::BanAccountResponse {
                            status: "not_found".to_string(),
                            account_id: req.account_id,
                            banned_at_ms: 0,
                            expires_at_ms: 0,
                        }));
                    }
                    Err(e) => {
                        return Err(tonic::Status::internal(format!(
                            "find_by_name error: {e}"
                        )));
                    }
                }
            };

            // 调 disable_player (改 status='disabled')
            self.impl_
                .disable_player(player_id, req.reason.clone())
                .await
                .map_err(Into::<tonic::Status>::into)?;

            // expires_at: duration=0 永久 = 0; 否则 banned_at + duration
            let expires_at_ms = if req.duration_seconds == 0 {
                0
            } else {
                banned_at_ms + (req.duration_seconds as i64) * 1000
            };

            Ok(Response::new(player_proto::BanAccountResponse {
                status: "banned".to_string(),
                account_id: req.account_id,
                banned_at_ms,
                expires_at_ms,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryPlayerRepository;
    use crate::repository::InMemoryPlayerSessionRepository;

    async fn make_service() -> (
        PlayerServiceImpl,
        Arc<InMemoryPlayerRepository>,
        Arc<InMemoryPlayerSessionRepository>,
    ) {
        let players = Arc::new(InMemoryPlayerRepository::new());
        let sessions = Arc::new(InMemoryPlayerSessionRepository::new());
        let svc = PlayerServiceImpl::new(
            players.clone() as Arc<dyn PlayerRepository>,
            sessions.clone() as Arc<dyn PlayerSessionRepository>,
        );
        (svc, players, sessions)
    }

    #[tokio::test]
    async fn register_creates_player() {
        let (svc, _, _) = make_service().await;
        let p = svc.register("alice".to_string()).await.unwrap();
        assert_eq!(p.name, "alice");
        assert_eq!(p.level, 1);
    }

    #[tokio::test]
    async fn register_duplicate_nickname_fails() {
        let (svc, _, _) = make_service().await;
        svc.register("bob".to_string()).await.unwrap();
        let err = svc.register("bob".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::NicknameTaken(_)));
    }

    #[tokio::test]
    async fn register_empty_name_fails() {
        let (svc, _, _) = make_service().await;
        let err = svc.register("".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn update_profile_changes_level() {
        let (svc, _, _) = make_service().await;
        let p = svc.register("carol".to_string()).await.unwrap();
        let updated = svc.update_profile(p.id, Some(50), Some(1)).await.unwrap();
        assert_eq!(updated.level, 50);
        assert_eq!(updated.vip_level, 1);
    }

    #[tokio::test]
    async fn update_profile_level_out_of_range() {
        let (svc, _, _) = make_service().await;
        let p = svc.register("dave".to_string()).await.unwrap();
        let err = svc
            .update_profile(p.id, Some(9999), None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn disable_player() {
        let (svc, _, _) = make_service().await;
        let p = svc.register("eve".to_string()).await.unwrap();
        let disabled = svc
            .disable_player(p.id, "test ban".to_string())
            .await
            .unwrap();
        assert_eq!(disabled.status, PlayerStatus::Disabled);
    }

    #[tokio::test]
    async fn heartbeat_slides_session() {
        let (_, _, sessions) = make_service().await;
        let player_id = Uuid::new_v4();
        let session = PlayerSession::new(player_id, "dev-1".to_string(), "127.0.0.1".to_string());
        sessions.save(&session).await.unwrap();

        let svc = PlayerServiceImpl::new(
            Arc::new(InMemoryPlayerRepository::new()),
            sessions.clone() as Arc<dyn PlayerSessionRepository>,
        );
        let updated = svc.heartbeat(session.id).await.unwrap();
        assert!(updated.expires_at > session.expires_at);
    }

    #[tokio::test]
    async fn find_by_id_returns_player() {
        let (svc, _, _) = make_service().await;
        let p = svc.register("frank".to_string()).await.unwrap();
        let found = svc.find_by_id(p.id).await.unwrap().unwrap();
        assert_eq!(found.name, "frank");
    }

    #[tokio::test]
    async fn health_check_returns_true() {
        let (svc, _, _) = make_service().await;
        assert!(svc.health_check().await.unwrap());
    }
}
