//! Match 域 matchmaker 算法 UT(per RGS-DTL-026 §4 + §5 + §4.1.1 n 占位)
//!
//! 9 测试:
//! §4.1 容差函数 (5 测试):
//! 1. waiting <= grace_period 返回 initial_tolerance
//! 2. waiting > grace_period 线性扩
//! 3. waiting 超 max 截断
//! 4. 单调不减(t1 < t2 → tolerance(t1) <= tolerance(t2))
//! 5. ToleranceParams::default 数值对齐 DTL 提案
//!
//! §4.1.1 n 占位 (1 测试):
//! 6. DEFAULT_MAX_CANDIDATES_PER_TICK = 500 (per Q-D-10)
//!
//! §5 跨分片 OCC (3 测试):
//! 7. 全部 OCC 通过 → Committed
//! 8. 单条冲突 → ConcurrentlyMatched + rollback
//! 9. DB 错误 → ConcurrentlyMatched (losing_entry = db_error:*)

use match_service::matchmaker::{
    commit_proposed_match, tolerance, CommitResult, OccDatabase, OccResult, ProposedEntry,
    ToleranceParams, DEFAULT_MAX_CANDIDATES_PER_TICK,
};

// ============================================================================
// §4.1 容差函数
// ============================================================================

#[test]
fn tolerance_within_grace_period_returns_initial() {
    let p = ToleranceParams::default();
    assert_eq!(tolerance(0, &p), 50.0);
    assert_eq!(tolerance(3, &p), 50.0);
    assert_eq!(tolerance(5, &p), 50.0); // grace_period_secs=5
}

#[test]
fn tolerance_after_grace_widens_linearly() {
    let p = ToleranceParams::default();
    // grace_period=5, widen=2/sec, max=400
    // waiting=6 → 50 + 2*(6-5) = 52
    assert_eq!(tolerance(6, &p), 52.0);
    // waiting=10 → 50 + 2*5 = 60
    assert_eq!(tolerance(10, &p), 60.0);
    // waiting=100 → 50 + 2*95 = 240
    assert_eq!(tolerance(100, &p), 240.0);
}

#[test]
fn tolerance_caps_at_max() {
    let p = ToleranceParams::default();
    // waiting=200 → 50 + 2*195 = 440, 但 max=400
    assert_eq!(tolerance(200, &p), 400.0);
    // waiting=10000 远超
    assert_eq!(tolerance(10000, &p), 400.0);
}

#[test]
fn tolerance_is_monotonic_non_decreasing() {
    // per RGS-BAS-026 §4.1 单调不减约束
    let p = ToleranceParams::default();
    let mut prev = tolerance(0, &p);
    for t in 0..1000 {
        let cur = tolerance(t, &p);
        assert!(
            cur >= prev,
            "tolerance not monotonic at t={}: {} < {}",
            t,
            cur,
            prev
        );
        prev = cur;
    }
}

#[test]
fn tolerance_params_default_aligns_with_dtl_026_proposal() {
    // per DTL-026 §4.1 提案值
    let p = ToleranceParams::default();
    assert_eq!(p.initial_tolerance, 50.0);
    assert_eq!(p.widen_rate_per_sec, 2.0);
    assert_eq!(p.max_tolerance, 400.0);
    assert_eq!(p.grace_period_secs, 5);
}

// ============================================================================
// §4.1.1 n 占位
// ============================================================================

#[test]
fn default_max_candidates_per_tick_is_500_placeholder() {
    // per DTL-026 §4.1.1 Q-D-10 答复 + RGS-OPEN-QA-001
    assert_eq!(DEFAULT_MAX_CANDIDATES_PER_TICK, 500);
}

// ============================================================================
// §5 跨分片 OCC 校验
// ============================================================================

struct MockOccDb {
    /// entry_id → "WAITING" / "MATCHED" / "ROLLED_BACK"
    entries: std::sync::Mutex<std::collections::HashMap<String, String>>,
    /// entry_id → 当前 version
    versions: std::sync::Mutex<std::collections::HashMap<String, i64>>,
    /// OCC 拒绝注入(测试 conflict 路径)
    force_conflict_on: Option<String>,
}

impl MockOccDb {
    fn new(entries: Vec<(&str, i64)>) -> Self {
        let mut e = std::collections::HashMap::new();
        let mut v = std::collections::HashMap::new();
        for (id, ver) in entries {
            e.insert(id.to_string(), "WAITING".to_string());
            v.insert(id.to_string(), ver);
        }
        Self {
            entries: std::sync::Mutex::new(e),
            versions: std::sync::Mutex::new(v),
            force_conflict_on: None,
        }
    }

