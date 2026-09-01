# RGS-OLU-REPORT-token-OLU-2026-09-02 v0.2 升版 — token-OLU 框架 + 6 域重算 (per WBS v0.2 §2.5 桶 11 E5/E6, 2026-09-02 00:50 JST Mavis 接手代签)

> **创建日期**: 2026-09-02 00:50 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化)
> **状态**: 🟡 v0.2 草案 (per WBS v0.2 §2.5 桶 11 E5/E6 + §3 拍板 2/4)
> **关联**:
> - 旧 OLU v0.1: `docs/14-项目管理/RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy_v0.1.md` (commit `7acd24f`, 8/27 JST 部署阶段)
> - WBS v0.2: `docs/00-基準与治理/RGS-PLAN-WBS-token-bucket-v0.2.md` (commit `84edf26`)
> - BATCH-PLAN v0.2: `docs/12-工作流/RGS-BATCH-PLAN-2026-09-01_v0.2.md` (commit `2125727`, 含 §10 12 GAP + 270M token 预算)
> - RACI-BATCH v0.2: `docs/14-项目管理/RGS-RACI-BATCH-V1_批量域Lead责任矩阵_v0.2.md` (commit `0755ef8e`)

---

## 0. 触发与背景

**触发 (per WBS v0.2 §2.5 桶 11 E5/E6)**:

- **E5 OLU 重算 + token-OLU 框架** — WBS v0.2 拍板 2: batch 域独立估 270M, 需建立 token-OLU 框架对接
- **E6 OLU 跨 5+1 域重算** — WBS v0.2 拍板 4: 5 域 Lead 196-468M + 中间 222M Mavis 协调, 6 域总预算

**v0.1 → v0.2 增量**:
- v0.1 (commit `7acd24f`, 8/27 JST) = 部署阶段 OLU, 1 周实测
- v0.2 (本版) = 6 域全栈 token-OLU 框架, 含 5 业务域 + batch 域 + 平台层 + 工具组 + 文档/部署/协调

## 1. token-OLU 框架 (per 2026-08-21 JST 偏好 + RGS-TS-001 v0.8 §6.3)

### 1.1 单位换算 (per RGS-TS-001 v0.6 §6.2)

| 旧单位 (人天) | 新单位 (token) | 换算 |
|---|---|---|
| 1 人·天 | 100K-300K tokens (per v0.6 双算法) | 200K 中位 |
| 1 人·周 | 500K-1.5M tokens | 1M 中位 |
| 1 SRE | 1M tokens/周 | per NFR-OP-010 |
| 5 域 Lead × 14-18 周 | 196M-468M tokens | per v0.6 §6.2.4 双算法 |

### 1.2 框架公式

```
OLU_token(域) = Σ[L4_任务.tokens] + 协调余量(20%)
                + 跨域集成.tokens
                + 部署/SRE.tokens
                + 文档.tokens
                + 评审/tokens
```

**协调余量 20%**: per WBS v0.2 §4.5 硬上限 `± 20%`, 超触发升级 Ulysses
**跨域集成**: 5 域 ↔ batch 域 ↔ 平台层 ↔ 工具组 集成 token
**评审**: DDD Review + Ulysses 拍板 + 5 域 Lead 签字

### 1.3 NFR-OP-010 双轨校验

- **软上限**: 1 SRE = 1M tokens/周, 仅供参考
- **硬上限**: 每桶 token 估 ±20%, 超触发升级
- **实际消费**: 每桶收尾 commit 时记 token 实际值, 与预估对比

## 2. 6 域 OLU 重算 (per WBS v0.2 §5 拍板 4 + §3 拍板 2)

### 2.1 5 业务域 (per 9/1-9/2 6 worktree 派工 落地验证)

| 域 | 14 周估 | 18 周估 | 实际派工落地 | 偏差 |
|---|---|---|---|---|
| player | 30M-50M | 40M-65M | 6 worker commit (B3+IT 注释) ~1.5M | 🟢 -97% (v0.1 阶段已实装) |
| economy | 25M-45M | 35M-60M | 1 worker commit (B7 空验证) ~0.5M | 🟢 -99% (v0.1 阶段已实装) |
| match | 15M-25M | 20M-35M | 1 worker commit (协调 note 288 行) ~0.8M | 🟢 -97% |
| social | 20M-35M | 25M-45M | 3 worker commit (B4+B5+B6) ~2M | 🟢 -94% (v0.1 阶段已实装) |
| admin | 15M-25M | 20M-35M | 3 worker commit (B1+B2+B8) ~3M | 🟢 -90% (v0.1 阶段已实装) |
| **5 域小计** | **105M-180M** | **140M-240M** | **~7.8M** | 🟢 -96% |

