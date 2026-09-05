//! 协议号 → gRPC method 路由表 (per 9/4 改进路线图.md Phase 1 协议网关)
//!
//! ## 设计 (W14 1351 路由 codegen)
//! 1351 条协议号 → gRPC service.method 路由, 全 codegen from `data/api_routes_2026-09-04.tsv`.
//!
//! ## 数据源
//! - 9/4 API 清单-全量提取-2026-09-04.tsv (1351 条) → `data/api_routes_2026-09-04.tsv` (committed)
//! - build.rs codegen: `$OUT_DIR/generated_routes.rs` → `GENERATED_ROUTES` const
//!
//! ## 协议码 → gRPC service 路由映射 (per W6 + W7 8 域 demo)
//! - 1xxx (1000-9999): cluster_ops.v1.ClusterOpsService
//! - 10xxx-19xxx: player.v1.PlayerService
//! - 20xxx-21xxx: battle.v1.BattleService
//! - 22xxx-26xxx: economy.v1.EconomyService
//! - 27xxx-28xxx: social.v1.SocialService
//! - 30xxx: batch.v1.BatchService
//! - 40xxx: admin.v1.AdminService
//! - 50xxx+: cluster_ops.v1.ClusterOpsService
//!
//! ## 9 demo 路由覆写 (per W7 PHASE1_5_DEMO_ROUTES)
//! 10101/10201/10202/20001/20002/20101/30101/40101/50101 共 9 条
//! 强制使用真实 service.method (其余 1342 条用默认 Method_<code>).
//!
//! ## 已知缺口
//! - 113 条 title="(无标题)" 占位 → name = "Unknown" + method = Method_<code>
//! - 协议码 → gRPC 业务方法名: 除 9 demo 路由外, 其余用 Method_<code> 占位 (Phase 2 接 7 域真实 .proto)
//! - 协议码冲突检测: 1351 条全 unique (build.rs 硬断言)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::proto::v1 as gateway_proto_v1;

/// build.rs codegen 1351 条路由 (per W14 task)
include!(concat!(env!("OUT_DIR"), "/generated_routes.rs"));

pub type RouteEntry = gateway_proto_v1::RouteEntry;

/// 协议网关路由表 (动态 + 静态混合)
///
/// Phase 1.5 (W14): RouteTable::new() 直接加载 GENERATED_ROUTES (1351 条 codegen).
/// 内部用 std::sync::RwLock (避免新增 dep), Phase 1.5 评估 parking_lot 复用.
pub struct RouteTable {
    inner: Arc<RwLock<HashMap<u32, RouteEntry>>>,
}

