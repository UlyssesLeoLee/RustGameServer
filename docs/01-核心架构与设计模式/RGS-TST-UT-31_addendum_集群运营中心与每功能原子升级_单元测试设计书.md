# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 31 集群运营中心与每功能原子升级 — 单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-31 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-031 addendum 需求定义书（ARC-051）、RGS-BAS-031 v0.1 addendum 基本设计书（11 章）、RGS-DTL-031 v0.3 详细设计书（2026-08-25 复核对账版）、RGS-ADR-0051 架构决定 |
| V模型层级 | TL-1 单元试验 ↔ RGS-BAS-031 v0.1 全部 11 章 + RGS-DTL-031 v0.3 复核对账子节（12 行 BAS→DTL 映射） |
| 依据标准 | IPA『共通フレーム 2013』詳細設計工程 |
| 制定日 | 2026-08-19（v0.1 初版）/ 2026-08-27（v0.2 升版） |
| 制定者 | 架构师（v0.1）/ 架构师（Mavis 接手 agent per DEC-008，v0.2） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 本主题域源文档全集 | RGS-REQ-031、RGS-BAS-031 v0.1（addendum）、RGS-DTL-031 v0.3（2026-08-25 复核对账版，per commit `5743747`）、RGS-ADR-0051 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。覆盖 feature_registry 元数据管理、CEM 探针解析、PFAU 状态机迁移、AdminService 转发逻辑、ClusterOpsService 内部模块 |
| 0.2 | 2026-08-27 | 架构师（Mavis 接手 agent per DEC-008）| 升版：① 头表映射基线 RGS-DTL-031 v0.1→v0.3（含 BAS-031 v0.1 全部 11 章 + DTL-031 v0.3 复核对账 12 行映射，per commit `5743747`）② §1.5 字段级映射目标从"BAS-031 §3 Schema + §4 状态机"升级为"BAS-031 v0.1 全部 11 章 + DTL-031 v0.3 复核对账子节 + DTL §1~§11" ③ 追加 10 条 v0.2 新增用例（G001~G010 — 覆盖 DTL-031 v0.3 复核对账 4 条结论的可 UT 化项：不可代签自检 / 实质重写判定 / 不引入新设计 / 修订历史完整） ④ 8 条"暂缓"用例（A020/C001/C003/C004/C008/C012/C013/ADR-003 — per commit `b74ccc3` 临时禁用 cluster-ops drill + saga + 移 18 个 tests 至 `tests-disabled/`，per RGS-DOCS-HEALTH-2026-08-27 §2.3 关键发现 INC-002 归因冲突） ⑤ 末尾追加 §5 "v0.2 增量（per DTL-031 v0.3 + b74ccc3）" |

> ⚠️ v0.2 含 8 条"暂缓"用例，per b74ccc3 + RGS-DOCS-HEALTH-2026-08-27 §2.3 关键发现（INC-002 归因冲突），待新开 RGS-INC-003 后恢复

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（QA） | | | UT 覆盖率与既有 QA-001 80% 阈值 |
| 审批（负责人） | | | 本测试设计书的基准化 |

---

## 目次

1. 前言
   1.1 目的
   1.2 适用范围
   1.3 关联文档
   1.4 记述规则（含 1.4.1 强度用语）
   1.5 字段级映射说明
   1.6 命名约定
2. 测试策略
3. 测试用例
   3.1 模块 A：feature_registry 元数据操作
   3.2 模块 B：CEM 探针订阅器事件解析
   3.3 模块 C：PFAU 状态机迁移
   3.4 模块 D：AdminService 转发到 ClusterOpsService
   3.5 模块 E：Feature 四种形态的差异化处理
   3.6 模块 F：RBAC 角色权限校验
   3.7 ADR-0051 决策验证
4. 追溯性矩阵
5. 测试执行计划
6. 通过判定基准
7. 风险与未决事项（TBD 处置）

---

## 1. 前言

## 1.1 目的

本文档为 V 模型中 **TL-1 单元试验**层级的设计书，对应主题 31（ARC-051 集群运营中心 + 中心事件管理 + 每功能原子升级）。本版本（0.1）核心：

- **元数据单元验证**：feature_registry 表的 CRUD、约束、版本历史不可变性
- **CEM 探针单元验证**：事件解析、Schema 校验、批量 UPSERT 逻辑
- **PFAU 状态机单元验证**：合法/非法迁移、超时、暂停、自动回滚触发条件
- **AdminService 转发单元验证**：转发到 ClusterOpsService 的入参/出参映射、RBAC 校验
- **ADR-0051 决策验证**：ARC-051 决定项的每条实现位置+测试位置+守门位置

## 1.2 适用范围

| 范畴 | 说明 |
|---|---|
| 适用 | ClusterOpsService 内部模块（FeatureRegistry、PFAURunner、CEMProbeAggregator、DLQOperator、ReplayOperator）、AdminService 转发逻辑、`admin_db` 新增表的操作 |
| 不适用 | 跨服务集成（COC UI ↔ AdminService ↔ ClusterOpsService）、端到端业务（PFAU 升级演练）、性能（1000 Feature 加载时延）——见 RGS-TST-IT-31 / RGS-TST-ST-31 |

## 1.3 关联文档

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-REQ-031 | 需求定义书（ARC-051） | 父需求 |
| RGS-BAS-031 | 基本设计书 | UT 验证对象 |
| RGS-ADR-0051 | 架构决定 | §3.7 验证 |
| RGS-BAS-002 | 功能挂载基本设计 | 复用既有`admin_db` 迁移规范 |
| RGS-BAS-003 | GM 后台基本设计 | AdminService 转发实现基准 |
| RGS-REQ-001 §12.2 | QA-001/002/003 | 覆盖率/属性/状态机门禁 |

## 1.4 记述规则

### 1.4.1 强度用语

本文档遵循 RFC 2119 与 IPA 共通フレーム 2013 规定的强度用语：

| 中文 | 日文 | 英文 | 强度 | 含义 |
|---|---|---|---|---|
| 必须 | 必須 | MUST / SHALL | 强 | 不可偏离的强制要求 |
| 不得 | …してはならない | MUST NOT / SHALL NOT | 强 | 不可偏离的禁止项 |
| 应当 | 望ましい | SHOULD | 中 | 强烈建议但有例外 |
| 不应当 | …すべきでない | SHOULD NOT | 中 | 强烈不建议但有例外 |
| 可以 | …してもよい | MAY | 弱 | 可选 |

### 1.4.2 覆盖类型符号

| 符号 | 含义 |
|---|---|
| N | 正常路径（Happy Path） |
| A | 异常路径（Abnormal） |
| B | 边界（Boundary） |
| S | 状态机迁移（State transition） |
| P | 性能（Performance 冒烟） |
| E | 错误注入（Error Injection） |

### 1.4.3 优先级符号

| 符号 | 优先级 |
|---|---|
| ◎ | 最高（必须 100% 通过） |
| ○ | 高（必须 ≥95% 通过） |
| △ | 中（可后续补） |
| × | 低/暂不实施 |

## 1.5 字段级映射说明

每条用例"对应设计"列格式：`<文档ID> §<章节> <表/图/字段名>`。

**v0.1 映射基线**（保留）：以 RGS-BAS-031 §3 Schema 与 §4 状态机为主要映射目标。

