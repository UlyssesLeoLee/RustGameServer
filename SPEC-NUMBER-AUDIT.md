# Spec 数字对账审计

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-27
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **基点 commit**：`139b80a`（RGS 历史扩量终审 commit）
> **签批**：⏳ 待 Ulysses 终审

---

## 0. 报告目的

Phase C 54 份 spec 草案（per commit `876a2a7`）+ 子代理 A 15 项 P1 修复（per commit `0e00318`）+ 子代理 B Phase D 骨架（per commit `6f3c90a`）后，存在多处"数字"在不同 spec / arch / docs 文档中**自相矛盾**。本审计列当前真实数字 + 期望值 + 修复建议。

## 1. 数字一致性矩阵

| 维度 | 任务原文 (brief) | spec (per 子代理 A 修复后) | arch (per 0e00318) | 实现 (per 6f3c90a + 子代理 A phase-d-impl) | 一致? | 修复建议 |
|---|---|---|---|---|---|---|
| **star CLI 命令数（MVP）** | 17 | 17（per cli/01 §2 主表）| 17（per arch/03 §2.2）| 3（per 子代理 B 骨架）| ⚠️ spec/arch 一致，**实现仅 3 个** stub | 子代理 A Phase D 任务 1-3 后续补 14 个（17 - 3 = 14）|
| **star CLI 命令数（扩展）**| 无 | 11（per cli/01 §2 附表）| 6（per arch/03 §2.2）| 0 | ⚠️ spec 11 vs arch 6 数字不一致 | Phase D 实施对账 |
| **star CLI 命令数（合计）**| 17 | 17 + 11 = 28 | 17 + 6 = 23 | 3 | — | — |
| **MCP tool 数（spec/mcp/01 §2）**| 13 | 16（per P1-F 加 submit 后）| 14（per arch/03 §2.3，13 MVP + 1 扩展）| 16（per 6f3c90a 16 tool stub）| ⚠️ spec 16 vs arch 14 vs 实现 16 | Phase D 实施对账（arch/03 §2.3 需升 v0.3 说明 16 = 13 MVP + 3 扩展）|
| **MCP tool 数（arch/03 §2.3）**| 14 | — | 14（per 0e00318）| — | — | 0e00318 已修（加 MVP 13 子集边界表）|
| **REST endpoint 数（spec/rest/01）**| 12 | 12（per 0e00318 + 14 endpoint）| — | 0 | ⚠️ spec 12 vs 14 不一致 | Phase D 实施对账 |
| **REST endpoint 数（arch/05 §5）**| — | — | 14（per 0e00318）| 0 | — | 0e00318 已修（加 MVP 12 子集边界）|
| **Universal Submit 步数**| 11 | 12（per 0e00318 文字+列表统一）| — | 12（per 6f3c90a submit stub 注释）| ✅ 全部 12 | 已修 |
| **Agent Task Lifecycle 状态数**| 9 + 4 = 13 | 9 + 5 = 14（per 0e00318 保留 5 异常）| — | — | ⚠️ spec 14 vs brief 13 | 任务摘要 9+4 是简写，实际 9+5 = 14（spec 修订行已说明）|
| **Agent Resume 字段数**| 无 | 11（per 0e00318 agent-api/01 §3.17）| — | — | ✅ | 已修 |

## 2. 修复建议（按 P0 优先级）

| 优先级 | 修法 | 涉及 spec | 工作量 |
|---|---|---|---|
| **P0-1** | 子代理 A Phase D 任务 1-3 已实装 3 个 CLI 命令，剩 14 个补到 17 MVP + 11 扩展 = 28 | cli/01 | ~3 子代理并行 ~500K tokens |
| **P0-2** | arch/03 §2.3 MCP tool 数从 14 升 v0.3：16 = 13 MVP + 3 扩展 | arch/03 | ~5K tokens（spec text only）|
| **P0-3** | arch/05 §5 REST endpoint 数对齐：12 MVP + 2 扩展 = 14（已 0e00318 修）| arch/05 | 已修 |
| **P1-1** | 子代理 A Phase D 后续：MCP transport 实装（per spec/mcp/01 §1 stdio / Streamable HTTP）| mcp/01 | ~1 子代理 ~1M tokens |
| **P1-2** | Universal Submit 端到端实装（per 6f3c90a submit stub 注释）| flows/05 + star-cli | Phase D 任务 3 后续 |
| **P1-3** | spec/mcp/01 §2 vs arch/03 §2.3 数字差对齐（16 vs 14）| mcp/01 + arch/03 | ~10K tokens |
| **P2-1** | cli/01 §2 11 扩展 vs arch/03 §2.2 6 扩展数字差（11 vs 6）| cli/01 + arch/03 | ~5K tokens |

## 3. 守门规则

| 守门 | 状态 |
|---|---|
| 不改 commit hash | ✅ |
| 不写代码 | ✅（本审计仅 .md 文本）|
| 不 commit（Mavis 终审）| ✅ |
| 不沿用 bc23d6c 叙事 | ✅ |
| 缺标比错标（per user.md 2026-08-26 强证据）| ✅（5 处未对齐数字显式列）|

## 4. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-08-27 | 🟡 草案 v0.1；3 P0 + 3 P1 + 1 P2 待 Phase D 实施消解 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM）| ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：9 维度数字一致性矩阵 + 7 修复建议 + 守门 | 子代理 B 任务 2 失败后 Mavis 接手 |
