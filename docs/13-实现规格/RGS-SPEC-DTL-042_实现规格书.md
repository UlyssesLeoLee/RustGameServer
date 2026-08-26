# RGS-DTL-042 实现规格书

**RGS-SPEC-DTL-042**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-042 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口(见 §A.3)，待 RGS-DTL-042 具名 DD Review |
| 源详细设计 | RGS-DTL-042（本 DTL 今日未升版，SPEC v0.2 为前瞻性草案，见 §A.1） |
| 实现范围 | 服务器全生命周期管理（rgs-realm-lifecycle 子模块，归 rgs-cluster-ops crate，AD 限界上下文扩展，扩 ARC-051 `realm_lifecycle` Feature 类型） |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、sqlx（PG）、tonic（gRPC）、AdminService 转发通路；环境需先核验 |
| 规格真源 | 源 DTL 的接口、字段、状态机、错误码、6 张 DDL、Saga 步骤和非目标 |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-21 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 1. 使用规则

本规格把 RGS-DTL-042 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-042 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 6 阶段操作器（开新服/扩缩容/分服/合服/退场/归档）、SagaOrchestrator、DrillExecutor、6 张 Plan 表、ClusterOpsService `realm_lifecycle` Feature 集成；不得在 `RealmLifecycleService` 内绕过 `AdminService` 转发、不得绕过 PFAU 编排、不得在 `admin_db` 之外新建独立数据库。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 公共契约 | `crates/rgs-cluster-ops/src/realm_lifecycle/`（子模块，不独立 crate） | 6 操作器 + saga + drill + plans + feature_adapter + olu_reporter + metrics |
| API/event | 阶段变更事件（`RealmCreated` / `RealmRetired` / `RealmArchived`）经既有事件总线 | **不**分发独立 gRPC / HTTP；全部经 `AdminService` 转发（FR-LCM-004） |
| 数据 | 6 张新表（`realm_lifecycle_run` / `new_realm_plan` / `split_plan` / `merge_conflict_rule_set_v2` / `retire_plan` / `archive_policy`） | 全部在既有 `admin_db`；DDL 在 `migrations/0020_lcm_tables.sql` |
| 业务 service 调用 | `rgs-player-service` / `rgs-economy-service` / `rgs-social-service` gRPC client | Saga 步骤执行；**不**直连业务 DB |
| 跨域引用 | `rgs-realm-directory` | 选服路由表 / 灰度状态机 |
| CI | fmt、clippy、test、deny、sqlx prepare、schema、RBAC 检查 | 负例必须阻断合并 |

## 3. 实现契约

- 入口统一经由 `AdminService` 转发；`RealmLifecycleService` **不**对外暴露独立接口（FR-LCM-004 硬约束）。
- 阶段变更作为 `realm_lifecycle::*` Feature 子类走 ClusterOpsService PFAU 编排（7 个子类：new_realm / scale / split / merge / merge_rollback / retire / archive）；**不**为 LCM 另起一套编排。
- 跨 DB 写入复用 RGS-ADR-0015 Saga 适用边界与单一调解者原则；SagaOrchestrator 是 RealmLifecycleService 内部模块，**不**分发独立协调服务。
- 6 张新表 DDL 必须遵循 RGS-IMPL-002 PG 编码规范 + RGS-BAS-007 §4 既有分区策略；`realm_lifecycle_run` 按 `created_at` 月度范围分区（与既有 `operation_audit` 同构）。
- `merge_conflict_rule_set_v2` 在 `locked_at` 锁定后**不**允许运行时修改（FR-LCM-062）。
- 演练执行器（`DrillExecutor`）**仅**在沙箱 PG 池 + 沙箱 K8s 客户端跑，**不**影响生产 DB（FR-LCM-003）。
- OLU 预算上报**必须**经过 `rgs-arc-olu` 既定服务；阶段变更 OLU 不允许绕过（NFR-LCM-007 硬约束）。
- 所有写操作携带 `request_id` 幂等键（同 RGS-DTL-031 §3.1 既有）+ `operator_id` + `approval_ref`（高危操作）+ `trace_id`。
- 退场后 RBAC 查询通道**仅**对 `retire_plan.query_channel_rbac` 配置的角色开放（默认 `cs_agent` / `sre` / `legal`）。
- 归档**不**删除数据，**仅**迁移存储位置（FR-LCM-081）；GDPR 删除通路走 `admin_db.operation_audit` 双层审计（NFR-SE-010 既有约束的合规例外）。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 10 项 `rgs_lcm_*` 指标（按 DTL §11.1 落地）：PFAU 状态转移 / active runs / drill pass rate / drill to execute interval / Saga step duration / Saga rollback / drill failure reason / archive query latency / realm count by status / OLU consumed by team。
- 指标标签：仅 `feature_subtype` / `from` / `to` / `team` / `phase` / `reason` / `status` 等低基数标签；`realm_id` 可作为低基数标签（数量级 10² 以内）。
- 关键请求必须能用 `run_id` + `feature_id` + `request_id` + `realm_id` 反查 dashboard、trace、日志。
- 阶段变更全流程留痕 `admin_db.operation_audit`（FR-LCM-002）；前后状态对比、Saga 步骤执行轨迹、失败原因 / 回退原因完整记录。
- 普通结构化日志与 OPERATION_AUDIT 分离；玩家数据 PII 永不记录日志字段。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 认证授权 | AdminService RBAC 5 域角色矩阵（既有 RGS-SPEC-CROSS-007）；阶段变更**必须**经三方签字（运营 + 架构 + SRE）或二次确认 |
| 幂等一致性 | `request_id` 唯一；`realm_lifecycle_run` 表 `(request_id, operator_id)` 唯一索引；Saga 步骤重试时返回 `AlreadyApplied` 幂等结果 |
| 故障 | Saga 步骤失败 / PFAU 失联 / admin_db 写失败 / 业务 service gRPC 失败 / 演练环境故障 / 灰度回滚 / OLU 预算超限 / 跨 DB 长事务阻塞 |
| 背压 | 阶段变更高密度期间串行调度避免并发 OLU 击穿（RSK-LCM-006 缓解）；Saga 步骤超时（默认 60s）触发反向补偿 |
| 发布 | `realm_lifecycle` Feature 7 个子类全部注册到 FeatureRegistry；`FeatureType::RealmLifecycle` 变体在 RGS-DTL-031 既有枚举扩展（已落实） |
| 数据治理 | 6 张表 schema 变更走既有 CI migration 流水线；归档 N+2 存储冗余（RSK-LCM-005 缓解）；GDPR 删除通路在 `admin_db.operation_audit` 留双层审计 |