**v0.2 映射基线**（升版，per DTL-031 v0.3 复核对账）：升级为 **BAS-031 v0.1 全部 11 章 + DTL-031 v0.3 末尾"## 复核对账"12 行映射表 + DTL-031 §1~§11 全章字段级**。具体变更：

1. v0.1 单一 BAS 映射 → v0.2 双映射（BAS-031 v0.1 + DTL-031 v0.3），§3.1~§3.7 模块 A~F 既有 103 条用例保持稳定
2. 新增 §5.3 v0.2 增量 10 条用例（G001~G010）— 覆盖 DTL-031 v0.3 复核对账 4 条结论中可 UT 化项
3. 新增 §5.4 v0.2 暂缓 8 条用例 — per b74ccc3 + RGS-DOCS-HEALTH-2026-08-27 §2.3 关键发现（INC-002 归因冲突）
4. "不能据此宣告 UT 已实施"语义 v0.2 仍保留：RGS-DTL-031 v0.3 仍标"草案・待具名人类审批"，DTL 升版本身不解除 §11.1 6 条 Gate

## 1.6 命名约定

| 对象 | 命名格式 | 示例 |
|---|---|---|
| 单元测试用例 | `TST-UT-31-<模块>-<编号>` | TST-UT-31-A001 |
| 模块代号 | A: feature_registry / B: CEM 探针 / C: PFAU 状态机 / D: AdminService 转发 / E: Feature 形态 / F: RBAC | — |

---

## 2. 测试策略

### 2.1 V 模型映射

```
REQ-031 (FR-CEM-001~052, FR-PFAU-001~051, FR-COC-001~042, FR-API-001~012, FR-DB-001~004, FR-INT-001~005)
  │
  ▼
BAS-031 (§3 Schema, §4 状态机, §5 探针, §6 API 契约, §7 UI, §8 RBAC, §9 联动点)
  │
  ▼
本 UT 设计书 ────► 验证 §3 Schema 操作 / §4 状态机迁移 / §5 探针解析 / §6 API 字段级 / §8 RBAC
  │
  ▼
实现代码（ClusterOpsService 各模块）
```

### 2.2 测试方法

- 单元框架：Rust 内置 `#[test]` + `cargo test`
- Mock：使用 `mockall` 替代外部依赖（DB、事件总线、AdminService）
- 属性测试：使用 `proptest` 验证状态机迁移的不变量
- 覆盖率门禁：QA-001 ≥ 80% 行覆盖

---

## 3. 测试用例

## 3.1 模块 A：feature_registry 元数据操作（RGS-BAS-031 §3.1, §3.2）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-31-A001 | BAS-031 §3.1 feature_registry | feature_id TEXT PK, feature_type ENUM, status ENUM | 创建 Feature 正常路径 | N | INSERT 新 Feature | 写入成功, 字段完整 | 字段值 100% 匹配 | ◎ |
| TST-UT-31-A002 | BAS-031 §3.1 | feature_type 枚举 | 拒绝非法枚举值 | A | INSERT feature_type='invalid' | `Err(ConstraintViolation)` | ENUM CHECK 约束 | ◎ |
| TST-UT-31-A003 | BAS-031 §3.1 | status 枚举 | 拒绝非法状态 | A | INSERT status='bad_status' | `Err(ConstraintViolation)` | ENUM CHECK 约束 | ◎ |
| TST-UT-31-A004 | BAS-031 §3.1 | depends_on TEXT[] | 默认值空数组 | N | INSERT 不指定 depends_on | `depends_on = '{}'` | DEFAULT 生效 | ○ |
| TST-UT-31-A005 | BAS-031 §3.1 | created_at/updated_at TIMESTAMPTZ | 默认值 | N | INSERT 后立即查询 | `now()` 接近当前时间 | DEFAULT 生效 | ○ |
| TST-UT-31-A006 | BAS-031 §3.1 | updated_at 触发器 | UPDATE 时自动刷新 | N | UPDATE display_name | updated_at > 旧值 | 触发器生效 | ◎ |
| TST-UT-31-A007 | BAS-031 §3.2 feature_version_history | BIGSERIAL history_id | 主键自增 | N | INSERT 两条历史 | history_id 严格递增 | 自增正确 | ◎ |
| TST-UT-31-A008 | BAS-031 §3.2 | trigger `prevent_feature_version_history_modify` | UPDATE 拒绝 | A | 尝试 UPDATE 一行 | `Exception: append-only` | trigger 生效 | ◎ |
| TST-UT-31-A009 | BAS-031 §3.2 | 同上 | DELETE 拒绝 | A | 尝试 DELETE 一行 | `Exception: append-only` | trigger 生效 | ◎ |
| TST-UT-31-A010 | BAS-031 §3.2 | state 枚举 | 拒绝非法状态 | A | INSERT state='unknown' | `Err(ConstraintViolation)` | ENUM CHECK | ◎ |
| TST-UT-31-A011 | BAS-031 §3.1 | 索引 idx_feature_registry_type_status | 索引存在 | N | `\d feature_registry` | 索引存在 | 索引创建正确 | ○ |
| TST-UT-31-A012 | BAS-031 §3.1 | feature_id 引用 | 引用不存在的 feature_id | A | INSERT feature_version_history(feature_id='nonexistent') | FK 违反 | FK 约束 | ◎ |
| TST-UT-31-A013 | BAS-031 §3.3 pfa_run_state | run_id UUID PK | 创建 PFAU 实例 | N | INSERT 新 run | UUID 生成正确, PK 唯一 | 主键约束 | ◎ |
| TST-UT-31-A014 | BAS-031 §3.3 | state 枚举 | 状态机迁移合法 | S | declared → canary_in_progress | 迁移成功 | 状态机合法路径 | ◎ |
| TST-UT-31-A015 | BAS-031 §3.3 | state 枚举 | 状态机迁移非法 | S | declared → completed (跳过 canary) | `Err(InvalidStateTransition)` | 状态机非法路径 | ◎ |
| TST-UT-31-A016 | BAS-031 §3.3 | direction 枚举 | 拒绝非法方向 | A | INSERT direction='sideways' | `Err(ConstraintViolation)` | ENUM CHECK | ◎ |
| TST-UT-31-A017 | BAS-031 §3.3 | batch_size_pct INT[] | 总和=100 边界 | B | [25,25,25,25] | 校验通过 (总和=100) | 边界值 | ○ |
| TST-UT-31-A018 | BAS-031 §3.3 | batch_size_pct | 总和≠100 拒绝 | A | [30,30,30] | `Err(InvalidBatchSize)` | 校验逻辑 | ◎ |
| TST-UT-31-A019 | BAS-031 §3.3 | target_node_ids TEXT[] | 节点数边界 | B | 1 个节点 vs 1000 个节点 | 1 节点通过, 1000 节点通过 | 大小限制 | ○ |
| TST-UT-31-A020 | BAS-031 §3.3 | last_heartbeat_at | 跨节点确认超时 | B | 模拟 120 秒无心跳 | 状态变为 paused | NFR-COC 超时逻辑 | ◎ |

