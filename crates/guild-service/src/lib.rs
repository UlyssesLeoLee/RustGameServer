#![allow(clippy::result_large_err)]

//! guild-service —— 公会(联盟)域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::GuildServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
