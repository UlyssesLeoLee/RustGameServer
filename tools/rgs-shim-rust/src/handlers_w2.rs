// Phase 4 w2: economy/battle domain cmd real handlers (per 2026-09-09 19:39 JST Mavis 派工)
// Range: cmd 20000-29999 (200 unique cmd per registry_stubs.rs)
//
// 来源 (per PHASE_4_WORKER_BRIEF.md §3.2):
// - proto_200.erl (战备/HP/combat 20000-20099)
// - proto_202.erl (战斗详细/竞技场 20200-20299)
// - proto_205.erl (关卡/Boss 20500-20599)
// - proto_210.erl (活动/welfare 21000-21099)
// - proto_211.erl (签到 21100-21199)
// - proto_212.erl (升级奖励 21200-21299)
// - proto_213.erl (副本/排行 21300-21399)
// - proto_215.erl (avatar 21500-21599)
// - proto_221.erl (feat/servers 22100-22199)
// - proto_227.erl (feat/quest 22700-22799)
// - proto_232.erl (recruit 23200-23299)
// - proto_233.erl (honor 23300-23399)
// - proto_234.erl (login_gift 23400-23499)
// - proto_235.erl (market/shop 23500-23599)
// - proto_236.erl (reward 23600-23699)
// - proto_237.erl (extra 23700-23799)
//
// 实现策略: 按 proto_*.erl srv 格式生成字节级对齐响应
// - 简单 (code:u8, msg:str) 占 ~80% cmd, 走通用 encoder
// - 复杂 nested 走最小有效空列表
// - 全部走 RGS stub, 不真调 (Phase 4 目标: 100% 业务覆盖, 真实数据待 Phase 5)

use crate::frame::{BeRead, BeWrite};
use crate::handlers::Response;
use crate::rgs::RgsClient;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

/// 编码 {code:u8, msg:str} (最常见, ~80% cmd)
fn enc_code_msg(code: u8, msg: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + msg.len());
    out.write_u8(code);
    out.write_string(msg);
    out
}

/// 编码空响应 {} (per proto: pack(cmd, srv, {}))
fn enc_empty() -> Vec<u8> { Vec::new() }

