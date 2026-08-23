# 工程 55 收尾 — RGS-REV-009 修复任务清单 (WF-1-55.27+)

> **编号说明**: 原 56.x 与 WBS §2A.2.56 工程 56 代码审查任务冲突。本表重编号为 WF-1-55.27+，
> 延续 WF-1-55.26 (5 commit 修复) 的工程 55 收尾工作，由 3 轮对抗性审查 RGS-REV-009 触发。

> 来源: RGS-REV-009 WF-1-55.26 5 commit 3 轮对抗性审查总报告
> 共识矩阵 13 issue (2 CRITICAL / 3 HIGH / 4 MEDIUM / 4 LOW)
> 仲裁: V1+V2+V4+V5 共识，反驳 V3 CONDITIONAL PASS

---

## P0 (merge-blocker, 必先修)

- [ ] **WF-1-55.27** CR-1: CC-4 真修 `ReserveHandler::execute` OCC cleanup
  - 文件: `crates/economy-service/src/saga_orchestrator.rs:248-289` (L253 save + L277 apply_atomic OCC 失败路径)
  - 同步: `crates/economy-service/src/saga_orchestrator.rs:369-394` (`ConfirmHandler::execute` 同样 OCC 模式)
  - 方案: L277 改 `match self.accounts.apply_atomic(...).await { Ok => Ok, Err(e) => { self.reservations.delete_by_id(r.id).await.ok(); tracing::warn!(...); Err(e) } }`
  - 必加: `#[sqlx::test]` 真 PG 集成测试，模拟 PG OCC 失败，验证 (a) reservation cleanup (b) 账户余额未减 (c) ledger 无条目
  - 估计: 0.5d (1d 含 PG 集成 test)
  - AC: 真实 `ReserveHandler::execute` 触发 OCC 失败后，saga 终态 Failed + reservation 表无 dangling 行 + account 余额未减 + ledger 无 +amount entry

- [ ] **WF-1-55.28** CR-2: 6 域 outbox CHECK 幂等 migration
  - 文件 (new): `crates/{admin,cluster-ops,economy,match,player,social}-service/migrations/0004_outbox_check.sql` (or 递增序号)
  - 内容: `DO $$ BEGIN ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed')); EXCEPTION WHEN duplicate_object THEN NULL; END $$;`
  - 必加: 真 PG 验证 (CI docker-compose 起 PG)，模拟 55.17 已部署环境，确认 CHECK 真的生效
  - 估计: 0.3d
  - AC: 6 域 migration 在 fresh DB + 已存在 outbox 表 (55.17 部署后) 两种环境下都成功添加 CHECK 约束，且不报 duplicate_object 错误

- [ ] **WF-1-55.29** HI-2-stub: DC-1.3 stub handler 改真 `ReserveHandler.compensate`
  - 文件: `crates/economy-service/src/saga_orchestrator.rs:935-1004`
  - 替换: `struct CompensateRecorder` + `FailingHandler` → 真实 `ReserveHandler` + `ConfirmHandler`
  - 必加: 构造 55.12 真实 OCC 失败场景（pre-existing 触发点），断言 `account.credit(refund_amount)` **不**被调用
  - 估计: 0.3d
  - AC: 真实 ReserveHandler.compensate 在崩溃恢复 + OCC 失败场景下，验证 55.12 资金幻影回归点 + 账户余额不再次 +amount + ledger 不再次写 Compensated entry

---

## P1 (merge-with-follow-up, 工程 55 收尾中期)

- [ ] **WF-1-55.30** HI-1: shared-platform server 端 mTLS bypass getter
  - 文件: `crates/shared-platform/src/channel.rs` (与 client 端 `mtls_bypassed_total()` 对称)
  - 改动: 加 `pub static SERVER_MTLS_BYPASSED_TOTAL: AtomicU64` + `pub fn server_mtls_bypassed_total() -> u64` getter
  - 6 域 main.rs: 把 `static MTLS_BYPASSED_TOTAL` 替换为 `SERVER_MTLS_BYPASSED_TOTAL.fetch_add(1, ...)`
  - 估计: 0.5d
  - AC: 6 域 server 端 counter 通过 shared-platform getter 暴露，与 client 端对称；后续 metrics 层可基于此 scrape

- [ ] **WF-1-55.31** HI-2-pg: rgs-testkit 加 `PgTestDatabase` fixture
  - 文件: `crates/rgs-testkit/src/pg_test_db.rs` (new)
  - 实现: 基于 `sqlx::test` + docker-compose 自动起 PG 容器
  - 强制: 56.x 起新代码用 `#[sqlx::test]` 写真 DB 集成测试，否则 review 不接受
  - 估计: 1d
  - AC: 提供 `pub async fn pg_pool() -> PgPool` 测试 fixture；CR-1 修复 + DC-1 后续 test 必须用它写真 DB 集成

