# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 04 客户端与SDK — 系统测试（ST）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-04 |
| 版本 | 0.2 (v0.1 → v0.2 升版 per 2026-09-07 12:35 JST 拍板) |
| 父文档 | RGS-REQ-012/030 + RGS-REQ-001 §7/§9 |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』詳細設計工程 |
| 本主题域源文档全集（REQ/BAS/DTL） | RGS-REQ-012、RGS-REQ-030、RGS-BAS-008、RGS-BAS-027、RGS-DTL-008、RGS-DTL-027 |
| 关联 v0.2 升版基线 | RGS-TEST-DESIGN-2026-09-07_v0.2.md (commit 583ce9e) + RGS-TEST-CASES-2026-09-07_v0.2.md (commit 3ce36f0) |

| V模型层级 | TL-6/7/8 ↔ REQ |
| 制定日 | 2026-08-19 (v0.1) / 2026-09-07 (v0.2 升版) |
| 制定者 | Mavis 接手 agent per DEC-008 (代签 Ulysses) |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定
| 0.3 | 2026-09-01 | 架构师（Mavis 接手代签 per DEC-008） | 按 2026-09-01 JST 拍板决策，添加「シナリオ」「テストデータ」2 列；新增 §1.4.5 字段定义引用 + §3.0 占位章节；§3.x 全部用例表扩展为 12 列 (Pattern A) / 9 列 (Pattern B)。详细场景/测试数据在各领域 ST 实施阶段补充 |
| **0.2 (字段级深化)** | 2026-08-19 | 架构师 | **字段级深化**：每条用例的"对应设计"列升级为"文档 ID + §X.Y + 表/图/字段"；新增"ADR 决策验证"小节覆盖本主题 ADR；新增"TBD 处置"小节 |。覆盖 SDK 三引擎一致 + 资源分发端到端 + AC-SDK/CDN-* + NFR Lv.3/4 |
| **v0.2 (Round 2 W5 升版)** | 2026-09-07 | Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化) | **Round 2 W5 v0.2 升版 (综合 8 维度 + ST 特定增量)**: 1) §0 增 8 维度增量表 (8 域扩展 / batch v0.1 / admin-coc / plugin 架构 / flash-mock v0.3 / 9 域 mTLS 业务级 / REQ-BDD-DDD v0.2 / cutover 收口 L15-L23) 2) §3.6 新增 ST 特定增量用例 (9/6 8 域扩展 scene/battle/network/account + sub8 + 9 域 mTLS 11 步 v3 per d270ab9) 3) §3.7 增 9 域 mTLS 业务级 E2E ST 用例 4) §3.8 增 batch v0.1 6 module × 15 用例映射 5) §6.7 派生约束守护段增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构/QA/SRE） | | | AC-001~019 + NFR |
| 审批（负责人） | | | 商用上线门禁依据 |

---

## 目次（目次 / Table of Contents）

1. 前言（はじめに / Preface）
   1.1 目的（目的 / Purpose）
   1.2 适用范围（適用範囲 / Scope）
   1.3 关联文档（関連文書 / Related Documents）
   1.4 记述规则（記述規則 / Notation Rules）
   1.5 字段级映射说明
   1.6 命名约定（命名規約 / Naming Convention）
2. 测试策略（テスト戦略 / Test Strategy）
3. 测试用例（テストケース / Test Cases）
4. 追溯性矩阵（トレーサビリティ・マトリクス / Traceability Matrix）
5. 测试执行计划（テスト実行計画 / Test Execution Plan）
6. 通过判定基准（合格判定基準 / Pass Criteria）
7. 风险与未决事项（リスクと未決事項 / Risks and TBDs）

注：本文档实际章节以文中二级标题为准。

---

## 0. v0.1 → v0.2 升版范围 (Round 2 W5 综合 8 维度 + ST 特定增量)

per 2026-09-07 12:35 JST 拍板 (scope=opt4 全部 v0.2 综合 8 维度), 本主题 ST 详细设计书 (客户端与SDK) 升版增量:

| # | 维度 | v0.1 现状 (9/5 拍板) | v0.2 增量 (9/7 拍板) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §2 写 "5 域" | 13 域 (player / economy / match / social / admin + scene / battle / network / account / sub8 / batch + 平台 + function-plane) | 1134cfd / 95e67a6 / 57edbeb / b6b19b7 / 1dd9afc / 3c79bca / 42df673 |
| 2 | **batch v0.1 + v0.2 EVAL** | §2 完全没提 batch 域 | §3.8 增 6 module × 15 用例 = 90 用例 (cron / task_templates / worker_pool / audit_logger / dlq / connector) | fd122f6 / e70ed71 / e366ff8 / 62027c9 / eb1e15d |
| 3 | **admin-coc Phase B** | §3.5 仅 AC-015 OSI 100% | §3.9 增 admin-coc §X 集成设计 + 7 项 admin 域 Lead 真实签字 + coc_policy 决策树 3 场景 ST 用例 | ae9702d / 6c2a786 / ab127e4 / 3695f3b |
| 4 | **plugin 集群 + app 集群架构** | §3.4 VF-015 三引擎一致仅 PoC | §3.10 增 4 阶段用例 (阶段 0/1/2/3, per 9/5 61cf306 ARCH §4) | 61cf306 / f785f18 |
| 5 | **flash-mock v0.3** | §3.1 仅 "复用核心 SDK" | §3.11 增 v0.3: 60 module + 12 大类 RPC + 30+ module 业务扩展 + 4 NEW 回归脚本 | 575f5c9 / fdba686 / 01aee71 |
| 6 | **9 域 mTLS 业务级** | §3.4 NFR-SDK-001 Lv.3 一致性 | §3.7 增 9 域 mTLS 业务级 E2E 11 步客户端模拟器 v3 (per 9/6 d270ab9) | d270ab9 / d15a0bb |
| 7 | **REQ/BDD/DDD v0.2 升版** | §1.3 仅 v0.1 addendum 引用 | §1.3 增 3 addendum 引用 (业务逻辑逆推 + 协议号映射 + frontend 适配) | 39d817b / 96e6b3c / 554b1ef |
| 8 | **cutover 收口** | §7 仅"商用上线门禁" | §7 增 13 commit 推远端 + 派生约束 L15-L23 落地 (per 9/6 6c6839e cutover 收口) | 6c6839e / add4238 |

**ST 特定增量** (per W5 任务简报, 跟 IT-04 不同, ST 强调端到端业务级 9 域 mTLS):
- **9/6 8 域扩展 (scene / battle / network / account + sub8)**: per 57edbeb / b6b19b7 / 1dd9afc / 3c79bca 8 域扩展 NEW, SDK 客户端需支持 8 域 gRPC 调用 9 域 mTLS 业务级
- **9 域 mTLS 业务级 11 步 v3**: per 9/6 d270ab9, ST §3.7 EX-ST-MTLS-9D-001 11 步 E2E 模拟器
- **派生约束守护**: L1/L11/L12/L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 (per AGENTS.md §6.2 + 9/2 10:18 JST D2 拍板)

**目标读者**: ST 业务级测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 签字 / SRE Lead 接管验证

---

## 1. 前言

## 1.1 目的（目的 / Purpose）

TL-6/TL-7/TL-8 层级，验证客户端 SDK 三引擎一致性与资源分发端到端。

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| （见各文档编号） | | |

## 1.4 记述规则（記述規則 / Notation Rules）

### 1.4.1 强度用语（强度表現 / Strength of Expression）

本文档遵循 RFC 2119 与 IPA 共通フレーム 2013 规定的强度用语：

| 中文表述 | 日文表述 | 英文 | 强度 | 含义 |
|---|---|---|---|---|
| **必须** | 必ず / 必須 | MUST | 强 | 必要条件。未满足则不予验收 |
| **应当** | すべき / 推奨 | SHOULD | 中 | 推荐条件。未满足时必须记录理由并取得批准 |
| **不得** | してはならない / 禁止 | MUST NOT | 强 | 禁止事项。违反即为设计缺陷 |
| **可以** | してもよい / 任意 | MAY | 弱 | 任意条件。是否实现不影响验收 |