## 3.2 模块 B：CEM 探针订阅器事件解析（RGS-BAS-031 §5）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-31-B001 | BAS-031 §5.2 探针工作流 | event_type 解析 | 正常事件解析 | N | 输入合法事件 payload | event_type 正确提取 | 解析逻辑 | ◎ |
| TST-UT-31-B002 | BAS-031 §5.2 | event_type 缺失 | 解析失败不阻塞 | A | 输入无 event_type | 写"未注册事件"告警, 继续监听 | 鲁棒性 | ◎ |
| TST-UT-31-B003 | BAS-031 §5.2 | event_type 已注册 | UPSERT 触发 | N | 输入已注册 event_type | event_producer_registry UPSERT | 写入逻辑 | ◎ |
| TST-UT-31-B004 | BAS-031 §5.2 | event_type 未注册 | 告警但继续 | A | 输入未注册 event_type | 告警, 不阻塞 | RSK-COC-001 | ◎ |
| TST-UT-31-B005 | BAS-031 §5.3 | payload 丢弃 | 不消费 payload | N | 输入 1MB payload | 探针内部不持有 payload | 内存安全 | ◎ |
| TST-UT-31-B006 | BAS-031 §5.3 | 批量 UPSERT 5 秒窗口 | 5 秒内多次 UPSERT 合并 | B | 1 秒内 1000 个事件 | 1 次 DB UPSERT (合并后) | 批处理逻辑 | ○ |
| TST-UT-31-B007 | BAS-031 §5.3 | 独立 Consumer Group | 不共享 offset | N | 启动两个探针实例 | 各自独立 offset | FR-API-012 | ◎ |
| TST-UT-31-B008 | BAS-031 §5.3 | 独立 Consumer Group | 不阻塞正常消费者 | A | 探针慢处理 | 正常消费者 lag 不增加 | 隔离性 | ◎ |
| TST-UT-31-B009 | BAS-031 §5.3 | OTel 指标导出 | producer 速率指标 | N | 输入 100 事件/秒 | OTel 导出 rate=100 | 指标采样 | ○ |
| TST-UT-31-B010 | BAS-031 §5.3 | OTel 指标导出 | schema 命中率 | N | 100 事件 95 个已注册 | rate_hit=0.95 | 指标正确 | ○ |

## 3.3 模块 C：PFAU 状态机迁移（RGS-BAS-031 §4.2）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-31-C001 | BAS-031 §4.2 状态机 | declared → canary_in_progress | 合法迁移 | S | 启动 PFAU | 状态切换 | 状态机定义 | ◎ |
| TST-UT-31-C002 | BAS-031 §4.2 | canary_in_progress → canary_confirmed | 合法迁移（全部批次完成） | S | 全部批次成功 | 状态切换 | 状态机定义 | ◎ |
| TST-UT-31-C003 | BAS-031 §4.2 | canary_in_progress → paused | 合法迁移（节点确认超时） | S | 模拟 1 节点超时 | 状态切换, pause_reason='confirmation_timeout' | NFR-COC 超时 | ◎ |
| TST-UT-31-C004 | BAS-031 §4.2 | canary_confirmed → completed | 合法迁移（最终完成） | S | 全部确认 | 状态切换, feature_registry.current_version 更新 | 状态机完成 | ◎ |
| TST-UT-31-C005 | BAS-031 §4.2 | completed → * | 终态不可迁移 | S | 尝试 completed → canary_in_progress | `Err(InvalidStateTransition)` | 终态不可逆 | ◎ |
| TST-UT-31-C006 | BAS-031 §4.2 | rolled_back → * | 终态不可迁移 | S | 尝试 rolled_back → canary_in_progress | `Err(InvalidStateTransition)` | 终态不可逆 | ◎ |
| TST-UT-31-C007 | BAS-031 §4.2 | 灰度批次推进 | current_batch 递增 | N | 第 1 批成功 | current_batch=2 | 批次推进 | ◎ |
| TST-UT-31-C008 | BAS-031 §4.2 | 灰度批次推进 | 全部批次完成 | B | 5 批全部成功 | current_batch=5, state=canary_confirmed | 完成路径 | ◎ |
| TST-UT-31-C009 | BAS-031 §4.2 | 自动回滚触发 | 节点失联自动回滚 | S | 模拟 K8s Pod 异常退出 | 触发回滚, 回到 from_version | FR-PFAU-022 | ◎ |
| TST-UT-31-C010 | BAS-031 §4.2 | 自动回滚触发 | 节点失联不误触发 | N | 节点正常但慢响应 | 不触发自动回滚 | 触发条件准确 | ◎ |
| TST-UT-31-C011 | BAS-031 §4.2 | 灰度批次观察期 | 必须等满观察期 | B | 观察期 5 秒, 立即推进 | `Err(ObservationWindowNotElapsed)` | 观察期强制 | ◎ |
| TST-UT-31-C012 | BAS-031 §4.2 | 跨节点一致性确认 | 全部节点确认 | N | 全部目标节点上报 | 状态切换 | 确认逻辑 | ◎ |
| TST-UT-31-C013 | BAS-031 §4.2 | 跨节点一致性确认 | 单节点未确认 | A | 1 节点 120 秒未确认 | 状态切换到 paused | 超时逻辑 | ◎ |
| TST-UT-31-C014 | BAS-031 §4.2 | feature_version_history 追加 | 每次完成追加历史 | N | 完成一次升级 | history 表新增 1 行, state=active | 不可变历史 | ◎ |
| TST-UT-31-C015 | BAS-031 §4.2 | 依赖不兼容自动拒绝 | Feature A 升级会破坏 B | A | 模拟不兼容版本 | 状态机拒绝完成, 提示"需先升级 B" | FR-PFAU-050 | ◎ |
| TST-UT-31-C016 | BAS-031 §4.2 | 升级与回滚对称 | 回滚走类似状态机 | S | 回滚 from_version | 类似 upgrade 流程, 目标 from_version | 双向对称 | ◎ |
| TST-UT-31-C017 | BAS-031 §4.2 | 暂停后 retry | 人工 retry 继续 | S | 暂停后人工 retry | 状态从 paused → canary_in_progress | 人工介入路径 | ◎ |
| TST-UT-31-C018 | BAS-031 §4.2 | 暂停后 skip | 人工 skip 当前批次 | S | 暂停后人工 skip | 跳过当前批次, 推进下一批 | 人工介入路径 | ◎ |
| TST-UT-31-C019 | BAS-031 §4.2 | 暂停后 rollback | 人工 rollback | S | 暂停后人工 rollback | 启动回滚状态机 | 人工介入路径 | ◎ |
| TST-UT-31-C020 | BAS-031 §4.2 | 灰度策略按节点百分比 | 20% 拆分 | N | batch_size_pct=[20,20,20,20,20] | 第 1 批取 20% 节点 | 策略实现 | ◎ |

