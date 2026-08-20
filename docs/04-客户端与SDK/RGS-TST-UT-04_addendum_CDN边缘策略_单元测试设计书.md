# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 04 客户端与SDK — CDN 边缘策略（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-04-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-030-ADD1 v0.2 + RGS-DTL-027 v0.2 §6.3〜§6.4 |
| V模型层级 | TL-1 单元试验 |
| 制定日 | 2026-08-19 |

---

## 1. 目的

覆盖 FR-CDN-030~035 CDN 边缘节点缓存与回源回退。

## 2. 测试用例

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-C001 | FR-CDN-030 经批准边缘缓存 | TTL 5min 后过期 | N |
| TST-UT-04-C002 | FR-CDN-031 缓存键 | {channel}/{version}/{region}/{file} 格式 | N |
| TST-UT-04-C003 | FR-CDN-031 | region=cn-east 路由 | N |
| TST-UT-04-C004 | FR-CDN-032 回源 | 边缘 miss → 已批准 DistributionBackend 源站 | N |
| TST-UT-04-C005 | FR-CDN-032 | 已批准源站失败 → 上一稳定版 | A |
| TST-UT-04-C006 | FR-CDN-033 channel | canary / stable / legacy 路由 | N |
| TST-UT-04-C007 | FR-CDN-034 切回 | channel 0% ≤ 30s | B |
| TST-UT-04-C008 | FR-CDN-035 强制更新 | client_version < min → 302 | A |
| TST-UT-04-C009 | ARC-045-1/2 启用门禁 | 缺 BOM、许可证/商业条款审查或 ADR 的后端拒绝；Approved profile 可激活 | A |
| TST-UT-04-C010 | ARC-045-5 Ed25519 | 签名校验 100ms | B |
| TST-UT-04-C011 | NFR-CDN-101 | 命中率统计正确 | N |
| TST-UT-04-C012 | NFR-CDN-103 | 灰度切流 30s | B |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CDN-030~035 | TST-UT-04-C001~C008 |
| ARC-045-1~5 | TST-UT-04-C009~C010 |
| NFR-CDN-101／103 | TST-UT-04-C011~C012 |
| NFR-CDN-105 | TST-UT-04-C009 |
| AC-CDN-101~105 | 全部 |
| AC-CDN-106 | TST-UT-04-C009 |

## 4. 通过判定

- 全部 PASS
- 命中率 ≥ 80% / 95%
- 切流 ≤ 30s

---

> 与 RGS-TST-UT-04 共存。
