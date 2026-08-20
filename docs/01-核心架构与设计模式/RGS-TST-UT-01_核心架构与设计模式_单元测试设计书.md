# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 01 核心架构与设计模式 — 单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-01 |
| 版本 | 0.2 |
| 父文档 | RGS-DTL-001/002/022/023/024 详细设计书 |
| V模型层级 | TL-1 单元试验 ↔ DTL 详细设计 |
| 依据标准 | IPA『共通フレーム 2013』詳細設計工程 |
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
| **0.2** | 2026-08-19 | 架构师 | **字段级深化**：每条用例的"对应设计"列升级为"文档 ID + §X.Y + 表/图/字段"；新增"ADR 决策验证"小节覆盖本主题 ADR；新增"TBD 处置"小节 |。覆盖 Actor/ECS/tick/QUIC/经济确定/挂载脚手架/容量分片/请求处理链/集群编排 |
| 0.2 | 2026-08-19 | 架构师 | **字段级深化**：每条用例的"对应设计章节"列升级为"对应设计 ID + 章节 + 表/图/字段"；新增"ADR 决策验证"小节覆盖 ADR-0001/0002/0007/0008/0015/0020/0022/0023/0026/0029；新增"TBD 处置"小节 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构） | | | 字段级映射与 DTL 物理设计的一致性 |
| 评审（QA） | | | QA-001 覆盖率 80% 达成路径 |
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

本文档为 V 模型中 **TL-1 单元试验**层级的设计书，对应主题 01 的 5 份详细设计书。本版本（0.2）相比 0.1 的核心升级：

- **字段级映射**：每条测试用例的"对应设计"列从"§X.Y 章节名"升级为"文档 ID + 章节 + 表号/图号/字段名"
- **ADR 决策验证**：新增 §3.10 小节，对本主题涉及的 10 份 ADR 每条决定项验证：实现位置 + 测试位置 + 守门位置
- **TBD 处置**：新增 §7 小节，明确每条 TBD 在测试中的"按保守假设实施 / 标记为预留 / 待前置条件"三种处置方式
- **强化 V 模型对应**：§2.1 给出 V 字映射图，明确 UT 验证 DTL 哪些字段

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-REQ-001/006/025/026/027 | 需求 | 父需求 |
| RGS-BAS-001 §3.1〜§6.5 | 部署构成、5 服务、API 字段级 | UT 验证对象 |
| RGS-BAS-002 §3-§11 | 挂载脚手架 | UT 验证对象 |
| RGS-BAS-010 §3 | 设计模式/算法 | UT 验证对象 |
| RGS-BAS-022/023/024 | 容量/请求链/部署 | UT 验证对象 |
| RGS-DTL-001 §2-§12 | 物理设计 | UT 一对一字段级 |
| RGS-DTL-002/022/023/024 | 同上 | 同上 |
| RGS-ADR-0001/0002/0007/0008/0015/0020/0022/0023/0026/0029 | 架构决策 | §3.10 验证 |
| RGS-REQ-001 §12.2 | QA-001/002/003 | 覆盖率/属性/状态机门禁 |

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

## 2.1 V 模型对应关系（精修版）

```
       用户需求                验收                    RGS-REQ-001/006/025/026/027  ┐
         ↕                                                  ↕                    │ ST
       验收试验                确认                  RGS-TST-ST-01               │
                                                                                    │
       基本设计                整合                    RGS-BAS-001/002/010/022/023/024  ┐  │
         ↕                                                  ↕                          │ IT
       集成试验                验证                  RGS-TST-IT-01                  │  │
                                                                                    │
        详细设计                单元                    RGS-DTL-001 §2〜§12 ┐ RGS-DTL-002 §2〜§6 ┐  │  │
         ↕                            ↕                          ↕                │ UT  ★ RGS-TST-UT-01 ★
       单元试验                证明               5服务DDL/协议  挂载脚手架  容量分片  请求链  集群部署  │  │
                                                                                    │
       实现                    —                       Rust 源码                  ┘  ┘  ┘
```

## 2.2 UT 验证的 DTL 字段（精修）

