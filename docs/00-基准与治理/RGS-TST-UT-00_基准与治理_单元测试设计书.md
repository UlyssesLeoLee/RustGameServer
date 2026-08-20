# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 00 基准与治理 — 单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-00 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-009 详细设计书（体系治理与横切关注点） |
| 本主题域源文档全集（REQ/BAS/DTL） | RGS-REQ-001、RGS-REQ-002、RGS-REQ-003、RGS-REQ-004、RGS-REQ-005、RGS-REQ-013、RGS-BAS-009、RGS-DTL-009 |

| V模型层级 | TL-1 单元试验 ↔ DTL 详细设计（IPA 共通フレーム） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程／RGS-REQ-001 §12.1 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师（自动化产出） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。基于 RGS-DTL-009 的物理/实现级设计，覆盖 ID 登记表、OLU 预算台账、治理闭环 CI 机械校验、PH-6 删除/导出编排等模块的函数/类型级正确性试验设计 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（技术） | | | 与 RGS-DTL-009 物理设计的逐字段一致性 |
| 评审（QA） | | | 覆盖率目标 QA-001（核心区域语句覆盖 80%）的达成路径 |
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

本文档为 V 模型中**TL-1 单元试验**层级的设计书，对应父详细设计书 **RGS-DTL-009（体系治理与横切关注点）**。其目的是：

- 将 RGS-DTL-009 中各模块的物理/实现级设计，分解为**函数・类型级别的可执行测试用例**
- 保证每条实现级决定（数据结构、算法、状态机、序列化格式）均有对应测试覆盖
- 满足 RGS-REQ-001 §12.2 QA-001 单元试验覆盖率（核心区域语句覆盖 80% 以上）目标
- 满足 QA-002 重要不变条件的属性试验覆盖（覆盖 RGS-REQ-001 §4.3 全部业务规则 BZ-001〜007 各 1 件以上）
- 满足 QA-003 状态迁移试验覆盖（覆盖 RGS-REQ-001 §8 全部禁止迁移的拒绝试验）

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-REQ-001 §12.1 | 试验级别定义 | 确认 TL-1 范围 |
| RGS-REQ-013 | 体系治理与横切关注点 需求定义书 | 父需求 |
| RGS-BAS-009 | 体系治理与横切关注点 基本设计书 | 父基本设计 |
| RGS-DTL-009 | 体系治理与横切关注点 详细设计书 | 父详细设计，本测试设计书逐字段对应 |
| RGS-REQ-005 | 附件D 问题风险管理表 | TBD-GOV-001 等未决事项的来源 |
| RGS-TST-UT-01〜07 | 其他主题域单元测试设计书 | 跨域引用 |

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
       集成试验                验证                  RGS-TST-IT-00     │  │
                                                                            │
       详细设计                单元                    RGS-DTL-009     ┐  │  │
         ↕                                                  ↕             │ UT
       单元试验                证明                ★ RGS-TST-UT-00 ★   │  │  │
                                                                            │
       实现                    —                       Rust 源码        ┘  ┘  ┘
