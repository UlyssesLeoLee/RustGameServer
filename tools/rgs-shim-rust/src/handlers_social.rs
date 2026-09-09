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

// ============================================================================
// w4 social 续做 79 handler (per 2026-09-09 20:17 JST Mavis 派工 shim 续做)
// 来源: zsyz_server/src/proto/proto_130.erl + proto_133.erl + proto_134.erl
//       proto_135.erl + proto_166.erl
// 字节级对齐 erlang pack(srv, ...): BE + len:u32+bytes 字符串 + 字段顺序严格
// 风格跟第一轮 14 POC 一样 (per 9/9 19:30 JST POC template)
//
// 注: 大多数 cmd 走 stub 路径 (code:u8(0) + msg:str or empty), 不调 RGS.
//     完整 RGS 集成留给后续 v0.4.2+ 派工 (per 9/9 19:25 JST plan B).
//     79 cmd 拆: proto_130=8 + proto_133=19 + proto_134=16 + proto_135=24 + proto_166=12
// ============================================================================

// proto_130.erl — 副本/关卡 (dungeon) — cmd 13000-13099 (续做 w4)
// 13001 cli: empty; srv: empty
pub fn handle_13001_dungeon_enter(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13001 dungeon_enter (w4 social stub)");
        // empty (per proto: empty)
        let out = Vec::new();
        Response { cmd, payload: out }
    })
}

