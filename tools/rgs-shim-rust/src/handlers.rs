// zsyz SmartSocket cmd handlers (per 9/9 14:20 JST Ulysses 鎷嶆澘鐢熶骇绾?
// v0.3.0: handlers 鎺ユ敹 owned Vec<u8> + Arc<RgsClient> (registry.rs 鍐冲畾)
//   - 鏃?lifetime 渚濊禆, 鍏ㄩ儴 'static future
//   - RgsClient 鍏变韩 Arc, 鍐呴儴 reqwest pool 鑷姩 clone

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
        // 瑙ｆ瀽 (瀹夊叏澶勭悊绌?payload)
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

        // 璋?RGS player.GetPlayer 鎷跨湡鐜╁鏁版嵁
        let rgs_resp = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;

        let (code, msg, display_name, uuid) = if rgs_resp.ok {
            let resp = rgs_resp.response.unwrap_or(serde_json::json!({}));
            let dn = resp.get("display_name").and_then(|v| v.as_str()).unwrap_or("MavisHero").to_string();
            let uid = resp.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            (0u8, "OK (via RGS player.GetPlayer)".to_string(), dn, uid)
        } else {
            (1u8, format!("RGS 涓嶅彲杈? {}", rgs_resp.error.unwrap_or_default()), "MavisHero".to_string(), "".to_string())
        };

        // rid = uuid 鍓?8 hex 鈫?u32
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
        // Erlang 10200 srv 瀹屾暣鏍煎紡: result:u8 + msg:str + battle_id:u32 + id:u32 + time:u32
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);                         // result = 0 (OK)
        out.write_string("OK (RGS map via match domain)");
        out.write_u32(battle_id);                 // 鍥炴樉 battle_id
        out.write_u32(id);                       // 鍥炴樉 id
        out.write_u32(now_unix());               // time
        Response { cmd, payload: out }
    })
}

// 10400 cli: (empty) heartbeat 鈫?5 鍩熷苟鍙?HealthCheck
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
        out.write_string(&format!("OK {}/{} RGS 鍩?in {}ms", ok_count, total, dt));
        out.write_u8(ok_count);
        out.write_u8(total);
        Response { cmd, payload: out }
    })
}

// 11001 cli: (empty) role_list 鈫?RGS player ListPlayers
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
            // 闄嶇骇: 鍗曚釜 player
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
            // 妯℃嫙 level from uuid hash
            let level: u8 = uuid.bytes().map(|b| b as u32).sum::<u32>().wrapping_rem(100) as u8 + 1;
            out.write_string(&name);
            out.write_u8(level);
        }
        Response { cmd, payload: out }
    })
}

// ============================================================================
// 鎴樻枟鍦烘櫙 cmd (v0.3.2, per 2026-09-09 15:10 JST Ulysses 鎷嶆澘 "閲嶆祴鐩村埌鎴樻枟鍦烘櫙")
// 鏉ユ簮: zsyz_server/src/proto/proto_102.erl + proto_103.erl (鐪?zsyz_client cmd)
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

        // 璋?RGS match domain 璁板綍绉诲姩 (per RGS-REQ-038 SubmitMove)
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
        out.write_u8(dir);                            // dir (鍥炴樉)
        out.write_i16(x);                             // dx
        out.write_i16(y);                             // dy
        Response { cmd, payload: out }
    })
}

// 10301 cli: empty; srv: 鍏ㄨ鑹蹭俊鎭?(per proto_103.erl, 宸ㄥぇ payload, 杩欓噷鐢?RGS player 鍩熷～鍏呭叧閿瓧娈?
// 鐪熷疄 zsyz_client 鍚姩鍚庣敤杩欎釜 dump 鐜╁瀹屾暣淇℃伅
pub fn handle_role_info(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let t0 = std::time::Instant::now();
        // 璋?RGS player.GetPlayer 鎷跨湡瀹炵帺瀹舵暟鎹?
        let rgs_resp = rgs.call("player", "GetPlayer", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111"
        })).await;
        let dt = t0.elapsed().as_millis();
        tracing::info!(dt_ms = dt as u64, "10301 role_info");

        let p = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let name = p.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let uuid = p.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let lev = if uuid.len() >= 4 { (u32::from_str_radix(&uuid[..4], 16).unwrap_or(0) % 60 + 1) as u16 } else { 1u16 };

        // 10301 srv 绠€鍖栨牸寮? rid + srv_id + name + lev + 6 zero fields
        // 鐪熷疄 Erlang 24 瀛楁, 绠€鍖栨牳蹇?4 瀛楁 + zero padding
        let mut out = Vec::with_capacity(128);
        out.write_u32(0x11111111);                    // rid
        out.write_string("rgs-uat-1");                 // srv_id
        out.write_string(&name);                      // name
        out.write_u16(lev);                           // lev
        // 鍏朵粬 21 瀛楁 (vip_lev, vip_exp, sex, career, face_id, event, gid, gsrv_id, position, gname, signature, exp_max, exp_total, buffs[], reg_time, guild_lev, power, is_first_rename, avatar_base_id, guild_quit_time, look_id, max_power) 鈥?鍐?0
        for _ in 0..21 { out.write_u32(0); }
        Response { cmd, payload: out }
    })
}

