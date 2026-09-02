# DDD-REVIEW-TEMPLATE-v0.2 — DDD Review 二审流程模板

> **创建日期**: 2026-09-02 14:07 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: B3 派生约束 (per 9/2 10:18 JST 拍板) — 打破 AI 自指闭环
> **配套**: AGENTS.md v0.6.1 §9.6 W1 D2 + RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1.1 §5.1 D2
> **作用域**: 所有 DDD Review 阶段材料 (UT+IT / ST / 13 域终审 / DB-DRAFT / PT 派工 / 部署恢复 / 跨域 saga / batch v0.1)

---

## 0. v0.1 → v0.2 变化总览

| 变化 | v0.1 (一审) | v0.2 (二审) | 依据 |
|---|---|---|---|
| **一审角色** | Mavis 写 + Mavis 审 (AI 自指) | Mavis 写 + **Mavis 自审 1 次后停手** (不允许自审自改循环) | B3 派生约束 |
| **二审角色** | (无) | **Ulysses 必审, 不可跳过** | B3 派生约束 |
| **签字栏** | 1 栏 (Mavis 写审) | 2 栏 (Mavis 自审 + Ulysses 二审) | B3 派生约束 |
| **状态机** | `⏳ 待审 → ✅ 通过` | `⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审 → ✅ 二审通过 / ❌ 二审打回` | B3 派生约束 |
| **打回流程** | (无, Mavis 自己改) | Ulysses 二审打回 → 回到 Mavis 改稿 → 重新走 Mavis 自审 → 再次进 Ulysses 二审 | B3 派生约束 |

---

## 1. 二审流程 (per B3 派生约束)

```
┌────────────────────────────────────────────────────────────────────┐
│ 1. 起草 (Mavis)                                                    │
│    - 用本模板 v0.2 起 DDD Review 文档                              │
│    - 路径: docs/14-项目管理/ddd-review/RGS-DDD-<date>-<topic>_v0.X.md│
└──────────────────────────┬─────────────────────────────────────────┘
                           ▼
┌────────────────────────────────────────────────────────────────────┐
│ 2. Mavis 自审 (1 次停手)                                          │
│    - 自查: 代签三件套 + DoD 段 + Evidence 段 + 派生约束守护段       │
│    - 自查: 缺标比错标 (per 8/26 JST), 已知缺口段必须填             │
│    - 自查: 禁回溯叙事 (per 8/26 JST), 无 "per X 历史形态" 类       │
│    - 自查: 凭据硬 ban (per 8/27 11:06 JST), 无 env value 痕迹      │
│    - ⚠️ **停手**: 自审 1 次后不再回头改稿, 直接进二审             │
└──────────────────────────┬─────────────────────────────────────────┘
                           ▼
┌────────────────────────────────────────────────────────────────────┐
│ 3. Ulysses 二审 (必到, 不可跳过)                                  │
│    - 看文档 + 看代签 + 看 DoD + 看 Evidence                        │
│    - 必查: 自指字段 (per L13) deferred 实时查询                    │
│    - 必查: 派生约束守护段 (L1/L1.1/L1.2 + L11/L12/L13/L14)         │
│    - 必查: hotfix 数 / commit ahead / md 行数 (业务 vs 治理)        │
│    - 二审决定: ✅ 通过 / ❌ 打回 (Mavis 改稿) / 🟡 有条件通过       │
└──────────────────────────┬─────────────────────────────────────────┘
                           ▼
                ┌──────────┴──────────┐
                ▼                     ▼
        ┌──────────────┐    ┌──────────────────┐
        │ 4a. ✅ 通过   │    │ 4b. ❌ 打回       │
        │ commit 落地 │    │ 回到 Mavis 改稿   │
        │ 状态机结束  │    │ 重走 2 → 3 循环   │
        └──────────────┘    └──────────────────┘
```

