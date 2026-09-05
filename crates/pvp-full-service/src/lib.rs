#![allow(clippy::result_large_err)]

//! pvp-full-service —— PVP/竞技 完整版域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::PvpFullServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
