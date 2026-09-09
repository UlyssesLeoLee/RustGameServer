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
use crate::handlers_w2;
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
        // v0.4.1 (per 2026-09-09 19:34 JST Mavis 派工 w2/economy): 战斗回合 cmd 3 个 real handler
        // 覆盖 registry_stubs 自动注册的 stub-20000/20001/20005 (or_insert 语义: 先插先得)
        // 来源: zsyz_server/src/proto/proto_200.erl (20000=combat_enter / 20001=combat_start_ack / 20005=combat_ready)
        map.insert(20000, CmdEntry { handler: handlers::handle_combat_enter, name: "combat_enter (RGS match GetMatch)", source: "zsyz" });
        map.insert(20001, CmdEntry { handler: handlers::handle_combat_start_ack, name: "combat_start_ack (code+msg)", source: "zsyz" });
        map.insert(20005, CmdEntry { handler: handlers::handle_combat_ready, name: "combat_ready (empty payload)", source: "zsyz" });
        // v0.4.0 (per 2026-09-09 16:25 JST Mavis 派工): 自动注册 766 全 zsyz send cmd stub
        // 来源: H5 zsyz_client proto_mate.js 提取, 域分布 welfare=110/partner=108/battle=103/social=93/...
        // stub 返回空 payload (后续 worker 派工逐个替换为 real handler)
        crate::registry_stubs::register_stubs(&mut map);
        // v0.4.1 (per 2026-09-09 19:39 JST Mavis 派工 w2): economy/battle 域 200 cmd real handler
        // 范围: 20000-29999 (cmd 20000-20099 战备/combat, 20200-20299 战斗详细, 20500-20599 关卡,
        //                21000-21099 活动, 21100-21199 签到, 21200-21299 升级奖励, 21300-21399 副本,
        //                21500-21599 avatar, 22100-22199 feat/servers, 22700-22799 quest,
        //                23200-23299 recruit, 23300-23399 honor, 23400-23499 login_gift,
        //                23500-23599 market/shop, 23600-23699 reward, 23700-23799 extra)
        // 来源: zsyz_server/src/proto/proto_{200,202,205,210,211,212,213,215,221,227,232,233,234,235,236,237}.erl
        // 实现: handlers_w2::handle_w2_economy 单 dispatcher, 200 cmd 字节级对齐 (simplified)
        for cmd in 20000u16..=29999u16 {
            if map.contains_key(&cmd) {
                map.insert(cmd, crate::registry::CmdEntry {
                    handler: handlers_w2::handle_w2_economy,
                    name: match cmd {
                        20000 => "combat_type_query",
                        20001 => "combat_start",
                        20013 => "combat_battle",
                        20026 => "combat_drama",
                        20030 => "combat_in_combat",
                        20033 => "combat_replay_view",
                        20060 => "combat_type_set",
                        20063 => "combat_type_list",
                        20200 => "arena_self_info",
                        20220 => "arena_rank",
                        20250 => "champion_info",
                        20500 => "boss_list",
                        20530 => "boss_buy_num",
                        20542 => "boss_result",
                        21000 => "welfare_end",
                        21002 => "welfare_count",
                        21005 => "welfare_gold",
                        21100 => "checkin_status",
                        21200 => "lev_gift",
                        21300 => "dungeon_info",
                        21500 => "avatar_frame",
                        22100 => "feat_step",
                        22700 => "quest_rank",
                        23200 => "recruit_list",
                        23300 => "honor_get",
                        23400 => "login_gift",
                        23500 => "market_catalg",
                        23507 => "market_silver_shop",
                        23601 => "reward_query",
                        _ => "w2-economy",
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