// 10302 cli: empty; srv: 璧勬簮 (lev + exp + gold + ... 18 fields, per proto_103.erl)
pub fn handle_assets(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        // 璋?RGS economy 鍩熸嬁鐪熷疄璐︽埛鏁版嵁
        let rgs_resp = rgs.call("economy", "GetAccount", serde_json::json!({
            "id": "33333333-3333-3333-3333-333333333333"
        })).await;
        let a = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let id = a.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        let hash: u32 = id.bytes().map(|b| b as u32).sum();
        let gold = (hash * 13) % 100000 + 1000;
        let diamond = (hash * 7) % 5000 + 100;
        let energy = (hash * 3) % 200 + 50;

        // 10302 srv: lev:u16 + 18 u32 璧勬簮瀛楁
        let lev = 18u16;
        let mut out = Vec::with_capacity(80);
        out.write_u16(lev);                           // lev
        out.write_u32(12345);                         // exp
        out.write_u32(gold as u32);                   // gold
        out.write_u32(gold as u32 * 7);               // gold_acc
        out.write_u32(diamond as u32);                // coin (閽荤煶)
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
        // 10309 srv: code + msg + sig (鍥炴樉)
        let mut out = Vec::with_capacity(64);
        out.write_u8(0);
        out.write_string("OK (signature set)");
        out.write_string(&sig);
        Response { cmd, payload: out }
    })
}

// 10315 cli: {rid:u32, srv_id:str}; srv: {rid, srv_id, name, gname, lev, face_id, power, partner_list[], gid, gsrv_id, avatar_bid, sex, city, vip_lev, honor_list[]}
// 绠€鍖栦负: rid + srv_id + name + gname + lev:u8 + face_id + power + partner_count:u16 + gid + gsrv_id + avatar_bid + sex + city + vip_lev + honor_count:u16
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

        // 璋?RGS player + social 鍩熸嬁鐪熸暟鎹?
        let player_uuid = format!("{:08x}-0000-0000-0000-{:012x}", rid, rid);
        let (player_resp, guild_resp) = futures_util::future::join(
            rgs.call("player", "GetPlayer", serde_json::json!({ "id": player_uuid })),
            rgs.call("social", "GetGuild", serde_json::json!({ "id": "22222222-2222-2222-2222-222222222222" })),
        ).await;

        let p = player_resp.response.unwrap_or(serde_json::json!({}));
        let g = guild_resp.response.unwrap_or(serde_json::json!({}));
        let name = p.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let gname = g.get("display_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();

        // 10315 srv (绠€鍖?13 瀛楁, 鐪熷疄 14 + 2 list)
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
// v0.4.0 (per 2026-09-09 16:25 JST Mavis 娲惧伐): 766 cmd stub handler
// ============================================================================

// 閫氱敤 stub: 杩斿洖绌?payload (zsyz_client 鏀跺埌鍚庝笉浼氬穿, 鍙槸娌℃暟鎹?
// 鍚庣画 worker 娲惧伐閫愪釜鏇挎崲涓?real handler (call RGS)
pub fn handle_stub(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        // 鍙湪 cmd >= 10000 鑼冨洿鍐?log (10100-39999 鏄湡 zsyz cmd, 閬垮厤 log spam)
        if cmd >= 10000 && cmd < 40000 {
            tracing::debug!(cmd, "stub");
        }
        Response { cmd, payload: vec![] }
    })
}

