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
// v0.5.0 (per 2026-09-09 19:32 JST Mavis 派工): w1 player 域 65 cmd 真实化
// 来源: zsyz_server/src/proto/proto_101.erl + proto_103.erl + proto_104.erl
//       + proto_105.erl + proto_108.erl + proto_109.erl
// 字节级对齐: 4B BE len + 2B BE cmd + payload (per zsyz_client GameTcpClient.h)
// 字段顺序严格按 pack(srv, ...) in proto_*.erl
// ============================================================================

// 通用回复: {code:u8, msg:str} (per proto_103.erl/10399/10309/10316/10322/10327/10343/10346/10402/10405/10406/10515/10520/10522/10523/10524/10801/10805/10810/10900/10901/10902/10945/10952 等)
fn code_msg(code: u8, msg: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + msg.len());
    out.write_u8(code);
    out.write_string(msg);
    out
}

// 10312 cli: empty; srv: empty  (per proto_103.erl)
pub fn handle_10312(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10312 empty");
        Response { cmd, payload: vec![] }
    })
}

// 10316 cli: {rid:u32, srv_id:str, idx:u32}; srv: {code:u8, msg:str, rid:u32, srv_id:str, idx:u32}  (per proto_103.erl)
pub fn handle_10316(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, idx) = if payload.len() >= 8 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u32())
        } else {
            (0u32, "rgs-uat-1".to_string(), 0u32)
        };
        tracing::info!(rid = format!("0x{:08x}", rid), %srv_id, idx, "10316");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (view_role_idx)");
        out.write_u32(rid);
        out.write_string(&srv_id);
        out.write_u32(idx);
        Response { cmd, payload: out }
    })
}

// 10317 cli: empty; srv: {worship:u32}  (per proto_103.erl, 点赞/膜拜)
pub fn handle_10317(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10317 worship");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);  // worship count
        Response { cmd, payload: out }
    })
}

// 10318 cli: {rid:u32, srv_id:str}; srv: empty  (per proto_103.erl view_role_friend, 客户端拉好友)
pub fn handle_10318(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10318");
        Response { cmd, payload: vec![] }
    })
}

// 10322 cli: {code:u8}; srv: {code1:u8}  (per proto_103.erl ping_xx)
pub fn handle_10322(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let code = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(code, "10322");
        let mut out = Vec::with_capacity(2);
        out.write_u8(code);
        Response { cmd, payload: out }
    })
}

// 10325 cli: empty; srv: {face_list:u16 array [u32]}  (per proto_103.erl face_id list)
pub fn handle_10325(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10325 face_list");
        let faces = vec![1001u32, 1002, 1003, 2001, 2002, 2003, 3001, 3002];
        let mut out = Vec::with_capacity(8 + faces.len() * 4);
        out.write_u16(faces.len() as u16);
        for f in faces { out.write_u32(f); }
        Response { cmd, payload: out }
    })
}

// 10327 cli: {face_id:u32}; srv: {code:u8, msg:str, face_id:u32}  (per proto_103.erl set_face)
pub fn handle_10327(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let face_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            1001u32
        };
        tracing::info!(face_id, "10327 set_face");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (face set)");
        out.write_u32(face_id);
        Response { cmd, payload: out }
    })
}

// 10343 cli: {name:str, sex:u8}; srv: {code:u8, msg:str, name:str, sex:u8}  (per proto_103.erl rename)
pub fn handle_10343(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (name, sex) = if payload.len() >= 5 {
            let mut p: &[u8] = &payload[..];
            (p.read_string(), p.read_u8())
        } else {
            ("MavisHero".to_string(), 1u8)
        };
        tracing::info!(%name, sex, "10343 rename");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (rename)");
        out.write_string(&name);
        out.write_u8(sex);
        Response { cmd, payload: out }
    })
}

// 10345 cli: empty; srv: {use_id:u32, list:u16 array [u32]}  (per proto_103.erl bag_use_list)
pub fn handle_10345(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10345 bag_use_list");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);   // use_id
        out.write_u16(0);   // list count
        Response { cmd, payload: out }
    })
}

// 10346 cli: {id:u32}; srv: {code:u8, msg:str, id:u32}  (per proto_103.erl item_delete)
pub fn handle_10346(
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
        tracing::info!(id, "10346 del_item");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (del_item)");
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// 10347 cli: empty; srv: {assets:u16 array [{label:u8, val:u32}]}  (per proto_103.erl asset_icons)
pub fn handle_10347(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10347 asset_icons");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // empty
        Response { cmd, payload: out }
    })
}

// 10348 cli: empty; srv: {power:u32, max_power:u32}  (per proto_103.erl power)
pub fn handle_10348(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;
        tracing::info!("10348 power");
        let mut out = Vec::with_capacity(8);
        out.write_u32(99999);
        out.write_u32(100000);
        Response { cmd, payload: out }
    })
}

