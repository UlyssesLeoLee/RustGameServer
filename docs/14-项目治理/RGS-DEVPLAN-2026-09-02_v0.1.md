# RGS-DEVPLAN-2026-09-02 v0.3 — 仓库盘点 + token 标准推进开发计划

> **创建日期**: 2026-09-02 19:00 JST (v0.1) → 2026-09-03 07:31 JST (v0.2) → **2026-09-03 07:34 JST (v0.3)**
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据 (v0.3)**: 9/3 07:34 JST Ulysses 拍板**用 token 标准推进, 不用日历周/日期锚点** (per 8/21 JST token-OLU 框架偏好: 1 人·天 ≈ 100K-300K tokens, 1 人·周 ≈ 500K-1.5M tokens, 1 SRE 上限 = 1 人·周 ≈ 1M tokens, 5 域独立 Lead × 14-18 周 = 80-120M tokens)
> **v0.3 关键改造**: 全文档删日历周/日期锚点/应急检查点, 改为 token 预算 + token 累计触发

---

## 0. 仓库合并现状 (9/2 19:00 JST 盘点, 沿用 v0.1)

### 0.1 分支全景

| 项 | 数值 | 说明 |
|---|---|---|
| main HEAD | `ebb6ba5` | chore(agents): AGENTS.md v0.6.8 升版 (9/2 18:41 JST) |
| origin/main HEAD | `55dce67` | docs(AGENTS): v0.3 纳入 L9/L11/L12 (9/1 16:00 JST) |
| main 领先 origin/main | **234 commit** (v0.1 提交后 +1) | 本地未推送 |
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

**merge-base 验证方法**: `git merge-base --is-ctor <branch> main` 全 15 个返回 TRUE + `git rev-list main..<branch>` 全 15 个返回 0 ahead.

### 0.3 推送决策 (per 9/3 07:31 JST Ulysses 拍板选项 2 + 9/3 07:34 JST token 标准改造)

**拍板 v1 (时间版)**: W37 冲刺后推送, 锚点 9/14 JST 周报 v0.3 落地后. **已废止** (per 9/3 07:34 JST 改造).

**拍板 v2 (token 标准, 当前)**:

- **推送触发**: **业务里程碑 token 累计达成**, 不等日历日
  - 业务里程碑 = 5 域 ST 业务 mTLS 5/5 + Phase C 阶段 A 4/4 步 + 22 测试函数 UT 11/11 PASS + E2E 准备就绪
  - token 累计 = 业务里程碑 + 治理 (L-CAND 评审) + batch 解冻 累计 ≥ **15M tokens** (估算)
- **推送范围**: main HEAD 全 commit (累积工作)
- **推送前置 4 验收 (token 触发)**:
  1. 5 域 mTLS 5/5 PASS ≈ 1.5M tokens
  2. 阶段 A 4/4 步完成 ≈ 0.8M tokens
  3. 22 测试函数 UT 11/11 PASS + E2E 准备 ≈ 2.5M tokens
  4. DDD Review 维护 + 派生约束守护 ≈ 0.5M tokens
  5. **业务里程碑累计 ≥ 5.3M tokens** 触发推送
- **推送后验证**: `git log origin/main..main --oneline | wc -l` 回到 ≤ 20 阈值, **不等具体日期**
- **应急降级 (token 触发)**: SRE 拍板悬空 = token 累计 0.5M 内必须出拍板, 超 1M token 走"选项 C 推迟后续阶段"

**不采用方案 (v0.2 时间版, 已废止)**:
- ❌ 选项 1 (立即推送): SRE 拍板悬空中, 无业务里程碑背书
- ❌ 选项 3 (分批推送): 增加 push 摩擦, 中间窗口 origin/main 不可读全
- ❌ **日历周锚点 (W37 D7 = 9/14 JST)**: AI 协作场景下 token 比日历准, 不等日期 (per 9/3 07:34 JST 拍板)

---

## 1. 未完成业务里程碑 (token 预算)

