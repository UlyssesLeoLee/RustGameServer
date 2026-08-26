# RGS-DTL-102 实现规格书

**RGS-SPEC-DTL-102**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-102 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口(见 §A.3)，待 RGS-DTL-102 具名 DD Review |
| 源详细设计 | RGS-DTL-102(本 DTL 今日未升版，SPEC v0.2 为前瞻性草案，见 §A.1)（Saga 故障恢复设计：状态机/Crash Recovery/HA/升级兼容/故障自检表） |
| 实现范围 | `saga-runtime` 内 `RecoveryWorker`（startup_scan/heartbeat_loop/reaper_loop）+ Saga Instance 11 状态机 + Fence Token OCC 机制 |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、PostgreSQL（`cluster_ops_db`，`saga_fence_token_seq` 序列）、`async-nats`（NATS JetStream，与 RGS-DTL-100/RGS-DTL-102 §9 一致基线） |
| 规格真源 | 源 DTL 的 Saga Instance 状态机（§1）、Crash Recovery 抢占 SQL（§2）、Fence Token OCC 契约（§3）、微服务重启重试表（§4）、Definition 升级兼容规则（§5）、Recovery Worker Rust 实现（§8） |

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

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 备注 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses(架构师兼 per DEC-008) | Ulysses(一人公司 12 角色兼任 per DEC-008) | 初版规格书。基于源 DTL-102 v0.1 起草:§1 使用规则 / §2 实现单元 / §3 实现契约 / §4 可观测性 / §5 安全容错与发布 / §6 测试规格 / §7 DoD / §8 Gate 证据。 | 初版 |
| 0.2 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 对齐源 DTL-102 当前版本(0.1) + 头表 0.2 + 新增 §A v0.2 对齐说明;**不引入新设计**;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2) | §A(新增) |

---

## 1. 使用规则

本规格把 RGS-DTL-102 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-102 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 Saga Instance 11 状态机（§1，含 PENDING/RUNNING/WAITING/RETRYING/COMPENSATING/COMPENSATED/COMPLETED/FAILED/TIMEOUT/CANCELED/EXPIRED）、`RecoveryWorker`（§8 三个循环：startup_scan/heartbeat_loop/reaper_loop）、Fence Token OCC 抢占/续约/写入校验三段 SQL 契约（§2/§3）；**不得**引入 Redis 或其他分布式锁组件替代 PostgreSQL Fence Token 机制（§3"为什么不用 Redis"，BR-111 硬约束）。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 状态机 | `saga-runtime` 内 Saga Instance 状态枚举与合法转移表 | 11 状态、转移路径与 §1 mermaid 状态图逐条一致，非法转移必须拒绝 |
| Fence Token 序列 | `cluster_ops_db`：`CREATE SEQUENCE saga_fence_token_seq START 1 INCREMENT 1` | 与 DTL §3.1 一致；`saga_instance.fence_token` 列已由 DTL-100 §7 定义，本规格不重复建表 |
| Recovery Worker | `saga-runtime/recovery.rs`：`RecoveryWorker` struct（`startup_scan`/`heartbeat_loop`/`reaper_loop`） | 字段（`grace_period`=60s 默认/`scan_interval`=5s 默认/`snapshot_interval`=30s 默认）与 §8 Rust 代码块一致 |
| 抢占/续约 SQL | Recovery Worker 内部 | `UPDATE saga_instance SET owner_pod=..., fence_token=nextval(...) WHERE saga_id=? AND (owner_pod=? OR updated_at < NOW() - grace_period)` 与 §2/§8 逐条一致 |
| Snapshot + Journal Replay | `saga-runtime` 恢复路径 | 有 snapshot 时加载 snapshot + replay `last_event_id` 之后事件；无 snapshot 时全量 replay `saga_event`（§2 步骤 136-141） |
| 微服务重试策略 | Saga Runtime 步超时/重试逻辑 | 5 次重试指数退避（0s/1s/2s/4s/8s），耗尽后 SagaFailed + Manual Intervention（§4 重试表） |
| Definition 升级兼容 | `saga_definition` 版本路由 | 新 Saga 走最新未 deprecated 版本；in-flight Saga 按启动时 `definition_id` 版本跑完，不得中途切版本（§5） |
| CI | fmt、clippy、test、deny checks | 负例必须阻断合并 |

## 3. 实现契约

