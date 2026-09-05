//! 协议号 → gRPC method 路由表 (per 9/4 改进路线图.md Phase 1 协议网关)
//!
//! ## 设计
//! 1351 条协议号 → gRPC service.method 路由, per W6 task "协议号 → gRPC method 路由表
//! (1351 条)". 本骨架仅 1 条 demo: 10101 → player.v1.PlayerService.CreateCharacter.
//!
//! ## 数据源
//! - 9/4 API 清单-全量提取-2026-09-04.tsv (1351 条)
//! - 9/4 API 清单-按文件分组-2026-09-04.tsv (96 文件 + 协议码区间)
//! - Phase 1.5 推进: 全 1351 条静态生成 (build.rs codegen)
//!
//! ## 已知缺口
//! - Phase 1 仅 1 条 demo
//! - 动态注册 (RegisterRoute RPC) 走 Arc<RwLock<RouteTable>>
//! - 协议码冲突检测 (多源 TSV 同一码) 留 Phase 1.5

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::proto::v1 as gateway_proto_v1;

pub type RouteEntry = gateway_proto_v1::RouteEntry;

/// 默认静态路由表 (Phase 1 骨架, 仅 1 条 demo: 10101 = 创建角色)
///
/// 数据来源: 9/4 API 清单-全量提取-2026-09-04.tsv 第 1 条
///   proto_101.erl 10101 创建角色
///
/// Phase 1.5 推进: 全 1351 条由 build.rs 从 TSV codegen, 启动加载.
pub const DEFAULT_ROUTES: &[(u32, &str, &str, &str, &str)] = &[
    (
        10101,
        "create_character",
        "player.v1.PlayerService",
        "CreateCharacter",
        "http://127.0.0.1:50051",
    ),
];

/// W7 扩展: 8 域 demo 路由表 (per 9/4 改进路线图 Phase 2 7 域 + 1 cluster_ops)
///
/// 协议码区间 (per 9/4 API 清单):
/// - 101xx-103xx player (账号/角色)
/// - 201xx-205xx economy (经济/商城)
/// - 102xx-103xx scene (场景/移动) — NEW per 9/4
/// - 200xx-205xx battle (战斗/PVE) — NEW per 9/4
/// - 301xx batch (批量任务) — per 9/1 REQ
/// - 401xx admin (后台管理)
/// - 501xx cluster_ops (健康检查)
///
/// Phase 1.5 推进: 1351 条全 codegen, 本表仅 8 条 demo 覆盖 7 域
pub const PHASE1_5_DEMO_ROUTES: &[(u32, &str, &str, &str, &str)] = &[
    // player (1)
    (
        10101,
        "create_character",
        "player.v1.PlayerService",
        "CreateCharacter",
        "http://127.0.0.1:50051",
    ),
    // economy (1)
    (
        20101,
        "add_currency",
        "economy.v1.EconomyService",
        "AddCurrency",
        "http://127.0.0.1:50052",
    ),
    // scene (2, NEW per 9/4)
    (
        10201,
        "enter_scene",
        "scene.v1.SceneService",
        "EnterScene",
        "http://127.0.0.1:50053",
    ),
    (
        10202,
        "leave_scene",
        "scene.v1.SceneService",
        "LeaveScene",
        "http://127.0.0.1:50053",
    ),
    // battle (2, NEW per 9/4)
    (
        20001,
        "start_pve",
        "battle.v1.BattleService",
        "StartPve",
        "http://127.0.0.1:50054",
    ),
    (
        20002,
        "end_pve",
        "battle.v1.BattleService",
        "EndPve",
        "http://127.0.0.1:50054",
    ),
    // batch (1, per 9/1 REQ)
    (
        30101,
        "submit_task",
        "batch.v1.BatchService",
        "SubmitTask",
        "http://127.0.0.1:50055",
    ),
    // admin (1)
    (
        40101,
        "issue_gm_command",
        "admin.v1.AdminService",
        "IssueGmCommand",
        "http://127.0.0.1:50056",
    ),
    // cluster_ops (1)
    (
        50101,
        "health_check",
        "cluster_ops.v1.ClusterOpsService",
        "HealthCheck",
        "http://127.0.0.1:50057",
    ),
];

/// 协议网关路由表 (动态 + 静态混合)
///
/// Phase 1 骨架: 内部用 std::sync::RwLock (避免新增 dep), Phase 1.5 评估 parking_lot 复用.
pub struct RouteTable {
    inner: Arc<RwLock<HashMap<u32, RouteEntry>>>,
}

impl Default for RouteTable {
    fn default() -> Self {
        let mut map = HashMap::new();
        for (code, name, svc, method, addr) in DEFAULT_ROUTES {
            let entry = RouteEntry {
                code: *code,
                name: (*name).to_string(),
                target_service: (*svc).to_string(),
                target_method: (*method).to_string(),
                target_addr: (*addr).to_string(),
            };
            map.insert(*code, entry);
        }
        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }
}

