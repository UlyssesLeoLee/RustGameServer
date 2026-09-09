// Cmd registry (per 9/9 13:50 JST v0.2 模式, Rust 重写 v0.3.0)
// 6 个内置 cmd; 后续 worker 扩自己域时, 加新表项 + handler 函数
//
// 设计: handler 取 owned Vec<u8> + Arc<RgsClient> (clone) + cmd,
// 返回 Pin<Box<dyn Future + Send>> 不绑 lifetime, 避免 HRTB 复杂度
// & self 的 lifetime 也不进 future (entry.handler 是 fn pointer, 不是闭包)
//
// v0.3.1 (per 2026-09-09 14:55 JST Ulysses 拍板):
// - 10101 / 10102 / 10103 / 10200 来自 zsyz_server proto_101.erl + proto_102.erl (真 zsyz cmd)
// - 10400 / 11001 是 shim-internal RGS 测试 cmd (Erlang 10400=quest_list, 11001=partner_list, 不复用)
// - 业务覆盖率 1.2% (4/514 real cmd), 1-2 周 4 worker 扩 (per 9/9 13:45 JST 拍板 A)
//
// v0.3.2 (per 2026-09-09 15:10 JST Ulysses 拍板 "重测直到战斗场景"):
// - 战斗场景 cmd 6 个: 10215 move / 10300 ping / 10301 role_info / 10302 assets / 10309 signature / 10315 view_role
// - 来源: zsyz_server proto_102.erl (10215) + proto_103.erl (10300/10301/10302/10309/10315)
// - 业务覆盖率 1.9% (10/514 real cmd)

use crate::handlers;
use crate::handlers_social;
use crate::handlers_w2;
use crate::handlers_w2_extra;
use crate::rgs::RgsClient;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

// 注意: 返回 future 不能 bind lifetime, 所以 future bound = 'static
// (handler 内部 clone Arc, 不持有外部引用)
pub type AsyncHandler = fn(
    u16,                 // cmd
    Vec<u8>,             // payload owned
    Arc<RgsClient>,      // rgs client shared
) -> Pin<Box<dyn Future<Output = handlers::Response> + Send>>;

pub struct CmdEntry {
    pub handler: AsyncHandler,
    #[allow(dead_code)]
    pub name: &'static str,
    /// 来源: "zsyz" = 真 zsyz_client cmd (per proto_*.erl); "shim" = shim-internal RGS 测试
    #[allow(dead_code)]
    pub source: &'static str,
}

pub struct Registry {
    map: std::collections::HashMap<u16, CmdEntry>,
}

