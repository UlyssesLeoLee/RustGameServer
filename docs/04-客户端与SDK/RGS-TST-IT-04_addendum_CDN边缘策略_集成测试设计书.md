# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 04 客户端与SDK — CDN 边缘策略（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-04-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-030-ADD1 v0.2 + RGS-DTL-027 v0.2 §6.3〜§6.4 |
| V模型层级 | TL-2 集成 / TL-3 契约 |
| 制定日 | 2026-08-19 |

---

## 1. 目的

验证 CDN 边缘节点 + 自托管源 + 商业 CDN 后端的端到端集成。

## 2. 测试用例

| 用例 ID | 试验级别 | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|
| TST-IT-04-C001 | [TL-2] | 已批准自托管 `DistributionBackend` → 源站端到端 | — | — |
| TST-IT-04-C002 | [TL-2] | 商业 CDN 后端可插拔且必须经 ApprovedBackendProfile 门禁 | — | — |
| TST-IT-04-C003 | [TL-3] | manifest HTTP 契约 | — | — |
| TST-IT-04-C004 | [TL-3] | patch HTTP 契约 | — | — |
| TST-IT-04-C005 | [TL-2] | 灰度推送：canary → stable 切换 | — | — |
| TST-IT-04-C006 | [TL-2] | 强制更新：客户端低于 min_supported_version | — | — |
| TST-IT-04-C007 | [TL-2] | 资源签名 Ed25519 验证 | — | — |
| TST-IT-04-C008 | [TL-2] | 跨 region 一致性 | — | — |
| TST-IT-04-C009 | [TL-2] | 已批准源站回源失败降级至上一稳定版 | — | — |
| TST-IT-04-C010 | [TL-2] | ARC-045-1/2 后端抽象层及 BOM/许可证/ADR 启用门禁 | — | — |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CDN-030~035 | TST-IT-04-C001~C006 |
| ARC-045-1/2/5 | TST-IT-04-C007/C010 |
| NFR-CDN-105、AC-CDN-106 | TST-IT-04-C001/C002/C010 |
| AC-CDN-101~106 | 全部 |

## 4. 通过判定

- 全部 PASS
- 签名 100% 校验
- 切流 ≤ 30s

---

> 与 RGS-TST-IT-04 共存。
