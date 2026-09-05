#![allow(clippy::result_large_err)]

//! leaderboard-extra-service —— 排行榜 + 图鉴 扩展域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::LeaderboardExtraServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
