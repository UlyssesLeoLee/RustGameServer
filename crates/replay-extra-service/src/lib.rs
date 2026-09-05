#![allow(clippy::result_large_err)]

//! replay-extra-service —— 视频/录像 扩展域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::ReplayExtraServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
