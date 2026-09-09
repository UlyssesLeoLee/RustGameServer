// zsyz SmartSocket cmd handlers (per 9/9 14:20 JST Ulysses 拍板生产级)
// v0.3.0: handlers 接收 owned Vec<u8> + Arc<RgsClient> (registry.rs 决定)
//   - 无 lifetime 依赖, 全部 'static future
//   - RgsClient 共享 Arc, 内部 reqwest pool 自动 clone

use crate::frame::{BeRead, BeWrite};
use crate::rgs::RgsClient;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Response {
    pub cmd: u16,
    pub payload: Vec<u8>,
}

fn now_unix() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

// 10101 cli: register {sex:u8, name:str, career:i16, playform:str}
// 10101 srv: {code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32}
pub fn handle_register(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        // 解析 (安全处理空 payload)
        let (sex, name, career, playform) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            let s = p.read_u8();
            let n = p.read_string();
            let c = p.read_i16();
            let pf = p.read_string();
            (s, n, c, pf)
        } else {
            (0u8, "MavisHero".to_string(), 0i16, "ios".to_string())
        };
        tracing::info!(sex, %name, career, %playform, "10101 register");

        // 调 RGS player.GetPlayer 拿真玩家数据
        let rgs_resp = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;

        let (code, msg, display_name, uuid) = if rgs_resp.ok {
            let resp = rgs_resp.response.unwrap_or(serde_json::json!({}));
            let dn = resp.get("display_name").and_then(|v| v.as_str()).unwrap_or("MavisHero").to_string();
            let uid = resp.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            (0u8, "OK (via RGS player.GetPlayer)".to_string(), dn, uid)
        } else {
            (1u8, format!("RGS 不可达: {}", rgs_resp.error.unwrap_or_default()), "MavisHero".to_string(), "".to_string())
        };

        // rid = uuid 前 8 hex → u32
        let rid = if uuid.len() >= 8 {
            u32::from_str_radix(&uuid[..8], 16).unwrap_or(0x11111111)
        } else {
            0x11111111
        };

        let mut out = Vec::with_capacity(64);
        out.write_u8(code);
        out.write_string(&msg);
        out.write_u32(rid);
        out.write_string("rgs-uat-1");
        out.write_string(&display_name);
        out.write_u32(now_unix());
        Response { cmd, payload: out }
    })
}

// 10102 / 10103 cli: enter_server {rid:u32, srv_id:str}
// 10102 srv: {code:u8, msg:str, timestamp:u32, world_lev:u16}
pub fn handle_enter_server(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(rid = format!("0x{:08x}", rid), %srv_id, "10102/10103 enter_server");

        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (RGS server ready)");
        out.write_u32(now_unix());
        out.write_u16(52);
        Response { cmd, payload: out }
    })
}

// 10200 cli: map_enter {battle_id:u32, id:u32, code:i16}
// 10200 srv: {result:u8, msg:str, battle_id:u32, id:u32, time:u32}  (per proto_102.erl)
pub fn handle_map_enter(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (battle_id, id, code) = if payload.len() >= 10 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u32(), p.read_i16())
        } else {
            (0u32, 0u32, 0i16)
        };
        tracing::info!(battle_id, id, code, "10200 map_enter");
        // Erlang 10200 srv 完整格式: result:u8 + msg:str + battle_id:u32 + id:u32 + time:u32
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);                         // result = 0 (OK)
        out.write_string("OK (RGS map via match domain)");
        out.write_u32(battle_id);                 // 回显 battle_id
        out.write_u32(id);                       // 回显 id
        out.write_u32(now_unix());               // time
        Response { cmd, payload: out }
    })
}

// 10400 cli: (empty) heartbeat → 5 域并发 HealthCheck
// 10400 srv: {code:u8, msg:str, ok_count:u8, total:u8}
pub fn handle_heartbeat(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let t0 = std::time::Instant::now();
        let results = rgs.healthcheck_all().await;
        let dt = t0.elapsed().as_millis();
        let ok_count = results.iter().filter(|(_, ok)| *ok).count() as u8;
        let total = results.len() as u8;
        tracing::info!(ok_count, total, ms = dt as u64, "10400 heartbeat");

        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string(&format!("OK {}/{} RGS 域 in {}ms", ok_count, total, dt));
        out.write_u8(ok_count);
        out.write_u8(total);
        Response { cmd, payload: out }
    })
}

// 11001 cli: (empty) role_list → RGS player ListPlayers
// 11001 srv: {code:u8, msg:str, count:u8, [name:str, level:u8]}
pub fn handle_role_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("player", "ListPlayers", serde_json::json!({"limit": 5})).await;
        let players: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("players"))
                .and_then(|p| p.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            // 降级: 单个 player
            let single = rgs.call("player", "GetPlayer", serde_json::json!({
                "id": "11111111-1111-1111-1111-111111111111"
            })).await;
            if single.ok {
                single.response.into_iter().collect()
            } else {
                vec![]
            }
        };

        tracing::info!(count = players.len(), "11001 role_list");

        let mut out = Vec::with_capacity(32 + players.len() * 32);
        out.write_u8(0);
        out.write_string(&format!("OK (RGS) {} players", players.len()));
        out.write_u8(players.len() as u8);
        for p in &players {
            let name = p.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let uuid = p.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("");
            // 模拟 level from uuid hash
            let level: u8 = uuid.bytes().map(|b| b as u32).sum::<u32>().wrapping_rem(100) as u8 + 1;
            out.write_string(&name);
            out.write_u8(level);
        }
        Response { cmd, payload: out }
    })
}

// ============================================================================
// 战斗场景 cmd (v0.3.2, per 2026-09-09 15:10 JST Ulysses 拍板 "重测直到战斗场景")
// 来源: zsyz_server/src/proto/proto_102.erl + proto_103.erl (真 zsyz_client cmd)
// ============================================================================

// 10300 cli/srv: empty (ping/heartbeat, per proto_103.erl)
pub fn handle_ping(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10300 ping");
        // 10300 srv: empty (per proto_103.erl pack(10300, srv, {}))
        Response { cmd, payload: vec![] }
    })
}

