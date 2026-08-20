# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 01 核心架构与设计模式 — 集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-01 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-001/002/010/022/023/024 基本设计书 |
| V模型层级 | TL-2 集成 / TL-3 契约 / TL-4 属性 / TL-5 状态机 ↔ BAS 基本设计 |
| 依据标准 | IPA『共通フレーム 2013』基本設計工程 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师（自动化产出 + 字段级深化） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 本主题域源文档全集（REQ/BAS/DTL） | RGS-REQ-006, RGS-REQ-025, RGS-REQ-026, RGS-REQ-027, RGS-BAS-001, RGS-BAS-002, RGS-BAS-010, RGS-BAS-022, RGS-BAS-023, RGS-BAS-024, RGS-DTL-001, RGS-DTL-002, RGS-DTL-022, RGS-DTL-023, RGS-DTL-024 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定
| **0.2** | 2026-08-19 | 架构师 | **字段级深化**：每条用例的"对应设计"列升级为"文档 ID + §X.Y + 表/图/字段"；新增"ADR 决策验证"小节覆盖本主题 ADR；新增"TBD 处置"小节 |。覆盖 5 服务协议、跨库主键、挂载脚手架、容量分片、请求处理链、集群部署 |
| 0.2 | 2026-08-19 | 架构师 | **字段级深化**：每条用例精确引用 BAS §X.Y + 接口字段；新增 §3.10 ADR 决策验证；新增 §6.5 NFR 覆盖索引；新增 §7 TBD 处置 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构） | | | BAS 接口契约一致性 |
| 评审（QA） | | | 集成覆盖率与 TL-3 契约 |
| 审批（负责人） | | | 本测试设计书的基准化 |

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


## 1. 前言

## 1.1 目的（目的 / Purpose）

TL-2/TL-3/TL-4/TL-5 层级，对应主题 01 的 6 份基本设计书。本版本（0.2）相比 0.1 的升级：

- **字段级映射**：每条用例的"对应设计"列升级为"BAS-XXX §X.Y + 接口/方法/字段"
- **ADR 决策验证**：§3.10 验证 10 份 ADR 在集成层级的实施
- **NFR 覆盖索引**：§6.5 列出全部 NFR 编号
- **TBD 处置**：§7 给出每条 TBD 在 IT 阶段的处理

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-BAS-001 §3-§7 | 部署/5 服务/数据/API/非功能 | IT 验证对象 |
| RGS-BAS-002 §3-§12 | 挂载脚手架 | IT 验证对象 |
| RGS-BAS-010 §3 | 设计模式/算法 | IT 验证对象 |
| RGS-BAS-022/023/024 | 容量/请求链/部署 | IT 验证对象 |
| RGS-ADR-0001/0002/0007/0008/0015/0020/0022/0023/0026/0029 | 架构决策 | §3.10 验证 |

**本主题域源文档全集**：
- REQ: RGS-REQ-006, RGS-REQ-025, RGS-REQ-026, RGS-REQ-027
- BAS: RGS-BAS-001, RGS-BAS-002, RGS-BAS-010, RGS-BAS-022, RGS-BAS-023, RGS-BAS-024
- DTL: RGS-DTL-001, RGS-DTL-002, RGS-DTL-022, RGS-DTL-023, RGS-DTL-024

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
- 运行时机：`cargo test --workspace`（主干 CI 必跑，QA-006 ≤ 15 min 约束内）


## 2. 测试策略

## 2.1 V 模型对应关系

```
需求   RGS-REQ-006/025/026/027  ┐ ST
                                │
基础   RGS-BAS-001/002/010/022/023/024  ┐ IT  ★ RGS-TST-IT-01 ★
                                │
详细   RGS-DTL-001/002/022/023/024  ┐ UT  RGS-TST-UT-01
                                │
实现   Rust 源码              ┘
```

## 2.2 集成层级

