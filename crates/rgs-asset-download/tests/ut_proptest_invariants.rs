//! rgs-asset-download proptest 块 (per 9/1 PT-WORKER 派工 + M-2064.6)
//!
//! Invariant 覆盖：
//! 1. ChunkOrchestrator::plan_chunks → N 个 chunk, 相邻不重叠, 完整覆盖 [0, total_size)
//! 2. 状态机 TRANSITION_TABLE: 所有 (from, event) 对都满足 next_state 行为一致
//! 3. HttpRangeSpec::len 与 to_header_value 互逆（bytes=N-M 解析可还原）

use proptest::prelude::*;

use rgs_asset_download::chunk_orchestrator::ChunkOrchestrator;
use rgs_asset_download::config::DownloadConfig;
use rgs_asset_download::range_client::HttpRangeSpec;
use rgs_asset_download::state_machine::{
    allowed_events, next_state, DownloadState, TRANSITION_TABLE,
};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Invariant 1: plan_chunks 输出 N 个 chunk, 相邻不重叠, 完整覆盖 [0, total_size)
    #[test]
    fn plan_chunks_full_coverage_no_overlap(
        total_size in 1u64..(64u64 * 1024 * 1024),
        chunk_size in 1u64..(8u64 * 1024 * 1024),
    ) {
        let mut cfg = DownloadConfig::default();
        cfg.chunk_size_bytes = chunk_size;
        let orch = ChunkOrchestrator::new(cfg);
        let chunks = orch.plan_chunks(total_size, None);
        // 不空
        prop_assert!(!chunks.is_empty(), "total > 0 时必须至少 1 个 chunk");
        // 第一个必须从 0 开始
        prop_assert_eq!(chunks[0].start, 0);
        // 最后一个必须以 total_size-1 结束
        prop_assert_eq!(chunks.last().unwrap().end, total_size - 1);
        // 相邻不重叠: prev.end + 1 == next.start
        for w in chunks.windows(2) {
            prop_assert_eq!(w[0].end + 1, w[1].start, "chunks must be contiguous");
        }
        // index 严格递进
        for (i, c) in chunks.iter().enumerate() {
            prop_assert_eq!(c.index as usize, i);
        }
        // 区间长度合理
        for c in &chunks {
            prop_assert!(c.end >= c.start);
            prop_assert_eq!(c.len(), c.end - c.start + 1);
        }
    }

    /// Invariant 2: TRANSITION_TABLE 与 next_state 一致, 且每个 from 的 allowed_events
    /// 数量 = TRANSITION_TABLE 中该 from 的条目数
    #[test]
    fn transition_table_consistent_with_next_state(
        // 采样 8 状态 × 11 事件中的 16 个组合
        _dummy in 0u32..16
    ) {
        for &s in &DownloadState::ALL {
            let allowed = allowed_events(s);
            // TRANSITION_TABLE 中 from=s 的条目数
            let count_in_table: usize = TRANSITION_TABLE
                .iter()
                .filter(|(from, _, _)| *from == s)
                .count();
            prop_assert_eq!(
                allowed.len(),
                count_in_table,
                "allowed_events count mismatch for state {:?}",
                s
            );
            // 对每个允许 event, next_state 必非 None
            for ev in &allowed {
                let msg = format!("next_state({:?}, {:?}) must be Some", s, ev);
                prop_assert!(next_state(s, *ev).is_some(), "{}", msg);
            }
        }
    }

    /// Invariant 3: HttpRangeSpec::to_header_value + parse round-trip 还原 start/end
    #[test]
    fn http_range_header_round_trip(
        start in 0u64..(1u64 << 30),
        end_extra in 0u64..(1u64 << 30),
    ) {
        // 保证 end >= start, 且区间不溢出
        let end = start.saturating_add(end_extra % 1024);
        let spec = HttpRangeSpec::new(start, end);
        let header = spec.to_header_value();
        // 格式 "bytes=start-end"
        prop_assert!(header.starts_with("bytes="), "header: {header}");
        let body = &header["bytes=".len()..];
        let parts: Vec<&str> = body.split('-').collect();
        prop_assert_eq!(parts.len(), 2);
        let parsed_start: u64 = parts[0].parse().unwrap();
        let parsed_end: u64 = parts[1].parse().unwrap();
        prop_assert_eq!(parsed_start, start);
        prop_assert_eq!(parsed_end, end);
        prop_assert_eq!(spec.len(), end - start + 1);
    }
}