| DTL 文档 | 验证字段 | 占比目标 |
|---|---|---|
| RGS-DTL-001 §2.1 player_db DDL | account/character/session_epoch/ban 4 表字段类型/约束/索引 | 100% 字段 |
| RGS-DTL-001 §2.2 economy_db DDL | inventory/inventory_item/wallet/ledger/outbox 5 表 | 100% 字段 |
| RGS-DTL-001 §6 match_db DDL | match/match_participant/match_result 3 表 | 100% 字段 |
| RGS-DTL-001 §7 social_db DDL | friend/guild/guild_member 3 表 | 100% 字段 |
| RGS-DTL-001 §8 admin_db DDL | operation_audit/compensation_batch 2 表 | 100% 字段 |
| RGS-DTL-001 §4、RGS-DTL-001 §9 Protocol | 5 服务 gRPC 协议线格式字段 | 100% 字段 |
| RGS-DTL-001 §5.1 tick 循环 | tick_at_world/handle_input/apply_movement/process_combat 伪代码 | 函数签名 |
| RGS-DTL-001 §5.2 AOI | grid_index/notify_neighbors 伪代码 | 函数签名 |
| RGS-DTL-001 §10 状态机 | match 状态机、event 分发器、Saga | 全部迁移 |
| RGS-BAS-002 §4.1 模板目录 | 5 子目录 | 文件存在 |
| RGS-DTL-002 §2.1、RGS-DTL-002 §2.3、RGS-DTL-002 §2.4、RGS-DTL-002 §3 Helm/清单 | Deployment/NetworkPolicy/ServiceMonitor/ExternalSecret 4 YAML | 字段 |
| RGS-DTL-002 §6.1 Mount Record | frontmatter+body | YAML 字段 |
| RGS-DTL-002 §6.5 生命周期状态机 | lifecycle_state 5 态 | 全部迁移 |
| RGS-DTL-022 §3.1 路由 | shard_router 哈希 | 函数 |
| RGS-DTL-023 §3 管道 | 4 前处理 + 4 后处理 | 函数签名 |
| RGS-DTL-024 §3-§8 集群 | 清单 DAG 状态机 | 全部 |

## 2.3 覆盖率策略

| 维度 | 目标 | 验证手段 |
|---|---|---|
| 语句覆盖率（核心区域） | ≥ 80%（QA-001） | `cargo tarpaulin` / `cargo llvm-cov` |
| 分支覆盖率 | ≥ 70% | 同上 |
| 业务规则不变条件（BZ-001〜007） | 100%（QA-002） | proptest 1000 次 |
| 状态机禁止迁移（ST-000-1〜4） | 100%（QA-003） | 状态机试验用例 |
| DDL 字段级覆盖 | 100%（本版本新增） | sqlx-cli + 数据库 snapshot diff |

## 2.4 与其他测试设计书的边界

| 边界 | 不在本文档 | 归属 |
|---|---|---|
| 跨 crate 集成 | 5 服务端到端 | RGS-TST-IT-01 |
| 端到端业务 | 100k CCU 性能 | RGS-TST-ST-01 |
| 性能 | tick p99 实测 | RGS-TST-ST-01 |
| ADR 决策的端到端验证 | 跨服务 ADR 实施 | RGS-TST-IT-01 |

---

## 3. 测试用例

## 3.1 模块 A：player_db DDL（RGS-DTL-001 §2.1）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 |
|---|---|---|---|---|---|---|---|
| TST-UT-01-A001 | DTL-001 §2.1 account | id UUID PK, created_at TIMESTAMPTZ, status ENUM | 字段类型与约束 | N | CREATE TABLE + INSERT | 字段类型完全一致 | 字段类型 100% 匹配 |
| TST-UT-01-A002 | DTL-001 §2.1 character | id UUID, player_id UUID FK, name VARCHAR(64) | 角色表 | N | 同上 | FK 约束生效 | FK 拒绝孤儿 |
| TST-UT-01-A003 | DTL-001 §2.1 session_epoch | player_id UUID PK, epoch BIGINT, issued_at | epoch 发行 | N | 插入 epoch=1 → 2 → 3 | 单调递增 | ST-001 §8.1 epoch 字段正确 |
| TST-UT-01-A004 | DTL-001 §2.1 session_epoch | player_id, epoch | 旧 epoch 拒绝 | A | 写 epoch=1（已过期） | `Err(EpochMismatch)` | ARC-005 |
| TST-UT-01-A005 | DTL-001 §2.1 ban | player_id, banned_at, reason | 封禁字段 | N | INSERT + UPDATE | status=Active→Banned | FR-PL-005 |
| TST-UT-01-A006 | DTL-001 §2.1 ban | status 索引 idx_ban_status | 索引生效 | N | EXPLAIN | Index Scan | 索引存在 |