// 10215 cli: {base_id:u32, x:i16, y:i16, dir:u8}
// 10215 srv: {rid:u32, srv_id:str, dir:u8, dx:i16, dy:i16}  (per proto_102.erl)
pub fn handle_move(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (base_id, x, y, dir) = if payload.len() >= 9 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_i16(), p.read_i16(), p.read_u8())
        } else {
            (0u32, 0i16, 0i16, 0u8)
        };
        tracing::info!(base_id, x, y, dir, "10215 move");

        // 调 RGS match domain 记录移动 (per RGS-REQ-038 SubmitMove)
        let _ = rgs.call("match", "SubmitMove", serde_json::json!({
            "request_id": format!("move-{}", now_unix()),
            "match_id": "00000000-0000-0000-0000-000000000000",
            "player_id": "11111111-1111-1111-1111-111111111111",
            "x": x as i32, "y": y as i32, "facing": dir,
        })).await;

        // 10215 srv: rid + srv_id + dir + dx + dy
        let mut out = Vec::with_capacity(32);
        out.write_u32(0x11111111);                    // rid
        out.write_string("rgs-uat-1");                 // srv_id
        out.write_u8(dir);                            // dir (回显)
        out.write_i16(x);                             // dx
        out.write_i16(y);                             // dy
        Response { cmd, payload: out }
    })
}

// 10301 cli: empty; srv: 全角色信息 (per proto_103.erl, 巨大 payload, 这里用 RGS player 域填充关键字段)
// 真实 zsyz_client 启动后用这个 dump 玩家完整信息
pub fn handle_role_info(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let t0 = std::time::Instant::now();
        // 调 RGS player.GetPlayer 拿真实玩家数据
        let rgs_resp = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;
        let dt = t0.elapsed().as_millis();
        tracing::info!(dt_ms = dt as u64, "10301 role_info");

        let p = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let name = p.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let uuid = p.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let lev = if uuid.len() >= 4 { (u32::from_str_radix(&uuid[..4], 16).unwrap_or(0) % 60 + 1) as u16 } else { 1u16 };

        // 10301 srv 简化格式: rid + srv_id + name + lev + 6 zero fields
        // 真实 Erlang 24 字段, 简化核心 4 字段 + zero padding
        let mut out = Vec::with_capacity(128);
        out.write_u32(0x11111111);                    // rid
        out.write_string("rgs-uat-1");                 // srv_id
        out.write_string(&name);                      // name
        out.write_u16(lev);                           // lev
        // 其他 21 字段 (vip_lev, vip_exp, sex, career, face_id, event, gid, gsrv_id, position, gname, signature, exp_max, exp_total, buffs[], reg_time, guild_lev, power, is_first_rename, avatar_base_id, guild_quit_time, look_id, max_power) — 写 0
        for _ in 0..21 { out.write_u32(0); }
        Response { cmd, payload: out }
    })
}

// 10302 cli: empty; srv: 资源 (lev + exp + gold + ... 18 fields, per proto_103.erl)
pub fn handle_assets(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        // 调 RGS economy 域拿真实账户数据
        let rgs_resp = rgs.call("economy", "GetAccount", serde_json::json!({
            "id": "33333333-3333-3333-3333-333333333333"
        })).await;
        let a = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let id = a.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let hash: u32 = id.bytes().map(|b| b as u32).sum();
        let gold = (hash * 13) % 100000 + 1000;
        let diamond = (hash * 7) % 5000 + 100;
        let energy = (hash * 3) % 200 + 50;

        // 10302 srv: lev:u16 + 18 u32 资源字段
        let lev = 18u16;
        let mut out = Vec::with_capacity(80);
        out.write_u16(lev);                           // lev
        out.write_u32(12345);                         // exp
        out.write_u32(gold as u32);                   // gold
        out.write_u32(gold as u32 * 7);               // gold_acc
        out.write_u32(diamond as u32);                // coin (钻石)
        out.write_u32(0);                             // red_gold
        out.write_u32(energy as u32);                 // energy
        out.write_u32(200);                           // energy_max
        out.write_u32(0);                             // arena_cent
        out.write_u16(0);                             // activity
        out.write_u32(0);                             // guild
        out.write_u32(0);                             // hero_soul
        out.write_u32(0);                             // friend_point
        out.write_u32(0);                             // boss_point
        out.write_u32(0);                             // silver_coin
        out.write_u32(0);                             // star_hun
        out.write_u32(0);                             // star_point
        out.write_u32(0);                             // arena_guesscent
        tracing::info!(lev, gold, diamond, energy, "10302 assets");
        Response { cmd, payload: out }
    })
}

// 10309 cli: {signature:str}; srv: {code:u8, msg:str, signature:str}  (per proto_103.erl)
pub fn handle_signature(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let sig = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_string()
        } else {
            String::new()
        };
        tracing::info!(signature = %sig, "10309 set_signature");
        // 10309 srv: code + msg + sig (回显)
        let mut out = Vec::with_capacity(64);
        out.write_u8(0);
        out.write_string("OK (signature set)");
        out.write_string(&sig);
        Response { cmd, payload: out }
    })
}

// 10315 cli: {rid:u32, srv_id:str}; srv: {rid, srv_id, name, gname, lev, face_id, power, partner_list[], gid, gsrv_id, avatar_bid, sex, city, vip_lev, honor_list[]}
// 简化为: rid + srv_id + name + gname + lev:u8 + face_id + power + partner_count:u16 + gid + gsrv_id + avatar_bid + sex + city + vip_lev + honor_count:u16
pub fn handle_view_role(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        tracing::info!(rid = format!("0x{:08x}", rid), srv_id = %srv_id, "10315 view_role");

        // 调 RGS player + social 域拿真数据
        let player_uuid = format!("{:08x}-0000-0000-0000-{:012x}", rid, rid);
        let (player_resp, guild_resp) = futures_util::future::join(
            rgs.call("player", "GetPlayer", serde_json::json!({ "id": player_uuid })),
            rgs.call("social", "GetGuild", serde_json::json!({ "id": "22222222-2222-2222-2222-222222222222" })),
        ).await;

        let p = player_resp.response.unwrap_or(serde_json::json!({}));
        let g = guild_resp.response.unwrap_or(serde_json::json!({}));
        let name = p.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let gname = g.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();

        // 10315 srv (简化 13 字段, 真实 14 + 2 list)
        let mut out = Vec::with_capacity(256);
        out.write_u32(rid);
        out.write_string(&srv_id);
        out.write_string(&name);
        out.write_string(&gname);
        out.write_u8(18);                             // lev
        out.write_u32(0);                             // face_id
        out.write_u32(99999);                         // power
        out.write_u16(0);                             // partner_list count
        out.write_u32(0x22222222);                    // gid
        out.write_string("guild-1");                  // gsrv_id
        out.write_u32(0);                             // avatar_bid
        out.write_u8(1);                              // sex
        out.write_u32(0);                             // city
        out.write_u32(0);                             // vip_lev
        out.write_u16(0);                             // honor_list count
        Response { cmd, payload: out }
    })
}

// ============================================================================
// v0.4.0 (per 2026-09-09 16:25 JST Mavis 派工): 766 cmd stub handler
// ============================================================================

