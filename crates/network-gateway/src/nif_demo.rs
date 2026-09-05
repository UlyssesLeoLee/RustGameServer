//! NIF demo (per 9/5 W13 task + ADR-007 rustler 0.36 选型 PoC)
//!
//! ## 范围 (W13 PoC, 30 min)
//! 4 NIF 演示: add / echo / bridge_route / version, 给 RGS 网络网关 ↔ Erlang/OTP 26
//! 双向调用选型验证.
//!
//! ## 关键设计 (per ADR-007 §Decision + W13 report §4.2)
//! - 业务 (`add` / `echo` / `bridge_route` / `version`) 是**纯 Rust**, 不依赖 BEAM,
//!   单测不需要 Erlang, `cargo test --lib` 无 `--features nif` 也能跑 (与 5 域编译模式
//!   保持一致, 不破坏 k3s image 构建).
//! - NIF 注解 (`#[rustler::nif]` + `rustler::init!`) 走 `#[cfg(feature = "nif")]`,
//!   默认 `nif` feature 关闭, 默认构建不引入 rustler 依赖, **0 BEAM 编译开销**.
//! - 启用命令: `cargo build -p network-gateway --features nif` (Phase 1.5 推进, 需 Erlang 26).
//!
//! ## 不动
//! - 7 域 gRPC client pool (`client_pool.rs` W15) 不动
//! - nif.rs W7 GrpcTarget 枚举 + bridge() stub 不动
//! - 5 域 (player / economy / match / social / admin) 全部不动
//! - batch 域不动
//! - 7 域独立 Lead 原则 (per 8/21 JST 5 域拒绝兼任 → 扩 7 域, 本任务不兼任)
//!
//! ## Erlang 端契约 (per crates/network-gateway/erlang_test/test_add.erl)
//! - `rgs_nif:add(A, B) -> integer()`  (双向调用验证)
//! - `rgs_nif:echo(V) -> {V, <<"echoed">>}`  (binary 形式, 与 NIF `&str` 编码一致)
//! - `rgs_nif:bridge_route(Code) -> {Service, Method, Rcode}`  (7 域路由表)
//! - `rgs_nif:version() -> {"rgs_nif", "0.1.0-w13", "nif_version_2_15"}`  (NIF 元数据)
//!
//! ## 已知缺口 (per 缺标比错标 §1.1 AGENTS.md, 显式列)
//! - Phase 1.5 装 Erlang 26 (per ADR-007 §Implementation Plan) 后才能跑
//!   `cargo build -p network-gateway --features nif` 端到端
//! - Erlang `bridge_route` 测试用 list 语法 (`"player.v1.PlayerService"`),
//!   NIF `&str` 编码是 binary, 需 Phase 1.5 改 test_add.erl 用 `<<"...">>` 形式
//! - BEAM 启动 + 关停 hook (on_load extension) Phase 1.5 走
//! - 真实调 7 域 gRPC client (bridge_route 内部走 tonic::transport::Channel) Phase 1.5 走
//!
//! ## 派生约束
//! - L3 (跨工具链决策前先 grep workspace 依赖) — rustler 0.36 在 crates.io 验证存在
//! - L12.1 (临时 log / .txt / .tmp_search* 不入 commit) — N/A, 无临时 log
//! - 8/27 11:06 JST 凭据硬 ban — 全文 0 凭据打印, ERLANG_COOKIE 走 k3s Secret
// ====================================================================

// ====================================================================
// 1. 业务层 (纯 Rust, 永远可用, 不依赖 BEAM / rustler)
// ====================================================================

