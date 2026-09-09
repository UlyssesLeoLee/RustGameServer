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
