# RGS-REV-008 工程 55 P0+收尾 12 commit 交叉审核总报告

**审核对象**：git log 10bd5b1..2fe68b4（13 commit 含 2 merge，覆盖 12 L4 任务）
**审核时间**：2026-08-22
**commit 基线**：2fe68b4
**审核方式**：4 个独立 verifier 子代理并行（独立 worktree + 独立 context + 强制落盘）

---

## 1. 4 份子报告

| ID | 维度 | Issues | 估时 | 文件 |
|----|------|--------|------|------|
| **A** | code review | **30** (4C/7H/11M/8L) | ~12d | [verify-A_code-review.md](./verify-A_code-review.md) |
| **B** | architecture consistency | **8** (1C/3M/4L) | ~50d | [verify-B_architecture-consistency.md](./verify-B_architecture-consistency.md) |
| **C** | security + Saga | **21** (4C/7H/7M/3L) | ~8d | [verify-C_security-saga.md](./verify-C_security-saga.md) |
| **D** | testing + integration | **11** (1C/3H/5M/2L) | ~4d | [verify-D_testing-integration.md](./verify-D_testing-integration.md) |

**汇总**：
- **Issues 总数（含重叠）**：70（10C/20H/26M/14L）
- **去重独立 issues**（按 file:line 主键）：~35
- **总估时**：~74d（与 A 报告实际工时比 token-OLU 估时 ~1-1.5 token-工作日）

---

## 2. CRITICAL Issues 清单（10 个 → 去重后 8 个独立）

| # | 报告 | 主题 | 状态 | 阻塞 |
|---|------|------|------|------|
| **AC-1** | A | 5 域 main.rs 把 55.18 fail-closed 防线用"dev/test fallback"绕开 | **未修** | 生产 |
| BC-1 | B | admin-service migrations 2 个 0002 冲突（sqlx::migrate! 编译失败）| **✅ 已修**（commit `9d8ed26`）| 编译 |
| **CC-1** | C | admin main.rs 未调 with_pool → 55.13 audit_log 事务化死代码 | **✅ 已修**（commit `9d8ed26`）| 生产 |
| **CC-2** | C | SagaOrchestrator::execute 强校验 Pending，崩溃恢复完全失效 | **✅ 已修**（commit `9d8ed26` 接受 3 状态）| 崩溃恢复 |
| **CC-3** | C | 6 域 outbox migration 缺 `CHECK(status IN ...)` 约束 | 未修 | 部署 |
| **CC-4** | C | apply_atomic OCC 失败时 reservation dangling + 资金幻影 | 未修 | 资金 |
| DC-1 | D | SagaOrchestrator::resume() 完全无测试 | 未修 | 测试 |
| (B/C 共 4) | 多 | 5 域 + cluster-ops main.rs dev fallback 静默降级（与 AC-1 同一根因）| 未修 | 生产 |

**去重后 4 个独立 CRITICAL 待修**：
1. **AC-1/CC-x** fail-closed 防线 6 域失守
2. **CC-3** outbox migration CHECK 约束
3. **CC-4** apply_atomic OCC 失败补偿 + reservation 清理
4. **DC-1** SagaOrchestrator::resume 缺测试

---

## 3. HIGH Issues 清单（20 个 → 去重后 14 个独立）

| 主题 | 报告 | 状态 |
|------|------|------|
| `OutboxRelay::run()` 无限循环无 integration test | D | 未修 |
| mTLS 真实 PEM 加载无测试（55.18 worker 报告"未做"已确认）| D | 未修 |
| `audit_log verify_chain()` 函数本身不存在 | D | 未修 |
| 5 域 mTLS fallback 静默降级（无 fail-fast / 无 metric）| C | 未修 |
| outbox NATS fallback 累积（无 retry 策略）| C | 未修 |
| tonic `client_auth_optional` 隐式依赖默认（55.18 0.12 API 限制）| C | 未修 |
| outbox lease 30s 硬编码（应在 config）| C | 未修 |
| InMemory outbox 忽略 executor | C | 未修 |
| reservation save 只更新 status 字段（OCC race 风险）| C | 未修 |
| 55.13 `append_atomic` 用 `let _ = latest_row;` 丢弃 prev_hash SELECT 结果 | A | 未修 |
| 55.13 service.rs audit_log PG/InMemory 两个分支重复 prev_hash 计算 | A | 未修 |
| 55.12 ReserveHandler `delete_by_id` 失败用 `let _ =` 吞错 | A | 未修 |
| 55.17 outbox mark_sent/mark_failed 无 status 校验 | A | 未修 |
| 55.23 saga 恢复任务无 graceful shutdown | A | 未修 |

---

## 4. 关键交叉发现（4 报告一致确认）

### 4.1 fail-closed 防线失守（最大安全漏洞）
- 55.18 加 `RpcChannelConfig.require_tls = true` 默认 + `mtls_bypassed_total` 计数器
- 55.21+22 在 6 域 main.rs 全部用 `match load_server_tls_config { Ok → Some, Err → warn + None }` 模式
- 后果：k8s 配错 cert 路径 → 6 域**全部静默走 insecure gRPC** + 计数器只 client 端可见，server 端不感知
- 修复建议：5 域 main.rs 改为 `Err → process::exit(1)` fail-fast（dev 用 `rgs-testkit` mock cert）

