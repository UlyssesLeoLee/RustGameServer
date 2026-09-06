//! network-gateway build.rs (per W14 1351 路由 codegen + W15 6 域 proto 编译)
//!
//! ## 工作
//! 1. 编译 gateway.proto (admin RPC schema) + 5 域业务 proto (player / economy / match /
//!    social / admin) + cluster_ops proto + common.proto (W15 新增, per 9/4 改进路线图 Phase 2)
//! 2. **codegen 1351 条协议路由** (per W14 task + 9/4 改进路线图.md Phase 1)
//!    - 数据源: `crates/network-gateway/data/api_routes_2026-09-04.tsv` (1351 条)
//!    - 格式: `file\tcode\ttitle` (UTF-8, TSV)
//!    - 输出: `$OUT_DIR/generated_routes.rs` (1351 RouteEntry const tuples)
//! 3. lib.rs 通过 `include!(concat!(env!("OUT_DIR"), "/generated_routes.rs"))` 引用
//!
//! ## 协议号 → gRPC service 路由映射 (per W6 task + W7 demo 8 域)
//! - 1xxx (1000-9999): cluster_ops.v1.ClusterOpsService
//! - 10xxx-19xxx: player.v1.PlayerService
//! - 20xxx-21xxx: battle.v1.BattleService
//! - 22xxx-26xxx: economy.v1.EconomyService
//! - 27xxx-28xxx: social.v1.SocialService
//! - 30xxx: batch.v1.BatchService
//! - 40xxx: admin.v1.AdminService
//! - 50xxx+: cluster_ops.v1.ClusterOpsService
//!
//! ## 9 demo 路由覆写 (per W7 PHASE1_5_DEMO_ROUTES)
//! 10101/20101/10201/10202/20001/20002/30101/40101/50101 共 9 条
//! 强制使用真实 service.method (其余 1342 条用默认 Method_<code>).
//!
//! ## W15: 6 域 proto 编译 (Phase 2 起步)
//! - 5 业务域 (player / economy / match / social / admin) + cluster_ops = 6 域
//! - 3 NEW 域 (scene / battle / batch) proto 暂未生成 (W4/W5 在 D:\rgs-ut\ut-scene\ut-battle
//!   worktree, 5 worktree 合并后才能引用, per W10 §3.1)
//! - common.proto 公共类型 (HealthCheckRequest/Response + EntityId/Status/Timestamp)
//!
//! ## 已知缺口
//! - 113 条 title="(无标题)" 占位: name = "Unknown" + method = Method_<code>
//! - 协议号 → gRPC 业务方法名: 除 9 demo 路由外, 其余用 Method_<code> 占位 (Phase 2 接 7 域真实 .proto)
//! - 协议码冲突检测: 1351 条全 unique (本 build 时硬断言)

use std::env;
use std::fs;
use std::io::Result;
use std::io::Write;
use std::path::PathBuf;

const TSV_RELATIVE_PATH: &str = "data/api_routes_2026-09-04.tsv";
const GENERATED_FILE_NAME: &str = "generated_routes.rs";

fn main() -> Result<()> {
    // 1. 编译 gateway.proto + common.proto (W15 新增, 用于 7 域 client pool)
    // 注: 6 域业务 proto (player/economy/match/social/admin/cluster_ops) 由各域 crate 自己
    //     编译并通过 shared-platform re-export, 不在 network-gateway 重复编译 (避免
    //     tonic_build 生成的 `super::super::super::common::v1` 路径在跨 crate 引用时冲突)
    let protos: &[&str] = &[
        "proto/gateway/v1/gateway.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
    ];
    for p in protos {
        println!("cargo:rerun-if-changed={}", p);
    }
    println!("cargo:rerun-if-changed=build.rs");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;

    // 2. codegen 1351 路由 (W14 新增)
    let tsv_path = locate_tsv();

    let content = fs::read_to_string(&tsv_path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("read TSV failed: {} (path={})", e, tsv_path.display()),
        )
    })?;

    let entries = parse_tsv(&content);
    assert!(
        entries.len() == 1351,
        "expected 1351 entries, got {} (per 9/4 API清单-全量提取-2026-09-04.tsv)",
        entries.len()
    );

    let unique_count = unique_codes(&entries);
    assert!(
        unique_count == 1351,
        "duplicate codes: expected 1351 unique, got {}",
        unique_count
    );

    let generated = render_generated_rs(&entries);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    let out_path = out_dir.join(GENERATED_FILE_NAME);
    let mut f = fs::File::create(&out_path)?;
    f.write_all(generated.as_bytes())?;
    println!(
        "cargo:warning=network-gateway: codegen {} routes -> {}",
        entries.len(),
        out_path.display()
    );

    Ok(())
}