impl Default for RouteTable {
    fn default() -> Self {
        // W14: 1351 全加载 (per task brief + codegen 推进)
        let mut map = HashMap::with_capacity(GENERATED_ROUTES.len());
        for (code, name, svc, method, addr) in GENERATED_ROUTES {
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

    /// W7 兼容: 加载 6 demo 路由 (per 9/4 改进路线图 Phase 2)
    ///
    /// W14 调整: W7 PHASE1_5_DEMO_ROUTES 9 条中有 5 条 (20101/10202/30101/40101/50101) 在
    /// 9/4 API 清单 TSV 中不存在 (TSV 来自真实 Erlang 源, 9 域合成 code 不可用).
    /// W14 demo 兼容缩为 6 条 (10101/10201/20001/20002/11000/25000), 集成测试仍可 roundtrip.
    pub fn with_phase15_demo() -> Self {
        // 6 demo 路由对应 code (W14 调整: 仅 TSV 真实存在的)
        const DEMO_CODES: &[u32] = &[10101, 10201, 20001, 20002, 11000, 25000];
        let mut map = HashMap::with_capacity(DEMO_CODES.len());
        for (code, name, svc, method, addr) in GENERATED_ROUTES {
            if DEMO_CODES.contains(code) {
                let entry = RouteEntry {
                    code: *code,
                    name: (*name).to_string(),
                    target_service: (*svc).to_string(),
                    target_method: (*method).to_string(),
                    target_addr: (*addr).to_string(),
                };
                map.insert(*code, entry);
            }
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

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_1351_routes_loaded() {
        // W14 codegen: 1351 全加载
        let rt = RouteTable::new();
        assert_eq!(rt.len(), 1351, "Phase 1.5 codegen 全部 1351 条加载");
        assert_eq!(rt.list().len(), 1351);
    }

    #[test]
    fn test_each_route_has_gRPC_target() {
        // 每条都有 svc + method
        let rt = RouteTable::new();
        for entry in rt.list() {
            assert!(!entry.target_service.is_empty(), "code {} 缺 service", entry.code);
            assert!(
                entry.target_service.contains('.'),
                "code {} service 应为 'pkg.Service' 形式: {}",
                entry.code,
                entry.target_service
            );
            assert!(!entry.target_method.is_empty(), "code {} 缺 method", entry.code);
            assert!(
                !entry.target_addr.is_empty() && entry.target_addr.starts_with("http"),
                "code {} addr 错: {}",
                entry.code,
                entry.target_addr
            );
        }
    }

    #[test]
    fn test_specific_route_10101() {
        // 10101 → player.CreateCharacter (per W7 9 demo + 9/4 TSV 第 1 条)
        let rt = RouteTable::new();
        let entry = rt.get(10101).expect("10101 must be registered");
        assert_eq!(entry.code, 10101);
        assert_eq!(entry.target_service, "player.v1.PlayerService");
        assert_eq!(entry.target_method, "CreateCharacter");
        assert_eq!(entry.target_addr, "http://127.0.0.1:50051");
    }

    #[test]
    fn test_specific_route_10201_enter_scene() {
        // 10201 → scene.EnterScene (per W7 9 demo + 9/4 TSV 真实存在)
        let rt = RouteTable::new();
        let entry = rt.get(10201).expect("10201 must be registered");
        assert_eq!(entry.target_service, "scene.v1.SceneService");
        assert_eq!(entry.target_method, "EnterScene");
    }

    #[test]
    fn test_specific_route_20001_battle_prepare() {
        // 20001 → battle.BattlePrepare (per W7 9 demo + 9/4 TSV 真实存在 "战斗准备")
        let rt = RouteTable::new();
        let entry = rt.get(20001).expect("20001 must be registered");
        assert_eq!(entry.target_service, "battle.v1.BattleService");
        assert_eq!(entry.target_method, "BattlePrepare");
    }

    #[test]
    fn test_specific_route_20002_round_start() {
        // 20002 → battle.RoundStart (per 9/4 TSV 真实存在 "回合开始")
        let rt = RouteTable::new();
        let entry = rt.get(20002).expect("20002 must be registered");
        assert_eq!(entry.target_service, "battle.v1.BattleService");
        assert_eq!(entry.target_method, "RoundStart");
    }

    #[test]
    fn test_specific_route_11000_partner_data() {
        // 11000 → player.GetPartnerData (per 9/4 TSV 真实存在 "请求伙伴数据")
        let rt = RouteTable::new();
        let entry = rt.get(11000).expect("11000 must be registered");
        assert_eq!(entry.target_service, "player.v1.PlayerService");
        assert_eq!(entry.target_method, "GetPartnerData");
    }

    #[test]
    fn test_specific_route_25000_push_base_info() {
        // 25000 → economy.PushBaseInfo (per 9/4 TSV 真实存在 "推送基础信息")
        let rt = RouteTable::new();
        let entry = rt.get(25000).expect("25000 must be registered");
        assert_eq!(entry.target_service, "economy.v1.EconomyService");
        assert_eq!(entry.target_method, "PushBaseInfo");
    }

    #[test]
    fn test_routes_cover_real_domains() {
        // 实际 codegen 覆盖: player / scene / battle / economy (per 9/4 TSV 真实分布)
        // batch / admin / cluster_ops 是 9/4 TSV 合成 域, 不在真实 1351 里
        let rt = RouteTable::new();
        let domains: std::collections::HashSet<_> = rt
            .list()
            .iter()
            .map(|e| {
                e.target_service
                    .split('.')
                    .next()
                    .unwrap_or("?")
                    .to_string()
            })
            .collect();
        assert!(domains.contains("player"), "应有 player 域");
        assert!(domains.contains("scene"), "应有 scene 域");
        assert!(domains.contains("battle"), "应有 battle 域");
        assert!(domains.contains("economy"), "应有 economy 域");
        // 5 域实际覆盖 (player + scene + battle + economy + cluster_ops legacy 1xxx)
        assert!(domains.contains("cluster_ops"), "1xxx legacy → cluster_ops");
    }

    #[test]
    fn test_routes_have_unique_codes() {
        // 1351 unique codes (per build.rs 硬断言)
        let rt = RouteTable::new();
        let list = rt.list();
        let unique: std::collections::HashSet<_> = list.iter().map(|e| e.code).collect();
        assert_eq!(unique.len(), list.len(), "code 必须 unique");
    }

    #[test]
    fn test_unknown_title_fallback() {
        // 113 条 "(无标题)" 占位: name 应为 "Unknown" 或派生名
        let rt = RouteTable::new();
        // 10211 (per W6 9/4 TSV 检查) 是 (无标题) 之一
        let entry = rt.get(10211).expect("10211 must be registered");
        // name 字段: build.rs 中 "Unknown" 或 file-derived
        // 不强求具体值, 但必须非空
        assert!(!entry.name.is_empty(), "code {} name 必须非空", entry.code);
    }

    #[test]
    fn test_default_route_uses_method_code_pattern() {
        // 1345 默认路由: method = Method_<code>
        let rt = RouteTable::new();
        // 10210 是真实存在但非覆写, 应该是默认 mapping
        let entry = rt.get(10210).expect("10210 must be registered");
        assert!(
            entry.target_method.starts_with("Method_"),
            "默认路由 method 应以 Method_ 开头: {}",
            entry.target_method
        );
    }

    // ===== W7 兼容测试 (与 integration_phase15_demo.rs 配套) =====

    #[test]
    fn phase15_demo_compat_has_six_routes() {
        // W7 兼容: with_phase15_demo() 返回 6 条 demo (W14 调整: 9 → 6, 仅 TSV 真实存在的)
        let rt = RouteTable::with_phase15_demo();
        assert_eq!(rt.len(), 6, "W14 6 demo 路由兼容 (W7 9 中 5 个 code 在 TSV 不存在)");
    }

    #[test]
    fn phase15_demo_compat_covers_real_domains() {
        // W7 兼容: 真实存在的 demo 域 (player / scene / battle / economy)
        let rt = RouteTable::with_phase15_demo();
        let domains: std::collections::HashSet<_> = rt
            .list()
            .iter()
            .map(|e| e.target_service.split('.').next().unwrap_or("?").to_string())
            .collect();
        assert!(domains.contains("player"));
        assert!(domains.contains("scene"));
        assert!(domains.contains("battle"));
        assert!(domains.contains("economy"));
        assert_eq!(domains.len(), 4, "W14 6 demo 跨 4 真实域");
    }

    #[test]
    fn phase15_demo_compat_all_codes_hit() {
        // W7 兼容: 6 demo codes 全部命中
        let rt = RouteTable::with_phase15_demo();
        let demo_codes: &[u32] = &[10101, 10201, 20001, 20002, 11000, 25000];
        for code in demo_codes {
            let entry = rt.get(*code).expect("all 6 codes must hit");
            assert_eq!(entry.code, *code);
        }
    }
}
