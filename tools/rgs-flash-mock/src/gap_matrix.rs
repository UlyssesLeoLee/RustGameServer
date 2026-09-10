// rgs-flash-mock v0.1 — gap matrix (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §4)
// per-RPC coverage tracking, 12 类别 22 RPC (per §3 表)
//
// Status:
// - Pass:        RGS 5 域 + card gRPC backend 支持 (5 类别: 战斗/PVP/经济/排行榜/GM)
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
    Misc,         // 12. 未分类 29 RPC
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

            // 12. 未分类 (29 total, 0 sampled) — v0.1 不抽样
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
            RpcCategory::Pay, RpcCategory::Rank, RpcCategory::Gm, RpcCategory::Misc,
        ].iter().map(|c| c.total_rpc_in_zsyz()).sum();

        let by_category: Vec<CategoryReport> = {
            let mut cats: Vec<RpcCategory> = vec![
                RpcCategory::Scene, RpcCategory::Role, RpcCategory::Combat, RpcCategory::Pvp,
                RpcCategory::Guild, RpcCategory::Econ, RpcCategory::Social, RpcCategory::Event,
                RpcCategory::Pay, RpcCategory::Rank, RpcCategory::Gm, RpcCategory::Misc,
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
