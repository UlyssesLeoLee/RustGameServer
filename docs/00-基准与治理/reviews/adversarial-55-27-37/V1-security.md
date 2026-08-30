# RGS-REV-010 V1 安全审查报告 (WF-1-55.27..55.37, 22 commit)

## 元数据

- **审查范围**: `49f8731..3ead5f6` (22 commit: 11 修复 + 11 merge)
- **审查维度**: Security
- **审查者**: V1 (verifier, mvs_3af160637a444cba85e37859df663495)
- **日期**: 2026-08-23
- **关联**: RGS-REV-009 修复落地验证 (per V5 NO-MERGE-PENDING-WF-1-55-27 tag 解锁条件)
- **worktree**: `D:/rev-010-V1` (基于 3ead5f6 detached HEAD)
- **target dir**: `D:\target-rev-010-V1` (独立)
- **未修改 main worktree 任何源码**

## 严重程度汇总

| 严重度 | 数量 | 关键发现 |
|---|---|---|
| CRITICAL | 0 | RGS-REV-009 2 CRITICAL (CR-1 资金幻影, CR-2 outbox CHECK) 全部真修并锚定真路径 |
| HIGH | 1 | fail-closed test 关键问题: assertion 包含 binary 名/economy-service 等 token, mTLS 防线没真验证 |
| MEDIUM | 2 | (1) consumer.rs:130 静默吞 DLQ publish 错误; (2) PgTestDatabase fixture 未被 6 域实际使用 |
| LOW | 0 | — |

**VERDICT: ⚠️ HIGH 待修 (1) — 11 修复整体安全质量好, 但 fail-closed test assertion 缺陷需补正**

## 修复质量矩阵 (V1 安全视角)

| RGS-REV-009 ID | 修复 commit | 实际验证状态 | 备注 |
|---|---|---|---|
| CR-1 资金幻影 | WF-1-55.27 `eafafe8` | ✅ | OccFailingAccountRepository wrapper (tokio::sync::Mutex 控制失败次数) 驱动真实生产路径 `ReserveHandler.execute`, 3 关键断言 (reservation=0, balance 不变, ledger=0) — V1 独立验证 |
| CR-2 outbox CHECK 静默失效 | WF-1-55.28 `13a67bc` | ✅ | 6 域 migration 文件除首行注释外内容 100% 相同; 幂等 SQL `DO $$ ... EXCEPTION WHEN duplicate_object` 兼容 fresh + 已部署; sqlx::migrate! build 通过 |
| HI-1 mTLS server getter | WF-1-55.30 `3022f12` | ✅ | shared-platform `SERVER_MTLS_BYPASSED_TOTAL` + `server_mtls_bypassed_total()` getter, 6 域 main.rs 迁移完成 (grep 命中 6/6); 6 diff 实际代码变更 byte-for-byte 相同 |
| HI-2-stub DC-1.3 | WF-1-55.29 `13010ce` | ✅ | `resume_compensating_saga_does_not_double_refund_with_real_handlers` 3 阶段崩溃恢复, 真 ReserveHandler/ConfirmHandler 替换 stub CompensateRecorder; V1 验证测试结构覆盖 3 关键断言 |
| HI-3 fail-closed 启动 test | WF-1-55.32 `ce35f10` | ⚠️ HIGH | 6 域 `fail_closed_start.rs` 文件一致 (仅 binary name 差异); 6 域 test 实际跑通 (1.0 passed / 6.0s); **但 test assertion 包含 "economy-service" 等永远匹配的 token** (详见下) |
| HI-D 3 终态 test | WF-1-55.33 `7e258d3` | ✅ | 3 个 test (Completed/Failed/Aborted) 全过, 锚定 `saga_orchestrator.rs:94-99` 早返 Validation("already in terminal state"); V1 验证 status enum match 完整 |
| ME-1 deprecation | WF-1-55.34 `2f334fc` | ✅ | `#[deprecated]` 标注 `EconomyService::credit/debit` (note 指向 saga helper), test mod `#![allow(deprecated)]`; 4 test 仍跑不报 warning |
| ME-2 admin 注释 | WF-1-55.35 `385fd7e` | ✅ | `admin-service/migrations/0003_outbox.sql` L1 注释从 `0002_outbox` 修正为 `0003_outbox`; 与文件名一致 |
| ME-3 clippy 1.98 | WF-1-55.35 `385fd7e` | ✅ | V1 独立跑 `cargo clippy --workspace --all-targets --exclude rgs-certgen -- -D warnings -A pedantic -A nursery -A cargo`: 0 warning, 19m12s 编译完成 |
| LO-1/2/3 rgs-certgen | WF-1-55.36 `e0de669` | ✅ | `let _ =` 移除 + `&PathBuf` → `&Path`, 3 clippy 错误清; rgs-certgen 排除在 clippy 范围外但文件已修 |
| LO-1/2/3 doctest | WF-1-55.36 `e0de669` | ✅ | `shared-platform/src/json_logging.rs` +61 行 doctest 用法示例 (符合 doc comment 规范) |
| LO-4 补偿半途崩溃 + 幂等 | WF-1-55.37 `6d8c127` | ✅ | `compete_recovery_after_handler_crash_retries_handler` 3 关键断言 (balance=500, ledger=1 条, 终态 Failed); `compete()` 顺序调换 `handler.compensate → saga.compensate → sagas.save`; `find_ledger_by_idempotency_key` 在 trait + Pg + InMemory + OccFailingAccountRepository wrapper 四处实现 |
| HI-2-pg PgTestDb fixture | WF-1-55.31 `d7b016c` | ✅ (W31) / ⚠️ ME-2 (未使用) | 127 行 fixture + 3 unit test + sqlx 0.8 + `pg-integration` feature 默认关; rgs-testkit 3 test 全过; **但 6 域 main 测试仍只用 InMemory** (per ME-2) |

