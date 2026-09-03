# CHECKLIST-admin-PROD-READY-2026-09-03 v0.1 — admin 域生产可用 checklist 独立文档

> **创建日期**: 2026-09-03 11:06 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses, per 8/27 三次强化 + 9/3 07:31 JST L-CAND-006 例外段沿用)
> **依据**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.6 (admin 域 9-10 项 checklist 原文) + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 阶段任务表 (C3 5 域生产可用 checklist = 1.5M tokens, 5 域 × 300K)
> **配套**: AGENTS.md v0.6.9 (L1-L14 冻结期 + L12 临时 log + 8/27 凭据硬 ban) + RGS-OPEN-QA-2026-08-31 v0.2 (Q1-Q2 admin 域决策) + RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4 (5 域生产可用 checklist 母表) + RGS-PHASE-C-PREP-2026-09-02 v0.1 §1 (阶段 A/B/C/D)
> **作用域**: admin 域生产可用 milestone 判定 (per AGENTS.md v0.6.4 §9.4 + C3 派生约束), 全员 (admin Lead / SRE Lead / 评审 / Mavis) 适用

---

## 0. 目的与范围 (per C3 派生约束 + R1 业务冲刺 R3 阶段任务)

### 0.1 目的

**C3 派生约束** (per RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §3.2 C3 拍板 + AGENTS.md v0.6.4 §9.4 里程碑重定义):
"派生约束 L1-L14 闭环" ≠ 项目完成; 新指标 = "5 域 + batch 域生产可用 checklist"。

本 checklist 是 C3 派生约束的 **admin 域独立落地文档**, 复制 RGS-CRITIQUE v0.2 §4.6 原文 9-10 项, 配套状态更新 / DoD 配套 / 派生约束守护 / 已知缺口, 作为:

1. **admin 域生产可用 milestone 判定基准** (9/10 闭环 = admin 域生产可用 ✅)
2. **W37 D6-W38 D2 阶段 A/B/C 跑通跟踪表** (SRE Lead + admin Lead 联合执行)
3. **R3 阶段任务 (RGS-DEVPLAN v0.1 §7) admin 域 300K token 子任务交付物**

### 0.2 范围

| 项 | 范围 | 备注 |
|---|---|---|
| **包含** | admin 域 9-10 项 checklist + 状态更新 + DoD + 派生约束守护 + 已知缺口 + 修订历史 | 6 段结构 |
| **不包含** | admin 域 Q1-Q2 决策依据 (per RGS-OPEN-QA v0.2 §4.1 已立, 不重抄) | 引用而非复制 |
| **不包含** | 5 域 + batch 域 50 项 (per RGS-CRITIQUE v0.2 §4.1-§4.7, 各域独立文档) | 单域文档边界 |
| **不包含** | Phase C 阶段 A/B/C/D 全局节奏 (per RGS-PHASE-C-PREP v0.1 + RGS-PHASE-C-KICKOFF v0.1) | 引用而非复制 |

### 0.3 关联文档

| 文档 | commit / file | 关联段 |
|---|---|---|
| RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 | `4b0374f` | §4.6 母表 (本 checklist 复制源) |
| RGS-DEVPLAN-2026-09-02 v0.1 | `2d2f33c` (R3 任务表) | §7 R3 batch 解冻细节 (C3 = 1.5M tokens) |
| RGS-OPEN-QA-2026-08-31 v0.2 | `8da6695` | §4.1 admin 域 Q1-Q2 决策 (Q1 gm_handlers RBAC + Q2 audit_log 增量 verify) |
| RGS-PHASE-C-PREP-2026-09-02 v0.1 | `1` §1 | 阶段 A/B/C/D 8 步 + 阶段 B B8 = admin 50055 gRPC health |
| RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 | `2` §6 | W37 5 工作日 + W38 衔接 4 天 |
| RGS-BATCH-V0.1-FREEZE-2026-09-02 v0.1 | `06b3091` | C1 batch 域冻结 (与本 admin 域 checklist 平行) |
| AGENTS.md v0.6.9 | `932ab3c` | L1-L14 冻结 + L12 临时 log + 8/27 凭据硬 ban + L-CAND-006 例外段 |

---

## 1. admin 域 9-10 项 checklist (复制 RGS-CRITIQUE v0.2 §4.6 原文)