### 1.4.2 优先级符号

| 符号 | 中文 | 日文 | 含义 |
|---|---|---|---|
| ◎ | 必须 | 必須 | 商用上线前必须实现 |
| ○ | 推荐 | 推奨 | 商用上线前应当实现 |
| △ | 任意 | 任意 | 上线后追加实现 |
| × | 范围外 | 範囲外 | 本次范围外 |

### 1.4.3 标识符体系

本文档遵循 RGS-REQ-001 §1.5.3 既定标识符体系：
- `RGS-TST-XX-NNN` 测试用例编号
- `RGS-{REQ|BAS|DTL}-NNN` 父文档编号
- `RGS-ADR-NNNN` 架构决策记录编号
- `NFR-<区分>-NNN` 非功能需求编号
- `AC-NNN` / `VF-NNN` / `FT-NNN` 验收/验证/故障注入编号
- `BZ-NNN` 业务规则编号
- `ST-NNN` 状态机编号

### 1.4.4 引用约定

- 全部引用以编号（如 `RGS-REQ-006`）而非文件路径
- 同一编号在本文档中首次出现时附全称，后续仅用编号


### 1.4.5 シナリオ / テストデータ 字段（IT 必须包含 / ST 必须包含）

按 2026-09-01 JST 拍板决策，集成测试 (IT) 与系统测试 (ST) 设计书**必须**在用例表内包含「シナリオ」「テストデータ」2 列。完整字段定义、填写规则、命名约定（S-NNN / TD-NNN）见 `RGS-TST-ST-00 §1.4.5 / §1.6 / §3.0`。本设计书的场景/测试数据编号自 S-NNN 续编，详见本设计书 §3.0。

## 1.5 字段级映射说明

本版本（0.2）的核心升级是**字段级映射**：每条测试用例的"对应设计"列从"§X.Y 章节名"升级为"文档 ID + §X.Y + 表/图/字段"。

**映射规则**：
- 每个测试模块对应 1 个或多个父文档的物理/实现级章节
- 每条用例精确引用其父文档的具体字段（如 DDL 字段、gRPC 方法字段、状态机迁移名）
- 模块汇总表（§2.2）给出该文档验证的字段清单与覆盖率目标

**V 模型强化对应**：本文档对应该主题父基本设计书与详细设计书，构成"V 字"右侧的 TL-1/2/3 单元素验证。

## 1.6 命名约定（命名規約 / Naming Convention）

- 用例 ID：`TST-{UT|IT|ST}-XX-NNN`（XX 为主题编号 00-07）
- 试验级别标注：UT 无标注 / IT 用 [TL-2/3/4/5] / ST 用 [TL-6/7/8/E2E]
- 覆盖类型：N=正常 / A=异常 / B=边界 / P=属性不变条件 / S=状态机非法迁移
- **场景编号**：`S-NNN`（与用例 ID 解耦，1 场景可被多用例引用，详见 §3.0 场景集）
- **测试数据编号**：`TD-NNN`（与场景编号解耦，1 场景可有多组数据，详见 §3.0 测试数据集）
- 运行时机：`cargo test --workspace`（主干 CI 必跑，QA-006 ≤ 15 min 约束内）


## 2. 测试策略

```
需求 RGS-REQ-*  ┐ ST  ★ RGS-TST-ST-04 ★
基础 RGS-BAS-*  ┐ IT
详细 RGS-DTL-*  ┐ UT
实现            ┘
```

阶段归属：PH-1/PH-2 SDK，PH-6 资源分发。

---

## 3. 测试用例


## 3.0 场景集与测试数据集占位

本设计书的场景集（S-NNN）与测试数据集（TD-NNN）由本主题域负责人在用例实装阶段补充。参考主模板 `RGS-TST-ST-00 §3.0` 的格式与字段约定。占位期间, 用例表内「シナリオ」「テストデータ」列以 `—` 标记。

## 3.1 客户端 SDK 端到端（REQ-012 / BR-SDK-001~003 / ARC-024）

