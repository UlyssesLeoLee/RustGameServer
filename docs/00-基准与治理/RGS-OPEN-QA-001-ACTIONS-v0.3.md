# RGS-OPEN-QA-001 下游动作跟踪表 v0.3

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OPEN-QA-001-ACTIONS |
| 版本 | 0.3（首次产出，per 24 条已答复疑问的「下游动作」集中跟踪）|
| 状态 | 🟢 跟踪表已建立，重量级动作待 Ulysses 确认是否升 L4 任务 |
| 依据 | RGS-OPEN-QA-001 v0.2（24 条全部答复）+ RGS-REV-011 v0.1（6 项缺口 follow-up 提议 8 个新 L4 任务）+ RGS-WBS-001 v0.3 瀑布式 + v0.6 进度表 + RGS-DEC-NOGO-001 v0.1 + RGS-IMPL-001 |
| 范围 | 24 条已答复疑问的 ~26 处「下游动作」标注 → 去重后约 22 个独立动作 |
| 不在本跟踪表范围 | RGS-OPEN-QA-001 v0.1.md 历史疑问原文（per 文档末"只能追加不修改历史"约束）|
| 责任人 | AI worker（跟踪表维护）+ Ulysses（重量级动作升 L4 决策）|
| 父文档 | [RGS-OPEN-QA-001 设计与制造编程疑问集 v0.1](RGS-OPEN-QA-001_设计制造编程疑问集_v0.1.md) |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.3 | 2026-08-24 | AI worker | **首次产出**：24 条答复栏下游动作去重 + 修正（编号冲突/骨架填充/NO-GO 已解除）+ 工作量分级（12 重量级 / 10 轻量级）|

---

## 1. 文档目的

RGS-OPEN-QA-001 v0.2 答复栏散落了 **26 处**「下游动作：...」标注。这些动作目前**只是文字标注，尚未执行**，也没有集中管理 —— 容易漏项、重复、或与既有编号冲突。

本跟踪表的作用：

1. **集中追踪**：24 条疑问的 ~26 个下游动作归一为 **23 个独立动作**
2. **修正编号冲突**：答复里给的 DTL-037/038 与既有文档冲突，按"按当前最大编号顺延"原则修正为 DTL-043/044
3. **修正填充 vs 新建**：C ROSS-001/007 是**填充**已存在骨架（v0.1 占位，NO-GO 已解除可直接升 v0.2），不是新建
4. **工作量分级**：13 重量级动作（开 L4 任务）/ 10 轻量级动作（直接完成）
5. **去重合并**：Q-D-09 与 Q-M-10、Q-G-01 与 Q-G-02、Q-M-08 与 Q-M-10 等合并为同一动作

---

## 2. 工作量分级标准

| 分级 | 判据 | 示例 | L4 任务 |
|---|---|---|---|
| **🟢 轻量级** | 填一节骨架文档 / 加一条 SOP checklist / 改一处状态标记 / 补一段说明 | 填 RGS-SPEC-CROSS-001 §2；DTL-019 v0.2 §3 加 4 渠道重试策略；RGS-TS-001 §5 改"已决策"标注 | **不开**，跟踪表直接完成 |
| **🔴 重量级** | 新建完整 DTL 文档 / 新建 ADR 或 DEC 审批包 / 跑 benchmark / 写新脚本框架 / 补多域集成测试套件 / 需论证 + 验证证据的 | 新建 DTL-043 v0.1 消息分发；RGS-DEC-019 PFAU RTO 分级；reservation IT + 混沌测试 | **开新 L4**（worktree 隔离 + 完成判据 + 验证证据）|

---

## 3. 跟踪表（22 个独立动作）

> **列定义**：
> - 序号：本表独立编号
> - 来源疑问 ID：RGS-OPEN-QA-001 v0.2 里的疑问编号（多条用 `,` 分隔 = 合并）
> - 动作描述：去重 + 修正后的最终动作
> - 目标文档/产物：影响的具体文档或代码
> - 类型：新建文档 / 填充骨架 / 改代码 / 升版 ADR-DEC / 建 tracking 脚本 / 补 SOP / 补测试
> - 优先级：继承来源疑问的 P0/P1/P2（合并时取最高）
> - 前置依赖
> - 工作量：🟢 轻量 / 🔴 重量
> - 状态：⬜ 未开始 / 🟡 进行中 / ✅ 完成

