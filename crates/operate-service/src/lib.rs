#![allow(clippy::result_large_err)]

//! operate-service —— 运营活动 域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::OperateServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
