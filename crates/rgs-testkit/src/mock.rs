//! Mock 工具集
//!
//! 提供 5 域测试用的 mock 能力：
//! - `DbMock`             模拟 PostgreSQL 连接（**已 deprecated, 见下方强约束**）
//! - `GrpcMock`           模拟 tonic gRPC server（trait + 实质实现 `TonicGrpcMock`）
//! - `NatsMock`           模拟 NATS JetStream subject（实质化 trait, 54.x）
//! - `InMemoryNatsMock`   NATS mock 的可工作实现: `Arc<Mutex<HashMap>>` subject store
//!
//! 53.3 骨架版：3 个 trait 占位 + 1 个空实现，让 5 域 DTL §6 测试规格
//! 引用时不报"unresolved import"。
//!
//! 53.4 CI 修正：clippy 1.98 的 `async_fn_in_trait` / `manual_async_fn` 互相对立
//! （async fn 触发前者，impl Future 触发后者），53.4 选择 `async fn` 形式
//! + 在 rust-ci.yml 用 `-A clippy::async_fn_in_trait` 抑制该 pedantic 警告。
//!
//! W10 (2026-08-28) P3 follow-up: 升级 DbMock / NoopMock 弃用警告
//! - 增加 #![deny(deprecated)] 让 *新* 代码引用 mock::DbMock 编译失败
//! - 旧代码 (2026-08-28 前) 仍允许 `#[allow(deprecated)]` 抑制
//!
//! 54.x 接入：
//! - `GrpcMock` 已升级：`TonicGrpcMock` 用 `mockito::Server` 启动真 HTTP mock
//!   server，提供 `url()` 给 tonic client connect，提供 `expect()` 注册
//!   expectation。5 域 gRPC 集成测试 fixture 可直接使用。
//! - `NatsMock` 已实质化为 `InMemoryNatsMock` —— 用 `Arc<Mutex<HashMap>>`
//!   模拟 subject store, 不引 `async-nats` 重依赖 (mock 自包含, 不需真 NATS
//!   server 就能跑单元 / 集成 test; 真 NATS 行为靠 Phase 0.5 Step 2+3 已部署的
//!   NATS server 验, 走 e2e / 集成 env). 测试用 abstraction, 生产路径直接用
//!   `async_nats::Client`.
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
//!
//! # gRPC / NATS mock 不在 deprecated 范围
//!
//! per RGS-REV-009 V3 H-1 / WF-1-55.31: 56.x 起禁止 NoopMock 用于 PG,
//! 但 **gRPC / NATS mock 是另一回事**（mockito 是 HTTP server, 真起 socket;
//! NATS mock 同理）, 不在 deprecated 范围, 可继续演进.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
///
/// 53.3 占位 trait；54.x 起由 `TonicGrpcMock` 给出实质实现
/// （基于 `mockito::Server` 启动真 HTTP mock server）。
///
/// # 用法
///
/// ```ignore
/// let mut m = TonicGrpcMock::new().await;
/// m.expect("POST", "/player.v1.PlayerService/Login", 200, br#"{"ok":true}"#);
/// let url = m.url(); // 给 tonic client::connect(url)
/// ```
pub trait GrpcMock: Send + Sync {
    /// 启动 mock server（占位兼容，实质工作由 `TonicGrpcMock::new` 完成）
    #[allow(async_fn_in_trait)]
    async fn serve(&self) -> anyhow::Result<()>;

    /// 返回 mock server URL（给 tonic client connect 用）
    fn url(&self) -> &str;

    /// 注册 HTTP expectation（mockito 风格：method + path → status + body）
    fn expect(&mut self, method: &str, path: &str, status: u16, body: &[u8]);
}

/// NATS JetStream subject mock 标记 trait
///
/// 54.x 接入：从占位升级为可工作 abstraction. 三个方法:
/// - `publish`     把 payload 追加到 subject 累积消息队列
/// - `subscribe`   取出该 subject 累积的全部消息 (FIFO)
/// - `received_count`  计数该 subject 已 publish 的消息数 (sync, 测试 assert 用)
///
/// 生产路径走 `async_nats::Client` (per Phase 0.5 Step 2+3), 本 trait 仅为
/// 单元 / 集成 test fixture. 不实现 JetStream 持久化 / ack / consumer group
/// 等高级语义——那些需要真 NATS server, 走 e2e / 集成 env 验.
pub trait NatsMock: Send + Sync {
    /// 模拟 subject publish
    #[allow(async_fn_in_trait)]
    async fn publish(&self, subject: &str, payload: &[u8]) -> anyhow::Result<()>;

    /// 取出该 subject 累积的所有消息（FIFO）
    #[allow(async_fn_in_trait)]
    async fn subscribe(&self, subject: &str) -> anyhow::Result<Vec<Vec<u8>>>;

