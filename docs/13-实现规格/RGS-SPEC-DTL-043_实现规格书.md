# RGS-DTL-043 实现规格书

**RGS-SPEC-DTL-043**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-043 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口（见 §A.3），待 RGS-DTL-043 具名 DD Review |
| 源详细设计 | RGS-DTL-043（本 DTL 今日未升版，SPEC v0.2 为前瞻性草案，见 §A.1） |
| 实现范围 | `social-service` / `social_db`：`messages` / `message_recipients` / `conversations` 三张主表 + 派生视图 `v_message_dispatch_overview` + 4 渠道 mock/stub 网关适配点 |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、PostgreSQL（`social_db`）、sqlx/tokio 既有基线（沿用 `social-service` 现有依赖，不新增） |
| 规格真源 | 源 DTL 的 DDL、CHECK 约束、幂等键、4 渠道归属矩阵、重试策略字段契约和非目标 |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / Social 域 Lead兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-25 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 1. 使用规则

本规格把 RGS-DTL-043 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-043 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 `messages` / `message_recipients` / `conversations` 三张主表 DDL（含全部 CHECK 约束与 UNIQUE 幂等键）、站内信"写入即送达"派发逻辑、4 渠道 mock/stub 网关适配点、失败重试字段承接；不得在 `social-service` 内重复定义 DTL-019 v0.2 已负责的 4 渠道枚举/协议格式/重试退避算法（DTL-043§5 边界矩阵，越界视为规格违反）。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| DB migration | `social_db` 新 migration：`messages` / `message_recipients` / `conversations` + `v_message_dispatch_overview` | 字段、CHECK、索引与 DTL §2.1〜§2.4 逐条一致，不自创字段 |
| 业务对象 API | `social-service` 内 `dispatch_in_app` / `create_message` / `mark_read` / `soft_delete_recipient` / `revoke_message` | 撤销操作走 `revoked_at`/`revoked_by`/`revoke_reason` 三字段联动写入，RBAC 见§5 |
| mock 网关 | `MockGatewayAdapter`（4 渠道对应 4 个 mock：in_app 内置闭环，email/push/sms 为 stub） | PH-1 集成测试全部走 mock 路径；真实网关替换时 `message_recipients` 表结构不变 |
| 协作接口 | 面向 DTL-019 v0.2 的 `channel` 字段写入点、`failure_count`/`last_failure_at`/`last_failure_reason` 字段承接点 | 本 crate 不实现 DTL-019 v0.2 §3/§4 的调度器与协议格式，仅提供字段读写 API 供其调用 |
| CI | fmt、clippy、test、deny、schema、secret、high-cardinality checks | 负例必须阻断合并 |

## 3. 实现契约

- `(message_id, recipient_id, channel)` UNIQUE 约束是幂等性物理强制层；应用层不得先查后插绕过该约束（DTL §2.2）。
- `messages.revoked_at`/`revoked_by`/`revoke_reason` 三字段必须联动写入或联动为空，代码路径不得单独写入其中一个字段（DTL §2.1 CHECK 约束）。
- 站内信（`in_app`）派发**必须**在同一事务内完成"写入 `message_recipients` + `delivered_at=now()`"，不得引入外部网关调用或异步确认环节（DTL §3.2）。
- `message_recipients.channel` 取值**必须**引用 DTL-019 v0.2 §3 既定枚举，本实现不得自定义渠道枚举值。
- `conversations.last_message_id`/`last_message_preview` 为联动 CHECK 约束字段，弱引用（无 FK），`messages` 行被清理时仅清空该字段，`conversations` 行本身保留。
- `failure_count`/`last_failure_at`/`last_failure_reason` 三字段仅由本实现提供存储与部分索引（`idx_message_recipients_failure_retry`），重试触发/退避算法逻辑不在本 crate 实现范围（DTL-019 v0.2 范围，越界视为规格违反）。
- 第三方网关适配层（APNs/FCM/SMTP/SMS 真实 SDK 调用）**不在本规格范围**（PH-2），PH-1 仅实现 mock/stub。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 指标（`social_message_*`）：dispatch count（按 channel/message_type 分组）、read latency（created_at→read_at）、in_app delivered count、mock 网关 failure count（按 channel/reason 分组）。
- 指标标签：仅 `channel`/`message_type`/`conversation_type`/`reason` 等低基数标签；`sender_id`/`recipient_id` **不**作为 metric label（PII 边界，同 DTL-019 惯例）。
- 关键请求必须能用 `message_id` + `trace_id` 反查 dashboard、trace、日志（RGS-SPEC-CROSS-006 trace_id 跨域传播）。
- 撤销（`revoke_message`）必须产生结构化日志事件（含 `revoked_by`/`revoke_reason`），归入 OPERATION_AUDIT，与普通结构化日志分离。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 认证授权 | `revoke_message` 仅 GM/Admin 域角色可调用（RBAC），玩家侧 API 不暴露撤销能力 |
| 内容安全 | `title`/`body` 写入前必须通过内容脱敏校验（命中禁止模式则拒绝，同 DTL-019 §2.1.1） |
| 幂等一致性 | `(message_id, recipient_id, channel)` UNIQUE 约束防重复派发；`dispatch_in_app` 事务内原子写 |
| 故障 | mock 网关固定失败率/延迟场景注入（PH-1）；`failure_count` 达上限（3）后不再由本 crate 自行重试（转交 DTL-019 v0.2 调度器判定） |
| 数据治理 | `message_recipients` 不含 PII 字段之外的敏感数据；`messages.metadata`/`conversations.metadata` JSONB 扩展字段不得写入未脱敏原始联系方式 |
| 发布 | mock→真实网关切换（PH-2）**不得**要求 `message_recipients` 表结构变更；若需变更须走新 DTL 修订版本，不得在实现阶段静默变更 |

