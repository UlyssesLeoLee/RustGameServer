# RGS-DTL-101 实现规格书

**RGS-SPEC-DTL-101**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-101 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口(见 §A.3),待 RGS-DTL-101 具名 DD Review |
| 源详细设计 | RGS-DTL-101(本 DTL 今日未升版,SPEC v0.2 为前瞻性草案,见 §A.1) |
| 实现范围 | 共享类型 crate(`TransactionScope`/`OperationPolicy`/`AuthorityBoundary`)+ `OPERATION_REGISTRY`(30+ 操作矩阵)+ Command Layer 决策算法 `decide_command` + `check_authority_boundary` |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、serde（既有 workspace 依赖，不新增） |
| 规格真源 | 源 DTL 的 `TransactionScope`/`OperationPolicy`/`AuthorityBoundary` 类型定义、§5 Operation Policy 完整矩阵、§6 决策算法、§7 AuthorityBoundary 检查 |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-25 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 1. 使用规则

本规格把 RGS-DTL-101 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-101 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 `TransactionScope`/`OperationPolicy`/`AuthorityBoundary`/`RetryPolicy` 全部类型（DTL §2〜§4，字段级一致）、`OPERATION_REGISTRY`（§5 完整矩阵 30+ 操作，逐条注册不得遗漏）、`decide_command`（§6）与 `check_authority_boundary`（§7）两个强制校验函数；Command Layer **必须**强制经过 `decide_command`，**不得**存在绕过 OperationPolicy 直接派发 Saga/gRPC 的代码路径（DTL §0 "强制决策"核心原则）。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 共享类型 | 共享类型 crate 内 `operation_policy` 模块：`TransactionScope`/`OperationPolicy`/`AuthorityBoundary`/`RetryPolicy` | 字段/派生宏（`Serialize`/`Deserialize`/`PartialEq`/`Eq`/`Hash`）与 DTL §2〜§3 逐字段一致 |
| Registry | `OPERATION_REGISTRY`（静态注册表，`once_cell`/`lazy_static` 或等效机制，沿用既有 workspace 基线） | §5.1/§5.2 全部操作条目（GM 后台 27 项 + 客户端 12 项）逐条注册，字段（scope/authority/participants/timeout/audit/reason/2fa）与矩阵表一致 |
| 决策算法 | Command Layer 内 `decide_command` | reason/2FA 前置校验 → scope 分派 `CommandTarget`（Local/Grpc/Saga）三分支逻辑与 DTL §6 一致 |
| 权威边界检查 | Command Layer 内 `check_authority_boundary` | `payload_aggregate` → `expected_authority` 映射表（§7）+ SagaRuntime 写操作仅 `Actor::System` 可执行的强制检查 |
| CI | fmt、clippy、test、deny checks | 负例必须阻断合并；`OPERATION_REGISTRY` 遗漏条目需有 CI 检测（矩阵条目数量断言） |

## 3. 实现契约