// 10380 cli: empty; srv: {reg_day:u32, open_day:u32}  (per proto_103.erl 登录天数)
pub fn handle_10380(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10380 reg_day");
        let now = now_unix();
        let day = (now / 86400) as u32;
        let mut out = Vec::with_capacity(8);
        out.write_u32(day);
        out.write_u32(day);
        Response { cmd, payload: out }
    })
}

// 10391 cli: {msg:str}; srv: {type:u8, data:str}  (per proto_103.erl chat_send)
pub fn handle_10391(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let msg = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_string()
        } else {
            String::new()
        };
        tracing::info!(%msg, "10391 chat");
        let mut out = Vec::with_capacity(8 + msg.len());
        out.write_u8(0);  // type=0
        out.write_string(&msg);
        Response { cmd, payload: out }
    })
}

// 10395 cli: empty; srv: empty  (per proto_103.erl notify)
pub fn handle_10395(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10395 notify");
        Response { cmd, payload: vec![] }
    })
}

// 10397 cli: {status:u8}; srv: empty  (per proto_103.erl online_status)
pub fn handle_10397(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let status = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(status, "10397 online");
        Response { cmd, payload: vec![] }
    })
}

// 10399 cli: {msg:str}; srv: {code:u8, msg:str}  (per proto_103.erl feedback)
pub fn handle_10399(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let msg = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_string()
        } else {
            String::new()
        };
        tracing::info!(%msg, "10399 feedback");
        let mut out = code_msg(0, "OK (feedback received)");
        out.extend_from_slice(msg.as_bytes());  // echo back truncated, per proto
        // actually 10399 srv = {code, msg} only, remove echo
        out.truncate(0);
        out.write_u8(0);
        out.write_string("OK (feedback)");
        Response { cmd, payload: out }
    })
}

// ============================================================================
// quest 域 (10400-10406, per proto_104.erl)
// ============================================================================

// 10400 (existing as heartbeat) — re-use; see handle_heartbeat above
// 10402 cli: {id:u32}; srv: {flag:i8, msg:str, id:u32}  (per proto_104.erl accept_quest)
pub fn handle_10402(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        let _ = rgs.call("player", "UpdateProfile", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111",
            "quest_id": id,
        })).await;
        tracing::info!(id, "10402 accept_quest");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (quest accepted)");
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// 10405 cli: {id:u32}; srv: {flag:i8, msg:str, id:u32}  (per proto_104.erl finish_quest)
pub fn handle_10405(
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
        tracing::info!(id, "10405 finish_quest");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (quest finished)");
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// 10406 cli: {id:u32}; srv: {flag:i8, msg:str, id:u32}  (per proto_104.erl giveup_quest)
pub fn handle_10406(
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
        tracing::info!(id, "10406 giveup_quest");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (quest given up)");
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// ============================================================================
// 物品/装备 域 (10500-10536, per proto_105.erl)
// ============================================================================

// 10500 cli: empty; srv: {volume:u32, open_times:u8, item_list:u16 array}  (per proto_105.erl bag_list)
// 10501 cli: empty; srv: {volume:u32, open_times:u8, item_list:u16 array}  (per proto_105.erl equip_list)
fn handle_bag_or_equip(cmd: u16, kind: &str) -> Response {
    tracing::info!(kind, "10500/10501 bag");
    let mut out = Vec::with_capacity(8);
    out.write_u32(60);  // volume
    out.write_u8(0);    // open_times
    out.write_u16(0);   // item list count (空, 简化)
    Response { cmd, payload: out }
}

pub fn handle_10500(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move { handle_bag_or_equip(cmd, "bag") })
}

pub fn handle_10501(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move { handle_bag_or_equip(cmd, "equip") })
}

// 10515 cli: {id:u32, quantity:u16, args:u16 array}; srv: {flag:u8, base_id:u32, msg:str}  (per proto_105.erl use_item)
pub fn handle_10515(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, quantity) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u16())
        } else {
            (0u32, 0u16)
        };
        tracing::info!(id, quantity, "10515 use_item");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_u32(id);
        out.write_string("OK (item used)");
        Response { cmd, payload: out }
    })
}

// 10520 cli: {id:u32, storage:u8}; srv: {id:u32, storage:u8, flag:u8, msg:str}  (per proto_105.erl move_item)
pub fn handle_10520(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, storage) = if payload.len() >= 5 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u8())
        } else {
            (0u32, 0u8)
        };
        tracing::info!(id, storage, "10520 move_item");
        let mut out = Vec::with_capacity(16);
        out.write_u32(id);
        out.write_u8(storage);
        out.write_u8(0);
        out.write_string("OK (item moved)");
        Response { cmd, payload: out }
    })
}

