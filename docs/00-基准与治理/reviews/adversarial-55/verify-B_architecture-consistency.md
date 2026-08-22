# verify-B 工程 55 P0+收尾 architecture consistency 交叉审核

**审核对象**: `git log 7deff16..5ace5ad` (12 commits, 55.15 → 55.21+22)
**审核子代理**: verify-B architecture-consistency-adversarial
**审核时间**: 2026-08-23
**commit 基线**: 5ace5ad
**worktree**: `D:\RustGameServer-worktrees\verify-55-B-arch-consistency`

> 范围勘误：用户原始范围 `8c1dbfd..5ace5ad` 仅 3 commit，与"12 commits"描述不符。
> 实际 12 commit 范围为 `7deff16..5ace5ad`（55.15 → 55.21+22），本审核以此为准。

---

## 1. 严重度统计

- **CRITICAL**: 1（admin-service migration 版本冲突 → 编译期阻断）
- **HIGH**: 0
- **MEDIUM**: 3（MIGRATION_TEMPLATE 漂移 / OutboxRelay 测试覆盖缺口 / 服务间 IDL 未追加共享类型）
- **LOW**: 4（文档引用细微偏差 / 5 域 outbox migration 字段不严格统一 / env 模板无 status 信息 / mTLS fallback 静默降级）

---

## 2. 依赖图审计

| 域 | `shared-platform` path dep | shared-platform 反向依赖 5 域 | 跨域依赖（5 域之间） |
|----|-----------------------------|-------------------------------|---------------------|
| player-service | ✓（`Cargo.toml:31`, commit 5ace5ad）| ✗（shared-platform 仅在 `client.rs::ServiceId`、`subject.rs` 中**字符串枚举**提及 player/economy/match/social/admin/cluster-ops，无 crate 依赖） | ✗（`main.rs:20-25` 仅 `use player_service::*`）|
| economy-service | ✓（`Cargo.toml:31`, commit 5ace5ad）| 同上 | ✗（`main.rs:23-33` 仅 `use economy_service::*`）|
| match-service | ✓（`Cargo.toml:31`, commit 5ace5ad）| 同上 | ✗（`main.rs:20-25` 仅 `use match_service::*`）|
| social-service | ✓（`Cargo.toml:31`, commit 5ace5ad）| 同上 | ✗（`main.rs:20-25` 仅 `use social_service::*`）|
| admin-service | ✓（`Cargo.toml:32-35`, commit 5ace5ad）| 同上 | ✗（`main.rs:20-25` 仅 `use admin_service::*`）|
| cluster-ops | ✓（`Cargo.toml:31`, commit 5ace5ad）| 同上 | ✗（`main.rs:20-26` 仅 `use cluster_ops::*`）|

**结论**：6/6 域全部追加 `shared-platform = { path = "../shared-platform" }` 依赖（55.21+22 wire-up commit 5ace5ad）。shared-platform 通过 `ServiceId` 枚举和 `SubjectBuilder::parse` 知道 6 域名字，但仅作为路由元数据，**无 5 域 crate 反向依赖**。5 域之间**不互相 import**。依赖图符合 ARC-008 5 域独立原则。

---

## 3. OutboxRelay 泛型化影响矩阵

**关键变更**（commit 53a8d37, 55.17）：
- `OutboxRepository::append` 签名升级为 `async fn append<'e, E: PgExecutor<'e>>(&self, entry: &OutboxEntry, executor: E) -> Result<()>`（`outbox.rs:164-168`）
- 因 trait 含泛型方法，**`OutboxRepository` 失去 dyn-safe 性质**（Rust 限制：含泛型方法的 trait 不能 `dyn Trait`）
- `OutboxRelay` 同步改为 `pub struct OutboxRelay<R: OutboxRepository + 'static>`（`outbox_relay.rs:46-51`）

