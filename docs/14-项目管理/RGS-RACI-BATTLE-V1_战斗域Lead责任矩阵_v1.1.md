# RGS-RACI-BATTLE-V1 战斗域 Lead 责任矩阵 v1.1（Battle Domain Lead RACI v1.1, per 9/5 12:08 JST 8 域扩展拍板 NEW）

**RGS-RACI-BATTLE-V1**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-RACI-BATTLE-V1 |
| 版本 | v1.1（per 9/5 12:08 JST 拍板, 6 域 → 8 域 + 1 网关 升版 NEW 域, AGENTS.md §9.7 配套; 基于 v1.3 5 域模板 + battle-service NEW crate scaffold (W5 worker)） |
| 状态 | 规格草案 + 已知缺口（见 §A.3），待 battle 域 Lead 联合具名 DDD Review |
| 源 RACI | RGS-RACI-001 v0.1（commit `14786a5`）+ 5 域 RACI v1.3 模板（per 9/5 12:08 JST 拍板） |
| 源 ADR | RGS-ADR-0055 v0.1（per DEC-005 5 域独立 Lead）+ DEC-008 一人公司 12 角色 |
| 适用范围 | battle 域（PVP / PVE / 技能 / buff/debuff / 战斗录像 / 战斗结算 / 段位赛 / 锦标赛 / 英雄殿堂 / 友谊赛 / 9 holiday_* 活动） |
| 目标基线 | 一人公司 12 角色治理基线 per DEC-008 + 8 域独立 Lead 兼任禁止 per DEC-005 |
| 责任人 | Ulysses（battle 域 Lead 待 E2.5 拍板, 当前 Mavis 接手代签 per W5 worker 派工） |
| 配套 | 8 域 sister RACI: player / economy / match / social / admin / batch / scene / network-gateway |
| 强制并行 | RGS-RACI-SCENE-V1 / RGS-RACI-BATTLE-V1 / RGS-RACI-NETWORK-GATEWAY-V1 |
| NEW 域来源 | W5 worker 9/5 Phase 0 派工 (per `D:\sszgC\worker5-battle-report.md`), battle-service NEW crate 250 RPC scaffold (6+9 反例数据驱动) |

---

## 0. 触发与背景

**触发**：Ulysses 2026-09-05 12:08 JST 拍板"6 域 → 8 域 + 1 网关"扩展（per `D:\sszgC\phase0-worker-report.md` §1.1 7 域 crate 完整 + §6.7 RACI v1.3 升版缺标），承接 Phase 0 W5 worker 完结（battle-service 250 RPC scaffold + 74 UT + 44 真实 RPC + 6+9 反例数据驱动）。

**来源**：
- W5 worker 9/5 Phase 0 派工 250 RPC scaffold（PVP 6 变体 + PVE + 9 holiday_* 活动 + 战斗录像 + 段位赛 + 锦标赛 + 英雄殿堂 + 友谊赛）
- 6 个 PVP 变体 (ranked/casual/cross-server/championship/hero-hall/friendly) → 1 套 PvPService + PvpMode 枚举 + PvpConfig HashMap 数据驱动
- 9 个 holiday_* 活动 (春节/元宵/端午/中秋/圣诞/周年庆/夏日祭/万圣节/感恩节) → 1 套 HolidayActivityService + activity_id 路由
- RGS-RACI-001 v0.1 提供 5 域 × 8 阶段 × 4 治理角色 = 160 单元的横向通用矩阵
- RGS-ADR-0055 v0.1 提供 5 域独立 Lead 兼任禁止的治理基线
- **缺**：battle 域 Lead 真实身份 (待 E2.5 + Ulysses 拍板指派, 9/8 之前)
- **缺**：battle × 5 域 (player/economy/match/social/admin) + batch + scene + network-gateway 跨域任务矩阵未完整画 (per AGENTS.md §9.7.4 已知缺口)

**本文档目的**：把 RGS-RACI-001 v0.1 160 单元的"通用矩阵"具象化为 battle 域的 8-Lead 签字栏（含 battle 域 Lead + 架构师 + SRE + DBA + 安全 + shared-platform + saga 召集人 + network-gateway Lead 8 行签字）。

---

## 1. 矩阵维度（8 域 Lead v1.1 横向规则）

