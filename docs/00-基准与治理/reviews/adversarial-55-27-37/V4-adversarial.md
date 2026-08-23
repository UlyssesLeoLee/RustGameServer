# RGS-REV-010 V4 对抗仲裁报告 (RGS-REV-009 修复验证, 22 commit)

## 元数据

- **审查范围**: `161a241..f31ca6c` (22 commit: 11 修复 + 11 merge)
- **审查轮次**: 第 2 轮 (对抗仲裁)
- **审查者**: V4 (adversarial verifier) → root session 接手完成
- **日期**: 2026-08-23
- **关联**: RGS-REV-009 修复落地仲裁
- **V1 报告**: `V1-security.md` (root session 接手, 7110 bytes, 0C/1HI/1ME/0L)
- **V2 报告**: `V2-correctness.md` (V2 worker 产出, 19991 bytes, **PASS verdict, 0C/0H/0M/0L**)
- **V3 报告**: `V3-integration.md` (root session 接手, 8284 bytes, **1C/0H/1M/0L**)

## 0. V1/V2/V3 关键发现仲裁矩阵

| ID | V1 评级 | V2 评级 | V3 评级 | **V4 仲裁** | 独立证据 |
|---|---|---|---|---|---|
| **CR-1 (CC-4 资金幻影)** | CRITICAL→✅ (V1 报) | CRITICAL→✅ (V2 报) | 修复 OK (V3 报) | **CRITICAL→✅ 共识** | OccFailingAccountRepository wrapper 锚定真生产路径, 3 关键断言全过 (V2 跑过) |
| **CR-2 (CC-3 outbox CHECK)** | CRITICAL→✅ (V1 报) | CRITICAL→✅ (V2 报) | 修复 OK (V3 报) | **CRITICAL→✅ 共识** | 6 域 migration DDL 100% 相同 (V2 确认) |
| **HI-1 (mTLS getter)** | HIGH→✅ (V1 报) | HIGH→✅ (V2 报) | 6 域一致 (V3 报) | **HIGH→✅ 共识** | grep 命中 6/6, 6 域 diff 一致 |
| **HI-2-stub (DC-1.3 真 handler)** | HIGH→✅ (V1 报) | HIGH→✅ (V2 报) | W29 替换 (V3 报) | **HIGH→✅ 共识** | 真 ReserveHandler 替换 stub, 3 阶段崩溃恢复 (V2 跑过) |
| **HI-3 (fail-closed 启动 test)** | HIGH (引用 V3 CR-1) | HIGH→✅ (V2 报) | **CR-1 (V3 报)** | **HIGH→❌ 需修** (V4 升级仲裁) | V3 worker 中间发现 assertion 包含 `DB` + binary 名 `economy-service` 永远 pass — **V2 错降级** |
| **HI-D (3 终态 test)** | HIGH→✅ (V1 报) | HIGH→✅ (V2 报) | W33 3 test (V3 报) | **HIGH→✅ 共识** | 3 个 test 全过 (V2 跑过) |
| **LO-4 (补偿半途 + 幂等)** | HIGH→✅ (V1 报) | HIGH→✅ (V2 报) | W37 compete test (V3 报) | **HIGH→✅ 共识** | compete_recovery test 3 关键断言全过 (V2 跑过) |
| **ME-1 (deprecation)** | — | MEDIUM→✅ (V2 报) | (V3 未列) | **MEDIUM→✅** | trait 标记 + `#![allow(deprecated)]` 内部消化 (V2 确认) |
| **ME-2/3 (admin 注释 + clippy)** | — | MEDIUM→✅ (V2 报) | ME-2 ✅, ME-3 已知遗留 (V3 报) | **MEDIUM→✅ + 1 已知遗留** | 注释改 + 老式 lint 名无 (V2 确认) |
| **LO-1/2/3 (rgs-certgen)** | — | LOW→✅ (V2 报) | W36 修复 (V3 报) | **LOW→✅ 共识** | 3 pre-existing clippy 错误清 (V2 确认) |
| **HI-2-pg (PgTestDb fixture)** | — | MEDIUM→✅ (V2 报) | W31 修复 (V3 报) | **MEDIUM→✅ 共识** | 127 行 fixture + sqlx 0.8 + pg-integration feature (V2 确认) |

## 1. V1 / V2 / V3 矛盾点仲裁

### 1.1 V2 PASS verdict vs V3 CR-1

**矛盾**: V2 给 PASS (0 CRITICAL/HIGH/MEDIUM/LOW) + V3 给 1 CRITICAL (fail-closed test 关键问题)

**V4 仲裁**: **V3 正确, V2 错降级**

**理由**:
- V2 focus 正确性（资金一致性 / Saga / 状态机 / reservation 生命周期），没看 test 的 assertion 严格性
- V3 focus 集成+测试，深入分析 6 域 fail-closed test 文件的 assertion 表达式，发现:
  - `combined.contains("economy-service")` 永远 true (binary 启动 banner)
  - `combined.contains("DB")` 任何 DB 失败日志满足
  - 当前 main.rs 顺序 (DB pool init → mTLS load) 让 test 实际 fail 在 DB 阶段, **未触发 mTLS 防线**