### A 设计（10 条 → 10 个动作）

| 序号 | 来源疑问 | 动作描述 | 目标文档/产物 | 类型 | 优先级 | 前置依赖 | 工作量 | 状态 |
|---|---|---|---|---|---|---|---|---|
| A-01 | **Q-D-01**（已修正：037→**043**）| 新建 DTL-043 v0.1 消息分发（含 3 张主表 `messages`/`message_recipients`/`conversations`），直接进 1.0 状态 | `docs/07-社交运营与玩家治理/RGS-DTL-043_消息分发_v0.1.md`（**新建**）| 新建文档 | **P0** | DTL 编号现状盘点（见 §5 修正 #1）| 🔴 重量 | ⬜ |
| A-02 | **Q-D-02**（已修正：038→**044**）| 新建 DTL-044 v0.1 player 主表（`players`/`player_characters`/`player_inventory`）+ 反向补 `0001_init.sql` 已有 `players`/`player_sessions` 的文档说明 + 补 `player_characters`/`player_inventory` migration | `docs/02-运维安全与网络/RGS-DTL-044_player主表_v0.1.md`（**新建**）+ `crates/player-service/migrations/0004_*.sql`（**新建**）| 新建文档 + 改代码 | **P0** | DTL 编号现状盘点；migration 字段级 schema 确认 | 🔴 重量 | ⬜ |
| A-03 | **Q-D-03**（已修正：扩展 CROSS-**001** 而非 CROSS-002）| **填充**已存在的 `RGS-SPEC-CROSS-001_错误码字典_v0.1.md` 骨架，NO-GO 已解除可直接升 v0.2：① 沿用既有 4 位段（0001-0999 通用/1001-1999 player/2001-2999 economy/3001-3999 match/4001-4999 social/5001-5999 admin/6001-6999 cluster-ops）② 段内子类细分（00-19 校验/20-39 状态冲突/40-59 资源不足/60-79 外部依赖/80-99 内部错误）③ 跨域公共错误不占用 4 位域内码（用 gRPC status 表达）④ gRPC status（传输层）+ 域内 4 位 code（业务语义层）双层 | `docs/13-实现规格/RGS-SPEC-CROSS-001_错误码字典_v0.2.md`（v0.1→v0.2 升版）| 填充骨架 | **P1** | NO-GO 已解除（per RGS-DEC-NOGO-001 v0.1）| 🟢 轻量 | ⬜ |
| A-04 | **Q-D-04**（已修正：填 CROSS-**007** 而非新建 RGS-RBAC-001）| **填充**已存在的 `RGS-SPEC-CROSS-007_5域RBAC角色矩阵_v0.1.md` 骨架，NO-GO 已解除可直接升 v0.2：① 资源粒度 = 动作级（`player.read`/`player.write`/`player.audit`），域级（`player.*`）做聚合别名，不做行级 ② 角色枚举（GM/SRE/PM/业务方）登记在 CROSS-007 ⑤ OPA/Casbin 集成 PH-2 之后，PH-1 用枚举 + 中间件 fail-closed | `docs/13-实现规格/RGS-SPEC-CROSS-007_5域RBAC角色矩阵_v0.2.md` | 填充骨架 | **P1** | NO-GO 已解除 | 🟢 轻量 | ⬜ |
| A-05 | **Q-D-05** | 新建 RGS-DEC-019 PFAU RTO 分级（per Q-D-05）：① 13min 公式拆解（解释 780s ≠ 300s+120s 的端到端最坏估计来源）② RTO 分级方案（自动化可恢复路径走 5min，跨域需人工兜底故障走 15min PFAU 专属分级）③ 冻结 300s/120s 之前的论证（解决 13min > 5min RTO 字面冲突）| `docs/00-基准与治理/RGS-DEC-019_PFAU_RTO分级_v0.1.md`（**新建**）| 升版 DEC | **P1** | handoff §4.3 原始逐段计算回溯 | 🔴 重量 | ⬜ |
| A-06 | **Q-D-06** | ADR-0052 v0.2 修订：① 单副本容量 50-70k DAU / 5-7k QPS（双副本共享总容量 100k/10k）② all-reachable PFAU 拓扑下仲裁机制（leader lease / 分布式锁 / CRDT 式收敛，必须补充实现细节）③ 容量重算与 ADR-0052 同步修订 | `docs/08-架构决策记录/RGS-ADR-0052_Active-Active_ClusterOpsService与all-reachable_PFAU容错哲学_v0.2.md`（v0.1→v0.2 升版）| 升版 ADR | **P1** | 容量公式实测（NFR-OP-010 重算）| 🔴 重量 | ⬜ |
| A-07 | **Q-D-07** | DTL-026 v0.2 §7.1 补"自实现 Glicko-2"决策说明：① 不引入 `glicko-rs`（自实现约 200 行）② `rating`/`rd`/`volatility` 三元组作为持久化契约（跨版本兼容）③ τ 等 decay 参数先用 DTL §7.1 默认值，PH-1 不调优 | `docs/07-社交运营与玩家治理/RGS-DTL-026_详细设计书.md` §7.1 补段落 | 升版 DTL | **P1** | 无 | 🟢 轻量 | ⬜ |
| A-08 | **Q-D-08**（与 Q-D-01 合并处理）| DTL-019 v0.2 升版：① 去掉消息分发（拆给 DTL-043）② 4 渠道抽象（站内信/邮件/推送/短信）放在 §3 ③ 第三方网关适配层（APNs/FCM/SMTP/SMS）不在 PH-1 范围，PH-1 用 mock/stub 网关跑通链路 ④ 渠道重试策略：push 不重试（用户体验优先），邮件/短信 3 次指数退避 | `docs/07-社交运营与玩家治理/RGS-DTL-019_详细设计书.md` v0.1→v0.2 升版 | 升版 DTL | **P1** | A-01（DTL-043 新建）| 🟢 轻量 | ⬜ |
| A-09 | **Q-D-09 + Q-M-10**（合并）| 升版 RGS-SPEC-CROSS-003 v0.1→v0.2 填充骨架（NO-GO 已解除）：① **沿用既有命名** `rgs.events.<domain>.<aggregate>.<action>.<version>`（答复里写的"域.对象.动作"应理解为该模板的简写，不重定义）② 投递语义：至少一次 + 幂等消费（既有 outbox idempotent migration 落地）③ 补全订阅关系（**`MatchRatingChanged` → `PlayerRatingUpdated` 缓存失效** 等已知跨域事件）④ **NATS message header 格式冻结**（与 Q-M-03 的 `traceparent` 需求合并定义）| `docs/13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.2.md` | 填充骨架 | **P1** | NO-GO 已解除 | 🟢 轻量 | ⬜ |
| A-10 | **Q-D-10** | ① DTL-026 §4.1 补 n≤500 占位 + 降级/熔断策略（n 超上限先拆分撮合轮，降级后仍超才熔断 RetryAfter）② 新增 L4 任务：撮合 benchmark（**确定 n 上限的实测依据**）| `docs/07-社交运营与玩家治理/RGS-DTL-026_详细设计书.md` §4.1 补段落 + 新 L4（见 §4 B-04）| 升版 DTL + 新 L4 | **P2** | match 域 DDL 联检 | 🔴 重量 | ⬜ |