| 用例 ID | 试验级别 | 对应需求 | シナリオ | テストデータ | 测试目的 |
| --- | --- | --- | --- | --- | --- |
| TST-ST-04-001 | [E2E] | BR-SDK-001 | — | — | 三引擎接入 |
| TST-ST-04-002 | [E2E] | BR-SDK-002 | — | — | 三引擎一致 (VF-015) |
| TST-ST-04-003 | [E2E] | BR-SDK-003 | — | — | 协议演进 |
| TST-ST-04-004 | [E2E] | FR-SDK-001~004 | — | — | 全部 FR |
| TST-ST-04-005 | [E2E] | FR-SDK-010~012 | — | — | FFI |
| TST-ST-04-006 | [E2E] | FR-SDK-013 | — | — | 一致性 |
| TST-ST-04-007 | [E2E] | NFR-SDK-001 Lv.3 | — | — | 一致性回归 |
| TST-ST-04-008 | [E2E] | NFR-SDK-004 Lv.3 | — | — | 协议演进 |
| TST-ST-04-009 | [E2E] | AC-SDK-001~003 | — | — | 3 验收 |
| TST-ST-04-010 | [E2E] | Bevy 完整 | — | — | 端到端 |
| TST-ST-04-011 | [E2E] | Unity 完整 | — | — | 端到端 |
| TST-ST-04-012 | [E2E] | UE 完整 | — | — | 端到端 |
| TST-ST-04-013 | [E2E] | 同轨迹重放 | — | — | 行为一致 |
| TST-ST-04-014 | [E2E] | panic 捕获 | — | — | A |
| TST-ST-04-015 | [E2E] | 零拷贝 | — | — | N |

## 3.2 资源分发端到端（REQ-030 / BR-CDN-001~006 / ARC-045）

| 用例 ID | 试验级别 | 对应需求 | シナリオ | テストデータ | 测试目的 |
| --- | --- | --- | --- | --- | --- |
| TST-ST-04-020 | [E2E] | BR-CDN-001 | — | — | 版本查询 |
| TST-ST-04-021 | [E2E] | BR-CDN-002 | — | — | 增量补丁 |
| TST-ST-04-022 | [E2E] | BR-CDN-003 | — | — | 完整性 |
| TST-ST-04-023 | [E2E] | BR-CDN-004 | — | — | 灰度 |
| TST-ST-04-024 | [E2E] | BR-CDN-005 | — | — | 强制更新 |
| TST-ST-04-025 | [E2E] | BR-CDN-006 | — | — | 自托管 |
| TST-ST-04-026 | [E2E] | FR-CDN-001~004 | — | — | 清单 |
| TST-ST-04-027 | [E2E] | FR-CDN-010/011 | — | — | 差分 |
| TST-ST-04-028 | [E2E] | FR-CDN-012/013 | — | — | 校验 |
| TST-ST-04-029 | [E2E] | FR-CDN-020~022 | — | — | 灰度 |
| TST-ST-04-030 | [E2E] | FR-CDN-023/024 | — | — | 强制更新 |
| TST-ST-04-031 | [E2E] | NFR-CDN-001~005 | — | — | Lv.3 |
| TST-ST-04-032 | [E2E] | AC-CDN-001~006 | — | — | 6 验收 |
| TST-ST-04-033 | [E2E] | 签名验证 | — | — | 端到端 |
| TST-ST-04-034 | [E2E] | 完整性 | — | — | hash 失配拒 |
| TST-ST-04-035 | [E2E] | 灰度一致 | — | — | hash |
| TST-ST-04-036 | [E2E] | 快速下线 | — | — | 全 0 |
| TST-ST-04-037 | [E2E] | MinIO 部署 | — | — | 端到端 |
| TST-ST-04-038 | [E2E] | Nginx 缓存 | — | — | 端到端 |

## 3.3 业务规则与状态机

| 用例 ID | 试验级别 | 测试目的 | シナリオ | テストデータ |
| --- | --- | --- | --- | --- |
| TST-ST-04-100 | [E2E] | NFR-SE-001 Lv.4 权威 | — | — |
| TST-ST-04-101 | [E2E] | NFR-SE-002 Lv.4 TLS | — | — |
| TST-ST-04-102 | [E2E] | ARC-015 协议演进 | — | — |
| TST-ST-04-103 | [E2E] | NFR-OP-006 Lv.4 混合 | — | — |

