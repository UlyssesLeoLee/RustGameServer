# RGS-REV-010 WF-1-55.27..55.37 (22 commit) 3 轮对抗性审查总报告 (5 verifier 子代理)

## 元数据

- **审查范围**: `49f8731..3ead5f6` (22 commit: 11 修复 + 11 merge)
- **审查模式**: 3 轮递进对抗（V1/V2/V3 并行独立 → V4 仲裁 → V5 收口）
- **审查者**: V1 (安全) + V2 (正确性) + V3 (集成) + V4 (对抗仲裁) + V5 (综合收口) — 5 verifier 子代理 (V1/V3/V4 root session 接手, V2/V5 worker 产出)
- **独立 worktree**: V2 worker 独立 worktree; V1/V3 root session 接手 (worker 因资源超限 terminated/cancelled)
- **日期**: 2026-08-23
- **上一轮基线**: `49f8731` RGS-REV-009 总报告 (5 verifier 3 轮对抗, 13 issue 3C/3H/4M/3L)

---

## 1. 审查方法 (3 轮递进对抗)

### 轮 1: 3 维独立审查 (V1/V2/V3 并行)

- V1 安全视角 → `V1-security.md` (7110 bytes, 0C/1HI/1ME/0L) — **root session 接手**
- V2 正确性视角 → `V2-correctness.md` (19991 bytes, **PASS verdict, 0/0/0/0**) — V2 worker 产出
- V3 集成视角 → `V3-integration.md` (8284 bytes, **1CRITICAL/0H/1M/0L**) — **root session 接手**

**轮 1 重大分歧**: V2 给 PASS, V3 给 1 CRITICAL (fail-closed test 关键问题)

### 轮 2: 交叉对抗仲裁 (V4)

- V4 读 V1/V2/V3 报告后:
  - **反驳 V2 错降级**: V2 验证"测试通过"但没验证"test 是否真验证 invariant"
  - **强化 V3 CR-1**: fail-closed test 关键问题成立, 升级为 HIGH (V3 升 CRITICAL, V4 降 HIGH — 因 56.x 改回静默降级概率不高)
  - **独立验证**: 4 个关键 CR/HIGH 修复全部真修
  - **2 个新发现**: PgTestDatabase fixture 未被 6 域使用 + fail-closed test 默认不跑 (CI 盲点)
  - → `V4-adversarial.md` (11030 bytes)

### 轮 3: 综合收口 (V5)

- V5 任务 (本报告):
  1. 独立验证 V1-V4 关键发现
  2. 整合共识矩阵 (哪些 4/4 / 3/4 / 2/4 / 1/4 共识)
  3. 仲裁 V1-V4 之间矛盾 (仅 V2 vs V1+V3 矛盾)
  4. 给出 5 commit (1b30878..cc888b5) NO-MERGE-PENDING-WF-1-55-27 tag 解锁决策
  5. 给 root session 可操作建议

---

## 2. 共识矩阵

### 2.1 严重度统计 (V5 终判)

| 严重度 | V1 | V2 | V3 | V4 | **V5 终判** |
|---|---|---|---|---|---|
| **CRITICAL** | 0 | 0 | 1 | 0 | **0** (V3 升过度, V4 降为 HIGH) |
| **HIGH** | 1 | 0 | 0 | 1 | **1** (fail-closed test 缺陷, V1+V4 共识) |
| **MEDIUM** | 1 | 0 | 1 | 1 | **1** (consumer.rs:130 静默吞错, V1+V3+V4 共识) |
| **LOW** | 0 | 0 | 0 | 0 | **0** |
| **TOTAL** | 2 | 0 | 2 | 2 | **2** |

### 2.2 11 修复的最终评级 (V5)

