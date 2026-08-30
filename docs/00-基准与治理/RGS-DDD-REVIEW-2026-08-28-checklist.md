# DDD Review Checklist 2026-08-28

> **目的**:Ulysses 一次性审 9 决策草案 + 跨反馈处置 + S4 Phase 2 step 1+2 + 跑测 + 文档治理
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 17:09 JST)
> **状态**:⏳ OPEN (待 Ulysses DDD Review 拍板)
> **关联**:RGS-OPEN-QA v0.4 (推进中) + 9 决策草案 + 8 commits (c14d49b → 38097e8)

---

## Section A: 9 决策草案提交清单

### A1. 8 域 Lead 具名 (Q2 OPEN-QA)
- **文档**: `docs/00-基准与治理/RGS-LEAD-NAMING-8-域-2026-08-28.md`
- **决策**: 采纳 8 域 + 4 共享 = 12 角色 (player/economy/match/social/admin/cluster-ops/gm-backend/rgs-certgen + SRE/Platform/QA/PM)
- **已实装**: commit `12437ca`
- **已知缺口**: SRE Lead/平台/评审/PM 4 域 Lead 仍 ⏳ 不代签 (per 8/21 JST "拒绝兼任")
- **需 DDD 拍板**: 8 域 + 4 共享角色边界是否清晰? 是否需补 RACI 矩阵?

### A2. cluster-ops 终方案 A' (Q7)
- **文档**: `docs/00-基准与治理/RGS-TST-CLUSTER-OPS-OLD-DEBT-终方案决策.md`
- **决策**: git rm `tests-disabled/ut_state_machine.rs` + 3 文件 P3 follow-up
- **已实装**: commit `3e8d9ca` (per worktree main)
- **已知缺口**: 3 文件 P3 follow-up 仍未实装 (per 9 月计划)
- **需 DDD 拍板**: P3 优先级是否可推后到 9 月?

### A3. TBD-08-06 工具决策 D
- **文档**: `docs/00-基准与治理/RGS-TST-08-06-axum-test-vs-wiremock-工具决策.md`
- **决策**: 双工具并存 (7 域 wiremock 0.6 + 8 域 axum-test 16)
- **已实装**: 7 域 + 8 域 IT 文档齐全 (per commit `90aa3df`)
- **已知缺口**: 5 域 + gm-backend 是否需统一? 待观察
- **需 DDD 拍板**: 5 域是否也切 axum-test? (gm-backend 已用)

### A4. S4 Phase 2 step 1 (gm-backend admin-service gRPC client)
- **文档**: `docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md`
- **决策**: gm-backend 注入 tonic Channel + HealthView 调 admin-service gRPC + 失败降级
- **已实装**: commit `11a230a` + 设计 `38097e8` (49/49 PASS)
- **已知缺口**: 4 endpoint (ban/grant/maintenance/query_audit) 仍 stub, 待 Step 2
- **需 DDD 拍板**: Step 2 启动时机 (本 worktree 或下批次)?

### A5. S4 Phase 2 step 2 (admin-service 5 GM RPC)
- **文档**: ⏳ (待 worktree `it/s4-phase2-step2` 完成后生成)
- **决策**: admin.proto 加 4 RPC (BanAccount/GrantCompensation/SetMaintenance/QueryAuditLog) + gm-backend 4 endpoint 接通 + 失败降级
- **已实装**: ⏳ pending (worktree W1 启动但未出结果)
- **需 DDD 拍板**: 实装完是否先 DDD Review 再 push main?

### A6. W2 跨域 IT 链路用例
- **文档**: ⏳ (待 worktree `it/w2-cross-domain` 完成后生成)
- **决策**: 5 类跨域链路 + 至少 3 IT 用例 (cluster-ops ↔ 5 域 / cluster-ops ↔ admin / gm-backend → admin → 5 域 / gm-backend → admin → economy / cluster-ops ↔ gm-backend)
- **已实装**: ⏳ pending (worktree W2 启动但未出结果)
- **需 DDD 拍板**: 跨域链路优先级 (W2 内部用例排序)?

### A7. W4 S5 §3 真 NATS e2e
- **文档**: ⏳ (待 worktree `it/w4-s5-nats` 完成后生成)
- **决策**: k3s nats-0 port-forward + 7 真链路测试 (拉取/publish/ack/nack/并发/lease 过期/retry)
- **已实装**: ⏳ pending (worktree W3 启动但未出结果)
- **已知缺口**: mock 7/7 已 PASS, 真 NATS e2e 需 k3s NATS 就绪
- **需 DDD 拍板**: 真 NATS e2e vs mock 共存策略?

