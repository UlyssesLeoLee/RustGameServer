// rgs-flash-mock v0.1 — 12 类别 22 RPC stub handlers
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §3
//
// v0.1 PoC: HTTP 路由 → 调 RGS gRPC backend (skeleton, v0.2 接 mTLS)
// 当前 v0.1: 直接返回 stub response + 调 gap_matrix.record_call/record_response

use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use crate::gap_matrix::{RpcCategory, RpcStatus};
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
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(101, RpcCategory::Scene, RpcStatus::NotApplicable, "场景查询");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(101, RpcStatus::NotApplicable, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 101,
            "name": "GetScene",
            "status": "n-a",
            "reason": "RGS TCG 无场景/移动概念, 标记 N-A (per design §3 category 1)",
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
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(102, RpcCategory::Scene, RpcStatus::NotApplicable, "玩家移动");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(102, RpcStatus::NotApplicable, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 102,
            "name": "MovePlayer",
            "status": "n-a",
            "reason": "RGS TCG 无移动, 标记 N-A",
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
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(201, RpcCategory::Role, RpcStatus::Partial, "玩家档案查询");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(201, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 201,
            "name": "GetPlayerProfile",
            "status": "partial",
            "reason": "RGS v2 部分实装, 部分字段缺失 (per design §3 category 2)",
            "rgs_call": "player-service:50051 GetPlayerProfile",
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
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(202, RpcCategory::Role, RpcStatus::Partial, "技能升级");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(202, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 202,
            "name": "UpgradeSkill",
            "status": "partial",
            "reason": "卡组养成类比, 不完全对应 (RGS card-service:50061 CardInstance.level)",
            "mock_response": { "skill_id": 0, "new_level": 1, "cost_gold": 100 },
        }))
    }
}

// === 3. 战斗 PVE (241 total) — Pass ===
pub mod combat {
    use super::*;

    #[post("/combat/start")]
    pub async fn start_combat(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(301, RpcCategory::Combat, RpcStatus::Pass, "开始战斗");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(301, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 301,
            "name": "StartCombat",
            "status": "pass",
            "rgs_call": "match-service:50053 CreateMatch",
            "mock_response": { "match_id": "stub-match-001", "state": "active", "turn": 1 },
        }))
    }

    #[post("/combat/action")]
    pub async fn submit_action(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(302, RpcCategory::Combat, RpcStatus::Pass, "提交动作");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(302, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 302,
            "name": "SubmitAction",
            "status": "pass",
            "rgs_call": "match-service:50053 SubmitMove",
            "mock_response": { "accepted": true, "next_turn": 2 },
        }))
    }
}

// === 4. PVP/竞技 (151 total) — Pass ===
pub mod pvp {
    use super::*;

    #[post("/pvp/enqueue")]
    pub async fn enqueue_pvp(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(401, RpcCategory::Pvp, RpcStatus::Pass, "PVP 排队");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(401, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 401,
            "name": "EnqueuePVP",
            "status": "pass",
            "rgs_call": "match-service:50053 EnqueueMatchmaking",
            "mock_response": { "queue_position": 1, "estimated_wait_sec": 5 },
        }))
    }

    #[post("/pvp/get")]
    pub async fn get_pvp_match(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(402, RpcCategory::Pvp, RpcStatus::Pass, "PVP 比赛查询");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(402, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 402,
            "name": "GetPVPMatch",
            "status": "pass",
            "rgs_call": "match-service:50053 GetMatchState",
            "mock_response": { "match_id": "stub-pvp-001", "opponent": "stub-opponent", "score": [0, 0] },
        }))
    }
}

// === 5. 公会 (97 total) — Partial ===
pub mod guild {
    use super::*;

    #[post("/guild/get")]
    pub async fn get_guild(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(501, RpcCategory::Guild, RpcStatus::Partial, "公会查询");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(501, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 501,
            "name": "GetGuild",
            "status": "partial",
            "reason": "RGS social gRPC 4/6 handler 未 wire (per FLASH-OVERLAP §3.4)",
            "rgs_call": "social-service:50054 HealthCheck (get_guild stub)",
            "mock_response": { "guild_id": null, "members": 0, "missing": ["leave", "dissolve", "kick"] },
        }))
    }

    #[post("/guild/join")]
    pub async fn join_guild(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(502, RpcCategory::Guild, RpcStatus::Partial, "加入公会");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(502, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 502,
            "name": "JoinGuild",
            "status": "partial",
            "reason": "social gRPC handler 未 wire",
            "mock_response": { "joined": false, "reason": "stub-not-implemented" },
        }))
    }
}

// === 6. 经济 (90 total) — Pass ===
pub mod econ {
    use super::*;

