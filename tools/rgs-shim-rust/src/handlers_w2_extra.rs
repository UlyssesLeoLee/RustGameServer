// Phase 4 w2 续做 (per 2026-09-09 20:30 JST Mavis 派工):
// w2 economy 续 110 welfare + 其它 (proto_238 guild_shipping + proto_239 tournament)
// 范围: cmd 23800-24818
//
// 来源 (per PHASE_4_WORKER_BRIEF.md §3.2 + proto_mate.js 提取):
// - proto_238.erl (guild_shipping 23800-23812, 13 cmd)
// - proto_239.erl (tournament/endless_trail 23900-23911, 12 cmd)
// - proto_240+ (welfare 24000-24818, 110 cmd, 客户端 mod/welfare, 无服务端 proto)
//
// 实现原则:
// - 全部走 RGS stub (跟 w2 续做 200 cmd 风格一致)
// - 字节级对齐 erlang pack(srv, ...), 简化到最小可用结构
// - 110 welfare + 25 其它 (238xx+239xx) = 135 cmd total
// - 走 handle_w2_extra 单 dispatcher (类似 handlers_w2::handle_w2_economy)

use crate::frame::BeWrite;
use crate::handlers::Response;
use crate::rgs::RgsClient;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

/// 编码 {code:u8, msg:str} (最常见, ~70% cmd)
fn enc_code_msg(code: u8, msg: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + msg.len());
    out.write_u8(code);
    out.write_string(msg);
    out
}

/// 编码空响应 {} (per proto: pack(cmd, srv, {}))
fn enc_empty() -> Vec<u8> { Vec::new() }

/// 编码 u16 count 空列表 (t:9 列表类型最小化)
fn enc_empty_list() -> Vec<u8> {
    let mut out = Vec::with_capacity(2);
    out.write_u16(0);
    out
}