### B 制造/编程（10 条 → 9 个动作）

| 序号 | 来源疑问 | 动作描述 | 目标文档/产物 | 类型 | 优先级 | 前置依赖 | 工作量 | 状态 |
|---|---|---|---|---|---|---|---|---|
| B-01 | **Q-M-01** | ① DTL-015 v0.2 §3.4 新增「Saga 步骤编号映射」：1.0~6.0 对应 REV-005 附件 B 6 场景，场景内子步骤用 1.1/1.2 嵌套 ② DTL-016 v0.2 §3.4 同款 ③ **后**做 RGS-DEC-Q003 审批包（DTL 编号是 DEC 引用基础）| `docs/03-数据经济与交易/RGS-DTL-015_详细设计书.md` + `RGS-DTL-016_详细设计书.md` v0.1→v0.2 升版 | 升版 DTL | **P0** | 无（轻量级 DTL 升版）| 🟢 轻量 | ⬜ |
| B-02 | **Q-M-01**（DEC 部分，**重量级**）| 新建 RGS-DEC-Q003 跨 DB Saga 审批包：① 引用 DTL-015/016 §3.4 步骤编号 ② 6 场景决议 ③ 风险接受 + 补偿策略 ④ RACI ⑤ DTL-031 §8.2 解除阻断 | `docs/00-基准与治理/RGS-DEC-Q003_跨DBSaga审批_v0.1.md`（**新建**）+ DTL-031 §8.2 补 | 新建 DEC | **P0** | B-01（DTL 步骤编号先完成）| 🔴 重量 | ⬜ |
| B-03 | **Q-M-02** | 4 域（player/match/social/admin）各补 `rgs-testkit` dev-dependency + 同款集成测试骨架（`tests/integration_*.rs`，参考 `economy-service/tests/integration_outbox.rs`），约 4×0.5 人·天 | `crates/{player,match,social,admin}-service/Cargo.toml` + `crates/{player,match,social,admin}-service/tests/integration_*.rs` | 改代码 + 补测试 | **P0** | economy-service 现有 integration_outbox.rs 模板 | 🔴 重量 | ⬜ |
| B-04 | **Q-M-03** | ① 核实 WF-1-53.12 / WF-1-54.13 任务状态（OTel SDK 集成任务）② 若未完成优先落地 ③ NATS message header 加 `traceparent`（`async-nats` 0.42 已支持，不需升级），在 `shared-platform/src/producer.rs` / `consumer.rs` 手动注入/提取 ④ sqlx-tracing 10-20% 采样起步 ⑤ 5 域各自直接上报 OTLP（不经 cluster-ops 中转）| `crates/shared-platform/src/{producer,consumer}.rs` + 5 域 Cargo.toml sqlx feature + 5 域 OTLP exporter 配置 | 改代码 | **P0** | 53.12 / 54.13 任务状态确认 | 🔴 重量 | ⬜ |
| B-05 | **Q-M-04** | 写 CI/pre-commit 脚本 `scripts/verify_probe_consistency.ps1`（或 .yml）：① 6 份 manifest 的 probe 段做结构化 diff ② 阈值一致性全 6 份核对（**不只是抽查 2 份**）③ 任何一份 probe 段修改必须同步到其余 5 份（脚本/CI 校验而非人工记忆）④ `01-player-service.yaml` 的 `-connect-timeout=2s` / `periodSeconds=30/10` / `failureThreshold=3` / `timeoutSeconds=5/3` 6 份需逐一对照 | `scripts/verify_probe_consistency.ps1`（**新建**）+ CI 配置 | 建 tracking 脚本 | **P0** | 6 份 manifest probe 段当前实际参数全列 | 🔴 重量 | ⬜ |
| B-06 | **Q-M-05** | 补充证书轮转 SOP：① 手工 `kubectl apply` 流程（PH-1 范围）② 证书有效期提醒 ③ 轮转演练（测试 + 生产隔离）④ 自动化轮转标记为 PH-2 增强项 | `docs/deploy/04-env-setup-sop.md` 补一节（或新建 `docs/deploy/cert-rotation-sop.md`）| 补 SOP | **P1** | 当前证书有效期查询 | 🟢 轻量 | ⬜ |
| B-07 | **Q-M-06** | RGS-IMPL-100 v0.2 补 crate 选型确认段落（**代码已是现状，无需变更**）：① outbox 自实现（5 域已落地）② Saga step trait 用 native AFIT（Rust 1.98 支持）③ DLQ 落库（不落 JetStream）| `docs/13-实现规格/RGS-IMPL-100_Saga事务系统实施规范_v0.2.md` v0.1→v0.2 升版（一节补内容）| 升版 SPEC | **P1** | 无 | 🟢 轻量 | ⬜ |
| B-08 | **Q-M-07** | 新建 L4 子任务（3 个）：① `reservation` 集成测试（`tests/it_reservation_*.rs`，端到端 create→conflict→release/cleanup）② 混沌测试（DB 突然断开 / 死锁为 P1，row 被外部 DELETE 为 P2）③ OTel span 完整度断言（`reservation.create → saga.step → reservation.release/cleanup` 三层嵌套） | `crates/economy-service/tests/it_reservation_*.rs` + `crates/economy-service/tests/chaos_*.rs` | 补测试 | **P1** | economy-service IT 框架（Q-M-02 同款）| 🔴 重量 | ⬜ |
| B-09 | **Q-M-08 + Q-M-10**（Q-M-10 规范部分合并，代码部分见 B-04）| ① `scripts/verify_fail_closed.ps1`（下划线命名风格，与 `deploy_dev_k3s.ps1` 一致）固化 fail-closed 验证 ② 接入 CI：每次 manifest/RBAC 变更 PR 触发（不限新增域）③ 默认拒绝语义不变 ④ RGS-TS-001 §5 状态改"已决策：NATS JetStream"（去掉"未决"标注）| `scripts/verify_fail_closed.ps1`（**新建**）+ CI 配置 + `RGS-TS-001_主要技术选型报告.md` §5 改一行 | 建 tracking 脚本 + 改 TS-001 状态 | **P2** | phase-0-5 step 4 commit b9bc214 现有 fail-closed 验证内容 | 🔴 重量 | ⬜ |
| B-10 | **Q-M-09** | DTL 升版规范文档补"引用同步 checklist"一条：任何 DTL 升版时必须 grep 全仓库该 DTL 编号引用，逐一更新版本号标注（不追求实时自动化）| `docs/01-核心架构与设计模式/RGS-DTL-001_详细设计书.md` §1.3 DTL 升版规范（**或新建 DTL 升版 SOP 文档**）| 补 SOP | **P2** | 无 | 🟢 轻量 | ⬜ |