impl Registry {
    pub fn new() -> Self {
        let mut map = std::collections::HashMap::new();
        // 真 zsyz_client cmd (per zsyz_server/src/proto/proto_101.erl + proto_102.erl)
        map.insert(10101, CmdEntry { handler: handlers::handle_register, name: "register", source: "zsyz" });
        map.insert(10102, CmdEntry { handler: handlers::handle_enter_server, name: "enter_server", source: "zsyz" });
        map.insert(10103, CmdEntry { handler: handlers::handle_enter_server, name: "enter_server (alias)", source: "zsyz" });
        map.insert(10200, CmdEntry { handler: handlers::handle_map_enter, name: "map_enter", source: "zsyz" });
        // 战斗场景 cmd (v0.3.2, per 2026-09-09 15:10 JST Ulysses 拍板 "重测直到战斗场景")
        // 来源: proto_102.erl + proto_103.erl
        map.insert(10215, CmdEntry { handler: handlers::handle_move, name: "move (RGS match SubmitMove)", source: "zsyz" });
        map.insert(10300, CmdEntry { handler: handlers::handle_ping, name: "ping (empty payload)", source: "zsyz" });
        map.insert(10301, CmdEntry { handler: handlers::handle_role_info, name: "role_info (RGS player.GetPlayer)", source: "zsyz" });
        map.insert(10302, CmdEntry { handler: handlers::handle_assets, name: "assets (RGS economy.GetAccount)", source: "zsyz" });
        map.insert(10309, CmdEntry { handler: handlers::handle_signature, name: "signature (set/edit)", source: "zsyz" });
        map.insert(10315, CmdEntry { handler: handlers::handle_view_role, name: "view_role (RGS player+social)", source: "zsyz" });
        // shim-internal RGS 测试 cmd (Erlang 10400=quest_list/11001=partner_list, 不复用)
        map.insert(10400, CmdEntry { handler: handlers::handle_heartbeat, name: "heartbeat (RGS 5 域 HealthCheck)", source: "shim" });
        map.insert(11001, CmdEntry { handler: handlers::handle_role_list, name: "role_list (RGS player ListPlayers)", source: "shim" });

        // v0.5.0 (per 2026-09-09 19:32 JST Mavis 派工 w1): 53 个 player 域 stub 替换为 real handler
        // 来源: zsyz_server/src/proto/proto_103.erl + proto_104.erl + proto_105.erl + proto_108.erl + proto_109.erl
        // 业务覆盖: player 域 65/65 (100%) — 12 已有 + 53 新增
        // 注: or_insert 语义, 这里先 insert 优先; stubs 阶段仍保留, 但会被覆盖
        map.insert(10312, CmdEntry { handler: handlers::handle_10312, name: "10312 empty (RGS)", source: "zsyz" });
        map.insert(10316, CmdEntry { handler: handlers::handle_10316, name: "10316 view_role_idx", source: "zsyz" });
        map.insert(10317, CmdEntry { handler: handlers::handle_10317, name: "10317 worship", source: "zsyz" });
        map.insert(10318, CmdEntry { handler: handlers::handle_10318, name: "10318 friend_list", source: "zsyz" });
        map.insert(10322, CmdEntry { handler: handlers::handle_10322, name: "10322 ping_xx", source: "zsyz" });
        map.insert(10325, CmdEntry { handler: handlers::handle_10325, name: "10325 face_list", source: "zsyz" });
        map.insert(10327, CmdEntry { handler: handlers::handle_10327, name: "10327 set_face", source: "zsyz" });
        map.insert(10343, CmdEntry { handler: handlers::handle_10343, name: "10343 rename", source: "zsyz" });
        map.insert(10345, CmdEntry { handler: handlers::handle_10345, name: "10345 bag_use_list", source: "zsyz" });
        map.insert(10346, CmdEntry { handler: handlers::handle_10346, name: "10346 del_item", source: "zsyz" });
        map.insert(10347, CmdEntry { handler: handlers::handle_10347, name: "10347 asset_icons", source: "zsyz" });
        map.insert(10348, CmdEntry { handler: handlers::handle_10348, name: "10348 power (RGS player.GetPlayer)", source: "zsyz" });
        map.insert(10380, CmdEntry { handler: handlers::handle_10380, name: "10380 reg_day", source: "zsyz" });
        map.insert(10391, CmdEntry { handler: handlers::handle_10391, name: "10391 chat_send", source: "zsyz" });
        map.insert(10395, CmdEntry { handler: handlers::handle_10395, name: "10395 notify", source: "zsyz" });
        map.insert(10397, CmdEntry { handler: handlers::handle_10397, name: "10397 online_status", source: "zsyz" });
        map.insert(10399, CmdEntry { handler: handlers::handle_10399, name: "10399 feedback", source: "zsyz" });
        // quest 域
        map.insert(10402, CmdEntry { handler: handlers::handle_10402, name: "10402 accept_quest (RGS player.UpdateProfile)", source: "zsyz" });
        map.insert(10405, CmdEntry { handler: handlers::handle_10405, name: "10405 finish_quest", source: "zsyz" });
        map.insert(10406, CmdEntry { handler: handlers::handle_10406, name: "10406 giveup_quest", source: "zsyz" });
        // 物品/装备 域
        map.insert(10500, CmdEntry { handler: handlers::handle_10500, name: "10500 bag_list", source: "zsyz" });
        map.insert(10501, CmdEntry { handler: handlers::handle_10501, name: "10501 equip_list", source: "zsyz" });
        map.insert(10515, CmdEntry { handler: handlers::handle_10515, name: "10515 use_item", source: "zsyz" });
        map.insert(10520, CmdEntry { handler: handlers::handle_10520, name: "10520 move_item", source: "zsyz" });
        map.insert(10522, CmdEntry { handler: handlers::handle_10522, name: "10522 batch_use", source: "zsyz" });
        map.insert(10523, CmdEntry { handler: handlers::handle_10523, name: "10523 sell_item", source: "zsyz" });
        map.insert(10524, CmdEntry { handler: handlers::handle_10524, name: "10524 star_up", source: "zsyz" });
        map.insert(10525, CmdEntry { handler: handlers::handle_10525, name: "10525 star_list", source: "zsyz" });
        map.insert(10526, CmdEntry { handler: handlers::handle_10526, name: "10526 storage_info", source: "zsyz" });
        map.insert(10528, CmdEntry { handler: handlers::handle_10528, name: "10528 exp_pool", source: "zsyz" });
        map.insert(10535, CmdEntry { handler: handlers::handle_10535, name: "10535 bag_clear", source: "zsyz" });
        map.insert(10536, CmdEntry { handler: handlers::handle_10536, name: "10536 equip_refresh", source: "zsyz" });
        // 邮件 域
        map.insert(10800, CmdEntry { handler: handlers::handle_10800, name: "10800 mail_list", source: "zsyz" });
        map.insert(10801, CmdEntry { handler: handlers::handle_10801, name: "10801 mail_read", source: "zsyz" });
        map.insert(10802, CmdEntry { handler: handlers::handle_10802, name: "10802 mail_unread", source: "zsyz" });
        map.insert(10804, CmdEntry { handler: handlers::handle_10804, name: "10804 mail_delete", source: "zsyz" });
        map.insert(10805, CmdEntry { handler: handlers::handle_10805, name: "10805 mail_attach", source: "zsyz" });
        map.insert(10810, CmdEntry { handler: handlers::handle_10810, name: "10810 mail_issue", source: "zsyz" });
        // 10900-10999 杂项
        map.insert(10900, CmdEntry { handler: handlers::handle_10900, name: "10900 silence", source: "zsyz" });
        map.insert(10901, CmdEntry { handler: handlers::handle_10901, name: "10901 ban", source: "zsyz" });
        map.insert(10902, CmdEntry { handler: handlers::handle_10902, name: "10902 stop_role", source: "zsyz" });
        map.insert(10905, CmdEntry { handler: handlers::handle_10905, name: "10905 battle_state", source: "zsyz" });
        map.insert(10906, CmdEntry { handler: handlers::handle_10906, name: "10906 ping_state", source: "zsyz" });
        map.insert(10922, CmdEntry { handler: handlers::handle_10922, name: "10922 activity_list", source: "zsyz" });
        map.insert(10923, CmdEntry { handler: handlers::handle_10923, name: "10923 activity_join", source: "zsyz" });
        map.insert(10924, CmdEntry { handler: handlers::handle_10924, name: "10924 activity_my", source: "zsyz" });
        map.insert(10925, CmdEntry { handler: handlers::handle_10925, name: "10925 activity_query", source: "zsyz" });
        map.insert(10926, CmdEntry { handler: handlers::handle_10926, name: "10926 activity_open", source: "zsyz" });
        map.insert(10927, CmdEntry { handler: handlers::handle_10927, name: "10927 activity_end", source: "zsyz" });
        map.insert(10945, CmdEntry { handler: handlers::handle_10945, name: "10945 giftcard", source: "zsyz" });
        map.insert(10946, CmdEntry { handler: handlers::handle_10946, name: "10946 giftcard_status", source: "zsyz" });
        map.insert(10950, CmdEntry { handler: handlers::handle_10950, name: "10950 board", source: "zsyz" });
        map.insert(10952, CmdEntry { handler: handlers::handle_10952, name: "10952 board_action", source: "zsyz" });
        map.insert(10955, CmdEntry { handler: handlers::handle_10955, name: "10955 sdk_ping", source: "zsyz" });
        map.insert(10956, CmdEntry { handler: handlers::handle_10956, name: "10956 sdk_ack", source: "zsyz" });
        map.insert(10999, CmdEntry { handler: handlers::handle_10999, name: "10999 sdk_notify", source: "zsyz" });

        // v0.4.0 (per 2026-09-09 16:25 JST Mavis 派工): 自动注册 766 全 zsyz send cmd stub
        // 来源: H5 zsyz_client proto_mate.js 提取, 域分布 welfare=110/partner=108/battle=103/social=93/...
        // stub 返回空 payload (后续 worker 派工逐个替换为 real handler)
        // 注: w1 已替换 53 player 域 cmd, 这里只注册 w2-w5 范围的 stub (partner/battle/social/welfare/admin)
        crate::registry_stubs::register_stubs(&mut map);
        // v0.4.1 (per 2026-09-09 19:30 JST Mavis 派工 w4): social 域 14 POC handler
        // 来源: proto_130.erl (dungeon 4) + proto_133.erl (friend 4) + proto_134.erl (exchange 2) + proto_135.erl (guild 4)
        // 完整 93 cmd 扩需 1-2 周, 当前 14 handler 字节级对齐 erlang pack(srv, ...)
        // 已知缺口 (per 9/9 19:30 派工): 79/93 cmd 仍 stub (13001-13004/13007-13040 副本 + 13301-13334 好友 + 13402-13420 兑换 + 13501-13576 公会 + 13601-13608 + 16601-16900 跨服)
        map.insert(13000, CmdEntry { handler: handlers_social::handle_13000_dungeon_list, name: "dungeon_list (proto_130)", source: "zsyz" });
        map.insert(13005, CmdEntry { handler: handlers_social::handle_13005_dungeon_battle, name: "dungeon_battle (proto_130)", source: "zsyz" });
        map.insert(13006, CmdEntry { handler: handlers_social::handle_13006_dungeon_count, name: "dungeon_count (proto_130)", source: "zsyz" });
        map.insert(13011, CmdEntry { handler: handlers_social::handle_13011_buff_list, name: "buff_list (proto_130)", source: "zsyz" });
        map.insert(13300, CmdEntry { handler: handlers_social::handle_13300_friend_list, name: "friend_list (proto_133)", source: "zsyz" });
        map.insert(13303, CmdEntry { handler: handlers_social::handle_13303_add_friend, name: "add_friend (proto_133)", source: "zsyz" });
        map.insert(13311, CmdEntry { handler: handlers_social::handle_13311_friend_req_list, name: "friend_req_list (proto_133)", source: "zsyz" });
        map.insert(13315, CmdEntry { handler: handlers_social::handle_13315_delete_friend, name: "delete_friend (proto_133)", source: "zsyz" });
        map.insert(13401, CmdEntry { handler: handlers_social::handle_13401_exchange_list, name: "exchange_list (proto_134)", source: "zsyz" });
        map.insert(13408, CmdEntry { handler: handlers_social::handle_13408_exchange_action, name: "exchange_action (proto_134)", source: "zsyz" });
        map.insert(13500, CmdEntry { handler: handlers_social::handle_13500_create_guild, name: "create_guild (proto_135)", source: "zsyz" });
        map.insert(13518, CmdEntry { handler: handlers_social::handle_13518_guild_info, name: "guild_info (proto_135, RGS social.GetGuild)", source: "zsyz" });
        map.insert(13519, CmdEntry { handler: handlers_social::handle_13519_guild_members, name: "guild_members (proto_135)", source: "zsyz" });
        map.insert(13523, CmdEntry { handler: handlers_social::handle_13523_donate_info, name: "donate_info (proto_135)", source: "zsyz" });

        // Phase 4 w3 (per 2026-09-09 19:32 JST Mavis 派工): battle 域 66 cmd 真实 handler
        // 范围: 19800-19807 + 19901-19908 (战斗/录像, 15) + 25100-25841 (任务/成就/城市/矿脉, 51)
        // 覆盖 stub-19800..stub-19908 + stub-25100..stub-25841
        // --- 战斗结果 / 录像 (19800-19807, 19901-19908) ---
        map.insert(19800, CmdEntry { handler: handlers::handle_battle_result, name: "battle_result", source: "zsyz" });
        map.insert(19801, CmdEntry { handler: handlers::handle_battle_result_ack, name: "battle_result_ack", source: "zsyz" });
        map.insert(19802, CmdEntry { handler: handlers::handle_replay_list, name: "replay_list", source: "zsyz" });
        map.insert(19804, CmdEntry { handler: handlers::handle_replay_rewards, name: "replay_rewards", source: "zsyz" });
        map.insert(19805, CmdEntry { handler: handlers::handle_claim_replay_reward, name: "claim_replay_reward", source: "zsyz" });
        map.insert(19806, CmdEntry { handler: handlers::handle_battle_status, name: "battle_status", source: "zsyz" });
        map.insert(19807, CmdEntry { handler: handlers::handle_battle_result, name: "battle_opponent", source: "zsyz" });
        map.insert(19901, CmdEntry { handler: handlers::handle_replay_query, name: "replay_query", source: "zsyz" });
        map.insert(19902, CmdEntry { handler: handlers::handle_replay_paged_query, name: "replay_paged_query", source: "zsyz" });
        map.insert(19903, CmdEntry { handler: handlers::handle_replay_like, name: "replay_like", source: "zsyz" });
        map.insert(19904, CmdEntry { handler: handlers::handle_replay_query, name: "replay_op", source: "zsyz" });
        map.insert(19905, CmdEntry { handler: handlers::handle_replay_share, name: "replay_share", source: "zsyz" });
        map.insert(19906, CmdEntry { handler: handlers::handle_replay_like_count, name: "replay_like_count", source: "zsyz" });
        map.insert(19907, CmdEntry { handler: handlers::handle_replay_hero, name: "replay_hero", source: "zsyz" });
        map.insert(19908, CmdEntry { handler: handlers::handle_replay_detail, name: "replay_detail", source: "zsyz" });
        // --- 日常任务 (25100-25102) ---
        map.insert(25100, CmdEntry { handler: handlers::handle_daily_quest, name: "daily_quest", source: "zsyz" });
        map.insert(25101, CmdEntry { handler: handlers::handle_daily_quest_claim, name: "daily_quest_claim", source: "zsyz" });
        map.insert(25102, CmdEntry { handler: handlers::handle_daily_quest_flag, name: "daily_quest_flag", source: "zsyz" });
        // --- 月卡/周卡 (25300-25309) ---
        map.insert(25300, CmdEntry { handler: handlers::handle_card_state, name: "card_state", source: "zsyz" });
        map.insert(25301, CmdEntry { handler: handlers::handle_card_list, name: "card_list", source: "zsyz" });
        map.insert(25302, CmdEntry { handler: handlers::handle_card_op, name: "card_op", source: "zsyz" });
        map.insert(25303, CmdEntry { handler: handlers::handle_card_reward, name: "card_reward", source: "zsyz" });
        map.insert(25304, CmdEntry { handler: handlers::handle_card_claim, name: "card_claim", source: "zsyz" });
        map.insert(25305, CmdEntry { handler: handlers::handle_card_exp, name: "card_exp", source: "zsyz" });
        map.insert(25306, CmdEntry { handler: handlers::handle_card_gift, name: "card_gift", source: "zsyz" });
        map.insert(25307, CmdEntry { handler: handlers::handle_card_misc, name: "card_misc_07", source: "zsyz" });
        map.insert(25308, CmdEntry { handler: handlers::handle_card_misc, name: "card_misc_08", source: "zsyz" });
        map.insert(25309, CmdEntry { handler: handlers::handle_card_misc, name: "card_misc_09", source: "zsyz" });
        // --- 竞技场/挑战 (25400-25414) ---
        map.insert(25400, CmdEntry { handler: handlers::handle_arena_state, name: "arena_state", source: "zsyz" });
        map.insert(25401, CmdEntry { handler: handlers::handle_arena_ext, name: "arena_ext", source: "zsyz" });
        map.insert(25402, CmdEntry { handler: handlers::handle_arena_buy, name: "arena_buy", source: "zsyz" });
        map.insert(25403, CmdEntry { handler: handlers::handle_arena_reward, name: "arena_reward", source: "zsyz" });
        map.insert(25404, CmdEntry { handler: handlers::handle_arena_simple, name: "arena_simple_04", source: "zsyz" });
        map.insert(25405, CmdEntry { handler: handlers::handle_arena_battle, name: "arena_battle", source: "zsyz" });
        map.insert(25410, CmdEntry { handler: handlers::handle_arena_round, name: "arena_round", source: "zsyz" });
        map.insert(25411, CmdEntry { handler: handlers::handle_arena_buy_round, name: "arena_buy_round", source: "zsyz" });
        map.insert(25412, CmdEntry { handler: handlers::handle_arena_simple, name: "arena_simple_12", source: "zsyz" });
        map.insert(25413, CmdEntry { handler: handlers::handle_arena_simple, name: "arena_simple_13", source: "zsyz" });
        map.insert(25414, CmdEntry { handler: handlers::handle_arena_partner_list, name: "arena_partner_list", source: "zsyz" });
        // --- 城市/荣誉 (25800-25807) ---
        map.insert(25800, CmdEntry { handler: handlers::handle_city_enter, name: "city_enter", source: "zsyz" });
        map.insert(25801, CmdEntry { handler: handlers::handle_city_op, name: "city_op", source: "zsyz" });
        map.insert(25802, CmdEntry { handler: handlers::handle_city_rank, name: "city_rank", source: "zsyz" });
        map.insert(25805, CmdEntry { handler: handlers::handle_honor_set, name: "honor_set", source: "zsyz" });
        map.insert(25806, CmdEntry { handler: handlers::handle_honor_get, name: "honor_get", source: "zsyz" });
        map.insert(25807, CmdEntry { handler: handlers::handle_honor_default, name: "honor_default", source: "zsyz" });
        // --- 成就 (25810-25820) ---
        map.insert(25810, CmdEntry { handler: handlers::handle_achievement_list, name: "achievement_list_10", source: "zsyz" });
        map.insert(25811, CmdEntry { handler: handlers::handle_achievement_list, name: "achievement_list_11", source: "zsyz" });
        map.insert(25812, CmdEntry { handler: handlers::handle_achievement_claim, name: "achievement_claim", source: "zsyz" });
        map.insert(25813, CmdEntry { handler: handlers::handle_achievement_view, name: "achievement_view", source: "zsyz" });
        map.insert(25814, CmdEntry { handler: handlers::handle_achievement_simple, name: "achievement_simple_14", source: "zsyz" });
        map.insert(25815, CmdEntry { handler: handlers::handle_achievement_simple, name: "achievement_simple_15", source: "zsyz" });
        map.insert(25816, CmdEntry { handler: handlers::handle_achievement_share, name: "achievement_share_16", source: "zsyz" });
        map.insert(25817, CmdEntry { handler: handlers::handle_achievement_share_op, name: "achievement_share_op", source: "zsyz" });
        map.insert(25818, CmdEntry { handler: handlers::handle_achievement_share, name: "achievement_share_18", source: "zsyz" });
        map.insert(25819, CmdEntry { handler: handlers::handle_achievement_share_query, name: "achievement_share_query", source: "zsyz" });
        map.insert(25820, CmdEntry { handler: handlers::handle_achievement_share_reward, name: "achievement_share_reward", source: "zsyz" });
        // --- 矿脉/BBS (25830-25841) ---
        map.insert(25830, CmdEntry { handler: handlers::handle_room_grow, name: "room_grow", source: "zsyz" });
        map.insert(25831, CmdEntry { handler: handlers::handle_room_op, name: "room_op_31", source: "zsyz" });
        map.insert(25832, CmdEntry { handler: handlers::handle_room_other, name: "room_other", source: "zsyz" });
        map.insert(25835, CmdEntry { handler: handlers::handle_bbs_send, name: "bbs_send_35", source: "zsyz" });
        map.insert(25836, CmdEntry { handler: handlers::handle_bbs_send, name: "bbs_send_36", source: "zsyz" });
        map.insert(25837, CmdEntry { handler: handlers::handle_bbs_list, name: "bbs_list", source: "zsyz" });
        map.insert(25838, CmdEntry { handler: handlers::handle_bbs_delete, name: "bbs_delete", source: "zsyz" });
        map.insert(25839, CmdEntry { handler: handlers::handle_bbs_type, name: "bbs_type", source: "zsyz" });
        map.insert(25840, CmdEntry { handler: handlers::handle_bbs_praise, name: "bbs_praise", source: "zsyz" });
        map.insert(25841, CmdEntry { handler: handlers::handle_bbs_full, name: "bbs_full", source: "zsyz" });

        // v0.4.2 (per 2026-09-09 20:30 JST Mavis 派工 w2 续做): 110 welfare + 其它 (proto_238 guild_shipping + proto_239 tournament)
        // 范围: cmd 23800-24818 (238xx proto_238 押镖 13 cmd + 239xx proto_239 试炼 12 cmd + 24000-24818 welfare 110 cmd, 总 135 cmd)
        // 来源: zsyz_server/src/proto/proto_238.erl (guild_shipping 13) + proto_239.erl (tournament 12) + zsyz_client mod/welfare (110, 无服务端 proto)
        // 实现: handlers_w2_extra::handle_w2_extra 单 dispatcher, 135 cmd 字节级对齐 erlang pack(srv, ...) simplified
        // 业务覆盖: w2 economy 总 135 cmd (110 welfare + 25 其它), 总 ~345 cmd (200 + 110 + 25 + 10 之前)
        for cmd in 23800u16..=24818u16 {
            if map.contains_key(&cmd) {
                map.insert(cmd, crate::registry::CmdEntry {
                    handler: handlers_w2_extra::handle_w2_extra,
                    name: match cmd {
                        // proto_238 guild_shipping (13 cmd)
                        23800 => "guild_shipping_list",
                        23801 => "guild_shipping_info",
                        23802 => "guild_shipping_action",
                        23803 => "guild_shipping_status",
                        23804 => "guild_shipping_simple",
                        23805 => "guild_shipping_order",
                        23806 => "guild_shipping_assist_list",
                        23807 => "guild_shipping_assist",
                        23808 => "guild_shipping_item",
                        23809 => "guild_shipping_double",
                        23810 => "guild_shipping_claim",
                        23811 => "guild_shipping_state",
                        23812 => "guild_shipping_done",
                        // proto_239 tournament (12 cmd)
                        23900 => "tournament_info",
                        23901 => "tournament_formation",
                        23902 => "tournament_state",
                        23903 => "tournament_id_status",
                        23904 => "tournament_action",
                        23905 => "tournament_list_05",
                        23906 => "tournament_list_06",
                        23907 => "tournament_list_07",
                        23908 => "tournament_claim",
                        23909 => "tournament_friend",
                        23910 => "tournament_select",
                        23911 => "tournament_buff",
                        // welfare 110 cmd (24000-24818)
                        24000 => "welfare_plunder_list",
                        24001 => "welfare_plunder_op",
                        24002 => "welfare_status",
                        24003 => "welfare_notify",
                        24004 => "welfare_query",
                        24005 => "welfare_plunder_my",
                        24006 => "welfare_plunder_status",
                        24010 => "welfare_plunder_target",
                        24011 => "welfare_plunder_result",
                        24012 => "welfare_reward",
                        24013 => "welfare_logs",
                        24014 => "welfare_log_detail",
                        24015 => "welfare_op",
                        24017 => "welfare_id_op",
                        24018 => "welfare_replay",
                        24019 => "welfare_plunder_query",
                        24020 => "welfare_code",
                        24100 => "welfare_hallows_list",
                        24101 => "welfare_hallows_up",
                        24103 => "welfare_hallows_use",
                        24104 => "welfare_hallows_compose",
                        24107 => "welfare_hallows_info",
                        24108 => "welfare_hallows_status",
                        24120 => "welfare_hallows_notify",
                        24121 => "welfare_hallows_action",
                        24122 => "welfare_hallows_get",
                        24123 => "welfare_hallows_view",
                        24124 => "welfare_hallows_empty",
                        24125 => "welfare_hallows_state",
                        24126 => "welfare_hallows_flag",
                        24127 => "welfare_hallows_end",
                        24128 => "welfare_hallows_cd",
                        24129 => "welfare_hallows_reward",
                        24130 => "welfare_hallows_claim",
                        24131 => "welfare_hallows_query",
                        24132 => "welfare_hallows_exchange",
                        24133 => "welfare_hallows_done",
                        24200 => "welfare_hp_notify",
                        24201 => "welfare_hp_pos",
                        24202 => "welfare_hp_status",
                        24204 => "welfare_hp_action",
                        24205 => "welfare_hp_query",
                        24206 => "welfare_hp_done",
                        24207 => "welfare_hp_cd",
                        24208 => "welfare_hp_reward",
                        24209 => "welfare_hp_target",
                        24210 => "welfare_hp_battle",
                        24212 => "welfare_hp_log",
                        24213 => "welfare_hp_log_query",
                        24214 => "welfare_hp_log_detail",
                        24220 => "welfare_hp_rank",
                        24221 => "welfare_hp_rank_my",
                        24223 => "welfare_hp_rank_list",
                        24300 => "welfare_guild_state",
                        24301 => "welfare_guild_action",
                        24302 => "welfare_guild_view",
                        24303 => "welfare_guild_view_other",
                        24304 => "welfare_guild_list",
                        24305 => "welfare_guild_query",
                        24306 => "welfare_guild_notify",
                        24308 => "welfare_guild_join",
                        24309 => "welfare_guild_leave",
                        24310 => "welfare_guild_donate",
                        24311 => "welfare_guild_reward",
                        24312 => "welfare_guild_skill",
                        24313 => "welfare_guild_skill_up",
                        24314 => "welfare_guild_log",
                        24315 => "welfare_guild_log_query",
                        24316 => "welfare_guild_target",
                        24400 => "welfare_team_state",
                        24401 => "welfare_team_set",
                        24402 => "welfare_team_get",
                        24403 => "welfare_team_formation",
                        24404 => "welfare_team_list",
                        24405 => "welfare_team_query",
                        24406 => "welfare_team_notify",
                        24407 => "welfare_team_action",
                        24408 => "welfare_team_invite",
                        24409 => "welfare_team_accept",
                        24410 => "welfare_team_reject",
                        24411 => "welfare_team_leave",
                        24500 => "welfare_quest_list",
                        24501 => "welfare_quest_claim",
                        24502 => "welfare_quest_done",
                        24600 => "welfare_arena_state",
                        24601 => "welfare_arena_action",
                        24602 => "welfare_arena_query",
                        24603 => "welfare_arena_result",
                        24604 => "welfare_arena_reward",
                        24700 => "welfare_daily",
                        24701 => "welfare_daily_claim",
                        24702 => "welfare_daily_done",
                        24801 => "welfare_active_list",
                        24802 => "welfare_active_get",
                        24803 => "welfare_active_set",
                        24804 => "welfare_active_query",
                        24805 => "welfare_active_notify",
                        24806 => "welfare_active_action",
                        24807 => "welfare_active_reward",
                        24808 => "welfare_active_log",
                        24809 => "welfare_active_log_query",
                        24810 => "welfare_active_rank",
                        24811 => "welfare_active_state",
                        24812 => "welfare_active_claim",
                        24813 => "welfare_active_done",
                        24814 => "welfare_active_exchange",
                        24815 => "welfare_active_refresh",
                        24816 => "welfare_active_op",
                        24817 => "welfare_active_end",
                        24818 => "welfare_active_view",
                        _ => "w2-extra",
                    },
                    source: "zsyz",
                });
            }
        }
        Registry { map }
    }