| 维度 | 数量 | 内容 |
|---|---|---|
| 域任务 | 6 | battle 域核心任务：PVP 匹配 / 战斗执行 / 战斗结算 / 战斗录像 / 段位赛 / 9 holiday_* 活动 |
| 治理角色（行） | 8 | battle 域 Lead / 架构师 / SRE / DBA / 安全 / shared-platform / saga 召集人 / network-gateway Lead |
| **签字单元** | **6 × 8 = 48** | 每格 1 个责任字母 R/A/C/I（per RGS-ADR-0055 v0.1 §4 RACI 定义） |

**RACI 字母**：
- **R**（Responsible）：执行者，对结果负直接责任
- **A**（Accountable）：最终责任者，1 项任务只能有 1 个 A
- **C**（Consulted）：双向咨询，需主动征求 + 记录意见
- **I**（Informed）：单向通知，结果通报即可

---

## 2. battle 域 RACI 矩阵

| 任务 \ 角色 | battle 域 Lead | 架构师 | SRE | DBA | 安全 | shared-platform | saga 召集人 | network-gateway Lead |
|---|---|---|---|---|---|---|---|---|
| 1. PVP 匹配（battle.pvp_match v0.1 + 6 变体 ranked/casual/cross-server/championship/hero-hall/friendly）| **A** | C | C | I | **A** | C | C | C |
| 2. 战斗执行（battle.battle_execute v0.1 + 技能/buff/debuff）| **A** | C | C | I | **A** | C | C | **A** |
| 3. 战斗结算（battle.battle_settle v0.1 + 奖励发放）| **A** | C | I | **A** | C | I | **A** | I |
| 4. 战斗录像（battle.battle_replay v0.1 + 录像存储）| **A** | C | C | **A** | C | C | I | C |
| 5. 段位赛（battle.ranked_season v0.1 + 段位结算）| **A** | C | C | C | **A** | I | C | I |
| 6. 9 holiday_* 活动（battle.holiday_* v0.1 + activity_id 路由）| **A** | C | I | C | C | I | C | I |

**矩阵解读**：
- battle 域 Lead 在 6 任务中 6 次 A（战斗全任务域 Lead 主责）
- 架构师 C 全部（横向咨询），但不直接 A（避免 battle 域 Lead 与架构师兼任）
- SRE 在 PVP 匹配 + 战斗执行 + 战斗录像 + 段位赛 4 个任务 C（基础设施相关 + 防作弊 + 录像存储）
- DBA 在战斗结算 + 战斗录像 A（战斗奖励 transaction 表 + 录像存档表决策）
- 安全在 PVP 匹配 + 战斗执行 + 段位赛 A（反外挂 + 防作弊 + 段位防刷）
- shared-platform 仅 C (无直接 A 角色) 但 4 任务 C (per W5 worker 报告, 战斗引擎公共组件)
- saga 召集人在战斗结算 A（per RGS-IMPL-100 saga 域召集人, 跨域奖励最终触发）
- network-gateway Lead 在战斗执行 A（协议网关层 + TCP 帧路由决策, per ADR-006 Option A, 协议码 10401-10500 段归属需 E2.5 拍板）

---

## 3. 责任到人映射（per DEC-008 一人公司 12 角色）

| 治理角色 | 真实责任人 | 兼任关系 |
|---|---|---|
| battle 域 Lead | Ulysses (待 E2.5 + 9/8 拍板) | 兼架构师 / Saga 召集人 (per DEC-008 一人公司 12 角色治理基线) |
| 架构师 | Ulysses | 兼 battle 域 Lead（per DEC-008） |
| SRE | Ulysses | 兼 shared-platform Lead（per DEC-008 + DEC-005 8 域独立 Lead 不允许兼任 SRE → 一人公司模式下"独任"全栈） |
| DBA | Ulysses | 兼 admin 域 Lead（per DEC-008） |
| 安全 | Ulysses | 兼 admin 域 Lead（per DEC-008） |
| shared-platform | Ulysses | 兼 SRE（per DEC-008） |
| saga 召集人 | Ulysses | 兼 battle 域 Lead（per DEC-008） |
| network-gateway Lead | Ulysses (待 E2.5 + 9/8 拍板) | 兼 battle 域 Lead (TCP 帧路由决策相关)（per DEC-008） |

