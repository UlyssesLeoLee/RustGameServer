//! NIF (Native Implemented Function) 桥接 stub (per ADR-006 Option A)
//!
//! ## 范围
//! 协议网关 (network-gateway) 是 RGS 7 域 gRPC 业务在 Erlang 侧的入口.
//! 闪烁之光客户端 → web_conn.erl (BEAM) → NIF 调 RGS gRPC 业务域 → 返回字节流.
//!
//! ## 7 域 gRPC 目标 (per 9/4 改进路线图 Phase 2)
//! - player-service: 账号/角色/资产 (proto_code 101xx-103xx)
//! - economy-service: 货币/商城/拍卖 (proto_code 201xx-205xx)
//! - scene-service (NEW): 场景/移动 (proto_code 102xx-103xx)
//! - battle-service (NEW): 战斗/PVE (proto_code 200xx-205xx)
//! - batch-service: 批量任务 (per 9/1 REQ)
//! - admin-service: 后台管理
//! - cluster-ops: 健康检查 / metrics
//!
//! ## 本骨架 (Phase 1.5 stub)
//! - `GrpcTarget` 枚举: 7 域 gRPC 目标地址 (占位)
//! - `bridge()` 函数: 接收 (code, payload), 返回 (rcode, response_bytes)
//! - 不实际调 gRPC client (per Phase 1.5 + Phase 3 联调)
//!
//! ## 真实实装路径 (Phase 1.5)
//! 1. 装 rustler 0.36 + Erlang OTP 26.2
//! 2. 加 `rustler = "0.36"` + `rustler_init!` macro 到 lib.rs
//! 3. `#[rustler::nif]` 注解 `bridge` 函数, 让 BEAM 调
//! 4. 内部 `tonic::transport::Channel` 调 RGS 7 域 gRPC
//! 5. 业务响应 → Bytes → BEAM term → Erlang gen_server reply
//!
//! ## 参考
//! - ADR-006 Option A (RGS 内嵌 BEAM via rustler)
//! - 9/4 改进路线图 Phase 2: 7 域 RPC 实现

use bytes::Bytes;

/// 7 域 gRPC 业务目标 (per 9/4 改进路线图 Phase 2 7 域)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrpcTarget {
    /// 账号/角色/资产 (player-service @ 50051)
    Player,
    /// 货币/商城/拍卖 (economy-service @ 50052)
    Economy,
    /// 场景/移动 (scene-service @ 50053) — NEW per 9/4
    Scene,
    /// 战斗/PVE (battle-service @ 50054) — NEW per 9/4
    Battle,
    /// 批量任务 (batch-service @ 50055) — per 9/1 REQ
    Batch,
    /// 后台管理 (admin-service @ 50056)
    Admin,
    /// 健康检查/metrics (cluster-ops @ 50057)
    ClusterOps,
}

impl GrpcTarget {
    /// 默认 gRPC 地址 (per W6 router 端口规划, 5 域 + 2 NEW)
    pub fn default_addr(self) -> &'static str {
        match self {
            GrpcTarget::Player => "http://127.0.0.1:50051",
            GrpcTarget::Economy => "http://127.0.0.1:50052",
            GrpcTarget::Scene => "http://127.0.0.1:50053",
            GrpcTarget::Battle => "http://127.0.0.1:50054",
            GrpcTarget::Batch => "http://127.0.0.1:50055",
            GrpcTarget::Admin => "http://127.0.0.1:50056",
            GrpcTarget::ClusterOps => "http://127.0.0.1:50057",
        }
    }

    /// 服务名 (proto 路径)
    pub fn service_name(self) -> &'static str {
        match self {
            GrpcTarget::Player => "player.v1.PlayerService",
            GrpcTarget::Economy => "economy.v1.EconomyService",
            GrpcTarget::Scene => "scene.v1.SceneService",
            GrpcTarget::Battle => "battle.v1.BattleService",
            GrpcTarget::Batch => "batch.v1.BatchService",
            GrpcTarget::Admin => "admin.v1.AdminService",
            GrpcTarget::ClusterOps => "cluster_ops.v1.ClusterOpsService",
        }
    }

    /// 7 域全枚举 (admin RPC / NIF init 用)
    pub const ALL: [GrpcTarget; 7] = [
        GrpcTarget::Player,
        GrpcTarget::Economy,
        GrpcTarget::Scene,
        GrpcTarget::Battle,
        GrpcTarget::Batch,
        GrpcTarget::Admin,
        GrpcTarget::ClusterOps,
    ];
}

/// NIF 桥接结果 (Phase 1.5 stub: 不真调 gRPC, 仅返回路由决策)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeResult {
    /// gRPC 目标域
    pub target: GrpcTarget,
    /// gRPC method 名 (per router)
    pub method: String,
    /// 响应 rcode (0 = OK)
    pub rcode: u32,
    /// 响应 payload (Phase 1.5 stub: 占位, 真实实现 = gRPC response bytes)
    pub response_payload: Bytes,
}

/// NIF 桥接 (Phase 1.5 stub)
///
/// 真实实装: 调对应域 `tonic::transport::Channel` + service client method
/// 当前: 返回路由决策 + 占位 payload, Phase 1.5 接 rustler 注解
pub fn bridge(target: GrpcTarget, method: &str, _payload: &[u8]) -> BridgeResult {
    // Phase 1.5 stub: 不真调 gRPC client, 返回路由决策 + 占位 payload
    let response_payload = Bytes::from_static(b"NIF bridge stub: Phase 1.5 will call gRPC client");
    BridgeResult {
        target,
        method: method.to_string(),
        rcode: 0,
        response_payload,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_seven_targets_distinct_addrs() {
        let addrs: Vec<_> = GrpcTarget::ALL.iter().map(|t| t.default_addr()).collect();
        let unique: std::collections::HashSet<_> = addrs.iter().collect();
        assert_eq!(unique.len(), 7, "7 域 gRPC 地址必须唯一");
    }

    #[test]
    fn all_seven_targets_have_service_name() {
        for t in GrpcTarget::ALL {
            assert!(t.service_name().contains('.'), "service name 需 .v1.ServiceName 格式");
        }
    }

    #[test]
    fn bridge_returns_rcode_zero_for_stub() {
        let r = bridge(GrpcTarget::Player, "CreateCharacter", b"hello");
        assert_eq!(r.rcode, 0);
        assert_eq!(r.target, GrpcTarget::Player);
        assert_eq!(r.method, "CreateCharacter");
        assert!(!r.response_payload.is_empty());
    }

    #[test]
    fn bridge_seven_targets_round_trip() {
        for t in GrpcTarget::ALL {
            let r = bridge(t, "Ping", b"");
            assert_eq!(r.target, t);
            assert_eq!(r.rcode, 0);
        }
    }
}
