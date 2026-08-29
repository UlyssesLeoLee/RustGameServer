//! match-service 域 Service 业务实施（per RGS-DTL-016 §3）
//!
//! 54.7 实化：4 Service 业务方法（create_match / join_match / start_match / finish_match）
//! + gRPC 桥接 HealthCheck + GetMatch

use crate::entity::{Match, MatchMode, MatchParticipant, MatchStatus, Team};
use crate::error::Error;
use crate::repository::{MatchParticipantRepository, MatchRepository};
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait MatchService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    /// 创建对局
    async fn create_match(&self, room_id: String, mode: MatchMode) -> Result<Match>;

    /// 玩家加入对局
    async fn join_match(
        &self,
        match_id: Uuid,
        player_id: Uuid,
        team: Team,
    ) -> Result<MatchParticipant>;

    /// 开始对局
    async fn start_match(&self, match_id: Uuid) -> Result<Match>;

    /// 结束对局
    async fn finish_match(&self, match_id: Uuid, winner: Option<Team>) -> Result<Match>;
}

pub struct MatchServiceImpl {
    matches: Arc<dyn MatchRepository>,
    participants: Arc<dyn MatchParticipantRepository>,
}

impl MatchServiceImpl {
    pub fn new(
        matches: Arc<dyn MatchRepository>,
        participants: Arc<dyn MatchParticipantRepository>,
    ) -> Self {
        Self {
            matches,
            participants,
        }
    }

    pub async fn find_match_by_id(&self, id: Uuid) -> Result<Option<Match>> {
        self.matches.find_by_id(id).await
    }
}

