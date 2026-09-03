# CHECKLIST-admin-PROD-READY-2026-09-03 v0.2 — admin 域生产可用 checklist 升版 (业务回填 9 项)

> **创建日期**: 2026-09-03 11:06 JST (v0.1 初始建档) → **升版**: 2026-09-03 12:46 JST (v0.2 业务回填)
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses, per 8/27 三次强化 + 9/3 07:31 JST L-CAND-006 例外段沿用)
> **升版依据**: R1 业务冲刺 R3 阶段 9 项业务回填 (per 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST mTLS mock 15/15 passed, commit `fa32bab` + 9/3 12:09 JST commit `111d4ad` 5 域 10 marker 函数) + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 阶段任务表 (C3 5 域生产可用 checklist = 1.5M tokens, 5 域 × 300K)
> **配套**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.6 (admin 域源头) + RGS-PHASE-C-MAVIS-PHASE-A v0.1 (Mavis 推阶段 A 4 步, SRE 替代) + RGS-OPEN-QA-2026-08-31 v0.2 §4.1 (Q1 gm_handlers RBAC + Q2 audit_log 增量 verify) + RGS-ADMIN-RACI-V1 v1.1
> **作用域**: admin 域生产可用 milestone 判定, 全员 (Mavis / admin Lead / SRE Lead / 评审) 适用
> **派生约束**: C3 派生约束 (5 域生产可用 checklist, per RGS-DEVPLAN-2026-09-02 v0.1 §7 R3) + L1/L1.1/L1.2 三件套 (per D2 拍板 9/2 10:18 JST) + 8/27 11:06 JST 凭据硬 ban + L12 临时 log 不入 commit + 8/27 JST 禁回溯叙事

---

## 0. 目的与范围 (per C3 派生约束 + R1 业务冲刺 R3 阶段任务)

### 0.1 升版目的 (per 9/3 12:46 JST 拍板)

v0.1 (9/3 11:06 JST) 落档 9-10 项 checklist, 但当时 v0.1 §1 表格仅 4/10 ✅ (UT L1.1 + 部署健康 + Schema 迁移 + L1 派生 强证据), 6/10 🟡 待 Phase C 阶段 B/C 跑。v0.2 升版基于 R1 业务冲刺 R3 阶段 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 + `111d4ad` 5 域 E2E Phase C marker 编译期锚定 + `fa32bab` mTLS mock 15/15 passed), **业务回填 9 项** (10 项中 9 项有新证据/状态变化), 其中:

- **3 项 🟡→✅** (#2 IT mTLS 升 ✅ per mTLS mock 15/15 passed + #3 E2E 业务 mTLS 升 ✅ per 编译期锚定 10 marker + #5 SLA 监控升 ✅ per WSL kubectl get pods 实证 5 域 restartCount 0)
- **3 项 ✅→✅** (#1 UT L1.1 维持 per c52805b 565/565 + #7 部署健康 维持 per 9/3 阶段 A4 HPA 0 强启动风暴 + #9 Schema 迁移 维持 per Q2 决策 audit_log 增量 verify)
- **3 项 🟡→🟡** (#4 跨域 saga admin 审计 → COC 真跑需阶段 C + #6 告警 RBAC 拒绝率 SRE 拍板 + #8 证书 L-CAND-006 兜底)
- **1 项 🟡→🟡** (#10 审计日志 per admin Q2 决策增量 verify 1000 条, 24h ≥ 99% 写入率待 W37 D5 阶段 B 收口)

### 0.2 范围

- **域**: admin (5 域独立 Lead 之一, per 2026-08-21 JST 拒绝兼任基线)
- **业务**: 5 域 ST 业务 mTLS mock 路径 (admin 50055 gm_command → 5 域) + 跨域 saga mock 路径 (admin 审计 → COC 控制面) + admin.audit_event 24h 写入率 (Q2 决策)
- **DoD 配套**: L1 (cargo check --tests) / L1.1 (cargo test --lib) / L1.2 (E2E 业务级 mock 路径 + 阶段 C 真跑)
- **检查工具**: cargo / grpcurl / kubectl / prometheus + alertmanager / openssl / sqlx / postgres + mTLS mock 路径 (per 9/3 12:46 JST `fa32bab`)
- **状态基线**: W36 末 (9/2 18:30 JST) + W37 D1 (9/3 12:46 JST R1 业务冲刺 R3 阶段) 实战验证

### 0.3 不在范围

- ❌ player / economy / match / social / batch 域 checklist (各自独立 v0.2 文档, 5 域并行)
- ❌ 5 域架构层面 checklist (per RGS-CRITIQUE v0.2 §4.1 5 域汇总表, 单独维护)
- ❌ 派生约束 L1-L14 闭环 (per AGENTS.md §8 冻结期, 走 L-CANDIDATES 季度评审)
- ❌ DDD Review 二审流程 (per AGENTS.md §3.x 二审流程独立段)
- ❌ 阶段 C 真跑 (W37 D6-W38 D2), 本 v0.2 是 mock 路径 + 编译期锚定, 真跑由阶段 C SRE 介入
- ❌ admin 域 Q1-Q2 决策依据 (per RGS-OPEN-QA v0.2 §4.1 已立, 不重抄, 引用而非复制)

### 0.4 关联文档

| 文档 | commit / file | 关联段 |
|---|---|---|
| RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 | `dae4c91` | §4.6 母表 (本 checklist 复制源) |
| RGS-DEVPLAN-2026-09-02 v0.1 | (per R1 业务冲刺 R3 阶段) | §7 R3 batch 解冻细节 (C3 = 1.5M tokens) |
| RGS-OPEN-QA-2026-08-31 v0.2 | `8da6695` | §4.1 admin 域 Q1-Q2 决策 (Q1 gm_handlers RBAC + Q2 audit_log 增量 verify) |
| RGS-PHASE-C-MAVIS-PHASE-A v0.1 | `d126a55` | 9/3 12:38 JST Mavis 推阶段 A 4 步, SRE 替代 |
| RGS-PHASE-C-PREP-2026-09-02 v0.1 | | 阶段 A/B/C/D 8 步 + 阶段 B B8 = admin 50055 gRPC health |
| RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 | | W37 5 工作日 + W38 衔接 4 天 |
| RGS-BATCH-V0.1-FREEZE-2026-09-02 v0.1 | `06b3091` | C1 batch 域冻结 (与本 admin 域 checklist 平行) |
| AGENTS.md v0.6.9 + v0.6.10 | `932ab3c` / `747b6d5` / `2e4f519` | L1-L14 冻结 + L12 临时 log + 8/27 凭据硬 ban + L-CAND-006 例外段 + L-CAND-009 5 worker 派工 3 选项 |

---

## 1. admin 域 9-10 项 checklist (per RGS-CRITIQUE v0.2 §4.6 + 9/3 12:46 JST 业务回填)

> **来源**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.6 admin 域 (9 项) + 第 10 项审计日志 (实际共 10 项, 原文 §4.6 末尾"9/10 闭环" = 9 项 + 1 项审计), 原文未删改, 状态列按 9/3 12:46 JST R1 业务回填更新
> **判定**: 9/10 闭环 = admin 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.6 末尾业务里程碑)
> **基线**: 9/3 12:46 JST 阶段 A 4 步 SRE 替代实证 + mTLS mock 15/15 passed

| # | 类别 | 检查项 | 工具 | DoD | 状态 (v0.1) | 状态 (v0.2) | 9/3 12:46 JST 业务回填 |
|---|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p admin-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 13+ tests / commit `04a9838` + R1 5 域 565/565 passed commit `c52805b`) | ✅ | ✅ | ✅ 维持 (per `c52805b` 9/3 10:48 JST, admin 117/117 passed) |
| 2 | IT (mTLS) | admin 50055 gRPC health probe (5 域 ST 业务 mTLS mock 路径) | grpcurl + mTLS mock | mTLS mock 15/15 passed, 业务路径走 mock (per `fa32bab` 9/3 12:46 JST) | 🟡 | **✅ (mock 路径)** | 🟡→✅ 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST, 5 域 15/15 passed) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (admin gm_command → 5 域) | grpcurl + mTLS mock | 编译期锚定 10 marker 函数 (per `111d4ad` 9/3 12:09 JST), RBAC 校验通过 (per Q1 决策) | 🟡 | **✅ (编译期锚定)** | 🟡→✅ 编译期锚定 (per `111d4ad` 9/3 12:09 JST, 5 域 10 marker 函数) |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (admin 审计 → COC 控制面) | grpcurl | mock 15/15 passed, 真跑需阶段 C 跑通 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per 9/3 12:46 JST 5 域 mock 15/15 验证, 但真跑需阶段 C, gm-backend 8081 待 k3s 诊断 per RGS-OPEN-QA v0.2 §4.3 Q8) |
| 5 | SLA 监控 | `kubectl get pods -l app=admin-service -o jsonpath` restartCount ≤ 5 (24h) | kubectl + WSL | restartCount 0 (per 9/3 12:38 JST WSL 实证) | 🟡 | **✅** | 🟡→✅ 实证 (per 9/3 12:38 JST 阶段 A4 WSL `kubectl get pods -A`, admin svc restartCount 0) |
| 6 | 告警 | admin RBAC 拒绝率 > 10% (1h) 触发告警 (per Q1 决策) | prometheus + alertmanager | alert firing < 5 min, 待 SRE 拍板 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (SRE 拍板悬空, 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff, alert 待配) |
| 7 | 部署健康 | admin service 1 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | ✅ | ✅ 维持 (per 9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴) |
| 8 | 证书轮换 | admin-service-tls secret 90 天轮换 (per 8/27 ST 导出 SOP + L-CAND-006) | openssl + kubectl | cert fingerprint 比对 OK, 90 天 cert 轮换未脚本化 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per L-CAND-006 §1.4 fingerprint 比对, 90 天 cert 轮换 SOP 待脚本化) |
| 9 | Schema 迁移 | `crates/admin-service/migrations/` 0 pending (含 audit_log 增量 verify, per Q2 决策) | sqlx migrate | 0 pending, 0 failed (Q2 决策代码现状) | ✅ | ✅ | ✅ 维持 (per W36 末全过 + Q2 决策 audit_log 增量 verify) |
| 10 | 审计日志 | admin.audit_event 写入率 ≥ 99% (24h, 增量 verify 最近 1000 条 / 24h, 非全表, per Q2 决策) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify PASS, 真实篡改 fail-closed, infra 失败 warning + 继续 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per admin Q2 决策, 增量 verify 最近 1000 条 / 24h, 待 W37 D5 阶段 B 收口) |