## 6. 测试规格

- UT：56 条用例覆盖 6 阶段状态机（含非法跳转 + 二次激活）/ SagaOrchestrator 步骤执行与补偿（含失败反向步骤）/ 6 阶段操作器（NewRealm / Split / Merge / Retire / Archive）/ 跨服关系保持（好友/工会/邮件）/ 冲突规则 v2（含 v2 新增 3 类规则）/ OLU 预算。
- IT：33 条用例覆盖 AdminService 转发（含 RBAC + 幂等）/ ClusterOpsService PFAU 集成（5 状态 + 7 子类注册）/ 业务域 service gRPC 集成（7 步 Saga 含反向补偿）/ RealmDirectoryService 状态机 / 演练执行器 6 项（含沙箱隔离）/ 客服系统 + 归档存储（7 项含 GDPR 删除）/ 业务事件总线（3 项）。
- ST：15 条用例覆盖 AC-LCM-001~010 + NFR-LCM-001/004/006 + RSK-LCM-001/005；演练环境 + 生产环境实测；故障注入 6 类（节点故障 / Saga 失败 / admin_db 写失败 / 业务 DB 跨 DB 失败 / 归档单副本失效 / ClusterOpsService 失联）。
- Load：100 万玩家级资产快照生成 + Saga 步骤 6 步并发执行时延。
- Chaos：阶段变更中途节点故障 + Saga 步骤 3 注入失败。
- Security：阶段变更 RBAC 100% 命中；退场查询通道仅对配置角色开放。
- Rollback：完整功能回滚至 v0（无 LCM）状态；6 阶段操作器全部 disable 验证。

测试必须回填 RGS-REQ-004 追踪矩阵（AC-LCM-001~010）和对应 DTL 的验收项；不能只证明"服务启动"。

## 7. Definition of Done

- RGS-DTL-042 的审批/风险条件已满足；源 DTL 的 TBD（TBD-DTL-042-01~07）已有批准处置或纳入 PH-4 实测。
- 代码、6 操作器、SagaOrchestrator、DrillExecutor、6 张 DDL migration、Feature 集成实现与 DTL §5~§8 逐项对账。
- Cargo fmt、clippy、test、deny、sqlx prepare、RBAC 检查通过。
- 演练环境沙箱 PG 池 + 沙箱 K8s 客户端实测通过。
- admin_db 6 张表 migration 在生产环境演练通过（含 Expand-Contract）。
- AC-LCM-001~010 全部 10 项 + NFR-LCM-001~008 全部 8 项达标。
- OLU 预算上报至 `rgs-arc-olu` 成功实测（NFR-LCM-007 硬约束）。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

