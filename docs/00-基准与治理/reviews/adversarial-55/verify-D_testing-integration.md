# verify-D 工程 55 P0+收尾 testing + integration 交叉审核

**审核对象**: 工程 55 P0+收尾 12 L4 commit（git rev-list 不连续，覆盖 6 P0 + 收尾 3 + 收尾 1 housekeeping + 1 dev）
**审核子代理**: verify-D testing-integration-adversarial
**审核时间**: 2026-08-22
**commit 基线**: 5ace5ad（main）
**worktree**: D:\RustGameServer-worktrees\verify-55-D-testing

---

## 0. 范围注释

任务说 "git log 8c1dbfd..5ace5ad 这 12 个 commit" — 实际 `8c1dbfd..5ace5ad` 仅 3 commit
（55.21+22, 55.23, 55.24 — 即收尾 3 个），但任务列出 12 L4 项覆盖 55.P0 全集 + 收尾 + housekeeping。
本审核覆盖 12 L4 项的 testing + integration 角度（commit 范围 = `7deff16..5ace5ad`
含 2 个 merge commit，共 13 commit）：
- 55.15 (7deff16), 55.15 merge (87890ef)
- 55.16 (69ebcd1), 55.16 merge (b488f3a)
- 55.14 (33fca1e), 55.20 (9e55bbe), 55.13 (6b3cc5d)
- 55.12 (d8d33cf), 55.17 (53a8d37), 55.18 (8c1dbfd)
- 55.23 (421585c), 55.24 (465bfeb), 55.21+22 (5ace5ad)

---

## 1. 严重度统计

| 严重度 | 数量 | 备注 |
|--------|------|------|
| **CRITICAL** | 1 | 关键恢复路径完全无测试（SagaOrchestrator::resume） |
| **HIGH** | 3 | mTLS 真实 PEM 加载 / OutboxRelay::run / audit_log chain verify 函数缺失 |
| **MEDIUM** | 4 | 边界测试不全（amount=0, Uuid::nil）/ dev fallback 未测 / RBAC 空角色 / 30s 轮询 race |
| **LOW** | 2 | 注释 vs 代码一致性 / 文档测试用例命名 |

---

## 2. 测试覆盖矩阵

| L4 | commit | 新增 unit test | 关键路径覆盖 | 关键缺失 |
|----|--------|----------------|--------------|----------|
| 55.12 | d8d33cf | **+9** (7 saga_orch + 2 service) | reserve persist + 原子扣减 + confirm 标记 + 补偿退款 + 端到端 + 余额不足 dangling 清理 | **缺: SagaOrchestrator::resume()（崩溃恢复主路径，saga_orchestrator.rs:150）** |
| 55.13 | 6b3cc5d | **+3** (entity 2 + service 1) | SHA-256 长度前缀 + separator 碰撞 + atomic latest append | **缺: `verify_chain()` 函数不存在**（admin-service 整个无 chain verifier；grep 0 match）；只能加 1000 随机输入测试，未测端到端 append→verify |
| 55.14 | 33fca1e | **+6** | DomainAdmin 缺 scope 显式 deny + scope 边界（player/123 vs player_secret/1）+ SuperAdmin 不受影响 + resource_in_scope helper | 缺: `roles = vec![]` 空角色 edge case |
| 55.15 | 7deff16 | +0 (refactor) | 5 域 + cluster-ops main.rs 切到 PgRepository | 缺: PgRepository 真实 DB 集成测试（无 PG 实例） |
| 55.16 | 69ebcd1 | **+3** | OTel fallback + 每次新 UUID + SpanId 8→16 字节 0-pad | 缺: 端到端 gRPC request trace 验证（client→server 串联） |
| 55.17 | 53a8d37 | **+6** (5 outbox + 1 outbox_relay) | SKIP LOCKED + 事务化 + lease 过期重试 + in_flight 状态机 | **缺: `OutboxRelay::run()`（outbox_relay.rs:110）无限循环未做 integration test** |
| 55.18 | 8c1dbfd | **+10** (3 channel + 4 client + 3 tls) | mTLS fail-closed + bypass counter +1 + TLS 路径错误分类 + build_insecure_channel warn + service 域默认端口 | **缺: 真实 PEM 加载测试**（仅 missing file error path；worker 报告"未做"已确认） |
| 55.20 | 9e55bbe | +0 (非 Rust) | .env + 6 域独立密码 + scripts/generate_dev_passwords.ps1 | 缺: PowerShell 脚本单测（无 test infra） |
| 55.21+22 | 5ace5ad | +0 (refactor) | 5 域 + cluster-ops main.rs mTLS + outbox 接线 | **缺: dev fallback insecure gRPC 路径在 main.rs 级别的端到端测试**（仅在 channel.rs 单元测了 build_insecure_channel，5 域 main.rs 的 match load_server_tls_config() 逻辑无 test） |
| 55.23 | 421585c | +0 (refactor) | economy main.rs SagaOrchestrator + 30s 轮询 + resume | **缺: 30s recover 轮询 race（多实例并发 resume 同一 saga）** |
| 55.24 | 465bfeb | +2 doctest | rgs-testkit mock.rs + shared-platform json_logging.rs | OK |