- Recovery Worker 抢占 SQL **必须**使用 `SELECT ... FOR UPDATE SKIP LOCKED`（startup_scan 候选扫描）或 `WHERE owner_pod=? OR updated_at < NOW() - grace_period`（UPDATE 抢占），**不得**用应用层锁或轮询等待替代数据库层 OCC（§2/§3 硬约束）。
- 任何 Saga 表写入（`saga_step`/`saga_instance`）**必须**携带 `WHERE fence_token = ?` 校验；写入返回 0 rows affected 时**必须**视为"已失去持有权"，应用层需重新走抢占流程，**不得**忽略该结果继续写入（§3 第 4 点）。
- Fence Token **必须**通过 PostgreSQL `nextval('saga_fence_token_seq')` 单调递增分配，**不得**使用应用层自增计数器或时间戳代替（防止多副本时钟漂移导致 token 冲突，§3 第 1 点）。
- Recovery Worker 的 grace period（默认 60s）内，`owner_pod` 不匹配但 `fence_token` 未过期的 Saga **不得**立即抢占，须待 grace period 过后二次检测仍 stale 才抢占（§2 alt 分支，防止误抢占仍存活的 Pod）。
- 微服务 Pod 重启导致命令超时时，Saga Runtime **不得**立即判定失败，须按 §4 重试表（5 次指数退避 0s/1s/2s/4s/8s）重试耗尽后才转 SagaFailed + Manual Intervention（§4 硬约束，"Pod 重启不立即失败"）。
- Saga Definition 升级时，in-flight 使用旧版本的 Saga **必须**按启动时的 `definition_id` 版本跑完，**不得**中途切换到新版本（§5"关键约束"）；Saga Definition schema 变更**只能**新增 optional 字段，**不得**删除已有 v1 step 字段（向后兼容硬约束）。
- Reaper Worker（§8 `reaper_loop`）**必须**仅将 `expires_at < NOW()` 且状态为 RUNNING/WAITING/RETRYING 的 Saga 标记为 EXPIRED，**不得**对 COMPENSATING 状态的 Saga 做同样处理（补偿中的 Saga 不得被 Reaper 强制打断，避免补偿链路悬空）。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 指标（`saga_recovery_*`）：`startup_scan` 恢复计数（按 `outcome`=recovered/skipped 分组）、抢占失败计数（0 rows affected 事件）、`heartbeat_loop` 续约延迟、`reaper_loop` EXPIRED 标记计数、Recovery Worker 崩溃重启计数。
- 指标标签：仅 `pod_id`（低基数，副本数量级）/`saga_type`/`state`/`outcome` 等标签；`saga_id` **不**作为 metric label（高基数）。
- Saga 状态机的每次非法转移尝试（如 COMPLETED → RUNNING）**必须**产生告警级别日志，归入 OPERATION_AUDIT（异常路径，需人工介入排查根因）。
- 关键请求必须能用 `saga_id` + `trace_id` 反查 Recovery Worker 的抢占/续约历史（结合 DTL-100 Saga Store `saga_event`/`saga_audit`）。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 一致性 | Fence Token OCC 三段契约（抢占/续约/写入校验）在多副本并发场景下无双写（Chaos 测试验证，见§6 ST） |
| 故障 | K3s Pod crash 后 60s grace period 内不被误抢占；60s 后被正确抢占并从 snapshot/journal 恢复（§2 场景） |
| 幂等一致性 | 抢占后重发未 ACK 的 command 须走 `idempotency_key` 幂等（依赖 DTL-100 Inbox 契约，本规格不重复定义），不得因恢复流程产生重复副作用 |
| 升级安全 | Saga Definition 升级期间 v1/v2 双版本并行，Rolling Update（`maxUnavailable=0, maxSurge=1`），in-flight v1 Saga 不受影响（§5） |
| 不可恢复补偿 | Compensation 自身失败（§6 场景表 4 类）**必须**进入 Manual Intervention Queue，GM Saga Console 提供 Pause/Resume/Retry/Manual Compensate/Cancel 五种介入操作，且需 2FA + Audit Log（§6"GM 介入工具"） |
| 发布 | Recovery Worker 的 `grace_period`/`scan_interval`/`snapshot_interval` 三项超参数变更须走配置发布流程，不得硬编码热改 |

## 6. 测试规格

- UT：覆盖 Saga Instance 11 状态转移表（合法/非法转移路径）+ Fence Token 抢占 SQL 的 0 rows affected 分支处理 + Reaper Worker 对 COMPENSATING 状态的排除逻辑。
- IT：覆盖 §2 完整 Crash Recovery 时序（Pod A crash → grace period 等待 → Pod B 抢占 → snapshot/journal 恢复 → 未 ACK command 重发）+ §4 微服务重启重试表（5 次退避 + 耗尽转 Manual Intervention）+ §5 Definition v1/v2 双版本路由（新 Saga 走 v2，in-flight v1 Saga 跑完）。
- ST：多副本（3 replica）并发抢占同一 Saga 的 OCC 冲突验证（仅 1 个 Pod 抢占成功，其余 0 rows affected 并正确回退）；对应 DTL-100 §7 Saga Store Schema 联合验证。
- Chaos：K3s Pod 随机 kill 注入，验证 in-flight Saga 在 grace period 后被正确接管且无重复执行副作用；NATS JetStream 消息重复投递下 Inbox 幂等仍生效（与 RGS-DTL-100 联合验证）。
- Security：验证 GM Saga Console 介入操作（Pause/Resume/Retry/Manual Compensate/Cancel）均要求 2FA 且写入 Audit Log，非 GM 角色调用返回权限拒绝。

测试必须回填 RGS-REQ-004 追踪矩阵（Saga/COC 相关 AC 项）和 DTL-102 §7 故障自检表全部 14 项；不能只证明"服务启动"。

## 7. Definition of Done