## 3.4 模块 D：AdminService 转发到 ClusterOpsService（RGS-BAS-031 §6.1）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-31-D001 | BAS-031 §6.1 gRPC 方法列表 | RegisterFeature | 转发入参映射 | N | AdminService 接收请求, 转发到 ClusterOpsService.RegisterFeature | 字段一致 | 转发逻辑 | ◎ |
| TST-UT-31-D002 | BAS-031 §6.1 | DeclareFeatureUpgrade | 流式响应 | N | 调用后返回 Server stream | 流正确返回 PfaRunStateUpdate | 流式契约 | ◎ |
| TST-UT-31-D003 | BAS-031 §6.2.1 | RegisterFeatureRequest | request_id 幂等 | P | 重复提交相同 request_id | 第二次返回首次结果 (IDEMPOTENT_REPLAY) | FR-API-003 | ◎ |
| TST-UT-31-D004 | BAS-031 §6.2.1 | feature_id 重复注册 | 拒绝 | A | 注册已存在的 feature_id | `Err(FEATURE_NOT_FOUND)` 或 `Err(AlreadyExists)` | 错误码 | ◎ |
| TST-UT-31-D005 | BAS-031 §6.2.2 | DeclareFeatureUpgradeRequest | 灰度策略字段映射 | N | strategy=BATCH_PCT, batch_size_pct=[20,20,20,20,20] | ClusterOpsService 正确接收 | 字段映射 | ◎ |
| TST-UT-31-D006 | BAS-031 §6.3 错误码 | FEATURE_NOT_FOUND | 错误码映射 | A | 查找不存在的 feature_id | NOT_FOUND | 错误码 | ◎ |
| TST-UT-31-D007 | BAS-031 §6.3 | FEATURE_TYPE_MISMATCH | 类型不匹配拒绝 | A | 对配置型 Feature 调用 plug | INVALID_ARGUMENT | 错误码 | ◎ |
| TST-UT-31-D008 | BAS-031 §6.3 | PFAU_ALREADY_RUNNING | 已有 PFAU 进行中 | A | 启动 PFAU 时已有进行中实例 | FAILED_PRECONDITION | 错误码 | ◎ |
| TST-UT-31-D009 | BAS-031 §6.3 | PFAU_INVALID_STATE | 状态机不允许 | A | 对 completed 调 rollback 但无历史版本 | FAILED_PRECONDITION | 错误码 | ◎ |
| TST-UT-31-D010 | BAS-031 §6.3 | EVENT_NOT_REGISTERED | event_type 未注册 | A | FR-CEM-002 触发 | FAILED_PRECONDITION | 错误码 | ◎ |
| TST-UT-31-D011 | BAS-031 §6.3 | REPLAY_DENIED | 无白名单 | A | ReplayEvents 不带白名单 | PERMISSION_DENIED | 错误码 | ◎ |
| TST-UT-31-D012 | BAS-031 §6.3 | RBAC_DENIED | 角色不足 | A | cluster_operator 调高危操作 | PERMISSION_DENIED | 错误码 | ◎ |
| TST-UT-31-D013 | BAS-031 §6.1 | 流式响应中断 | 客户端断连 | A | 客户端中途断连 | ClusterOpsService 状态机可恢复 (从 DB 读) | 持久化设计 | ◎ |
| TST-UT-31-D014 | BAS-031 §6.2.3 | ReplayEventsRequest | replay_request_id 幂等 | P | 重复提交相同 replay_request_id | 第二次不重放 (IDEMPOTENT_REPLAY) | FR-CEM-042 | ◎ |
| TST-UT-31-D015 | BAS-031 §6.2.3 | target_consumer_group_whitelist | 必填校验 | A | 不传白名单 | `Err(REPLAY_DENIED)` | FR-CEM-052 | ◎ |
| **TST-UT-31-D016** | BAS-031 §6.1 自审补强 | DiscardDlqEvent | 正常丢弃 | N | AdminService.DiscardDlqEvent → ClusterOpsService.DiscardDlqEvent | dlq_event_id 字段一致, 事件从 DLQ 物理删除 | FR-CEM-041 | ◎ |
| **TST-UT-31-D017** | BAS-031 §6.2.4 自审补强 | DiscardDlqEventRequest | discard_reason 必填 | A | 不传 discard_reason | `Err(DISCARD_DENIED)` | FR-CEM-041 | ◎ |
| **TST-UT-31-D018** | BAS-031 §6.2.4 自审补强 | DiscardDlqEventRequest | request_id 幂等 | P | 重复提交相同 request_id | 第二次返回首次结果 (IDEMPOTENT_REPLAY) | FR-API-003 | ◎ |
| **TST-UT-31-D019** | BAS-031 §6.1 自审补强 | DiscardDlqEvent | 事件不存在 | A | 丢弃不存在的 dlq_event_id | `Err(DLQ_EVENT_NOT_FOUND)` | 错误码 | ◎ |
| **TST-UT-31-D020** | BAS-031 §6.1 自审补强 | DiscardDlqEvent | 审计写入 | I | 丢弃成功后查 operation_audit | 含 dlq.discard 操作, 含 discard_reason | FR-COC-040 | ◎ |
| **TST-UT-31-D021** | BAS-031 §6.1 自审补强 | DiscardDlqEvent | RBAC | A | cluster_operator 尝试丢弃 | `Err(RBAC_DENIED)` | FR-COC-041 | ◎ |
| **TST-UT-31-D022** | BAS-031 §6.1 自审补强 | ListDlqEvents | 列表分页 | N | 查询时间窗内 DLQ 事件 | 返回按 dead_at 倒序, 支持分页游标 | FR-CEM-041 | ◎ |
| **TST-UT-31-D023** | BAS-031 §6.1 自审补强 | ListDlqEvents | 按 Topic 筛选 | N | 指定 topic 过滤 | 仅返回该 topic 事件 | FR-CEM-041 | ◎ |

## 3.5 模块 E：Feature 四种形态的差异化处理（RGS-BAS-031 §4.1）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-31-E001 | BAS-031 §4.1 限界上下文型 | feature_type=BOUNDED_CONTEXT | ARC-018 联动 | N | ARC-018 挂载完成自动创建 Feature | Feature 自动创建, 字段完整 | FR-INT-001 | ◎ |
| TST-UT-31-E002 | BAS-031 §4.1 | 插件型 | feature_type=PLUGIN | ARC-021 联动 | N | ARC-021 注册插件自动创建 Feature | Feature 自动创建 | FR-INT-002 | ◎ |
| TST-UT-31-E003 | BAS-031 §4.1 | 补丁型 | feature_type=PATCH | 走 FF off/on 语义 | N | 上线补丁 Feature | 不触发 K8s 镜像回退, 仅切 FF | FR-PFAU-040~042 | ◎ |
| TST-UT-31-E004 | BAS-031 §4.1 | 补丁型 | 回滚语义 | 回滚仅切 FF | N | 回滚补丁 Feature | FF 关闭, K8s 镜像不变化 | FR-PFAU-042 | ◎ |
| TST-UT-31-E005 | BAS-031 §4.1 | 配置型 | feature_type=CONFIG | ARC-016 联动 | N | 配置条目变更 | Feature 自动创建 | ARC-016 联动 | ◎ |
| TST-UT-31-E006 | BAS-031 §4.1 | 补丁型 | 强制门禁 | FR-PFAU-041 | A | 未经自动化测试门禁的补丁 | 拒绝进入灰度 | 门禁逻辑 | ◎ |
| TST-UT-31-E007 | BAS-031 §4.1 | 插件型 | ARC-021 拒绝动态库 | RGS-ADR-0020 | A | 尝试上传 .so/.dll | 拒绝, 错误信息指明 ADR-0020 | ADR-0020 守门 | ◎ |
| TST-UT-31-E008 | BAS-031 §4.1 | 补丁型 | 强制门禁 | FR-PFAU-041 补丁不得改既有 API | A | 补丁尝试修改既有 gRPC 方法 | CI 校验失败 | ARC-015 | ◎ |

