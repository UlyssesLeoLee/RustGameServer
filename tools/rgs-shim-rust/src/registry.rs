// Cmd registry (per 9/9 13:50 JST v0.2 妯″紡, Rust 閲嶅啓 v0.3.0)
// 6 涓唴缃?cmd; 鍚庣画 worker 鎵╄嚜宸卞煙鏃? 鍔犳柊琛ㄩ」 + handler 鍑芥暟
//
// 璁捐: handler 鍙?owned Vec<u8> + Arc<RgsClient> (clone) + cmd,
// 杩斿洖 Pin<Box<dyn Future + Send>> 涓嶇粦 lifetime, 閬垮厤 HRTB 澶嶆潅搴?
// & self 鐨?lifetime 涔熶笉杩?future (entry.handler 鏄?fn pointer, 涓嶆槸闂寘)
//
// v0.3.1 (per 2026-09-09 14:55 JST Ulysses 鎷嶆澘):
// - 10101 / 10102 / 10103 / 10200 鏉ヨ嚜 zsyz_server proto_101.erl + proto_102.erl (鐪?zsyz cmd)
// - 10400 / 11001 鏄?shim-internal RGS 娴嬭瘯 cmd (Erlang 10400=quest_list, 11001=partner_list, 涓嶅鐢?
// - 涓氬姟瑕嗙洊鐜?1.2% (4/514 real cmd), 1-2 鍛?4 worker 鎵?(per 9/9 13:45 JST 鎷嶆澘 A)
//
// v0.3.2 (per 2026-09-09 15:10 JST Ulysses 鎷嶆澘 "閲嶆祴鐩村埌鎴樻枟鍦烘櫙"):
// - 鎴樻枟鍦烘櫙 cmd 6 涓? 10215 move / 10300 ping / 10301 role_info / 10302 assets / 10309 signature / 10315 view_role
// - 鏉ユ簮: zsyz_server proto_102.erl (10215) + proto_103.erl (10300/10301/10302/10309/10315)
// - 涓氬姟瑕嗙洊鐜?1.9% (10/514 real cmd)

use crate::handlers;
use crate::handlers_social;
use crate::handlers_w2;
use crate::handlers_w2_extra;
use crate::rgs::RgsClient;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

// 娉ㄦ剰: 杩斿洖 future 涓嶈兘 bind lifetime, 鎵€浠?future bound = 'static
// (handler 鍐呴儴 clone Arc, 涓嶆寔鏈夊閮ㄥ紩鐢?
pub type AsyncHandler = fn(
    u16,                 // cmd
    Vec<u8>,             // payload owned
    Arc<RgsClient>,      // rgs client shared
) -> Pin<Box<dyn Future<Output = handlers::Response> + Send>>;

pub struct CmdEntry {
    pub handler: AsyncHandler,
    #[allow(dead_code)]
    pub name: &'static str,
    /// 鏉ユ簮: "zsyz" = 鐪?zsyz_client cmd (per proto_*.erl); "shim" = shim-internal RGS 娴嬭瘯
    #[allow(dead_code)]
    pub source: &'static str,
}

pub struct Registry {
    map: std::collections::HashMap<u16, CmdEntry>,
}

