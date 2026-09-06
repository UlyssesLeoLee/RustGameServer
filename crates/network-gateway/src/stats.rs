//! 协议网关统计 (per W6 task GetStats RPC)
//!
//! ## 维度
//! - 收包总数 (per code + 总数)
//! - 成功转发数 (gRPC 调用成功)
//! - 失败数 (gRPC 错误 / 协议错 / 路由 miss)
//! - 路由 miss 数 (收到未注册 code)
//! - 当前 TCP 活跃连接数
//!
//! ## 实现
//! Phase 1 骨架: std::sync::atomic::AtomicU64 (无锁, 5 域已用). 后续可换 prometheus
//! 直采 (per Phase 4 指标).

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct GatewayStats {
    pub total_received: AtomicU64,
    pub total_forwarded: AtomicU64,
    pub total_failed: AtomicU64,
    pub total_route_miss: AtomicU64,
    pub active_connections: AtomicU64,
}

impl GatewayStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_received(&self) {
        self.total_received.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_forwarded(&self) {
        self.total_forwarded.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_failed(&self) {
        self.total_failed.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_route_miss(&self) {
        self.total_route_miss.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_active(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }
    pub fn dec_active(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            total_received: self.total_received.load(Ordering::Relaxed),
            total_forwarded: self.total_forwarded.load(Ordering::Relaxed),
            total_failed: self.total_failed.load(Ordering::Relaxed),
            total_route_miss: self.total_route_miss.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatsSnapshot {
    pub total_received: u64,
    pub total_forwarded: u64,
    pub total_failed: u64,
    pub total_route_miss: u64,
    pub active_connections: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_initially_zero() {
        let s = GatewayStats::new();
        let snap = s.snapshot();
        assert_eq!(snap.total_received, 0);
        assert_eq!(snap.total_forwarded, 0);
        assert_eq!(snap.total_failed, 0);
        assert_eq!(snap.total_route_miss, 0);
        assert_eq!(snap.active_connections, 0);
    }

    #[test]
    fn counters_increment_correctly() {
        let s = GatewayStats::new();
        s.inc_received();
        s.inc_received();
        s.inc_forwarded();
        s.inc_failed();
        s.inc_route_miss();
        s.inc_active();
        s.inc_active();
        s.dec_active();

        let snap = s.snapshot();
        assert_eq!(snap.total_received, 2);
        assert_eq!(snap.total_forwarded, 1);
        assert_eq!(snap.total_failed, 1);
        assert_eq!(snap.total_route_miss, 1);
        assert_eq!(snap.active_connections, 1);
    }
}