## 3.6 模块 F：RBAC 角色权限校验（RGS-BAS-031 §8）

| 用例 ID | 对应设计 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-UT-31-F001 | BAS-031 §8 RBAC | cluster_operator 读权限 | 可读功能矩阵 | N | cluster_operator 调用 ListFeatures | 返回结果 | 角色权限 | ◎ |
| TST-UT-31-F002 | BAS-031 §8 | cluster_operator 写权限 | 可单 Feature plug | N | cluster_operator 调用 plug 单 Feature | 成功 | FR-COC-041 | ◎ |
| TST-UT-31-F003 | BAS-031 §8 | cluster_operator 拒绝高危 | 拒绝按 Feature 回滚 | A | cluster_operator 调 RollbackFeature | RBAC_DENIED | FR-COC-041 | ◎ |
| TST-UT-31-F004 | BAS-031 §8 | cluster_admin 高危操作 | 可按 Feature 回滚 | N | cluster_admin 调 RollbackFeature | 成功 | FR-COC-041 | ◎ |
| TST-UT-31-F005 | BAS-031 §8 | cluster_admin 批量操作 | 可批量升级 (≤20) | N | cluster_admin 批量 10 个 Feature | 成功 | FR-COC-012 | ◎ |
| TST-UT-31-F006 | BAS-031 §8 | 批量上限 20 | 拒绝 21 个 | A | cluster_admin 批量 21 个 Feature | `Err(BatchSizeExceeded)` | FR-COC-012 | ◎ |
| TST-UT-31-F007 | BAS-031 §8 | viewer 角色 | 全部写操作拒绝 | A | viewer 调任何写操作 | RBAC_DENIED | 既有 viewer 角色 | ◎ |
| TST-UT-31-F008 | BAS-031 §8 | 角色继承 | cluster_operator 不继承 operator | N | cluster_operator 尝试封禁账号 | RBAC_DENIED | 角色不继承 | ◎ |
| TST-UT-31-F009 | BAS-031 §8 | 二次确认 | 高危操作触发 | N | cluster_admin 调 RollbackFeature | 返回需要二次确认 | FR-COC-021 | ◎ |
| TST-UT-31-F010 | BAS-031 §8 | DLQ 重放 | 仅 cluster_admin | A | cluster_operator 调 ReplayEvents | RBAC_DENIED | FR-COC-041 | ◎ |

## 3.7 ADR-0051 决策验证

| 用例 ID | 对应决策项 | 验证内容 | 实现位置 | 测试位置 | 守门位置 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-UT-31-ADR-001 | §2 决定 1: Feature 为统一操作单元 | 全部 Feature 走 feature_registry, ARC-018/021/042 各自流程均通过 | BAS-031 §3.1, §4.1 | TST-UT-31-A001, E001~E005 | BAS-031 §9 强制联动点 | ◎ |
| TST-UT-31-ADR-002 | §2 决定 2: CEM 六部分齐全 | 事件注册表/Schema/订阅/健康/DLQ/重放均实现 | BAS-031 §5, §3.4, §3.5 | TST-UT-31-B001~B010 | IT-31 + ST-31 | ◎ |
| TST-UT-31-ADR-003 | §2 决定 3: PFAU 状态机定义 | 8 状态迁移合法/非法路径全覆盖 | BAS-031 §4.2 | TST-UT-31-C001~C020 | ARC-051 状态机 | ◎ |
| TST-UT-31-ADR-004 | §2 决定 4: COC UI 不另开凭证 | 全部写操作经 AdminService | BAS-031 §6.1, §9 | TST-UT-31-D001~D015 | 渗透测试 (NFR-OPS-004) | ◎ |
| TST-UT-31-ADR-005 | §2 决定 5: 声明式+流式gRPC+幂等 | request_id 幂等, 流式响应 | BAS-031 §6.2 | TST-UT-31-D002, D003, D014 | FR-API-002~004 | ◎ |
| TST-UT-31-ADR-006 | §2 决定 6: DB 侧三类协同 | Outbox/FF/版本快照 trigger 实现 | BAS-031 §3.2 trigger | TST-UT-31-A008, A009 | FR-DB-001~004 | ◎ |
| TST-UT-31-ADR-007 | §3.1 否决: COC UI 不独立 | 共用 AdminService 凭证 | BAS-031 §6.1, §8 | TST-UT-31-D001 | NFR-OPS-004 | ◎ |
| TST-UT-31-ADR-008 | §3.2 否决: 自动重试失败节点 | 暂停 + 人工三选一 | BAS-031 §4.2 | TST-UT-31-C017~C019 | FR-PFAU-010 | ◎ |
| TST-UT-31-ADR-009 | §3.3 否决: CEM 不分库 | event_schema_registry 在 admin_db | BAS-031 §3.4 | TST-UT-31-A011 (索引) | ARC-008 | ◎ |
| TST-UT-31-ADR-010 | §3.4 否决: COC 不直调 Helm | COC UI → AdminService → ClusterOpsService → Helm | BAS-031 §6.1, §9 | TST-UT-31-D001 | 渗透测试 | ◎ |
| TST-UT-31-ADR-011 | §3.5 否决: 补丁型不传动态库 | 拒绝 .so/.dll 上传 | BAS-031 §4.1 插件型路径 | TST-UT-31-E007 | RGS-ADR-0020 守门 | ◎ |
| TST-UT-31-ADR-012 | §3.6 否决: COC 不作 VIZ 子页 | 顶级页面, 复用 VIZ 渲染 | BAS-031 §7.1 路由, §7.2 | 静态检查 (CI) | 路由表验证 | ◎ |

---

## 4. 追溯性矩阵

| 用例模块 | 覆盖的 REQ ID | 覆盖的 BAS ID | 覆盖的 ADR ID | 覆盖的 NFR ID | 覆盖的 AC ID |
|---|---|---|---|---|---|
| 3.1 元数据 (A001~A020) | FR-PFAU-001, 002, 003, 020, 021 | BAS-031 §3.1~§3.3 | ADR-0051 §3.3, §3.9 | NFR-COC-001, 003 | AC-COC-001, 002 |
| 3.2 探针 (B001~B010) | FR-CEM-001, 020, 030 | BAS-031 §3.4, §5 | ADR-0051 §3.3 | NFR-COC-005 | AC-COC-004 |
| 3.3 状态机 (C001~C020) | FR-PFAU-010, 011, 020, 021, 030, 050 | BAS-031 §4.2 | ADR-0051 §3.2 | NFR-COC-003, 004 | AC-COC-002, 003 |
| 3.4 转发 (D001~D015) | FR-API-001, 002, 003, 004, 005 | BAS-031 §6 | ADR-0051 §3.4 | NFR-COC-007 | AC-COC-006 |
| 3.5 Feature 形态 (E001~E008) | FR-PFAU-001, 040, 041, 042 / FR-INT-001, 002 | BAS-031 §4.1 | ADR-0051 §2.1, §3.5 | — | AC-COC-002, 003 |
| 3.6 RBAC (F001~F010) | FR-COC-001, 020, 021, 040, 041, 042 | BAS-031 §8 | ADR-0051 §3.1, §3.4 | NFR-COC-007, 008 | AC-COC-006 |

---

## 5. 测试执行计划

