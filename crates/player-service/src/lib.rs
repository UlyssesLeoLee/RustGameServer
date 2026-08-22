//! player-service —— 5 域玩家域微服务
//!
//! 域职责：账号生命周期 / 角色档案 / 好友关系 / 跨服身份 (active-active 模式)
//! 规范：RGS-REQ-018 / RGS-BAS-018 / RGS-DTL-018 / RGS-SPEC-DTL-018
//! DB：独立 player_db（per ARC-008 5 独立 DB 原则）
//! gRPC API：player/v1/player.proto
//!
//! 53.2 占位：cargo new 默认 lib.rs 已替换为 RGS 域占位。
//! 编码实现见 WBS v0.3 §2A.2.54 WF-1-54.1 / 54.6 / 54.7。
