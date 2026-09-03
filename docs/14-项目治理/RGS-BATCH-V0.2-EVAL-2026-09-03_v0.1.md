# RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 — batch 域 v0.2 评估报告

> **创建日期**: 2026-09-03 12:46 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **触发**: 2026-09-03 12:36 JST 拍板 3-options-together (per ask_user): C1 派生约束 (v0.1 冻结期) 评估 6 跳过项 v0.2 实施工作量 + RACI v1.2 草案 + IMPL-PLAN-BATCH-001 起草要点
> **依据**:
> - `RGS-BATCH-V0.1-FREEZE-2026-09-02_v0.1.md` (commit `06b3091`, C1 派生约束冻结公告)
> - `RGS-BATCH-REQUIREMENTS-2026-09-01_v0.1.md` (commit `fd122f6`, 12 GAP 列表)
> - `RGS-BATCH-BASIC-DESIGN-2026-09-01_v0.1.md` (commit `e366ff8`)
> - `RGS-BATCH-DETAILED-DESIGN-2026-09-01_v0.1.md` (commit `62027c9`)
> - `RGS-BATCH-PLAN-2026-09-01_v0.2.md` (commit `e70ed71`, 38 L4 任务 / 6 周 / 9.65M tokens)
> - `RGS-RACI-BATCH-V1_批量域Lead责任矩阵_v1.1.md` (5 域 Lead 签字 ✅, batch Lead ⏳)
> - `RGS-LEAD-RACI-001 v1.1` + `RGS-DDD-2026-09-02-13域终审_v0.2.md` (per A5 待落档)
> - 8/21 JST token-OLU 框架 (per RGS-TS-001 v0.7 §6.2.2.1: 1 人·天 ≈ 100K-300K tokens)
> **配套**: AGENTS.md v0.6.10 §7 batch 域派生约束 + L-CANDIDATES.md v0.3 (8 候选 + 1 保留位)
> **作用域**: 6 域 (player / economy / match / social / admin / **batch**) + 平台层 + 工具 crate

---

## 0. 背景与触发

### 0.1 背景 (per RGS-BATCH-V0.1-FREEZE-2026-09-02 v0.1 §0)

9/1 batch 域一天 4 件套 165 KB 落地 (REQ 39 KB + BASIC 37 KB + DETAILED 49 KB + PLAN 43 KB), 速度比实现快 10 倍。截至 9/2 10:18 JST, `tools/rgs-batch-backend` 还在 W2-W6 串行 commit (`faf40a8` L14 → `ea4c874` GAP-10 fix), **文档和实现进度严重错位**。per C1 派生约束 (9/2 10:18 JST 拍板), batch 域 v0.1 4 件套冻结, 不再升 v0.2, 直至 Phase C 触发解冻。

### 0.2 触发 (per 9/3 12:36 JST 拍板 3-options-together)

9/3 12:36 JST Ulysses 拍板: **C1 派生约束冻结期评估 6 跳过项 v0.2 实施工作量**, 评估结果纳入:
1. §1 12 GAP 中 6 跳过项的 v0.2 token 预算
2. §2 RACI v1.2 (5→6 域 batch 扩展) 草案
3. §3 IMPL-PLAN-BATCH-001 v0.1 起草要点
4. §4 RACI-BATCH-V1 v0.2 (per 当前 v1.1 升版, 不是 v0.1 新建) 起草要点

**冻结期合规**: 9/2 10:18 JST 拍板 L1-L14 冻结 6 个月 (至 2027-03-02 JST), 本评估**不增 L15**, 仅作 v0.2 启动前的输入文档, 待 Phase C 触发解冻后正式落地。

### 0.3 现状速览 (per 9/3 12:46 JST git 实证)

| 项目 | 状态 | 证据 |
|---|---|---|
| v0.1 4 件套 | 🔒 冻结 v0.1 | 4 文档 commit `fd122f6` + `e366ff8` + `62027c9` + `e70ed71` (per FREEZE §1.1) |
| 12 GAP 已实现 | 6/12 (50%) | GAP-1 `0e2dc91` / GAP-2 `3f6074a`+`d5468c6`+`0190c92` / GAP-5 `eb116f6` / GAP-6 `15ff16f` / GAP-8 `eb116f6` / GAP-10 `deb5c94`+`bc63265`+`ea4c874` (per FREEZE §2) |
| 12 GAP 跳过 | 6/12 (50%) | GAP-3 / GAP-4 / GAP-7 / GAP-9 / GAP-11 / GAP-12 (per FREEZE §2 + 本评估 §1) |
| 38 L4 任务 | 6 周 / 9.65M tokens | per PLAN v0.2 §3 (W1-W6, 9.65M tokens v0.5 算法, 4.4M-15.6M v0.6 算法) |
| RACI v1.1 (5 域 + batch) | 5 域 Lead 签字 ✅, batch Lead ⏳ | per `RGS-RACI-BATCH-V1_v1.1.md` §5 (9/2 00:40 JST) |
| IMPL-PLAN-BATCH-001 | ⏳ 未起草 | per 5 域 IMPL-PLAN v0.2 范式 (per W37 WBS §2.5 桶 11 E1) |
| Phase C 状态 | ⏳ 待启 | per `RGS-PHASE-C-MAVIS-PHASE-A-2026-09-03_v0.1.md` + `RGS-PHASE-C-PREP-2026-09-02_v0.1.md` |