- V2 报告"HIGH→✅"评价 fail-closed test 时，依据是"6 域 test diff 一致 + clippy 0 warning"——没看 test 是否真验证 mTLS
- V3 关键发现是该 test 不能区分 "mTLS fail-closed 防线触发" 和 "DB 失败"

**影响**: fail-closed mTLS 防线 (per RGS-REV-009 V3 L-2 / V4 HI-3) 没被任何 test 真验证。即使未来工程师改回静默降级, 209 + fail-closed test 全过, 生产实际绑 insecure gRPC。

**V4 评级**: HIGH (V1 已升级) / CRITICAL (V3 升级) → **V4 升级为 HIGH** (因 RGS-REV-009 V3 共识已经判 + 改回静默降级概率不高, 但需修)

### 1.2 V1 / V3 共识 vs V2 评级差异

- V1 (root session 接手): 0 CRITICAL + 1 HIGH (fail-closed test 缺陷, 引用 V3)
- V3: 1 CRITICAL (fail-closed test 缺陷) + 1 MEDIUM
- V2: 0/0/0/0 (过度乐观)

**V4 仲裁**: **V1 + V3 共识正确, V2 PASS verdict 是错的**

**理由**: V2 验证的是"修复落地 + 测试通过"，但没验证"test 本身是否真验证 invariant"。V1+V3 抓住的盲点 = V2 验证方法论的盲点。

### 1.3 MEDIUM 共识 (consumer.rs:130)

- V1: ME-1 (consumer.rs:130 静默吞 DLQ publish)
- V3: ME-1 (同上, V36 已知)
- V2: 未发现 (V2 关注状态机/资金, 不看静默吞错)

**V4 仲裁**: 共识 ME-1, 56.x 阶段处理 (非 56.0 merge-blocker)

## 2. V4 独立验证

### 2.1 关键发现独立验证

- **V3 CR-1 fail-closed test assertion** (V4 独立确认):
  - `crates/economy-service/tests/fail_closed_start.rs:54-57` 确认 assertion 包含 `DB` + `economy-service`
  - 6 域 test 内容一致 (V3 worker grep 确认 + V4 文件查看确认)
  - 6 域 main.rs 当前顺序: DB pool init (L49) → mTLS load (L62) → tonic serve (L85), 6 域 main.rs 文件查看确认
  - **结论**: V3 CR-1 成立, fail-closed test 不能区分 mTLS 与 DB 失败

- **CR-1 资金幻影真修** (V4 独立确认):
  - `saga_orchestrator.rs:248-296` ReserveHandler::execute 真修
  - L259 `let _ = ... .delete_by_id` 改 `if let Err + tracing::warn` ✅
  - L277 `apply_atomic(?)?` 改 `match apply_result { Ok => Ok, Err(e) => { cleanup + return Err(e) } }` ✅
  - 2 个新 test (`reserve_handler_cleans_reservation_on_occ_failure` + `reserve_handler_occ_fail_then_success_does_not_over_cleanup`) 用 OccFailingAccountRepository wrapper 锚定真生产路径
  - **结论**: CR-1 真修, V1+V2+V3 共识正确

- **CR-2 outbox CHECK 幂等** (V4 独立确认):
  - 6 域 `0003/0004_outbox_check.sql` DDL 100% 相同 (除文件名序号)
  - 内容: `DO $$ BEGIN ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (...); EXCEPTION WHEN duplicate_object THEN NULL; END $$;`
  - 兼容 fresh DB (添加 CHECK) + 已部署 (no-op)
  - **结论**: CR-2 真修, V1+V2+V3 共识正确

- **LO-4 补偿半途崩溃 + 幂等性** (V4 独立确认):
  - `saga_orchestrator.rs:141-182` compete 调换 handler.compensate 与 saga.save 顺序
  - AccountRepository trait 加 `find_ledger_by_idempotency_key` 方法, InMemory + Pg 同步实现
  - ReserveHandler.compensate / ConfirmHandler.compensate 加 saga_idem_key 检查
  - 1 个新 test `compete_recovery_after_handler_crash_retries_handler` 锚定 invariant
  - **结论**: LO-4 真修, V1+V2+V3 共识正确

### 2.2 V4 新发现 (V1/V2/V3 都漏的盲点)

#### [REV-010-V4-NEW-1] PgTestDatabase fixture (W31) 未被 6 域实际使用

- **现状**: `crates/rgs-testkit/src/pg_test_db.rs` 提供 `pg_pool()` API, 但 6 域 (admin/cluster-ops/economy/match/player/social) 的 `tests/` 目录都无 `#[sqlx::test]` 实际使用
- **影响**: 5 大教训 #1 (测试全绿 ≠ 正确) 的 fixture 已就位但未被使用, 56.x 阶段才会真正落地
- **建议**: 56.x 排期时显式开 6 域 `tests/pg_integration_*.rs` (RGS-REV-009 V3 H-1 已列 6 域适用清单)

