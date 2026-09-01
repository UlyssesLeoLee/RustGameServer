# RGS-RACI-BATCH-V1 批量域 Lead 责任矩阵 v1.1 (per 2026-09-02 00:40 JST Mavis 接手代签)

> **创建日期**: 2026-09-02 00:40 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**:
> - AGENTS.md v0.4 §7 batch 域派生约束 (commit `30c7bae`)
> - WBS v0.2 §7 桶 11 Phase E (commit `84edf26`)
> - 5 域 RACI v1.1 模板 (player / economy / match / social / admin)
> **关联**:
> - 5 域 RACI: `docs/14-项目管理/RGS-RACI-{PLAYER,ECONOMY,MATCH,SOCIAL,ADMIN}-V1_*.md`
> - BATCH 4 件套: `docs/00-基準与治理/batch/RGS-BATCH-{REQUIREMENTS,BASIC,DETAILED,PLAN}-2026-09-01_v0.1.md`
> - AGENTS.md §7 batch 域派生约束: 12 条 + 5 不破坏 + 4 复用 + 3 引用

---

## 0. 一句话当前状态

batch 域是 6 域扩展中的**第 6 域** (per AGENTS.md v0.4 §7 + DEC-008),**不与 5 域 Lead 兼任** (per 8/21 JST 决策基线)。本 RACI v1.1 明确 batch 域 Lead 责任矩阵 + 决策路径 (per WBS v0.2 §4.3 拍板 3: batch 域 Ulysses 拍板门)。

## 1. 5 域 → 6 域扩展

| 域 | Lead | 代签 | 状态 |
|---|---|---|---|
| player | Ulysses | 真实身份 (8/21 5 域独立 Lead) | 🟢 v1.1 |
| economy | Ulysses | 真实身份 | 🟢 v1.1 |
| match | Ulysses | 真实身份 | 🟢 v1.1 |
| social | Ulysses | 真实身份 | 🟢 v1.1 |
| admin | Ulysses | 真实身份 | 🟢 v1.1 |
| **batch** | **(待指派 per E2)** | **Mavis 接手代签 (per WBS v0.2 §4.3 拍板 3)** | **🟡 v1.1 占位** |

**batch 域 Lead 待 RACI v1.2 / E2 落档时指派**——per BATCH REQ §0 + AGENTS.md v0.4 §7, 6 域独立 Lead 拒绝兼任。

## 2. batch 域 RACI 矩阵

### 2.1 R (Responsible) — 责任执行

| 任务 | batch Lead | 5 域 Lead | 架构师 | SRE | QA |
|---|---|---|---|---|---|
| BATCH REQ 维护 | **R** | C | A | I | I |
| BATCH BASIC 维护 | **R** | C | A | I | I |
| BATCH DETAILED 维护 | **R** | C | A | I | I |
| BATCH PLAN 维护 | **R** | C | A | I | I |
| rgs-batch-console 实现 | **R** | I | A | C | I |
| rgs-batch-backend 实现 | **R** | I | A | C | I |
| batch 域 UT+IT 落地 | **R** | I | A | I | C |
| batch 域 ST 落地 | **R** | I | A | C | C |
| 5 域 gRPC client 集成 | **R** | C (5 域各派 1 协调人) | A | I | I |
| mavis cron 告警集成 (v0.2) | **R** | I | A | I | I |
| rgs-web 深联动 (v0.2) | **R** + rgs-web Lead | I | A | I | I |

### 2.2 A (Accountable) — 责任归属

- **架构师(Mavis 接手 per DEC-008)**: batch 域架构决策 / 跨域协调 / 拍板决议 6 域同步
- **batch Lead**: batch 域所有任务最终质量, 派生决策需 Ulysses 拍板 (per WBS v0.2 §4.3 拍板 3)

### 2.3 C (Consulted) — 协商

- **5 域 Lead**: 跨域集成 (5 域 gRPC client 调用 + saga 触发 + audit 共享) 必须 5 域各派 1 协调人
- **SRE Lead**: 部署 + 资源上限 + namespace 隔离 (per BATCH REQ §10.3)
- **QA Lead**: 验收测试 + 覆盖率

### 2.4 I (Informed) — 知会

- **5 域 Lead**: batch 域决策 / 实现进度 / 跨域影响
- **架构师**: batch 域里程碑 + DDD Review 节点

## 3. batch 域决策路径 (per WBS v0.2 §4.3 拍板 3)

### 3.1 派生决策需 Ulysses 拍板

- batch 域 RACI 修订
- batch 域派生约束新增 (类似 AGENTS.md v0.4 §7 12 条)
- 跨域集成 (5 域 gRPC + saga + audit)
- 资源策略 (k3s 上限 + namespace 隔离)
- v0.2 评估项 (GAP-1~12, per BATCH REQ §9)

### 3.2 Mavis 可默认代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)

- batch 域日常 commit (1 commit 1 段, 代签三件套)
- batch 域文档维护 (REQ/BASIC/DETAILED/PLAN 修订历史)
- batch 域 UT/IT 跑测 (per L1 强约束 cargo check --tests)
- batch 域跨域协调 1-on-1 (per DDD Review 协调模板)

### 3.3 临时越界 (per L9 + WBS v0.2 §4.4)

- 部署恢复期 Mavis 可临时改 yaml, 24h 内 commit + 修订历史写明 "临时越界 + Ulysses 追认" 三件套
- 不允许扩展到: 日常 commit / feature dev / 业务实装
- 不追溯改写历史文档"审批者=—" (per 8/27 19:39 JST 决策)

## 4. batch 域 DDD Review 节点 (per AGENTS.md v0.4 §7 + WBS v0.2 §2.5)

| 节点 | 触发 | 必填 |
|---|---|---|
| E1 BATCH-IMPL-PLAN v0.2 升版 | 9/8 之前 | 5 域 Lead 签字 + batch Lead 签字 + 架构师签字 |
| E2 BATCH-RACI-V1 v0.1 | 9/8 之前 | 5 域 Lead 签字 + 架构师签字 + Ulysses 拍板 |
| E3 rgs-batch-console + rgs-batch-backend 38 L4 任务 | 9/15 之前 W1-W2 落地 | 架构师签字 + 5 域 Lead 协调签字 |
| E4 k3s 资源策略 | 9/15 之前 | SRE Lead 签字 + 架构师签字 + Ulysses 拍板 |
| E5 OLU 重算 + token-OLU 框架 | 9/22 之前 | 5 域 Lead 签字 + batch Lead 签字 + 架构师签字 |
| E6 OLU 跨 5+1 域 重算 | 9/29 之前 | 5 域 Lead 签字 + batch Lead 签字 + 架构师签字 |
| E7 ADR 升版 | 10/6 之前 | 5 域 Lead 签字 + 架构师签字 |
| E8 BATCH v0.2 评估项 (GAP-1~12) | 10/13 之前 | 5 域 Lead 签字 + batch Lead 签字 + 架构师签字 + Ulysses 拍板 |

## 5. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v1.0 | (占位) | — | 6 域扩展基线 (per AGENTS.md v0.4 §7) |
| **v1.1** | **2026-09-02 00:40 JST** | **架构师(Mavis 接手 agent per DEC-008)** | **batch 域 RACI 矩阵 + 决策路径 + DDD Review 节点 (per WBS v0.2 §4.3 拍板 3 + §2.5 桶 11 Phase E)** |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