**新增测试总数**: 9 + 3 + 6 + 3 + 6 + 10 = **37 新 unit test** + 2 doctest fix
（任务说"55.12 SagaOrchestrator handler 7 新测试" = saga_orchestrator.rs 部分，
但 55.12 实际还多 2 个 service.rs 测试 = 总 9）

---

## 3. 跨域集成审计

### 3.1 Cargo.toml 一致性

| 域 | shared-platform dep | path | features |
|----|---------------------|------|----------|
| admin-service | ✓ | `../shared-platform` | (none) |
| cluster-ops | ✓ | `../shared-platform` | (none) |
| economy-service | ✓ | `../shared-platform` | (none) |
| match-service | ✓ | `../shared-platform` | (none) |
| player-service | ✓ | `../shared-platform` | (none) |
| social-service | ✓ | `../shared-platform` | (none) |

**Cargo.lock**: 55.21+22 同步更新 6 个 `shared-platform` 引用 — workspace 一致 ✓

### 3.2 跨域 main.rs shared-platform import 一致性

所有 6 域 + cluster-ops main.rs 共享完全相同的 5 行 import：
```rust
use shared_platform::messaging::{build_messaging_client, MessagingConfig};
use shared_platform::outbox::PgOutboxRepository;
use shared_platform::outbox_relay::{OutboxRelay, RelayConfig};
use shared_platform::producer::{Producer, ProducerConfig};
use shared_platform::tls::load_server_tls_config;
```
**6 域 + cluster-ops = 7/7 完全一致** ✓

### 3.3 OutboxRelay 泛型实例化

```rust
let outbox_repo: Arc<PgOutboxRepository> = Arc::new(PgOutboxRepository::new(pool.clone()));
let relay = OutboxRelay::new(outbox_repo, producer, RelayConfig::default());
```
所有 6 域 + cluster-ops main.rs 都按相同模式实例化 ✓ — 编译通过（cargo build 0 error）。

### 3.4 mTLS cert 路径跨域共享

`RGS_TLS_DIR` env var 6 域统一，fallback 默认 `/etc/rgs/certs` ✓
但 **fallback insecure gRPC 路径无 test 覆盖**（dev/test 环境下 PEM 缺失 → `tracing::warn!` + 走无 TLS），
只在 `channel.rs::build_insecure_channel_emits_warning` 测了 client 端，
5 域 main.rs 的 `match load_server_tls_config() { Ok => Some(cfg), Err => None }` 逻辑无 test。

### 3.5 SagaOrchestrator 可见性

```rust
// crates/economy-service/src/lib.rs
pub mod saga_orchestrator;
```
**SagaOrchestrator 是 economy-service 内部模块，其他 5 域不可见** ✓ — 这与 DTL-100 §3
"经济域独占 Saga 编排" 设计一致，无跨域耦合。

