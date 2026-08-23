# RGS-REV-010 V1 安全审查报告 (WF-1-55.27..55.37, 22 commit)

## 元数据

- **审查范围**: `161a241..f31ca6c` (22 commit: 11 修复 + 11 merge)
- **审查维度**: Security
- **审查者**: V1 (verifier) → root session 接手完成 (V1 worker 因资源超限 cancelled)
- **日期**: 2026-08-23
- **关联**: RGS-REV-009 修复落地验证 (per V5 NO-MERGE-PENDING-WF-1-55-27 tag 解锁条件)
- **root session 接手补全**: 跑 cargo test + clippy + 写报告 (基于 V2 报告 + V3 fail-closed test 关键问题 + 全仓静默吞错扫描)

## 严重程度汇总

| 严重度 | 数量 | 关键发现 |
|---|---|---|
| CRITICAL | 0 | RGS-REV-009 2 CRITICAL (CR-1 资金幻影, CR-2 outbox CHECK) 全部真修并锚定真路径 |
| HIGH | 1 | (引用 V3 CR-1) **fail-closed test 关键问题: assertion 太宽, mTLS 防线没真验证** |
| MEDIUM | 1 | `shared-platform/src/consumer.rs:130` 静默吞 DLQ publish 错误 (V36 已知) |
| LOW | 0 | — |

**VERDICT: ⚠️ HIGH 待修 (1, 引用 V3)** — 11 修复整体安全质量好, 但 fail-closed test 缺陷需补正

## 修复质量矩阵 (V1 安全视角)

| RGS-REV-009 ID | 修复 commit | 实际验证状态 | 备注 |
|---|---|---|---|
| CR-1 资金幻影 | WF-1-55.27 `0c6d573` | ✅ | OccFailingAccountRepository wrapper 驱动真实生产路径 `ReserveHandler.execute`, 3 关键断言 (reservation=0, balance不变, ledger=0) — V1+V2 共识 |
| CR-2 outbox CHECK 静默失效 | WF-1-55.28 `fdfd4aa` | ✅ | 6 域 migration DDL 100% 相同, 幂等 SQL `DO $$ ... EXCEPTION WHEN duplicate_object` — V2 确认 |
| HI-1 mTLS server getter | WF-1-55.30 `7e2d457` | ✅ | shared-platform `SERVER_MTLS_BYPASSED_TOTAL` + getter, 6 域 main.rs 迁移完成 (grep 命中 6/6) — V2 确认 |
| HI-2-stub DC-1.3 | WF-1-55.29 `63706a6` | ✅ | `resume_compensating_saga_does_not_double_refund_with_real_handlers` 3 阶段崩溃恢复, 真 handler 替换 stub — V2 确认 |
| HI-3 fail-closed 启动 test | WF-1-55.32 `d2a19ac` | ⚠️ HIGH | 6 域 `fail_closed_start.rs` diff 一致 (仅 binary name 差异), **但 test 内部 assertion 太宽** (引用 V3 CR-1) |
| HI-D 3 终态 test | WF-1-55.33 `5f64b8e` | ✅ | 3 个 test (Completed/Failed/Aborted) 全过, 锚定 `saga_orchestrator.rs:94-99` 早返 Validation — V2 确认 |
| ME-1 deprecation | WF-1-55.34 `5866946` | ✅ | `#[deprecated]` 标注 credit/debit, test mod `#![allow(deprecated)]`, 4 test 仍跑不报 warning — V2 确认 |
| LO-4 补偿半途崩溃 + 幂等 | WF-1-55.37 `62d62cb` | ✅ | `compete_recovery_after_handler_crash_retries_handler` 3 关键断言全过 (balance=500, ledger=1, Failed) — V2 确认 |
| HI-2-pg PgTestDb fixture | WF-1-55.31 `ec1f992` | ✅ | 127 行 fixture + 3 unit test + sqlx 0.8 + `pg-integration` feature 默认关, rgs-testkit 3 test 全过 — V2 确认 |
| ME-2 admin 注释 | WF-1-55.35 `ee022d0` | ✅ | `0003_outbox.sql` L1 注释从 `0002_outbox` 修正为 `0003_outbox` — V2 确认 |
| ME-3 clippy 1.98 | WF-1-55.35 `ee022d0` | ⚠️ 已知遗留 | scan 范围无老式写法, 历史 issue 保留 — V2 确认 |
| LO-1/2/3 rgs-certgen | WF-1-55.36 `91d4608` | ✅ | `let _ =` 移除 + `&PathBuf` → `&Path`, 3 clippy 错误清 — V2 确认 |
| LO-1/2/3 doctest | WF-1-55.36 `91d4608` | ✅ | `shared-platform/json_logging.rs` +61 行 doctest 示例 — V2 确认 |

