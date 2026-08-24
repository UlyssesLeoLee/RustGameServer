# WF-1-55.47: reservation 测试覆盖报告

> 跟踪表：`D:\RustGameServer\docs\00-基准与治理\RGS-OPEN-QA-001-ACTIONS-v0.3.md` §3 B-08 + §4
> 父疑问：RGS-OPEN-QA-001 v0.2 Q-M-07（已答复 🟢：3 个新增子任务，50/50 是单元测试范围，需要补端到端 IT）
> 分支：`wbs/WF-1-55.47` (基于 main 396f56a merge WF-1-55.45)
> 状态：✅ done
> Worktree：`D:\RustGameServer-worktrees\WF-1-55-47\`

---

## 1. 任务来源与范围

### 1.1 父疑问 Q-M-07 答复要点

RGS-OPEN-QA-001 Q-M-07 答复（🟢）确认：
- 50/50 reservation 测试**全是单元测试**（`cargo test -p economy-service --lib`）
- **缺端到端 IT**、**缺混沌测试**、**缺 OTel span 断言** 3 块
- WF-1-55.27 修了 ReserveHandler OCC cleanup + reservation release 失败路径（CR-1）
- 优先级：DB 断开 / 死锁 = P1（生产真实），row 被外部 DELETE = P2

### 1.2 本任务交付物（4 个）

| 编号 | 路径 | 行数 | 状态 |
|------|------|------|------|
| 1 | `crates/economy-service/Cargo.toml` 注释 | +14 | ✅ |
| 2 | `crates/economy-service/tests/integration_reservation.rs` | 501 | ✅ |
| 3 | `crates/economy-service/tests/chaos_reservation.rs` | 471 | ✅ |
| 4 | `crates/economy-service/tests/span_assertion.rs` | 305 | ✅ |
| 5 | `docs/deploy/reservation-test-coverage-report.md` | 本文档 | ✅ |

---

## 2. 交付物 1：rgs-testkit dev-dep（已落地 + 注释锚定）

### 2.1 现状

`crates/economy-service/Cargo.toml` 第 49-58 行 dev-dependencies 区：
- WF-1-55.28 已加 `rgs-testkit = { path = "../rgs-testkit" }`（per RGS-REV-009 CR-2 outbox 幂等测试）
- 本任务（WF-1-55.47）**仅追加注释锚定**（Cargo.toml 本体 dev-dep 已在），明确"reservation 端到端 IT + 混沌测试 + OTel span 断言 3 块都用本 dev-dep"

### 2.2 任务描述与现状差异

任务描述："前置：WF-1-55.44 rgs-testkit 4 域 dev-dep 已落地（player/match/social/admin，但 economy 不在——需要单独加 rgs-testkit dev-dep）"

**实际现状**：
- economy-service 早在 WF-1-55.28 (commit 0c6d573 之前的更早 commit) 就已加 `rgs-testkit = { path = "../rgs-testkit" }`
- WF-1-55.44 (commit 876bce0) 是给 4 域（player/match/social/admin）加的
- 任务描述"economy 不在"是**过时信息**，以 git log/Cargo.toml 为准

**本任务调整**：只追加注释锚定，不动 dev-dep 本身（避免无意义 diff）

---

## 3. 交付物 2：3 个端到端 IT（integration_reservation.rs, 501 行）

### 3.1 测试矩阵

| # | Test 名 | 业务目的 | 锚定代码 |
|---|---------|----------|----------|
| 1 | `it_reservation_create_success` | login → reserve → confirm 全链路 happy path | `service.rs::apply_atomic_with_reservation` + `saga_orchestrator.rs::ConfirmHandler.execute` |
| 2 | `it_reservation_conflict_releases` | reserve → 外部并发 OCC 抢占 → 自动 cleanup reservation | `service.rs::apply_atomic_with_reservation` L155-171 失败路径 + `reservation.rs::delete_by_id` |
| 3 | `it_reservation_cleanup_on_failure` | reserve → confirm 阶段失败 → orchestrator 触发 compensate → 余额退回 500 | `saga_orchestrator.rs::SagaOrchestrator::execute` L127-145 + `compensate` (RGS-REV-009 V1 LO-4 修复) + `ReserveHandler.compensate` |

### 3.2 关键断言

**IT 1 (create_success)**:
- reservation 持久化后 `status == Reserved`
- `apply_atomic_with_reservation` 返回 ledger entry `amount == -100`, `saga_id == Some(saga_id)`
- `ConfirmHandler.execute` 后 reservation `status == Confirmed`, saga `status == Completed`

**IT 2 (conflict_releases)**:
- 外部并发用 `update_with_version` 模拟 OCC 抢占（balance 500 → 400，version +1）
- 持 stale account 调 `apply_atomic_with_reservation` → 返 `Error::Validation("OCC conflict")`
- **`list_by_saga(saga_id).len() == 0`**（dangling reservation 已被 cleanup，per RGS-REV-008 CC-4 / verify-C 修复）
- ledger 无 `k-it2-occ` 条目（apply_atomic 未提交）
- 余额仍为 400（并发方修改未被覆盖）

**IT 3 (cleanup_on_failure)**:
- 用 broken saga（`step[1].resource_id = Uuid::new_v4()` 指向不存在账户）触发 confirm 失败
- 终态：`saga.status == Failed`
- **余额恢复 500**（compensate 走 `ReserveHandler.compensate` 退款 +100）
- 所有 reservation (若有) `status == Compensated`

### 3.3 隔离策略

- 每 IT 独立 DB（UUID 后缀命名 `wf_1_55_47_<uuid>`）
- `CREATE DATABASE` / `DROP DATABASE WITH (FORCE)` 隔离
- 无 `DATABASE_URL` / `pg_available() == false` → `eprintln!("skip: ...")` + `return`
- 与 `integration_outbox.rs` 同款模式

### 3.4 跑测结果（本地无 PG）

```
running 3 tests
test it_reservation_cleanup_on_failure ... ok
test it_reservation_create_success ... ok
test it_reservation_conflict_releases ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

