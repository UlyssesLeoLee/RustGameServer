//! replay-service —— 卡牌游戏回放微服务 (per RGS-DTL-038 §3 DEC-038-03 + 桶 13)
//!
//! ## 域职责 (per DEC-038-03 推荐 A)
//! - 元数据存 PostgreSQL (replays 表, per §7.1 #7)
//! - 回放数据存对象存储 (cluster-ops S3-兼容), 本地用 LocalFs 模拟
//! - 4 RPC + 1 内部 + 1 健康检查
//! - 生命周期: 天梯 90d / 休闲 7d / 房间 30d
//!
//! ## 桶 13 实化
//! - 1 entity: ReplayMeta (回放元数据) + 1 业务实体 Replay (元数据 + 数据)
//! - 1 enum: ReplayMode (模式)
//! - Pg + InMemory 双实现 repository
//! - StorageBackend trait: put / get / delete / list / exists / size
//! - LocalFsBackend: 本地文件系统后端 (mock cluster-ops)
//! - InMemoryBackend: 内存后端 (单测)
//! - 5 RPC handler 完整业务逻辑
//! - 1 张表 migration
//!
//! ## 集成
//! - match-service session 结束自动调 SaveReplay (TODO 推 W36+)
//! - 客户端: 回放拉 / 流 (per DTL-038 §2 跨域交互)
//!
//! ## DB
//! 独立 replay_db (per ARC-008 5 独立 DB 原则的卡牌游戏 6 域独立 DB 扩展)
//!
//! ## mTLS
//! per RGS-REV-007 CH4 / DEC-015 P1 — 默认强制 mTLS, RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod proto;
pub mod entity;
pub mod error;
pub mod storage;
pub mod repository;
pub mod service;
pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub use error::{Error, Result};
pub use repository::{
    InMemoryReplayRepository, Page, PageRequest, PgReplayRepository, ReplayRepository,
};
pub use service::{ReplayDomainService, ReplayServiceImpl};
pub use storage::{InMemoryBackend, LocalFsBackend, StorageBackend};