### 1.1 5 域 ST 业务 mTLS

| 状态 | token 预算 | token 累计触发 | 阻塞 | 依赖 |
|---|---|---|---|---|
| 🟡 1/5 (gm-backend 8081 HTTP) | 1.5M (5 域 × 300K) | 5/5 PASS 触发下一里程碑 | SRE Lead 阶段 A 拍板悬空 | Phase C 阶段 A 全 4 步 |
| 1 跳待跑 | — | — | container minimal image 无 grpcurl/curl/wget, SRE 拍板 sidecar 选型 | 阶段 B 8 步 (per RGS-PHASE-C-PREP §1) |

### 1.2 Phase C 阶段 A 全 4 步

| 步骤 | 内容 | token 预算 | 当前状态 |
|---|---|---|---|
| A1 | `kubectl get nodes` 节点状态 | 50K | ✅ 9/2 16:10 JST 已确认 (ulyssespc Ready 31h) |
| A2 | `kubectl get pods -A` 全 namespace 状态 | 100K | ⏳ SRE 拍板后跑 |
| A3 | **prometheus ReplicaSet 缩容** (本次发现) | 300K (含 PVC 修复) | ⏳ SRE 拍板后跑 (per A3 PVC lock 抢锁根因) |
| A4 | HPA / minReplicas 检查 (per §2.5 L6 教训) | 350K (含 HPA 修复) | ⏳ SRE 拍板后跑 |
| **阶段 A 小计** | | **800K** | |

**关键阻塞**: **SRE Lead 拍板悬空** (per RGS-PHASE-C-KICKOFF v0.1 §3.1, 4 选 1 拍板项, 已 8h+ 悬空). 候选 L-CAND-004 (SRE 拍板超时防御) 走 token 触发: token 累计 0.5M 内必须出拍板, 超 1M token 走"选项 C 推迟"。

### 1.3 22 测试函数真跑

| 测试包 | 来源 | 数量 | token 预算 | 期望 | 阻塞 |
|---|---|---|---|---|---|
| UT 11 函数 | RGS-TEST-RUN-PLAN v0.1 | 11 | 1.1M (11 × 100K, cargo test --lib) | 11/11 PASS | 不需 SRE 介入, 立即可跑 |
| E2E 11 函数 | RGS-TEST-RUN-PLAN v0.1 | 11 | 2.75M (11 × 250K, 含集成场景) | 11/11 PASS | 需 Phase C 阶段 B/C 完成 |
| 5 域跨域 saga | BATCH-PLAN v0.2 W4-W6 | 1 套 | 1.5M | PASS | 需 38 L4 任务落地 |
| mTLS 业务级 1 跳 | RGS-PHASE-C-PREP §2.4 | 1 | 200K (grpcurl + cert) | SERVING | 需 B4-B8 + certs 导出 |
| 跨域 saga 真实交易 | RGS-PHASE-C-PREP §2.5 | 1 | 500K | 三域 OK + ledger | 需 C6 阶段 |
| batch 域 GAP-10 跨域 saga | commit `ea4c874` | 1 | 300K | batch → saga OK | 需 batch-backend 跑通 |
| **22 函数小计** | | | **≈ 6.35M** | | |

### 1.4 业务指标承诺 vs 现状 (token 累计维度)

| 指标 | token 预算 | 现状 (9/3 07:34 JST) | 累计触发 |
|---|---|---|---|
| 5 域 ST 业务 mTLS | 1.5M | 🟡 1/5 (gm-backend 8081 HTTP ≈ 300K 已花) | 5/5 PASS |
| Phase C 阶段 A | 0.8M | ⏳ 准备包就绪 (commit `4498dca` ≈ 100K 已花) | 4/4 步 |
| DDD Review v0.2 | 0.5M | 🟢 9 份完成 + 9 份自动通过收口 ≈ 0.4M 已花 | 维护 |
| batch 域 v0.1 冻结 | (冻结期不计) | 🔒 C1 冻结 (commit `06b3091`) | 解冻 token 触发 |
| **业务里程碑累计** | **≈ 2.8M** | **累计 ≈ 0.8M (28.6%)** | **≥ 5.3M 触发推送** |