---

## 1. 12 GAP 6 跳过项 v0.2 评估

> **评估方法**: per 8/21 JST token-OLU 框架 (RGS-TS-001 v0.7 §6.2.2.1), 1 人·天 ≈ 100K-300K tokens, 取中位 200K/人·天 作预算基准 + 上下界 100K/300K 写明风险。L4 任务拆分按 PLAN v0.2 §3 范式 (每 L4 ≤ 2 人·天 或 ≤ 500K tokens)。

### 1.1 GAP-3 WebSocket 推送 (F-28) — 替换 30s 轮询

| 维度 | 评估 |
|---|---|
| **F 编号** | F-28 (per REQ §3.3) |
| **当前 v0.1** | 30s 轮询 task 进度 (per F-6 + NFR-27) |
| **v0.2 目标** | WebSocket 双向推送 + SSE-WS bridge (per GAP-2 SSE 已实现 commit `3f6074a`) |
| **L4 任务数** | 3 (WS endpoint 1 + 客户端订阅 1 + SSE-WS bridge 1) |
| **人·天** | 2.5 (中位) |
| **token (v0.5 算法 200K/人·天)** | 500K (取中位) |
| **token (v0.6 算法 100K-300K/人·天)** | 250K-750K |
| **实施周次** | 解冻后 W1 (1 worker 1 周) |
| **前置** | F-6 任务进度 (✅ 已实现) + GAP-2 SSE endpoint (commit `3f6074a`) |
| **风险** | actix-web ws 协议 0.13 集成 (per gm-backend 范式 + 8/27 ST 实践) |
| **配套** | rgs-batch-console 7 页面改订阅 WS 替代轮询 (per BA-W4-7 部分回填) |
| **实施验收** | WS endpoint `/api/v1/tasks/{id}/ws` + console 订阅 + SSE-WS 兼容 + 单元测试 ≥ 3 |

### 1.2 GAP-4 mavis cron 告警 (F-23 / IR-4) — 失败时通知

| 维度 | 评估 |
|---|---|
| **F 编号** | F-23 (per REQ §3.2) + IR-4 (per REQ §5) |
| **当前 v0.1** | 无告警通知 (per REQ §1.3 痛点 5) |
| **v0.2 目标** | mavis cron self-reminder 失败告警 (per OLU-WEB IR-8 v0.2 实践) |
| **L4 任务数** | 2 (mavis cron 集成 1 + 失败检测 hook 1) |
| **人·天** | 1.5 (中位) |
| **token (v0.5 算法)** | 300K |
| **token (v0.6 算法)** | 150K-450K |
| **实施周次** | 解冻后 W2 (1 worker 1 周) |
| **前置** | mavis runtime 存在 (per 5 域派生, 需查 mavis cron API 文档) |
| **风险** | 依赖 mavis cron 稳定; 软依赖 (失败 fallback: console 标红 + 站内信) |
| **配套** | rgs-batch-console 失败任务标红 + trace_id 跳转 |
| **实施验收** | mavis cron self-reminder API 集成 + 失败 1 次 → 5 min 内通知 + UT ≥ 2 + IT 1 |

### 1.3 GAP-7 任务优先级 (F-30) — 多 worker 池 + 优先级队列

| 维度 | 评估 |
|---|---|
| **F 编号** | F-30 (per REQ §3.3) |
| **当前 v0.1** | FIFO 队列 (per BA-W2-4 worker_pool + 限流) |
| **v0.2 目标** | 多 worker 池 (P0/P1/P2 三池) + 优先级队列 + 5 域 RPM 重配置 |
| **L4 任务数** | 4 (priority queue schema 1 + 多 worker 池路由 1 + 5 域 RPM 重配 1 + UI 优先级选择 1) |
| **人·天** | 4.0 (中位) |
| **token (v0.5 算法)** | 800K |
| **token (v0.6 算法)** | 400K-1200K |
| **实施周次** | 解冻后 W2-W3 (1 worker 2 周) |
| **前置** | BA-W2-4 worker_pool (✅ 已实现) + BA-W3-2/3 调度器 (✅ 已实现) + worker_pool M-4 RPM 配置 |
| **风险** | 优先级反转 / 饥饿; 缓解: 加 aging (等待时间越长优先级越高) |
| **配套** | rgs-batch-console 任务提交页加优先级选择 + worker_pool admin UI |
| **实施验收** | 3 优先级池 + 优先级反转 UT + 5 域 RPM 重配生效 + 端到端 IT 1 |