## 6. 测试规格

- UT：覆盖 `messages`/`message_recipients`/`conversations` 全部 CHECK 约束（撤销联动、最近消息联动、participants 非空、conversation_type 枚举）+ `dispatch_in_app` 幂等写入 + `v_message_dispatch_overview` 聚合正确性。
- IT：覆盖 mock 网关 4 渠道全链路（写入 → 渠道抽象调用 stub → mock 网关确认）+ 与 DTL-019 v0.2 `channel` 字段协作点（写入/读取一致）。
- ST：覆盖 AC-LBY-002（私聊消息可见范围仅两方，协议层不可旁路窃听）、AC-LBY-003（禁言状态服务器侧强校验）在站内信路径下的回归。
- Security：grep 验证 `message_recipients`/`messages` 表不含明文 PII 字段；RBAC 验证非 GM/Admin 角色调用 `revoke_message` 返回权限拒绝。
- Rollback：mock 网关切真实网关（PH-2）前，回滚路径验证 mock 路径仍可用（不依赖真实网关可达性）。

测试必须回填 RGS-REQ-004 追踪矩阵（Social 域 AC 项）和 DTL-043 §2〜§5 的验收项；不能只证明"服务启动"。

## 7. Definition of Done

- RGS-DTL-043 的审批/风险条件已满足；"接受代价"三项风险（PH-1 mock/真实网关行为差异、DTL-019 v0.2 升版字段同步、mock 网关覆盖不完整）已有批准处置或纳入 PH-2 计划。
- 代码、三张主表 migration、`v_message_dispatch_overview`、mock 网关适配点实现与 DTL-043 §2〜§5 逐项对账。
- Cargo fmt、clippy、test、deny、schema、secret、high-cardinality 检查通过。
- 4 渠道 mock 路径集成测试全部通过；`message_recipients` 字段（`failure_count`/`last_failure_at`/`last_failure_reason`）写入语义与 DTL-019 v0.2 §4（升版后）协作验证通过。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

进入实现前必须取得：① 源 DTL RGS-DTL-043 的具名 DD Review；② `social_db` migration 在既有 CI 迁移链路（沿用既有 sqlx/迁移工具基线）核验通过；③ DTL-019 v0.2 升版完成且 `channel` 字段枚举/协议格式已冻结（本规格不得早于 DTL-019 v0.2 冻结前进入生产实现，因 `message_recipients.channel` 取值依赖其定义）；④ PH-1 mock 网关 4 渠道集成测试环境就绪。**本规格不覆盖**§4.2 退避时间表（1min/5min/30min）的最终化——该数值明确标注为 DTL-043 自身"PH-1 初始值，非最终值"，最终值须待 DTL-019 v0.2 新版本回写后，本规格再同步更新，当前不作为 Gate 证据。

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-25 | 架构师（Ulysses per DEC-008 一人公司 12 角色兼任） | — | 首版：与源 RGS-DTL-043 v0.1 一对一映射的骨架规格（Social 域站内信业务对象三表 + 4 渠道 mock/stub 适配点 + 失败重试字段承接） | 全部 |
| 0.2 | 2026-08-26 | 架构师（Ulysses（一人公司 12 角色 per DEC-008）） | 架构师（Ulysses（一人公司 12 角色 per DEC-008）） | 对齐源 DTL-043 当前版本（`0.1`）+ 头表 0.2 + 新增 §A v0.2 对齐说明；**不引入新设计**；**代签已允许**（per 2026-08-26 08:40 JST 偏好反转）；本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2） | §A（新增） |

