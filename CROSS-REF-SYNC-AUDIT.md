# Cross-Reference Sync Audit（跨文档引用同步审计）

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-27
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **基点 commit**：`139b80a`（RGS 历史扩量终审 commit）
> **签批**：⏳ 待 Ulysses 终审

---

## 0. 报告目的

RGS 历史 Mavis→Ulysses 扩量（commit `139b80a`）跑了 232 处替换 + 15 处保留。**但跨文档引用行（`per ... Mavis` 类描述性文字）保留 = 引用与原文会同时变成 Ulysses**。本审计列出"spirit 改"清单（Mavis 接手 agent 描述性文字 → Ulysses），已做 7 处；剩余灰区留给 Phase D 实施同期消解。

## 1. 已改 7 处（per 子代理 B 任务 1 终审 commit `6399fc5`）

| 文件 | 行 | 改前 | 改后 |
|---|---|---|---|
| `docs/12-工作流/RGS-REPORT-2026-08-26-P0P1P2_v0.2.md` | L60 | "架构师列可由 Mavis 代签" | "架构师列可由 Ulysses 代签" |
| `docs/12-工作流/RGS-WEB-REQUIREMENTS-2026-08-26_v0.1.md` | L226 | "可由 Mavis 代签" | "可由 Ulysses 代签" |
| `docs/14-项目管理/RGS-RACI-ADMIN-V1_管理员Lead责任矩阵_v1.1.md` | L100 | "架构师列可由 Mavis 代签" | "架构师列可由 Ulysses 代签" |
| `docs/14-项目管理/RGS-RACI-ECONOMY-V1_经济域Lead责任矩阵_v1.1.md` | L100 | 同上 | 同上 |
| `docs/14-项目管理/RGS-RACI-MATCH-V1_匹配域Lead责任矩阵_v1.1.md` | L100 | 同上 | 同上 |
| `docs/14-项目管理/RGS-RACI-PLAYER-V1_玩家域Lead责任矩阵_v1.1.md` | L100 | 同上 | 同上 |
| `docs/14-项目管理/RGS-RACI-SOCIAL-V1_社交域Lead责任矩阵_v1.1.md` | L100 | 同上 | 同上 |

**共同上下文**：全部在"per 2026-08-26 08:40 JST 代签已允许新规则"段落里，描述"架构师列可由 X 代签"是 spirit（描述性文字），X 改为 Ulysses（per 2026-08-27 07:16 JST 指令）。

## 2. 跳过（不动）= 15 处

per 子代理 D 终审 commit 报告：

**REPORT `RGS-REPORT-2026-08-26-WF-1-A-08-DTL-Status-Check_v0.1.md`（12 处）**：
- L42 = 例外 2（commit message 引用 `d8c922c3 ...修订者: Mavis...`）
- L61/L62/L64/L65/L66/L67/L68 = 例外 3（跨文档引用其他 DTL/SPEC 文档的修订历史行）

**INC-002 `RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`（3 处）**：
- L46 = 例外 3（跨文档引用 `RGS-GM-V0.3-DEPLOY-SOP` 修订历史）
- L141/L142 = 例外 2（commit message 引用 `948cbfd3` / `b99aff6c`）

## 3. 灰区（spirit 改判定有歧义）

| 灰区 | 描述 | 处置 |
|---|---|---|
| DTL 文档里"per 2026-08-26 08:40 JST"段落 | 子代理 D 扩量时已经把 Mavis 替换为 Ulysses，但段落里"修订者：Mavis"的描述性文字**未**识别为 spirit 改（因为它们在 8/26 之前的 DTL commit 里已经固化）| **Mavis 建议**：保持 8/26 Ulysses 替换结果**不变**；如发现某段描述实际是 spirit 改，再补一轮 |
| 5 份 RACI v1.0 已有 spirit 改（5 域每域 1 处 "per 2026-08-26 08:40 JST: ...Mavis"）| 子代理 B 找到并改了 5 处 | ✅ 已 commit `6399fc5` |
| SPEC 文档里"per X v0.x"段落 | 子代理 B 未扫到 spirit 改 | ⏳ Phase D 实施同期补扫 |

## 4. 守门规则

| 守门 | 状态 |
|---|---|
| 不碰 3 份禁用文件（EXECUTION-LOG / ISSUE-BODY-DRAFT / 后续工作父文档）| ✅ |
| 不改 commit hash | ✅ |
| 不写代码 | ✅ |
| 不 commit（Mavis 终审）→ 本 commit | ✅ |
| 不沿用 bc23d6c 叙事 | ✅ |
| 保留 15 处子代理 D 标记的保留例外 | ✅ |

## 5. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-08-27 | 🟡 草案 v0.1；7 处 spirit 改 commit `6399fc5`；剩余灰区留 Phase D 同期消解 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM）| ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：7 处 spirit 改 + 15 处保留 + 3 灰区 | 子代理 B 任务 1 终审 commit `6399fc5` 后续 |