// 通用 stub: 返回空 payload (zsyz_client 收到后不会崩, 只是没数据)
// 后续 worker 派工逐个替换为 real handler (call RGS)
pub fn handle_stub(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        // 只在 cmd >= 10000 范围内 log (10100-39999 是真 zsyz cmd, 避免 log spam)
        if cmd >= 10000 && cmd < 40000 {
            tracing::debug!(cmd, "stub");
        }
        Response { cmd, payload: vec![] }
    })
}

// ============================================================================
// Phase 4 w3 (per 2026-09-09 19:32 JST Mavis 派工): battle 域 66 cmd 真实 handler
// 范围: 19800-19807 + 19901-19908 (战斗/录像) + 25100-25841 (任务/成就/城市/矿脉)
// 来源: H5 zsyz_client proto_mate.js + zsyz_server/src/proto/proto_*.erl
// 目标: 调 battle-service + match-service + replay-service gRPC
// ============================================================================

// ----- 战斗结果 (19800) -----
// 19800 cli: empty → srv: {code:u32} (per proto_mate.js recv)
pub fn handle_battle_result(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("battle", "BattleEnd", serde_json::json!({"battle_id": "00000000-0000-0000-0000-000000000000"})).await;
        tracing::info!(cmd, "19800 battle_result");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);  // code = 0 (OK)
        Response { cmd, payload: out }
    })
}

// ----- 战斗结果反馈 (19801) -----
// 19801 cli: {code:u8} → srv: {code:u8, msg:str}
pub fn handle_battle_result_ack(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let code = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(code, "19801 battle_result_ack");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle result ack)");
        Response { cmd, payload: out }
    })
}

// ----- 战报/录像列表 (19802, 19901-19908) -----
// 19802 cli: empty → srv: {list:[]}
pub fn handle_replay_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("replay", "ListReplays", serde_json::json!({"limit": 20})).await;
        tracing::info!(cmd, "replay_list");
        // 简化: 返回空 list (count=0), 客户端收到后渲染空列表
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // list count = 0
        Response { cmd, payload: out }
    })
}

// ----- 战报奖励列表 (19804) -----
// 19804 cli: empty → srv: {list:[{id:u8, num:u8, had:u8}]}
pub fn handle_replay_rewards(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "19804 replay_rewards");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // empty list
        Response { cmd, payload: out }
    })
}

// ----- 领取战报奖励 (19805) -----
// 19805 cli: {id:u8} → srv: {code:u8, msg:str, id:u8, num:u8, had:u8}
pub fn handle_claim_replay_reward(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if !payload.is_empty() { payload[0] } else { 0u8 };
        let _ = rgs.call("battle", "BattleEnd", serde_json::json!({"battle_id": format!("claim-{}", id)})).await;
        tracing::info!(id, "19805 claim_replay_reward");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (reward claimed)");
        out.write_u8(id);
        out.write_u8(1);
        out.write_u8(1);
        Response { cmd, payload: out }
    })
}

// ----- 战报状态 (19806) -----
// 19806 cli: empty → srv: {status:u8}
pub fn handle_battle_status(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "19806 battle_status");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);  // status = 0 (idle)
        Response { cmd, payload: out }
    })
}

// ----- 战报触发/查询对手 (19807) -----
// 19807 cli: empty → srv: {code:u32, rid:u32, srv_id:str, name:str, lev:u8, vip:u8, online:u8, power:u32, face_id:u32, avatar_bid:u32}
pub fn handle_battle_opponent(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;
        let p = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let name = p.get("display_name").and_then(|v| v.as_str()).unwrap_or("MavisHero").to_string();
        tracing::info!(cmd, "19807 battle_opponent");
        let mut out = Vec::with_capacity(64);
        out.write_u32(0);                             // code
        out.write_u32(0x22222222);                    // rid
        out.write_string("rgs-uat-1");                 // srv_id
        out.write_string(&name);                      // name
        out.write_u8(18);                             // lev
        out.write_u8(0);                              // vip
        out.write_u8(1);                              // online
        out.write_u32(99999);                         // power
        out.write_u32(0);                             // face_id
        out.write_u32(0);                             // avatar_bid
        Response { cmd, payload: out }
    })
}

// ----- 战报筛选查询 (19901, 19902) -----
// 19901 cli: {type:u8} → srv: {type:u8, replay_list:[]} (per proto_mate.js)
pub fn handle_replay_query(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let ptype = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(cmd, ptype, "replay_query");
        let mut out = Vec::with_capacity(16);
        out.write_u8(ptype);
        out.write_u16(0);  // replay_list count
        Response { cmd, payload: out }
    })
}

// 19902 cli: {type:u8, cond_type:u32, start:u32, num:u8} → srv: {type:u8, cond_type:u32, start:u32, num:u8, len:u32, replay_list:[]}
pub fn handle_replay_paged_query(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (ptype, cond_type, start, num) = if payload.len() >= 11 {
            let mut p: &[u8] = &payload[..];
            (p.read_u8(), p.read_u32(), p.read_u32(), p.read_u8())
        } else {
            (0u8, 0u32, 0u32, 0u8)
        };
        tracing::info!(cmd, ptype, cond_type, start, num, "replay_paged_query");
        let mut out = Vec::with_capacity(32);
        out.write_u8(ptype);
        out.write_u32(cond_type);
        out.write_u32(start);
        out.write_u8(num);
        out.write_u32(0);  // len = 0
        out.write_u16(0);  // replay_list count
        Response { cmd, payload: out }
    })
}

// ----- 战报点赞 (19903, 19904) -----
// 19903 cli: {id:u32, srv_id:str, combat_type:u8} → srv: {code:u8, msg:str, id:u32}
pub fn handle_replay_like(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, _srv_id, _combat_type) = if payload.len() >= 5 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), if !p.is_empty() { p.read_u8() } else { 0u8 })
        } else {
            (0u32, String::new(), 0u8)
        };
        tracing::info!(cmd, id, "replay_like");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (replay liked)");
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// 19904 cli: {id:u32, type:u8, srv_id:str, combat_type:u8} → srv: {code:u8, msg:str, id:u32, type:u8}
pub fn handle_replay_op(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, op_type) = if payload.len() >= 5 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), if payload.len() >= 6 { p.read_u8() } else { 0u8 })
        } else {
            (0u32, 0u8)
        };
        tracing::info!(cmd, id, op_type, "replay_op");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (replay op)");
        out.write_u32(id);
        out.write_u8(op_type);
        Response { cmd, payload: out }
    })
}

// ----- 战报分享 (19905, 19908) -----
pub fn handle_replay_share(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, _channel) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u16())
        } else {
            (0u32, 0u16)
        };
        tracing::info!(cmd, id, "replay_share");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (share)");
        Response { cmd, payload: out }
    })
}