    pub fn dispatch(
        &self,
        cmd: u16,
        payload: Vec<u8>,
        rgs: Arc<RgsClient>,
    ) -> Pin<Box<dyn Future<Output = handlers::Response> + Send>> {
        match self.map.get(&cmd) {
            Some(entry) => (entry.handler)(cmd, payload, rgs),
            None => {
                tracing::warn!(cmd, "stub (no handler)");
                Box::pin(async move { handlers::Response { cmd, payload: vec![] } })
            }
        }
    }

    pub fn list(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.map.keys().copied().collect();
        v.sort();
        v
    }

    #[allow(dead_code)]
    pub fn list_real_zsyz(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.map.iter()
            .filter(|(_, e)| e.source == "zsyz")
            .map(|(k, _)| *k)
            .collect();
        v.sort();
        v
    }
}
// Cmd registry (per 9/9 13:50 JST v0.2 模式, Rust 重写 v0.3.0)
// 6 个内置 cmd; 后续 worker 扩自己域时, 加新表项 + handler 函数
//
// 设计: handler 取 owned Vec<u8> + Arc<RgsClient> (clone) + cmd,
// 返回 Pin<Box<dyn Future + Send>> 不绑 lifetime, 避免 HRTB 复杂度
// & self 的 lifetime 也不进 future (entry.handler 是 fn pointer, 不是闭包)
//
// v0.3.1 (per 2026-09-09 14:55 JST Ulysses 拍板):
// - 10101 / 10102 / 10103 / 10200 来自 zsyz_server proto_101.erl + proto_102.erl (真 zsyz cmd)
// - 10400 / 11001 是 shim-internal RGS 测试 cmd (Erlang 10400=quest_list, 11001=partner_list, 不复用)
// - 业务覆盖率 1.2% (4/514 real cmd), 1-2 周 4 worker 扩 (per 9/9 13:45 JST 拍板 A)
//
// v0.3.2 (per 2026-09-09 15:10 JST Ulysses 拍板 "重测直到战斗场景"):
// - 战斗场景 cmd 6 个: 10215 move / 10300 ping / 10301 role_info / 10302 assets / 10309 signature / 10315 view_role
// - 来源: zsyz_server proto_102.erl (10215) + proto_103.erl (10300/10301/10302/10309/10315)
// - 业务覆盖率 1.9% (10/514 real cmd)