### C 治理/流程（4 条 → 3 个动作）

| 序号 | 来源疑问 | 动作描述 | 目标文档/产物 | 类型 | 优先级 | 前置依赖 | 工作量 | 状态 |
|---|---|---|---|---|---|---|---|---|
| C-01 | **Q-G-01 + Q-G-02**（合并）| ① 新建 RGS-ADR-0055 DEC-005/008 兼容论证（组织设计原则 vs 实际执行约束 + 流程化补偿清单逐项标负责 CI/工具）② RGS-PLAN-001 v1.0 §1.2 补 RACI 简表（4 类决策：代码合并/DTL 升版/生产发布/资金相关）③ 决策类别"生产发布"和"资金相关"的 A 必须 Ulysses 本人**显式签字**（不能用 PR review 替代）| `docs/08-架构决策记录/RGS-ADR-0055_DEC-005_008兼容论证_v0.1.md`（**新建**）+ `docs/12-工作流/RGS-PLAN-001_项目实施计划_v1.1.md` §1.2 补 RACI 表 | 新建 ADR + 升版 PLAN | **P0** | DEC-005/008 原文引用 | 🔴 重量 | ⬜ |
| C-02 | **Q-G-03** | RGS-TS-001 §6.2 补"worktree 共享池 + 软上限告警"具体阈值参数（**双轨制 OLU 框架下，token/周与人·天/周双轨并报**，不替代）：① 共享总池计数，每 worktree 软上限告警（如预估 8K token，超 150% 告警但不强制中断）② 跨 worktree 决策对话计入共享池 ③ 硬约束优先级：AI 上下文窗口 > 单次会话成本 | `docs/10-技术选型/RGS-TS-001_主要技术选型报告.md` §6.2 补一段 | 升版 TS-001 | **P1** | PH-1 首轮实测后校准 | 🟢 轻量 | ⬜ |
| C-03 | **Q-G-04** | ① WBS-001 §8 补 B-CODE/C-CODE log 新模板（强制包含"验证证据"字段：commit hash / 测试输出摘要 / CI run 链接，不允许只填"已完成"）② 现有 log 逐份核验（**实际是 7 G-CODE + 4 B-CODE = 11 份 log**，不是 11 份 B-CODE；逐份按 WBS §8.3 SOP 判定"已按 SOP 验证=完全 done"或"仅文档重写未附验证证据=partial 需补验证"）③ 完成判据：commit hash + 测试输出 + CI run 链接 | `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.7.md` §8 补新模板 + 11 份 log 逐份核验 | 升版 WBS + 补 SOP + 核验 | **P2** | WBS-001 §8.3 SOP 原文 | 🔴 重量 | ⬜ |