### 1.4 GAP-9 任务模板版本化 (M-2 增强) — version + history + 回滚

| 维度 | 评估 |
|---|---|
| **F 编号** | F-19 增强 (per REQ §3.2 + §4.1 M-2) |
| **当前 v0.1** | 仅最新版本可用 (per REQ §9.1 GAP-8) |
| **v0.2 目标** | task_template.version 字段 + history table + UI 回滚按钮 |
| **L4 任务数** | 3 (schema migration 1 + history table 1 + UI 回滚 1) |
| **人·天** | 2.5 (中位) |
| **token (v0.5 算法)** | 500K |
| **token (v0.6 算法)** | 250K-750K |
| **实施周次** | 解冻后 W1 (1 worker 1 周) |
| **前置** | BA-W1-5 PG schema + task_template M-2 (✅ 已实现) |
| **风险** | 模板回滚副作用 (回滚时其他正在执行任务); 缓解: 回滚时锁当前执行 + warning |
| **配套** | rgs-batch-console templates 页加版本列表 + diff 视图 + 回滚按钮 |
| **实施验收** | schema migration + history table 落地 + UI 回滚 + UT ≥ 3 + IT 1 |

### 1.5 GAP-11 任务超时 kill (NFR-24 强化) — worker 强制 kill

| 维度 | 评估 |
|---|---|
| **F 编号** | F-22 强化 (per REQ §3.2 + NFR-24) |
| **当前 v0.1** | 仅标记超时 (per BA-W2-5 + F-22) |
| **v0.2 目标** | tokio task abort + 资源回收 + DLQ 自动入队 |
| **L4 任务数** | 2 (tokio task abort 1 + 资源回收 + DLQ 1) |
| **人·天** | 1.5 (中位) |
| **token (v0.5 算法)** | 300K |
| **token (v0.6 算法)** | 150K-450K |
| **实施周次** | 解冻后 W2 (1 worker 1 周) |
| **前置** | F-8 失败重试 + DLQ (✅ 已实现) + tokio task management (BA-W2-4) |
| **风险** | kill 时未完成副作用 (transaction 未 commit); 缓解: 用 tokio CancellationToken 协作式取消 |
| **配套** | rgs-batch-console 任务详情页显示 kill 状态 + reason |
| **实施验收** | tokio task abort + 资源回收 + DLQ 自动入队 + UT ≥ 3 + IT 1 (模拟长跑任务 timeout) |

### 1.6 GAP-12 batch 域 Lead RACI 同步 — RACI v1.1 → v1.2 升版

| 维度 | 评估 |
|---|---|
| **F 编号** | REQ §10.2 同步事项 + AGENTS.md §3.2 RACI v1.2 待 A5 落档 |
| **当前 v0.1** | RGS-RACI-BATCH-V1 v1.1 已落地 (commit 9/2 00:40 JST, 5 域 Lead 签字 ✅, batch Lead ⏳) |
| **v0.2 目标** | RGS-RACI-BATCH-V1 v1.2 升版 (5 域 Lead + batch Lead 全部签字 + 架构师签字 + Ulysses 拍板) + RGS-LEAD-RACI-001 v1.1 → v1.2 (5→6 域 batch 扩展) |
| **L4 任务数** | 1 (文档升版 + 签字流程) |
| **人·天** | 0.5 (纯文档, 不写代码) |
| **token (v0.5 算法)** | 100K |
| **token (v0.6 算法)** | 50K-150K |
| **实施周次** | 解冻后立即 (0.5 人·天) |
| **前置** | 5 域 Lead 全部 ✅ 签字 (per 6 worktree 派工 9/1-9/2 23:57 JST 6 merge commit 落地) + batch Lead 指派 (per E2 拍板) + Ulysses 拍板 |
| **风险** | Ulysses 拍板时间窗口 (per 9/2 已知缺口 §6); 缓解: 拍板需求 ask_user + 给 3 选项 |
| **配套** | RGS-LEAD-RACI-001 v1.1 → v1.2 升版 (5 域 → 6 域 batch 扩展) + A5 落档 |
| **实施验收** | RGS-RACI-BATCH-V1 v1.2 commit + RGS-LEAD-RACI-001 v1.2 commit + 6 域 Lead 全部签字 ✅ + 架构师签字 ✅ + Ulysses 拍板 ✅ |

---

## 2. RACI v1.2 (5→6 域 batch 扩展) 草案

### 2.1 现状 (per 9/3 12:46 JST git 实证)

