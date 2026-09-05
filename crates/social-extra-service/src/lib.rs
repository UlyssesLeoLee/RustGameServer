#![allow(clippy::result_large_err)]

//! social-extra-service —— 社交扩展域

pub mod entity;
pub mod error;
pub mod service;
pub use error::{Error, Result};
pub use service::SocialExtraServiceImpl;

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
