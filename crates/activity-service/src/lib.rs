#![allow(clippy::result_large_err)]

//! activity-service —— 活动运营域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::ActivityServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