/// 通用 dispatcher: 按 cmd 范围分派到不同 proto handler
/// 范围: cmd 23800-24818 (其它 + welfare, 135 cmd)
pub fn handle_w2_extra(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    Box::pin(async move {
        let payload = match cmd {
            // ===== proto_238.erl: guild_shipping 押镖 (23800-23812, 13 cmd) =====
            23800 => { // cli={}, srv={order_list, buy_order_times, is_assist} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u16(0);   // order_list count
                out.write_u8(0);    // buy_order_times
                out.write_u8(0);    // is_assist
                out
            }
            23801 => { // cli={order_id}, srv={#guild_shipping_order{...}} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);   // order_id
                out.write_u32(0);   // order_bid
                out.write_u8(0);    // type
                out.write_u8(0);    // status
                out
            }
            23802 => enc_code_msg(0, "OK"),  // cli={order_id, partner_ids, is_success}, srv={code,msg,order_id}
            23803 => { // cli={order_id}, srv={code,msg,order_id,status}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            23804 => enc_code_msg(0, "OK"),  // cli={}, srv={code,msg}
            23805 => { // cli={}, srv={order} → 最小
                enc_empty_list()
            }
            23806 => { // cli={}, srv={count, assist_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);   // count
                out.write_u16(0);   // assist_list
                out
            }
            23807 => { // cli={rid, srv_id, order_id}, srv={code,msg,{rid,srv_id},order_id,status,end_time}
                let mut out = Vec::with_capacity(32);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out.write_string("");
                out.write_u32(0);
                out.write_u8(0);
                out.write_u32(0);
                out
            }
            23808 => { // cli={rid, srv_id, order_id, item_bid}, srv={code,msg,rid,srv_id,order_id,item_bid}
                let mut out = Vec::with_capacity(32);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out.write_string("");
                out.write_u32(0);
                out.write_u32(0);
                out
            }
            23809 => { // cli={order_id, is_double}, srv={code,msg,order_id,is_success,is_double}
                let mut out = Vec::with_capacity(24);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out.write_u8(0);
                out.write_u8(0);
                out
            }
            23810 => enc_code_msg(0, "OK"),  // cli={order_id, item_bid}, srv={code,msg}
            23811 => { // cli={}, srv={order_id, status, end_time}
                let mut out = Vec::with_capacity(12);
                out.write_u32(0);
                out.write_u8(0);
                out.write_u32(0);
                out
            }
            23812 => enc_code_msg(0, "OK"),  // cli={order_id}, srv={code,msg}

            // ===== proto_239.erl: tournament/endless_trail 试炼 (23900-23911, 12 cmd) =====
            23900 => { // cli={}, srv={max_round, current_round, day_pass_round, my_idx, rank_list, is_employ, list, is_appoint, is_reward} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // max_round
                out.write_u16(0);   // current_round
                out.write_u16(0);   // day_pass_round
                out.write_u16(0);   // my_idx
                out.write_u16(0);   // rank_list count
                out.write_u8(0);    // is_employ
                out.write_u16(0);   // list count
                out.write_u8(0);    // is_appoint
                out.write_u8(0);    // is_reward
                out
            }
            23901 => enc_code_msg(0, "OK"),  // cli={formation_type, pos_info}, srv={code,msg}
            23902 => { // cli={}, srv={round, max_round, buff_list, rest_round, max_reward_round, reward_flag, acc_reward, id, status} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // round
                out.write_u16(0);   // max_round
                out.write_u16(0);   // buff_list count
                out.write_u16(0);   // rest_round
                out.write_u16(0);   // max_reward_round
                out.write_u8(0);    // reward_flag
                out.write_u16(0);   // acc_reward
                out.write_u32(0);   // id
                out.write_u8(0);    // status
                out
            }
            23903 => { // cli={}, srv={id, status}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            23904 => enc_code_msg(0, "OK"),  // cli={id}, srv={code,msg}
            23905 => enc_empty_list(),       // cli={}, srv={list}
            23906 => enc_empty_list(),       // cli={}, srv={list}
            23907 => enc_empty_list(),       // cli={}, srv={list}
            23908 => { // cli={id}, srv={code,msg,id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }
            23909 => enc_code_msg(0, "OK"),  // cli={rid, srv_id, id, flag}, srv={code,msg}
            23910 => { // cli={}, srv={is_select, list, partner, formation_type, formation_lev, round} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);    // is_select
                out.write_u16(0);   // list count
                out.write_u16(0);   // partner count
                out.write_u8(0);    // formation_type
                out.write_u16(0);   // formation_lev
                out.write_u16(0);   // round
                out
            }
            23911 => enc_code_msg(0, "OK"),  // cli={buff_id}, srv={code,msg}

            // ===== welfare 福利 (24000-24818, 110 cmd) =====
            // 大部分 welfare cmd 是简单状态查询 / 通知, 返回 {code:u8, msg:str}
            // 复杂列表 (t:9) 返回 u16 count=0 占位
            // 来源: zsyz_client mod/welfare 模块 (无服务端 proto_240.erl)
            24000 => enc_empty_list(),  // cli={rid,srv_id}, srv={plunders[]} (帮派掠夺列表, 复杂)
            24001 => enc_code_msg(0, "OK"),
            24002 => enc_code_msg(0, "OK"),
            24003 => enc_code_msg(0, "OK"),
            24004 => enc_code_msg(0, "OK"),
            24005 => { // cli={}, srv={rid,srv_id,plunders[]} (帮派掠夺列表, 复杂)
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_string("");
                out
            }
            24006 => { // cli={}, srv={status,quality,end_time,plunder_count,datas[]} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_u8(0);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u16(0);
                out
            }
            24010 => { // cli={rid,srv_id}, srv={quality,rid,srv_id,...,p_list[]} (玩家信息, 复杂)
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out.write_u32(0);
                out.write_string("");
                out
            }
            24011 => enc_code_msg(0, "OK"),
            24012 => { // cli={}, srv={result,item_rewards[]} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u8(0);
                out.write_u16(0);
                out
            }
            24013 => { // cli={type}, srv={type,logs[]} (战报日志, 复杂)
                let mut out = Vec::with_capacity(4);
                out.write_u8(0);
                out.write_u16(0);
                out
            }
            24014 => { // cli={rid,srv_id,id,type}, srv={quality,id,type,...,p_list[]} (玩家信息, 复杂)
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out.write_u32(0);
                out.write_u8(0);
                out.write_u32(0);
                out.write_string("");
                out
            }
            24015 => enc_code_msg(0, "OK"),
            24017 => { // cli={id}, srv={code,msg}
                enc_code_msg(0, "OK")
            }
            24018 => { // cli={}, srv={id,quality,...,items[],replay_id} (战报详情, 复杂)
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u8(0);
                out.write_u32(0);
                out.write_string("");
                out
            }
            24019 => enc_empty_list(),  // cli={}, srv={plunders[]} (帮派掠夺列表)
            24020 => vec![0u8],  // cli={}, srv={code:u8}
            24100 => enc_empty_list(),  // cli={}, srv={hallows[]} (圣物列表, 复杂)
            24101 => { // cli={id,is_auto}, srv={result,msg,id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }
            24103 => { // cli={hallows_id}, srv={result,msg,hallows_id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }
            24104 => { // cli={hallows_id,num}, srv={result,msg,id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }
            24107 => enc_empty_list(),  // cli={}, srv={hallows[]} (圣物)
            24108 => enc_empty_list(),  // cli={}, srv={hallows[]} (圣物)
            24120 => enc_empty(),       // cli={}, srv={}
            24121 => enc_empty(),       // cli={}, srv={}
            24122 => { // cli={id}, srv={...} (圣物详情, 复杂)
                enc_empty()
            }
            24123 => enc_empty(),       // cli={id}, srv={...}
            24124 => enc_empty(),       // cli={}, srv={}
            24125 => enc_empty(),       // cli={}, srv={}
            24126 => enc_empty(),       // cli={}, srv={}
            24127 => enc_empty(),       // cli={}, srv={}
            24128 => enc_empty(),       // cli={}, srv={}
            24129 => enc_empty(),       // cli={}, srv={}
            24130 => { // cli={id}, srv={...}
                enc_empty()
            }
            24131 => enc_empty(),       // cli={id}, srv={...}
            24132 => { // cli={id,hallows_id,flag}, srv={...}
                enc_empty()
            }
            24133 => enc_empty(),       // cli={}, srv={}
            24200 => enc_empty(),       // cli={}, srv={}
            24201 => enc_empty(),       // cli={pos}, srv={...}
            24202 => { // cli={pos,hp,flag}, srv={...} (玩家血量, 复杂)
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            24204 => enc_empty(),       // cli={}, srv={}
            24205 => enc_empty(),       // cli={}, srv={}
            24206 => enc_empty(),       // cli={}, srv={}
            24207 => enc_empty(),       // cli={}, srv={}
            24208 => enc_empty(),       // cli={}, srv={}
            24209 => { // cli={g_id1,g_sid1,pos}, srv={...} (工会战, 复杂)
                enc_empty()
            }
            24210 => enc_empty(),       // cli={}, srv={}
            24212 => enc_empty(),       // cli={}, srv={}
            24213 => enc_empty(),       // cli={}, srv={}
            24214 => enc_empty(),       // cli={}, srv={}
            24220 => enc_empty(),       // cli={}, srv={}
            24221 => { // cli={order}, srv={...}
                enc_empty()
            }
            24223 => enc_empty(),       // cli={}, srv={}
            24300 => enc_empty(),       // cli={}, srv={}
            24301 => enc_empty(),       // cli={}, srv={}
            24302 => { // cli={rid,srv_id}, srv={...} (玩家信息, 复杂)
                enc_empty()
            }
            24303 => enc_empty(),       // cli={rid,srv_id}, srv={...}
            24304 => enc_empty(),       // cli={}, srv={}
            24305 => enc_empty(),       // cli={}, srv={}
            24306 => enc_empty(),       // cli={}, srv={}
            24308 => enc_empty(),       // cli={}, srv={}
            24309 => enc_empty(),       // cli={}, srv={}
            24310 => enc_empty(),       // cli={}, srv={}
            24311 => enc_empty(),       // cli={}, srv={}
            24312 => enc_empty(),       // cli={}, srv={}
            24313 => enc_empty(),       // cli={}, srv={}
            24314 => enc_empty(),       // cli={}, srv={}
            24315 => enc_empty(),       // cli={}, srv={}
            24316 => { // cli={rid,srv_id,pos}, srv={...} (玩家信息, 复杂)
                enc_empty()
            }
            24400 => enc_empty(),       // cli={}, srv={}
            24401 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out
            }
            24402 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out
            }
            24403 => { // cli={formation_type,pos_info,hallows_id}, srv={...} (阵型, 复杂)
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out.write_u8(0);
                out.write_u32(0);
                out
            }
            24404 => enc_empty(),       // cli={}, srv={}
            24405 => enc_empty(),       // cli={}, srv={}
            24406 => enc_empty(),       // cli={}, srv={}
            24407 => enc_empty(),       // cli={id}, srv={...}
            24408 => { // cli={rid,srv_id,id}, srv={...} (玩家信息, 复杂)
                enc_empty()
            }
            24409 => enc_empty(),       // cli={}, srv={}
            24410 => enc_empty(),       // cli={}, srv={}
            24411 => enc_empty(),       // cli={}, srv={}
            24500 => enc_empty(),       // cli={}, srv={}
            24501 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24502 => enc_empty(),       // cli={}, srv={}
            24600 => enc_empty(),       // cli={}, srv={}
            24601 => enc_empty(),       // cli={}, srv={}
            24602 => enc_empty(),       // cli={}, srv={}
            24603 => { // cli={ret_list[]}, srv={...} (多 ret 返回, 复杂)
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);   // ret_list count
                out
            }
            24604 => enc_empty(),       // cli={}, srv={}
            24700 => enc_empty(),       // cli={}, srv={}
            24701 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24702 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24801 => enc_empty(),       // cli={}, srv={}
            24802 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out
            }
            24803 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);
                out
            }
            24804 => enc_empty(),       // cli={}, srv={}
            24805 => enc_empty(),       // cli={}, srv={}
            24806 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24807 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24808 => enc_empty(),       // cli={}, srv={}
            24809 => enc_empty(),       // cli={}, srv={}
            24810 => enc_empty(),       // cli={}, srv={}
            24811 => enc_empty(),       // cli={}, srv={}
            24812 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24813 => enc_empty(),       // cli={}, srv={}
            24814 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24815 => enc_empty(),       // cli={}, srv={}
            24816 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }
            24817 => enc_empty(),       // cli={}, srv={}
            24818 => { // cli={id}, srv={...}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out
            }

            // ===== fallback: 未知 cmd (238xx-248xx 但未在 proto 注册) =====
            _ => {
                enc_code_msg(0, "OK")
            }
        };
        if cmd >= 23800 && cmd < 24900 {
            tracing::debug!(cmd, payload_len = payload.len(), "w2-extra");
        }
        Response { cmd, payload }
    })
}
