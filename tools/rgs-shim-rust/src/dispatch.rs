// shim v0.5 dispatch table (per 9/9 19:38 JST design 5acc8e9 + 5 worker merge 9/9 22:30 JST)
// 5 worker 业务 handler merge 后, main.rs 走 Dispatcher → Domain 路由 → registry 查表
//
// v0.5 价值:
// 1. cmd range 分类 (Domain::from_cmd) — 给 per-domain hook 留位
// 2. per-domain AtomicU64 counter — 业务可观测性
// 3. 未知 cmd stub fallback — 兼容 5 worker 进度差异 (per L24 派生约束)
//
// 后续 v0.6: 替换 dispatch_player/.../admin 内部为 5 域 gRPC client (经 rgs-proxy 8084)

use crate::handlers::Response;
use crate::rgs::RgsClient;
use crate::registry::Registry;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Player,
    Economy,
    Battle,
    Social,
    Admin,
    ShimInternal,
    Unknown,
}

impl Domain {
    /// cmd 范围 → 域 service (per SHIM_V05_DISPATCH_DESIGN.md §3.2 + 5 worker merge 实测)
    /// 注意: 13000-14999 + 16000-16999 全 social (w4 范围), 14000-14999 不再分给 admin
    /// 11000-11999 (partner) + 24000-24999 (welfare) + 30000-30100 (gift) 全 admin (w5)
    /// 24000-24999 必须放 20000-29999 之前 (subset, Rust 优先匹配前者)
    pub fn from_cmd(cmd: u16) -> Self {
        match cmd {
            // shim-internal (per 9/9 14:55 JST 拍板, 跟 erlang 10400/11001 冲突)
            10400 | 11001 => Domain::ShimInternal,
            // 5 worker 业务 handler 范围 (不重叠, sub-range 优先)
            10000..=10999 => Domain::Player,
            11000..=11999 => Domain::Admin, // w5 partner
            13000..=14999 | 16000..=16999 => Domain::Social, // w4 social
            19000..=19999 | 25000..=25999 => Domain::Battle, // w3 battle
            24000..=24999 => Domain::Admin, // w5 welfare (subset of 20000-29999 economy, 先匹配)
            20000..=29999 => Domain::Economy, // w2 economy
            30000..=30100 => Domain::Admin, // w5 gift
            // unknown: stub fallback
            _ => Domain::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Domain::Player => "player",
            Domain::Economy => "economy",
            Domain::Battle => "battle",
            Domain::Social => "social",
            Domain::Admin => "admin",
            Domain::ShimInternal => "shim-internal",
            Domain::Unknown => "unknown",
        }
    }
}

/// per-domain cmd 计数 (业务可观测性, per v0.5 design §3.4.7 metric)
#[derive(Default, Debug)]
pub struct DomainMetrics {
    pub player: AtomicU64,
    pub economy: AtomicU64,
    pub battle: AtomicU64,
    pub social: AtomicU64,
    pub admin: AtomicU64,
    pub shim_internal: AtomicU64,
    pub unknown: AtomicU64,
    pub stub_fallback: AtomicU64,
}

impl DomainMetrics {
    pub fn incr(&self, d: Domain) {
        let c = match d {
            Domain::Player => &self.player,
            Domain::Economy => &self.economy,
            Domain::Battle => &self.battle,
            Domain::Social => &self.social,
            Domain::Admin => &self.admin,
            Domain::ShimInternal => &self.shim_internal,
            Domain::Unknown => &self.unknown,
        };
        c.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_stub(&self) {
        self.stub_fallback.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Vec<(&'static str, u64)> {
        vec![
            ("player", self.player.load(Ordering::Relaxed)),
            ("economy", self.economy.load(Ordering::Relaxed)),
            ("battle", self.battle.load(Ordering::Relaxed)),
            ("social", self.social.load(Ordering::Relaxed)),
            ("admin", self.admin.load(Ordering::Relaxed)),
            ("shim-internal", self.shim_internal.load(Ordering::Relaxed)),
            ("unknown", self.unknown.load(Ordering::Relaxed)),
            ("stub-fallback", self.stub_fallback.load(Ordering::Relaxed)),
        ]
    }
}

pub struct Dispatcher {
    rgs: Arc<RgsClient>,
    registry: Arc<Registry>,
    metrics: Arc<DomainMetrics>,
}

impl Dispatcher {
    pub fn new(rgs: Arc<RgsClient>, registry: Arc<Registry>) -> Self {
        Self {
            rgs,
            registry,
            metrics: Arc::new(DomainMetrics::default()),
        }
    }

    pub fn metrics(&self) -> Arc<DomainMetrics> {
        self.metrics.clone()
    }

    /// Dispatch a cmd to its handler.
    /// 流程: cmd → Domain 分类 → registry 查表 (5 worker merge 后 766+ cmd 全注册) → handler
    /// 未知 cmd: stub fallback (per L24 派生约束, 返 code:0 + msg:OK)
    pub async fn dispatch(&self, cmd: u16, payload: Vec<u8>) -> Response {
        let domain = Domain::from_cmd(cmd);
        self.metrics.incr(domain);

        // 5 worker merge 后, registry 已有 766+ cmd 真实 handler, 直接查表
        let resp = self.registry.dispatch(cmd, payload, self.rgs.clone()).await;

        // 未知 cmd (registry 没注册): stub fallback
        if resp.payload.is_empty() {
            // 注: payload empty 可能是 real handler 真返 0 数据, 也可能是 stub
            // 当前 dispatch 路径: 5 worker handler 全有, registry 不返 empty, 所以这里主要是 "registry 真没注册" 的情况
            self.metrics.incr_stub();
        }

        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_from_cmd() {
        // shim-internal
        assert_eq!(Domain::from_cmd(10400), Domain::ShimInternal);
        assert_eq!(Domain::from_cmd(11001), Domain::ShimInternal);
        // 5 worker 业务 handler 范围 (实测 disjoint)
        assert_eq!(Domain::from_cmd(10101), Domain::Player);
        assert_eq!(Domain::from_cmd(20000), Domain::Economy);
        assert_eq!(Domain::from_cmd(19800), Domain::Battle);
        assert_eq!(Domain::from_cmd(25100), Domain::Battle);
        assert_eq!(Domain::from_cmd(13000), Domain::Social);
        assert_eq!(Domain::from_cmd(16600), Domain::Social);
        assert_eq!(Domain::from_cmd(14100), Domain::Social); // 14000-14999 全 social, 不再 admin
        assert_eq!(Domain::from_cmd(11050), Domain::Admin); // w5 partner
        assert_eq!(Domain::from_cmd(30100), Domain::Admin); // w5 gift
        assert_eq!(Domain::from_cmd(24500), Domain::Admin); // w5 welfare
        // unknown
        assert_eq!(Domain::from_cmd(65535), Domain::Unknown);
    }

    #[test]
    fn test_metrics_incr() {
        let m = DomainMetrics::default();
        m.incr(Domain::Player);
        m.incr(Domain::Player);
        m.incr(Domain::Economy);
        m.incr_stub();
        let snap = m.snapshot();
        let player = snap.iter().find(|(n, _)| *n == "player").unwrap().1;
        let economy = snap.iter().find(|(n, _)| *n == "economy").unwrap().1;
        let stub = snap.iter().find(|(n, _)| *n == "stub-fallback").unwrap().1;
        assert_eq!(player, 2);
        assert_eq!(economy, 1);
        assert_eq!(stub, 1);
    }
}