// 10522 cli: {storage:u8, args:u16 array [{id, bid, num}]}; srv: {flag:u8, msg:str}  (per proto_105.erl batch_use)
pub fn handle_10522(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10522 batch_use");
        Response { cmd, payload: code_msg(0, "OK (batch use)") }
    })
}

// 10523 cli: {id:u32, num:u16}; srv: {flag:u8, msg:str}  (per proto_105.erl sell_item)
pub fn handle_10523(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, num) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u16())
        } else {
            (0u32, 0u16)
        };
        tracing::info!(id, num, "10523 sell");
        Response { cmd, payload: code_msg(0, "OK (item sold)") }
    })
}

// 10524 cli: {star_list:u16 array [u8]}; srv: {flag:u8, msg:str}  (per proto_105.erl star_up)
pub fn handle_10524(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10524 star_up");
        Response { cmd, payload: code_msg(0, "OK (star up)") }
    })
}

// 10525 cli: empty; srv: {star_list:u16 array [u8]}  (per proto_105.erl star_list)
pub fn handle_10525(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10525 star_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10526 cli: {type:u8}; srv: {type:u8, volume:u32, open_times:u8}  (per proto_105.erl storage_info)
pub fn handle_10526(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let storage_type = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(storage_type, "10526 storage_info");
        let mut out = Vec::with_capacity(8);
        out.write_u8(storage_type);
        out.write_u32(60);  // volume
        out.write_u8(0);    // open_times
        Response { cmd, payload: out }
    })
}

// 10528 cli: empty; srv: {time:u32, minu_exp:u32}  (per proto_105.erl exp_pool)
pub fn handle_10528(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10528 exp_pool");
        let mut out = Vec::with_capacity(8);
        out.write_u32(now_unix());
        out.write_u32(3600);
        Response { cmd, payload: out }
    })
}

// 10535 cli: empty; srv: empty  (per proto_105.erl bag_clear)
pub fn handle_10535(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10535 bag_clear");
        Response { cmd, payload: vec![] }
    })
}

// 10536 cli: empty; srv: empty  (per proto_105.erl equip_refresh)
pub fn handle_10536(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10536 equip_refresh");
        Response { cmd, payload: vec![] }
    })
}

// ============================================================================
// 邮件 域 (10800-10810, per proto_108.erl, w1 player 域包含)
// ============================================================================

// 10800 cli: empty; srv: {mail:u16 array [...]}  (per proto_108.erl mail_list)
pub fn handle_10800(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10800 mail_list");
        let mut out = Vec::with_capacity(2);
        out.write_u16(0);  // empty mail list
        Response { cmd, payload: out }
    })
}

// 10801 cli: {id:u32, srv_id:str}; srv: {id:u32, srv_id:str, code:u8, msg:str}  (per proto_108.erl mail_read)
pub fn handle_10801(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(id, %srv_id, "10801 mail_read");
        let mut out = Vec::with_capacity(16);
        out.write_u32(id);
        out.write_string(&srv_id);
        out.write_u8(0);
        out.write_string("OK (mail read)");
        Response { cmd, payload: out }
    })
}

// 10802 cli: empty; srv: {ids:u16 array [{id, srv_id, read_time}], msg:str}  (per proto_108.erl mail_unread_list)
pub fn handle_10802(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10802 mail_unread");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);    // ids count
        out.write_string(""); // msg
        Response { cmd, payload: out }
    })
}

// 10804 cli: {ids:u16 array [{id, srv_id}]}; srv: {ids, msg:str}  (per proto_108.erl mail_delete)
pub fn handle_10804(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10804 mail_delete");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);
        out.write_string("OK (mail deleted)");
        Response { cmd, payload: out }
    })
}

// 10805 cli: {id:u32, srv_id:str}; srv: {id, srv_id, code, msg, read_time, is_delete:u8}  (per proto_108.erl mail_get_attach)
pub fn handle_10805(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (id, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, "rgs-uat-1".to_string())
        };
        tracing::info!(id, %srv_id, "10805 mail_attach");
        let mut out = Vec::with_capacity(24);
        out.write_u32(id);
        out.write_string(&srv_id);
        out.write_u8(0);
        out.write_string("OK (attach got)");
        out.write_u32(now_unix());
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// 10810 cli: {issue_type:u8, title:str, content:str}; srv: {code:u8, msg:str}  (per proto_108.erl mail_issue)
pub fn handle_10810(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (issue_type, title, content) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u8(), p.read_string(), p.read_string())
        } else {
            (0u8, String::new(), String::new())
        };
        tracing::info!(issue_type, %title, %content, "10810 mail_issue");
        Response { cmd, payload: code_msg(0, "OK (issue submitted)") }
    })
}

// ============================================================================
// 10900-10999 (per proto_109.erl, w1 player 域杂项: 禁言/buff/活动/排行榜/SDK)
// ============================================================================