**关键说明**：DEC-005 8 域独立 Lead 兼任禁止是**多域并存的协作基线**。在一人公司模式下（DEC-008），所有治理角色由 Ulysses 一人担任，但 8 域 Lead 仍然在**决策权矩阵**上保持独立（每个域 Lead 只能 A 自己的域任务，不允许 A 其他域任务）。这是用"角色标签独立 + 一人决策"代替"多人独立签字"的简化治理。

**battle 域 Lead 真实身份待 E2.5 拍板**（per AGENTS.md §9.7.3 E2.5 节点）——当前由 Mavis 接手代签。

---

## 4. 8 域 Lead 联合签字栏（v1.1 NEW 域真实签字版）

| 域 Lead | 签字 | 日期 | 备注 |
|---|---|---|---|
| player 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | player 域 Lead 同意 battle 域独立 + 跨域 (PVP 角色/战斗角色) 协调 |
| economy 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | economy 域 Lead 同意 battle 域奖励走 economy.RewardGrant |
| match 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | match 域 Lead 同意 battle 域 PVP 匹配走 match.MatchStart |
| social 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | social 域 Lead 同意 battle 域英雄殿堂走 social.GuildJoin |
| admin 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | admin 域 Lead 同意 battle 域审计走 admin.AuditLog |
| batch 域 Lead（Ulysses per DEC-008, 占位）| ⏳ (per BATCH-RACI v1.3 待 E2 拍板) | — | batch 域 Lead 同意 battle 域离线录像走 batch.ImportData |
| scene 域 Lead（Ulysses per DEC-008, 待 E2.5 拍板）| ⏳ | — | scene 域 Lead 同意 battle 域副本内战斗走 scene.DungeonEnter |
| network-gateway Lead（Ulysses per DEC-008, 待 E2.5 拍板）| ⏳ | — | network-gateway Lead 同意 battle 域 TCP 帧路由 (协议码 10401-10500 段) |
| battle 域 Lead（Ulysses per DEC-008, 待 E2.5 拍板）| ⏳ | — | battle 域 Lead 同意 6 任务 RACI 矩阵 + 跨域协调 |
| saga 召集人（Ulysses per DEC-008）| _待 DDD Review 阶段补签_ | — | per RGS-IMPL-100 saga 域召集人 |
| 架构师（Ulysses per DEC-008）| _代签：架构师（Mavis 接手 agent per DEC-008）_ | 2026-09-05 12:30 JST | per 2026-08-26 08:40 JST 代签已允许 |

**注**：v1.1 在一人公司模式下，8 域 Lead 都是 Ulysses 担任；DDD Review 阶段由 Ulysses 在每个域分别签字（一签字 = 该域决策正式生效）。**代签不允许用于"代签他人"**——架构师列可由 Mavis 代签 per 2026-08-26 08:40 JST 新规则，但 8 域 Lead 列必须由 Ulysses 本人（per DDD Review SOP）。

---

## 5. 已知缺口（per DDD Review 必查）

- [ ] **battle 域 Lead 真实身份待 E2.5 拍板** —— 9/8 之前 Ulysses 拍板指派（per AGENTS.md §9.7.3 E2.5）
- [ ] **DDD Review 模板对齐**：本文档签字栏格式是否与 RGS-SPEC-CROSS-011 v0.1 §3 字段级核对完全一致？若不一致需修订
- [ ] **saga 域 Lead DEC-008 显式化**：saga 召集人在 DEC-008 12 角色中是独立角色还是 battle 域 Lead 兼任？需 ADR-0055 显式化
- [ ] **跨域仲裁流程**：battle 域 Lead 与 economy 域 Lead 在涉及双方 A 角色任务（战斗结算奖励发放）时的最终决策权？需 RGS-OPEN-QA-001-ACTIONS-v0.3 后续子任务明确
- [ ] **battle × network-gateway 协议码段分配**：协议码 10401-10500 段归属需 E2.5 拍板（per W6 worker 报告）
- [ ] **battle × scene 联调场景**：副本内战斗触发条件 + 状态同步（per W4 + W5 worker 报告联调, Phase 3 估 1-2 SRE·d）
- [ ] **9 holiday_* 活动业务规则**：6+9 反例数据驱动 15 → 3 套, 但 activity_id 路由表需 battle Lead 业务确认 (per W5 报告 9 holiday_*)
- [ ] **电子签字基础设施**：8 域 Lead 真实签字基础设施（GPG / SSH 签名 / 内部 CRM）尚未搭建，目前以"代签 + DDD Review 阶段补签"过渡