**关键约束**:
- **Mavis 自审 1 次停手** (步骤 2): 避免 Mavis 改稿 → 自审 → 改稿 → 自审 的内循环 (B3 派生约束核心)
- **Ulysses 二审必到** (步骤 3): 不可跳过, 不可"等下次" (B3 派生约束)
- **打回循环上限**: 最多 2 次打回, 第 3 次强制要么通过要么冻结 (per v0.1.1 §6 风险)

---

## 2. 文档结构模板 (复制本段到 DDD Review 实例文档)

### 2.1 顶部元信息 (Frontmatter)

```markdown
# RGS-DDD-<date>-<topic> — <阶段名> DDD Review

| 字段 | 值 |
|---|---|
| 文档 ID | RGS-DDD-<date>-<topic> |
| 版本 | v0.X |
| 创建日期 | <YYYY-MM-DD> JST |
| 创建者 | 架构师(Mavis 接手 agent per DEC-008) |
| 类型 | DDD Review 二审材料 (per DDD-REVIEW-TEMPLATE-v0.2) |
| 关联 | <commit SHA> / <RGS-OLU-REPORT> / <RGS-RACI-*> / <Phase SRE HANDOFF> |
| 基线 commit | `<SHA>` (main) |
| 范围 | <5 域 / 13 域 / 跨域 saga / batch 域 / 等> |
| 阶段 | <UT / IT / ST / 部署 / 评审 / 等> |
| 状态 | ⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审 → ✅/❌/🟡 |
```

### 2.2 正文结构 (8 段)

```markdown
## 1. 执行摘要 (Executive Summary)
- 时间窗 + 操作者 + 范围 + 阶段 + 风格
- 最终产出表: <域> | <UT commit> | <IT commit> | +行 | test 数 | 编译状态

## 2. 基线与分支拓扑
- 分支图 (worktree 路径)
- 基线 commit SHA
- 分支策略

## 3. <主题 1>: <具体内容>
- 表格 + 引用 commit / file:line

## 4. <主题 2>: <具体内容>
- 同上

## 5. 派生约束守护 (per AGENTS.md §8 冻结 + 拍板)
- L1 / L1.1 / L1.2 三件套状态
- L11 / L12 / L13 / L14 派生约束状态
- 拍板依据 (引 RGS-CRITIQUE-IMPROVEMENT-* + RGS-WEEKLY-*)

## 6. merge 落地验证 (或阶段落地)
- merge commit 列表 (--no-ff 保留拓扑)
- 最终 main 状态 (HEAD / cargo check / 编译时间)

## 7. 后续工作 (per WBS v0.2 / BATCH-PLAN)
- Phase A 剩余 (checkbox)
- Phase B 业务实装
- Phase C SRE 介入
- Phase D 基础设施

## 8. 已知缺口 (per 8/26 JST 缺标比错标)
- 真未做 vs 已做不彻底区分
- 后续跟踪 commit / issue
```

### 2.3 签字栏 (v0.2 新增, 关键变化)

```markdown
---

## 9. 签字栏 (per DDD-REVIEW-TEMPLATE-v0.2 二审流程)

### 9.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ / ❌ | author / 审批 / 修订人 |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ / ❌ | L1 / L1.1 / L1.2 |
| Evidence 段 (commit SHA / file:line) | ✅ / ❌ | git log + Read 实证 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ / ❌ | 8/27 11:06 JST 凭据硬 ban |
| 缺标比错标 (per 8/26 JST) | ✅ / ❌ | §8 已知缺口段 |
| 禁回溯叙事 (per 8/26 JST) | ✅ / ❌ | 无 "per X 历史形态" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ / ❌ | 无 env value 痕迹 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: <YYYY-MM-DD> JST

### 9.2 Ulysses 二审 (必到, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ✅ / ❌ | git log + grep 实证 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ / ❌ | cargo check / test 状态 |
| 业务 vs 治理指标 (per v0.1.1 §9.4 里程碑重定义) | ✅ / ❌ | hotfix 数 / commit ahead / md 行数 |
| commit ahead 合理性 (per 当前 sprint 范围) | ✅ / ❌ | 应在 ±20 commit 范围 |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ✅ / ❌ | 拍板项已执行 vs 仅承诺 |
| 跟 RGS-WEEKLY 一致性 (若存在) | ✅ / ❌ | 周报双指标对齐 |

**Ulysses 二审决定**:

- [ ] ✅ 通过 — 落地, 状态机结束
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 9.1 → 9.2 循环 (打回次数: <1/2/3>)

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: <YYYY-MM-DD> JST

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | <YYYY-MM-DD> | 架构师(Mavis 接手 agent per DEC-008) | 初始 DDD Review 一审材料 |
| v0.2 | <YYYY-MM-DD> | 架构师(Mavis 接手 agent per DEC-008) | 二审流程升级 (per B3 派生约束 9/2 10:18 JST 拍板): 加 §9 签字栏 (Mavis 自审 1 次停手 + Ulysses 二审必到) |
```