- RGS-DTL-102 的状态机、Crash Recovery、HA、微服务重启兼容、Definition 升级兼容、Compensation 不可恢复处理与实现逐项对账。
- Cargo fmt、clippy、test、deny 检查通过。
- §7 故障自检表 14 项在实现完成后逐项复核为"✅"并附实测证据（不得沿用 DTL 文档中的设计期断言作为实现期证据）。
- 与 RGS-DTL-100（Saga 业务模式）、RGS-DTL-101（OperationPolicy/AuthorityBoundary）的接口契约联合验证通过（三份 DTL 为同侪文档，非独立可验收）。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

进入实现前必须取得：① 源 DTL RGS-DTL-102 的具名 DD Review；② RGS-DTL-100/RGS-DTL-101 同侪文档已定稿（三者共同构成 Saga 子系统完整设计，不得单独进入实现）；③ 多副本 OCC 并发抢占实测（3 replica Chaos 测试，验证无双写）；④ K3s Pod crash-to-recovery 实测延迟（grace period 60s + 抢占 + snapshot 恢复总耗时，需纳入 SLO 预算核验）。**本规格不覆盖**：`saga_snapshot` 快照生成频率（`snapshot_interval`=30s）的性能调优最终值——DTL §8 已标注为默认值，最终调优留待实测后在 DTL 修订版本中更新，本规格仅承接当前默认值实现，不作为 Gate 阻塞条件。

---

## A. v0.2 对齐说明（2026-08-26，基于源 DTL 今日状态）

> **本节定位**：本 SPEC v0.2 是 17 份"未升版 DTL"的前瞻性 v0.2 草案（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2）。**不引入新设计**——仅落实/复核源 DTL 当前版本 + 父 BAS 既有内容；正文本 §1~§8 不重写，新增内容仅本节。

### A.1 源 DTL 今日升版增量（前瞻性视角）

- **源 DTL**：RGS-DTL-102
- **源 DTL 今日状态**：`0.1`（`2026-08-21`）
- **源 DTL 升版路径**：**今日未升版**（`git log --since="2026-08-26 00:00" --until="2026-08-26 23:59" -- docs/**/RGS-DTL-102_*.md` 无 commit）
- **源 DTL 升版类型**：**前瞻性草案**（非"今日升版沉淀"）
- **核心要点**：源 DTL 头表末次升版为 `0.1（初版）`（2026-08-21），制定者 架构师（Ulysses 兼，per DEC-008 一人公司），修订历史仅一行（初版：Saga Instance 状态机 / K3s Pod Crash Recovery / Saga Runtime HA 多副本 OCC / 微服务 Pod 重启兼容 / 升级兼容性 / 故障自检表）。本 SPEC v0.2 不复用任何 TBD 升版内容。

### A.2 对本 SPEC 的影响（实现侧）

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.1 同步 | 与源 DTL `0.1` 同步（范围不变，仅元数据对齐） |
| 源 DTL 真源 | RGS-DTL-102 v0.1 | RGS-DTL-102 `0.1`（具体修订见 §A.1） |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review（本 SPEC v0.2 不阻塞） |
| §8 Gate 证据 | 待 ① 源 DTL DD Review ② Rust 1.98 stable CI ③ PostgreSQL 18.6 迁移演练 ④ K3s 能力核验 | 同 v0.1（本前瞻性草案不新增 Gate） |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全（per DTL-036 v1.4.1 hotfix 复盘 §修式）。本节列出来源 DTL 升版自身声明的待办 / 缺口，本 SPEC 不预设处置方案，待 DDD Review 阶段配套决策。

- 源 DTL 升版内容仅为 前瞻性草案（本 DTL 今日未升版）时，本节无新缺口继承。
- 若源 DTL 升版伴随 §3 已知缺口清单（如 RGS-DTL-036 v1.4.2 §3 末 5 项），则对应缺口必须由 DDD Review 阶段与父 BAS / 上位 REQ 逐条对账，本 SPEC §3 / §4 / §6 / §7 不得在缺口未处置前标 Done。
- 本 SPEC v0.2 自审 0 项发现，**等 DDD Review 阶段再行检查**。

### A.4 引用链与证据

- 源 DTL 修订历史条目：见 RGS-DTL-102 §修订历史表（本 DTL 今日未升版，引用最新一次历史升版）
- 父 BAS 升版条目：见对应父 RGS-BAS-NNN §修订历史表（本 DTL 对应父 BAS，本日是否升版需自审）
- 同期 SPEC 调整总报告：[RGS-SPEC-000 详细设计规格化总表 v0.3](../RGS-SPEC-000_详细设计规格化总表.md) + RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2_v0.1.md（17 份前瞻性 SPEC v0.2 同批）
- **代签已允许**（per 2026-08-26 08:40 JST 偏好反转）：本节"审批者"列 = 真实责任署名 "架构师(Mavis 接手 agent per DEC-008)"，**不**再受"审批者 = —"硬约束（原占位状态见 git 历史）

> **本 v0.2 调整严格遵循**：① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 代签已允许（新规则） ⑤ 缺标比错标更安全（per DTL-036 hotfix 复盘修式）。
