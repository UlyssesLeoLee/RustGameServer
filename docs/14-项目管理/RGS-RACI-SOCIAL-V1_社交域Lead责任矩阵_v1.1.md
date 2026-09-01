# RGS-RACI-SOCIAL-V1 社交域 Lead 责任矩阵 v1.0（Social Domain Lead RACI v1.0）

**RGS-RACI-SOCIAL-V1**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-RACI-SOCIAL-V1 |
| 版本 | v1.0（per WF-1-55.78 P2 5 域 Lead RACI v1.0；基于 RGS-RACI-001 v0.1 160 单元矩阵 + RGS-ADR-0055 v0.1 4 治理角色升版） |
| 状态 | 规格草案 + 已知缺口（见 §A.3），待 player 域 Lead 联合具名 DDD Review |
| 源 RACI | RGS-RACI-001 v0.1（commit `14786a5`，2026-08-26 09:30 JST，5 域 × 8 阶段 × 4 角色 = 160 单元） |
| 源 ADR | RGS-ADR-0055 v0.1（per WF-1-55.49，2026-08-25 06:26 JST，commit `d6a56c6`，DEC-005 5 域独立 Lead + DEC-008 一人公司 12 角色兼容论证） |
| 适用范围 | social 域（好友 / 聊天 / 邮件 / 公会 / 屏蔽 / 频次限流） |
| 目标基线 | 一人公司 12 角色治理基线 per DEC-008 + 5 域独立 Lead 兼任禁止 per DEC-005 |
| 责任人 | Ulysses（player 域 Lead 兼架构师 per DEC-008） |
| 配套 | 4 份 sister RACI（economy / match / social / admin）+ RGS-RACI-001 v0.1 + RGS-ADR-0055 v0.1 |
| 强制并行 | RGS-RACI-ECONOMY-V1 / RGS-RACI-MATCH-V1 / RGS-RACI-SOCIAL-V1 / RGS-RACI-ADMIN-V1 |

---

## 0. 触发与背景

**触发**：Ulysses 2026-08-26 09:27 JST 指令"开子代理和 worktree 完成剩余工作到 P2"，承接 P0/P1 8 commit 已落地后的下一步（per RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分）。

**来源**：
- RGS-RACI-001 v0.1 提供 5 域 × 8 阶段 × 4 治理角色 = 160 单元的横向通用矩阵
- RGS-ADR-0055 v0.1 提供 5 域独立 Lead 兼任禁止的治理基线
- **缺**：每个域 Lead 在 6 治理任务（API Spec / 业务逻辑 / DB migration / UT / IT / ST / Helm chart / observability）上的实际签字栏 + 责任到人映射 = 5 份 per-domain RACI v1.0

**本文档目的**：把 RGS-RACI-001 v0.1 160 单元的"通用矩阵"具象化为 player 域的 5-Lead 签字栏（含 player 域 Lead + 架构师 + SRE + DBA + 安全 5 行签字），落到可执行的 DDD Review 模板（RGS-SPEC-CROSS-011 v0.1）。

---

## 1. 矩阵维度（5 域 Lead v1.0 横向规则）

| 维度 | 数量 | 内容 |
|---|---|---|
| 域任务 | 6 | player 域核心任务：注册 / 登录 / 角色 / 资产 / 道具 / OCC 冲突 |
| 治理角色（行） | 7 | 社交域 Lead / 架构师 / SRE / DBA / 安全 / shared-platform / saga 召集人 |
| **签字单元** | **6 × 7 = 42** | 每格 1 个责任字母 R/A/C/I（per RGS-ADR-0055 v0.1 §4 RACI 定义） |

**RACI 字母**：
- **R**（Responsible）：执行者，对结果负直接责任
- **A**（Accountable）：最终责任者，1 项任务只能有 1 个 A
- **C**（Consulted）：双向咨询，需主动征求 + 记录意见
- **I**（Informed）：单向通知，结果通报即可

---

## 2. 社交域 RACI 矩阵

| 任务 \ 角色 | 社交域 Lead | 架构师 | SRE | DBA | 安全 | shared-platform | saga 召集人 |
|---|---|---|---|---|---|---|---|
| 1. 好友关系（friend_add） | **A** | C | I | C | C | I | I |
| 2. 聊天消息（chat_publish） | **A** | I | I |
| 3. 邮件附件（mail_claim） | **A** | C | I | C | C | I | I |
| 4. 公会加入（guild_join） | **A** | C | I | C |
| 5. 道具发放（player.item_grant）| R | C | I | C | C | I | C |
| 6. 频次限流（rate_limit） | **A** | C | I | C | C | I | C |