| 调用方 | 旧用法 (Arc<dyn>) | 新用法 (泛型) | 兼容？| 证据 |
|--------|-------------------|---------------|--------|------|
| player-service `main.rs:65` | n/a（55.17 前是 InMemory）| `let outbox_repo: Arc<PgOutboxRepository>` 显式标注 | ✓ | 5ace5ad commit |
| economy-service `main.rs:141` | n/a | `let outbox_repo: Arc<PgOutboxRepository>` 显式标注 | ✓ | 5ace5ad commit |
| match-service `main.rs:64` | n/a | `let outbox_repo: Arc<PgOutboxRepository>` 显式标注 | ✓ | 5ace5ad commit |
| social-service `main.rs:64` | n/a | `let outbox_repo: Arc<PgOutboxRepository>` 显式标注 | ✓ | 5ace5ad commit |
| admin-service `main.rs:63` | n/a | `let outbox_repo: Arc<PgOutboxRepository>` 显式标注 | ✓ | 5ace5ad commit |
| cluster-ops `main.rs:66` | n/a | `let outbox_repo: Arc<PgOutboxRepository>` 显式标注 | ✓ | 5ace5ad commit |
| `shared-platform/src/outbox_relay.rs` 测试 | n/a | `let repo: Arc<InMemoryOutboxRepository>` 显式标注 | ✓ | outbox_relay.rs:177,195 |

**全仓 grep 结果**：`dyn OutboxRepository` / `Box<dyn OutboxRepository>` / `Arc<dyn OutboxRepository>` **零匹配**。所有调用方都使用具体类型 `Arc<PgOutboxRepository>` 或 `Arc<InMemoryOutboxRepository>`，泛型推断无歧义。

**泛型实例化模式**（6 域 main.rs 全部一致）：
```rust
let outbox_repo: Arc<PgOutboxRepository> = Arc::new(PgOutboxRepository::new(pool.clone()));
let relay = OutboxRelay::new(outbox_repo, producer, RelayConfig::default());
tokio::spawn(async move {
    let _nats_keepalive = nats_client;
    Arc::new(relay).run().await;
});
```

**结论**：6/6 域通过显式 `Arc<PgOutboxRepository>` 类型标注触发 `R = PgOutboxRepository` 推断，泛型化升级**无破坏点**。`run(self: Arc<Self>)` 模式（`outbox_relay.rs:110`）正确：调用方需 `Arc::new(relay).run()` 才能将 `self: Arc<Self>` 移交。

---

## 4. mTLS API 一致性矩阵

**关键变更**（commit 8c1dbfd, 55.18）：
- 新增 `pub fn load_server_tls_config(server_cert_path: &Path, server_key_path: &Path, client_ca_cert_path: &Path) -> Result<ServerTlsConfig, TlsError>`（`tls.rs:92-118`）
- 拆分 `build_secure_channel`（默认 mTLS）+ `build_insecure_channel`（显式 opt-out + `mTLS_bypassed_total++`）
- `RpcChannelConfig.require_tls: bool` 默认 `true`（fail-closed，`channel.rs:72`）

| 域 | `load_server_tls_config` 调用 | PEM 路径 | fallback insecure | 模板一致？|
|----|-------------------------------|----------|-------------------|-----------|
| player-service | `main.rs:101-105` | `{tls_dir}/server.pem`, `server.key`, `ca.pem` | ✓ warn + None | ✓ |
| economy-service | `main.rs:177-181` | 同上 | ✓ warn + None | ✓ |
| match-service | `main.rs:100-104` | 同上 | ✓ warn + None | ✓ |
| social-service | `main.rs:100-104` | 同上 | ✓ warn + None | ✓ |
| admin-service | `main.rs:99-103` | 同上 | ✓ warn + None | ✓ |
| cluster-ops | `main.rs:102-106` | 同上 | ✓ warn + None | ✓ |

**全仓 grep 验证**：`load_server_tls_config` 6/6 域使用相同 3-path 模式；`server_builder.tls_config(...)` 配 `if let Some(tls_cfg)` fallback 模板 6/6 一致。

**`RpcChannelConfig.require_tls` fail-closed 验证**（`channel.rs:60-75`）：
- `Default` impl 中 `require_tls: true` ✓
- `build_channel` 行为矩阵（`channel.rs:90-98`）：
  - `tls=Some, _` → mTLS
  - `tls=None, require_tls=true` → `TlsRequired` 错误
  - `tls=None, require_tls=false` → 明文 + warn + counter++