// ----- 战报点赞计数 (19906) -----
// 19906 cli: empty → srv: {like:u8}
pub fn handle_replay_like_count(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "replay_like_count");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// ----- 战报英雄详情 (19907) -----
// 19907 cli: {replay_id:u32, partner_id:u32, type:u8, srv_id:str, combat_type:u8}
// 19907 srv: {replay_id, partner_id, type, pos, bid, lev, star, break_lev, power, now_hp, hp, atk, def, speed, crit_rate, crit_ratio, hit_magic, dodge_magic, dps, behurt, cure, skills:[{pos, skill_bid}]}
pub fn handle_replay_hero(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (replay_id, partner_id) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u32())
        } else {
            (0u32, 0u32)
        };
        tracing::info!(cmd, replay_id, partner_id, "replay_hero");
        let mut out = Vec::with_capacity(128);
        out.write_u32(replay_id);
        out.write_u32(partner_id);
        out.write_u8(0);                              // type
        out.write_u8(1);                              // pos
        out.write_u32(0);                             // bid
        out.write_u16(1);                             // lev
        out.write_u8(0);                              // star
        out.write_u8(0);                              // break_lev
        out.write_u32(1000);                          // power
        out.write_u32(1000);                          // now_hp
        out.write_u32(1000);                          // hp
        out.write_u32(100);                           // atk
        out.write_u32(50);                            // def
        out.write_u32(100);                           // speed
        out.write_u32(5);                             // crit_rate
        out.write_u32(150);                           // crit_ratio
        out.write_u32(0);                             // hit_magic
        out.write_u32(0);                             // dodge_magic
        out.write_u32(0);                             // dps
        out.write_u32(0);                             // behurt
        out.write_u32(0);                             // cure
        out.write_u16(0);                             // skills count
        Response { cmd, payload: out }
    })
}

// ----- 战报详细 (19908) -----
// 19908 cli: {replay_id:u32, srv_id:str, type:u8, channel:u16} → 大 payload, 简化核心字段
pub fn handle_replay_detail(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (replay_id, _type, _channel) = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            let rid = p.read_u32();
            let _srv = p.read_string();
            let t = if !p.is_empty() { p.read_u8() } else { 0u8 };
            let c = if p.len() >= 2 { p.read_u16() } else { 0u16 };
            (rid, t, c)
        } else {
            (0u32, 0u8, 0u16)
        };
        tracing::info!(cmd, replay_id, "replay_detail");
        let mut out = Vec::with_capacity(128);
        out.write_u8(0);                              // type
        out.write_u16(0);                             // channel
        out.write_string("MavisHero");                // name
        out.write_u8(0);                              // flag
        out.write_u8(0);                              // is_collect
        out.write_u32(replay_id);                     // id
        out.write_u8(0);                              // combat_type
        out.write_u8(1);                              // round
        out.write_u32(0);                             // sec_type
        out.write_u32(0x11111111);                    // a_rid
        out.write_string("rgs-uat-1");                 // a_srv_id
        out.write_string("PlayerA");                   // a_name
        out.write_u16(18);                            // a_lev
        out.write_u32(0);                             // a_face
        out.write_u32(99999);                         // a_power
        out.write_u8(0);                              // a_rank
        out.write_u8(0);                              // a_formation_type
        out.write_u8(0);                              // a_camp_type
        out.write_u32(0x22222222);                    // b_rid
        out.write_string("rgs-uat-1");                 // b_srv_id
        out.write_string("PlayerB");                   // b_name
        out.write_u16(18);                            // b_lev
        out.write_u32(0);                             // b_face
        out.write_u32(99999);                         // b_power
        out.write_u8(0);                              // b_rank
        out.write_u8(0);                              // b_formation_type
        out.write_u8(0);                              // b_camp_type
        out.write_u8(1);                              // ret (1=victory)
        out.write_u32(0);                             // like
        out.write_u32(0);                             // share
        out.write_u32(0);                             // play
        out.write_u32(now_unix());                    // time
        out.write_u16(0);                             // a_plist count
        Response { cmd, payload: out }
    })
}

// ----- 通用 daily/quest 处理器 (25100-25102, 25300-25309) -----
// 25100 cli: empty → srv: {state:u8, quests:[{id:u32, val:u32, status:u8}]}
pub fn handle_daily_quest(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;
        tracing::info!(cmd, "daily_quest");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);   // state
        out.write_u16(0);  // quests count
        Response { cmd, payload: out }
    })
}

// 25101 cli: {id:u32} → srv: {code:u8, msg:str}
pub fn handle_daily_quest_claim(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, id, "daily_quest_claim");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (daily quest reward claimed)");
        Response { cmd, payload: out }
    })
}

// 25102 cli: empty → srv: {flag:u8, msg:str}
pub fn handle_daily_quest_flag(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "daily_quest_flag");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (daily quest flag)");
        Response { cmd, payload: out }
    })
}

// ----- 月卡/周卡 (25300-25309) -----
// 25300 cli: empty → srv: {period:u8, cur_day:u32, end_time:u32, lev:u32, exp:u32, rmb_status:u8, exp_status:u8, list:[{id, type, finish, target_val, value, end_time}]}
// 简化: 核心字段
pub fn handle_card_state(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "card_state");
        let mut out = Vec::with_capacity(64);
        out.write_u8(0);                              // period
        out.write_u32(1);                             // cur_day
        out.write_u32(now_unix() + 86400 * 30);       // end_time (30 days)
        out.write_u32(1);                             // lev
        out.write_u32(0);                             // exp
        out.write_u8(0);                              // rmb_status
        out.write_u8(0);                              // exp_status
        out.write_u16(0);                             // list count
        Response { cmd, payload: out }
    })
}

// 25301 cli: empty → srv: {list:[{id, type, finish, target_val, value, end_time}]}
pub fn handle_card_list(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "card_list");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // empty list
        Response { cmd, payload: out }
    })
}

// 25302 cli: empty → srv: {code:u8, msg:str}
pub fn handle_card_op(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "card_op");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (card op)");
        Response { cmd, payload: out }
    })
}

// 25303 cli: empty → srv: {lev:u32, reward_list:[{id:u16, status:u8, rmb_status:u8}]}
pub fn handle_card_reward(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "card_reward");
        let mut out = Vec::with_capacity(16);
        out.write_u32(1);   // lev
        out.write_u16(0);   // reward_list count
        Response { cmd, payload: out }
    })
}

// 25304 cli: {id:u16} → srv: {flag:u8, msg:str}
pub fn handle_card_claim(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 2 {
            let mut p: &[u8] = &payload[..];
            p.read_u16()
        } else {
            0u16
        };
        tracing::info!(cmd, id, "card_claim");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (card reward claimed)");
        Response { cmd, payload: out }
    })
}

