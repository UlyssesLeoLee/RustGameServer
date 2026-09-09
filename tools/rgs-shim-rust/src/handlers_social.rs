// Social domain handlers (per 2026-09-09 19:30 JST Mavis 派工 w4)
// 来源: zsyz_server/src/proto/proto_130.erl (dungeon) + proto_133.erl (friend)
//       proto_134.erl (exchange/mall) + proto_135.erl (guild)
//       proto_166.erl (mail) + proto_168.erl (cross-server)
//
// v0.4.1 (per 9/9 19:32 JST Ulysses 拍板选项 A): w4 POC 14 handler
// 完整 93 cmd 扩需 1-2 周 (per 9/9 19:25 JST 拍板 brief 估算 4-5 天/200 cmd)
//
// 字节级对齐 erlang (per 9/9 13:45 JST 拍板): BE + len:u32+bytes 字符串 + 字段顺序严格按 pack(srv, ...)

use crate::frame::{BeRead, BeWrite};
use crate::handlers::Response;
use crate::rgs::RgsClient;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

// ============================================================================
// proto_130.erl — 副本/关卡 (dungeon) — cmd 13000-13040
// ============================================================================

// 13000 cli: empty; srv: {mode:u8, chapter_id:u8, dun_id:u32, status:u8, cool_time:u32, max_dun_id:u32, auto_num:u8, auto_num_max:u8, mode_list[]}
//   简化: 返回空 mode_list
pub fn handle_13000_dungeon_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("social", "HealthCheck", serde_json::json!({})).await;
        tracing::info!("13000 dungeon_list (w4 POC)");
        let mut out = Vec::with_capacity(24);
        out.write_u8(1); out.write_u8(1); out.write_u32(10001);
        out.write_u8(0); out.write_u32(0); out.write_u32(10099);
        out.write_u8(0); out.write_u8(10);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13005 cli: {dun_id:u32, num:u16}; srv: {code:u8, msg:str, dun_id:u32, num:u16, items[{bid:u32, num:u32}]}
pub fn handle_13005_dungeon_battle(
    cmd: u16, payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (dun_id, num) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..]; (p.read_u32(), p.read_u16())
        } else { (0u32, 1u16) };
        tracing::info!(dun_id, num, "13005 dungeon_battle (w4 POC)");
        let mut out = Vec::with_capacity(64);
        out.write_u8(0); out.write_string("OK (dungeon battle w4 POC)");
        out.write_u32(dun_id); out.write_u16(num);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13006 cli: empty; srv: {fast_combat_num:u8, fast_combat_max:u8, auto_num:u8, auto_num_max:u8}
pub fn handle_13006_dungeon_count(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13006 dungeon_count (w4 POC)");
        let mut out = Vec::with_capacity(8);
        out.write_u8(0); out.write_u8(10); out.write_u8(0); out.write_u8(10);
        Response { cmd, payload: out }
    })
}

// 13011 cli: empty; srv: {buff_list[{bid:u32, end_time:u32}]}
pub fn handle_13011_buff_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13011 buff_list (w4 POC)");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_133.erl — 好友 (friend) — cmd 13300-13334
// ============================================================================

// 13300 cli: empty; srv: {present_count:u16, draw_count:u16, draw_all:u16, friend_list[]}
pub fn handle_13300_friend_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13300 friend_list (w4 POC)");
        let mut out = Vec::with_capacity(16);
        out.write_u16(0); out.write_u16(0); out.write_u16(0); out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13303 cli: {rid:u32, srv_id:str}; srv: {code:u8, msg:str}
pub fn handle_13303_add_friend(
    cmd: u16, payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..]; (p.read_u32(), p.read_string())
        } else { (0u32, String::new()) };
        tracing::info!(rid = format!("0x{:08x}", rid), %srv_id, "13303 add_friend (w4 POC)");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0); out.write_string("OK (add_friend w4 POC)");
        Response { cmd, payload: out }
    })
}

// 13311 cli: empty; srv: {friend_req_list[]}
pub fn handle_13311_friend_req_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13311 friend_req_list (w4 POC)");
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13315 cli: {rid:u32, srv_id:str}; srv: {code:u8}
pub fn handle_13315_delete_friend(
    cmd: u16, payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..]; (p.read_u32(), p.read_string())
        } else { (0u32, String::new()) };
        tracing::info!(rid = format!("0x{:08x}", rid), %srv_id, "13315 delete_friend (w4 POC)");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_135.erl — 公会 (guild) — cmd 13500-13576
