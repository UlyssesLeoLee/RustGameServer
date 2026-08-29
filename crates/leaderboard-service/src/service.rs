//! leaderboard-service 域 Service 业务实施 (per RGS-DTL-038 §3 + §3.1 + RGS-REQ-038 §FR-007)
//!
//! 4 RPC + 1 内部 AddEntry:
//! - GetRankedLeaderboard: 天梯榜 (MMR-based, ranked 必填 season_id)
//! - GetCasualLeaderboard: 休闲榜 (胜场 / 局数)
//! - GetCollectionLeaderboard: 集换价值榜 (玩家收藏总价值)
//! - GetPlayerRank: 玩家自己在 3 类榜单的位置
//! - AddEntry: 内部 API (其他服务: match-service / card-service 调用, 不暴露 client)
//!
//! 排序算法:
//! - PgRepository: 走 idx_lb_type_period_season_score 索引, score DESC + LIMIT/OFFSET
//! - InMemoryRepository: BTreeMap 模拟 (按 score 倒序遍历)
//! - 后续 PH-1 可换 Redis 排序集 (per DTL-038 §3 DEC-038-02 备注)

use crate::entity::{LeaderboardEntry, LeaderboardPeriod, LeaderboardType};
use crate::error::Error;
use crate::repository::LeaderboardRepository;
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// 分页默认值 / 上限 (per DTL-038 §3.1 限流)
const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;
/// 默认榜单大小
#[allow(dead_code)]
const DEFAULT_LEADERBOARD_LIMIT: u32 = 100;

#[async_trait]
pub trait LeaderboardDomainService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    /// 拉取天梯榜
    async fn get_ranked_leaderboard(
        &self,
        period: LeaderboardPeriod,
        season_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LeaderboardEntry>, u32, bool)>; // (entries, total, has_next)

    /// 拉取休闲榜
    async fn get_casual_leaderboard(
        &self,
        period: LeaderboardPeriod,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LeaderboardEntry>, u32, bool)>;

    /// 拉取集换价值榜
    async fn get_collection_leaderboard(
        &self,
        period: LeaderboardPeriod,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LeaderboardEntry>, u32, bool)>;

    /// 玩家在 3 类榜单的位置
    async fn get_player_rank(
        &self,
        player_id: Uuid,
        period: LeaderboardPeriod,
    ) -> Result<(Option<LeaderboardEntry>, Option<LeaderboardEntry>, Option<LeaderboardEntry>)>;

    /// 内部入榜写
    async fn add_entry(
        &self,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: String,
        player_id: Uuid,
        display_name: String,
        score: i64,
        wins: u32,
        losses: u32,
    ) -> Result<(LeaderboardEntry, bool)>; // (entry, rank_changed)
}

pub struct LeaderboardServiceImpl {
    repo: Arc<dyn LeaderboardRepository>,
}

impl LeaderboardServiceImpl {
    pub fn new(repo: Arc<dyn LeaderboardRepository>) -> Self {
        Self { repo }
    }

    /// 规范化 page + page_size
    fn normalize_page(page: u32, page_size: u32) -> Result<(i64, i64)> {
        if page == 0 {
            return Err(Error::InvalidPage("page must be >= 1".to_string()));
        }
        let ps = if page_size == 0 {
            DEFAULT_PAGE_SIZE
        } else {
            page_size.min(MAX_PAGE_SIZE)
        };
        let offset = ((page - 1) as i64) * (ps as i64);
        Ok((offset, ps as i64))
    }

