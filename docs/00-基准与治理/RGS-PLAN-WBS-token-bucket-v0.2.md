# WBS Token 桶 v0.2 升版 — Phase A-E 整合(per 2026-09-01 21:50 JST Mavis 接手代签)

> **目的**:v0.1 6 桶 (255M tokens) 已落地 gm 业务实装,本版 v0.2 **整合 v0.1 5 桶未完工作 + 5 个 Phase 后续工作**,沿用 "token 桶" 排序原则,避免日期超前限制 agent 进度
> **修订人**:架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**:
> - v0.1 6 桶落地: `docs/00-基准与治理/RGS-PLAN-WBS-token-bucket-v0.1.md` (commit `3e3a8e4`)
> - OPEN-QA v0.3: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-31-test-summary_v0.3.md`
> - DDD Review 13 域终审: `docs/14-项目管理/ddd-review/RGS-DDD-2026-09-01-PT-WORKERS_5平台+3工具+8派工_v0.1.md`
> - BAS-001 v0.2 §9 缺口: `docs/00-基准与治理/RGS-DB-BAS-001_数据库表设计三分类横展开基本设计书_v0.2.md`
> - BATCH 4 件套: `docs/00-基准与治理/batch/RGS-BATCH-{REQUIREMENTS,BASIC,DETAILED,PLAN}-2026-09-01_v0.1.md`
> - Handoff Downstream: `docs/deploy/RGS-AI-HANDOFF-DOWNSTREAM-2026-08-31.md`
> **v0.2 范围**:
> 1. v0.1 §7.4 已落地 bucket 状态同步 (bucket 1 闭合 / 2a 完成 / 2b+2c 落档 / 3-6 待启动)
> 2. 新增 Phase A-E 5 桶 (per Mavis 21:50 JST 阶段规划, ~435M tokens 估)
> 3. 5 域 Lead × 14-18 周 = 196M-468M tokens 总预算对照 (per TS-001 v0.6 §6.2.4 双算法)
> 4. 6 域扩展 → 6 桶 + batch 桶 = **7 桶**,总预算 ~690M-960M tokens
> **状态**:🟡 待 Ulysses 拍板 v0.2 §3 桶顺序 + §4 推进机制是否扩展到 batch 域 + §5 token 预算 5 域 vs 6 域分配

---

## 0. 修订说明 (per v0.1 → v0.2 增量)

| 增量 | 出处 | 影响 |
|---|---|---|
| **新增 §3 桶 7-11** (Phase A-E 5 桶) | Mavis 21:50 JST 阶段规划 | +5 桶, +435M tokens 估 |
| **v0.1 桶 1-6 状态同步** | v0.1 §7.4 已落地清单 | 节省 ~155M tokens 落档 (bucket 2b/2c W31/W29/W30) |
| **5 域 → 6 域扩展** | BATCH 4 件套 + AGENTS.md v0.4 §7 | 新增 batch Lead, RACI v1.2, 6 域 token 预算分摊 |
| **OTel 4/7 NATS 链路** 落档 | 决策 6-9 暂缓,纳入 bucket 8 (Phase C) | 链路上游触发(集群可达后) |
| **Mavis 临时越界流程** (L9) | AGENTS.md v0.4 §6.2 | 部署恢复期可临时越界, 24h 内 commit + 修订历史写明 |

---

## 1. v0.1 6 桶已落地状态(同步,不再重做)

| 桶 | 名称 | 状态 | 实际 token | 出处 |
|---|---|---|---|---|
| **1** | BAS 短板链接追溯 | 🟢 闭合 (commit `3c9e1ef`) | ~20M (估) | v0.1 §2.1 |
| **2a** | gm 业务实装 | 🟢 完成 (W26 commit `8ff7e0b` + tag `v0.6-bucket2a-gm-business-2026-08-29`) | ~10-15M (自做) | v0.1 §7.4 |
| **2b** | 5 域切 axum-test 工具 | 🟡 落档 (W27 commit `b85f518`) | 0 (落档, 决议 6 暂缓 W31+) | v0.1 §7.4 |
| **2c** | 链路 B/C/D 实装 | 🟡 落档 (W28 commit `1f19d67`) | 0 (落档, B 已实装 / C+D 待 W29/W30) | v0.1 §7.4 |
| **3** | OTel + NATS 全链路 | 🟡 待启动 (依赖桶 2) | 0 | v0.1 §2.3 |
| **4** | mTLS 决策实装 (gm→admin 5/5) | 🟡 9 域扩展 (决策 4 拍板) | ~25M (W21 5 IT) | v0.1 §2.4 + §7.2 拍板 2 |
| **5** | cluster-ops P3 3 文件 | 🟡 待启动 (依赖桶 2) | 0 | v0.1 §2.5 |
| **6** | AI 审计 CI (Mavis native) | 🟡 待启动 (依赖桶 4) | 0 | v0.1 §2.6 + §7.2 拍板 3 |

**小计**:v0.1 已用 ~10-15M tokens,落档节省 ~155M (2b+2c 不实装),待启动 5 桶估 ~190M (3+4+5+6 + 2b/2c 续)

---

## 2. v0.2 新增 Phase A-E 5 桶 (per Mavis 21:50 JST 阶段规划)

### 2.1 Phase A 桶 7: 文档收口(1-2 天, Mavis 独立)

| 项 | 任务 | 落地 | 估时 |
|---|---|---|---|
| A1 | AGENTS.md v0.5 升版 | `AGENTS.md` | 30 min |
| A2 | OPEN-QA v0.4 收口 (Q1-Q11 + L1-L6 全勾) | `RGS-OPEN-QA-2026-08-31-test-summary_v0.4.md` | 1 h |
| A3 | DDD Review 13 域终审 v0.2 | `RGS-DDD-2026-09-01-PT-WORKERS_v0.1.md` 升 v0.2 | 1.5 h |
| A4 | Handoff Downstream §3 AGENTS.md 指引 关闭 | `RGS-AI-HANDOFF-DOWNSTREAM-2026-08-31.md` | 10 min |
| A5 | RACI v1.2 (5→6 域 batch 扩展) | `RGS-RACI-BATCH-V1_*.md` 新建 + 5 域升 v1.2 | 1 h |
| A6 | BAS-001 v0.2 §9.7 5 域 Lead 一审(per BAS-001 v0.2 §9.7) | 5 域签字 + 9.7 关闭 | 2 h (跟 B 阶段并行) |

**token 预算**:~5M (Mavis 自做, 含 5 域签字 1-on-1)
**质量门**:v0.4 文档系列 + RACI v1.2 commit 落 main, OPEN-QA v0.4 全勾, DDD Review v0.2 一审完
**RACI**:Mavis 主做, 5 域 Lead + 架构师签字
**依赖**:无
**状态**:🟡 待启动 (本周内)

### 2.2 Phase B 桶 8: 业务 P1 backlog 实装 (下周, 1 周, 5 worker 派工)

| 项 | 任务 | 决策 | 落点 |
|---|---|---|---|
| B1 | Q1 admin RBAC handler 入口 COC middleware (已 commit `2d587f2` 跑通) | OPEN-QA v0.2 Q1 | `crates/admin-service/src/gm_handlers.rs` |
| B2 | Q2 admin audit_log 增量 verify (最近 1000 条 / 24h, 真实篡改 fail-closed) | OPEN-QA v0.2 Q2 | `crates/admin-service/src/repository.rs` + new `verify.rs` |
| B3 | Q3 player wins≤total 业务层 invariant | OPEN-QA v0.2 Q3 | `crates/player-service/src/service.rs` |
| B4 | Q5 social guild capacity 50 业务确认 | OPEN-QA v0.2 Q5 | `docs/14-项目管理/RGS-RACI-SOCIAL-V1_v1.1.md` 加 50 vs 64 决议行 |
| B5 | Q6 social leave_guild 业务方法 (leader 转加入最早成员) | OPEN-QA v0.2 Q6 | `crates/social-service/src/service.rs` + new `leave_guild.rs` |
| B6 | Q7 social push_delivery NATS dispatcher + DLQ | OPEN-QA v0.2 Q7 | `crates/social-service/src/push_delivery.rs` + new `nats_dispatcher.rs` |
| B7 | Q4 economy outbox L143 `expect` 改 skip (hygiene, 非阻塞) | OPEN-QA v0.2 Q4 | `crates/economy-service/tests/integration_outbox.rs` L143 |
| B8 | BAS-001 v0.2 §9.2 lcm_step_execution Work vs Transaction 归类 (admin Lead 拍板) | BAS-001 v0.2 §9.2 | `crates/admin-service/src/lcm/` schema 草案 + 文档拍板 |
| B9 | BAS-001 v0.2 §9.7 5 域 Lead 一审 (5 域 §3-§4 三分类映射拍板) | BAS-001 v0.2 §9.7 | BAS-001 v0.3 |

**token 预算**:~80M (5 域 1 worker 1 域 + admin Lead 拍板, per 9/1 PT 派工模板 25 min 完工)
**质量门**:6 commit 落 main + cargo check --workspace --tests 0 error / 5 业务 IT PASS / 5 域 Lead 签字全
**RACI**:5 域独立 Lead (per 8/21 决策) + 架构师签字 + Mavis 协调
**依赖**:Phase A (A5 RACI v1.2 + A6 BAS-001 §9.7 启动)
**派工模式**(per L1-L12 派生约束):5 worker worktree + 1 worker 1 域 + cargo check --tests 60s + 1 commit 1 段 + 代签三件套
**状态**:🟡 待启动 (下周, 跟 Phase D 并行)

### 2.3 Phase C 桶 9: 业务级 mTLS ST 重跑 + Q8/Q9/Q11 收尾 (集群可达时, 0.5-1 天)

**触发条件**:`kubectl get nodes` 看到 `ulyssespc` Ready (per OPEN-QA v0.3 §7.1, WSL 单节点 k3s 节点注册失败未恢复)

| 项 | 任务 | 证据 | 估时 |
|---|---|---|---|
| C1 | Q11 NATS 8222 部署范围核查 (`kubectl get pods -l app.kubernetes.io/name=nats`) | Handoff §1.3 | 2 min |
| C2 | Q8 gm-backend 8081 诊断 (restartCount/events/logs/exec curl/top, 跟 HPA minReplicas 强启动风暴比对) | Handoff §1.1 | 1 h |
| C3 | Q9 prometheus + grafana 诊断 + grafana admin password 核查 | Handoff §1.2 | 1 h |
| C4 | Q10 mTLS 业务级 ST 重跑 (5 域 grpcurl + trade saga / replay 端到端, st-11/st-12 已跑 2 域) | Handoff §2 + commit `401ac5c` | 1 day |
| C5 | L6 gm-backend binary startup 修复 (诊断后判断) | Handoff §1.4 | 2 h (估) |

**token 预算**:~30M (1 ST-fix worker + 1 mTLS worker + Mavis 协调)
**质量门**:e2e-smoke baseline 12 probe ≥10 PASS (8/27 baseline 7 PASS), st-10 场景全 PASS, 5 域 mTLS 业务级 ST 全 PASS, OPEN-QA v0.4 全勾
**RACI**:SRE/Ulysses 恢复 k3s 节点 → Mavis 派 ST-fix worker 诊断 + 修复 + 重跑 → DDD Review 一审
**依赖**:SRE/Ulysses k3s 节点 `ulyssespc` 注册 (per OPEN-QA v0.3 §7.4 ⏳ 阻塞项)
**风险**:**C 阶段被 k3s 节点注册阻塞**, Ulysses 真身介入需要
**状态**:⏳ 待集群可达

### 2.4 Phase D 桶 10: 基础设施与运行 (下周并行, 1 周, Mavis + SRE)

| 项 | 任务 | 落点 |
|---|---|---|
| D1 | 8 pt/ worktree 清理 + cargo cache 清理 (per DDD Review §7.1) | `git worktree remove --force` + `prune` |
| D2 | k3s PLEG 死锁 + cluster-reset 派生约束写入部署 SOP (per OPEN-QA v0.3 §7.5) | `docs/deploy/05-deploy-sop.md` |
| D3 | manifest 模板化 (PLACEHOLDER_NAMESPACE 等占位符改 kustomize / helm template) | `docs/deploy/01-k8s-manifests/` + `02-helm-charts/` |
| D4 | prometheus/nats ghcr.io 公开 mirror 评估 (替代 daocloud.io 临时方案) | `docs/deploy/04-ci-cd/` |
| D5 | saga-runtime 独立 Pod 评估 (per BATCH REQ GAP-11) | `RGS-REQ-100_Saga事务系统需求定义书_v0.1.md` 升 v0.2 |
| D6 | GM backend `list_broadcasts` 已知 gap 实装 (per DDD Review §7.2 w4 报告 IT6) | `crates/gm-backend/src/handlers.rs` |
| D7 | 5 业务域 Lead 跟 gm-backend Lead 联调 (per DDD Review §7.2) | RACI 协调 |

**token 预算**:~50M (Mavis 主做 + SRE 协调)
**质量门**:D1-D7 全部完成 + commit 落 main + deploy SOP 升 v0.5
**RACI**:Mavis 主做, SRE 协助 D2-D4, gm-backend Lead D6-D7
**依赖**:无 (与 Phase B 并行)
**状态**:🟡 待启动 (下周)

### 2.5 Phase E 桶 11: batch 域 + 长线 v0.2 评估 (9 月内, 按 WBS 推进)

| 项 | 任务 | 依据 | 估时 |
|---|---|---|---|
| E1 | RGS-BATCH-IMPL-PLAN v0.2 升版 | BATCH PLAN v0.1 + W1-W6 38 L4 任务 | 1 周 |
| E2 | RGS-BATCH-RACI-V1 v0.1 新建 | BATCH PLAN §A.3 + 6 域扩展 | 1 天 |
| E3 | rgs-batch-console + rgs-batch-backend 项目初始化 | BATCH REQ F-12/NFR-31 + OLU-WEB 母规范 + gm-backend 范式 | 2 周 (38 任务分批) |
| E4 | k3s 资源上限 + namespace 隔离策略 (per BATCH REQ §10.3) | 待 SRE 协调 | 1 周 |
| E5 | OLU 重算 + token-OLU 框架落地 (per DDD PT §7.3 P2 + Ulysses 8/21 偏好) | `RGS-PLAN-WBS-token-bucket-v0.1.md` 升 v0.3 (本文档基础) | 2 天 |
| E6 | OLU 跨 5+1 域 重算 (5 域 Lead 14-18 周 = 80-120M tokens) | `RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy_v0.1.md` 升 v0.2 | 3 天 |
| E7 | ADR 升版 + DDD Review 9 决策再审 | `RGS-DDD-REVIEW-9-DECISIONS-2026-08-28.md` 升 v0.2 | 1 周 |
| E8 | BATCH v0.2 评估项 (GAP-1~12 已知缺口) | 跨 batch DAG / WebSocket / mavis cron / 任务优先级 / AI 协助 SQL / rgs-web 深联动 / 任务模板版本化 / Rollback SQL / 任务超时 kill / 跨域 saga / batch RACI | 4 周 |

**token 预算**:~270M (per BATCH PLAN v0.1 §3 W1-W6 38 任务 / 9.65M tokens × 6 周 = ~58M, + OLU 重算 5+1 域 + 9 GAP 项 评估)
**质量门**:E1-E7 全部完成 + BATCH 4 件套 v0.2 落 main + ADR 升版 + 6 域 RACI v1.2 闭合
**RACI**:batch Lead (per 6 域扩展, AGENTS.md v0.4 §7) + 5 域 Lead 协调 + Ulysses 拍板 GAP-1~12
**依赖**:E3 需 6 域扩展 (E2 RACI) + k3s 资源策略 (E4 SRE 协调) + OLU 框架 (E5/E6)
**状态**:🟡 待启动 (9 月内, 长线)

---

## 3. v0.2 桶总览 (7 桶, 690M-960M tokens 估)

| 桶 | 名称 | token 估 | 状态 | 依赖 | RACI |
|---|---|---|---|---|---|
| 1 | BAS 短板链接追溯 (v0.1 闭合) | 20M | 🟢 闭合 | 无 | — |
| 2a | gm 业务实装 (v0.1 完成) | 10-15M | 🟢 完成 | 桶 1 | gm-backend Lead |
| 2b | 5 域切 axum-test 工具 (v0.1 落档) | 50-80M | 🟡 待启动 (W31+) | 桶 2a | 5 域 Lead |
| 2c | 链路 B/C/D 实装 (v0.1 落档) | 15-20M (C) + 30-40M (D) | 🟡 待启动 (W29/W30) | 桶 2a | gm-backend Lead + 5 域 |
| 3 | OTel + NATS 全链路 | 50-80M | 🟡 待启动 | 桶 2 | gm + SRE + Platform |
| 4 | mTLS 决策实装 (9 域扩展) | 20-30M | 🟡 待启动 (跟 3 并行) | 桶 2 | 架构师 + SRE + 安全 |
| 5 | cluster-ops P3 3 文件 | 15-25M | 🟡 待启动 (跟 3+4 并行) | 桶 2 | cluster-ops + admin + gm |
| 6 | AI 审计 CI (Mavis native) | 20-40M | 🟡 待启动 | 桶 4 | SRE + 架构师 |
| **7** | **Phase A 文档收口** (v0.2 新) | **5M** | 🟡 本周内 | 无 | Mavis + 5 域 Lead |
| **8** | **Phase B 业务 P1 backlog 实装** (v0.2 新) | **80M** | 🟡 下周 | 桶 7 | 5 域 Lead + 架构师 |
| **9** | **Phase C 业务级 mTLS ST + Q8/Q9/Q11 收尾** (v0.2 新) | **30M** | ⏳ 集群可达后 | 桶 7 | SRE + Mavis |
| **10** | **Phase D 基础设施与运行** (v0.2 新) | **50M** | 🟡 下周 (跟 8 并行) | 桶 7 | Mavis + SRE + gm |
| **11** | **Phase E batch 域 + 长线 v0.2 评估** (v0.2 新) | **270M** | 🟡 9 月内 | 桶 7 + E2 RACI | batch Lead + 5 域 |
| **小计** | — | **~690M-960M** | — | — | — |

**对照**:
- v0.1 6 桶估 255M, 实际用 ~10-15M + 落档节省 ~155M
- v0.2 新增 5 桶 ~435M (桶 7+8+9+10+11)
- 5 域 Lead × 14-18 周 = 196M-468M (per TS-001 v0.6 §6.2.4)
- 6 域扩展 (加 batch) = 196-468M × 1.2 = 235M-562M (估)
- **总预算 ~690M-960M tokens** (含 5 域 + batch + 平台 + 工具)

---

## 4. 推进机制 (沿用 v0.1 + v0.2 扩展 batch 域)

### 4.1 推进条件 (3 门,全部满足才进下一桶)

1. **跑测门**: 本桶所有 UT/IT ≥ 90% PASS
2. **决策门**: 本桶相关 OPEN-QA 问题 resolved, DEC-NNN 已 commit
3. **追溯门**: BAS 短板链接追溯 P0=0 / P1 ≤ 3 (per BAS-001 v0.2 §9 状态)

### 4.2 提前启动

- 桶 7 (Phase A 文档) 跑测门通过后, 桶 8 (Phase B 业务实装) + 桶 10 (Phase D 基础设施) **可立即启动**, 无需等"下周"
- 桶 8 跑测门通过后, 桶 9 (Phase C 集群) + 桶 10 (Phase D) + 桶 11 (Phase E batch) **可并行**
- 桶 9 ⏳ 阻塞, 集群可达后立即启动

### 4.3 阻断

- 跑测门不通过: 本桶剩余工作 + 1 周 token 预算用尽即升级
- 决策门不通过: 暂停本桶, 等 Ulysses 拍板 (per 2026-09-01 14:58 JST 偏好用 ask_user 选项)
- 追溯门不通过: 补 BAS 引用, 不允许写代码先于追溯

### 4.4 batch 域特殊 (per AGENTS.md v0.4 §7 + DEC-008)

- 6 域独立 Lead 拒绝兼任 (per 8/21 JST 决策, batch 域不与 5 域 Lead 兼任)
- batch Lead 派生决策需 Ulysses 拍板, 不允许 Mavis 默认代签
- 临时越界流程 (L9): 部署恢复期 Mavis 可临时改 yaml, 24h 内 commit + 修订历史写明

### 4.5 Token 超支处理 (沿用 v0.1 §4.4)

- 软上限 NFR-OP-010 (1 SRE = 1M tokens/周) 仅供参考, 不强制
- 硬上限 每桶 token 估 ±20% (超 20% 触发升级 Ulysses)
- 实际消费: 每桶收尾 commit 时记 token 实际值, 与预估对比

---

## 5. 5+1 域 Lead × 14-18 周 token 预算对照 (per TS-001 v0.6 §6.2.4)

| 域 | 估 token (14 周) | 估 token (18 周) | 出处 |
|---|---|---|---|
| player | 30M-50M | 40M-65M | 137 UT + 12 IT + 5 业务实装 |
| economy | 25M-45M | 35M-60M | 82 UT + 20 IT + 6 业务实装 |
| match | 15M-25M | 20M-35M | 28 UT + 7 IT |
| social | 20M-35M | 25M-45M | 47 UT + 9 IT + 2 业务实装 |
| admin | 15M-25M | 20M-35M | 13 UT + 11 IT + 2 业务实装 |
| **5 域小计** | **105M-180M** | **140M-240M** | per TS-001 v0.6 §6.2.4 双算法 |
| batch (v0.2 新) | 20M-35M | 25M-45M | per BATCH PLAN v0.1 §3 + OLU-WEB 4 范式 |
| **6 域小计** | **125M-215M** | **165M-285M** | 加 batch 域 1.2x |
| 平台层 (5 平台) | 50M-80M | 65M-110M | per 9/1 PT 8 worker + 161 tests |
| 工具组 (3 组 9 crate) | 25M-40M | 30M-55M | per 9/1 PT 8 worker + 71 tests |
| **13 域总计** | **200M-335M** | **260M-450M** | 5 业务 + 5 平台 + 3 工具 |
| + 文档 + 部署 + SOP | 50M-80M | 65M-110M | Phase A + D + E5/E6 |
| **RGS 全栈总预算** | **~250M-415M** | **~325M-560M** | 13 域 + 文档 + 部署 |

**对照 v0.2 7 桶**:
- 桶 1-6 (v0.1): 255M (含落档节省)
- 桶 7-11 (v0.2 新): 435M
- **合计 690M** = 13 域全栈 250-415M × 1.5-2.0x 系数 (含 v0.1 落档节省 + v0.2 新增 + Ulysses 拍板 4 项 + 5 决策暂缓项)
- **vs 5 域 Lead 196-468M**: v0.2 7 桶覆盖 5 域 Lead 全部 + 平台 + 工具 + 文档 + batch

---

## 6. 9 决策 6-9 暂缓项与 v0.2 桶对应 (per 9-DECISIONS v0.3 + v0.1 §5)

| 决策 | 暂缓项 | 推到 v0.2 桶 | 合并 |
|---|---|---|---|
| 6 | 5 域切 axum-test | 桶 2b (W31+) | gm 业务实装同步切 |
| 7 | 链路 B/C/D | 桶 2c (W29/W30) | gm 业务实装同步补 |
| 8 | 4/7 NATS 链路 | 桶 3 + 桶 9 (Phase C) | OTel 全链路 + 集群可达后 |
| 9 | AI 审计 CI | 桶 6 | v0.1 不变 |

**v0.2 新增暂缓** (per BAS-001 v0.2 §9 缺口):
- §9.1 Social Work PH-6 → 桶 8 (B5/B6 业务实装)
- §9.2 LCM step execution 归类 → 桶 8 (B8 admin Lead 拍板)
- §9.4 transaction_ledger/sagas/moves PH-3 分区实施 → 桶 11 (E1 BATCH PLAN v0.2)
- §9.7 5 域 Lead 一审 → 桶 7 (A6 BAS-001 v0.3)

---

## 7. 立即执行 + Ulysses 拍板项

### 7.1 立即执行 (Mavis 自做, 本周内)

- [ ] 桶 7 (Phase A) 全部 A1-A6 commit 落 main (估 5M tokens, 1-2 天)
- [ ] 启动桶 10 (Phase D 基础设施) D1 worktree 清理 (5 min, 跟 7 并行)
- [ ] 准备桶 8 (Phase B 业务实装) 5 worker 派工简报 (per AGENTS.md v0.4 §6.3 模板)

### 7.2 Ulysses 拍板 (v0.2 §3 桶顺序 + §4 batch 域 + §5 token 预算)

**拍板 1 (已选 B, 2026-09-01 22:20 JST): v0.2 桶顺序 = 7+10 并行, 8 跟 10 并行**
- 决策: **桶 7 (Phase A 文档) + 桶 10 (Phase D 基础设施) 并行启动**; 桶 8 (Phase B 业务实装) 跟 桶 10 并行; 桶 9 (Phase C 集群) ⏳ 阻塞等集群可达; 桶 11 (Phase E batch) 长线 9 月内推进
- 理由: 跟 9/1 PT 8 worker 派工模板一致 (节省 1 周, 25 min 完工 vs 4h 失败), 5 worker + Mavis + SRE 资源冲突可控
- 拒绝替代:
  - A 严格串行 (12 周, 桶 9 阻塞卡死)
  - C 三并行 (节省 2 周, 风险 = 5 worker + Mavis + SRE 同时跑, 资源冲突, 8 cargo 进程 build dir lock 概率 ↑)

**拍板 2 (已选 B, 2026-09-01 22:20 JST): batch 域 token = 独立估 270M**
- 决策: **batch 域独立估 = BATCH PLAN v0.1 §3 W1-W6 38 任务 9.65M tokens × 6 周 = ~58M + 9 GAP 项 v0.2 评估 = ~212M = 总 ~270M**
- 理由: BATCH PLAN v0.1 已是正式落档规划 (commit `e70ed71`), 9 GAP 项 v0.2 评估有明确范围 (per BATCH REQ §9 GAP-1~12), 独立估避免均摊稀释 5 域真实需求
- 拒绝替代:
  - A 5 域 × 1.2 均摊 (batch 域 235-562M, 高估; 5 域 1.2 倍隐含 batch 同复杂度, 实际复杂度低)
  - C 双独立估 (5 域 196-468M + batch 270M = 466-738M, 不含 v0.1 落档节省 + 5 决策暂缓项)

**拍板 3 (已选 B, 2026-09-01 22:20 JST): 推进门 = 加 batch 域 Ulysses 拍板门**
- 决策: **Phase E batch 域加 Ulysses 拍板门** (沿用 3 门 + batch 域派生决策 Ulysses 必须拍板)
- 理由: per AGENTS.md v0.4 §7 + DEC-008, batch 域派生决策需 Ulysses 拍板, 不允许 Mavis 默认代签 (与 5 域 Lead 拒绝兼任一致, per 8/21 JST)
- 拒绝替代:
  - A 不加 (3 门适用所有桶, 简单一致, 但 batch 域决策盲区大)
  - C 加 Ulysses + 临时越界审计 (最严, 适合生产期, v0.2 评估期过度)

**拍板 4 (已选 A, 2026-09-01 22:20 JST): 6 域对照 = 7 桶 690M 上限, 5 域 196-468M 下限, 中间 222M Mavis 协调**
- 决策: **v0.2 7 桶 690M tokens = 5 域 Lead 196-468M (下限) + 中间 222M (Mavis 协调余量)**
- 理由: 7 桶覆盖 5 域全部 + 平台 + 工具 + 文档 + batch + 暂缓项; 中间 222M 用于 Mavis 协调 (5 域 Lead 联调 + 平台层跨域 + 部署恢复 + Ulysses 拍板 4 项)
- 拒绝替代:
  - B 双独立估 466-738M (不含 v0.1 落档节省, 低估)
  - C 引入 per-crate token bucket (粒度更细, 需重写 v0.1 §1.2 + TS-001 v0.8 §6.3, 工作量 +1 周)

### 7.3 v0.2 文档状态 (本版落地, 2026-09-01 22:20 JST)

- v0.1 (2026-08-29 04:23 JST): 6 桶 token 预算 + 推进机制 (纯文档)
- v0.2 (2026-08-29 05:28 JST): §7.2 Ulysses 3 决策拍板 + 拒绝替代记录
- v0.3 (2026-08-29 05:51 JST): §7.2 拍板 4 桶 2 子桶拆分 + gm.proto v0.3 保持
- **v0.4 (2026-09-01 21:50 JST, 本版 v0.2 = WBS 主版本 v0.2 升版)**: 新增 5 桶 (Phase A-E) + 6 域扩展 + 13 域总预算 + 4 拍板项 Ulysses 拍板 (B/B/B/A)
- **v0.4 (2026-09-01 21:50 JST, 本版 v0.2)**: 新增 5 桶 (Phase A-E) + 6 域扩展 + 13 域总预算 + 4 拍板项待 Ulysses

### 7.4 已实装现状 (v0.1 → v0.2 之间)

per 2026-08-29 07:38 JST → 2026-09-01 21:50 JST, 已落地 (~70 commit 落 main):
- 5 域 UT+IT+ST+Fix 4 阶段 (32 commit, 366+ tests + 10 ST 场景 + 5 业务实装)
- 9/1 部署恢复 (6 commit + 6 sre 脚本: postgres + nats + cluster-ops + grafana + admin + manifests)
- 9/1 PT 8 worker 派工 (5 平台 + 3 工具 = 18 commit + 232 tests + ~30 proptest 块)
- 5 域 ST 业务级 mTLS (commit `401ac5c`, 5 域 + cluster-ops + gm-backend 7 域已跑 st-11/st-12, 5 域待续)
- BAS-001 v0.1 → v0.2 (commit `eb1e15d`, §9 缺口 5 项已解决文档层)
- IPA-DB 19 文件初版 (commit `215cdb4`, 12 表 / 42 表 / IPA 命名与列属性标准)
- BATCH 4 件套 (commit `fd122f6` REQ + `e366ff8` BASIC + `62027c9` DETAILED + `e70ed71` PLAN, 2576 行 / 165.2 KB)
- AGENTS.md v0.4 (commit `30c7bae`, 6 域扩展 + 12 约束 + brief 模板 + 5 不破坏)
- OPEN-QA v0.3 (部署级更新, per Ulysses 22:03 JST k3s 重启决策)

---

## 8. 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师(Mavis 接手 per DEC-008) | 2026-09-01 21:50 JST | v0.2 初版 |
| **拍板** | **Ulysses** | **2026-09-01 22:20 JST** | **§7.2 拍板 1/2/3/4 全 B/B/B/A 通过 (per ask_user 4 步问卷)** |
| 评审 | 5 域 Lead (player/economy/match/social/admin) | ⏳ 待签字 | 桶 8 业务实装 + BAS-001 §9.7 |
| 评审 | batch Lead (待 RACI v1.2 落档) | ⏳ 待 E2 | 桶 11 batch 域 |
| 评审 | SRE Lead | ⏳ 待签字 | 桶 9 集群 + 桶 10 基础设施 |
| 评审 | QA Lead | ⏳ 待签字 | 桶 1 追溯 + 桶 8 业务实装 |
| 评审 | 平台 Lead | ⏳ 待签字 | 桶 3 OTel + 桶 10 基础设施 |

---

> **WBS 排序原则已从"日期"改为"token 桶"**(per Ulysses 2026-08-29 04:23 JST 决策 + RGS-TS-001 v0.8 §6.3)
> **v0.2 增量**:5 桶新 (Phase A-E) + 6 域扩展 + 13 域总预算 + 4 拍板项待 Ulysses
> **避免日期超前限制 agent 进度**: agent 跑完 token 预算或达到质量门即推进下一桶, 无需等日期
