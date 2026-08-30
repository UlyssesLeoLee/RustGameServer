# DDD Review Meeting 2026-08-28 (启动会 + 9 项决议)

> **目的**:DDD Review 启动会议程 + 9 项决策草案决议表
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 18:08 JST)
> **状态**:⏳ OPEN (会议记录,等 Ulysses 拍板)
> **关联**:RGS-DDD-REVIEW-2026-08-28-checklist.md + summary.md + OPEN-QA v0.4

---

## 1. 会议基本信息

- **日期**: 2026-08-28 (per Ulysses 排期, 时间待定)
- **时长**: 60-90 min (per DDD Review checklist §E)
- **出席者**: Ulysses (架构师 + 一人公司 12 角色) / Mavis (接手 agent per DEC-008)
- **议程**: 9 项决策草案逐条审议 + 决议 + 实装路径
- **前置**: main HEAD `dba953b` (8 commits 推 origin)
- **记录**: 本文档 + 9 项决议 (Section 2)
- **决议格式**: 9 列表格,每行 = 1 草案 (议题 / 草案 / 风险 / 决议 / 实装路径 / 状态 / 决策人 / 日期 / 备注)

---

## 2. 9 项决议

### 决议 1 — OPEN-QA v0.3 → v0.4 拍板
- **议题**: OPEN-QA v0.4 升级 (Q2/Q4/Q7 resolved) 是否拍板
- **草案**: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.4.md` (24,000+ bytes)
- **风险**: 模板固定化后,后续每次迭代都需重写 v0.x;可能限制 OPEN-QA 灵活性
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 已实装 (commit ae32266),拍板后 status 改 `✅ Closed`
- **决策人**: Ulysses
- **备注**: 若拒绝,OPEN-QA 仍 v0.3 待续

### 决议 2 — 8 域 Lead 12 角色采纳
- **议题**: 8 域 Lead 具名 + 4 共享角色 (SRE/Platform/QA/PM) 是否采纳
- **草案**: `docs/00-基准与治理/RGS-LEAD-NAMING-8-域-2026-08-28.md`
- **风险**: RACI 矩阵未补,8 域 Lead 责任边界可能模糊
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 已实装 (commit 12437ca),拍板后 commit be27937 代签补全 + RACI 矩阵 W6 9 月初
- **决策人**: Ulysses
- **备注**: 8 域 + 4 共享 = 12 角色 (per DEC-008 一人公司 12 角色)

### 决议 3 — cluster-ops 终方案 A'
- **议题**: git rm tests-disabled/ut_state_machine.rs + 3 文件 P3 follow-up
- **草案**: `docs/00-基准与治理/RGS-TST-CLUSTER-OPS-OLD-DEBT-终方案决策.md`
- **风险**: 3 文件 P3 follow-up 仍未实装,9 月前需排期
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: A' 已实装 (commit 3e8d9ca),P3 follow-up 推后到 9 月
- **决策人**: Ulysses
- **备注**: P3 3 文件: rgs-testkit mock 弃用警告 / admin-service 55.13 升级 / gm-backend 业务 5 endpoint

### 决议 4 — S4 Phase 2 step 1 实际交付
- **议题**: gm-backend admin-service gRPC client 注入 (commit 11a230a + 38097e8)
- **草案**: `docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md` (5,716 bytes)
- **风险**: 4 endpoint 仍 stub, S4 Phase 2 step 2 需 admin-service 加 5 GM RPC (已实装 commit 1da9388)
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: step 1 已实装 (gm-backend 49/49 PASS),step 2 已实装 (gm-backend 53/53 + admin-service 35/35)
- **决策人**: Ulysses
- **备注**: 已知缺口: mTLS / JWT propagation / circuit breaker 待 Step 3+

### 决议 5 — S4 Phase 2 step 2 实际交付
- **议题**: admin-service 5 GM RPC (BanAccount/GrantCompensation/SetMaintenance/QueryAuditLog + HealthView 已有)
- **草案**: ⏳ 待生成 (本 worktree 推进)
- **风险**: 4 endpoint 调 gRPC 500ms timeout 失败降级 InMemory,生产环境 admin-service 不可达时仍可服务
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 已实装 (commit 1da9388, gm-backend 53/53 + admin-service 35/35)
- **决策人**: Ulysses
- **备注**: Step 3+ 需错误处理 + circuit breaker + chaos test + mTLS

### 决议 6 — TBD-08-06 工具决策 D
- **议题**: 双工具并存 (7 域 wiremock 0.6 + 8 域 axum-test 16)
- **草案**: `docs/00-基准与治理/RGS-TST-08-06-axum-test-vs-wiremock-工具决策.md`
- **风险**: 5 域是否切 axum-test 决策待定,统一性可能丢失
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 已实装 (commit 90aa3df),7 域 + 8 域 IT 文档齐全
- **决策人**: Ulysses
- **备注**: 5 域 (player/economy/match/social/admin) 现仍用 InMemory mock,待观察统一时机

### 决议 7 — W2 跨域 IT 链路用例
- **议题**: 5 类跨域链路 (cluster-ops ↔ 5 域 / cluster-ops ↔ admin / gm-backend → admin → 5 域 / gm-backend → admin → economy / cluster-ops ↔ gm-backend)
- **草案**: `docs/00-基准与治理/RGS-TST-CROSS-DOMAIN-链路-IT-设计书.md` (4,511 bytes)
- **风险**: 链路 B/C/D 需 admin-service → 5 域 gRPC client + 5 域暴露 GM RPC,Step 3+ 业务实装工作量大
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 链路 A 简化版 (cluster-ops 3 副本 self-check) 已实装 commit 86d27e5,链路 E (gm-backend 53/53) 隐含已实装
- **决策人**: Ulysses
- **备注**: 5 类链路完成度: 链路 A 1/1 + 链路 E 53/53 + 链路 B/C/D ⏳ Step 3+

### 决议 8 — W4 S5 §3 真 NATS e2e
- **议题**: k3s nats-0 port-forward 14222 + 7 真链路测试
- **草案**: `docs/00-基准与治理/RGS-TST-S5-outbox-NATS-IT-设计书.md`
- **风险**: 7 真链路 (lease 过期 / retry 退避 / JetStream 持久化) 需更深 async-nats API
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 3/7 真链路 PASS (connect / pubsub / request-reply),其余 4/7 (lease 过期 / retry 退避 / 并发 / 持久化) ⏳ Step 3+
- **决策人**: Ulysses
- **备注**: mock 7/7 (it_outbox_nats.rs commit b6cf3d8) + 真 3/7 (commit a39af02) = 10/14 总 NATS 测试

### 决议 9 — AI 审计提示词集成 CI
- **议题**: AI 审计提示词 (9 维度 + 10 重点核查项) 是否集成 PR automation
- **草案**: `docs/00-基准与治理/AI-AUDIT-PROMPT-Mavis-2026-08-28.md` (9,489 bytes)
- **风险**: 集成 CI 增加 PR 延迟 (10-30s per PR), 误报可能多
- **决议**: ⏳ 待 Ulysses 拍板
- **实装路径**: 已落档,未集成 CI。集成路径: `.github/workflows/ai-audit.yml` 加 step `mavis --audit-pr $PR_BODY` 或 OpenAI API call
- **决策人**: Ulysses
- **备注**: 9 维度: 决策追踪 / 代码治理 / 测试设计 / 文档治理 / 跑测 / 覆盖 / 集成 / 部署 / 异常处理

---

## 3. 议程时间分配 (60-90 min)

| 时间 | 议题 | 决议号 | 时长 |
|---|---|---|---|
| 0:00-0:05 | 议程介绍 + 状态报告 | - | 5 min |
| 0:05-0:10 | OPEN-QA v0.4 拍板 | 1 | 5 min |
| 0:10-0:20 | 8 域 Lead 12 角色 + RACI | 2 | 10 min |
| 0:20-0:25 | cluster-ops 终方案 A' + P3 排期 | 3 | 5 min |
| 0:25-0:35 | S4 Phase 2 step 1+2 实际交付认可 | 4 + 5 | 10 min |
| 0:35-0:40 | TBD-08-06 工具决策 D + 5 域统一时机 | 6 | 5 min |
| 0:40-0:50 | W2 跨域 IT 5 类链路优先级 | 7 | 10 min |
| 0:50-1:00 | W4 S5 真 NATS 7 链路 + mock 共存 | 8 | 10 min |
| 1:00-1:10 | AI 审计提示词集成 CI | 9 | 10 min |
| 1:10-1:20 | 9 月 WBS W6-W8 安排 (BAS 章节级追溯 / gm-backend 业务 / OTel) | - | 10 min |
| 1:20-1:30 | Q&A + 决议记录确认 | - | 10 min |

---

## 4. 决议记录格式

每项决议记录:
- 决议号 (1-9)
- 草案: 文档路径
- 拍板结果: ✅ Approved / ❌ Rejected / ⏳ Pending
- 决策人: Ulysses
- 日期: 2026-08-28
- 实装状态: 已 commit / 待 commit / 不实装
- 备注: 详细说明

决议最终落档到 `docs/00-基准与治理/RGS-DDD-REVIEW-MEETING-2026-08-28-RESOLUTIONS.md` (本 worktree 推进)。

---

## 5. 后续动作 (决议后)

- **W6 9 月初**: BAS 章节级追溯 35 份 → IT 文档 (per 8/21 OLU 框架, 80-120M tokens)
- **W7 9 月中**: gm-backend 5 GM RPC 业务实装 (BanAccount → player / Compensation → economy / Maintenance → cluster-ops / AuditLog → admin)
- **W8 9 月末**: PH-1 OTel 全链路 sqlx-tracing sample 10-20%
- **W9 10 月初**: mTLS to admin-service 决策实装 (per BAS-003 §2.1, 55.26 fail-closed 已实装)
- **W10 10 月中**: cluster-ops 3 文件 P3 follow-up (per 决议 3)

---

## 6. 参考

- main HEAD: `dba953b` (8 commits pushed)
- DDD Review checklist: `docs/00-基准与治理/RGS-DDD-REVIEW-2026-08-28-checklist.md` (9,754 bytes)
- DDD Review summary: `docs/00-基准与治理/RGS-DDD-REVIEW-2026-08-28-summary.md` (4,804 bytes)
- OPEN-QA v0.4: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.4.md` (24,000+ bytes)
- 4 worktree (W1/W2/W3/W4) 已 merge 完毕,清理见 Decision 3 worktree
- 8 commits: `38097e8` / `11a230a` / `b6cf3d8` / `c6dc816` / `3e8d9ca` / `b87f1b3` / `ec0f11a` / `12437ca` / `3357c10` / `be27937` / `90aa3df` / `43a2e08` / `b4df2ed` / `b8359b9` / `2b3ad09` / `11a230a` / `38097e8` / `678549a` / `1da9388` / `86d27e5` / `a39af02` / `ae32266` / `8494ad1` / `5465872` / `dba953b`