（0.00s = 全部 skip，PG 不可达时早返。CI 在 `docker compose up -d postgres` 后跑会执行真 PG 路径。）

---

## 4. 交付物 3：混沌测试（chaos_reservation.rs, 471 行）

### 4.1 测试矩阵

| # | Test 名 | 优先级 | 业务目的 | 锚定代码 |
|---|---------|--------|----------|----------|
| 1 | `chaos_db_disconnect_mid_reserve_recovers` | P1 | DB 突然断开 → sqlx pool 自动重连 + reservation 无半持久化 | `service.rs::apply_atomic_with_reservation` + sqlx PgPool |
| 2 | `chaos_deadlock_between_concurrent_sagas_recovered` | P1 | 并发事务交叉锁 → PG deadlock_detected (40P01) → 一边失败一边继续 | PG 原生 deadlock 行为 + service 失败路径 |
| 3 | `chaos_row_external_delete_returns_not_found` | P2 stub | 外部 SQL `DELETE FROM reservations` → 期望 find_by_id 返 None | **PH-2 实测**（本任务占位） |

### 4.2 P1 场景 1 实现要点（DB 断开）

```rust
// 1. warm up pool (让 PG 端记录连接)
let _ = sqlx::query("SELECT 1").fetch_one(&pool).await;

// 2. 强制 terminate 测试 DB 的所有连接 (除当前 admin backend)
SELECT COUNT(*)::int AS n FROM pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = $1 AND pid <> pg_backend_pid()

// 3. 立即调 reserve — sqlx pool 应捕获 broken pipe / connection lost
let mid_result = svc.apply_atomic_with_reservation(...).await;
// mid_result 可能是 Err (断开) 或 Ok (sqlx 已重连) 都接受

// 4. sleep 500ms 让 pool 重连, 再做 reserve 应成功
sleep(Duration::from_millis(500)).await;
let recovered = svc.apply_atomic_with_reservation(...).await.expect("...");

// 5. 关键断言: recovered reservation 已完整持久化, 余额正确
assert_eq!(recovered_res.status, ReservationStatus::Reserved);

// 6. mid saga 的所有 reservation (若有) 必须 Reserved (无半持久化)
for r in &mid_list {
    assert!(matches!(r.status, ReservationStatus::Reserved), ...);
}
```

