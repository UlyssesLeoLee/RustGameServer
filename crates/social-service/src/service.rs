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
        tracing::debug!(
            operation = "service_entry",
            service = "social-service",
            method = "CreateGuild",
            name = %name,
            leader_id = %leader_id,
            "enter create_guild"
        );
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
        tracing::debug!(
            operation = "service_entry",
            service = "social-service",
            method = "JoinGuild",
            guild_id = %guild_id,
            player_id = %player_id,
            "enter join_guild"
        );
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
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "social-service",
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

        async fn get_guild(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<social_proto::Guild>, Status> {
            let id_str = request.get_ref().id.clone();
            let guild_id_parsed = Uuid::parse_str(&id_str).ok();
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "social-service",
                method = "GetGuild",
                guild_id = %guild_id_parsed.as_ref().map(|u| u.to_string()).unwrap_or_else(|| id_str.clone()),
                "enter grpc handler"
            );
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

    #[tokio::test]
    async fn create_guild_rejects_empty_name() {
        let s = svc();
        let err = s
            .create_guild("".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_guild_rejects_whitespace_only_name() {
        let s = svc();
        let err = s
            .create_guild("   ".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_guild_rejects_too_long_name() {
        let s = svc();
        let long = "a".repeat(65);
        let err = s
            .create_guild(long, "".to_string(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_guild_accepts_max_len_name() {
        let s = svc();
        let name = "b".repeat(64);
        let g = s
            .create_guild(name.clone(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(g.name, name);
    }

    #[tokio::test]
    async fn join_guild_rejects_full_guild() {
        let s = svc();
        let g = s
            .create_guild("X".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        // leader 已在 member_count=1 状态,需要再加 49 个成员达到 50 上限
        for _ in 0..49 {
            s.join_guild(g.id, Uuid::new_v4()).await.unwrap();
        }
        // 第 50 个新人应被拒(guild 已 50)
        let err = s.join_guild(g.id, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, Error::GuildFull { .. }));
    }

    #[tokio::test]
    async fn join_guild_rejects_already_member() {
        let s = svc();
        let g = s
            .create_guild("Y".to_string(), "".to_string(), Uuid::new_v4())
            .await
            .unwrap();
        let player = Uuid::new_v4();
        s.join_guild(g.id, player).await.unwrap();
        // 同一 player 再 join 应被拒
        let err = s.join_guild(g.id, player).await.unwrap_err();
        assert!(matches!(err, Error::AlreadyInGuild { .. }));
    }

    #[tokio::test]
    async fn join_guild_rejects_nonexistent_guild() {
        let s = svc();
        let err = s
            .join_guild(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn promote_to_officer_rejects_leader() {
        let s = svc();
        let leader = Uuid::new_v4();
        let g = s
            .create_guild("Z".to_string(), "".to_string(), leader)
            .await
            .unwrap();
        // 查找 leader 自己的 member id
        let members = s
            .members
            .find_by_player(leader)
            .await
            .unwrap();
        assert_eq!(members.len(), 1);
        let leader_member_id = members[0].id;
        let err = s
            .promote_to_officer(leader_member_id)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InsufficientPermission { .. }));
    }

    #[tokio::test]
    async fn promote_to_officer_rejects_nonexistent_member() {
        let s = svc();
        let err = s
            .promote_to_officer(Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn dissolve_guild_rejects_nonexistent() {
        let s = svc();
        let err = s.dissolve_guild(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn find_guild_by_id_returns_none_for_missing() {
        let s = svc();
        let g = s
            .find_guild_by_id(Uuid::new_v4())
            .await
            .unwrap();
        assert!(g.is_none());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::repository::{InMemoryGuildMemberRepository, InMemoryGuildRepository};
    use proptest::prelude::*;
    use std::sync::Arc;

    fn svc() -> SocialServiceImpl {
        SocialServiceImpl::new(
            Arc::new(InMemoryGuildRepository::new()),
            Arc::new(InMemoryGuildMemberRepository::new()),
        )
    }

    proptest! {
        /// create_guild 对 [A-Za-z0-9]{1..=64} 名字必成功,member_count=1
        #[test]
        fn create_guild_happy_path_random_names(
            name in "[A-Za-z0-9]{1,64}",
            desc in ".*",
            leader_bytes in any::<[u8; 16]>(),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let s = svc();
                let leader = Uuid::from_bytes(leader_bytes);
                let g = s.create_guild(name.clone(), desc.clone(), leader).await.unwrap();
                prop_assert_eq!(g.name, name);
                prop_assert_eq!(g.leader_id, leader);
                prop_assert_eq!(g.member_count, 1);
                prop_assert_eq!(g.level, 1);
                prop_assert_eq!(g.experience, 0);
            });
        }

        /// create_guild 重复同名必失败 (Conflict)
        #[test]
        fn create_guild_duplicate_name_fails_random(
            name in "[A-Za-z0-9]{1,16}",
            l1 in any::<[u8; 16]>(),
            l2 in any::<[u8; 16]>(),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let s = svc();
                s.create_guild(name.clone(), "".to_string(), Uuid::from_bytes(l1))
                    .await
                    .unwrap();
                let err = s
                    .create_guild(name, "".to_string(), Uuid::from_bytes(l2))
                    .await
                    .unwrap_err();
                prop_assert!(matches!(err, Error::Conflict(_)));
            });
        }

        /// 名字长度 == 65 必被拒(超过 64 上限)
        #[test]
        fn create_guild_name_too_long_rejected(
            name in "[a]{65,100}",
            leader_bytes in any::<[u8; 16]>(),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let s = svc();
                let err = s
                    .create_guild(name, "".to_string(), Uuid::from_bytes(leader_bytes))
                    .await
                    .unwrap_err();
                prop_assert!(matches!(err, Error::Validation(_)));
            });
        }
    }
}
