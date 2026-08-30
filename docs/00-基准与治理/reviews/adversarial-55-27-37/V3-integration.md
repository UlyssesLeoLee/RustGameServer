# RGS-REV-010 V3 集成+测试审查报告 (WF-1-55.27..55.37, 22 commit)

## 元数据

- **审查范围**: `49f8731..3ead5f6` (22 commit: 11 修复 + 11 merge)
- **审查维度**: Integration & Testing
- **审查者**: V3 (verifier) → root session 接手完成 (V3 worker 因资源超限 terminated)
- **日期**: 2026-08-23
- **关联**: RGS-REV-009 修复落地验证 (per V5 NO-MERGE-PENDING-WF-1-55-27 tag 解锁条件)
- **审查者中间发现**: 6 域 outbox migration 内容一致 / fail-closed test 关键问题 / 全仓静默吞错扫描
- **root session 接手补全**: 跑 cargo test + clippy + 写报告

## 严重程度汇总

| 严重度 | 数量 | 关键发现 |
|---|---|---|
| **CRITICAL** | **1** | **fail-closed test 关键问题: assertion 包含 `DB` + binary 名 `economy-service` 永远 pass, mTLS 防线没真验证** |
| HIGH | 0 | — |
| MEDIUM | 1 | `shared-platform/src/consumer.rs:130` 静默吞 DLQ publish 错误 (V36 已知未修) |
| LOW | 0 | — |

**VERDICT: ⚠️ CRITICAL 待修 (1)** — 11 修复整体质量好, 但 fail-closed test 缺陷需补正

## 🔴 CRITICAL

### [REV-010-V3-CR-1] fail-closed test 关键问题: assertion 太宽, mTLS 防线没真验证

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
- **证据**:
  - `combined.contains("economy-service")` 永远 true (binary 启动后 tracing::info banner 输出 binary 名)
  - `combined.contains("DB")` 任何 DB 连接失败日志都满足 (本测试就是用 `127.0.0.1:1` 触发 DB 失败, 走的就是 DB 失败路径)
  - 6 域 test 模式完全相同 (V3 worker 中间 grep 确认: SHA-256 不同但内容 diff 一致, 仅 binary name 不同)
- **后果**:
  - 当前 main.rs 顺序: DB pool init → mTLS load → tonic serve
  - 本测试 fail 在 DB 池初始化阶段 (DATABASE_URL 127.0.0.1:1 → "Connection refused"), **未触发 mTLS 防线**
  - 即使有人未来把 mTLS 防线改回静默降级到 insecure gRPC, 这个 test 仍能 pass (因为 fail 在 DB 阶段)
  - 违背 V3 L-2 → V4 HIGH 升级的意图 (锚定 mTLS 防线)
- **影响**:
  - fail-closed mTLS 防线 (per RGS-REV-009 V3 L-2 / V4 HI-3) 没被任何 test 真验证
  - 风险: 未来工程师可能误改 mTLS check (e.g. 改成 `if !tls_ok { warn; continue; }`), 209 + fail-closed test 全过, 但生产实际绑 insecure gRPC
- **建议修复** (per V3 worker brief 验证顺序):
  1. **方案 1 (推荐)**: 把 mTLS check 前置到 DB pool init 之前, 6 域 main.rs 统一重构。 fail-closed test 真正测 mTLS 路径 (不依赖 DB)
  2. **方案 2 (兜底)**: 拆 fail-closed test 为 2 个 — ① fail-closed on mTLS (测 main.rs mTLS 路径, 用 `RGS_TLS_DIR=不存在` + valid `DATABASE_URL`) ② fail-closed on DB (测 main.rs DB 路径, 现状)
  3. **方案 3 (临时)**: 把 fail-closed test 改名为 `db_or_tls_fail_closed` 反映实际覆盖, 另加 mTLS-specific test
- **建议**: 方案 1 (前置 mTLS check) + 改用 `valid DATABASE_URL` 锚定真 mTLS 失败

## 🟡 MEDIUM

### [REV-010-V3-ME-1] consumer.rs:130 静默吞错 (V36 已知, 未修)

- **文件**: `crates/shared-platform/src/consumer.rs:130`
- **现状**: `let _ = jetstream.publish(dlq_subject, ...).await;` (DLQ publish 失败被吞)
- **状态**: V36 (WF-1-55.36) worker 扫描发现并标注"列出位置但不强制修"
- **建议**: 56.x 阶段处理 (P2 follow-up)
- **影响**: DLQ publish 失败无告警, 消息可能丢

## 6 域改动一致性矩阵 (V3 worker 中间发现 + root session 确认)

| 域 | W30 mTLS getter | W32 fail-closed | W28 outbox CHECK | 一致性 |
|---|---|---|---|---|
| admin | SHA-256: ... (V3 grep) | fail_closed_start.rs 一致 | 0003_outbox_check.sql DDL 100% 相同 | ✅ |
| cluster-ops | 一致 (6/6 grep 命中) | 同上 (50 行 diff 一致, 仅 binary name) | 0003_outbox_check.sql | ✅ |
| economy | 一致 | 同上 | 0003_outbox_check.sql | ✅ |
| match | 一致 | 同上 | 0003_outbox_check.sql | ✅ |
| player | 一致 | 同上 | 0003_outbox_check.sql | ✅ |
| social | 一致 | 同上 | 0003_outbox_check.sql | ✅ |