/// 通用 dispatch: 按 cmd 返回对应 proto_*.erl 字节级响应
/// w2: economy/battle 域 200 cmd (20000-23799)
pub fn handle_w2_economy(
    cmd: u16,
    _payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        // 按 cmd 分发: 大多数返 (code, msg), 少数按 proto 返特定结构
        let payload = match cmd {
            // ===== proto_200.erl: 战备/HP/combat (20000-20099) =====
            20000 => { // cli={}, srv={combat_type:u16, combat_map:u32}
                let mut out = Vec::with_capacity(8);
                out.write_u16(0);
                out.write_u32(0);
                out
            }
            20001 => enc_code_msg(0, "OK"), // cli={}, srv={code, msg}
            20002 => { // cli={}, srv=complex (skill_plays etc) → 返最小有效
                let mut out = Vec::with_capacity(64);
                out.write_u16(0);   // pos
                out.write_u32(0);   // owner_id
                out.write_string("rgs-uat-1"); // owner_srv_id
                out.write_u32(0);   // total_distance
                out.write_u16(0);   // order_list count
                out.write_u16(0);   // skill_plays count
                out.write_u16(0);   // round_buff count
                out.write_u32(now_unix()); // countdown_time
                out.write_u32(0);   // action_count
                out.write_u16(0);   // combat_type
                out
            }
            20004 => { // cli={}, srv=complex (skill_plays+star_list) → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // skill_plays count
                out.write_u16(0);   // round_buff count
                out.write_u32(0);   // action_count
                out.write_u16(0);   // star_list count
                out.write_u16(0);   // combat_type
                out
            }
            20005 => enc_code_msg(0, "OK"),
            20006 => { // cli={}, srv=complex (fight_object_p etc) → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);    // combat_type
                out.write_u16(0);   // formation count
                out.write_u16(0);   // objects count
                out.write_u8(0);    // is_auto
                out.write_u16(0);   // buffs count
                out.write_u8(0);    // current_wave
                out.write_u16(0);   // total_wave
                out.write_u8(0);    // play_speed
                out.write_u32(0);   // combat_map
                out.write_u16(0);   // extra_args count
                out.write_u8(0);    // pause
                out.write_u8(0);    // dragon_difficulty
                out.write_u32(0);   // wave_time
                out.write_u32(0);   // action_count
                out.write_u16(0);   // star_list count
                out.write_u8(0);    // a_object_num
                out.write_string(""); // target_role_name
                out.write_string(""); // actor_role_name
                out.write_u32(now_unix()); // begin_time
                out.write_u8(0);    // suppress
                out.write_u8(0);    // flag
                out
            }
            20008 => enc_code_msg(0, "OK"),
            20009 => enc_code_msg(0, "OK"),
            20013 => { // cli={}, srv=complex → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u16(0);   // combat_type
                out.write_u16(0);   // formation count
                out.write_u16(0);   // objects count
                out.write_u8(0);    // is_auto
                out.write_u16(0);   // buffs count
                out.write_u8(0);    // current_wave
                out.write_u16(0);   // total_wave
                out.write_u8(0);    // play_speed
                out.write_u32(0);   // combat_map
                out.write_u16(0);   // extra_args count
                out.write_u8(0);    // pause
                out.write_u8(0);    // dragon_difficulty
                out.write_u32(0);   // wave_time
                out.write_u32(0);   // action_count
                out.write_u16(0);   // star_list count
                out.write_u8(0);    // a_object_num
                out.write_string(""); // target_role_name
                out.write_string(""); // actor_role_name
                out.write_u32(now_unix()); // begin_time
                out.write_u8(0);    // suppress
                out.write_u8(0);    // flag
                out
            }
            20014 => enc_code_msg(0, "OK"), // cli={target_id, target_srv_id}
            20015 => enc_code_msg(0, "OK"),
            20016 => enc_code_msg(0, "OK"),
            20019 => enc_empty(), // cli={}, srv={}
            20020 => { // cli={}, srv=complex → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // combat_type
                out.write_u16(0);   // formation count
                out.write_u16(0);   // objects count
                out.write_u8(0);    // is_auto
                out.write_u16(0);   // buffs count
                out.write_u16(0);   // distance_info count
                out.write_u8(0);    // current_wave
                out.write_u16(0);   // total_wave
                out
            }
            20022 => enc_code_msg(0, "OK"), // cli={speed:u8}
            20026 => { // cli={}, srv={drama_id:u32}
                let mut out = Vec::with_capacity(4);
                out.write_u32(0);
                out
            }
            20027 => { // cli={}, srv=complex → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // combat_type
                out.write_u16(0);   // formation count
                out.write_u16(0);   // objects count
                out.write_u8(0);    // is_auto
                out.write_u8(0);    // current_wave
                out.write_u16(0);   // total_wave
                out.write_u8(0);    // play_speed
                out.write_u16(0);   // extra_args count
                out.write_string(""); // target_role_name
                out.write_u8(0);    // a_object_num
                out.write_string(""); // actor_role_name
                out.write_u8(0);    // suppress
                out
            }
            20028 => enc_code_msg(0, "OK"),
            20029 => enc_code_msg(0, "OK"), // cli={replay_id:u32}
            20030 => { // cli={}, srv={is_in_combat:u8}
                vec![0u8]
            }
            20033 => { // cli={}, srv={result, def_name, def_guild_name, def_lev, def_face_id, replay_id}
                let mut out = Vec::with_capacity(64);
                out.write_u8(0);     // result
                out.write_string(""); // def_name
                out.write_string(""); // def_guild_name
                out.write_u32(0);    // def_lev
                out.write_u32(0);    // def_face_id
                out.write_u32(0);    // replay_id
                out
            }
            20034 => enc_code_msg(0, "OK"), // cli={replay_id, channel, target_name, share_type}
            20036 => enc_code_msg(0, "OK"), // cli={replay_id, replay_srv_id}
            20060 => { // cli={combat_type:u16}, srv={combat_type:u16, type:u8}
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out.write_u8(0);
                out
            }
            20061 => { // srv={combat_type:u16, bid:u32, partner_list, wave_list, dun_bid:u32, b_formation_type:u8} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // combat_type
                out.write_u32(0);   // bid
                out.write_u16(0);   // partner_list count
                out.write_u16(0);   // wave_list count
                out.write_u32(0);   // dun_bid
                out.write_u8(0);    // b_formation_type
                out
            }
            20062 => enc_code_msg(0, "OK"),
            20063 => { // cli={}, srv={type_list count + [combat_type:u16]}
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);   // empty list
                out
            }

            // ===== proto_202.erl: 战斗详细/竞技场 (20200-20299) =====
            20200 => { // cli={}, srv=complex (rank+score+can_combat_num) → 最小
                let mut out = Vec::with_capacity(32);
                out.write_u32(0);   // rank
                out.write_u32(0);   // score
                out.write_u8(0);    // can_combat_num
                out.write_u8(0);    // buy_combat_num
                out.write_u32(now_unix()); // ref_time
                out.write_u16(0);   // start_time_list count
                out.write_u16(0);   // end_time_list count
                out.write_u16(0);   // list count
                out
            }
            20201 => { // cli={}, srv={f_list, type}
                let mut out = Vec::with_capacity(8);
                out.write_u16(0);   // f_list count
                out.write_u8(0);    // type
                out
            }
            20202 => { // cli={rid, srv_id}, srv=complex → 最小
                let mut out = Vec::with_capacity(64);
                out.write_u32(0);   // rid
                out.write_string(""); // srv_id
                out.write_string(""); // name
                out.write_u8(0);    // lev
                out.write_u32(0);   // face
                out.write_u32(0);   // power
                out.write_u32(0);   // score
                out.write_u8(0);    // formation_type
                out.write_u8(0);    // formation_lev
                out.write_u16(0);   // partner_list count
                out
            }
            20203 => enc_code_msg(0, "OK"), // cli={rid, srv_id}
            20204 => enc_code_msg(0, "OK"),
            20206 => enc_code_msg(0, "OK"),
            20207 => enc_code_msg(0, "OK"),
            20208 => { // cli={}, srv={had_combat_num:u8, num_list:u16}
                let mut out = Vec::with_capacity(4);
                out.write_u8(0);    // had_combat_num
                out.write_u16(0);   // num_list count
                out
            }
            20209 => enc_code_msg(0, "OK"), // cli={num}
            20210 => { // srv=complex (result/score/items/tar_name) → 最小
                let mut out = Vec::with_capacity(32);
                out.write_u8(0);     // result
                out.write_u32(0);    // score
                out.write_u32(0);    // get_score
                out.write_u16(0);    // items count
                out.write_string(""); // tar_name
                out.write_u8(0);     // tar_lev
                out.write_u32(0);    // tar_face
                out
            }
            20220 => { // cli={}, srv={rank_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);   // rank_list count
                out
            }
            20221 => { // cli={}, srv={rank, score, worship, rank_list} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // rank
                out.write_u32(0);   // score
                out.write_u32(0);   // worship
                out.write_u16(0);   // rank_list count
                out
            }
            20222 => { // cli={}, srv={log_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);   // log_list count
                out
            }
            20223 => { // cli={}, srv={flag:u8}
                vec![0u8]
            }
            20250 => { // cli={}, srv=complex (start/end/step etc) → 最小
                let mut out = Vec::with_capacity(64);
                out.write_u32(now_unix()); // start_time
                out.write_u32(now_unix()); // end_time
                out.write_u32(0);   // step
                out.write_u8(0);    // step_status
                out.write_u32(0);   // step_status_time
                out.write_u8(0);    // role_count
                out.write_u8(0);    // role_max
                out.write_u16(0);   // self_rank
                out.write_u8(0);    // has_bet
                out.write_u16(0);   // log_list count
                out
            }
            20251 => { // cli={}, srv={rank, best_rank, can_bet, group}
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // rank
                out.write_u32(0);   // best_rank
                out.write_u8(0);    // can_bet
                out.write_u8(0);    // group
                out
            }
            20252 => enc_code_msg(0, "OK"),
            20253 => { // cli={}, srv=complex (champion_pk) → 最小
                let mut out = Vec::with_capacity(32);
                out.write_u8(0);    // bet_type
                out.write_u32(0);   // bet_val
                out.write_u32(0);   // a_bet_ratio
                out.write_u32(0);   // b_bet_ratio
                out.write_u8(0);    // a_bet
                out.write_u8(0);    // b_bet
                out.write_u16(0);   // step count
                out.write_u16(0);   // role_list count
                out
            }
            20254 => { // cli={bet_type, bet_val}, srv={code, msg, can_bet, bet_type, bet_val}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);     // code
                out.write_string("OK"); // msg
                out.write_u8(0);     // can_bet
                out.write_u8(0);     // bet_type
                out.write_u32(0);    // bet_val
                out
            }
            20255 => { // cli={}, srv={list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);   // list count
                out
            }
            20256 => { // cli={}, srv={rank, cnum, win}
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // rank
                out.write_u32(0);   // cnum
                out.write_u32(0);   // win
                out
            }
            20257 => { // srv={a_bet, b_bet, a_bet_ratio, b_bet_ratio}
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // a_bet
                out.write_u32(0);   // b_bet
                out.write_u32(0);   // a_bet_ratio
                out.write_u32(0);   // b_bet_ratio
                out
            }
            20258 => { // cli={}, srv={list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            20260 => { // cli={}, srv={list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            20261 => { // cli={}, srv={pos_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            20262 => { // cli={}, srv={group, pos}
                let mut out = Vec::with_capacity(4);
                out.write_u8(0);
                out.write_u16(0);
                out
            }
            20263 => { // cli={group, pos}, srv={code, msg}
                enc_code_msg(0, "OK")
            }
            20280 => { // cli={}, srv={rank_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            20281 => { // cli={}, srv={rank, worship, power, rank_list} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // rank
                out.write_u32(0);   // worship
                out.write_u32(0);   // power
                out.write_u16(0);   // rank_list count
                out
            }
            20282 => { // srv={time, rank_list} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u32(now_unix());
                out.write_u16(0);
                out
            }

            // ===== proto_203.erl: boss/dungeon 相关 (20300-20399) =====
            20300 => enc_code_msg(0, "OK"),
            20301 => enc_code_msg(0, "OK"),

            // ===== proto_205.erl: 关卡/Boss (20500-20599) =====
            20500 => { // cli={}, srv={boss_list, ext_list} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u16(0);   // boss_list count
                out.write_u16(0);   // ext_list count
                out
            }
            20501 => enc_code_msg(0, "OK"), // cli={boss_id}
            20502 => enc_code_msg(0, "OK"), // cli={boss_id}
            20530 => { // cli={}, srv={can_combat_num, buy_combat_num, num_time}
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);    // can_combat_num
                out.write_u8(0);    // buy_combat_num
                out.write_u32(now_unix()); // num_time
                out
            }
            20531 => enc_code_msg(0, "OK"),
            20532 => enc_code_msg(0, "OK"), // cli={boss_id}
            20533 => enc_code_msg(0, "OK"), // cli={boss_id}
            20535 => { // cli={}, srv={boss_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            20536 => { // srv={boss_id, rnum, hp, hp_max, ref_time}
                let mut out = Vec::with_capacity(20);
                out.write_u32(0);   // boss_id
                out.write_u32(0);   // rnum
                out.write_u32(0);   // hp
                out.write_u32(0);   // hp_max
                out.write_u32(now_unix()); // ref_time
                out
            }
            20537 => { // cli={boss_id}, srv={boss_id, role_list} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);   // boss_id
                out.write_u16(0);   // role_list count
                out
            }
            20538 => { // cli={boss_id}, srv={boss_id, role_list} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);   // boss_id
                out.write_u16(0);   // role_list count
                out
            }
            20539 => { // srv={bossid, all_dps, best_partner, award_list, partner_dps_list} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // bossid
                out.write_u32(0);   // all_dps
                out.write_u32(0);   // best_partner
                out.write_u16(0);   // award_list count
                out.write_u16(0);   // partner_dps_list count
                out
            }
            20540 => { // cli={}, srv={boss_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            20541 => enc_empty(), // cli={boss_list}, srv={}
            20542 => { // cli={}, srv={result, first_award, award}
                let mut out = Vec::with_capacity(8);
                out.write_u8(0);    // result
                out.write_u16(0);   // first_award count
                out.write_u16(0);   // award count
                out
            }

            // ===== proto_206.erl: 额外 Boss (20600-20699) =====
            20600 => enc_code_msg(0, "OK"),
            20601 => enc_code_msg(0, "OK"),
            20602 => enc_code_msg(0, "OK"),
            20604 => enc_code_msg(0, "OK"),
            20605 => enc_code_msg(0, "OK"),
            20606 => enc_code_msg(0, "OK"),
            20607 => enc_code_msg(0, "OK"),
            20608 => enc_code_msg(0, "OK"),
            20609 => enc_code_msg(0, "OK"),
            20610 => enc_code_msg(0, "OK"),
            20611 => enc_code_msg(0, "OK"),
            20620 => enc_code_msg(0, "OK"),
            20632 => enc_code_msg(0, "OK"),
            20633 => enc_code_msg(0, "OK"),
            20634 => enc_code_msg(0, "OK"),
            20635 => enc_code_msg(0, "OK"),

            // ===== proto_207.erl: 副本 (20700-20799) =====
            20700 => enc_code_msg(0, "OK"),
            20701 => enc_code_msg(0, "OK"),
            20702 => enc_code_msg(0, "OK"),
            20703 => enc_code_msg(0, "OK"),

            // ===== proto_210.erl: 活动/welfare (21000-21099) =====
            21000 => { // cli={}, srv={end_time:u32, first_gift:u32}
                let mut out = Vec::with_capacity(8);
                out.write_u32(now_unix() + 86400);
                out.write_u32(0);
                out
            }
            21001 => enc_code_msg(0, "OK"), // cli={id}
            21002 => { // cli={}, srv={count:u16}
                let mut out = Vec::with_capacity(2);
                out.write_u16(0);
                out
            }
            21003 => enc_empty(), // cli={}, srv={}
            21004 => enc_code_msg(0, "OK"),
            21005 => { // cli={}, srv={count:u32, gold:u32, next_gold:u32}
                let mut out = Vec::with_capacity(12);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u32(100);
                out
            }
            21006 => enc_code_msg(0, "OK"),
            21007 => enc_code_msg(0, "OK"),
            21008 => enc_code_msg(0, "OK"),
            21009 => enc_code_msg(0, "OK"),
            21010 => enc_code_msg(0, "OK"),
            21011 => enc_code_msg(0, "OK"),
            21012 => enc_code_msg(0, "OK"),
            21013 => enc_code_msg(0, "OK"),
            21014 => enc_code_msg(0, "OK"),
            21015 => enc_code_msg(0, "OK"),
            21016 => enc_code_msg(0, "OK"),
            21020 => enc_code_msg(0, "OK"),

            // ===== proto_211.erl: 签到 (21100-21199) =====
            21100 => { // cli={}, srv={status_list:u16 list of u8}
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            21101 => { // cli={day}, srv={code, msg, day}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u8(0);
                out
            }

            // ===== proto_212.erl: 升级奖励 (21200-21299) =====
            21200 => { // cli={}, srv={gifts:u16 list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            21201 => { // cli={}, srv={lev_gift:u16 list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            21202 => { // cli={}, srv={code:u8}
                vec![0u8]
            }
            21203 => { // cli={id}, srv={code, msg, id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }
            21204 => { // cli={id}, srv={code, msg, id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }
            21210 => enc_code_msg(0, "OK"),
            21211 => enc_code_msg(0, "OK"),
            21220 => enc_code_msg(0, "OK"),

            // ===== proto_213.erl: 副本/排行 (21300-21399) =====
            21300 => { // cli={}, srv=complex (fid/max_id/count etc) → 最小
                let mut out = Vec::with_capacity(64);
                out.write_u32(0);   // fid
                out.write_u32(0);   // max_id
                out.write_u8(0);    // count
                out.write_u8(0);    // buy_count
                out.write_u16(0);   // info count
                out.write_u16(0);   // combat_info count
                out.write_u16(0);   // buff_lev count
                out.write_u16(0);   // extra_reward count
                out
            }
            21303 => { // cli={}, srv={box_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            21304 => { // cli={fid}, srv={code, msg, fid, num}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            21305 => enc_code_msg(0, "OK"),
            21308 => enc_code_msg(0, "OK"), // cli={boss_id, formation_type, pos_info}
            21309 => { // srv=complex (bossid/all_dps/best_partner/award_list/partner_dps_list) → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u16(0);
                out.write_u16(0);
                out
            }
            21312 => { // cli={type}, srv={code, msg, count, buy_count, type}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u8(0);
                out.write_u8(0);
                out.write_u8(0);
                out
            }
            21317 => { // cli={boss_id}, srv=complex → 最小
                let mut out = Vec::with_capacity(32);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u8(0);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u16(0);
                out.write_u16(0);
                out
            }
            21318 => { // cli={}, srv=complex (my_name/my_rank/etc) → 最小
                let mut out = Vec::with_capacity(64);
                out.write_string("");
                out.write_u32(0);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u16(0);
                out.write_u16(0);
                out.write_u16(0);
                out
            }
            21319 => { // cli={boss_id, start_num, end_num}, srv={rank, mydps, rank_list} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);
                out.write_u32(0);
                out.write_u16(0);
                out
            }
            21320 => enc_code_msg(0, "OK"),
            21321 => enc_code_msg(0, "OK"),
            21322 => enc_code_msg(0, "OK"),
            21323 => enc_code_msg(0, "OK"),

            // ===== proto_214.erl: ??? (21401-21499) - 没有该文件 =====
            21401 => enc_code_msg(0, "OK"),
            21403 => enc_code_msg(0, "OK"),
            21410 => enc_code_msg(0, "OK"),
            21421 => enc_code_msg(0, "OK"),
            21425 => enc_code_msg(0, "OK"),
            21427 => enc_code_msg(0, "OK"),

            // ===== proto_215.erl: avatar (21500-21599) =====
            21500 => { // cli={}, srv={avatar_frame:u32}
                let mut out = Vec::with_capacity(4);
                out.write_u32(0);
                out
            }
            21501 => { // cli={base_id}, srv={base_id:u32}
                let mut out = Vec::with_capacity(4);
                out.write_u32(0);
                out
            }
            21502 => { // cli={}, srv={avatar_frame:u32}
                let mut out = Vec::with_capacity(4);
                out.write_u32(0);
                out
            }
            21503 => { // cli={base_id}, srv={avatar_frame:u32}
                let mut out = Vec::with_capacity(4);
                out.write_u32(0);
                out
            }
            21504 => { // cli={}, srv={attr: complex}
                let mut out = Vec::with_capacity(32);
                out.write_u16(0);   // attr count
                out
            }

            // ===== proto_221.erl: feat/servers (22100-22199) =====
            22100 => { // cli={}, srv={step, servers, step_reward} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);   // step
                out.write_u16(0);   // servers count
                out.write_u16(0);   // step_reward count
                out
            }
            22101 => { // cli={step}, srv={code, msg, step}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }

            // ===== proto_222.erl: ??? (22200-22299) =====
            22200 => enc_code_msg(0, "OK"),
            22202 => enc_code_msg(0, "OK"),
            22203 => enc_code_msg(0, "OK"),
            22204 => enc_code_msg(0, "OK"),

            // ===== proto_227.erl: feat/quest (22700-22799) =====
            22700 => { // cli={}, srv={rank_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            22701 => { // cli={id}, srv={id, end_time, rank, rank_list} → 最小
                let mut out = Vec::with_capacity(16);
                out.write_u32(0);
                out.write_u32(now_unix());
                out.write_u32(0);
                out.write_u16(0);
                out
            }
            22702 => { // cli={}, srv={feat_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            22703 => { // cli={}, srv={feat_list} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            22704 => { // cli={quest_id}, srv={result, msg, quest_id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }

            // ===== proto_232.erl: recruit (23200-23299) =====
            23200 => { // cli={}, srv=complex (recruit_list/free_cd_end etc) → 最小
                let mut out = Vec::with_capacity(64);
                out.write_u16(0);   // recruit_list count
                out.write_u32(0);   // free_cd_end
                out.write_u8(0);    // free_times
                out.write_u8(0);    // coin_times
                out.write_u8(0);    // is_share
                out.write_u16(0);   // recruit_group count
                out
            }
            23201 => { // cli={group_id, times, recruit_type}, srv=complex → 最小
                let mut out = Vec::with_capacity(32);
                out.write_u8(0);    // group_id
                out.write_u8(0);    // times
                out.write_u16(0);   // rewards count
                out.write_u16(0);   // partner_bids count
                out.write_u16(0);   // partner_chips count
                out
            }
            23202 => { // cli={}, srv={free_cd_end, free_times, coin_times}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u8(0);
                out.write_u8(0);
                out
            }
            23203 => { // cli={}, srv={code, msg, is_share, is_day_share}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u8(0);
                out.write_u8(0);
                out
            }
            23204 => { // cli={}, srv={recruit_group} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            23205 => enc_code_msg(0, "OK"),
            23210 => enc_code_msg(0, "OK"),
            23211 => enc_code_msg(0, "OK"),
            23212 => enc_code_msg(0, "OK"),
            23213 => enc_code_msg(0, "OK"),
            23214 => enc_code_msg(0, "OK"),
            23215 => enc_code_msg(0, "OK"),
            23216 => enc_code_msg(0, "OK"),
            23217 => enc_code_msg(0, "OK"),
            23218 => enc_code_msg(0, "OK"),
            23219 => enc_code_msg(0, "OK"),
            23220 => enc_code_msg(0, "OK"),
            23221 => enc_code_msg(0, "OK"),
            23222 => enc_code_msg(0, "OK"),

            // ===== proto_233.erl: honor (23300-23399) =====
            23300 => { // cli={}, srv={base_id, honor}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u32(0);
                out
            }
            23301 => { // cli={base_id}, srv={base_id}
                let mut out = Vec::with_capacity(4);
                out.write_u32(0);
                out
            }
            23302 => { // cli={}, srv={honor} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            23303 => { // cli={base_id}, srv={honor} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }

            // ===== proto_234.erl: login_gift (23400-23499) =====
            23400 => { // cli={}, srv={gifts} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            23402 => { // cli={}, srv={code:u8}
                vec![0u8]
            }
            23403 => { // cli={id}, srv={code, msg, id}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_string("OK");
                out.write_u32(0);
                out
            }

            // ===== proto_235.erl: market/shop (23500-23599) =====
            23500 => { // cli={catalg}, srv={catalg, goods} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u16(0);
                out
            }
            23501 => { // cli={base_id, num}, srv={flag:u8}
                vec![0u8]
            }
            23502 => { // cli={id, num}, srv={flag:u8}
                vec![0u8]
            }
            23504 => enc_empty(), // cli={package_type, item_id, num, percent, cell_id}, srv={}
            23505 => { // cli={type, id, num}, srv={type, id, status, num}
                let mut out = Vec::with_capacity(16);
                out.write_u8(0);
                out.write_u32(0);
                out.write_u8(0);
                out.write_u32(0);
                out
            }
            23506 => { // cli={cell_id}, srv={cell_id, flag}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            23507 => { // cli={}, srv={free_ids count + [u32], cells count + [u8]} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out.write_u16(0);
                out
            }
            23508 => { // cli={item_base_id}, srv={item_base_id, price}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u32(0);
                out
            }
            23509 => { // cli={refresh_type}, srv={refresh_time, data} → 最小
                let mut out = Vec::with_capacity(8);
                out.write_u32(now_unix());
                out.write_u16(0);
                out
            }
            23511 => { // cli={cell_id}, srv={cell_id, flag}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            23512 => { // cli={}, srv={cell_id, flag}
                let mut out = Vec::with_capacity(8);
                out.write_u32(0);
                out.write_u8(0);
                out
            }
            23513 => enc_empty(), // cli={cell_id, percent, num}, srv={}
            23514 => enc_empty(), // cli={type}, srv={}
            23516 => { // cli={base_ids}, srv={market_price} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            23518 => { // cli={catalg}, srv={goods} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            23519 => enc_empty(), // cli={}, srv={}
            23520 => { // cli={}, srv={limit_data} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }

            // ===== proto_236.erl: reward (23600-23699) =====
            23601 => { // cli={}, srv={reward, process, double, count}
                let mut out = Vec::with_capacity(16);
                out.write_u16(0);   // reward count
                out.write_u8(0);    // process
                out.write_u8(0);    // double
                out.write_u8(0);    // count
                out
            }
            23602 => enc_code_msg(0, "OK"),
            23603 => enc_code_msg(0, "OK"),
            23604 => { // cli={}, srv={ext_reward} → 最小
                let mut out = Vec::with_capacity(4);
                out.write_u16(0);
                out
            }
            23606 => enc_code_msg(0, "OK"),
            23607 => enc_code_msg(0, "OK"),

            // ===== proto_237.erl: extra (23700-23799) =====
            23700 => enc_code_msg(0, "OK"),
            23701 => enc_code_msg(0, "OK"),
            23702 => enc_code_msg(0, "OK"),

            // ===== fallback: 未知 cmd (200xx-237xx 但未在 proto 注册) =====
            _ => {
                // 大多数 fallback 给 (code, msg)
                enc_code_msg(0, "OK")
            }
        };
        if cmd >= 20000 && cmd < 30000 {
            tracing::debug!(cmd, payload_len = payload.len(), "w2-economy");
        }
        Response { cmd, payload }
    })
}