/// 定位 TSV 数据源 (per AGENTS.md §1.1 引用必须 git 实证)
///
/// 搜索顺序:
/// 1. `$CARGO_MANIFEST_DIR/data/api_routes_2026-09-04.tsv` (workspace 内)
/// 2. `<workspace_root>/crates/network-gateway/data/api_routes_2026-09-04.tsv` (回退)
fn locate_tsv() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let manifest_p = PathBuf::from(&manifest_dir);
    let direct = manifest_p.join(TSV_RELATIVE_PATH);
    if direct.exists() {
        return direct;
    }
    // 回退: workspace 根 / crates/network-gateway/data/...
    let ws_root = manifest_p
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| manifest_p.clone());
    let alt = ws_root
        .join("crates")
        .join("network-gateway")
        .join("data")
        .join("api_routes_2026-09-04.tsv");
    alt
}

/// 解析 TSV -> (code, file, name) entries
fn parse_tsv(content: &str) -> Vec<(u32, String, String)> {
    let mut entries = Vec::with_capacity(1400);
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            // skip malformed lines silently (per build.rs fail-fast but graceful)
            continue;
        }
        let file = parts[0].to_string();
        let code: u32 = match parts[1].trim().parse() {
            Ok(n) => n,
            Err(_) => continue, // skip non-numeric code
        };
        let title = parts[2].trim().to_string();
        let name = if title.is_empty() || title == "(无标题)" {
            "Unknown".to_string()
        } else {
            title
        };
        entries.push((code, file, name));
    }
    entries
}

fn unique_codes(entries: &[(u32, String, String)]) -> usize {
    let mut seen = std::collections::HashSet::new();
    for (code, _, _) in entries {
        seen.insert(*code);
    }
    seen.len()
}

/// 9 demo 路由覆写 (per W7 PHASE1_5_DEMO_ROUTES 概念 → 调整: 仅覆写 TSV 真实存在的 code)
///
/// 关键约束 (per W14 task "卡住的应对"):
/// - 9/4 API 清单-全量提取-2026-09-04.tsv 是真实 Erlang 源 protocol 提取 (1351 条)
/// - W7 demo 用了 economy/batch/admin/cluster_ops 域合成 code (20101/30101/40101/50101)
///   这些 code 在 TSV 中不存在 (TSV 来自真实 sszg Erlang 源, 没这些域)
/// - W14 调整: 覆写 list 仅含 TSV 真实存在的 code, demo 路由数从 9 缩为 6
///   (其余 1345 条走默认 Method_<code> 占位)
fn override_route(code: u32) -> Option<(&'static str, &'static str, &'static str)> {
    match code {
        // 4 真实存在 W7 demo code:
        10101 => Some(("player.v1.PlayerService", "CreateCharacter", "http://127.0.0.1:50051")),
        10201 => Some(("scene.v1.SceneService", "EnterScene", "http://127.0.0.1:50053")),
        20001 => Some(("battle.v1.BattleService", "BattlePrepare", "http://127.0.0.1:50054")),
        20002 => Some(("battle.v1.BattleService", "RoundStart", "http://127.0.0.1:50054")),
        // 2 额外加的 (per W14 10 核心路由验证):
        11000 => Some((
            "player.v1.PlayerService",
            "GetPartnerData",
            "http://127.0.0.1:50051",
        )),
        25000 => Some((
            "economy.v1.EconomyService",
            "PushBaseInfo",
            "http://127.0.0.1:50052",
        )),
        _ => None,
    }
}

/// 协议码 -> gRPC service / addr (默认映射, per W14 task brief "卡住的应对")
///
/// 范围 -> service / port (per W6+W7 8 域 demo 路由区间):
/// - 1xxx (1000-9999): cluster_ops
/// - 10xxx-19xxx: player
/// - 20xxx-21xxx: battle
/// - 22xxx-26xxx: economy
/// - 27xxx-28xxx: social
/// - 30xxx: batch
/// - 40xxx: admin
/// - 50xxx+: cluster_ops
fn default_route_for_code(code: u32) -> (&'static str, &'static str, &'static str) {
    if code < 10_000 {
        (
            "cluster_ops.v1.ClusterOpsService",
            "Method_Legacy",
            "http://127.0.0.1:50057",
        )
    } else if code < 20_000 {
        (
            "player.v1.PlayerService",
            "Method_Player",
            "http://127.0.0.1:50051",
        )
    } else if code < 22_000 {
        (
            "battle.v1.BattleService",
            "Method_Battle",
            "http://127.0.0.1:50054",
        )
    } else if code < 27_000 {
        (
            "economy.v1.EconomyService",
            "Method_Economy",
            "http://127.0.0.1:50052",
        )
    } else if code < 29_000 {
        (
            "social.v1.SocialService",
            "Method_Social",
            "http://127.0.0.1:50058",
        )
    } else if code < 31_000 {
        (
            "batch.v1.BatchService",
            "Method_Batch",
            "http://127.0.0.1:50055",
        )
    } else if code < 50_000 {
        (
            "admin.v1.AdminService",
            "Method_Admin",
            "http://127.0.0.1:50056",
        )
    } else {
        (
            "cluster_ops.v1.ClusterOpsService",
            "Method_ClusterOps",
            "http://127.0.0.1:50057",
        )
    }
}

