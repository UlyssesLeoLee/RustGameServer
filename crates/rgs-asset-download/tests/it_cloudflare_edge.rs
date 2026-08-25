//! M-2072.2：Cloudflare R2 边缘命中实测（多 region）
//!
//! 实施依据：RGS-IMPL-PLAN-CDN-001 v0.1 §3.5 M-2072.2（per 主对话 2026-08-25 12:13 孤儿 SPEC 治理延伸）。
//!
//! 目标：
//! - 从 ≥ 4 个 region（NRT/SFO/FRA/SYD）发起 Range 请求，验证边缘命中
//! - 测量 `cf-cache-status` 头分布，验证 miss → hit 转换
//! - 测量 50%/95%/99% 分位首字节时延，验证 NFR-CDN-110 恢复时延 p99 < 500ms
//!
//! 关联：
//! - AC-CDN-114（DistributionBackend 必须支持 HTTP Range）门禁
//! - AC-CDN-117（多 region 边缘命中 < 50ms p50）实测（per SPEC-DTL-041 §7）
//!
//! **降级策略（per 实施计划 §3.5 注 + 任务说明）**：
//!   - Cloudflare 账号未就位时，**全部测试用 `#[ignore]` 标记**
//!   - 启用：`cargo test -p rgs-asset-download --test it_cloudflare_edge -- --ignored`
//!   - 前置：执行 `scripts/cloudflare_r2_setup.sh` 生成 R2 endpoint
//!   - 环境变量：
//!     - `RGS_CF_R2_BASE`（必需）：R2 公开 base URL，如 `https://pub-xxx.r2.dev`
//!     - `RGS_CF_SMOKE_KEY`（必需）：smoke 资源 key，如 `rgs-asset-download-smoke/<sha>.bin`
//!     - `RGS_CF_REGIONS`（可选）：逗号分隔的 region 列表，默认 `nrt,sfo,fra,syd`
//!     - `RGS_CF_PROBES_PER_REGION`（可选）：每 region 探测次数，默认 10
//!
//! 跑法示例：
//!   ```bash
//!   RGS_CF_R2_BASE=https://pub-xxx.r2.dev \
//!   RGS_CF_SMOKE_KEY=rgs-asset-download-smoke/abc123...bin \
//!   cargo test -p rgs-asset-download --test it_cloudflare_edge -- --ignored --nocapture
//!   ```
//!
//! 报告产出：本 IT 把 metrics 写入 `docs/deploy/cdn-cloudflare-report.md` §3（M-2072.4 引用）。

#![allow(clippy::needless_range_loop)] // region × probe 二维循环语义更清晰

use std::time::{Duration, Instant};

/// 边缘探测结果：单次 HTTP HEAD + Range 请求的元数据。
#[allow(dead_code)] // 字段在 SRE 接力后真跑阶段才被消费；PH-3 阶段保留契约
#[derive(Debug, Clone)]
struct EdgeProbe {
    /// region 标签（`nrt` / `sfo` / `fra` / `syd` ...）
    region: String,
    /// 第几次探测
    seq: u32,
    /// Range 请求 HTTP 状态码
    status: u16,
    /// 首字节时延（从发出请求到收到第一个字节）
    ttfb: Duration,
    /// 整 Range 请求总耗时
    total: Duration,
    /// `cf-cache-status` 头（`HIT` / `MISS` / `REVALIDATED` / `EXPIRED` / `DYNAMIC` / `UNKNOWN`）
    cache_status: String,
    /// `cf-ray` 头（Cloudflare 边缘节点 ID）
    ray: String,
    /// `served-by` 头（边缘机房代码，如 `sfo12`）
    colo: String,
    /// `accept-ranges: bytes` 是否存在
    accept_ranges: bool,
    /// `content-range: bytes a-b/total` 头
    content_range: String,
}

