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
        // v0.4.0 (per 2026-09-09 16:25 JST Mavis 派工): 自动注册 766 全 zsyz send cmd stub
        // 来源: H5 zsyz_client proto_mate.js 提取, 域分布 welfare=110/partner=108/battle=103/social=93/...
        // stub 返回空 payload (后续 worker 派工逐个替换为 real handler)
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