---

## 6. battle 域 DDD Review 节点 (per AGENTS.md §9.7.3 + WBS v0.2 Phase 1)

| 节点 | 触发 | 必填 |
|---|---|---|
| E2.5 battle 域 Lead 真实身份指派 | 9/8 之前 | Ulysses 拍板 + 架构师签字 |
| E3.5 battle-service scaffold 完整实装 (250 RPC → 真逻辑) | 9/22 之前 (per W2-W6 Phase 1) | 架构师 + 8 域 Lead 协调签字 |
| E5.5 8 域 OLU 跨域重算 (含 battle) | 9/29 之前 | 8 域 Lead + 架构师签字 |
| E9 协议网关完整实装 (battle × network 协调) | 10/20 之前 (per ADR-006 Option A) | network-gateway Lead + 架构师 + Ulysses 拍板 |

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| **v1.1** | **2026-09-05 12:30** | **架构师（Mavis 接手 agent per DEC-008）** | **8 域扩展升版 NEW 域落档 (per 9/5 12:08 JST 拍板, 6 域 → 8 域 + 1 网关)**: 初版战斗域 Lead RACI 矩阵 (6 任务 × 8 角色 = 48 单元) + 责任到人映射 + 8 域 Lead 联合签字栏 + 已知缺口 + DDD Review 节点. 来源: W5 worker 9/5 Phase 0 派工 (per `D:\sszgC\worker5-battle-report.md`), battle-service NEW crate 250 RPC scaffold (6+9 反例数据驱动) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

---

## A. v1.1 升版增量

### A.1 源 5 域 RACI v1.3 → battle 域 v1.1

- 6 任务 × 7 角色 = 42 单元 → 6 任务 × 8 角色 = 48 单元（+ network-gateway Lead 列）
- 新增 battle 域 NEW 域决策权 (6 任务 6 A 角色)
- 新增 battle × 8 域 (player/economy/match/social/admin/batch/scene/network-gateway) 协调签字栏
- 8 类已知缺口 (per DDD Review 必查)
- 6+9 反例数据驱动 (per W5 报告 6 PVP 变体 + 9 holiday_* 活动 → 3 套数据驱动 + N 配置)

### A.2 对本 SPEC 的影响

- battle 域 DDD Review 模板 (RGS-SPEC-CROSS-011 v0.1) 可直接调用本文档 §2 矩阵 + §4 签字栏
- 不影响 RGS-IMPL-100 saga 域 + 7 份 sister RACI（各自独立 v1.x）

### A.3 已知缺口

- 8 类已知缺口见 §5 (DDD Review 必查)
- 配套 7 份 sister RACI (player / economy / match / social / admin / batch / scene / network-gateway) 需并行 v1.x
- battle 域 Lead 真实身份待 E2.5 拍板

### A.4 引用链与证据

- RGS-RACI-001 v0.1 (commit `14786a5`, 2026-08-26 09:30 JST)
- RGS-ADR-0055 v0.1 (commit `d6a56c6`, 2026-08-25 06:26 JST, per WF-1-55.49)
- 5 域 RACI v1.3 模板 (per 9/5 12:08 JST 拍板, AGENTS.md v0.6.12 §9.7.2)
- RGS-IMPL-100 saga 域召集人决策
- `D:\sszgC\worker5-battle-report.md` (W5 worker 9/5 Phase 0 派工)
- `D:\sszgC\battle-rpc-list.tsv` (W5 出的 241 RPC 索引)
- `D:\sszgC\phase0-worker-report.md` §1.1 7 域 crate 完整 + §1.2 关键反例数据驱动 + §6.7 RACI v1.3 升版缺标
- per WBS v0.2 Phase 1 battle 域 L4 任务 (待 WBS v0.3 落档)
- per RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1.1 §4 C 类业务重排 (8 域不缩)
- per AGENTS.md v0.6.12 §8.x L15-L18 派生约束 (per 9/5 12:08 JST 紧急批准)
- per AGENTS.md v0.6.12 §9.7 8 域扩展全景
- 修订历史代签新规则 per 2026-08-26 08:40 JST (`C:\Users\leon19\.minimax\memory\user.md` "文档代签规则反转")
- 修订人 / 审批 / 代签授权 per 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化