---

## 4. 重量级动作汇总（13 个，建议开新 L4）

> **L4 编号说明**：WBS 实际最大编号是 **WF-1-55.37**（per 瀑布式 WBS v0.3），REV-011 提议的 55.32~41 中 55.32~37 与既有任务冲突，**新 L4 任务需从 WF-1-55.38 开始顺延**（详见 §5 修正 #5）。

| 新 L4 # | 关联动作 | 建议 token | 前置 | 完成判据（不可空）|
|---|---|---|---|---|
| **WF-1-55.38** | A-01 新建 DTL-043 消息分发 | 8K | DTL 编号现状盘点 | DTL-043 v0.1 落地（含 3 张主表 DDL + 4 渠道抽象归属说明）|
| **WF-1-55.39** | A-02 新建 DTL-044 player 主表 + migration | 12K | DTL 编号盘点 + migration schema | DTL-044 v0.1 + 0001 反向 doc + 0004 migration（含 `player_characters`/`player_inventory` 表）|
| **WF-1-55.40** | A-05 RGS-DEC-019 PFAU RTO 分级 | 8K | handoff §4.3 回溯 | RGS-DEC-019 v0.1 + 13min 公式拆解 + RTO 分级方案 + DTL-031 §4.3 冻结 |
| **WF-1-55.41** | A-06 ADR-0052 v0.2 修订 | 8K | 容量实测 | ADR-0052 v0.2（含仲裁机制 + 50-70k/5-7k QPS 单副本容量）|
| **WF-1-55.42** | A-10 DTL-026 §4.1 benchmark 子任务 | 6K | match 域 DDL | benchmark 实测结果 + DTL-026 §4.1 补 n 上限 + 降级策略 |
| **WF-1-55.43** | B-02 RGS-DEC-Q003 跨 DB Saga 审批包 | 8K | B-01（DTL-015/016 §3.4）| RGS-DEC-Q003 v0.1 + 6 场景决议 + RACI + DTL-031 §8.2 解除 |
| **WF-1-55.44** | B-03 4 域 rgs-testkit 集成测试骨架 | 8K | economy 现有模板 | 4 域各 1 份 `tests/integration_*.rs` + cargo test pass |
| **WF-1-55.45** | B-04 OTel 启用 + NATS traceparent + sqlx-tracing + 5 域 OTLP | 12K | 53.12 / 54.13 状态确认 | NATS header `traceparent` 注入 + 5 域 OTLP exporter + sqlx-tracing 10-20% |
| **WF-1-55.46** | B-05 `verify_probe_consistency.ps1` CI 脚本 | 6K | 6 份 manifest probe 段全列 | 脚本落地 + 全 6 份 probe 段 diff 报告 + CI 接入 |
| **WF-1-55.47** | B-08 reservation IT + 混沌测试 + span 断言 | 12K | economy IT 框架 | `it_reservation_*.rs` + 混沌测试（DB 断开/死锁 P1）+ span 断言全 pass |
| **WF-1-55.48** | B-09 `verify_fail_closed.ps1` + CI 接入 + TS-001 §5 状态 | 6K | phase-0-5 step 4 现有 fail-closed 内容 | 脚本落地 + CI 接入 + TS-001 §5 状态改"已决策：NATS JetStream" |
| **WF-1-55.49** | C-01 ADR-0055 DEC-005/008 兼容 + RACI | 8K | DEC-005/008 原文 | ADR-0055 v0.1 + PLAN-001 v1.1 §1.2 RACI 简表（4 类决策）|
| **WF-1-55.50** | C-03 WBS §8 log 模板 + 11 份 log 逐份核验 | 10K | WBS §8.3 SOP | 新模板落地 + 11 份 log 逐份核验报告（每份标 done/partial）|