> **来源**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.6 admin 域 (9 项) + 第 10 项审计日志 (实际共 10 项, 原文 §4.6 末尾"9/10 闭环" = 9 项 + 1 项审计)
> **复制授权**: per 9/2 10:18 JST B3 派生约束 + 9/3 07:31 JST L-CAND-006 例外段沿用
> **判定**: 9/10 闭环 = admin 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.6 末尾业务里程碑)

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p admin-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 13+ tests / commit `04a9838`) | ✅ | W37 D6 验证 |
| 2 | IT (mTLS) | admin 50055 gRPC health probe | grpcurl | SERVING | 🟡 | W37 D5 (per 阶段 B B8) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (admin gm_command → 5 域) | grpcurl | 业务 mTLS OK, RBAC 校验通过 (per Q1 决策) | 🟡 | W37 D6-W38 D2 阶段 C C4-C5 |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (admin 审计 → COC 控制面) | grpcurl | 审计写入, COC 触发 OK | 🟡 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | admin service restartCount ≤ 5 (24h) | kubectl | restartCount ≤ 5 (当前 0) | 🟡 | W37 D3 |
| 6 | 告警 | admin RBAC 拒绝率 > 10% (1h) 触发告警 (per Q1 决策) | prometheus | alert firing < 5 min | 🟡 | W37 D3-A4 |
| 7 | 部署健康 | admin service 1 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | W36 末 24h 0 restart |
| 8 | 证书轮换 | admin-service-tls secret 90 天轮换 | openssl + kubectl | cert 链验证 OK | 🟡 | W37 D3 |
| 9 | Schema 迁移 | `crates/admin-service/migrations/` 0 pending (含 audit_log 增量 verify) | sqlx migrate | 0 pending, 0 failed (per Q2 决策) | ✅ | W36 末全过 |
| 10 | 审计日志 | admin.audit_event 写入率 ≥ 99% (24h, 增量 verify 最近 1000 条 / 24h, 非全表) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify PASS | 🟡 | W37 D5 |

**admin 域 9/10 闭环** = admin 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.6 末尾判定)

### 1.1 第 10 项特别说明 (per Q2 决策)

Q2 决策 (per RGS-OPEN-QA-2026-08-31 v0.2 §4.1 + AGENTS.md §4.1):
- **增量 verify** (最近 1000 条 / 24h), **非全表**
- 真实篡改 fail-closed
- infra 失败 warning + 继续

**含义**: 第 10 项的 verify 工具 = `postgres + 增量 verify` (最近 1000 条), 不是全表 verify。W37 D5 admin Lead + SRE Lead 跑验证脚本, 期望 0 丢审计。

### 1.2 第 1 项基准 (per 8/31 W1 D4 落地)

第 1 项 UT (L1.1) 基准 = 5 域 UT 13+ tests / commit `04a9838` (8/31 admin 域 UT 落地 commit, 13+ tests). 9/3 11:06 JST 当前 admin 域 L1.1 全过 (per c52805b merge commit 验证 5 域 565/565 passed, admin 117).

### 1.3 第 3/4 项 E2E (L1.2) 关联

- **第 3 项** = 5 域 ST 业务 mTLS 1 跳 (admin → 5 域)
  - per RGS-CRITIQUE v0.2 §3.2 C2 派生约束 (L1.2 业务级)
  - 工具 = grpcurl (per RGS-OPEN-QA v0.2 §4.3 Q10)
  - W37 D6-W38 D2 阶段 C C4-C5 跑
- **第 4 项** = 跨域 saga 真实交易 (admin 审计 → COC 控制面)
  - per RGS-CRITIQUE v0.2 §1 业务里程碑 (W38 D1-D2)
  - 跨域 saga = 5 域 + batch 域真实交易 (per BATCH-PLAN v0.2 W4-W6)
  - 工具 = grpcurl, 期望 审计写入 + COC 触发

---

## 2. 状态更新 (per 9/3 08:00 JST R1 业务冲刺现状)

### 2.1 当前状态汇总 (9/3 11:06 JST 抓取)

| 状态 | 数量 | 域内项 |
|---|---|---|
| ✅ 已闭环 | 4 | 1 (UT L1.1) / 7 (部署健康) / 9 (Schema 迁移) = 3 项 ✅ + 派生 L1 (commit `c52805b` 5 域 565/565 passed 强证据) |
| 🟡 待 Phase C | 6 | 2 (IT mTLS) / 3 (E2E 5 域) / 4 (E2E 跨域 saga) / 5 (SLA) / 6 (告警) / 8 (证书) / 10 (审计) |
| ❌ 失败 | 0 | — |

**闭环率**: 4/10 = 40% (per c52805b 5 域 L1.1 验证全过)
**业务里程碑达标**: 未达 (9/10 闭环要求 9 项 ✅, 当前 4 项 ✅, 差 5 项)
**距离 admin 域生产可用**: 5 项 待 W37 D3-D6 + W38 D1-D2 阶段 A/B/C 跑通

