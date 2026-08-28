//! WF-1-55.44: social-service rgs-testkit 集成测试骨架
//!
//! ## 目的
//! 验证 social-service 已正确接入 rgs-testkit FixtureBuilder (含 `social_message()`
//! sample 工厂 + FixtureBuilder 链式 `with_message`), 且 outbox CHECK 约束
//! 幂等 migration 行为正确 (per RGS-OPEN-QA-001 v0.2 Q-M-02 答复).
//!
//! ## 范围 (4 域对称骨架)
//! 1. **`#[rgs_testkit::pg_test]` 真 PG 集成测试** —— 验证 guilds 表可 INSERT/SELECT
//!    走 FixtureBuilder 自定义 social message sample data (per DTL-019 friend /
//!    DTL-020 message).
//! 2. **outbox CHECK 约束幂等验证** —— 同 player/match 模板.
//!
//! ## 跑测前置
//! - `DATABASE_URL=postgres://postgres@127.0.0.1:5555/social_db` (或任何可写 PG)
//! - migrations 已跑过 (本 test 假设 migrations 存在, 不自动 migrate)

use rgs_testkit::fixture::{self, FixtureBuilder, SocialFixture};
use rgs_testkit::pg_test;
use sqlx::PgPool;

// ============================================================================
// Test 1: FixtureBuilder 链式 API 自定义 social message sample data
// ============================================================================

/// 验证 FixtureBuilder<SocialFixture> 链式覆盖 message 字段.
///
/// 强约束 (per WF-1-55.31): 用 rgs_testkit::pg_test 强制走真 PG 路径,
/// 禁止 InMemory 假象.
#[pg_test]
async fn social_fixture_builder_customizes_message(_pool: PgPool) {
    // 默认 fixture: message="Hello from test"
    let default_msg: SocialFixture = fixture::social_message("alice", "bob");
    assert_eq!(default_msg.message, "Hello from test");
    assert_eq!(default_msg.player_id, "alice");
    assert_eq!(default_msg.friend_id, "bob");

    // FixtureBuilder 链式覆盖
    let custom: SocialFixture = FixtureBuilder::new(fixture::social_message("alice", "bob"))
        .with_message("Custom greeting from integration test")
        .build();

    assert_eq!(custom.player_id, "alice");
    assert_eq!(custom.friend_id, "bob");
    assert_eq!(custom.message, "Custom greeting from integration test");
    // sent_at 由 fixture::social_message() 填, 保留
    assert!(custom.sent_at <= chrono::Utc::now());
}

// ============================================================================
// Test 2: 真 PG 路径下 guild INSERT/SELECT 走 FixtureBuilder 链
// ============================================================================

/// 验证 social-service guilds 表可 INSERT, 跨 DTL-019 (friend) / DTL-020
/// (message) 集成样本一致.
///
/// 注: social-service 当前 schema 用 guilds/guild_members (per DTL-026 §3.1
/// 5 域 social 域实现选型), 不直接存 social message; 本 test 主要验证
/// FixtureBuilder 链 → 域 sample → 真 PG INSERT 闭环, 不绑定具体 schema 字段.
#[pg_test]
async fn social_fixture_creates_guild_in_real_pg(pool: PgPool) {
    // 用 social_message fixture 当作社交事件的 source-of-truth 占位
    let msg: SocialFixture = FixtureBuilder::new(fixture::social_message("alice", "bob"))
        .with_message("Greetings from WF-1-55.44 integration test")
        .build();

    // INSERT 一个 guild (leader_id = msg.player_id 解析为 UUID)
    let leader_uuid =
        uuid::Uuid::parse_str(&msg.player_id).unwrap_or_else(|_| uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO guilds (id, name, description, leader_id, level, member_count, experience)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(format!("guild-{}", msg.friend_id)) // name UNIQUE
    .bind(&msg.message) // 用 message 作 guild description
    .bind(leader_uuid)
    .bind(1_i32)
    .bind(1_i32)
    .bind(0_i64)
    .execute(&pool)
    .await
    .expect("INSERT guild 走真 PG 必须成功");

    // SELECT 读回
    let description: String = sqlx::query_scalar(
        "SELECT description FROM guilds WHERE leader_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(leader_uuid)
    .fetch_one(&pool)
    .await
    .expect("SELECT guild 走真 PG 必须成功");

    assert_eq!(description, "Greetings from WF-1-55.44 integration test");
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
         VALUES (gen_random_uuid(), 'wf_1_55_44', '{}'::jsonb, gen_random_uuid(), 'sent')",
    )
    .execute(&pool)
    .await;

    assert!(
        insert_valid.is_ok(),
        "INSERT valid status 'sent' 必须成功, got {:?}",
        insert_valid
    );
}