| 文档 | 版本 | 状态 | 签字栏 |
|---|---|---|---|
| `RGS-LEAD-RACI-001` | v1.1 | 🟢 5 域独立 Lead + 签字 ✅ | 5 域 Lead ✅ + 架构师 ✅ + 平台/集群/SRE/DBA/安全/PM ⏳ |
| `RGS-RACI-PLAYER-V1` | v1.1 | 🟢 5 域独立 RACI | player Lead ✅ + 架构师 ✅ |
| `RGS-RACI-ECONOMY-V1` | v1.1 | 🟢 | economy Lead ✅ + 架构师 ✅ |
| `RGS-RACI-MATCH-V1` | v1.1 | 🟢 | match Lead ✅ + 架构师 ✅ |
| `RGS-RACI-SOCIAL-V1` | v1.1 | 🟢 | social Lead ✅ + 架构师 ✅ |
| `RGS-RACI-ADMIN-V1` | v1.1 | 🟢 | admin Lead ✅ + 架构师 ✅ |
| `RGS-RACI-BATCH-V1` | v1.1 | 🟡 6 域扩展版, batch Lead ⏳ | 5 域 Lead ✅ + 架构师 ✅ + batch Lead ⏳ (per 9/2 00:40 JST 落地) |

### 2.2 v1.2 草案 (per 5→6 域 batch 扩展 + A5 待落档)

| 项目 | v1.1 → v1.2 升版内容 |
|---|---|
| **RGS-LEAD-RACI-001** | v1.1 (5 域) → v1.2 (6 域 = 5 域 + batch) — 新增 batch Lead 签字栏 + batch 域 12 GAP 责任矩阵 + 跨域协调 1-on-1 流程 (per RACI-BATCH-V1 v1.1 §3.1) |
| **RGS-RACI-BATCH-V1** | v1.1 (5 域 Lead 签字 ✅, batch Lead ⏳) → v1.2 (batch Lead 签字 ✅ + Ulysses 拍板 ✅) — 加 batch Lead 真实身份 + 6 域协调 RACI 矩阵 + v0.2 12 GAP 6 跳过项 责任分配 |
| **签字栏扩展** | 5 域 Lead ✅ + batch Lead ⏳ → ✅ + 架构师 ✅ + 平台/集群/SRE/DBA/安全/PM ⏳ → ✅ (per A5 拍板) |
| **A5 落档路径** | per `RGS-DDD-2026-09-02-13域终审_v0.2.md` §4.1 + `RGS-BATCH-V0.1-FREEZE-2026-09-02 v0.1` §3.3 |

### 2.3 升版步骤 (per 9/3 12:46 JST 评估)

1. **Phase C 触发** (per RGS-BATCH-V0.1-FREEZE §3.1) — 5 域 E2E 业务 mTLS 跑通 + Phase C SRE 介入完成
2. **batch Lead 指派** (per WBS v0.2 §2.5 桶 11 E2) — Ulysses 拍板, 6 域独立 Lead
3. **RACI-BATCH-V1 v1.2 起草** (本评估 §4 + 0.5 人·天 token 预算)
4. **RGS-LEAD-RACI-001 v1.2 起草** (1.0 人·天 token 预算, 6 域签字栏)
5. **6 域 Lead 签字** (per 8/21 JST 独立基线, Ulysses = 5 域 Lead 真实身份 + batch Lead 待 E2 指派)
6. **Ulysses 拍板** (per ask_user 3 选项: A 升 v1.2 / B 维持 v1.1 6 个月 / C 重新评估 12/2 季度评审)

### 2.4 token 预算

- 文档起草: 100K (RACI-BATCH-V1 v1.2) + 200K (RGS-LEAD-RACI-001 v1.2) = **300K (v0.5 算法) / 150K-450K (v0.6 算法)**
- 签字流程: 50K (6 域 Lead 确认 + 拍板) = **50K**
- **合计**: 350K (v0.5) / 200K-500K (v0.6) ≈ **0.5-1.0 人·天**

---

## 3. IMPL-PLAN-BATCH-001 v0.1 起草要点

### 3.1 范式 (per 5 域 IMPL-PLAN v0.2)

| 5 域 | commit | v0.x |
|---|---|---|
| `RGS-IMPL-PLAN-PLAYER-001` | — | v0.2 |
| `RGS-IMPL-PLAN-ECONOMY-001` | — | v0.2 (per §3 RACI 矩阵 + §A 已知缺口 3 段) |
| `RGS-IMPL-PLAN-MATCH-001` | — | v0.2 |
| `RGS-IMPL-PLAN-SOCIAL-001` | — | v0.2 |
| `RGS-IMPL-PLAN-ADMIN-001` | — | v0.2 |
| `RGS-IMPL-PLAN-SAGA-001` | — | v0.1 |
| `RGS-IMPL-PLAN-LCM-001` | — | v0.1 |
| `RGS-IMPL-PLAN-CDN-001` | — | v0.1 |
| **RGS-IMPL-PLAN-BATCH-001** | **⏳ 待起草** | **v0.1 (草案)** |