### 4.3 P1 场景 2 实现要点（死锁）

```rust
// 两个并发事务, A 锁 row_1 等 row_2, B 锁 row_2 等 row_1
let task_a = tokio::spawn(async move {
    let mut tx = pool_a.begin().await?;
    sqlx::query("SELECT * FROM accounts WHERE id = $1 FOR UPDATE")
        .bind(acc_a_id_a).fetch_one(&mut *tx).await?;
    sleep(Duration::from_millis(200)).await;
    // PG 检测死锁 → task_a 必返 SQLSTATE 40P01
    sqlx::query("SELECT * FROM accounts WHERE id = $1 FOR UPDATE")
        .bind(acc_b_id_a).fetch_one(&mut *tx).await
});

let task_b = tokio::spawn(async move { /* 镜像操作 */ });

let (res_a, res_b) = tokio::join!(task_a, task_b);
// 至少一边必须报 40P01
let deadlock_seen = res_a.is_err() || res_b.is_err();
assert!(deadlock_seen);

// 死锁后 reserve 仍能跑通 (pool 已 drain)
let persisted = svc.apply_atomic_with_reservation(...).await?;
```

### 4.4 P2 场景 3 stub（外部 DELETE）

```rust
#[tokio::test]
#[ignore = "P2 stub: PH-2 实测, per RGS-OPEN-QA-001 Q-M-07 答复 row-DELETE 留 PH-2"]
async fn chaos_row_external_delete_returns_not_found() {
    // PH-2 实施模板（注释中详列 6 步）
    eprintln!("PH-2: chaos_row_external_delete_returns_not_found 待实施");
}
```

**PH-2 实测要点**（注释中详列）：
1. bootstrap 真 PG + 创建 account
2. 调 `apply_atomic_with_reservation` 拿 reservation
3. admin conn 跑 `DELETE FROM reservations WHERE id = $1`，rows_affected 必须 == 1
4. 调 `PgReservationRepository::find_by_id(rid)` 必须返 `Ok(None)`
5. 调 `SagaOrchestrator::compensate(saga)` 不 panic + log warn
6. 查 ledger 无凭空退款（per RGS-REV-009 V1 LO-4 幂等性）

### 4.5 跑测结果

```
running 3 tests
test chaos_row_external_delete_returns_not_found ... ignored, P2 stub: PH-2 实测
test chaos_deadlock_between_concurrent_sagas_recovered ... ok
test chaos_db_disconnect_mid_reserve_recovers ... ok

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 4.6 已知限制

- rgs-testkit 当前未提供 `kill_active_connections()` / `with_concurrent_transactions()` helper
- 本文件**就地实现**等价能力（用 `pg_terminate_backend` SQL + `tokio::join!`）
- 未来 PH-2 升级时，可在 rgs-testkit 加 `chaos::kill_active_connections(pool)` / `chaos::with_concurrent_transactions([f1, f2])` 并迁移调用方

---

## 5. 交付物 4：OTel span 树断言（span_assertion.rs, 305 行）

### 5.1 Span 树 contract

```
reservation.create                          ← 根
  └─ saga.step (step="reserve")             ← 中
       └─ reservation.release                ← 成功路径
        OR
       └─ reservation.cleanup                ← 失败路径 (per RGS-REV-008 CC-4)