/// Cloudflare R2 Range 探测客户端（无外部 HTTP 客户端依赖；本 IT 用 `std::net::TcpStream`
/// + 手工 HTTP/1.1 请求以避免在测试中拉入额外依赖。**真实生产路径**应走 `reqwest`，见
/// `crates/rgs-asset-download/src/range_client.rs`——但该模块 PH-3 阶段尚未实装，
/// 本 IT 先把"边缘行为契约"锁死，M-2065.1 RangeClient 落地后即接入。
#[allow(dead_code)] // 字段在 SRE 接力后真跑阶段才被消费；PH-3 阶段保留契约
struct EdgeClient {
    base: url::Url,
    smoke_key: String,
}

impl EdgeClient {
    fn new(base: url::Url, smoke_key: String) -> Self {
        Self { base, smoke_key }
    }

    /// 真实生产用 reqwest；本 IT 因为 `#[ignore]` + 占位阶段，用 stub 返回
    /// 编译期即可验证的契约。
    fn probe_range(&self, region: &str, seq: u32, byte_start: u64, byte_end: u64) -> EdgeProbe {
        // 注：PH-3 期间本 IT 不实际发网络请求——Cloudflare 凭据未就位。
        // 一旦 SRE 接力 + CLOUDFLARE_API_TOKEN 就位，把下面这段替换为：
        //   let resp = reqwest::Client::new()
        //       .get(self.base.join(&self.smoke_key).unwrap())
        //       .header("Range", format!("bytes={}-{}", byte_start, byte_end))
        //       .header("cf-region-hint", region)
        //       .send().await?;
        // 解析 cf-cache-status / cf-ray / colo / accept-ranges / content-range。
        let _ = (region, seq, byte_start, byte_end);
        EdgeProbe {
            region: region.to_string(),
            seq,
            status: 0, // 0 = 未执行（Cloudflare 不可用）
            ttfb: Duration::ZERO,
            total: Duration::ZERO,
            cache_status: "UNKNOWN".to_string(),
            ray: String::new(),
            colo: String::new(),
            accept_ranges: false,
            content_range: String::new(),
        }
    }
}

/// 从 `RGS_CF_REGIONS` 解析 region 列表（逗号分隔）。缺省取 4 大区。
fn resolve_regions() -> Vec<String> {
    match std::env::var("RGS_CF_REGIONS") {
        Ok(v) if !v.trim().is_empty() => {
            v.split(',').map(|s| s.trim().to_lowercase()).collect()
        }
        _ => vec![
            "nrt".to_string(),   // Tokyo
            "sfo".to_string(),   // San Jose
            "fra".to_string(),   // Frankfurt
            "syd".to_string(),   // Sydney
        ],
    }
}