| 阶段 | 用例范围 | 预计时长 | 前置条件 |
|---|---|---|---|
| **Phase 1: 单元冒烟** | A001~A020, B001~B010, C001~C020 | 0.5 天 | admin_db 测试实例 + ClusterOpsService 内部模块 Mock |
| **Phase 2: 状态机属性测试** | C001~C020 用 proptest 扩 | 0.5 天 | Phase 1 通过 |
| **Phase 3: ADR 决策验证** | ADR-001~ADR-012 | 0.5 天 | Phase 1, 2 通过 |
| **Phase 4: 覆盖率收口** | 全部 + 覆盖率补齐 | 0.5 天 | Phase 3 通过 |

总计: **2 天**

---

## 6. 通过判定基准

| 门禁 | 阈值 |
|---|---|
| 全部 ◎ 用例 | 100% 通过 |
| 全部 ○ 用例 | ≥95% 通过 |
| 行覆盖率（QA-001） | ≥80% |
| 分支覆盖率 | ≥70% |
| 状态机迁移覆盖 | 合法/非法路径各 100% |
| ADR 守门 | 全部决策项有实现+测试+守门 |

---

## 7. 风险与未决事项（TBD 处置）

| TBD ID | 内容 | UT 处置 |
|---|---|---|
| TBD-COC-001 | 无限画布前端选型 | **按保守假设实施** — UT 假设前端组件通过 props 接收渲染数据, 不绑定具体库 |
| TBD-COC-002 | 补丁型金丝雀测试门禁 | **标记为预留** — TST-UT-31-E006 暂以"自动化测试门禁"统称, 详细阈值在 PH-7 实测后定 |
| TBD-COC-003 | 批量回滚上限 20 | **已实施** — TST-UT-31-F006 验证上限 20 拒绝 21 |
| TBD-COC-004 | 事件注册变更事件保留 | **按保守假设实施** — UT 假设为系统级事件, schema_version 强制 |
| TBD-COC-005 | feature_registry 分区策略 | **待前置条件** — 留给 PH-7 详细设计阶段 |
| TBD-COC-006 | pfa_run_state 历史归档 | **待前置条件** — UT 不验证归档, 集成测试验证 |
| RSK-COC-001 | CI 校验脚本 check-cem-coverage.sh | **已实施** — TST-UT-31-B003, B004 验证 fail-fast 行为 |
| RSK-COC-002 | PFAU 超时阈值 120 秒 | **已实施** — TST-UT-31-A020, C013 验证 |
| RSK-COC-003 | COC UI 弱化 RGS-OPS-001 | **不在 UT 范围** — 由 UX 评审 + 集成测试覆盖 |

---

> 本文档配套 RGS-REQ-031 需求定义书、RGS-BAS-031 基本设计书、RGS-ADR-0051 架构决定。后续将产出 RGS-TST-IT-31（集成）与 RGS-TST-ST-31（系统）。

---

## 5. v0.2 增量（per DTL-031 v0.3 + b74ccc3）

> 本节为 v0.2 升版新增的**纯追加**章节，v0.1 §1~§4（103 条用例）+ §5~§7（执行计划 / 通过判定 / TBD）原样保留。v0.1 既有用例不删除、不修改，仅在新追加条目中按"对应 v0.1 章节"列做关联标注，便于追溯。

### 5.1 背景与变更依据

v0.2 升版由两条独立变更触发，二者均不重写 DTL/BAS 既有设计，仅做**测试设计书的同步对账 + 暂缓标记**：

1. **DTL-031 v0.2 → v0.3**（per commit `5743747`，2026-08-25，作者：架构师（Mavis 接手 agent per DEC-008））
   - 实质重写 = **否**：本批次父 BAS-031 实质未升版（per git log，父 RGS-BAS-031 末次变更 2026-08-21 commit `adb3e34` 仅装饰性修订：`admin_db` 锚点大小写归一、§9.1 联动点指向占位改为指向 DTL-031、结尾段加入 DTL v0.1 草案已形成说明，版本号仍为 v0.1，修订历史仅 1 行）
   - DTL-031 v0.3 实质变更仅 3 项元数据层：① 头表版本号 0.2→0.3 ② 修订历史表追加 v0.3 行 ③ 末尾追加"## 复核对账"子节（12 行 BAS-031 章节→DTL-031 章节映射表 + 4 条复核结论：实质重写判定 / 不可代签自检 / 不引入新设计 / 开放项）
   - DTL-031 v0.3 头部声明"本文落实 DEC-001~004 的候选实现规则"，正文不重写；§1~§11 字段与 v0.2 实质等价
2. **b74ccc3 临时禁用 cluster-ops drill + saga**（per commit `b74ccc3`，2026-08-27 07:36 JST）
   - 移除 `crates/cluster-ops/src/realm_lifecycle/drill/*`（6 文件，编译失败）+ `crates/cluster-ops/src/realm_lifecycle/saga/*`（4 文件，编译失败）+ 18 个 tests 移至 `crates/cluster-ops/tests-disabled/`（`drill_chaos.rs` / `drill_lcm_001~008_010.rs` / `drill_nfr.rs` / `drill_risk.rs` / `fail_closed_start.rs` / `it_cross_domain.rs` / `load_snapshot.rs` / `ut_feature_adapter.rs` / `ut_olu.rs` / `ut_saga.rs` / `ut_state_machine.rs`）
   - **INC-002 归因冲突**（per RGS-DOCS-HEALTH-2026-08-27 §2.3 关键发现）：b74ccc3 commit message 标"per RGS-INC-002 v0.1 复盘"是**归因错误**——RGS-INC-002 v0.1 复盘（`docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`，commit `43e6108`，全文 219 行）主题为"5 域 + cluster-ops 6 份 gRPC server 在 WSL k3s 上从 binary exit 1 → 6/6 pod 1/1 Running 0 RESTARTS + 6/6 gRPC TCP-OK"，**全文 0 处提及** `drill` / `saga` / `编译死锁`（已 `Select-String` 实证）。INC-002 复盘与 cluster-ops drill/saga 编译死锁是**两个独立事故**
   - b74ccc3 实际未决项（per commit body）：① `drill/executor.rs` 补 19 个 SagaStepKind variant（InitDirectory 等）② `saga/steps.rs` 改 SagaStep struct（替换 trait，与 playbook.rs 假设一致）③ `saga/SagaStep::new(phase, kind)` 关联函数实现 ④ `saga/SagaPhase + StepStatus + RealmId` type 定义 ⑤ 18 个 tests 重新启用
   - 恢复前置条件：建议新开 `RGS-INC-003_集群运营中心_drill_saga_编译死锁复盘` 归档真正根因后，再恢复 18 个 tests 至 `tests/` 目录（**非本 UT 设计书职责范围**，本设计书仅做"暂缓"标记 + 引用归档位置）

### 5.2 映射基线升版（v0.1 §1.5 / §2.1 / 头表 8 行）