- 每个 Command **必须**先查 `OPERATION_REGISTRY`，未注册的 `operation` 一律返回 `CommandError::UnknownOperation`，**不得**为未知操作提供默认 scope（DTL §6 `decide_command` 首行逻辑，防止"隐式分布式"反模式）。
- `requires_reason=true` 的操作缺少 `reason` **必须**拒绝（`CommandError::ReasonRequired`）；`requires_2fa=true` 的操作 2FA 校验失败**必须**拒绝（`CommandError::TwoFactorRequired`）——两项检查顺序按 DTL §6 先后次序（reason 先于 2FA）。
- `AuthorityBoundary::SagaRuntime` 的写操作**仅** `Actor::System` 可执行，任何 `Actor::User`/`Actor::Gm` 尝试直接写 Saga 状态**必须**拒绝（`AuthorityError::SagaStateWriteForbidden`，DTL §7 第 2 条）——对应 DTL-100 §7 `saga_instance`/`saga_step` 等表"不可被其他服务直接写"的约束在应用层的强制点。
- §5.3 反例表列出的 6 类操作（改昵称/单次扣货币/登录/加好友/单条消息/改密码）**必须**注册为 `SINGLE_SERVICE`，**不得**注册为 `DISTRIBUTED_SAGA`；`OPERATION_REGISTRY` 初始化时的单元测试须显式断言这 6 项的 scope。
- `operation_authority` 与 `payload_aggregate` 映射（§7）不一致时**必须**返回 `AuthorityError::AuthorityMismatch`，不得静默放行。
- Saga 触发条件遵循 BR-102（DTL §1 "白名单驱动"），本规格**不**重新定义 BR-102 判定逻辑本身，仅要求 `OPERATION_REGISTRY` 条目的 scope 赋值与 BR-102 判定结果一致（矛盾时以 BR-102 为准，需更新 DTL 而非静默改注册表）。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 指标（`command_decision_*`）：按 `operation`/`scope`/`target_type`（Local/Grpc/Saga）分组的决策计数；`CommandError` 类型分组的拒绝计数（`UnknownOperation`/`ReasonRequired`/`TwoFactorRequired`/`AuthorityMismatch`/`SagaStateWriteForbidden`）。
- 指标标签：仅 `operation`/`scope`/`error_type` 等低基数标签；`actor` 具体身份（`player_id`/`gm_id`）**不**作为 metric label。
- `requires_audit=true` 的操作决策结果**必须**产生结构化日志事件（归入 OPERATION_AUDIT），日志需可用 `operation` + `trace_id` 反查。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 认证授权 | `AuthorityBoundary::SagaRuntime` 写操作的 `Actor::System` 强制检查（§7 第 2 条）；GM 操作的 `requires_reason`/`requires_2fa` 强制检查（§6） |
| 反模式防护 | §8 反模式表 6 项（Saga 滥用/隐式分布式/回滚恐慌/UI 强制成功/浏览器即 Coordinator/跨服不隔离）均需有对应的代码审查检查点或自动化检测（如 grep 禁止 service-to-service 同步调用链） |
| 一致性 | `OPERATION_REGISTRY` 条目数量与 DTL §5.1/§5.2 矩阵行数一致性校验（CI 断言，防止矩阵更新后注册表遗漏同步） |
| 数据治理 | `CommandRequest.payload` 内容遵循各域自身 PII 边界，本规格不重复定义 |
| 发布 | `OPERATION_REGISTRY` 变更（新增/修改操作条目）须随 DTL-101 修订版本同步，不得在代码中单方面新增未登记于 DTL 的操作 |

## 6. 测试规格

- UT：覆盖 `TransactionScope` 全部方法（`requires_server`/`requires_message_bus`/`requires_saga_runtime`/`requires_audit`）+ `decide_command` 三分支（Local/Grpc/Saga）+ reason/2FA 前置校验拒绝路径 + `check_authority_boundary` 映射表全部 8 类 aggregate（account/character/inventory/economy(含 currency/balance 别名)/match(含 match_room 别名)/guild/mail/saga）+ SagaRuntime 写保护 + 未知 aggregate（`_` 分支）返回 `UnknownAggregate` 错误路径。
- IT：覆盖 §5.3 反例表 6 项操作的 scope 断言（防止未来误改为 DISTRIBUTED_SAGA）+ `OPERATION_REGISTRY` 全部 30+ 条目的 scope/authority/participants 字段与矩阵表逐条比对（回归测试，矩阵表变更需同步更新此测试）。
- ST：Command Layer 端到端验证——伪造绕过 `decide_command` 直接调用 Saga Runtime 的请求必须被拒绝（验证"强制决策"核心原则无绕过路径）。
- Security：验证非 `Actor::System` 请求写 Saga 状态（`saga_instance`/`saga_step` 等表）在 gRPC/应用层均被拒绝，不仅依赖数据库权限。

测试必须回填 RGS-REQ-004 追踪矩阵（Saga/COC 相关 AC 项）和 DTL-101 §5〜§8 的验收项；不能只证明"服务启动"。

## 7. Definition of Done

- RGS-DTL-101 的类型定义、`OPERATION_REGISTRY` 矩阵、决策算法、AuthorityBoundary 检查与实现逐项对账。
- Cargo fmt、clippy、test、deny 检查通过。
- §5.3 反例表 6 项操作的 scope 回归测试通过；`OPERATION_REGISTRY` 条目数量与 DTL 矩阵行数一致性 CI 检查通过。
- 与 RGS-DTL-100（Saga 业务模式）、RGS-DTL-102（故障恢复）的接口契约联合验证通过（三份 DTL 为同侪文档，非独立可验收）。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

