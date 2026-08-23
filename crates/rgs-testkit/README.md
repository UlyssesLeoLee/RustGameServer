# rgs-testkit — RustGameServer 测试套件骨架

> 4 大子模块: `mock` / `helper` / `fixture` / `pg_test_db`
> 规范: RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4

| 模块 | 用途 | 何时引入 |
|---|---|---|
| `mock` | DB / gRPC / NATS mock trait | 53.3 (InMemory 占位) |
| `helper` | config 加载 / tracing 初始化 / assert helper | 53.3 |
| `fixture` | sample data + init_test_db 占位 | 53.3 |
| `pg_test_db` | **真 PG 测试 fixture (DATABASE_URL + #[sqlx::test])** | **55.31 (RGS-REV-009 V3 H-1 共识)** |

---

## `pg_test_db` 子模块 (RGS-REV-009 V3 H-1)

### 背景 (为什么需要)

RGS-REV-009 V3 审查发现: 5 commit 新增 6 个 test 全部是 InMemory unit test, 209
test 全过但 2 个 CRITICAL 没被发现. 根因: `InMemoryAccountRepository::apply_atomic`
的 OCC 行为虽真但**需要手动 bump version** 才能触发, 真正 PG 行为 + 事务隔离
+ 并发竞争需要真 PG 才能暴露.

55.31 任务 (HI-2-pg) 在 rgs-testkit 加 `pg_testDatabase` fixture, **强制 56.x 起
新代码用 `#[sqlx::test]` 写真 DB 集成测试**, 防 '209 test pass ≠ correct'
假象复发.

### API 概览

```rust
use rgs_testkit::pg_test_db::{pg_pool, pg_available, DATABASE_URL_ENV, DEFAULT_POOL_SIZE};

// 1) 探测 PG 是否可达 (用于 test 内 gate, 无 DATABASE_URL 或连不上均返回 false)
let ok: bool = pg_available().await;

// 2) 拿 PgPool (Err 时返回 anyhow::Error)
let pool: sqlx::PgPool = pg_pool().await?;
```

### 默认行为 vs `--features pg-integration`

| 跑法 | 是否编 PG 集成 test | 是否需 DATABASE_URL | 何时用 |
|---|---|---|---|
| `cargo test` | ❌ 不编 | ❌ 不需要 | CI 默认 / 离线开发 |
| `cargo test --features pg-integration` | ✅ 编译 | ✅ 需要 (env var) | PR 合并前 / 56.x 起 saga 改动 |
| `cargo test -p rgs-testkit --features pg-integration -- --include-ignored` | ✅ 跑 `pg_test_db_smoke_connects_and_selects_one` | ✅ | 本地 smoke 验 |

`pg-integration` feature 默认关, 是为了**保护 CI**: 没有 PG 的环境跑 `cargo test`
时, 集成 test 连代码都不编, 不会因为缺 `DATABASE_URL` 报错.

### 域 crate 集成 test 写法 (推荐)

**适用**: 56.x 起所有 saga / 事务 / OCC / 并发相关 test.

```rust
// crates/economy-service/tests/pg_integration_resume.rs (新增)
use rgs_testkit::pg_test_db::pg_pool;
use sqlx::PgPool;

#[sqlx::test]
async fn resume_concurrent_race_rejected_by_occ(pool: PgPool) {
    // 真 PG OCC: UPDATE ... WHERE version = ?  0 row → 拒绝
    // 比 InMemory `stored.version = original_version + 99` 模拟更真
    let repo = economy_service::PgAccountRepository::new(pool.clone());
    // ... 准备 account + reservation
    // ... 2 个并发 resume(同 saga_id) 触发竞争
    // ... 断言: 仅 1 个成功, 另一个收到 OCC conflict
}

#[sqlx::test]
async fn saga_crash_recovery_uses_real_transaction_isolation(pool: PgPool) {
    // 测 PG Read Committed 隔离级下, saga_orchestrator::resume 的可见性
    // ... 跨事务 prepare + commit, 验 30s 轮询能 resume
}
```

**为什么必须真 PG**:

1. **OCC 真行为**: `UPDATE accounts SET version = version + 1 WHERE id = ? AND version = ?`
   0 row → PG 返回 `rows_affected = 0`, 应用层拒绝. InMemory 需手动 `stored.version = original + 99`,
   跳过 UPDATE 路径, 漏测 PG `RETURNING` / 约束触发器 / index deadlock 等场景.
2. **事务隔离**: PG 默认 Read Committed, 跨事务 SELECT 可见性有定义. InMemory
   单线程 `Mutex<Vec<...>>` 无并发语义.
3. **连接池耗尽 / 重试**: `PgPoolOptions::max_connections` + sqlx 内部 retry 在
   InMemory 不存在.
4. **约束**: CHECK / FK / UNIQUE 由 PG enforce, InMemory 需手动 `if x.balance < 0 { panic!() }`,
   漏测 PG 实际错误码 (e.g. `23514 check_violation`).

### 启 PG (本地 / CI)

#### 本地 (Windows + Docker Desktop)

```powershell
# 1. 启 Docker Desktop (UI 启 或 管理员 PowerShell: Start-Service docker)
Start-Service docker   # 若失败, 用 Docker Desktop UI 启

# 2. 启 6 独立 PG 容器 (per ARC-008, 5 域 + cluster_ops)
cd D:\wf-1-55-31
# 复制 .env.compose.example → .env.compose, 填密码
Copy-Item docker/compose/.env.compose.example docker/compose/.env.compose
# 编辑 .env.compose 填 6 个 *_DB_PASSWORD
docker compose -f docker/compose/docker-compose.yml --env-file docker/compose/.env.compose --profile dev up -d player-db economy-db match-db social-db admin-db cluster-ops-db

# 3. 等 healthcheck 通过
docker compose -f docker/compose/docker-compose.yml ps   # 6 DB 状态 healthy

# 4. 注入 DATABASE_URL (每个域一个; smoke test 用任意一个)
$env:DATABASE_URL = "postgres://player_user:<password>@localhost:15432/player_db"
# 或用 economy: $env:DATABASE_URL = "postgres://economy_user:<password>@localhost:15433/economy_db"

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

### 故障排查

| 症状 | 原因 | 修法 |
|---|---|---|
| `DATABASE_URL env var not set` | env 没注 | `export DATABASE_URL=...` |
| `connection refused` | PG 容器没启 / 端口错 | `docker compose ps` 查 healthcheck |
| `password authentication failed` | 密码错 / 用户错 | 查 `.env.compose` + 容器内 `psql` 验 |
| `database "xxx_db" does not exist` | 容器用 `POSTGRES_DB` 自动建库, 名错 | 对齐 env var `*_DB_NAME` |
| 跑 `cargo test --features pg-integration` 仍不编 PG test | Cargo feature cache 旧 | `cargo clean -p rgs-testkit` + 重跑 |
| `#[sqlx::test]` 报 `no DATABASE_URL set` | sqlx 0.8 内部强制 | 设 `DATABASE_URL=...` env var |
| `#[sqlx::test]` 报 `query syntax error at compile time` | sqlx offline mode 启用 | 本工作区未启用, 忽略; 如启用, `cargo sqlx prepare` |

### 已知限制 (56.x 演进项)

1. **无 testcontainers-rs**: Docker Desktop 启停开销大 (10-20s), 当前方案依赖
   CI 预启 docker compose. 56.x 可考虑加 `testcontainers` 后备 (每 test 一个临时容器).
2. **5 域独立 DB → 5 个 DATABASE_URL**: 本 fixture 只暴露 `DATABASE_URL` 默认入口.
   5 域 crate 各自需要 `DATABASE_URL_<DOMAIN>` (e.g. `DATABASE_URL_ECONOMY`) 的
   wrapper, 56.x 加 `pg_test_db::pg_pool_for_domain("economy")` 之类 API.
3. **sqlx offline mode 未启用**: 本 fixture 运行时连 PG, 离线 build 需先
   `cargo sqlx prepare` 生成 `.sqlx/`. 56.x 看是否要开.
4. **PG 18.6 feature**: 当前 docker-compose 用 postgres:18.6, sqlx 0.8 默认
   feature 集适配 PG 11+, 兼容性 OK. 如未来切 PG 19, 验证 `RETURNING *` + 索引 hint 兼容.

### 5 域适用清单 (56.x PR check 项)

- [ ] **economy-service**: `resume_concurrent_race_rejected_by_occ` + `saga_crash_recovery_pg_isolation` (RGS-REV-009 V3 H-1 / DC-1)
- [ ] **player-service**: `level_up_atomic_occ` + `currency_grant_transactional`
- [ ] **match-service**: `match_assignment_concurrent_no_double_book`
- [ ] **social-service**: `guild_join_rollback_on_duplicate`
- [ ] **admin-service**: `audit_log_immutable_insert` + `rbac_check_concurrent_role_change`
- [ ] **cluster-ops**: `node_heartbeat_upsert_occ` + `cluster_election_concurrent`

---

## 版本历史

| 版本 | 日期 | 变更 |
|---|---|---|
| 0.1 (53.3) | 2026-08 | 3 子模块骨架 (mock/helper/fixture) |
| 0.2 (55.31) | 2026-08-23 | **+ pg_test_db 子模块 (RGS-REV-009 V3 H-1)**, `pg-integration` feature 默认关, fixture 提供 `pg_pool()` / `pg_available()` / `database_url()` + 文档 |
