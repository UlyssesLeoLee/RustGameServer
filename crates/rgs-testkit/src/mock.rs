//! Mock 工具集
//!
//! 提供 5 域测试用的 mock 能力：
//! - `DbMock`     模拟 PostgreSQL 连接（**已 deprecated, 见下方强约束**）
//! - `GrpcMock`   模拟 tonic gRPC server（占位 trait）
//! - `NatsMock`   模拟 NATS JetStream subject（占位 trait）
//!
//! 53.3 骨架版：3 个 trait 占位 + 1 个空实现，让 5 域 DTL §6 测试规格
//! 引用时不报"unresolved import"。
//!
//! 53.4 CI 修正：clippy 1.98 的 `async_fn_in_trait` / `manual_async_fn` 互相对立
//! （async fn 触发前者，impl Future 触发后者），53.4 选择 `async fn` 形式
//! + 在 rust-ci.yml 用 `-A clippy::async_fn_in_trait` 抑制该 pedantic 警告。
//!   54.x 接入 sqlx-mock / mockito / async-nats-mock 时再决定最终 API 形式。
//!
//! # ⚠️ 强约束 (per WF-1-55.31 retry, RGS-REV-009 V3 H-1 共识)
//!
//! `DbMock` / `NoopMock` / `mock_url` 全部已加 `#[deprecated]` 警告.
//! 56.x 起, **禁止** 用 `NoopMock` 或 `mock_url()` 模拟 PG 连接作为 saga /
//! 事务 / OCC / outbox 测试 fixture. 根因: InMemoryAccountRepository::apply_atomic
//! 的 OCC 行为虽真但需手动 bump version 才能触发, 真正 PG `UPDATE ... WHERE
//! version = ?` 0 row 行为 + 事务隔离 + 并发竞争需要真 PG 才能暴露.
//! "209 test pass ≠ correct" 假象 (RGS-REV-009 V3 H-1) 即源于此.
//!
//! **唯一接受的替代**: `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`.

/// PostgreSQL 连接池 mock 标记 trait
///
/// **DEPRECATED**: 用 `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]` 取代.
/// 56.x 起, 任何 saga / 事务 / OCC / outbox 测试禁止用本 trait.
#[deprecated(
    since = "0.2.0",
    note = "Mock PG 不等价真 PG (OCC / 事务隔离 / 并发竞争). \
            用 rgs_testkit::pg_pool() + #[rgs_testkit::pg_test] 取代. \
            详见 RGS-REV-009 V3 H-1 / WF-1-55.31."
)]
pub trait DbMock: Send + Sync {
    /// 返回 mock 连接字符串（用于 sqlx::PgPool::connect_lazy）
    fn mock_url(&self) -> &str;
}

/// tonic gRPC server mock 标记 trait
pub trait GrpcMock: Send + Sync {
    /// 启动 mock server（占位）
    #[allow(async_fn_in_trait)]
    async fn serve(&self) -> anyhow::Result<()>;
}

/// NATS JetStream subject mock 标记 trait
pub trait NatsMock: Send + Sync {
    /// 模拟 subject publish（占位）
    #[allow(async_fn_in_trait)]
    async fn publish(&self, subject: &str, payload: &[u8]) -> anyhow::Result<()>;
}

/// 默认空实现（53.3 占位；54.x 接入 sqlx-mock / mockito / async-nats-mock）
///
/// **DEPRECATED**: 用 `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]` 取代.
/// 56.x 起, 任何 saga / 事务 / OCC / outbox 测试禁止用本类型.
#[deprecated(
    since = "0.2.0",
    note = "Mock PG 不等价真 PG. 用 rgs_testkit::pg_pool() + #[rgs_testkit::pg_test] 取代. \
            详见 RGS-REV-009 V3 H-1 / WF-1-55.31."
)]
pub struct NoopMock;

#[allow(deprecated)]
impl DbMock for NoopMock {
    fn mock_url(&self) -> &str {
        "postgres://mock@localhost:5432/mock"
    }
}

#[allow(deprecated)]
impl GrpcMock for NoopMock {
    async fn serve(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[allow(deprecated)]
impl NatsMock for NoopMock {
    async fn publish(&self, _subject: &str, _payload: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
}