---

## 2. 未完成治理派生约束 (token 预算)

### 2.1 L-CANDIDATES 8 条候选 (token 触发季度评审)

| 编号 | 内容 | token 预算 | 类型 | 来源 |
|---|---|---|---|---|
| L-CAND-001 | A1 RGS-BAS-037 (265KB) 拆 4 份 ≤70KB | 800K (重排 + grep 全文改引用) | 文档减肥 | RGS-CRITIQUE v0.1.1 §3.1 |
| L-CAND-002 | A3 AGENTS.md 6 月一归档 (v0.6 → archive, 主 ≤20KB) | 200K | 文档减肥 | RGS-CRITIQUE v0.1.1 §3.1 |
| L-CAND-003 | A4 document-registry.toml 强制 80KB 上限 + CI 校验 | 300K (改 file + CI) | 文档减肥 | RGS-CRITIQUE v0.1.1 §3.1 |
| L-CAND-004 | L15 SRE Lead 拍板超时防御 (token 累计自动降级) | 200K (脚本 + 模板) | 业务类 | W37 v0.1 §3 + Phase C KICKOFF §3.1 |
| L-CAND-005 | L15 业务里程碑 commit 必带 git 实证 (commit SHA + file:line) | 150K (commit 模板升 v0.2) | 业务类 | W37 v0.1 §0.1 + RGS-CRITIQUE v0.1.1 §2.5 |
| L-CAND-006 | L15 k8s secret 导出硬 ban (cert 内容不入 commit, fingerprint 比对) | 100K (改 export script + .gitignore) | 安全类 | 8/27 11:06 JST hard ban + W37 v0.1 §3 |
| L-CAND-007 | L15 派生约束引用版本锁 (CI pre-commit 检查) | 300K (pre-commit + md 引用检查) | 治理类 | W37 v0.1 §5 + RGS-CRITIQUE v0.1.1 §2.3 |
| L-CAND-008 | (保留位) L1-L14 冻结期内 Mavis 发现 | — | — | — |
| **8 条 L-CAND 季度评审** | | **≈ 2.05M** (评审) | | |

**L-CAND-006 例外路径** (per 8/27 安全派生约束例外条款): 凭据泄露 = 立即生效, **不等季度评审**。SRE Lead 介入前 (9/3 07:34 JST 已悬空) 可单独 commit + 写入 AGENTS.md §8 例外段, token 预算 100K 立即花。

### 2.2 C 类 / D 类 (token 触发)

| 派生约束 | 内容 | token 预算 | DoD |
|---|---|---|---|
| C2 | PT 文档/流程自审 (8 worker 派工模板 + 临时 log 清理 + cargo dir lock 防御) | 800K | 自审报告 + 流程模板升版 |
| C3 | 5 域生产可用 checklist (per v0.1.1 §9.4 里程碑重定义) | 1.5M (5 域 × 300K) | 5 域 checklist 完成 + Ulysses 二审 |
| D1 | L1.2 E2E 业务级跑通 (5 域 E2E 22 测试函数合并 verdict) | 1.2M (22 函数 + 集成) | 22/22 PASS + 1 commit |

**C2 + C3 + D1 派生约束小计**: ≈ 3.5M tokens (业务冲刺 token 之外)

---

## 3. 未完成 batch 域 (token 预算)

### 3.1 v0.1 冻结现状 (per §1.1 + §2 12 GAP 状态, 沿用 v0.1)

| 状态 | 数量 | 说明 |
|---|---|---|
| ✅ 已实现 | 6/12 | GAP-1 跨 batch DAG / GAP-2 SSE 流式 / GAP-5 AI SQL / GAP-6 rgs-web bridge / GAP-8 Rollback SQL / GAP-10 跨域 saga 触发 |
| 🟡 v0.1 跳过 | 6/12 | GAP-3 WebSocket / GAP-4 mavis cron / GAP-7 优先级 / GAP-9 模板版本化 / GAP-11 超时 kill / GAP-12 RACI 同步 |