**结论**：mTLS API 接线在 6 域中**模板完全一致**。⚠ 注意：fallback 在生产环境若 cert 文件不存在会**静默降级为 insecure**（仅打 warn + counter++），是显式 opt-out 模式（per RGS-REV-007 CH4 显式 opt-out 原则），但**没有 fail-closed 强校验**（见 LOW-3）。

---

## 5. Migration 命名一致性

| 域 | outbox migration | audit migration | 命名规范? |
|----|------------------|-----------------|----------|
| player-service | `0002_outbox.sql` (53a8d37) | n/a | ✓ 唯一 |
| economy-service | `0003_outbox.sql` (53a8d37) | n/a | ✓ 续号（注释说明：0002 已被 `0002_saga_init.sql` 占用）|
| match-service | `0002_outbox.sql` (53a8d37) | n/a | ✓ 唯一 |
| social-service | `0002_outbox.sql` (53a8d37) | n/a | ✓ 唯一 |
| admin-service | `0002_outbox.sql` (53a8d37) | `0002_audit_prev_hash_unique.sql` (6b3cc5d) | **⚠ 冲突：两个 0002_ 文件**（见 CRITICAL-1）|
| cluster-ops | `0002_outbox.sql` (53a8d37) | n/a | ✓ 唯一 |

**migrator 一致性**（`db.rs:30`）：6 域全部使用 `sqlx::migrate!("./migrations")` 宏（`sqlx 0.8.6`，`Cargo.lock:2556-2557`）。

---

## 6. Trait 一致性

### 6.1 `SagaStepHandler::compensate` 第二参数（55.12）

- `economy-service/src/saga_orchestrator.rs:37-47`：
  ```rust
  pub trait SagaStepHandler: Send + Sync {
      fn name(&self) -> &str;
      async fn execute(&self, saga: &mut Saga) -> Result<()>;
      async fn compensate(&self, saga: &mut Saga, resource_id: Option<Uuid>) -> Result<()>;
  }
  ```
- 实现者：`ReserveHandler::compensate`（line 272）、`ConfirmHandler::compensate`（line 377）、`FailingHandler::compensate`（line 507）、`RecordingHandler::compensate`（line 785）— **4/4 全部更新到 2 参数签名**。
- 调用方：`SagaOrchestrator::compensate`（line 123-141）正确传递 `step.resource_id`。
- **范围**：trait 是 economy-service 私有，**未跨域泄漏**。shared-platform 零匹配 `SagaStepHandler`。

### 6.2 Repository trait 模式

6 域各自定义 `*Repository` trait（player 有 `PlayerRepository` + `PlayerSessionRepository`；economy 有 `AccountRepository` + `TransactionLedgerRepository` + `ReservationRepository` + `SagaRepository`；等等）。所有 5 域 Repository 都用 `Arc<dyn Repository>` 模式（`main.rs:58-60` 等）。

**⚠ 注意**：`OutboxRepository` 是**唯一含泛型方法**的 trait，破坏了 `dyn`-ability。其他 Repository 都用 `Arc<dyn XxxRepository>` 没问题，但 `OutboxRepository` 必须用具体类型。这是局部不一致点，已在第 3 节说明。

### 6.3 `Authorizer` trait

- `shared-platform/src/rbac.rs:128-131`：
  ```rust
  pub trait Authorizer: Send + Sync {
      fn check(&self, subject: &Subject, permission: &str, resource: &str) -> CheckResult;
  }
  ```
- 实现：`SimpleAuthorizer`（`rbac.rs:164-212`），覆盖 5 角色 + scope 校验。
- 55.14 修复（commit 33fca1e）：
  - DomainAdmin 缺 `domain_scope` → 显式 deny（line 172-178）— 旧版 *:* 全权绕过已堵
  - `resource_in_scope` 边界：scope="player" 不匹配 "player_secret/..."（line 222-227）

---

## 7. proto 字段模板一致性

