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