### 3.2 v0.1 草案要点 (per 5 域范式 + DDD Review v0.2 §4.3 E1)

| 段 | 内容 | 来源 |
|---|---|---|
| **头表** | 文档编号 + v0.1 + 父文档 (RGS-WBS-001 v0.3 + RGS-BATCH-PLAN-2026-09-01 v0.2) + 源详细设计 (RGS-BATCH-DETAILED v0.1) + 适用范围 (tools/rgs-batch-backend + tools/rgs-batch-console) + 目标基线 (Rust 1.98 + Node 22 + actix-web 4.14.1 + PostgreSQL 18.6 + K3s) + 责任人 (batch 域 Lead) + 触发 (WBS v0.2 §2.5 桶 11 E1) | per 5 域 IMPL-PLAN 头表范式 |
| **修订历史** | v0.1 初始: 域职责 + 实施阶段 + 验收 + §3 RACI 矩阵 (per RGS-LEAD-RACI-001 v1.2) + §A 已知缺口 3 段 | per 5 域 v0.2 升版增量 |
| **§1 域职责** | 6 周落地 38 L4 任务 + 16 张 PG 表 + 37 API endpoint + 6 周 9.65M tokens (per PLAN v0.2 §3) + 5 不破坏 (per BASIC §6.2) + 4 复用 (per BASIC §6) + 3 引用 (per REQ §0) | per PLAN v0.2 + BASIC v0.1 + REQ v0.1 |
| **§2 实施阶段** | 8 任务簇 × 4 L4 任务 = 32 L4 (per 5 域 v0.2 范式, 5 域是 8×4=32) | per 5 域 IMPL-PLAN v0.2 §2 范式 |
| **§3 RACI 矩阵** | 6 治理角色 × 7 实施任务 RACI 映射 (per RGS-LEAD-RACI-001 v1.2, 5→6 域 batch 扩展) | per RGS-ADR-0055 v0.1 §4 |
| **§4 验收** | 5 类 + 5 性能基准 (per REQ v0.1 §8.1 + §8.2) + 12 GAP 中 6 跳过项 v0.2 评估 (per 本评估 §1) | per REQ v0.1 §8 |
| **§A 已知缺口** | 3 段: 6 跳过项 v0.2 工作量 (per 本评估 §1) + 冻结期长度 (per FREEZE §5) + 5 域 binary 未来调外部 LLM 未登记 (per OLU-WEB F-W5 + REQ F-W3) | per 5 域 IMPL-PLAN v0.2 §A 范式 |

### 3.3 token 预算

- 文档起草: **500K (v0.5 算法) / 250K-750K (v0.6 算法)** ≈ 1.5-2.5 人·天
- 实施: 9.65M tokens (per PLAN v0.2 §3, 6 周)
- **合计**: 9.65M (实施) + 500K (起草) = **10.15M tokens** ≈ 35-50 人·天

---

## 4. RACI-BATCH-V1 v0.2 起草要点 (per 当前 v1.1 升版, 不是 v0.1 新建)

### 4.1 范式 (per 5 域独立 RACI v1.1 + v1.1 → v1.2 升版路径)

> **注意**: per 9/3 12:46 JST git 实证, `RGS-RACI-BATCH-V1_v1.1.md` 已落地 (9/2 00:40 JST, 5 域 Lead 签字 ✅, batch Lead ⏳), v0.2 升版**不是新建**, 是 v1.1 → v1.2 升版。

### 4.2 v1.2 升版要点 (per v1.1 现状 + A5 待落档)

| 段 | v1.1 现状 | v1.2 升版增量 |
|---|---|---|
| **§0 一句话当前状态** | 6 域扩展第 6 域 + 独立 Lead 拒绝兼任 | 加 v0.2 6 跳过项评估引用 (本评估 §1) + A5 落档路径 (per DDD v0.2 §4.1) |
| **§1 5→6 域扩展表** | 5 域 Lead + batch Lead ⏳ | batch Lead ✅ + 真实身份 (per E2 拍板) |
| **§2 RACI 矩阵** | 2.1 R + 2.2 A + 2.3 C + 2.4 I | 加 6 域协调 1-on-1 流程 (per RACI-BATCH-V1 v1.1 §3.1) + 6 跳过项 v0.2 责任分配 (本评估 §1) |
| **§3 决策路径** | 3.1 派生决策需 Ulysses 拍板 + 3.2 Mavis 可默认代签 + 3.3 临时越界 (L9) | 加 3.4 v0.2 6 跳过项拍板路径 (per ask_user 3 选项) |
| **§4 DDD Review 节点** | E1-E8 (per WBS v0.2 §2.5 桶 11) | E1-E8 + 6 跳过项 v0.2 节点 (本评估 §1 + §3.3) |
| **§5 5 域 Lead 签字栏** | 5 域 Lead ✅ + batch Lead ⏳ | 6 域 Lead 全部 ✅ + 架构师 ✅ + 平台/集群/SRE/DBA/安全/PM ⏳ → ✅ (per A5 拍板) |
| **§6 修订历史** | v1.0 占位 + v1.1 9/2 00:40 JST | 加 v1.2 升版 (per Phase C 触发 + A5 落档) |