---

## 4. 回归审计

### 4.1 cherry-pick 冲突解决验证

任务怀疑 "55.23 + 55.21+22 都改 economy main.rs — git 3-way merge 后是否完整"。
**实际不是 merge commit**，是顺序追加：
- 7deff16 (55.15) → 421585c (55.23) → 465bfeb (55.24) → 5ace5ad (55.21+22)
- 每个 commit 在 first-parent 链上 linear，无 conflict resolution

**economy main.rs 现状包含 3 段完整接线**：
- ✓ 55.23: SagaOrchestrator + ReserveHandler/ConfirmHandler (line 85-102) + 30s 崩溃恢复轮询 (line 104-136)
- ✓ 55.22: PgOutboxRepository + OutboxRelay::run() (line 140-169) + dev NATS fallback
- ✓ 55.21: mTLS load_server_tls_config + tonic tls_config (line 174-201) + dev PEM fallback

**cargo build --workspace**: 0 error, 0 warning ✓
**cargo test --workspace**: 203 lib + 2 doc + 9 integration = **214 passed, 0 failed** ✓

### 4.2 Cargo.lock 共享依赖一致

6 域 + cluster-ops 在 55.21+22 同步加 `shared-platform = { path = "../shared-platform" }`，
Cargo.lock 自动同步 6 个 `"shared-platform"` 引用 ✓

### 4.3 旧 InMemory 测试回归

`cargo test --workspace --lib` 全过：
- admin-service: 18 passed
- cluster-ops: 16 passed
- economy-service: 36 passed
- match-service: 16 passed
- player-service: 24 passed
- rgs-hello: 0 passed
- shared-platform: 78 passed
- social-service: 15 passed
- **合计 203 lib tests pass** ✓

---

## 5. 边界测试矩阵

| 输入 | 期望 | 实测 | 评价 |
|------|------|------|------|
| `credit(account_id=valid, amount=0)` | Validation("amount must be > 0") | **未直接测**（service.rs:94 校验存在，apply_atomic_with_reservation_rejects_non_positive_amount 只测 helper） | **MEDIUM** — 缺 public API 直接测试 |
| `credit(account_id=valid, amount=-100)` | Validation | 未测 | **MEDIUM** — 同上 |
| `debit(account_id=valid, amount=0)` | Validation | 未测 | **MEDIUM** — 同上 |
| `debit(account_id=valid, amount=-100)` | Validation | 未测 | **MEDIUM** — 同上 |
| `credit(account_id=Uuid::nil(), amount=100)` | NotFound("Account") | 未测（find_by_id 返 None → NotFound 路径） | **MEDIUM** — 缺 nil 边界测 |
| `reserve(uuid=Uuid::nil())` (saga) | load_active_account 返 NotFound | 未测 | **MEDIUM** |
| reservation 过期后释放 | 30s 后自动 Compensated 或标过期 | **未测**（reservation.rs 无 expire/leasing 逻辑，仅有 status 状态机） | **MEDIUM** — reservation 过期清理逻辑本身**不存在**，要补 |
| outbox 巨大 payload (>1MB) | 拒绝或限制 | **未测**（InMemoryOutboxRepository 无 size limit） | **LOW** — Pg 端有 TOAST 8KB 限制，但应用层无 |
| RBAC `roles = vec![]` | deny | **未测**（rbac.rs:46 Subject.roles 字段无专门空角色测试） | **MEDIUM** |
| `OutboxRelay::run()` 多副本并发 | SKIP LOCKED 串行化 | **未测**（run 是无限循环，无 integration test） | **HIGH** |
| `SagaOrchestrator::resume(saga_id)` | 重新加载 + 续跑 | **未测**（saga_orchestrator.rs:150 函数无 test） | **CRITICAL** |
| 5 域 main.rs `RGS_TLS_DIR` PEM 缺失 | warn + 降级 insecure gRPC | **未测**（main.rs match 逻辑无 test） | **MEDIUM** |
| 5 域 main.rs `NATS_URI` 不可达 | warn + relay DISABLED | **未测**（main.rs match 逻辑无 test） | **LOW** |
| audit_log append → verify chain | chain 完整 | **不可能**（verify_chain 函数不存在） | **HIGH** — 缺特性 |