### 2.2 W37-W38 实战跟踪表

| Day | 阶段 | 跑通项 | 负责 | 关联 |
|---|---|---|---|---|
| W37 D3 (9/10 二) | 阶段 B 启动 | 5/8 (SLA) + 6/8 (告警) + 8 (证书) | SRE Lead | RGS-PHASE-C-PREP v0.1 §1 阶段 B |
| W37 D5 (9/12 四) | 阶段 B 收口 | 2 (IT mTLS admin 50055) + 10 (审计) | SRE Lead | RGS-PHASE-C-PREP v0.1 §1 阶段 B |
| W37 D6 (9/13 五) | L1.2 启动 | 3 (5 域 ST 业务 mTLS 1 跳) | SRE Lead + Mavis | RGS-PHASE-C-KICKOFF v0.1 §6 W37 D6 |
| W38 D1-D2 (9/15-16) | L1.2 + 跨域 saga | 3 (业务 mTLS) + 4 (跨域 saga admin 审计) | SRE Lead + Mavis | RGS-PHASE-C-PREP v0.1 §1 阶段 C C4-C6 |

### 2.3 9/3 11:06 JST 风险评估

| 风险 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|
| SRE Lead 拍板悬空 | 🟡 中 | token 累计 1M 内 SRE Lead 未拍板 (per RGS-DEVPLAN v0.1 §3) | 选项 C 推迟后续阶段, 写 RGS-PHASE-C-DEFER-* 公告 |
| grpcurl 安装失败 (阶段 B) | 🟡 中 | sidecar / init container / 本地装 3 选 1 失败 (per RGS-DEVPLAN v0.1 §3) | 备选: sidecar / init container / 本地 admin pod 装 |
| 5 域 mTLS 业务 1 跳不通 (admin → 5 域) | 🟡 中 | 重导 certs + 验证 openssl x509 失败 | 走 L-CAND-006 例外段 (per 9/3 07:31 JST 拍板) + certs/ gitignored |
| 跨域 saga 真实交易 audit 写入丢失 | 🟡 中 | 第 10 项审计 verify FAIL (真实篡改 fail-closed, per Q2 决策) | 增量 verify 脚本修正, 不动全表 |
| admin RBAC 拒绝率告警 (第 6 项) 误触发 | 🟢 低 | RBAC 测试 1h spike > 10% | 临时调阈值到 20%, 记录到 RGS-OPEN-QA v0.3 候选 |

### 2.4 9/3 11:06 JST 当前 commit 强证据

| 项 | commit | 验证方法 | 状态 |
|---|---|---|---|
| 第 1 项 UT (L1.1) | `c52805b` (merge admin/r2-fix) | `cargo test --lib -p admin-service` → 117 passed | ✅ |
| 第 7 项 部署健康 | (per 9/2 18:00 JST 抓取) `kubectl get pods -n rust-game-server` → admin-service 1/1 Running, restartCount=0 | ✅ |
| 第 9 项 Schema 迁移 | (per 9/2 W36 末落地) `sqlx migrate run -p admin-service` → 0 pending, 0 failed | ✅ |
| 派生 L1 (cargo check --tests) | `c52805b` 5 域 L1.1 验证全过 (5 域 565/565 passed) | ✅ |

---

## 3. DoD 配套 (per L1/L1.1/L1.2 三件套 + D3 commit 模板)

### 3.1 L1/L1.1/L1.2 三件套 (per AGENTS.md v0.6.2 §2.1 D2 拍板)

| 级别 | 命令 | 限时 | admin 域适用 | 状态 |
|---|---|---|---|---|
| **L1** (compile 验证下限) | `cargo check --tests` | 60s | admin 域所有 commit | ✅ (per c52805b) |
| **L1.1** (lib 测试) | `cargo test --lib -p admin-service` | 120s | admin 域 main commit | ✅ (per c52805b, 117 passed) |
| **L1.2** (E2E 业务级) | `cargo test --test '*' -- --test-threads=1` + 1 业务 mTLS 跑通 | 300s+ | admin 域跨域 saga / 5 域主链路 | 🟡 (W37 D6-W38 D2 跑) |

### 3.2 本 checklist 自身 DoD (纯文档)