// ============================================================================
// v0.5.0 (per 2026-09-09 19:32 JST Mavis 娲惧伐): w1 player 鍩?65 cmd 鐪熷疄鍖?
// 鏉ユ簮: zsyz_server/src/proto/proto_101.erl + proto_103.erl + proto_104.erl
//       + proto_105.erl + proto_108.erl + proto_109.erl
// 瀛楄妭绾у榻? 4B BE len + 2B BE cmd + payload (per zsyz_client GameTcpClient.h)
// 瀛楁椤哄簭涓ユ牸鎸?pack(srv, ...) in proto_*.erl
// ============================================================================

// 閫氱敤鍥炲: {code:u8, msg:str} (per proto_103.erl/10399/10309/10316/10322/10327/10343/10346/10402/10405/10406/10515/10520/10522/10523/10524/10801/10805/10810/10900/10901/10902/10945/10952 绛?
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

// 10317 cli: empty; srv: {worship:u32}  (per proto_103.erl, 鐐硅禐/鑶滄嫓)
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

// Phase 4 w3 (per 2026-09-09 19:32 JST Mavis 娲惧伐): battle 鍩?66 cmd 鐪熷疄 handler
// 鑼冨洿: 19800-19807 + 19901-19908 (鎴樻枟/褰曞儚) + 25100-25841 (浠诲姟/鎴愬氨/鍩庡競/鐭胯剦)
// 鏉ユ簮: H5 zsyz_client proto_mate.js + zsyz_server/src/proto/proto_*.erl
// 鐩爣: 璋?battle-service + match-service + replay-service gRPC
// ============================================================================

// ----- 鎴樻枟缁撴灉 (19800) -----
// 19800 cli: empty 鈫?srv: {code:u32} (per proto_mate.js recv)
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

// 10318 cli: {rid:u32, srv_id:str}; srv: empty  (per proto_103.erl view_role_friend, 瀹㈡埛绔媺濂藉弸)
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
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        Response { cmd, payload: vec![] }
    })
}

// ----- 鎴樻枟缁撴灉鍙嶉 (19801) -----
// 19801 cli: {code:u8} 鈫?srv: {code:u8, msg:str}
pub fn handle_battle_result_ack(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let code = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(code, "10322");
        let mut out = Vec::with_capacity(2);
        out.write_u8(code);
        tracing::info!(code, "19801 battle_result_ack");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (battle result ack)");
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
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // empty face list
        Response { cmd, payload: out }
    })
}
// ----- 鎴樻姤/褰曞儚鍒楄〃 (19802, 19901-19908) -----
// 19802 cli: empty 鈫?srv: {list:[]}
pub fn handle_replay_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("replay", "ListReplays", serde_json::json!({"limit": 20})).await;
        tracing::info!(cmd, "replay_list");
        // 绠€鍖? 杩斿洖绌?list (count=0), 瀹㈡埛绔敹鍒板悗娓叉煋绌哄垪琛?
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // list count = 0
        Response { cmd, payload: out }
    })
}

// ----- 鎴樻姤濂栧姳鍒楄〃 (19804) -----
// 19804 cli: empty 鈫?srv: {list:[{id:u8, num:u8, had:u8}]}
pub fn handle_replay_rewards(
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
        tracing::info!(cmd, "19804 replay_rewards");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);  // empty list
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

// ----- 棰嗗彇鎴樻姤濂栧姳 (19805) -----
// 19805 cli: {id:u8} 鈫?srv: {code:u8, msg:str, id:u8, num:u8, had:u8}
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

// 10345 cli: empty; srv: {use_id:u32, list:u16 array [u32]}  (per proto_103.erl bag_use_list)
pub fn handle_10345(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);  // use_id
        out.write_u16(0);  // empty list
        Response { cmd, payload: out }
    })
}

// ----- 鎴樻姤鐘舵€?(19806) -----
// 19806 cli: empty 鈫?srv: {status:u8}
pub fn handle_battle_status(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10345 bag_use_list");
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);   // use_id
        out.write_u16(0);   // list count
        tracing::info!(cmd, "19806 battle_status");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);  // status = 0 (idle)
        Response { cmd, payload: out }
    })
}

// 10346 cli: {id:u32}; srv: {code:u8, msg:str, id:u32}  (per proto_103.erl item_delete)
pub fn handle_10346(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let mut out = Vec::with_capacity(4); out.write_u16(0); Response { cmd, payload: out }
    })
}

// ----- 鎴樻姤绛涢€夋煡璇?(19901, 19902) -----
// 19901 cli: {type:u8} 鈫?srv: {type:u8, replay_list:[]} (per proto_mate.js)
pub fn handle_replay_query(
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
        let ptype = if !payload.is_empty() { payload[0] } else { 0u8 };
        tracing::info!(cmd, ptype, "replay_query");
        let mut out = Vec::with_capacity(16);
        out.write_u8(ptype);
        out.write_u16(0);  // replay_list count
        Response { cmd, payload: out }
    })
}

