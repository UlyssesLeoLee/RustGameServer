//! social-service 域 Service 业务实施（per RGS-DTL-026 §3）
//!
//! 54.7 实化：4 Service 业务方法（create_guild / join_guild / promote_member / dissolve_guild）
//! + gRPC 桥接 HealthCheck + GetGuild

use crate::entity::{Guild, GuildMember, GuildRole};
use crate::error::Error;
use crate::repository::{GuildMemberRepository, GuildRepository};
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait SocialService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    async fn create_guild(
        &self,
        name: String,
        description: String,
        leader_id: Uuid,
    ) -> Result<Guild>;
    async fn join_guild(&self, guild_id: Uuid, player_id: Uuid) -> Result<GuildMember>;
    async fn promote_to_officer(&self, member_id: Uuid) -> Result<GuildMember>;
    async fn dissolve_guild(&self, guild_id: Uuid) -> Result<Guild>;
}

pub struct SocialServiceImpl {
    guilds: Arc<dyn GuildRepository>,
    members: Arc<dyn GuildMemberRepository>,
}

impl SocialServiceImpl {
    pub fn new(guilds: Arc<dyn GuildRepository>, members: Arc<dyn GuildMemberRepository>) -> Self {
        Self { guilds, members }
    }

    pub async fn find_guild_by_id(&self, id: Uuid) -> Result<Option<Guild>> {
        self.guilds.find_by_id(id).await
    }
}

#[async_trait]
impl SocialService for SocialServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn create_guild(
        &self,
        name: String,
        description: String,
        leader_id: Uuid,
    ) -> Result<Guild> {
        if name.trim().is_empty() {
            return Err(Error::Validation(
                "guild name must not be empty".to_string(),
            ));
        }
        if name.len() > 64 {
            return Err(Error::Validation(
                "guild name too long (max 64)".to_string(),
            ));
        }
        if self.guilds.find_by_name(&name).await?.is_some() {
            return Err(Error::Conflict(format!(
                "guild name {} already in use",
                name
            )));
        }
        let guild = Guild::new(name, description, leader_id);
        self.guilds.save(&guild).await?;
        // 自动加 leader 为 member
        let mut leader = GuildMember::new(guild.id, leader_id);
        leader.role = GuildRole::Leader;
        self.members.save(&leader).await?;
        Ok(guild)
    }

    async fn join_guild(&self, guild_id: Uuid, player_id: Uuid) -> Result<GuildMember> {
        let guild = self
            .guilds
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Guild",
                id: guild_id.to_string(),
            })?;
        // 简单限制：50 人
        if guild.member_count >= 50 {
            return Err(Error::GuildFull {
                guild_id: guild_id.to_string(),
            });
        }
        // 检查是否已在其他公会
        let existing = self.members.find_by_player(player_id).await?;
        if !existing.is_empty() {
            return Err(Error::AlreadyInGuild {
                player_id: player_id.to_string(),
                guild_id: existing[0].guild_id.to_string(),
            });
        }
        let member = GuildMember::new(guild_id, player_id);
        self.members.save(&member).await?;
        // 更新 guild.member_count
        let mut g = guild;
        g.member_count += 1;
        g.updated_at = chrono::Utc::now();
        self.guilds.save(&g).await?;
        Ok(member)
    }

    async fn promote_to_officer(&self, member_id: Uuid) -> Result<GuildMember> {
        let mut member =
            self.members
                .find_by_id(member_id)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "GuildMember",
                    id: member_id.to_string(),
                })?;
        if member.role == GuildRole::Leader {
            return Err(Error::InsufficientPermission {
                required: "leader is already top".to_string(),
                actual: "leader".to_string(),
            });
        }
        member.promote_to_officer();
        self.members.save(&member).await?;
        Ok(member)
    }

    async fn dissolve_guild(&self, guild_id: Uuid) -> Result<Guild> {
        let guild = self
            .guilds
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Guild",
                id: guild_id.to_string(),
            })?;
        // 删所有 member
        let members = self.members.list_by_guild(guild_id).await?;
        for m in members {
            self.members.delete_by_id(m.id).await?;
        }
        // 删 guild 本身
        self.guilds.delete_by_id(guild_id).await?;
        Ok(guild)
    }
}

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as social_proto;

    pub struct SocialGrpcService {
        pub impl_: Arc<SocialServiceImpl>,
    }

    impl SocialGrpcService {
        pub fn new(impl_: Arc<SocialServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl social_proto::social_service_server::SocialService for SocialGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
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

        async fn get_guild(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<social_proto::Guild>, Status> {
            let id_str = request.get_ref().id.clone();
            let guild_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let g = self
                .impl_
                .find_guild_by_id(guild_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("guild {}", id_str)))?;
            Ok(Response::new(social_proto::Guild {
                id: Some(common_proto::EntityId {
                    id: g.id.to_string(),
                }),
                status: g.level as i32,
                created_at: Some(common_proto::Timestamp {
                    seconds: g.created_at.timestamp(),
                    nanos: g.created_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: g.name,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InMemoryGuildMemberRepository, InMemoryGuildRepository};

    fn svc() -> SocialServiceImpl {
        SocialServiceImpl::new(
            Arc::new(InMemoryGuildRepository::new()),
            Arc::new(InMemoryGuildMemberRepository::new()),
        )
    }

    #[tokio::test]
    async fn create_guild_adds_leader() {
        let s = svc();
        let leader = Uuid::new_v4();
        let g = s
            .create_guild("Knights".to_string(), "brave".to_string(), leader)
            .await
            .unwrap();
        assert_eq!(g.name, "Knights");
        assert_eq!(g.member_count, 1);
        // leader 通过 join_guild 验证：再 join 一次应得 member_count=2
        s.join_guild(g.id, Uuid::new_v4()).await.unwrap();
        let updated = s.find_guild_by_id(g.id).await.unwrap().unwrap();
        assert_eq!(updated.member_count, 2);
    }

    #[tokio::test]
    async fn create_duplicate_name_fails() {
        let s = svc();
        s.create_guild("A".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        let err = s
            .create_guild("A".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn join_guild_increments_count() {
        let s = svc();
        let g = s
            .create_guild("B".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        let member = s.join_guild(g.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(member.role, GuildRole::Member);
        let updated = s.find_guild_by_id(g.id).await.unwrap().unwrap();
        assert_eq!(updated.member_count, 2);
    }

    #[tokio::test]
    async fn promote_to_officer() {
        let s = svc();
        let g = s
            .create_guild("C".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        let player = Uuid::new_v4();
        let member = s.join_guild(g.id, player).await.unwrap();
        let promoted = s.promote_to_officer(member.id).await.unwrap();
        assert_eq!(promoted.role, GuildRole::Officer);
    }

    #[tokio::test]
    async fn dissolve_guild_removes_all() {
        let s = svc();
        let g = s
            .create_guild("D".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        s.join_guild(g.id, Uuid::new_v4()).await.unwrap();
        s.dissolve_guild(g.id).await.unwrap();
        let g2 = s.find_guild_by_id(g.id).await.unwrap();
        assert!(g2.is_none());
    }

    #[tokio::test]
    async fn health_check() {
        let s = svc();
        assert!(s.health_check().await.unwrap());
    }
}