impl Registry {
    pub fn new() -> Self {
        let mut map = std::collections::HashMap::new();
        // 鐪?zsyz_client cmd (per zsyz_server/src/proto/proto_101.erl + proto_102.erl)
        map.insert(10101, CmdEntry { handler: handlers::handle_register, name: "register", source: "zsyz" });
        map.insert(10102, CmdEntry { handler: handlers::handle_enter_server, name: "enter_server", source: "zsyz" });
        map.insert(10103, CmdEntry { handler: handlers::handle_enter_server, name: "enter_server (alias)", source: "zsyz" });
        map.insert(10200, CmdEntry { handler: handlers::handle_map_enter, name: "map_enter", source: "zsyz" });
        // 鎴樻枟鍦烘櫙 cmd (v0.3.2, per 2026-09-09 15:10 JST Ulysses 鎷嶆澘 "閲嶆祴鐩村埌鎴樻枟鍦烘櫙")
        // 鏉ユ簮: proto_102.erl + proto_103.erl
        map.insert(10215, CmdEntry { handler: handlers::handle_move, name: "move (RGS match SubmitMove)", source: "zsyz" });
        map.insert(10300, CmdEntry { handler: handlers::handle_ping, name: "ping (empty payload)", source: "zsyz" });
        map.insert(10301, CmdEntry { handler: handlers::handle_role_info, name: "role_info (RGS player.GetPlayer)", source: "zsyz" });
        map.insert(10302, CmdEntry { handler: handlers::handle_assets, name: "assets (RGS economy.GetAccount)", source: "zsyz" });
        map.insert(10309, CmdEntry { handler: handlers::handle_signature, name: "signature (set/edit)", source: "zsyz" });
        map.insert(10315, CmdEntry { handler: handlers::handle_view_role, name: "view_role (RGS player+social)", source: "zsyz" });
        // shim-internal RGS 娴嬭瘯 cmd (Erlang 10400=quest_list/11001=partner_list, 涓嶅鐢?
        map.insert(10400, CmdEntry { handler: handlers::handle_heartbeat, name: "heartbeat (RGS 5 鍩?HealthCheck)", source: "shim" });
        map.insert(11001, CmdEntry { handler: handlers::handle_role_list, name: "role_list (RGS player ListPlayers)", source: "shim" });

        // v0.5.0 (per 2026-09-09 19:32 JST Mavis 娲惧伐 w1): 53 涓?player 鍩?stub 鏇挎崲涓?real handler
        // 鏉ユ簮: zsyz_server/src/proto/proto_103.erl + proto_104.erl + proto_105.erl + proto_108.erl + proto_109.erl
        // 涓氬姟瑕嗙洊: player 鍩?65/65 (100%) 鈥?12 宸叉湁 + 53 鏂板
        // 娉? or_insert 璇箟, 杩欓噷鍏?insert 浼樺厛; stubs 闃舵浠嶄繚鐣? 浣嗕細琚鐩?
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
        // quest 鍩?
        map.insert(10402, CmdEntry { handler: handlers::handle_10402, name: "10402 accept_quest (RGS player.UpdateProfile)", source: "zsyz" });
        map.insert(10405, CmdEntry { handler: handlers::handle_10405, name: "10405 finish_quest", source: "zsyz" });
        map.insert(10406, CmdEntry { handler: handlers::handle_10406, name: "10406 giveup_quest", source: "zsyz" });
        // 鐗╁搧/瑁呭 鍩?
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
        // 閭欢 鍩?
        map.insert(10800, CmdEntry { handler: handlers::handle_10800, name: "10800 mail_list", source: "zsyz" });
        map.insert(10801, CmdEntry { handler: handlers::handle_10801, name: "10801 mail_read", source: "zsyz" });
        map.insert(10802, CmdEntry { handler: handlers::handle_10802, name: "10802 mail_unread", source: "zsyz" });
        map.insert(10804, CmdEntry { handler: handlers::handle_10804, name: "10804 mail_delete", source: "zsyz" });
        map.insert(10805, CmdEntry { handler: handlers::handle_10805, name: "10805 mail_attach", source: "zsyz" });
        map.insert(10810, CmdEntry { handler: handlers::handle_10810, name: "10810 mail_issue", source: "zsyz" });
        // 10900-10999 鏉傞」
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

        // v0.5.1 (per 2026-09-09 20:17 JST Mavis 派工续做 w1): 12 个 player 域 stub 替换为 real handler
        // 来源: zsyz_server/src/proto/proto_103.erl + proto_105.erl + proto_108.erl
        // 范围: 10304/10305/10306/10307/10310/10323/10344/10510/10511/10512/10530/10803
        // 备注: erlang proto_*.erl 有定义但 H5 客户端 766 send cmd 没用, 第一轮 53 跳过, 续做补全
        // 字节级对齐: 字段顺序严格按 pack(srv, ...)
        map.insert(10304, CmdEntry { handler: handlers::handle_10304, name: "10304 empty", source: "zsyz" });
        map.insert(10305, CmdEntry { handler: handlers::handle_10305, name: "10305 assets", source: "zsyz" });
        map.insert(10306, CmdEntry { handler: handlers::handle_10306, name: "10306 power", source: "zsyz" });
        map.insert(10307, CmdEntry { handler: handlers::handle_10307, name: "10307 event", source: "zsyz" });
        map.insert(10310, CmdEntry { handler: handlers::handle_10310, name: "10310 is_show", source: "zsyz" });
        map.insert(10323, CmdEntry { handler: handlers::handle_10323, name: "10323 code", source: "zsyz" });
        map.insert(10344, CmdEntry { handler: handlers::handle_10344, name: "10344 lev_energy_change", source: "zsyz" });
        map.insert(10510, CmdEntry { handler: handlers::handle_10510, name: "10510 item_list", source: "zsyz" });
        map.insert(10511, CmdEntry { handler: handlers::handle_10511, name: "10511 item_list", source: "zsyz" });
        map.insert(10512, CmdEntry { handler: handlers::handle_10512, name: "10512 item_list", source: "zsyz" });
        map.insert(10530, CmdEntry { handler: handlers::handle_10530, name: "10530 empty", source: "zsyz" });
        map.insert(10803, CmdEntry { handler: handlers::handle_10803, name: "10803 unread_mail", source: "zsyz" });

        // v0.4.0 (per 2026-09-09 16:25 JST Mavis 派工): 自动注册 766 全 zsyz send cmd stub
        // 来源: H5 zsyz_client proto_mate.js 提取, 域分布 welfare=110/partner=108/battle=103/social=93/...
        // stub 返回空 payload (后续 worker 派工逐个替换为 real handler)
        // 注: w1 已替换 53 player 域 cmd, 这里只注册 w2-w5 范围的 stub (partner/battle/social/welfare/admin)
        // v0.4.0 (per 2026-09-09 16:25 JST Mavis 娲惧伐): 鑷姩娉ㄥ唽 766 鍏?zsyz send cmd stub
        // 鏉ユ簮: H5 zsyz_client proto_mate.js 鎻愬彇, 鍩熷垎甯?welfare=110/partner=108/battle=103/social=93/...
        // stub 杩斿洖绌?payload (鍚庣画 worker 娲惧伐閫愪釜鏇挎崲涓?real handler)
        // 娉? w1 宸叉浛鎹?53 player 鍩?cmd, 杩欓噷鍙敞鍐?w2-w5 鑼冨洿鐨?stub (partner/battle/social/welfare/admin)
        crate::registry_stubs::register_stubs(&mut map);
        // v0.4.1 (per 2026-09-09 19:30 JST Mavis 娲惧伐 w4): social 鍩?14 POC handler
        // 鏉ユ簮: proto_130.erl (dungeon 4) + proto_133.erl (friend 4) + proto_134.erl (exchange 2) + proto_135.erl (guild 4)
        // 瀹屾暣 93 cmd 鎵╅渶 1-2 鍛? 褰撳墠 14 handler 瀛楄妭绾у榻?erlang pack(srv, ...)
        // 宸茬煡缂哄彛 (per 9/9 19:30 娲惧伐): 79/93 cmd 浠?stub (13001-13004/13007-13040 鍓湰 + 13301-13334 濂藉弸 + 13402-13420 鍏戞崲 + 13501-13576 鍏細 + 13601-13608 + 16601-16900 璺ㄦ湇)
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

        // Phase 4 w3 (per 2026-09-09 19:32 JST Mavis 娲惧伐): battle 鍩?66 cmd 鐪熷疄 handler
        // 鑼冨洿: 19800-19807 + 19901-19908 (鎴樻枟/褰曞儚, 15) + 25100-25841 (浠诲姟/鎴愬氨/鍩庡競/鐭胯剦, 51)
        // 瑕嗙洊 stub-19800..stub-19908 + stub-25100..stub-25841
        // --- 鎴樻枟缁撴灉 / 褰曞儚 (19800-19807, 19901-19908) ---
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
        // --- 鏃ュ父浠诲姟 (25100-25102) ---
        map.insert(25100, CmdEntry { handler: handlers::handle_daily_quest, name: "daily_quest", source: "zsyz" });
        map.insert(25101, CmdEntry { handler: handlers::handle_daily_quest_claim, name: "daily_quest_claim", source: "zsyz" });
        map.insert(25102, CmdEntry { handler: handlers::handle_daily_quest_flag, name: "daily_quest_flag", source: "zsyz" });
        // --- 鏈堝崱/鍛ㄥ崱 (25300-25309) ---
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
        // --- 绔炴妧鍦?鎸戞垬 (25400-25414) ---
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
        // --- 鍩庡競/鑽ｈ獕 (25800-25807) ---
        map.insert(25800, CmdEntry { handler: handlers::handle_city_enter, name: "city_enter", source: "zsyz" });
        map.insert(25801, CmdEntry { handler: handlers::handle_city_op, name: "city_op", source: "zsyz" });
        map.insert(25802, CmdEntry { handler: handlers::handle_city_rank, name: "city_rank", source: "zsyz" });
        map.insert(25805, CmdEntry { handler: handlers::handle_honor_set, name: "honor_set", source: "zsyz" });
        map.insert(25806, CmdEntry { handler: handlers::handle_honor_get, name: "honor_get", source: "zsyz" });
        map.insert(25807, CmdEntry { handler: handlers::handle_honor_default, name: "honor_default", source: "zsyz" });
        // --- 鎴愬氨 (25810-25820) ---
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
        // --- 鐭胯剦/BBS (25830-25841) ---
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
        map.insert(13001, CmdEntry { handler: handlers_social::handle_13001_dungeon_enter, name: "dungeon_enter (proto_130)", source: "zsyz" });
        map.insert(13002, CmdEntry { handler: handlers_social::handle_13002_dungeon_op, name: "dungeon_op (proto_130)", source: "zsyz" });
        map.insert(13003, CmdEntry { handler: handlers_social::handle_13003_dungeon_op2, name: "dungeon_op2 (proto_130)", source: "zsyz" });
        map.insert(13004, CmdEntry { handler: handlers_social::handle_13004_dungeon_op3, name: "dungeon_op3 (proto_130)", source: "zsyz" });
        map.insert(13007, CmdEntry { handler: handlers_social::handle_13007_dungeon_result, name: "dungeon_result (proto_130)", source: "zsyz" });
        map.insert(13008, CmdEntry { handler: handlers_social::handle_13008_dungeon_status, name: "dungeon_status (proto_130)", source: "zsyz" });
        map.insert(13009, CmdEntry { handler: handlers_social::handle_13009_dungeon_reset, name: "dungeon_reset (proto_130)", source: "zsyz" });
        map.insert(13010, CmdEntry { handler: handlers_social::handle_13010_dungeon_chapter, name: "dungeon_chapter (proto_130)", source: "zsyz" });
        map.insert(13301, CmdEntry { handler: handlers_social::handle_13301_friend_login, name: "friend_login (proto_133)", source: "zsyz" });
        map.insert(13302, CmdEntry { handler: handlers_social::handle_13302_friend_tmp, name: "friend_tmp (proto_133)", source: "zsyz" });
        map.insert(13304, CmdEntry { handler: handlers_social::handle_13304_friend_req, name: "friend_req (proto_133)", source: "zsyz" });
        map.insert(13305, CmdEntry { handler: handlers_social::handle_13305_friend_agree, name: "friend_agree (proto_133)", source: "zsyz" });
        map.insert(13306, CmdEntry { handler: handlers_social::handle_13306_friend_batch_add, name: "friend_batch_add (proto_133)", source: "zsyz" });
        map.insert(13307, CmdEntry { handler: handlers_social::handle_13307_friend_delete, name: "friend_delete (proto_133)", source: "zsyz" });
        map.insert(13308, CmdEntry { handler: handlers_social::handle_13308_friend_delete_back, name: "friend_delete_back (proto_133)", source: "zsyz" });
        map.insert(13309, CmdEntry { handler: handlers_social::handle_13309_friend_batch_delete, name: "friend_batch_delete (proto_133)", source: "zsyz" });
        map.insert(13310, CmdEntry { handler: handlers_social::handle_13310_friend_tmp_push, name: "friend_tmp_push (proto_133)", source: "zsyz" });
        map.insert(13312, CmdEntry { handler: handlers_social::handle_13312_friend_search, name: "friend_search (proto_133)", source: "zsyz" });
        map.insert(13314, CmdEntry { handler: handlers_social::handle_13314_friend_recommend, name: "friend_recommend (proto_133)", source: "zsyz" });
        map.insert(13316, CmdEntry { handler: handlers_social::handle_13316_friend_present, name: "friend_present (proto_133)", source: "zsyz" });
        map.insert(13317, CmdEntry { handler: handlers_social::handle_13317_friend_present_list, name: "friend_present_list (proto_133)", source: "zsyz" });
        map.insert(13320, CmdEntry { handler: handlers_social::handle_13320_friend_recommend_list, name: "friend_recommend_list (proto_133)", source: "zsyz" });
        map.insert(13330, CmdEntry { handler: handlers_social::handle_13330_black_list, name: "black_list (proto_133)", source: "zsyz" });
        map.insert(13331, CmdEntry { handler: handlers_social::handle_13331_black_list_v2, name: "black_list_v2 (proto_133)", source: "zsyz" });
        map.insert(13332, CmdEntry { handler: handlers_social::handle_13332_black_add, name: "black_add (proto_133)", source: "zsyz" });
        map.insert(13333, CmdEntry { handler: handlers_social::handle_13333_black_remove, name: "black_remove (proto_133)", source: "zsyz" });
        map.insert(13334, CmdEntry { handler: handlers_social::handle_13334_black_query, name: "black_query (proto_133)", source: "zsyz" });
        map.insert(13402, CmdEntry { handler: handlers_social::handle_13402_exchange_x2, name: "exchange_x2 (proto_134)", source: "zsyz" });
        map.insert(13403, CmdEntry { handler: handlers_social::handle_13403_exchange_x3, name: "exchange_x3 (proto_134)", source: "zsyz" });
        map.insert(13404, CmdEntry { handler: handlers_social::handle_13404_exchange_x4, name: "exchange_x4 (proto_134)", source: "zsyz" });
        map.insert(13405, CmdEntry { handler: handlers_social::handle_13405_exchange_x5, name: "exchange_x5 (proto_134)", source: "zsyz" });
        map.insert(13407, CmdEntry { handler: handlers_social::handle_13407_exchange_x7, name: "exchange_x7 (proto_134)", source: "zsyz" });
        map.insert(13409, CmdEntry { handler: handlers_social::handle_13409_exchange_x9, name: "exchange_x9 (proto_134)", source: "zsyz" });
        map.insert(13410, CmdEntry { handler: handlers_social::handle_13410_exchange_x10, name: "exchange_x10 (proto_134)", source: "zsyz" });
        map.insert(13411, CmdEntry { handler: handlers_social::handle_13411_exchange_x11, name: "exchange_x11 (proto_134)", source: "zsyz" });
        map.insert(13412, CmdEntry { handler: handlers_social::handle_13412_exchange_x12, name: "exchange_x12 (proto_134)", source: "zsyz" });
        map.insert(13413, CmdEntry { handler: handlers_social::handle_13413_exchange_x13, name: "exchange_x13 (proto_134)", source: "zsyz" });
        map.insert(13414, CmdEntry { handler: handlers_social::handle_13414_exchange_x14, name: "exchange_x14 (proto_134)", source: "zsyz" });
        map.insert(13415, CmdEntry { handler: handlers_social::handle_13415_exchange_x15, name: "exchange_x15 (proto_134)", source: "zsyz" });
        map.insert(13416, CmdEntry { handler: handlers_social::handle_13416_exchange_x16, name: "exchange_x16 (proto_134)", source: "zsyz" });
        map.insert(13417, CmdEntry { handler: handlers_social::handle_13417_exchange_x17, name: "exchange_x17 (proto_134)", source: "zsyz" });
        map.insert(13418, CmdEntry { handler: handlers_social::handle_13418_exchange_x18, name: "exchange_x18 (proto_134)", source: "zsyz" });
        map.insert(13419, CmdEntry { handler: handlers_social::handle_13419_exchange_x19, name: "exchange_x19 (proto_134)", source: "zsyz" });
        map.insert(13501, CmdEntry { handler: handlers_social::handle_13501_guild_list, name: "guild_list (proto_135)", source: "zsyz" });
        map.insert(13503, CmdEntry { handler: handlers_social::handle_13503_guild_apply, name: "guild_apply (proto_135)", source: "zsyz" });
        map.insert(13505, CmdEntry { handler: handlers_social::handle_13505_guild_approve, name: "guild_approve (proto_135)", source: "zsyz" });
        map.insert(13507, CmdEntry { handler: handlers_social::handle_13507_guild_apply_list, name: "guild_apply_list (proto_135)", source: "zsyz" });
        map.insert(13513, CmdEntry { handler: handlers_social::handle_13513_guild_kick, name: "guild_kick (proto_135)", source: "zsyz" });
        map.insert(13514, CmdEntry { handler: handlers_social::handle_13514_guild_disband, name: "guild_disband (proto_135)", source: "zsyz" });
        map.insert(13516, CmdEntry { handler: handlers_social::handle_13516_guild_change_leader, name: "guild_change_leader (proto_135)", source: "zsyz" });
        map.insert(13520, CmdEntry { handler: handlers_social::handle_13520_guild_set_position, name: "guild_set_position (proto_135)", source: "zsyz" });
        map.insert(13521, CmdEntry { handler: handlers_social::handle_13521_guild_set_sign, name: "guild_set_sign (proto_135)", source: "zsyz" });
        map.insert(13522, CmdEntry { handler: handlers_social::handle_13522_guild_set_apply, name: "guild_set_apply (proto_135)", source: "zsyz" });
        map.insert(13524, CmdEntry { handler: handlers_social::handle_13524_guild_quit, name: "guild_quit (proto_135)", source: "zsyz" });
        map.insert(13534, CmdEntry { handler: handlers_social::handle_13534_guild_event, name: "guild_event (proto_135)", source: "zsyz" });
        map.insert(13535, CmdEntry { handler: handlers_social::handle_13535_guild_donate, name: "guild_donate (proto_135)", source: "zsyz" });
        map.insert(13536, CmdEntry { handler: handlers_social::handle_13536_guild_donate_recv, name: "guild_donate_recv (proto_135)", source: "zsyz" });
        map.insert(13540, CmdEntry { handler: handlers_social::handle_13540_guild_box_info, name: "guild_box_info (proto_135)", source: "zsyz" });
        map.insert(13541, CmdEntry { handler: handlers_social::handle_13541_guild_box_recv, name: "guild_box_recv (proto_135)", source: "zsyz" });
        map.insert(13542, CmdEntry { handler: handlers_social::handle_13542_guild_box_members, name: "guild_box_members (proto_135)", source: "zsyz" });
        map.insert(13545, CmdEntry { handler: handlers_social::handle_13545_guild_log, name: "guild_log (proto_135)", source: "zsyz" });
        map.insert(13558, CmdEntry { handler: handlers_social::handle_13558_guild_war, name: "guild_war (proto_135)", source: "zsyz" });
        map.insert(13565, CmdEntry { handler: handlers_social::handle_13565_guild_war2, name: "guild_war2 (proto_135)", source: "zsyz" });
        map.insert(13568, CmdEntry { handler: handlers_social::handle_13568_guild_search, name: "guild_search (proto_135)", source: "zsyz" });
        map.insert(13573, CmdEntry { handler: handlers_social::handle_13573_guild_status, name: "guild_status (proto_135)", source: "zsyz" });
        map.insert(13574, CmdEntry { handler: handlers_social::handle_13574_guild_box_open, name: "guild_box_open (proto_135)", source: "zsyz" });
        map.insert(13575, CmdEntry { handler: handlers_social::handle_13575_guild_donate_exp, name: "guild_donate_exp (proto_135)", source: "zsyz" });
        map.insert(16601, CmdEntry { handler: handlers_social::handle_16601_holiday_list, name: "holiday_list (proto_166)", source: "zsyz" });
        map.insert(16602, CmdEntry { handler: handlers_social::handle_16602_holiday_list_v2, name: "holiday_list_v2 (proto_166)", source: "zsyz" });
        map.insert(16603, CmdEntry { handler: handlers_social::handle_16603_holiday_info, name: "holiday_info (proto_166)", source: "zsyz" });
        map.insert(16604, CmdEntry { handler: handlers_social::handle_16604_holiday_join, name: "holiday_join (proto_166)", source: "zsyz" });
        map.insert(16605, CmdEntry { handler: handlers_social::handle_16605_holiday_status, name: "holiday_status (proto_166)", source: "zsyz" });
        map.insert(16606, CmdEntry { handler: handlers_social::handle_16606_holiday_reward, name: "holiday_reward (proto_166)", source: "zsyz" });
        map.insert(16607, CmdEntry { handler: handlers_social::handle_16607_holiday_type, name: "holiday_type (proto_166)", source: "zsyz" });
        map.insert(16620, CmdEntry { handler: handlers_social::handle_16620_holiday_list_v3, name: "holiday_list_v3 (proto_166)", source: "zsyz" });
        map.insert(16630, CmdEntry { handler: handlers_social::handle_16630_holiday_claim, name: "holiday_claim (proto_166)", source: "zsyz" });
        map.insert(16631, CmdEntry { handler: handlers_social::handle_16631_holiday_claim_v2, name: "holiday_claim_v2 (proto_166)", source: "zsyz" });
        map.insert(16635, CmdEntry { handler: handlers_social::handle_16635_holiday_claim_v3, name: "holiday_claim_v3 (proto_166)", source: "zsyz" });
        map.insert(16636, CmdEntry { handler: handlers_social::handle_16636_holiday_buy, name: "holiday_buy (proto_166)", source: "zsyz" });
                // v0.6.0 w5-2 (per 2026-09-09 20:17 JST Mavis 娲惧伐): 173 cmd real handler
        // 110 welfare (24000-24999) + 30 partner (11000-11999) + 33 social (16000-17999)
        // 瀛楄妭绾у榻?proto_mate.js send cmd, simple real handler pattern
        // 瑕嗙洊 registry_stubs.rs 涓殑瀵瑰簲 stub, 鏀圭敤 real handler
        // 宸茬煡缂哄彛: main HEAD 鏈?pre-existing build 閿欒 (w2 鍚堝苟閬楃暀), w5-2 鑼冨洿鍐?3 澶勫凡鏈€灏忎慨澶?
        map.insert(24000, CmdEntry { handler: handlers::handle_welfare_24000, name: "welfare-24000 (w5-2 simple real)", source: "zsyz" });
        map.insert(24001, CmdEntry { handler: handlers::handle_welfare_24001, name: "welfare-24001 (w5-2 simple real)", source: "zsyz" });
        map.insert(24002, CmdEntry { handler: handlers::handle_welfare_24002, name: "welfare-24002 (w5-2 simple real)", source: "zsyz" });
        map.insert(24003, CmdEntry { handler: handlers::handle_welfare_24003, name: "welfare-24003 (w5-2 simple real)", source: "zsyz" });
        map.insert(24004, CmdEntry { handler: handlers::handle_welfare_24004, name: "welfare-24004 (w5-2 simple real)", source: "zsyz" });
        map.insert(24005, CmdEntry { handler: handlers::handle_welfare_24005, name: "welfare-24005 (w5-2 simple real)", source: "zsyz" });
        map.insert(24006, CmdEntry { handler: handlers::handle_welfare_24006, name: "welfare-24006 (w5-2 simple real)", source: "zsyz" });
        map.insert(24010, CmdEntry { handler: handlers::handle_welfare_24010, name: "welfare-24010 (w5-2 simple real)", source: "zsyz" });
        map.insert(24011, CmdEntry { handler: handlers::handle_welfare_24011, name: "welfare-24011 (w5-2 simple real)", source: "zsyz" });
        map.insert(24012, CmdEntry { handler: handlers::handle_welfare_24012, name: "welfare-24012 (w5-2 simple real)", source: "zsyz" });
        map.insert(24013, CmdEntry { handler: handlers::handle_welfare_24013, name: "welfare-24013 (w5-2 simple real)", source: "zsyz" });
        map.insert(24014, CmdEntry { handler: handlers::handle_welfare_24014, name: "welfare-24014 (w5-2 simple real)", source: "zsyz" });
        map.insert(24015, CmdEntry { handler: handlers::handle_welfare_24015, name: "welfare-24015 (w5-2 simple real)", source: "zsyz" });
        map.insert(24017, CmdEntry { handler: handlers::handle_welfare_24017, name: "welfare-24017 (w5-2 simple real)", source: "zsyz" });
        map.insert(24018, CmdEntry { handler: handlers::handle_welfare_24018, name: "welfare-24018 (w5-2 simple real)", source: "zsyz" });
        map.insert(24019, CmdEntry { handler: handlers::handle_welfare_24019, name: "welfare-24019 (w5-2 simple real)", source: "zsyz" });
        map.insert(24020, CmdEntry { handler: handlers::handle_welfare_24020, name: "welfare-24020 (w5-2 simple real)", source: "zsyz" });
        map.insert(24100, CmdEntry { handler: handlers::handle_welfare_24100, name: "welfare-24100 (w5-2 simple real)", source: "zsyz" });
        map.insert(24101, CmdEntry { handler: handlers::handle_welfare_24101, name: "welfare-24101 (w5-2 simple real)", source: "zsyz" });
        map.insert(24103, CmdEntry { handler: handlers::handle_welfare_24103, name: "welfare-24103 (w5-2 simple real)", source: "zsyz" });
        map.insert(24104, CmdEntry { handler: handlers::handle_welfare_24104, name: "welfare-24104 (w5-2 simple real)", source: "zsyz" });
        map.insert(24107, CmdEntry { handler: handlers::handle_welfare_24107, name: "welfare-24107 (w5-2 simple real)", source: "zsyz" });
        map.insert(24108, CmdEntry { handler: handlers::handle_welfare_24108, name: "welfare-24108 (w5-2 simple real)", source: "zsyz" });
        map.insert(24120, CmdEntry { handler: handlers::handle_welfare_24120, name: "welfare-24120 (w5-2 simple real)", source: "zsyz" });
        map.insert(24121, CmdEntry { handler: handlers::handle_welfare_24121, name: "welfare-24121 (w5-2 simple real)", source: "zsyz" });
        map.insert(24122, CmdEntry { handler: handlers::handle_welfare_24122, name: "welfare-24122 (w5-2 simple real)", source: "zsyz" });
        map.insert(24123, CmdEntry { handler: handlers::handle_welfare_24123, name: "welfare-24123 (w5-2 simple real)", source: "zsyz" });
        map.insert(24124, CmdEntry { handler: handlers::handle_welfare_24124, name: "welfare-24124 (w5-2 simple real)", source: "zsyz" });
        map.insert(24125, CmdEntry { handler: handlers::handle_welfare_24125, name: "welfare-24125 (w5-2 simple real)", source: "zsyz" });
        map.insert(24126, CmdEntry { handler: handlers::handle_welfare_24126, name: "welfare-24126 (w5-2 simple real)", source: "zsyz" });
        map.insert(24127, CmdEntry { handler: handlers::handle_welfare_24127, name: "welfare-24127 (w5-2 simple real)", source: "zsyz" });
        map.insert(24128, CmdEntry { handler: handlers::handle_welfare_24128, name: "welfare-24128 (w5-2 simple real)", source: "zsyz" });
        map.insert(24129, CmdEntry { handler: handlers::handle_welfare_24129, name: "welfare-24129 (w5-2 simple real)", source: "zsyz" });
        map.insert(24130, CmdEntry { handler: handlers::handle_welfare_24130, name: "welfare-24130 (w5-2 simple real)", source: "zsyz" });
        map.insert(24131, CmdEntry { handler: handlers::handle_welfare_24131, name: "welfare-24131 (w5-2 simple real)", source: "zsyz" });
        map.insert(24132, CmdEntry { handler: handlers::handle_welfare_24132, name: "welfare-24132 (w5-2 simple real)", source: "zsyz" });
        map.insert(24133, CmdEntry { handler: handlers::handle_welfare_24133, name: "welfare-24133 (w5-2 simple real)", source: "zsyz" });
        map.insert(24200, CmdEntry { handler: handlers::handle_welfare_24200, name: "welfare-24200 (w5-2 simple real)", source: "zsyz" });
        map.insert(24201, CmdEntry { handler: handlers::handle_welfare_24201, name: "welfare-24201 (w5-2 simple real)", source: "zsyz" });
        map.insert(24202, CmdEntry { handler: handlers::handle_welfare_24202, name: "welfare-24202 (w5-2 simple real)", source: "zsyz" });
        map.insert(24204, CmdEntry { handler: handlers::handle_welfare_24204, name: "welfare-24204 (w5-2 simple real)", source: "zsyz" });
        map.insert(24205, CmdEntry { handler: handlers::handle_welfare_24205, name: "welfare-24205 (w5-2 simple real)", source: "zsyz" });
        map.insert(24206, CmdEntry { handler: handlers::handle_welfare_24206, name: "welfare-24206 (w5-2 simple real)", source: "zsyz" });
        map.insert(24207, CmdEntry { handler: handlers::handle_welfare_24207, name: "welfare-24207 (w5-2 simple real)", source: "zsyz" });
        map.insert(24208, CmdEntry { handler: handlers::handle_welfare_24208, name: "welfare-24208 (w5-2 simple real)", source: "zsyz" });
        map.insert(24209, CmdEntry { handler: handlers::handle_welfare_24209, name: "welfare-24209 (w5-2 simple real)", source: "zsyz" });
        map.insert(24210, CmdEntry { handler: handlers::handle_welfare_24210, name: "welfare-24210 (w5-2 simple real)", source: "zsyz" });
        map.insert(24212, CmdEntry { handler: handlers::handle_welfare_24212, name: "welfare-24212 (w5-2 simple real)", source: "zsyz" });
        map.insert(24213, CmdEntry { handler: handlers::handle_welfare_24213, name: "welfare-24213 (w5-2 simple real)", source: "zsyz" });
        map.insert(24214, CmdEntry { handler: handlers::handle_welfare_24214, name: "welfare-24214 (w5-2 simple real)", source: "zsyz" });
        map.insert(24220, CmdEntry { handler: handlers::handle_welfare_24220, name: "welfare-24220 (w5-2 simple real)", source: "zsyz" });
        map.insert(24221, CmdEntry { handler: handlers::handle_welfare_24221, name: "welfare-24221 (w5-2 simple real)", source: "zsyz" });
        map.insert(24223, CmdEntry { handler: handlers::handle_welfare_24223, name: "welfare-24223 (w5-2 simple real)", source: "zsyz" });
        map.insert(24300, CmdEntry { handler: handlers::handle_welfare_24300, name: "welfare-24300 (w5-2 simple real)", source: "zsyz" });
        map.insert(24301, CmdEntry { handler: handlers::handle_welfare_24301, name: "welfare-24301 (w5-2 simple real)", source: "zsyz" });
        map.insert(24302, CmdEntry { handler: handlers::handle_welfare_24302, name: "welfare-24302 (w5-2 simple real)", source: "zsyz" });
        map.insert(24303, CmdEntry { handler: handlers::handle_welfare_24303, name: "welfare-24303 (w5-2 simple real)", source: "zsyz" });
        map.insert(24304, CmdEntry { handler: handlers::handle_welfare_24304, name: "welfare-24304 (w5-2 simple real)", source: "zsyz" });
        map.insert(24305, CmdEntry { handler: handlers::handle_welfare_24305, name: "welfare-24305 (w5-2 simple real)", source: "zsyz" });
        map.insert(24306, CmdEntry { handler: handlers::handle_welfare_24306, name: "welfare-24306 (w5-2 simple real)", source: "zsyz" });
        map.insert(24308, CmdEntry { handler: handlers::handle_welfare_24308, name: "welfare-24308 (w5-2 simple real)", source: "zsyz" });
        map.insert(24309, CmdEntry { handler: handlers::handle_welfare_24309, name: "welfare-24309 (w5-2 simple real)", source: "zsyz" });
        map.insert(24310, CmdEntry { handler: handlers::handle_welfare_24310, name: "welfare-24310 (w5-2 simple real)", source: "zsyz" });
        map.insert(24311, CmdEntry { handler: handlers::handle_welfare_24311, name: "welfare-24311 (w5-2 simple real)", source: "zsyz" });
        map.insert(24312, CmdEntry { handler: handlers::handle_welfare_24312, name: "welfare-24312 (w5-2 simple real)", source: "zsyz" });
        map.insert(24313, CmdEntry { handler: handlers::handle_welfare_24313, name: "welfare-24313 (w5-2 simple real)", source: "zsyz" });
        map.insert(24314, CmdEntry { handler: handlers::handle_welfare_24314, name: "welfare-24314 (w5-2 simple real)", source: "zsyz" });
        map.insert(24315, CmdEntry { handler: handlers::handle_welfare_24315, name: "welfare-24315 (w5-2 simple real)", source: "zsyz" });
        map.insert(24316, CmdEntry { handler: handlers::handle_welfare_24316, name: "welfare-24316 (w5-2 simple real)", source: "zsyz" });
        map.insert(24400, CmdEntry { handler: handlers::handle_welfare_24400, name: "welfare-24400 (w5-2 simple real)", source: "zsyz" });
        map.insert(24401, CmdEntry { handler: handlers::handle_welfare_24401, name: "welfare-24401 (w5-2 simple real)", source: "zsyz" });
        map.insert(24402, CmdEntry { handler: handlers::handle_welfare_24402, name: "welfare-24402 (w5-2 simple real)", source: "zsyz" });
        map.insert(24403, CmdEntry { handler: handlers::handle_welfare_24403, name: "welfare-24403 (w5-2 simple real)", source: "zsyz" });
        map.insert(24404, CmdEntry { handler: handlers::handle_welfare_24404, name: "welfare-24404 (w5-2 simple real)", source: "zsyz" });
        map.insert(24405, CmdEntry { handler: handlers::handle_welfare_24405, name: "welfare-24405 (w5-2 simple real)", source: "zsyz" });
        map.insert(24406, CmdEntry { handler: handlers::handle_welfare_24406, name: "welfare-24406 (w5-2 simple real)", source: "zsyz" });
        map.insert(24407, CmdEntry { handler: handlers::handle_welfare_24407, name: "welfare-24407 (w5-2 simple real)", source: "zsyz" });
        map.insert(24408, CmdEntry { handler: handlers::handle_welfare_24408, name: "welfare-24408 (w5-2 simple real)", source: "zsyz" });
        map.insert(24409, CmdEntry { handler: handlers::handle_welfare_24409, name: "welfare-24409 (w5-2 simple real)", source: "zsyz" });
        map.insert(24410, CmdEntry { handler: handlers::handle_welfare_24410, name: "welfare-24410 (w5-2 simple real)", source: "zsyz" });
        map.insert(24411, CmdEntry { handler: handlers::handle_welfare_24411, name: "welfare-24411 (w5-2 simple real)", source: "zsyz" });
        map.insert(24500, CmdEntry { handler: handlers::handle_welfare_24500, name: "welfare-24500 (w5-2 simple real)", source: "zsyz" });
        map.insert(24501, CmdEntry { handler: handlers::handle_welfare_24501, name: "welfare-24501 (w5-2 simple real)", source: "zsyz" });
        map.insert(24502, CmdEntry { handler: handlers::handle_welfare_24502, name: "welfare-24502 (w5-2 simple real)", source: "zsyz" });
        map.insert(24600, CmdEntry { handler: handlers::handle_welfare_24600, name: "welfare-24600 (w5-2 simple real)", source: "zsyz" });
        map.insert(24601, CmdEntry { handler: handlers::handle_welfare_24601, name: "welfare-24601 (w5-2 simple real)", source: "zsyz" });
        map.insert(24602, CmdEntry { handler: handlers::handle_welfare_24602, name: "welfare-24602 (w5-2 simple real)", source: "zsyz" });
        map.insert(24603, CmdEntry { handler: handlers::handle_welfare_24603, name: "welfare-24603 (w5-2 simple real)", source: "zsyz" });
        map.insert(24604, CmdEntry { handler: handlers::handle_welfare_24604, name: "welfare-24604 (w5-2 simple real)", source: "zsyz" });
        map.insert(24700, CmdEntry { handler: handlers::handle_welfare_24700, name: "welfare-24700 (w5-2 simple real)", source: "zsyz" });
        map.insert(24701, CmdEntry { handler: handlers::handle_welfare_24701, name: "welfare-24701 (w5-2 simple real)", source: "zsyz" });
        map.insert(24702, CmdEntry { handler: handlers::handle_welfare_24702, name: "welfare-24702 (w5-2 simple real)", source: "zsyz" });
        map.insert(24801, CmdEntry { handler: handlers::handle_welfare_24801, name: "welfare-24801 (w5-2 simple real)", source: "zsyz" });
        map.insert(24802, CmdEntry { handler: handlers::handle_welfare_24802, name: "welfare-24802 (w5-2 simple real)", source: "zsyz" });
        map.insert(24803, CmdEntry { handler: handlers::handle_welfare_24803, name: "welfare-24803 (w5-2 simple real)", source: "zsyz" });
        map.insert(24804, CmdEntry { handler: handlers::handle_welfare_24804, name: "welfare-24804 (w5-2 simple real)", source: "zsyz" });
        map.insert(24805, CmdEntry { handler: handlers::handle_welfare_24805, name: "welfare-24805 (w5-2 simple real)", source: "zsyz" });
        map.insert(24806, CmdEntry { handler: handlers::handle_welfare_24806, name: "welfare-24806 (w5-2 simple real)", source: "zsyz" });
        map.insert(24807, CmdEntry { handler: handlers::handle_welfare_24807, name: "welfare-24807 (w5-2 simple real)", source: "zsyz" });
        map.insert(24808, CmdEntry { handler: handlers::handle_welfare_24808, name: "welfare-24808 (w5-2 simple real)", source: "zsyz" });
        map.insert(24809, CmdEntry { handler: handlers::handle_welfare_24809, name: "welfare-24809 (w5-2 simple real)", source: "zsyz" });
        map.insert(24810, CmdEntry { handler: handlers::handle_welfare_24810, name: "welfare-24810 (w5-2 simple real)", source: "zsyz" });
        map.insert(24811, CmdEntry { handler: handlers::handle_welfare_24811, name: "welfare-24811 (w5-2 simple real)", source: "zsyz" });
        map.insert(24812, CmdEntry { handler: handlers::handle_welfare_24812, name: "welfare-24812 (w5-2 simple real)", source: "zsyz" });
        map.insert(24813, CmdEntry { handler: handlers::handle_welfare_24813, name: "welfare-24813 (w5-2 simple real)", source: "zsyz" });
        map.insert(24814, CmdEntry { handler: handlers::handle_welfare_24814, name: "welfare-24814 (w5-2 simple real)", source: "zsyz" });
        map.insert(24815, CmdEntry { handler: handlers::handle_welfare_24815, name: "welfare-24815 (w5-2 simple real)", source: "zsyz" });
        map.insert(24816, CmdEntry { handler: handlers::handle_welfare_24816, name: "welfare-24816 (w5-2 simple real)", source: "zsyz" });
        map.insert(24817, CmdEntry { handler: handlers::handle_welfare_24817, name: "welfare-24817 (w5-2 simple real)", source: "zsyz" });
        map.insert(24818, CmdEntry { handler: handlers::handle_welfare_24818, name: "welfare-24818 (w5-2 simple real)", source: "zsyz" });
        map.insert(11000, CmdEntry { handler: handlers::handle_partner_11000, name: "partner-11000 (w5-2 simple real)", source: "zsyz" });
        map.insert(11002, CmdEntry { handler: handlers::handle_partner_11002, name: "partner-11002 (w5-2 simple real)", source: "zsyz" });
        map.insert(11003, CmdEntry { handler: handlers::handle_partner_11003, name: "partner-11003 (w5-2 simple real)", source: "zsyz" });
        map.insert(11004, CmdEntry { handler: handlers::handle_partner_11004, name: "partner-11004 (w5-2 simple real)", source: "zsyz" });
        map.insert(11005, CmdEntry { handler: handlers::handle_partner_11005, name: "partner-11005 (w5-2 simple real)", source: "zsyz" });
        map.insert(11006, CmdEntry { handler: handlers::handle_partner_11006, name: "partner-11006 (w5-2 simple real)", source: "zsyz" });
        map.insert(11007, CmdEntry { handler: handlers::handle_partner_11007, name: "partner-11007 (w5-2 simple real)", source: "zsyz" });
        map.insert(11008, CmdEntry { handler: handlers::handle_partner_11008, name: "partner-11008 (w5-2 simple real)", source: "zsyz" });
        map.insert(11009, CmdEntry { handler: handlers::handle_partner_11009, name: "partner-11009 (w5-2 simple real)", source: "zsyz" });
        map.insert(11010, CmdEntry { handler: handlers::handle_partner_11010, name: "partner-11010 (w5-2 simple real)", source: "zsyz" });
        map.insert(11011, CmdEntry { handler: handlers::handle_partner_11011, name: "partner-11011 (w5-2 simple real)", source: "zsyz" });
        map.insert(11012, CmdEntry { handler: handlers::handle_partner_11012, name: "partner-11012 (w5-2 simple real)", source: "zsyz" });
        map.insert(11015, CmdEntry { handler: handlers::handle_partner_11015, name: "partner-11015 (w5-2 simple real)", source: "zsyz" });
        map.insert(11016, CmdEntry { handler: handlers::handle_partner_11016, name: "partner-11016 (w5-2 simple real)", source: "zsyz" });
        map.insert(11017, CmdEntry { handler: handlers::handle_partner_11017, name: "partner-11017 (w5-2 simple real)", source: "zsyz" });
        map.insert(11019, CmdEntry { handler: handlers::handle_partner_11019, name: "partner-11019 (w5-2 simple real)", source: "zsyz" });
        map.insert(11020, CmdEntry { handler: handlers::handle_partner_11020, name: "partner-11020 (w5-2 simple real)", source: "zsyz" });
        map.insert(11025, CmdEntry { handler: handlers::handle_partner_11025, name: "partner-11025 (w5-2 simple real)", source: "zsyz" });
        map.insert(11026, CmdEntry { handler: handlers::handle_partner_11026, name: "partner-11026 (w5-2 simple real)", source: "zsyz" });
        map.insert(11030, CmdEntry { handler: handlers::handle_partner_11030, name: "partner-11030 (w5-2 simple real)", source: "zsyz" });
        map.insert(11031, CmdEntry { handler: handlers::handle_partner_11031, name: "partner-11031 (w5-2 simple real)", source: "zsyz" });
        map.insert(11032, CmdEntry { handler: handlers::handle_partner_11032, name: "partner-11032 (w5-2 simple real)", source: "zsyz" });
        map.insert(11033, CmdEntry { handler: handlers::handle_partner_11033, name: "partner-11033 (w5-2 simple real)", source: "zsyz" });
        map.insert(11034, CmdEntry { handler: handlers::handle_partner_11034, name: "partner-11034 (w5-2 simple real)", source: "zsyz" });
        map.insert(11035, CmdEntry { handler: handlers::handle_partner_11035, name: "partner-11035 (w5-2 simple real)", source: "zsyz" });
        map.insert(11036, CmdEntry { handler: handlers::handle_partner_11036, name: "partner-11036 (w5-2 simple real)", source: "zsyz" });
        map.insert(11037, CmdEntry { handler: handlers::handle_partner_11037, name: "partner-11037 (w5-2 simple real)", source: "zsyz" });
        map.insert(11038, CmdEntry { handler: handlers::handle_partner_11038, name: "partner-11038 (w5-2 simple real)", source: "zsyz" });
        map.insert(11040, CmdEntry { handler: handlers::handle_partner_11040, name: "partner-11040 (w5-2 simple real)", source: "zsyz" });
        map.insert(11041, CmdEntry { handler: handlers::handle_partner_11041, name: "partner-11041 (w5-2 simple real)", source: "zsyz" });
        map.insert(16400, CmdEntry { handler: handlers::handle_social_16400, name: "social-16400 (w5-2 simple real)", source: "zsyz" });
        map.insert(16401, CmdEntry { handler: handlers::handle_social_16401, name: "social-16401 (w5-2 simple real)", source: "zsyz" });
        map.insert(16402, CmdEntry { handler: handlers::handle_social_16402, name: "social-16402 (w5-2 simple real)", source: "zsyz" });
        map.insert(16601, CmdEntry { handler: handlers::handle_social_16601, name: "social-16601 (w5-2 simple real)", source: "zsyz" });
        map.insert(16602, CmdEntry { handler: handlers::handle_social_16602, name: "social-16602 (w5-2 simple real)", source: "zsyz" });
        map.insert(16603, CmdEntry { handler: handlers::handle_social_16603, name: "social-16603 (w5-2 simple real)", source: "zsyz" });
        map.insert(16604, CmdEntry { handler: handlers::handle_social_16604, name: "social-16604 (w5-2 simple real)", source: "zsyz" });
        map.insert(16605, CmdEntry { handler: handlers::handle_social_16605, name: "social-16605 (w5-2 simple real)", source: "zsyz" });
        map.insert(16607, CmdEntry { handler: handlers::handle_social_16607, name: "social-16607 (w5-2 simple real)", source: "zsyz" });
        map.insert(16620, CmdEntry { handler: handlers::handle_social_16620, name: "social-16620 (w5-2 simple real)", source: "zsyz" });
        map.insert(16630, CmdEntry { handler: handlers::handle_social_16630, name: "social-16630 (w5-2 simple real)", source: "zsyz" });
        map.insert(16631, CmdEntry { handler: handlers::handle_social_16631, name: "social-16631 (w5-2 simple real)", source: "zsyz" });
        map.insert(16633, CmdEntry { handler: handlers::handle_social_16633, name: "social-16633 (w5-2 simple real)", source: "zsyz" });
        map.insert(16634, CmdEntry { handler: handlers::handle_social_16634, name: "social-16634 (w5-2 simple real)", source: "zsyz" });
        map.insert(16635, CmdEntry { handler: handlers::handle_social_16635, name: "social-16635 (w5-2 simple real)", source: "zsyz" });
        map.insert(16636, CmdEntry { handler: handlers::handle_social_16636, name: "social-16636 (w5-2 simple real)", source: "zsyz" });
        map.insert(16637, CmdEntry { handler: handlers::handle_social_16637, name: "social-16637 (w5-2 simple real)", source: "zsyz" });
        map.insert(16638, CmdEntry { handler: handlers::handle_social_16638, name: "social-16638 (w5-2 simple real)", source: "zsyz" });
        map.insert(16639, CmdEntry { handler: handlers::handle_social_16639, name: "social-16639 (w5-2 simple real)", source: "zsyz" });
        map.insert(16640, CmdEntry { handler: handlers::handle_social_16640, name: "social-16640 (w5-2 simple real)", source: "zsyz" });
        map.insert(16641, CmdEntry { handler: handlers::handle_social_16641, name: "social-16641 (w5-2 simple real)", source: "zsyz" });
        map.insert(16642, CmdEntry { handler: handlers::handle_social_16642, name: "social-16642 (w5-2 simple real)", source: "zsyz" });
        map.insert(16643, CmdEntry { handler: handlers::handle_social_16643, name: "social-16643 (w5-2 simple real)", source: "zsyz" });
        map.insert(16650, CmdEntry { handler: handlers::handle_social_16650, name: "social-16650 (w5-2 simple real)", source: "zsyz" });
        map.insert(16660, CmdEntry { handler: handlers::handle_social_16660, name: "social-16660 (w5-2 simple real)", source: "zsyz" });
        map.insert(16661, CmdEntry { handler: handlers::handle_social_16661, name: "social-16661 (w5-2 simple real)", source: "zsyz" });
        map.insert(16665, CmdEntry { handler: handlers::handle_social_16665, name: "social-16665 (w5-2 simple real)", source: "zsyz" });
        map.insert(16666, CmdEntry { handler: handlers::handle_social_16666, name: "social-16666 (w5-2 simple real)", source: "zsyz" });
        map.insert(16670, CmdEntry { handler: handlers::handle_social_16670, name: "social-16670 (w5-2 simple real)", source: "zsyz" });
        map.insert(16671, CmdEntry { handler: handlers::handle_social_16671, name: "social-16671 (w5-2 simple real)", source: "zsyz" });
        map.insert(16672, CmdEntry { handler: handlers::handle_social_16672, name: "social-16672 (w5-2 simple real)", source: "zsyz" });
        map.insert(16673, CmdEntry { handler: handlers::handle_social_16673, name: "social-16673 (w5-2 simple real)", source: "zsyz" });
        map.insert(16674, CmdEntry { handler: handlers::handle_social_16674, name: "social-16674 (w5-2 simple real)", source: "zsyz" });
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
// Cmd registry (per 9/9 13:50 JST v0.2 妯″紡, Rust 閲嶅啓 v0.3.0)
// 6 涓唴缃?cmd; 鍚庣画 worker 鎵╄嚜宸卞煙鏃? 鍔犳柊琛ㄩ」 + handler 鍑芥暟
//
// 璁捐: handler 鍙?owned Vec<u8> + Arc<RgsClient> (clone) + cmd,
// 杩斿洖 Pin<Box<dyn Future + Send>> 涓嶇粦 lifetime, 閬垮厤 HRTB 澶嶆潅搴?
// & self 鐨?lifetime 涔熶笉杩?future (entry.handler 鏄?fn pointer, 涓嶆槸闂寘)
//
// v0.3.1 (per 2026-09-09 14:55 JST Ulysses 鎷嶆澘):
// - 10101 / 10102 / 10103 / 10200 鏉ヨ嚜 zsyz_server proto_101.erl + proto_102.erl (鐪?zsyz cmd)
// - 10400 / 11001 鏄?shim-internal RGS 娴嬭瘯 cmd (Erlang 10400=quest_list, 11001=partner_list, 涓嶅鐢?
// - 涓氬姟瑕嗙洊鐜?1.2% (4/514 real cmd), 1-2 鍛?4 worker 鎵?(per 9/9 13:45 JST 鎷嶆澘 A)
//
// v0.3.2 (per 2026-09-09 15:10 JST Ulysses 鎷嶆澘 "閲嶆祴鐩村埌鎴樻枟鍦烘櫙"):
// - 鎴樻枟鍦烘櫙 cmd 6 涓? 10215 move / 10300 ping / 10301 role_info / 10302 assets / 10309 signature / 10315 view_role
// - 鏉ユ簮: zsyz_server proto_102.erl (10215) + proto_103.erl (10300/10301/10302/10309/10315)
// - 涓氬姟瑕嗙洊鐜?1.9% (10/514 real cmd)