// ============================================================================

// 13500 cli: {name:str, sign:str, apply_type:u8, apply_lev:u8}; srv: {code:u8, msg:str}
pub fn handle_13500_create_guild(
    cmd: u16, payload: Vec<u8>, rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (name, sign, apply_type, apply_lev) = if payload.len() >= 10 {
            let mut p: &[u8] = &payload[..];
            (p.read_string(), p.read_string(), p.read_u8(), p.read_u8())
        } else { (String::from("MavisGuild"), String::from("w4 POC"), 0u8, 1u8) };
        tracing::info!(%name, %sign, apply_type, apply_lev, "13500 create_guild (w4 POC)");
        let _ = rgs.call("social", "HealthCheck", serde_json::json!({})).await;
        let mut out = Vec::with_capacity(64);
        out.write_u8(0); out.write_string("OK (create_guild w4 POC)");
        Response { cmd, payload: out }
    })
}

// 13518 cli: empty; srv: {gid:u32, gsrv_id:str, name:str, lev:u8, members_num:u8, members_max:u8,
//   leader_name:str, rid:u32, srv_id:str, sign:str, exp:u32, day_exp:u32, apply_type:u8,
//   apply_lev:u8, recruit_num:u8, rank_idx:i16}
// 调 RGS social.GetGuild 拿真数据
pub fn handle_13518_guild_info(
    cmd: u16, _payload: Vec<u8>, rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "GetGuild", serde_json::json!({
            "id": "22222222-2222-2222-2222-222222222222"
        })).await;
        let guild = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let name = guild.get("display_name").and_then(|v| v.as_str()).unwrap_or("MavisGuild").to_string();
        tracing::info!(%name, "13518 guild_info (w4 POC, RGS social.GetGuild)");
        let mut out = Vec::with_capacity(256);
        out.write_u32(0x22222222);
        out.write_string("guild-1");
        out.write_string(&name);
        out.write_u8(1); out.write_u8(1); out.write_u8(50);
        out.write_string("MavisHero");
        out.write_u32(0x11111111);
        out.write_string("rgs-uat-1");
        out.write_string("w4 POC guild");
        out.write_u32(0); out.write_u32(0);
        out.write_u8(0); out.write_u8(1); out.write_u8(0);
        out.write_i16(0);
        Response { cmd, payload: out }
    })
}

// 13519 cli: empty; srv: {members[]}
//   简化: 单成员 (自己)
pub fn handle_13519_guild_members(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13519 guild_members (w4 POC)");
        let mut out = Vec::with_capacity(128);
        out.write_u16(1);
        out.write_u32(0x11111111);
        out.write_string("rgs-uat-1");
        out.write_string("MavisHero");
        out.write_u8(18); out.write_u32(0);
        out.write_u8(1); out.write_u8(1); out.write_u8(0); out.write_u32(99999);
        out.write_u32(now_unix()); out.write_u32(now_unix());
        out.write_u32(0); out.write_u32(0); out.write_u32(0); out.write_u8(1);
        Response { cmd, payload: out }
    })
}

// 13523 cli: empty; srv: {donate_list[], boxes[], donate_exp:u32, day_send_num:u8, day_recv_num:u8}
pub fn handle_13523_donate_info(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13523 donate_info (w4 POC)");
        let mut out = Vec::with_capacity(16);
        out.write_u16(0); out.write_u16(0); out.write_u32(0); out.write_u8(0); out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_134.erl — 兑换 (exchange/mall) — cmd 13401-13420
// ============================================================================

// 13401 cli: {type:u8}; srv: {code:u8, msg:str, type:u8, is_half:u8, item_list[]}
pub fn handle_13401_exchange_list(
    cmd: u16, payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let ptype = if !payload.is_empty() { let mut p: &[u8] = &payload[..]; p.read_u8() } else { 0u8 };
        tracing::info!(ptype, "13401 exchange_list (w4 POC)");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0); out.write_string("OK (exchange_list w4 POC)");
        out.write_u8(ptype); out.write_u8(0); out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13408 cli: {type:u8}; srv: {code:u8}
pub fn handle_13408_exchange_action(
    cmd: u16, payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let ptype = if !payload.is_empty() { let mut p: &[u8] = &payload[..]; p.read_u8() } else { 0u8 };
        tracing::info!(ptype, "13408 exchange_action (w4 POC)");
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}