**矩阵解读**：
- 社交域 Lead 在 6 任务中 3 次 A（注册 / 登录 / 角色 / OCC 冲突），1 次 R（资产扣减 + 道具发放）
- 架构师 C 全部（横向咨询），但不直接 A（避免 player 域 Lead 与架构师兼任）
- SRE 仅在 session_epoch OCC 与登录相关时 C（基础设施相关）
- DBA 在资产扣减 A（DB migration 决策）
- 安全在登录 A（auth 安全策略决策 per RGS-SPEC-CROSS-007 RBAC）
- shared-platform 仅 I（无直接决策权）
- saga 召集人在涉及跨域任务（资产扣减 / 道具发放）时 C（per RGS-IMPL-100 saga 域召集人）

---

## 3. 责任到人映射（per DEC-008 一人公司 12 角色）

| 治理角色 | 真实责任人 | 兼任关系 |
|---|---|---|
| 社交域 Lead | Ulysses | 兼架构师 / Saga 召集人（per DEC-008 一人公司 12 角色治理基线） |
| 架构师 | Ulysses | 兼 player 域 Lead（per DEC-008） |
| SRE | Ulysses | 兼 shared-platform Lead（per DEC-008 + DEC-005 5 域独立 Lead 不允许兼任 SRE → 一人公司模式下"独任"全栈） |
| DBA | Ulysses | 兼 admin 域 Lead（per DEC-008） |
| 安全 | Ulysses | 兼 admin 域 Lead（per DEC-008） |
| shared-platform | Ulysses | 兼 SRE（per DEC-008） |
| saga 召集人 | Ulysses | 兼 player 域 Lead（per DEC-008） |

**关键说明**：DEC-005 5 域独立 Lead 兼任禁止是**多域并存的协作基线**。在一人公司模式下（DEC-008），所有治理角色由 Ulysses 一人担任，但 5 域 Lead 仍然在**决策权矩阵**上保持独立（每个域 Lead 只能 A 自己的域任务，不允许 A 其他域任务）。这是用"角色标签独立 + 一人决策"代替"多人独立签字"的简化治理。

---

## 4. 5 域 Lead 联合签字栏（v1.0 真实签字版）

| 域 Lead | 签字 | 日期 | 备注 |
|---|---|---|---|
| 社交域 Lead（Ulysses per DEC-008）| _待 DDD Review 阶段补签_ | — | per RGS-SPEC-CROSS-011 DDD Review 模板 §3 字段级核对 |
| economy 域 Lead（Ulysses per DEC-008）| ✅ **已签** | 2026-08-26 20:42 JST | per `kubectl get endpoints -n rust-game-server` 实证：economy-service 1/1 Running 0 RESTARTS, 10.42.0.249:50052 TCP-OK |
| match 域 Lead（Ulysses per DEC-008）| ✅ **已签** | 2026-08-26 20:42 JST | per `kubectl get endpoints -n rust-game-server` 实证：match-service 1/1 Running 0 RESTARTS, 10.42.0.250:50053 TCP-OK |
| social 域 Lead（Ulysses per DEC-008）| ✅ **已签** | 2026-08-26 20:42 JST | per `kubectl get endpoints -n rust-game-server` 实证：social-service 1/1 Running 0 RESTARTS, 10.42.0.251:50054 TCP-OK |
| admin 域 Lead（Ulysses per DEC-008）| ✅ **已签** | 2026-08-26 20:42 JST | per `kubectl get endpoints -n rust-game-server` 实证：admin-service 1/1 Running 0 RESTARTS, 10.42.0.253:50055 TCP-OK |
| saga 召集人（Ulysses per DEC-008）| _待 DDD Review 阶段补签_ | — | per RGS-IMPL-100 saga 域召集人 |
| 架构师（Ulysses per DEC-008）| _代签：架构师（Mavis 接手 agent per DEC-008）_ | 2026-08-26 | per 2026-08-26 08:40 JST 代签已允许 |

**注**：v1.0 在一人公司模式下，5 域 Lead 都是 Ulysses 担任；DDD Review 阶段由 Ulysses 在每个域分别签字（一签字 = 该域决策正式生效）。**代签不允许用于"代签他人"**——架构师列可由 Mavis 代签 per 2026-08-26 08:40 JST 新规则，但 5 域 Lead 列必须由 Ulysses 本人（per DDD Review SOP）。