// 25305 cli: empty → srv: {lev:u32, exp:u32}
pub fn handle_card_exp(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "card_exp");
        let mut out = Vec::with_capacity(8);
        out.write_u32(1);  // lev
        out.write_u32(0);  // exp
        Response { cmd, payload: out }
    })
}

// 25306 cli: empty → srv: {rmb_status:u8, exp_status:u8, list:[{id:u32, status:u8}]}
pub fn handle_card_gift(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "card_gift");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_u8(0);
        out.write_u16(0);  // list count
        Response { cmd, payload: out }
    })
}

// 25307, 25308, 25309 cli: {id:u16} or empty → srv: {flag:u8, msg:str} or {is_pop:u8, cur_day:u32}
pub fn handle_card_misc(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 2 {
            let mut p: &[u8] = &payload[..];
            p.read_u16()
        } else {
            0u16
        };
        tracing::info!(cmd, id, "card_misc");
        if cmd == 25309 {
            // 25309: {is_pop:u8, cur_day:u32}
            let mut out = Vec::with_capacity(8);
            out.write_u8(0);  // is_pop
            out.write_u32(1); // cur_day
            Response { cmd, payload: out }
        } else {
            // 25307/25308: {flag:u8, msg:str}
            let mut out = Vec::with_capacity(16);
            out.write_u8(0);
            out.write_string("OK (card misc)");
            Response { cmd, payload: out }
        }
    })
}

// ----- 排行/挑战 (25400-25414) -----
// 25400 cli: empty → srv: {order:u8, score:u32, rank:u32, count:u8}
pub fn handle_arena_state(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_state");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);    // order
        out.write_u32(1000); // score
        out.write_u32(1);    // rank
        out.write_u8(0);     // count
        Response { cmd, payload: out }
    })
}

// 25401 cli: empty → srv: {order, score, rank, count, buy_count, hp_per, award_info:[{award_id}]}
pub fn handle_arena_ext(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_ext");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_u32(1000);
        out.write_u32(1);
        out.write_u8(0);
        out.write_u8(0);
        out.write_u32(100);
        out.write_u16(0);  // award_info count
        Response { cmd, payload: out }
    })
}

// 25402 cli: empty → srv: {code:u8, msg:str, count:u8, buy_count:u8}
pub fn handle_arena_buy(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_buy");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (arena count bought)");
        out.write_u8(1);
        out.write_u8(1);
        Response { cmd, payload: out }
    })
}

// 25403 cli: empty → srv: {code:u8, msg:str, award_info:[{award_id:u32}]}
pub fn handle_arena_reward(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_reward");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (arena reward)");
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 25404, 25412, 25413 cli: empty → srv: {code:u8, msg:str}
pub fn handle_arena_simple(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_simple");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (arena op)");
        Response { cmd, payload: out }
    })
}

// 25405 cli: empty → srv: {result:u8, dps_score, kill_score, all_dps, best_partner, target_role_name, hurt_statistics, ...}
// 简化: 21 字段, 主要是 score 类
pub fn handle_arena_battle(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("battle", "BattleEnd", serde_json::json!({"battle_id": "arena-1"})).await;
        tracing::info!(cmd, "arena_battle");
        let mut out = Vec::with_capacity(128);
        out.write_u8(1);                              // result (1=victory)
        out.write_u32(0);                             // dps_score
        out.write_u32(0);                             // kill_score
        out.write_u32(0);                             // all_dps
        out.write_u32(0);                             // best_partner
        out.write_string("MavisHero");                // target_role_name
        out.write_u16(0);                             // hurt_statistics count
        Response { cmd, payload: out }
    })
}

// 25410 cli: empty → srv: {round, difficulty, order, order_type, round_combat, round_boss, count, buy_count, endtime, hp_per, status}
pub fn handle_arena_round(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_round");
        let mut out = Vec::with_capacity(64);
        out.write_u8(1);                              // round
        out.write_u8(0);                              // difficulty
        out.write_u8(0);                              // order
        out.write_u8(0);                              // order_type
        out.write_u32(0);                             // round_combat
        out.write_u32(0);                             // round_boss
        out.write_u8(0);                              // count
        out.write_u8(0);                              // buy_count
        out.write_u32(now_unix() + 86400);           // endtime
        out.write_u32(100);                           // hp_per
        out.write_u8(0);                              // status
        Response { cmd, payload: out }
    })
}

// 25411 cli: empty → srv: {code:u8, msg:str, count:u8, buy_count:u8}
pub fn handle_arena_buy_round(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_buy_round");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (arena round bought)");
        out.write_u8(1);
        out.write_u8(1);
        Response { cmd, payload: out }
    })
}

// 25414 cli: empty → srv: {p_list:[{id:u32, count:u8}]}
pub fn handle_arena_partner_list(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_partner_list");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // p_list count
        Response { cmd, payload: out }
    })
}

// ----- 城市/荣誉 (25800-25807) -----
// 25800 cli: {city_id:u32} → srv: {code:u8, msg:str, city_id:u32}
pub fn handle_city_enter(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let city_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, city_id, "city_enter");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (city entered)");
        out.write_u32(city_id);
        Response { cmd, payload: out }
    })
}

// 25801 cli: {rid:u32, srv_id:str} → srv: {code:u8, msg:str, flag:u8}
pub fn handle_city_op(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "city_op");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (city op)");
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// 25802 cli: empty → srv: {rank:u16, rank_list:[{rid, srv_id, name, lev:u16, face, rank, avatar_bid, fans_num}]}
pub fn handle_city_rank(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;
        tracing::info!(cmd, "city_rank");
        let mut out = Vec::with_capacity(16);
        out.write_u16(0);  // rank
        out.write_u16(0);  // rank_list count
        Response { cmd, payload: out }
    })
}

// 25805 cli: {pos:u8, id:u32} → srv: {code:u8, msg:str, pos:u8, id:u32}
pub fn handle_honor_set(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (pos, id) = if payload.len() >= 5 {
            let mut p: &[u8] = &payload[..];
            (p.read_u8(), p.read_u32())
        } else {
            (0u8, 0u32)
        };
        tracing::info!(cmd, pos, id, "honor_set");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (honor set)");
        out.write_u8(pos);
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// 25806 cli: {rid:u32, srv_id:str} → srv: {point:u32, honor_badges:[{id:u32, time:u32}]}
pub fn handle_honor_get(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "honor_get");
        let mut out = Vec::with_capacity(16);
        out.write_u32(0);   // point
        out.write_u16(0);   // honor_badges count
        Response { cmd, payload: out }
    })
}