    /// 校验 ranked 榜的 season_id
    fn validate_ranked_season(season_id: &str) -> Result<()> {
        if season_id.is_empty() {
            return Err(Error::InvalidLeaderboardSpec(
                "ranked leaderboard requires non-empty season_id".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl LeaderboardDomainService for LeaderboardServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn get_ranked_leaderboard(
        &self,
        period: LeaderboardPeriod,
        season_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LeaderboardEntry>, u32, bool)> {
        Self::validate_ranked_season(&season_id)?;
        let (offset, limit) = Self::normalize_page(page, page_size)?;
        let (entries, total) = self
            .repo
            .list_by_board(LeaderboardType::Ranked, period, &season_id, limit, offset)
            .await?;
        let total_u32 = total as u32;
        let has_next = (offset + limit) < total;
        Ok((entries, total_u32, has_next))
    }

    async fn get_casual_leaderboard(
        &self,
        period: LeaderboardPeriod,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LeaderboardEntry>, u32, bool)> {
        let (offset, limit) = Self::normalize_page(page, page_size)?;
        // 休闲榜按 ALL_TIME / WEEKLY / MONTHLY / SEASONAL 各自 partition
        let (entries, total) = self
            .repo
            .list_by_board(LeaderboardType::Casual, period, "", limit, offset)
            .await?;
        let total_u32 = total as u32;
        let has_next = (offset + limit) < total;
        Ok((entries, total_u32, has_next))
    }

    async fn get_collection_leaderboard(
        &self,
        period: LeaderboardPeriod,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<LeaderboardEntry>, u32, bool)> {
        let (offset, limit) = Self::normalize_page(page, page_size)?;
        let (entries, total) = self
            .repo
            .list_by_board(LeaderboardType::Collection, period, "", limit, offset)
            .await?;
        let total_u32 = total as u32;
        let has_next = (offset + limit) < total;
        Ok((entries, total_u32, has_next))
    }

    async fn get_player_rank(
        &self,
        player_id: Uuid,
        period: LeaderboardPeriod,
    ) -> Result<(Option<LeaderboardEntry>, Option<LeaderboardEntry>, Option<LeaderboardEntry>)> {
        // ranked 拿不到合理 season_id 时跳过 ranked (per FR-007 玩家在榜位置
        // 返回 3 类; ranked 必须有 season_id, 缺省时该字段为 None, 不报错)
        let ranked = self
            .repo
            .find_by_player(player_id, LeaderboardType::Ranked, period, "")
            .await
            .ok()
            .flatten();
        let casual = self
            .repo
            .find_by_player(player_id, LeaderboardType::Casual, period, "")
            .await?;
        let collection = self
            .repo
            .find_by_player(player_id, LeaderboardType::Collection, period, "")
            .await?;
        Ok((ranked, casual, collection))
    }

    async fn add_entry(
        &self,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: String,
        player_id: Uuid,
        display_name: String,
        score: i64,
        wins: u32,
        losses: u32,
    ) -> Result<(LeaderboardEntry, bool)> {
        if leaderboard_type == LeaderboardType::Ranked {
            Self::validate_ranked_season(&season_id)?;
        }
        if display_name.is_empty() {
            return Err(Error::Validation("display_name must not be empty".to_string()));
        }
        let entry = LeaderboardEntry::new(
            leaderboard_type,
            period,
            if leaderboard_type == LeaderboardType::Ranked {
                season_id
            } else {
                String::new()
            },
            player_id,
            display_name,
            score,
            wins,
            losses,
        );
        self.repo.upsert(&entry).await
    }
}

// ============================================================================
// gRPC 桥接
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as lb_proto;

    pub struct LeaderboardGrpcService {
        pub impl_: Arc<LeaderboardServiceImpl>,
    }

    impl LeaderboardGrpcService {
        pub fn new(impl_: Arc<LeaderboardServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    fn parse_period(p: i32) -> Result<LeaderboardPeriod> {
        match p {
            x if x == lb_proto::LeaderboardPeriod::Weekly as i32 => {
                Ok(LeaderboardPeriod::Weekly)
            }
            x if x == lb_proto::LeaderboardPeriod::Monthly as i32 => {
                Ok(LeaderboardPeriod::Monthly)
            }
            x if x == lb_proto::LeaderboardPeriod::Seasonal as i32 => {
                Ok(LeaderboardPeriod::Seasonal)
            }
            x if x == lb_proto::LeaderboardPeriod::AllTime as i32 => {
                Ok(LeaderboardPeriod::AllTime)
            }
            _ => Err(Error::InvalidLeaderboardSpec(format!(
                "unknown period enum value: {}",
                p
            ))),
        }
    }

    fn period_to_proto(p: LeaderboardPeriod) -> i32 {
        match p {
            LeaderboardPeriod::Weekly => lb_proto::LeaderboardPeriod::Weekly as i32,
            LeaderboardPeriod::Monthly => lb_proto::LeaderboardPeriod::Monthly as i32,
            LeaderboardPeriod::Seasonal => lb_proto::LeaderboardPeriod::Seasonal as i32,
            LeaderboardPeriod::AllTime => lb_proto::LeaderboardPeriod::AllTime as i32,
        }
    }

    fn entry_to_proto(e: &LeaderboardEntry) -> lb_proto::LeaderboardEntry {
        lb_proto::LeaderboardEntry {
            rank: e.rank,
            player_id: e.player_id.to_string(),
            display_name: e.display_name.clone(),
            score: e.score,
            wins: e.wins,
            losses: e.losses,
            updated_at: Some(common_proto::Timestamp {
                seconds: e.updated_at.timestamp(),
                nanos: e.updated_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }

    #[tonic::async_trait]
    impl lb_proto::leaderboard_service_server::LeaderboardService for LeaderboardGrpcService {
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

        async fn get_ranked_leaderboard(
            &self,
            request: Request<lb_proto::GetRankedLeaderboardRequest>,
        ) -> std::result::Result<Response<lb_proto::GetRankedLeaderboardResponse>, Status> {
            let req = request.get_ref();
            let period = parse_period(req.period)?;
            let (page, page_size) = req
                .page
                .as_ref()
                .map(|p| (p.page, p.page_size))
                .unwrap_or((1, 0));
            let season_id = if req.season_id.is_empty() {
                // 默认尝试 active season (mock: 固定 "season_default")
                "season_default".to_string()
            } else {
                req.season_id.clone()
            };
            let (entries, total, has_next) = self
                .impl_
                .get_ranked_leaderboard(period, season_id.clone(), page, page_size)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(lb_proto::GetRankedLeaderboardResponse {
                entries: entries.iter().map(entry_to_proto).collect(),
                page: Some(common_proto::PageResponse {
                    total,
                    has_next,
                    next_cursor: if has_next { format!("{}", page + 1) } else { String::new() },
                }),
                period: period_to_proto(period),
                season_id,
            }))
        }

        async fn get_casual_leaderboard(
            &self,
            request: Request<lb_proto::GetCasualLeaderboardRequest>,
        ) -> std::result::Result<Response<lb_proto::GetCasualLeaderboardResponse>, Status> {
            let req = request.get_ref();
            let period = parse_period(req.period)?;
            let (page, page_size) = req
                .page
                .as_ref()
                .map(|p| (p.page, p.page_size))
                .unwrap_or((1, 0));
            let (entries, total, has_next) = self
                .impl_
                .get_casual_leaderboard(period, page, page_size)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(lb_proto::GetCasualLeaderboardResponse {
                entries: entries.iter().map(entry_to_proto).collect(),
                page: Some(common_proto::PageResponse {
                    total,
                    has_next,
                    next_cursor: if has_next { format!("{}", page + 1) } else { String::new() },
                }),
                period: period_to_proto(period),
            }))
        }

        async fn get_collection_leaderboard(
            &self,
            request: Request<lb_proto::GetCollectionLeaderboardRequest>,
        ) -> std::result::Result<Response<lb_proto::GetCollectionLeaderboardResponse>, Status> {
            let req = request.get_ref();
            let period = parse_period(req.period)?;
            let (page, page_size) = req
                .page
                .as_ref()
                .map(|p| (p.page, p.page_size))
                .unwrap_or((1, 0));
            let (entries, total, has_next) = self
                .impl_
                .get_collection_leaderboard(period, page, page_size)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(lb_proto::GetCollectionLeaderboardResponse {
                entries: entries.iter().map(entry_to_proto).collect(),
                page: Some(common_proto::PageResponse {
                    total,
                    has_next,
                    next_cursor: if has_next { format!("{}", page + 1) } else { String::new() },
                }),
                period: period_to_proto(period),
            }))
        }

        async fn get_player_rank(
            &self,
            request: Request<lb_proto::GetPlayerRankRequest>,
        ) -> std::result::Result<Response<lb_proto::GetPlayerRankResponse>, Status> {
            let req = request.get_ref();
            let period = parse_period(req.period)?;
            let player_id = Uuid::parse_str(&req.player_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.player_id)))?;
            let (ranked, casual, collection) = self
                .impl_
                .get_player_rank(player_id, period)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(lb_proto::GetPlayerRankResponse {
                player_id: req.player_id.clone(),
                ranked: ranked.as_ref().map(entry_to_proto),
                casual: casual.as_ref().map(entry_to_proto),
                collection: collection.as_ref().map(entry_to_proto),
                period: period_to_proto(period),
            }))
        }

        async fn add_entry(
            &self,
            request: Request<lb_proto::AddEntryRequest>,
        ) -> std::result::Result<Response<lb_proto::AddEntryResponse>, Status> {
            let req = request.get_ref();
            let leaderboard_type = match req.leaderboard_type {
                x if x == lb_proto::LeaderboardType::Ranked as i32 => LeaderboardType::Ranked,
                x if x == lb_proto::LeaderboardType::Casual as i32 => LeaderboardType::Casual,
                x if x == lb_proto::LeaderboardType::Collection as i32 => {
                    LeaderboardType::Collection
                }
                _ => {
                    return Err(Status::invalid_argument(format!(
                        "unknown leaderboard_type: {}",
                        req.leaderboard_type
                    )));
                }
            };
            let period = parse_period(req.period)?;
            let player_id = Uuid::parse_str(&req.player_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.player_id)))?;
            let (entry, rank_changed) = self
                .impl_
                .add_entry(
                    leaderboard_type,
                    period,
                    req.season_id.clone(),
                    player_id,
                    req.display_name.clone(),
                    req.score,
                    req.wins,
                    req.losses,
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(lb_proto::AddEntryResponse {
                entry: Some(entry_to_proto(&entry)),
                rank_changed,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryLeaderboardRepository;

    fn svc() -> LeaderboardServiceImpl {
        LeaderboardServiceImpl::new(Arc::new(InMemoryLeaderboardRepository::new()))
    }

    #[tokio::test]
    async fn get_ranked_leaderboard_requires_season_id() {
        let s = svc();
        let err = s
            .get_ranked_leaderboard(LeaderboardPeriod::Seasonal, "".to_string(), 1, 10)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidLeaderboardSpec(_)));
    }

    #[tokio::test]
    async fn get_casual_leaderboard_returns_empty_initially() {
        let s = svc();
        let (entries, total, has_next) = s
            .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 20)
            .await
            .unwrap();
        assert!(entries.is_empty());
        assert_eq!(total, 0);
        assert!(!has_next);
    }

    #[tokio::test]
    async fn add_entry_then_get_casual_leaderboard() {
        let s = svc();
        let p = Uuid::new_v4();
        let (entry, rank_changed) = s
            .add_entry(
                LeaderboardType::Casual,
                LeaderboardPeriod::Weekly,
                "".to_string(),
                p,
                "alice".to_string(),
                100,
                10,
                5,
            )
            .await
            .unwrap();
        assert_eq!(entry.score, 100);
        assert_eq!(entry.rank, 1);
        assert!(rank_changed);

        let (entries, total, _) = s
            .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 20)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].display_name, "alice");
    }

    #[tokio::test]
    async fn get_player_rank_returns_three_entries() {
        let s = svc();
        let p = Uuid::new_v4();
        s.add_entry(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            p,
            "bob".to_string(),
            5,
            3,
            1,
        )
        .await
        .unwrap();
        s.add_entry(
            LeaderboardType::Collection,
            LeaderboardPeriod::AllTime,
            "".to_string(),
            p,
            "bob".to_string(),
            500,
            0,
            0,
        )
        .await
        .unwrap();

        let (ranked, casual, collection) = s.get_player_rank(p, LeaderboardPeriod::Weekly).await.unwrap();
        // ranked 必须有 season_id 才能入榜; 这里未入 ranked, casual+collection 已入
        assert!(ranked.is_none());
        assert!(casual.is_some());
        assert_eq!(casual.as_ref().unwrap().score, 5);
        // collection 的 period 是 AllTime, 与 Weekly 不同, 该查为 None
        assert!(collection.is_none());

        let (_, casual2, _) = s.get_player_rank(p, LeaderboardPeriod::AllTime).await.unwrap();
        assert!(casual2.is_none()); // casual 是 Weekly, AllTime 查不到
    }
}
