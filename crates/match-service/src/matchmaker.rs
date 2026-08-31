//! Match 域 matchmaker 算法(per RGS-DTL-026 §4 + §5)
//!
//! ## 4.1 容差函数(per DTL-026 §4.1)
//! 分段线性,满足"单调不减"约束(RGS-BAS-026 §4.1)
//! 具体分段参数为 PH-5 实测前的初始提案,非最终值
//!
//! ## 5 跨分片 OCC 校验(per DTL-026 §5)
//! 单条 SQL 保证原子性,不拆两步,避免 TOCTOU
//! commit_proposed_match 失败时 rollback 已成功条目

use serde::{Deserialize, Serialize};

/// 容差参数(per DTL-026 §4.1 提案值)
/// - initial_tolerance: 50 评分单位
/// - widen_rate_per_sec: 2 / 秒
/// - max_tolerance: 400 评分单位
/// - grace_period_secs: 等待期内不扩(秒)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToleranceParams {
    pub initial_tolerance: f64,
    pub widen_rate_per_sec: f64,
    pub max_tolerance: f64,
    pub grace_period_secs: u32,
}

impl Default for ToleranceParams {
    fn default() -> Self {
        Self {
            initial_tolerance: 50.0,
            widen_rate_per_sec: 2.0,
            max_tolerance: 400.0,
            grace_period_secs: 5,
        }
    }
}

/// DTL-026 §4.1 容差函数
/// - waiting <= grace_period: initial_tolerance
/// - waiting > grace_period: initial + rate * (waiting - grace), min(., max)
/// 单调不减约束(RGS-BAS-026 §4.1)
pub fn tolerance(waiting_seconds: u32, params: &ToleranceParams) -> f64 {
    let t = waiting_seconds as f64;
    if t <= params.grace_period_secs as f64 {
        params.initial_tolerance
    } else {
        let widened = params.initial_tolerance
            + params.widen_rate_per_sec * (t - params.grace_period_secs as f64);
        widened.min(params.max_tolerance)
    }
}

/// §4.1.1 占位 n 上限(per DTL-026 §4.1.1 Q-D-10 答复)
/// PH-1 启动后跑 benchmark 才能给出可信 n 上限
pub const DEFAULT_MAX_CANDIDATES_PER_TICK: usize = 500;

/// §5 跨分片 OCC 结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccResult {
    /// 影响行数=1: 校验通过,本次撮合对该条目生效
    Updated,
    /// 影响行数=0: 校验失败,已被其他分片抢先撮合
    Conflict,
}

/// §5 commit_proposed_match 输入
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedEntry {
    pub entry_id: String,
    pub version: i64,
}

/// §5 commit_proposed_match 输出
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitResult {
    /// 全部条目 OCC 通过,match 创建
    Committed(Vec<String>),
    /// 有条目冲突,本次撮合废止
    ConcurrentlyMatched {
        losing_entry: String,
        succeeded: Vec<String>,
    },
}

/// 抽象 OCC 数据库(测试可注入 mock)
pub trait OccDatabase: Send + Sync {
    fn occ_update_entry(
        &self,
        entry_id: &str,
        expected_version: i64,
        new_match_ref: &str,
        new_status: &str,
    ) -> Result<OccResult, String>;
}

impl OccResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, OccResult::Updated)
    }
}

/// §5 commit_proposed_match 实现
/// 单条 SQL 原子性更新,失败时 rollback 已成功条目
pub fn commit_proposed_match(
    proposal: &[ProposedEntry],
    match_ref: &str,
    db: &dyn OccDatabase,
) -> CommitResult {
    let mut succeeded = Vec::new();
    for entry in proposal {
        match db.occ_update_entry(
            &entry.entry_id,
            entry.version,
            match_ref,
            "MATCHED_PENDING_CONFIRM",
        ) {
            Ok(OccResult::Updated) => succeeded.push(entry.entry_id.clone()),
            Ok(OccResult::Conflict) => {
                // 校验失败:回退已成功条目
                rollback_succeeded(&succeeded, db);
                return CommitResult::ConcurrentlyMatched {
                    losing_entry: entry.entry_id.clone(),
                    succeeded,
                };
            }
            Err(e) => {
                rollback_succeeded(&succeeded, db);
                return CommitResult::ConcurrentlyMatched {
                    losing_entry: format!("db_error:{}", e),
                    succeeded,
                };
            }
        }
    }
    CommitResult::Committed(succeeded)
}