#[async_trait]
impl MatchService for MatchServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn create_match(&self, room_id: String, mode: MatchMode) -> Result<Match> {
        tracing::debug!(
            operation = "matchmaking_entry",
            service = "match-service",
            method = "create_match",
            room_id = %room_id,
            mode = ?mode,
            "matchmaking: create match"
        );
        if room_id.is_empty() {
            return Err(Error::Validation("room_id must not be empty".to_string()));
        }
        if self.matches.find_by_room_id(&room_id).await?.is_some() {
            return Err(Error::Conflict(format!(
                "room_id {} already in use",
                room_id
            )));
        }
        let m = Match::new(room_id, mode);
        self.matches.save(&m).await?;
        Ok(m)
    }

    async fn join_match(
        &self,
        match_id: Uuid,
        player_id: Uuid,
        team: Team,
    ) -> Result<MatchParticipant> {
        tracing::debug!(
            operation = "matchmaking_candidate_join",
            service = "match-service",
            method = "join_match",
            match_id = %match_id,
            player_id = %player_id,
            team = ?team,
            "matchmaking: candidate join"
        );
        let m = self
            .matches
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Match",
                id: match_id.to_string(),
            })?;
        if m.status != MatchStatus::Waiting {
            return Err(Error::MatchAlreadyStarted(match_id.to_string()));
        }
        // 检查是否已加入
        let existing = self.participants.list_by_match(match_id).await?;
        if existing.iter().any(|p| p.player_id == player_id) {
            return Err(Error::Conflict(format!(
                "player {} already in match",
                player_id
            )));
        }
        let p = MatchParticipant::new(match_id, player_id, team);
        self.participants.save(&p).await?;
        Ok(p)
    }

    async fn start_match(&self, match_id: Uuid) -> Result<Match> {
        let mut m = self
            .matches
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Match",
                id: match_id.to_string(),
            })?;
        if m.status != MatchStatus::Waiting {
            return Err(Error::MatchAlreadyStarted(match_id.to_string()));
        }
        m.start();
        self.matches.save(&m).await?;
        Ok(m)
    }

    async fn finish_match(&self, match_id: Uuid, winner: Option<Team>) -> Result<Match> {
        let mut m = self
            .matches
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Match",
                id: match_id.to_string(),
            })?;
        if m.status != MatchStatus::InProgress {
            return Err(Error::Validation(format!(
                "match {} is not in progress",
                match_id
            )));
        }
        m.finish(winner);
        self.matches.save(&m).await?;
        Ok(m)
    }
}

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as match_proto;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tonic::codegen::tokio_stream::Stream;

    // v2 增量: SubscribeMatch 用的空流类型 (per RGS-DTL-038 §4.2)
    // 桶 7 阶段: 占位空流, 业务实装 (桶 9 session/turn) 替换
    pub struct EmptyMatchEventStream;

    impl Stream for EmptyMatchEventStream {
        type Item = std::result::Result<match_proto::MatchEvent, Status>;
        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(None)
        }
    }

    pub struct MatchGrpcService {
        pub impl_: Arc<MatchServiceImpl>,
    }

    impl MatchGrpcService {
        pub fn new(impl_: Arc<MatchServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl match_proto::match_service_server::MatchService for MatchGrpcService {
        // v2 增量: SubscribeMatch 流关联类型 (per RGS-DTL-038 §4.2 — stream RPC)
        // 桶 7 阶段: stub 用空流类型
        // 桶 9 (session/turn) 起实装
        type SubscribeMatchStream = EmptyMatchEventStream;
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "match-service",
                method = "HealthCheck",
                "enter grpc handler"
            );
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: if healthy {
                    common_proto::Status::Ok as i32
                } else {
                    common_proto::Status::Failed as i32
                },
                message: if healthy {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
            }))
        }

        async fn get_match(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<match_proto::Match>, Status> {
            let id_str = request.get_ref().id.clone();
            let match_id_parsed = Uuid::parse_str(&id_str).ok();
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "match-service",
                method = "GetMatch",
                match_id = %match_id_parsed.as_ref().map(|u| u.to_string()).unwrap_or_else(|| id_str.clone()),
                "enter grpc handler"
            );
            let match_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let m = self
                .impl_
                .find_match_by_id(match_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("match {}", id_str)))?;
            Ok(Response::new(match_proto::Match {
                id: Some(common_proto::EntityId {
                    id: m.id.to_string(),
                }),
                status: m.status as i32,
                created_at: Some(common_proto::Timestamp {
                    seconds: m.created_at.timestamp(),
                    nanos: m.created_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: m.room_id,
                // v2 增量字段 (per RGS-DTL-038 §4.2): 现有 v1 entity 无对应字段, 返回默认值
                // 桶 9 (session/turn) 起按 9 DEC 落地业务实装
                mode: 0,                            // GAME_MODE_UNSPECIFIED
                players: vec![],                     // 占位空数组
                board_snapshot_ref: String::new(),   // 占位空字符串
                turn_index: 0,                       // 占位 0
            }))
        }

        // ========================================================================
        // v2 增量 (per RGS-DTL-038 §4.2 — match-service session/turn 抽象)
        // 桶 7 (proto 设计) 阶段: 全部 stub — 返回 unimplemented
        // 桶 9 (session/turn) 起按 9 DEC 落地业务实装
        // ========================================================================

        async fn enqueue_matchmaking(
            &self,
            _request: Request<match_proto::EnqueueMatchmakingRequest>,
        ) -> std::result::Result<Response<match_proto::EnqueueMatchmakingResponse>, Status> {
            Err(Status::unimplemented(
                "EnqueueMatchmaking stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn cancel_matchmaking(
            &self,
            _request: Request<match_proto::CancelMatchmakingRequest>,
        ) -> std::result::Result<Response<match_proto::CancelMatchmakingResponse>, Status> {
            Err(Status::unimplemented(
                "CancelMatchmaking stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn get_matchmaking_status(
            &self,
            _request: Request<match_proto::GetMatchmakingStatusRequest>,
        ) -> std::result::Result<Response<match_proto::GetMatchmakingStatusResponse>, Status> {
            Err(Status::unimplemented(
                "GetMatchmakingStatus stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn create_match(
            &self,
            _request: Request<match_proto::CreateMatchRequest>,
        ) -> std::result::Result<Response<match_proto::CreateMatchResponse>, Status> {
            Err(Status::unimplemented(
                "CreateMatch stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn join_match(
            &self,
            _request: Request<match_proto::JoinMatchRequest>,
        ) -> std::result::Result<Response<match_proto::JoinMatchResponse>, Status> {
            Err(Status::unimplemented(
                "JoinMatch stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn leave_match(
            &self,
            _request: Request<match_proto::LeaveMatchRequest>,
        ) -> std::result::Result<Response<match_proto::LeaveMatchResponse>, Status> {
            Err(Status::unimplemented(
                "LeaveMatch stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn get_match_state(
            &self,
            _request: Request<match_proto::GetMatchStateRequest>,
        ) -> std::result::Result<Response<match_proto::GetMatchStateResponse>, Status> {
            Err(Status::unimplemented(
                "GetMatchState stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn submit_move(
            &self,
            _request: Request<match_proto::SubmitMoveRequest>,
        ) -> std::result::Result<Response<match_proto::SubmitMoveResponse>, Status> {
            Err(Status::unimplemented(
                "SubmitMove stub (桶 7), 桶 9 (session/turn) 起实装 — per RGS-DTL-038 §4.2",
            ))
        }

        async fn subscribe_match(
            &self,
            _request: Request<match_proto::SubscribeMatchRequest>,
        ) -> std::result::Result<Response<Self::SubscribeMatchStream>, Status> {
            // 流式 RPC stub: 返回空流, 业务实装 (桶 9) 替换
            Ok(Response::new(EmptyMatchEventStream))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InMemoryMatchParticipantRepository, InMemoryMatchRepository};

    fn svc() -> MatchServiceImpl {
        MatchServiceImpl::new(
            Arc::new(InMemoryMatchRepository::new()),
            Arc::new(InMemoryMatchParticipantRepository::new()),
        )
    }

    #[tokio::test]
    async fn create_and_get_match() {
        let s = svc();
        let m = s
            .create_match("r1".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        assert_eq!(m.status, MatchStatus::Waiting);
        let loaded = s.find_match_by_id(m.id).await.unwrap().unwrap();
        assert_eq!(loaded.room_id, "r1");
    }

    #[tokio::test]
    async fn create_duplicate_room_fails() {
        let s = svc();
        s.create_match("r1".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        let err = s
            .create_match("r1".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn join_then_start_then_finish() {
        let s = svc();
        let m = s
            .create_match("r2".to_string(), MatchMode::FiveVsFive)
            .await
            .unwrap();
        let p_id = Uuid::new_v4();
        s.join_match(m.id, p_id, Team::Blue).await.unwrap();

        let started = s.start_match(m.id).await.unwrap();
        assert_eq!(started.status, MatchStatus::InProgress);

        let finished = s.finish_match(m.id, Some(Team::Blue)).await.unwrap();
        assert_eq!(finished.status, MatchStatus::Finished);
        assert_eq!(finished.winner_team, Some(Team::Blue));
    }

    #[tokio::test]
    async fn start_already_started_fails() {
        let s = svc();
        let m = s
            .create_match("r3".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        s.start_match(m.id).await.unwrap();
        let err = s.start_match(m.id).await.unwrap_err();
        assert!(matches!(err, Error::MatchAlreadyStarted(_)));
    }

    #[tokio::test]
    async fn join_after_start_fails() {
        let s = svc();
        let m = s
            .create_match("r4".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        s.start_match(m.id).await.unwrap();
        let err = s
            .join_match(m.id, Uuid::new_v4(), Team::Blue)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MatchAlreadyStarted(_)));
    }

    #[tokio::test]
    async fn health_check() {
        let s = svc();
        assert!(s.health_check().await.unwrap());
    }
}