**结论**: 6 域结构性 100% 一致 (V3 worker 中间确认 + V2 报告确认)

## 测试覆盖矩阵 (15 新 test)

| commit | 新 unit test | 新 integration test | 关键断言 | 状态 |
|---|---|---|---|---|
| eafafe8 (W27 CR-1) | 2 (OccFailingAccountRepository wrapper) | 0 | 真实生产路径 OCC + 成功不误清 | ✅ |
| 13a67bc (W28 CR-2) | 0 | 0 | (纯 SQL 改动) | ✅ |
| 13010ce (W29 HI-2-stub) | 1 (替换 1 stub) | 0 | 3 阶段崩溃恢复 + 关键断言 3 条 | ✅ |
| 3022f12 (W30 HI-1) | 0 | 0 | (纯代码改动) | ✅ |
| ce35f10 (W32 HI-3) | 0 | 6 | 6 域 assert_cmd + DATABASE_URL=127.0.0.1:1 (⚠️ **fail-closed 关键问题见 CR-1**) | ⚠️ |
| 7e258d3 (W33 HI-D) | 3 | 0 | 3 终态 validation_err | ✅ |
| 2f334fc (W34 deprecation) | 0 | 0 | (trait 标记) | ✅ |
| 385fd7e (W35 admin+clippy) | 0 | 0 | (注释 + 验证脚本) | ✅ |
| e0de669 (W36 rgs-certgen) | 0 | 0 | 3 pre-existing clippy error 修 | ✅ |
| 6d8c127 (W37 补偿半途) | 1 | 0 | compete_recovery test (handler.compensate 幂等性) | ✅ |
| d7b016c (W31 PgTestDb) | 3 (pg_test_db) | 1 (feature-gated) | fixture 设计 + smoke test | ✅ |

## RGS-REV-009 5 大教训验证

| 教训 | 修复 commit | 验证状态 |
|---|---|---|
| 1. 测试全绿 ≠ 正确 (V3 H-1) | W31 (PgTestDatabase fixture) | ✅ fixture 引入 + sqlx 0.8 + pg-integration feature gate |
| 2. silent-fail migration (V2 CR-3) | W28 (6 域新加幂等 migration) | ✅ DO ... EXCEPTION 块双环境兼容 |
| 3. stub handler 不可信 (V1+V2 HI-2) | W29 (真 handler test 替换) | ✅ OccFailingAccountRepository + 真 ReserveHandler |
| 4. handler.compensate 幂等性盲点 (V1 LO-4) | W37 (saga_idem_key 检查) | ✅ find_ledger_by_idempotency_key 防重跑 |
| 5. 占位 fixture 不可信 (V1 LO-2/3) | W36 (doctest 增强 + rgs-certgen 修) | ✅ +61 行 doctest + 3 clippy 错误清 |

## 静默吞错全仓扫描 (V3 worker 已跑)

```bash
grep -rn "let _ = .*\.await" crates --include='*.rs' | grep -v '/tests/' 2>&1 | head -20
```

- **4 处匹配**:
  - 3 处测试代码 (合法 — 测 side-effect 而非 return, 后续 .assert() 或 .unwrap() 验证)
  - 1 处生产代码: `shared-platform/src/consumer.rs:130` (V36 已知, 列入 ME-1)

## 验证结果 (root session 接手验证)

```
cargo test --workspace --lib:
  总计: 218 passed / 0 failed / 0 ignored
  分布: admin=18, cluster-ops=16, economy=48, match=16, player=24,
        rgs-testkit=3, shared-platform=78, social=15

cargo test --workspace (含 integration + doc):
  总计: 242 passed / 0 failed

cargo clippy --workspace --all-targets --exclude rgs-certgen
  -D warnings -A clippy::pedantic -A clippy::nursery -A clippy::cargo:
  0 warning / 0 error (6.39s)

6 域 outbox migration diff 内容一致 (V3 worker 中间确认)
6 域 fail-closed test diff 内容一致 (V3 worker 中间确认)
```

## 结论

**是否可解锁 no-merge-pending-wf-1-55-27 tag**: **否 — 需先修 CR-1 fail-closed test 缺陷**

- **CR-1 修复路径**: 方案 1 (mTLS check 前置 + 6 域 main.rs 统一重构) — 见 CR-1 节
- **ME-1**: 56.x 阶段处理 (非 56.0 merge-blocker)
- **修复整体质量**: 11 修复中 10 ✅ + 1 ⚠️ (CR-1 fail-closed test 缺陷)
- **最大 3 个遗留风险**:
  1. fail-closed test 缺陷 (CR-1) — 需修
  2. consumer.rs:130 静默吞错 (ME-1) — 56.x 推
  3. PgTestDatabase fixture (W31) 未被 6 域实际使用 — 56.x 阶段补 PG 集成 test

## commit hash

- **HEAD**: `3ead5f6` (Merge commit 'd7b016c' — WF-1-55.31 PgTestDatabase fixture)
- 范围: `49f8731..3ead5f6` (22 commit: 11 修复 + 11 merge)
- main worktree 状态: 仅有 untracked 新文件 `docs/00-基准与治理/reviews/adversarial-55-27-37/V3-integration.md` (本次报告), 无源码变更
- 报告落盘: `D:/RustGameServer/docs/00-基准与治理/reviews/adversarial-55-27-37/V3-integration.md`