## 3.2 模块 B：economy_db DDL（RGS-DTL-001 §2.2）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-B001 | DTL-001 §2.2 inventory | player_id, capacity INT | 背包容量 | N |
| TST-UT-01-B002 | DTL-001 §2.2 inventory_item | item_id UUID, template_id, quantity BIGINT, version BIGINT | 道具字段 | N |
| TST-UT-01-B003 | DTL-001 §2.2 inventory_item | version + WHERE version=? | OCC | A |
| TST-UT-01-B004 | DTL-001 §2.2 wallet | player_id, currency_type, balance BIGINT, version | 货币字段 | N |
| TST-UT-01-B005 | DTL-001 §2.2 wallet | balance < 0 | BZ-001 拒绝 | A |
| TST-UT-01-B006 | DTL-001 §2.2 ledger | seq BIGSERIAL, player_id, delta BIGINT, occurred_at | 流水 | N |
| TST-UT-01-B007 | DTL-001 §2.2 ledger | APPEND ONLY | 不可改 | A |
| TST-UT-01-B008 | DTL-001 §2.2 outbox | event_id UUID, payload JSONB, published_at NULLABLE | outbox 字段 | N |
| TST-UT-01-B009 | DTL-001 §2.2 outbox | event_id 唯一约束 | 拒重 | A |
| TST-UT-01-B010 | DTL-001 §2.2 outbox | DR-011 同事务 | 原子性 | A |

## 3.3 模块 C：5 服务 gRPC 协议（RGS-DTL-001 §3）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-C001 | DTL-001 §3.1 PlayerService.Register | request{device_id, platform, client_version} | 注册协议 | N |
| TST-UT-01-C002 | DTL-001 §3.1 PlayerService.Login | request{account_id, credential_hash} | 登录协议 | N |
| TST-UT-01-C003 | DTL-001 §3.1 PlayerService.Logout | request{session_id} | 登出协议 | N |
| TST-UT-01-C004 | DTL-001 §3.2 EconomyService.Determine | request{request_id, player_id, session_epoch, op, expected_version} | 确定请求 | N |
| TST-UT-01-C005 | DTL-001 §3.2 EconomyService.Determine | request_id 唯一约束 | 幂等 | P |
| TST-UT-01-C006 | DTL-001 §3.3 MatchService.Queue | request{player_id, mode, mmr} | 匹配协议 | N |
| TST-UT-01-C007 | DTL-001 §3.3 MatchService.Settle | request{match_id, results[]} | 结算协议 | N |
| TST-UT-01-C008 | DTL-001 §3.4 SocialService.Friend | request{action, target_id} | 好友协议 | N |
| TST-UT-01-C009 | DTL-001 §3.4 SocialService.Guild | request{action, guild_id, role} | 公会协议 | N |
| TST-UT-01-C010 | DTL-001 §3.5 AdminService.BanAccount | request{operator_id, target_id, duration, reason} | 封禁 API | N |
| TST-UT-01-C011 | DTL-001 §3.5 AdminService.ReloadConfigTable | request{table_name, version} | 数值表热更 | N |
| TST-UT-01-C012 | DTL-001 §4.4 ResultCode 枚举 | 32 个错误码 | 完整枚举 | N |

## 3.4 模块 D：tick 循环伪代码（RGS-DTL-001 §5.1）