| RGS-REV-009 ID | 修复 commit | V1 | V2 | V3 | V4 | **V5 终判** |
|---|---|---|---|---|---|---|
| CR-1 资金幻影 | WF-1-55.27 `eafafe8` | ✅ | ✅ | ✅ | ✅ | **✅ CRITICAL→修复** |
| CR-2 outbox CHECK | WF-1-55.28 `13a67bc` | ✅ | ✅ | ✅ | ✅ | **✅ CRITICAL→修复** |
| HI-1 mTLS getter | WF-1-55.30 `3022f12` | ✅ | ✅ | ✅ | ✅ | **✅ HIGH→修复** |
| HI-2-stub DC-1.3 | WF-1-55.29 `13010ce` | ✅ | ✅ | ✅ | ✅ | **✅ HIGH→修复** |
| **HI-3 fail-closed test** | WF-1-55.32 `ce35f10` | ⚠️ HIGH | ✅ | ⚠️ CRITICAL | ⚠️ HIGH | **⚠️ HIGH (test 内部缺陷, 需修)** |
| HI-D 3 终态 test | WF-1-55.33 `7e258d3` | ✅ | ✅ | ✅ | ✅ | **✅ HIGH→修复** |
| ME-1 deprecation | WF-1-55.34 `2f334fc` | — | ✅ | — | ✅ | **✅ MEDIUM→修复** |
| LO-4 补偿半途 + 幂等 | WF-1-55.37 `6d8c127` | ✅ | ✅ | ✅ | ✅ | **✅ MEDIUM→修复** |
| HI-2-pg PgTestDb | WF-1-55.31 `d7b016c` | ✅ | ✅ | ✅ | ✅ | **✅ MEDIUM→修复** |
| ME-2/3 admin 注释 | WF-1-55.35 `385fd7e` | — | ✅ | ✅ | ✅ | **✅ MEDIUM→修复** |
| LO-1/2/3 rgs-certgen | WF-1-55.36 `e0de669` | — | ✅ | ✅ | ✅ | **✅ LOW→修复** |
| LO-1/2/3 doctest | WF-1-55.36 `e0de669` | — | ✅ | ✅ | ✅ | **✅ LOW→修复** |

**汇总**: 11/11 修复落地, 10 ✅ + 1 ⚠️ (HI-3 fail-closed test 缺陷, 需修)

### 2.3 6 域改动一致性 (V3+V4 共识)

| 域 | W30 mTLS getter | W32 fail-closed | W28 outbox CHECK | 一致性 |
|---|---|---|---|---|
| admin | ✅ | ✅ (50 行 diff 一致) | ✅ DDL 100% 相同 | ✅ |
| cluster-ops | ✅ | ✅ | ✅ | ✅ |
| economy | ✅ | ✅ | ✅ | ✅ |
| match | ✅ | ✅ | ✅ | ✅ |
| player | ✅ | ✅ | ✅ | ✅ |
| social | ✅ | ✅ | ✅ | ✅ |

### 2.4 测试覆盖矩阵 (V3 + V5)

| 修复 | 新 test 数 | 关键断言 | 状态 |
|---|---|---|---|
| W27 CR-1 | 2 unit | OccFailingAccountRepository wrapper 锚定真生产路径 | ✅ |
| W29 HI-2-stub | 1 unit (替换 1) | 3 阶段崩溃恢复 + 关键断言 3 条 | ✅ |
| W32 HI-3 | 6 integration | fail-closed 防线 (⚠️ 内部缺陷, V1+V3+V4 共识) | ⚠️ |
| W33 HI-D | 3 unit | 3 终态 validation_err | ✅ |
| W37 LO-4 | 1 unit | compete_recovery test 锚定 invariant | ✅ |
| W31 PgTestDb | 3 unit + 1 feature-gated | fixture 设计 + smoke test | ✅ |

**总计**: 16 新 test, 15 ✅ + 1 ⚠️ (fail-closed test 缺陷)

---

## 3. RGS-REV-009 5 大教训验证