| 域 | proto 文件 | 实体消息 | 字段顺序 |
|----|------------|----------|----------|
| player-service | `proto/player/v1/player.proto` | `Player` | `id (1) + status (2) + created_at (3) + display_name (4)` |
| economy-service | `proto/economy/v1/economy.proto` | `Account` | 同上 |
| match-service | `proto/match/v1/match.proto` | `Match` | 同上 |
| social-service | `proto/social/v1/social.proto` | `Guild` | 同上 |
| admin-service | `proto/admin/v1/admin.proto` | `AdminOp` | 同上 |
| cluster-ops | `proto/cluster_ops/v1/cluster_ops.proto` | `Node` | 同上 |

6/6 proto 全部使用 `id / status / created_at / display_name` 4 字段模板（per 54.x 模板）。**55.x 范围未触动 proto**（仅服务侧 mTLS / outbox / saga 逻辑）。

---

## 8. DTL / RGS-REV / DEC 追溯

| 文档/代码位置 | 引用 | 状态 |
|--------------|------|------|
| `shared-platform/src/outbox.rs:1` | `RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005` | ✓ |
| `shared-platform/src/outbox.rs:4` | `RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1` | ✓ |
| `shared-platform/src/tls.rs:81` | `RGS-REV-007 CH4 + DEC-015 P1` | ✓ |
| `shared-platform/src/rbac.rs:1` | `RGS-DTL-019 §3 + DEC-005 + ARC-051` | ✓ |
| `shared-platform/src/rbac.rs:168` | `RGS-REV-007 AC6=CM5 / DEC-015 P1`（DomainAdmin 缺 scope 显式 deny）| ✓ |
| `shared-platform/src/rbac.rs:216` | `RGS-REV-007 AC6 / DEC-015 P1`（scope 边界）| ✓ |
| `economy-service/src/saga.rs:1` | `RGS-DTL-100 Saga Q-003` | ✓ |
| `economy-service/src/saga_orchestrator.rs:1` | `RGS-DTL-100 §3-§5` | ✓ |
| `admin-service/src/entity.rs:3` | `RGS-DTL-019 §3 + ARC-051` | ✓ |
| `admin-service/src/entity.rs:8` | `RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1`（hash 升级）| ✓ |
| `admin-service/migrations/0002_audit_prev_hash_unique.sql:1` | `RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1` | ✓ |

**用户预期 vs 实际**：
- 用户说 "55.17 shared-platform/outbox.rs 引用 DTL-100 §3 §4 §5" — 实际只引用 §5.3（outbox 主题本身只关心 §5.3；§3 状态机、§4 补偿在 `economy-service/src/saga_orchestrator.rs` 引用）。这是**模块化切分**，不算错，但**outbox.rs 文件头注释可补充一句"§3 状态机见 saga_orchestrator.rs"**。→ LOW-2

---

## 9. CRITICAL Issues

### BC-1. admin-service migration 版本号重复（编译期阻断）

- **位置**：`crates/admin-service/migrations/`
  - `0001_init.sql`（commit f3cfb2c, 54.4）
  - `0002_audit_prev_hash_unique.sql`（commit 6b3cc5d, 55.13）
  - `0002_outbox.sql`（commit 53a8d37, 55.17）
- **问题**：`admin-service` 有**两个** `0002_` 前缀文件，违反 sqlx 迁移版本号唯一性约束。
  - sqlx 0.8.6（`Cargo.lock:2556`）的 `sqlx::migrate!("./migrations")` 宏在**编译期**展开 `Migrator::new`（`sqlx-macros`），若发现两个文件提取出同一版本号 `0002`，会**直接 panic/编译错误**。
  - 经济域通过 `0003_outbox.sql` 正确避开了 0002（`0002_saga_init.sql` 占用），并写明注释"注：经济域已有 0002_saga_init.sql（per WF-1-55.1），本迁移用 0003 续号"（`economy-service/migrations/0003_outbox.sql:2`）。
  - **但 55.17 在 admin 域添加 outbox 时未检查 0002 是否已被 audit 占用**，未延续 economy 域的"续号"纪律。
- **影响**：
  - `admin-service` 的 `sqlx::migrate!("./migrations")`（`db.rs:30`）**无法编译**。
  - 整个 `cargo build -p admin-service` 失败，进而 `cargo build --workspace` 失败。
  - 部署 admin 域的 CI pipeline 必然失败。