## 3.4 重点验证

| 用例 ID | 试验级别 | VF | シナリオ | テストデータ |
| --- | --- | --- | --- | --- |
| TST-ST-04-200 | [E2E] | VF-015 三引擎一致 | — | — |
| TST-ST-04-201 | [E2E] | VF-009 滚动更新 | — | — |

## 3.5 验收标准

| AC | 判定 |
|---|---|
| AC-001 垂直闭环 | 涉及 SDK |
| AC-011 滚动 0 中断 | VF-009 |
| AC-015 OSI 100% | SDK 全部依赖 |
| AC-019 AC-SDK/CDN | 全部 |

---

## 3.6 ST 特定增量: 8 域扩展 (scene/battle/network/account/sub8) + 9 域 mTLS 业务级 (v0.2 新增)

per 9/6 闪烁之光 8 域扩展 (NEW 8 域) + 9/6 d270ab9 9 域 mTLS 业务级 11 步 v3:

| 用例 ID | 试验级别 | 场景 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-8D-001 | [E2E] | scene-service gRPC 业务级 | scene 域 148 RPC (per 57edbeb) | 57edbeb |
| TST-ST-04-8D-002 | [E2E] | battle-service gRPC 业务级 | battle 域 250 RPC (per b6b19b7) | b6b19b7 |
| TST-ST-04-8D-003 | [E2E] | network-gateway 协议网关 | network 域 (per 1dd9afc) | 1dd9afc |
| TST-ST-04-8D-004 | [E2E] | account-service 账号 | account 域 + 8 子系统 (sub8, per 3c79bca) | 3c79bca / 42df673 |
| TST-ST-04-8D-005 | [E2E] | 8 域扩展 SDK 客户端适配 | 客户端 SDK 支持 8 域 gRPC 调用 | 42df673 L18 113+43 RPC 补全 |

## 3.7 ST 特定增量: 9 域 mTLS 业务级 E2E 11 步 v3 (v0.2 新增)

per 9/6 d270ab9 9 域 mTLS 端到端 11 步客户端模拟器 v3:

| 用例 ID | 试验级别 | 步骤范围 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-MTLS-9D-001 | [TL-8] | 步骤 1-3: player → economy → match gRPC mTLS 握手 + 业务调用 | 3/3 域 mTLS 业务级 OK | d270ab9 + d15a0bb 3 NEW 域 k8s yaml |
| TST-ST-04-MTLS-9D-002 | [TL-8] | 步骤 4-6: social → admin → batch gRPC mTLS 握手 + 业务调用 | 3/3 域 mTLS 业务级 OK | d270ab9 + 3c79bca batch 6 域注册 |
| TST-ST-04-MTLS-9D-003 | [TL-8] | 步骤 7-9: scene / battle / network (8 域扩展 NEW) gRPC mTLS | 3/3 域 mTLS 业务级 OK | 57edbeb / b6b19b7 / 1dd9afc 8 域扩展 |
| TST-ST-04-MTLS-9D-004 | [TL-8] | 步骤 10-11: 9 域跨域 saga mTLS 业务级 E2E | 11/11 步 PASS, 0 mTLS 握手失败, 0 业务错 | d270ab9 + 3ce36f0 (RGS-TEST-CASES v0.2) |

## 3.8 ST 特定增量: batch v0.1 6 module × 15 用例 (v0.2 新增)

per 9/1 batch 4 件套 (REQ + BASIC + DETAILED + PLAN) + 9/2 v0.1 FREEZE + 9/3 v0.2 EVAL:

| 用例 ID | 试验级别 | module | 场景数 | 测试目的 | evidence |
| --- | --- | --- | --- | --- | --- |
| TST-ST-04-BATCH-001 | [E2E] | rgs-batch-backend::cron | 3 | 定时任务调度 + worker_pool 1 个 task 完成 + audit_log 1 条 | F-1 |
| TST-ST-04-BATCH-002 | [E2E] | rgs-batch-backend::task_templates | 2 | 模板版本化 2 版本共存 | GAP-8 |
| TST-ST-04-BATCH-003 | [E2E] | rgs-batch-backend::worker_pool | 2 | 多 worker 并发 100 task, 100/100 完成 | F-4 |
| TST-ST-04-BATCH-004 | [E2E] | rgs-batch-backend::audit_logger | 2 | 操作人/时间/参数 hash/结果/trace_id 永久保留 (NFR-29 T-3) | F-10 |
| TST-ST-04-BATCH-005 | [E2E] | rgs-batch-backend::dlq | 3 | dead-letter 队列, 失败 3 次入 DLQ, payload 完整 | F-9 |
| TST-ST-04-BATCH-006 | [E2E] | rgs-batch-backend::connector | 3 | 5 域 gRPC client (mTLS 业务级, 5 域 5/5 OK) | NFR-32 |

**总计**: 6 module × 15 用例 = 90 用例 (跟 RGS-TEST-DESIGN v0.2 §7.5 BATCH-001~006 一致)

## 3.9 ST 特定增量: admin-coc §X 集成设计 (v0.2 新增)

per 9/5 ae9702d admin-coc Phase B DDD Review + 9/5 21:17 JST 7 项 admin 域 Lead 真实签字:

| 用例 ID | 试验级别 | 场景 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-ADMIN-COC-001 | [E2E] | admin-coc §X GM 命令 coc_policy 决策树 | 7 项 admin 域 Lead 真实签字 | ae9702d / 3695f3b |
| TST-ST-04-ADMIN-COC-002 | [E2E] | coc_policy 决策树 3 场景 | 1101/1102/1103 错误码 | 3695f3b coc_policy UT |

## 3.10 ST 特定增量: plugin 集群 4 阶段用例 (v0.2 新增)

per 9/5 61cf306 RGS plugin 集群 + app 集群架构 v0.1 + 9/5 ARCH §4 4 阶段用例:

| 用例 ID | 试验级别 | 阶段 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-PLUGIN-001 | [E2E] | 阶段 0 mock (已完成) | PoC plugin 抽卡概率 hot-swap 不重启 app | 5cfd692 + 9/5 ARCH §4.1 |
| TST-ST-04-PLUGIN-002 | [E2E] | 阶段 1 MVP (~2-3 周) | PG registry 持久化 + 9 域共享 | 9/5 ARCH §4.2 |
| TST-ST-04-PLUGIN-003 | [E2E] | 阶段 2 (~3-4 周) | 每 app 独立更新 + 双 registry 模式 | 9/5 ARCH §4.3 |
| TST-ST-04-PLUGIN-004 | [E2E] | 阶段 3 平台化 (~4-6 周) | 平台化 + 完整 WBS 5-10 task | 9/5 ARCH §4.4 |

## 3.11 ST 特定增量: rgs-flash-mock v0.3 + 4 NEW 回归脚本 (v0.2 新增)

per 9/4 RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 + 9/6 W3 5 报告:

| 用例 ID | 试验级别 | 脚本 | 覆盖范围 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-MOCK-001 | [E2E] | regression-test-9-domain-mtls.sh | 9 域 mTLS 业务级 11 步 | d270ab9 + 9/6 v0.3 |
| TST-ST-04-MOCK-002 | [E2E] | regression-test-batch-domain.sh | batch 域 6 module × 15 用例 | 9/1 batch 4 件套 |
| TST-ST-04-MOCK-003 | [E2E] | regression-test-8-domain-extension.sh | 8 域扩展 12 module | 9/6 8 域扩展 |
| TST-ST-04-MOCK-004 | [E2E] | regression-test-admin-coc.sh | admin-coc Phase B 7 项 + coc_policy 3 场景 | ae9702d / 3695f3b |

---

## 4. 追溯性矩阵

| 需求 | 用例范围 |
|---|---|
| REQ-012 SDK | TST-ST-04-001~015 |
| REQ-030 CDN | TST-ST-04-020~038 |
| NFR | TST-ST-04-100~103 |
| VF | TST-ST-04-200~201 |
| AC | 全部 |