| 层级 | 描述 | 本主题示例 |
|---|---|---|
| L1 | crate 内多模块 | 5 服务同进程 |
| L2 | workspace 内多 crate | services/* 跨 crate |
| L3 | 进程间 gRPC/HTTP | RT → EconomyService |
| L4 | 跨服务 + DB | EconomyService → economy_db |
| L5 | 跨服务 + 外部依赖 | 挂载脚手架 → 外部 MySQL |

## 2.3 契约试验（TL-3）

- gRPC：`tonic-build` 生成的 client stub + 兼容性脚本
- 事件：apicurio-registry 兼容性 API
- DB：sqlx migration + Expand-Contract 验证
- Helm chart：`helm template` + 静态分析

## 2.4 覆盖率策略

| 维度 | 目标 |
|---|---|
| 接口契约覆盖率 | 100%（全部 public API + 事件 topic） |
| 集成路径覆盖率 | ≥ 70%（跨模块主路径） |
| 缺陷密度 | ≤ 1.0 件/KLOC（QA-004） |
| 属性不变条件 | BZ-001〜007 各 1 件以上（QA-002） |
| 状态机非法迁移 | 全部拒绝（QA-003） |

---

## 3. 测试用例

## 3.1 模块 A：5 服务协议集成（RGS-BAS-001 §4）

| 用例 ID | 对应设计 | 字段级 | 试验级别 | 测试目的 |
|---|---|---|---|---|
| TST-IT-01-A001 | BAS-001 §4.4 PlayerService | request{device_id, client_version} → response{account_id, character_id} | [TL-2] | 端到端 |
| TST-IT-01-A002 | BAS-001 §4.4 PlayerService gRPC | schema_version 字段 | [TL-3] | 契约兼容 |
| TST-IT-01-A003 | BAS-001 §4.5 EconomyService.Determine | request_id UUID, session_epoch BIGINT | [TL-2] | 端到端 |
| TST-IT-01-A004 | BAS-001 §4.5 EconomyService gRPC | v1 client 调 v2 server | [TL-3] | 兼容性 |
| TST-IT-01-A005 | BAS-001 §4.5 RT → Economy | gRPC async over mTLS | [TL-2] | 跨进程 |
| TST-IT-01-A006 | BAS-001 §4.5 RT → Player | gRPC session_epoch 写入 | [TL-2] | 跨进程 |
| TST-IT-01-A007 | BAS-001 §4.7 跨服务事件 | event.schema_version, event_id | [TL-3] | 契约 |
| TST-IT-01-A008 | BAS-001 §5.1 跨库主键 | UUID↔BIGINT 双向 | [TL-2] | 跨 DB |
| TST-IT-01-A009 | BAS-001 §4.4-§4.5 5 服务并发 | 1000 并发 | [TL-2] | 性能冒烟 |
| TST-IT-01-A010 | BAS-001 §8 ST-001 跨服务 | 状态机非法迁移 | [TL-5] | 状态机 |

## 3.2 模块 B：挂载脚手架集成（RGS-BAS-002 §4-§12）

| 用例 ID | 对应设计 | 字段级 | 试验级别 | 测试目的 |
|---|---|---|---|---|
| TST-IT-01-B001 | BAS-002 §4 cargo-generate | template.git 字段 | [TL-2] | 端到端 |
| TST-IT-01-B002 | BAS-002 §4 Helm chart 4 模板 | Deployment.replicas | [TL-3] | 渲染正确 |
| TST-IT-01-B003 | BAS-002 §5 DB provisioning | SQL 脚本重入 | [TL-2] | 幂等 |
| TST-IT-01-B004 | BAS-002 §5 Mount Record | frontmatter.id 唯一 | [TL-2] | 写入 |
| TST-IT-01-B005 | BAS-002 §6 挂载检查清单 | 13 项 | [TL-2] | 全部 PASS |
| TST-IT-01-B006 | BAS-002 §6.5 排他锁 | uq_deploy_runs_cluster_running | [TL-2] | 同 cluster 并发仅 1 成功 |
| TST-IT-01-B007 | BAS-002 §6 网关路由 | routes[] 字段 | [TL-2] | 路由登记 |
| TST-IT-01-B008 | BAS-002 §6 事件 Schema | schema_version 必填 | [TL-2] | 事件登记 |
| TST-IT-01-B009 | BAS-002 §6 OTel | service.name 字段 | [TL-2] | 可观测接入 |
| TST-IT-01-B010 | BAS-002 §7 灰度发布 | canary_weight 字段 | [TL-2] | 切 1%/10%/100% |
| TST-IT-01-B011 | BAS-002 §7 流量回退 | 路由权重置 0 | [TL-2] | p99<10s |
| TST-IT-01-B012 | BAS-002 §8 退场 | lifecycle_state=Decommissioning | [TL-2] | 端到端 |
| TST-IT-01-B013 | BAS-002 §8 退场幂等 | re_decommission | [TL-2] | Ok(AlreadyGone) |
| TST-IT-01-B014 | BAS-002 §10 ARC-018 5 要素 | DB+部署+gRPC+事件+可观测 | [TL-2] | 全部具备 |
| TST-IT-01-B015 | BAS-002 §11 NFR-MNT-002 | error_rate_increment | [TL-2] | ≤10% 阈值 |
| TST-IT-01-B016 | BAS-002 §11 NFR-MNT-003 | DB 故障注入 | [TL-2] | 不影响既有 |
| TST-IT-01-B017 | BAS-002 §11 NFR-MNT-006 | Helm chart 同源 | [TL-3] | 模板漂移检测 |

## 3.3 模块 C：设计模式与算法（RGS-BAS-010 §3）

| 用例 ID | 对应设计 | 测试目的 | 试验级别 |
|---|---|---|---|
| TST-IT-01-C001 | BAS-010 §3.1 Strategy/State | GoF 模式端到端 | [TL-2] |
| TST-IT-01-C002 | BAS-010 §3.2 Outbox/CQRS | 架构模式 | [TL-2] |
| TST-IT-01-C003 | BAS-010 §3.3 G-001 OCC | 冲突检测 | [TL-2] |
| TST-IT-01-C004 | BAS-010 §3.3 G-002 Idempotency | 1000 次 | [TL-4] |
| TST-IT-01-C005 | BAS-010 §3.9 Deterministic Gate | 智能层确定性 | [TL-2] |
| TST-IT-01-C006 | BAS-010 §3.9 Dual-Mode OLU | 双模式切换 | [TL-2] |
| TST-IT-01-C007 | BAS-010 §3.10 反模式 | 静态分析 | [TL-2] |

## 3.4 模块 D：弹性容量分片（RGS-BAS-022 §3-§5）

| 用例 ID | 对应设计 | 字段级 | 试验级别 | 测试目的 |
|---|---|---|---|---|
| TST-IT-01-D001 | BAS-022 §3.1 一致性 hash | shard_id 字段 | [TL-2] | 8 shard 路由一致 |
| TST-IT-01-D002 | BAS-022 §3.2 T0 5万 | capacity_tier 字段 | [TL-2] | 部署正确 |
| TST-IT-01-D003 | BAS-022 §3.2 T1 20万 | 同上 | [TL-2] | 同上 |
| TST-IT-01-D004 | BAS-022 §3.2 T2 100万 | 同上 | [TL-2] | 同上 |
| TST-IT-01-D005 | BAS-022 §3.2 T3 1000万 | 同上 | [TL-2] | 同上 |
| TST-IT-01-D006 | BAS-022 §3.3 跨分片 API | list_cross_shard | [TL-3] | 契约 |
| TST-IT-01-D007 | BAS-022 §4.1 HPA | metrics_server URL | [TL-2] | 触发 |
| TST-IT-01-D008 | BAS-022 §4.2 预测预热 | predict_window 字段 | [TL-2] | 流量预判 |
| TST-IT-01-D009 | BAS-022 §5 NFR-CAP-001~005 | 跨分片 BZ-001 | [TL-4] | 守恒 |
| TST-IT-01-D010 | BAS-022 §5 NFR-CAP-002 | 突发流量 | [TL-2] | 不中断 |

## 3.5 模块 E：请求处理链（RGS-BAS-023 §3-§5）

| 用例 ID | 对应设计 | 字段级 | 试验级别 | 测试目的 |
|---|---|---|---|---|
| TST-IT-01-E001 | BAS-023 §3.1 全部服务接入 | 5/5 接入 | [TL-2] | 端到端 |
| TST-IT-01-E002 | BAS-023 §3.2 鉴权失败短路 | 401 | [TL-2] | 不进业务 |
| TST-IT-01-E003 | BAS-023 §3.3 限流 | 100/秒 | [TL-2] | 跨服务 |
| TST-IT-01-E004 | BAS-023 §3.4 幂等 | idempotency_key | [TL-2] | 跨服务 |
| TST-IT-01-E005 | BAS-023 §4 字段级脱敏 | password→*** | [TL-2] | 跨服务 |
| TST-IT-01-E006 | BAS-023 §5 错误格式 | {code, message} | [TL-2] | 5 服务统一 |
| TST-IT-01-E007 | BAS-023 §6 脚手架强制 | 新 App 自动接入 | [TL-2] | 挂载 |
| TST-IT-01-E008 | BAS-023 §6 旁路禁止 | bypass 拒绝 | [TL-2] | 安全合规 |

## 3.6 模块 F：集群部署编排（RGS-BAS-024 §3-§10）

| 用例 ID | 对应设计 | 字段级 | 试验级别 | 测试目的 |
|---|---|---|---|---|
| TST-IT-01-F001 | BAS-024 §3 清单 | cluster_name, apps[] | [TL-2] | 解析 |
| TST-IT-01-F002 | BAS-024 §3 | 缺 cluster_name | [TL-2] | 拒 |
| TST-IT-01-F003 | BAS-024 §4 DAG | 拓扑排序 | [TL-2] | 正确 |
| TST-IT-01-F004 | BAS-024 §4 | A→B→A 循环 | [TL-2] | Err(Cycle) |
| TST-IT-01-F005 | BAS-024 §5 状态机 | run_id 字段 | [TL-2] | 完整迁移 |
| TST-IT-01-F006 | BAS-024 §6 排他约束 | uq_deploy_runs_cluster_running | [TL-2] | 同 cluster 1 RUNNING |
| TST-IT-01-F007 | BAS-024 §7 dry-run | --dry-run 字段 | [TL-2] | 不真部署 |
| TST-IT-01-F008 | BAS-024 §8 幂等编排 | resume_token | [TL-2] | 中断续 |
| TST-IT-01-F009 | BAS-024 §8 回滚 | --rollback 字段 | [TL-2] | 失败回滚 |
| TST-IT-01-F010 | BAS-024 §9 联动脚手架 | new_app 字段 | [TL-2] | 入清单 |
| TST-IT-01-F011 | BAS-024 §10 跨 cluster | cluster_name 多值 | [TL-2] | 编排 |
| TST-IT-01-F012 | BAS-024 §10 工具无关 | CLI/Helm 字段 | [TL-3] | 同步 |

## 3.7 跨服务业务规则（TL-4）

| 用例 ID | 业务规则 | 试验级别 | 跨服务 |
|---|---|---|---|
| TST-IT-01-100 | BZ-001 货币非负 | [TL-4] | 5 服务 |
| TST-IT-01-101 | BZ-002 支付幂等 | [TL-4] | 跨服务 |
| TST-IT-01-102 | BZ-003 流水复原 | [TL-4] | 跨服务 |
| TST-IT-01-103 | BZ-007 交易原子性 | [TL-4] | 跨 Saga |
| TST-IT-01-104 | ARC-005 epoch | [TL-4] | 跨服务 |
| TST-IT-01-105 | ARC-007 实时不同步 DB | [TL-4] | 跨服务 |
| TST-IT-01-106 | ARC-009 OCC | [TL-4] | 跨服务 |

## 3.8 跨服务状态机（TL-5）

| 用例 ID | 状态机 | 试验级别 |
|---|---|---|
| TST-IT-01-110 | ST-001 跨服务 | [TL-5] |
| TST-IT-01-111 | ST-002 跨服务 | [TL-5] |
| TST-IT-01-112 | ST-003 跨 Saga | [TL-5] |
| TST-IT-01-113 | ST-005 跨服务 | [TL-5] |

## 3.9 ARC-013 死锁防止跨服务（BAS-001 §7.2.1）

| 用例 ID | 测试目的 | 试验级别 |
|---|---|---|
| TST-IT-01-120 | A→B 不形成同步循环 | [TL-2] |
| TST-IT-01-121 | 优先级不同独立通道 | [TL-2] |
| TST-IT-01-122 | 全部 mailbox bound | [TL-2] |

## 3.10 模块 J：ADR 决策验证（集成层级）

| 用例 ID | ADR | 集成层验证 | 试验级别 |
|---|---|---|---|
| TST-IT-01-K001 | ADR-0001 Actor 粒度 | RT 进程崩溃后 EconomyService gRPC 仍接受 RT-001 重连 epoch | [TL-2] |
| TST-IT-01-K002 | ADR-0002 状态同步 | 客户端 SDK 与 RT 端到端协议字段一致 | [TL-2] |
| TST-IT-01-K003 | ADR-0007 道具货币统合 | inventory + wallet 同 schema | [TL-2] |
| TST-IT-01-K004 | ADR-0008 中间件判定 | 新中间件无 ADR 被 CLI 拒绝 | [TL-2] |
| TST-IT-01-K005 | ADR-0015 Saga 边界 | 实时路径 gRPC 不调工作流 | [TL-2] |
| TST-IT-01-K006 | ADR-0020 拒绝动态库 | dlopen 调用被拒 | [TL-2] |
| TST-IT-01-K007 | ADR-0022 业务逻辑不入库 | 存储过程仅 CRUD | [TL-2] |
| TST-IT-01-K008 | ADR-0023 客户端核心逻辑 | 三引擎同输入 | [TL-2] |
| TST-IT-01-K009 | ADR-0026 智能层只读 | 不写经济 | [TL-2] |
| TST-IT-01-K010 | ADR-0029 确定性闸门 | L4 写必经 | [TL-2] |

---

## 4. 追溯性矩阵（精修版）

| 基本设计章节 | 字段级 | 用例 ID 范围 | 试验级别 |
|---|---|---|---|
| BAS-001 §4.4 PlayerService | 11 字段 | TST-IT-01-A001~002 | TL-2/3 |
| BAS-001 §4.5 EconomyService | 9 字段 | TST-IT-01-A003~006 | TL-2/3 |
| BAS-001 §4.5 RT | gRPC mTLS | TST-IT-01-A005~006 | TL-2 |
| BAS-001 §4.7 事件 | schema_version | TST-IT-01-A007 | TL-3 |
| BAS-001 §5.1 跨库 | UUID↔BIGINT | TST-IT-01-A008 | TL-2 |
| BAS-001 §4.4-§4.5 5 服务 | 11 方法 | TST-IT-01-A009 | TL-2 |
| BAS-001 §7.2.1 死锁 | 方向性 | TST-IT-01-120~122 | TL-2 |
| BAS-001 §8 状态机 | 4 状态机 | TST-IT-01-110~113 | TL-5 |
| BAS-002 §4 挂载 | 5 要素 | TST-IT-01-B001~017 | TL-2/3 |
| BAS-010 §3 模式 | 13 算法 | TST-IT-01-C001~007 | TL-2/4 |
| BAS-022 §3-§5 容量 | 4 档 T0~T3 | TST-IT-01-D001~010 | TL-2/3/4 |
| BAS-023 §3-§5 请求链 | 8 阶段 | TST-IT-01-E001~008 | TL-2 |
| BAS-024 §3-§10 集群 | 12 字段 | TST-IT-01-F001~012 | TL-2/3 |
| BZ-* 跨服务 | 7 条 | TST-IT-01-100~106 | TL-4 |
| ST-* 跨服务 | 5 条 | TST-IT-01-110~113 | TL-5 |
| ADR 决定项 | 10 条 | TST-IT-01-K001~010 | TL-2 |
| AC-004 全部禁止迁移 | — | TST-IT-01-110~113 | 跨主题 |
| AC-019 领域验收 | — | 全部 | 跨主题 |

---

## 5. 测试执行计划

| 触发 | 范围 | 时限 |
|---|---|---|
| 每次 PR 推送 | 受影响 crate 的 L1/L2 集成 | < 8 min |
| 每次合并至 main | L1〜L4 全部集成 + TL-3 契约 | < 12 min（QA-006 内） |
| 每晚 nightly | L5 外部依赖 Mock + 10000 次属性迭代 | 不阻塞主干 |
| 每次领域文档变更 | 跨域引用一致性 + 治理 CI 校验 | < 3 min |

## 5.1 测试环境

| 组件 | 配置 |
|---|---|
| PostgreSQL | ephemeral container, postgresql:16-alpine |
| Redis（缓存） | ephemeral container, redis:7-alpine |
| paymock | RGS-BAS-012 §4 mock 服务 |
| 治理 CI | git + ephemeral runner |
| k6（按需） | RGS-BAS-012 §5 |

## 5.2 覆盖率门禁

- 接口契约覆盖率：100%
- 集成路径覆盖率：≥ 70%
- 缺陷密度：≤ 1.0 件/KLOC（QA-004）
- 业务规则属性：proptest 默认 1000 次迭代无失败
- 跨服务状态机：100% 非法迁移被拒
- 静态检查：`cargo clippy --all-targets -- -D warnings` 通过
- 跨语言集成：k6 ↔ gov-ci 通信正常

---

## 6. 通过判定基准

| 维度 | 基准 |
|---|---|
| 所有用例 PASS | TST-IT-01-A001~K010 全部通过 |
| 接口契约 | 100% 兼容（gRPC + 事件 + DB） |
| 集成路径 | ≥ 70% 覆盖 |
| 属性不变条件 | 1000 次迭代无失败 |
| 状态机 | 跨模块全部非法迁移被拒 |
| 缺陷密度 | ≤ 1.0 件/KLOC |
| 静态检查 | `cargo clippy --all-targets -- -D warnings` 通过 |
| 跨语言集成 | k6 ↔ gov-ci 通信正常 |

## 6.5 NFR 覆盖索引

本主题域覆盖的非功能需求编号全集（按 RGS-REQ-003 等级 Lv.2/3/4 全覆盖）：

- **NFR-AV-***：NFR-AV-001, NFR-AV-002, NFR-AV-007, NFR-AV-008
- **NFR-PE-***：NFR-PE-001, NFR-PE-002, NFR-PE-013, NFR-PE-014, NFR-PE-015, NFR-PE-016, NFR-PE-017, NFR-PE-018, NFR-PE-019
- **NFR-OP-***：NFR-OP-001, NFR-OP-002, NFR-OP-003, NFR-OP-004, NFR-OP-005, NFR-OP-006, NFR-OP-007, NFR-OP-008, NFR-OP-009, NFR-OP-010
- **NFR-MI-***：NFR-MI-001, NFR-MI-002, NFR-MI-003, NFR-MI-004, NFR-MI-005
- **NFR-SE-***：NFR-SE-001, NFR-SE-002, NFR-SE-003, NFR-SE-004, NFR-SE-005, NFR-SE-006, NFR-SE-007, NFR-SE-008, NFR-SE-009, NFR-SE-010, NFR-SE-011, NFR-SE-012
- **NFR-EN-***：NFR-EN-001, NFR-EN-002, NFR-EN-003, NFR-EN-004, NFR-EN-005
- **NFR-RT-***：NFR-RT-001, NFR-RT-005, NFR-RT-008, NFR-RT-009, NFR-RT-013
- **NFR-PL-***：NFR-PL-001, NFR-PL-002, NFR-PL-003, NFR-PL-004, NFR-PL-005, NFR-PL-006
- **NFR-EC-***：NFR-EC-001, NFR-EC-002, NFR-EC-003, NFR-EC-004, NFR-EC-005, NFR-EC-006, NFR-EC-007, NFR-EC-008
- **NFR-MT-***：NFR-MT-001, NFR-MT-002, NFR-MT-003
- **NFR-GD-***：NFR-GD-001, NFR-GD-002, NFR-GD-003
- **NFR-EV-***：NFR-EV-001, NFR-EV-002, NFR-EV-003, NFR-EV-004, NFR-EV-005, NFR-EV-006
- **NFR-WF-***：NFR-WF-001, NFR-WF-002, NFR-WF-003
- **NFR-OB-***：NFR-OB-001, NFR-OB-002, NFR-OB-003, NFR-OB-004, NFR-OB-005
- **NFR-AD-***：NFR-AD-001, NFR-AD-002, NFR-AD-003, NFR-AD-004, NFR-AD-005

## 7. TBD 处置

| TBD 编号 | 描述 | 处置 |
|---|---|---|
| TBD-CAP-001 | T3 多区域校准 | IT 用 8 shard 占位 |
| TBD-CAP-002 | 跨分片能力 | IT 仅测 list_cross_shard |
| TBD-PPL-001 | 限流算法 | 用 100/秒 |
| TBD-DEP-001 | Schema 校验实现 | 任意实现 |
| TBD-ADR-001 | 治理 CI 误判率 | 实测统计 |

---

> 本文档为 RGS-TST 系列主题 01 集成测试设计书（**字段级深化版 0.2**）。