- **修复方案**（互斥二选一）：
  - **方案 A**（推荐，向后兼容）：将 `0002_outbox.sql` 重命名为 `0003_outbox.sql`（沿用 economy 续号模式），并加注释说明。
  - **方案 B**：将 `0002_audit_prev_hash_unique.sql` 重命名为 `0003_audit_prev_hash_unique.sql`（但 0002_outbox 是 55.17 新加，应让 audit 移走）。
- **阻塞阶段**：PR 合并即阻断，必须修复后才能合入 main。

---

## 10. HIGH Issues

无。

---

## 11. MEDIUM Issues

### BC-2. shared-platform MIGRATION_TEMPLATE 漂移（维护性）

- **位置**：`crates/shared-platform/src/outbox.rs:464-489`（MIGRATION_TEMPLATE 常量）
- **问题**：
  - shared-platform 暴露 `pub const MIGRATION_TEMPLATE: &str`，注释说"54.11 模板：各域 migrations 应包含本表"。
  - **全仓零调用**：`grep MIGRATION_TEMPLATE` 仅在 `lib.rs:65`（pub use）和 `outbox.rs:464`（定义）出现，**6 域 migration 没有任何一个 include 或复制该模板**。
  - 字段定义已**漂移**：
    | 字段 | MIGRATION_TEMPLATE | 6 域 outbox migration 实际 |
    |------|--------------------|----------------------------|
    | subject | `TEXT NOT NULL`（无限长）| `VARCHAR(256) NOT NULL`（5 域限制 256）|
    | payload | `TEXT NOT NULL` | `JSONB NOT NULL`（5 域均用 JSONB）|
    | status | `TEXT NOT NULL DEFAULT 'pending' CHECK (...)` | `VARCHAR(16) NOT NULL DEFAULT 'pending'`（**无 CHECK 约束**）|
- **影响**：
  - 模板与现实分叉，未来若有人按 MIGRATION_TEMPLATE 生成新域 migration，会出现 status 无 CHECK、字段类型不一致问题。
  - 共享库应有"单一真相"，当前是"参考文档已脱钩"。
- **修复方案**：
  - 将 MIGRATION_TEMPLATE 字段类型对齐 6 域现状（subject VARCHAR(256), payload JSONB, status VARCHAR(16) + CHECK）
  - 在 6 域 migration 文件加注释"per shared-platform/src/outbox.rs::MIGRATION_TEMPLATE"建立引用
  - 考虑加一个 `#[test]` 验证 MIGRATION_TEMPLATE 与 PgOutboxRepository::append 的字段一致
- **阻塞阶段**：非 PR 阻断，但应在下个迭代清理。

### BC-3. OutboxRelay 泛型化的测试覆盖缺口

- **位置**：`crates/shared-platform/src/outbox_relay.rs:157-227`（tests 模块）
- **问题**：
  - 55.17 把 `OutboxRelay` 改成 `OutboxRelay<R: OutboxRepository + 'static>`，但 **测试模块没有覆盖 `OutboxRelay::new` 实际调用路径**。
  - `relay_tick_empty`（line 175-184）只测了 `repo.list_pending(10)`，**没有构造 OutboxRelay**。
  - `relay_uses_in_flight_state`（line 192-227）同上，**只测了 repo，未测 relay**。
  - 整文件测试中**没有任何 `OutboxRelay::new(...)` 调用**，因此：
    - 泛型推断 `R = InMemoryOutboxRepository` 实际可行的路径**未被 CI 覆盖**
    - `OutboxRelay<R>` 的 `run(self: Arc<Self>)` 方法（line 110）也未在 test 中调用
- **影响**：
  - 6 域 main.rs 调用的 `OutboxRelay::new(outbox_repo, producer, RelayConfig::default())` 这条路径没有 unit test 守护。
  - 未来若 OutboxRelay 内部字段调整（例如加 `Arc<Tracer>`），CI 不会立即发现断点。
- **修复方案**：
  - 增加 `relay_runs_one_tick_with_inmemory_repo` 集成 test：构造 `OutboxRelay<InMemoryOutboxRepository>` → 调 `tick()` → 验证 empty。
  - 增加 `relay_publishes_to_producer` test（用 mock producer 替代真 NATS）。
