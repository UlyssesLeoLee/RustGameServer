# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 07 社交运营与玩家治理 — 风控规则 DSL（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-07-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-028-ADD1 + RGS-DTL-025（v0.3，DSL 增补） |
| V模型层级 | TL-1 单元试验 |
| 制定日 | 2026-08-19 |

---

## 1. 目的

覆盖 FR-ANT-005~012 风控规则 DSL 引擎。

## 2. 测试用例

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-07-R001 | FR-ANT-005 DSL 语法 | rule(ctx) → Score 函数定义 | N |
| TST-UT-07-R002 | FR-ANT-006 ctx API | ctx.player / ctx.action / ctx.history | N |
| TST-UT-07-R003 | FR-ANT-006 | ctx 字段越界 → 0 分（fail-safe） | A |
| TST-UT-07-R004 | FR-ANT-007 仓库 | .rhai 文件按 (channel, version) 管理 | N |
| TST-UT-07-R005 | FR-ANT-008 热更新 | 仓库变更 → 5s 内全节点生效 | B |
| TST-UT-07-R006 | FR-ANT-009 超时 | 单规则 100ms 超时 → 0 分 | B |
| TST-UT-07-R007 | FR-ANT-010 channel | canary/stable 路由正确 | N |
| TST-UT-07-R008 | FR-ANT-011 回滚 | 仓库回退 → 5s 全节点回滚 | B |
| TST-UT-07-R009 | FR-ANT-012 沙箱 | 禁止 fs.open() | A |
| TST-UT-07-R010 | FR-ANT-012 | 禁止 net.http() | A |
| TST-UT-07-R011 | FR-ANT-012 | 禁止 syscall | A |
| TST-UT-07-R012 | ARC-043-1 Rhai | Rhai 1.x API 正确 | N |
| TST-UT-07-R013 | ARC-043-6 版本 | 节点本地缓存最近 3 版本 | N |
| TST-UT-07-R014 | ARC-043-2 仓库 | Git + MinIO 落地 | N |
| TST-UT-07-R015 | ARC-043-4 沙箱 | 仅 ctx + math/string/array/时间白名单 | A |
| TST-UT-07-R016 | NFR-ANT-104 | 误判申诉率 ≤ 5% | P |
| TST-UT-07-R017 | NFR-ANT-105 | 沙箱逃逸 0 例 | A |
| TST-UT-07-R018 | DTL-025 v0.3 §6.2 制品签名 | 篡改 hash、签名、撤销密钥或可变 URI 一律拒绝并审计 | A |
| TST-UT-07-R019 | DTL-025 v0.3 §6.3 沙箱 | 操作数/深度/内存/100ms 超限及非法输出均返回 `0`，无副作用 | A |
| TST-UT-07-R020 | DTL-025 v0.3 §6.4 围栏 | 旧 epoch、跳号、跨 channel、未确认版本不得执行 | A |
| TST-UT-07-R021 | DTL-025 v0.3 §6.5 分区 | 租约失效、清单不可读或签名冲突时停止 DSL 执行并告警 | A |
| TST-UT-07-R022 | DTL-025 v0.3 §6.6 回滚 | 回滚 manifest、审批/原因和节点确认审计缺一不可 | A |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-ANT-005~012 | TST-UT-07-R001~R012 |
| ARC-043-1/2/4/6 | TST-UT-07-R012~R015 |
| NFR-ANT-104/105 | TST-UT-07-R016~R017 |
| DTL-025 v0.3 §6.2~§6.6 | TST-UT-07-R018~R022 |
| AC-ANT-101~105 | 全部 |

## 4. 通过判定

- 全部 PASS
- 沙箱逃逸 0 例
- 误判率 ≤ 5%

---

> 与 RGS-TST-UT-07 共存。
