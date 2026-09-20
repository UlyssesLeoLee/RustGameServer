// rgs-flash-mock v0.3 — gap matrix (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §4)
// per-RPC coverage tracking, 13 类别 23 RPC (per §3 表, v0.3 加 card)
//
// Status:
// - Pass:        RGS 5 域 + card + leaderboard gRPC backend 支持 (6 类别: 战斗/PVP/经济/排行榜/GM/卡牌)
// - Partial:     RGS 部分支持 (5 类别: 养成/公会/社交/活动/付费)
// - NotApplicable: RGS 品类不适用 (1 类别: 场景/移动, RGS TCG 无场景)
// - NotImplemented: mock v0.1 不抽样 (v0.2+ 补)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RpcStatus {
    Pass,
    Partial,
    NotApplicable,
    NotImplemented,
}

impl RpcStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RpcStatus::Pass => "pass",
            RpcStatus::Partial => "partial",
            RpcStatus::NotApplicable => "n-a",
            RpcStatus::NotImplemented => "not-implemented",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcCategory {
    Scene,        // 1. 场景/移动 148 RPC
    Role,         // 2. 角色养成 198 RPC
    Combat,       // 3. 战斗 PVE 241 RPC
    Pvp,          // 4. PVP/竞技 151 RPC
    Guild,        // 5. 公会 97 RPC
    Econ,         // 6. 经济 90 RPC
    Social,       // 7. 社交 123 RPC
    Event,        // 8. 活动运营 184 RPC
    Pay,          // 9. 付费/商业化 43 RPC
    Rank,         // 10. 排行榜/图鉴 10 RPC
    Gm,           // 11. GM/运维 37 RPC
    Card,         // 12. 卡牌/收藏 (v0.3 NEW, per RGS-DTL-038 §4.4) 80 RPC
    Misc,         // 13. 未分类 29 RPC
}

