//! rgs-testkit —— RustGameServer 测试套件骨架
//!
//! 四大子模块：
//! - `mock`        DB / gRPC / NATS JetStream mock 工具 (deprecated, 见下方强约束)
//! - `helper`      config 加载 / tracing 初始化 / 公共 assert helper
//! - `fixture`     sample data 工厂 + 6 DB init/teardown
//! - `pg_test_db`  真 PG 测试 fixture (per RGS-REV-009 V3 H-1 / WF-1-55.31,
//!   防 '209 test pass ≠ correct' 假象复发)
//!
//! 5 域 + cluster-ops + shared-platform 测试统一引用本 crate。
//! 规范：RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4
//!
//! 53.3 骨架：3 个子模块各 1-2 个最小可用 API，self-test 覆盖。
//! 55.31 增 `pg_test_db` 子模块, 强制 56.x 起新代码用 `#[sqlx::test]` 写真 DB 集成测试.
//!
//! # 强约束 (per WF-1-55.31 retry, RGS-REV-009 V3 H-1 共识)
//!
//! **56.x 起, 所有**新增**的 saga / 事务 / OCC / outbox 相关测试**必须**用
//! 真 PG (`#[rgs_testkit::pg_test]`), 禁止用 `#[tokio::test]` + InMemoryRepository
//! 或 `MockPool`. 根因: InMemoryAccountRepository::apply_atomic 的 OCC 行为需
//! 手动 bump version 才能触发, 与真 PG `UPDATE ... WHERE version = ?` 0 row
//! 行为不等价 → "209 test pass ≠ correct" 假象.
//!
//! ## 唯一接受的 API
//!
//! - **`pg_pool()`** —— `rgs_testkit::pg_pool().await` 拿真 `PgPool`
//! - **`pg_test`** —— `#[rgs_testkit::pg_test]` 是 `#[sqlx::test]` 的强约束别名,
//!   内部仍然 call sqlx-macros, 但通过本 crate 单一入口强制使用
//!
//! ## 拒绝的 API (编译期 `#[deprecated]` 警告)
//!
//! - `mock::DbMock` / `mock::NoopMock` / `mock_url()` —— 已被 `pg_pool()` 取代
//! - `#[tokio::test]` for DB / saga / OCC / outbox test (单元 test 仍可用)
//! - 任何手写 `InMemoryAccountRepository::new()` 用作 saga/事务 test fixture
//!
//! ## 编译期 `compile_fail` 锚定
//!
//! 本 lib.rs 顶部 + `pg_test_db` 子模块有 `compile_fail` doctest, 演示错误
//! 用法 (会 fail to compile, 防静默通过).
//!
//! ## 退出条件
//!
//! `cargo test -p rgs-testkit` 必须全过 + 任何新 test crate 加 `MockPool` 引用
//! 应产生 `#[deprecated]` 警告 (不静默通过).

pub mod fixture;
pub mod helper;
pub mod mock;
pub mod pg_test_db;

// ============================================================================
// 强约束 re-export (per WF-1-55.31 retry, RGS-REV-009 V3 H-1 共识)
// ============================================================================

/// 真 PG 连接池强约束入口 (re-export of `pg_test_db::pg_pool`).
///
/// 56.x 起, 任何 saga / 事务 / OCC / outbox 测试**必须**走本函数拿真 PgPool,
/// 禁止用 `InMemoryAccountRepository::new()` 或 `mock::NoopMock` 替代.
///
/// # Examples
///
/// ```no_run
/// use rgs_testkit::pg_pool;
///
/// # async fn run() {
/// let pool = pg_pool().await.expect("DATABASE_URL not set or PG unreachable");
/// // ... 真 PG 行为测试
/// # }
/// ```
pub use pg_test_db::pg_pool;

/// 真 PG 集成测试 macro 强约束别名 (re-export of `sqlx::test`).
///
/// 56.x 起, 任何 DB / saga / 事务 / OCC / outbox 集成测试**必须**用
/// `#[rgs_testkit::pg_test]`, 禁止用裸 `#[tokio::test]` 或 `#[sqlx::test]`.
/// 走本 re-export 的目的是: 单一入口, 未来如要加 pre/post hook (e.g. trace_id
/// 注入, 慢查询埋点), 改本 crate 一处即可全生效.
///
/// # Examples
///
/// ```ignore
/// // sqlx::test 宏需 DATABASE_URL 在编译时配置, 跳过 doctest 编译
/// use rgs_testkit::pg_test;
/// use sqlx::PgPool;
///
/// #[pg_test]
/// async fn my_real_pg_test(pool: PgPool) {
///     // pool 来自 sqlx::test 自动注入, 事务回滚隔离
///     let row: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&pool).await.unwrap();
///     assert_eq!(row, 1);
/// }
/// ```
pub use sqlx::test as pg_test;