    /// 计数该 subject 已 publish 的消息数 (sync, 用于测试 assert)
    fn received_count(&self, subject: &str) -> usize;
}

/// InMemory NATS subject store (轻量 mock, 不引 `async-nats` 重依赖)
///
/// 模拟 NATS JetStream subject 行为: `publish` 存, `subscribe` 取 (FIFO).
/// 用于 5 域跨域事件测试 fixture (per DTL-021~025 + ARC-051).
///
/// **不**实现: JetStream 持久化 / ack / consumer group / wildcard subject
/// 匹配 / 流控. 这些高级语义需要真 NATS server, 走 e2e / 集成 env 验.
///
/// 线程安全: `Arc<Mutex<HashMap>>` —— 5 域并发 publish / subscribe 安全.
pub struct InMemoryNatsMock {
    store: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
}

impl InMemoryNatsMock {
    /// 新建空 store
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryNatsMock {
    fn default() -> Self {
        Self::new()
    }
}

impl NatsMock for InMemoryNatsMock {
    async fn publish(&self, subject: &str, payload: &[u8]) -> anyhow::Result<()> {
        let mut store = self.store.lock().expect("InMemoryNatsMock mutex poisoned");
        store
            .entry(subject.to_string())
            .or_default()
            .push(payload.to_vec());
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> anyhow::Result<Vec<Vec<u8>>> {
        let store = self.store.lock().expect("InMemoryNatsMock mutex poisoned");
        Ok(store.get(subject).cloned().unwrap_or_default())
    }

    fn received_count(&self, subject: &str) -> usize {
        let store = self.store.lock().expect("InMemoryNatsMock mutex poisoned");
        store.get(subject).map(|v| v.len()).unwrap_or(0)
    }
}

/// 实质 `GrpcMock` 实现（基于 `mockito::Server` 启动真 HTTP mock server）
///
/// 与 `NoopMock` 不同，本实现真正启动一个 HTTP mock server，注册 expectation，
/// 然后把 `url()` 给 tonic client 端 connect 用。用于 5 域 gRPC 集成测试
/// fixture（player / economy / match / social / admin 域 DTL §6）。
///
/// # 生命周期
///
/// `TonicGrpcMock` 持有 `mockito::ServerGuard`，只要实例存活 mock server 就活着。
/// 超出 scope 时 server 归还 pool，expectation 失效。
pub struct TonicGrpcMock {
    server: mockito::ServerGuard,
    /// 缓存 URL string, 避免 `url()` 返回 dangling borrow
    url: String,
}

impl TonicGrpcMock {
    /// 启动一个新 mock server（async, lazy bind 一个随机端口）
    pub async fn new() -> Self {
        let server = mockito::Server::new_async().await;
        let url = server.url(); // mockito 返回 `http://127.0.0.1:PORT`
        Self { server, url }
    }
}

impl GrpcMock for TonicGrpcMock {
    /// mockito server 在 `new_async` 时已 lazy 启动；本方法仅作 trait 兼容占位
    async fn serve(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn url(&self) -> &str {
        &self.url
    }

    fn expect(&mut self, method: &str, path: &str, status: u16, body: &[u8]) {
        // mockito `with_status` 签名是 `usize`, 显式转换 (HTTP 状态码 < 65536, 无损)
        let mut m = self.server.mock(method, path);
        m = m.with_status(status as usize);
        if !body.is_empty() {
            m = m.with_body(body);
        }
        m.create();
    }
}

/// 默认空实现（53.3 占位；54.x 接入 sqlx-mock / mockito / async-nats-mock）
///
/// **DEPRECATED**: 用 `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]` 取代.
/// 56.x 起, 任何 saga / 事务 / OCC / outbox 测试禁止用本类型.
///
/// 注意：`NoopMock` 仅 PG 部分 deprecated；`GrpcMock` / `NatsMock` 实现仍可用
/// （向后兼容），但实质工作请用 `TonicGrpcMock`。
#[deprecated(
    since = "0.2.0",
    note = "Mock PG 不等价真 PG. 用 rgs_testkit::pg_pool() + #[rgs_testkit::pg_test] 取代. \
            详见 RGS-REV-009 V3 H-1 / WF-1-55.31. \
            (注: GrpcMock / NatsMock 实现仍可用, 实质 mock 用 TonicGrpcMock.)"
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

    fn url(&self) -> &str {
        "http://mock.invalid"
    }

    fn expect(&mut self, _method: &str, _path: &str, _status: u16, _body: &[u8]) {
        // NoopMock 不实际 mock 任何东西
    }
}

#[allow(deprecated)]
impl NatsMock for NoopMock {
    async fn publish(&self, _subject: &str, _payload: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _subject: &str) -> anyhow::Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    fn received_count(&self, _subject: &str) -> usize {
        0
    }
}