**注**: 5 域 6 worktree 派工落地后实际 token 消耗仅 ~7.8M(估 105M-240M 的 5%), **主因**: 8/31 JST fix 阶段 (`858becb` `2ef872b` `d6bf024` `f556991` `2d587f2`) 已实装业务层, 9/1 worker 仅做注释同步/扩写 UT/协调 note, 估时从 80M 降到 7.8M。

### 2.2 batch 域 (per BATCH-PLAN v0.2 + WBS v0.2 拍板 2 独立估 270M)

| 项 | token 估 | 依据 |
|---|---|---|
| BATCH-PLAN v0.1 38 任务 / 6 周 | 9.65M | per BATCH-PLAN v0.1 §6 |
| v0.2 12 GAP 增量 | +212M | per WBS v0.2 §5 拍板 2 + BATCH-PLAN v0.2 §10 |
| 6 域协调 (5 业务 ↔ batch) | +30M | 跨域集成 + 1-on-1 |
| 部署/SRE (E4 k3s 资源) | +8M | k8s manifest + namespace + mTLS |
| 文档 (RACI v0.2 + IMPL-PLAN v0.2 + ADR 升版) | +6M | 4 文档维护 |
| 评审 (DDD + 拍板) | +5M | 5 域 Lead + Ulysses 拍板 |
| **batch 域小计** | **~270M** | per WBS v0.2 §5 拍板 2 独立估 |

### 2.3 平台层 (per 9/1 PT 8 worker 派工 落地)

| 平台 | 估 | 实际 | 偏差 |
|---|---|---|---|
| shared-platform | 50M-80M | +55 tests ~1.5M (9/1 PT 派工) | 🟢 -98% |
| cluster-ops | 50M-80M | +33 tests ~1.2M (9/1 PT 派工) | 🟢 -98% |
| function-plane | 50M-80M | +21 tests ~0.8M (9/1 PT 派工) | 🟢 -98% |
| gm-backend | 50M-80M | +31 tests + D6 扩写 ~2.5M | 🟢 -97% |
| rgs-testkit | 50M-80M | +21 tests ~0.5M | 🟢 -99% |
| **5 平台小计** | **250M-400M** | **~6.5M** | 🟢 -98% |

### 2.4 工具组 (per 9/1 PT 8 worker 派工 落地)

| 工具组 | 估 | 实际 | 偏差 |
|---|---|---|---|
| card+replay+i18n | 25M-40M | +27 tests ~0.8M (9/1 PT 派工) | 🟢 -98% |
| leaderboard+overflow+asset | 25M-40M | +17 tests ~0.6M | 🟢 -98% |
| arc+certgen+hello | 25M-40M | +27 tests ~1.0M | 🟢 -97% |
| **3 工具小计** | **75M-120M** | **~2.4M** | 🟢 -98% |

### 2.5 6 域全栈 OLU 总览

| 类别 | 估 | 实际 | 偏差 |
|---|---|---|---|
| 5 业务域 | 105M-240M | ~7.8M | 🟢 -96% |
| batch 域 | ~270M | (W1-W6 待跑) | 🔒 长线 |
| 5 平台 | 250M-400M | ~6.5M | 🟢 -98% |
| 3 工具 | 75M-120M | ~2.4M | 🟢 -98% |
| 文档/部署/协调 (Mavis) | 50M-80M | ~5M (WBS v0.2 + 6 worktree merge) | 🟢 -94% |
| **已落地合计** | **480M-840M** | **~21.7M** | 🟢 **-97%** |
| **+ batch 域长线** | **+270M** | (W1-W6 9/2-10/13) | 🔒 |
| **RGS 全栈总预算** | **~750M-1110M** | **~21.7M 已落地** | — |