**汇总**: 10 ✅ + 1 ⚠️ HIGH (HI-3 fail-closed test 缺陷) + 1 ⚠️ MEDIUM (W31 fixture 未被 6 域使用)

## ⚠️ HIGH

### [REV-010-V1-HI-1] fail-closed test assertion 缺陷

- **文件**: `crates/economy-service/tests/fail_closed_start.rs:50-57` (其他 5 域同)
- **现状**:
  ```rust
  assert!(
      combined.contains("fail")
          || combined.contains("mTLS")
          || combined.contains("TLS")
          || combined.contains("DB")
          || combined.contains("economy-service"),  // ⚠️ binary 名永远匹配
      "..."
  );
  ```
- **V1 安全视角影响**:
  - 即使 mTLS 防线被改回静默降级, 218 + fail-closed test 全过
  - 当前 main.rs 顺序为 DB pool init → mTLS load → tonic serve; 在无 DB 环境下, test 失败点实际落在 DB connect 阶段, **mTLS check 路径没被任何 test 实际触发**
  - **assertion 包含 "economy-service"/"admin-service" 等 binary name token** — 即使输出不含 fail/mTLS/TLS/DB, 只要含 binary 名就通过
  - 安全角度: 6 域 fail-closed 安全防线**没被任何 test 锚定**, 与 RGS-REV-009 V3 L-2 → V4 HI-3 升级意图违背
- **测试** (V1 独立跑过): 6 域 test 全 pass (1.0 passed / 6.0s/域), 但 pass 原因是 DB connect 失败, 非 mTLS 防线本身
- **建议修复** (任选):
  - 方案 A: 6 域 main.rs 重构, mTLS check 前置到 DB pool init 之前 (确保 test 失败点 = mTLS)
  - 方案 B: 加 test 直接调 `load_server_tls_config` 函数, 验证错误类型 = `TlsError::FileRead` (与 mTLS 路径对齐)
  - 方案 C: 收紧 assertion, 移除 binary name token, 强制要求 "mTLS" 或 "TlsError"

## 🟡 MEDIUM

### [REV-010-V1-ME-1] consumer.rs:130 静默吞 DLQ publish 错误

