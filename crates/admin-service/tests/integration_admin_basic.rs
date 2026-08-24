//! WF-1-55.44: admin-service rgs-testkit 集成测试骨架
//!
//! ## 目的
//! 验证 admin-service 已正确接入 rgs-testkit FixtureBuilder (含 `admin_action()`
//! sample 工厂 + FixtureBuilder 链式 `with_action` / `with_target`), 且 outbox
//! CHECK 约束幂等 migration 行为正确 (per RGS-OPEN-QA-001 v0.2 Q-M-02 答复).
//!
//! ## 范围 (4 域对称骨架)
//! 1. **`#[rgs_testkit::pg_test]` 真 PG 集成测试** —— 验证 audit_log 哈希链
//!    INSERT 走 FixtureBuilder 自定义 admin action sample (per DTL-031 admin
//!    action 4 种: ban / mute / promote / demote; per RGS-SEC-100 §7 audit_log
//!    hash 链 + append-only 触发器).
//! 2. **outbox CHECK 约束幂等验证** —— 同 player/match/social 模板.
//!
//! ## 跑测前置
//! - `DATABASE_URL=postgres://postgres@127.0.0.1:5555/admin_db` (或任何可写 PG)
//! - migrations 已跑过 (本 test 假设 migrations 存在, 不自动 migrate)

use rgs_testkit::fixture::{self, FixtureBuilder, AdminFixture};
use rgs_testkit::pg_test;
use sqlx::PgPool;

// ============================================================================
// Test 1: FixtureBuilder 链式 API 自定义 admin action sample data
// ============================================================================

/// 验证 FixtureBuilder<AdminFixture> 链式覆盖 action / target 字段.
///
/// 强约束 (per WF-1-55.31): 用 rgs_testkit::pg_test 强制走真 PG 路径,
/// 禁止 InMemory 假象.
#[pg_test]
async fn admin_fixture_builder_customizes_action_and_target(_pool: PgPool) {
    // 默认 fixture: action="ban", target="player-test-001"
    let default_action: AdminFixture = fixture::admin_action("admin-001", "ban", "player-test-001");
    assert_eq!(default_action.action, "ban");
    assert_eq!(default_action.target_id, "player-test-001");
    assert_eq!(default_action.admin_id, "admin-001");

    // FixtureBuilder 链式覆盖
    let mute_action: AdminFixture = FixtureBuilder::new(fixture::admin_action(
        "admin-001",
        "ban",
        "player-test-001",
    ))
    .with_action("mute")
    .with_target("player-spammer-007")
    .build();

    assert_eq!(mute_action.admin_id, "admin-001");
    assert_eq!(mute_action.action, "mute");
    assert_eq!(mute_action.target_id, "player-spammer-007");
}

// ============================================================================
// Test 2: 真 PG 路径下 audit_log INSERT 走 FixtureBuilder
// ============================================================================

/// 验证 admin-service audit_log 哈希链 INSERT 走 FixtureBuilder 自定义
/// admin action sample (per RGS-SEC-100 §7).
///
/// audit_log 模式 (per 0001_init.sql + 0002_audit.sql):
///   id, actor_id, action, target, payload, prev_hash, hash (UNIQUE),
///   prev_hash UNIQUE (per 0002_audit_prev_hash_unique)
///
/// 哈希链: 简化本 test 用 `0..0` 64-hex-char prev_hash + 派生 hash,
/// 实际 service 层 (per WF-1-55.13 sha2 升级) 用 sha256(prev_hash || row_data).
#[pg_test]
async fn admin_fixture_creates_audit_log_in_real_pg(pool: PgPool) {
    let act: AdminFixture = FixtureBuilder::new(fixture::admin_action(
        "admin-uuid-placeholder",
        "ban",
        "player-test-001",
    ))
    .with_action("promote")
    .with_target("player-new-mod-007")
    .build();

    // actor_id 必须是合法 UUID; admin_id 是占位 string, 解析失败时用 new_v4
    let actor_uuid =
        uuid::Uuid::parse_str(&act.admin_id).unwrap_or_else(|_| uuid::Uuid::new_v4());

    // prev_hash 与 hash 用不同 64-hex-char 占位 (实服务走 sha256)
    let prev_hash = "0".repeat(64);
    let row_hash = format!("{:064x}", uuid::Uuid::new_v4().as_u128());

    sqlx::query(
        "INSERT INTO audit_log (id, actor_id, action, target, payload, prev_hash, hash)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(actor_uuid)
    .bind(&act.action)
    .bind(&act.target_id)
    .bind(format!(
        "{{\"performed_at\":\"{}\",\"action\":\"{}\"}}",
        act.performed_at.to_rfc3339(),
        act.action
    ))
    .bind(&prev_hash)
    .bind(&row_hash)
    .execute(&pool)
    .await
    .expect("INSERT audit_log 走真 PG 必须成功");

    // SELECT 读回 (per actor + action 唯一定位)
    let (action, target): (String, String) = sqlx::query_as(
        "SELECT action, target FROM audit_log WHERE actor_id = $1 AND action = $2",
    )
    .bind(actor_uuid)
    .bind(&act.action)
    .fetch_one(&pool)
    .await
    .expect("SELECT audit_log 走真 PG 必须成功");

    assert_eq!(action, "promote");
    assert_eq!(target, "player-new-mod-007");
}

// ============================================================================
// Test 3: outbox CHECK 约束幂等 + invalid status 拒 insert
// ============================================================================

/// 验证 0004_outbox_check_idempotent.sql 的 `chk_outbox_status` CHECK 约束
/// 真的存在, 且 (1) 拒 invalid status (2) 允 valid status.
///
/// 注: admin-service 用 0004 而非 0003 (per 0002_audit_prev_hash_unique
/// 占了 0002 序号, outbox 在 0003, idempotent 在 0004).
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
        "chk_outbox_status CHECK 约束必须存在 (0004_outbox_check_idempotent.sql 已跑过)"
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
         VALUES (gen_random_uuid(), 'wf_1_55_44', '{}'::jsonb, gen_random_uuid(), 'failed')",
    )
    .execute(&pool)
    .await;

    assert!(
        insert_valid.is_ok(),
        "INSERT valid status 'failed' 必须成功, got {:?}",
        insert_valid
    );
}