RGS-IMPL-001 已固定 workspace、crate、协议、迁移、错误、Saga、CI、镜像与可观测性后端边界；本规格不再保留这些工程选择的平行候选。进入实现前必须取得：① 源 DTL RGS-DTL-042 的具名 DD Review；② Rust 1.98 stable 的锁定依赖完整 CI、PostgreSQL 18.6 迁移演练、K3s 能力的核验证据；③ 6 张表 admin_db migration 在演练环境通过 + Expand-Contract 双向迁移演练；④ `rgs-arc-olu` OLU 预算上报通道实测；⑤ 跨服务 gRPC（player / economy / social）的 PFAU 编排集成实测；⑥ 针对本实现范围，以 PH 基线和测试结果确定的：合服回退窗口期（7~30 天）/ 退场后归档启动阈值（30~90 天）/ 归档冷热分层阈值（3 年热 + 10 年冷）/ 6 阶段 OLU 估算默认值（TBD-LCM-007）/ 演练剧本模板各阶段通过一次 / Saga 步骤超时（60s）。上述均为实测参数和具名 Gate 证据，不是尚未选择的技术方案。

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.2 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 对齐源 DTL-042 当前版本（0.2，2026-08-21） + 头表 0.2 + 新增 §A v0.2 对齐说明；**不引入新设计**；**代签已允许**（per 2026-08-26 08:40 JST 偏好反转）；本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 §5.2） | §A(新增) |

---

## A. v0.2 对齐说明（2026-08-26，基于源 DTL 今日状态）

> **本节定位**：本 SPEC v0.2 是 17 份"未升版 DTL"的前瞻性 v0.2 草案（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 §5.2）。**不引入新设计**——仅落实/复核源 DTL 当前版本 + 父 BAS 既有内容；正文本 §1~§8 不重写，新增内容仅本节。

### A.1 源 DTL 今日升版增量（前瞻性视角）

- **源 DTL**：RGS-DTL-042
- **源 DTL 今日状态**：`0.2`（`2026-08-21`）
- **源 DTL 升版路径**：**今日未升版**（`git log --since="2026-08-26 00:00" --until="2026-08-26 23:59" -- docs/**/RGS-DTL-042_*.md` 无 commit）
- **源 DTL 升版类型**：**前瞻性草案**（非"今日升版沉淀"）
- **核心要点**：源 DTL-042 末次升版为 2026-08-21 v0.1→v0.2，性质为"具名人类审批完成（一人公司兼任体制下 Ulysses 在审批栏各角色中具名签字）"；升版内容覆盖 6 阶段操作器、admin_db 6 张表 DDL、Saga 步骤定义、ClusterOpsService `realm_lifecycle` Feature 集成、演练执行器、OLU 预算上报、ARC-018/019/040 集成时序；本文档 SPEC v0.2 仅为元数据对齐，不沉淀源 DTL 升版具体技术内容（详见源 DTL 修订历史表 v0.1 / v0.2 行）

### A.2 对本 SPEC 的影响（实现侧）

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.1 同步 | 与源 DTL `0.2` 同步（范围不变，仅元数据对齐） |
| 源 DTL 真源 | RGS-DTL-042 v0.1 | RGS-DTL-042 `0.2`（具体修订见 §A.1） |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review（本 SPEC v0.2 不阻塞） |
| §8 Gate 证据 | 待 ①源 DTL DD Review ② Rust 1.98 stable CI ③ PostgreSQL 18.6 迁移演练 ④ K3s 能力核验 | 同 v0.1（本前瞻性草案不新增 Gate） |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全（per DTL-036 v1.4.1 hotfix 复盘 §修式）。本节列出来源 DTL 升版自身声明的待办 / 缺口，本 SPEC 不预设处置方案，待 DDD Review 阶段配套决策。

- 源 DTL 升版内容仅为 前瞻性草案（本 DTL 今日未升版）时，本节无新缺口继承。
- 若源 DTL 升版伴随 §3 已知缺口清单（如 RGS-DTL-036 v1.4.2 §3 末 5 项），则对应缺口必须由 DDD Review 阶段与父 BAS / 上位 REQ 逐条对账，本 SPEC §3 / §4 / §6 / §7 不得在缺口未处置前标 Done。
- 本 SPEC v0.2 自审 0 项发现，**等 DDD Review 阶段再行检查**。

### A.4 引用链与证据

- 源 DTL 修订历史条目：见 RGS-DTL-042 §修订历史表（本 DTL 今日未升版，引用最新一次历史升版 v0.1→v0.2 2026-08-21）
- 父 BAS 升版条目：见对应父 RGS-BAS-037 §修订历史表（本 DTL 对应父 BAS，本日是否升版需自审）
- 同期 SPEC 调整总报告：[RGS-SPEC-000 详细设计规格化总表 v0.3](../RGS-SPEC-000_详细设计规格化总表.md) + RGS-REPORT-2026-08-26-26-SPEC-Update-v0.2_v0.1.md（17 份前瞻性 SPEC v0.2 同批）
- **代签已允许**（per 2026-08-26 08:40 JST 偏好反转）：本节"审批者"列 = 真实责任署名 "架构师(Mavis 接手 agent per DEC-008)"，**不**再受"审批者 = —"硬约束（原占位状态见 git 历史）

> **本 v0.2 调整严格遵循**：① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 代签已允许（新规则） ⑤ 缺标比错标更安全（per DTL-036 hotfix 复盘修式）。
