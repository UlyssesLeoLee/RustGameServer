//! Mock 工具集
//!
//! 提供 5 域测试用的 mock 能力：
//! - `DbMock`     模拟 PostgreSQL 连接（占位 trait，53.3 仅声明）
//! - `GrpcMock`   模拟 tonic gRPC server（占位 trait）
//! - `NatsMock`   模拟 NATS JetStream subject（占位 trait）
//!
//! 53.3 骨架版：3 个 trait 占位 + 1 个空实现，让 5 域 DTL §6 测试规格
//! 引用时不报"unresolved import"。

/// PostgreSQL 连接池 mock 标记 trait
pub trait DbMock: Send + Sync {
    /// 返回 mock 连接字符串（用于 sqlx::PgPool::connect_lazy）
    fn mock_url(&self) -> &str;
}

/// tonic gRPC server mock 标记 trait
pub trait GrpcMock: Send + Sync {
    /// 启动 mock server（占位）
    async fn serve(&self) -> anyhow::Result<()>;
}

/// NATS JetStream subject mock 标记 trait
pub trait NatsMock: Send + Sync {
    /// 模拟 subject publish（占位）
    async fn publish(&self, subject: &str, payload: &[u8]) -> anyhow::Result<()>;
}

/// 默认空实现（53.3 占位；54.x 接入 sqlx-mock / mockito / async-nats-mock）
pub struct NoopMock;

impl DbMock for NoopMock {
    fn mock_url(&self) -> &str { "postgres://mock@localhost:5432/mock" }
}

impl GrpcMock for NoopMock {
    async fn serve(&self) -> anyhow::Result<()> { Ok(()) }
}

impl NatsMock for NoopMock {
    async fn publish(&self, _subject: &str, _payload: &[u8]) -> anyhow::Result<()> { Ok(()) }
}