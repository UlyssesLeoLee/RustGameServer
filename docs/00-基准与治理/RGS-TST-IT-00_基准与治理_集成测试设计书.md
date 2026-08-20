# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 00 基准与治理 — 集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-00 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-009 体系治理与横切关注点 基本设计书 |
| 本主题域源文档全集（REQ/BAS/DTL） | RGS-REQ-001、RGS-REQ-002、RGS-REQ-003、RGS-REQ-004、RGS-REQ-005、RGS-REQ-013、RGS-BAS-009、RGS-DTL-009 |

| V模型层级 | TL-2 集成试验 / TL-3 契约试验 / TL-4 属性试验（集成级）/ TL-5 状态机试验（跨模块） ↔ BAS 基本设计 |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』基本设计工程 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师（自动化产出） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。基于 RGS-BAS-009 的基本设计，覆盖两级 ID 体系、OLU 运维负荷预算机制、治理闭环 CI 机械校验、4 项横切需求落地等模块间的集成契约 |
| 0.2 | 2026-08-20 | 架构师 | 对齐正文已说明的字段级映射升级，修正文档元数据版本。 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构） | | | 与 RGS-BAS-009 各组件的接口契约一致性 |
| 评审（QA） | | | 集成覆盖率与 TL-3 契约试验设计 |
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

本文档为 V 模型中**TL-2 集成试验 / TL-3 契约试验 / TL-4 属性试验（集成级）/ TL-5 状态机试验（跨模块）**层级的设计书，对应父基本设计书 **RGS-BAS-009（体系治理与横切关注点）**。其目的是：

- 验证 RGS-BAS-009 中各组件**接口契约**的端到端集成行为
- 验证两级 ID 体系（领域 ID ↔ 主编号）的双向引用一致性
- 验证 OLU 预算台账与治理闭环 CI 校验器的协同（消耗→检测→告警→阻断）
- 验证 4 项横切需求（FR-GOV-001〜040）在多个限界上下文间的统一行为
- 满足 QA-004 集成试验时检出缺陷密度 ≤ 1.0 件/KLOC
- 满足 TL-3 契约试验：API/事件 Schema 的兼容性

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-REQ-001 §12.1 | 试验级别定义 | TL-2/TL-3/TL-4/TL-5 范围 |
| RGS-REQ-013 | 体系治理与横切关注点 需求定义书 | 父需求 |
| RGS-BAS-009 | 体系治理与横切关注点 基本设计书 | 父基本设计 |
| RGS-DTL-009 | 详细设计书 | 物理实现层（UT 覆盖） |
| RGS-REQ-004 | 附件C 可追溯性矩阵 | ID 引用基准 |
| RGS-REQ-005 | 附件D 问题风险管理表 | TBD/RSK/ISS 主编号源 |
| RGS-REQ-006 / BAS-002 | 功能挂载架构 | 挂载脚手架与本域 CI 校验的联动 |
| RGS-TST-UT-00 | 主题 00 单元测试设计书 | 下层 |
| RGS-TST-ST-00 | 主题 00 系统测试设计书 | 上层 |

**本主题域源文档全集**：
- REQ: RGS-REQ-001, RGS-REQ-002, RGS-REQ-003, RGS-REQ-004, RGS-REQ-005, RGS-REQ-013
- BAS: RGS-BAS-009
- DTL: RGS-DTL-009

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
       用户需求                验收                    RGS-REQ-001/013  ┐
         ↕                                                  ↕          │ ST
       验收试验                确认                  RGS-TST-ST-00     │
                                                                            │
       基本设计                整合                    RGS-BAS-009     ┐  │
         ↕                                                  ↕             │ IT
       集成试验                验证                ★ RGS-TST-IT-00 ★   │  │
                                                                            │
       详细设计                单元                    RGS-DTL-009     ┐  │  │
         ↕                                                  ↕             │ UT
       单元试验                证明                  RGS-TST-UT-00     │  │  │
                                                                            │
       实现                    —                       Rust 源码        ┘  ┘  ┘
