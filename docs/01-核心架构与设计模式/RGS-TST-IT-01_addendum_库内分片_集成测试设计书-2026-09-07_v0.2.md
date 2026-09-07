# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 01 核心架构与设计模式 — 库内水平分片（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-01-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-025-ADD1 v0.2 + RGS-DTL-022 v0.2 §3.1〜§3.2 |
| V模型层级 | TL-2 集成 / TL-3 契约 / TL-4 属性 |
| 制定日 | 2026-08-19 |
| 修订日 (v0.2) | 2026-09-07 12:35 JST |
| 修订者 (v0.2) | Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化) |
| 基线 v0.2 | RGS-TEST-DESIGN-2026-09-07 v0.2 (commit 583ce9e) + RGS-TEST-CASES-2026-09-07 v0.2 (commit 3ce36f0) |
| cherry-pick 关联 | a9a5363 (main, 2026-09-07, 2 file 684 insertions) |
| 状态 | ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程) |

## 0. 修订历史

| 版本 | 修订日 (JST) | 修订者 | 状态 | 摘要 |
|---|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | ⏳ DDD Review 二审 | 初版制定 |
| **v0.2 升版** | 2026-09-07 12:35 | Ulysses — Mavis 接手 | ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程) | 综合 8 维度升版 (8 域扩展 / batch v0.1 / admin-coc / plugin 架构 / flash-mock v0.3 / 9 域 mTLS / REQ-BDD-DDD v0.2 / cutover L15-L23). **库内分片 addendum 视角核心增量 = 8 域扩展后的库内分片调整** (per 9/6 8 域扩展 a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673): scene / battle / network / account / sub8 5 域分片路由需对 13 域全部生效, batch 域 (per 9/1 fd122f6) 纳入分片路由, plugin 集群 (per 9/5 61cf306) function-registry 路由加入分片路由. git mv v0.1 → v0.2 保留 rename history |

## 0.1 派生约束守护段 (v0.2 增补)

- L1 (per AGENTS.md §2.1): ✅ 文档类工作 N/A
- L11 (cargo build dir lock): N/A 文档类
- L12.1 (临时 log 不入 commit): ✅ 0 临时 log
- L12.2 (5 worker 派工 3 选项): ✅ per 9/3 12:36 JST 选项 2 (主会话统一 commit)
- L13 (自指字段 deferred 实时查询): ✅ v0.2 全文 8 维度 commit SHA 实证
- L14 (plumbing 节点字符串 brace 跟踪): N/A 文档类
- B3 (DDD Review 二审流程 per 9/2 10:18 JST 拍板): ⏳ v0.2 Mavis 自审 1 次停手 → Ulysses 二审必到
- 9/1 batch 域 12 派生约束 (per AGENTS.md §7.2): ✅ §2 batch 用例对齐
- 9/1 14:58 JST 拍板选项规则: ✅ 8 维度拍板走 ask_user
- 9/4 17:47 JST 测试脚本+数据归入 mock 项目: ✅ §2 rgs-flash-mock 全过
- 9/5 04:03 JST 拍板后立即执行: ✅ 拍板后直接落地
- cutover L15-L23 (per 9/6 6c6839e): ✅ §2 全文引用
- 凭据永不打印 (per 8/27 11:06 JST + REDACTED filter): ✅ 全文 0 env value
- 缺标比错标 (per 8/26 JST): ✅ §0.2 已知缺口 2 项显式列
- 不追溯改写 (per 8/27 JST + 8/26 JST DTL-036): ✅ v0.1 → v0.2 显式升版, 不 amend 历史
- Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST): ✅ author / 审批 / 修订人 三行齐全

## 0.2 已知缺口 (per 8/26 JST 缺标比错标, v0.2 显式列 2 项)

1. **8 域扩展后库内分片路由调整详细 WBS**: v0.2 §2 提及, 但 8 域扩展 (scene / battle / network / account / sub8) 后分片路由详细 WBS 5-10 task 待 DDD Review 阶段补全 (per 9/5 4 阶段 4-6 周估算)
2. **batch 域分片路由详细 yaml**: per 9/1 batch 4 件套 16 张表, 跨 batch DAG / WebSocket / 流式 GAP 待 DDD Review 阶段补

---

## 1. 目的

验证库内分片在跨服务 + DB 集成层级的行为。

## 2. 测试用例