---

## 6. CRITICAL Issues

### DC-1. SagaOrchestrator::resume() 完全无测试

- **位置**: `crates/economy-service/src/saga_orchestrator.rs:150`
- **问题**:
  ```rust
  pub async fn resume(&self, saga_id: Uuid) -> Result<()> {
      let mut saga = self.sagas.find_by_id(saga_id).await?...
      self.execute(&mut saga).await
  }
  ```
  8 个 #[tokio::test] 全部覆盖 `execute()`，但**`resume()` 是 55.23 economy main.rs 30s 崩溃恢复后台任务的核心调用**（main.rs:115 `orch.resume(id).await`），无任何 unit test。
- **影响**:
  - 回归风险：若未来改 resume 签名/语义，编译过但线上崩溃恢复路径会静默失败
  - 55.23 main.rs 的 `list_running` + `resume` race 条件无 test 验证（多副本可能并发 resume 同一 saga）
- **修复建议**:
  1. 加 `resume_pending_saga_continues_from_running_step` test：start saga → 中途模拟 crash → 用 InMemorySagaRepository.find_by_id 重新加载 → resume 应续跑
  2. 加 `resume_completed_saga_returns_error` test：resume 已 Completed 的 saga 应 Validation
  3. 加 `resume_not_found_saga_returns_error` test：saga_id 不存在应 NotFound
  4. 加 `resume_concurrent_calls_are_idempotent` test：两个并发 resume 同一 saga 应不重复执行（用 Arc<Mutex<bool>> flag）

---

## 7. HIGH Issues

### DH-1. mTLS 真实 PEM 加载无测试

- **位置**: `crates/shared-platform/src/tls.rs` (`load_server_tls_config`, `load_client_tls`, `load_server_identity`)
- **问题**:
  5 个 tls.rs 测试**全部是 missing file 错误路径**：
  - `file_read_error_includes_path`
  - `client_tls_config_input_required_fields`
  - `load_server_tls_config_missing_file_returns_err`
  - `load_server_tls_config_missing_server_key_returns_err`
  - `load_server_identity_missing_file_returns_err`

  worker 报告"未做"，已确认。55.18 commit message 写"实化：mTLS client_auth_required"，但**实际生产代码用真实 PEM 时的加载路径无 test 验证**。
- **影响**:
  - tonic Server::builder().tls_config() 接到 load_server_tls_config 输出后是否能正常 serve 无验证
  - 真实证书格式（PEM / DER）解析失败时的错误信息无验证
- **修复建议**:
  1. 用 `rcgen` 或 openssl 生成测试用自签名证书（key + cert + CA），存到 `tests/fixtures/test_cert.pem` 等
  2. 加 `load_server_tls_config_valid_pem_succeeds` test
  3. 加 `load_client_tls_with_real_pem_connects_to_test_server` integration test（用 tonic in-process server）
  4. 加 `invalid_pem_format_returns_typed_error` test

### DH-2. OutboxRelay::run() 无 integration test

- **位置**: `crates/shared-platform/src/outbox_relay.rs:110`
  ```rust
  pub async fn run(self: Arc<Self>) { /* 无限循环 + 间隔 + tick */ }
  ```
- **问题**:
  3 个 outbox_relay test 全部测 `tick()` 或 `RelayConfig`：
  - `relay_config_default`
  - `relay_tick_empty`
  - `relay_uses_in_flight_state`（实为测 InMemoryOutboxRepository 状态机）

  `run()` 是后台 polling 循环（per 55.22 wiring 5 域 + cluster-ops + economy main.rs:158 `Arc::new(relay).run().await`），**无任何 test**。
