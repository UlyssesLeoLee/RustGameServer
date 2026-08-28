# DDD Review Summary 2026-08-28

> **一页式**:Ulysses 一次性审阅用
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 17:09 JST)

---

## 1. 进度 (current state)

main HEAD: **`16460a4`** — 已 push origin

**已完成 8 commits** (since 2026-08-28 12:00 JST):
| commit | 标题 |
|---|---|
| `c5c9f5f` | S4 Phase 1 gm.proto 编译实装 |
| `acd0454` | S5 outbox NATS mock IT + 7 mock 测试 |
| `73bcb19` | BAS×TST 覆盖审计 44/44 = 100% |
| `de86d80` | 6 域独立 UT + 跨域回归 + 旧债决策 |
| `3c7d670` | 测试结果核对 + test-evidence v4 |
| `404e3ea` | TBD-08 UT-08 模块 D + 4 域 PFAU |
| `d34e2d7` | 8 域 Lead 具名 + cluster-ops 旧债终方案 + OPEN-QA v0.3 |
| `580cde3` / `4c8c7f9` / `df986ec` | IT 准入 + 代签 + 01-07 域 IT 补全 |
| `a227e0c` | 跨反馈 9 条 (F1-F9) 处置 |
| `94ba812` | UT-09 rgs-certgen 17 测试 |
| `1790b18` | G3 fixture 修复 (sqlx leo19 + 3 非 fixture) |
| `1c2bf91` | G3+G4 evidence 落档 (81/81 PASS, 75.9% 覆盖) |
| `d023594` | S4 Phase 2 step 1 (admin-service gRPC client 注入) |
| `16460a4` | S4 Phase 2 step 1 设计文档 |

**当前 DDD Review 状态**:⏳ 9 决策草案就绪 (5 已实装, 3 worktree W1/W2/W3 启动但 worker 模式未真跑, 1 OPEN-QA v0.4 待升级)

---

## 2. 关键决策 (per Ulysses 2026-08-27 12:43 JST 指令)

1. **gm-backend 作为第 8 域微服务** (per BAS-003 §2.1 APIGW 角色)
2. **8 域 Lead 独立, 拒绝兼任** (per DEC-005 + 8/21 JST 决议)
3. **AI 协作下用 token 不用人·天算 OLU** (per 8/21 JST 决议)
4. **代签允许 Mavis 接手 Ulysses** (per 8/26 04:30 + 19:39/20:56/21:59 三次强化)
5. **环境变量禁止打印, 只可 invoke** (per 8/27 11:06 JST hard ban)
6. **S4 Phase 2 step 1 = HealthView 调 admin-service gRPC + 4 endpoint 保留 stub** (per 16:32 JST)
7. **G3 fixture 用 postgres-superuser 共享** (per 16:11 JST sqlx 0.8.6 leo19 根因诊断)

---

## 3. 跑测 (G3 + G4 + gm-backend)

### G3 跑测 (workspace, commit `1c2bf91`)
- **81/81 targets PASS, 0 fail**
- 663/663 test cases PASS, 37 ignored (Cloudflare PH-5 opt-in)
- evidence: `docs/00-基准与治理/.test-evidence/g3-g4-20260828-070349/`

### G4 覆盖率 (commit `1c2bf91`)
- **Workspace line coverage: 75.9%** (8829/11639 行)
- **14/14 域 ≥ 60%** (rgs-hello 空 stub 0% 除外)
- TOP: rgs-arc-olu 100% / rgs-certgen 95.5% / rgs-testkit 93% / gm-backend 91.2%
- MIN: match-service 62.2%

### gm-backend (commit `d023594`)
- **49/49 PASS, 0 fail** (上轮 36 + 13 含 6 IT)
- 含 JWT/audit/outbox NATS mock/admin gRPC/5 endpoint

### 9 域累计
- **324/324 PASS** (workspace 整合, 含 gm-backend 49)

---

## 4. 文档 (18 份 TST + 19 份 IT + 35 份 BAS)

### TST 文档 (18 份)
- **RGS-TST-00~09 UT** (10 份): 9 域 UT + UT-00 总览
- **RGS-TST-00~09 IT** (9 份): 9 域 IT + IT-00 v0.2
- **44/44 BAS 引用 100%** (commit `73bcb19`)

### BAS 文档 (35 份)
- 全 9 域 + cluster-ops + gm-backend 覆盖
- 7 域 BAS × TST 双向引用

### 治理文档
- OPEN-QA v0.3 (Q2/Q4/Q7 推进)
- 9 决策草案 + 跨反馈 9 条处置
- 代签审核 17/18 PASS
- AI 审计提示词 (9 维度 + 10 重点)
- test-evidence.ps1 v4
- g3-g4-runner.sh v3 (port-forward 15432 + superuser)
- db-url.sh / db-connect-check.py / extract-coverage.ps1

---

## 5. 下一步 (per WBS)

### 即时 (W5)
- ⏳ 4 worktree W1/W2/W3 worker 启动后无 commit — Ulysses 决策:是否让我直接实装?
- ⏳ W4 (本 worktree) commit DDD Review checklist + summary
- ⏳ merge 4 branch → main, 重测 81/81 + 49/49

### 9 月初 (W6)
- BAS 章节级追溯 35 份 → IT 文档 (80-120M tokens)
- 1 文件/周 × 7 周

### 9 月中 (W7)
- gm-backend 5 GM RPC 业务实装 (BanAccount/Compensation/Maintenance/AuditLog)
- 端到端跑通 5 域 + gm-backend (60-100M tokens)

### 9 月末 (W8)
- PH-1 OTel 全链路 sqlx-tracing sample 10-20%
- 5 域 + gm-backend + cluster-ops + shared-platform + NATS (50-80M tokens)

---

## 6. 风险 / 已知缺口 (per DDD Review checklist Section F)

- **P0**: W1/W2/W3 worker 模式不可靠 (4 worker 全 succeed 但 worktree 无 commit)
- **P1**: 3 cluster-ops 旧债 P3 follow-up / S4 Phase 2 step 3+ / mTLS 决策
- **P2**: JWT propagation / match-service 62% 覆盖提升 / W2 5 类链路
- **P3**: OPEN-QA 模板固定化 / AI 审计 CI hook / rgs-hello stub

---

## 7. 决策点 (需 DDD Review 拍板)

1. **OPEN-QA v0.3 → v0.4** 升级 (Q2/Q4/Q7 resolved, 模板固定化?)
2. **8 域 Lead 12 角色** 边界 + RACI 矩阵
3. **cluster-ops 终方案 A'** P3 3 文件优先级
4. **S4 Phase 2 step 1** 49/49 接受
5. **TBD-08-06 工具决策 D** 5 域是否切 axum-test
6. **W1/W2/W3 worktree** 让我直接实装 or 保持待审?
7. **AI 审计提示词** 进 PR automation?

预计 DDD Review 时长: 60-90 min