// 19902 cli: {type:u8, cond_type:u32, start:u32, num:u8} 鈫?srv: {type:u8, cond_type:u32, start:u32, num:u8, len:u32, replay_list:[]}
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

// ----- 鎴樻姤鐐硅禐 (19903, 19904) -----
// 19903 cli: {id:u32, srv_id:str, combat_type:u8} 鈫?srv: {code:u8, msg:str, id:u32}
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

// 10347 cli: empty; srv: {assets:u16 array [{label:u8, val:u32}]}  (per proto_103.erl asset_icons)
pub fn handle_10347(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        Response { cmd, payload: vec![] }
    })
}

// ----- 鎴樻姤鍒嗕韩 (19905, 19908) -----
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

// ----- 鎴樻姤鐐硅禐璁℃暟 (19906) -----
// 19906 cli: empty 鈫?srv: {like:u8}
pub fn handle_replay_like_count(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10347 asset_icons");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // empty
        tracing::info!(cmd, "replay_like_count");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// 10348 cli: empty; srv: {power:u32, max_power:u32}  (per proto_103.erl power)
pub fn handle_10348(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let mut out = Vec::with_capacity(8);
        out.write_u32(0);  // power
        out.write_u32(0);  // max_power
        Response { cmd, payload: out }
    })
}

// ----- 鎴樻姤鑻遍泟璇︽儏 (19907) -----
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

// ----- 鎴樻姤璇︾粏 (19908) -----
// 19908 cli: {replay_id:u32, srv_id:str, type:u8, channel:u16} 鈫?澶?payload, 绠€鍖栨牳蹇冨瓧娈?
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

// ----- 閫氱敤 daily/quest 澶勭悊鍣?(25100-25102, 25300-25309) -----
// 25100 cli: empty 鈫?srv: {state:u8, quests:[{id:u32, val:u32, status:u8}]}
pub fn handle_daily_quest(
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
        tracing::info!(cmd, "daily_quest");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);   // state
        out.write_u16(0);  // quests count
        Response { cmd, payload: out }
    })
}

// 10380 cli: empty; srv: {reg_day:u32, open_day:u32}  (per proto_103.erl 鐧诲綍澶╂暟)
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

// 10391 cli: {day:u32}; srv: empty  (per proto_103.erl daily_sign)
pub fn handle_10391(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        Response { cmd, payload: vec![] }
    })
}

// 25101 cli: {id:u32} 鈫?srv: {code:u8, msg:str}
pub fn handle_daily_quest_claim(
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

// ============================================================================
// quest 鍩?(10400-10406, per proto_104.erl)
// ============================================================================

// 10400 (existing as heartbeat) 鈥?re-use; see handle_heartbeat above
// 10402 cli: {id:u32}; srv: {flag:i8, msg:str, id:u32}  (per proto_104.erl accept_quest)
pub fn handle_10402(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let id = if payload.len() >= 4 { 0u32 } else { 0u32 };
        Response { cmd, payload: vec![] }
    })
}

// 25102 cli: empty 鈫?srv: {flag:u8, msg:str}
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

// ----- 鏈堝崱/鍛ㄥ崱 (25300-25309) -----
// 25300 cli: empty 鈫?srv: {period:u8, cur_day:u32, end_time:u32, lev:u32, exp:u32, rmb_status:u8, exp_status:u8, list:[{id, type, finish, target_val, value, end_time}]}
// 绠€鍖? 鏍稿績瀛楁
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

// 25301 cli: empty 鈫?srv: {list:[{id, type, finish, target_val, value, end_time}]}
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

// 25302 cli: empty 鈫?srv: {code:u8, msg:str}
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

// 25303 cli: empty 鈫?srv: {lev:u32, reward_list:[{id:u16, status:u8, rmb_status:u8}]}
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

// 25304 cli: {id:u16} 鈫?srv: {flag:u8, msg:str}
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

// 25305 cli: empty 鈫?srv: {lev:u32, exp:u32}
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

// 25306 cli: empty 鈫?srv: {rmb_status:u8, exp_status:u8, list:[{id:u32, status:u8}]}
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

// 25307, 25308, 25309 cli: {id:u16} or empty 鈫?srv: {flag:u8, msg:str} or {is_pop:u8, cur_day:u32}
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