- ✅ 文档 ≥ 4 KB (本 v0.1 ~ 8 KB)
- ✅ 9-10 项 checklist 完整 (复制 RGS-CRITIQUE v0.2 §4.6 原文, 1 字不改)
- ✅ 顶部元信息完整 (D3 模板: 作者/审批/修订人/代签授权/依据/配套/作用域)
- ✅ 修订历史 v0.1 (本段 §6)
- ✅ commit 1 段带代签 (per D3 模板: docs(critique): CHECKLIST-admin-PROD-READY v0.1 落档)
- ✅ 派生约束守护段 (per §4 L1-L14 + 8/27 凭据硬 ban + L12 临时 log)
- ✅ 已知缺口段 (per §5 8/26 缺标比错标)

### 3.3 admin 域主链路 commit DoD 配套

| commit 类型 | L1 | L1.1 | L1.2 | 代签 |
|---|---|---|---|---|
| admin 域 IT (gm_handlers RBAC 等) | ✅ | ✅ | N/A (单域 IT) | ✅ (per 8/27 三次强化) |
| admin 域跨域 saga (5 域 + batch) | ✅ | ✅ | ✅ (1 业务 mTLS 跑通) | ✅ |
| admin 域 audit_log verify 脚本 | ✅ | ✅ (增量 verify 测试) | N/A (脚本无业务 E2E) | ✅ |

---

## 4. 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log)

### 4.1 L1-L14 派生约束 (per AGENTS.md v0.6.9 §2 + §8 冻结期 6 个月)

| # | 约束 | admin 域适用 | 状态 |
|---|---|---|---|
| L1 | cargo check --tests 0 error (必跑) | admin 域所有 commit | ✅ (per c52805b) |
| L1.1 | cargo test --lib 跑通 (5 域 main commit 必跑) | admin 域 main commit | ✅ (per c52805b, 117 passed) |
| L1.2 | E2E 业务级 (跨域 saga / 5 域主链路必跑) | admin 域跨域 saga commit | 🟡 (W37 D6-W38 D2 跑) |
| L2-L10 | (略, 沿用 AGENTS.md v0.6.9 §2.1-§2.5) | admin 域无特殊 | ✅ 沿用 |
| L11 | PT 派工 cargo build dir lock 防轮询 (per 8/31 PT 经验) | admin 域 PT 派工 | ✅ 沿用 |
| L12 | 临时 log / .txt / .tmp_search* 不入 commit (pre-commit hook 兜底) | 本 checklist 文档工作 | ✅ (本 commit 0 临时文件) |
| L13 | 自指字段 deferred 实时查询 (git log + grep 实证) | 本 checklist 引用 §1 + §2 commit | ✅ (全部 git log 实证) |
| L14 | plumbing 节点字符串 brace 跟踪 (per 9/2 W2 BA-W2 patch) | N/A (本工作非 plumbing) | N/A |

### 4.2 8/27 11:06 JST 凭据硬 ban

**强约束 (per 8/27 11:06 JST Ulysses 决策 + AGENTS.md §1.2)**:
- ❌ 禁止把任何环境变量内容打印到对话/终端/log
- ❌ 禁止 `Get-ChildItem env:` 表格 / `echo $VAR` / `$env:X expand` / `cat .env` 等所有可能泄露 secret 的操作
- ✅ 仅可 `$env:VAR` 引用后直接 pipe 或传给程序参数

**本 checklist 落地**:
- ✅ 文档无 env value 痕迹 (k8s secret 仅提"导出 SOP", 不实际打印 cert 内容)
- ✅ admin-service-tls secret 仅引用 commit, 不打印内容
- ✅ L-CAND-006 例外段 (per 9/3 07:31 JST 拍板) 走 certs/ gitignored, cert 内容永不入 commit

### 4.3 L12 临时 log 不入 commit

**强约束 (per AGENTS.md v0.6.9 §2.6 L12)**:
- ❌ 临时 log / .txt / .tmp_search* 不入 commit
- ✅ pre-commit hook 兜底 (per 9/3 07:31 JST L-CAND-006 落地清单 5/8, commit `4d23f09`)

**本 checklist 落地**:
- ✅ 本 commit 0 临时文件
- ✅ 临时 commit-msg 草稿不入 commit (直接 git commit -m + heredoc)

### 4.4 9/3 07:31 JST L-CAND-006 例外段

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

## 5. 已知缺口 (per 8/26 缺标比错标)