**总 token 估算**：~112K tokens（per RGS-TS-001 §6.2 双轨制 token-OLU，约 1.1-1.5 SRE·周 AI 协作；不替换人·天估算）

**L4 编号修正提醒**：上述 WF-1-55.38~50 是**建议编号**，但必须由 Ulysses 确认最终映射（避免与未来 REV-011 v0.2 / 后续评审的 8 个新 L4 重名）。建议 Ulysses 决策：① 直接采用 55.38~50 ② 或与 REV-011 §3 的 55.32~41 重新协调（会与 55.32~37 既有任务冲突，必须先做 5 段编号清理）。

---

## 5. 已知修正（与答复栏原文的偏差）

| # | 来源疑问 | 答复栏原文 | 修正后 | 修正理由 |
|---|---|---|---|---|
| **1** | Q-D-01 | "新建 `RGS-DTL-XXX_消息分发_v0.1.md`（编号按当前最大 DTL 号顺延，AI worker 落地时核实不冲突）" | **DTL-043 v0.1**（不是 037，037 已被 Economy 域占用）| 实际 DTL 编号已用到 042（DTL-041=客户端断点续传 / DTL-042=服务器全生命周期管理）。grep `docs/` 全仓确认 037/038/039/040/041/042 全部已用 |
| **2** | Q-D-02 | "新建 **DTL-038**（`players`/`player_characters`/`player_inventory`）" | **DTL-044 v0.1**（不是 038，038 已被 Match 域占用）| 同上，038=Match 域 |
| **3** | Q-D-03 | "新建或扩展 RGS-SPEC-CROSS-002 §2 登记命名空间分配表" | **填充 RGS-SPEC-CROSS-001 v0.1 升 v0.2**（不是扩展 CROSS-002）| CROSS-001 v0.1 §2.2 已定义 4 位段（0001-0999/1001-1999/.../6001-6999），是错误码字典的正确位置；CROSS-002 是 gRPC Proto 风格指南，主题不同 |
| **4** | Q-D-04 | "建议**单独新建 RGS-RBAC-001**" | **填充 RGS-SPEC-CROSS-007 v0.1 升 v0.2** | CROSS-007 v0.1 标题就是"5 域 RBAC 角色矩阵"，且 v0.1 是占位 NO-GO 状态，NO-GO 已解除可直接升 v0.2 |
| **5** | Q-G-02 | "REV-011 §3 提议 8 个 L4 任务（WF-1-55.32~41）" | **实际是 10 个**（55.32~41），且 55.32~37 已被瀑布式 WBS v0.3 占用（不同主题）| grep 瀑布式 WBS v0.3 确认：55.32=HI-3 fail-closed 启动 IT / 55.33=HI-D DC-1 终态 test / 55.34=ME-1 apply_atomic 弃用 / 55.35=ME-2/3 admin migration 注释 / 55.36=ME-4+LO-1/2/3 / 55.37=LO-4 补偿半途崩溃。**新 L4 须从 55.38 起编号** |
| **6** | Q-G-04 | "现有 11 份 B-CODE log 重写需逐份核实" | **实际 11 份 = 7 G-CODE + 4 B-CODE**（不是 11 份 B-CODE）| grep `docs/deploy/07-no-go-checklist*.md` 确认：G-CODE 7 份（01-07） + B-CODE 4 份（01-04）= 11 份 log。Q-G-04 答复里"11 份 B-CODE"是早期描述未更新 |
| **7** | Q-D-09 | "采用 '**域.对象.动作**' 命名（`economy.trade.completed`）" | **沿用既有 `rgs.events.<domain>.<aggregate>.<action>.<version>` 命名**（per RGS-SPEC-CROSS-003 v0.1 §2.2 已定义）| 答复里的"域.对象.动作"是 `rgs.events.*` 模板的简写，不重定义命名空间；按既有规范升 v0.2 补订阅关系即可 |
| **8** | Q-G-03 | "RGS-TS-001 §6.2 补充 'worktree 共享池 + 软上限告警' 具体阈值参数" | **TS-001 §6.2 v0.6 已定调"双轨制 OLU"**（人·天 + token 双轨并报，**不替代**）| v0.6 §6.2.6 校准路径 4 节点（PH-0.5/PH-1/PH-3/PH-7）已存在，本动作是补 worktree 共享池段而非重定义双轨制 |
| **9** | Q-M-10 | "RGS-TS-001 §5 状态改为'已决策：NATS JetStream'（去掉'未决'标注）" | 答复已准确，按答复执行即可 | 无修正，仅做合并（与 B-09 同一动作）|