/// 纯 Rust 加法 (i64 wrapping, 与 Erlang `+` 语义一致, 溢出 wrap 不 panic).
///
/// Erlang 端契约: `rgs_nif:add(A, B) -> integer()` (per test_add.erl test_add_2_3 等).
pub fn add(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

/// 纯 Rust echo: 返回 `(value, "echoed")` 元组.
///
/// Erlang 端契约: `rgs_nif:echo(V) -> {V, <<"echoed">>}` (binary 形式).
/// NIF 编码: `&'static str` 走 rustler Binary 编码 → `<<"echoed">>`, 匹配.
pub fn echo(value: i64) -> (i64, &'static str) {
    (value, "echoed")
}

/// 7 域路由表 demo (per 9/4 改进路线图 Phase 1 + ADR-007 §PoC 验证).
///
/// 输入: 协议码 (u32, 与 W14 build.rs 1351 codegen 一致)
/// 输出: (target_service, target_method, rcode)
///
/// Erlang 端契约: `rgs_nif:bridge_route(Code) -> {Service, Method, Rcode}`.
///
/// 已知缺口: 当前仅 2 demo 路由 (10101 player / 20001 battle) 显式覆写,
/// 其余 1349 条走默认占位 (per 9/4 W14 codegen, Phase 2 接 7 域真实 .proto 后替换).
pub fn bridge_route(code: u32) -> (&'static str, &'static str, u32) {
    match code {
        // 10101 = 创建角色 (per W7 PHASE1_5_DEMO_ROUTES + test_add.erl)
        10101 => ("player.v1.PlayerService", "CreateOrUpdate", 0),
        // 20001 = 战斗行动 (per test_add.erl)
        20001 => ("battle.v1.BattleService", "BattleAction", 0),
        // 10201 = 进入场景 (per W14 codegen 6 demo)
        10201 => ("scene.v1.SceneService", "EnterScene", 0),
        // 20002 = 战斗回合开始 (per W14 codegen)
        20002 => ("battle.v1.BattleService", "RoundStart", 0),
        // 11000 = 玩家伙伴数据 (per W14 codegen)
        11000 => ("player.v1.PlayerService", "GetPartnerData", 0),
        // 25000 = 推送基础信息 (per W14 codegen)
        25000 => ("economy.v1.EconomyService", "PushBaseInfo", 0),
        // 默认占位 (per W14 Method_<code> 占位模式)
        _ => ("unknown.v1.UnknownService", "Method_Unknown", 0),
    }
}

/// NIF 元数据 (per ADR-007 §PoC 验证).
///
/// Erlang 端契约: `rgs_nif:version() -> {"rgs_nif", "0.1.0-w13", "nif_version_2_15"}`.
/// 用途: 验证 NIF 双向调用 + rustler::init! 成功.
pub fn version() -> (&'static str, &'static str, &'static str) {
    ("rgs_nif", "0.1.0-w13", "nif_version_2_15")
}

// ====================================================================
// 2. NIF 绑定 (feature = "nif" 启用, 默认关闭)
// ====================================================================

#[cfg(feature = "nif")]
mod nif_bindings {
    use rustler::{Env, NifResult, Term};

    /// NIF 入口: `rgs_nif:add(A, B) -> integer()`
    #[rustler::nif]
    pub fn add(a: i64, b: i64) -> i64 {
        super::add(a, b)
    }

    /// NIF 入口: `rgs_nif:echo(V) -> {V, <<"echoed">>}`
    #[rustler::nif]
    pub fn echo(value: i64) -> (i64, &'static str) {
        super::echo(value)
    }

    /// NIF 入口: `rgs_nif:bridge_route(Code) -> {Service, Method, Rcode}`
    #[rustler::nif]
    pub fn bridge_route(code: u32) -> (&'static str, &'static str, u32) {
        super::bridge_route(code)
    }

    /// NIF 入口: `rgs_nif:version() -> {"rgs_nif", "0.1.0-w13", "nif_version_2_15"}`
    #[rustler::nif]
    pub fn version<'a>(env: Env<'a>, _info: Term<'a>) -> NifResult<Term<'a>> {
        let (name, ver, nif_ver) = super::version();
        Ok((name, ver, nif_ver).encode(env))
    }

    /// NIF 加载 hook (rustler::init! 调, Phase 1.5 扩展 BEAM 启动初始化)
    fn on_load<'a>(_env: Env<'a>, _info: Term<'a>) -> bool {
        // 当前 W13 PoC: 不做额外初始化, 加载即返回 true.
        // Phase 1.5 扩展: 注册 EPMD 节点 / 加载 7 域 gRPC cert / 初始化 metrics
        true
    }

    // 编码辅助: tuple → Term (rustler 0.36 API)
    use rustler::Encoder;

    rustler::init!(
        b"rgs_nif\0",
        [add, echo, bridge_route, version],
        load = on_load
    );
}