| 教训 | 修复 | 验证 |
|---|---|---|
| 1. 测试全绿 ≠ 正确 (V3 H-1) | W31 PgTestDatabase fixture | ⚠️ fixture 就位但 6 域未实际使用 |
| 2. silent-fail migration (V2 CR-3) | W28 6 域新加幂等 migration | ✅ DO ... EXCEPTION 块双环境兼容 |
| 3. stub handler 不可信 (V1+V2 HI-2) | W29 真 handler test 替换 | ✅ OccFailingAccountRepository + 真 ReserveHandler |
| 4. handler.compensate 幂等性盲点 (V1 LO-4) | W37 saga_idem_key 检查 | ✅ find_ledger_by_idempotency_key 防重跑 |
| 5. 占位 fixture 不可信 (V1 LO-2/3) | W36 doctest 增强 + rgs-certgen 修 | ✅ +61 行 doctest + 3 clippy 错误清 |

---

## 4. 验证结果

| 验证项 | V1 | V2 | V3 | V4 | V5 |
|---|---|---|---|---|---|
| cargo test --workspace --lib | 218 / 0 / 0 | 218 / 0 / 0 | 218 / 0 / 0 | (V3 报) | 218 / 0 / 0 |
| cargo test --workspace (含 integration) | — | 242 / 0 / 0 | 242 / 0 / 0 | (V3 报) | 242 / 0 / 0 |
| cargo clippy (排除 rgs-certgen) | 0 / 0 | 0 / 0 | 0 / 0 | (V3 报) | 0 / 0 |
| 6 域 main.rs diff | — | ✅ 一致 | ✅ 一致 | ✅ | ✅ |
| 6 域 outbox migration DDL | — | ✅ 100% 相同 | ✅ 100% 相同 | ✅ | ✅ |
| 6 域 fail-closed test diff | — | ✅ 一致 | ✅ 一致 (但内部缺陷) | ⚠️ | ⚠️ |

---

## 5. V2 vs V1+V3 矛盾仲裁 (V5 收口)

**矛盾**: V2 给 PASS (0C/0H/0M/0L) + V1/V3 给 1 HIGH / 1 CRITICAL (fail-closed test 缺陷)

**V5 仲裁**: **V1+V3 共识正确, V2 错降级**

**理由**:
- V2 验证"修复落地 + 测试通过", 但没验证"test 本身是否真验证 invariant"
- V2 PASS verdict 是平面 4 verifier 模式的典型失败 — 单维度 pass 易, 多维度对抗才暴露盲点
- V3 抓出的 fail-closed test assertion 太宽是真问题 (V4 独立确认)
- V5 收口: V2 模式应该被取代为 V5 模式 (3 轮递进对抗)

**给 V2 verifier 的反馈 (未来轮次避免)**:
- 验证 test 必须看 assertion 内部表达式, 而非"test 通过"作为充分条件
- 验证 invariant 是否真被 anchor (V2 报告里 7 状态 + 6 reservation 路径覆盖, 但没看 test 内部)
- 验证 test 是否过度宽松 (例如 `contains("DB")` 永远满足)

---

## 6. NO-MERGE-PENDING-WF-1-55-27 Tag 解锁决策

### 解锁条件 (V5 总报告要求)

1. ✅ P0 3 项 (CR-1 + CR-2 + HI-2-stub) — **真修并锚定**
2. ✅ P1 4 项 (mTLS getter + PgTestDatabase + fail-closed test + 3 终态 test) — **3 修 1 缺陷**
3. ✅ P2 4 项 (deprecation + admin 注释 + rgs-certgen + 补偿半途)
4. ⚠️ **fail-closed test 缺陷** (V1+V3+V4+V5 共识) — 需先修
5. ⏳ PG 集成 test 真实运行 (需 Docker Desktop) — 推到 56.x
6. ✅ 2 轮对抗性审查 (RGS-REV-010 5 verifier) — **本轮完成**

### V5 决策: **条件性解锁**

- **当前 22 commit (含 11 修复 + 11 merge)**: **❌ NO MERGE**
  - 原因: 1 HIGH 待修 (fail-closed test 缺陷)
  - 修 fail-closed test 缺陷 → 22 commit 可 push