- **影响**:
  - 多副本 relay 协同：InMemoryOutboxRepository 测试用单 repo 模拟，已覆盖；Pg + SKIP LOCKED 需真 DB 测，缺
  - graceful shutdown（Ctrl+C、信号）路径无验证
  - backoff 重试间隔在 run 循环中是否正确触发无验证
- **修复建议**:
  1. 加 `outbox_relay_run_cancels_on_drop` test：spawn run → tokio::time::sleep(200ms) → drop handle → 验证循环退出
  2. 加 `outbox_relay_run_processes_pending_entries` test：预放 3 条 → run 100ms → 验证 mark_sent 调用次数
  3. 真 PG + SKIP LOCKED 集成测在 sqlx-mock 接入时再做（per 55.24 + rgs-testkit）

### DH-3. audit_log hash chain verifier 函数不存在

- **位置**: `crates/admin-service/src/` 全部源文件
- **问题**:
  `grep "fn verify|verify_chain|chain_integrity" crates/admin-service/src/` → **0 match**

  audit_log 只有 `compute_hash()`（entity.rs:135），无对应 verifier。55.13 commit message 写"实化：audit_log hash 升级 FNV-1a → SHA-256 + 长度前缀"，但**只解决 hash 碰撞，未提供 chain integrity verify 接口**。
- **影响**:
  - 审计日志被篡改（手动 UPDATE/DELETE）**无任何运行时检测**（per RGS-SEC-100 §7 要求"hash chain 防篡改"）
  - admin-service service.rs 现有 `audit_log_chains` test（line 308）只测了 `append` 连续 hash 链生成，**未测"重新从 latest append 一条后，能否验证整条链"**
- **修复建议**:
  1. 加 `pub fn verify_hash_chain(entries: &[AuditLogEntry]) -> Result<(), ChainError>` 到 entity.rs 或新 module
  2. 验证：每条 `entry.prev_hash == entries[i-1].hash`，第一条 `prev_hash == 64 个 0`
  3. 重新 compute 每条 hash 与 `entry.hash` 对比
  4. 加 unit test: `verify_chain_valid_chain_ok` / `verify_chain_tampered_hash_detected` / `verify_chain_broken_link_detected`
  5. 加 `chain_verifier_runs_on_each_audit_query` integration test

---

## 8. MEDIUM Issues

### DM-1. credit / debit 边界无直接 unit test

- **位置**: `crates/economy-service/src/service.rs:94, 141, 184`
  ```rust
  if amount <= 0 {
      return Err(Error::Validation("amount must be > 0".to_string()));
  }
  ```
- **问题**:
  只有 `apply_atomic_with_reservation_rejects_non_positive_amount`（service.rs:494）测了 helper，**public `credit(amount=0)` / `credit(amount=-1)` / `debit(amount=0)` / `debit(amount=-1)` 无测试**。
- **影响**:
  - gRPC handler 走 EconomyService::credit 路径，amount=0 客户端绕过 apply_atomic_with_reservation 直接打 credit 会怎样？已校验，但无 test 证据
- **修复建议**:
  ```rust
  #[tokio::test]
  async fn credit_rejects_zero_amount() {
      let (svc, _, _) = make_service_paired();
      let err = svc.credit(Uuid::new_v4(), 0, "k".into()).await.unwrap_err();
      assert!(matches!(err, Error::Validation(_)));
  }
  #[tokio::test]
  async fn credit_rejects_negative_amount() { /* ... */ }
  #[tokio::test]
  async fn debit_rejects_zero_amount() { /* ... */ }
  #[tokio::test]
  async fn debit_rejects_negative_amount() { /* ... */ }
  ```

### DM-2. Uuid::nil() 边界无测试

- **位置**: `crates/economy-service/src/service.rs:155` `find_by_id(account_id)` / saga_orchestrator.rs:230 `current_resource_id`
- **问题**:
  调用方传 `Uuid::nil()` 作为 account_id 时，InMemory / Pg 都返 None → `NotFound`。**无 test 验证 nil 不会触发其他副作用**（例如空字符串 path 误匹配）。
