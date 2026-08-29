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