// 13002 cli: empty; srv: code:u8,msg:str
pub fn handle_13002_dungeon_op(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13002 dungeon_op (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13003 cli: empty; srv: code:u8,msg:str
pub fn handle_13003_dungeon_op2(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13003 dungeon_op2 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13004 cli: empty; srv: code:u8,msg:str
pub fn handle_13004_dungeon_op3(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13004 dungeon_op3 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13007 cli: (server-push); srv: type:u8,dun_id:u32,time:u32,old_lev:u8,new_lev:u8,items[]
pub fn handle_13007_dungeon_result(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13007 dungeon_result (w4 social stub)");
        // empty list[] (per proto: type:u8,dun_id:u32,time:u32,old_lev:u8,new_lev:u8,items[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13008 cli: empty; srv: id:u32,status:u8
pub fn handle_13008_dungeon_status(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13008 dungeon_status (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: id:u32,status:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13009 cli: id:u32; srv: code:u8,msg:str
pub fn handle_13009_dungeon_reset(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13009 dungeon_reset (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13010 cli: (server-push); srv: dun_id:u32,chapter_id:u8
pub fn handle_13010_dungeon_chapter(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13010 dungeon_chapter (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: dun_id:u32,chapter_id:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// proto_133.erl — 好友 (friend) — cmd 13300-13399 (续做 w4)
// 13301 cli: (server-push); srv: rid:u32,srv_id:str,login_time:u32,is_online:u8,login_out_time:u32
pub fn handle_13301_friend_login(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13301 friend_login (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: rid:u32,srv_id:str,login_time:u32,is_online:u8,login_out_time:u32)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13302 cli: (server-push); srv: (friend_tmp record)
pub fn handle_13302_friend_tmp(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13302 friend_tmp (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: (friend_tmp record))
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13304 cli: (server-push); srv: (friend_req record)
pub fn handle_13304_friend_req(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13304 friend_req (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: (friend_req record))
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13305 cli: rid:u32,srv_id:str,agreed:u8; srv: rid:u32,srv_id:str,code:u8,msg:str
pub fn handle_13305_friend_agree(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13305 friend_agree (w4 social stub)");
        // code:u8(0) + msg:str (per proto: rid:u32,srv_id:str,code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13306 cli: role_ids[]; srv: code:u8,msg:str
pub fn handle_13306_friend_batch_add(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13306 friend_batch_add (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13307 cli: rid:u32,srv_id:str; srv: code:u8,msg:str,rid:u32,srv_id:str
pub fn handle_13307_friend_delete(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13307 friend_delete (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: code:u8,msg:str,rid:u32,srv_id:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13308 cli: (server-push); srv: rid:u32,srv_id:str
pub fn handle_13308_friend_delete_back(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13308 friend_delete_back (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: rid:u32,srv_id:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13309 cli: role_ids[]; srv: role_ids[]
pub fn handle_13309_friend_batch_delete(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13309 friend_batch_delete (w4 social stub)");
        // empty list[] (per proto: role_ids[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13310 cli: (server-push); srv: (friend_tmp record)
pub fn handle_13310_friend_tmp_push(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13310 friend_tmp_push (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: (friend_tmp record))
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13312 cli: empty; srv: code:u8,msg:str
pub fn handle_13312_friend_search(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13312 friend_search (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13314 cli: name:str; srv: role_list[]
pub fn handle_13314_friend_recommend(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13314 friend_recommend (w4 social stub)");
        // empty list[] (per proto: role_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13316 cli: rid:u32,srv_id:str,code:u32; srv: rid:u32,srv_id:str,present_count:u16,draw_count:u16,code:u8,msg:str,type:u8,is_present:u8,is_draw:u8
pub fn handle_13316_friend_present(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13316 friend_present (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: rid:u32,srv_id:str,present_count:u16,draw_count:u16,code:u8,msg:str,type:u8,is_present:u8,is_draw:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13317 cli: code:u32,list[]; srv: code:u8,msg:str,type:u8,list[]
pub fn handle_13317_friend_present_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13317 friend_present_list (w4 social stub)");
        // type:u8(0) + list[] (per proto: code:u8,msg:str,type:u8,list[])
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13320 cli: empty; srv: recommend_list[]
pub fn handle_13320_friend_recommend_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13320 friend_recommend_list (w4 social stub)");
        // empty list[] (per proto: recommend_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13330 cli: empty; srv: black_list[]
pub fn handle_13330_black_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13330 black_list (w4 social stub)");
        // empty list[] (per proto: black_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13331 cli: (server-push); srv: type:u8,black_list[]
pub fn handle_13331_black_list_v2(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13331 black_list_v2 (w4 social stub)");
        // empty list[] (per proto: type:u8,black_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13332 cli: rid:u32,srv_id:str; srv: code:u8,msg:str,rid:u32,srv_id:str
pub fn handle_13332_black_add(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13332 black_add (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: code:u8,msg:str,rid:u32,srv_id:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13333 cli: rid:u32,srv_id:str; srv: code:u8,msg:str,rid:u32,srv_id:str
pub fn handle_13333_black_remove(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13333 black_remove (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: code:u8,msg:str,rid:u32,srv_id:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13334 cli: (no cli); srv: (no srv)
pub fn handle_13334_black_query(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13334 black_query (w4 social stub)");
        // empty (per proto: (no srv))
        let out = Vec::new();
        Response { cmd, payload: out }
    })
}

// proto_134.erl — 兑换 (exchange/mall) — cmd 13400-13499 (续做 w4)
// 13402 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13402_exchange_x2(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13402 exchange_x2 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13403 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13403_exchange_x3(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13403 exchange_x3 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13404 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13404_exchange_x4(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13404 exchange_x4 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13405 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13405_exchange_x5(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13405 exchange_x5 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13407 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13407_exchange_x7(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13407 exchange_x7 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13409 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13409_exchange_x9(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13409 exchange_x9 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13410 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13410_exchange_x10(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13410 exchange_x10 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13411 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13411_exchange_x11(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13411 exchange_x11 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13412 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13412_exchange_x12(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13412 exchange_x12 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13413 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13413_exchange_x13(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13413 exchange_x13 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13414 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13414_exchange_x14(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13414 exchange_x14 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13415 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13415_exchange_x15(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13415 exchange_x15 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13416 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13416_exchange_x16(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13416 exchange_x16 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13417 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13417_exchange_x17(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13417 exchange_x17 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13418 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13418_exchange_x18(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13418 exchange_x18 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13419 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13419_exchange_x19(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13419 exchange_x19 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// proto_135.erl — 公会 (guild) — cmd 13500-13599 (续做 w4)
// 13501 cli: page:u16,flag:u8,num:u16,name:str; srv: page:u16,flag:u8,num:u16,page_total:u16,all_count:u32,name:str,guilds[]
pub fn handle_13501_guild_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13501 guild_list (w4 social stub)");
        // empty list[] (per proto: page:u16,flag:u8,num:u16,page_total:u16,all_count:u32,name:str,guilds[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13503 cli: gid:u32,gsrv_id:str,type:u8; srv: code:u8,msg:str,gid:u32,gsrv_id:str,is_apply:u8
pub fn handle_13503_guild_apply(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13503 guild_apply (w4 social stub)");
        // code:u8 + msg:str + gid + gsrv_id + is_apply (per proto: code:u8,msg:str,gid:u32,gsrv_id:str,is_apply:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (apply)");
        out.write_u32(0);
        out.write_string("guild-1");
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// 13505 cli: type:u8,rid:u32,srv_id:str; srv: type:u8,rid:u32,srv_id:str,code:u8,msg:str
pub fn handle_13505_guild_approve(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13505 guild_approve (w4 social stub)");
        // code:u8(0) + msg:str (per proto: type:u8,rid:u32,srv_id:str,code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13507 cli: page:u8,num:u8; srv: page:u8,page_total:u8,num:u8,guids[]
pub fn handle_13507_guild_apply_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13507 guild_apply_list (w4 social stub)");
        // empty list[] (per proto: page:u8,page_total:u8,num:u8,guids[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13513 cli: rid:u32,srv_id:str; srv: rid:u32,srv_id:str,code:u8,msg:str
pub fn handle_13513_guild_kick(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13513 guild_kick (w4 social stub)");
        // code:u8(0) + msg:str (per proto: rid:u32,srv_id:str,code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13514 cli: empty; srv: code:u8,msg:str
pub fn handle_13514_guild_disband(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13514 guild_disband (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13516 cli: empty; srv: code:u8,msg:str
pub fn handle_13516_guild_change_leader(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13516 guild_change_leader (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13520 cli: rid:u32,srv_id:str,position:u8; srv: rid:u32,srv_id:str,position:u8,code:u8,msg:str
pub fn handle_13520_guild_set_position(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13520 guild_set_position (w4 social stub)");
        // code:u8(0) + msg:str (per proto: rid:u32,srv_id:str,position:u8,code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13521 cli: sign:str; srv: code:u8,msg:str
pub fn handle_13521_guild_set_sign(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13521 guild_set_sign (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13522 cli: apply_type:u8,apply_lev:u8; srv: code:u8,msg:str
pub fn handle_13522_guild_set_apply(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13522 guild_set_apply (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13524 cli: type:u8; srv: code:u8,msg:str
pub fn handle_13524_guild_quit(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13524 guild_quit (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13534 cli: empty; srv: type:u8,list[]
pub fn handle_13534_guild_event(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13534 guild_event (w4 social stub)");
        // type:u8(0) + list[] (per proto: type:u8,list[])
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13535 cli: type:u8,num:u16,msg_id:u32,loss_type:u8; srv: code:u8,msg:str,day_send_num:u8
pub fn handle_13535_guild_donate(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13535 guild_donate (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str,day_send_num:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13536 cli: id:u32; srv: code:u8,msg:str,id:u32,type:u8,val:u32,day_recv_num:u8
pub fn handle_13536_guild_donate_recv(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13536 guild_donate_recv (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: code:u8,msg:str,id:u32,type:u8,val:u32,day_recv_num:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13540 cli: id:u32; srv: id:u32,type:u8,name:str,face_id:u32,avatar_bid:u32,post:u8,val:u32,list[]
pub fn handle_13540_guild_box_info(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13540 guild_box_info (w4 social stub)");
        // empty list[] (per proto: id:u32,type:u8,name:str,face_id:u32,avatar_bid:u32,post:u8,val:u32,list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13541 cli: empty; srv: code:u8,msg:str
pub fn handle_13541_guild_box_recv(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13541 guild_box_recv (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13542 cli: (server-push); srv: type:u8,members[]
pub fn handle_13542_guild_box_members(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13542 guild_box_members (w4 social stub)");
        // empty list[] (per proto: type:u8,members[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13545 cli: empty; srv: list[]
pub fn handle_13545_guild_log(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13545 guild_log (w4 social stub)");
        // empty list[] (per proto: list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 13558 cli: empty; srv: code:u8,msg:str
pub fn handle_13558_guild_war(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13558 guild_war (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13565 cli: empty; srv: code:u8,msg:str
pub fn handle_13565_guild_war2(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13565 guild_war2 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13568 cli: name:str; srv: code:u8,msg:str
pub fn handle_13568_guild_search(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13568 guild_search (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13573 cli: empty; srv: code:u8
pub fn handle_13573_guild_status(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13573 guild_status (w4 social stub)");
        // code:u8(0) (per proto: code:u8)
        let mut out = Vec::with_capacity(4);
        out.write_u8(0);
        Response { cmd, payload: out }
    })
}

// 13574 cli: box_id:u32; srv: code:u8,msg:str,box_id:u32
pub fn handle_13574_guild_box_open(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13574 guild_box_open (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str,box_id:u32)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 13575 cli: empty; srv: donate_exp:u32
pub fn handle_13575_guild_donate_exp(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("13575 guild_donate_exp (w4 social stub)");
        // donate_exp:u32(0) (per proto)
        let mut out = Vec::with_capacity(4);
        out.write_u32(0);
        Response { cmd, payload: out }
    })
}

// proto_166.erl — 节日/活动 (holiday) — cmd 16600-16699 (续做 w4)
// 16601 cli: type:u8; srv: holiday_list[]
pub fn handle_16601_holiday_list(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16601 holiday_list (w4 social stub)");
        // empty list[] (per proto: holiday_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16602 cli: type:u8; srv: type:u8,holiday_list[]
pub fn handle_16602_holiday_list_v2(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16602 holiday_list_v2 (w4 social stub)");
        // empty list[] (per proto: type:u8,holiday_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16603 cli: bid:u32; srv: bid:u32,remain_sec:u32,finish:u8,aim_list[],item_effect_list[],args[],client_reward[],rank_list[]
pub fn handle_16603_holiday_info(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16603 holiday_info (w4 social stub)");
        // empty list[] (per proto: bid:u32,remain_sec:u32,finish:u8,aim_list[],item_effect_list[],args[],client_reward[],rank_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16604 cli: bid:u32,aim:u32,arg:u32; srv: code:u8,msg:str
pub fn handle_16604_holiday_join(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16604 holiday_join (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 16605 cli: bid_list[]; srv: status[]
pub fn handle_16605_holiday_status(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16605 holiday_status (w4 social stub)");
        // empty list[] (per proto: status[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16606 cli: (server-push); srv: type:u8,bid:u32,can_get_num:u16
pub fn handle_16606_holiday_reward(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16606 holiday_reward (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: type:u8,bid:u32,can_get_num:u16)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 16607 cli: empty; srv: type:u8
pub fn handle_16607_holiday_type(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16607 holiday_type (w4 social stub)");
        // code:u8(0) + msg:str fallback (per proto: type:u8)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 16620 cli: id_list[]; srv: holiday_list[]
pub fn handle_16620_holiday_list_v3(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16620 holiday_list_v3 (w4 social stub)");
        // empty list[] (per proto: holiday_list[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16630 cli: empty; srv: code:u8,items[]
pub fn handle_16630_holiday_claim(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16630 holiday_claim (w4 social stub)");
        // empty list[] (per proto: code:u8,items[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16631 cli: empty; srv: code:u8,msg:str
pub fn handle_16631_holiday_claim_v2(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16631 holiday_claim_v2 (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}

// 16635 cli: empty; srv: code:u8,items[]
pub fn handle_16635_holiday_claim_v3(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16635 holiday_claim_v3 (w4 social stub)");
        // empty list[] (per proto: code:u8,items[])
        let mut out = Vec::with_capacity(4);
        out.write_u16(0);
        Response { cmd, payload: out }
    })
}

// 16636 cli: number:u32; srv: code:u8,msg:str
pub fn handle_16636_holiday_buy(
    cmd: u16, _payload: Vec<u8>, _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        tracing::info!("16636 holiday_buy (w4 social stub)");
        // code:u8(0) + msg:str (per proto: code:u8,msg:str)
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (w4 social stub)");
        Response { cmd, payload: out }
    })
}