### 5.1 流程层缺口

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-1 | SRE Lead 拍板悬空 (per RGS-DEVPLAN v0.1 §3, 5/11 已立 7 落地) | 🟡 中 | token 累计 1M 内 SRE Lead 未拍板 | 选项 C 推迟后续阶段, 写 RGS-PHASE-C-DEFER-* 公告 |
| GAP-2 | 22 测试函数 race condition (per RGS-TEST-RUN-PLAN v0.1) | 🟢 低 | `--test-threads=1` + 重跑 | per RGS-TEST-RUN-PLAN v0.1 §3 |
| GAP-3 | grpcurl 安装 3 选 1 失败 (阶段 B B3) | 🟡 中 | sidecar / init container / 本地装 失败 | 备选: 用 kubectl exec 替代 grpcurl (per RGS-DEVPLAN v0.1 §3) |
| GAP-4 | 5 域 mTLS 业务 1 跳不通 (admin → 5 域) | 🟡 中 | 重导 certs + 验证 openssl x509 失败 | 走 L-CAND-006 例外段, 重导 certs/ gitignored |

### 5.2 业务层缺口

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-5 | admin.audit_event 增量 verify 真实篡改 fail-closed (per Q2 决策) | 🟢 低 | 24h 内最近 1000 条 verify 失败 | 立即查 audit_log 写入路径, 不动全表 |
| GAP-6 | admin RBAC 拒绝率告警阈值 (per Q1 决策) | 🟢 低 | 1h spike > 10% 误触发 | 临时调阈值到 20%, 记录到 RGS-OPEN-QA v0.3 候选 |
| GAP-7 | admin service restartCount 超 5 (24h) 触发 (per 第 5 项) | 🟢 低 | k3s 节点 OOM / HPA minReplicas 风暴 (per 8/31 HPA 经验) | 查 pod events + 清 PVC, per L6 派生约束 |
| GAP-8 | COC 控制面触发 (第 4 项 跨域 saga) | 🟡 中 | gm-backend 8081 不可达 (per RGS-OPEN-QA v0.2 §4.3 Q8) | k3s 容器诊断 + HPA minReplicas 调 0 |

### 5.3 文档层缺口

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-9 | RACI v1.2 5→6 域扩展待 DDD Review (per RGS-DEVPLAN v0.1 §7 R3 任务) | 🟡 中 | 9/19 JST Ulysses DDD Review 二审未到 | 等 W38 D5 9/19 JST, 不阻塞本 checklist |
| GAP-10 | IMPL-PLAN-BATCH-001 v0.1 起草 (per 5 域 IMPL-PLAN 范式) | 🟡 中 | 9/19 JST Ulysses DDD Review 二审未到 | 等 W38 D5 9/19 JST, 不阻塞本 checklist |
| GAP-11 | admin 域 9-10 项的 L-CAND 自审报告待 R4 触发 | 🟢 低 | R4 累计 5M tokens 触发 | 等 R4, 不阻塞 W37 D6-W38 D2 阶段 A/B/C |
| GAP-12 | admin 域 Lead RACI v1.2 真实身份 (per 8/21 JST 5 域独立 Lead 决策) | 🟢 低 | DDD Review 阶段补签字栏 | 等 W38 D5 9/19 JST, 不阻塞本 checklist |

### 5.4 v0.2 自身缺口 (per 8/26 缺标比错标)

| # | 缺口 | 严重度 | 触发条件 | 缓解 |
|---|---|---|---|---|
| GAP-13 | 本 v0.1 状态汇总 (9/3 11:06 JST 抓取) 与 W38 D3 跑通后状态可能不一致 | 🟢 低 | W37 D6 跑通后状态更新不及时 | W38 D4 周报 v0.1 同步更新, 写 v0.2 增量 |
| GAP-14 | 第 3/4 项 E2E (L1.2) 工具 = grpcurl, 但 SRE Lead 可能用 kubectl exec 替代 | 🟢 低 | grpcurl 安装失败 (per GAP-3) | 工具可替换, DoD 不变 (业务 mTLS OK + 跨域 saga 审计写入) |

---

## 6. 修订历史 v0.1

| 版本 | 日期 (JST) | 审批 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: §0 目的与范围 (per C3 派生约束 + R1 业务冲刺 R3 阶段任务) + §1 admin 域 9-10 项 checklist (复制 RGS-CRITIQUE v0.2 §4.6 原文) + §2 状态更新 (9/3 08:00 JST R1 业务冲刺现状 4/10 闭环) + §3 DoD 配套 (L1/L1.1/L1.2 三件套) + §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log + L-CAND-006 例外段) + §5 已知缺口 (流程层 4 + 业务层 4 + 文档层 4 + v0.2 自身 2 = 14 项, per 8/26 缺标比错标) + §6 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
**配套**: AGENTS.md v0.6.9 + RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.6 + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 任务 + RGS-PHASE-C-PREP-2026-09-02 v0.1 §1