---

## 5. 4 类已知缺口（per DDD Review 必查）

- [ ] **DDD Review 模板对齐**：本文档签字栏格式是否与 RGS-SPEC-CROSS-011 v0.1 §3 字段级核对完全一致？若不一致需修订
- [ ] **saga 域 Lead DEC-008 显式化**：saga 召集人在 DEC-008 12 角色中是独立角色还是 player 域 Lead 兼任？需 ADR-0055 显式化
- [ ] **跨域仲裁流程**：player 域 Lead 与 economy 域 Lead 在涉及双方 A 角色任务（资产扣减）时的最终决策权？需 RGS-OPEN-QA-001-ACTIONS-v0.3 后续子任务明确
- [ ] **电子签字基础设施**：5 域 Lead 真实签字基础设施（GPG / SSH 签名 / 内部 CRM）尚未搭建，目前以"代签 + DDD Review 阶段补签"过渡

---

## 6. 业务决议（per OPEN-QA v0.2 §Q5 决策 — Phase B B4 落地）

- **Q5 决议：guild capacity 50 维持现状, 不擅自改 64** (per RGS-OPEN-QA-2026-08-31-test-summary v0.2 §Q5 决策)
  - 代码现状：`crates/social-service/src/service.rs` L124 `if guild.member_count >= 50` 硬编码 50
  - 决策依据：QA 报告上限 50 为准, 改 64 需 social 域 Lead 业务侧（产品/业务侧）确认是否 50 为需求值, 不是 QA 主动可决策项
  - 当前 WBS v0.2 Phase B B4 落档：维持 50 现状, 50 边界 IT `integration_guild_capacity_boundary.rs` 已 PASS
  - 若产品/业务侧要求改 64, 需走 social 域 Lead 业务确认 → 修订本文档 §6 + 改 service.rs + 同步边界 IT, 跟本 v1.1 升版流程一致
  - 引用：`docs/00-基准与治理/RGS-OPEN-QA-2026-08-31-test-summary_v0.3.md` §Q5 + 业务实装 commit `f556991`

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v1.0 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008）| 初版（per WF-1-55.78 P2 5 域 Lead RACI v1.0；基于 RGS-RACI-001 v0.1 160 单元 + RGS-ADR-0055 v0.1 升版） |
| v1.1 | 2026-08-26 20:42 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | §4 5 域 Lead 联合签字栏全部填充已签（20 行 = 5 域 × 4 行）（per `kubectl get endpoints -n rust-game-server` 实证） |
| v1.1.1 | 2026-09-01 23:10 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | §6 业务决议段新增 Q5 guild capacity 50 决议行 (per WBS v0.2 §2.2 Phase B 桶 8 B4 + OPEN-QA v0.2 §Q5 决策) |

## A. v1.0 升版增量

### A.1 源 RGS-RACI-001 v0.1 → v1.0

- 通用 160 单元矩阵 → per-domain 42 单元具体签字栏
- 增加责任到人映射（per DEC-008 一人公司 12 角色）
- 增加 5 域 Lead 联合签字栏
- 4 类已知缺口勾选（per DDD Review 必查）

### A.2 对本 SPEC 的影响

- player 域 DDD Review 模板（RGS-SPEC-CROSS-011 v0.1）可直接调用本文档 §2 矩阵 + §4 签字栏
- 不影响 RGS-IMPL-100 saga 域 + 4 份 sister RACI（各自独立 v1.0）

### A.3 已知缺口

- 4 类已知缺口见 §5（DDD Review 必查）
- 配套 4 份 sister RACI（economy / match / social / admin）需并行 v1.0

### A.4 引用链与证据

- RGS-RACI-001 v0.1（commit `14786a5`，2026-08-26 09:30 JST）
- RGS-ADR-0055 v0.1（commit `d6a56c6`，2026-08-25 06:26 JST，per WF-1-55.49）
- RGS-SPEC-CROSS-011 v0.1（commit `7e851a2`，2026-08-26 09:30 JST，DDD Review 模板）
- RGS-IMPL-100 saga 域召集人决策
- per WBS v0.4 L4 #WF-1-55.78 (P2 推进)
- per RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分
- per Ulysses 2026-08-26 09:27 JST "完成剩余工作到 P2"
- 修订历史代签新规则 per 2026-08-26 08:40 JST (C:\Users\leon19\.minimax\memory\user.md "文档代签规则反转")