- **影响**:
  - saga_orchestrator ReserveHandler::compensate 用 `resource_id.ok_or(...)` 返 Validation（OK），但 nil Uuid 仍能通过 → 后续 find_by_id 返 NotFound 是预期行为
- **修复建议**:
  加 1 个 test 验证 `credit(Uuid::nil(), 100, "k".into())` → NotFound("Account")

### DM-3. 5 域 main.rs dev fallback 路径无测试

- **位置**: 6 域 + cluster-ops main.rs 第 95-115 行附近的 `match load_server_tls_config() { Err => warn, Ok => Some(cfg) }`
- **问题**:
  5 域 main.rs 行为：
  - PEM 缺失 → `tracing::warn!` + 走 `Server::builder()` 无 `.tls_config()`
  - NATS 不可达 → `tracing::warn!` + 跳过 OutboxRelay spawn

  **这两段 match 逻辑无 test 覆盖**。main.rs 本身就是 binary 入口无法直接测，但可抽 helper 到 lib 测。
- **影响**:
  - 5 域 binary 启动时若 RGS_TLS_DIR 不存在，应降级 insecure gRPC — 行为无 test 担保
  - 5 域 binary 启动时若 NATS 不可达，应跳过 relay — 行为无 test 担保
- **修复建议**:
  1. 把 `load_server_tls_config_with_fallback(path) -> Option<ServerTlsConfig>` 抽到 shared-platform
  2. 加 `tls_fallback_returns_none_on_missing_pem` test
  3. 把 `start_outbox_relay_if_nats_reachable(repo, uri, name) -> bool` 抽到 shared-platform
  4. 加 unit test 验证返回 bool

### DM-4. RBAC 空角色 / 多重角色 edge case

- **位置**: `crates/shared-platform/src/rbac.rs:46` `pub roles: Vec<Role>`
- **问题**:
  6 个新 RBAC test 覆盖了缺 scope、scope 边界、SuperAdmin 跨域，但**`Subject { roles: vec![] }` 空角色** 和 **`Subject { roles: vec![Role::Player, Role::DomainAdmin] }` 多重角色** edge case 未测。
- **影响**:
  - 空角色 → `SimpleAuthorizer::check` 应 deny（任何 require_role 都不匹配）
  - 多重角色：当前实现是 OR（任一 role 满足即 allow）还是 AND（全部 role 满足才 allow）？代码未明确 + 无 test 担保
- **修复建议**:
  1. 文档化 "multi-role OR semantics" + 加 test `multi_role_union_allows_any_scope`
  2. 加 test `empty_roles_denies_all`

### DM-5. SagaOrchestrator 30s 轮询 race 条件

- **位置**: `crates/economy-service/src/main.rs:104-136`
  ```rust
  tokio::spawn(async move {
      loop {
          sagas_for_recover.list_running(SAGA_RECOVER_BATCH).await ...
          for saga in running {
              let id = saga.id;
              if let Err(e) = orch.resume(id).await { ... }
          }
          tokio::time::sleep(Duration::from_secs(SAGA_RECOVER_INTERVAL_SECS)).await;
      }
  });
  ```
- **问题**:
  多副本 economy 部署时，每个副本都起 30s 轮询 → 同一 saga 被并发 resume。`resume()` 调用 `execute()`，execute 内 mark_running/save 已有 OCC（per `entity.rs`），**但 race window 在 list_running 与 resume 之间**（TOCTOU）。
- **影响**:
  - 同一 saga 被并发处理 → 重复 execute step（虽然 idempotency_key 防账目重复写，但 reservation 会被双重 save）
- **修复建议**:
  1. 短期：在 `resume()` 入口加 `SELECT ... FOR UPDATE SKIP LOCKED`（Pg）或 `try_lock`（InMemory）
  2. 加 integration test `resume_concurrent_two_callers_only_one_executes`
  3. 长期：用 LISTEN/NOTIFY 或 advisory lock 跨副本协调

---

## 9. LOW Issues

### DL-1. outbox 巨大 payload 边界