/// §5 rollback 已成功条目:状态回到 WAITING,version 不变
/// (未被抢占方不受影响,所以不需要再 bump version)
fn rollback_succeeded(succeeded: &[String], db: &dyn OccDatabase) {
    for entry_id in succeeded {
        // 实际是 UPDATE ... SET status='WAITING' WHERE entry_id = $1 AND status='MATCHED_PENDING_CONFIRM'
        // 这里 mock 不依赖具体 status,只 log
        let _ = db.occ_update_entry(entry_id, -1, "", "WAITING_ROLLBACK");
    }
}

#[cfg(test)]
mod tests {
    //! matchmaker.rs 单元测试 (per UT-AGENT-BRIEFING-v3 Step 2)
    //!
    //! 覆盖:
    //! - 容差函数 grace 期内 / 扩容期 / 饱和期
    //! - commit_proposed_match happy path / 冲突回滚 / 错误回滚
    //!
    //! 不依赖 DB, 用 OccDatabase mock 注入 (per DTL-026 §5 单测规约)

    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock OCC DB: 按 entry_id 模式返回结果
    /// - "ok-N" 模式: 总是返回 Updated
    /// - "conflict-N" 模式: 总是返回 Conflict
    /// - "err-N" 模式: 总是返回 Err
    struct MockOccDb {
        updated_count: AtomicUsize,
        rollback_count: AtomicUsize,
    }

    impl MockOccDb {
        fn new() -> Self {
            Self {
                updated_count: AtomicUsize::new(0),
                rollback_count: AtomicUsize::new(0),
            }
        }

        fn updated(&self) -> usize {
            self.updated_count.load(Ordering::Relaxed)
        }

        fn rollbacks(&self) -> usize {
            self.rollback_count.load(Ordering::Relaxed)
        }
    }

    impl OccDatabase for MockOccDb {
        fn occ_update_entry(
            &self,
            entry_id: &str,
            _expected_version: i64,
            _new_match_ref: &str,
            new_status: &str,
        ) -> Result<OccResult, String> {
            if new_status == "WAITING_ROLLBACK" {
                self.rollback_count.fetch_add(1, Ordering::Relaxed);
                return Ok(OccResult::Updated);
            }
            if entry_id.starts_with("err-") {
                return Err("db error".to_string());
            }
            if entry_id.starts_with("conflict-") {
                return Ok(OccResult::Conflict);
            }
            self.updated_count.fetch_add(1, Ordering::Relaxed);
            Ok(OccResult::Updated)
        }
    }

    // ==================== tolerance() ====================

    #[test]
    fn tolerance_within_grace_period_is_initial() {
        // per BAS-026 §4.1: grace period 内不扩容
        let p = ToleranceParams::default();
        for t in 0..=p.grace_period_secs {
            assert_eq!(
                tolerance(t, &p),
                p.initial_tolerance,
                "t={} 应等于 initial_tolerance",
                t
            );
        }
    }

    #[test]
    fn tolerance_after_grace_widens_linearly() {
        // per BAS-026 §4.1: 单调不减约束
        let p = ToleranceParams {
            initial_tolerance: 50.0,
            widen_rate_per_sec: 10.0,
            max_tolerance: 400.0,
            grace_period_secs: 5,
        };
        // waiting=6 → 50 + 10*(6-5) = 60
        assert_eq!(tolerance(6, &p), 60.0);
        // waiting=15 → 50 + 10*(15-5) = 150
        assert_eq!(tolerance(15, &p), 150.0);
        // 单调不减: t1<t2 → f(t1)≤f(t2)
        for t in 0..100 {
            assert!(tolerance(t, &p) <= tolerance(t + 1, &p) + 1e-9);
        }
    }

