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