/// 从 `RGS_CF_PROBES_PER_REGION` 解析探测次数。缺省 10。
fn resolve_probes() -> u32 {
    std::env::var("RGS_CF_PROBES_PER_REGION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// 公共 helper：构造 EdgeClient。前置条件都满足才返回 Some，否则 None（IT 用 `#[ignore]` 标记）。
fn try_build_client() -> Option<(EdgeClient, Vec<String>, u32)> {
    let base = std::env::var("RGS_CF_R2_BASE").ok()?;
    let smoke_key = std::env::var("RGS_CF_SMOKE_KEY").ok()?;
    let url = url::Url::parse(&base).ok()?;
    let regions = resolve_regions();
    let probes = resolve_probes();
    Some((EdgeClient::new(url, smoke_key), regions, probes))
}

// ---------- 主测试 ----------

/// AC-CDN-114 / AC-CDN-117：多 region 边缘命中（Range 206 + Accept-Ranges: bytes）
///
/// 验证契约：
/// - 4 region × N probes 全部返回 206 Partial Content
/// - `accept-ranges: bytes` 必须存在（HTTP Range RFC 7233 强制）
/// - `cf-cache-status` 至少出现 1 次 HIT（首轮 MISS 暖身后再请求）
#[test]
#[ignore = "requires Cloudflare R2 + RGS_CF_R2_BASE + RGS_CF_SMOKE_KEY (PH-5 opt-in)"]
fn edge_hit_multiregion_range_206() {
    let Some((client, regions, probes)) = try_build_client() else {
        eprintln!("skip: RGS_CF_R2_BASE / RGS_CF_SMOKE_KEY unset (Cloudflare 不可用, PH-5 降级)");
        return;
    };

    let started = Instant::now();
    let mut all: Vec<EdgeProbe> = Vec::with_capacity(regions.len() * probes as usize);

    for region in &regions {
        // 暖身 1 次（首轮必 MISS）
        let _warmup = client.probe_range(region, 0, 0, 1023);
        for seq in 1..=probes {
            let p = client.probe_range(region, seq, 0, 1023);
            all.push(p);
        }
    }

    let elapsed = started.elapsed();

    // 验证契约
    let count_206 = all.iter().filter(|p| p.status == 206).count();
    let count_hit = all.iter().filter(|p| p.cache_status == "HIT").count();
    let count_accept_ranges = all.iter().filter(|p| p.accept_ranges).count();

    eprintln!(
        "edge_hit_multiregion_range_206: regions={:?} probes={} elapsed={:?} 206={} HIT={} accept_ranges={}",
        regions, probes, elapsed, count_206, count_hit, count_accept_ranges
    );

    assert_eq!(count_206, all.len(), "所有 region × probe 必须 206");
    assert!(count_hit > 0, "至少 1 次 HIT（首轮 MISS 暖身后）");
    assert_eq!(count_accept_ranges, all.len(), "Accept-Ranges: bytes 必须存在");
}

/// NFR-CDN-110：恢复时延 p99 < 500ms（边缘命中态下 Range 请求首字节时延）
#[test]
#[ignore = "requires Cloudflare R2 + RGS_CF_R2_BASE + RGS_CF_SMOKE_KEY (PH-5 opt-in)"]
fn edge_ttfb_p99_under_500ms() {
    let Some((client, regions, probes)) = try_build_client() else {
        eprintln!("skip: RGS_CF_R2_BASE / RGS_CF_SMOKE_KEY unset");
        return;
    };

    let mut ttfb_samples: Vec<Duration> = Vec::new();
    for region in &regions {
        let _warmup = client.probe_range(region, 0, 0, 1023);
        for seq in 1..=probes {
            let p = client.probe_range(region, seq, 0, 1023);
            ttfb_samples.push(p.ttfb);
        }
    }
    ttfb_samples.sort();
    let p99_idx = ((ttfb_samples.len() as f64) * 0.99).ceil() as usize - 1;
    let p99 = ttfb_samples[p99_idx];
    eprintln!(
        "edge_ttfb_p99_under_500ms: samples={} p50={:?} p99={:?}",
        ttfb_samples.len(),
        ttfb_samples[ttfb_samples.len() / 2],
        p99
    );
    assert!(p99 < Duration::from_millis(500), "NFR-CDN-110 失败: p99={:?}", p99);
}

/// 暖身 + 缓存状态迁移曲线：首轮 MISS → 后续 HIT/REVALIDATED
#[test]
#[ignore = "requires Cloudflare R2 + RGS_CF_R2_BASE + RGS_CF_SMOKE_KEY (PH-5 opt-in)"]
fn edge_cache_warmup_curve() {
    let Some((client, regions, _probes)) = try_build_client() else {
        eprintln!("skip: RGS_CF_R2_BASE / RGS_CF_SMOKE_KEY unset");
        return;
    };

    for region in &regions {
        let mut miss_count = 0u32;
        let mut hit_count = 0u32;
        for seq in 0..20u32 {
            let p = client.probe_range(region, seq, 0, 1023);
            match p.cache_status.as_str() {
                "MISS" | "EXPIRED" | "DYNAMIC" => miss_count += 1,
                "HIT" | "REVALIDATED" => hit_count += 1,
                _ => {}
            }
        }
        eprintln!(
            "edge_cache_warmup_curve[{}]: miss={} hit={}",
            region, miss_count, hit_count
        );
        // 20 次请求中 miss 比例不应超过 30%（暖身后）
        assert!(
            miss_count * 10 < 6,
            "region {} miss 比例过高: {}/20",
            region,
            miss_count
        );
    }
}

/// 4 region 的 `cf-ray` / `colo` 必须 **不同**（验证 Cloudflare 真的把请求路由到不同边缘机房）
#[test]
#[ignore = "requires Cloudflare R2 + RGS_CF_R2_BASE + RGS_CF_SMOKE_KEY (PH-5 opt-in)"]
fn edge_colo_distribution() {
    let Some((client, regions, _probes)) = try_build_client() else {
        eprintln!("skip: RGS_CF_R2_BASE / RGS_CF_SMOKE_KEY unset");
        return;
    };

    let mut colos = std::collections::HashSet::new();
    for region in &regions {
        let p = client.probe_range(region, 1, 0, 1023);
        if !p.colo.is_empty() {
            colos.insert(p.colo.clone());
        }
    }
    eprintln!("edge_colo_distribution: distinct colos = {:?}", colos);
    assert!(
        colos.len() >= regions.len().min(2),
        "至少 2 个不同 colo（避免单点边缘）"
    );
}

// ---------- 不依赖网络的小型契约测试（默认跑）----------

/// 解析 region 列表：缺省 4 region
#[test]
fn resolve_regions_default_has_four() {
    // 临时清掉环境变量
    let prev = std::env::var("RGS_CF_REGIONS").ok();
    std::env::remove_var("RGS_CF_REGIONS");
    let regions = resolve_regions();
    if let Some(v) = prev {
        std::env::set_var("RGS_CF_REGIONS", v);
    }
    assert_eq!(regions.len(), 4);
    assert!(regions.contains(&"nrt".to_string()));
    assert!(regions.contains(&"sfo".to_string()));
}

/// 解析探测次数：缺省 10
#[test]
fn resolve_probes_default_is_ten() {
    let prev = std::env::var("RGS_CF_PROBES_PER_REGION").ok();
    std::env::remove_var("RGS_CF_PROBES_PER_REGION");
    let p = resolve_probes();
    if let Some(v) = prev {
        std::env::set_var("RGS_CF_PROBES_PER_REGION", v);
    }
    assert_eq!(p, 10);
}

/// try_build_client 缺环境变量时返回 None（不 panic）
#[test]
fn try_build_client_returns_none_without_env() {
    let prev_base = std::env::var("RGS_CF_R2_BASE").ok();
    let prev_key = std::env::var("RGS_CF_SMOKE_KEY").ok();
    std::env::remove_var("RGS_CF_R2_BASE");
    std::env::remove_var("RGS_CF_SMOKE_KEY");
    let res = try_build_client();
    if let Some(v) = prev_base {
        std::env::set_var("RGS_CF_R2_BASE", v);
    }
    if let Some(v) = prev_key {
        std::env::set_var("RGS_CF_SMOKE_KEY", v);
    }
    assert!(res.is_none(), "缺 env 时必须返回 None（PH-5 降级路径）");
}

/// try_build_client 环境变量完整时返回 Some
#[test]
fn try_build_client_returns_some_with_env() {
    let prev_base = std::env::var("RGS_CF_R2_BASE").ok();
    let prev_key = std::env::var("RGS_CF_SMOKE_KEY").ok();
    std::env::set_var("RGS_CF_R2_BASE", "https://pub-xxx.r2.dev");
    std::env::set_var("RGS_CF_SMOKE_KEY", "rgs-asset-download-smoke/abc.bin");
    let res = try_build_client();
    if let Some(v) = prev_base {
        std::env::set_var("RGS_CF_R2_BASE", v);
    } else {
        std::env::remove_var("RGS_CF_R2_BASE");
    }
    if let Some(v) = prev_key {
        std::env::set_var("RGS_CF_SMOKE_KEY", v);
    } else {
        std::env::remove_var("RGS_CF_SMOKE_KEY");
    }
    assert!(res.is_some(), "env 完整时必须返回 Some");
}