### 4.3 token 预算

- 文档起草: **100K (v0.5 算法) / 50K-150K (v0.6 算法)** ≈ 0.5 人·天
- 签字流程: **50K** ≈ 0.25 人·天
- **合计**: **150K (v0.5) / 100K-200K (v0.6)** ≈ 0.5-1.0 人·天

---

## 5. token 预算汇总 (per 8/21 JST token-OLU 框架)

### 5.1 v0.2 6 跳过项 + RACI v1.2 + IMPL-PLAN-BATCH-001 合计

| 项目 | 人·天 (中位) | token v0.5 算法 (200K/人·天) | token v0.6 算法 (100K-300K/人·天) |
|---|---:|---:|---:|
| GAP-3 WebSocket 推送 | 2.5 | 500K | 250K-750K |
| GAP-4 mavis cron 告警 | 1.5 | 300K | 150K-450K |
| GAP-7 任务优先级 | 4.0 | 800K | 400K-1200K |
| GAP-9 任务模板版本化 | 2.5 | 500K | 250K-750K |
| GAP-11 任务超时 kill | 1.5 | 300K | 150K-450K |
| GAP-12 RACI 同步 | 0.5 | 100K | 50K-150K |
| **6 跳过项小计** | **12.5** | **2.5M** | **1.25M-3.75M** |
| + RACI v1.2 (RGS-LEAD-RACI-001 + RACI-BATCH-V1) | 1.5 | 300K | 150K-450K |
| + IMPL-PLAN-BATCH-001 v0.1 起草 | 2.0 | 500K (不含 9.65M 实施) | 250K-750K |
| + RGS-LEAD-RACI-001 v1.2 起草 | 1.0 | 200K | 100K-300K |
| **v0.2 评估+文档合计** | **17.0** | **3.5M** | **1.75M-5.25M** |
| + 6 跳过项 v0.2 实施 (含在 IMPL-PLAN-BATCH-001) | 35-50 | 9.65M (per PLAN v0.2) | 4.4M-15.6M (per PLAN v0.2) |
| **v0.2 全量合计** | **52-67** | **13.15M** | **6.15M-20.85M** |

### 5.2 NFR-OP-010 双轨校验 (per RGS-TS-001 v0.7 §6.2.4)

- **人·天轨**: 52-67 人·天 / 6 周 (实施) + 17 人·天 (评估+文档) = 69-84 人·天 / 8 周 (含 2 周评估) = **8.6-10.5 人·天/周 ≤ 20 ✓ 绿**
- **token 轨 (v0.5)**: 13.15M / 8 周 = 1.64M tokens/周 ≤ 20M ✓ 绿
- **token 轨 (v0.6)**: 6.15M-20.85M / 8 周 = 0.77M-2.6M tokens/周 ≤ 20M ✓ 绿
- **留足余量**: v0.6 下界 6.15M / 8 周 = 768K tokens/周 = 3.8% NFR 上限 ✓

### 5.3 冻结期合规 (per C1 派生约束)

- **本评估不实施**: 评估报告本身 = 17 人·天 / 3.5M tokens (v0.5), 实施待 Phase C 触发解冻
- **本评估不增 L15**: 仅作 v0.2 启动前的输入文档, 不写代码, 走 L-CANDIDATES.md 季度评审路径 (12/2 JST)

---

## 6. 解冻触发条件 (per RGS-BATCH-V0.1-FREEZE §3 + 本评估增量)

### 6.1 硬性条件 (满足后自动解冻, 沿用 FREEZE §3.1)

| 条件 | 来源 | 当前状态 (per 9/3 12:46 JST) |
|---|---|---|
| 5 域 E2E 业务 mTLS 跑通 | D1 派生约束 (per v0.1.1 §3.4) | ⏳ Phase C 介入后 (per RGS-PHASE-C-MAVIS-PHASE-A-2026-09-03_v0.1.md + 9/3 12:36 JST 拍板 main-mtls-mock commit `fa32bab` 实证 5 域 15/15 passed) |
| Phase C SRE 介入完成 | RGS-PHASE-C-SRE-HANDOFF v0.1 23 checklist 步骤 | ⏳ 阶段 A 4 步待执行 (per RGS-PHASE-C-MAVIS-PHASE-A v0.1) |
| 5 域 + gm-backend 业务 E2E 22 测试函数真跑 | RGS-TEST-RUN-PLAN v0.1 (commit `82671df`) | ⏳ Phase C 介入后 |