| 映射维度 | v0.1 基线 | v0.2 基线（per DTL-031 v0.3） |
|---|---|---|
| 头表版本 | 0.1 | 0.2 |
| V 模型层映射 | TL-1 ↔ RGS-DTL-031 v0.1 草案（字段级映射与审批待完成） | TL-1 ↔ RGS-BAS-031 v0.1 全部 11 章 + RGS-DTL-031 v0.3 复核对账子节（12 行 BAS→DTL 映射） |
| 源文档全集 | RGS-REQ-031、RGS-BAS-031、RGS-ADR-0051、（待）RGS-DTL-031 | RGS-REQ-031、RGS-BAS-031 v0.1（addendum）、RGS-DTL-031 v0.3（2026-08-25 复核对账版）、RGS-ADR-0051 |
| §1.5 主要映射目标 | BAS-031 §3 Schema + §4 状态机 | BAS-031 v0.1 全部 11 章 + DTL-031 v0.3 复核对账 12 行 + DTL §1~§11 |
| 追溯性矩阵 | §3.1~§3.7 模块 A~F 映射 BAS §3~§6/§8 | 模块 A~F 既有 103 条用例保持稳定；§5.3/§5.4 追加 18 条 v0.2 增量映射至 DTL-031 v0.3 复核对账子节 + b74ccc3 暂缓范围 |
| 既有用例数量 | 103 条（A001~A020 + B001~B010 + C001~C020 + D001~D023 + E001~E008 + F001~F010 + ADR-001~ADR-012） | 103 条 v0.1 + 10 条 v0.2 新增（G001~G010）+ 8 条 v0.2 暂缓（v0.1 既有用例标"暂缓 per b74ccc3"） |

### 5.3 v0.2 新增测试用例（覆盖 DTL-031 v0.3 复核对账 4 条结论的可 UT 化项）

> 本节 10 条新增用例（G001~G010），覆盖 DTL-031 v0.3 末尾"## 复核对账"子节中 4 条复核结论的**可单元测试化**部分。命名规则：模块代号 G（v0.2 Governance / 治理对账模块），不复用 v0.1 模块 A~F（避免与既有元数据 / 探针 / 状态机 / 转发 / Feature / RBAC 用例混淆）。

| 用例 ID | 对应 DTL-031 v0.3 | 字段级 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| **TST-UT-31-G001** | v0.3 末尾"## 复核对账"-不可代签自检 | BAS-031 §1.2 / DTL-031 §1.2 不可代签红线 | CI 静态扫描 BAS-031 + DTL-031 修订历史"审批者"列 | N | 解析 BAS-031 + DTL-031 修订历史表，校验"审批者"列非空且非代签短语（"Mavis"/"agent" 单字签发） | DEC/ADR/NOGO 决策类文件"审批者" = "—"（不预填任何姓名）；SPEC/DTL/报告类"审批者" = 具名（含 "架构师（Mavis 接手 agent per DEC-008）"）| 治理红线自检 | ◎ |
| **TST-UT-31-G002** | v0.3 §"复核结论"-实质重写判定 | BAS-031 v0.1 + DTL-031 v0.3 git diff | DTL 升版正文未重写 BAS | N | 拉取 BAS-031 v0.1（commit `adb3e34`）+ DTL-031 v0.3（commit `5743747`）全文，章节拓扑比对 | DTL §1~§11 全在 BAS 11 章范围；DTL 净增行仅元数据层 + 复核对账表 | 实质重写 = 否 | ○ |
| **TST-UT-31-G003** | v0.3 §"复核结论"-不引入新设计 | DTL-031 v0.2→v0.3 净增行 | v0.3 不引入新需求/设计/决策 | N | git diff v0.2→v0.3，分类净增行（需求 / 设计 / 决策 / 元数据） | 元数据层（版本号 / 修订历史 / 复核对账表）100% 占比；新需求/新设计/新决策 = 0 行 | 不引入新设计 | △ |
| **TST-UT-31-G004** | v0.3 头表"v0.1→v0.2→v0.3" | 修订历史表 3 行 | 修订历史完整 + 不可代签 | N | 解析 BAS-031 + DTL-031 修订历史表，校验行数 = 当前版本号 + 1（如 v0.3 应有 3 行 v0.1/v0.2/v0.3） | 3 行存在；"审批者"列符合 G001 红线 | 文档完整性 | ◎ |
| **TST-UT-31-G005** | DTL-031 v0.3 §1.2 五条硬禁止 | 不新建控制面数据库 / 不绕过 AdminService / 不强制改 Publisher SDK / 不绕过 ADR-0020 / 不绕过 ARC-008 | 五条硬禁止各 1 UT 覆盖 | N | 单元 mock 各禁止场景（尝试新建 DB / 直调 COC 写 / 改 SDK / 上传 .so / 跳过 ARC-008）| 五条全禁止，且错误信息指明对应 ADR | 硬约束 5/5 | ◎ |
| **TST-UT-31-G006** | DTL-031 v0.3 §2.1 组件图 | 7 个节点：ClusterOpsService / FeatureRegistry / PFAURunner / CEMProbeAggregator / DLQOperator / ReplayOperator / Redis 短租约 + 既有 GitOps | 组件图节点 100% 覆盖 UT | N | 对照组件图 7 节点各写 ≥1 UT（v0.1 §3.1~§3.6 已有 6 节点覆盖，需补 Redis 短租约节点 UT）| 节点 7/7 覆盖（v0.1 已覆盖 6，新增 Redis 短租约 1）| 拓扑覆盖 | ○ |
| **TST-UT-31-G007** | DTL-031 v0.3 §3.1 "不另起一套表名，只补充字段语义" | 7 张表/视图：`feature_registry` / `feature_version_history` / `pfa_run_state` / `event_schema_registry` / `event_producer_registry` / `event_dlq_view` / `coc_audit_view` | 七表全在 admin_db 落位 | N | CI 扫描 7 表 DDL 路径（应在 `crates/admin-service/migrations/` 或 `crates/cluster-ops/migrations/`，per RGS-BAS-002 既有规范）| 7/7 落在 admin_db 迁移目录 | 表归属 | ◎ |
| **TST-UT-31-G008** | DTL-031 v0.3 §6.3 强制联动 4 条 | ARC-018 注册 bounded_context Feature / ARC-021 注册 plugin Feature / ARC-042 Helm Release 后必须 `NotifyFeatureDeployed` / 全部 COC 写经 AdminService 转发 | 4 条联动 UT 验证 | N | 单元 mock ARC 注册事件，校验 FeatureRegistry 自动创建/上报路径 | 联动触发 Feature 创建/上报 4/4 | 强制联动 | ◎ |
| **TST-UT-31-G009** | DTL-031 v0.3 §7.2 错误语义 | 10 个错误码：`ALREADY_EXISTS` / `ABORTED` / `FAILED_PRECONDITION` / `DEADLINE_EXCEEDED` / `PERMISSION_DENIED` / `FEATURE_NOT_FOUND` / `PFAU_*` / `EVENT_NOT_REGISTERED` / `REPLAY_DENIED` / `DISCARD_DENIED` / `IDEMPOTENT_REPLAY` | 错误码覆盖 | A | 单元触发每错误码 1 次 | 10/10 错误码可触发 | 错误码全 | ◎ |
| **TST-UT-31-G010** | DTL-031 v0.3 §11.3 开放项 + §11.1 Gate | TBD-COC-001/002 + OLU（DEC-001/003/004 叠加） + 6 项 §11.1 Gate（QA-001 具名审批 / Q-003 三场景验证 / Q-004 ARC 组合 / Q-025 字段级 DD Review / ADR-0052 Active-Active / 5 项 DTL 契约评审）| 开放项 UT 状态 | N | 扫描 §11.1 6 项 Gate 当前状态 | 6/6 仍"待 Gate 批准"；v0.3 升版不解除任何 Gate | 开放项状态 | ○ |

