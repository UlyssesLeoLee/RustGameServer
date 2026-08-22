#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! match-service —— 5 域匹配微服务业务骨架。
//!
//! 域职责：房间匹配、对战撮合、Match Slot Reservation、不可逆比赛结算。
//! 规范：RGS-REQ-016 / RGS-BAS-016 / RGS-DTL-016 / RGS-SPEC-DTL-016。
//! DB：独立 match_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：match/v1/match.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.1 骨架：4 子模块（error / service / repository / entity）；
//! 实际业务逻辑待 WF-1-54.5-54.7。

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