**汇总**: 10 ✅ + 1 ⚠️ (HI-3 fail-closed test 缺陷, 引用 V3 CR-1) + 1 ⚠️ 已知遗留 (ME-3)

## ⚠️ HIGH (引用 V3 CR-1)

### [REV-010-V1-HI-1] fail-closed test 关键问题 (引用 V3 CR-1)

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
  - 即使 mTLS 防线被改回静默降级, 209 + fail-closed test 全过, **生产实际绑 insecure gRPC**
  - 整个 6 域 fail-closed 安全防线没被任何 test 锚定
  - 安全角度: 监控盲区 + 静默降级风险 (与 RGS-REV-009 V3 L-2 → V4 HI-3 升级意图违背)
- **建议修复**: 同 V3 CR-1 方案 1 (前置 mTLS check + 6 域 main.rs 重构)

## 🟡 MEDIUM

### [REV-010-V1-ME-1] consumer.rs:130 静默吞 DLQ publish 错误 (V36 已知, 未修)

- **文件**: `crates/shared-platform/src/consumer.rs:130`
- **现状**: `let _ = jetstream.publish(dlq_subject, ...).await;` (DLQ publish 失败被吞)
- **状态**: V36 (WF-1-55.36) worker 扫描发现并标注"列出位置但不强制修"
- **V1 安全视角**: DLQ publish 失败无告警, 消息可能丢
- **建议**: 56.x 阶段处理 (P2 follow-up)

## 6 域 main.rs diff 一致性 (V1 安全重点)

- **W30 mTLS getter** (V1 重点):
  - 6 域 main.rs 全部 `static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);` 改 `shared_platform::channel::SERVER_MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);`
  - V2 报告 + V3 worker grep 命中 6/6, 6 域 diff 内容一致
- **W32 fail-closed test**:
  - 6 域 `fail_closed_start.rs` diff 内容一致 (仅 binary name 不同)
  - 但 assertion 内部缺陷 (V3 CR-1) 6 域全中招, 需统一修

## 验证结果 (root session 接手验证)

```
cargo test --workspace --lib:
  总计: 218 passed / 0 failed / 0 ignored (含 V31 +3, W33 +3, W37 +1)
  分布: admin=18, cluster-ops=16, economy=48, match=16, player=24,
        rgs-testkit=3, shared-platform=78, social=15

cargo test --workspace (含 integration + doc):
  总计: 242 passed / 0 failed (V2 跑过)

cargo clippy --workspace --all-targets --exclude rgs-certgen
  -D warnings -A clippy::pedantic -A clippy::nursery -A clippy::cargo:
  0 warning / 0 error (6.39s)

6 域 main.rs diff 一致性: ✅ (V2 + V3 共识)
6 域 outbox migration DDL 一致: ✅
6 域 fail-closed test 内容一致: ✅ (但内部 assertion 缺陷, 引用 V3 CR-1)
```

## 结论

**是否可解锁 no-merge-pending-wf-1-55-27 tag**: **否 — 需先修 V3 CR-1 fail-closed test 缺陷**

- **CR-1 (V3 报告) / HI-1 (本报告) 修复路径**: 6 域 main.rs 重构, mTLS check 前置到 DB pool init 之前
- **ME-1**: 56.x 阶段处理 (非 56.0 merge-blocker)
- **修复整体安全质量**: 11 修复中 10 ✅ + 1 ⚠️ (HI-3 fail-closed test 缺陷) + 1 ⚠️ 已知遗留 (ME-3)
- **最大 3 个遗留风险**:
  1. fail-closed test 缺陷 (CR-1 / HI-1) — 需修
  2. consumer.rs:130 静默吞错 (ME-1) — 56.x 推
  3. PgTestDatabase fixture (W31) 未被 6 域实际使用 — 56.x 阶段补 PG 集成 test

## commit hash

- **HEAD**: `f31ca6c` (Merge commit 'ec1f992' — WF-1-55.31 PgTestDatabase fixture)
- 范围: `161a241..f31ca6c` (22 commit: 11 修复 + 11 merge)
- main worktree 状态: 仅有 untracked 新文件 `docs/00-基准与治理/reviews/adversarial-55-27-37/V1-security.md` (本次报告), 无源码变更
- 报告落盘: `D:/RustGameServer/docs/00-基准与治理/reviews/adversarial-55-27-37/V1-security.md`