### 5.4 v0.2 暂缓用例（per b74ccc3 + RGS-DOCS-HEALTH-2026-08-27 §2.3 关键发现 — INC-002 归因冲突）

> v0.1 addendum §3.3 PFAU 状态机 + §3.1 A020 last_heartbeat_at + §3.7 ADR-003 等用例的设计意图**依赖** `crates/cluster-ops/tests/` 下 `ut_saga.rs` / `ut_state_machine.rs` / `drill_lcm_001~008_010.rs` / `drill_chaos.rs` / `drill_nfr.rs` / `drill_risk.rs` 等 backing tests 作为 `cargo test -p cluster-ops` 的可执行载体。b74ccc3 将 18 个该类 tests 移至 `crates/cluster-ops/tests-disabled/`，因此以下 8 条用例**当前无法 `cargo test -p cluster-ops` 通过**，本节标"暂缓"。
>
> **INC-002 归因冲突说明**（per RGS-DOCS-HEALTH-2026-08-27 §2.3 关键发现）：b74ccc3 commit message 标"per RGS-INC-002 v0.1 复盘"是**归因错误**——RGS-INC-002 v0.1 复盘（`docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`，commit `43e6108`）主题为 k3s 部署（5 域 + cluster-ops 6 份 gRPC server 在 WSL k3s 上 binary exit 1 → 6/6 pod 1/1 Running 0 RESTARTS + 6/6 gRPC TCP-OK），**全文 0 处提及** `drill` / `saga` / `编译死锁`（已 `Select-String -Pattern 'drill|编译死锁|saga' RGS-INC-002_...v0.1.md` 实证无匹配）。INC-002 复盘与 cluster-ops drill/saga 编译死锁是**两个独立事故**，真正的修复依据在 b74ccc3 commit body 列出的 5 条"未决项"（`drill/executor.rs` 补 19 个 SagaStepKind variant + `saga/steps.rs` 改 SagaStep struct + `saga/SagaStep::new(phase, kind)` 关联函数 + `saga/SagaPhase + StepStatus + RealmId` type 定义 + 18 个 tests 重新启用）。**注意 ID 复用**：`RGS-INC-002` 编号被两份不同文档复用——本事故复盘报告，以及 `docs/01-核心架构与设计模式/RGS-INC-002_Phase_0.5_启动计划_v0.1.md`（内容无关，仅编号撞车）；引用时务必带完整路径。
>
> **处置路径**（不在本 UT 设计书职责范围，仅做记录）：建议**新开** `RGS-INC-003_集群运营中心_drill_saga_编译死锁复盘`，归档 b74ccc3 真正根因（`drill/executor.rs` 缺 19 个 SagaStepKind variant / `saga/steps.rs` SagaStep struct 与 playbook.rs 假设不一致等），由 cluster-ops 域在 RGS-INC-003 批准后实施修复并恢复 18 个 backing tests 至 `tests/` 目录；本设计书暂缓用例待 RGS-INC-003 批准 + 18 个 tests 重新启用后，由 v0.3 升版恢复标"可执行"。

| 用例 ID | 原 v0.1 章节 | 对应 backing tests（当前在 `tests-disabled/`）| 暂缓原因 | 暂缓期参考 | INC-002 归因冲突备注 |
|---|---|---|---|---|---|
| **TST-UT-31-A020**（标"暂缓"）| §3.1 A020 last_heartbeat_at 跨节点确认超时 | `drill_lcm_002.rs`（推断，per 文件名 `002` 与"超时"主题对应）| b74ccc3 移至 tests-disabled/ | 待 RGS-INC-003 归档后 | b74ccc3 标"per INC-002"为归因错误（INC-002 主题为 k3s 部署），实际根因 per b74ccc3 commit body §未决项 |
| **TST-UT-31-C001**（标"暂缓"）| §3.3 C001 declared → canary_in_progress | `ut_state_machine.rs` + `ut_saga.rs` | 同上 | 同上 | 同上 |
| **TST-UT-31-C003**（标"暂缓"）| §3.3 C003 节点确认超时 canary_in_progress → paused | `drill_lcm_002.rs` | 同上 | 同上 | 同上 |
| **TST-UT-31-C004**（标"暂缓"）| §3.3 C004 canary_confirmed → completed | `ut_state_machine.rs` | 同上 | 同上 | 同上 |
| **TST-UT-31-C008**（标"暂缓"）| §3.3 C008 5 批全部完成 | `drill_lcm_008_010.rs`（推断，per 文件名 `008_010` 与"5 批完成"主题对应）| 同上 | 同上 | 同上 |
| **TST-UT-31-C012**（标"暂缓"）| §3.3 C012 全部节点确认 | `drill_lcm_001.rs`（推断，per 文件名 `001` 与"全节点 ACK"主题对应）| 同上 | 同上 | 同上 |
| **TST-UT-31-C013**（标"暂缓"）| §3.3 C013 单节点未确认超时 | `drill_lcm_002.rs` | 同上 | 同上 | 同上 |
| **TST-UT-31-ADR-003**（标"暂缓"）| §3.7 ADR-003 8 状态迁移合法/非法全覆盖 | `ut_state_machine.rs` + `ut_saga.rs` | 同上 | 同上 | 同上 |

### 5.5 处置路径与下一步

| 路径 | 责任方 | 触发条件 | 完成后对本 UT 设计书的影响 |
|---|---|---|---|
| **A. 新开 RGS-INC-003 复盘** | cluster-ops 域 Lead | b74ccc3 归因错误已确认（per RGS-DOCS-HEALTH-2026-08-27 §2.3）| RGS-INC-003 归档真正根因 + 修复路径 |
| **B. 修复 b74ccc3 5 条未决项** | cluster-ops 域 Lead + 实施 agent | RGS-INC-003 已批准 | 18 个 backing tests 可移回 `tests/` 目录 |
| **C. 恢复 18 个 backing tests** | cluster-ops 域实施 agent | B 完成 | 本设计书 8 条暂缓用例（A020/C001/C003/C004/C008/C012/C013/ADR-003）恢复"可执行" |
| **D. UT 设计书 v0.3 升版** | 本主题域 UT 设计书 owner（即架构师 / Mavis 接手 agent per DEC-008）| C 完成 | §5.4 暂缓 8 条用例标"可执行 per RGS-INC-003 + 18 tests 恢复"；§5.4 表格移除"暂缓"列；§5.3 同步追加"暂缓→可执行"变更说明 |

**本 v0.2 升版不实施 A/B/C/D 任一路径**——本设计书 owner 仅完成 v0.2 升版的元数据层对账（§5.1 背景 / §5.2 映射基线 / §5.3 10 条新增 / §5.4 8 条暂缓），实际修复 + tests 恢复在 cluster-ops 域另一 worktree 实施。

---

> v0.2 升版结束。本文档配套 RGS-REQ-031 需求定义书、RGS-BAS-031 v0.1 addendum 基本设计书、RGS-DTL-031 v0.3 详细设计书（2026-08-25 复核对账版）、RGS-ADR-0051 架构决定。后续将产出 RGS-TST-IT-31（集成）与 RGS-TST-ST-31（系统）。