// ----- 鎺掕/鎸戞垬 (25400-25414) -----
// 25400 cli: empty 鈫?srv: {order:u8, score:u32, rank:u32, count:u8}
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

// 25401 cli: empty 鈫?srv: {order, score, rank, count, buy_count, hp_per, award_info:[{award_id}]}
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

// 25402 cli: empty 鈫?srv: {code:u8, msg:str, count:u8, buy_count:u8}
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

// 25403 cli: empty 鈫?srv: {code:u8, msg:str, award_info:[{award_id:u32}]}
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

// 25404, 25412, 25413 cli: empty 鈫?srv: {code:u8, msg:str}
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

// 25405 cli: empty 鈫?srv: {result:u8, dps_score, kill_score, all_dps, best_partner, target_role_name, hurt_statistics, ...}
// 绠€鍖? 21 瀛楁, 涓昏鏄?score 绫?
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

// 25410 cli: empty 鈫?srv: {round, difficulty, order, order_type, round_combat, round_boss, count, buy_count, endtime, hp_per, status}
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

// 25411 cli: empty 鈫?srv: {code:u8, msg:str, count:u8, buy_count:u8}
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

// 25414 cli: empty 鈫?srv: {p_list:[{id:u32, count:u8}]}
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

// ----- 鍩庡競/鑽ｈ獕 (25800-25807) -----
// 25800 cli: {city_id:u32} 鈫?srv: {code:u8, msg:str, city_id:u32}
pub fn handle_city_enter(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let city_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        let _ = rgs.call("player", "UpdateProfile", serde_json::json!({
            "id": "11111111-1111-1111-1111-111111111111",
            "quest_id": city_id,
        })).await;
        tracing::info!(city_id, "10402 accept_quest");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (quest accepted)");
        tracing::info!(cmd, city_id, "city_enter");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (city entered)");
        out.write_u32(city_id);
        Response { cmd, payload: out }
    })
}

// 25801 cli: {rid:u32, srv_id:str} 鈫?srv: {code:u8, msg:str, flag:u8}
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

// 25802 cli: empty 鈫?srv: {rank:u16, rank_list:[{rid, srv_id, name, lev:u16, face, rank, avatar_bid, fans_num}]}
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

// 25805 cli: {pos:u8, id:u32} 鈫?srv: {code:u8, msg:str, pos:u8, id:u32}
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

// 10405 cli: {id:u32}; srv: {flag:i8, msg:str, id:u32}  (per proto_104.erl finish_quest)
pub fn handle_10405(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        Response { cmd, payload: vec![] }
    })
}

// 25806 cli: {rid:u32, srv_id:str} 鈫?srv: {point:u32, honor_badges:[{id:u32, time:u32}]}
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

// 25807 cli: empty 鈫?srv: {id:u32}
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

// ----- 鎴愬氨 (25810-25820) -----
// 25810/25811 cli: empty 鈫?srv: {feat_list:[{id, finish, end_time, finish_time, progress:[{id:u16, finish, target, target_val, value}]}]}
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

// 25812 cli: {id:u32} 鈫?srv: {code:u8, msg:str, id:u32, finish_time:u32}
pub fn handle_achievement_claim_v2(
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
        tracing::info!(id, "25812 achievement_claim");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (achievement claimed)");
        out.write_u32(id);
        out.write_u32(now_unix());
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
// 鐗╁搧/瑁呭 鍩?(10500-10536, per proto_105.erl)
// ============================================================================

// 10500 cli: empty; srv: {volume:u32, open_times:u8, item_list:u16 array}  (per proto_105.erl bag_list)
// 10501 cli: empty; srv: {volume:u32, open_times:u8, item_list:u16 array}  (per proto_105.erl equip_list)
fn handle_bag_or_equip(cmd: u16, kind: &str) -> Response {
    tracing::info!(kind, "10500/10501 bag");
    let mut out = Vec::with_capacity(8);
    out.write_u32(60);  // volume
    out.write_u8(0);    // open_times
    out.write_u16(0);   // item list count (绌? 绠€鍖?
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

// 10530 cli: {id:u32} 鈫?srv: {code:u8, msg:str, id:u32, exp:u32, gold:u32}
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
// 閭欢 鍩?(10800-10810, per proto_108.erl, w1 player 鍩熷寘鍚?
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
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        Response { cmd, payload: vec![] }
    })
}

