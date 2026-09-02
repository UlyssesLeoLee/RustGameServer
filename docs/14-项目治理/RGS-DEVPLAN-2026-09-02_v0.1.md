# RGS-DEVPLAN-2026-09-02 v0.1 — 9/2 19:00 JST 仓库盘点 + 未完成任务开发计划

> **创建日期**: 2026-09-02 19:00 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/2 19:00 JST 仓库盘点 (main `ebb6ba5` vs origin/main `55dce67` = 233 commit 未推送) + RGS-WEEKLY-2026-W37 v0.1 (commit `8d69cef`) + RGS-BATCH-V0.1-FREEZE v0.1 (commit `06b3091`) + RGS-PHASE-C-PREP v0.1 (commit `4498dca`) + L-CANDIDATES v0.2 (commit `ee3c7e7`)
> **作用域**: 6 域 (player / economy / match / social / admin / batch) + 平台层 + 工具 crate, 9/2 → 12/2 季度评审窗口

---

## 0. 仓库合并现状 (9/2 19:00 JST 盘点)

### 0.1 分支全景

| 项 | 数值 | 说明 |
|---|---|---|
| main HEAD | `ebb6ba5` | chore(agents): AGENTS.md v0.6.8 升版 (9/2 18:41 JST) |
| origin/main HEAD | `55dce67` | docs(AGENTS): v0.3 纳入 L9/L11/L12 (9/1 16:00 JST) |
| main 领先 origin/main | **233 commit** | 本地未推送, 含 9/1 17:00-9/2 全天工作 |
| 非 main 本地分支 | **15** | ut/* 5 + fix/* 4 + st/* 1 + feat/* 4 + claude/* 1 |

### 0.2 智能合并结论: 0 待合并

**关键发现**: 全部 15 个非 main 分支都已被 main 通过 merge commit 完整吸收.

| 分支 | merge-base vs branch | main 视角 ahead/behind | 状态 |
|---|---|---|---|
| claude/ci-cd-correctness-s6qohc | branch_at_merge_base | 348 / 0 | ✅ 已并入 main |
| feat/auto-20260901-3e13c819 | branch_at_merge_base | 182 / 0 | ✅ 已并入 main |
| feat/w37-critique-v0.2 | branch_at_merge_base | 6 / 0 | ✅ 通过 merge `4b0374f` 合并 |
| feat/w37-e2e-fillin | branch_at_merge_base | 6 / 0 | ✅ 通过 merge `2d2f33c` 合并 |
| feat/w37-l15-candidate | branch_at_merge_base | 6 / 0 | ✅ 通过 merge `15fd69b` 合并 |
| fix/admin-rbac-audit-verify | branch_at_merge_base | 278 / 0 | ✅ 已并入 main |
| fix/economy-outbox-skip | branch_at_merge_base | 278 / 0 | ✅ 已并入 main |
| fix/player-wins-le-total | branch_at_merge_base | 278 / 0 | ✅ 已并入 main |
| fix/social-leave-guild-push-dispatcher | branch_at_merge_base | 278 / 0 | ✅ 已并入 main |
| st/mock-server-and-scripts | branch_at_merge_base | 302 / 0 | ✅ 已并入 main |
| ut/admin / economy / match / player / social | branch_at_merge_base | 304-305 / 0 | ✅ 5 域 IT 全部已并入 |

**merge-base 验证方法**: `git merge-base --is-ancestor <branch> main` 全 15 个返回 TRUE + `git rev-list main..<branch>` 全 15 个返回 0 ahead.

**未推送 233 commit 已全部落地在 main** (不需要"合并先进分支到 main"操作, 因为它们早就合并了).

### 0.3 推送决策 (待 Ulysses 拍板)

- **推送时机**: 9/2 19:00 JST 后 = 立即 vs 业务里程碑后
- **推送范围**: main HEAD 全 233 commit vs 选 W37 冲刺后
- **推送前置**: SRE Lead 拍板悬空, 推送不阻塞 SRE 介入

---

## 1. 未完成业务里程碑 (per W37 v0.1 §0.1 双指标)

### 1.1 5 域 ST 业务 mTLS (W37 D3-5, 9/10-12 JST)

| 状态 | 目标 | 阻塞 | 依赖 |
|---|---|---|---|
| 🟡 1/5 (gm-backend 8081 HTTP) | 🟢 5/5 | SRE Lead 阶段 A 拍板悬空 | Phase C 阶段 A 全 4 步 |
| **1 跳待跑** | player 50051 / economy 50052 / match 50053 / social 50054 / admin 50055 gRPC mTLS health probe | container minimal image 无 grpcurl/curl/wget, SRE 拍板 sidecar 选型 | 阶段 B 8 步 (per RGS-PHASE-C-PREP §1) |

### 1.2 Phase C 阶段 A 全 4 步 (W37 D2, 9/9 JST)

| 步骤 | 内容 | 当前状态 |
|---|---|---|
| A1 | `kubectl get nodes` 节点状态 | ✅ 9/2 16:10 JST 已确认 (ulyssespc Ready 31h) |
| A2 | `kubectl get pods -A` 全 namespace 状态 | ⏳ SRE 拍板后跑 |
| A3 | **prometheus ReplicaSet 缩容** (本次发现) | ⏳ SRE 拍板后跑 (per A3 PVC lock 抢锁根因) |
| A4 | HPA / minReplicas 检查 (per §2.5 L6 教训) | ⏳ SRE 拍板后跑 |

**关键阻塞**: **SRE Lead 拍板悬空** (per RGS-PHASE-C-KICKOFF v0.1 §3.1, 4 选 1 拍板项, 已 8h+ 悬空). 9/2 17:32 JST 启动预热走"选项 1" (Mavis-side), SRE Lead 不可达. 候选 L-CAND-004 (SRE 拍板超时防御) 待 12/2 季度评审, 暂时无自动降级.

### 1.3 22 测试函数真跑 (W37 D6 + W38)

| 测试包 | 来源 | 数量 | 期望 | 阻塞 |
|---|---|---|---|---|
| UT 11 函数 | RGS-TEST-RUN-PLAN v0.1 | 11 | 11/11 PASS | 不需 SRE 介入, 立即可跑 (cargo test --lib) |
| E2E 11 函数 | RGS-TEST-RUN-PLAN v0.1 | 11 | 11/11 PASS | 需 Phase C 阶段 B/C 完成 |
| 5 域跨域 saga | BATCH-PLAN v0.2 W4-W6 | 1 套 | PASS | 需 38 L4 任务落地 |
| mTLS 业务级 1 跳 | RGS-PHASE-C-PREP §2.4 | 1 | SERVING | 需 B4-B8 + certs 导出 |
| 跨域 saga 真实交易 | RGS-PHASE-C-PREP §2.5 | 1 | 三域 OK + ledger | 需 C6 阶段 |
| batch 域 GAP-10 跨域 saga | commit `ea4c874` | 1 | batch → saga OK | 需 batch-backend 跑通 |

### 1.4 业务指标承诺 vs 现状

| 指标 (W36 末) | W37 目标 | W38 目标 | 现状 (9/2 19:00) |
|---|---|---|---|
| 5 域 ST 业务 mTLS | 🟡 5/5 | — | 🟡 1/5 |
| Phase C 阶段 A | 🟡 4/4 步 | — | ⏳ 准备包就绪 (commit `4498dca`) |
| DDD Review v0.2 | 🟢 维护 | — | 🟢 9 份完成 + 9 份自动通过收口 |
| batch 域 v0.1 冻结 | 🟡 W38 解冻 | 🟢 解冻 | 🔒 C1 冻结 (commit `06b3091`) |
| commit ahead origin/main | ≤ 20 | ≤ 20 | ⚠️ 233 (5 域 E2E 跑通后回落) |

---

## 2. 未完成治理派生约束 (per AGENTS.md v0.6.8 + L-CANDIDATES v0.2)

### 2.1 L-CANDIDATES 8 条候选 (12/2 季度评审)

| 编号 | 内容 | 类型 | 来源 | 评审 |
|---|---|---|---|---|
| L-CAND-001 | A1 RGS-BAS-037 (265KB) 拆 4 份 ≤70KB | 文档减肥 | RGS-CRITIQUE v0.1.1 §3.1 | 12/2 Q4 |
| L-CAND-002 | A3 AGENTS.md 6 月一归档 (v0.6 → archive, 主 ≤20KB) | 文档减肥 | RGS-CRITIQUE v0.1.1 §3.1 | 12/2 Q4 |
| L-CAND-003 | A4 document-registry.toml 强制 80KB 上限 + CI 校验 | 文档减肥 | RGS-CRITIQUE v0.1.1 §3.1 | 12/2 Q4 |
| L-CAND-004 | L15 SRE Lead 拍板超时防御 (24h 自动降级) | 业务类 | W37 v0.1 §3 + Phase C KICKOFF §3.1 | 12/2 Q4 |
| L-CAND-005 | L15 业务里程碑 commit 必带 git 实证 (commit SHA + file:line) | 业务类 | W37 v0.1 §0.1 + RGS-CRITIQUE v0.1.1 §2.5 | 12/2 Q4 |
| L-CAND-006 | L15 k8s secret 导出硬 ban (cert 内容不入 commit, fingerprint 比对) | 安全类 | 8/27 11:06 JST hard ban + W37 v0.1 §3 | **可走安全例外立即生效** (per §0 第 4 项) |
| L-CAND-007 | L15 派生约束引用版本锁 (CI pre-commit 检查) | 治理类 | W37 v0.1 §5 + RGS-CRITIQUE v0.1.1 §2.3 | 12/2 Q4 |
| L-CAND-008 | (保留位) L1-L14 冻结期内 Mavis 发现 | — | — | 12/2 Q4 |

**L-CAND-006 例外路径** (per 8/27 安全派生约束例外条款): 凭据泄露 = 立即生效. Mavis 上报 Ulysses 拍板后 9/2-9/9 期间可单独 commit + 写入 AGENTS.md §8 例外段. **建议本周 (9/2-9/9) 优先走例外路径, 9/9 SRE Lead 拍板后 certs 导出时立刻生效**.

### 2.2 C 类 / D 类 (W2-3 拍板, 9/15-9/28 JST)

| 派生约束 | 内容 | 时窗 | DoD |
|---|---|---|---|
| C2 | PT 文档/流程自审 (8 worker 派工模板 + 临时 log 清理 + cargo dir lock 防御) | W2 (9/15-9/21) | 自审报告 + 流程模板升版 |
| C3 | 5 域生产可用 checklist (per v0.1.1 §9.4 里程碑重定义) | W2-W3 (9/15-9/28) | 5 域 checklist 完成 + Ulysses 二审 |
| D1 | L1.2 E2E 业务级跑通 (5 域 E2E 22 测试函数合并 verdict) | W2 (9/15-9/21) | 22/22 PASS + 1 commit |

---

## 3. 未完成 batch 域 (per RGS-BATCH-V0.1-FREEZE v0.1)

### 3.1 v0.1 冻结现状 (per §1.1 + §2 12 GAP 状态)

| 状态 | 数量 | 说明 |
|---|---|---|
| ✅ 已实现 | 6/12 | GAP-1 跨 batch DAG / GAP-2 SSE 流式 / GAP-5 AI SQL / GAP-6 rgs-web bridge / GAP-8 Rollback SQL / GAP-10 跨域 saga 触发 |
| 🟡 v0.1 跳过 | 6/12 | GAP-3 WebSocket / GAP-4 mavis cron / GAP-7 优先级 / GAP-9 模板版本化 / GAP-11 超时 kill / GAP-12 RACI 同步 |

### 3.2 W2-W6 38 L4 任务 (per BATCH-PLAN v0.2 §3)

- 38 L4 任务, 54 人·天 / 9.65M tokens, 6 周落地
- W1 (9/1-9/7) 6 任务 ✅ 部分落地 (per C1 冻结)
- W2-W6 (9/8-10/12) 32 任务 🟡 冻结期继续 (修 bug OK, 不开新功能)
- 触发解冻: Phase C 阶段 C 跑通 (per §3.1 硬性条件)

### 3.3 batch 域 v0.2 评估 (per §5 已知缺口)

- **冻结期长度未知**: 保守估计 1-2 周 (取决于 Phase C SRE 介入节奏)
- **12 GAP 中 6 跳过项的 v0.2 工作量未精算**: Phase C 后由 batch Lead 出 v0.2 评估
- **batch 域 Lead RACI 同步 (GAP-12)**: v0.1 跳过, 解冻后补
- **k3s 资源上限 + namespace 隔离策略 (per REQ §10.3)**: 冻结期不影响, 解冻后协调
- **5 域 binary 未来调外部 LLM 未登记 (per OLU-WEB F-25)**: 冻结期不影响, v0.2 评估

---

## 4. 未完成 ST 阶段 (per RGS-OPEN-QA v0.2 Q8-Q11 决策)

| Q# | 决策 | 阻塞 | 依赖 |
|---|---|---|---|
| Q8 | gm-backend 8081 ✅ HTTP 通 (主会话打头阵 1 跳), 业务级 mTLS 待 SRE | 阶段 B | 阶段 A 全 4 步 |
| Q9 | prometheus/grafana ❌ (prometheus CrashLoopBackOff 27h) | 阶段 A3 | SRE Lead 拍板 |
| Q10 | mTLS 业务级 ST (5 域 → gm-backend 8443) | 阶段 B | 阶段 A |
| Q11 | NATS 部署范围核查 (1 条 kubectl get pods) | 不阻塞 SRE | 立即可跑 |

---

## 5. 未完成 DDD Review (per RGS-DDD-PRE-AUDIT-2026-09-02 v0.1)

- ✅ 11 份历史 DDD Review 走 v0.2 二审模板 (commit `f2d33cc` + `a0774e4`)
- ⏳ 后续 DDD Review 走 B3 二审流程 (Mavis 写+自审 1 次 → Ulysses 必审)
- 状态机: ⏳ → 🟡 → ⏳ → ✅/❌/🟡
- 打回循环上限 2 次, 第 3 次强制 ✅ 或 🟡 冻结

---

## 6. 紧急阻塞清单 (9/2 19:00 JST 风险登记)

| 风险 | 等级 | 触发条件 | 应对 |
|---|---|---|---|
| **SRE Lead 拍板悬空 (W37 D2 9/9 JST)** | 🟡 中 | 9/2 17:32 JST 起算 24h | 选项 C (推迟 W38), 写 RGS-PHASE-C-DEFER-* 公告 (L-CAND-004 候选) |
| **prometheus ReplicaSet 修复失效** | 🟡 中 | A3 步骤 PVC lock 抢锁 | 备选: 删 RS 重 scale, 清 PVC |
| **grpcurl 安装失败** | 🟡 中 | container minimal image 无 apk/apt | sidecar / init container / 本地 admin pod 装 |
| **5 域 mTLS 业务 1 跳不通** | 🟡 中 | cert 链异常 | 重导 certs + 验证 openssl x509 |
| **22 测试函数 race condition** | 🟢 低 | `--test-threads=1` 兜底 | per RGS-TEST-RUN-PLAN v0.1 |
| **W37 周报 v0.3 (9/14) D4 派生约束漏** | 🟢 低 | Mavis 自审 | 沿用 v0.3 模板 |

---

## 7. 周时间窗路线图 (W37-W40)

### W37 (9/8-9/14) 业务冲刺周

| Day | 日期 | 关键任务 | 责任人 | DoD |
|---|---|---|---|---|
| D1 | 9/8 一 | RGS-WEEKLY-2026-W37 v0.1 启动预热 | Mavis | ✅ 9/2 18:16 JST 完成 (commit `8d69cef`) |
| **D2** | **9/9 二** | **Phase C 阶段 A 全 4 步 (A1-A4)** | **SRE Lead** | **1 commit 串阶段 A 完结** |
| D3 | 9/10 三 | 阶段 B 启: 5 域 certs 导出 | SRE Lead | 6 cert yaml 文件 (per B1-B2) |
| D4 | 9/11 四 | 阶段 B 中: grpcurl 装 + player/economy health probe | SRE Lead | 2 跳 health probe (per B3-B5) |
| D5 | 9/12 五 | 阶段 B 末: match/social/admin health probe | SRE Lead | 3 跳 + 阶段 B 完 (per B6-B8) |
| D6 | 9/13 六 | 阶段 C 启: 11 UT 真跑 | SRE Lead + Mavis | 11/11 PASS (per C1) |
| **D7** | **9/14 日** | **RGS-WEEKLY-2026-W37 v0.3 + 11 E2E 准备** | **Mavis + SRE Lead** | **周报 v0.3 + 11 E2E 准备** |

### W38 (9/15-9/21) Phase C 收口周

| 任务 | 依赖 | 责任人 | DoD |
|---|---|---|---|
| 阶段 C 11 E2E 真跑 (per C2) | W37 阶段 C | Mavis + SRE Lead | 11/11 PASS |
| 阶段 C 5 域跨域 saga (per C3) | 38 L4 任务部分落地 | Mavis | 1 套 saga PASS |
| 阶段 C mTLS 业务级 1+2 跳 (per C4-C5) | 阶段 B 完 | SRE Lead | 业务 mTLS OK |
| 阶段 C 跨域 saga 真实交易 (per C6) | 5 域 E2E 跑通 | Mavis | 1 笔测试交易 OK |
| 阶段 C batch GAP-10 (per C7) | batch-backend 跑通 | batch Lead | batch → saga OK |
| 阶段 C 22 测试函数合并 verdict (per C8) | C1-C7 全 OK | Mavis | 22/22 PASS, 1 commit |
| 阶段 D1 5 域生产可用里程碑 | 阶段 C 完 | Mavis + Ulysses | 业务里程碑 ✅ |
| 阶段 D2 batch 域 v0.1 解冻 | D1 完 | Ulysses 拍板 | RGS-BATCH-V0.1-UNFREEZE-* |
| 阶段 D3 RGS-CRITIQUE v0.2 升版 | D1 完 | Mavis 自审 + Ulysses 二审 | v0.2 升版 |

### W39 (9/22-9/28) batch 解冻 + 季度评审准备周

| 任务 | 依赖 | 责任人 | DoD |
|---|---|---|---|
| batch 域 v0.2 评估 (12 GAP 6 跳过) | W38 D2 完 | batch Lead | v0.2 评估报告 |
| RACI v1.2 扩展 5 域 → 6 域 (per PLAN A.3) | batch 解冻 | Mavis | RACI v1.2 落档 |
| IMPL-PLAN-BATCH-001 v0.1 起草 (per PLAN A.3) | batch 解冻 | batch Lead | v0.1 起草 |
| RACI-BATCH-V1 v0.1 独立 RACI (per PLAN A.3) | batch 解冻 | batch Lead | v0.1 落档 |
| C2 PT 文档/流程自审 | W38 完 | Mavis | 自审报告 + 模板升版 |
| C3 5 域生产可用 checklist | W38 D1 完 | Mavis + Ulysses 二审 | checklist 完 |

### W40 (9/29-10/5) 季度评审准备周

| 任务 | 依赖 | 责任人 | DoD |
|---|---|---|---|
| 8 条 L-CAND 候选自审报告 | 9/29-10/3 | Mavis | 自审报告 v0.1 |
| L-CAND-006 立即生效路径 (可选) | SRE Lead 拍板 | Mavis | AGENTS.md §8 例外段 |
| 派生约束版本锁 CI pre-commit 草案 (per L-CAND-007) | Mavis 自审 | Mavis | 草案 v0.1 |
| 业务里程碑 commit git 实证模板 (per L-CAND-005) | Mavis 自审 | Mavis | D3 commit 模板升 v0.2 |

### 12/2 (Q4) 季度评审周

| 任务 | 依赖 | 责任人 | DoD |
|---|---|---|---|
| 8 条 L-CAND 候选拍板 | 12/2 JST | Ulysses | 通过升 AGENTS.md, 未通过清出候选清单 |
| L1-L14 冻结期维持 (至 2027-03-02) | — | — | 不动 |

---

## 8. 季度评审路线图 (12/2 JST)

| 评审日 | 入档候选 | 通过 | 清出 | 状态 |
|---|---|---|---|---|
| 2026-12-02 (Q4) | L-CAND-001/002/003 (A 类) + L-CAND-004/005/006/007 (L15) + L-CAND-008 (保留) | — | — | ⏳ 待评审 |
| 2027-03-02 (Q1) | (Q4 评审后入档新候选) | — | — | 待启 |
| 2027-06-02 (Q2) | (Q1 评审后入档新候选) | — | — | 待启 |
| 2027-09-02 (Q3) | L1-L14 冻结期届满, 重新评估 | — | — | 待启 |

**L-CAND-006 例外路径**: 安全相关可立即生效, 不走 12/2 季度评审. 9/2-9/9 期间可单独 commit + 写入 AGENTS.md §8 例外段.

---

## 9. 已知缺口 (per 8/26 JST 缺标比错标)

- **23 commit main 领先 origin/main 233 commit** 未推送决策待 Ulysses 拍板
- **SRE Lead 时间窗口**: 阶段 A 1.5h 起步, SRE 拍板窗口未定, 不可达时走 L-CAND-004 候选
- **Phase C 阶段 C 22 测试函数**: 11 UT 立即可跑, 11 E2E 需阶段 B 完成
- **A 类 4 条候选清单 (12/2 季度评审)**: 不阻塞 W37 sprint, 12/2 前可入档
- **A4 HPA 资源**: 当前集群无 HPA, SRE 跑 A4 步骤可能发现 ingress/cert-manager HPA, 影响 5 域业务
- **派生约束 L1-L14 冻结 6 个月 (至 2027-03-02)**: 期间不增 L15, 新约束进候选清单
- **batch 域 v0.1 解冻 5 域 E2E 跑通后**: 不可预判, 走 14:58 拍板规则
- **OLU token 重算 (RGS-TS-001 §6.2)**: 1 人·天 ≈ 100K-300K tokens, 待 SRE Lead + PM 校准

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 19:00 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 9/2 19:00 JST 仓库盘点 (0 待合并先进分支) + 7 大未完成任务路线图 (W37 业务冲刺 / W38 Phase C 收口 / W39 batch 解冻 / W40 季度评审准备 / 12/2 Q4) + 8 条 L-CAND 候选清单 + 9 条已知缺口, per W37 v0.1 + RGS-BATCH-V0.1-FREEZE v0.1 + RGS-PHASE-C-PREP v0.1 + L-CANDIDATES v0.2 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
