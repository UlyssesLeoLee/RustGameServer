//! ArchiveOperator::query_archive helper (P1-3 第二步, 2026-09-05)
//!
//! 归档查询 (延迟指标)
//! per M-2074.5
//!
//! **拆分原因**: impl ArchiveOperator 624 行单文件, 拆 4 个超大 method body 到 helper,
//! archive.rs 主 impl 留 8 行 forward 委派, 总大小 1443 → ~700 行.
//! **pub API 不变**, ArchiveOperator::query_archive(...) 调用, 内部委托 helper.

use super::super::archive::*;

/// 归档查询 (延迟指标)
    pub(super) async fn query_archive<Q, R>(
    {
        let start = Instant::now();
        let result = query_fn.await;
        let elapsed = start.elapsed().as_secs_f64();

        // 强制采集延迟指标（即便查询失败也要记录，per DTL-042 §11.1 实测参数）
        metrics::observe_archive_query_latency(
            query_kind,
            tier_to_metric_label(realm_status),
            elapsed,
        );

        result
    }