---

## 6. 轻量级动作（10 个，跟踪表直接完成）

| 序号 | 关联动作 | 建议完成方式 | 验证证据 |
|---|---|---|---|
| A-03 | Q-D-03 错误码命名空间 v0.2 填充 | 单次编辑 1 份 SPEC，~3K token | `RGS-SPEC-CROSS-001_v0.2.md` 落地 + commit hash |
| A-04 | Q-D-04 RBAC 角色矩阵 v0.2 填充 | 单次编辑 1 份 SPEC，~2K token | `RGS-SPEC-CROSS-007_v0.2.md` 落地 + commit hash |
| A-07 | Q-D-07 DTL-026 §7.1 Glicko-2 自实现补段 | 1 节 DTL 补内容，~1K token | DTL-026 commit hash + diff 行号 |
| A-08 | Q-D-08 DTL-019 v0.2 升版 | 1 份 DTL v0.1→v0.2，~2K token | DTL-019 v0.2 + commit hash |
| A-09 | Q-D-09 + Q-M-10 SPEC-CROSS-003 v0.2 填充 | 1 份 SPEC 升版 + 冻结 header，~3K token | `RGS-SPEC-CROSS-003_v0.2.md` 落地 + commit hash |
| B-01 | Q-M-01 DTL-015/016 §3.4 Saga 步骤编号 | 2 份 DTL 各补 1 节，~2K token | DTL-015/016 commit + diff |
| B-06 | Q-M-05 证书轮转 SOP | 1 节 SOP 文档，~1K token | SOP 文档 commit + 流程图 |
| B-07 | Q-M-06 RGS-IMPL-100 crate 选型确认段 | 1 段补内容，~1K token | IMPL-100 v0.2 commit |
| B-10 | Q-M-09 DTL 升版规范"引用同步 checklist" | 1 条 SOP 补，~1K token | DTL 升版规范 commit |
| C-02 | Q-G-03 TS-001 §6.2 worktree 共享池段 | 1 段补内容，~2K token | TS-001 commit |

