//! rgs-flash-mock gap matrix
//!
//! per-RPC 覆盖率跟踪 + GET /coverage endpoint
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §4 gap matrix schema

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcStatus {
    /// RGS 已实现 + mock 调用成功
    Pass,
    /// RGS 部分实现 (e.g. trait 6 method, gRPC 2 wire)
    Partial,
    /// RGS 未实装 (mock 返回 placeholder)
    NotImplemented,
    /// RGS 品类不适用 (e.g. 场景/移动 TCG 无)
    NotApplicable,
    /// 调用 RGS 失败 (gRPC error)
    Error,
}

impl RpcStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Partial => "Partial",
            Self::NotImplemented => "NotImplemented",
            Self::NotApplicable => "NotApplicable",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRecord {
    pub rpc_code: u32,
    pub category: String,
    pub rpc_name: String,
    pub rgs_backend: String,
    pub rgs_rpc: String,
    pub status: RpcStatus,
    pub last_latency_ms: Option<f64>,
    pub call_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub timestamp: DateTime<Utc>,
    pub total_rpcs: usize,
    pub by_status: HashMap<String, usize>,
    pub by_category: HashMap<String, CategoryCoverage>,
    pub overall_coverage: String,
    pub rpcs: Vec<RpcRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCoverage {
    pub total: usize,
    pub pass: usize,
    pub partial: usize,
    pub not_implemented: usize,
    pub not_applicable: usize,
    pub error: usize,
    pub coverage: String,
}

/// 全局 gap matrix 状态 (in-memory, per v0.1 设计)
#[derive(Clone)]
pub struct GapMatrix {
    inner: Arc<RwLock<HashMap<u32, RpcRecord>>>,
}

impl GapMatrix {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册 RPC (启动时从 handlers 加载, v0.1 手动注册)
    pub async fn register(&self, record: RpcRecord) {
        let mut map = self.inner.write().await;
        map.insert(record.rpc_code, record);
    }

    /// 记录一次调用 (成功)
    pub async fn record_call(&self, rpc_code: u32, latency_ms: f64) {
        self.record_call_with_status(rpc_code, RpcStatus::Pass, latency_ms).await;
    }

    /// 记录一次调用 (指定状态)
    pub async fn record_call_with_status(&self, rpc_code: u32, status: RpcStatus, latency_ms: f64) {
        let mut map = self.inner.write().await;
        if let Some(record) = map.get_mut(&rpc_code) {
            record.call_count += 1;
            record.last_seen_at = Utc::now();
            record.last_latency_ms = Some(latency_ms);
            match status {
                RpcStatus::Pass => {
                    record.success_count += 1;
                    record.status = RpcStatus::Pass;
                }
                RpcStatus::Partial => {
                    record.success_count += 1;
                    record.status = RpcStatus::Partial;
                }
                _ => {
                    record.failure_count += 1;
                    record.status = status;
                }
            }
        }
    }

    /// 生成 coverage report (GET /coverage 返回)
    pub async fn report(&self) -> CoverageReport {
        let map = self.inner.read().await;
        let rpcs: Vec<RpcRecord> = map.values().cloned().collect();
        let total = rpcs.len();

        let mut by_status: HashMap<String, usize> = HashMap::new();
        let mut by_category: HashMap<String, CategoryCoverage> = HashMap::new();

        for record in &rpcs {
            *by_status.entry(record.status.as_str().to_string()).or_insert(0) += 1;

            let cat = by_category.entry(record.category.clone()).or_insert(CategoryCoverage {
                total: 0,
                pass: 0,
                partial: 0,
                not_implemented: 0,
                not_applicable: 0,
                error: 0,
                coverage: "0%".to_string(),
            });
            cat.total += 1;
            match record.status {
                RpcStatus::Pass => cat.pass += 1,
                RpcStatus::Partial => cat.partial += 1,
                RpcStatus::NotImplemented => cat.not_implemented += 1,
                RpcStatus::NotApplicable => cat.not_applicable += 1,
                RpcStatus::Error => cat.error += 1,
            }
        }

        // 计算每类覆盖率 (Pass + Partial) / total
        for cat in by_category.values_mut() {
            if cat.total > 0 {
                let covered = cat.pass + cat.partial;
                let pct = (covered * 100) / cat.total;
                cat.coverage = format!("{}%", pct);
            }
        }

        // 整体覆盖率 (Pass + Partial) / total
        let covered: usize = by_status.get("Pass").copied().unwrap_or(0)
            + by_status.get("Partial").copied().unwrap_or(0);
        let overall_coverage = if total > 0 {
            format!("{}%", (covered * 100) / total)
        } else {
            "N/A".to_string()
        };

        CoverageReport {
            timestamp: Utc::now(),
            total_rpcs: total,
            by_status,
            by_category,
            overall_coverage,
            rpcs,
        }
    }
}

impl Default for GapMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// 12 大类 RPC 初始注册 (per 设计 doc §3 v0.1 抽样 22 RPC)
pub fn initial_rpc_records() -> Vec<RpcRecord> {
    let now = Utc::now();
    let mk = |rpc_code: u32, category: &str, rpc_name: &str, rgs_backend: &str, rgs_rpc: &str, status: RpcStatus| RpcRecord {
        rpc_code,
        category: category.to_string(),
        rpc_name: rpc_name.to_string(),
        rgs_backend: rgs_backend.to_string(),
        rgs_rpc: rgs_rpc.to_string(),
        status,
        last_latency_ms: None,
        call_count: 0,
        success_count: 0,
        failure_count: 0,
        first_seen_at: now,
        last_seen_at: now,
    };

    vec![
        // 1. 场景/移动 (148 RPC) — TCG 不适用
        mk(101, "场景/移动", "GetScene", "match-service:50053", "GetMatch", RpcStatus::NotApplicable),
        mk(102, "场景/移动", "MovePlayer", "player-service:50051", "(无对应)", RpcStatus::NotApplicable),
        // 2. 角色养成 (198 RPC) — 部分类比
        mk(201, "角色养成", "GetPlayerProfile", "player-service:50051", "GetPlayerProfile", RpcStatus::Partial),
        mk(202, "角色养成", "UpgradeSkill", "card-service:50061", "(部分, CardInstance.level)", RpcStatus::Partial),
        // 3. 战斗 PVE (241 RPC) — RGS match v2
        mk(301, "战斗 PVE", "StartCombat", "match-service:50053", "CreateMatch", RpcStatus::Pass),
        mk(302, "战斗 PVE", "SubmitAction", "match-service:50053", "SubmitMove", RpcStatus::Pass),
        // 4. PVP/竞技 (151 RPC) — RGS match v2
        mk(401, "PVP/竞技", "EnqueuePVP", "match-service:50053", "EnqueueMatchmaking", RpcStatus::Pass),
        mk(402, "PVP/竞技", "GetPVPMatch", "match-service:50053", "GetMatchState", RpcStatus::Pass),
        // 5. 公会 (97 RPC) — RGS social gRPC 4/6 handler 未 wire
        mk(501, "公会", "GetGuild", "social-service:50054", "HealthCheck (get_guild stub)", RpcStatus::Partial),
        mk(502, "公会", "JoinGuild", "social-service:50054", "(gRPC handler 未 wire)", RpcStatus::Partial),
        // 6. 经济 (90 RPC) — RGS economy v2
        mk(601, "经济", "GetAccount", "economy-service:50052", "GetAccount", RpcStatus::Pass),
        mk(602, "经济", "CreateAuction", "economy-service:50052", "CreateAuction", RpcStatus::Pass),
        // 7. 社交 (123 RPC) — RGS social 缺好友/邮件
        mk(701, "社交", "GetFriendList", "social-service:50054", "(缺, social 0 import)", RpcStatus::NotImplemented),
        mk(702, "社交", "SendMessage", "social-service:50054", "(缺)", RpcStatus::NotImplemented),
        // 8. 活动运营 (184 RPC) — RGS 缺数据驱动活动框架
        mk(801, "活动运营", "GetActiveEvent", "batch-backend:8790", "task_templates Master", RpcStatus::Partial),
        mk(802, "活动运营", "ClaimReward", "card-service:50061", "AddCardToCollection.source=Event", RpcStatus::Partial),
        // 9. 付费/商业化 (43 RPC) — RGS 抽卡/开包不同
        mk(901, "付费/商业化", "Recharge", "economy-service:50052", "(pay 模块缺)", RpcStatus::NotImplemented),
        mk(902, "付费/商业化", "QueryRechargeHistory", "economy-service:50052", "(缺)", RpcStatus::NotImplemented),
        // 10. 排行榜/图鉴 (10 RPC) — RGS leaderboard 域
        mk(1001, "排行榜/图鉴", "GetLeaderboard", "leaderboard-service:50056", "(leaderboard 域)", RpcStatus::Pass),
        // 11. GM/运维 (37 RPC) — RGS admin + gm-backend
        mk(1101, "GM/运维", "BanAccount", "admin-service:50055", "BanAccount", RpcStatus::Pass),
        mk(1102, "GM/运维", "GrantCompensation", "admin-service:50055", "GrantCompensation", RpcStatus::Pass),
        // 12. 未分类 (29 RPC) — v0.1 不抽样
        // (待 v0.2+ 补)
    ]
}