### 6.2 软性条件 (满足后建议解冻, 沿用 FREEZE §3.2)

| 条件 | 来源 |
|---|---|
| 5 域生产可用 checklist 完成 (per v0.1.1 §9.4 里程碑重定义) | C3 派生约束 |
| 业务 commit ahead 回到 ≤ 20 阈值 (per v0.1.1 §9.4) | 全局 |
| **新增 (本评估)**: RACI v1.2 (5→6 域 batch 扩展) 升版完成 | A5 拍板 (per DDD v0.2 §4.1) |
| **新增 (本评估)**: 6 跳过项 v0.2 评估报告 (本评估) Ulysses 拍板 ✅ | per ask_user 3 选项 (per 9/3 12:36 JST 拍板规则) |

### 6.3 解冻流程 (per 14:58 拍板规则, 沿用 FREEZE §3.3)

1. Phase C 介入完成 + 22 测试函数真跑 = Mavis 写 `RGS-BATCH-V0.1-UNFREEZE-2026-XX-XX_v0.1.md` 公告
2. Ulysses 拍板 (per ask_user)
3. batch 域 v0.2 文档回归, 4 件套解除冻结, 12 GAP 中 6 跳过的进入 v0.2 backlog
4. **新增 (本评估)**: 6 跳过项 v0.2 工作量按本评估 §1 派工 (per L12 派生约束 3 选项 + per-worker CARGO_TARGET_DIR + staggered 30s)

---

## 7. 派生约束守护

### 7.1 已落地派生约束 (per 9/3 12:46 JST 现状)

| 约束 | 状态 | 本评估合规 |
|---|---|---|
| **L1-L14 冻结期 (per 9/2 10:18 JST 拍板)** | 🔒 6 个月 (至 2027-03-02) | ✅ 本评估不增 L15, 走 L-CANDIDATES.md 季度评审路径 |
| **8/27 11:06 JST 凭据硬 ban** (env value 永不打**印值**) | ✅ 永久 | ✅ 本评估 0 env value, 仅引用 env var 名 (BATCH_DB_PASSWORD / GRPC_CERT_PATH_*) |
| **8/27 19:39/20:56/21:59 JST 代签三次强化** (Mavis 默认代签 Ulysses) | ✅ 永久 | ✅ 本评估修订人 = Ulysses — Mavis 接手, 审批 = 架构师(Mavis 接手) |
| **8/26 04:30 JST 禁回溯叙事** (per X 历史形态 禁止) | ✅ 永久 | ✅ 本评估 0 回溯叙事, 引用 commit SHA 实证 |
| **8/26 缺标比错标** (显式列已知缺口) | ✅ 永久 | ✅ §8 已知缺口 3 段 |
| **9/3 12:36 JST L12 正式 (5 worker 派工 3 选项 + per-worker CARGO_TARGET_DIR + staggered + DoD 简报明文 worker 不 commit)** | ✅ 升正式 (per L-CAND-009) | ✅ 本评估走"worker 不 commit, 报告即可"路径, 主会话统一 git add N files + 1 commit |
| **9/1 18:30 JST DB 横展三分类** (Work / Transaction / Master) | ✅ 永久 | ✅ batch 16 张表 100% 归类 (per REQ §4.4) |
| **9/1 14:58 JST 拍板用选项** (ask_user) | ✅ 永久 | ✅ 解冻流程 3 选项 + 6 跳过项拍板路径 3 选项 |
| **9/1 13:03/13:05 JST envoy 独立 deployment** (不选 nginx, 不选 istio sidecar) | ✅ 永久 | ✅ batch envoy 独立 deployment (per REQ §0 + F-11) |
| **6 域独立 Lead 拒绝兼任** (per 8/21 JST 基线) | ✅ 永久 | ✅ batch 域独立 Lead (per REQ F-16 + R-5) |
| **9/2 10:18 JST C1 派生约束 (batch v0.1 冻结)** | ✅ 永久 | ✅ 本评估不实施, 评估 + 文档不增 L15 |

### 7.2 L-CANDIDATES 入档 (per 9/2 11:00 JST 流程)

- **本评估不增 L15**: 6 跳过项 + RACI v1.2 + IMPL-PLAN-BATCH-001 均为 batch 域 v0.2 实施内容, 走批次 v0.2 派工路径, 不需 L15 派生约束
- **保留位 L-CAND-008**: 待 L1-L14 冻结期内 Mavis 发现 (per 12/2 季度评审)
- **走 12/2 季度评审**: 本评估作为 v0.2 启动输入, 不阻塞 W37 sprint

---

## 8. 已知缺口 (per 8/26 JST 缺标比错标)

### 8.1 评估数据缺口

