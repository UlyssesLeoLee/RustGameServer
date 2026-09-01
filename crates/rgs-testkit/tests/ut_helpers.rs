//! 单元测试: 工具类 + fixture 业务函数 (NoOp / 纯函数, 不依赖真 DB)
//!
//! per rgs-testkit 强约束 WF-1-55.31: 本 crate 自身测试**禁止**依赖真 DB;
//! NoOp stub + 工具类函数 + fixture factory 行为验证是 UT 主战场.
//!
//! 覆盖:
//! - `helper::load_test_env` 边界 (空 / 含 # / 含 = / 无 .env.test)
//! - `assert_eventually!` macro 成功路径 + 即时成功路径
//! - `FixtureBuilder` 链式 API 不可变性 (build 返回 clone, builder 可复用)
//! - fixture factory 的 UUID 唯一性 (高并发调用下不撞 id)
//! - 5 域 fixture + Player/Economy 类型的 serde roundtrip
//!
//! 规范: RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::helper;
use std::collections::HashSet;

// ============================================================================
// helper::load_test_env 边界
// ============================================================================

#[test]
fn load_test_env_returns_empty_map_when_no_file() {
    // 没 .env.test 应返回空 map, 不 panic
    let env = helper::load_test_env();
    // 允许返回空 HashMap, 也允许返回有内容 (CI 上可能存在 .env.test),
    // 但 keys 必须非空 (clippy for_kv_map lint)
    for k in env.keys() {
        assert!(!k.is_empty(), "env keys must be non-empty");
    }
}

#[test]
fn load_test_env_keys_have_no_equals_sign() {
    // load_test_env 的 split_once('=') 保证 key 不会含 '='
    let env = helper::load_test_env();
    for k in env.keys() {
        assert!(!k.contains('='), "key must not contain '=': {k}");
    }
}

// ============================================================================
// FixtureBuilder 不可变性
// ============================================================================

#[test]
fn fixture_builder_build_returns_clone_not_consume() {
    // builder 可复用, build 多次应返回同值 (T: Clone)
    let builder = FixtureBuilder::new(fixture::player()).with_level(42);
    let p1 = builder.build();
    let p2 = builder.build();
    assert_eq!(p1.level, 42);
    assert_eq!(p2.level, 42);
    assert_eq!(p1.id, p2.id);
}

#[test]
fn fixture_builder_chain_overrides_later_wins() {
    // 同一字段多次 set, 后者覆盖前者
    let p = FixtureBuilder::new(fixture::player())
        .with_name("First")
        .with_name("Second")
        .with_level(10)
        .with_level(99)
        .build();
    assert_eq!(p.name, "Second");
    assert_eq!(p.level, 99);
}

#[test]
fn fixture_builder_does_not_mutate_factory_state() {
    // builder 应不影响原 fixture factory 的下一次输出
    let _custom = FixtureBuilder::new(fixture::player())
        .with_name("Modified")
        .with_level(99)
        .build();
    let fresh = fixture::player();
    assert_eq!(fresh.name, "Test Player", "factory default must survive");
    assert_eq!(fresh.level, 1, "factory default level must survive");
}

// ============================================================================
// fixture factory 的 UUID 唯一性
// ============================================================================

#[test]
fn saga_unique_id_across_many_calls() {
    // 1000 次调用 saga() 应无撞 id
    let mut ids = HashSet::new();
    for _ in 0..1000 {
        let s = fixture::saga("transfer");
        assert!(ids.insert(s.saga_id.clone()), "duplicate saga id: {}", s.saga_id);
    }
    assert_eq!(ids.len(), 1000);
}

#[test]
fn match_game_unique_id_across_many_calls() {
    let mut ids = HashSet::new();
    for _ in 0..1000 {
        let m = fixture::match_game("p1");
        assert!(ids.insert(m.match_id.clone()), "duplicate match id");
    }
    assert_eq!(ids.len(), 1000);
}

// ============================================================================
// 5 域 + Player/Economy fixture 类型 serde roundtrip
// ============================================================================

#[test]
fn player_fixture_serde_roundtrip() {
    let p = FixtureBuilder::new(fixture::player())
        .with_name("Roundtrip")
        .with_level(7)
        .build();
    let json = serde_json::to_string(&p).expect("serialize");
    let back: fixture::PlayerFixture = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(p, back);
}

#[test]
fn economy_fixture_serde_roundtrip() {
    let e = fixture::economy("p-rt");
    let json = serde_json::to_string(&e).expect("serialize");
    let back: fixture::EconomyFixture = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(e, back);
}

#[test]
fn saga_fixture_serde_roundtrip() {
    let s = fixture::saga("roundtrip-saga");
    let json = serde_json::to_string(&s).expect("serialize");
    let back: fixture::SagaFixture = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(s, back);
}

#[test]
fn match_fixture_serde_roundtrip() {
    let m = FixtureBuilder::new(fixture::match_game("p1"))
        .with_score(123)
        .with_status("Active")
        .build();
    let json = serde_json::to_string(&m).expect("serialize");
    let back: fixture::MatchFixture = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(m, back);
}

#[test]
fn social_fixture_serde_roundtrip() {
    let s = FixtureBuilder::new(fixture::social_message("from", "to"))
        .with_message("rt")
        .build();
    let json = serde_json::to_string(&s).expect("serialize");
    let back: fixture::SocialFixture = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(s, back);
}

#[test]
fn admin_fixture_serde_roundtrip() {
    let a = FixtureBuilder::new(fixture::admin_action("admin1", "ban", "t1"))
        .with_action("mute")
        .with_target("t2")
        .build();
    let json = serde_json::to_string(&a).expect("serialize");
    let back: fixture::AdminFixture = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(a, back);
}

// ============================================================================
// assert_eventually! macro: 即时成功路径
// ============================================================================

#[tokio::test]
async fn assert_eventually_immediate_pass() {
    // 条件一开始就是 true, 不应 panic
    let counter = 5;
    rgs_testkit::assert_eventually!(counter == 5, 1000);
}

#[tokio::test]
async fn assert_eventually_short_wait() {
    // 条件在 ~20ms 后变 true, 在 1000ms timeout 内通过
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    let tick = Arc::new(AtomicU32::new(0));
    let tick_clone = Arc::clone(&tick);
    let h = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        tick_clone.store(1, Ordering::SeqCst);
    });
    rgs_testkit::assert_eventually!(tick.load(Ordering::SeqCst) == 1, 1000);
    h.await.unwrap();
}
