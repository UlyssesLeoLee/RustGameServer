//! PgTestDatabase fixture (per RGS-REV-009 V3 H-1 / WF-1-55.31)
//!
//! 提供真 PG 测试 fixture, 强制 56.x 起新代码用 `#[sqlx::test]` 写真 DB 集成测试.
//! 防 '209 test pass ≠ correct' 假象复发 (RGS-REV-009 V3 H-1 共识: DC-1 + CC-4 新增
//! test 全是 InMemory unit test, InMemoryAccountRepository 的 OCC 行为虽真但需手动
//! bump version 才能触发, 与真 PG `UPDATE ... WHERE version = ?` 0 row 不等价).
//!
//! # 强约束 (per WF-1-55.31 retry)
//!
//! - **新代码必须**用 `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
//! - **禁止**用 `rgs_testkit::mock::NoopMock` 或 `rgs_testkit::mock::DbMock`
//!   模拟 PG 连接 (已 `#[deprecated]`)
//!
//! # 用法 (推荐, `#[rgs_testkit::pg_test]` 宏 + 本 fixture)
//!
//! ```ignore
//! // 在域 crate (如 economy-service) 的 tests/ 目录加真 DB 集成 test:
//! use rgs_testkit::pg_pool;
//! use rgs_testkit::pg_test;
//! use sqlx::PgPool;
//!
//! #[pg_test]
//! async fn resume_concurrent_race_rejected_by_occ(pool: PgPool) {
//!     let repo = PgAccountRepository::new(pool.clone());
//!     // ... 真 PG 行为测试 (OCC, 事务隔离, 并发竞争)
//! }
//! ```
//!
//! # 拒绝的反 pattern (编译期锚定)
//!
//! 以下模式会产生 `#[deprecated]` 警告 (不 fail compile, 但 CI 应将 deprecation
//! 升级为 deny 强制拒绝):
//!
//! ```no_run
//! // 错误用法: 仍用 NoopMock 模拟 PG (per RGS-REV-009 V3 H-1 强约束, 56.x 起禁止)
//! use rgs_testkit::mock::NoopMock;
//! use rgs_testkit::mock::DbMock;
//!
//! #[tokio::test]
//! async fn saga_test_with_mock() {
//!     let mock = NoopMock;  // <-- #[deprecated] 警告
//!     let _url = DbMock::mock_url(&mock);  // <-- #[deprecated] 警告
//!     // 实际项目里这里会 panic / 静默通过 / 假象, 不等价真 PG 行为
//! }
//! ```
//!
//! # 启 PG (CI / 本地)
//!
//! ```bash
//! # 启 Docker Desktop 后:
//! docker compose -f docker/compose/docker-compose.yml up -d postgres
//! export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
//! cargo test -p economy-service --features pg-integration -- --include-ignored
//! ```
//!
//! # 跳过机制
//!
//! - **默认 (`cargo test`)**: `pg-integration` feature off, 集成 test 不编译, 不需要 `DATABASE_URL`
//! - **`--features pg-integration`**: 集成 test 编译, 运行时读 `DATABASE_URL` 连真 PG
//! - **PG 不可达**: `pg_available()` 返回 `false`, 域 crate 可用 `if !pg_available().await { return; }` 跳过
//!
//! # 设计权衡
//!
//! - 不引入 testcontainers-rs: Docker Desktop 启停开销大, CI 默认跑 docker compose up
//!   起一次 PG, 多 test 共享 `DATABASE_URL` (sqlx::test 内部用事务回滚隔离, 每 test 独立)
//! - 不引入独立 `rgs-testkit-pg` 子 crate: fixture 30 行, 拆 crate 成本 > 收益
//! - 不强制所有 test 用 PG: InMemory unit test 仍有价值 (快 / 沙箱), 但 56.x 起**新加
//!   的** saga / 事务 / OCC 相关 test 必须用 `#[rgs_testkit::pg_test]` 覆真 DB
//!
//! # 已知限制
//!
//! - sqlx 0.8 的 `#[sqlx::test]` 宏需 `DATABASE_URL` 在**编译时**配置 sqlx offline
//!   cache (`.sqlx/` 目录). 工作区当前未启用 `cargo sqlx prepare`, 因此本 fixture
//!   不依赖 offline mode, 运行时直接连 PG. 后续如离线 build, 需 `cargo sqlx prepare`.
//! - 5 域各自独立 DB (per ARC-008), 5 个 `DATABASE_URL_<DOMAIN>` env var 由各域
//!   crate 的 `#[sqlx::test]` 调用方注入, 本 fixture 只暴露 `DATABASE_URL` 默认入口.

use sqlx::PgPool;

/// 默认 PG 连接 URL 的 env var 名 (sqlx::test 宏 + sqlx::PgPool::from_env 共用)
pub const DATABASE_URL_ENV: &str = "DATABASE_URL";

/// 默认池大小 (适合 CI 单进程, 高并发 case 调高)
pub const DEFAULT_POOL_SIZE: u32 = 8;

/// 读 `DATABASE_URL` env var, 未设置返回 None (不 panic, 用于测试 gate)
pub fn database_url() -> Option<String> {
    std::env::var(DATABASE_URL_ENV).ok()
}

/// 真 PG 连接池; 失败时返回 Err (调用方决定 skip / fail).
///
/// `DATABASE_URL` 未设置 → 立即 Err (避免卡死到 sqlx 内部 connect timeout).
/// 默认 `max_connections = DEFAULT_POOL_SIZE`.
///
/// # Errors
/// - `DATABASE_URL` env var 未设置
/// - sqlx 连接 / 握手失败 (PG 不可达 / 鉴权错)
pub async fn pg_pool() -> anyhow::Result<PgPool> {
    let url =
        database_url().ok_or_else(|| anyhow::anyhow!("{} env var not set", DATABASE_URL_ENV))?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(DEFAULT_POOL_SIZE)
        .connect(&url)
        .await?;
    Ok(pool)
}

/// 探测 PG 是否可达, 用于 `#[test]` 内 gate (无 env var 或连不上均返回 false).
///
/// 适合做 "PG 可达就跑, 不可达就 skip" 的 fallback:
/// ```ignore
/// #[tokio::test]
/// async fn my_pg_test() {
///     if !rgs_testkit::pg_test_db::pg_available().await {
///         eprintln!("skip: PG not reachable");
///         return;
///     }
///     let pool = rgs_testkit::pg_test_db::pg_pool().await.unwrap();
///     // ... 真 PG 测试
/// }
/// ```
pub async fn pg_available() -> bool {
    pg_pool().await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_url_env_name_is_documented() {
        // 防有人改 env var 名忘了改文档
        assert_eq!(DATABASE_URL_ENV, "DATABASE_URL");
    }

    #[test]
    fn default_pool_size_is_reasonable_for_ci() {
        // CI 单进程跑 6 域测试, 8 够用; 高并发 case 由调用方自调 PgPoolOptions
        assert_eq!(DEFAULT_POOL_SIZE, 8);
    }

    #[test]
    fn database_url_returns_none_or_some_without_panic() {
        // 显式 unset 后验证
        let prev = std::env::var(DATABASE_URL_ENV).ok();
        std::env::remove_var(DATABASE_URL_ENV);
        assert!(database_url().is_none());
        if let Some(v) = prev {
            std::env::set_var(DATABASE_URL_ENV, v);
        }
    }
}