    #[post("/econ/account")]
    pub async fn get_account(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(601, RpcCategory::Econ, RpcStatus::Pass, "账户查询");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(601, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 601,
            "name": "GetAccount",
            "status": "pass",
            "rgs_call": "economy-service:50052 GetAccount",
            "mock_response": { "gold": 1000, "diamond": 50, "energy": 100 },
        }))
    }

    #[post("/econ/auction")]
    pub async fn create_auction(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(602, RpcCategory::Econ, RpcStatus::Pass, "创建拍卖");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(602, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 602,
            "name": "CreateAuction",
            "status": "pass",
            "rgs_call": "economy-service:50052 CreateAuction",
            "mock_response": { "auction_id": "stub-auction-001", "price": 100 },
        }))
    }
}

// === 7. 社交 (123 total) — Partial ===
pub mod friend {
    use super::*;

    #[post("/friend/list")]
    pub async fn list(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(701, RpcCategory::Social, RpcStatus::Partial, "好友列表");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(701, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 701,
            "name": "GetFriendList",
            "status": "partial",
            "reason": "RGS social 缺好友/邮件 (per design §3 category 7)",
            "mock_response": { "friends": [], "missing": ["block", "search", "recommend"] },
        }))
    }

    #[post("/friend/send")]
    pub async fn send(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(702, RpcCategory::Social, RpcStatus::Partial, "发送消息");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(702, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 702,
            "name": "SendMessage",
            "status": "partial",
            "mock_response": { "sent": false, "reason": "stub-not-implemented" },
        }))
    }
}

// === 8. 活动运营 (184 total) — Partial ===
pub mod event {
    use super::*;

    #[post("/event/active")]
    pub async fn active(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(801, RpcCategory::Event, RpcStatus::Partial, "活动查询");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(801, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 801,
            "name": "GetActiveEvent",
            "status": "partial",
            "reason": "RGS 缺数据驱动活动框架 (per handoff v0.1 §2.1.3 反例)",
            "rgs_call": "batch (task_templates) + card (AddCardToCollection.source=Event)",
            "mock_response": { "events": [], "missing": ["data_driven_template"] },
        }))
    }

    #[post("/event/claim")]
    pub async fn claim(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(802, RpcCategory::Event, RpcStatus::Partial, "领取奖励");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(802, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 802,
            "name": "ClaimReward",
            "status": "partial",
            "mock_response": { "claimed": false, "reason": "stub-not-implemented" },
        }))
    }
}

// === 9. 付费 (43 total) — Partial ===
pub mod pay {
    use super::*;

    #[post("/pay/recharge")]
    pub async fn recharge(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(901, RpcCategory::Pay, RpcStatus::Partial, "充值");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(901, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 901,
            "name": "Recharge",
            "status": "partial",
            "reason": "RGS 抽卡/开包不同 (per design §3 category 9)",
            "rgs_call": "economy + payment (mock)",
            "mock_response": { "order_id": "stub-pay-001", "amount_cny": 0, "diamond_added": 0 },
        }))
    }

    #[post("/pay/history")]
    pub async fn history(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(902, RpcCategory::Pay, RpcStatus::Partial, "充值历史");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(902, RpcStatus::Partial, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 902,
            "name": "QueryRechargeHistory",
            "status": "partial",
            "rgs_call": "economy",
            "mock_response": { "history": [] },
        }))
    }
}

// === 10. 排行榜 (10 total) — Pass ===
pub mod rank {
    use super::*;

    #[post("/rank/leaderboard")]
    pub async fn leaderboard(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1001, RpcCategory::Rank, RpcStatus::Pass, "排行榜");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1001, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1001,
            "name": "GetLeaderboard",
            "status": "pass",
            "rgs_call": "leaderboard (现有, 跨域共享)",
            "mock_response": {
                "rank_type": "level",
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
    pub async fn ban(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1101, RpcCategory::Gm, RpcStatus::Pass, "封号");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1101, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1101,
            "name": "BanAccount",
            "status": "pass",
            "rgs_call": "admin-service:50055 BanAccount + gm-backend:8081 (同 RPC)",
            "mock_response": { "banned": true, "duration_hours": 24 },
        }))
    }

    #[post("/gm/grant")]
    pub async fn grant(
        data: web::Data<AppState>,
    ) -> HttpResponse {
        let mut matrix = data.matrix.lock().await;
        matrix.record_call(1102, RpcCategory::Gm, RpcStatus::Pass, "补偿发放");
        let start = std::time::Instant::now();
        let latency = start.elapsed().as_millis() as u64;
        matrix.record_response(1102, RpcStatus::Pass, latency);
        drop(matrix);

        HttpResponse::Ok().json(serde_json::json!({
            "rpc": 1102,
            "name": "GrantCompensation",
            "status": "pass",
            "rgs_call": "admin + gm-backend",
            "mock_response": { "granted": true, "items": [{ "id": 1, "count": 100 }] },
        }))
    }
}