// ====================================================================
// 3. 单元测试 (13 tests, 纯 Rust, 不依赖 BEAM / rustler)
// ====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- add 业务 (4 tests, 与 test_add.erl 一致 + 边界) -----

    #[test]
    fn add_2_3_equals_5() {
        assert_eq!(add(2, 3), 5, "add(2, 3) = 5 (per test_add.erl test_add_2_3)");
    }

    #[test]
    fn add_100_200_equals_300() {
        assert_eq!(add(100, 200), 300, "add(100, 200) = 300 (per test_add.erl test_add_100_200)");
    }

    #[test]
    fn add_negative_5_plus_5_equals_0() {
        assert_eq!(add(-5, 5), 0, "add(-5, 5) = 0 (per test_add.erl test_add_negative)");
    }

    #[test]
    fn add_zero_is_identity() {
        assert_eq!(add(0, 0), 0, "add(0, 0) = 0");
        assert_eq!(add(42, 0), 42, "add(42, 0) = 42 (zero identity)");
        assert_eq!(add(0, -7), -7, "add(0, -7) = -7");
    }

    // ----- echo 业务 (2 tests) -----

    #[test]
    fn echo_returns_value_and_echoed_str() {
        let (v, s) = echo(42);
        assert_eq!(v, 42);
        assert_eq!(s, "echoed", "per test_add.erl test_echo <<\"echoed\">>");
    }

    #[test]
    fn echo_handles_zero_and_negative() {
        assert_eq!(echo(0), (0, "echoed"));
        assert_eq!(echo(-1), (-1, "echoed"));
    }

    // ----- bridge_route 业务 (5 tests) -----

    #[test]
    fn bridge_route_player_create_or_update() {
        let (svc, method, rcode) = bridge_route(10101);
        assert_eq!(svc, "player.v1.PlayerService");
        assert_eq!(method, "CreateOrUpdate");
        assert_eq!(rcode, 0, "rcode=0 表示路由 OK (per W14 demo 路由覆写语义)");
    }

    #[test]
    fn bridge_route_battle_battle_action() {
        let (svc, method, rcode) = bridge_route(20001);
        assert_eq!(svc, "battle.v1.BattleService");
        assert_eq!(method, "BattleAction");
        assert_eq!(rcode, 0);
    }

    #[test]
    fn bridge_route_w14_six_demos() {
        // 6 demo 路由 (per W14 build.rs override_route 6 条)
        assert_eq!(bridge_route(10101).0, "player.v1.PlayerService");
        assert_eq!(bridge_route(10201).0, "scene.v1.SceneService");
        assert_eq!(bridge_route(20001).0, "battle.v1.BattleService");
        assert_eq!(bridge_route(20002).0, "battle.v1.BattleService");
        assert_eq!(bridge_route(11000).0, "player.v1.PlayerService");
        assert_eq!(bridge_route(25000).0, "economy.v1.EconomyService");
    }

    #[test]
    fn bridge_route_unknown_code_returns_unknown() {
        // 不在 override 列表 → 默认占位
        let (svc, method, rcode) = bridge_route(99999);
        assert_eq!(svc, "unknown.v1.UnknownService");
        assert!(method.starts_with("Method_"));
        assert_eq!(rcode, 0);
    }

    #[test]
    fn bridge_route_rcode_always_zero() {
        // 当前 stub 行为: rcode=0 表示路由决策 OK, 真实 NIF Phase 1.5 接 gRPC 返真实 rcode
        for code in [10101, 10201, 11000, 20001, 20002, 25000, 99999] {
            let (_, _, rcode) = bridge_route(code);
            assert_eq!(rcode, 0, "所有 code 应返 rcode=0 stub 行为, code={}", code);
        }
    }

    // ----- version 业务 (2 tests) -----

    #[test]
    fn version_returns_three_strs() {
        let (name, ver, nif_ver) = version();
        assert_eq!(name, "rgs_nif");
        assert_eq!(ver, "0.1.0-w13");
        assert_eq!(nif_ver, "nif_version_2_15");
    }

    #[test]
    fn version_format_consistent() {
        // 验证 NIF 版本号格式 (NIF 2.15 = Erlang 26, per ADR-007 §Decision Option A)
        let (_, _, nif_ver) = version();
        assert!(nif_ver.starts_with("nif_version_2_"), "NIF version 应 nif_version_2_<minor>");
    }
}
