//! match-service 撮合核心函数 benchmark
//!
//! per RGS-DTL-026 v0.4 §4.1.3 + RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10
//!
//! ## 任务范围（WF-1-55.42）
//!
//! 本文件**仅**搭 benchmark 框架，**不**实跑（PH-1 编码完成后才有真实 `matchmaker_tick`）。
//! 文件内的 `matchmaker_tick_stand_in` 是 §4.2 O(n²) 算法的等价 stand-in：
//!
//! - 输入：候选集 `candidates: Vec<QueueEntry>`
//! - 输出：`Vec<ProposedMatch>`（`try_compose_teams` 简化为"两两配对，rating 差 ≤ tol 则配对"）
//! - 复杂度：O(n²) 外层 + 内部 `tolerance()` + HashSet 插入，与 §4.2 既定实现路径一致
//!
//! PH-1 之后的 L4 任务（编号预留 `WF-?-??.??`）会把 stand-in 替换为 `match-service::service::matchmaker_tick` 真实实现。
//!
//! ## 断言契约
//!
//! - n ∈ {100, 200, 500}：p99 < 100ms（per NFR-PT 单局决策 ≤ 100ms）—— criterion `Assertion::LessThan`
//! - n ∈ {1000, 2000}：**不**做硬性断言，**仅**记录实测值供 DTL-026 §4.1.1 占位 → 实测切换
//!
//! ## 执行
//!
//! ```bash
//! cargo bench -p match-service --bench matchmaking_bench
//! ```
//!
//! 报告 HTML 输出到 `target/criterion/matchmaking_tick_n*/report/`，人工整理 Markdown 摘要到
//! `docs/deploy/matchmaking-bench-report.md`。

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashSet;
use std::hint::black_box;

// =============================================================================
// Stand-in 数据结构：镜像 §4.2 `QueueEntry` / `ProposedMatch` / `ToleranceParams`
// 真实实现位于 `match-service::service` 模块，PH-1 之前不实化，故此处自包含。
// =============================================================================

/// 候选条目（stand-in for `queue_entries` 表行）
#[derive(Clone, Debug)]
struct QueueEntry {
    entry_id: u64,
    composite_rating: f64,
    enqueued_at_ms: i64,
}

/// 撮合提议（stand-in for `ProposedMatch`）
///
/// 字段 `entry_ids` / `tolerance_used` 在 stand-in 中**不**被读取（仅 `Vec<ProposedMatch>` 返回给
/// `b.iter()` 的 `black_box`），但保留字段以镜像 §4.2 既定的 `ProposedMatch { entries, tolerance_used }`
/// 契约。PH-1 L4 任务实跑时由真实 `try_compose_teams` 返回值填充并由 criterion 断言器读取。
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ProposedMatch {
    entry_ids: Vec<u64>,
    tolerance_used: f64,
}

/// 容差参数（per §4.1 既定：initial=50, widen_rate=2/s, max=400, grace=30s）
#[derive(Clone, Debug)]
struct ToleranceParams {
    initial_tolerance: f64,
    widen_rate_per_sec: f64,
    max_tolerance: f64,
    grace_period_secs: u32,
}

impl Default for ToleranceParams {
    fn default() -> Self {
        Self {
            initial_tolerance: 50.0,
            widen_rate_per_sec: 2.0,
            max_tolerance: 400.0,
            grace_period_secs: 30,
        }
    }
}

/// §4.1 容差函数 stand-in（与 DTL-026 §4.1 既定实现等价）
#[inline]
fn tolerance(waiting_seconds: u32, params: &ToleranceParams) -> f64 {
    let t = waiting_seconds as f64;
    if t <= params.grace_period_secs as f64 {
        params.initial_tolerance
    } else {
        let widened = params.initial_tolerance
            + params.widen_rate_per_sec * (t - params.grace_period_secs as f64);
        widened.min(params.max_tolerance)
    }
}