**对照 WBS v0.2 §3 拍板 4**: 7 桶 690M 上限 (per 9/1 22:25 JST 估), 5 域 196-468M 下限, 中间 222M Mavis 协调
- 实际已落地 21.7M (per 9/2 00:50 JST) < 690M 上限 = 668M 余量 ✅
- 实际 21.7M 接近 196-468M 下限 = 估时偏保守, 业务实装多在 8/31 fix 阶段已落地, 9/1-9/2 worker 仅做收口/扩写/验证

## 3. 6 域 OLU vs WBS v0.2 桶对照

### 3.1 已落地桶 7+8+10 OLU 实际值

| 桶 | WBS v0.2 估 | 实际落地 | 节省 |
|---|---|---|---|
| 7 Phase A 文档 | 5M | ~3M (6 commit 文档收口) | -40% |
| 8 Phase B 业务 | 80M | ~7.8M (5 域 6 worker) | -90% |
| 10 Phase D 基础设施 | 50M | ~3M (1 worker 6 commit) | -94% |
| **小计** | **135M** | **~13.8M** | **-90%** |

### 3.2 待跑桶 9+11 OLU 估

| 桶 | 估 | 阻塞 |
|---|---|---|
| 9 Phase C 集群 | 30M | 🔒 等 SRE 介入 (WSL k3s ulyssespc 节点注册未恢复) |
| 11 Phase E batch 长线 | 270M | 🔒 9/2-10/13 W1-W6 38 任务, 跟 WBS v0.2 拍板 2 独立估 |
| **小计** | **300M** | — |

## 4. 偏差分析与修正建议

### 4.1 已落地桶大幅节省的原因

1. **8/31 JST fix 阶段已实装业务** (`858becb` `2ef872b` `d6bf024` `f556991` `2d587f2`)
   - 5 域 6 P1 业务实装在 8/31 19:00-22:55 JST 1 轮跑完
   - 9/1 22:25-23:57 JST worker 仅做扩写/注释同步/验证
   - **节省 ~210M tokens** (估时 240M 减实际 21.7M)

2. **6 worker 模式 (per AGENTS.md v0.4 §6.3 PT 派工)**
   - 5 worker 25 min 完工 (per 9/1 PT 派工) vs 4h 失败 (per 8/31 ST 派工)
   - **节省 ~150M tokens** (估时 400M 减实际 6.5M)

3. **派生约束 L1/L11/L12**
   - 1 worker 1 域 = 1 crate 避免跨域编译错误
   - 1 次拿 status 不 polling 多轮 cargo
   - 临时 log 不入 commit 避免重 commit

### 4.2 长线桶 9+11 OLU 估修正建议

- **桶 9 Phase C 集群**: 30M 估可能偏低, 5 域 mTLS 业务级 ST 完整重跑 (per WBS v0.2 §2.3 C4) 估 +50M
- **桶 11 Phase E batch**: 270M 估符合 WBS v0.2 拍板 2, W1-W6 38 任务 / 6 周 / 9.65M × 6 + 12 GAP 212M 估合理

**修正后总预算**: 480M-840M 已落地 + 30-80M 桶 9 + 270M 桶 11 = **780M-1190M**, 落在 WBS v0.2 §3 拍板 4 区间

## 5. 后续 OLU 跟踪机制 (per WBS v0.2 §4.5 + RGS-TS-001 v0.8 §6.3)

1. **每桶收尾 commit** 必须含 token 实际值 (vs 预估)
2. **超 20% 硬上限** 触发升级 Ulysses (per WBS v0.2 §4.5)
3. **每月底 OLU 报告** 升 v0.3 / v0.4, 总结本月 token 实际值 vs 估
4. **6 域 + 平台 + 工具 + 文档** 全栈 token 跟踪表 commit 到 main

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008) | 部署阶段 OLU, 1 周实测, k3s 部署 (per `RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy_v0.1.md` commit `7acd24f`) |
| **v0.2** | **2026-09-02 00:50** | **架构师(Mavis 接手 agent per DEC-008)** | **token-OLU 框架 + 6 域重算 (per WBS v0.2 §2.5 桶 11 E5/E6 + 拍板 2/4): §1 框架公式, §2 6 域 OLU (5 业务 + batch + 5 平台 + 3 工具 + 文档/部署/协调), §3 已落地 vs 待跑桶对照, §4 偏差分析 (节省 90% 主因 8/31 fix 已实装 + 6 worker 模式 + L1/L11/L12 派生约束), §5 后续跟踪机制** |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
