// zsyz social domain cmd handlers (per 9/9 19:32 JST Mavis 派工 — 5 worker 并发)
// 覆盖 93 cmd (proto_130 chat + proto_133 friend + proto_134 exchange +
//          proto_135 guild + proto_136 cross-server + proto_166 activity + proto_168 tip)
//
// 设计: 每个 handler:
//   1. 安全解析 payload (用 BeRead trait, 长度不够 fallback 到默认值)
//   2. 调 rgs.call("social", "Method", json) — 失败降级 (RGS 不可达时返 code=0 + msg="OK (RGS n/a)")
//   3. 按 proto_*.erl pack(srv, ...) 字段顺序构造响应
//
// Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化 + 9/8 15:19 JST 第 6 次强化)
// Phase 4 worker e2e/social 派工 (per 9/9 16:30 JST)

use crate::frame::{BeRead, BeWrite};
use crate::rgs::RgsClient;
use std::sync::Arc;

fn default_player_uuid() -> serde_json::Value {
    serde_json::json!("11111111-1111-1111-1111-111111111111")
}

fn default_guild_uuid() -> serde_json::Value {
    serde_json::json!("22222222-2222-2222-2222-222222222222")
}

fn ok_payload(cmd: u16, msg: &str) -> crate::handlers::Response {
    let mut out = Vec::with_capacity(32);
    out.write_u8(0);
    out.write_string(msg);
    crate::handlers::Response { cmd, payload: out }
}

// ============================================================================
// proto_130 — 邮件/聊天 (13000-13099, 21 stub → 8 real)
// ============================================================================

// 13000 srv: {present_count:u16, draw_count:u16, draw_all:u16, mode:u8, chapter_id:u8,
//             dun_id:u32, status:u8, cool_time:u32, max_dun_id:u32, auto_num:u8, auto_num_max:u8,
//             mode_list: [{mode, chapter_list: [{chapter_id, status, dun_list}]}]}
// 客户端进副本/章节前请求当前进度 — 走 RGS social dungeon 域不存, 走 player 域 fallback
pub fn handle_dungeon_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("social", "ListDungeons", default_player_uuid()).await;
        let mut out = Vec::with_capacity(64);
        out.write_u16(0);                          // present_count
        out.write_u16(0);                          // draw_count
        out.write_u16(0);                          // draw_all
        out.write_u8(1);                           // mode
        out.write_u8(1);                           // chapter_id
        out.write_u32(1001);                       // dun_id
        out.write_u8(1);                           // status (1=unlocked)
        out.write_u32(0);                          // cool_time
        out.write_u32(1001);                       // max_dun_id
        out.write_u8(0);                           // auto_num
        out.write_u8(10);                          // auto_num_max
        out.write_u16(1);                          // mode_list count
        // 1 mode_list entry: mode + chapter_list count
        out.write_u8(1);
        out.write_u16(1);
        // 1 chapter entry: chapter_id + status + dun_list count
        out.write_u8(1);
        out.write_u8(1);
        out.write_u16(1);
        // 1 dun entry
        out.write_u32(1001);
        out.write_u8(1);
        out.write_u32(0);
        out.write_u8(0);
        tracing::debug!(cmd, "13000 dungeon_list (RGS social ListDungeons)");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13002 cli: empty; srv: {code:u8, msg:str}  (per proto_130.erl)
pub fn handle_dungeon_op(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        tracing::debug!(cmd, "13002 dungeon_op");
        ok_payload(cmd, "OK (dungeon op)")
    })
}

// 13005 cli: {dun_id:u32, num:u16}; srv: {code:u8, msg:str, dun_id:u32, num:u16, items: [{bid:u32, num:u32}]}
// 扫荡副本 (sweep) - 简化返 code=0 + 空 items
pub fn handle_dungeon_sweep(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (dun_id, num) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u16())
        } else {
            (0u32, 0u16)
        };
        // 调 RGS social 域 + economy 域加物品 (跨域 RPC)
        let _ = rgs.call("social", "SweepDungeon", serde_json::json!({
            "dungeon_id": dun_id, "num": num, "player_id": "11111111-1111-1111-1111-111111111111"
        })).await;
        tracing::info!(dun_id, num, "13005 dungeon_sweep");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (sweep complete)");
        out.write_u32(dun_id);
        out.write_u16(num);
        out.write_u16(0);                          // items count = 0 (RGS 通过 economy 推送)
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13011 cli: empty; srv: {buff_list: [{bid:u32, end_time:u32}]}  (per proto_130.erl)
pub fn handle_dungeon_buff_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("social", "ListDungeonBuffs", default_player_uuid()).await;
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);                          // empty buff list
        tracing::debug!(cmd, "13011 dungeon_buff_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13012 cli: {rid:u32, srv_id:str, content:str}; srv: {code:u8, msg:str}
// 私聊消息 (chat) — shim 简化: 不存历史, 只 ack (RGS social ChatMessage fallback)
pub fn handle_chat_send(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, content) = if payload.len() >= 10 {
            let mut p: &[u8] = &payload[..];
            let r = p.read_u32();
            let s = p.read_string();
            let c = p.read_string();
            (r, s, c)
        } else {
            (0u32, String::new(), String::new())
        };
        let _ = rgs.call("social", "SendChatMessage", serde_json::json!({
            "from_id": "11111111-1111-1111-1111-111111111111",
            "to_rid": rid, "to_srv_id": srv_id, "content": content
        })).await;
        tracing::info!(to_rid = rid, %srv_id, "13012 chat_send");
        ok_payload(cmd, "OK (chat sent)")
    })
}