### 4.2 SagaOrchestrator::execute 三状态
- 原实现强校验 `status == Pending`
- 55.23 30s 崩溃恢复轮询 `list_running(100)` 找的全是 `Running`/`Compensating` → 全部被拒
- **已修**：execute 现在接受 Pending/Running/Compensating（commit `9d8ed26`）

### 4.3 audit_log 死代码
- 55.13 升级 SHA-256 + 事务化 + UNIQUE(prev_hash) + append_atomic trait
- admin main.rs 调 `AdminServiceImpl::new(users, audit)` 而非 `.with_pool(pool)` → 走 InMemory fallback
- **已修**：admin main.rs 调 with_pool（commit `9d8ed26`）
- 仍缺：`verify_chain()` 函数本身**不存在**，即使升级到 SHA-256 也没独立验证器

### 4.4 apply_atomic OCC 失败补偿
- 55.1 修复 credit/debit 多步非原子
- 但 OCC 失败时 service.debit 直接返 Error，account 已改 ledger 未写 → 状态不一致
- reservation save 失败 / apply_atomic 失败 → reservation 留在 DB 无清理（dangling）

### 4.5 admin migrations 0002 冲突
- 55.13 加 `0002_audit_prev_hash_unique.sql`
- 55.17 加 `0002_outbox.sql`（应该是 `0003_outbox.sql`）
- sqlx::migrate! 0.8.6 编译期 reject 重复版本号
- **已修**：重命名为 `0002_audit.sql` + `0003_outbox.sql`（commit `9d8ed26`）

### 4.6 范围口径校正
- 4 报告都指出任务原范围 `ec43377..2fe68b4 = 3 commit` 笔误
- 实际范围 `10bd5b1..2fe68b4 = 13 commit`（55.15 → 55.21+22）
- 这是流程问题，建议 WBS 任务书模板加"自动 git rev-list 范围校验"

---

## 5. 修复优先级矩阵

| 优先级 | Issue | 估时 | 阻塞 | 建议阶段 |
|--------|-------|------|------|----------|
| **P0-1** | AC-1 fail-closed 6 域失守 | 0.5d | 生产部署 | 55.26 |
| **P0-2** | DC-1 SagaOrchestrator::resume 测试 | 0.3d | 崩溃恢复担保 | 55.26 |
| **P0-3** | CC-3 outbox migration CHECK 约束 | 0.2d | 部署 | 55.27 |
| **P0-4** | CC-4 apply_atomic OCC 失败 + reservation cleanup | 1.0d | 资金一致性 | 55.27 |
| P1-1 | DH-1 mTLS 真实 PEM 加载测试 | 0.3d | 测试覆盖 | 55.28 |
| P1-2 | DH-2 OutboxRelay::run() integration test | 0.5d | 测试覆盖 | 55.28 |
| P1-3 | DH-3 audit_log verify_chain() 函数 | 0.5d | 完整性保证 | 55.28 |
| P1-4 | 5 域 mTLS fallback 静默降级 | 0.3d | 生产 | 55.29 |
| P2 | 14 个剩余 HIGH | 合计 ~3d | 中 | 56.x |

---

## 6. 决策建议

### 6.1 立即修（55.26 单一 milestone）
- AC-1 fail-closed 6 域失守 — `match Err → exit(1)` 模式
- DC-1 SagaOrchestrator::resume 缺测试 — 加 4 个 test 覆盖 Pending→Running 转移 + Running step 重入

### 6.2 55.27
- CC-3 outbox CHECK 约束
- CC-4 apply_atomic OCC 失败补偿

### 6.3 56.x
- 14 HIGH 剩余 + 26 MEDIUM + 14 LOW

### 6.4 长期
- 范围口径模板化（WBS 任务书 + verifier brief 都用 `git rev-list --count` 自校）

---

## 7. 关联文档

- A: [verify-A_code-review.md](./verify-A_code-review.md) (21KB)
- B: [verify-B_architecture-consistency.md](./verify-B_architecture-consistency.md) (26KB)
- C: [verify-C_security-saga.md](./verify-C_security-saga.md) (25KB)
- D: [verify-D_testing-integration.md](./verify-D_testing-integration.md) (26KB)
- WF-1-55.25 commit `9d8ed26` (3 CRITICAL 修复 + 3 报告入仓)
- RGS-REV-007 工程 53+54 对抗性审核总报告 (前一轮)

---

## 8. 签字栏

| 角色 | 签字 | 日期 |
|------|------|------|
| 4 verifier 子代理 | code-review/arch-consistency/security-saga/testing-integration | 2026-08-22 |
| PM (Ulysses 一身 12 角色) | `<签名>` | `<日期>` |

**注**：本报告由 4 个 AI adversarial 子代理生成（不代签具名责任人，per RGS-EXEC-001 §6）。

---

## 9. 变更记录

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| v0.1 | 2026-08-22 | 初稿：4 verifier 子代理审核汇总 | 4 adversarial sub-agents + Ulysses (Mavis) |