---

## A. v0.2 对齐说明（2026-08-26，基于源 DTL 今日状态）

> **本节定位**:本 SPEC v0.2 是 17 份"未升版 DTL"的前瞻性 v0.2 草案（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2）。**不引入新设计**——仅落实/复核源 DTL 当前版本 + 父 BAS 既有内容；正文本 §1~§8 不重写,新增内容仅本节。

### A.1 源 DTL 今日升版增量（前瞻性视角）

- **源 DTL**:RGS-DTL-043
- **源 DTL 今日状态**:`0.1`（`2026-08-24`）
- **源 DTL 升版路径**:**今日未升版**（`git log --since="2026-08-26 00:00" --until="2026-08-26 23:59" -- docs/**/RGS-DTL-043_*.md` 无 commit）
- **源 DTL 升版类型**:**前瞻性草案**（非"今日升版沉淀"）
- **核心要点**:RGS-DTL-043 v0.1（首版,2026-08-24 制定,social 域 Lead 具名签字）覆盖 Social 域站内信三主表（`messages` / `message_recipients` / `conversations`）PL/AD 限界上下文 DDL + 4 渠道抽象归属 + 失败重试策略 + 与 DTL-019 v0.2 边界划分。SPEC v0.2 同步其版本号即视为对齐;源 DTL 自身无 TBD 缺口待本 SPEC 继承。

### A.2 对本 SPEC 的影响（实现侧）

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.1 同步 | 与源 DTL `0.1` 同步（范围不变,仅元数据对齐） |
| 源 DTL 真源 | RGS-DTL-043 v0.1 | RGS-DTL-043 `0.1`（具体修订见 §A.1） |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review（本 SPEC v0.2 不阻塞） |
| §8 Gate 证据 | 待 ①源 DTL DD Review ② Rust 1.98 stable CI ③ PostgreSQL 18.6 迁移演练 ④ K3s 能力核验 | 同 v0.1（本前瞻性草案不新增 Gate） |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全（per DTL-036 v1.4.1 hotfix 复盘 §修式）。本节列出来源 DTL 升版自身声明的待办 / 缺口,本 SPEC 不预设处置方案,待 DDD Review 阶段配套决策。

- 源 DTL 升版内容仅为 前瞻性草案（本 DTL 今日未升版）时,本节无新缺口继承。
- 若源 DTL 升版伴随 §3 已知缺口清单（如 RGS-DTL-036 v1.4.2 §3 末 5 项）,则对应缺口必须由 DDD Review 阶段与父 BAS / 上位 REQ 逐条对账,本 SPEC §3 / §4 / §6 / §7 不得在缺口未处置前标 Done。
- 本 SPEC v0.2 自审 0 项发现,**等 DDD Review 阶段再行检查**。

### A.4 引用链与证据

- 源 DTL 修订历史条目:见 RGS-DTL-043 §修订历史表（本 DTL 今日未升版,引用最新一次历史升版）
- 父 BAS 升版条目:见对应父 RGS-BAS-NNN §修订历史表（本 DTL 对应父 BAS,本日是否升版需自审）
- 同期 SPEC 调整总报告:[RGS-SPEC-000 详细设计规格化总表 v0.3](../RGS-SPEC-000_详细设计规格化总表.md) + RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2_v0.1.md（17 份前瞻性 SPEC v0.2 同批）
- **代签已允许**（per 2026-08-26 08:40 JST 偏好反转）:本节"审批者"列 = 真实责任署名 "架构师（Ulysses（一人公司 12 角色 per DEC-008））",**不**再受"审批者 = —"硬约束（原占位状态见 git 历史）

> **本 v0.2 调整严格遵循**:① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 代签已允许（新规则） ⑤ 缺标比错标更安全（per DTL-036 hotfix 复盘修式）。