| 用例 ID | 对应设计 | 函数签名 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-D001 | DTL-001 §5.1 tick_at_world | `fn tick_at_world(world: &mut World, tick: u64) -> TickReport` | tick 入口 | N |
| TST-UT-01-D002 | DTL-001 §5.1 handle_input | `fn handle_input(world, input: PlayerInput) -> Result<()>` | 输入处理 | N |
| TST-UT-01-D003 | DTL-001 §5.1 apply_movement | `fn apply_movement(world, dt: Duration) -> Result<()>` | 移动 | N |
| TST-UT-01-D004 | DTL-001 §5.1 process_combat | `fn process_combat(world) -> Vec<CombatEvent>` | 战斗 | N |
| TST-UT-01-D005 | DTL-001 §5.1 update_aoi | `fn update_aoi(world) -> AoiReport` | AOI 更新 | N |
| TST-UT-01-D006 | DTL-001 §5.1 emit_diff | `fn emit_diff(world, baseline: Snapshot) -> Diff` | 差分生成 | N |
| TST-UT-01-D007 | DTL-001 §5.1 | 20Hz tick 间隔 | 调度精度 | B |
| TST-UT-01-D008 | DTL-001 §5.1 | 满载 300 实体 p99 < 25ms | 性能冒烟 | B |

## 3.5 模块 E：AOI 算法（RGS-DTL-001 §5.2）

| 用例 ID | 对应设计 | 函数签名 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-E001 | DTL-001 §5.2 grid_index | `fn grid_index_insert(idx, entity_id, x, y)` | 网格插入 | N |
| TST-UT-01-E002 | DTL-001 §5.2 grid_index | 跨网格移动 | 通知邻居 | N |
| TST-UT-01-E003 | DTL-001 §5.2 notify_neighbors | 边界 | 双向通知 | B |
| TST-UT-01-E004 | DTL-001 §5.2 G-003 | 9 宫格查询 | 算法正确 | N |
| TST-UT-01-E005 | DTL-001 §5.2 | 1000 实体 1ms 完成 | 性能冒烟 | B |

## 3.6 模块 F：状态机（RGS-DTL-001 §10）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-01-F001 | DTL-001 §10.1 match 状态机 ST-002 | Created→Waiting 合法 | N |
| TST-UT-01-F002 | DTL-001 §10.1 | Created→Finished 跳级 | S |
| TST-UT-01-F003 | DTL-001 §10.1 | Archived→* 全拒 | S |
| TST-UT-01-F004 | DTL-001 §10.2 Outbox 分发器 | pending→published | N |
| TST-UT-01-F005 | DTL-001 §10.3 Saga 状态 | Initiated→PaymentPending | N |
| TST-UT-01-F006 | DTL-001 §10.3 | Refunded→Delivered | S |
| TST-UT-01-F007 | DTL-001 §10.4 Trace 字段 | 6 ID 必填 | N |

## 3.7 模块 G：挂载脚手架（RGS-DTL-002 §2-§6）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-G001 | BAS-002 §4.1 模板目录 | services/helm/ci/db/obs | 5 子目录 | N |
| TST-UT-01-G002 | DTL-002 §2.1 Deployment | replicas, resources, liveness | YAML 字段 | N |
| TST-UT-01-G003 | DTL-002 §3 NetworkPolicy | podSelector, policyTypes=[Ingress,Egress] | 零信任 | N |
| TST-UT-01-G004 | DTL-002 §2.3 ServiceMonitor | endpoints, interval=15s | OTel | N |
| TST-UT-01-G005 | DTL-002 §2.4 ExternalSecret | secretStoreRef, refreshInterval | 密钥 | N |
| TST-UT-01-G006 | DTL-002 §6.1 Mount Record | frontmatter{id, type, owner} + body | 完整 | N |
| TST-UT-01-G007 | DTL-002 §6.5 状态机 | Idle→Mounting→Running | 合法 | N |
| TST-UT-01-G008 | DTL-002 §6.5 | Idle→Decommissioning | S |
| TST-UT-01-G009 | DTL-002 §6.5 | 同 cluster 并发 | 排他锁 | A |
| TST-UT-01-G010 | DTL-002 §6.4 退场安全网（幂等要求待 DTL/实现定义） | FT-013 基础 | P |

## 3.8 模块 H：容量分片（RGS-DTL-022 §3-§4）