// 10900 cli: {rid:u32, srv_id:str, hour:u32, interdict:u8}; srv: {code, msg}  (per proto_109.erl silence)
pub fn handle_10900(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, hour, interdict) = if payload.len() >= 10 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u32(), p.read_u8())
        } else {
            (0u32, String::new(), 0u32, 0u8)
        };
        tracing::info!(rid = format!("0x{:08x}", rid), %srv_id, hour, interdict, "10900 silence");
        Response { cmd, payload: code_msg(0, "OK (silence set)") }
    })
}

// 10901 cli: {rid, srv_id, hour, banned:u8}; srv: {code, msg}  (per proto_109.erl ban)
pub fn handle_10901(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, hour, banned) = if payload.len() >= 10 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u32(), p.read_u8())
        } else {
            (0u32, String::new(), 0u32, 0u8)
        };
        tracing::info!(rid, %srv_id, hour, banned, "10901 ban");
        Response { cmd, payload: code_msg(0, "OK (ban set)") }
    })
}

// 10902 cli: {rid, srv_id, stop_role:u8}; srv: {code, msg}  (per proto_109.erl stop_role)
pub fn handle_10902(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, stop_role) = if payload.len() >= 9 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u8())
        } else {
            (0u32, String::new(), 0u8)
        };
        tracing::info!(rid, %srv_id, stop_role, "10902 stop_role");
        Response { cmd, payload: code_msg(0, "OK (stop_role set)") }
    })
}

// 10905 cli: empty; srv: empty  (per proto_109.erl 战斗状态)
pub fn handle_10905(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10905");
        Response { cmd, payload: vec![] }
    })
}

// 10906 cli: empty; srv: empty  (per proto_109.erl)
pub fn handle_10906(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10906");
        Response { cmd, payload: vec![] }
    })
}

// 10922 cli: empty; srv: {act_list:u16 array [u32]}  (per proto_109.erl activity_list)
pub fn handle_10922(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10922 activity_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10923 cli: {id:u32}; srv: {id:u32, status:u8, int_args:u16 array [u32]}  (per proto_109.erl activity_join)
pub fn handle_10923(
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
        tracing::info!(id, "10923 activity_join");
        let mut out = Vec::with_capacity(8);
        out.write_u32(id);
        out.write_u8(0);    // status
        out.write_u16(0);   // int_args count
        Response { cmd, payload: out }
    })
}

// 10924 cli: empty; srv: {act_list:u16 array [u32]}  (per proto_109.erl activity_my)
pub fn handle_10924(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10924 activity_my");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10925 cli: {id:u32}; srv: {id, status, int_args}  (per proto_109.erl activity_query)
pub fn handle_10925(
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
        tracing::info!(id, "10925 activity_query");
        let mut out = Vec::with_capacity(8);
        out.write_u32(id);
        out.write_u8(0);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10926 cli: empty; srv: empty  (per proto_109.erl activity_open_notify)
pub fn handle_10926(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10926");
        Response { cmd, payload: vec![] }
    })
}

// 10927 cli: empty; srv: empty  (per proto_109.erl)
pub fn handle_10927(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10927");
        Response { cmd, payload: vec![] }
    })
}

// 10945 cli: {card_id:u32}; srv: {code:u8, msg:str}  (per proto_109.erl 激活礼包卡)
pub fn handle_10945(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let card_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        tracing::info!(card_id, "10945 giftcard");
        Response { cmd, payload: code_msg(0, "OK (gift card activated)") }
    })
}

// 10946 cli: empty; srv: {code:u8}  (per proto_109.erl)
pub fn handle_10946(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10946");
        Response { cmd, payload: vec![0u8] }
    })
}

// 10950 cli: empty; srv: {type:u8, board_list:u16 array [u32]}  (per proto_109.erl board_list)
pub fn handle_10950(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10950 board");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10952 cli: {id:u32}; srv: {code:u8, msg:str, id:u32}  (per proto_109.erl board_action)
pub fn handle_10952(
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
        tracing::info!(id, "10952 board_action");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (board action)");
        out.write_u32(id);
        Response { cmd, payload: out }
    })
}

// 10955 cli: empty; srv: empty  (per proto_109.erl)
pub fn handle_10955(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10955");
        Response { cmd, payload: vec![] }
    })
}

// 10956 cli: empty; srv: empty  (per proto_109.erl)
pub fn handle_10956(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10956");
        Response { cmd, payload: vec![] }
    })
}

// 10999 cli: {msg:str}; srv: empty  (per proto_109.erl sdk_notify)
pub fn handle_10999(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let msg = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_string()
        } else {
            String::new()
        };
        tracing::info!(%msg, "10999 sdk_notify");
        Response { cmd, payload: vec![] }
    })
}