**总 token 估算**：~18K tokens（约 0.2 SRE·周 AI 协作）

**轻量级动作不在 WBS 单开 L4 任务**，但每次完成必须：
1. 走 `wbs_task_progress.ps1 -L4Id <父任务>` 进度登记（如有父 L4 任务）
2. 提交 commit + 引用 commit hash
3. 在本跟踪表里改状态 ⬜→✅

---

## 7. 边界与不做的事

| 边界 | 原因 |
|---|---|
| ❌ 不改 RGS-OPEN-QA-001 v0.1.md 历史疑问原文 | per 文档末尾"只能追加不修改历史"约束 |
| ❌ 不写 DTL-043/044 / CROSS-001/007 v0.2 实际内容 | 本任务只建跟踪表 + 建议 L4 任务；实际文档撰写是后续 WF-1-55.38~ 等任务 |
| ❌ 不直接升 WBS-001 v0.6 → v0.7 | 等 Ulysses 确认重量级分级（12 个建议 L4）+ 编号映射后再升版 |
| ❌ 不在跟踪表里自我标 ✅ | 状态变更必须经 Ulysses 终审签字 + 提供 commit hash / 测试输出证据 |

---

## 8. 关联文档

- **父疑问集**：[RGS-OPEN-QA-001 v0.1](RGS-OPEN-QA-001_设计制造编程疑问集_v0.1.md)
- **WBS**：[RGS-WBS-001 瀑布式 v0.3](..\12-工作流\RGS-WBS-001_瀑布式工作分解结构_v0.3.md) + [L4 进度表 v0.6](..\12-工作流\RGS-WBS-001_L4任务进度表_v0.4.md)
- **实施计划**：[RGS-PLAN-001 v1.0](..\12-工作流\RGS-PLAN-001_项目实施计划_v1.0.md)
- **技术选型**：[RGS-TS-001 v0.6](..\10-技术选型\RGS-TS-001_主要技术选型报告.md)
- **NO-GO 解除决议**：[RGS-DEC-NOGO-001 v0.1](RGS-DEC-NOGO-001_一人公司NO-GO解除决议_v0.1.md)
- **REV-011 6 项缺口**：[RGS-REV-011 v0.1](reviews\RGS-REV-011_5域DTL_6项缺口FollowUp_v0.1.md)
- **Worktree 模式**：[RGS-WT-001 §11](..\12-工作流\RGS-WT-001_GitWorktree隔离开发方案.md)

---

> **本跟踪表由 AI worker 维护，Ulysses 终审**。状态变更规则：⬜→🟡 时必须填"开始日期 + worktree 路径"；🟡→✅ 时必须填"完成日期 + commit hash + 验证证据"。**不强制每次轻量级动作都开会签**（10 个轻量级合计 ~18K token 适合 1 个 L4 任务批量完成）。