| 用例 ID | 对应设计 | 函数 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-H001 | DTL-022 §3.1 shard_router | `fn route(player_id, num_shards) -> ShardId` | 一致性 hash | P |
| TST-UT-01-H002 | DTL-022 §3.2 T0=5万 | 配置正确 | N |
| TST-UT-01-H003 | DTL-022 §3.2 T1=20万 | 同 | N |
| TST-UT-01-H004 | DTL-022 §3.2 T2=100万 | 同 | N |
| TST-UT-01-H005 | DTL-022 §3.2 T3=1000万 | 同 | N |
| TST-UT-01-H006 | DTL-022 §3.3 list_cross_shard | 跨分片查询 | N |
| TST-UT-01-H007 | DTL-022 §4.1 HPA | CPU>70% 触发 | B |

## 3.9 模块 I：请求处理链（RGS-DTL-023 §3）

| 用例 ID | 对应设计 | 函数 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-I001 | DTL-023 §3.1 auth | `fn auth(req) -> Result<Principal>` | 鉴权 | N |
| TST-UT-01-I002 | DTL-023 §3.1 ratelimit | 100/秒 | 限流 | B |
| TST-UT-01-I003 | DTL-023 §3.1 validate | 字段校验 | A |
| TST-UT-01-I004 | DTL-023 §3.1 idempotent | 幂等键 | P |
| TST-UT-01-I005 | DTL-023 §3.2 redact | password→*** | A |
| TST-UT-01-I006 | DTL-023 §3.2 trace | span 注入 | N |
| TST-UT-01-I007 | DTL-023 §3.2 audit | GM 操作写 | N |
| TST-UT-01-I008 | DTL-023 §3.2 err | {code, message} | N |

## 3.10 模块 J：集群部署（RGS-DTL-024 §3-§8）

| 用例 ID | 对应设计 | 函数 | 测试目的 | 覆盖类型 |
|---|---|---|---|---|
| TST-UT-01-J001 | DTL-024 §3 ClusterManifest | cluster_name, apps[] | 清单 | N |
| TST-UT-01-J002 | DTL-024 §4 DAG | 拓扑排序 | N |
| TST-UT-01-J003 | DTL-024 §4 | A→B→A 循环检测 | A |
| TST-UT-01-J004 | DTL-024 §5 状态机 | Pending→Running→Succeeded | N |
| TST-UT-01-J005 | DTL-024 §5 | uq_deploy_runs_cluster_running 排他 | A |
| TST-UT-01-J006 | DTL-024 §6 dry-run | 不真部署 | N |
| TST-UT-01-J007 | DTL-024 §7 回滚 | 失败恢复 | A |
| TST-UT-01-J008 | DTL-024 §8 续跑 | 续上次步骤 | N |

## 3.11 模块 K：ADR 决策验证（新增）

本节验证每条 ADR 决定项的"实现位置 + 测试位置 + 守门位置"是否完备。

| 用例 ID | 对应 ADR | 决定项 | 实现位置（DTL） | 测试位置 | 守门位置 |
|---|---|---|---|---|---|
| TST-UT-01-K001 | RGS-ADR-0001 | Actor 粒度＝场景，玩家为 ECS 实体 | DTL-001 §5.1 tick_actor + §5.1 Entity | TST-UT-01-D001~006 + IT-01 §3.1 | lint: `assert_not_send<SceneActor>` |
| TST-UT-01-K002 | RGS-ADR-0002 | 状态同步+预测+和解 | DTL-001 §5.1 emit_diff + §3.1 QUIC | TST-UT-01-D006 + IT-01 | FT-007 |
| TST-UT-01-K003 | RGS-ADR-0007 | 道具与货币统合 | DTL-001 §2.2 inventory + wallet 同表空间 | TST-UT-01-B001~010 | DDL migration check |
| TST-UT-01-K004 | RGS-ADR-0008 | 中间件导入判定 | DTL-001 §1.3 + DTL-002 §3.3 | 静态扫描 | ADR 缺失 CI 阻断 |
| TST-UT-01-K005 | RGS-ADR-0015 | Saga 适用边界 | DTL-001 §10.3 Saga 状态 | TST-UT-01-F005~006 | lint: realtime_fn_not_call_saga |
| TST-UT-01-K006 | RGS-ADR-0020 | 拒绝动态链接库 | DTL-005 §3 沙箱 | TST-UT-02-062 (主题 02) | clippy: no_dlopen |
| TST-UT-01-K007 | RGS-ADR-0022 | 业务逻辑不入库 | DTL-007 §7 存储过程 | TST-UT-03-017~018 (主题 03) | migration check |
| TST-UT-01-K008 | RGS-ADR-0023 | 客户端核心逻辑单一实现 | DTL-008 §3 (主题 04) | TST-UT-04-001~004 (主题 04) | 三引擎一致性回归 |
| TST-UT-01-K009 | RGS-ADR-0026 | 仿生分层+智能层只读 | DTL-011 §4-§6 (主题 05) | TST-UT-05-010~017 (主题 05) | lint: smart_layer_not_write |
| TST-UT-01-K010 | RGS-ADR-0029 | 确定性分级 L0-L4 + 闸门 | DTL-011 §3-§6 (主题 05) | TST-UT-05-001~007 (主题 05) | 闸门静态检查 |

