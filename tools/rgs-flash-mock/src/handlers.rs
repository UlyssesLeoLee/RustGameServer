// rgs-flash-mock v0.3 — 12 类别 22 RPC stub handlers + 真实 7 域 mTLS gRPC 调用
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §3 + §2.3 数据流
//
// v0.1 PoC: HTTP 路由 → 直接返回 stub response
// v0.2 升级: HTTP 路由 → 调 RGS gRPC backend (mTLS 业务级) → 真实 health check / profile 查询
//           → 失败 fallback 到 mock response + 记录 last_error
// v0.3 升级: 7 域 mTLS (5 域 + card + leaderboard) → 真实 HealthCheck + GetPlayerCollection + GetRankedLeaderboard
//           + 2 新 handler: /card/collection + /rank/leaderboard (升级为真实 leaderboard gRPC 调用)

use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use std::time::Instant;
use tonic::Request;

use crate::gap_matrix::{RpcCategory, RpcStatus};
use crate::grpc_clients::GrpcClients;
use crate::grpc_clients::proto::common::v1 as common;
use crate::AppState;

// === 1. 场景/移动 (148 total) — RGS TCG 无场景, N-A ===
pub mod scene {
    use super::*;

    #[derive(Deserialize)]
    pub struct GetSceneReq {
        pub scene_id: Option<u32>,
        pub player_id: Option<String>,
    }

    #[post("/scene/get")]
    pub async fn get_scene(
        data: web::Data<AppState>,
        _req: web::Json<GetSceneReq>,
    ) -> HttpResponse {
        // RGS TCG 无场景, 但仍调 match HealthCheck 验 5 域 mTLS
        let grpc_result = call_match_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(101, RpcCategory::Scene, RpcStatus::NotApplicable, "场景查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(101, RpcStatus::NotApplicable, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 101,
            "name": "GetScene",
            "status": "n-a",
            "reason": "RGS TCG 无场景/移动概念, 标记 N-A (per design §3 category 1)",
            "grpc_probe": grpc_result,
            "mock_response": { "scene_id": 0, "npcs": [], "terrain": "n-a" },
        }))
    }

    #[derive(Deserialize)]
    pub struct MoveReq {
        pub x: Option<f32>,
        pub y: Option<f32>,
    }

    #[post("/scene/move")]
    pub async fn move_player(
        data: web::Data<AppState>,
        _req: web::Json<MoveReq>,
    ) -> HttpResponse {
        let grpc_result = call_match_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(102, RpcCategory::Scene, RpcStatus::NotApplicable, "玩家移动");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(102, RpcStatus::NotApplicable, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 102,
            "name": "MovePlayer",
            "status": "n-a",
            "reason": "RGS TCG 无移动, 标记 N-A",
            "grpc_probe": grpc_result,
            "mock_response": { "moved": false },
        }))
    }
}

// === 2. 角色养成 (198 total) — Partial ===
pub mod role {
    use super::*;

    #[derive(Deserialize)]
    pub struct GetProfileReq {
        pub player_id: Option<String>,
    }