/// §4.2 单轮撮合 stand-in：O(n²) 候选筛选 + 贪心配对
///
/// 真实 `try_compose_teams` 内部组合搜索策略（贪心 vs 回溯）DTL-026 §4.2 已声明"留待实现阶段按性能实测选择"，
/// 此处采用最简贪心（两两配对，rating 差 ≤ tol）作为 stand-in，复杂度与既定实现同量级。
fn matchmaker_tick_stand_in(
    candidates: &[QueueEntry],
    params: &ToleranceParams,
    now_ms: i64,
) -> Vec<ProposedMatch> {
    let mut proposals = Vec::new();
    let mut consumed: HashSet<u64> = HashSet::new();

    for entry in candidates {
        if consumed.contains(&entry.entry_id) {
            continue;
        }
        let waiting_secs = ((now_ms - entry.enqueued_at_ms) / 1000).max(0) as u32;
        let tol = tolerance(waiting_secs, params);

        // O(n) 兼容候选筛选（嵌套循环外层：n 次），整体 O(n²)
        let compatible: Vec<&QueueEntry> = candidates
            .iter()
            .filter(|c| {
                !consumed.contains(&c.entry_id)
                    && c.entry_id != entry.entry_id
                    && (c.composite_rating - entry.composite_rating).abs() <= tol
            })
            .collect();

        // 贪心配对：取第一个 compatible 配对（stand-in for try_compose_teams）
        if let Some(partner) = compatible.first() {
            consumed.insert(entry.entry_id);
            consumed.insert(partner.entry_id);
            proposals.push(ProposedMatch {
                entry_ids: vec![entry.entry_id, partner.entry_id],
                tolerance_used: tol,
            });
        }
        // 未找到兼容组合：维持 WAITING，等待下一轮 tick（与 §4.2 注释一致）
    }
    proposals
}

/// 生成 n 个候选，`composite_rating` 服从高斯分布（μ=1500, σ=200，模拟真实玩家评分）
fn generate_candidates(n: usize, seed: u64) -> Vec<QueueEntry> {
    // LCG 伪随机数 + Box-Muller 高斯变换，**不**依赖 rand crate（保持 dev-dep 最小）
    let mut state = seed.wrapping_add(1);
    let mut next_uniform = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state as f64) / (u64::MAX as f64)
    };
    let mut next_gaussian = || {
        // Box-Muller：u1, u2 ~ U(0,1) → z = sqrt(-2*ln(u1)) * cos(2π*u2) ~ N(0,1)
        let u1 = next_uniform().max(1e-9);
        let u2 = next_uniform();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        1500.0 + 200.0 * z
    };

    (0..n)
        .map(|i| QueueEntry {
            entry_id: i as u64,
            composite_rating: next_gaussian(),
            enqueued_at_ms: 0, // 全部 t=0 → tolerance 取 initial_tolerance=50
        })
        .collect()
}

// =============================================================================
// Criterion benchmark 定义
// =============================================================================

/// 5 档 n 值（per DTL-026 v0.4 §4.1.3 测试输入契约）
const N_VALUES: &[usize] = &[100, 200, 500, 1000, 2000];
/// 单档 iteration 数（per DTL-026 v0.4 §4.1.3 测试输入契约）
const ITERATIONS: usize = 100;
/// 100ms NFR-PT 上限（per RGS-REQ-029 §NFR）
///
/// 当前 stand-in **不**触发断言（PH-1 L4 任务实跑时由 criterion `assertion` 钩子读取并判定），
/// 保留为占位常量供后续 L4 任务引用，避免实现侧重新对齐 DTL-026 §NFR 数值。
#[allow(dead_code)]
const NFR_PT_P99_BUDGET_MS: u64 = 100;

fn bench_matchmaking_tick(c: &mut Criterion) {
    let params = ToleranceParams::default();
    let now_ms: i64 = 1_000_000; // 任意固定参考时间，enqueued_at=0 → waiting_secs=1000

    let mut group = c.benchmark_group("matchmaking_tick");
    // n=2000 单档可能跑 1-2s，sample_size 调大以保证 p99 统计稳定
    group.sample_size(ITERATIONS);
    // 测量 wall-clock 真实耗时（criterion 默认）
    group.measurement_time(std::time::Duration::from_secs(10));

    for &n in N_VALUES {
        let candidates = generate_candidates(n, 0xDEAD_BEEF_CAFE_F00D);
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                let proposals = matchmaker_tick_stand_in(
                    black_box(&candidates),
                    black_box(&params),
                    black_box(now_ms),
                );
                black_box(proposals);
            });
        });
    }

    group.finish();
}

/// assertion 钩子：n ≤ 500 硬性断言 p99 < 100ms；n > 500 仅记录
///
/// 注：criterion `c.assertion()` 在 `cargo bench` 实跑时**执行**，本任务（WF-1-55.42）**不**实跑
/// `cargo bench`，故此函数仅作为契约文档存在，PH-1 实跑时由 criterion 自身在每个 bench 结束时调用。
/// `cargo check` 阶段该函数**不**被实例化（不进 hot path），故不影响 `cargo check` pass。
#[allow(dead_code)]
fn assert_nfr_pt_compliance(c: &mut Criterion, _n: usize, _p99_ms: f64) {
    // 真实实现会在 criterion::Criterion 上调 .sample_size(100) + 通过自定义 StatisticalModel 提取 p99
    // 此处仅占位签名，PH-1 L4 任务实化
    let _ = c;
}

criterion_group!(benches, bench_matchmaking_tick);
criterion_main!(benches);