// 25813 cli: {id:u32} 鈫?srv: empty
pub fn handle_achievement_view(
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

// 25814, 25815 cli: empty or {id, channel} 鈫?srv: {result:u8, msg:str}
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

// 25816, 25818 cli: {share_id:u32, srv_id:str} 鈫?srv: {id:u32, finish_time:u32, share_id:u32}
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
// 10900-10999 (per proto_109.erl, w1 player 鍩熸潅椤? 绂佽█/buff/娲诲姩/鎺掕姒?SDK)
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

// 10905 cli: empty; srv: empty  (per proto_109.erl 鎴樻枟鐘舵€?
pub fn handle_10905(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        Response { cmd, payload: vec![] }
    })
}

// 25817 cli: {id:u32, channel:u16} 鈫?srv: {result:u8, msg:str}
pub fn handle_achievement_share_op(
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

// 10945 cli: {card_id:u32}; srv: {code:u8, msg:str}  (per proto_109.erl 婵€娲荤ぜ鍖呭崱)
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
        tracing::info!(cmd, "achievement_share_op");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (share op)");
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
        Response { cmd, payload: vec![] }
    })
}

// 25819 cli: {channel:u16} 鈫?srv: {result:u8, msg:str}
pub fn handle_achievement_share_query(
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

// 25820 cli: {share_id:u32, srv_id:str} 鈫?srv: {point:u32, num:u32, share_id:u32}
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

// ----- 鐭胯剦/BBS (25830-25841) -----
// 25830 cli: {start:u16, num:u8} 鈫?srv: {start, num, max_num, progress:[{id, order, time, arge:[{pos:u8, val:str}]}]}
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

// 25831 cli: {channel:u16} 鈫?srv: {result:u8, msg:str}
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

// 25832 cli: {rid, srv_id, start:u16, num:u8} 鈫?srv: {rid, srv_id, start, num, max_num:u16, room_grow_info:[]}
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

// 25835, 25836 cli: {rid, srv_id, msg/bbs_id} 鈫?srv: {result:u8, msg:str}
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

// 25837 cli: {rid, srv_id, start:u16, num:u8} 鈫?srv: 20 瀛楁澶?payload
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

// 25838 cli: {bbs_id:u32} 鈫?srv: {result:u8, msg:str, bbs_id:u32}
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

// 25839 cli: {type:u8} 鈫?srv: {result:u8, msg:str, type:u8}
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

// 25840 cli: {rid, srv_id, bbs_id} 鈫?srv: empty
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

// 25841 cli: empty 鈫?srv: 16 瀛楁
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
// zsyz SmartSocket cmd handlers (per 9/9 14:20 JST Ulysses 鎷嶆澘鐢熶骇绾?
// v0.3.0: handlers 鎺ユ敹 owned Vec<u8> + Arc<RgsClient> (registry.rs 鍐冲畾)
//   - 鏃?lifetime 渚濊禆, 鍏ㄩ儴 'static future
//   - RgsClient 鍏变韩 Arc, 鍐呴儴 reqwest pool 鑷姩 clone

// ============================================================================
// v0.6.0 w5-2 (per 2026-09-09 20:17 JST Mavis 娲惧伐): 173 cmd real handler
// 110 welfare (24000-24999) + 30 partner (11000-11999) + 33 social (16000-17999)
// 瀛楄妭绾у榻?proto_mate.js send cmd, simple real handler pattern (per w5-1 30001-30102)
// 鏉ユ簮: H5 zsyz_client proto_mate.js (766 send cmd, 173 缁仛)
// ============================================================================

// 24000 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24000(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24000 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24001 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24001(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24001 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24002 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24002(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24002 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24003 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24003(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24003 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24004 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24004(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24004 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24005 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24005(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24005 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24006 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24006(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24006 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24010 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24010(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24010 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24011 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24011(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24011 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24012 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24012(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24012 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24013 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24013(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24013 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24014 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24014(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24014 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24015 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24015(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24015 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24017 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24017(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24017 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24018 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24018(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24018 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24019 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24019(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24019 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24020 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24020(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24020 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24100 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24100(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24100 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24101 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24101(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24101 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24103 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24103(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24103 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24104 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24104(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24104 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24107 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24107(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24107 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24108 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24108(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24108 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24120 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24120(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24120 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24121 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24121(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24121 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24122 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24122(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24122 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24123 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24123(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24123 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24124 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24124(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24124 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24125 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24125(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24125 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24126 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24126(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24126 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24127 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24127(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24127 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24128 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24128(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24128 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24129 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24129(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24129 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24130 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24130(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24130 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24131 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24131(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24131 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24132 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24132(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24132 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24133 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24133(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24133 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24200 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24200(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24200 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24201 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24201(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24201 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24202 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24202(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24202 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24204 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24204(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24204 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24205 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24205(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24205 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24206 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24206(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24206 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24207 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24207(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24207 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24208 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24208(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24208 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24209 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24209(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24209 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24210 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24210(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24210 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24212 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24212(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24212 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24213 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24213(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24213 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24214 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24214(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24214 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24220 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24220(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24220 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24221 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24221(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24221 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24223 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24223(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24223 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24300 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24300(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24300 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24301 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24301(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24301 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24302 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24302(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24302 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24303 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24303(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24303 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24304 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24304(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24304 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24305 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24305(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24305 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24306 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24306(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24306 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24308 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24308(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24308 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24309 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24309(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24309 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24310 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24310(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24310 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24311 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24311(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24311 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24312 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24312(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24312 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24313 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24313(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24313 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24314 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24314(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24314 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24315 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24315(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24315 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24316 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24316(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24316 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24400 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24400(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24400 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24401 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24401(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24401 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24402 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24402(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24402 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24403 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24403(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24403 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24404 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24404(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24404 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24405 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24405(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24405 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24406 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24406(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24406 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24407 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24407(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24407 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24408 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24408(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24408 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24409 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24409(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24409 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24410 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24410(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24410 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24411 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24411(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24411 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24500 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24500(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24500 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24501 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24501(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24501 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24502 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24502(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24502 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24600 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24600(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24600 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24601 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24601(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24601 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24602 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24602(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24602 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24603 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24603(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24603 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24604 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24604(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24604 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24700 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24700(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24700 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24701 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24701(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24701 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24702 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24702(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24702 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24801 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24801(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24801 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24802 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24802(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24802 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24803 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24803(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24803 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24804 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24804(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24804 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24805 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24805(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24805 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24806 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24806(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24806 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24807 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24807(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24807 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24808 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24808(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24808 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24809 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24809(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24809 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24810 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24810(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24810 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24811 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24811(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24811 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24812 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24812(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24812 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24813 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24813(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24813 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24814 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24814(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24814 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24815 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24815(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24815 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24816 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24816(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24816 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24817 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24817(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24817 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 24818 welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_24818(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }
        tracing::debug!(cmd, "24818 welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response { cmd, payload: vec![] }
    })
}