## 3.12 模块 L：业务规则与状态机（QA-002/003 门禁）

| 用例 ID | 业务规则 | 覆盖类型 |
|---|---|---|
| TST-UT-01-L001 | BZ-001 货币非负 | P |
| TST-UT-01-L002 | BZ-002 支付幂等 | P |
| TST-UT-01-L003 | BZ-003 流水复原 | P |
| TST-UT-01-L004 | BZ-004 客户端伤害被拒 | P |
| TST-UT-01-L005 | BZ-005 归档对局不可变 | P |
| TST-UT-01-L006 | BZ-006 封禁不可建会话 | P |
| TST-UT-01-L007 | BZ-007 交易原子性 | P |
| TST-UT-01-L010 | ST-001 Terminating→Active 拒 | S |
| TST-UT-01-L011 | ST-002 Finished→Running 拒 | S |
| TST-UT-01-L012 | ST-002 Archived→* 拒 | S |
| TST-UT-01-L013 | ST-003 Refunded→Delivered 拒 | S |
| TST-UT-01-L014 | ST-004 Settled→Draft 拒 | S |
| TST-UT-01-L015 | ST-005 Banned→Active 拒 | S |

## 3.13 模块 M：ARC-013 死锁防止 + CON-008 无界

| 用例 ID | 对应 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-01-M001 | BAS-001 §7.2.1 | A→B 不形成同步循环 | A |
| TST-UT-01-M002 | BAS-001 §7.2.1 | 跨 Actor 调用为非阻塞 | N |
| TST-UT-01-M003 | CON-008 | 全部 mailbox 构造时需 bound | N |
| TST-UT-01-M004 | CON-008 | 静态扫描禁用 `unbounded_channel` | A |

---

## 4. 追溯性矩阵（精修版）

| 详细设计章节 | 字段级 | 用例 ID 范围 | QA 门禁 |
|---|---|---|---|
| DTL-001 §2.1 player_db | 4 表字段 | TST-UT-01-A001~006 | QA-001 |
| DTL-001 §2.2 economy_db | 5 表字段 | TST-UT-01-B001~010 | QA-001, QA-002 |
| DTL-001 §6-§8 match/social/admin | 8 表字段 | 主题 03/07 用例 | QA-001 |
| DTL-001 §4、DTL-001 §9 5 服务 gRPC | 11 方法字段 | TST-UT-01-C001~012 | QA-001 |
| DTL-001 §4.4 ResultCode | 32 错误码 | TST-UT-01-C012 | QA-001 |
| DTL-001 §5.1 tick | 6 函数签名 | TST-UT-01-D001~008 | QA-001 |
| DTL-001 §5.2 AOI | 2 函数 | TST-UT-01-E001~005 | QA-001 |
| DTL-001 §10 状态机 | 4 状态机 | TST-UT-01-F001~007 | QA-003 |
| BAS-002 §4.1 模板 | 5 子目录 | TST-UT-01-G001 | QA-001 |
| DTL-002 §2.1、DTL-002 §2.3、DTL-002 §2.4、DTL-002 §3 清单 | 4 YAML 字段 | TST-UT-01-G002~005 | QA-001 |
| DTL-002 §6.5 状态机 | 5 态 | TST-UT-01-G007~009 | QA-003 |
| DTL-002 §6.4 退场安全网（幂等待定义） | FT-013 基础 | TST-UT-01-G010 | QA-003 |
| DTL-022 §3 路由 | hash 函数 | TST-UT-01-H001~006 | QA-001 |
| DTL-022 §4 HPA | 阈值 | TST-UT-01-H007 | QA-001 |
| DTL-023 §3 管道 | 8 函数 | TST-UT-01-I001~008 | QA-001 |
| DTL-024 §3-§8 集群 | 8 函数 | TST-UT-01-J001~008 | QA-001 |
| ADR-0001/0002/0007/0008/0015/0020/0022/0023/0026/0029 | 决定项 | TST-UT-01-K001~010 | QA-005 |
| BZ-* 7 条 | 不变条件 | TST-UT-01-L001~007 | QA-002 |
| ST-* 5 条 | 非法迁移 | TST-UT-01-L010~015 | QA-003 |
| ARC-013/CON-008 | 死锁/无界 | TST-UT-01-M001~004 | QA-001 |
| AC-004 全部禁止迁移 | — | TST-UT-01-L010~015 | 跨主题 |
| QA-001 覆盖率 | — | 全部 | 80% |
| QA-002 属性试验 | — | L001~007 + B003~010 | 1000 次 |
| QA-003 状态机 | — | L010~015 + F002~003/G008~009 | 100% |