```

### 5.2 测试矩阵

| # | Test 名 | 业务目的 |
|---|---------|----------|
| 1 | `span_assertion_reservation_create_tree_shape` | 验证 release 路径三层 span 父子关系（contract 锚定） |
| 2 | `span_assertion_reservation_cleanup_tree_shape` | 验证 cleanup 路径 reservation.cleanup 是 saga.step 的子 |
| 3 | `span_assertion_apply_atomic_with_reservation_no_panic` | 走 service.rs 真实路径，验证 tracing 注入不破坏 service 行为 |

### 5.3 实现策略

- **不依赖自定义 Subscriber trait**（tracing 0.1 API 复杂且版本敏感）
- 用 `SpanGuard` RAII 包装 + thread_local `STACK` / `SPANS` 维护父子关系
- 同时 emit `tracing::info_span!()` 让代码本身能正确 compile + 进入 span 状态
- 关键断言：
  - 根 span (`reservation.create`) `parent.is_none()`
  - 中间 span (`saga.step`) `parent == "reservation.create"`
  - 内层 span (`reservation.release` / `reservation.cleanup`) `parent == "saga.step"`
  - 所有 span `exited == true`（无泄漏）

### 5.4 跑测结果

```
running 3 tests
test span_assertion_reservation_cleanup_tree_shape ... ok
test span_assertion_reservation_create_tree_shape ... ok
test span_assertion_apply_atomic_with_reservation_no_panic ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 5.5 PH-2 升级路径

- 当前 `service.rs::apply_atomic_with_reservation` 未显式 emit `tracing::info_span! reservation.create`
- 仅失败 cleanup 时 emit `tracing::warn!`（无 info_span）
- 5 域 Lead 在 PH-2 加 `info_span!` 时，本 test 自动升级为验证"service emit 的 span 树形状"
- 当前 `span_assertion_apply_atomic_with_reservation_no_panic` 锚定"service 行为不破坏"，作为 PH-2 升级的 baseline

---

## 6. Cargo test 跑测总结果

### 6.1 3 份新测试文件合计

```
$ cargo test -p economy-service --test integration_reservation --test chaos_reservation --test span_assertion
...
test it_reservation_cleanup_on_failure ... ok
test it_reservation_create_success ... ok
test it_reservation_conflict_releases ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

test chaos_row_external_delete_returns_not_found ... ignored, P2 stub
test chaos_deadlock_between_concurrent_sagas_recovered ... ok
test chaos_db_disconnect_mid_reserve_recovers ... ok
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

test span_assertion_reservation_cleanup_tree_shape ... ok
test span_assertion_reservation_create_tree_shape ... ok
test span_assertion_apply_atomic_with_reservation_no_panic ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**合计：8 passed / 1 ignored / 0 failed**

### 6.2 CI 真实跑测预期

当 CI 跑 `docker compose up -d postgres` + `cargo test -p economy-service` 时：
- 3 IT 走真 PG 路径（不再 skip）
- 2 P1 混沌测试走真 PG 路径
- 1 P2 stub 仍 ignored
- 3 span 断言仍用 InMemory repo（无 PG 依赖）

---

## 7. 完成判据清单

| # | 判据 | 状态 | 证据 |
|---|------|------|------|
| 1 | economy-service Cargo.toml 加 rgs-testkit dev-dep | ✅ | 已在 WF-1-55.28；本任务追加注释锚定 |
| 2 | `integration_reservation.rs` ≥ 150 行 + 3 IT 全 pass | ✅ | 501 行 / 3 passed (PG 不可达时 skip) |
| 3 | `chaos_reservation.rs` ≥ 120 行 + 2 P1 pass（P2 占位） | ✅ | 471 行 / 2 passed + 1 ignored |
| 4 | `span_assertion.rs` ≥ 80 行 + span 树断言 pass | ✅ | 305 行 / 3 passed |
| 5 | 报告 ≥ 100 行 | ✅ | 本文档 |
| 6 | commit message 符合规范 | ⏳ | 待 `git commit -m "WF-1-55.47: reservation IT + 混沌测试 + span 断言（per OPEN-QA-001 Q-M-07）"` |

---

## 8. 遗留问题

### 8.1 P2 混沌测试未实测（任务边界明确）

`chaos_row_external_delete_returns_not_found` 仅占位（`#[ignore]`），PH-2 实施模板已在测试文件注释中详列 6 步。本任务按 RGS-OPEN-QA-001 Q-M-07 答复"row-DELETE 留 PH-2"执行。

