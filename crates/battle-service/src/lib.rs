//! battle-service —— 7 域战斗微服务 (per 路线图 §3 W5 + 9/4 MD §2 + 闪烁之光借鉴)
//!
//! 域职责 (7 域独立 Lead, per 8/21 JST 5 域 → 9/1 JST batch 域 → 9/5 JST battle 域):
//!   1. BattleEngineService  - 战斗引擎 (proto_200, 31 RPC)
//!   2. PvPService           - 1 服务覆盖 6 个 PVP 变体 (proto_202, 30 RPC 数据驱动)
//!   3. BossService          - PVE BOSS (proto_205, 15 RPC)
//!   4. RoomService          - 房间战 + 矿战 (proto_206, 46 RPC)
//!   5. InstanceService      - 副本 (proto_207, 6 RPC)
//!   6. EndlessTowerService  - 无尽塔 (proto_239, 12 RPC)
//!   7. EscortService        - 护送 (proto_240, 17 RPC)
//!   8. HolyEquipService     - 圣器养成 (proto_241, 23 RPC)
//!   9. GuildWarService      - 公会战 (proto_242, 17 RPC)
//!  10. CrossServerService   - 跨服 PVP (proto_243, 19 RPC, 复用 PvPService 数据驱动模式)
//!  11. ExpeditionService    - 远征 (proto_244, 15 RPC)
//!  12. HolidayActivityService - 1 服务覆盖 9 个 holiday_* 变体 (proto_248, 18 RPC 数据驱动)
//!
//! 反例原则 (per 9/4 MD §4 + 路线图 §0.3):
//!   - 6 个 PVP 变体 不重复 6 套代码, 1 个 PvPService + PvPConfig (ranked/casual/cross-server/...)
//!   - 9 个 holiday_* 活动 不重复 9 套代码, 1 个 HolidayActivityService + ActivityConfig (bid:93031/...)
//!
//! 业务实装策略 (per W5 简报 + L1/L1.1 DoD):
//!   - 30 RPC 真实逻辑 (核心战斗生命周期 / 数据驱动框架 / 业务校验)
//!   - 220 RPC Unimplemented stub (后续 Phase 3 业务实装)
//!   - 30+ UTs (per L1.1, 覆盖 30 个真实 RPC + 枚举/校验/数据驱动模式)
//!
//! DB: 独立 battle_db (per ARC-008 5 独立 DB 原则 → 7 域扩展)
//! gRPC API: battle/v1/battle.proto (12 service, 250 RPC 含 12 HealthCheck)

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod proto;
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
pub mod entity;
pub mod error;
pub mod service;
pub mod config;

pub use error::{Error, Result};
pub use entity::{
    BattleMode, BattleOutcome, BattlePhase, EscortQuality, HolidayActivity, HolidayReward,
    MineResource, PvpMode, PvpRanking, RoomBuff, RoomType,
};
pub use service::{
    BattleServiceImpl, BossServiceImpl, CrossServerServiceImpl, EndlessTowerServiceImpl,
    EscortServiceImpl, ExpeditionServiceImpl, GuildWarServiceImpl, HolidayActivityServiceImpl,
    HolyEquipServiceImpl, InstanceServiceImpl, PvPServiceImpl, RoomServiceImpl,
};