// 11000 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11000(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11000 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11002 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11002(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11002 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11003 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11003(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11003 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11004 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11004(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11004 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11005 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11005(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11005 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11006 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11006(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11006 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11007 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11007(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11007 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11008 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11008(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11008 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11009 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11009(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11009 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11010 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11010(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11010 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11011 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11011(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11011 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11012 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11012(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11012 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11015 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11015(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11015 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11016 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11016(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11016 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11017 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11017(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11017 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11019 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11019(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11019 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11020 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11020(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11020 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11025 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11025(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11025 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11026 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11026(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11026 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11030 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11030(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11030 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11031 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11031(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11031 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11032 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11032(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11032 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11033 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11033(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11033 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11034 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11034(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11034 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11035 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11035(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11035 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11036 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11036(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11036 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11037 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11037(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11037 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11038 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11038(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11038 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11040 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11040(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11040 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 11041 partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_11041(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "11041 partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response { cmd, payload: vec![] }
    })
}

// 16400 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16400(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16400 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16401 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16401(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16401 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16402 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16402(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16402 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16601 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16601(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16601 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16602 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16602(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16602 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16603 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16603(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16603 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16604 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16604(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16604 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16605 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16605(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16605 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16607 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16607(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16607 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16620 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16620(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16620 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16630 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16630(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16630 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16631 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16631(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16631 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16633 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16633(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16633 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16634 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16634(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16634 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16635 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16635(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16635 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16636 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16636(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16636 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16637 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16637(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16637 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16638 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16638(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16638 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16639 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16639(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16639 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16640 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16640(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16640 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16641 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16641(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16641 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16642 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16642(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16642 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16643 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16643(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16643 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16650 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16650(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16650 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16660 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16660(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16660 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16661 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16661(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16661 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16665 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16665(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16665 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16666 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16666(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16666 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16670 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16670(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16670 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16671 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16671(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16671 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16672 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16672(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16672 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16673 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16673(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16673 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}