### A8. AI 审计提示词
- **文档**: `docs/00-基准与治理/AI-AUDIT-PROMPT-Mavis-2026-08-28.md` (9,489 字节)
- **决策**: 9 维度 + 10 重点核查项 (per 代码治理 + 决策追踪 + 测试设计 + 文档治理)
- **已实装**: 落档, 未集成到 CI hook
- **需 DDD 拍板**: 是否集成到 PR 自动化? (ci.yml 加 step?)

### A9. OPEN-QA v0.3 → v0.4 推进
- **文档**: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.1.md` (本 worktree 升级)
- **决策**: Q2 / Q4 / Q7 全部 resolved
- **已实装**: ⏳ pending (本 worktree 推进中)
- **需 DDD 拍板**: OPEN-QA 模板是否固定化 (per 后续每次迭代)?

---

## Section B: 跨反馈处置 9 条 (F1-F9)

**文档**: `docs/00-基准与治理/RGS-TST-PEERREVIEW-2026-08-28-feedback-handling.md` (commit `43a2e08`)

| 反馈 | 内容 | 处置 | 状态 |
|---|---|---|---|
| F1 | DTL-040 根因诊断新证据 | Q4 OPEN-QA 推进 + 跨反馈处置报告 | ✅ |
| F2 | 代签审核缺栏 | 17/18 PASS + 10 处缺栏补全 (commit `be27937`) | ✅ |
| F3 | path byte-level 偏差 | `00-基准与治理/` 保留决议 (per 8/26 04:30) | ✅ |
| F4 | BAS × TST 覆盖审计 | 44/44 BAS 引用 100% (commit `c6dc816`) | ✅ |
| F5 | F8 v0.2 services[] 5 子字段 | UT-08 模块 D 字段级协议实装 (commit `ec0f11a`) | ✅ |
| F6 | DTL 章节级追溯 | 6 域 + cluster-ops 独立 UT + 章节级追溯 (commit `3e8d9ca`) | ✅ |
| F7 | test-evidence v4 | `test-evidence.ps1` 升级 (commit `b87f1b3`) | ✅ |
| F8 | GM 后台 5 endpoint 字段级协议 | 实装 (commit `c14d49b`) | ✅ |
| F9 | IT 准入核对 | 推荐路径 (commit `3357c10`) | ✅ |

**已知缺口**: 9 条全部处置, 0 pending。

---

## Section C: 测试 + 覆盖率 summary

### C.1 G3 workspace 跑测 (commit `2b3ad09`)
- **81/81 targets PASS, 0 fail** (上轮 17 fail 已全修)
- 663/663 test cases PASS, 37 ignored (PH-5 Cloudflare opt-in)
- evidence: `docs/00-基准与治理/.test-evidence/g3-g4-20260828-070349/`

### C.2 G4 覆盖率 (commit `2b3ad09`)
- **Workspace line coverage: 75.9%** (8829/11639 行)
- **14/14 域 + 共享 crate ≥ 60%** (rgs-hello 空 stub 0%)
- TOP: rgs-arc-olu 100% / rgs-certgen 95.5% / rgs-testkit 93% / gm-backend 91.2%
- MIN: match-service 62.2% (≥ 60% 阈值)

### C.3 gm-backend 49/49 (commit `11a230a`)
- 含 8 JWT UT + 7 audit UT + 7 outbox NATS mock UT + 6 admin gRPC IT + 12 5 endpoint IT + 9 其他
- 上轮 36 → 49 (+13)
- 0 fail

### C.4 9 域累计
- player 28 / economy 57 / match 29 / social 21 / admin 32 / cluster-ops 56 / gm-backend 49 / rgs-certgen 17 / rgs-testkit 35
- 9 域共 324/324 PASS (workspace 整合, 含 49 gm-backend)

---

## Section D: 文档治理 summary

### D.1 18 份 TST 文档头表 BAS 引用
- 44/44 BAS 引用 100% (commit `c6dc816`)
- 7 域 TST 头表 + UT-08/09 + IT-00~09 全覆盖

### D.2 代签审核
- 17/18 PASS, 1 个 ⏳ (5 域 Lead 拒绝兼任, 仍 Mavis 接手)
- 10 处缺栏补全 (commit `be27937`)

### D.3 35 份 BAS 文档
- 全覆盖 + 跨引用 (DTL-018/019/003/040 + 7 域 + gm-backend + cluster-ops)

### D.4 19 份 IT 文档
- IT-00 v0.2 + IT-01~09 全覆盖 (commit `90aa3df`)
- 7 域 IT 设计齐全

### D.5 跑测手册
- `G3-G4-it-main-stage-runbook.md` (commit `3357c10`) — 主阶段入口
- `G3-G4-it-main-stage-runbook.md` 待补: W2 跨域 + W4 S5 真 NATS 入口

---

## Section E: DDD Review 议程建议 (优先级)

| 序 | 议题 | 决策草案 | 决策点 |
|---|---|---|---|
| 1 | OPEN-QA v0.3 → v0.4 拍板 (Q2/Q4/Q7) | A9 | 升级模板是否固定化? |
| 2 | 8 域 Lead 具名采纳 | A1 | 12 角色边界 + RACI |
| 3 | cluster-ops 终方案 A' | A2 | P3 3 文件优先级 |
| 4 | S4 Phase 2 step 1 实际交付 | A4 | 49/49 PASS 是否接受 |
| 5 | TBD-08-06 工具决策 D | A3 | 5 域是否切 axum-test |
| 6 | S4 Phase 2 step 2 启动 | A5 | worktree W1 是否继续 |
| 7 | W2 跨域 IT 启动 | A6 | worktree W2 是否继续 |
| 8 | W4 S5 真 NATS e2e | A7 | worktree W3 是否继续 |
| 9 | AI 审计提示词集成 | A8 | 是否进 PR automation |

**预计时长**: 60-90 min
**前置**: 全部 9 worktree 状态已知 (W1/W2/W3 待 W1 完成后查看)

---

## Section F: 已知缺口 + 风险登记

### F.1 P0 (阻塞, 立即)
- ⏳ W1 / W2 / W3 worktree 状态未知 — worker 模式启动后立即 succeeded 但 worktree 无 commit
- ⏳ admin-service 5 GM RPC 待实装 (A5)

### F.2 P1 (重要, 9 月前)
- ⏳ 3 个 cluster-ops 旧债文件 P3 follow-up (A2)
- ⏳ S4 Phase 2 step 3+ 错误处理 + circuit breaker + chaos (A4)
- ⏳ mTLS to admin-service 决策 (per BAS-003 §2.1)

### F.3 P2 (中等, 季度内)
- ⏳ JWT propagation gRPC metadata (A4)
- ⏳ match-service 62.2% 覆盖率待提升 (C.2)
- ⏳ W2 跨域 IT 5 类链路完成度 (A6)
- ⏳ 5 域 + cluster-ops 4 文件 P3 follow-up

### F.4 P3 (低, 后续)
- ⏳ OPEN-QA 模板固定化 (A9)
- ⏳ AI 审计提示词集成 CI hook (A8)
- ⏳ rgs-hello stub 处理 (C.2 0% 覆盖)

---

## Section G: 下一步 WBS 提议

### W5: DDD Review 落地 (本批次)
- W1 → push `it/s4-phase2-step2` to origin (if worker 完成)
- W2 → push `it/w2-cross-domain` to origin
- W3 → push `it/w4-s5-nats` to origin
- W4 → push `docs/ddd-review` to origin (本 worktree 推进中)
- Ulysses 一次性审 4 branch → merge main
- 跑 G3 重测确认 81/81 + 49/49 仍 PASS

### W6: BAS 章节级追溯 (9 月初)
- 35 份 BAS 文档逐章追溯到 IT 文档
- 7 域 + gm-backend + cluster-ops 9 域全
- 估算 80-120M tokens (per 8/21 OLU 框架)

### W7: gm-backend 业务实现 (9 月中)
- 5 GM RPC 业务实装 (BanAccount → player / Compensation → economy / Maintenance → cluster-ops / AuditLog → admin)
- 端到端跑通 5 域 + gm-backend
- 估算 60-100M tokens

### W8: PH-1 OTel 全链路 (9 月末)
- sqlx-tracing sample ratio 10-20%
- 5 域 + gm-backend + cluster-ops + shared-platform + NATS
- per 8/22 OLU 框架

---

## H: 引用

- **main branch HEAD**: `38097e8` (S4 Phase 2 step 1 设计文档)
- **worktree list**: 8 个 (main + 4 旧 + 3 新)
  - main: `D:/RustGameServer`
  - 旧: `fix-drill-compile`, `M1`, `M2`, `M3` (8/22 之前的 WBS M 段)
  - 新: `s4-phase2-step2`, `w2-cross-domain`, `w4-s5-nats`, `ddd-review` (本批次)
- **session id**: `mvs_25e9300a4de240af9fc2e31f5eb99eaa`
- **当前时间**: 2026-08-28 17:09 JST