- [ ] **WF-1-55.32** HI-3: 6 域 fail-closed 启动 integration test
  - 文件: `crates/{admin,cluster-ops,economy,match,player,social}-service/tests/fail_closed_start.rs` (new)
  - 实现: `assert_cmd::Command` 启动 binary，指定不存在的 `RGS_TLS_DIR=/nonexistent`
  - 断言: 进程非 0 退出 + stderr 含 "mTLS config load failed"
  - 估计: 0.5d (6 域各 1 个 test)
  - AC: 6 域 binary 在 `RGS_TLS_DIR` 缺 PEM 文件时 fail-closed 退 1，且 stderr 含 fail-closed 关键字

- [ ] **WF-1-55.33** HI-D: DC-1 补 3 个终态 test
  - 文件: `crates/economy-service/src/saga_orchestrator.rs` (新增 3 test)
  - 实现: `resume_completed_saga_returns_validation_err` / `resume_failed_saga_returns_validation_err` / `resume_aborted_saga_returns_validation_err`
  - 估计: 0.3d
  - AC: 3 个终态 test 全 pass + 验证 `execute()` (L94-99) 对终态返 `Error::Validation("already in terminal state")`

---

## P2 (Defer to 工程 56+, 非阻断)

- [ ] **WF-1-55.34** ME-1: `apply_atomic` 裸调用加 `#[deprecated]`
  - 文件: `crates/economy-service/src/service.rs:141/208/256` (3 处 apply_atomic)
  - 改动: 加 `#[deprecated(note = "use apply_atomic_with_reservation for saga path")]`
  - 估计: 0.1d
  - AC: 56.x 后续新代码若用裸 apply_atomic 触发 `#[deprecated]` warning

- [ ] **WF-1-55.35** ME-2/3: admin 注释 + clippy 脚本升级
  - 文件: `crates/admin-service/migrations/0003_outbox.sql:1` (注释改 `0003_outbox`)
  - 文件: 全局 `clippy.toml` 或 verify 脚本 (`-A pedantic` → `-A clippy::pedantic`)
  - 估计: 0.1d
  - AC: 注释与文件名一致；新 verify 脚本在 clippy 1.98 上一致通过

- [ ] **WF-1-55.36** ME-4 + LO-1/2/3: 静默吞错 + doctest + pre-existing 收尾
  - ME-4: `crates/economy-service/src/saga_orchestrator.rs:259` `let _ = ... .delete_by_id(r.id).await` → `if let Err(e) = ... { tracing::warn!(...) }`
  - LO-1: `crates/shared-platform/src/json_logging.rs:11-13` 删 `no_run` 或加 `compile_fail` 二次 doctest
  - LO-2: `crates/rgs-certgen/src/main.rs:74/100` 3 个 pre-existing clippy error (`&PathBuf` → `&Path` / let-binding unit)
  - LO-3: RGS-REV-008 verify-C HC-5 (outbox lease 30s) / HC-7 (reservation ON CONFLICT) / MC-3 (reservation 5min GC)
  - 估计: 0.5d
  - AC: 4 项全部收尾，`cargo clippy --workspace --all-targets -D warnings` 0 error 0 warning

- [ ] **WF-1-55.37** LO-4: V1 CC-4-COMPENSATION-CRASH 补偿半途崩溃 → 资金丢失
  - 文件: `crates/economy-service/src/saga_orchestrator.rs:141-166` (compete 函数)
  - 方案: 调换 `handler.compensate` 与 `saga.save` 顺序；或加 reconciliation cron 扫 Failed saga
  - 估计: 1d (含 reconciliation cron 设计)
  - AC: 模拟 L155 save 后 / L161 handler.compensate 前崩溃场景，验证 resume 后 refund 不会丢失

---

## 跟踪

| 类别 | 编号 | 数量 | 总估时 |
|---|---|---|---|
| P0 merge-blocker | WF-1-55.27/28/29 | 3 | ~2d (CR-1 1d + CR-2 0.3d + HI-2-stub 0.3d + 集成 0.4d) |
| P1 merge-with-follow-up | WF-1-55.30/31/32/33 | 4 | ~2.3d |
| P2 defer to 工程 56+ | WF-1-55.34/35/36/37 | 4 | ~1.7d |
| **合计** | **WF-1-55.27 ~ 55.37** | **11** | **~6d** |

**关键依赖**:
- WF-1-55.31 (PgTestDatabase fixture) 是 WF-1-55.27 (CR-1) + WF-1-55.29 (HI-2-stub) 的前置（否则无法写真 DB 集成）
- WF-1-55.32 (fail-closed 启动 test) 需 6 域 binary 都装上 assert_cmd 支持
- 建议工程 55 收尾前 2 周完成全部 P0 + 启动 P1

**完成判定** (merge 准入):
- 11 项全部完成
- 4+ verifier 2 轮对抗性审查通过
- `cargo test --workspace` 含 `#[sqlx::test]` 真 DB 集成全过
- 重新跑 `cargo clippy --workspace --all-targets -D warnings` 0 error
- main worktree 重新跑完整 WF-1-55.26 验证脚本

---

**Source**: RGS-REV-009 总报告 (`_total_RGS-REV-009.md`) V5 verifier
**Date**: 2026-08-23