impl RpcCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RpcCategory::Scene => "scene",
            RpcCategory::Role => "role",
            RpcCategory::Combat => "combat",
            RpcCategory::Pvp => "pvp",
            RpcCategory::Guild => "guild",
            RpcCategory::Econ => "econ",
            RpcCategory::Social => "social",
            RpcCategory::Event => "event",
            RpcCategory::Pay => "pay",
            RpcCategory::Rank => "rank",
            RpcCategory::Gm => "gm",
            RpcCategory::Card => "card",
            RpcCategory::Misc => "misc",
        }
    }

    pub fn total_rpc_in_zsyz(&self) -> u32 {
        match self {
            RpcCategory::Scene => 148,
            RpcCategory::Role => 198,
            RpcCategory::Combat => 241,
            RpcCategory::Pvp => 151,
            RpcCategory::Guild => 97,
            RpcCategory::Econ => 90,
            RpcCategory::Social => 123,
            RpcCategory::Event => 184,
            RpcCategory::Pay => 43,
            RpcCategory::Rank => 10,
            RpcCategory::Gm => 37,
            RpcCategory::Card => 80, // v0.3 NEW (per RGS-DTL-038 §4.4 估算)
            RpcCategory::Misc => 29,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRecord {
    pub code: u32,
    pub name: String,
    pub category: RpcCategory,
    pub status: RpcStatus,
    pub description: String,
    pub rgs_backend: String,
    pub call_count: u64,
    pub last_latency_ms: u64,
    pub last_called_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category: String,
    pub zsyz_total: u32,
    pub mock_sampled: u32,
    pub pass: u32,
    pub partial: u32,
    pub not_applicable: u32,
    pub not_implemented: u32,
    pub coverage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub service: String,
    pub version: String,
    pub zsyz_total: u32,
    pub mock_sampled: u32,
    pub pass: u32,
    pub partial: u32,
    pub not_applicable: u32,
    pub not_implemented: u32,
    pub overall_coverage_pct: f64,
    pub by_category: Vec<CategoryReport>,
    pub rpcs: Vec<RpcRecord>,
}

pub struct GapMatrix {
    records: HashMap<u32, RpcRecord>,
    started_at: Instant,
}

impl GapMatrix {
    pub fn new() -> Self {
        let mut records = HashMap::new();

        // 12 类别 22 RPC stub 注册 (per design §3 表)
        let rpcs: Vec<(u32, &str, RpcCategory, RpcStatus, &str, &str)> = vec![
            // 1. 场景/移动 (148 total, 2 sampled) — RGS TCG 无场景, N-A
            (101, "GetScene", RpcCategory::Scene, RpcStatus::NotApplicable,
             "场景信息查询", "match (match_id routing) + player (session)"),
            (102, "MovePlayer", RpcCategory::Scene, RpcStatus::NotApplicable,
             "玩家移动", "(无对应)"),

            // 2. 角色养成 (198 total, 2 sampled) — RGS 部分类比 (卡组养成), Partial
            (201, "GetPlayerProfile", RpcCategory::Role, RpcStatus::Partial,
             "玩家档案查询", "player (PlayerProfile)"),
            (202, "UpgradeSkill", RpcCategory::Role, RpcStatus::Partial,
             "技能升级", "card (CardInstance.level)"),

            // 3. 战斗 PVE (241 total, 2 sampled) — RGS match v2, Pass
            (301, "StartCombat", RpcCategory::Combat, RpcStatus::Pass,
             "开始战斗", "match (CreateMatch)"),
            (302, "SubmitAction", RpcCategory::Combat, RpcStatus::Pass,
             "提交动作", "match (SubmitMove)"),

            // 4. PVP/竞技 (151 total, 2 sampled) — RGS match v2, Pass
            (401, "EnqueuePVP", RpcCategory::Pvp, RpcStatus::Pass,
             "PVP 排队", "match (EnqueueMatchmaking)"),
            (402, "GetPVPMatch", RpcCategory::Pvp, RpcStatus::Pass,
             "PVP 比赛查询", "match (GetMatchState)"),

            // 5. 公会 (97 total, 2 sampled) — RGS social 4/6 handler 未 wire, Partial
            (501, "GetGuild", RpcCategory::Guild, RpcStatus::Partial,
             "公会信息查询", "social (HealthCheck + get_guild stub)"),
            (502, "JoinGuild", RpcCategory::Guild, RpcStatus::Partial,
             "加入公会", "social (gRPC handler 未 wire)"),

            // 6. 经济 (90 total, 2 sampled) — RGS economy v2, Pass
            (601, "GetAccount", RpcCategory::Econ, RpcStatus::Pass,
             "账户查询", "economy (GetAccount)"),
            (602, "CreateAuction", RpcCategory::Econ, RpcStatus::Pass,
             "创建拍卖", "economy (CreateAuction)"),

            // 7. 社交 (123 total, 2 sampled) — RGS social 缺好友/邮件, Partial
            (701, "GetFriendList", RpcCategory::Social, RpcStatus::Partial,
             "好友列表查询", "social (mock 友好)"),
            (702, "SendMessage", RpcCategory::Social, RpcStatus::Partial,
             "发送消息", "social (mock)"),

            // 8. 活动运营 (184 total, 2 sampled) — RGS 缺数据驱动活动, Partial
            (801, "GetActiveEvent", RpcCategory::Event, RpcStatus::Partial,
             "活动查询", "batch (task_templates) + card (AddCardToCollection)"),
            (802, "ClaimReward", RpcCategory::Event, RpcStatus::Partial,
             "领取奖励", "batch + card"),

            // 9. 付费/商业化 (43 total, 2 sampled) — RGS 抽卡/开包不同, Partial
            (901, "Recharge", RpcCategory::Pay, RpcStatus::Partial,
             "充值", "economy + payment (mock)"),
            (902, "QueryRechargeHistory", RpcCategory::Pay, RpcStatus::Partial,
             "充值历史", "economy"),

            // 10. 排行榜 (10 total, 1 sampled) — RGS leaderboard, Pass
            (1001, "GetLeaderboard", RpcCategory::Rank, RpcStatus::Pass,
             "排行榜", "leaderboard (现有)"),

            // 11. GM (37 total, 2 sampled) — RGS admin + gm-backend, Pass
            (1101, "BanAccount", RpcCategory::Gm, RpcStatus::Pass,
             "封号", "admin (BanAccount) + gm-backend (同 RPC)"),
            (1102, "GrantCompensation", RpcCategory::Gm, RpcStatus::Pass,
             "补偿发放", "admin + gm-backend"),

            // 12. 卡牌/收藏 (80 total, 1 sampled, v0.3 NEW) — RGS card-service v0.2, Pass
            (1201, "GetPlayerCollection", RpcCategory::Card, RpcStatus::Pass,
             "卡牌收藏查询", "card (GetPlayerCollection)"),

            // 13. 未分类 (29 total, 0 sampled) — v0.1 不抽样
            // v0.2+ 补
        ];

        for (code, name, category, status, description, backend) in rpcs {
            records.insert(code, RpcRecord {
                code,
                name: name.to_string(),
                category,
                status,
                description: description.to_string(),
                rgs_backend: backend.to_string(),
                call_count: 0,
                last_latency_ms: 0,
                last_called_at: None,
            });
        }

        Self {
            records,
            started_at: Instant::now(),
        }
    }

    pub fn total(&self) -> usize {
        self.records.len()
    }

    pub fn count_by_status(&self, status: RpcStatus) -> u32 {
        self.records.values().filter(|r| r.status == status).count() as u32
    }

    pub fn record_call(&mut self, code: u32, _category: RpcCategory, _status: RpcStatus, _description: &str) {
        if let Some(rec) = self.records.get_mut(&code) {
            rec.call_count += 1;
        }
    }

    pub fn record_response(&mut self, code: u32, _status: RpcStatus, latency_ms: u64) {
        if let Some(rec) = self.records.get_mut(&code) {
            rec.last_latency_ms = latency_ms;
            rec.last_called_at = Some(chrono::Utc::now());
        }
    }

    pub fn report(&self) -> CoverageReport {
        let zsyz_total: u32 = [
            RpcCategory::Scene, RpcCategory::Role, RpcCategory::Combat, RpcCategory::Pvp,
            RpcCategory::Guild, RpcCategory::Econ, RpcCategory::Social, RpcCategory::Event,
            RpcCategory::Pay, RpcCategory::Rank, RpcCategory::Gm, RpcCategory::Card, RpcCategory::Misc,
        ].iter().map(|c| c.total_rpc_in_zsyz()).sum();

        let by_category: Vec<CategoryReport> = {
            let mut cats: Vec<RpcCategory> = vec![
                RpcCategory::Scene, RpcCategory::Role, RpcCategory::Combat, RpcCategory::Pvp,
                RpcCategory::Guild, RpcCategory::Econ, RpcCategory::Social, RpcCategory::Event,
                RpcCategory::Pay, RpcCategory::Rank, RpcCategory::Gm, RpcCategory::Card, RpcCategory::Misc,
            ];
            cats.dedup();
            cats.iter().map(|cat| {
                let rpcs_in_cat: Vec<&RpcRecord> = self.records.values().filter(|r| r.category == *cat).collect();
                let sampled = rpcs_in_cat.len() as u32;
                let pass = rpcs_in_cat.iter().filter(|r| r.status == RpcStatus::Pass).count() as u32;
                let partial = rpcs_in_cat.iter().filter(|r| r.status == RpcStatus::Partial).count() as u32;
                let na = rpcs_in_cat.iter().filter(|r| r.status == RpcStatus::NotApplicable).count() as u32;
                let ni = rpcs_in_cat.iter().filter(|r| r.status == RpcStatus::NotImplemented).count() as u32;
                let coverage = if cat.total_rpc_in_zsyz() == 0 { 0.0 } else { (sampled as f64 / cat.total_rpc_in_zsyz() as f64) * 100.0 };
                CategoryReport {
                    category: cat.as_str().to_string(),
                    zsyz_total: cat.total_rpc_in_zsyz(),
                    mock_sampled: sampled,
                    pass, partial, not_applicable: na, not_implemented: ni,
                    coverage_pct: (coverage * 100.0).round() / 100.0,
                }
            }).collect()
        };

        let mut rpcs: Vec<RpcRecord> = self.records.values().cloned().collect();
        rpcs.sort_by_key(|r| r.code);

        let mock_sampled = rpcs.len() as u32;
        let pass = self.count_by_status(RpcStatus::Pass);
        let partial = self.count_by_status(RpcStatus::Partial);
        let na = self.count_by_status(RpcStatus::NotApplicable);
        let ni = self.count_by_status(RpcStatus::NotImplemented);
        let coverage = (mock_sampled as f64 / zsyz_total as f64) * 100.0;

        CoverageReport {
            service: "rgs-flash-mock".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            zsyz_total,
            mock_sampled,
            pass, partial, not_applicable: na, not_implemented: ni,
            overall_coverage_pct: (coverage * 100.0).round() / 100.0,
            by_category,
            rpcs,
        }
    }
}

#[cfg(test)]
mod tests {
    //! UT for rgs-flash-mock gap_matrix (per ULYS-141 + RGS-TEST-DESIGN v0.2 §1 L1.1)
    //!
    //! Coverage:
    //!   - RpcStatus::as_str() — 4 状态映射
    //!   - RpcCategory::as_str() — 13 类别名称
    //!   - RpcCategory::total_rpc_in_zsyz() — 闪烁之光原版 RPC 总数
    //!   - GapMatrix::new() — 22 RPC stub 注册正确
    //!   - GapMatrix::count_by_status() — 各状态计数
    //!   - GapMatrix::record_call() — 调用计数累加
    //!   - GapMatrix::record_response() — 延迟 + 调用时间戳
    //!   - GapMatrix::report() — 覆盖率 + by_category 完整性
    //!
    //! 派生约束守护 (per AGENTS.md §2.1 L1):
    //!   - L1: cargo check --tests 0 error
    //!   - L1.1: cargo test --lib 全过
    //!   - 凭据永不打印 (8/27 11:06 JST hard ban)
    //!   - 代签规则 (8/27 19:39/20:56/21:59 JST 三次强化)

    use super::*;

    // ---- RpcStatus ----

    #[test]
    fn rpc_status_as_str_all_variants() {
        assert_eq!(RpcStatus::Pass.as_str(), "pass");
        assert_eq!(RpcStatus::Partial.as_str(), "partial");
        assert_eq!(RpcStatus::NotApplicable.as_str(), "n-a");
        assert_eq!(RpcStatus::NotImplemented.as_str(), "not-implemented");
    }

    #[test]
    fn rpc_status_total_count_is_four() {
        // 保证派生约束: 状态机封闭 (per design §4)
        let all = [
            RpcStatus::Pass,
            RpcStatus::Partial,
            RpcStatus::NotApplicable,
            RpcStatus::NotImplemented,
        ];
        for s in all {
            // each status maps to non-empty string
            assert!(!s.as_str().is_empty());
        }
        assert_eq!(all.len(), 4);
    }

    // ---- RpcCategory ----

    #[test]
    fn rpc_category_as_str_all_13() {
        // 13 类别 (per v0.3: 12 + card)
        assert_eq!(RpcCategory::Scene.as_str(), "scene");
        assert_eq!(RpcCategory::Role.as_str(), "role");
        assert_eq!(RpcCategory::Combat.as_str(), "combat");
        assert_eq!(RpcCategory::Pvp.as_str(), "pvp");
        assert_eq!(RpcCategory::Guild.as_str(), "guild");
        assert_eq!(RpcCategory::Econ.as_str(), "econ");
        assert_eq!(RpcCategory::Social.as_str(), "social");
        assert_eq!(RpcCategory::Event.as_str(), "event");
        assert_eq!(RpcCategory::Pay.as_str(), "pay");
        assert_eq!(RpcCategory::Rank.as_str(), "rank");
        assert_eq!(RpcCategory::Gm.as_str(), "gm");
        assert_eq!(RpcCategory::Card.as_str(), "card");
        assert_eq!(RpcCategory::Misc.as_str(), "misc");
    }

    #[test]
    fn rpc_category_total_rpc_in_zsyz_matches_design() {
        // 13 类别 RPC 总数 (per design §3 表, v0.3 加 card=80)
        assert_eq!(RpcCategory::Scene.total_rpc_in_zsyz(), 148);
        assert_eq!(RpcCategory::Role.total_rpc_in_zsyz(), 198);
        assert_eq!(RpcCategory::Combat.total_rpc_in_zsyz(), 241);
        assert_eq!(RpcCategory::Pvp.total_rpc_in_zsyz(), 151);
        assert_eq!(RpcCategory::Guild.total_rpc_in_zsyz(), 97);
        assert_eq!(RpcCategory::Econ.total_rpc_in_zsyz(), 90);
        assert_eq!(RpcCategory::Social.total_rpc_in_zsyz(), 123);
        assert_eq!(RpcCategory::Event.total_rpc_in_zsyz(), 184);
        assert_eq!(RpcCategory::Pay.total_rpc_in_zsyz(), 43);
        assert_eq!(RpcCategory::Rank.total_rpc_in_zsyz(), 10);
        assert_eq!(RpcCategory::Gm.total_rpc_in_zsyz(), 37);
        assert_eq!(RpcCategory::Card.total_rpc_in_zsyz(), 80);
        assert_eq!(RpcCategory::Misc.total_rpc_in_zsyz(), 29);
    }

    #[test]
    fn rpc_category_total_sum_matches_zsyz_full_count() {
        // 闪烁之光原版 RPC 总和 (148+198+241+151+97+90+123+184+43+10+37+80+29 = 1431)
        let cats = [
            RpcCategory::Scene, RpcCategory::Role, RpcCategory::Combat, RpcCategory::Pvp,
            RpcCategory::Guild, RpcCategory::Econ, RpcCategory::Social, RpcCategory::Event,
            RpcCategory::Pay, RpcCategory::Rank, RpcCategory::Gm, RpcCategory::Card,
            RpcCategory::Misc,
        ];
        let sum: u32 = cats.iter().map(|c| c.total_rpc_in_zsyz()).sum();
        assert_eq!(sum, 1431, "闪烁之光 RPC 总数必须等于 1431 (12 + card v0.3)");
    }

    // ---- GapMatrix construction ----

    #[test]
    fn gap_matrix_new_has_22_rpc_stubs() {
        // 12 类别 22 RPC stub (per design §3 表, v0.3 加 card=1201 GetPlayerCollection)
        let m = GapMatrix::new();
        assert_eq!(m.total(), 22, "GapMatrix::new() 必须注册 22 RPC stub");
    }

    #[test]
    fn gap_matrix_new_contains_sampled_rpc_codes() {
        // 抽样 RPC 编号必须存在 (101/102/201/202/301/302/...)
        let m = GapMatrix::new();
        let codes = [101, 102, 201, 202, 301, 302, 401, 402, 501, 502, 601, 602, 701, 702, 801, 802, 901, 902, 1001, 1101, 1102, 1201];
        for c in codes {
            assert!(m.count_by_status(RpcStatus::Pass) + m.count_by_status(RpcStatus::Partial)
                + m.count_by_status(RpcStatus::NotApplicable)
                + m.count_by_status(RpcStatus::NotImplemented) > 0);
            // 间接验证: count 包含此 code
            let _ = c;
        }
        // 直接通过 report() 验证
        let r = m.report();
        let rcode_set: std::collections::HashSet<u32> = r.rpcs.iter().map(|x| x.code).collect();
        for c in codes {
            assert!(rcode_set.contains(&c), "code {} 必须在 report.rpcs 中", c);
        }
    }

    // ---- GapMatrix::count_by_status ----

    #[test]
    fn gap_matrix_count_by_status_initial_distribution() {
        // 初始注册: 2 NotApplicable (scene 101/102) + 8 Partial (role/guild/social/event/pay)
        //          + 8 Pass (combat/pvp/econ/rank/gm/card 部分) + 0 NotImplemented
        // 实际: 2 N-A (scene) + 7 Partial (role/guild/social/event/pay 5 类 = 10 个) + 8 Pass + 3 Pass (gm/rank/card) = 22
        let m = GapMatrix::new();
        let na = m.count_by_status(RpcStatus::NotApplicable);
        let p = m.count_by_status(RpcStatus::Pass);
        let pa = m.count_by_status(RpcStatus::Partial);
        let ni = m.count_by_status(RpcStatus::NotImplemented);
        assert_eq!(na + p + pa + ni, 22, "4 状态计数之和必须等于 22 RPC");
        assert_eq!(na, 2, "scene 2 个 NotApplicable (101/102)");
        assert_eq!(ni, 0, "v0.1 不抽样, NotImplemented 初始为 0");
        assert!(p > 0 && pa > 0, "Pass + Partial 都必须有正计数");
    }

    // ---- GapMatrix::record_call ----

    #[test]
    fn gap_matrix_record_call_increments_counter() {
        let mut m = GapMatrix::new();
        let before = m.report().rpcs.iter().find(|r| r.code == 301).unwrap().call_count;
        m.record_call(301, RpcCategory::Combat, RpcStatus::Pass, "测试调用");
        let after = m.report().rpcs.iter().find(|r| r.code == 301).unwrap().call_count;
        assert_eq!(after, before + 1, "record_call 必须累加 call_count");
    }

    #[test]
    fn gap_matrix_record_call_multiple_calls() {
        let mut m = GapMatrix::new();
        for _ in 0..5 {
            m.record_call(201, RpcCategory::Role, RpcStatus::Partial, "多次调用");
        }
        let rec = m.report().rpcs.iter().find(|r| r.code == 201).unwrap().clone();
        assert_eq!(rec.call_count, 5, "5 次 record_call 必须累加到 5");
    }

    #[test]
    fn gap_matrix_record_call_unknown_code_is_silent() {
        // 未知 code 不应 panic (per fail-closed 精神 8/27 55.26)
        let mut m = GapMatrix::new();
        m.record_call(99999, RpcCategory::Scene, RpcStatus::Pass, "未知 code");
        // 不应 panic, count 仍为 0
        assert_eq!(m.report().rpcs.iter().find(|r| r.code == 99999).map(|r| r.call_count), None);
    }

    // ---- GapMatrix::record_response ----

    #[test]
    fn gap_matrix_record_response_sets_latency_and_timestamp() {
        let mut m = GapMatrix::new();
        m.record_response(301, RpcStatus::Pass, 42);
        let rec = m.report().rpcs.iter().find(|r| r.code == 301).unwrap().clone();
        assert_eq!(rec.last_latency_ms, 42, "latency 必须记录");
        assert!(rec.last_called_at.is_some(), "last_called_at 必须被设置");
    }

    #[test]
    fn gap_matrix_record_response_overwrites_previous_latency() {
        let mut m = GapMatrix::new();
        m.record_response(201, RpcStatus::Partial, 100);
        m.record_response(201, RpcStatus::Partial, 50);
        let rec = m.report().rpcs.iter().find(|r| r.code == 201).unwrap().clone();
        assert_eq!(rec.last_latency_ms, 50, "最新 latency 必须覆盖旧值");
    }

    // ---- GapMatrix::report ----

    #[test]
    fn gap_matrix_report_metadata_correct() {
        let m = GapMatrix::new();
        let r = m.report();
        assert_eq!(r.service, "rgs-flash-mock");
        assert_eq!(r.zsyz_total, 1431, "闪烁之光原版 RPC 总数");
        assert_eq!(r.mock_sampled, 22);
        // pass + partial + n-a + not-implemented 必须 == mock_sampled
        assert_eq!(r.pass + r.partial + r.not_applicable + r.not_implemented, r.mock_sampled);
    }

    #[test]
    fn gap_matrix_report_has_13_category_breakdown() {
        let m = GapMatrix::new();
        let r = m.report();
        assert_eq!(r.by_category.len(), 13, "13 类别必须全部出现, 包括 0 sampled 的 misc");
    }

    #[test]
    fn gap_matrix_report_coverage_pct_in_range() {
        let m = GapMatrix::new();
        let r = m.report();
        // 22/1431 ≈ 1.54%
        assert!(r.overall_coverage_pct >= 1.0 && r.overall_coverage_pct <= 2.0,
                "coverage_pct 必须 ~1.54%, 实际 {}", r.overall_coverage_pct);
    }

    #[test]
    fn gap_matrix_report_rpcs_sorted_by_code() {
        let m = GapMatrix::new();
        let r = m.report();
        let codes: Vec<u32> = r.rpcs.iter().map(|x| x.code).collect();
        let mut sorted = codes.clone();
        sorted.sort();
        assert_eq!(codes, sorted, "report.rpcs 必须按 code 升序");
    }

    #[test]
    fn gap_matrix_category_misc_has_zero_coverage() {
        // Misc 类别未抽样 (per design v0.1 defer v0.2+)
        let m = GapMatrix::new();
        let r = m.report();
        let misc = r.by_category.iter().find(|c| c.category == "misc").unwrap();
        assert_eq!(misc.mock_sampled, 0);
        assert_eq!(misc.coverage_pct, 0.0);
    }
}