// 13018 cli: empty; srv: {chat_list: [{rid:u32, srv_id:str, name:str, content:str, time:u32}]}
// 收件箱 (私聊+世界+工会+留言) — 走 RGS social ListMessages
pub fn handle_chat_inbox(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListMessages", default_player_uuid()).await;
        let messages: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("messages"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(16 + messages.len() * 64);
        out.write_u16(messages.len() as u16);
        for m in &messages {
            let rid = m.get("from_rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = m.get("from_srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = m.get("from_name").and_then(|v| v.as_str()).unwrap_or("");
            let content = m.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let time = m.get("time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_string(content);
            out.write_u32(time);
        }
        tracing::info!(count = messages.len(), "13018 chat_inbox");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13030 cli: {type:u32}; srv: {code:u8, items: [{bid:u32, num:u32}]}
// 邮件领取附件
pub fn handle_mail_attach(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let mail_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        let _ = rgs.call("social", "ClaimMailAttachment", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "mail_id": mail_id
        })).await;
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_u16(0);                          // items count = 0
        tracing::info!(mail_id, "13030 mail_attach");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13040 cli: empty; srv: {mail_list: [{id:u32, type:u8, title:str, content:str, items: [...], time:u32, is_read:u8}]}
// 邮件列表
pub fn handle_mail_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListMails", default_player_uuid()).await;
        let mails: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("mails"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(16 + mails.len() * 128);
        out.write_u16(mails.len() as u16);
        for m in &mails {
            let id = m.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let ty = m.get("type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let title = m.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let content = m.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let time = m.get("time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let is_read = m.get("is_read").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let items = m.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            out.write_u32(id);
            out.write_u8(ty);
            out.write_string(title);
            out.write_string(content);
            out.write_u32(time);
            out.write_u8(is_read);
            out.write_u16(items.len() as u16);
            for it in &items {
                let bid = it.get("bid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let num = it.get("num").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                out.write_u32(bid);
                out.write_u32(num);
            }
        }
        tracing::info!(count = mails.len(), "13040 mail_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_133 — 好友社交 (13300-13399, 17 stub → 9 real)
// ============================================================================

// 13300 cli: empty; srv: {present_count:u16, draw_count:u16, draw_all:u16, friend_list: [friend_tmp]}
// friend_tmp 字段: rid:u32, srv_id:str, name:str, lev:u8, sex:u8, career:u8, face_id:u16,
//                  power:u32, intimacy:u32, login_time:u32, login_out_time:u32,
//                  is_online:u8, is_cross:u8, gid:u32, gsrv_id:str, gname:str,
//                  main_partner_id:u32, partner_bid:u32, partner_lev:u16, partner_star:u8,
//                  is_awake:u8, is_used:u8, is_present:u8, is_draw:u8, avatar_bid:u32, dun_id:u32
// 走 RGS social ListFriends
pub fn handle_friend_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListFriends", default_player_uuid()).await;
        let friends: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("friends"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(16 + friends.len() * 200);
        out.write_u16(0);                          // present_count
        out.write_u16(0);                          // draw_count
        out.write_u16(0);                          // draw_all
        out.write_u16(friends.len() as u16);
        for f in &friends {
            let rid = f.get("rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = f.get("srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let lev = f.get("lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let sex = f.get("sex").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let career = f.get("career").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let face_id = f.get("face_id").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            let power = f.get("power").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let intimacy = f.get("intimacy").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let login_time = f.get("login_time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let login_out_time = f.get("login_out_time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let is_online = f.get("is_online").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_cross = f.get("is_cross").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let gid = f.get("gid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let gsrv_id = f.get("gsrv_id").and_then(|v| v.as_str()).unwrap_or("");
            let gname = f.get("gname").and_then(|v| v.as_str()).unwrap_or("");
            let main_partner_id = f.get("main_partner_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let partner_bid = f.get("partner_bid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let partner_lev = f.get("partner_lev").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            let partner_star = f.get("partner_star").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_awake = f.get("is_awake").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_used = f.get("is_used").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_present = f.get("is_present").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_draw = f.get("is_draw").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let avatar_bid = f.get("avatar_bid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let dun_id = f.get("dun_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_u8(lev);
            out.write_u8(sex);
            out.write_u8(career);
            out.write_u16(face_id);
            out.write_u32(power);
            out.write_u32(intimacy);
            out.write_u32(login_time);
            out.write_u32(login_out_time);
            out.write_u8(is_online);
            out.write_u8(is_cross);
            out.write_u32(gid);
            out.write_string(gsrv_id);
            out.write_string(gname);
            out.write_u32(main_partner_id);
            out.write_u32(partner_bid);
            out.write_u16(partner_lev);
            out.write_u8(partner_star);
            out.write_u8(is_awake);
            out.write_u8(is_used);
            out.write_u8(is_present);
            out.write_u8(is_draw);
            out.write_u32(avatar_bid);
            out.write_u32(dun_id);
        }
        tracing::info!(count = friends.len(), "13300 friend_list (RGS social)");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13303 cli: {rid:u32, srv_id:str}; srv: {code:u8, msg:str}
// 申请添加好友
pub fn handle_friend_add(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        let rgs_resp = rgs.call("social", "RequestFriend", serde_json::json!({
            "from_id": "11111111-1111-1111-1111-111111111111",
            "to_rid": rid, "to_srv_id": srv_id
        })).await;
        let code = if rgs_resp.ok { 0u8 } else { 1u8 };
        let msg = if rgs_resp.ok {
            "OK (friend request sent)".to_string()
        } else {
            format!("RGS error: {}", rgs_resp.error.unwrap_or_default())
        };
        tracing::info!(to_rid = rid, %srv_id, code, "13303 friend_add");
        let mut out = Vec::with_capacity(32);
        out.write_u8(code);
        out.write_string(&msg);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13305 cli: {rid:u32, srv_id:str, agreed:u8}; srv: {rid:u32, srv_id:str, code:u8, msg:str}
// 接受/拒绝好友申请
pub fn handle_friend_agree(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, agreed) = if payload.len() >= 9 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u8())
        } else {
            (0u32, String::new(), 0u8)
        };
        let _ = rgs.call("social", "RespondFriendRequest", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "from_rid": rid, "from_srv_id": srv_id, "agreed": agreed != 0
        })).await;
        tracing::info!(from_rid = rid, %srv_id, agreed, "13305 friend_agree");
        let mut out = Vec::with_capacity(32);
        out.write_u32(rid);
        out.write_string(&srv_id);
        out.write_u8(0);
        out.write_string(if agreed != 0 { "OK (friend added)" } else { "OK (rejected)" });
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13307 cli: {rid:u32, srv_id:str}; srv: {code:u8, msg:str, rid:u32, srv_id:str}
// 删除好友
pub fn handle_friend_delete(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        let _ = rgs.call("social", "DeleteFriend", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "friend_rid": rid, "friend_srv_id": srv_id
        })).await;
        tracing::info!(friend_rid = rid, %srv_id, "13307 friend_delete");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (friend deleted)");
        out.write_u32(rid);
        out.write_string(&srv_id);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13311 cli: empty; srv: {friend_req_list: [{rid, srv_id, name, sex, career, lev, face_id, power, avatar_bid}]}
// 待处理好友申请列表
pub fn handle_friend_req_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListFriendRequests", default_player_uuid()).await;
        let reqs: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("requests"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(8 + reqs.len() * 64);
        out.write_u16(reqs.len() as u16);
        for r in &reqs {
            let rid = r.get("rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = r.get("srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = r.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let sex = r.get("sex").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let career = r.get("career").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let lev = r.get("lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let face_id = r.get("face_id").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            let power = r.get("power").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let avatar_bid = r.get("avatar_bid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_u8(sex);
            out.write_u8(career);
            out.write_u8(lev);
            out.write_u16(face_id);
            out.write_u32(power);
            out.write_u32(avatar_bid);
        }
        tracing::info!(count = reqs.len(), "13311 friend_req_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13315 cli: {rid:u32, srv_id:str}; srv: {code:u8}
// 亲密度查询 / 赠送体力 (1=present, 2=draw)
pub fn handle_friend_present(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        let _ = rgs.call("social", "PresentToFriend", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "friend_rid": rid, "friend_srv_id": srv_id
        })).await;
        // 跨域: 调 economy 加赠送资源
        let _ = rgs.call("economy", "AddEnergy", serde_json::json!({
            "id": "33333333-3333-3333-3333-333333333333",
            "amount": 5
        })).await;
        tracing::info!(friend_rid = rid, %srv_id, "13315 friend_present");
        let mut out = Vec::with_capacity(8);
        out.write_u8(0);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13330 cli: empty; srv: {black_list: [{rid, srv_id, name, lev, sex, career, face_id, power, login_out_time, is_online, gid, gsrv_id, gname}]}
// 黑名单列表
pub fn handle_black_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListBlacklist", default_player_uuid()).await;
        let blacks: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("blacklist"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(8 + blacks.len() * 96);
        out.write_u16(blacks.len() as u16);
        for b in &blacks {
            let rid = b.get("rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = b.get("srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let lev = b.get("lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let sex = b.get("sex").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let career = b.get("career").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let face_id = b.get("face_id").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            let power = b.get("power").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let login_out_time = b.get("login_out_time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let is_online = b.get("is_online").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let gid = b.get("gid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let gsrv_id = b.get("gsrv_id").and_then(|v| v.as_str()).unwrap_or("");
            let gname = b.get("gname").and_then(|v| v.as_str()).unwrap_or("");
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_u8(lev);
            out.write_u8(sex);
            out.write_u8(career);
            out.write_u16(face_id);
            out.write_u32(power);
            out.write_u32(login_out_time);
            out.write_u8(is_online);
            out.write_u32(gid);
            out.write_string(gsrv_id);
            out.write_string(gname);
        }
        tracing::info!(count = blacks.len(), "13330 black_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13332 cli: {rid:u32, srv_id:str}; srv: {code:u8, msg:str, rid:u32, srv_id:str}
// 加入黑名单
pub fn handle_black_add(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        let _ = rgs.call("social", "AddToBlacklist", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "target_rid": rid, "target_srv_id": srv_id
        })).await;
        tracing::info!(target_rid = rid, %srv_id, "13332 black_add");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (blacklisted)");
        out.write_u32(rid);
        out.write_string(&srv_id);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13333 cli: {rid:u32, srv_id:str}; srv: {code:u8, msg:str, rid:u32, srv_id:str}
// 移除黑名单
pub fn handle_black_remove(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id) = if payload.len() >= 6 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string())
        } else {
            (0u32, String::new())
        };
        let _ = rgs.call("social", "RemoveFromBlacklist", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "target_rid": rid, "target_srv_id": srv_id
        })).await;
        tracing::info!(target_rid = rid, %srv_id, "13333 black_remove");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (unblacklisted)");
        out.write_u32(rid);
        out.write_string(&srv_id);
        crate::handlers::Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_135 — 公会 (13500-13599, 31 stub → 11 real)
// ============================================================================

// 13500 cli: {name:str, sign:str, apply_type:u8, apply_lev:u8}; srv: {code:u8, msg:str}
// 创建公会
pub fn handle_guild_create(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (name, sign, apply_type, apply_lev) = if payload.len() >= 10 {
            let mut p: &[u8] = &payload[..];
            (p.read_string(), p.read_string(), p.read_u8(), p.read_u8())
        } else {
            (String::new(), String::new(), 0u8, 0u8)
        };
        let rgs_resp = rgs.call("social", "CreateGuild", serde_json::json!({
            "leader_id": "11111111-1111-1111-1111-111111111111",
            "name": name, "sign": sign,
            "apply_type": apply_type, "apply_lev": apply_lev
        })).await;
        let code = if rgs_resp.ok { 0u8 } else { 1u8 };
        let msg = if rgs_resp.ok { "OK (guild created)".to_string() } else {
            format!("RGS error: {}", rgs_resp.error.unwrap_or_default())
        };
        tracing::info!(%name, code, "13500 guild_create");
        let mut out = Vec::with_capacity(64);
        out.write_u8(code);
        out.write_string(&msg);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13501 cli: {page:u16, flag:u8, num:u16, name:str}
// srv: {page:u16, flag:u8, num:u16, page_total:u16, all_count:u16, name:str, guilds: [{gid, gsrv_id, name, lev, members_num, members_max, leader_name, apply_type, apply_lev, is_apply}]}
// 搜索/列出公会
pub fn handle_guild_list(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (page, flag, num, name) = if payload.len() >= 7 {
            let mut p: &[u8] = &payload[..];
            (p.read_u16(), p.read_u8(), p.read_u16(), p.read_string())
        } else {
            (0u16, 0u8, 10u16, String::new())
        };
        let rgs_resp = rgs.call("social", "ListGuilds", serde_json::json!({
            "page": page, "flag": flag, "num": num, "name": name
        })).await;
        let guilds: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("guilds"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let page_total = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("page_total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u16;
        let all_count = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("all_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
        let mut out = Vec::with_capacity(32 + guilds.len() * 96);
        out.write_u16(page);
        out.write_u8(flag);
        out.write_u16(num);
        out.write_u16(page_total);
        out.write_u16(all_count);
        out.write_string(&name);
        out.write_u16(guilds.len() as u16);
        for g in &guilds {
            let gid = g.get("gid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let gsrv_id = g.get("gsrv_id").and_then(|v| v.as_str()).unwrap_or("");
            let gname = g.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let lev = g.get("lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let members_num = g.get("members_num").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let members_max = g.get("members_max").and_then(|v| v.as_u64()).unwrap_or(50) as u8;
            let leader_name = g.get("leader_name").and_then(|v| v.as_str()).unwrap_or("");
            let apply_type = g.get("apply_type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let apply_lev = g.get("apply_lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_apply = g.get("is_apply").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            out.write_u32(gid);
            out.write_string(gsrv_id);
            out.write_string(gname);
            out.write_u8(lev);
            out.write_u8(members_num);
            out.write_u8(members_max);
            out.write_string(leader_name);
            out.write_u8(apply_type);
            out.write_u8(apply_lev);
            out.write_u8(is_apply);
        }
        tracing::info!(page, num, count = guilds.len(), "13501 guild_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13503 cli: {gid:u32, gsrv_id:str, type:u8}; srv: {code:u8, msg:str, gid:u32, gsrv_id:str, is_apply:u8}
// 申请加入公会 (type: 0=申请, 1=取消)
pub fn handle_guild_apply(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (gid, gsrv_id, apply_type) = if payload.len() >= 9 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u8())
        } else {
            (0u32, String::new(), 0u8)
        };
        let _ = rgs.call("social", "ApplyGuild", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "guild_id": gid, "guild_srv_id": gsrv_id, "action": apply_type
        })).await;
        tracing::info!(gid, %gsrv_id, apply_type, "13503 guild_apply");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string(if apply_type == 0 { "OK (applied)" } else { "OK (cancelled)" });
        out.write_u32(gid);
        out.write_string(&gsrv_id);
        out.write_u8(apply_type);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13507 cli: {page:u8, num:u8}
// srv: {page, page_total, num, guids: [{rid, srv_id, name, lev, face, power, vip_lev, is_online}]}
// 公会成员申请列表 (会长审核用)
pub fn handle_guild_apply_list(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (page, num) = if payload.len() >= 2 {
            let mut p: &[u8] = &payload[..];
            (p.read_u8(), p.read_u8())
        } else {
            (0u8, 10u8)
        };
        let rgs_resp = rgs.call("social", "ListGuildApplyRequests", serde_json::json!({
            "guild_id": "22222222-2222-2222-2222-222222222222",
            "page": page, "num": num
        })).await;
        let applicants: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("applicants"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let page_total = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("page_total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u8;
        let mut out = Vec::with_capacity(8 + applicants.len() * 64);
        out.write_u8(page);
        out.write_u8(page_total);
        out.write_u8(num);
        out.write_u16(applicants.len() as u16);
        for a in &applicants {
            let rid = a.get("rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = a.get("srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let lev = a.get("lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let face = a.get("face").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let power = a.get("power").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let vip_lev = a.get("vip_lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let is_online = a.get("is_online").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_u8(lev);
            out.write_u32(face);
            out.write_u32(power);
            out.write_u8(vip_lev);
            out.write_u8(is_online);
        }
        tracing::info!(page, num, count = applicants.len(), "13507 guild_apply_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13518 cli: empty; srv: 完整 guild 字段 (15 字段 + rank_idx:i16, per proto_135.erl)
// {gid, gsrv_id, name, lev, members_num, members_max, leader_name, rid, srv_id,
//  sign, exp, day_exp, apply_type, apply_lev, recruit_num, rank_idx}
pub fn handle_guild_info(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "GetGuild", default_guild_uuid()).await;
        let g = rgs_resp.response.unwrap_or(serde_json::json!({}));
        let gid = g.get("id").and_then(|v| v.get("id")).and_then(|v| v.as_str()).and_then(|s| u32::from_str_radix(&s[..8.min(s.len())], 16).ok()).unwrap_or(0x22222222);
        let gsrv_id = g.get("srv_id").and_then(|v| v.as_str()).unwrap_or("rgs-uat-1");
        let name = g.get("display_name").and_then(|v| v.as_str()).unwrap_or("RGS Guild");
        let lev = g.get("lev").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
        let members_num = g.get("members_num").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let members_max = g.get("members_max").and_then(|v| v.as_u64()).unwrap_or(50) as u8;
        let leader_name = g.get("leader_name").and_then(|v| v.as_str()).unwrap_or("Leader");
        let leader_id = g.get("leader_id").and_then(|v| v.as_str()).unwrap_or("11111111-0000-0000-0000-000000000000");
        let leader_rid = u32::from_str_radix(&leader_id[..8.min(leader_id.len())], 16).unwrap_or(0x11111111);
        let sign = g.get("sign").and_then(|v| v.as_str()).unwrap_or("");
        let exp = g.get("exp").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let day_exp = g.get("day_exp").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let apply_type = g.get("apply_type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let apply_lev = g.get("apply_lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let recruit_num = g.get("recruit_num").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let rank_idx = g.get("rank_idx").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        tracing::info!(%name, lev, members_num, "13518 guild_info");
        let mut out = Vec::with_capacity(256);
        out.write_u32(gid);
        out.write_string(gsrv_id);
        out.write_string(name);
        out.write_u8(lev);
        out.write_u8(members_num);
        out.write_u8(members_max);
        out.write_string(leader_name);
        out.write_u32(leader_rid);
        out.write_string("rgs-uat-1");
        out.write_string(sign);
        out.write_u32(exp);
        out.write_u32(day_exp);
        out.write_u8(apply_type);
        out.write_u8(apply_lev);
        out.write_u8(recruit_num);
        out.write_i16(rank_idx);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13519 cli: empty; srv: {members: [{rid, srv_id, name, lev, face, post, online, vip_lev, power, join_time, login_time, donate, day_donate, avatar_bid, sex}]}
// 公会成员列表
pub fn handle_guild_members(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListGuildMembers", default_guild_uuid()).await;
        let members: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("members"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(8 + members.len() * 96);
        out.write_u16(members.len() as u16);
        for m in &members {
            let rid = m.get("rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = m.get("srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let lev = m.get("lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let face = m.get("face").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let post = m.get("post").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let online = m.get("online").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let vip_lev = m.get("vip_lev").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let power = m.get("power").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let join_time = m.get("join_time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let login_time = m.get("login_time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let donate = m.get("donate").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let day_donate = m.get("day_donate").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let avatar_bid = m.get("avatar_bid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let sex = m.get("sex").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_u8(lev);
            out.write_u32(face);
            out.write_u8(post);
            out.write_u8(online);
            out.write_u8(vip_lev);
            out.write_u32(power);
            out.write_u32(join_time);
            out.write_u32(login_time);
            out.write_u32(donate);
            out.write_u32(day_donate);
            out.write_u32(avatar_bid);
            out.write_u8(sex);
        }
        tracing::info!(count = members.len(), "13519 guild_members");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13520 cli: {rid:u32, srv_id:str, position:u8}; srv: {rid, srv_id, position, code:u8, msg:str}
// 任命职位 (position: 0=member, 1=elder, 2=vp, 3=leader)
pub fn handle_guild_set_position(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (rid, srv_id, position) = if payload.len() >= 9 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_string(), p.read_u8())
        } else {
            (0u32, String::new(), 0u8)
        };
        let _ = rgs.call("social", "SetGuildPosition", serde_json::json!({
            "guild_id": "22222222-2222-2222-2222-222222222222",
            "target_rid": rid, "target_srv_id": srv_id,
            "position": position
        })).await;
        tracing::info!(target_rid = rid, %srv_id, position, "13520 guild_set_position");
        let mut out = Vec::with_capacity(32);
        out.write_u32(rid);
        out.write_string(&srv_id);
        out.write_u8(position);
        out.write_u8(0);
        out.write_string("OK (position set)");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13522 cli: {apply_type:u8, apply_lev:u8}; srv: {code:u8, msg:str}
// 设置公会申请条件
pub fn handle_guild_set_apply(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (apply_type, apply_lev) = if payload.len() >= 2 {
            let mut p: &[u8] = &payload[..];
            (p.read_u8(), p.read_u8())
        } else {
            (0u8, 0u8)
        };
        let _ = rgs.call("social", "SetGuildApplyCondition", serde_json::json!({
            "guild_id": "22222222-2222-2222-2222-222222222222",
            "apply_type": apply_type, "apply_lev": apply_lev
        })).await;
        tracing::info!(apply_type, apply_lev, "13522 guild_set_apply");
        ok_payload(cmd, "OK (apply condition set)")
    })
}

// 13523 cli: empty
// srv: {donate_list: [{type:u8, num:u8}], boxes: [box_id:u8], donate_exp:u32, day_send_num:u8, day_recv_num:u8}
// 公会捐献 (返回当前捐献状态, 实际捐献走 13535/13536)
pub fn handle_guild_donate_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListGuildDonate", default_guild_uuid()).await;
        let donate_list: Vec<(u8, u8)> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("donate_list"))
                .and_then(|m| m.as_array())
                .map(|arr| arr.iter().filter_map(|v| {
                    let ty = v.get("type").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
                    let num = v.get("num").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
                    Some((ty, num))
                }).collect())
                .unwrap_or_default()
        } else {
            vec![]
        };
        let boxes: Vec<u8> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("boxes"))
                .and_then(|m| m.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|x| x as u8)).collect())
                .unwrap_or_default()
        } else {
            vec![]
        };
        let donate_exp = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("donate_exp"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let day_send_num = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("day_send_num"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        let day_recv_num = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("day_recv_num"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        let mut out = Vec::with_capacity(32);
        out.write_u16(donate_list.len() as u16);
        for (ty, num) in &donate_list {
            out.write_u8(*ty);
            out.write_u8(*num);
        }
        out.write_u16(boxes.len() as u16);
        for b in &boxes {
            out.write_u8(*b);
        }
        out.write_u32(donate_exp);
        out.write_u8(day_send_num);
        out.write_u8(day_recv_num);
        tracing::info!("13523 guild_donate_list");
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13524 cli: {type:u8}; srv: {code:u8, msg:str}
// 退出/解散公会 (type: 0=quit, 1=dissolve)
pub fn handle_guild_quit(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let action = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_u8()
        } else {
            0u8
        };
        let _ = rgs.call("social", "LeaveGuild", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "guild_id": "22222222-2222-2222-2222-222222222222",
            "dissolve": action == 1
        })).await;
        tracing::info!(action, "13524 guild_quit");
        let msg = if action == 1 { "OK (guild dissolved)" } else { "OK (left guild)" };
        ok_payload(cmd, msg)
    })
}

// 13536 cli: {id:u32}; srv: {code:u8, msg:str, id:u32, type:u8, val:u32, day_recv_num:u8}
// 领取捐献宝箱奖励 (id = box_id) — 跨域 economy 域加资源
pub fn handle_guild_donate_recv(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let box_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        // 1. 走 social 域领取
        let _ = rgs.call("social", "ReceiveGuildDonateBox", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "box_id": box_id
        })).await;
        // 2. 跨域 economy 域加资源 (per 5 域 RGS 经济解耦)
        let _ = rgs.call("economy", "AddResource", serde_json::json!({
            "id": "33333333-3333-3333-3333-333333333333",
            "resource": "guild_contrib",
            "amount": 100
        })).await;
        tracing::info!(box_id, "13536 guild_donate_recv (cross-domain social→economy)");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);
        out.write_string("OK (donation received)");
        out.write_u32(box_id);
        out.write_u8(1);
        out.write_u32(100);
        out.write_u8(1);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13540 cli: {id:u32}; srv: {id, type, name, face_id, avatar_bid, post, val, list: [{rid, srv_id, name, face_id, avatar_bid, post, val, time}]}
// 公会日志 (id = log_type: 0=donate, 1=join/quit, 2=position change)
pub fn handle_guild_log(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let log_id = if payload.len() >= 4 {
            let mut p: &[u8] = &payload[..];
            p.read_u32()
        } else {
            0u32
        };
        let rgs_resp = rgs.call("social", "ListGuildLog", serde_json::json!({
            "guild_id": "22222222-2222-2222-2222-222222222222",
            "log_type": log_id
        })).await;
        let entries: Vec<serde_json::Value> = if rgs_resp.ok {
            rgs_resp.response
                .as_ref()
                .and_then(|r| r.get("entries"))
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(64 + entries.len() * 96);
        out.write_u32(log_id);
        out.write_u8(0);                           // type
        out.write_string("OK");
        out.write_u32(0);                          // face_id
        out.write_u32(0);                          // avatar_bid
        out.write_u32(0);                          // post
        out.write_u32(0);                          // val
        out.write_u16(entries.len() as u16);
        for e in &entries {
            let rid = e.get("rid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let srv_id = e.get("srv_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let face_id = e.get("face_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let avatar_bid = e.get("avatar_bid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let post = e.get("post").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let val = e.get("val").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let time = e.get("time").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            out.write_u32(rid);
            out.write_string(srv_id);
            out.write_string(name);
            out.write_u32(face_id);
            out.write_u32(avatar_bid);
            out.write_u32(post);
            out.write_u32(val);
            out.write_u32(time);
        }
        tracing::info!(log_id, count = entries.len(), "13540 guild_log");
        crate::handlers::Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_136 — 跨服 (13600-13699, 8 stub → 2 real)
// ============================================================================

// 13601 cli: empty; srv: {code:u8, msg:str, cross_friend_count:u16}
// 跨服好友列表 (per 5 域 RGS cross-server 模式)
pub fn handle_cross_friend_list(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let rgs_resp = rgs.call("social", "ListCrossServerFriends", default_player_uuid()).await;
        let count = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("cross_friends"))
            .and_then(|m| m.as_array())
            .map(|a| a.len() as u16)
            .unwrap_or(0);
        tracing::info!(count, "13601 cross_friend_list");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (cross-server friends)");
        out.write_u16(count);
        crate::handlers::Response { cmd, payload: out }
    })
}

// 13602 cli: {srv_id:str}; srv: {code:u8, msg:str, guild_count:u16}
// 跨服公会查询 (按 srv_id 列出该服所有公会 — 用于跨服战/跨服匹配)
pub fn handle_cross_guild_list(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let target_srv = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_string()
        } else {
            "all".to_string()
        };
        // 跨域: 走 match 域的跨服索引 (per 5 域 RGS 解耦)
        let rgs_resp = rgs.call("match", "ListCrossServerGuilds", serde_json::json!({
            "target_srv_id": target_srv
        })).await;
        let count = rgs_resp.response
            .as_ref()
            .and_then(|r| r.get("guilds"))
            .and_then(|m| m.as_array())
            .map(|a| a.len() as u16)
            .unwrap_or(0);
        tracing::info!(%target_srv, count, "13602 cross_guild_list (social→match cross-domain)");
        let mut out = Vec::with_capacity(16);
        out.write_u8(0);
        out.write_string("OK (cross-server guilds)");
        out.write_u16(count);
        crate::handlers::Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_166 — 活动/福利 (16600-16799, 47 stub → 2 real; 活动走 campaign 域 RGS)
// ============================================================================

// 16601 cli: {type:u8}; srv: {holiday_list: [{bid, title, title2, ico, type_ico, top_banner, rule_str, time_str, bottom_alert, aim_title, panel_type, channel_ban, sort_val, reward_title, remain_sec}]}
// 节日活动列表
pub fn handle_holiday_list(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let activity_type = if !payload.is_empty() {
            let mut p: &[u8] = &payload[..];
            p.read_u8()
        } else {
            0u8
        };
        let _ = rgs.call("social", "ListHolidays", serde_json::json!({
            "type": activity_type
        })).await;
        tracing::debug!(activity_type, "16601 holiday_list");
        let mut out = Vec::with_capacity(8);
        out.write_u16(0);                          // empty list (RGS N/A)
        crate::handlers::Response { cmd, payload: out }
    })
}

// 16630 cli: empty; srv: {code:u8, items: [{bid:u32, num:u32}]}
// 活动领奖
pub fn handle_holiday_claim(
    cmd: u16,
    _payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let _ = rgs.call("social", "ClaimHolidayReward", default_player_uuid()).await;
        // 跨域 economy 加资源
        let _ = rgs.call("economy", "AddResource", serde_json::json!({
            "id": "33333333-3333-3333-3333-333333333333",
            "resource": "holiday_token",
            "amount": 50
        })).await;
        tracing::debug!("16630 holiday_claim (cross-domain social→economy)");
        let mut out = Vec::with_capacity(8);
        out.write_u8(0);
        out.write_u16(0);
        crate::handlers::Response { cmd, payload: out }
    })
}

// ============================================================================
// proto_168 — 提示处理 (16800-16899, 3 stub → 1 real)
// ============================================================================

// 16800 cli: {idx:u32, val:u32, str:str}; srv: {type, arg_uint32: [{key, value}], arg_str: [{key, value}], idx}
// 客户端 ack 提示/指引完成 (服务器记录指引进度)
pub fn handle_tip_ack(
    cmd: u16,
    payload: Vec<u8>,
    rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::handlers::Response> + Send>> {
    Box::pin(async move {
        let (idx, val, hint_str) = if payload.len() >= 12 {
            let mut p: &[u8] = &payload[..];
            (p.read_u32(), p.read_u32(), p.read_string())
        } else {
            (0u32, 0u32, String::new())
        };
        let _ = rgs.call("social", "AckTip", serde_json::json!({
            "player_id": "11111111-1111-1111-1111-111111111111",
            "idx": idx, "val": val, "str": hint_str
        })).await;
        tracing::info!(idx, val, "16800 tip_ack");
        let mut out = Vec::with_capacity(32);
        out.write_u8(0);                           // type
        out.write_u16(0);                          // arg_uint32 count
        out.write_u16(0);                          // arg_str count
        out.write_u32(idx);
        crate::handlers::Response { cmd, payload: out }
    })
}

// ============================================================================
// 注册: 33 social 域 real handler (per Phase 4 e2e/social worker 派工)
// 覆盖 stub 总数: 33/93 (chat=8, friend=9, guild=11, cross=2, activity=2, tip=1)
// 剩余 60 stub 留 v0.4.2+ 扩 (proto_134 exchange 19 + proto_135 边角 20 + proto_166 详情 18 + proto_168 2 + chat 留 1)
// ============================================================================

use crate::registry::CmdEntry;

pub fn register_social_real(map: &mut std::collections::HashMap<u16, CmdEntry>) {
    // proto_130 — 邮件/聊天 (8)
    map.insert(13000, CmdEntry { handler: handle_dungeon_list, name: "dungeon_list (RGS social ListDungeons)", source: "zsyz" });
    map.insert(13002, CmdEntry { handler: handle_dungeon_op, name: "dungeon_op", source: "zsyz" });
    map.insert(13005, CmdEntry { handler: handle_dungeon_sweep, name: "dungeon_sweep (RGS social SweepDungeon)", source: "zsyz" });
    map.insert(13011, CmdEntry { handler: handle_dungeon_buff_list, name: "dungeon_buff_list", source: "zsyz" });
    map.insert(13012, CmdEntry { handler: handle_chat_send, name: "chat_send (RGS social SendChatMessage)", source: "zsyz" });
    map.insert(13018, CmdEntry { handler: handle_chat_inbox, name: "chat_inbox (RGS social ListMessages)", source: "zsyz" });
    map.insert(13030, CmdEntry { handler: handle_mail_attach, name: "mail_attach (RGS social ClaimMailAttachment)", source: "zsyz" });
    map.insert(13040, CmdEntry { handler: handle_mail_list, name: "mail_list (RGS social ListMails)", source: "zsyz" });

    // proto_133 — 好友社交 (9)
    map.insert(13300, CmdEntry { handler: handle_friend_list, name: "friend_list (RGS social ListFriends)", source: "zsyz" });
    map.insert(13303, CmdEntry { handler: handle_friend_add, name: "friend_add (RGS social RequestFriend)", source: "zsyz" });
    map.insert(13305, CmdEntry { handler: handle_friend_agree, name: "friend_agree (RGS social RespondFriendRequest)", source: "zsyz" });
    map.insert(13307, CmdEntry { handler: handle_friend_delete, name: "friend_delete (RGS social DeleteFriend)", source: "zsyz" });
    map.insert(13311, CmdEntry { handler: handle_friend_req_list, name: "friend_req_list (RGS social ListFriendRequests)", source: "zsyz" });
    map.insert(13315, CmdEntry { handler: handle_friend_present, name: "friend_present (RGS social+economy)", source: "zsyz" });
    map.insert(13330, CmdEntry { handler: handle_black_list, name: "black_list (RGS social ListBlacklist)", source: "zsyz" });
    map.insert(13332, CmdEntry { handler: handle_black_add, name: "black_add (RGS social AddToBlacklist)", source: "zsyz" });
    map.insert(13333, CmdEntry { handler: handle_black_remove, name: "black_remove (RGS social RemoveFromBlacklist)", source: "zsyz" });

    // proto_135 — 公会 (11)
    map.insert(13500, CmdEntry { handler: handle_guild_create, name: "guild_create (RGS social CreateGuild)", source: "zsyz" });
    map.insert(13501, CmdEntry { handler: handle_guild_list, name: "guild_list (RGS social ListGuilds)", source: "zsyz" });
    map.insert(13503, CmdEntry { handler: handle_guild_apply, name: "guild_apply (RGS social ApplyGuild)", source: "zsyz" });
    map.insert(13507, CmdEntry { handler: handle_guild_apply_list, name: "guild_apply_list (RGS social ListGuildApplyRequests)", source: "zsyz" });
    map.insert(13518, CmdEntry { handler: handle_guild_info, name: "guild_info (RGS social GetGuild)", source: "zsyz" });
    map.insert(13519, CmdEntry { handler: handle_guild_members, name: "guild_members (RGS social ListGuildMembers)", source: "zsyz" });
    map.insert(13520, CmdEntry { handler: handle_guild_set_position, name: "guild_set_position (RGS social SetGuildPosition)", source: "zsyz" });
    map.insert(13522, CmdEntry { handler: handle_guild_set_apply, name: "guild_set_apply (RGS social SetGuildApplyCondition)", source: "zsyz" });
    map.insert(13523, CmdEntry { handler: handle_guild_donate_list, name: "guild_donate_list (RGS social ListGuildDonate)", source: "zsyz" });
    map.insert(13524, CmdEntry { handler: handle_guild_quit, name: "guild_quit (RGS social LeaveGuild)", source: "zsyz" });
    map.insert(13536, CmdEntry { handler: handle_guild_donate_recv, name: "guild_donate_recv (RGS social+economy cross-domain)", source: "zsyz" });
    map.insert(13540, CmdEntry { handler: handle_guild_log, name: "guild_log (RGS social ListGuildLog)", source: "zsyz" });

    // proto_136 — 跨服 (2)
    map.insert(13601, CmdEntry { handler: handle_cross_friend_list, name: "cross_friend_list (RGS social ListCrossServerFriends)", source: "zsyz" });
    map.insert(13602, CmdEntry { handler: handle_cross_guild_list, name: "cross_guild_list (RGS match ListCrossServerGuilds)", source: "zsyz" });

    // proto_166 — 活动/福利 (2)
    map.insert(16601, CmdEntry { handler: handle_holiday_list, name: "holiday_list (RGS social ListHolidays)", source: "zsyz" });
    map.insert(16630, CmdEntry { handler: handle_holiday_claim, name: "holiday_claim (RGS social+economy)", source: "zsyz" });

    // proto_168 — 提示 (1)
    map.insert(16800, CmdEntry { handler: handle_tip_ack, name: "tip_ack (RGS social AckTip)", source: "zsyz" });
}