- **阻塞阶段**：非 PR 阻断，但应在下个 sprint 补上。

### BC-4. shared-platform 缺乏跨域 IDL 共享类型

- **位置**：`crates/shared-platform/proto/common/v1/common.proto`
- **问题**：
  - 当前 `common.proto` 提供 `EntityId` / `Status` / `Timestamp` 基础类型。
  - **6 域 `id / status / created_at / display_name` 模板下，每域还自己定义了 `display_name: string` 字段**（如 `Player.display_name`, `Account.display_name`, `Match.display_name`...），但显示名格式、最长限制、是否必填、是否唯一在 6 域各自实现。
  - 如果未来要做"全服玩家搜索"等跨域 RPC，需要先统一 `display_name` 约束（最大长度、字符集、是否 trim/normalize）。
- **影响**：
  - 跨域一致性未在 IDL 层强制。
  - 6 域 `display_name` 各自处理，可能出现某个域允许 256 字符另一个域限制 32 字符的不一致。
- **修复方案**：
  - 在 `common.proto` 加 `DisplayName` 共享 message（如 `string DisplayName { max_len: 64; pattern: "^[\\w\\- ]+$" }`），6 域引用。
  - 或在 DTL / ARC-051 文档化"display_name 6 域统一约束"。
- **阻塞阶段**：非 PR 阻断，但属于架构债务。

---

## 12. LOW Issues

### BC-5. outbox.rs 注释未引用 DTL-100 §3 / §4

- **位置**：`crates/shared-platform/src/outbox.rs:1`
- **问题**：注释说"per RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005"，但用户预期"55.17 引用 DTL-100 §3 §4 §5"。§3（状态机）和 §4（补偿）实际由 economy/saga_orchestrator.rs 引用。outbox 自身只关心 §5.3（事务性消息），无需修改 DTL 引用。
- **建议**：在 outbox.rs 顶部加 1 行交叉引用"状态机见 economy-service::saga_orchestrator.rs (per DTL-100 §3)"。
- **影响**：注释精度，不影响功能。

### BC-6. 6 域 outbox migration 字段定义未严格统一

- **位置**：
  - 5 域 `0002_outbox.sql` / `0003_outbox.sql`：`subject VARCHAR(256) NOT NULL`, `payload JSONB NOT NULL`, `status VARCHAR(16) NOT NULL DEFAULT 'pending'`（**无 CHECK 约束**）
  - MIGRATION_TEMPLATE：`subject TEXT NOT NULL`, `payload TEXT NOT NULL`, `status TEXT NOT NULL DEFAULT 'pending' CHECK (...)`（有 CHECK 但用 TEXT）
- **问题**：6 域实际表无 status CHECK 约束，意味着应用层如果误写 `status='PANIC'`（大写）也会成功入库，破坏状态机不变量。
- **影响**：
  - 业务层 bug 会污染数据库。
  - 与 MIGRATION_TEMPLATE 设计意图不符（template 有 CHECK）。
- **建议**：在 6 域 outbox migration 加 `CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))`，对齐 MIGRATION_TEMPLATE。

### BC-7. .env.example 缺 NATS / RGS_TLS_DIR 状态信息

- **位置**：`.env.example`（commit 9e55bbe, 55.20）
- **问题**：55.20 把 6 域密码 + superuser 全部 7 个独立化为 `CHANGE_ME_*_PASSWORD` 占位符，并通过 `scripts/generate_dev_passwords.ps1` 自动生成。但 .env.example 没有：
  - `NATS_URL_LOCAL` 的 `tls://` 模式提示（55.21+22 之后 gRPC 强制 mTLS，dev 模式也应配套）
  - `RGS_TLS_DIR` 默认路径的注释（main.rs 默认 `/etc/rgs/certs`，但 dev 通常用相对路径）
  - `require_tls=false` 显式 opt-out 的环境变量说明
- **影响**：dev onboarding 文档略不完整，团队成员可能错过 mTLS 启用条件。

### BC-8. mTLS fallback 静默降级为 insecure

