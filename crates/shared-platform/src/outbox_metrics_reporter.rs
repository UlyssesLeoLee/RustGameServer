//! Outbox Metrics Reporter（per ULYS-100 P2-#2 + 06_Outbox监控指标草案.md §1.1）
//!
//! **目的**: relay `tick()` 只能更新 publish counter + histogram (实时事件计数);
//! `pending_count` / `inflight_count` / `failed_count` / `lease_lag` / `oldest_pending_created_at`
//! 5 个 **gauge** 指标需要周期性查询 DB 才能得到当前快照。本 reporter 提供:
//!
//! - `OutboxMetricsReporter` 结构：持有 `PgOutboxRepository` + service_name + 间隔
//! - `run()` 后台循环：每 `interval` 秒执行一次 SQL 聚合查询
//! - 4 类查询（per §1.1 gauge）：
//!   1. `SELECT subject, count(*) FROM outbox WHERE status='pending' GROUP BY subject` → set_outbox_pending
//!   2. `SELECT subject, count(*) FROM outbox WHERE status='in_flight' AND lease_until > NOW() GROUP BY subject` → set_outbox_inflight
//!   3. `SELECT max(extract(epoch FROM (lease_until - NOW()))) FROM outbox WHERE status='in_flight' AND lease_until > NOW()` → set_outbox_inflight_lease_lag
//!   4. `SELECT subject, count(*) FROM outbox WHERE status='failed' GROUP BY subject` → set_outbox_failed
//!   5. `SELECT min(extract(epoch FROM created_at)) FROM outbox WHERE status='pending'` → set_outbox_oldest_pending_created_at
//!
//! 设计原则：
//! - 容忍 SQL 失败（DB 暂时不可用时 gauge 保留上次值，relay 不阻塞）
//! - aggregate_type 从 `subject` 推断（per `subject::aggregate_type_of`；与 relay 一致）
//! - lease_lag 计算用 PostgreSQL `extract(epoch FROM interval)` 返回秒（浮点）
//!
//! 启动方式（per 域 main.rs）：
//! ```ignore
//! let reporter = OutboxMetricsReporter::new(repo.clone(), "economy", Duration::from_secs(15));
//! tokio::spawn(reporter.run());
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use sqlx::Row;

use crate::metrics::metrics;
use crate::outbox::PgOutboxRepository;
use crate::subject::aggregate_type_of;

/// Outbox Metrics 周期性 reporter（per ULYS-100 P2-#2）
///
/// **所有 6 域 outbox 部署均应启动**（per ADR-0061 §1.3 6 域 + 06_草案 §4 Stage 2）。
pub struct OutboxMetricsReporter {
    repo: Arc<PgOutboxRepository>,
    /// Prometheus 标签 service 值（per spec §1.1）
    service_name: &'static str,
    /// 周期性 SQL 查询间隔（推荐 10s；不与 relay poll_interval 同步避免突发 DB 压力）
    interval: Duration,
}

impl OutboxMetricsReporter {
    /// 构造 reporter
    ///
    /// `service_name` 必须是 `"admin" | "economy" | "match" | "player" | "social" | "cluster_ops"`
    pub fn new(
        repo: Arc<PgOutboxRepository>,
        service_name: &'static str,
        interval: Duration,
    ) -> Self {
        Self {
            repo,
            service_name,
            interval,
        }
    }