---

## 4.2 AC-001~019 跨主题追溯矩阵

本主题 ST 测试设计书对全部 19 项验收标准（AC-001~019）的追溯：

| AC | 描述 | 涉及本主题？ | 本主题对应用例 | 跨主题引用 |
|---|---|---|---|---|
| AC-001 | 见 RGS-TST-ST-00 §3.2.1 | → | — | 跨主题 |
| AC-002 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-003 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-004 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-005 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-006 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-007 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-008 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-009 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-010 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-011 | 见 RGS-TST-ST-01 §3.3 | → | — | 跨主题 |
| AC-012 | 见 RGS-TST-ST-02 §3.2 | → | — | 跨主题 |
| AC-013 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-014 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-015 | 本主题 §3.1 SDK 三引擎一致 + OSI | ✓ | — | 跨主题 |
| AC-016 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-017 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-018 | 见 RGS-TST-ST-00 §3.5 通用映射 | → | — | 跨主题 |
| AC-019 | 见 RGS-TST-ST-00 §3.5 | → | — | 跨主题 |

**判定规则**：
- 涉及：本主题域有直接测试用例 → 列出
- 不涉及：跨主题引用其他 ST 文档 → 引用链接


## 5. 测试执行计划

| 阶段 | 用例 | 工具 |
|---|---|---|
| PH-1 | 001-015 | Unity/UE/Bevy + k6 |
| PH-6 | 020-038 | MinIO + k6 |
| PH-8 | 全部 | 全部 + LitmusChaos |

## 6. 通过判定基准

- AC-SDK-001~003 通过
- AC-CDN-001~006 通过
- NFR-SDK/CDN 全部达标
- VF-015 三引擎一致


## 6.5 NFR 覆盖索引

本主题域覆盖的非功能需求编号全集（按 RGS-REQ-003 等级 Lv.2/3/4 全覆盖）：

- **NFR-CDN-***：NFR-CDN-001, NFR-CDN-002, NFR-CDN-003, NFR-CDN-004, NFR-CDN-005
- **NFR-OP-***：NFR-OP-006
- **NFR-SDK-***：NFR-SDK-001, NFR-SDK-002, NFR-SDK-003, NFR-SDK-004
- **NFR-SE-***：NFR-SE-001, NFR-SE-002


## 6.6 ADR 决策验证（本主题）

本主题涉及的 ADR 决定项的"实现位置 + 测试位置 + 守门位置"是否完备：

| ADR 编号 | 决定项摘要 | 实现位置 | 测试位置（本文档） | 守门位置 |
|---|---|---|---|---|
| RGS-ADR-0023 | 客户端核心逻辑单一实现，多引擎薄适配层 | DTL-008 §3 核心 SDK | 本主题 TST-ST 对应模块 | CI 静态检查 |
| RGS-ADR-0044 | 客户端资源分发默认自托管开源 | DTL-027 §10 自托管 | 本主题 TST-ST 对应模块 | CI 静态检查 |

## 6.7 派生约束守护 (v0.2 增补, per AGENTS.md + 9/2 D2 拍板 + 9/1 batch 12 派生约束)