    #[post("/role/profile")]
    pub async fn get_profile(
        data: web::Data<AppState>,
        req: web::Json<GetProfileReq>,
    ) -> HttpResponse {
        // 真实调 player GetPlayerProfile
        let grpc_result = call_player_get_profile(
            &data.clients,
            req.player_id.clone().unwrap_or_else(|| "unknown".to_string()),
        )
        .await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(201, RpcCategory::Role, RpcStatus::Partial, "玩家档案查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(201, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 201,
            "name": "GetPlayerProfile",
            "status": status_str,
            "reason": "RGS v2 部分实装, 部分字段缺失 (per design §3 category 2)",
            "rgs_call": "player-service:50051 GetPlayerProfile",
            "grpc_probe": grpc_result,
            "mock_response": {
                "player_id": req.player_id.clone().unwrap_or_else(|| "unknown".to_string()),
                "level": 1,
                "exp": 0,
                "gold": 0,
                "vip_level": 0,
                "missing_fields": ["honor", "title", "avatar_frame"],
            },
        }))
    }

    #[derive(Deserialize)]
    pub struct UpgradeReq {
        pub skill_id: Option<u32>,
    }

    #[post("/role/upgrade_skill")]
    pub async fn upgrade_skill(
        data: web::Data<AppState>,
        _req: web::Json<UpgradeReq>,
    ) -> HttpResponse {
        // v0.3 升级: 调 card gRPC HealthCheck (5 域 player → 7 域 card 新域, per RGS-DTL-038 §4.4)
        let grpc_result = call_card_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(202, RpcCategory::Role, RpcStatus::Partial, "技能升级");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(202, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 202,
            "name": "UpgradeSkill",
            "status": "partial",
            "reason": "卡组养成类比, 不完全对应",
            "rgs_call": "card-service:50061 HealthCheck (v0.3 接入 card 域, 替代 v0.2 player probe)",
            "grpc_probe": grpc_result,
            "mock_response": { "skill_id": 0, "new_level": 1, "cost_gold": 100 },
        }))
    }
}

// === 3. 战斗 PVE (241 total) — Pass ===
pub mod combat {
    use super::*;

    #[post("/combat/start")]
    pub async fn start_combat(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_match_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(301, RpcCategory::Combat, RpcStatus::Pass, "开始战斗");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(301, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 301,
            "name": "StartCombat",
            "status": status_str,
            "rgs_call": "match-service:50053 CreateMatch (v0.2 用 HealthCheck probe mTLS)",
            "grpc_probe": grpc_result,
            "mock_response": { "match_id": "stub-match-001", "state": "active", "turn": 1 },
        }))
    }

    #[post("/combat/action")]
    pub async fn submit_action(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_match_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(302, RpcCategory::Combat, RpcStatus::Pass, "提交动作");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(302, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 302,
            "name": "SubmitAction",
            "status": status_str,
            "rgs_call": "match-service:50053 SubmitMove (v0.2 用 HealthCheck probe mTLS)",
            "grpc_probe": grpc_result,
            "mock_response": { "accepted": true, "next_turn": 2 },
        }))
    }
}

// === 4. PVP/竞技 (151 total) — Pass ===
pub mod pvp {
    use super::*;

    #[post("/pvp/enqueue")]
    pub async fn enqueue_pvp(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_match_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(401, RpcCategory::Pvp, RpcStatus::Pass, "PVP 排队");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(401, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 401,
            "name": "EnqueuePVP",
            "status": status_str,
            "rgs_call": "match-service:50053 EnqueueMatchmaking (v0.2 用 HealthCheck probe mTLS)",
            "grpc_probe": grpc_result,
            "mock_response": { "queue_position": 1, "estimated_wait_sec": 5 },
        }))
    }

    #[post("/pvp/get")]
    pub async fn get_pvp_match(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_match_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(402, RpcCategory::Pvp, RpcStatus::Pass, "PVP 比赛查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(402, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 402,
            "name": "GetPVPMatch",
            "status": status_str,
            "rgs_call": "match-service:50053 GetMatchState (v0.2 用 HealthCheck probe mTLS)",
            "grpc_probe": grpc_result,
            "mock_response": { "match_id": "stub-pvp-001", "opponent": "stub-opponent", "score": [0, 0] },
        }))
    }
}

// === 5. 公会 (97 total) — Partial ===
pub mod guild {
    use super::*;