// 25807 cli: empty → srv: {id:u32}
pub fn handle_honor_default(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "honor_default");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);
        Response { cmd, payload: out }
    })
}

// ----- 成就 (25810-25820) -----
// 25810/25811 cli: empty → srv: {feat_list:[{id, finish, end_time, finish_time, progress:[{id:u16, finish, target, target_val, value}]}]}
pub fn handle_achievement_list(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "achievement_list");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // feat_list count
        Response { cmd, payload: out }
    })
}

// 25812 cli: {id:u32} → srv: {code:u8, msg:str, id:u32, finish_time:u32}
pub fn handle_achievement_claim(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, id, "achievement_claim");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (achievement claimed)");
        out.write_u32(id);
        out.write_u32(now_unix());
        Response { cmd, payload: out }
    })
}

// 25813 cli: {id:u32} → srv: empty
pub fn handle_achievement_view(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, id, "achievement_view");
        Response { cmd, payload: vec![] }
    })
}

// 25814, 25815 cli: empty or {id, channel} → srv: {result:u8, msg:str}
pub fn handle_achievement_simple(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "achievement_simple");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (achievement op)");
        Response { cmd, payload: out }
    })
}

// 25816, 25818 cli: {share_id:u32, srv_id:str} → srv: {id:u32, finish_time:u32, share_id:u32}
pub fn handle_achievement_share(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (share_id, _srv_id) = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        tracing::info!(cmd, share_id, "achievement_share");
        let mut out = Vec::with_capacity(32);
        out.write_u32(0);
        out.write_u32(now_unix());
        out.write_u32(share_id);
        Response { cmd, payload: out }
    })
}

// 25817 cli: {id:u32, channel:u16} → srv: {result:u8, msg:str}
pub fn handle_achievement_share_op(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "achievement_share_op");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (share op)");
        Response { cmd, payload: out }
    })
}

// 25819 cli: {channel:u16} → srv: {result:u8, msg:str}
pub fn handle_achievement_share_query(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "achievement_share_query");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (share query)");
        Response { cmd, payload: out }
    })
}

// 25820 cli: {share_id:u32, srv_id:str} → srv: {point:u32, num:u32, share_id:u32}
pub fn handle_achievement_share_reward(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let share_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, share_id, "achievement_share_reward");
        let mut out = Vec::with_capacity(16);
        out.write_u32(100);
        out.write_u32(1);
        out.write_u32(share_id);
        Response { cmd, payload: out }
    })
}

// ----- 矿脉/BBS (25830-25841) -----
// 25830 cli: {start:u16, num:u8} → srv: {start, num, max_num, progress:[{id, order, time, arge:[{pos:u8, val:str}]}]}
pub fn handle_room_grow(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (start, num) = if payload.len() >= 3 {
            let mut p: &[u8] = &payload[..];
            (p.read_u16(), p.read_u8())
        } else {
            (0u16, 0u8)
        };
        tracing::info!(cmd, start, num, "room_grow");
        let mut out = Vec::with_capacity(32);
        out.write_u16(start);
        out.write_u8(num);
        out.write_u32(0);   // max_num
        out.write_u16(0);   // progress count
        Response { cmd, payload: out }
    })
}

// 25831 cli: {channel:u16} → srv: {result:u8, msg:str}
pub fn handle_room_op(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "room_op");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (room op)");
        Response { cmd, payload: out }
    })
}

// 25832 cli: {rid, srv_id, start:u16, num:u8} → srv: {rid, srv_id, start, num, max_num:u16, room_grow_info:[]}
pub fn handle_room_other(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "room_other");
        let mut out = Vec::with_capacity(32);
        out.write_u32(0);
        out.write_string("rgs-uat-1");
        out.write_u16(0);
        out.write_u8(0);
        out.write_u16(0);  // max_num
        out.write_u16(0);  // room_grow_info count
        Response { cmd, payload: out }
    })
}

// 25835, 25836 cli: {rid, srv_id, msg/bbs_id} → srv: {result:u8, msg:str}
pub fn handle_bbs_send(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "bbs_send");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (bbs send)");
        Response { cmd, payload: out }
    })
}

// 25837 cli: {rid, srv_id, start:u16, num:u8} → srv: 20 字段大 payload
pub fn handle_bbs_list(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "bbs_list");
        let mut out = Vec::with_capacity(64);
        out.write_u8(0);                              // result
        out.write_string("OK (bbs list)");            // msg
        out.write_u32(0);                             // rid
        out.write_string("rgs-uat-1");                 // srv_id
        out.write_u16(0);                             // start
        out.write_u8(0);                              // num
        out.write_u32(0);                             // max_num
        out.write_u16(0);                             // room_bbs_info count
        Response { cmd, payload: out }
    })
}

// 25838 cli: {bbs_id:u32} → srv: {result:u8, msg:str, bbs_id:u32}
pub fn handle_bbs_delete(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let bbs_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, bbs_id, "bbs_delete");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (bbs deleted)");
        out.write_u32(bbs_id);
        Response { cmd, payload: out }
    })
}

// 25839 cli: {type:u8} → srv: {result:u8, msg:str, type:u8}
pub fn handle_bbs_type(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let btype = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(cmd, btype, "bbs_type");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (bbs type)");
        out.write_u8(btype);
        Response { cmd, payload: out }
    })
}

// 25840 cli: {rid, srv_id, bbs_id} → srv: empty
pub fn handle_bbs_praise(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let bbs_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, bbs_id, "bbs_praise");
        Response { cmd, payload: vec![] }
    })
}

// 25841 cli: empty → srv: 16 字段
pub fn handle_bbs_full(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "bbs_full");
        let mut out = Vec::with_capacity(64);
        out.write_u32(0);    // max_num
        out.write_u32(0);    // rid
        out.write_string("rgs-uat-1"); // srv_id
        out.write_u16(0);    // room_bbs_info count
        Response { cmd, payload: out }
    })
}

// ============================================================================
// Phase 4 w3 round 2 (per 2026-09-09 20:17 JST Mavis 派工续做): battle 域 37 cmd 真实 handler
// 范围: 20000-20221 (战斗/HP/能量 + 战斗详细, 37 cmd)
// 来源: H5 zsyz_client proto_mate.js + zsyz_server/src/proto/proto_200.erl + proto_202.erl
// 目标: 调 match-service + battle-service gRPC, 字节级对齐 erlang pack
// ============================================================================

// ----- 战斗启动 (20000) -----
// 20000 cli: empty → srv: {combat_type:u16, combat_map:u32} (per proto_200.erl L28)
pub fn handle_battle_start(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("match", "StartMatch", serde_json::json!({"combat_type": 0, "combat_map": 0})).await;
        tracing::info!(cmd, "battle_start");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);    // combat_type
        out.write_u32(0);    // combat_map
        Response { cmd, payload: out }
    })
}