- **位置**：6 域 main.rs 的 `load_server_tls_config` 调用块（`if let Some(tls_cfg) = tls_config { ... } else { warn!("⚠ mTLS DISABLED") }`）
- **问题**：55.21 设计是"显式 opt-out"模式（per RGS-REV-007 CH4），但 fallback 触发条件是**PEM 文件读取失败**，这在生产环境通常是**误配置**而非有意 opt-out。当前 fallback 仅 `tracing::warn!`，没有 fail-fast 或 require 显式环境变量 `RGS_REQUIRE_TLS=true`。
- **影响**：
  - 生产误配置（如 volume mount 错）会**静默降级为明文 gRPC**，仅靠日志告警。
  - `mtls_bypassed_total` 计数器（`channel.rs:82`）只跟踪 client 端 `require_tls=false`，**不跟踪 server 端 fallback**。
- **建议**：
  - 加 `RGS_REQUIRE_TLS` 环境变量（默认 `true`），仅在 `true` 时允许 fallback 降级。
  - 加 `mTLS_bypassed_total` 计数器也覆盖 server 端 fallback（`ServerTlsConfig` 缺失路径）。
  - 或在 `RGS_ENV=production` 时硬性 fail-fast。

---

## 13. 修复优先级矩阵

| Issue | 严重度 | 文件 | 估时 | 阻塞阶段 |
|-------|--------|------|------|----------|
| **BC-1** migration 重复 | **CRITICAL** | `admin-service/migrations/0002_outbox.sql` | 5 分钟（rename + 注释）| **PR 合并即阻断**（编译期）|
| BC-2 MIGRATION_TEMPLATE 漂移 | MEDIUM | `shared-platform/src/outbox.rs:464-489` + 6 域 outbox migration | 1 小时（对齐字段 + 加引用注释）| 下个 sprint（55.25+） |
| BC-3 OutboxRelay 测试缺口 | MEDIUM | `shared-platform/src/outbox_relay.rs:157-227` | 2 小时（加 2-3 个 unit test）| 下个 sprint（55.25+）|
| BC-4 跨域 display_name IDL | MEDIUM | `shared-platform/proto/common/v1/common.proto` + 6 域 proto | 4 小时（协商 + 6 域同步迁移）| Q3 末尾决策 |
| BC-5 DTL 引用注释 | LOW | `shared-platform/src/outbox.rs:1` | 5 分钟 | 不阻塞 |
| BC-6 status CHECK 约束 | LOW | 6 域 outbox migration | 30 分钟（加 CHECK 约束）| 不阻塞（runtime 防御） |
| BC-7 .env.example 文档 | LOW | `.env.example` | 15 分钟 | 不阻塞 |
| BC-8 server-side fail-fast | LOW | 6 域 main.rs + channel.rs | 4 小时（加 RGS_REQUIRE_TLS + counter）| Q3 安全审计项 |

---

## 14. 审计员签注

<审计员>: verify-B (architecture-consistency-adversarial)
<签名>: <占位 — 待最终 commit>
<worktree>: `D:\RustGameServer-worktrees\verify-55-B-arch-consistency`
<commit 范围>: `7deff16..5ace5ad` (12 commits, 55.15 → 55.21+22 P0+收尾)
<DTL 追溯样本>: DTL-019 / DTL-100 / ARC-051 / DEC-005 / DEC-015 / RGS-REV-007 §3.5/§4/§5/§6/CH1/CH2/CH3/CH4/AC2/AC3/AC4/AC5/AC6/CM5/AH1/M6 / RGS-SPEC-CROSS-002/005/006 / RGS-SEC-100 §7 / RGS-DEC-018 M6-A

**审核局限性**:
- 仅静态代码阅读 + git log/diff，未跑 `cargo build` / `cargo clippy` / 集成测试
- 未验证 sqlx 0.8.6 在 0002 collision 下的**具体错误信息**（虽然 sqlx 文档明确声明"version numbers must be unique"）
- 未跑 `cargo metadata --no-deps` 验证依赖图闭环
- 未读 `rgs-certgen` 和 `rgs-testkit` 的全部源码（仅看 commit 提到的范围）
- 未审计 outbox `InMemory` 实现与 `Pg` 实现在并发场景下的语义对齐（仅看代码，未跑 stress test）