### 3.2 38 L4 任务 (per BATCH-PLAN v0.2 §3)

- 38 L4 任务, **54 人·天 / 9.65M tokens** (per BATCH-PLAN v0.2 §3 原始估算)
- 冻结期累计已花 ≈ 3M tokens (per 6/12 GAP 已实现的工作量)
- 剩余 32 L4 任务 ≈ **6.65M tokens** (W2-W6 期间累计)
- 冻结期 (修 bug OK, 不开新功能) 继续推进, token 累计不计冻结期日历

### 3.3 触发解冻条件 (token 触发, 不等"1-2 周")

- **硬性触发**: 业务里程碑 token 累计 ≥ 5.3M (5 域 mTLS 5/5 + 阶段 A 4/4 + 22 测试函数 UT 11/11 + DDD 维护) → 写 `RGS-BATCH-V0.1-UNFREEZE-*` 公告
- **软性触发**: 业务 commit ahead 回落 ≤ 20 阈值 (per v0.1.1 §9.4)
- **Ulysses 拍板**: token 累计达标后 0.5M 内必须出拍板 (per 14:58 拍板规则 + L-CAND-004 候选)

### 3.4 batch 域 v0.2 评估

- **12 GAP 中 6 跳过项的 v0.2 工作量未精算**: 触发解冻后由 batch Lead 出 v0.2 token 评估
- **batch 域 Lead RACI 同步 (GAP-12)**: v0.1 跳过, 解冻后补, token 预算 200K
- **k3s 资源上限 + namespace 隔离策略 (per REQ §10.3)**: 解冻后协调, token 预算 500K
- **5 域 binary 未来调外部 LLM 未登记 (per OLU-WEB F-25)**: v0.2 评估, token 预算 300K

---

## 4. 未完成 ST 阶段 (per RGS-OPEN-QA v0.2 Q8-Q11 决策, 沿用 v0.1)

| Q# | 决策 | 阻塞 | 依赖 |
|---|---|---|---|
| Q8 | gm-backend 8081 ✅ HTTP 通 (主会话打头阵 1 跳), 业务级 mTLS 待 SRE | 阶段 B | 阶段 A 全 4 步 |
| Q9 | prometheus/grafana ❌ (prometheus CrashLoopBackOff 27h) | 阶段 A3 | SRE Lead 拍板 |
| Q10 | mTLS 业务级 ST (5 域 → gm-backend 8443) | 阶段 B | 阶段 A |
| Q11 | NATS 部署范围核查 (1 条 kubectl get pods) | 不阻塞 SRE | 立即可跑 |

---

## 5. 未完成 DDD Review (per RGS-DDD-PRE-AUDIT-2026-09-02 v0.1, 沿用 v0.1)

- ✅ 11 份历史 DDD Review 走 v0.2 二审模板 (commit `f2d33cc` + `a0774e4`)
- ⏳ 后续 DDD Review 走 B3 二审流程 (Mavis 写+自审 1 次 → Ulysses 必审)
- 状态机: ⏳ → 🟡 → ⏳ → ✅/❌/🟡
- 打回循环上限 2 次, 第 3 次强制 ✅ 或 🟡 冻结
- token 预算: 自审 100K / 份, Ulysses 二审 50K / 份

---

## 6. 紧急阻塞清单 (token 触发, 9/3 07:34 JST 风险登记)