| # | 缺口 | 影响 | 待补阶段 |
|---|---|---|---|
| GAP-EVAL-1 | mavis cron 实际 API 文档未读 (per GAP-4 §1.2) | GAP-4 token 估算下限风险 | 解冻后 W2 派工前必读 mavis cron 文档 |
| GAP-EVAL-2 | actix-web ws 0.13 实际版本未 grep workspace 依赖 (per GAP-3 §1.1) | GAP-3 token 估算下限风险 | 解冻后 W1 派工前必跑 L3 grep `axum\|hyper\|warp\|actix\|rocket` Cargo.toml |
| GAP-EVAL-3 | tokio task abort 副作用实测未跑 (per GAP-11 §1.5) | GAP-11 token 估算下限风险 | 解冻后 W2 派工前必跑 CancellationToken 协作式取消验证 |
| GAP-EVAL-4 | priority queue 库选型未拍板 (per GAP-7 §1.3) | GAP-7 token 估算上限风险 | 解冻后 W2 派工前必走 ask_user 3 选项 (tokio priority queue / crossbeam / 自研) |
| GAP-EVAL-5 | 历史回滚 schema migration 工具未拍板 (per GAP-9 §1.4) | GAP-9 token 估算上限风险 | 解冻后 W1 派工前必走 ask_user 3 选项 (sqlx migrate revert / refinery / 自研) |
| GAP-EVAL-6 | RACI v1.2 跨域协调 1-on-1 流程未细化 (per §2.2 + §4.2) | 升版工作量 +0.5 人·天 | 解冻后 W0 派工前必出 RACI v1.2 升版模板 |

### 8.2 流程缺口

| # | 缺口 | 影响 | 待补阶段 |
|---|---|---|---|
| GAP-FLOW-1 | Ulysses 拍板时间窗口不定 (per FREEZE §5) | 升版 RACI v1.2 时间未定 | Phase C 触发后 24h 内 ask_user 拍板 |
| GAP-FLOW-2 | batch Lead 真实身份待 E2 拍板 (per FREEZE §4 + RACI-BATCH-V1 v1.1 §1) | RACI v1.2 签字栏 ⏳ | E2 拍板后 24h 内补 |
| GAP-FLOW-3 | 6 跳过项实际工作 vs 估算 偏差 (per §1 v0.5/v0.6 算法) | 实际 token 流未知 | 解冻后每 L4 任务 commit 时填"实际 token"段 (per L13 自指字段) |
| GAP-FLOW-4 | Phase C SRE 介入完成时间未定 (per §6.1) | 解冻时间未定 | per RGS-PHASE-C-MAVIS-PHASE-A v0.1 + RGS-PHASE-C-PREP v0.1 |
| GAP-FLOW-5 | 5 域 binary 未来调外部 LLM 未登记 (per OLU-WEB F-25 + FREEZE §5) | batch 任务 token 估算不准 | v0.2 评估 (per OLU-WEB F-25 + R-6) |
| GAP-FLOW-6 | 6 跳过项中 GAP-4 软依赖 mavis cron (per §1.2) | v0.2 落地节奏受阻 | 软依赖 fallback: console 标红 + 站内信 |

### 8.3 文档缺口

| # | 缺口 | 影响 | 待补阶段 |
|---|---|---|---|
| GAP-DOC-1 | IMPL-PLAN-BATCH-001 v0.1 尚未起草 (per REQ §10.2 + WBS v0.2 §2.5 桶 11 E1) | batch 域实施计划缺失 | 解冻后 W0 起草 (per §3) |
| GAP-DOC-2 | RACI-LEAD-RACI-001 v1.1 平台/集群/SRE/DBA/安全/PM 签字 ⏳ (per A5 落档) | RACI v1.2 签字栏不全 | A5 拍板后 24h 内补 |
| GAP-DOC-3 | RACI-BATCH-V1 v1.1 修订历史 v0.1 占位未补 (per v1.1 §6 v1.0 占位) | 文档治理基线 | 解冻后 v1.2 升版时合并补 |
| GAP-DOC-4 | RGS-LEAD-RACI-001 v1.1 5 域 batch 扩展段未起草 (per AGENTS.md §3.2 RACI v1.2 待 A5 落档) | RACI v1.2 升版工作量 +1.0 人·天 | 解冻后 v1.2 起草时合并补 (per §3 + §4) |

---

## 9. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 12:46 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: batch 域 v0.2 评估报告 (12 GAP 6 跳过项 token 预算 + RACI v1.2 草案 + IMPL-PLAN-BATCH-001 v0.1 起草要点 + RACI-BATCH-V1 v1.2 升版要点 + 17 人·天 / 3.5M tokens 预算 + 解冻触发条件 + 派生约束守护 + 已知缺口 3 段), per 9/3 12:36 JST 拍板 3-options-together + C1 派生约束冻结期 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