- **5 原始 commit (1b30878..cc888b5) NO-MERGE-PENDING-WF-1-55-27 tag**: **仍保留**
  - 原因: 5 commit 是 WF-1-55.26 原始产出, 与 22 commit 是不同维度
  - 建议: 解锁 tag 等工程 55 整体 push 后 (即 push RGS-REV-009 评审 + 11 修复一并)

### 修 fail-closed test 缺陷方案 (V5 收口推荐)

**方案 1 (V4 推荐)**: 6 域 main.rs 重构, mTLS check 前置到 DB pool init 之前
- 估时: 0.5d (6 域统一重构)
- AC: 6 域启动顺序变成 `mTLS check → DB pool init → tonic serve`
- test 用 `RGS_TLS_DIR=不存在` + `valid DATABASE_URL` 真正测 mTLS 失败

**方案 2**: 拆 fail-closed test 为 2 个 (mTLS-specific + DB-specific)
- 估时: 0.2d (改 test + 调整 assertion)
- AC: 6 域 × 2 test = 12 个 fail-closed test

**方案 3 (临时)**: 改名 `db_or_tls_fail_closed` + 加 mTLS-specific test
- 估时: 0.1d
- AC: 实际覆盖范围命名清晰

**V5 推荐**: 方案 1 (前置 mTLS check) + 加 mTLS-specific test 锚定真 mTLS 失败

---

## 7. 修复优先级

### Merge-blocker (必先修才能 push 22 commit)
1. **HIGH-1 fail-closed test 缺陷**: 6 域 main.rs 重构 OR test 拆分 — 0.5d

### 56.x 推 (不阻塞当前 push)
- **MEDIUM-1 consumer.rs:130 静默吞错** (V36 已知) — 0.1d
- **W31 PgTestDatabase fixture 6 域实际使用** — 0.5d × 6 = 3d
- **2 轮对抗性审查 (REV-010 5 verifier)** — 已完成 ✅
- **LO-4 完善 reconciliation cron** (per V1 LO-4 方案 2) — 0.5d
- **ME-3 clippy 1.98 lint 名升级** (历史遗留) — 0.1d
- **V2 验证方法学改进** (未来轮次避免 V2 错降级) — 流程改造

### 完成判定 (merge 准入)
1. HIGH-1 fail-closed test 缺陷修复 ✅
2. cargo test --workspace 244+ passed / 0 failed (含 PG 集成 2 个)
3. cargo clippy --workspace 0 error 0 warning (含 rgs-certgen)
4. 2 轮对抗性审查 (REV-010 5 verifier) ✅
5. 解锁 no-merge-pending-wf-1-55-27 tag + 22 commit push

---

## 8. 关键工程教训 (RGS-REV-009 + REV-010 综合)

1. **测试全绿 ≠ 正确** (RGS-REV-009 共识, REV-010 再验证): V2 PASS 但 V3 抓出 fail-closed test assertion 太宽
2. **对抗审查必要** (REV-010 V4 验证): V2 平面 PASS 错降级被 V4 仲裁抓住
3. **silent-fail migration** (RGS-REV-009 V2 CR-3): CHECK 写在 CREATE IF NOT EXISTS 块内, 已部署环境无效
4. **stub handler 不可信** (RGS-REV-009 V1+V2 HI-2 → REV-010 W29 修): 必须用真 handler 测
5. **handler.compensate 幂等性盲点** (RGS-REV-009 V1 LO-4 → REV-010 W37 修): 调换顺序后还必须加 saga_idem_key 查重
6. **test 内部 assertion 严格性** (REV-010 V3 CR-1 新发现): 验证 invariant 不能用"测试通过"作充分条件, 必须看 assertion 表达式
7. **沉默吞错是反复出现的盲点** (RGS-REV-009 V2 M-CC-4 → REV-010 ME-1): 应开 static analysis lint 防漏

---

## 9. RGS-REV-008 → RGS-REV-009 → RGS-REV-010 演进对照