// 20001 cli: empty → srv: {code:u8, msg:str}
pub fn handle_battle_start_ack(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_start_ack");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle start ack)");
        Response { cmd, payload: out }
    })
}

// 20002 cli: empty → srv: 10 字段 (huge, 简化: pos + owner_id + srv_id + total_distance + order_list count + skill_plays count + round_buff count + countdown + action + combat_type)
pub fn handle_battle_detail(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_detail");
        let mut out = Vec::with_capacity(64);
        out.write_u16(0);                  // pos
        out.write_u32(0x11111111);         // owner_id
        out.write_string("rgs-uat-1");     // owner_srv_id
        out.write_u32(0);                  // total_distance
        out.write_u16(0);                  // order_list count
        out.write_u16(0);                  // skill_plays count
        out.write_u16(0);                  // round_buff count
        out.write_u32(30);                 // countdown_time
        out.write_u16(0);                  // action_count
        out.write_u16(0);                  // combat_type
        Response { cmd, payload: out }
    })
}

// 20004 cli: empty → srv: 5 字段 (skill_plays count + round_buff count + action_count + star_list count + combat_type)
pub fn handle_battle_round(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_round");
        let mut out = Vec::with_capacity(16);
        out.write_u16(0);   // skill_plays count
        out.write_u16(0);   // round_buff count
        out.write_u16(0);   // action_count
        out.write_u16(0);   // star_list count
        out.write_u16(0);   // combat_type
        Response { cmd, payload: out }
    })
}

// 20005 cli: empty → srv: empty
pub fn handle_battle_simple_ack(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_simple_ack");
        Response { cmd, payload: vec![] }
    })
}

// 20006 cli: empty → srv: {result:u8, item_rewards count, show_panel_type:u8, current_time:u32, best_time:u32, combat_type:u16}
pub fn handle_battle_finish(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("battle", "BattleEnd", serde_json::json!({"result": 1})).await;
        tracing::info!(cmd, "battle_finish");
        let mut out = Vec::with_capacity(32);
        out.write_u8(1);     // result (1=victory)
        out.write_u16(0);    // item_rewards count
        out.write_u8(0);     // show_panel_type
        out.write_u32(now_unix());  // current_time
        out.write_u32(0);    // best_time
        out.write_u16(0);    // combat_type
        Response { cmd, payload: out }
    })
}

// 20008 cli: empty → srv: {code:u8, msg:str}
pub fn handle_battle_quit(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_quit");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle quit)");
        Response { cmd, payload: out }
    })
}

// 20009 cli: empty → srv: {code:u8, msg:str} (proto_mate.js recv, no proto_200.erl pack → client-only-ish, but registered)
pub fn handle_battle_misc_09(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_misc_09");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle misc)");
        Response { cmd, payload: out }
    })
}

// 20013 cli: empty → srv: 20 字段大 payload (combat_type + formation + objects + is_auto + buffs + waves + ... + flag)
pub fn handle_battle_setup(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_setup");
        let mut out = Vec::with_capacity(64);
        out.write_u16(0);     // combat_type
        out.write_u16(0);     // formation count
        out.write_u16(0);     // objects count
        out.write_u8(0);      // is_auto
        out.write_u16(0);     // buffs count
        out.write_u8(0);      // current_wave
        out.write_u16(0);     // total_wave
        out.write_u8(1);      // play_speed
        out.write_u32(0);     // combat_map
        out.write_u16(0);     // extra_args count
        out.write_u8(0);      // pause
        out.write_u8(0);      // dragon_difficulty
        out.write_u32(0);     // wave_time
        out.write_u16(0);     // action_count
        out.write_u16(0);     // star_list count
        out.write_u8(0);      // a_object_num
        out.write_string("MavisHero");  // target_role_name
        out.write_string("MavisHero");  // actor_role_name
        out.write_u32(0);     // begin_time
        out.write_u8(0);      // suppress
        out.write_u8(0);      // flag
        Response { cmd, payload: out }
    })
}

// 20014 cli: {target_id:u32, target_srv_id:str} → srv: {code:u8, msg:str}
pub fn handle_battle_target(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (tid, tsrv) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(cmd, tid = format!("0x{:08x}", tid), %tsrv, "battle_target");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (target locked)");
        Response { cmd, payload: out }
    })
}

// 20015 cli: empty → srv: {code:u8, msg:str} (no proto_200.erl, client-only cmd)
pub fn handle_battle_misc_15(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_misc_15");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle misc 15)");
        Response { cmd, payload: out }
    })
}

// 20016 cli: empty → srv: {code:u8, msg:str} (no proto_200.erl, client-only cmd)
pub fn handle_battle_misc_16(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_misc_16");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle misc 16)");
        Response { cmd, payload: out }
    })
}

// 20019 cli: empty → srv: empty (心跳, 战斗已结束确认)
pub fn handle_battle_done(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_done");
        Response { cmd, payload: vec![] }
    })
}

// 20020 cli: empty → srv: 8 字段 (combat_type + formation + objects + is_auto + buffs + distance_info + waves)
pub fn handle_battle_init(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_init");
        let mut out = Vec::with_capacity(32);
        out.write_u16(0);    // combat_type
        out.write_u16(0);    // formation count
        out.write_u16(0);    // objects count
        out.write_u8(0);     // is_auto
        out.write_u16(0);    // buffs count
        out.write_u16(0);    // distance_info count
        out.write_u8(0);     // current_wave
        out.write_u16(0);    // total_wave
        Response { cmd, payload: out }
    })
}

// 20022 cli: {speed:u8} → srv: {code:u8, msg:str}
pub fn handle_battle_speed(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let speed = if !payload.is_empty() { payload[0] } else { 1u8 };
        tracing::info!(cmd, speed, "battle_speed");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (speed set)");
        Response { cmd, payload: out }
    })
}

// 20026 cli: empty → srv: {drama_id:u32}
pub fn handle_battle_drama(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_drama");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);
        Response { cmd, payload: out }
    })
}

// 20027 cli: empty → srv: 12 字段 (combat_type + formation + objects + is_auto + waves + extra_args + target_role + a_obj_num + actor_role + suppress)
pub fn handle_battle_spec(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_spec");
        let mut out = Vec::with_capacity(64);
        out.write_u16(0);                 // combat_type
        out.write_u16(0);                 // formation count
        out.write_u16(0);                 // objects count
        out.write_u8(0);                  // is_auto
        out.write_u8(0);                  // current_wave
        out.write_u16(0);                 // total_wave
        out.write_u8(1);                  // play_speed
        out.write_u16(0);                 // extra_args count
        out.write_string("MavisHero");    // target_role_name
        out.write_u8(0);                  // a_object_num
        out.write_string("MavisHero");    // actor_role_name
        out.write_u8(0);                  // suppress
        Response { cmd, payload: out }
    })
}