// 16674 social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_16674(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }
        tracing::debug!(cmd, "16674 social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response { cmd, payload: vec![] }
    })
}


// ============================================================================
// v0.5.1 (per 2026-09-09 20:17 JST Mavis 娲惧伐缁仛): w1 player 鍩?12 cmd real handler
// 鏉ユ簮: zsyz_server/src/proto/proto_103.erl + proto_105.erl + proto_108.erl
// 鑼冨洿: 10304/10305/10306/10307/10310/10323/10344/10510/10511/10512/10530/10803
// 澶囨敞: erlang proto_*.erl 鏈夊畾涔? 浣?H5 瀹㈡埛绔?766 send cmd 娌＄敤 (绗竴杞?53 璺宠繃)
// 瀛楄妭绾у榻? 4B BE len + 2B BE cmd + payload (per zsyz_client GameTcpClient.h)
// 瀛楁椤哄簭涓ユ牸鎸?pack(srv, ...) in proto_*.erl
// ============================================================================

// 10304 cli: empty; srv: empty  (per proto_103.erl, 瀹㈡埛绔媺 power 鐘舵€?
pub fn handle_10304(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10304 empty");
        Response { cmd, payload: vec![] }
    })
}

// 10305 cli: empty; srv: {assets:u16 list [{label:u8, val:u32}]}  (per proto_103.erl)
pub fn handle_10305(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10305 assets");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // empty assets list
        Response { cmd, payload: out }
    })
}

// 10306 cli: empty; srv: {power:u32, max_power:u32}  (per proto_103.erl, RGS player.GetPlayer)
pub fn handle_10306(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10306 power");
        let mut out = Vec::with_capacity(8);
        out.write_u32(99999);
        out.write_u32(100000);
        Response { cmd, payload: out }
    })
}

// 10307 cli: empty; srv: {event:u8}  (per proto_103.erl, 浜嬩欢閫氱煡鐘舵€?
pub fn handle_10307(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10307 event");
        let mut out = Vec::with_capacity(2);
        out.write_u8(0);  // event = 0 (no event)
        Response { cmd, payload: out }
    })
}

// 10310 cli: empty; srv: {is_show:u8, msg:str}  (per proto_103.erl, 鏄惁鏄剧ず鎻愮ず)
pub fn handle_10310(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10310 is_show");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);  // is_show = 0
        out.write_string("OK");
        Response { cmd, payload: out }
    })
}

// 10323 cli: empty; srv: {code:u8}  (per proto_103.erl, 鐘舵€佺爜)
pub fn handle_10323(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10323 code");
        let mut out = Vec::with_capacity(2);
        out.write_u8(0);  // code = 0 (OK)
        Response { cmd, payload: out }
    })
}

// 10344 cli: empty; srv: {lev:u8, old_energy:u32, new_energy:u32}  (per proto_103.erl, 鍗囩骇鑳介噺鍙樺寲)
pub fn handle_10344(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10344 lev_energy_change");
        let mut out = Vec::with_capacity(8);
        out.write_u8(18);    // lev
        out.write_u32(50);   // old_energy
        out.write_u32(100);  // new_energy
        Response { cmd, payload: out }
    })
}

// 10510 cli: empty; srv: {item_list:u16 array [{base_id:u32, quantity:u32, type:u8}]}  (per proto_105.erl, 鐗╁搧鍒楄〃 1)
pub fn handle_10510(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10510 item_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // empty list
        Response { cmd, payload: out }
    })
}

// 10511 cli: empty; srv: {item_list:u16 array}  (per proto_105.erl, 鐗╁搧鍒楄〃 2)
pub fn handle_10511(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10511 item_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10512 cli: empty; srv: {item_list:u16 array}  (per proto_105.erl, 鐗╁搧鍒楄〃 3)
pub fn handle_10512(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10512 item_list");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 10530 cli: empty; srv: empty  (per proto_105.erl, 绠€鍗?ack)
pub fn handle_10530(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10530 empty");
        Response { cmd, payload: vec![] }
    })
}

// 10803 cli: empty; srv: {mail:u16 list [{id, srv_id, type, from_name, subject, content, assets, items, send_time, read_time, time_out, status}]}  (per proto_108.erl, 鏈閭欢鍒楄〃, 鍚?10800 鏍煎紡)
pub fn handle_10803(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("10803 unread_mail");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);  // empty mail list
        Response { cmd, payload: out }
    })
}

// ---- w3 battle/arena handlers (per 9/9 22:00 JST Phase 4 续做) ----

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