impl RouteTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// W7 扩展: 用 8 域 demo 路由表构造 (per 9/4 改进路线图 Phase 2)
    pub fn with_phase15_demo() -> Self {
        let mut map = HashMap::new();
        for (code, name, svc, method, addr) in PHASE1_5_DEMO_ROUTES {
            let entry = RouteEntry {
                code: *code,
                name: (*name).to_string(),
                target_service: (*svc).to_string(),
                target_method: (*method).to_string(),
                target_addr: (*addr).to_string(),
            };
            map.insert(*code, entry);
        }
        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }

    /// 查表 (返回克隆, 无锁后访问)
    pub fn get(&self, code: u32) -> Option<RouteEntry> {
        self.inner.read().ok().and_then(|g| g.get(&code).cloned())
    }

    /// 注册 (返回错误信息字符串, 成功 "" / 失败 "code already registered")
    pub fn register(&self, entry: RouteEntry) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|e| e.to_string())?;
        if guard.contains_key(&entry.code) {
            return Err(format!("code {} already registered", entry.code));
        }
        guard.insert(entry.code, entry);
        Ok(())
    }

    /// 全列表 (admin RPC 用)
    pub fn list(&self) -> Vec<RouteEntry> {
        self.inner
            .read()
            .ok()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 当前路由条数
    pub fn len(&self) -> usize {
        self.inner.read().ok().map(|g| g.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_routes_have_10101() {
        let rt = RouteTable::new();
        let entry = rt.get(10101).expect("10101 must be in default routes");
        assert_eq!(entry.code, 10101);
        assert_eq!(entry.target_method, "CreateCharacter");
        assert_eq!(entry.target_service, "player.v1.PlayerService");
    }

    #[test]
    fn register_then_get() {
        let rt = RouteTable::new();
        let entry = RouteEntry {
            code: 99999,
            name: "test".into(),
            target_service: "test.v1.Test".into(),
            target_method: "Ping".into(),
            target_addr: "http://127.0.0.1:9999".into(),
        };
        rt.register(entry.clone()).expect("first register ok");
        let got = rt.get(99999).expect("must find 99999");
        assert_eq!(got.name, "test");
    }

    #[test]
    fn duplicate_register_rejected() {
        let rt = RouteTable::new();
        let entry = RouteEntry {
            code: 10101, // 已在默认表
            name: "dup".into(),
            target_service: "x".into(),
            target_method: "Y".into(),
            target_addr: "z".into(),
        };
        let err = rt.register(entry).unwrap_err();
        assert!(err.contains("already registered"));
    }

    #[test]
    fn list_returns_all_default() {
        let rt = RouteTable::new();
        let list = rt.list();
        assert_eq!(list.len(), DEFAULT_ROUTES.len());
        assert!(list.len() >= 1, "Phase 1 骨架至少 1 条 demo");
    }

    #[test]
    fn len_matches_list() {
        let rt = RouteTable::new();
        assert_eq!(rt.len(), rt.list().len());
    }

    // ===== W7 扩展: 8 域 demo 路由表测试 =====

    #[test]
    fn phase15_demo_has_nine_routes() {
        // 7 域 + cluster_ops = 8 域, 部分域 (scene/battle) 2 条 demo
        // 实际: player 1 + economy 1 + scene 2 + battle 2 + batch 1 + admin 1 + cluster_ops 1 = 9
        assert_eq!(PHASE1_5_DEMO_ROUTES.len(), 9, "8 域 demo 路由条数 (含 scene/battle 各 2)");
    }

    #[test]
    fn phase15_demo_with_factory_constructs_table() {
        let rt = RouteTable::with_phase15_demo();
        assert_eq!(rt.len(), 9);
    }

    #[test]
    fn phase15_demo_covers_seven_domains() {
        // 验证 7 域 (player / economy / scene / battle / batch / admin / cluster_ops) 都有
        let rt = RouteTable::with_phase15_demo();
        let domains: std::collections::HashSet<_> = rt
            .list()
            .iter()
            .map(|e| e.target_service.split('.').next().unwrap_or("?").to_string())
            .collect();
        // 7 域: player / economy / scene / battle / batch / admin / cluster_ops
        assert!(domains.contains("player"));
        assert!(domains.contains("economy"));
        assert!(domains.contains("scene"));
        assert!(domains.contains("battle"));
        assert!(domains.contains("batch"));
        assert!(domains.contains("admin"));
        assert!(domains.contains("cluster_ops"));
        assert_eq!(domains.len(), 7);
    }

    #[test]
    fn phase15_demo_all_codes_hit() {
        let rt = RouteTable::with_phase15_demo();
        for (code, name, _svc, method, _addr) in PHASE1_5_DEMO_ROUTES {
            let entry = rt.get(*code).expect("all 8 codes must hit");
            assert_eq!(entry.code, *code);
            assert_eq!(entry.name, *name);
            assert_eq!(entry.target_method, *method);
        }
    }
}