    #[post("/guild/get")]
    pub async fn get_guild(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_social_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(501, RpcCategory::Guild, RpcStatus::Partial, "公会查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(501, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 501,
            "name": "GetGuild",
            "status": status_str,
            "reason": "RGS social gRPC 4/6 handler 未 wire (per FLASH-OVERLAP §3.4)",
            "rgs_call": "social-service:50054 HealthCheck (GetGuild handler v0.2 仍 stub)",
            "grpc_probe": grpc_result,
            "mock_response": { "guild_id": null, "members": 0, "missing": ["leave", "dissolve", "kick"] },
        }))
    }

    #[post("/guild/join")]
    pub async fn join_guild(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_social_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(502, RpcCategory::Guild, RpcStatus::Partial, "加入公会");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(502, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 502,
            "name": "JoinGuild",
            "status": status_str,
            "reason": "social gRPC handler 未 wire",
            "grpc_probe": grpc_result,
            "mock_response": { "joined": false, "reason": "stub-not-implemented" },
        }))
    }
}

// === 6. 经济 (90 total) — Pass ===
pub mod econ {
    use super::*;

    #[post("/econ/account")]
    pub async fn get_account(data: web::Data<AppState>) -> HttpResponse {
        // 真实调 economy GetAccount
        let grpc_result = call_economy_get_account(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(601, RpcCategory::Econ, RpcStatus::Pass, "账户查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(601, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 601,
            "name": "GetAccount",
            "status": status_str,
            "rgs_call": "economy-service:50052 GetAccount",
            "grpc_probe": grpc_result,
            "mock_response": { "gold": 1000, "diamond": 50, "energy": 100 },
        }))
    }

    #[post("/econ/auction")]
    pub async fn create_auction(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_economy_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(602, RpcCategory::Econ, RpcStatus::Pass, "创建拍卖");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(602, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 602,
            "name": "CreateAuction",
            "status": status_str,
            "rgs_call": "economy-service:50052 CreateAuction (v0.2 用 HealthCheck probe mTLS)",
            "grpc_probe": grpc_result,
            "mock_response": { "auction_id": "stub-auction-001", "price": 100 },
        }))
    }
}

// === 7. 社交 (123 total) — Partial ===
pub mod friend {
    use super::*;

    #[post("/friend/list")]
    pub async fn list(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_social_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(701, RpcCategory::Social, RpcStatus::Partial, "好友列表");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(701, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 701,
            "name": "GetFriendList",
            "status": status_str,
            "reason": "RGS social 缺好友/邮件 (per design §3 category 7)",
            "grpc_probe": grpc_result,
            "mock_response": { "friends": [], "missing": ["block", "search", "recommend"] },
        }))
    }

    #[post("/friend/send")]
    pub async fn send(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_social_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(702, RpcCategory::Social, RpcStatus::Partial, "发送消息");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(702, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 702,
            "name": "SendMessage",
            "status": status_str,
            "grpc_probe": grpc_result,
            "mock_response": { "sent": false, "reason": "stub-not-implemented" },
        }))
    }
}

// === 8. 活动运营 (184 total) — Partial ===
pub mod event {
    use super::*;

    #[post("/event/active")]
    pub async fn active(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_admin_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(801, RpcCategory::Event, RpcStatus::Partial, "活动查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(801, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 801,
            "name": "GetActiveEvent",
            "status": status_str,
            "reason": "RGS 缺数据驱动活动框架 (per handoff v0.1 §2.1.3 反例)",
            "rgs_call": "batch (task_templates) + card (v0.2 admin probe 占位)",
            "grpc_probe": grpc_result,
            "mock_response": { "events": [], "missing": ["data_driven_template"] },
        }))
    }

    #[post("/event/claim")]
    pub async fn claim(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_admin_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(802, RpcCategory::Event, RpcStatus::Partial, "领取奖励");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(802, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 802,
            "name": "ClaimReward",
            "status": status_str,
            "grpc_probe": grpc_result,
            "mock_response": { "claimed": false, "reason": "stub-not-implemented" },
        }))
    }
}

// === 9. 付费 (43 total) — Partial ===
pub mod pay {
    use super::*;