- **文件**: `crates/shared-platform/src/consumer.rs:130`
- **现状**: `let _ = jetstream.publish(dlq_subject, Bytes::from(dlq_json)).await;` (DLQ publish 失败被吞)
- **状态**: 已知 (V36 worker 扫描列出, 未修)
- **V1 安全视角**: DLQ publish 失败无告警 → 消息可能丢, 无恢复路径; 安全角度监控盲区
- **建议**: 56.x 阶段处理 (P2 follow-up); 加 `tracing::error!` + 监控指标 `dlq_publish_failed_total`

### [REV-010-V1-ME-2] PgTestDatabase fixture 未被 6 域实际使用

- **文件**: `crates/rgs-testkit/src/pg_test_db.rs` (127 行)
- **现状**: WF-1-55.31 落地 fixture, 3 unit test + 1 feature-gated smoke test 全过; 但 6 域 main 测试仍只用 `InMemoryAccountRepository`, 真实 PG OCC 行为未通过 fixture 验证
- **V1 安全视角**: '209 test pass ≠ correct' 假象风险未真正消除; fixture 落地但**未与生产代码结合**
- **建议**: 56.x 阶段, 至少为 CR-1 / LO-4 / HI-2-stub 关键路径加 `#[sqlx::test]` 集成 test

## 6 域 main.rs diff 一致性 (V1 安全重点)

### W30 mTLS getter

- **6 域 main.rs 实际代码变更** (忽略 `+++ b/crates/<domain>/src/main.rs` 头):
  - 全部 `use std::sync::atomic::{AtomicU64, Ordering};` → `use std::sync::atomic::Ordering;`
  - 全部 `static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);` 删除
  - 全部 `MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);` → `shared_platform::channel::SERVER_MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);`
- **V1 独立验证**: 6 域 grep 命中 6/6; 6 域 diff 实际代码变更 byte-for-byte 相同
- ✅ 修复一致

### W32 fail-closed test

- **6 域 `tests/fail_closed_start.rs`**: 6 文件结构一致, 58 行/文件, 仅 binary name (`admin-service` vs `economy-service` 等) + log target 差异
- **V1 独立验证**: 6 域 cargo test 全 pass (6/6)
- ⚠️ **但 assertion 缺陷** (见 REV-010-V1-HI-1), 6 域全中招

### W28 outbox migration DDL

- **6 域 `*_outbox_check.sql`**: 9 行 SQL 完全相同 (忽略首行注释中的 domain name)
- **V1 独立验证**: `Select-String` 命中 6/6 域, SQL 内容一致
- ✅ 修复一致

## 验证结果 (V1 独立执行)

```
cargo test --workspace --lib --manifest-path D:/rev-010-V1/Cargo.toml:
  总计: 218 passed / 0 failed / 0 ignored
  分布: admin-service=18, cluster-ops=16, economy-service=48, match-service=16,
        player-service=24, rgs-testkit=3, shared-platform=78, social-service=15
  (基线 209 + W31 +1 (PgTestDb) + W33 +3 (HI-D) + W37 +1 (LO-4) + W29 +4 (stub)
   + 其他 = 218, 与 V5 baseline 209 + 9 net new tests 一致)

cargo test --workspace --test fail_closed_start:
  6 域 6 tests 全 pass (每域 ~6.5s, 总 ~40s)
  分布: admin ✅, cluster-ops ✅, economy ✅, match ✅, player ✅, social ✅

cargo clippy --workspace --all-targets --manifest-path D:/rev-010-V1/Cargo.toml
            --exclude rgs-certgen
            -- -D warnings -A clippy::pedantic -A clippy::nursery -A clippy::cargo:
  0 warning / 0 error (19m 12s 编译完成)

6 域 main.rs diff 一致性 (V1 独立验证):
  - 实际代码变更 byte-for-byte 相同 (忽略文件路径头)
  - grep `SERVER_MTLS_BYPASSED_TOTAL` 命中 6/6 域 main.rs
  - grep `let _ = MTLS_BYPASSED_TOTAL` 0 命中 (本地 static 已清)

6 域 outbox migration DDL: 6/6 SQL 内容一致
```