| 风险 | 等级 | token 触发 | 应对 |
|---|---|---|---|
| **SRE Lead 拍板悬空** | 🟡 中 | token 累计 0.5M 内必须出拍板, 超 1M token 走选项 C 推迟 | 选项 C 推迟后续阶段, 写 RGS-PHASE-C-DEFER-* 公告 (L-CAND-004 候选) |
| **prometheus ReplicaSet 修复失效** | 🟡 中 | token 累计 0.5M (A3 步骤 + 备选修复) | 备选: 删 RS 重 scale, 清 PVC |
| **grpcurl 安装失败** | 🟡 中 | token 累计 0.3M (sidecar / init container / 本地装) | sidecar / init container / 本地 admin pod 装 |
| **5 域 mTLS 业务 1 跳不通** | 🟡 中 | token 累计 0.5M (重导 + 验证) | 重导 certs + 验证 openssl x509 |
| **22 测试函数 race condition** | 🟢 低 | token 累计 0.2M (--test-threads=1 + 重跑) | per RGS-TEST-RUN-PLAN v0.1 |
| **派生约束漏守护** | 🟢 低 | Mavis 自审不计额外 token | 沿用模板 |

---

## 7. token 推进路线图 (替代原 W37-W40 时间窗路线图)

### 推进总览 (5 个 R-stage, 累计 ≈ 30-40M tokens)

| R-stage | 内容 | token 预算 | 累计 | 触发 |
|---|---|---|---|---|
| **R1 业务冲刺** | 5 域 mTLS + 阶段 A + 22 UT + DDD 维护 | 5.3M | 5.3M | **触发推送** + Phase C 阶段 B/C 启动 |
| **R2 Phase C 收口** | 阶段 B + 阶段 C (11 E2E + 5 域跨域 saga + mTLS 业务级 1+2 跳 + 跨域 saga 真实交易 + batch GAP-10) | 15M | 20.3M | **触发业务里程碑完成** + batch 域解冻 |
| **R3 batch 解冻** | RACI v1.2 5→6 域 + IMPL-PLAN-BATCH-001 v0.1 + RACI-BATCH-V1 v0.1 + C2/C3 自审 | 8M | 28.3M | **触发 8 条 L-CAND 候选自审报告** |
| **R4 季度评审准备** | L-CAND 自审报告 + L-CAND-006 立即生效路径 + L-CAND-007 CI 草案 + L-CAND-005 commit 模板 | 5M | 33.3M | **触发 Ulysses 季度评审** |
| **R5 季度评审** | 8 条 L-CAND 拍板 + L1-L14 冻结期维持评估 | 2M | 35.3M | 12/2 Q4 评审 / 累计 ≥ 35M token 触发 |

### R1 业务冲刺细节 (5.3M tokens)

| 任务 | token 预算 | DoD |
|---|---|---|
| 阶段 A 全 4 步 (per §1.2) | 800K | 1 commit 串阶段 A 完结 + 0 CrashLoopBackOff |
| 5 域 mTLS 5/5 (per §1.1) | 1.5M | 5/5 gRPC health probe SERVING |
| 22 测试函数 UT 11/11 (per §1.3) | 1.1M | 11/11 PASS (cargo test --lib) |
| 22 测试函数 E2E 准备 (per §1.3) | 1.4M | 11 E2E 函数 stub + 集成场景就绪 |
| DDD Review v0.2 维护 + 派生约束守护 | 500K | 维护已完 + 8 L-CAND 入档 |
| **R1 小计** | **5.3M** | **业务里程碑完成, 触发推送** |

### R2 Phase C 收口细节 (15M tokens)

| 任务 | token 预算 | DoD |
|---|---|---|
| 阶段 B 5 域 certs 导出 + mTLS 业务级 1+2 跳 | 4M | 业务 mTLS OK + cert fingerprint 验证 |
| 阶段 C 11 E2E 真跑 (per §1.3) | 2.75M | 11/11 PASS |
| 5 域跨域 saga (per BATCH-PLAN v0.2 W4-W6) | 1.5M | 1 套 saga PASS |
| 跨域 saga 真实交易 (player → economy → admin) | 500K | 1 笔测试交易 OK + ledger 写入 |
| batch 域 GAP-10 跨域 saga 触发 (per commit `ea4c874`) | 300K | batch → saga OK |
| 22 测试函数合并 verdict (per C8) | 1.5M | 22/22 PASS + 1 commit |
| 阶段 D1 5 域生产可用里程碑达成 | 2M | 业务里程碑 ✅ |
| 阶段 D2 batch 域 v0.1 解冻 + D3 RGS-CRITIQUE v0.2 升版 | 2.45M | 解冻公告 + v0.2 升版 |
| **R2 小计** | **15M** | **业务里程碑完成, 触发 batch 解冻** |

