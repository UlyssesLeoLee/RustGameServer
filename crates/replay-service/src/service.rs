//! replay-service 域 Service 业务实装 (per RGS-DTL-038 §3 DEC-038-03 + 桶 13)
//!
//! ## 4 RPC (per replay.proto ReplayService)
//! 1. HealthCheck
//! 2. SaveReplay (内部 / match-service session 结束调用)
//! 3. GetReplay (一次性拉完整回放)
//! 4. ListReplays (按 player / mode 过滤 + 分页)
//! 5. StreamReplay (server streaming, 大回放用)
//!
//! ## 设计原则
//! - 元数据走 ReplayRepository (PostgreSQL)
//! - 数据走 StorageBackend (对象存储: LocalFs mock / S3 生产)
//! - SaveReplay: 写数据到对象存储 + 写元数据到 PG (顺序: 先存储后元数据, 失败可补偿)
//! - StreamReplay: server streaming, 按 chunk 推送 (默认 64 KiB)
//! - 集成: match-service session 结束自动调 SaveReplay (TODO 推 W36+)
//!
//! ## 生命周期
//! - 启动时检查过期 (delete_expired cron, TODO)
//! - 默认 TTL: 天梯 90d / 休闲 7d / 房间 30d

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use futures::stream::{self, Stream, StreamExt};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::entity::{Replay, ReplayChunk, ReplayFilter, ReplayMeta, ReplayMode};
use crate::error::Error;
use crate::repository::{PageRequest, ReplayRepository};
use crate::storage::{build_object_key, StorageBackend};
use crate::Result;

/// 默认 chunk size (bytes) — StreamReplay 流式读取
pub const DEFAULT_CHUNK_SIZE: u32 = 64 * 1024; // 64 KiB
/// 最小 chunk size (防止单 chunk 太小)
pub const MIN_CHUNK_SIZE: u32 = 1024; // 1 KiB
/// 最大 chunk size (防止单 chunk 太大)
pub const MAX_CHUNK_SIZE: u32 = 1024 * 1024; // 1 MiB

// ============================================================================
// Domain Service trait
// ============================================================================

/// replay-service 域 Service trait (业务层, gRPC 桥接在 grpc_service 子模块)
#[async_trait]
pub trait ReplayDomainService: Send + Sync {
    /// 1. 健康检查
    async fn health_check(&self) -> Result<bool>;

    /// 2. 保存回放 (match-service / saga 调用)
    ///
    /// 业务流:
    /// 1. 校验参数 (player_a 非空, match_id 非 nil)
    /// 2. 生成 UUID + object_key
    /// 3. 写入对象存储 (StorageBackend.put)
    /// 4. 写入元数据 (ReplayRepository.insert)
    /// 5. 返回 SaveReplayResponse (含 meta + object_key)
    async fn save_replay(
        &self,
        match_id: Uuid,
        player_a: String,
        player_b: Option<String>,
        mode: ReplayMode,
        data: Vec<u8>,
        duration_secs: u32,
        custom_ttl_secs: i64,
        saga_id: Option<String>,
    ) -> Result<ReplayMeta>;

    /// 3. 拉取完整回放 (元数据 + 数据)
    async fn get_replay(&self, replay_id: Uuid) -> Result<Replay>;

    /// 4. 列出回放 (按过滤 + 分页)
    async fn list_replays(
        &self,
        filter: &ReplayFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<ReplayMeta>, i64, bool)>; // (items, total, has_next)

    /// 5. 流式读取回放 (返回 stream, 内部 chunked 推送)
    async fn stream_replay(
        &self,
        replay_id: Uuid,
        chunk_size: u32,
        start_offset: u64,
    ) -> Result<Box<dyn Stream<Item = Result<ReplayChunk>> + Send + Unpin>>;

    /// 6. 删除回放 (元数据 + 对象数据, 内部清理用, 默认不暴露 client)
    async fn delete_replay(&self, replay_id: Uuid) -> Result<bool>;

    /// 7. 清理过期元数据 (cron job 用, TODO 启动时调用)
    /// 返回: (清理元数据数量, 清理对象 key 列表)
    async fn cleanup_expired(&self) -> Result<(u64, Vec<String>)>;
}

// ============================================================================
// ServiceImpl
// ============================================================================

pub struct ReplayServiceImpl {
    repo: Arc<dyn ReplayRepository>,
    storage: Arc<dyn StorageBackend>,
}