    #[post("/pay/recharge")]
    pub async fn recharge(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_economy_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(901, RpcCategory::Pay, RpcStatus::Partial, "充值");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(901, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 901,
            "name": "Recharge",
            "status": status_str,
            "reason": "RGS 抽卡/开包不同 (per design §3 category 9)",
            "rgs_call": "economy + payment (v0.2 economy probe 占位)",
            "grpc_probe": grpc_result,
            "mock_response": { "order_id": "stub-pay-001", "amount_cny": 0, "diamond_added": 0 },
        }))
    }

    #[post("/pay/history")]
    pub async fn history(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_economy_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(902, RpcCategory::Pay, RpcStatus::Partial, "充值历史");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(902, RpcStatus::Partial, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "partial" } else { "partial-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 902,
            "name": "QueryRechargeHistory",
            "status": status_str,
            "rgs_call": "economy (v0.2 economy probe)",
            "grpc_probe": grpc_result,
            "mock_response": { "history": [] },
        }))
    }
}

// === 10. 排行榜 (10 total) — Pass ===
pub mod rank {
    use super::*;

    #[post("/rank/leaderboard")]
    pub async fn leaderboard(data: web::Data<AppState>) -> HttpResponse {
        // v0.3 升级: 真实调 leaderboard GetRankedLeaderboard (7 域新域, per RGS-DTL-038 §3)
        let grpc_result = call_leaderboard_get_ranked(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1001, RpcCategory::Rank, RpcStatus::Pass, "排行榜");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1001, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1001,
            "name": "GetLeaderboard",
            "status": status_str,
            "rgs_call": "leaderboard-service:50062 GetRankedLeaderboard (v0.3 接入 leaderboard 域, 替代 v0.2 player probe)",
            "grpc_probe": grpc_result,
            "mock_response": {
                "rank_type": "ranked",
                "period": "weekly",
                "entries": [
                    { "rank": 1, "player_id": "stub-001", "score": 999 },
                    { "rank": 2, "player_id": "stub-002", "score": 888 },
                ],
            },
        }))
    }
}

// === 11. GM (37 total) — Pass ===
pub mod gm {
    use super::*;

    #[post("/gm/ban")]
    pub async fn ban(data: web::Data<AppState>) -> HttpResponse {
        // 真实调 admin BanAccount (mocked input, 5 域 admin v0.2 应支持)
        let grpc_result = call_admin_ban(&data.clients, "stub-target-player", "v0.2 mock test ban").await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1101, RpcCategory::Gm, RpcStatus::Pass, "封号");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1101, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1101,
            "name": "BanAccount",
            "status": status_str,
            "rgs_call": "admin-service:50055 BanAccount",
            "grpc_probe": grpc_result,
            "mock_response": { "banned": true, "duration_hours": 24 },
        }))
    }

    #[post("/gm/grant")]
    pub async fn grant(data: web::Data<AppState>) -> HttpResponse {
        let grpc_result = call_admin_health(&data.clients).await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1102, RpcCategory::Gm, RpcStatus::Pass, "补偿发放");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1102, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1102,
            "name": "GrantCompensation",
            "status": status_str,
            "rgs_call": "admin-service:50055 GrantCompensation (v0.2 HealthCheck probe)",
            "grpc_probe": grpc_result,
            "mock_response": { "granted": true, "items": [{ "id": 1, "count": 100 }] },
        }))
    }
}

// === 12. 卡牌 (v0.3 NEW, per RGS-DTL-038 §4.4) — Pass ===
pub mod card {
    use super::*;

    #[derive(Deserialize)]
    pub struct GetCollectionReq {
        pub player_id: Option<String>,
    }

    #[post("/card/collection")]
    pub async fn get_collection(
        data: web::Data<AppState>,
        req: web::Json<GetCollectionReq>,
    ) -> HttpResponse {
        // v0.3 NEW: 真实调 card GetPlayerCollection
        let grpc_result = call_card_get_player_collection(
            &data.clients,
            req.player_id.clone().unwrap_or_else(|| "unknown".to_string()),
        )
        .await;

        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1201, RpcCategory::Card, RpcStatus::Pass, "卡牌收藏查询");
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1201, RpcStatus::Pass, latency);
        drop(matrix);

        let status_str = if grpc_result.success { "pass" } else { "pass-fallback" };

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1201,
            "name": "GetPlayerCollection",
            "status": status_str,
            "rgs_call": "card-service:50061 GetPlayerCollection (v0.3 NEW, 7 域 mTLS 业务级)",
            "grpc_probe": grpc_result,
            "mock_response": {
                "player_id": req.player_id.clone().unwrap_or_else(|| "unknown".to_string()),
                "total_count": 0,
                "instances": [],
            },
        }))
    }
}