### R3 batch 解冻细节 (8M tokens)

| 任务 | token 预算 | DoD |
|---|---|---|
| RACI v1.2 扩展 5 域 → 6 域 (per PLAN A.3) | 500K | RACI v1.2 落档 |
| IMPL-PLAN-BATCH-001 v0.1 起草 (per PLAN A.3) | 1.5M | v0.1 起草 |
| RACI-BATCH-V1 v0.1 独立 RACI (per PLAN A.3) | 1M | v0.1 落档 |
| 12 GAP 6 跳过项 v0.2 评估 (GAP-3/4/7/9/11/12) | 3M | v0.2 评估报告 + token 预算 |
| C2 PT 文档/流程自审 | 800K | 自审报告 + 流程模板升版 |
| C3 5 域生产可用 checklist | 1.5M (5 域 × 300K) | 5 域 checklist 完成 + Ulysses 二审 |
| **R3 小计** | **≈ 8.3M** | **触发 8 条 L-CAND 候选自审报告** |

### R4 季度评审准备细节 (5M tokens)

| 任务 | token 预算 | DoD |
|---|---|---|
| 8 条 L-CAND 候选自审报告 (per §2.1) | 2.05M | 自审报告 v0.1 |
| L-CAND-006 立即生效路径 (可选) | 100K | AGENTS.md §8 例外段 |
| L-CAND-007 派生约束版本锁 CI pre-commit 草案 | 300K | 草案 v0.1 |
| L-CAND-005 业务里程碑 commit git 实证模板 (D3 commit 模板升 v0.2) | 150K | D3 commit 模板升 v0.2 |
| L-CAND-001/002/003 A 类文档减肥 准备 | 1.3M | A1/A3/A4 准备就绪 |
| L-CAND-004 SRE 拍板超时防御 token 触发实施 | 200K | 防御脚本落地 |
| R5 评审准备合计 | 900K | 评审材料 + 决策表 |
| **R4 小计** | **≈ 5M** | **触发 Ulysses 季度评审** |

### R5 季度评审细节 (2M tokens)

| 任务 | token 预算 | DoD |
|---|---|---|
| 8 条 L-CAND 拍板 (per §2.1) | 1M | 通过升 AGENTS.md, 未通过清出候选清单 |
| L1-L14 冻结期维持评估 (至 2027-03-02) | 500K | 不动 / 微调 |
| 季度评审机制升 v0.2 (新候选 12/2 → 3/2 循环) | 500K | 机制文档升 v0.2 |
| **R5 小计** | **≈ 2M** | **季度评审完成, 进入下一循环** |

---

## 8. 季度评审路线图 (token 累计触发, 不等"12/2 JST")

| 评审日 / 触发 | 入档候选 | 触发条件 | 状态 |
|---|---|---|---|
| **R4 累计 5M tokens 触发** | L-CAND-001/002/003 (A 类) + L-CAND-004/005/006/007 (L15) + L-CAND-008 (保留) | R3 业务里程碑完成 + R4 自审报告就绪 | ⏳ token 触发待 R4 累计达成 |
| **R5 累计 2M tokens 触发** | (Q4 评审后入档新候选) | R4 自审报告 + R5 拍板材料就绪 | 待启 |
| **下次评审累计触发** | (R5 评审后入档新候选) | R5 拍板完成 + 新候选入档 | 待启 |
| **冻结期满 2027-03-02 JST** | L1-L14 冻结期届满, 重新评估 | 6 个月冻结期满 | 待启 |

**L-CAND-006 例外路径** (per 8/27 安全派生约束例外条款): 安全相关可立即生效, **不等 R4 季度评审**。9/3 07:34 JST 拍板后 9/3-9/9 期间可单独 commit + 写入 AGENTS.md §8 例外段, token 预算 100K 立即花。