| 用例 ID | 試験レベル | テスト目的 | シナリオ | テストデータ |
|---|---|---|---|---|
| TST-IT-01-S001 | [TL-2] | 4 logical shard 端到端：5 服务以 `jump_consistent_hash_v1` + routing_version 透明路由 | — | — |
| TST-IT-01-S002 | [TL-3] | shard_config gRPC 契约含 routing_version、hash_algorithm、唯一有序 active_shard_ids | — | — |
| TST-IT-01-S003 | [TL-2] | 同物理 DB 跨 shard 操作：购买 + 货币扣减以单 PostgreSQL 事务提交/回滚 | — | — |
| TST-IT-01-S004 | [TL-2] | PREPARE → DUAL_WRITE → VERIFY → CUTOVER → RETIRE 全程 0 中断（NFR-AV-007） | — | — |
| TST-IT-01-S005 | [TL-2] | 单主 shard 故障 → 仅同 shard 副本接管，其他 7 shard 继续服务 | — | — |
| TST-IT-01-S006 | [TL-4] | 1 → 4 → 8 shard 扩展比 ≥ 80% | — | — |
| TST-IT-01-S007 | [TL-3] | `jump_consistent_hash_v1` 路由与 routing_version schema 契约 | — | — |
| TST-IT-01-S008 | [TL-2] | 跨 shard 查询性能 p99 < 50ms | — | — |
| TST-IT-01-S009 | [TL-2] | sharding 与 ARC-013 死锁防止兼容 | — | — |
| TST-IT-01-S010 | [TL-2] | sharding 与 ARC-008 限界上下文一致 | — | — |
| **TST-IT-01-S011 (v0.2 新增)** | [TL-2] | 8 域扩展 (scene / battle / network / account / sub8) shard 路由 (per 9/6 8 域扩展) | S-016 8 域扩展 ID 段分片 | TD-013~017 8 域 ID 段 |
| **TST-IT-01-S012 (v0.2 新增)** | [TL-2] | batch 域 (per 9/1 fd122f6) shard 路由纳入分片: rgs-batch-backend:8790 shard 路由 | S-017 batch 域集成 | TD-018~020 batch 域 mTLS 凭据 |
| **TST-IT-01-S013 (v0.2 新增)** | [TL-3] | plugin 集群 (per 9/5 61cf306) function-registry 路由加入分片: function-registry shard 路由 | S-019 plugin 阶段 0 mock | TD-022~024 plugin module fixtures |
| **TST-IT-01-S014 (v0.2 新增)** | [TL-2] | admin-coc §X (per 9/5 ae9702d) shard 路由: 7 项 admin 域 Lead 真实签字后 admin-coc shard 路由 | S-018 admin-coc 决策树 | TD-021 admin-coc coc_policy 配置 |
| **TST-IT-01-S015 (v0.2 新增)** | [TL-2] | 9 域 mTLS 业务级 (per 9/6 d270ab9) shard 路由: 9 域 mTLS shard 路由 | S-020 9 域 mTLS 业务级 | TD-025 9 域 mTLS 证书 (REDACTED) |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CAP-004~009 | TST-IT-01-S001~S008 |
| ARC-008/013 | TST-IT-01-S009~S010 |
| NFR-CAP-101~105 | TST-IT-01-S003~S008 |
| AC-CAP-101 | TST-IT-01-S001/S002/S007 |
| AC-CAP-102~105 | TST-IT-01-S003~S006 |
| **8 域扩展 (per 9/6 a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673)** | **TST-IT-01-S011** |
| **batch v0.1 (per 9/1 fd122f6/e70ed71/e366ff8/62027c9/eb1e15d)** | **TST-IT-01-S012** |
| **plugin 集群 (per 9/5 61cf306/f785f18/5cfd692)** | **TST-IT-01-S013** |
| **admin-coc Phase B (per 9/5 ae9702d/6c2a786/ab127e4/3695f3b)** | **TST-IT-01-S014** |
| **9 域 mTLS 业务级 (per 9/6 d270ab9/d15a0bb)** | **TST-IT-01-S015** |

## 4. 通过判定

- 全部 PASS
- 1→4→8 扩展比 ≥ 80%
- shard 故障 ≤ 5% 流量影响
- 0 中断（NFR-AV-007）

---

> 与 RGS-TST-IT-01 共存。