---

## 5. 测试执行计划

| 触发 | 范围 | 时限 |
|---|---|---|
| 每次 commit | 受影响 crate | < 30s（本地） |
| 每次 PR | 全 workspace 全部 UT | < 5 min |
| 合并至 main | 全 + 属性 1000 次 | < 10 min（QA-006 内） |
| 每晚 nightly | UT + 属性 10000 次 + 模糊测试 | 不阻塞主干 |

## 5.1 测试夹具

| 夹具 | 路径 | 用途 |
|---|---|---|
| `fixtures/ddl_snapshot.sql` | `tests/fixtures/dtl-001/` | player_db/economy_db DDL snapshot |
| `fixtures/proto/5services.proto` | 同上 | 5 服务 proto 定义 |
| `fixtures/tick_seq.jsonl` | 同上 | tick 时序回放 |
| `fixtures/cluster_manifest.yaml` | `tests/fixtures/dtl-024/` | 集群清单样本 |

## 5.2 覆盖率门禁

- 核心区域语句覆盖 ≥ 80%（QA-001）
- 关键不变条件（BZ-001〜007）100% 覆盖（QA-002）
- 状态机非法迁移（ST-000-1〜4）100% 覆盖（QA-003）
- **DDL 字段级覆盖 100%（本版本新增）**
- **ADR 决定项验证 100%（本版本新增）**
- 不达标时 PR 检查（`cargo llvm-cov fail-under-lines 80`）阻断合并

---

## 6. 通过判定基准

| 维度 | 基准 |
|---|---|
| 所有用例 PASS | TST-UT-01-A001~M004 全部通过 |
| 语句覆盖率 | ≥ 80%（核心区域） |
| 业务规则属性 | proptest 默认 1000 次迭代无失败 |
| 状态机 | 全部禁止迁移被拒绝 |
| 静态检查 | `cargo clippy -- -D warnings` 通过 |
| 格式化 | `cargo fmt --check` 通过 |
| DDL 字段级 | 100% 字段类型与 DTL §2.1-2.5 一致 |
| ADR 决定项 | 全部 10 条 ADR 的决定项有对应实现+测试+守门 |
| 审计可追溯 | 每次失败用例须附 issue 链接（QA-005 复发防止） |

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
| TBD-CAP-001 | T3 多区域方案校准值 | 标记预留，UT 用保守假设"8 shard" |
| TBD-CAP-002 | 跨分片能力清单 | 标记预留，UT 仅测"list_cross_shard 返回全集" |
| TBD-PPL-001 | 限流算法参数 | 由 NFR-SEC-008 决定；UT 用"100/秒"占位 |
| TBD-DEP-001 | Schema 校验实现语言 | 标记预留，UT 用"任意实现" |
| TBD-DTL-001 | DTL 章节细化 | 本版本已细化到 §X.Y + 表/图/字段 |
| TBD-ADR-001 | 治理 CI 误判率 | 实测统计，UT 暂不设门禁 |

---

> 本文档为 RGS-TST 系列主题 01 单元测试设计书（**字段级深化版 0.2**）。其他主题（02-07）见对应 `RGS-TST-UT-02〜07_*.md`。