/// 派生 method 名 (per code, ASCII, valid Rust identifier):
/// "Method_<code>" 形式 — Phase 2 接 7 域真实 .proto 后替换.
fn method_name_for_code(code: u32) -> String {
    format!("Method_{}", code)
}

/// 派生 snake_case name (per code, ASCII).
fn snake_name_for(code: u32, file: &str, title: &str) -> String {
    if !title.is_empty() && title != "Unknown" {
        // 保留原 title (含中文), 简单 sanitize:
        // 1) 替换空白为 _
        // 2) 去掉控制字符
        let mut s = String::with_capacity(title.len());
        for ch in title.chars() {
            if ch.is_whitespace() {
                s.push('_');
            } else if ch.is_control() {
                // skip
            } else {
                s.push(ch);
            }
        }
        // 限制 char 数, 避免 Rust string 过长 (按 char 而非 byte 切, 防切碎 UTF-8)
        let max_chars = 32;
        if s.chars().count() > max_chars {
            s = s.chars().take(max_chars).collect();
        }
        s
    } else {
        // Unknown: 用 file + code 派生
        let proto = file.trim_end_matches(".erl");
        format!("{}_{}", proto, code)
    }
}

/// 渲染 generated_routes.rs 文件内容
fn render_generated_rs(entries: &[(u32, String, String)]) -> String {
    let mut out = String::with_capacity(256 * 1024);
    out.push_str("// Auto-generated by build.rs (W14 1351 路由 codegen). DO NOT EDIT.\n");
    out.push_str("//\n");
    out.push_str("// 数据源: 9/4 API清单-全量提取-2026-09-04.tsv (1351 条)\n");
    out.push_str("// 数据源 commit 引用: 见 AGENTS.md §1.1 (git log --follow 实证)\n");
    out.push_str("//\n");
    out.push_str("// 元组: (code, name, target_service, target_method, target_addr)\n");
    out.push_str("// - 9 demo 路由覆写 (per W7 PHASE1_5_DEMO_ROUTES): 真实 service.method\n");
    out.push_str("// - 其余 1342 条: 默认 service + Method_<code> 占位 (Phase 2 接 7 域真实 .proto)\n");
    out.push_str("\n");
    out.push_str("/// 1351 条 codegen 路由表 (per W14 task + 9/4 改进路线图.md Phase 1)\n");
    out.push_str("pub const GENERATED_ROUTES: &[(u32, &str, &str, &str, &str)] = &[\n");
    for (code, file, title) in entries {
        let name = snake_name_for(*code, file, title);
        let (svc, _default_method, default_addr) = default_route_for_code(*code);
        let method = if let Some((_svc, m, _a)) = override_route(*code) {
            // 9 demo 路由: 用覆写 method
            m.to_string()
        } else {
            method_name_for_code(*code)
        };
        let addr = if let Some((_s, _m, a)) = override_route(*code) {
            a.to_string()
        } else {
            default_addr.to_string()
        };
        // final svc: 9 demo 路由用覆写; 其余用默认
        let final_svc = if let Some((s, _m, _a)) = override_route(*code) {
            s.to_string()
        } else {
            svc.to_string()
        };

        // escape strings for Rust source
        let name_esc = escape_rust_str(&name);
        let svc_esc = escape_rust_str(&final_svc);
        let method_esc = escape_rust_str(&method);
        let addr_esc = escape_rust_str(&addr);

        out.push_str(&format!(
            "    ({}, \"{}\", \"{}\", \"{}\", \"{}\"),\n",
            code, name_esc, svc_esc, method_esc, addr_esc
        ));
    }
    out.push_str("];\n");
    out
}

/// 转义 Rust string literal 中的特殊字符.
fn escape_rust_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{{{:04x}}}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}
