# rgs-testkit — RustGameServer 测试套件骨架

> 4 大子模块: `mock` / `helper` / `fixture` / `pg_test_db`
> 规范: RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4

| 模块 | 用途 | 何时用 |
|---|---|---|
| `mock` | DB / gRPC / NATS mock trait + `NoopMock` 占位实现 | 53.3 占位, **56.x 起被 `pg_test_db` 取代** (仅留兼容) |
| `helper` | config 加载 / tracing 初始化 / `assert_eventually!` | 所有测试 |
| `fixture` | sample data 工厂 + `init_test_db` 占位 | 5 域测试 fixture (53.3 仅 3 个域) |
| `pg_test_db` | **真 PG fixture (DATABASE_URL + #[sqlx::test])** | 56.x 起 saga/事务/OCC/outbox 测试 |

---

## 1. 快速开始

```toml
# 域 crate Cargo.toml
[dev-dependencies]
rgs-testkit = { path = "../rgs-testkit" }
```

```rust
// 真 PG 集成测试 (56.x 起所有 saga / 事务 / OCC / outbox test 必须)
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn my_real_pg_test(pool: PgPool) {
    // pool 由 sqlx::test 自动注入, 事务回滚隔离
    let row: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(row, 1);
}
```

---

## 2. 模块总览 (53.3 骨架)

| 子模块 | 公开 API | 状态 |
|---|---|---|
| `mock` | `DbMock` (trait, deprecated), `GrpcMock` (trait), `NatsMock` (trait), `NoopMock` (struct, deprecated) | 占位 trait, 54.x 接入 `mockito` / `async-nats-mock` |
| `helper` | `init_tracing()`, `load_test_env() -> HashMap<String, String>`, `assert_eventually!` 宏 | 已实现 |
| `fixture` | `player() -> PlayerFixture`, `economy(player_id) -> EconomyFixture`, `saga(saga_type) -> SagaFixture`, `init_test_db(name) -> String` | 53.3 3 域, 54.x 补 5 域 (match/social/admin) |
| `pg_test_db` | `pg_pool() -> Result<PgPool>`, `pg_available() -> bool`, `database_url() -> Option<String>`, 常量 `DATABASE_URL_ENV` / `DEFAULT_POOL_SIZE` | **生产就绪 (WF-1-55.31)** |

> **⚠️ 强约束**: 56.x 起, `mock` 子模块的 `DbMock` / `NoopMock` 已 `#[deprecated]`,
> 所有新增的 saga / 事务 / OCC / outbox 测试**必须**走 `pg_test_db` + `pg_test`.
> 详见 §7.

---

## 3. mock 模块

### 3.1 形态 (53.3 占位, 54.x 接入计划)

```rust
use rgs_testkit::mock::{DbMock, GrpcMock, NatsMock, NoopMock};

#[tokio::main]
async fn main() {
    // NoopMock 同时实现 DbMock / GrpcMock / NatsMock 三个 trait
    // 全部方法均为占位,返回 Ok(()),不真实模拟任何行为
    let m = NoopMock;

    // GrpcMock::serve —— 占位
    let _ = m.serve().await;

    // NatsMock::publish —— 占位
    let _ = m.publish("player.events", b"{}").await;

    // ⚠️ DbMock 已 deprecated, 用 rgs_testkit::pg_pool() 取代
    // let _ = m.mock_url();
}
```

### 3.2 54.x 接入路线图

| Trait | 54.x 计划实现 | 用途 |
|---|---|---|
| `GrpcMock` | `TonicGrpcMock` (mockito 集成) | 5 域跨域 gRPC 单元/集成 test |
| `NatsMock` | `InMemoryNatsMock` (in-memory subject store) | 5 域 event bus 单元/集成 test |
| `DbMock` | **永久 deprecated**, 1:1 替换为 `pg_pool()` | — |

> 接入前**禁止**在 saga/事务/OCC/outbox 测试里用 `NoopMock` 替代真 PG
> (per RGS-REV-009 V3 H-1).

### 3.3 ⚠️ DbMock / NoopMock 已 deprecated

```rust
// ❌ 56.x 起禁止 (产生 deprecation 警告, CI deny 后 fail)
use rgs_testkit::mock::NoopMock;
let m = NoopMock;
let _ = m.mock_url();  // ⚠️ deprecated since 0.2.0

// ✅ 唯一接受的替代
use rgs_testkit::{pg_pool, pg_test};
```

---

## 4. helper 模块

```rust
use rgs_testkit::helper::{init_tracing, load_test_env};
use rgs_testkit::assert_eventually;  // 宏从 crate root re-export

#[tokio::test]
async fn helper_demo() {
    init_tracing();  // 幂等, 多次调用仅首次生效

    let env = load_test_env();
    for key in env.keys() {
        eprintln!("env: {}", key);
    }

    // 异步轮询断言: 1s 内等条件 true
    let mut counter = 0;
    let task = tokio::spawn(async {
        for _ in 0..50 {
            counter += 1;
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    });
    task.await.unwrap();
    assert_eventually!(counter == 50, 1000).await;
}
```

> `assert_eventually!` 是 `macro_rules!` 宏, 从 `rgs_testkit` crate root 调用
> (per `#[macro_export]`).

---

## 5. fixture 模块

### 5.1 已实现的 factory (53.3)

| 域 | factory | 返回 |
|---|---|---|
| player | `fixture::player()` | `PlayerFixture { id, name, level, created_at }` |
| economy | `fixture::economy(player_id)` | `EconomyFixture { player_id, currency, gold }` |
| saga | `fixture::saga(saga_type)` | `SagaFixture { saga_id, saga_type, step, state }` |

### 5.2 使用示例

```rust
use rgs_testkit::fixture;

#[test]
fn player_factory_default() {
    let p = fixture::player();
    assert_eq!(p.id, "player-test-001");
    assert_eq!(p.name, "Test Player");
    assert_eq!(p.level, 1);
}

#[test]
fn economy_factory_per_player() {
    let e = fixture::economy("alice");
    assert_eq!(e.player_id, "alice");
    assert_eq!(e.currency, 1000);
    assert_eq!(e.gold, 50);
}

#[test]
fn saga_factory_unique_id() {
    let s1 = fixture::saga("transfer");
    let s2 = fixture::saga("transfer");
    // saga_id 内部含 uuid v4, 每次唯一
    assert_ne!(s1.saga_id, s2.saga_id);
    assert_eq!(s1.saga_type, "transfer");
    assert_eq!(s1.state, "Pending");
}
```

### 5.3 init_test_db (53.3 占位)

```rust
// 53.3 占位: 返回 fake URL, 54.x 接入 testcontainers-rs
let url = rgs_testkit::fixture::init_test_db("player_db").await.unwrap();
// → "postgres://test:test@localhost:5432/player_db" (非真连)
// 56.x 起: 改用 rgs_testkit::pg_test_db::pg_pool() + #[pg_test] 替代
```

### 5.4 54.x 路线图

- `match_game(player_id) -> MatchFixture`
- `social_message(from, to) -> SocialFixture`
- `admin_action(admin, action, target) -> AdminFixture`
- `FixtureBuilder` 链式 API (`.with_name()`, `.with_level()` 等)

5 域 fixture 接入前,各域测试可用 `#[derive(Default)]` 自建 struct + `serde::Serialize`.

---

## 6. pg_test_db 模块 (强约束 / 56.x 必须)

### 6.1 API

```rust
use rgs_testkit::pg_test_db::{pg_pool, pg_available, database_url, DATABASE_URL_ENV, DEFAULT_POOL_SIZE};

// 1) 探测 PG 可达
let ok: bool = pg_available().await;

// 2) 拿 PgPool
let pool: sqlx::PgPool = pg_pool().await?;

// 3) 读 env var (不 panic, 用于 test gate)
let url: Option<String> = database_url();

// 4) 常量
assert_eq!(DATABASE_URL_ENV, "DATABASE_URL");
assert_eq!(DEFAULT_POOL_SIZE, 8);
```

### 6.2 集成测试写法 (推荐)

```rust
// 域 crate tests/pg_integration_resume.rs
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn resume_concurrent_race_rejected_by_occ(pool: PgPool) {
    // 真 PG OCC: UPDATE ... WHERE version = ?  0 row → 拒绝
    // 比 InMemory `stored.version = original_version + 99` 模拟更真
    let repo = economy_service::PgAccountRepository::new(pool.clone());
    // ... 准备 account + reservation
    // ... 2 个并发 resume(同 saga_id) 触发竞争
    // ... 断言: 仅 1 个成功, 另一个收到 OCC conflict
}

#[pg_test]
async fn saga_crash_recovery_uses_real_transaction_isolation(pool: PgPool) {
    // 测 PG Read Committed 隔离级下, saga_orchestrator::resume 的可见性
    // ... 跨事务 prepare + commit, 验 30s 轮询能 resume
}
```

> `pg_test` 是 `sqlx::test` 的 re-export (`pub use sqlx::test as pg_test;`).
> 走本 re-export 单一入口, 未来加 pre/post hook (e.g. trace_id 注入, 慢查询埋点)
> 改本 crate 一处即可全生效.

### 6.3 默认行为 vs `--features pg-integration`

| 命令 | 跑 PG 集成 test? | 需 DATABASE_URL? | 何时用 |
|---|---|---|---|
| `cargo test` | ❌ (集成 test 不编译) | ❌ | CI 默认 / 离线开发 |
| `cargo test --features pg-integration` | ✅ | ✅ (env var) | PR 合并前 / 56.x 起 saga 改动 |
| `cargo test -p rgs-testkit --features pg-integration -- --include-ignored` | ✅ (smoke) | ✅ | 验证 fixture |

`pg-integration` feature 默认关, 保护没 PG 的 CI 环境: 不需要 `DATABASE_URL`.

### 6.4 启 PG (本地 / CI)

#### 本地 (Windows + Docker Desktop)

```powershell
# 1. 启 Docker Desktop (UI 启 或 管理员 PowerShell: Start-Service docker)
Start-Service docker   # 若失败, 用 Docker Desktop UI 启

# 2. 启 6 独立 PG 容器 (per ARC-008, 5 域 + cluster_ops)
cd D:\wf-1-55-31
Copy-Item docker/compose/.env.compose.example docker/compose/.env.compose
# 编辑 .env.compose 填 6 个 *_DB_PASSWORD
docker compose -f docker/compose/docker-compose.yml --env-file docker/compose/.env.compose --profile dev up -d player-db economy-db match-db social-db admin-db cluster-ops-db

# 3. 等 healthcheck 通过
docker compose -f docker/compose/docker-compose.yml ps   # 6 DB 状态 healthy

# 4. 注入 DATABASE_URL (每个域一个; smoke test 用任意一个)
$env:DATABASE_URL = "postgres://player_user:<password>@localhost:15432/player_db"

# 5. 跑 PG 集成 test
cargo test -p rgs-testkit --features pg-integration -- --include-ignored
cargo test -p economy-service --features pg-integration -- --include-ignored
```

#### CI (Linux runner, 多域 PG 容器)

```yaml
# .github/workflows/ci.yml (56.x 计划) 片段
services:
  player-db:
    image: postgres:18.6
    env:
      POSTGRES_DB: player_db
      POSTGRES_USER: player_user
      POSTGRES_PASSWORD: test
    ports: [15432:5432]
    options: --health-cmd "pg_isready" --health-interval 5s
  # ... 其他 5 DB 类似
env:
  DATABASE_URL: postgres://player_user:test@localhost:15432/player_db
steps:
  - run: cargo test --features pg-integration -- --include-ignored
```

### 6.5 故障排查

| 症状 | 原因 | 修法 |
|---|---|---|
| `DATABASE_URL env var not set` | env 没注 | `export DATABASE_URL=...` |
| `connection refused` | PG 容器没启 / 端口错 | `docker compose ps` 查 healthcheck |
| `password authentication failed` | 密码错 / 用户错 | 查 `.env.compose` + 容器内 `psql` 验 |
| `database "xxx_db" does not exist` | 容器用 `POSTGRES_DB` 自动建库, 名错 | 对齐 env var `*_DB_NAME` |
| 跑 `cargo test --features pg-integration` 仍不编 PG test | Cargo feature cache 旧 | `cargo clean -p rgs-testkit` + 重跑 |
| `#[sqlx::test]` 报 `no DATABASE_URL set` | sqlx 0.8 内部强制 | 设 `DATABASE_URL=...` env var |
| `#[sqlx::test]` 报 `query syntax error at compile time` | sqlx offline mode 启用 | 本工作区未启用, 忽略; 如启用, `cargo sqlx prepare` |

### 6.6 已知限制 (56.x 演进项)

1. **无 testcontainers-rs**: Docker Desktop 启停开销大 (10-20s), 当前方案依赖
   CI 预启 docker compose. 56.x 可考虑加 `testcontainers` 后备 (每 test 一个临时容器).
2. **5 域独立 DB → 5 个 DATABASE_URL**: 本 fixture 只暴露 `DATABASE_URL` 默认入口.
   5 域 crate 各自需要 `DATABASE_URL_<DOMAIN>` (e.g. `DATABASE_URL_ECONOMY`) 的
   wrapper, 56.x 加 `pg_test_db::pg_pool_for_domain("economy")` 之类 API.
3. **sqlx offline mode 未启用**: 本 fixture 运行时连 PG, 离线 build 需先
   `cargo sqlx prepare` 生成 `.sqlx/`. 56.x 看是否要开.
4. **PG 18.6 feature**: 当前 docker-compose 用 postgres:18.6, sqlx 0.8 默认
   feature 集适配 PG 11+, 兼容性 OK. 如未来切 PG 19, 验证 `RETURNING *` + 索引 hint 兼容.

### 6.7 5 域适用清单 (56.x PR check 项)

- [ ] **economy-service**: `resume_concurrent_race_rejected_by_occ` + `saga_crash_recovery_pg_isolation` (RGS-REV-009 V3 H-1 / DC-1)
- [ ] **player-service**: `level_up_atomic_occ` + `currency_grant_transactional`
- [ ] **match-service**: `match_assignment_concurrent_no_double_book`
- [ ] **social-service**: `guild_join_rollback_on_duplicate`
- [ ] **admin-service**: `audit_log_immutable_insert` + `rbac_check_concurrent_role_change`
- [ ] **cluster-ops**: `node_heartbeat_upsert_occ` + `cluster_election_concurrent`

---

## 7. 强约束 (per RGS-REV-009 V3 H-1)

**56.x 起, 所有新增的** saga / 事务 / OCC / outbox 测试**必须**用真 PG
(`#[rgs_testkit::pg_test]`), 禁止 `#[tokio::test]` + InMemoryRepository / `NoopMock`.

根因: RGS-REV-009 V3 审查发现 5 commit 新增 6 个 test 全部是 InMemory unit test,
209 test 全过但 2 个 CRITICAL 没被发现. `InMemoryAccountRepository::apply_atomic`
的 OCC 行为虽真但**需要手动 bump version** 才能触发, 真正 PG 行为 + 事务隔离
+ 并发竞争需要真 PG 才能暴露.

### 唯一接受的 API

- `pg_pool()` —— `rgs_testkit::pg_pool().await` 拿真 `PgPool`
- `pg_test` —— `#[rgs_testkit::pg_test]` 是 `#[sqlx::test]` 的强约束别名

### 拒绝的 API (编译期 `#[deprecated]` 警告)

- `mock::DbMock` / `mock::NoopMock` / `mock_url()`
- `#[tokio::test]` for DB / saga / OCC / outbox test (单元 test 仍可用)
- 手写 `InMemoryAccountRepository::new()` 用作 saga/事务 test fixture

---

## 8. CHANGELOG

### v0.2.0 (2026-08-24) — 文档完善 + 5 域引导
- **新增** `examples/mock_grpc_demo.rs` (GrpcMock trait 占位演示)
- **新增** `examples/mock_nats_demo.rs` (NatsMock trait 占位演示)
- **新增** `examples/fixture_builder_demo.rs` (3 域 fixture 演示)
- **新增** 5 域 crate README 各自加 1 段 "## 测试 (rgs-testkit)" 引导
- **改进** rgs-testkit/README.md: 9 章节结构 + CHANGELOG + 5 域使用示例

### v0.2.0-pre (55.31) — pg_test_db 强约束
- **新增** `pg_test_db` 子模块 (`pg_pool()` / `pg_available()` / `database_url()`)
- **新增** `pub use pg_test_db::pg_pool;` (re-export 强约束入口)
- **新增** `pub use sqlx::test as pg_test;` (强约束别名)
- **新增** `pg-integration` feature (默认关, 保护没 PG 的 CI)
- **修改** `mock::DbMock` / `mock::NoopMock` / `mock_url()` 加 `#[deprecated]` 警告
- **新增** `tests/pg_test_db_smoke_connects_and_selects_one` (PG 连通性 smoke test)
- **新增** 文档: 真 PG 集成测试背景 / API / 默认行为 vs feature / 故障排查

### v0.1.0 (53.3) — 骨架
- 4 模块占位实现 (`mock` / `helper` / `fixture` / `pg_test_db` 占位)
- 3 个 fixture factory (`player` / `economy` / `saga`)
- 11 个 self_test
- 1 个 deprecation example (`test_deprecated_warns.rs`)

---

## 9. 关联

- 父 crate: `crates/rgs-testkit/`
- 规范: RGS-SPEC-000 §2.4 + RGS-IMPL-001 §3
- 强约束: RGS-REV-009 V3 H-1 / WF-1-55.31
- 5 域: DTL-018(player) / DTL-015-016(economy) / DTL-026(match) / DTL-019-020(social) / DTL-031(admin)
- examples: `test_deprecated_warns` (53.3) + `mock_grpc_demo` / `mock_nats_demo` / `fixture_builder_demo` (v0.2.0)