进入实现前必须取得:① 源 DTL RGS-DTL-101 的具名 DD Review;② RGS-DTL-100/RGS-DTL-102 同侪文档已定稿(三者共同构成 Saga 子系统完整设计,不得单独进入实现);③ `OPERATION_REGISTRY` 初始化性能核验(静态注册表在服务启动路径上的初始化开销,不得影响冷启动时延预算)。**本规格不覆盖**:BR-102 Saga 触发判定条件本身的重新论证——DTL §1 已将其列为既定白名单依据,本规格仅要求注册表赋值与 BR-102 结果一致,不重新推导 BR-102。

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 备注 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-25 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 初版。基于 RGS-DTL-101 v0.1(2026-08-21),转译为实现规格:§1 使用规则 + §2 实现单元 + §3 实现契约 + §4 可观测性 + §5 安全容错 + §6 测试规格 + §7 DoD + §8 Gate 证据。 | 头表 + §1~§8 |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 对齐源 DTL-101 当前版本(`0.1`) + 头表 0.2 + 新增 §A v0.2 对齐说明;**不引入新设计**;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2) | §A(新增) |

---

## A. v0.2 对齐说明(2026-08-26,基于源 DTL 今日状态)

> **本节定位**:本 SPEC v0.2 是 17 份"未升版 DTL"的前瞻性 v0.2 草案(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2)。**不引入新设计**——仅落实/复核源 DTL 当前版本 + 父 BAS 既有内容;正文本 §1~§8 不重写,新增内容仅本节。

### A.1 源 DTL 今日升版增量(前瞻性视角)

- **源 DTL**:RGS-DTL-101
- **源 DTL 今日状态**:`0.1`(`2026-08-21`)
- **源 DTL 升版路径**:**今日未升版**(`git log --since="2026-08-26 00:00" --until="2026-08-26 23:59" -- docs/**/RGS-DTL-101_*.md` 无 commit)
- **源 DTL 升版类型**:**前瞻性草案**(非"今日升版沉淀")
- **核心要点**:源 DTL v0.1 初版定义 OperationPolicyRegistry / TransactionScope 5 级决策枚举 / AuthorityBoundary 11 类权威边界 / 30+ 操作完整矩阵(§5.1 后台 27 项 + §5.2 客户端 12 项)/ §5.3 反 Saga 升级 6 项反例 / §6 决策算法 / §7 权威边界检查 / §8 6 类反模式。本 SPEC v0.2 仅落实/复核该既有内容,无新设计引入。

### A.2 对本 SPEC 的影响(实现侧)

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.1 同步 | 与源 DTL `0.1` 同步(范围不变,仅元数据对齐) |
| 源 DTL 真源 | RGS-DTL-101 v0.1 | RGS-DTL-101 `0.1`(具体修订见 §A.1) |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review(本 SPEC v0.2 不阻塞) |
| §8 Gate 证据 | 待 ①源 DTL DD Review ② Rust 1.98 stable CI ③ PostgreSQL 18.6 迁移演练 ④ K3s 能力核验 | 同 v0.1(本前瞻性草案不新增 Gate) |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全(per DTL-036 v1.4.1 hotfix 复盘 §修式)。本节列出来源 DTL 升版自身声明的待办 / 缺口,本 SPEC 不预设处置方案,待 DDD Review 阶段配套决策。

- 源 DTL 升版内容仅为 前瞻性草案(本 DTL 今日未升版)时,本节无新缺口继承。
- 若源 DTL 升版伴随 §3 已知缺口清单(如 RGS-DTL-036 v1.4.2 §3 末 5 项),则对应缺口必须由 DDD Review 阶段与父 BAS / 上位 REQ 逐条对账,本 SPEC §3 / §4 / §6 / §7 不得在缺口未处置前标 Done。
- 本 SPEC v0.2 自审 0 项发现,**等 DDD Review 阶段再行检查**。

### A.4 引用链与证据

- 源 DTL 修订历史条目:见 RGS-DTL-101 §修订历史表(本 DTL 今日未升版,引用最新一次历史升版)
- 父 BAS 升版条目:见对应父 RGS-BAS-NNN §修订历史表(本 DTL 对应父 BAS,本日是否升版需自审)
- 同期 SPEC 调整总报告:[RGS-SPEC-000 详细设计规格化总表 v0.3](../RGS-SPEC-000_详细设计规格化总表.md) + RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2_v0.1.md(17 份前瞻性 SPEC v0.2 同批)
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = 真实责任署名 "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束(原占位状态见 git 历史)

> **本 v0.2 调整严格遵循**:① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 代签已允许(新规则) ⑤ 缺标比错标更安全(per DTL-036 hotfix 复盘修式)。