impl ReplayServiceImpl {
    /// 工厂: 注入 repo + storage
    pub fn new(repo: Arc<dyn ReplayRepository>, storage: Arc<dyn StorageBackend>) -> Self {
        Self { repo, storage }
    }
}

#[async_trait]
impl ReplayDomainService for ReplayServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn save_replay(
        &self,
        match_id: Uuid,
        player_a: String,
        player_b: Option<String>,
        mode: ReplayMode,
        data: Vec<u8>,
        duration_secs: u32,
        custom_ttl_secs: i64,
        _saga_id: Option<String>,
    ) -> Result<ReplayMeta> {
        // 1. 校验
        if player_a.is_empty() {
            return Err(Error::Validation("player_a must not be empty".to_string()));
        }
        if match_id.is_nil() {
            return Err(Error::Validation("match_id must not be nil UUID".to_string()));
        }
        if mode == ReplayMode::Unspecified {
            return Err(Error::Validation("mode must be specified".to_string()));
        }

        let now = Utc::now();
        let replay_id = Uuid::new_v4();
        let object_key = build_object_key(&replay_id, now);

        // 2. 写对象存储 (per DEC-038-03, 顺序: 先存后元数据)
        self.storage
            .put(&object_key, Bytes::from(data.clone()))
            .await
            .map_err(|e| Error::Storage(format!("save_replay put failed: {}", e)))?;

        // 3. 构造元数据
        let mut meta = ReplayMeta::new(match_id, player_a, player_b, mode, object_key.clone());
        meta.object_size = data.len() as i64;
        meta.duration_secs = duration_secs;
        if custom_ttl_secs > 0 {
            meta = meta.with_custom_ttl(custom_ttl_secs);
        }
        // replay_id 用我们生成的 (而不是 ReplayMeta::new 内部生成的)
        meta.replay_id = replay_id;
        meta.object_size = data.len() as i64;
        meta.validate()?;

        // 4. 写元数据 (元数据失败时, 删掉已写入的对象 — best-effort 补偿)
        if let Err(e) = self.repo.insert(&meta).await {
            // 补偿: 删除已写入的对象
            let _ = self.storage.delete(&meta.object_key).await;
            return Err(e);
        }

        Ok(meta)
    }

    async fn get_replay(&self, replay_id: Uuid) -> Result<Replay> {
        // 1. 拉元数据
        let meta = self
            .repo
            .find_by_id(replay_id)
            .await?
            .ok_or_else(|| Error::ReplayNotFound(replay_id.to_string()))?;

        // 2. 拉数据
        let data = self
            .storage
            .get(&meta.object_key)
            .await?
            .ok_or_else(|| Error::Storage(format!(
                "object missing for replay {}: {}",
                replay_id, meta.object_key
            )))?;

        Ok(Replay::new(meta, data.to_vec()))
    }

    async fn list_replays(
        &self,
        filter: &ReplayFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<ReplayMeta>, i64, bool)> {
        let page = self.repo.list(filter, page_req).await?;
        let has_next = page.has_next;
        Ok((page.items, page.total, has_next))
    }

    async fn stream_replay(
        &self,
        replay_id: Uuid,
        chunk_size: u32,
        start_offset: u64,
    ) -> Result<Box<dyn Stream<Item = Result<ReplayChunk>> + Send + Unpin>> {
        // 1. 拉元数据 (验证存在)
        let meta = self
            .repo
            .find_by_id(replay_id)
            .await?
            .ok_or_else(|| Error::ReplayNotFound(replay_id.to_string()))?;

        // 2. 校验 chunk_size
        let chunk_size = if chunk_size == 0 {
            DEFAULT_CHUNK_SIZE
        } else {
            chunk_size.clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE)
        };

        // 3. 构造 stream
        let storage = self.storage.clone();
        let object_key = meta.object_key.clone();
        let total_size = meta.object_size.max(0) as u64;
        let chunk_index_start = (start_offset / chunk_size as u64) as u32;

        let stream = stream::unfold(
            (chunk_index_start, start_offset),
            move |(idx, offset)| {
                let storage = storage.clone();
                let object_key = object_key.clone();
                async move {
                    if offset >= total_size {
                        return None;
                    }
                    // 读一段 (从 offset 开始, chunk_size bytes)
                    let read_len = std::cmp::min(chunk_size as u64, total_size - offset) as usize;
                    let data = match storage.get(&object_key).await {
                        Ok(Some(d)) => d,
                        Ok(None) => {
                            return Some((
                                Err(Error::Storage(format!(
                                    "object missing: {}",
                                    object_key
                                ))),
                                (idx, offset + chunk_size as u64),
                            ));
                        }
                        Err(e) => {
                            return Some((
                                Err(Error::StreamFailed(format!(
                                    "read failed at offset {}: {}",
                                    offset, e
                                ))),
                                (idx, offset + chunk_size as u64),
                            ));
                        }
                    };
                    // 截取 [offset, offset+read_len)
                    let start = offset as usize;
                    let end = (offset + read_len as u64) as usize;
                    let payload = data.slice(start..end).to_vec();
                    let new_offset = offset + read_len as u64;
                    let is_last = new_offset >= total_size;
                    let chunk = ReplayChunk::new(replay_id, offset, payload, is_last, idx);
                    Some((Ok(chunk), (idx + 1, new_offset)))
                }
            },
        );

        Ok(Box::new(Box::pin(stream)))
    }

    async fn delete_replay(&self, replay_id: Uuid) -> Result<bool> {
        // 1. 查元数据 (拿 object_key)
        let meta = self.repo.find_by_id(replay_id).await?;
        let meta = match meta {
            Some(m) => m,
            None => return Ok(false),
        };
        // 2. 删对象 (best-effort, 失败仅 warn)
        let _ = self.storage.delete(&meta.object_key).await;
        // 3. 删元数据
        self.repo.delete(replay_id).await
    }

    async fn cleanup_expired(&self) -> Result<(u64, Vec<String>)> {
        let now = Utc::now();
        // 1. 查所有过期元数据 (需先 list 再删, 因为 InMemory 删完无法拿 key)
        //    注: PG 实现可直接走 delete_expired + 单独 list, 这里用 list 一次拿全 key
        let all_page = self
            .repo
            .list(
                &ReplayFilter {
                    include_expired: true,
                    ..Default::default()
                },
                PageRequest {
                    page: 1,
                    page_size: 1000,
                },
            )
            .await?;
        let expired: Vec<ReplayMeta> = all_page
            .items
            .into_iter()
            .filter(|m| m.expires_at < now)
            .collect();
        let expired_keys: Vec<String> = expired.iter().map(|m| m.object_key.clone()).collect();
        // 2. 删对象 (best-effort)
        for key in &expired_keys {
            let _ = self.storage.delete(key).await;
        }
        // 3. 删元数据
        let removed = self.repo.delete_expired(now).await?;
        Ok((removed, expired_keys))
    }
}