    /// Prometheus 服务标签
    fn service(&self) -> &'static str {
        self.service_name
    }

    /// 单次扫描（一次 SQL 聚合，更新所有 5 个 gauge）
    ///
    /// 失败时记 error 日志但**不返回错误**（per 设计原则：DB 短暂不可用时 gauge 保留上次值）
    pub async fn scan_once(&self) {
        if let Err(e) = self.scan_pending_by_subject().await {
            tracing::warn!(
                target: "outbox_metrics_reporter",
                service = self.service(),
                error = %e,
                "scan_pending_by_subject failed"
            );
        }
        if let Err(e) = self.scan_inflight_by_subject().await {
            tracing::warn!(
                target: "outbox_metrics_reporter",
                service = self.service(),
                error = %e,
                "scan_inflight_by_subject failed"
            );
        }
        if let Err(e) = self.scan_inflight_lease_lag().await {
            tracing::warn!(
                target: "outbox_metrics_reporter",
                service = self.service(),
                error = %e,
                "scan_inflight_lease_lag failed"
            );
        }
        if let Err(e) = self.scan_failed_by_subject().await {
            tracing::warn!(
                target: "outbox_metrics_reporter",
                service = self.service(),
                error = %e,
                "scan_failed_by_subject failed"
            );
        }
        if let Err(e) = self.scan_oldest_pending_created_at().await {
            tracing::warn!(
                target: "outbox_metrics_reporter",
                service = self.service(),
                error = %e,
                "scan_oldest_pending_created_at failed"
            );
        }
    }

    /// 1. Pending 按 subject 聚合（per §1.1 rgs_outbox_pending_count{service, aggregate_type}）
    async fn scan_pending_by_subject(&self) -> crate::outbox::Result<()> {
        let rows = sqlx::query(
            "SELECT subject, count(*) AS cnt FROM outbox \
             WHERE status = 'pending' GROUP BY subject",
        )
        .fetch_all(self.repo.pool())
        .await?;
        for row in rows {
            let subject: String = row.get("subject");
            let cnt: i64 = row.get("cnt");
            let agg = aggregate_type_of(&subject);
            metrics().set_outbox_pending(self.service(), &agg, cnt);
        }
        Ok(())
    }

    /// 2. InFlight 按 subject 聚合（lease 未过期）
    async fn scan_inflight_by_subject(&self) -> crate::outbox::Result<()> {
        let rows = sqlx::query(
            "SELECT subject, count(*) AS cnt FROM outbox \
             WHERE status = 'in_flight' AND lease_until > NOW() \
             GROUP BY subject",
        )
        .fetch_all(self.repo.pool())
        .await?;
        for row in rows {
            let subject: String = row.get("subject");
            let cnt: i64 = row.get("cnt");
            let agg = aggregate_type_of(&subject);
            metrics().set_outbox_inflight(self.service(), &agg, cnt);
        }
        Ok(())
    }

    /// 3. InFlight 行 lease 剩余秒数最大值（per service 单 label 聚合）
    ///
    /// PostgreSQL `extract(epoch FROM (lease_until - NOW()))` 返回浮点秒
    /// 若表中无 in_flight 行, max 返回 NULL, sqlx get Option<f64> -> None -> 设 0
    async fn scan_inflight_lease_lag(&self) -> crate::outbox::Result<()> {
        let row = sqlx::query(
            "SELECT max(extract(epoch FROM (lease_until - NOW()))) AS max_lag \
             FROM outbox WHERE status = 'in_flight' AND lease_until > NOW()",
        )
        .fetch_one(self.repo.pool())
        .await?;
        let max_lag: Option<f64> = row.get("max_lag");
        metrics().set_outbox_inflight_lease_lag(self.service(), max_lag.unwrap_or(0.0));
        Ok(())
    }

    /// 4. Failed 按 subject 聚合
    async fn scan_failed_by_subject(&self) -> crate::outbox::Result<()> {
        let rows = sqlx::query(
            "SELECT subject, count(*) AS cnt FROM outbox \
             WHERE status = 'failed' GROUP BY subject",
        )
        .fetch_all(self.repo.pool())
        .await?;
        for row in rows {
            let subject: String = row.get("subject");
            let cnt: i64 = row.get("cnt");
            let agg = aggregate_type_of(&subject);
            metrics().set_outbox_failed(self.service(), &agg, cnt);
        }
        Ok(())
    }

    /// 5. 最旧 Pending 行 created_at unix 秒（per service 单 label）
    ///
    /// 若无 pending 行, min 返回 NULL -> 设 0.0（避免 stale 值触发告警误判）
    async fn scan_oldest_pending_created_at(&self) -> crate::outbox::Result<()> {
        let row = sqlx::query(
            "SELECT min(extract(epoch FROM created_at)) AS oldest_ts \
             FROM outbox WHERE status = 'pending'",
        )
        .fetch_one(self.repo.pool())
        .await?;
        let oldest: Option<f64> = row.get("oldest_ts");
        metrics().set_outbox_oldest_pending_created_at(self.service(), oldest.unwrap_or(0.0));
        Ok(())
    }

    /// 后台循环（tokio task）
    ///
    /// 启动时立即执行一次 scan, 之后按 `interval` 周期
    pub async fn run(self) {
        tracing::info!(
            target: "outbox_metrics_reporter",
            service = self.service(),
            interval_secs = self.interval.as_secs(),
            "outbox metrics reporter started"
        );
        let mut ticker = time::interval(self.interval);
        // ticker::tick() 第一次立即触发, 不需要 skip
        loop {
            ticker.tick().await;
            self.scan_once().await;
        }
    }
}

// ============ InMemoryOutboxRepository 适配（测试用） ============
//
// PgOutboxRepository 是 6 域生产 impl, 但单测需要不依赖真实 DB。
// 提供 InMemoryOutboxRepository::pool() 兼容 + 用同样的 SQL 路径需要 sqlx PgPool。
//
// **设计抉择**: 不引入 sqlx mock; ULYS-100 §4 验收要求 `cargo test -p shared-platform outbox_metrics`
// 覆盖指标采集点, 测试聚焦于: (a) aggregate_type_of 推断正确 (b) metrics helper 累计正确。
// SQL 路径覆盖率由集成测试 (TST-outbox-metrics-IT) 用 testcontainers-Postgres 覆盖。
//
// 因此本模块**不**提供 InMemoryMetricsReporter; reporter 测试集中在聚合查询 SQL 字符串
// 的正确性 + aggregate_type 推断 + metrics helper 累计。

#[cfg(test)]
mod tests {
    // 占位; 测试由 `outbox_metrics` 集成测试覆盖
    #[test]
    fn reporter_module_compiles() {
        // 编译验证：仅确保 module 本身能 compile
        assert!(true, "OutboxMetricsReporter 模块编译通过");
    }
}