**未涵盖**:
- 实际部署 / k3s 集成测试
- NATS JetStream 真实连接验证
- mTLS 证书签发 / 轮转流程
- 性能压测（OutboxRelay poll 频率 vs DB 负载）
- Saga orchestrator 的崩溃恢复实际行为

**审计结论**: **FAIL**（因 CRITICAL-1 admin-service migration 0002 冲突导致编译阻断）

---

## 附录 A: 12 commits 详细清单

| # | commit | 标题 | 文件数 | 关键变更 |
|---|--------|------|--------|----------|
| 1 | 7deff16 | 55.15 5 域+cluster-ops main.rs InMemory→Pg 接线 | 多 | Pg 接入 6 域 main |
| 2 | 87890ef | 55.15 merge | merge | 合并 |
| 3 | 69ebcd1 | 55.16 client_interceptor trace_id 从 Span 提取 | 1 (grpc_tracing.rs) | OTel trace_id 桥接 |
| 4 | b488f3a | 55.16 merge | merge | 合并 |
| 5 | 33fca1e | 55.14 RBAC DomainAdmin 缺 scope 显式 deny + scope 边界修复 | 1 (rbac.rs) | RBAC 安全修复 |
| 6 | 9e55bbe | 55.20 dev 密码 6 域独立化 | 3 (.env.example + script + doc) | 7 独立密码 |
| 7 | 6b3cc5d | 55.13 audit_log FNV-1a → SHA-256 + 事务 | 1+ (admin entity.rs + 0002 migration) | hash 升级 |
| 8 | d8d33cf | 55.12 SagaOrchestrator handler 实化 | 1 (saga_orchestrator.rs) | Saga 步进实化 |
| 9 | 53a8d37 | 55.17 outbox SKIP LOCKED + 事务边界 + 6 域 migration | 8 (outbox.rs + outbox_relay.rs + 6 migration) | outbox 升级 |
| 10 | 8c1dbfd | 55.18 mTLS client_auth_required 实化 | 4 (channel.rs + client.rs + lib.rs + tls.rs) | mTLS 强制校验 |
| 11 | 421585c | 55.23 economy main.rs SagaOrchestrator 接线 | 1 (economy main.rs) | Saga 收尾 |
| 12 | 465bfeb | 55.24 housekeeping pre-existing doctest + clippy | 2 (rgs-testkit + json_logging) | 小修 |
| (额外) | 5ace5ad | 55.21+22 5 域 main.rs mTLS + outbox 接线 | 6 域 main.rs | 当前 HEAD |

> 注：用户原范围 `8c1dbfd..5ace5ad` 实际只覆盖 #10~12 + HEAD = 3 commit。
> 本审核按 "12 commits" 描述扩展到 `7deff16..5ace5ad`（含 55.15 P0 起点）。

---

## 附录 B: 关键证据文件路径

- 依赖图：`crates/{player,economy,match,social,admin}-service/Cargo.toml` + `crates/cluster-ops/Cargo.toml` + `crates/shared-platform/Cargo.toml`
- Outbox API：`crates/shared-platform/src/outbox.rs` + `crates/shared-platform/src/outbox_relay.rs`
- 6 域 main.rs：`crates/{player,economy,match,social,admin}-service/src/main.rs` + `crates/cluster-ops/src/main.rs`
- mTLS API：`crates/shared-platform/src/tls.rs:92-118` + `crates/shared-platform/src/channel.rs:43-127`
- RBAC：`crates/shared-platform/src/rbac.rs`
- audit_log：`crates/admin-service/src/entity.rs` + `crates/admin-service/migrations/0001_init.sql` + `crates/admin-service/migrations/0002_audit_prev_hash_unique.sql`
- proto 模板：`crates/{player,economy,match,social,admin}-service/proto/*/v1/*.proto` + `crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto` + `crates/shared-platform/proto/common/v1/common.proto`
- 收尾配置：`.env.example`（commit 9e55bbe）+ `scripts/generate_dev_passwords.ps1`