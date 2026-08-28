#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! social-service —— 5 域社交微服务业务骨架。
//!
//! 域职责：好友、公会、聊天、邮件。
//! 规范：RGS-REQ-026 / RGS-BAS-026 / RGS-DTL-026 / RGS-SPEC-DTL-026。
//! DB：独立 social_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：social/v1/social.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.6 实化：entity 2 个 + Repository trait + PgRepository sqlx impl + InMemoryRepository 测用。

pub mod entity;
pub mod error;
pub mod push_delivery;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    GuildMemberRepository, GuildRepository, InMemoryGuildMemberRepository, InMemoryGuildRepository,
    PgGuildMemberRepository, PgGuildRepository,
};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