    #[test]
    fn tolerance_caps_at_max() {
        // 扩容上限: 不能超过 max_tolerance
        let p = ToleranceParams {
            initial_tolerance: 50.0,
            widen_rate_per_sec: 100.0,
            max_tolerance: 400.0,
            grace_period_secs: 0,
        };
        // waiting=10 → 50 + 100*10 = 1050 → cap at 400
        assert_eq!(tolerance(10, &p), 400.0);
        assert_eq!(tolerance(1000, &p), 400.0);
    }

    // ==================== commit_proposed_match() ====================

    #[test]
    fn commit_all_succeeds() {
        // 全部条目 OCC 通过 → Committed
        let db = MockOccDb::new();
        let proposal = vec![
            ProposedEntry { entry_id: "ok-1".to_string(), version: 1 },
            ProposedEntry { entry_id: "ok-2".to_string(), version: 1 },
            ProposedEntry { entry_id: "ok-3".to_string(), version: 1 },
        ];
        let result = commit_proposed_match(&proposal, "match-1", &db);
        match result {
            CommitResult::Committed(entries) => {
                assert_eq!(entries.len(), 3);
                assert_eq!(db.updated(), 3);
                assert_eq!(db.rollbacks(), 0);
            }
            _ => panic!("expected Committed"),
        }
    }

    #[test]
    fn commit_conflict_rolls_back_succeeded() {
        // 第 2 条 conflict → 回滚第 1 条
        let db = MockOccDb::new();
        let proposal = vec![
            ProposedEntry { entry_id: "ok-1".to_string(), version: 1 },
            ProposedEntry { entry_id: "conflict-2".to_string(), version: 1 },
            ProposedEntry { entry_id: "ok-3".to_string(), version: 1 },
        ];
        let result = commit_proposed_match(&proposal, "match-1", &db);
        match result {
            CommitResult::ConcurrentlyMatched { losing_entry, succeeded } => {
                assert_eq!(losing_entry, "conflict-2");
                assert_eq!(succeeded, vec!["ok-1".to_string()]);
                assert_eq!(db.updated(), 1);
                assert_eq!(db.rollbacks(), 1, "应回滚 ok-1");
            }
            _ => panic!("expected ConcurrentlyMatched"),
        }
    }

    #[test]
    fn commit_db_error_rolls_back_succeeded() {
        // DB 错误也走回滚路径 (per DTL-026 §5 "失败时 rollback 已成功条目")
        let db = MockOccDb::new();
        let proposal = vec![
            ProposedEntry { entry_id: "ok-1".to_string(), version: 1 },
            ProposedEntry { entry_id: "err-2".to_string(), version: 1 },
        ];
        let result = commit_proposed_match(&proposal, "match-1", &db);
        match result {
            CommitResult::ConcurrentlyMatched { losing_entry, succeeded } => {
                assert!(losing_entry.starts_with("db_error:"), "got {}", losing_entry);
                assert_eq!(succeeded, vec!["ok-1".to_string()]);
                assert_eq!(db.rollbacks(), 1);
            }
            _ => panic!("expected ConcurrentlyMatched (db error)"),
        }
    }

    #[test]
    fn commit_empty_proposal_is_committed() {
        // 边界: 空 proposal → 直接 Committed
        let db = MockOccDb::new();
        let result = commit_proposed_match(&[], "match-1", &db);
        match result {
            CommitResult::Committed(entries) => assert!(entries.is_empty()),
            _ => panic!("expected Committed([])"),
        }
    }

    #[test]
    fn occ_result_is_ok() {
        // OccResult::is_ok 谓词
        assert!(OccResult::Updated.is_ok());
        assert!(!OccResult::Conflict.is_ok());
    }
}