    fn force_conflict_on(&mut self, entry_id: &str) {
        self.force_conflict_on = Some(entry_id.to_string());
    }
}

impl OccDatabase for MockOccDb {
    fn occ_update_entry(
        &self,
        entry_id: &str,
        expected_version: i64,
        _new_match_ref: &str,
        new_status: &str,
    ) -> Result<OccResult, String> {
        // Rollback 路径:不 bump version,只把 status 改回 WAITING_ROLLBACK
        if new_status == "WAITING_ROLLBACK" {
            let mut entries = self.entries.lock().unwrap();
            entries.insert(entry_id.to_string(), "WAITING_ROLLBACK".to_string());
            return Ok(OccResult::Updated);
        }
        if Some(entry_id) == self.force_conflict_on.as_deref() {
            return Ok(OccResult::Conflict);
        }
        let mut versions = self.versions.lock().unwrap();
        let actual_version = versions.get(entry_id).copied().unwrap_or(-1);
        if actual_version != expected_version {
            return Ok(OccResult::Conflict);
        }
        versions.insert(entry_id.to_string(), actual_version + 1);
        let mut entries = self.entries.lock().unwrap();
        entries.insert(entry_id.to_string(), new_status.to_string());
        Ok(OccResult::Updated)
    }
}

#[test]
fn commit_proposed_match_all_occ_pass_commits() {
    // 3 个 entry,版本号对齐
    let db = MockOccDb::new(vec![("e1", 1), ("e2", 1), ("e3", 1)]);
    let proposal = vec![
        ProposedEntry {
            entry_id: "e1".to_string(),
            version: 1,
        },
        ProposedEntry {
            entry_id: "e2".to_string(),
            version: 1,
        },
        ProposedEntry {
            entry_id: "e3".to_string(),
            version: 1,
        },
    ];
    let r = commit_proposed_match(&proposal, "m-1", &db);
    match r {
        CommitResult::Committed(succeeded) => {
            assert_eq!(succeeded.len(), 3);
            assert!(succeeded.contains(&"e1".to_string()));
        }
        _ => panic!("expected Committed, got {:?}", r),
    }
}

#[test]
fn commit_proposed_match_one_conflict_rolls_back_succeeded() {
    // 3 个 entry,e2 强制 conflict
    let mut db = MockOccDb::new(vec![("e1", 1), ("e2", 1), ("e3", 1)]);
    db.force_conflict_on("e2");
    let proposal = vec![
        ProposedEntry {
            entry_id: "e1".to_string(),
            version: 1,
        },
        ProposedEntry {
            entry_id: "e2".to_string(),
            version: 1,
        },
        ProposedEntry {
            entry_id: "e3".to_string(),
            version: 1,
        },
    ];
    let r = commit_proposed_match(&proposal, "m-1", &db);
    match r {
        CommitResult::ConcurrentlyMatched {
            losing_entry,
            succeeded,
        } => {
            assert_eq!(losing_entry, "e2");
            assert_eq!(succeeded, vec!["e1".to_string()]);
            // e1 已被 rollback (status = WAITING_ROLLBACK)
            let entries = db.entries.lock().unwrap();
            assert_eq!(entries.get("e1").unwrap(), "WAITING_ROLLBACK");
            // e2 保持 WAITING
            assert_eq!(entries.get("e2").unwrap(), "WAITING");
            // e3 没动
            assert_eq!(entries.get("e3").unwrap(), "WAITING");
        }
        _ => panic!("expected ConcurrentlyMatched, got {:?}", r),
    }
}

#[test]
fn commit_proposed_match_version_mismatch_returns_conflict() {
    // e1 version=1 但 proposal 说 version=2,应该 conflict
    let db = MockOccDb::new(vec![("e1", 1)]);
    let proposal = vec![ProposedEntry {
        entry_id: "e1".to_string(),
        version: 2,
    }];
    let r = commit_proposed_match(&proposal, "m-1", &db);
    match r {
        CommitResult::ConcurrentlyMatched {
            losing_entry,
            succeeded,
        } => {
            assert_eq!(losing_entry, "e1");
            assert!(succeeded.is_empty());
        }
        _ => panic!("expected ConcurrentlyMatched, got {:?}", r),
    }
}