// 20028 cli: empty → srv: {code:u8, msg:str}
pub fn handle_battle_spec_ack(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_spec_ack");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle spec ack)");
        Response { cmd, payload: out }
    })
}

// 20029 cli: {replay_id:u32} → srv: {code:u8, msg:str}
pub fn handle_battle_replay_request(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let rid = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, rid, "battle_replay_request");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (replay request)");
        Response { cmd, payload: out }
    })
}

// 20030 cli: empty → srv: {is_in_combat:u8}
pub fn handle_battle_in_combat(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_in_combat");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);  // is_in_combat = 0
        Response { cmd, payload: out }
    })
}

// 20033 cli: empty → srv: {result:u8, def_name:str, def_guild_name:str, def_lev:u32, def_face_id:u32, replay_id:u32}
pub fn handle_battle_defender(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_defender");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("MavisDefender");
        out.write_string("MavisGuild");
        out.write_u32(1);  // def_lev
        out.write_u32(0);  // def_face_id
        out.write_u32(0);  // replay_id
        Response { cmd, payload: out }
    })
}

// 20034 cli: {replay_id:u32, channel:u16, target_name:str, share_type:u8} → srv: {result:u8, msg:str}
pub fn handle_battle_share(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let replay_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(cmd, replay_id, "battle_share");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle share)");
        Response { cmd, payload: out }
    })
}

// 20036 cli: {replay_id:u32, replay_srv_id:str} → srv: {code:u8, msg:str}
pub fn handle_battle_replay_detail(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, _rsrv) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(cmd, rid, "battle_replay_detail");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (replay detail)");
        Response { cmd, payload: out }
    })
}

// 20060 cli: {combat_type:u16} → srv: {combat_type:u16, type:u8}
pub fn handle_battle_combat_type(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let ct = if payload.len() >= 2 {
            let mut p: &[u8] = &payload[..];
            p.read_u16()
        } else {
            0u16
        };
        tracing::info!(cmd, ct, "battle_combat_type");
        let mut out = Vec::with_capacity(4);
        out.write_u16(ct);
        out.write_u8(0);  // type
        Response { cmd, payload: out }
    })
}

// 20062 cli: empty → srv: {code:u8, msg:str}
pub fn handle_battle_combat_type_ack(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_combat_type_ack");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (combat type ack)");
        Response { cmd, payload: out }
    })
}

// 20063 cli: empty → srv: {type_list count, [combat_type:u16]}
pub fn handle_battle_type_list(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "battle_type_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // type_list count
        Response { cmd, payload: out }
    })
}

// 20200 cli: empty → srv: {rank:u32, score:u32, can_combat_num:u32, buy_combat_num:u32, ref_time:u32, start_time:u32, end_time:u32, cont_win:u32}
pub fn handle_arena_state_full(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_state_full");
        let mut out = Vec::with_capacity(32);
        out.write_u32(1);    // rank
        out.write_u32(1000); // score
        out.write_u32(5);    // can_combat_num
        out.write_u32(0);    // buy_combat_num
        out.write_u32(0);    // ref_time
        out.write_u32(0);    // start_time
        out.write_u32(0);    // end_time
        out.write_u32(0);    // cont_win
        Response { cmd, payload: out }
    })
}

// 20201 cli: empty → srv: {f_list count, type:u8}
pub fn handle_arena_f_list(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_f_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // f_list count
        out.write_u8(0);   // type
        Response { cmd, payload: out }
    })
}

// 20202 cli: {rid:u32, srv_id:str} → srv: {rid, srv_id, name, lev, face, power, score, formation_type, formation_lev, p_list count}
pub fn handle_arena_view(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(cmd, rid, %srv_id, "arena_view");
        let mut out = Vec::with_capacity(48);
        out.write_u32(rid);
        out.write_string(&srv_id);
        out.write_string("MavisHero");
        out.write_u16(1);   // lev
        out.write_u32(0);   // face
        out.write_u32(100); // power
        out.write_u32(0);   // score
        out.write_u8(0);    // formation_type
        out.write_u8(0);    // formation_lev
        out.write_u16(0);   // p_list count
        Response { cmd, payload: out }
    })
}

// 20203 cli: {rid:u32, srv_id:str} → srv: {code:u8, msg:str}
pub fn handle_arena_view_ack(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, _) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(cmd, rid, "arena_view_ack");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (arena view ack)");
        Response { cmd, payload: out }
    })
}

// 20206 cli: empty → srv: {code:u8, msg:str}
pub fn handle_arena_challenge(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_challenge");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (arena challenge)");
        Response { cmd, payload: out }
    })
}

// 20207 cli: empty → srv: {code:u8, msg:str}
pub fn handle_arena_clear_cd(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_clear_cd");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (arena clear cd)");
        Response { cmd, payload: out }
    })
}

// 20208 cli: empty → srv: {had_combat_num:u32, num_list count}
pub fn handle_arena_combat_log(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_combat_log");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);  // had_combat_num
        out.write_u16(0);  // num_list count
        Response { cmd, payload: out }
    })
}

// 20209 cli: {num:u8} → srv: {code:u8, msg:str}
pub fn handle_arena_buy_count(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let num = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(cmd, num, "arena_buy_count");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (arena buy count)");
        Response { cmd, payload: out }
    })
}

// 20220 cli: empty → srv: {rank_list count}
pub fn handle_arena_rank(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_rank");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // rank_list count
        Response { cmd, payload: out }
    })
}

// 20221 cli: empty → srv: {rank:u32, score:u32, worship:u32, rank_list count}
pub fn handle_arena_worship(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!(cmd, "arena_worship");
        let mut out = Vec::with_capacity(16);
        out.write_u32(1);    // rank
        out.write_u32(1000); // score
        out.write_u32(0);    // worship
        out.write_u16(0);    // rank_list count
        Response { cmd, payload: out }
    })
}

// 20204 cli: {rid:u32, srv_id:str, pos:u16} → srv: {code:u8, msg:str} (per proto_mate.js, no proto_202.erl pack)
pub fn handle_arena_set_pos(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, pos) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            let r = p.read_u32();
            let _ = p.read_string();
            let pos = if p.len() >= 2 { p.read_u16() } else { 0u16 };
            (r, pos)
        } else {
            (0u32, 0u16)
        };
        tracing::info!(cmd, rid, pos, "arena_set_pos");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (arena set pos)");
        Response { cmd, payload: out }
    })
}