---

## 9. 已知缺口 (per 8/26 JST 缺标比错标, 沿用 v0.1)

- **234 commit main 领先 origin/main** 未推送 → token 累计 5.3M 触发推送 (per §0.3 v2)
- **SRE Lead 时间窗口**: 不等日历, 走 token 触发 (0.5M 内必须出拍板, 超 1M 走选项 C 推迟)
- **Phase C 阶段 C 22 测试函数**: 11 UT 立即可跑 (token 1.1M), 11 E2E 需阶段 B 完成 (token 2.75M)
- **A 类 4 条候选清单**: 不阻塞 R1 业务冲刺, R4 季度评审准备阶段处理
- **A4 HPA 资源**: R1 阶段 A4 步骤 token 350K 内可能发现 ingress/cert-manager HPA
- **派生约束 L1-L14 冻结 6 个月 (至 2027-03-02)**: 期间不增 L15, 新约束进候选清单
- **batch 域 v0.1 解冻**: token 累计 5.3M 触发, 走 14:58 拍板规则
- **OLU token 重算 (RGS-TS-001 §6.2)**: 1 人·天 ≈ 100K-300K tokens, 跟本表 token 估算对齐

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 19:00 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 9/2 19:00 JST 仓库盘点 (0 待合并先进分支) + 7 大未完成任务路线图 (W37 业务冲刺 / W38 Phase C 收口 / W39 batch 解冻 / W40 季度评审准备 / 12/2 Q4) + 8 条 L-CAND 候选清单 + 9 条已知缺口, per W37 v0.1 + RGS-BATCH-V0.1-FREEZE v0.1 + RGS-PHASE-C-PREP v0.1 + L-CANDIDATES v0.2 |
| v0.2 | 2026-09-03 07:31 | 架构师(Mavis 接手 agent per DEC-008) | §0.3 推送决策落档: Ulysses 9/3 07:31 JST 拍板**选项 2 = W37 冲刺后推送 (9/14 JST 周报 v0.3 落地后)**, 中间窗口 9/3-9/14 维持本地 234+ commit ahead 不推送, SRE 拍板悬空走选项 C 推迟 W38 推送延后到 9/15; 推送前置 4 验收 + 推送后验证回到 ≤ 20 阈值; 不采用方案 1/3 写明原因 |
| v0.3 | 2026-09-03 07:34 | 架构师(Mavis 接手 agent per DEC-008) | **token 标准推进改造**: Ulysses 9/3 07:34 JST 拍板"不要限制推进的时间, 应该用 token 标准推进" (per 8/21 JST token-OLU 框架偏好); v0.2 时间版 (W37-W40 + 9/14 锚点 + 9/9 应急检查点) **全部废止**, 改 token 预算 + token 累计触发; 推送触发 = 业务里程碑 token 累计 ≥ 5.3M (不是 9/14 日历日); §0.3 v1 废止 + v2 拍板; §1 业务里程碑每项加 token 预算 (5 域 mTLS 1.5M / 阶段 A 800K / 22 测试函数 ≈ 6.35M); §2 治理派生约束加 token 预算 (8 L-CAND 季度评审 ≈ 2.05M, C2+C3+D1 ≈ 3.5M); §3 batch 域 38 L4 任务 = 9.65M tokens (沿用 BATCH-PLAN v0.2 §3 估算), 触发解冻 = 业务 token 累计 5.3M (不是"1-2 周"); §6 紧急阻塞改 token 触发 (0.5M 内必须出拍板, 超 1M 走选项 C); **§7 token 推进路线图 (R1-R5)** 替代原 W37-W40 时间窗, 累计 ≈ 35.3M tokens, 每 R-stage 列 token 预算 + 累计 + 触发; §8 季度评审路线图改 token 累计触发, 不等"12/2 JST"; 9 条已知缺口全部去日期锚点 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
