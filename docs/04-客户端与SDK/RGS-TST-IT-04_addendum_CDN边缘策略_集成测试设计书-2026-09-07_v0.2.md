# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 04 客户端与SDK — CDN 边缘策略（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-04-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-030-ADD1 v0.2 + RGS-DTL-027 v0.2 §6.3〜§6.4 |
| 升版基线 | v0.1 (2026-08-19) → v0.2 (2026-09-07, 8 维度增量) |
| V模型层级 | TL-2 集成 / TL-3 契约 |
| 制定日 | 2026-08-19 |
| 升版日 | 2026-09-07 |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定 |
| **0.2** | 2026-09-07 | 架构师（Mavis 接手代签 per DEC-008） | 8 维度增量升版：① 8 域扩展（per 9/6 闪烁之光兼容 a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673）② 9 域 mTLS 业务级（per 9/6 d270ab9 11 步 v3 + d15a0bb 3 NEW 域 k8s yaml）③ envoy 独立 deployment（per 9/1 13:03/13:05 JST 决策）④ 派生约束守护 L15-L23（per 9/6 6c6839e cutover） |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 升版（v0.2 自审） | 架构师（Mavis 接手代签 per DEC-008） | 2026-09-07 | 8 维度增量；B3 二审待 |
| 评审（架构） |  |  | 待 Ulysses 二审 |

---

## 0. v0.1 → v0.2 升版范围（8 维度增量，主题特定子集）

| # | 维度 | v0.1 现状 (8/19) | v0.2 增量 (9/7) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §1 写"5 域 + 平台层" | 13 域（5 域 + scene/battle/network/account/sub8 + batch + 平台 + function-plane），CDN 边缘策略在 scene/battle asset bundle 命中的覆盖范围扩 | a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673 |
| 2 | **9 域 mTLS 业务级** | §2 TST-IT-04-C001~C010 未含 mTLS E2E | §2 增 TST-IT-04-C011~C014：9 域 mTLS 业务级 CDN 边缘命中与回源（per d270ab9 11 步客户端模拟器 v3 + d15a0bb 3 NEW 域 k8s yaml） | d270ab9 / d15a0bb |
| 3 | **envoy 独立 deployment** | §2 未涉及边缘层选型 | §2 增 TST-IT-04-C015：envoy 独立 deployment（不是 istio sidecar，不是 nginx），CDN 边缘 → envoy → 源站（per 9/1 13:03/13:05 JST 决策） | 9/1 13:03/13:05 JST |
| 4 | **派生约束守护 L15-L23** | §4 通过判定未含派生约束 | §4 增 §派生约束守护段，列出 L15-L23 落地状态（per 9/6 6c6839e cutover） | 6c6839e / add4238 / PHASE-0-TO-4-FINAL.md |

**已知缺口（per 缺标比错标安全, per 8/26 JST）**:
- k3s 集群实际可达性待 ST Phase C 验证（per Q8 决策）
- 商业 CDN 候选清单（Cloudflare/AWS CloudFront/Fastly）尚未落地
- 边缘节点命中真实 100 万级流量回归待 PH-7 阶段

---

## 1. 目的

验证 CDN 边缘节点 + 自托管源 + 商业 CDN 后端的端到端集成，并新增 8 域 asset bundle 命中 / 9 域 mTLS 业务级 / envoy 独立 deployment 三类场景。

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
| **TST-IT-04-C011** | **[TL-4/E2E]** | **8 域 asset bundle 边缘命中（per 9/6 闪烁之光兼容）** | scene 场景 bundle / battle 战斗 bundle / network 协议包 / account 账号资源 / sub8 8 子系统 各自命中 | 13 域 × 1000 资源 = 13000 样本 |
| **TST-IT-04-C012** | **[TL-4/E2E]** | **9 域 mTLS 业务级 CDN 边缘命中（per 9/6 d270ab9）** | mTLS 客户端模拟器 v3 走完 11 步，CDN 边缘命中 + 回源全部走 mTLS 通道 | 11 步客户端模拟器 v3 |
| **TST-IT-04-C013** | **[TL-4/E2E]** | **3 NEW 域 k8s yaml 落档 + mTLS 业务级（per 9/6 d15a0bb）** | scene / battle / account 三个新域的 k8s yaml + mTLS cert 导入 + CDN 边缘命中 | 3 NEW 域 yaml + 3 mTLS cert |
| **TST-IT-04-C014** | **[TL-4]** | **9 域 mTLS cert 轮换（per L-CAND-006）** | 9 域 cert 轮换后 CDN 边缘命中保持，0 中断 | 9 域 × 30 天 cert |
| **TST-IT-04-C015** | **[TL-3]** | **envoy 独立 deployment 边缘层（per 9/1 13:03/13:05 JST 决策）** | envoy deployment + ClusterIP svc，CDN 边缘 → envoy → 源站链路完整，不选 nginx 不选 istio sidecar | envoy 1 replica + 2 ClusterIP svc |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CDN-030~035 | TST-IT-04-C001~C006 |
| ARC-045-1/2/5 | TST-IT-04-C007/C010 |
| NFR-CDN-105、AC-CDN-106 | TST-IT-04-C001/C002/C010 |
| AC-CDN-101~106 | 全部 |
| 9/6 8 域扩展 | TST-IT-04-C011 |
| 9/6 d270ab9 9 域 mTLS 业务级 | TST-IT-04-C012/C013 |
| L-CAND-006 9 域 mTLS cert 轮换 | TST-IT-04-C014 |
| 9/1 13:03/13:05 JST envoy 独立 deployment | TST-IT-04-C015 |

## 4. 通过判定

- 全部 PASS（含 v0.1 10 条 + v0.2 5 条 = 15 条）
- 签名 100% 校验
- 切流 ≤ 30s
- 8 域 asset bundle 边缘命中率 ≥ 80%
- 9 域 mTLS 业务级 cert 验证 100% 通过
- envoy 独立 deployment 0 启动失败

## 5. 派生约束守护

| 派生约束 | 状态 | 引用 |
|---|---|---|
| L1 (cargo check 60s) | ✅ N/A (doc) | AGENTS.md §2.1 |
| L11 (cargo build dir lock) | ✅ N/A (doc) | AGENTS.md §2.2 |
| L12 (临时 log 不入 commit) | ✅ (git status 0 untracked) | AGENTS.md §2.2 |
| L13 (8 维度 git show --stat 实证) | ✅ (本节 §0 引用全部 commit 实证) | AGENTS.md §2.2 |
| L15 (native binary 跨工具链) | ⏳ (待 ST Phase C) | 6c6839e |
| L19 (mTLS 业务级 = saga 触达) | ✅ (TST-IT-04-C012 落档) | 6c6839e |
| L22 (协议码映射表) | ✅ (L-CAND-007 落档) | 6c6839e |
| L23 (4 层自动探针) | ✅ (CDN 边缘 + envoy 探针) | 6c6839e |
| B3 (DDD Review 二审) | ⏳ (Mavis 自审 1 次停手, Ulysses 二审待) | AGENTS.md §3.x |

---

> 与 RGS-TST-IT-04 共存（v0.2 升版）。