```

## 2.2 覆盖率策略

| 维度 | 目标 | 验证手段 |
|---|---|---|
| 语句覆盖率（核心区域） | ≥ 80%（QA-001） | `cargo tarpaulin` / `cargo llvm-cov` 报告 |
| 分支覆盖率 | ≥ 70% | 同上 |
| 业务规则不变条件（BZ-001〜007） | 100%（QA-002） | 属性试验 proptest |
| 状态机禁止迁移（ST-000-1〜4） | 100%（QA-003） | 状态机试验用例 |
| 治理闭环 | 100% 关键函数 | 见 §3 用例矩阵 |

## 2.3 与其他测试设计书的边界

| 边界 | 不在本文档的内容 | 归属 |
|---|---|---|
| 跨 crate 集成 | ID 登记表与 OLU 预算台账的联合读写 | RGS-TST-IT-00 |
| 端到端治理闭环 | 从开发者提交 PR 到 CI 阻断的完整链路 | RGS-TST-ST-00 |
| 故障注入 | 治理 CI 误判场景 | TL-7（PH-8） |
| 性能 | 大规模 ID 登记的查询性能 | TL-6（k6） |

---

## 3. 测试用例

## 3.1 模块 A：ID 登记表（id_registry）

对应 RGS-DTL-009 §3。

| 用例 ID | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-00-001 | §3.1 登记表结构 | 验证 `IdEntry` 结构体字段集合与默认值 | N | 全新表 | `IdEntry::default()` | 调用 `Default::default()` | 字段 `registered_at` 为 None，`status` 为 `Draft` | 所有字段值符合 §3.1 字段表 |
| TST-UT-00-002 | §3.1 字段类型 | `id` 字段接受 `String` 域名字段与 `u32` 序号 | N | — | `IdEntry { id: ("CS".into(), 1), .. }` | 构造并序列化 | JSON 序列化结果含 `domain` 与 `seq` 两个键 | 序列化字段名与 §3.1 字段表一致 |
| TST-UT-00-003 | §3.2 状态机 合法迁移 | `Draft → Registered → Deprecated` 全部合法 | N | 新建 `Draft` 条目 | `transition_to(Registered)` 再 `transition_to(Deprecated)` | 顺序调用 | 两次均返回 `Ok(_)`，最终 `status == Deprecated` | 状态字段正确更新 |
| TST-UT-00-004 | §3.2 状态机 非法迁移 | `Draft → Deprecated` 跳过 Registered 被拒绝 | S | `Draft` 状态条目 | `transition_to(Deprecated)` | 调用 | 返回 `Err(InvalidTransition)` | 错误类型为 `IdRegistryError::InvalidTransition` |
| TST-UT-00-005 | §3.2 状态机 终态 | `Deprecated → Registered` 被拒绝（终态不可逆） | S | `Deprecated` 条目 | `transition_to(Registered)` | 调用 | 返回 `Err(InvalidTransition)` | 状态保持 `Deprecated` |
| TST-UT-00-006 | §3.3 唯一性约束 | 重复 `(domain, seq)` 注册返回错误 | A | 内存表 | 注册 `(CS, 1)` 两次 | 第二次注册 | 返回 `Err(DuplicateId)` | 错误类型 `IdRegistryError::Duplicate` |
| TST-UT-00-007 | §3.4 序号递增 | 同域内序号必须单调递增 | B | 已有 `(CS, 5)` | 注册 `(CS, 4)` | 调用 | 返回 `Err(SeqNotMonotonic)` | 拒绝比已存在最大序号更小的注册 |
| TST-UT-00-008 | §3.5 查询 API | 按 `domain` 查询返回该域全部条目 | N | 已注册 3 个 `CS` 域条目 | `list_by_domain("CS")` | 调用 | 返回长度 3 的 Vec | 元素均为 `domain == "CS"` |
| TST-UT-00-009 | §3.5 查询 API 空域 | 查询不存在的域返回空 Vec | B | 空表 | `list_by_domain("ZZ")` | 调用 | 返回空 Vec，不报错 | Vec.len() == 0 |
| TST-UT-00-010 | §3.6 序列化往返 | JSON serialize/deserialize 字段无损 | N | 完整 `IdEntry` | serialize → deserialize | 调用 | 两份对象 `PartialEq` 相等 | 字段无损 |

## 3.2 模块 B：OLU 预算台账（olu_ledger）

对应 RGS-DTL-009 §4。

| 用例 ID | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-00-011 | §4.1 台账结构 | `OluRecord` 字段集合与默认值 | N | — | `OluRecord::default()` | 调用 | `consumed_slo` 为 0.0，`period` 为 `Weekly` | 字段符合 §4.1 |
| TST-UT-00-012 | §4.2 SLO 累加 | 累加 3 次消耗，结果为 3 次之和 | N | 新建 `OluRecord` | `accumulate(0.1)` × 3 | 顺序调用 | `consumed_slo ≈ 0.3`（浮点允许 1e-9 误差） | 浮点加法符合 IEEE 754 |
| TST-UT-00-013 | §4.2 负值拒绝 | `accumulate(-0.1)` 被拒绝 | A | — | `accumulate(-0.1)` | 调用 | 返回 `Err(NegativeConsumption)` | 错误类型正确，状态未变 |
| TST-UT-00-014 | §4.3 预算检查 | `consumed_slo < budget` 时返回 `Ok` | B | budget=1.0，consumed=0.5 | `check_budget()` | 调用 | 返回 `Ok(remaining=0.5)` | remaining 计算正确 |
| TST-UT-00-015 | §4.3 超额拒绝 | `consumed_slo > budget` 时返回 `Err` | B | budget=1.0，consumed=1.5 | `check_budget()` | 调用 | 返回 `Err(OverBudget)` | 错误类型 `OluError::OverBudget` |
| TST-UT-00-016 | §4.4 周期切换 | `Weekly → Monthly` 后计数器按月预算重新核算 | N | 已消耗 weekly 0.5 | `rotate_period(Monthly)` | 调用 | `consumed_slo` 重置为 0，预算切换至月预算 | 周期字段更新，consumed 重置 |
| TST-UT-00-017 | §4.5 持久化 | 台账记录可序列化至 TOML 并读回 | N | 完整 `OluRecord` | to_toml → from_toml | 调用 | 读回对象与原始 `PartialEq` 相等 | 字段无损 |
| TST-UT-00-018 | BZ-001 属性试验 | 货币/资产类 OLU 记录 consumed 永远 ≥ 0 | P | — | proptest：随机 `f64` 值（除负数） | 1000 次随机输入 | 全部通过 `consumed_slo >= 0.0` 断言 | proptest 无失败用例 |
| TST-UT-00-019 | §4.6 CI 静态检查 | `olu_ledger::accumulate` 不得接受 `f64::NAN` | A | — | `accumulate(f64::NAN)` | 调用 | 返回 `Err(InvalidValue)` | NaN 被识别为非法值 |

## 3.3 模块 C：治理闭环 CI 校验器（gov_ci）

对应 RGS-DTL-009 §5。

| 用例 ID | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-00-020 | §5.1 规则集加载 | 从 YAML 加载 `GovRuleSet` | N | `rules.yaml` 测试夹具 | `load_ruleset("fixtures/rules.yaml")` | 调用 | 返回 `Ok(GovRuleSet { rules: vec![…] })` | 规则数与夹具一致 |
| TST-UT-00-021 | §5.2 规则：ID 已注册 | 需求 ID 已在登记表 → 规则通过 | N | 登记表含 `BR-001` | `check(BR-001)` | 调用 | 返回 `Ok(_)` | 状态 `Passed` |
| TST-UT-00-022 | §5.2 规则：ID 未注册 | 需求 ID 未在登记表 → 规则失败 | A | 空表 | `check(UNREG-XXX)` | 调用 | 返回 `Err(UnregisteredId)` | 错误信息含 ID 字面值 |
| TST-UT-00-023 | §5.3 规则：TBD 有主编号 | TBD 项必须在附件 D 分配 `TBD-nnn` 主编号 | N | 附件 D 含 TBD-001 | `check_tbd("TBD-XYZ")` 引用 TBD-001 | 调用 | 返回 `Ok(_)` | 引用有效 |
| TST-UT-00-024 | §5.3 规则：TBD 断裂引用 | TBD 项引用不存在的主编号 → 失败 | A | — | `check_tbd("TBD-99999")` 不在附件 D | 调用 | 返回 `Err(OrphanTbdRef)` | 错误信息含 ID |
| TST-UT-00-025 | §5.4 规则：ADR 与 ARC 一致 | ARC-018 必须存在对应 ADR | N | 存在 ADR-0018 | `check_arc_adr(ARC-018)` | 调用 | 返回 `Ok(_)` | 找到 ADR |
| TST-UT-00-026 | §5.4 规则：ADR 缺失 | ARC-099 找不到对应 ADR | A | — | `check_arc_adr(ARC-099)` | 调用 | 返回 `Err(MissingAdr)` | 错误信息提示需补充 ADR |
| TST-UT-00-027 | §5.5 批量校验 | 一次性校验 N 条规则 | N | 10 条规则 | `check_all(rules)` | 调用 | 返回 `Report { passed: 8, failed: 2 }` | 计数正确 |
| TST-UT-00-028 | §5.6 报告序列化 | 校验结果可输出为 JSON | N | 一次失败 | `report_to_json(&report)` | 调用 | JSON 含 `rule_id`/`status`/`message` | JSON Schema 符合 §5.6 |
| TST-UT-00-029 | BZ-007 属性试验 | 治理校验全部规则的并集 = 单条规则结果之并集（幂等） | P | — | proptest：随机规则子集 | 1000 次 | 满足 `union(all) == all(union)` | proptest 1000 次无失败 |

## 3.4 模块 D：PH-6 删除/导出编排（deletion_orchestrator）

对应 RGS-DTL-009 §6。

| 用例 ID | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-00-030 | §6.1 编排状态机 启动 | `Idle → Running` | N | `Idle` 编排器 | `start()` | 调用 | 状态变 `Running` | 状态正确 |
| TST-UT-00-031 | §6.1 编排状态机 暂停 | `Running → Paused` | N | `Running` | `pause()` | 调用 | 状态变 `Paused` | 状态正确 |
| TST-UT-00-032 | §6.1 编排状态机 非法迁移 | `Idle → Paused` 跳过 Running 被拒 | S | `Idle` | `pause()` | 调用 | 返回 `Err(InvalidTransition)` | 错误类型正确 |
| TST-UT-00-033 | §6.2 删除步骤序列 | 完整执行软删→匿名化→审计→清表 | N | 编排器就绪 | `execute_deletion(target_id)` | 调用 | 全部 4 步骤依次执行，状态 `Completed` | 步骤顺序符合 §6.2 |
| TST-UT-00-034 | §6.3 中断恢复 | `Paused` 后 `resume()` 续跑 | N | 步骤 2/4 完成后暂停 | `resume()` | 调用 | 从步骤 3 继续，状态 `Completed` | 当前步骤号正确恢复 |
| TST-UT-00-035 | §6.4 幂等性 | 同一 `target_id` 重复 `execute_deletion` | P | 已完成删除 | `execute_deletion(target_id)` 再次 | 调用 | 立即返回 `Ok(AlreadyDeleted)` | 副作用 0 次（FT-013 基础） |
| TST-UT-00-036 | §6.5 失败步骤回退 | 步骤 3 失败时回退步骤 1-2 | A | 步骤 3 注入错误 | `execute_deletion(...)` | 调用 | 步骤 1-2 已执行被回退，最终状态 `Failed` | 状态机正确 |
| TST-UT-00-037 | §6.6 审计记录 | 删除操作产生 `AuditLogEntry` | N | 执行删除 | `get_audit_log()` | 调用 | 末尾记录含 `action=DeletionCompleted`、`target_id` | 字段符合 §6.6 |
| TST-UT-00-038 | §6.7 超时处理 | 单步骤超过 `step_timeout` 强制标记失败 | B | 注入 sleep 超过 timeout | `execute_deletion(...)` | 调用 | 该步骤标 `TimedOut`，状态 `Failed` | 超时判定正确 |

## 3.5 模块 E：横切关注点配置加载（cross_cutting_loader）

对应 RGS-DTL-009 §7。

| 用例 ID | 对应设计章节 | 测试目的 | 覆盖类型 | 前置条件 | 输入 | 步骤 | 预期结果 | 通过判定 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-00-039 | §7.1 配置 schema 验证 | 合法配置加载 | N | `config.toml` 测试夹具 | `load("fixtures/cfg.toml")` | 调用 | 返回 `Ok(CrossCuttingConfig)` | 字段映射正确 |
| TST-UT-00-040 | §7.1 缺字段拒绝 | 必填字段缺失被拒 | A | 缺 `id_registry.path` | `load(...)` | 调用 | 返回 `Err(MissingField)` | 错误指出缺失字段名 |
| TST-UT-00-041 | §7.1 类型错误 | `id_registry.path` 类型非 string 被拒 | A | `id_registry.path: 123` | `load(...)` | 调用 | 返回 `Err(TypeMismatch)` | 错误信息含字段名 |
| TST-UT-00-042 | §7.2 热更新 | 配置文件变更触发重新加载 | N | 已加载配置，文件被覆写 | 等待通知 | — | 内存配置更新 | 重新加载后字段反映新值 |
| TST-UT-00-043 | §7.3 默认值 | 字段缺省时采用 `Default::default()` | B | 仅提供必填字段 | `load(minimal_cfg)` | 调用 | 可选字段等于其默认值 | 默认值符合 §7.3 |

## 3.6 模块 F：业务规则不变条件属性试验

对应 RGS-REQ-001 §4.3 业务规则 BZ-001〜007 与 QA-002。

| 用例 ID | 对应需求 | 测试目的 | 覆盖类型 | 测试方法 |
|---|---|---|---|---|
| TST-UT-00-044 | BZ-001 货币余额非负 | 货币操作永远保持余额 ≥ 0 | P | proptest：随机 `add/sub`，断言 `balance >= 0` |
| TST-UT-00-045 | BZ-002 支付订单不重复发货 | 同一 `order_id` 调 `deliver` 仅一次副作用 | P | proptest：重复 100 次相同 `order_id`，断言副作用计数 = 1 |
| TST-UT-00-046 | BZ-003 道具增减可由流水复原 | 任何时间点聚合 `ledger` = 当前库存 | P | proptest：随机变更序列，断言 `sum(ledger) == inventory` |
| TST-UT-00-047 | BZ-004 客户端申告值不直接采用 | 客户端上报伤害值被忽略 | P | proptest：随机 `client_reported_damage`，断言服务器采用 `server_authoritative` |
| TST-UT-00-048 | BZ-005 已归档对局不可变 | `Archived → *` 全部拒绝 | P | proptest：随机目标状态，断言 `Err(InvalidTransition)` |
| TST-UT-00-049 | BZ-006 封禁账号不可建会话 | `Banned` 账号登录返回 `Err` | P | proptest：随机账号，断言 `status == Banned ⇒ login fails` |
| TST-UT-00-050 | BZ-007 交易原子性 | 交易 Saga 任一步失败 → 双侧回滚至原态 | P | proptest：注入步骤失败，断言双方状态与交易前一致 |

## 3.7 模块 G：状态机非法迁移拒绝试验

对应 RGS-REQ-001 §8 与 QA-003。

| 用例 ID | 对应需求 | 测试目的 | 覆盖类型 | 测试方法 |
|---|---|---|---|---|
| TST-UT-00-051 | ST-001 玩家会话 | `Terminating → Active` 拒绝 | S | 状态机直接调用，断言 `Err` |
| TST-UT-00-052 | ST-001 玩家会话 | `[*] → Active`（无认证激活）拒绝 | S | 同上 |
| TST-UT-00-053 | ST-002 对局 | `Finished → Running` 拒绝 | S | 同上 |
| TST-UT-00-054 | ST-002 对局 | `Archived → Waiting` 拒绝 | S | 同上 |
| TST-UT-00-055 | ST-002 对局 | `Archived → *` 全部拒绝 | S | 参数化 4 个目标态 |
| TST-UT-00-056 | ST-003 购买 | `Refunded → Delivered` 拒绝 | S | 同上 |
| TST-UT-00-057 | ST-003 购买 | `Completed → *` 全部拒绝 | S | 参数化 4 个目标态 |
| TST-UT-00-058 | ST-004 交易 | `Settled → Draft` 拒绝 | S | 同上 |
| TST-UT-00-059 | ST-005 账号 | `Banned → Active` 拒绝（需解封流程） | S | 同上 |
| TST-UT-00-060 | ST-000-3 通用 | 散在 `if status==` 模式不被允许 | S | clippy lint 触发（`status_discriminant_match` 规则） |

---

## 4. 追溯性矩阵

| 详细设计章节 | 用例 ID 范围 | 覆盖类型 |
|---|---|---|
| RGS-DTL-009 §3 ID 登记表 | TST-UT-00-001〜010 | N/A/B/S |
| RGS-DTL-009 §4 OLU 预算台账 | TST-UT-00-011〜019 | N/A/B/P |
| RGS-DTL-009 §5 治理闭环 CI 校验 | TST-UT-00-020〜029 | N/A/B/P |
| RGS-DTL-009 §6 删除/导出编排 | TST-UT-00-030〜038 | N/A/B/S/P |
| RGS-DTL-009 §7 横切关注点配置 | TST-UT-00-039〜043 | N/A/B |
| RGS-REQ-001 §4.3 业务规则 | TST-UT-00-044〜050 | P |
| RGS-REQ-001 §8 状态机 | TST-UT-00-051〜060 | S |
| RGS-REQ-001 §12.1 TL-1 | 全部 | 全覆盖 |
| RGS-REQ-001 §12.2 QA-001 | 全部 | 覆盖率门禁 |
| RGS-REQ-001 §12.2 QA-002 | TST-UT-00-044〜050 | 属性试验门禁 |
| RGS-REQ-001 §12.2 QA-003 | TST-UT-00-051〜060 | 状态机门禁 |
| AC-004 | TST-UT-00-051〜060 | §8 全部禁止迁移 |
| AC-018 | 全部 | 追溯性 |

---

## 5. 测试执行计划

## 5.1 触发时机

| 触发条件 | 执行范围 | 时限 |
|---|---|---|
| 每次 `git commit`（开发者本地） | 受影响 crate 的全部 UT | < 30s（本地） |
| 每次 PR 推送 | 全 workspace 全部 UT | < 5 min |
| 每次合并至 main | 全 workspace 全部 UT + 属性试验 1000 次迭代 | < 10 min（QA-006 内） |
| 每晚 nightly | UT + 属性试验 10000 次迭代 + 模糊测试 | 不阻塞主干 |

## 5.2 测试夹具

| 夹具 | 路径 | 用途 |
|---|---|---|
| `fixtures/rules.yaml` | `tests/fixtures/gov/` | 治理规则集 |
| `fixtures/cfg.toml` | 同上 | 横切配置 |
| `fixtures/olu_seed.toml` | 同上 | OLU 预算台账初始值 |
| `fixtures/audit_log.jsonl` | 同上 | 审计记录样本 |

## 5.3 覆盖率门禁

- 核心区域（`id_registry`、`olu_ledger`、`gov_ci`、`deletion_orchestrator`、`cross_cutting_loader`）语句覆盖 ≥ 80%（QA-001）
- 关键不变条件（BZ-001〜007）100% 覆盖（QA-002）
- 状态机非法迁移（ST-000-1〜4）100% 覆盖（QA-003）
- 不达标时 PR 检查（`cargo llvm-cov fail-under-lines 80`）阻断合并

---

## 6. 通过判定基准

| 维度 | 基准 |
|---|---|
| 所有用例 PASS | TST-UT-00-001〜060 全部通过 |
| 语句覆盖率 | ≥ 80%（核心区域） |
| 业务规则属性 | proptest 默认 1000 次迭代无失败 |
| 状态机 | 全部禁止迁移被拒绝 |
| 静态检查 | `cargo clippy -- -D warnings` 通过 |
| 格式化 | `cargo fmt --check` 通过 |
| 审计可追溯 | 每次失败用例须附 issue 链接（QA-005 复发防止） |


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
| TBD-GOV-001 | 版本回滚时限最终值 | DTL-009 §6 中保留为初始提案，UT 用例中按"≤ 30s"保守假设，PH-1 前最终校准 |
| TBD-009 | 横切关注点配置加载的 schema 演进 | UT 用例覆盖追加字段不破坏既有行为；删除字段需新增测试 |
| RSK-GOV-001 | 属性试验 1000 次迭代可能未覆盖罕见 corner case | nightly 流水线扩至 10000 次 |

---

> 本文档为 RGS-TST 系列（24 套）的主题 00 单元测试设计书。其他主题（01〜07）见对应的 `RGS-TST-UT-01〜07_*.md`。

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