---

## 3. 二审打回循环上限 (per v0.1.1 §6 风险)

| 打回次数 | 处理 |
|---|---|
| 第 1 次打回 | 回到 Mavis 改稿 → 重走 9.1 自审 → 9.2 二审 |
| 第 2 次打回 | 同样流程, Mavis 必须有显著改动 (非微调) |
| 第 3 次强制 | 二审选 ✅ 通过 (降级接受) 或 🟡 冻结 (本季度不再评审) |

**理由**: 避免打回无限循环, 保持 DDD Review 节奏 (per v0.1.1 §6 + WBS v0.2 sprint 约束)

---

## 4. 已知缺口 (per 8/26 JST 缺标比错标)

- **打回循环第 3 次强制**: 仍可能误判 (二审疲劳 / Mavis 改不动 / 业务真卡住), 兜底靠 9.2 🟡 冻结选项
- **Ulysses 二审时间窗口**: per 9/2 v0.1.1 §6 风险, Ulysses 时间不定可能拖慢 DDD Review, 需配合 14:58 JST ask_user 拍板规则
- **跨域 saga / batch 域 DDD Review**: 本模板适用, 但跨域场景额外需要"主会话打头阵" (per AGENTS.md §2.3 L4) 验证一审质量
- **Mavis 自审 vs Ulysses 二审的标准对齐**: 当前为"形式合规 + 派生约束 + 业务指标"三层, 未来可能加"业务深度"评估 (待 12/2 季度评审)

---

## 5. 配套文档

| 文档 | 路径 | 关系 |
|---|---|---|
| RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1.1 | `docs/14-项目治理/` | B3 派生约束拍板依据 |
| AGENTS.md v0.6.1 §8 / §9.6 | `AGENTS.md` | L1-L14 冻结 + W1 D2 任务 |
| L-CANDIDATES.md v0.1 | `docs/14-项目治理/` | 候选清单 (本模板不属 L 段) |
| RGS-WEEKLY-* | `docs/14-项目治理/` | 周报双指标 (D4 派生约束) |
| RGS-OLU-REPORT-* | `docs/14-项目治理/` | OLU 评估报告 (二审参考) |
| RGS-RACI-* | `docs/14-项目管理/` | RACI v1.2 (5→6 域 batch 扩展) |

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 16:30 | 架构师(Mavis 接手 agent per DEC-008) | 初始模板 (一审 + 单签字栏 + 状态机 `⏳ → ✅`), per `bd0884f` |
| v0.2 | 2026-09-02 14:07 | 架构师(Mavis 接手 agent per DEC-008) | 二审流程升级 (per B3 派生约束 9/2 10:18 JST 拍板): 加 §0 变化总览 + §1 二审流程图 (Mavis 自审 1 次停手 + Ulysses 二审必到) + §2 文档结构模板 (含 §9 签字栏 2 段) + §3 打回循环上限 + §4 已知缺口 + §5 配套文档 + §6 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
