//! i18n-service —— 卡牌游戏多语言文案微服务
//!
//! 域职责: 多语言文案(GetText / GetTexts / ListLanguages) + Redis 缓存 + DB 持久化
//! 规范: RGS-REQ-038 §NFR-005 + RGS-DTL-038 §4.1 + DEC-038-05 推荐 A
//! DB: 独立 i18n_db(per ARC-008 5 独立 DB 原则)
//! gRPC API: i18n/v1/i18n.proto(per WF-1-54.2 + WBS v0.3 §2.2 桶 14)
//!
//! ## 实化状态
//! - 桶 14 (commit 01f4be5): skeleton 8 文件 (entity / error / repository / migrations / etc)
//! - W35 桶 14 补完 (本 PR): 3 RPC handler 完整业务 + 6 UT + Redis 缓存占位 (BTreeMap + 5 min TTL)

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{I18nRepository, InMemoryI18nRepository, PgI18nRepository};
pub use service::{
    GetTextResult, GetTextsResult, I18nService, I18nServiceImpl, TtlCache,
};

pub mod proto {
    #![allow(clippy::all)]
    /// i18n.v1 生成的 gRPC 类型
    pub mod v1 {
        tonic::include_proto!("i18n.v1");
    }
}

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