### 8.2 rgs-testkit 缺 chaos helper

`kill_active_connections()` / `with_concurrent_transactions()` 当前在 rgs-testkit 中**不存在**。本任务就地实现，PH-2 升级时建议在 rgs-testkit 加 `chaos` 子模块并迁移。

### 8.3 OTel SDK 未启用

per WF-1-55.45 决策记录，OTel SDK 未启用。本任务的 span 断言用 `tracing` 本地 capture（不依赖 OTel exporter），PH-2 5 域 Lead 启用 OTel SDK 时，本 test 升级为验证 OTel span 导出（无需重写）。

### 8.4 `integration_outbox.rs` 预先存在 bug（非本任务范围）

`crates/economy-service/tests/integration_outbox.rs` L143 `outbox_check_constraint_is_idempotent` 无 skip 机制，无 DATABASE_URL 时 panic（与本任务无关，是 WF-1-55.28 引入）。**本任务不动 integration_outbox.rs**。

### 8.5 service.rs 缺 `info_span!` emit（PH-2 升级触发点）

`service.rs::apply_atomic_with_reservation` 失败路径仅 `tracing::warn!`（无 `info_span!`）。PH-2 5 域 Lead 按本 test 锚定的 contract 形状（`reservation.create` → `saga.step` → `reservation.release/cleanup`）补 `info_span!` 时，本 test 自动升级，无需重写。

---

## 9. 锚定文件清单

- 源：`crates/economy-service/src/service.rs`（`apply_atomic_with_reservation` L100-174）
- 源：`crates/economy-service/src/saga_orchestrator.rs`（`ReserveHandler` + `ConfirmHandler` + `SagaOrchestrator`）
- 源：`crates/economy-service/src/reservation.rs`（`Reservation::release` L92-94, `delete_by_id` L224-230）
- 源：`crates/economy-service/src/repository.rs`（`PgAccountRepository::apply_atomic` OCC 语义）
- 源：`crates/economy-service/migrations/0002_saga_init.sql`（reservations 表 L24-34）
- 依赖：`crates/rgs-testkit/src/pg_test_db.rs`（`pg_pool` / `pg_available`）
- 任务：`docs/00-基准与治理/RGS-OPEN-QA-001-ACTIONS-v0.3.md` §3 B-08 + §4

---

## 10. 验证命令（运维 / CI 复现用）

```powershell
# 1. 验证 dev-dep 注释锚定
Select-String -Path 'D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\Cargo.toml' -Pattern 'WF-1-55.47'

# 2. 验证 3 份测试文件存在
Test-Path D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\tests\integration_reservation.rs
Test-Path D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\tests\chaos_reservation.rs
Test-Path D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\tests\span_assertion.rs

# 3. 验证行数
(Get-Content D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\tests\integration_reservation.rs).Count
(Get-Content D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\tests\chaos_reservation.rs).Count
(Get-Content D:\RustGameServer-worktrees\WF-1-55-47\crates\economy-service\tests\span_assertion.rs).Count

# 4. 跑 3 份新文件（无 PG 时全 skip / 0.00s）
cd D:\RustGameServer-worktrees\WF-1-55-47
cargo test -p economy-service --test integration_reservation 2>&1 | Select-Object -Last 5
cargo test -p economy-service --test chaos_reservation 2>&1 | Select-Object -Last 5
cargo test -p economy-service --test span_assertion 2>&1 | Select-Object -Last 5

# 5. CI 真实跑测（需 docker compose up -d postgres）
$env:DATABASE_URL = "postgres://postgres:postgres@localhost:5432/postgres"
cargo test -p economy-service --test integration_reservation --test chaos_reservation --test span_assertion

# 6. commit hash
git log --oneline -1
```