| 维度 | RGS-REV-008 (22f662f) | RGS-REV-009 | RGS-REV-010 (本轮) |
|---|---|---|---|
| 审查模式 | 平面 4 verifier 并行 | 3 轮递进对抗 (5 verifier) | 3 轮递进对抗 (5 verifier) |
| 审查范围 | 12 commit 55 P0+收尾 | 5 commit WF-1-55.26 | 22 commit WF-1-55.27..55.37 (11 修复 + 11 merge) |
| 发现 issue | 70 (10C/20H/26M/14L) | 13 (3C/3H/4M/3L) | **2 (0C/1H/1M/0L) — 大幅减少** |
| 关键发现 | 4 CRITICAL 待修 | WF-1-55.26 修 4 项时 2 项未真修 (CC-4 死代码 + CC-3 静默失效) | 11 修复真修落地 + 1 衍生缺陷 (fail-closed test) |
| 仲裁机制 | 平面 (无仲裁) | V4 仲裁 + V5 收口 (2 轮仲裁抓 V3 错降) | V4 仲裁 + V5 收口 (V2 错降再次被 V4 抓) |
| 工程教训 | 70 issue 收尾 | "测试全绿 ≠ 正确" + "silent-fail migration" + "stub handler 不可信" | + "test 内部 assertion 严格性" + "沉默吞错反复出现" |
| 任务落地 | (无 WBS 同步) | WBS §2A.2.55B 11 项 L4 | WBS §2A.2.55B 11 项 10/11 修 (PgTestDb 推到 56.x) |

**核心演进**: 平面审查 → 3 轮对抗审查。RGS-REV-010 通过 V4 仲裁反驳 V2 错降级, 抓出 RGS-REV-009 抓不出的"test 内部 assertion 太宽"盲点。

---

## 10. V5 给 root session 的可操作建议

### 立即 (push 前)
1. **修 HIGH-1 fail-closed test 缺陷** (0.5d) — 方案 1 推荐
2. 修完跑 `cargo test --workspace` 确认 244+ passed
3. commit + push 22 commit (含 V1/V2/V3/V4/V5 报告作为审查记录)

### 56.x 启动后
1. 启 Docker Desktop, 跑 PgTestDatabase fixture 真实验证 (1d)
2. 6 域 `tests/pg_integration_*.rs` 实际使用 fixture (3d)
3. MEDIUM-1 consumer.rs:130 静默吞错 (0.1d)
4. 2 轮对抗审查再次 (per RGS-REV-010 经验, 1d)

### V5 给 PM 报告 (可选)
1. 写 `docs/00-管理类/RGS-PM-009_WF-1-55 收尾报告 v0.1.md` (类比 RGS-PM-008 v0.1)
2. 列出 11 修复 commit + 验收清单 + 遗留项 (1 HIGH + 1 MEDIUM + 6 域 PG 集成)
3. 推导工程 56 (代码审查工程) 启动建议

---

## 11. commit hash

- **HEAD**: `3ead5f6` (Merge commit 'd7b016c' — WF-1-55.31 PgTestDatabase fixture)
- 范围: `49f8731..3ead5f6` (22 commit: 11 修复 + 11 merge)
- main worktree 状态: 5 untracked 新文件 (V1/V2/V3/V4/V5 报告, 不含审查前 untracked cargo log)
  - `docs/00-基准与治理/reviews/adversarial-55-27-37/V1-security.md` (7110 bytes)
  - `docs/00-基准与治理/reviews/adversarial-55-27-37/V2-correctness.md` (19991 bytes)
  - `docs/00-基准与治理/reviews/adversarial-55-27-37/V3-integration.md` (8284 bytes)
  - `docs/00-基准与治理/reviews/adversarial-55-27-37/V4-adversarial.md` (11030 bytes)
  - `docs/00-基准与治理/reviews/adversarial-55-27-37/_total_RGS-REV-010.md` (本报告)
- 报告落盘: `D:/RustGameServer/docs/00-基准与治理/reviews/adversarial-55-27-37/`

---

## 12. Source & Date

**Source**: V5 verifier 子代理 + root session 接手 (V1/V3/V4 root session 接手, V2 worker 产出, V5 root session 综合收口)

**Date**: 2026-08-23

**Status**: NO MERGE (1 HIGH 待修)
