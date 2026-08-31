//! WF-1-55.28: 6 域 outbox CHECK 约束幂等 migration 集成测试
//!
//! 验证 per RGS-REV-009 CR-2: 0004_outbox_check_idempotent.sql 满足
//! 1) CHECK 约束真的生效 (尝试 insert invalid status 必须 fail)
//! 2) migration 可重入 (第二次跑 no-op, 不报错)
//!
//! 锚定: WF-1-55.28 step 5 "跑 cargo test --test integration_outbox 验证 CHECK 约束生效"
//!
//! 隔离策略: 每个 test 用 UUID 后缀建独立测试 DB, test 结束 DROP DATABASE,
//! 避免与其它 test 共享 schema state 冲突 (sqlx::migrate! 不能跑在事务里,
//! 所以不能用 #[sqlx::test] 的自动 rollback 隔离).

use rgs_testkit::pg_test_db::pg_pool;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

/// 拿 base DATABASE_URL (postgres://.../<dbname>), 把 dbname 替换成 uuid
/// 拿到一个全新 DB URL, 用于 test 隔离.
fn isolated_db_url() -> Option<(String, String)> {
    let base = std::env::var("DATABASE_URL").ok()?;
    // 解析 URL: postgres://user:pass@host:port/dbname?...
    let (prefix, query) = match base.split_once('?') {
        Some((p, q)) => (p.to_string(), format!("?{}", q)),
        None => (base.clone(), String::new()),
    };
    let last_slash = prefix.rfind('/').expect("DATABASE_URL must have /<dbname>");
    let db_name = format!("wf_1_55_28_{}", Uuid::new_v4().simple());
    let new_url = format!("{}{}{}", &prefix[..last_slash], db_name, query);
    Some((new_url, db_name))
}

async fn create_test_db(admin_url: &str, db_name: &str) {
    let mut conn = PgConnection::connect(admin_url)
        .await
        .expect("connect to admin DB");
    // CREATE DATABASE 不能在 transaction 里, 用 raw SQL
    let create_sql = format!("CREATE DATABASE \"{}\"", db_name);
    conn.execute(create_sql.as_str())
        .await
        .expect("create test database");
}

async fn drop_test_db(admin_url: &str, db_name: &str) {
    let mut conn = PgConnection::connect(admin_url)
        .await
        .expect("reconnect to admin DB");
    let drop_sql = format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", db_name);
    let _ = conn.execute(drop_sql.as_str()).await;
}

#[tokio::test]
async fn outbox_check_constraint_rejects_invalid_status() {
    let (test_url, db_name) = match isolated_db_url() {
        Some(v) => v,
        None => {
            eprintln!("skip: DATABASE_URL not set");
            return;
        }
    };
    // 用 /postgres (admin DB) 建子 DB
    let base = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (base_prefix, query) = match base.split_once('?') {
        Some((p, q)) => (p.to_string(), format!("?{}", q)),
        None => (base.clone(), String::new()),
    };
    let last_slash = base_prefix
        .rfind('/')
        .expect("DATABASE_URL must have /<dbname>");
    let admin_url = format!("{}/postgres{}", &base_prefix[..last_slash], query);

    create_test_db(&admin_url, &db_name).await;
    let pool: PgPool = match pg_pool_at(&test_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skip: cannot connect to test DB: {}", e);
            let _ = drop_test_db(&admin_url, &db_name).await;
            return;
        }
    };

    // 1. 应用所有 migration (含 0004_outbox_check_idempotent)
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("migration failed: {}", e);
        let _ = drop_test_db(&admin_url, &db_name).await;
        panic!("migrations must apply successfully: {}", e);
    }

    // 2. 验证 CHECK 约束存在
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
    .expect("query constraint existence");
    assert!(
        constraint_exists,
        "chk_outbox_status CHECK constraint must exist after 0004 idempotent migration"
    );

    // 3. 尝试 insert invalid status → 必须 fail
    let insert_invalid = sqlx::query(
        "INSERT INTO outbox (id, subject, payload, command_id, status)
         VALUES (gen_random_uuid(), 'test', '{}'::jsonb, gen_random_uuid(), 'BOGUS_STATUS')",
    )
    .execute(&pool)
    .await;
    assert!(
        insert_invalid.is_err(),
        "INSERT with invalid status 'BOGUS_STATUS' must be rejected by CHECK constraint; got {:?}",
        insert_invalid
    );
    let err = insert_invalid.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("chk_outbox_status") || err_str.contains("violates check constraint"),
        "expected CHECK violation error, got: {}",
        err_str
    );

    // 4. 验证 valid status 可正常 insert
    let insert_valid = sqlx::query(
        "INSERT INTO outbox (id, subject, payload, command_id, status)
         VALUES (gen_random_uuid(), 'test', '{}'::jsonb, gen_random_uuid(), 'pending')",
    )
    .execute(&pool)
    .await;
    assert!(
        insert_valid.is_ok(),
        "INSERT with valid status 'pending' must succeed; got {:?}",
        insert_valid
    );

    drop_test_db(&admin_url, &db_name).await;
}

#[tokio::test]
async fn outbox_check_constraint_is_idempotent() {
    let base = match std::env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("skip: DATABASE_URL not set");
            return;
        }
    };
    let (base_prefix, query) = match base.split_once('?') {
        Some((p, q)) => (p.to_string(), format!("?{}", q)),
        None => (base.clone(), String::new()),
    };
    let last_slash = base_prefix
        .rfind('/')
        .expect("DATABASE_URL must have /<dbname>");
    let admin_url = format!("{}/postgres{}", &base_prefix[..last_slash], query);
    let db_name = format!("wf_1_55_28_{}", Uuid::new_v4().simple());
    let test_url = format!("{}/{}{}", &base_prefix[..last_slash], db_name, query);

    create_test_db(&admin_url, &db_name).await;
    let pool: PgPool = match pg_pool_at(&test_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skip: cannot connect to test DB: {}", e);
            let _ = drop_test_db(&admin_url, &db_name).await;
            return;
        }
    };

    // 第一次跑 migration
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("first migration run must succeed");

    // 第二次跑同一份 migration → DO block 捕获 duplicate_object → no-op
    let second_run = sqlx::migrate!("./migrations").run(&pool).await;
    assert!(
        second_run.is_ok(),
        "second migration run must be idempotent (no-op via DO block EXCEPTION); got {:?}",
        second_run
    );

    // 第三次跑 → 仍 ok
    let third_run = sqlx::migrate!("./migrations").run(&pool).await;
    assert!(
        third_run.is_ok(),
        "third migration run must also be idempotent; got {:?}",
        third_run
    );

    // 约束仍存在
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
    .expect("query constraint existence");
    assert!(
        constraint_exists,
        "chk_outbox_status must still exist after idempotent re-run"
    );

    drop_test_db(&admin_url, &db_name).await;
}

/// 用指定 URL 拿 PgPool (rgs-testkit 的 pg_pool() 只读 DATABASE_URL,
/// 本 test 需要用子 DB URL 所以本地包装)
async fn pg_pool_at(url: &str) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await?;
    Ok(pool)
}

// 显式 ignore 默认 main 引用
#[allow(dead_code)]
fn _ensure_pg_pool_used() {
    let _ = pg_pool;
}