// ============================================================================
// gRPC 桥接
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as replay_proto;

    pub struct ReplayGrpcService {
        pub impl_: Arc<ReplayServiceImpl>,
    }

    impl ReplayGrpcService {
        pub fn new(impl_: Arc<ReplayServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    fn parse_mode(v: i32) -> Result<ReplayMode> {
        match v {
            x if x == replay_proto::ReplayMode::Ranked as i32 => Ok(ReplayMode::Ranked),
            x if x == replay_proto::ReplayMode::Casual as i32 => Ok(ReplayMode::Casual),
            x if x == replay_proto::ReplayMode::Room as i32 => Ok(ReplayMode::Room),
            x if x == replay_proto::ReplayMode::PveAi as i32 => Ok(ReplayMode::PveAi),
            _ => Err(Error::Validation(format!("unknown ReplayMode enum value: {}", v))),
        }
    }

    fn mode_to_proto(m: ReplayMode) -> i32 {
        match m {
            ReplayMode::Ranked => replay_proto::ReplayMode::Ranked as i32,
            ReplayMode::Casual => replay_proto::ReplayMode::Casual as i32,
            ReplayMode::Room => replay_proto::ReplayMode::Room as i32,
            ReplayMode::PveAi => replay_proto::ReplayMode::PveAi as i32,
            ReplayMode::Unspecified => replay_proto::ReplayMode::Unspecified as i32,
        }
    }

    fn meta_to_proto(m: &ReplayMeta) -> replay_proto::ReplayMeta {
        replay_proto::ReplayMeta {
            replay_id: m.replay_id.to_string(),
            match_id: m.match_id.to_string(),
            player_a: m.player_a.clone(),
            // proto3 string is empty for None (sentinel)
            player_b: m.player_b.clone().unwrap_or_default(),
            mode: mode_to_proto(m.mode) as i32,
            object_key: m.object_key.clone(),
            object_size: m.object_size,
            duration_secs: m.duration_secs,
            created_at: Some(common_proto::Timestamp {
                seconds: m.created_at.timestamp(),
                nanos: m.created_at.timestamp_subsec_nanos() as i32,
            }),
            expires_at: Some(common_proto::Timestamp {
                seconds: m.expires_at.timestamp(),
                nanos: m.expires_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }

    fn parse_uuid(s: &str, field: &str) -> Result<Uuid> {
        Uuid::parse_str(s).map_err(|_| {
            Error::Validation(format!("invalid UUID for {}: '{}'", field, s))
        })
    }

    fn parse_page(req: &Option<common_proto::PageRequest>) -> PageRequest {
        req.as_ref()
            .map(|p| PageRequest {
                page: if p.page == 0 { 1 } else { p.page },
                page_size: p.page_size,
            })
            .unwrap_or_default()
    }

    #[tonic::async_trait]
    impl replay_proto::replay_service_server::ReplayService for ReplayGrpcService {
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

        async fn save_replay(
            &self,
            request: Request<replay_proto::SaveReplayRequest>,
        ) -> std::result::Result<Response<replay_proto::SaveReplayResponse>, Status> {
            let req = request.get_ref();
            let match_id = parse_uuid(&req.match_id, "match_id")?;
            let player_b = if req.player_b.is_empty() {
                None
            } else {
                Some(req.player_b.clone())
            };
            let mode = parse_mode(req.mode)?;
            let saga_id = if req.saga_id.is_empty() {
                None
            } else {
                Some(req.saga_id.clone())
            };
            let meta = self
                .impl_
                .save_replay(
                    match_id,
                    req.player_a.clone(),
                    player_b,
                    mode,
                    req.data.clone(),
                    req.duration_secs,
                    req.custom_ttl_secs,
                    saga_id,
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(replay_proto::SaveReplayResponse {
                meta: Some(meta_to_proto(&meta)),
                object_key: meta.object_key.clone(),
            }))
        }

        async fn get_replay(
            &self,
            request: Request<replay_proto::GetReplayRequest>,
        ) -> std::result::Result<Response<replay_proto::Replay>, Status> {
            let req = request.get_ref();
            let id = parse_uuid(&req.replay_id, "replay_id")?;
            let r = self
                .impl_
                .get_replay(id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(replay_proto::Replay {
                meta: Some(meta_to_proto(&r.meta)),
                data: r.data,
            }))
        }

        async fn list_replays(
            &self,
            request: Request<replay_proto::ListReplaysRequest>,
        ) -> std::result::Result<Response<replay_proto::ReplayList>, Status> {
            let req = request.get_ref();
            let mode_filter = if req.mode_filter == 0 {
                None
            } else {
                Some(parse_mode(req.mode_filter)?)
            };
            let player_a_filter = if req.player_a_filter.is_empty() {
                None
            } else {
                Some(req.player_a_filter.clone())
            };
            let player_b_filter = if req.player_b_filter.is_empty() {
                None
            } else {
                Some(req.player_b_filter.clone())
            };
            let filter = ReplayFilter {
                player_a_filter,
                player_b_filter,
                mode_filter,
                include_expired: req.include_expired,
            };
            let page_req = parse_page(&req.page);
            let (items, total, has_next) = self
                .impl_
                .list_replays(&filter, page_req)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let page = page_req;
            Ok(Response::new(replay_proto::ReplayList {
                items: items.iter().map(meta_to_proto).collect(),
                page: Some(common_proto::PageResponse {
                    total: total as u32,
                    has_next,
                    next_cursor: if has_next {
                        format!("{}", page.page + 1)
                    } else {
                        String::new()
                    },
                }),
            }))
        }

        type StreamReplayStream =
            std::pin::Pin<Box<dyn Stream<Item = std::result::Result<replay_proto::ReplayChunk, Status>> + Send>>;

        async fn stream_replay(
            &self,
            request: Request<replay_proto::StreamReplayRequest>,
        ) -> std::result::Result<Response<Self::StreamReplayStream>, Status> {
            let req = request.get_ref();
            let id = parse_uuid(&req.replay_id, "replay_id")?;
            let chunk_size = req.chunk_size;
            let start_offset = req.start_offset;
            let inner = self
                .impl_
                .stream_replay(id, chunk_size, start_offset)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let outer = inner.map(|chunk_result| {
                chunk_result
                    .map(|c| replay_proto::ReplayChunk {
                        replay_id: c.replay_id.to_string(),
                        offset: c.offset,
                        payload: c.payload,
                        is_last: c.is_last,
                        chunk_index: c.chunk_index,
                    })
                    .map_err(Into::<tonic::Status>::into)
            });
            Ok(Response::new(Box::pin(outer)))
        }
    }
}