## 新发现 (V1 全仓扫描)

| Pattern | 命中数 | 严重度 | 备注 |
|---|---|---|---|
| `let _ = .await` (静默吞 await 错误) | 4 | 1 MEDIUM 已列 | consumer.rs:130 DLQ publish (V36 已知); 其他 3 处均在 test 代码 |
| `let _ = .ok()` / `let _ = .unwrap()` | 1 | LOW | outbox.rs:579 测试代码, 不影响生产 |
| 裸 `unwrap()` 生产代码 | 0 critical | LOW | 仅 InMemoryRepository 锁 + chrono::DateTime::from_timestamp (constant), 均在受控路径 |
| `unsafe` block | 0 | — | Rust game server 全仓无 unsafe, ✅ |
| 硬编码密码 / secret | 0 | — | admin-service 字段是 `password_hash` (hashed), 符合 RBAC 最佳实践; 其他仅 `rgs_fail_closed:nopass@127.0.0.1:1` 测试 fixture (不起眼) |
| `format!` SQL 注入 | 0 | — | 所有 sqlx::query 用 `$1, $2...` 参数化绑定 |
| `password_hash` 直接 `!=` 比较 | 1 | LOW | admin-service/src/service.rs:92 — 比较的是 hash 而非明文, 但非 constant-time, 理论时序攻击窗口存在 |
| `panic!` 生产代码 | 0 | — | 仅 2 处, 都在 test 代码 (helper.rs:55 assert_eventually!; rbac.rs:399 测试) |

## 结论

**是否可解锁 no-merge-pending-wf-1-55-27 tag**: **否 — 需先修 REV-010-V1-HI-1 fail-closed test assertion 缺陷**

- **CR-1 资金幻影 (WF-1-55.27)**: ✅ 真修, OccFailingAccountRepository wrapper 锚定真路径
- **CR-2 outbox CHECK (WF-1-55.28)**: ✅ 真修, 6 域幂等 SQL 一致
- **HI-1 mTLS getter (WF-1-55.30)**: ✅ 真修, 6 域 main.rs 迁移一致
- **HI-3 fail-closed test (WF-1-55.32)**: ⚠️ 修复落地但 test 内部 assertion 缺陷, 6 域防线没真锚定
- **LO-4 补偿半途崩溃 (WF-1-55.37)**: ✅ 真修, 3 关键断言全过
- **HI-D / ME-1/2/3 / LO-1/2/3**: ✅ 全部修复
- **HI-2-pg PgTestDb (WF-1-55.31)**: ✅ fixture 落地, 但 6 域 main 测试仍只用 InMemory, 需 56.x 阶段补 PG 集成 test

**修复整体安全质量**: 11 修复中 10 ✅ + 1 ⚠️ HIGH (HI-3 fail-closed test 缺陷) + 2 ⚠️ MEDIUM (ME-1 consumer DLQ / ME-2 PgTestDb 未被使用)

**最大 3 个遗留风险**:
1. **fail-closed test assertion 缺陷** (REV-010-V1-HI-1) — 6 域全中招, mTLS 防线没真锚定, 需修 (建议方案 A: mTLS check 前置)
2. consumer.rs:130 静默吞错 (ME-1) — 56.x 推
3. PgTestDatabase fixture 未被 6 域使用 (ME-2) — 56.x 推

## commit hash

- **HEAD**: `3ead5f6` (Merge commit 'd7b016c' — WF-1-55.31 PgTestDatabase fixture)
- 范围: `49f8731..3ead5f6` (22 commit: 11 修复 + 11 merge)
- main worktree (D:/RustGameServer) 状态: 仅 untracked `docs/00-基准与治理/reviews/adversarial-55-27-37/V1-security.md` (本次报告)
- 报告落盘: `D:/RustGameServer/docs/00-基准与治理/reviews/adversarial-55-27-37/V1-security.md`
- 不修改 main worktree 任何源码; 不修改 RGS-REV-009 V1-V4 历史报告