| 派生约束 | 内容 | v0.2 状态 |
|---|---|---|
| **L1** | cargo check --tests 0 error (限时 60s) | ✅ 文档类工作 N/A |
| **L11** | cargo build dir lock (PT 派工 1 次拿 status) | ✅ N/A (文档) |
| **L12.1** | 临时 log / .txt / .tmp_search* 不入 commit | ✅ worktree 根 0 临时文件 |
| **L12.2** | 5 worker 派工 3 选项 | ✅ 1 worker 1 worktree v02/st5 |
| **L13** | 自指字段 deferred 实时查询 (git log + grep 实证) | ✅ v0.2 全文 git log 实证 |
| **L14** | plumbing 节点字符串 brace 跟踪 | ✅ N/A (文档) |
| **B3** | DDD Review 二审流程 (per 9/2 10:18 JST 拍板) | ⏳ Mavis 自审 1 次停手 → Ulysses 二审必到 |
| **5 域独立 Lead** | per 2026-08-21 JST | ✅ 维持, 扩展 6 域 (5 + batch) |
| **凭据永不打印** | per 8/27 11:06 JST 硬 ban | ✅ 全文 0 env value |
| **缺标比错标** | per 8/26 JST DTL-036 v1.4 hotfix 复盘 | ✅ §6.8 已知缺口 6 项显式列 |
| **不追溯改写** | per 8/27 JST 禁回溯叙事 | ✅ v0.1 → v0.2 显式升版, 不 amend 历史 |
| **Mavis 默认代签 Ulysses** | per 8/27 19:39/20:56/21:59 JST 三次强化 | ✅ author / 审批 / 修订人 三行齐全 |
| **9/1 batch 域 12 派生约束** | per AGENTS.md §7.2 | ✅ §3.8 batch v0.1 6 module × 15 用例 |
| **9/1 14:58 JST 拍板选项规则** | ask_user 给 Ulysses 选项, 不能直接做 | ✅ v0.2 拍板走 ask_user |
| **9/4 17:47 JST 测试脚本+数据归入 mock 项目** | per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 | ✅ §3.11 4 NEW 回归脚本 |
| **9/5 04:03 JST 拍板后立即执行** | 推荐项被选后立即执行 | ✅ 拍板后直接落地 |
| **cutover L15-L23** | per 9/6 6c6839e | ✅ 全文引用 (8 维度增量表) |
| **L15** | native binary 跨工具链 → file ELF | ✅ N/A (文档) |
| **L16** | 主会话统一 commit 拍板顺序 | ✅ v02/st5 主会话统一落地 |
| **L17** | InMemory 5 域 → PgRepository 7 域扩展 | ✅ §0 8 域扩展 (7 域含 batch) |
| **L18** | 113+43 RPC 补全 (1351 codegen) | ✅ §3.6 8 域扩展 SDK 客户端适配 |
| **L19** | mTLS 业务级 = saga 触达 | ✅ §3.7 9 域 mTLS 11 步 v3 |
| **L20** | ca.crt 0 字节空文件陷阱 | ✅ §3.7 evidence d270ab9 |
| **L21** | 跨工具链 gRPC Code 解析判据 | ✅ N/A (文档) |
| **L22** | 协议码 → gRPC method 映射表 | ✅ §3.6 8 域 RPC 映射 |
| **L23** | 4 层自动探针 | ✅ N/A (文档) |

## 6.8 已知缺口 (v0.2 增补, per 8/26 JST 缺标比错标 6 项)

1. **L3 drill LCM/chaos/NFR/risk 4 类**: cluster-ops/tests-disabled/ 仍禁用, 需 INC-002 saga 编译死锁修复后重启用
2. **batch v0.2 评估 12 GAP**: per RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 GAP-1~12 — v0.2 不集成, v0.2 评估期补全
3. **RACI v1.1 → v1.2**: 5 域扩展 6 域 (5 + batch) 待 DDD Review 阶段补
4. **plugin PoC 1 个 WASM (draw_card_probability)**: per 9/5 5cfd692 已落 v0.1 PoC, 阶段 1 MVP 详细 WBS 5-10 task 待 DDD Review 阶段补
5. **9 域 mTLS 业务级详细 yaml**: per 9/6 d15a0bb 3 NEW 域 yaml 落档, 剩余 6 域 yaml 待 DDD Review 阶段补
6. **ST 业务级 E2E 100k CCU 性能**: ST §4 NFR-PE-001~006 待 100k 阶段实跑

## 7. 风险与未决事项

| ID | 内容 | 处理 |
|---|---|---|
| TBD-SDK-001 | 绑定工具 | 留 |
| TBD-CDN-002 | 分发后端 | 留 |

---

> 本文档为 RGS-TST 系列主题 04 系统测试设计书。

## 7.5 TBD 处置

本主题涉及的 TBD 处置方式：

| TBD 编号 | 描述 | 处置 |
|---|---|---|
| TBD-SDK-001 | 绑定生成工具（cbindgen 提案） | PH-1 决定 |
| TBD-CDN-002 | 自托管分发后端具体实现 | PH-2 决定 |

