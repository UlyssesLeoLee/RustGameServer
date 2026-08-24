//! WF-1-55.44: match-service rgs-testkit 集成测试骨架
//!
//! ## 目的
//! 验证 match-service 已正确接入 rgs-testkit FixtureBuilder (含 `match_game()`
//! sample 工厂 + FixtureBuilder 链式 `with_score` / `with_status`), 且 outbox
//! CHECK 约束幂等 migration 行为正确 (per RGS-OPEN-QA-001 v0.2 Q-M-02 答复).
//!
//! ## 范围 (4 域对称骨架)
//! 1. **`#[rgs_testkit::pg_test]` 真 PG 集成测试** —— 验证 matches 表可 INSERT/SELECT
//!    走 FixtureBuilder 自定义 match sample data (per DTL-016 match lifecycle:
//!    Pending / Active / Completed).
//! 2. **outbox CHECK 约束幂等验证** —— 同 player-service 模板.
//!
//! ## 跑测前置
//! - `DATABASE_URL=postgres://postgres@127.0.0.1:5555/match_db` (或任何可写 PG)
//! - migrations 已跑过 (本 test 假设 migrations 存在, 不自动 migrate)

use rgs_testkit::fixture::{self, FixtureBuilder, MatchFixture};
use rgs_testkit::pg_test;
use sqlx::PgPool;

// ============================================================================
// Test 1: FixtureBuilder 链式 API 自定义 match sample data
// ============================================================================

/// 验证 FixtureBuilder<MatchFixture> 链式覆盖 score / status 字段.
///
/// 强约束 (per WF-1-55.31): 用 rgs_testkit::pg_test 强制走真 PG 路径,
/// 禁止 InMemory 假象.
#[pg_test]
async fn match_fixture_builder_customizes_score_and_status(_pool: PgPool) {
    // 默认 fixture: score=0, status=Pending
    let default_match: MatchFixture = fixture::match_game("player-test-001");
    assert_eq!(default_match.score, 0);
    assert_eq!(default_match.status, "Pending");

    // FixtureBuilder 链式覆盖
    let active: MatchFixture = FixtureBuilder::new(fixture::match_game("player-test-001"))
        .with_score(42)
        .with_status("Active")
        .build();

    assert_eq!(active.player_id, "player-test-001");
    assert_eq!(active.score, 42);
    assert_eq!(active.status, "Active");
    // match_id 是新生成 UUID, 与 default 不同
    assert_ne!(active.match_id, default_match.match_id);
}

// ============================================================================
// Test 2: 真 PG 路径下 match INSERT/SELECT 走 FixtureBuilder
// ============================================================================

/// 验证 FixtureBuilder 产出的 MatchFixture 字段可作为 INSERT 参数写入
/// 真 PG (per ARC-008 match_db 独立), 并可 SELECT 读回.
///
/// matches 表模式 (per DTL-016 §3.1):
///   id, room_id, mode, status, winner_team?, scheduled_at, started_at?, ended_at?
///
/// 注: MatchFixture.match_id 形如 "match-test-<uuid>", 不能直接当 DB UUID 列;
/// INSERT 时单独生成 UUID, 用 fixture status/score 字段.
#[pg_test]
async fn match_fixture_inserts_and_reads_back_in_real_pg(pool: PgPool) {
    let m: MatchFixture = FixtureBuilder::new(fixture::match_game("player-aragorn"))
        .with_status("in_progress")
        .with_score(17)
        .build();

    // 单独生成 UUID (因为 fixture match_id 是 "match-test-<uuid>" 复合字符串,
    // DB id 列只接受 36-char UUID)
    let match_uuid = uuid::Uuid::new_v4();

    // INSERT 走玩家域 matches 表
    sqlx::query(
        "INSERT INTO matches (id, room_id, mode, status, scheduled_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(match_uuid)
    .bind(format!("room-{}", match_uuid)) // room_id 必须 UNIQUE
    .bind("1v1")
    .bind(&m.status)
    .bind(m.started_at)
    .execute(&pool)
    .await
    .expect("INSERT match 走真 PG 必须成功");

    // SELECT 读回
    let (status, mode): (String, String) =
        sqlx::query_as("SELECT status, mode FROM matches WHERE id = $1")
            .bind(match_uuid)
            .fetch_one(&pool)
            .await
            .expect("SELECT match 走真 PG 必须成功");

    assert_eq!(status, "in_progress");
    assert_eq!(mode, "1v1");
}

// ============================================================================
// Test 3: outbox CHECK 约束幂等 + invalid status 拒 insert
// ============================================================================

/// 验证 0003_outbox_check_idempotent.sql 的 `chk_outbox_status` CHECK 约束
/// 真的存在, 且 (1) 拒 invalid status (2) 允 valid status.
///
/// 锚定: WF-1-55.28 outbox 幂等 migration 行为契约, 5 域应统一采用
/// (per RGS-OPEN-QA-001 v0.2 Q-M-02 答复).
#[pg_test]
async fn outbox_check_constraint_rejects_invalid_status(pool: PgPool) {
    // 1. 约束存在性
    let constraint_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.table_constraints
            WHERE table_name = 'outbox'
              AND constraint_name = 'chk_outbox_status'
              AND constraint_type = 'CHECK'
         )",
    )
    .fetch_one(&pool)
    .await
    .expect("query constraint existence 必须成功");

    assert!(
        constraint_exists,
        "chk_outbox_status CHECK 约束必须存在 (0003_outbox_check_idempotent.sql 已跑过)"
    );

    // 2. invalid status 必须被拒
    let insert_invalid = sqlx::query(
        "INSERT INTO outbox (id, subject, payload, command_id, status)
         VALUES (gen_random_uuid(), 'wf_1_55_44', '{}'::jsonb, gen_random_uuid(), 'BOGUS_STATUS')",
    )
    .execute(&pool)
    .await;

    assert!(
        insert_invalid.is_err(),
        "INSERT invalid status 'BOGUS_STATUS' 必须被 CHECK 约束拒, got {:?}",
        insert_invalid
    );
    let err = insert_invalid.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("chk_outbox_status") || err_str.contains("violates check constraint"),
        "expected CHECK violation error, got: {}",
        err_str
    );

    // 3. valid status 必须可 insert
    let insert_valid = sqlx::query(
        "INSERT INTO outbox (id, subject, payload, command_id, status)
         VALUES (gen_random_uuid(), 'wf_1_55_44', '{}'::jsonb, gen_random_uuid(), 'in_flight')",
    )
    .execute(&pool)
    .await;

    assert!(
        insert_valid.is_ok(),
        "INSERT valid status 'in_flight' 必须成功, got {:?}",
        insert_valid
    );
}