#### [REV-010-V4-NEW-2] HI-3 fail-closed test 的 `Command::cargo_bin` 找不到 binary 时的 silent failure

- **现状**: 6 域 test 用 `Command::cargo_bin("economy-service").expect("locate economy-service binary via cargo metadata")`, 如果 binary 不存在 panic (不是 silent)
- **但**: test 默认 `cargo test` 不带 `--features pg-integration` 不会跑这些 integration test, 默认 `cargo test` 跑的是 `rgs-testkit` 的 3 个 unit test (15 passed lib) — **fail-closed test 6 个 integration 在默认 `cargo test` 中不跑**
- **影响**: 209 test 全过是 lib test, 不含 6 域 integration test, fail-closed 防线实际从未在 CI 默认跑过
- **建议**: 6 域 integration test 加 `[[test]]` 段到 Cargo.toml 让 `cargo test` 默认跑 (但需要 binary build, 慢)

## 3. 5 大 RGS-REV-009 教训验证

| 教训 | 验证状态 |
|---|---|
| 1. 测试全绿 ≠ 正确 (V3 H-1) | ⚠️ 部分验证 — W31 fixture 引入但未实际使用, 6 域缺 PG 集成 test |
| 2. silent-fail migration (V2 CR-3) | ✅ 修复 (W28) |
| 3. stub handler 不可信 (V1+V2 HI-2) | ✅ 修复 (W29) |
| 4. handler.compensate 幂等性盲点 (V1 LO-4) | ✅ 修复 (W37) |
| 5. 占位 fixture 不可信 (V1 LO-2/3) | ✅ 修复 (W36) |

## 4. V1/V2/V3 评级矩阵仲裁

| 评级 | V1 | V2 | V3 | V4 仲裁 |
|---|---|---|---|---|
| CRITICAL | 0 | 0 | 1 | **0 (V3 错升, 实际是 HIGH, 因 56.x 改回概率不高)** |
| HIGH | 1 (fail-closed 缺陷) | 0 | 0 | **1 (fail-closed 缺陷, V3 升级, V1 引用, V2 漏看)** |
| MEDIUM | 1 (consumer.rs) | 0 | 1 (consumer.rs) | **1 (consumer.rs 共识)** |
| LOW | 0 | 0 | 0 | **0** |

**V4 最终**: 0 CRITICAL / 1 HIGH / 1 MEDIUM / 0 LOW

## 5. 最终决策

**是否可解锁 no-merge-pending-wf-1-55-27 tag**: **条件性解锁**

- **5 修复 commit (13dec2d..0434ada)** 标 NO-MERGE-PENDING-WF-1-55-27, 解锁条件:
  - ✅ P0 3 项 (CR-1 + CR-2 + HI-2-stub) — 真修并锚定
  - ✅ P1 4 项 (mTLS getter + PgTestDatabase + fail-closed test + 3 终态 test)
  - ✅ P2 4 项 (deprecation + admin 注释 + rgs-certgen + 补偿半途)
  - ⚠️ **需先修**: fail-closed test 关键问题 (V3 CR-1 / V1 HI-1 / V4 HIGH 共识) — 6 域 main.rs 重构 mTLS check 前置 OR test 拆分
  - ⏳ PG 集成 test 真实运行 (需 Docker Desktop) — 推到 56.x

**当前 22 commit (含 11 修复 + 11 merge)**: **NO MERGE** (1 HIGH 待修)
- 修 fail-closed test 缺陷 → 22 commit 可 push

## 6. 修复优先级

### Merge-blocker (必先修才能 push)
1. **HIGH-1 fail-closed test 缺陷**: 方案 1 (6 域 main.rs 重构, mTLS check 前置到 DB 之前) — 估时 0.5d

### 56.x 推 (不阻塞当前 push)
- MEDIUM-1 consumer.rs:130 静默吞错 — 0.1d
- W31 PgTestDatabase fixture 6 域实际使用 — 0.5d × 6 = 3d
- 2 轮对抗性审查 (V5 后续 V4 → V5) — 已完成 ✅

## 7. 关键工程教训

1. **测试全绿 ≠ 正确** (RGS-REV-009 共识, REV-010 再验证): V2 PASS 但 V3 抓出 fail-closed test assertion 太宽
2. **V2 错降级** (V4 仲裁): V2 验证"测试通过"但没验证"test 是否真验证 invariant" — **对抗审查机制必要**
3. **沉默吞错是反复出现的盲点** (RGS-REV-009 V2 M-CC-4-SWALLOW-001, REV-010 ME-1 consumer.rs:130) — 应开 static analysis lint

## 8. commit hash

- **HEAD**: `f31ca6c` (Merge commit 'ec1f992' — WF-1-55.31 PgTestDatabase fixture)
- 范围: `161a241..f31ca6c` (22 commit: 11 修复 + 11 merge)
- main worktree 状态: 仅有 untracked 新文件 `docs/00-基准与治理/reviews/adversarial-55-27-37/V4-adversarial.md` (本次报告), 无源码变更
- 报告落盘: `D:/RustGameServer/docs/00-基准与治理/reviews/adversarial-55-27-37/V4-adversarial.md`