// === gRPC 业务级调用 helper (5 域真实 mTLS) ===

/// 通用 gRPC probe 结果 (handler 返回 JSON 包装)
#[derive(Debug, Clone, serde::Serialize)]
pub struct GrpcProbeResult {
    pub domain: String,
    pub rpc: String,
    pub success: bool,
    pub latency_ms: u64,
    pub response_summary: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl GrpcProbeResult {
    fn new(domain: &str, rpc: &str) -> Self {
        Self {
            domain: domain.to_string(),
            rpc: rpc.to_string(),
            success: false,
            latency_ms: 0,
            response_summary: None,
            error: None,
        }
    }
}

/// player HealthCheck
async fn call_player_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("player", "HealthCheck");
    let Some(mut client) = clients.player.clone() else {
        result.error = Some("player gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// player GetPlayerProfile
async fn call_player_get_profile(clients: &GrpcClients, player_id: String) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("player", "GetPlayerProfile");
    let Some(mut client) = clients.player.clone() else {
        result.error = Some("player gRPC client not connected".to_string());
        return result;
    };
    use crate::grpc_clients::player::GetPlayerProfileRequest;
    let req = Request::new(GetPlayerProfileRequest {
        request_id: format!("rgs-flash-mock-{}", chrono::Utc::now().timestamp_millis()),
        player_id,
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.get_player_profile(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let p = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "player_id": p.player_id,
                "ranked_score": p.ranked_score,
                "ranked_tier": p.ranked_tier,
                "total_matches": p.total_matches,
                "total_wins": p.total_wins,
                "collection_count": p.collection_count,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// economy HealthCheck
async fn call_economy_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("economy", "HealthCheck");
    let Some(mut client) = clients.economy.clone() else {
        result.error = Some("economy gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// economy GetAccount (per 5 域 main.rs v0.2 实装)
async fn call_economy_get_account(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("economy", "GetAccount");
    let Some(mut client) = clients.economy.clone() else {
        result.error = Some("economy gRPC client not connected".to_string());
        return result;
    };
    use crate::grpc_clients::proto::common::v1::EntityId;
    let req = Request::new(EntityId {
        id: "stub-account-id".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.get_account(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let a = resp.into_inner();
            // EntityId is a prost message (no serde derive), unwrap to string for JSON
            let id_str = a.id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
            result.response_summary = Some(serde_json::json!({
                "id": id_str,
                "status": a.status,
                "display_name": a.display_name,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// match HealthCheck
async fn call_match_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("match", "HealthCheck");
    let Some(mut client) = clients.r#match.clone() else {
        result.error = Some("match gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// social HealthCheck
async fn call_social_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("social", "HealthCheck");
    let Some(mut client) = clients.social.clone() else {
        result.error = Some("social gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// admin HealthCheck
async fn call_admin_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("admin", "HealthCheck");
    let Some(mut client) = clients.admin.clone() else {
        result.error = Some("admin gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// admin BanAccount (per 5 域 admin v0.2 实装, 仅 stub 测试用)
async fn call_admin_ban(clients: &GrpcClients, account_id: &str, reason: &str) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("admin", "BanAccount");
    let Some(mut client) = clients.admin.clone() else {
        result.error = Some("admin gRPC client not connected".to_string());
        return result;
    };
    use crate::grpc_clients::admin::BanAccountRequest;
    let req = Request::new(BanAccountRequest {
        request_id: format!("rgs-flash-mock-{}", chrono::Utc::now().timestamp_millis()),
        account_id: account_id.to_string(),
        reason: reason.to_string(),
        duration_seconds: 3600,
        force_disconnect_session: true,
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.ban_account(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let b = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": b.status,
                "op": b.op,
                "accepted_at_ms": b.accepted_at_ms,
                "disconnected_sessions": b.disconnected_sessions,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

// === v0.3 NEW: card + leaderboard 域 mTLS 业务级 helper ===

/// card HealthCheck (v0.3 NEW, per RGS-DTL-038 §4.4)
async fn call_card_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("card", "HealthCheck");
    let Some(mut client) = clients.card.clone() else {
        result.error = Some("card gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// card GetPlayerCollection (v0.3 NEW, per RGS-DTL-038 §4.4)
async fn call_card_get_player_collection(clients: &GrpcClients, player_id: String) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("card", "GetPlayerCollection");
    let Some(mut client) = clients.card.clone() else {
        result.error = Some("card gRPC client not connected".to_string());
        return result;
    };
    use crate::grpc_clients::card::GetPlayerCollectionRequest;
    use crate::grpc_clients::proto::common::v1 as common_v1;
    use crate::grpc_clients::proto::common::v1::PageRequest;
    let req = Request::new(GetPlayerCollectionRequest {
        request_id: format!("rgs-flash-mock-{}", chrono::Utc::now().timestamp_millis()),
        player: Some(common_v1::PlayerId {
            player_id: Some(common_v1::EntityId { id: player_id }),
            display_name: String::new(),
            rank_score: 0,
            level: 0,
        }),
        page: Some(PageRequest {
            page: 1,
            page_size: 10,
            cursor: String::new(),
        }),
        rarity_filter: 0, // UNSPECIFIED
        series_id_filter: String::new(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.get_player_collection(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "total_count": inner.total_count,
                "instances_count": inner.instances.len(),
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// leaderboard HealthCheck (v0.3 NEW, per RGS-DTL-038 §3 DEC-038-02)
async fn call_leaderboard_health(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("leaderboard", "HealthCheck");
    let Some(mut client) = clients.leaderboard.clone() else {
        result.error = Some("leaderboard gRPC client not connected".to_string());
        return result;
    };
    let req = Request::new(common::HealthCheckRequest {
        service: "rgs-flash-mock".to_string(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.health_check(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "status": inner.status,
                "message": inner.message,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}

/// leaderboard GetRankedLeaderboard (v0.3 NEW, per RGS-DTL-038 §3 DEC-038-02)
async fn call_leaderboard_get_ranked(clients: &GrpcClients) -> GrpcProbeResult {
    let mut result = GrpcProbeResult::new("leaderboard", "GetRankedLeaderboard");
    let Some(mut client) = clients.leaderboard.clone() else {
        result.error = Some("leaderboard gRPC client not connected".to_string());
        return result;
    };
    use crate::grpc_clients::leaderboard::{
        GetRankedLeaderboardRequest, LeaderboardPeriod,
    };
    use crate::grpc_clients::proto::common::v1::PageRequest;
    let req = Request::new(GetRankedLeaderboardRequest {
        period: LeaderboardPeriod::Weekly as i32,
        page: Some(PageRequest {
            page: 1,
            page_size: 10,
            cursor: String::new(),
        }),
        season_id: String::new(),
    });
    let start = Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.get_ranked_leaderboard(req),
    )
    .await
    {
        Ok(Ok(resp)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.success = true;
            let inner = resp.into_inner();
            result.response_summary = Some(serde_json::json!({
                "entries_count": inner.entries.len(),
                "period": inner.period,
                "season_id": inner.season_id,
            }));
        }
        Ok(Err(e)) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some(format!("gRPC error: {}", e));
        }
        Err(_) => {
            result.latency_ms = start.elapsed().as_millis() as u64;
            result.error = Some("timeout (3s)".to_string());
        }
    }
    result
}
