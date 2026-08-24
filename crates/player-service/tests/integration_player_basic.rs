//! WF-1-55.44: player-service rgs-testkit 集成测试骨架
//!
//! ## 目的
//! 验证 player-service 已正确接入 rgs-testkit FixtureBuilder, 且 outbox CHECK 约束
//! 幂等 migration 行为正确 (per RGS-OPEN-QA-001 v0.2 Q-M-02 答复「rgs-testkit FixtureBuilder
//! 已支持, 5 域应统一采用」).
//!
//! ## 范围 (4 域对称骨架, 本文件为 player-service 模板)
//! 1. **`#[rgs_testkit::pg_test]` 真 PG 集成测试** —— 验证 players 表可 INSERT/SELECT
//!    走 FixtureBuilder 自定义 player sample data, 强约束 per WF-1-55.31 PgTestDatabase
//!    fixture 防 InMemory 假象.
//! 2. **outbox CHECK 约束幂等验证** —— 验证 0003_outbox_check_idempotent.sql 的
//!    `chk_outbox_status` CHECK 约束真的存在, 且对 invalid status 拒 insert.
//!
//! ## 与 economy-service integration_outbox.rs 的区别
//! - economy-service 模板用 `#[tokio::test]` + 手动 isolated DB URL
//! - 本文件用 **`#[rgs_testkit::pg_test]`** (per WF-1-55.31 retry 强约束),
//!   sqlx::test 宏自动 create-per-test-DB 隔离 + 事务回滚
//! - 额外覆盖 FixtureBuilder 链式 API, 验证 sample data 可定制
//!
//! ## 跑测前置
//! - `DATABASE_URL=postgres://postgres@127.0.0.1:5555/player_db` (或任何可写 PG)
//! - migrations 已跑过 (本 test 假设 migrations 存在, 不自动 migrate)
//!
//! ## 跳过机制
//! - 无 DATABASE_URL: sqlx::test 宏会编译失败, 故本文件假定 CI 注入 env
//! - 不可达 PG: sqlx::test 宏会 panic, 视作 CI 失败 (per RGS-REV-009 V3 H-1 共识:
//!   "假象通过比 fail 更危险")

use rgs_testkit::fixture::{self, FixtureBuilder, PlayerFixture};
use rgs_testkit::pg_test;
use sqlx::PgPool;

// 注意: 本文件用 `#[sqlx::test(migrations = "tests/migrations")]` 而非默认
// `./migrations`. 原因: 0004_player_characters_inventory.sql 包含跨表前向 FK
// (player_characters.fk_pc_weapon REFERENCES player_inventory), 而
// player_inventory 也在同一文件靠后位置定义; sqlx 0.8 的 `migrate!` 宏按
// statement 顺序执行, 不会等整文件 COMMIT 再校验 FK, 导致 fresh DB 跑该
// migration 失败. 我们对 `player_inventory/player_characters` 表的 CRITICAL
// 集成测试不在本 WF-1-55.44 范围 (per Q-D-02 答复 #1 由 WF-1-55.39 接管),
// 所以本骨架只用 0001/0002/0003 即可, 不需要 0004.

// ============================================================================
// Test 1: FixtureBuilder 链式 API 自定义 player sample data
// ============================================================================

/// 验证 FixtureBuilder 可链式覆盖 name / level 字段, 产出 PlayerFixture
/// 完全等价于 fixture::player() 的基础结构 + 自定义值.
///
/// 强约束 (per WF-1-55.31 / RGS-OPEN-QA-001 Q-M-02): 用 rgs_testkit::pg_test
/// 强制走真 PG 路径, 禁止 InMemory 假象.
#[pg_test(migrations = "tests/migrations")]
async fn player_fixture_builder_customizes_name_and_level(_pool: PgPool) {
    // 默认 fixture: name=Test Player, level=1
    let default_player: PlayerFixture = fixture::player();
    assert_eq!(default_player.name, "Test Player");
    assert_eq!(default_player.level, 1);

    // FixtureBuilder 链式覆盖
    let custom: PlayerFixture = FixtureBuilder::new(fixture::player())
        .with_name("Aragorn")
        .with_level(99)
        .build();

    assert_eq!(custom.name, "Aragorn");
    assert_eq!(custom.level, 99);
    // 其他字段 (id, created_at) 由 fixture::player() 提供, 保留原值
    assert!(!custom.id.is_empty(), "id 应由 fixture::player() 赋值");
}

// ============================================================================
// Test 2: 真 PG 路径下 player INSERT/SELECT 走 FixtureBuilder
// ============================================================================

/// 验证 FixtureBuilder 产出的 PlayerFixture 字段可作为 INSERT 参数写入
/// 真 PG (per ARC-008 player_db 独立), 并可 SELECT 读回.
///
/// 强约束 (per WF-1-55.31 PgTestDatabase): 用 `#[rgs_testkit::pg_test]`
/// 宏拿真 PgPool, 不用 InMemory mock.
///
/// 注: PlayerFixture.id 形如 "player-test-001" 是占位 string, DB id 列是 UUID;
/// INSERT 时单独生成 UUID, 用 fixture name/level 字段.
#[pg_test(migrations = "tests/migrations")]
async fn player_fixture_inserts_and_reads_back_in_real_pg(pool: PgPool) {
    let p: PlayerFixture = FixtureBuilder::new(fixture::player())
        .with_name("Legolas")
        .with_level(85)
        .build();

    // 单独生成 UUID (PlayerFixture.id 是占位 string, DB id 列是 UUID)
    let player_uuid = uuid::Uuid::new_v4();

    // INSERT 走玩家域 players 表 (per DTL-018 §3.1)
    sqlx::query(
        "INSERT INTO players (id, name, level, vip_level, status)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(player_uuid)
    .bind(&p.name)
    .bind(p.level as i32)
    .bind(0_i32)
    .bind("active")
    .execute(&pool)
    .await
    .expect("INSERT player 走真 PG 必须成功");

    // SELECT 读回
    let (name, level): (String, i32) =
        sqlx::query_as("SELECT name, level FROM players WHERE id = $1")
            .bind(player_uuid)
            .fetch_one(&pool)
            .await
            .expect("SELECT player 走真 PG 必须成功");

    assert_eq!(name, "Legolas");
    assert_eq!(level, 85);
}

// ============================================================================
// Test 3: outbox CHECK 约束幂等 + invalid status 拒 insert
// ============================================================================

/// 验证 0003_outbox_check_idempotent.sql 的 `chk_outbox_status` CHECK 约束
/// 真的存在, 且 (1) 拒 invalid status (2) 允 valid status.
///
/// 锚定: WF-1-55.28 outbox 幂等 migration 行为契约 (per RGS-REV-009 CR-2),
/// 5 域应统一采用 (per RGS-OPEN-QA-001 v0.2 Q-M-02 答复).
#[pg_test(migrations = "tests/migrations")]
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
         VALUES (gen_random_uuid(), 'wf_1_55_44', '{}'::jsonb, gen_random_uuid(), 'pending')",
    )
    .execute(&pool)
    .await;

    assert!(
        insert_valid.is_ok(),
        "INSERT valid status 'pending' 必须成功, got {:?}",
        insert_valid
    );
}