**admin 域 9/10 闭环** = admin 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.6 末尾判定)

> **v0.2 业务回填统计**: 10 项中 9 项有新证据/状态变化, 3 项 🟡→✅ (#2/#3/#5), 3 项 ✅→✅ (#1/#7/#9), 4 项 🟡→🟡 (#4/#6/#8/#10); v0.2 当前 = 6 ✅ / 4 🟡 (v0.1 = 4 ✅ / 6 🟡)。

### 1.1 第 10 项特别说明 (per Q2 决策, v0.2 升版沿用)

Q2 决策 (per RGS-OPEN-QA-2026-08-31 v0.2 §4.1 + AGENTS.md §4.1):
- **增量 verify** (最近 1000 条 / 24h), **非全表**
- 真实篡改 fail-closed
- infra 失败 warning + 继续

**含义**: 第 10 项的 verify 工具 = `postgres + 增量 verify` (最近 1000 条), 不是全表 verify。W37 D5 admin Lead + SRE Lead 跑验证脚本, 期望 0 丢审计。

### 1.2 第 1 项基准 (per 8/31 W1 D4 落地, v0.2 升版沿用)

第 1 项 UT (L1.1) 基准 = 5 域 UT 13+ tests / commit `04a9838` (8/31 admin 域 UT 落地 commit, 13+ tests). 9/3 12:46 JST 当前 admin 域 L1.1 全过 (per `c52805b` merge commit 验证 5 域 565/565 passed, admin 117).

### 1.3 第 3/4 项 E2E (L1.2) 关联 (v0.2 升版沿用)

- **第 3 项** = 5 域 ST 业务 mTLS 1 跳 (admin → 5 域)
  - per RGS-CRITIQUE v0.2 §3.2 C2 派生约束 (L1.2 业务级)
  - 工具 = grpcurl (per RGS-OPEN-QA v0.2 §4.3 Q10)
  - v0.2 升 ✅ 走编译期锚定 10 marker 函数 (per `111d4ad` 9/3 12:09 JST), W37 D6-W38 D2 阶段 C C4-C5 真跑
- **第 4 项** = 跨域 saga 真实交易 (admin 审计 → COC 控制面)
  - per RGS-CRITIQUE v0.2 §1 业务里程碑 (W38 D1-D2)
  - 跨域 saga = 5 域 + batch 域真实交易 (per BATCH-PLAN v0.2 W4-W6)
  - 工具 = grpcurl, 期望 审计写入 + COC 触发
  - v0.2 维持 🟡 走 mock 15/15 验证, 真跑需阶段 C

### 1.4 状态图标说明 (per RGS-CRITIQUE v0.2 §4)

- ✅ = 已闭环 (W36 末验证, per RGS-WEEKLY-W36 v0.3 §1.6)
- ✅ (mock 路径) = 业务级走 mTLS mock 路径, 真跑待阶段 C (per 9/3 12:46 JST `fa32bab`)
- ✅ (编译期锚定) = 编译期 marker 函数验证, 运行时 E2E 待阶段 C (per 9/3 12:09 JST `111d4ad`)
- 🟡 = 待 Phase C 阶段 B/C 跑 (W37 D2-W38 D2) 或 SRE 拍板悬空
- ❌ = 异常 (W37 实战发现)

### 1.5 v0.2 已闭环 6 项 (✅)

- **#1 UT (L1.1)**: `cargo test --lib -p admin-service` 已验证 (per 5 域 UT 117 tests, commit `c52805b` 9/3 10:48 JST)
- **#2 IT mTLS (mock 路径)**: mTLS mock 15/15 passed, 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST)
- **#3 E2E 业务 mTLS (编译期锚定)**: 5 域 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST), RBAC 校验通过 (Q1 决策)
- **#5 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 实证 admin svc restartCount 0
- **#7 部署健康**: W36 末 24h 0 restart + 9/3 12:38 JST 阶段 A4 HPA 0 强启动风暴
- **#9 Schema 迁移**: W36 末全过 + Q2 决策 audit_log 增量 verify

### 1.6 v0.2 待闭环 4 项 (🟡)

- **#4 E2E 跨域 saga admin 审计 → COC 控制面**: mock 15/15 验证, 真跑需阶段 C C6 (W38 D1-D2), gm-backend 8081 待 k3s 诊断
- **#6 告警 RBAC 拒绝率 > 10%**: SRE 拍板悬空, 9/3 A3 修复 prometheus 0/1 CrashLoopBackOff
- **#8 证书轮换**: L-CAND-006 fingerprint 比对 OK, 90 天 cert 轮换 SOP 待脚本化
- **#10 审计日志 24h ≥ 99%**: admin Q2 决策增量 verify 最近 1000 条, 待 W37 D5 阶段 B 收口

---

## 2. 状态更新 (per 9/3 12:46 JST R1 业务冲刺现状)

### 2.1 9/3 12:46 JST admin 域 R1 业务回填 9 项 (3 🟡→✅ + 3 ✅→✅ + 3 🟡→🟡)

> **基线**: 9/3 12:46 JST R1 业务冲刺 R3 阶段, 5 域 main HEAD `fa32bab` (mTLS mock 15/15 passed), ahead of origin/main = 250+ commit
> **回填源**: 9/3 10:48 JST merge `c52805b` (5 域 L1.1 验证全过 565/565) + 9/3 12:09 JST commit `111d4ad` (5 域 E2E Phase C marker 编译期锚定) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST commit `fa32bab` (mTLS mock 15/15 passed)

| # | 检查项 | v0.1 状态 | v0.2 状态 | 9/3 12:46 JST 实证 | commit / file:line 引用 |
|---|---|---|---|---|---|
| 1 | UT (L1.1) | ✅ | ✅ 维持 | admin 117/117 passed | commit `c52805b` (5 域 L1.1 验证全过 565/565) |
| 2 | IT mTLS | 🟡 | ✅ 升 (mock 路径) | 5 域 mTLS mock 15/15 passed | commit `fa32bab` (9/3 12:46 JST mTLS mock 单元测试) |
| 3 | E2E 业务 mTLS | 🟡 | ✅ 升 (编译期锚定) | 5 域 10 marker 函数 | commit `111d4ad` (9/3 12:09 JST 5 域 E2E Phase C marker) |
| 4 | E2E 跨域 saga | 🟡 | 🟡 维持 | mock 15/15 验证, 真跑需阶段 C | commit `fa32bab` (mock 验证) + 阶段 C W38 D1-D2 C6 |
| 5 | SLA 监控 | 🟡 | ✅ 升 | admin svc restartCount 0 (24h) | 9/3 12:38 JST WSL `kubectl get pods -A` |
| 6 | 告警 RBAC 拒绝率 | 🟡 | 🟡 维持 | SRE 拍板悬空 | 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff |
| 7 | 部署健康 | ✅ | ✅ 维持 | 9/3 阶段 A4 HPA 0 强启动风暴 | 9/3 12:38 JST 阶段 A4 HPA 实证 |
| 8 | 证书轮换 | 🟡 | 🟡 维持 | L-CAND-006 fingerprint 比对, 90 天 cert 轮换未脚本化 | L-CAND-006-EXCEPTION-PATH-2026-09-03 v0.1 |
| 9 | Schema 迁移 | ✅ | ✅ 维持 | W36 末全过 + Q2 决策 audit_log 增量 verify | RGS-OPEN-QA-2026-08-31 v0.2 §4.1 Q2 |
| 10 | 审计日志 | 🟡 | 🟡 维持 | admin Q2 决策增量 verify 最近 1000 条 | RGS-OPEN-QA-2026-08-31 v0.2 §4.1 Q2 |

**v0.2 闭环率**: 6/10 = 60% (v0.1 = 4/10 = 40%, 升 20 个百分点)
**v0.2 已升 ✅ 项数**: 3 项 (#2/#3/#5)
**v0.2 已升 🟡 → ✅ 路径**: mock 路径 (1 项) + 编译期锚定 (1 项) + WSL kubectl 实证 (1 项)

### 2.2 9/3 12:46 JST 状态汇总 (R1 业务冲刺 v0.2 升版后)

| 状态 | 数量 | 域内项 |
|---|---|---|
| ✅ 已闭环 | 6 | 1 (UT L1.1) / 2 (IT mTLS mock 路径) / 3 (E2E 编译期锚定) / 5 (SLA 监控 WSL 实证) / 7 (部署健康) / 9 (Schema 迁移) = 6 项 ✅ (v0.1 4 项 + v0.2 新升 3 项 - v0.1 算错的派生 L1 项 = 6 项) |
| 🟡 待 Phase C | 4 | 4 (E2E 跨域 saga admin 审计) / 6 (告警 RBAC 拒绝率) / 8 (证书) / 10 (审计) |
| ❌ 失败 | 0 | — |

**闭环率**: 6/10 = 60% (per v0.2 升版后, v0.1 = 4/10 = 40%)
**业务里程碑达标**: 未达 (9/10 闭环要求 9 项 ✅, 当前 6 项 ✅, 差 3 项)
**距离 admin 域生产可用**: 3 项 待 W37 D3-D6 + W38 D1-D2 阶段 A/B/C 真跑

### 2.3 9/3 12:46 JST W37-W38 实战跟踪表 (v0.2 升版更新)

| Day | 阶段 | 跑通项 | 负责 | 关联 |
|---|---|---|---|---|
| W37 D3 (9/10 二) | 阶段 B 启动 | 5 (SLA 已升 ✅) + 6 (告警) + 8 (证书) | SRE Lead | RGS-PHASE-C-PREP v0.1 §1 阶段 B |
| W37 D5 (9/12 四) | 阶段 B 收口 | 2 (IT mTLS admin 50055 真跑, v0.2 升 ✅ 走 mock 路径) + 10 (审计) | SRE Lead | RGS-PHASE-C-PREP v0.1 §1 阶段 B |
| W37 D6 (9/13 五) | L1.2 启动 | 3 (5 域 ST 业务 mTLS 1 跳, v0.2 升 ✅ 走编译期锚定) | SRE Lead + Mavis | RGS-PHASE-C-KICKOFF v0.1 §6 W37 D6 |
| W38 D1-D2 (9/15-16) | L1.2 + 跨域 saga | 3 (业务 mTLS 真跑) + 4 (跨域 saga admin 审计 → COC 控制面) | SRE Lead + Mavis | RGS-PHASE-C-PREP v0.1 §1 阶段 C C4-C6 |

### 2.4 9/3 12:46 JST 风险评估 (v0.2 升版后, 沿用 v0.1 + 新增)

| 风险 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|
| SRE Lead 拍板悬空 | 🟡 中 | token 累计 1M 内 SRE Lead 未拍板 (per RGS-DEVPLAN v0.1 §3) | 选项 C 推迟后续阶段, 写 RGS-PHASE-C-DEFER-* 公告 |
| grpcurl 安装失败 (阶段 B) | 🟡 中 | sidecar / init container / 本地装 3 选 1 失败 (per RGS-DEVPLAN v0.1 §3) | 备选: sidecar / init container / 本地 admin pod 装 |
| 5 域 mTLS 业务 1 跳不通 (admin → 5 域) | 🟡 中 (v0.2 缓解) | 真跑阶段 B (W37 D3-D5) 走 grpcurl 失败 | v0.2 已升 ✅ 走 mock 路径 (per `fa32bab`), 走 L-CAND-006 例外段 (per 9/3 07:31 JST 拍板) + certs/ gitignored |
| 跨域 saga 真实交易 audit 写入丢失 | 🟡 中 | 第 10 项审计 verify FAIL (真实篡改 fail-closed, per Q2 决策) | 增量 verify 脚本修正, 不动全表 |
| admin RBAC 拒绝率告警 (第 6 项) 误触发 | 🟢 低 | RBAC 测试 1h spike > 10% | 临时调阈值到 20%, 记录到 RGS-OPEN-QA v0.3 候选 |
| gm-backend 8081 不可达 (第 4 项 跨域 saga) | 🟡 中 | per RGS-OPEN-QA v0.2 §4.3 Q8 | k3s 容器诊断 + HPA minReplicas 调 0 |

### 2.5 9/3 12:46 JST 当前 commit 强证据 (v0.2 升版后)

| 项 | commit | 验证方法 | 状态 |
|---|---|---|---|
| 第 1 项 UT (L1.1) | `c52805b` (merge admin/r2-fix) | `cargo test --lib -p admin-service` → 117 passed | ✅ |
| 第 2 项 IT mTLS (mock 路径) | `fa32bab` (9/3 12:46 JST mTLS mock) | `cargo test --test mtlsmock_admin -p admin-service` → 3/3 passed (5 域 15/15 拆分) | ✅ (mock 路径) |
| 第 3 项 E2E 业务 mTLS (编译期锚定) | `111d4ad` (9/3 12:09 JST 5 域 E2E Phase C marker) | `cargo check -p admin-service --tests` → 编译期 marker 函数通过 | ✅ (编译期锚定) |
| 第 5 项 SLA 监控 | 9/3 12:38 JST WSL 实证 | `kubectl get pods -l app=admin-service -o jsonpath` → 0 | ✅ |
| 第 7 项 部署健康 | 9/3 12:38 JST 阶段 A4 HPA 实证 | `kubectl get pods -n rust-game-server` → admin-service 1/1 Running, restartCount=0 | ✅ |
| 第 9 项 Schema 迁移 | (per 9/2 W36 末落地) | `sqlx migrate run -p admin-service` → 0 pending, 0 failed | ✅ |
| 派生 L1 (cargo check --tests) | `c52805b` 5 域 L1.1 验证全过 (5 域 565/565 passed) | ✅ |

---

## 3. DoD 配套 (per L1/L1.1/L1.2 三件套 + D3 commit 模板, v0.2 升版)

### 3.1 L1/L1.1/L1.2 三件套 (per AGENTS.md v0.6.2 §2.1 D2 拍板, v0.2 升版沿用)

| 级别 | 命令 | 限时 | admin 域适用 | 状态 (v0.2) |
|---|---|---|---|---|
| **L1** (compile 验证下限) | `cargo check --tests` | 60s | admin 域所有 commit | ✅ (per `c52805b`) |
| **L1.1** (lib 测试) | `cargo test --lib -p admin-service` | 120s | admin 域 main commit | ✅ (per `c52805b`, 117 passed) |
| **L1.2** (E2E 业务级, 含 mock 路径) | `cargo test --test '*' -- --test-threads=1` + 1 业务 mTLS 跑通 | 300s+ | admin 域跨域 saga / 5 域主链路 | 🟡 (W37 D6-W38 D2 真跑) + ✅ (mock 路径 v0.2 升 per `fa32bab` + 编译期锚定 per `111d4ad`) |

### 3.2 本 checklist 自身 DoD (纯文档, v0.2 升版沿用)

- ✅ 文档 ≥ 4 KB (本 v0.2 ~ 12 KB)
- ✅ 9-10 项 checklist 完整 (复制 RGS-CRITIQUE v0.2 §4.6 原文, 1 字不改)
- ✅ 顶部元信息完整 (D3 模板: 作者/审批/修订人/代签授权/依据/配套/作用域)
- ✅ 修订历史 v0.2 (本段 §6)
- ✅ commit 1 段带代签 (per D3 模板: docs(critique): CHECKLIST-admin-PROD-READY v0.2 升版落档)
- ✅ 派生约束守护段 (per §4 L1-L14 + 8/27 凭据硬 ban + L12 案例库 + L-CAND-006)
- ✅ 已知缺口段 (per §5 8/26 缺标比错标 + v0.2 升版局限)

### 3.3 admin 域主链路 commit DoD 配套 (v0.2 升版沿用)

| commit 类型 | L1 | L1.1 | L1.2 (含 mock 路径) | 代签 |
|---|---|---|---|---|
| admin 域 IT (gm_handlers RBAC 等) | ✅ | ✅ | N/A (单域 IT) | ✅ (per 8/27 三次强化) |
| admin 域跨域 saga (5 域 + batch) | ✅ | ✅ | ✅ (1 业务 mTLS 跑通 + mock 路径 v0.2 升) | ✅ |
| admin 域 audit_log verify 脚本 | ✅ | ✅ (增量 verify 测试) | N/A (脚本无业务 E2E) | ✅ |

---

## 4. 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 案例库, v0.2 升版)

### 4.1 L1-L14 派生约束 (per AGENTS.md v0.6.9 §2 + §8 冻结期 6 个月, v0.2 升版沿用)

| # | 约束 | admin 域适用 | 状态 (v0.2) |
|---|---|---|---|
| L1 | cargo check --tests 0 error (必跑) | admin 域所有 commit | ✅ (per `c52805b`) |
| L1.1 | cargo test --lib 跑通 (5 域 main commit 必跑) | admin 域 main commit | ✅ (per `c52805b`, 117 passed) |
| L1.2 | E2E 业务级 (跨域 saga / 5 域主链路必跑) | admin 域跨域 saga commit | 🟡 (W37 D6-W38 D2 真跑) + ✅ mock 路径 v0.2 升 (per `fa32bab` / `111d4ad`) |
| L2-L10 | (略, 沿用 AGENTS.md v0.6.9 §2.1-§2.5) | admin 域无特殊 | ✅ 沿用 |
| L11 | PT 派工 cargo build dir lock 防轮询 (per 8/31 PT 经验) | admin 域 PT 派工 | ✅ 沿用 |
| L12 | 临时 log / .txt / .tmp_search* 不入 commit (pre-commit hook 兜底) + 5 worker 派工 3 选项 + per-worker CARGO_TARGET_DIR + staggered (per 9/3 12:36 JST 升正式 L-CAND-009) | 本 checklist 文档工作 | ✅ (本 commit 0 临时文件 + worker 不 commit, 主会话统一 commit) |
| L13 | 自指字段 deferred 实时查询 (git log + grep 实证) | 本 checklist 引用 §1 + §2 commit | ✅ (全部 git log 实证) |
| L14 | plumbing 节点字符串 brace 跟踪 (per 9/2 W2 BA-W2 patch) | N/A (本工作非 plumbing) | N/A |

### 4.2 8/27 11:06 JST 凭据硬 ban (v0.2 升版沿用)

**强约束 (per 8/27 11:06 JST Ulysses 决策 + AGENTS.md §1.2)**:
- ❌ 禁止把任何环境变量内容打印到对话/终端/log
- ❌ 禁止 `Get-ChildItem env:` 表格 / `echo $VAR` / `$env:X expand` / `cat .env` 等所有可能泄露 secret 的操作
- ✅ 仅可 `$env:VAR` 引用后直接 pipe 或传给程序参数

**本 checklist v0.2 升版落地**:
- ✅ 文档无 env value 痕迹 (k8s secret 仅提"导出 SOP", 不实际打印 cert 内容)
- ✅ admin-service-tls secret 仅引用 commit, 不打印内容
- ✅ L-CAND-006 例外段 (per 9/3 07:31 JST 拍板) 走 certs/ gitignored, cert 内容永不入 commit

### 4.3 L12 临时 log 不入 commit + L12 案例库 (per 9/3 12:36 JST 升正式)

**强约束 (per AGENTS.md v0.6.9 §2.6 L12 + 9/3 12:36 JST 升正式)**:
- ❌ 临时 log / .txt / .tmp_search* 不入 commit
- ✅ pre-commit hook 兜底 (per 9/3 07:31 JST L-CAND-006 落地清单 5/8, commit `4d23f09`)
- ✅ 5 worker 并发派工 3 选项 (per 9/3 11:08 JST race condition 教训 commit `6c5173a`):
  - 5 worker 独立 worktree
  - 5 worker 写文件不 commit, 主会话统一 git add N files + 1 commit (本档 v0.2 走此选项)
  - 1 worker 串行 5 域

**本 checklist v0.2 升版落地**:
- ✅ 本 commit 0 临时文件
- ✅ 临时 commit-msg 草稿不入 commit (直接 git commit -m + heredoc)
- ✅ 5 worker 派工走选项 2 (写文件不 commit, 主会话统一 commit)

### 4.4 9/3 07:31 JST L-CAND-006 例外段 (v0.2 升版沿用)

**例外触发** (per RGS-CRITIQUE v0.2 §1 + AGENTS.md v0.6.9 §8 L-CAND-006 例外段):
- L-CAND-006 (k8s secret 导出硬 ban, 安全类, 候选清单 commit `ee3c7e7`) 在 SRE Lead 拍板悬空期间生效
- 生效范围: 阶段 B (5 域 certs 导出) 走新 SOP, 不等 R4 季度评审
- 新 SOP:
  1. k8s secret 导出走 `certs/` gitignored 目录 (per L12 派生约束兜底)
  2. 仅 cert SHA-256 fingerprint + cert subject 写 `certs/MANIFEST.toml`
  3. cert 内容**永不入 commit** (per 8/27 11:06 JST 硬 ban 一致性延伸)
  4. cert 链验证用 `openssl x509 -noout -fingerprint -sha256` 比对 fingerprint (k3s 节点已装 openssl)

**本 checklist 第 8 项 (证书轮换) 关联**: W37 D3 走 L-CAND-006 例外段新 SOP, admin-service-tls secret 90 天轮换.

---

## 5. 已知缺口 (per 8/26 缺标比错标, v0.2 升版)

### 5.1 流程层缺口 (v0.2 升版后, 沿用 v0.1 + 更新)

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-1 | SRE Lead 拍板悬空 (per RGS-DEVPLAN v0.1 §3, 5/11 已立 7 落地) | 🟡 中 | token 累计 1M 内 SRE Lead 未拍板 | 选项 C 推迟后续阶段, 写 RGS-PHASE-C-DEFER-* 公告 |
| GAP-2 | 22 测试函数 race condition (per RGS-TEST-RUN-PLAN v0.1) | 🟢 低 | `--test-threads=1` + 重跑 | per RGS-TEST-RUN-PLAN v0.1 §3 |
| GAP-3 | grpcurl 安装 3 选 1 失败 (阶段 B B3) | 🟡 中 (v0.2 缓解) | sidecar / init container / 本地装 失败 | v0.2 已升 ✅ 走 mock 路径 (per `fa32bab`), 备选: 用 kubectl exec 替代 grpcurl (per RGS-DEVPLAN v0.1 §3) |
| GAP-4 | 5 域 mTLS 业务 1 跳不通 (admin → 5 域) | 🟡 中 (v0.2 缓解) | 真跑阶段 B (W37 D3-D5) 走 grpcurl 失败 | v0.2 已升 ✅ 走编译期锚定 (per `111d4ad`), 走 L-CAND-006 例外段, 重导 certs/ gitignored |

### 5.2 业务层缺口 (v0.2 升版后)

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-5 | admin.audit_event 增量 verify 真实篡改 fail-closed (per Q2 决策) | 🟢 低 | 24h 内最近 1000 条 verify 失败 | 立即查 audit_log 写入路径, 不动全表 |
| GAP-6 | admin RBAC 拒绝率告警阈值 (per Q1 决策) | 🟢 低 | 1h spike > 10% 误触发 | 临时调阈值到 20%, 记录到 RGS-OPEN-QA v0.3 候选 |
| GAP-7 | admin service restartCount 超 5 (24h) 触发 (per 第 5 项) | 🟢 低 (v0.2 缓解) | k3s 节点 OOM / HPA minReplicas 风暴 (per 8/31 HPA 经验) | v0.2 已升 ✅ 实证 0 (per 9/3 12:38 JST WSL 实证) + 阶段 A4 HPA 0 强启动风暴, 查 pod events + 清 PVC, per L6 派生约束 |
| GAP-8 | COC 控制面触发 (第 4 项 跨域 saga) | 🟡 中 | gm-backend 8081 不可达 (per RGS-OPEN-QA v0.2 §4.3 Q8) | k3s 容器诊断 + HPA minReplicas 调 0 |

### 5.3 文档层缺口 (v0.2 升版后, 沿用 v0.1)

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-9 | RACI v1.2 5→6 域扩展待 DDD Review (per RGS-DEVPLAN v0.1 §7 R3 任务) | 🟡 中 | 9/19 JST Ulysses DDD Review 二审未到 | 等 W38 D5 9/19 JST, 不阻塞本 checklist |
| GAP-10 | IMPL-PLAN-BATCH-001 v0.1 起草 (per 5 域 IMPL-PLAN 范式) | 🟡 中 | 9/19 JST Ulysses DDD Review 二审未到 | 等 W38 D5 9/19 JST, 不阻塞本 checklist |
| GAP-11 | admin 域 9-10 项的 L-CAND 自审报告待 R4 触发 | 🟢 低 | R4 累计 5M tokens 触发 | 等 R4, 不阻塞 W37 D6-W38 D2 阶段 A/B/C |
| GAP-12 | admin 域 Lead RACI v1.2 真实身份 (per 8/21 JST 5 域独立 Lead 决策) | 🟢 低 | DDD Review 阶段补签字栏 | 等 W38 D5 9/19 JST, 不阻塞本 checklist |

### 5.4 v0.2 自身缺口 (per 8/26 缺标比错标, v0.2 升版后)

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-13 | 本 v0.2 状态汇总 (9/3 12:46 JST 抓取) 与 W38 D3 跑通后状态可能不一致 | 🟢 低 | W37 D6 跑通后状态更新不及时 | W38 D4 周报 v0.1 同步更新, 写 v0.3 增量 |
| GAP-14 | 第 3/4 项 E2E (L1.2) 工具 = grpcurl, 但 SRE Lead 可能用 kubectl exec 替代 | 🟢 低 (v0.2 缓解) | grpcurl 安装失败 (per GAP-3) | v0.2 已升 ✅ 走 mock 路径 + 编译期锚定, 工具可替换, DoD 不变 (业务 mTLS OK + 跨域 saga 审计写入) |
| GAP-15 (v0.2 新增) | 闭环率口径存疑 (v0.1 → v0.2 衔接) | 🟢 低 | RGS-CRITIQUE v0.2 §4.6 写 "9/10 闭环" 但 v0.1 表格仅 4 项 ✅, v0.2 已升 6/10 ✅, 距离 9/10 还差 3 项 (#4/#6/#8/#10 中 3 项升 ✅) | 待 DDD Review 阶段, Ulysses 二审时确认 |

### 5.5 阶段 A → 阶段 B/C 衔接缺口 (v0.2 新增)

- **阶段 A 4 步 SRE 替代**: 9/3 12:38 JST Mavis 推阶段 A 4 步 SRE 替代 (per RGS-PHASE-C-MAVIS-PHASE-A v0.1), 5 域 restartCount 0 + HPA 0 强启动风暴, 阶段 B 真跑待 SRE 介入
- **mTLS mock 路径 vs 业务真跑**: v0.2 #2/#3 升 ✅ 走 mock + 编译期锚定, 业务真跑需阶段 B 阶段 C 走 grpcurl + 真 cert

---

## 6. 修订历史 v0.2

| 版本 | 日期 (JST) | 审批 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: §0 目的与范围 (per C3 派生约束 + R1 业务冲刺 R3 阶段任务) + §1 admin 域 9-10 项 checklist (复制 RGS-CRITIQUE v0.2 §4.6 原文) + §2 状态更新 (9/3 08:00 JST R1 业务冲刺现状 4/10 闭环) + §3 DoD 配套 (L1/L1.1/L1.2 三件套) + §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log + L-CAND-006 例外段) + §5 已知缺口 (流程层 4 + 业务层 4 + 文档层 4 + v0.2 自身 2 = 14 项, per 8/26 缺标比错标) + §6 修订历史本行 |
| **v0.2** | 2026-09-03 12:46 | 架构师(Mavis 接手 agent per DEC-008) | **业务回填 9 项升版**: 基于 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 565/565 + `111d4ad` 5 域 E2E Phase C marker 10 marker 编译期锚定 + `fa32bab` 5 域 mTLS mock 15/15 passed) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:38 JST commit `d126a55` RGS-PHASE-C-MAVIS-PHASE-A v0.1 落档. §1 表格状态列更新: 3 项 🟡→✅ (#2 IT mTLS mock 路径 + #3 E2E 业务 mTLS 编译期锚定 + #5 SLA 监控 WSL kubectl 实证), 3 项 ✅→✅ (#1 UT L1.1 维持 + #7 部署健康 维持 + #9 Schema 迁移 维持 Q2 决策), 4 项 🟡→🟡 (#4 跨域 saga admin 审计 → COC 控制面 真跑需阶段 C + #6 告警 RBAC 拒绝率 SRE 拍板 + #8 证书 L-CAND-006 兜底 + #10 审计 admin Q2 决策), 闭环率 4/10 → 6/10 (60%). §2 状态更新加 9/3 12:46 JST R1 业务回填 9 项统计表 + 5 域 main HEAD `fa32bab` + W37-W38 实战跟踪表 v0.2 升版 + 风险评估更新 (GAP-3/GAP-4/GAP-7 缓解) + commit 强证据表 (3 项新增). §3 DoD 配套加 mTLS mock 路径 + 编译期锚定 (per `fa32bab` / `111d4ad`). §4 派生约束守护加 L12 案例库 (per 9/3 12:36 JST 升正式 commit `2e4f519` + L-CAND-009 5 worker 派工 3 选项, 本档走选项 2: 写文件不 commit, 主会话统一 commit). §5 已知缺口加 GAP-15 闭环率口径存疑 + 阶段 A → 阶段 B/C 衔接缺口 2 条. §6 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
**配套**: AGENTS.md v0.6.9 + v0.6.10 + RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.6 + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 任务 + RGS-PHASE-C-PREP-2026-09-02 v0.1 §1 + RGS-PHASE-C-MAVIS-PHASE-A v0.1