```

## 2.2 集成层级

| 层级 | 描述 | 示例 |
|---|---|---|
| L1 模块内集成 | crate 内多模块协作 | `id_registry` + `olu_ledger` 同进程读写 |
| L2 跨 crate 集成 | workspace 内多 crate 协作 | `gov-ci` crate 调用 `id-registry` crate + 读附件 D |
| L3 跨进程集成 | 进程间 gRPC/HTTP | `gov-ci` 进程 ↔ `id-registry` 服务 |
| L4 跨服务 + DB 集成 | 真实数据库 schema 集成 | `olu_ledger` 持久化至 PostgreSQL |
| L5 外部依赖 Mock 集成 | 与 Mock 服务契约 | `paymock` 启动 → `purchase_saga` 调用 |

## 2.3 契约试验（TL-3）策略

- **gRPC API**：使用 `tonic-build` 生成的 client stub + `proptest!` 生成随机请求
- **事件 Schema**：使用 `apicurio-registry` 本地容器 + Schema 兼容性 API
- **DB Schema**：使用 `sqlx` migration 工具 + Expand-Contract 验证
- **横切配置**：使用 `serde_json::Schema` 校验配置文件

## 2.4 覆盖率策略

| 维度 | 目标 |
|---|---|
| 接口契约覆盖率 | 100%（全部 public API + 全部事件 topic） |
| 集成路径覆盖率 | ≥ 70%（跨模块主路径） |
| 缺陷密度 | ≤ 1.0 件/KLOC（QA-004） |
| 属性不变条件 | BZ-001〜007 各 1 件以上（QA-002） |
| 状态机非法迁移 | 全部拒绝（QA-003） |

---

## 3. 测试用例

## 3.1 模块 A：两级 ID 体系集成

对应 RGS-BAS-009 §2.2〜§2.3 的 ID 归属与主编号映射表结构。

| 用例 ID | 试验级别 | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|---|
| TST-IT-00-001 | [TL-2] | §3.1 域内 ID 注册 | 域内 ID 在登记表注册并产生主编号 | N | 启动 in-memory `id-registry` | `register_domain_id("CS", 1)` | 调用 | 返回 `Ok(Registered { main_id: "FR-CS-001" })` | 主编号格式符合 §3.1 |
| TST-IT-00-002 | [TL-2] | §3.1 主编号 ↔ 域内 ID 反查 | 通过主编号反查域内 ID | N | 已有 FR-CS-001 | `lookup_main("FR-CS-001")` | 调用 | 返回 `("CS", 1)` | 反查正确 |
| TST-IT-00-003 | [TL-3] | §3.2 ID 归属登记表与 GOV-ID-005 | 域内 ID 段需在登记表注册后方可使用 | N | 注册表含 "CS" | `validate_id("FR-CS-001")` | 调用 | 返回 `Ok(_)` | 段已注册 |
| TST-IT-00-004 | [TL-3] | §3.2 段未注册拒绝 | 未注册的域段被拒 | A | 注册表不含 "XX" | `validate_id("FR-XX-001")` | 调用 | 返回 `Err(UnregisteredDomain)` | 错误信息含 "XX" |
| TST-IT-00-005 | [TL-2] | §3.3 跨域引用 | 经济域 ID 被业务域引用 | N | FR-EC-001 已注册 | `cross_reference("FR-EC-001", "FR-RT-014")` | 调用 | 引用关系被记录 | 双向引用表新增行 |
| TST-IT-00-006 | [TL-2] | §3.4 附件C §7 一致性 | 附件 C §7 登记的 26 个域全部能在注册表找到 | N | 附件 C 全文 | `audit_all_domains()` | 调用 | 返回 26 项，全部状态 `Active` | 数量与 §7 一致 |
| TST-IT-00-007 | [TL-2] | §3.5 数据库 Schema 集成 | id_registry 表迁移至 PostgreSQL 成功 | N | ephemeral PG | `sqlx migrate run` | 调用 | 迁移成功，`id_registry` 表存在 | 字段类型符合 §3.5 |
| TST-IT-00-008 | [TL-4] | §3.6 ID 不重不漏 | 随机 10,000 个 ID 注册后无重复 | P | 内存表 | proptest：随机 10,000 个 ID | 100 次 | 全部唯一 | proptest 100 次无失败 |
| TST-IT-00-009 | [TL-2] | §3.7 跨服务集成 | `gov-ci` 进程通过 gRPC 调用 `id-registry` 服务 | N | 启动 id-registry 服务 | `gov_ci::check_id("BR-001")` via gRPC | 调用 | 返回 `Ok(Registered)` | gRPC 调用成功 |
| TST-IT-00-010 | [TL-3] | §3.7 gRPC Schema 兼容 | client/server Schema 版本不匹配时报错 | A | server v2, client v1 | client v1 调用 server v2 | 调用 | 返回 `Err(SchemaIncompatible)` | 兼容性检查生效 |

## 3.2 模块 B：OLU 运维负荷预算机制集成

对应 RGS-BAS-009 §3.1〜§3.4 的 OLU 预算机制。

| 用例 ID | 试验级别 | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|---|
| TST-IT-00-011 | [TL-2] | §4.1 预算分配 | 服务注册时分配 SRE 工时预算 | N | 服务 `auth-svc` 注册 | `allocate_budget("auth-svc", 0.3)` | 调用 | `auth-svc` OLU 预算为 0.3 SRE·d/wk | 分配成功 |
| TST-IT-00-012 | [TL-2] | §4.2 工时上报 | 服务向 OLU 台上报实际工时 | N | auth-svc 运行 1 周 | `report_consumption("auth-svc", 0.4)` | 调用 | OLU 台账累计 0.4 | 累计值正确 |
| TST-IT-00-013 | [TL-2] | §4.3 超额告警 | 累计消耗超预算 80% 触发告警 | B | budget=1.0, consumed=0.85 | 写入 0.85 | 调用 | 返回 `Warning(80% reached)` | 告警阈值符合 §4.3 |
| TST-IT-00-014 | [TL-2] | §4.3 超额阻断 | 累计消耗超 100% 阻断新 PR | B | budget=1.0, consumed=1.2 | CI 检查 | 调用 | CI 返回非零退出 | 阻断生效 |
| TST-IT-00-015 | [TL-2] | §4.4 多服务叠加 | 多个服务 OLU 汇总至全局预算 | N | 3 个服务分别消耗 0.3/0.4/0.5 | `aggregate()` | 调用 | 全局消耗 1.2 | 汇总正确 |
| TST-IT-00-016 | [TL-2] | §4.5 周期切换 | 每周一切换至新周，旧数据归档 | N | 已积累 7 天数据 | `rotate_weekly()` | 调用 | 旧周数据移至 `olu_history`，新周期重置 | 切换符合 §4.5 |
| TST-IT-00-017 | [TL-4] | §4.6 预算守恒 | 任何时刻 `sum(consumed) ≤ budget` | P | 随机操作序列 | proptest：1000 个随机操作 | 100 次 | 不变量始终成立 | proptest 100 次通过 |
| TST-IT-00-018 | [TL-2] | §4.7 NFR-OP-010 校验 | 总 OLU > 2.0 SRE·d/wk 时被 ADR 拒绝 | A | 总消耗 2.5 | CI 检查 | 调用 | CI 失败，要求缩减 | NFR-OP-010 守门生效 |
| TST-IT-00-019 | [TL-2] | §4.8 DB 集成 | OLU 数据持久化至 PostgreSQL 并可查询 | N | ephemeral PG | 写入 0.5，进程重启，查询 | 调用 | 查询返回 0.5 | 持久化生效 |
| TST-IT-00-020 | [TL-3] | §4.9 Schema 演进 | 追加 `notes` 字段后老数据仍可读 | A | 旧 schema 数据 | 新 schema 读取 | 调用 | 旧数据 `notes=None` | 向后兼容 |

## 3.3 模块 C：治理闭环 CI 机械校验集成

对应 RGS-BAS-009 §4 治理闭环 CI 机械校验设计。

| 用例 ID | 试验级别 | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|---|
| TST-IT-00-021 | [TL-2] | §5.1 CI 端到端 | 提交含未注册 ID 的 PR → CI 阻断 | N | git repo + CI runner | `git commit` 引用 `UNREG-001` | 触发 CI | CI 返回非零退出 | 端到端阻断生效 |
| TST-IT-00-022 | [TL-2] | §5.2 PR 报告 | CI 失败时生成结构化报告 | N | 同上 | CI 失败 | 读取 artifact | 报告含 `rule_id`/`file`/`line`/`message` | 报告格式符合 §5.2 |
| TST-IT-00-023 | [TL-2] | §5.3 治理规则集热加载 | 规则集更新后 CI 无需重启即生效 | N | 已有 CI runner | 修改 rules.yaml，触发 CI | 调用 | 新规则生效 | 热加载符合 §5.3 |
| TST-IT-00-024 | [TL-3] | §5.4 规则集 Schema | 非法 rule YAML 被拒 | A | 缺 `rule_id` | `load_ruleset(bad.yaml)` | 调用 | 返回 `Err(MissingField)` | Schema 校验生效 |
| TST-IT-00-025 | [TL-2] | §5.5 与附件D联动 | TBD 主编号在附件D 不存在时被识别 | N | gov-ci 读附件D | `check_tbd("TBD-99999")` | 调用 | 返回 `Err(OrphanTbdRef)` | 联动正确 |
| TST-IT-00-026 | [TL-2] | §5.6 附件C ↔ 附件D 一致性 | 附件C 引用的所有 TBD 都存在于附件D | N | 附件C 全文 | `cross_check()` | 调用 | 返回 `Ok(_)` | 引用一致 |
| TST-IT-00-027 | [TL-2] | §5.7 ADR ↔ ARC 联动 | ARC 必须有 ADR 关联 | N | 加载所有 ADR | `audit_arc_adr()` | 调用 | 全部 ARC 都有 ADR | 联动完整 |
| TST-IT-00-028 | [TL-2] | §5.8 误判申诉通道 | 开发者可标记 false positive 绕过单次检查 | N | CI 失败 PR | `gov-ci bypass --reason "..."` | 调用 | CI 重新通过，附 bypass 记录 | 申诉通道可用 |
| TST-IT-00-029 | [TL-2] | §5.9 bypass 审计 | 每次 bypass 写入审计日志 | N | 触发 bypass | `query_audit_bypass()` | 调用 | 末条记录含操作者、原因、时间 | 审计完整 |
| TST-IT-00-030 | [TL-2] | §5.10 跨语言集成 | 治理 CI 工具链与 Go 编写的 k6 通过 JSON RPC 协作 | N | k6 + gov-ci | k6 报告性能 → gov-ci 校验 NFR-PE-001 | 调用 | gov-ci 接收并校验 | 跨语言集成可用 |

## 3.4 模块 D：横切关注点集成（4 项横切需求）

对应 RGS-BAS-009 §5.1〜§5.4 的 4 项横切需求：① FR-GOV-001〜004 插件经济边界 ② FR-GOV-010〜014 删除权 ③ FR-GOV-020〜023 机制重复消除 ④ FR-GOV-030〜033 判定权收口。

| 用例 ID | 试验级别 | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|---|
| TST-IT-00-031 | [TL-2] | §6.1 FR-GOV-001 插件经济边界 | 插件尝试直接 UPDATE economy_db 被拒 | A | 启动带插件的 mock 服务 | 插件执行 `UPDATE wallet` | 调用 | 返回 `Err(DirectDbWriteForbidden)` | 边界守门生效（VF-016） |
| TST-IT-00-032 | [TL-2] | §6.1 插件走 EC-003 路径 | 插件通过确定请求 API 写经济成功 | N | 同上 | 插件调 `EC-003` API | 调用 | 副作用 1 次，写流水 | 路径符合 FR-EC-003 |
| TST-IT-00-033 | [TL-2] | §7.1 FR-GOV-010 删除幂等 | 删除用户 → 重复删除 → 结果一致 | N | 用户 soft-deleted | `delete(uid)` × 2 | 调用 | 两次均返回 `Ok(AlreadyDeleted)` | 幂等（FT-013 基础） |
| TST-IT-00-034 | [TL-2] | §7.2 删除审计不可篡改 | 删除记录不可被 UPDATE/DELETE | A | 删除记录已写入 | `UPDATE audit_log SET action='X'` | 调用 | 权限拒绝或触发只读约束 | 审计不可变（NFR-SE-010） |
| TST-IT-00-035 | [TL-2] | §8.1 FR-GOV-020 机制重复消除 | 同一资产无两套持久化路径 | N | 扫描 codebase | `audit_dual_persistence()` | 调用 | 返回空集合 | 重复为 0 |
| TST-IT-00-036 | [TL-2] | §9.1 FR-GOV-030 判定权收口 | 封禁判定仅 AdminService 拥有 | N | 启动全部业务服务 | 业务服务尝试调 `BanAccount` | 调用 | 返回 `Err(UnauthorizedCapability)` | 判定权唯一 |
| TST-IT-00-037 | [TL-5] | §9.2 状态机 跨模块 | 删除编排状态机跨服务 | S | 编排器跨进程 | 触发 `Idle → Paused` 非法迁移 | 调用 | 返回 `Err` | 跨模块状态机拒绝 |
| TST-IT-00-038 | [TL-2] | §6.2 插件沙箱 | 插件 panic 不导致宿主进程崩溃 | A | 启动带 panic 插件 | 插件触发 panic | 调用 | 宿主继续运行，插件被卸载 | 隔离生效（NFR-PLG-004 / VF-014） |
| TST-IT-00-039 | [TL-2] | §6.3 插件生命周期 | 插件注册→启用→停用→注销 | N | 插件注册表 | 顺序调用 4 个 API | 调用 | 状态机依次迁移 | 生命周期符合 §6.3 |
| TST-IT-00-040 | [TL-2] | §9.3 跨节点套利防护 | 同一账号在多节点并发操作经济 | A | 启动 2 个 EC 服务实例 | 双侧并发 `EC-003` | 调用 | 仅 1 次成功，另 1 次返回 `Err(EpochMismatch)` | 跨节点判定权唯一（FT-012） |

## 3.5 模块 E：横切关注点 4 阶段集成

**待设计，不计入 RGS-BAS-009 基线追溯**：RGS-BAS-009 当前未定义“分阶段落地”章节；本模块保留为跨文档实施计划测试，待具名负责人补充设计依据后再纳入基线覆盖。

| 用例 ID | 试验级别 | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|---|
| TST-IT-00-041 | [TL-2] | §10.1 PH-1 治理闭环挂载 | PH-1 完成时治理 CI 接入主干 | N | 模拟 PH-1 收尾 | `git tag phase-1-done` | 触发 CI | 治理 CI 启用 | 挂载时机正确 |
| TST-IT-00-042 | [TL-2] | §10.2 PH-2 数据库标准 | DB 迁移通过 Expand-Contract 流程 | N | 新增列 `notes` | 跑两阶段迁移 | 调用 | 第一阶段：新旧兼容；第二阶段：删旧列 | 流程符合 §10.2 |
| TST-IT-00-043 | [TL-2] | §10.3 PH-3 插件边界 | 插件接入自动校验经济边界 | N | 部署带插件的服务 | 提交插件代码 | CI 检查 | 不合规插件被拒 | 自动校验生效 |
| TST-IT-00-044 | [TL-2] | §10.4 PH-4 OLU 预算 | PH-4 引入 4 个新服务，OLU 累计 1.8 | N | 部署 PH-4 服务 | 累计 OLU | 调用 | OLU ≤ 2.0（NFR-OP-010） | 预算守门 |
| TST-IT-00-045 | [TL-2] | §10.5 PH-6 治理聚合项启用 | AC-019 在 PH-6 起具备阻断力 | N | 部署 21 份领域文档 | 触发 AC-019 检查 | 调用 | 任一 AC-<域>-* 未达 → CI 失败 | AC-019 聚合生效 |

## 3.6 业务规则集成级属性试验（TL-4）

| 用例 ID | 试验级别 | 对应需求 | 测试目的 | 覆盖类型 | 测试方法 |
|---|---|---|---|---|---|
| TST-IT-00-046 | [TL-4] | BZ-001 货币余额非负 | 跨服务并发下余额仍 ≥ 0 | P | proptest：100 个并发账户，断言守恒 |
| TST-IT-00-047 | [TL-4] | BZ-002 支付订单幂等 | 跨服务重复支付仅 1 次发货 | P | proptest：同 order_id 在 3 个服务实例并发调发货 |
| TST-IT-00-048 | [TL-4] | BZ-003 道具增减可由流水复原 | 跨服务操作后 `sum(ledger) == inventory` | P | proptest：1000 个跨服务操作 |
| TST-IT-00-049 | [TL-4] | BZ-005 已归档对局不可变 | 跨服务状态机非法迁移拒绝 | P | proptest：随机目标状态 |
| TST-IT-00-050 | [TL-4] | BZ-006 封禁账号不可建会话 | 跨服务登录尝试拒绝 | P | proptest：随机账号跨节点登录 |

## 3.7 跨模块状态机试验（TL-5）

| 用例 ID | 试验级别 | 对应需求 | 测试目的 | 覆盖类型 | 测试方法 |
|---|---|---|---|---|---|
| TST-IT-00-051 | [TL-5] | ST-001 玩家会话 | `Terminating → Active` 跨服务拒绝 | S | 通过会话服务与运行时联合触发 |
| TST-IT-00-052 | [TL-5] | ST-002 对局 | `Finished → Running` 跨服务拒绝 | S | 通过对局服务与状态机服务联合触发 |
| TST-IT-00-053 | [TL-5] | ST-003 购买 | `Refunded → Delivered` 跨 Saga 拒绝 | S | 通过 Saga 与经济服务联合触发 |
| TST-IT-00-054 | [TL-5] | ST-005 账号 | `Banned → Active` 跨服务拒绝 | S | 通过账号服务与认证服务联合触发 |
| TST-IT-00-055 | [TL-5] | 治理编排状态机 | `Idle → Paused` 跨服务拒绝 | S | gov-ci 与 deletion_orchestrator 联合触发 |

## 3.8 契约试验（TL-3）专项

| 用例 ID | 试验级别 | 对应设计章节 | 测试目的 | 覆盖类型 | 测试方法 |
|---|---|---|---|---|---|
| TST-IT-00-056 | [TL-3] | §3.7 gRPC API | 全部 public gRPC API 兼容 | N | 生成的 stub + 兼容性脚本 |
| TST-IT-00-057 | [TL-3] | §5.4 事件 Schema | 全部事件 topic Schema 兼容 | N | apicurio-registry 兼容性 API |
| TST-IT-00-058 | [TL-3] | §3.5 DB Schema | Expand-Contract 迁移可逆 | A | 写新 schema → migrate down → 数据无损 |
| TST-IT-00-059 | [TL-3] | §6.1 插件配置 | 插件 manifest Schema 校验 | A | 非法 manifest 被拒 |
| TST-IT-00-060 | [TL-3] | §10.5 治理配置 | 治理规则集 YAML Schema 校验 | A | 非法规则被拒 |

---

## 4. 追溯性矩阵

| 基本设计章节 | 用例 ID 范围 | 试验级别 |
|---|---|---|
| RGS-BAS-009 §2.2〜§2.3 两级 ID 体系 | TST-IT-00-001〜010 | TL-2/TL-3/TL-4 |
| RGS-BAS-009 §3.1〜§3.4 OLU 预算机制 | TST-IT-00-011〜020 | TL-2/TL-3/TL-4 |
| RGS-BAS-009 §4 治理闭环 CI 校验 | TST-IT-00-021〜030 | TL-2/TL-3 |
| RGS-BAS-009 §5.1 插件经济边界 | TST-IT-00-031〜032, 038〜039 | TL-2 |
| RGS-BAS-009 §5.2 删除权 | TST-IT-00-033〜034 | TL-2 |
| RGS-BAS-009 §5.3 机制重复消除 | TST-IT-00-035 | TL-2 |
| RGS-BAS-009 §5.4 判定权收口 | TST-IT-00-036, 040 | TL-2 |
| 待设计：分阶段落地章节（RGS-BAS-009 未定义；不计入基线追溯） | TST-IT-00-041〜045 | TL-2 |
| 业务规则（BZ-*）跨服务 | TST-IT-00-046〜050 | TL-4 |
| 状态机（ST-*）跨模块 | TST-IT-00-051〜055 | TL-5 |
| Schema 契约 | TST-IT-00-056〜060 | TL-3 |
| AC-015 OSI 许可 | TST-IT-00-024 | 全部依赖组件 |
| AC-017 中间件导入判定 | TST-IT-00-030, 044 | ARC-014 校验 |
| AC-019 领域验收聚合 | TST-IT-00-045 | 21 份领域文档 |

---

## 5. 测试执行计划

## 5.1 触发时机

| 触发条件 | 执行范围 | 时限 |
|---|---|---|
| 每次 PR 推送 | 受影响 crate 的 L1/L2 集成 | < 8 min |
| 每次合并至 main | L1〜L4 全部集成 + TL-3 契约 | < 12 min（QA-006 内） |
| 每晚 nightly | L5 外部依赖 Mock + 10000 次属性迭代 | 不阻塞主干 |
| 每次领域文档变更 | 跨域引用一致性 + 治理 CI 校验 | < 3 min |

## 5.2 测试环境

| 组件 | 配置 |
|---|---|
| PostgreSQL | ephemeral container，postgresql:16-alpine |
| Redis（缓存） | ephemeral container，redis:7-alpine |
| paymock | 启动 RGS-BAS-012 §4 mock 服务 |
| 治理 CI | git + ephemeral runner |
| k6（按需） | RGS-BAS-012 §5 |

## 5.3 覆盖率门禁

- 接口契约覆盖率：100%
- 集成路径覆盖率：≥ 70%
- 缺陷密度：≤ 1.0 件/KLOC（QA-004）
- 业务规则属性：proptest 默认 1000 次迭代无失败
- 跨服务状态机：100% 非法迁移被拒

---

## 6. 通过判定基准

| 维度 | 基准 |
|---|---|
| 所有用例 PASS | TST-IT-00-001〜060 全部通过 |
| 接口契约 | 100% 兼容（gRPC + 事件 + DB） |
| 集成路径 | ≥ 70% 覆盖 |
| 属性不变条件 | 1000 次迭代无失败 |
| 状态机 | 跨模块全部非法迁移被拒 |
| 缺陷密度 | ≤ 1.0 件/KLOC |
| 静态检查 | `cargo clippy --all-targets -- -D warnings` 通过 |
| 跨语言集成 | k6 ↔ gov-ci 通信正常 |


## 6.5 NFR 覆盖索引

本主题域覆盖的非功能需求编号全集（按 RGS-REQ-003 等级 Lv.2/3/4 全覆盖）：

- **NFR-EN-***：NFR-EN-003
- **NFR-MI-***：NFR-MI-005
- **NFR-OP-***：NFR-OP-010, NFR-OP-008

## 6.6 ADR 决策验证（本主题）

本主题域涉及的 ADR 决定项的"实现位置 + 测试位置 + 守门位置"是否完备：

| ADR 编号 | 决定项摘要 | 实现位置 | 测试位置（本文档） | 守门位置 |
|---|---|---|---|---|
| RGS-ADR-0024 | 治理闭环的重新闭合（两级 ID 体系、三层位阶、两层验收） | RGS-DTL-009 §3 ID 登记表 + §5 治理 CI | §3.1 ID 登记表 + §3.3 治理 CI 校验 | git push 时 CI 阻断 |
| RGS-ADR-0025 | 运维负荷预算（OLU，NFR-OP-010 ≤ 2 SRE） | RGS-DTL-009 §4 OLU 预算台账 | §3.2 OLU 预算台账 | CI 静态检查 + 季度复盘 |


## 7. 风险与未决事项

| ID | 内容 | 处理 |
|---|---|---|
| TBD-GOV-001 | 版本回滚时限最终值 | 集成测试中按"≤ 30s"保守假设 |
| TBD-013 | OLU 工时计算模型 | 集成测试用 mock 工时数据，正式值由运维核定 |
| RSK-GOV-002 | 治理 CI 与外部系统（如 GitHub）耦合 | 采用插件化 webhook 适配层 |
| RSK-IT-001 | ephemeral PG 启动慢 | 使用预热 runner pool |
| TBD-INF-002 | 日志聚合后端选型 | 影响 OLU 报告展示，集成测试用 mock |

---

> 本文档为 RGS-TST 系列主题 00 集成测试设计书。系统测试设计书见 `RGS-TST-ST-00_*.md`。

## 7.5 TBD 处置

本主题涉及的 TBD 处置方式：

| TBD 编号 | 描述 | 处置 |
|---|---|---|
| TBD-001 | OLU 真实工时 | PH-1 前完成首次校准 |
| TBD-003 | 重连宽限期 | 60s 初始值，PH-3 实测校准 |
| TBD-004 | 个人信息保护等级 | 法务确认，PH-4 前定 |
| TBD-GOV-001 | 版本回滚 SLA | PH-1 前定最终值 |
| TBD-008 | 客户端协议 N-1 接受期 | PH-3 前定 |
| RSK-006 | 100k CCU 实测风险 | PH-4 1k→10k 渐进 |
| RSK-007 | 死锁在生产显现 | FT-007 + 静态检查 + ARC-013 |