- **位置**: `crates/shared-platform/src/outbox.rs:106` `OutboxEntry::new(subject, payload, command_id)`
- **问题**:
  - InMemory 无 size 限制
  - Pg TOAST 8KB 自动压缩，但 application 层无显式限流
- **影响**: 客户端传 10MB JSON payload → list_pending 时 OOM 风险
- **建议**: 加 `OutboxEntry::new` 校验 payload.len() < 64KB（或 1MB）

### DL-2. 55.18 mTLS 注释 vs 代码一致性

- **位置**: `crates/shared-platform/src/tls.rs:86, 113`
  ```rust
  // tls.rs:86
  /// `client_auth_optional = false`，即 **强制要求客户端出示证书**
  // tls.rs:113
  // tonic 0.12: 不调用 client_auth_optional(true) 即保持 required (default)
  ```
- **结论**: **注释与代码一致** ✓ — `tonic::transport::ServerTlsConfig` 默认 `client_auth_optional=false`，代码正确依赖 default

### DL-3. 55.17 outbox in_flight 状态机

- **位置**: `crates/shared-platform/src/outbox.rs:129` `mark_in_flight(&mut self, lease: Duration)`
- **结论**: **实现完整** ✓ — `OutboxStatus::InFlight + lease_until` 状态机有 `relay_uses_in_flight_state` test 覆盖

### DL-4. 55.12 SagaOrchestrator 注释"崩溃可恢复"

- **位置**: `crates/economy-service/src/saga_orchestrator.rs:10` 注释 `状态机每步都 persist 到 saga 表（崩溃可恢复）`
- **结论**: **功能有，但** `resume()` 测试缺失（见 CRITICAL DC-1）

---

## 10. 修复优先级矩阵

| Issue | 严重度 | 文件 | 估时 (token) | 阻塞 |
|-------|--------|------|--------------|------|
| DC-1 SagaOrchestrator::resume test | CRITICAL | saga_orchestrator.rs | ~30K tokens (3 test) | 是 — 55.23 wiring 无 test 担保 |
| DH-1 mTLS 真实 PEM 加载 test | HIGH | tls.rs + tests/fixtures | ~50K tokens (cert gen + 3 test) | 否 — 失败路径已测，dev 环境够用 |
| DH-2 OutboxRelay::run() test | HIGH | outbox_relay.rs | ~25K tokens (2 test) | 否 — tick 已测 |
| DH-3 audit_log chain verifier | HIGH | admin-service/src/entity.rs (新) | ~80K tokens (新模块 + 4 test) | 是 — RGS-SEC-100 §7 显式要求 |
| DM-1 credit/debit amount=0/-1 test | MEDIUM | service.rs | ~10K tokens (4 test) | 否 |
| DM-2 Uuid::nil() test | MEDIUM | service.rs + saga_orchestrator.rs | ~8K tokens (2 test) | 否 |
| DM-3 dev fallback helper 抽取 | MEDIUM | 5 域 main.rs + shared-platform | ~40K tokens (refactor + 2 test) | 否 |
| DM-4 RBAC empty/multi-role test | MEDIUM | rbac.rs | ~6K tokens (2 test) | 否 |
| DM-5 30s 轮询 race | MEDIUM | saga_orchestrator.rs + main.rs | ~50K tokens (Pg FOR UPDATE + 2 test) | 是 — 多副本部署 OLU 风险 |
| DL-1 outbox payload size limit | LOW | outbox.rs | ~5K tokens (1 check + 1 test) | 否 |

**总估时**: ~304K tokens ≈ 1 人·周（按 Ulysses AI 协作 token-OLU 框架：1 人·周 ≈ 500K-1.5M tokens）

---

## 11. 验证证据汇总

| 检查项 | 命令 | 结果 |
|--------|------|------|
| Lib test 全部通过 | `cargo test --workspace --lib` | 203 passed, 0 failed |
| Doc test 全部通过 | `cargo test --workspace --doc` | 2 passed, 0 failed |
| 全 workspace test | `cargo test --workspace` | 214 passed, 0 failed |
| 编译清洁 | `cargo build --workspace` | 0 error, 0 warning |
| 6 域 + cluster-ops main.rs 编译 | (隐含在上) | ✓ |
| shared-platform dep 一致 | grep 7 main.rs | 7/7 一致 |
| outbox.rs SKIP LOCKED 实现 | list_pending_skips_locked_rows (line 593) | ✓ InMemory 模拟 |
| outbox.rs lease retry 实现 | lease_expiry_retry_picks_up_expired_in_flight (line 642) | ✓ InMemory 模拟 |
| outbox.rs 事务化 | append_in_transaction_persists (line 620) | ✓ InMemory 模拟 |
| mTLS fail-closed | build_channel_no_tls_returns_tls_required_error (channel.rs:159) | ✓ |
| mTLS bypass counter | build_channel_with_require_tls_false_increments_bypass_counter (channel.rs:175) | ✓ |
| mTLS real PEM load | (无 test) | ✗ |
| audit_log SHA-256 + separator | compute_hash_collision_resistance_basic (entity.rs:204) + compute_hash_separator_independence (entity.rs:230) | ✓ |
| audit_log chain verify | (函数不存在) | ✗ |
| RBAC 缺 scope 显式 deny | domain_admin_without_scope_denied (rbac.rs:383) | ✓ |
| RBAC scope 边界 | domain_admin_scope_boundary_does_not_match_prefix (rbac.rs:406) | ✓ |
| RBAC SuperAdmin 跨域 | superadmin_still_works (rbac.rs:430) | ✓ |
| trace_id OTel fallback | client_interceptor_fallback_when_no_otel (grpc_tracing.rs:175) | ✓ |
| trace_id SpanId 0-pad | build_traceparent_with_padded_span_id (grpc_tracing.rs:208) | ✓ |
| SagaOrchestrator::execute 7 path | saga_orchestrator.rs lines 529-769 | ✓ 8 tests |
| SagaOrchestrator::resume | (无 test) | ✗ — CRITICAL |
| OutboxRelay::tick | relay_uses_in_flight_state (outbox_relay.rs:193) | ✓ |
| OutboxRelay::run | (无 test) | ✗ — HIGH |
| main.rs PEM fallback | (无 test) | ✗ — MEDIUM |
| main.rs NATS fallback | (无 test) | ✗ — MEDIUM |
| 30s recover race | (无 test) | ✗ — MEDIUM |
| credit/debit amount=0/-1 | (无 test) | ✗ — MEDIUM |
| Uuid::nil() as account_id | (无 test) | ✗ — MEDIUM |
| RBAC empty roles | (无 test) | ✗ — MEDIUM |
| outbox 巨大 payload | (无 test) | ✗ — LOW |

**未验证项**:
- 未跑真 PG / 真 NATS 集成测试（无 CI 接入）
- 未用 proptest 随机化测试
- 未测 5 域 main.rs 二进制实际启动（受本地无 docker-compose 限制）
- 未审计 .sql migration 与 Pg 实现的 SQL 注入（属 code-review 而非 testing 角度）

---

## 12. 审计员签注

<审计员>: verify-D (testing-integration-adversarial)
<签名>: <占位>
<worktree>: D:\RustGameServer-worktrees\verify-55-D-testing
<base commit>: 5ace5ad
<范围>: 工程 55 P0+收尾 12 L4 commit (testing + integration 角度)

**总结**:
- 测试通过率 100%（214/214）✓
- 跨域集成一致性 100%（Cargo.toml + main.rs imports）✓
- 关键缺口 1 个 CRITICAL (resume) + 3 个 HIGH (mTLS PEM / run() / chain verify) + 5 个 MEDIUM
- 主要风险：崩溃恢复主路径无 test 担保，跨副本部署 OLU 风险存在
- 建议优先修复：DC-1 (resume) + DH-3 (chain verify) + DM-5 (race) — 阻塞 production deployment
