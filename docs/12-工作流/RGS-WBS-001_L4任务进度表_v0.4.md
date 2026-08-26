# RGS-WBS-001 L4 任务进度表 v0.7

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001-ADD3 |
| 版本 | 0.8（v0.7 → v0.8：**WF-1-55.77~79 新增 3 个 P2 L4 任务**（per Ulysses 2026-08-26 09:27 JST "完成剩余工作到 P2" + RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分），承接 P0/P1 8 commit 已落地后的下一步；§3 汇总 L4 总数 142 → 145，pending 137 → 140；§4 详细表新增 3 行 pending（**§2A.2.55.续3** 段，编号起点 55.77 = v0.7 末位 55.50 之后）；修订历史加本版记录。本版本不重命名文件名（保留 v0.4.md 文件名，只是 head 升 v0.8），避免 git mv 触发无关 diff）|
| 依据 | RGS-WBS-001 v0.3 §2A L4 任务清单 + §6.3 进度字段 + §13 跨会话恢复 + RGS-OPEN-QA-001-ACTIONS v0.3 |
| 状态 | 🟠 **占位**（NO-GO 未解除前为空表；G-CODE-06 实测通过后由 wbs_task_progress.ps1 自动填充）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 父文档 | [RGS-WBS-001 瀑布式工作分解结构 v0.3](RGS-WBS-001_瀑布式工作分解结构_v0.3.md) |
| 关联 | RGS-WT-001 v0.2 §11（WBS L4 任务 worktree 模式）+ scripts/wbs_*.ps1 + phase-0-5 反馈单 Issue 5 + RGS-OPEN-QA-001-ACTIONS-v0.3 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.3 | 2026-08-21 | Ulysses | **占位初版**：per RGS-WBS-001 v0.3 §6.3 + §13 跨会话恢复。状态为占位，等 G-CODE-06 实测通过后由 `wbs_task_progress.ps1` 自动填充。 |
| 0.4 | 2026-08-24 | Ulysses（一人公司 12 角色全签 per DEC-008）| **NO-GO 形式上解除后手工补登**：§3 WF-0.5 阶段 7→4 pending + 3 done；§4 手工补登 WF-0.5-1/2/3（Phase 0.5 Step 1+5 / 2+3 / 4 部署）三行 done。WF-0.5-6（worker 失败）由主对话接手，状态仍为 pending（不计入 done）。 |
| 0.5 | 2026-08-24 | worker-self（per DEC-008 一人公司治理基线，phase-0-5/feedback-handler worktree commit 见行末）| **WF-1-55.27 真修合并入 main**（per phase-0-5 反馈单 Issue 4）：① §3 汇总 WF-1 113 pending → 112 pending + 1 done；合计 125 pending → 124 pending + 4 done ② §4 任务表加 1 行 WF-1-55.27 done 100%，commit 是 merge 后 hash（保留 c96efe8 + a80fa94 + f6a6f3f + 14036d6 4 个原始 commit 的关联）③ 新增 §8 WBS 状态维护 SOP（per 反馈单 Issue 5）锁定「手工编辑 v0.X 进度表写 done 100%」与「攒到后续 Phase 再补」两个 anti-pattern。 |
| 0.6 | 2026-08-24 | worker-self（per DEC-008）| **WF-0.5-8 handoff state sync done**（per handoff §11.6/§11.7/§11.8 状态同步 commit，4 B-CODE 等 SRE 接力后 v0.7 升版）：① §3 汇总 128 → 129 L4 任务，4 done → 5 done（WF-0.5-8 done），124 pending 不变 ② §4 详细表加 1 行 WF-0.5-8 done 100%，commit 是 merge hash ③ 修订历史加本行。**本版本不重命名文件名**（保留 v0.4.md 文件名，只是 head 升 v0.6），避免 git mv 触发无关 diff。 |
| **0.7** | **2026-08-24** | **worker-self（per DEC-008）** | **WF-1-55.38~50 新增 13 个 L4 任务 + §8.5 log 模板 + §9 核验报告引用**（per RGS-OPEN-QA-001-ACTIONS-v0.3 §4 重量级动作汇总；来源 RGS-OPEN-QA-001 v0.2 24 条已答复疑问的下游动作去重 + 工作量分级；编号起点 = 瀑布式 WBS v0.3 实际最大编号 55.37 + 1 = 55.38，**绕开 REV-011 提议的 55.32~41 与既有任务 55.32~37 的编号冲突**）：① §3 汇总 129 → 142 L4 任务，pending 124 → 137，5 done 不变 ② §4 详细表新增 13 行 pending ③ §8.5 新增 B-CODE/C-CODE log 强制验证证据模板（per OPEN-QA-001 Q-G-04）④ §9 引用 `docs/deploy/code-logs-verification-report.md` 11 份 log 逐份核验报告（7 G-CODE + 4 B-CODE）。**本版本不重命名文件名**（保留 v0.4.md 文件名，只是 head 升 v0.7），避免 git mv 触发无关 diff。 |
| **0.8** | **2026-08-26** | **架构师（Mavis 接手 agent per DEC-008）** | **P2 推进：3 新 L4 任务（WF-1-55.77 WBS 升版 + WF-1-55.78 5 域 Lead RACI v1.0 + WF-1-55.79 CROSS-008~012 二次校正）**（per Ulysses 2026-08-26 09:27 JST 指令"开子代理和 worktree 完成剩余工作到 P2"，承接 P0/P1 8 commit 已落地后的下一步；来源 = RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分）：① §3 汇总 142 → 145 L4 任务，pending 137 → 140 ② §4 详细表新增 3 行 pending（**§2A.2.55.续3** 段，编号起点 55.77 = v0.7 末位 55.50 之后）③ §A 已知缺口更新（per P0/P1 8 commit 落地后回填）。**本版本不重命名文件名**（保留 v0.4.md 文件名，只是 head 升 v0.8），避免 git mv 触发无关 diff。 |

---

## 1. 文档目的

本表是 **WBS L4 任务实时进度**的集中视图：
- **数据源**：所有 L4 任务 worktree 根目录的 `.wbs-task-marker` JSON 文件
- **更新机制**：`wbs_task_progress.ps1 -L4Id <id> -Status {start|progress|done|blocked} [-Progress N]`
- **视图刷新**：`wbs_list.ps1 -Summary` 自动汇总各 stage 状态

**为什么单独一张表（不直接读 .wbs-task-marker）**：
1. 人类 review / PM 签字用（不直接进 worktree 看 JSON）
2. 跨 worktree 状态汇总（避免每个 worktree 单独看）
3. 历史 archive（marker 可能在 worktree 删除后丢失，但本表保留历史）

## 2. 进度字段定义（per RGS-WBS-001 v0.3 §6.3）

每个 L4 任务的状态机：

```
pending → in_progress (start) → done (done)
                    ↓
                 blocked (blocked) → in_progress (start)
```

| 字段 | 类型 | 取值 |
|---|---|---|
| `l4_id` | string | `WF-X-XX.X` 格式 |
| `status` | enum | `pending` / `in_progress` / `done` / `blocked` |
| `progress` | int | 0-100（仅 in_progress 时有效）|
| `started_at` | ISO 8601 | 首次 start 时间 |
| `updated_at` | ISO 8601 | 最近一次更新 |
| `worktree` | path | worktree 绝对路径 |
| `blocked_reason` | string | 仅 blocked 时填 |

## 3. 进度汇总（实时表）

> **本节由 `wbs_list.ps1 -Summary` 自动生成**。v0.5 状态：NO-GO 形式上解除，WF-0.5 阶段 3 个 done + WF-1 阶段 1 个 done（WF-1-55.27 真修）已手工补登（其余仍 pending，等 wbs_task_progress.ps1 实测通过后自动覆盖）。

| Stage | 任务数 | pending | in_progress | done | blocked |
|---|---|---|---|---|---|
| WF-0 | 2 | 2 | 0 | 0 | 0 |
| WF-0.5 | 8 | 4 | 0 | 4 | 0 |
| WF-1 | 129 | 128 | 0 | 1 | 0 |
| WF-2 ~ WF-7 | 6 | 6 | 0 | 0 | 0 |
| **合计** | **145** | **140** | **0** | **5** | **0** |

> **注**：WF-0 / WF-7 等 9 个 stage 大类行不计入 L4 任务数；实际 L4 任务数 = 142 - 7（阶段占位）= 135。详见 `wbs_list.ps1 -Summary` 输出。
>
> **v0.6 变化**：WF-0.5 阶段 3 done → 4 done（WF-0.5-8 新增 done），合计 4 done → 5 done。
>
> **v0.7 变化**：WF-1 阶段 113 → 126 任务（+13 来自 WF-1-55.38~50 OPEN-QA-001 重量级动作），pending 112 → 125，合计 129 → 142。
>
> **v0.8 变化**：P0/P1 8 commit 落地后，P2 阶段启动。142 → 145 L4 任务（+3 来自 WF-1-55.77~79 P2 推进），pending 137 → 140，5 done 不变（v0.7 已含 WF-1-55.49 ADR-0055 done 由 v0.4 期间的 P0 任务回填）。新增段 **§2A.2.55.续3** 3 个 L4 任务。

## 4. L4 任务详细进度表

> **本节由 `wbs_task_progress.ps1` 调用时自动更新**。v0.4 状态：NO-GO 形式上解除，WF-0.5-1/2/3 三行 done 已手工补登（其余仍 pending，等 G-CODE-06 实测通过 + wbs_task_progress.ps1 调用后开始自动覆盖）。

| L4 # | 任务摘要 | owner | status | progress | 启动时间 | 备注 |
|---|---|---|---|---|---|---|
| WF-0.5-1 | Phase 0.5 Step 1+5 部署（5 域 manifest + docker image） | Ulysses（per DEC-008）| done | 100% | 2026-08-24T07:30:00+09:00 | commit 7046936（per 1190 行 + 4467080）|
| WF-0.5-2 | Phase 0.5 Step 2+3 部署（NATS + OTel/Prom/Grafana） | Ulysses（per DEC-008）| done | 100% | 2026-08-24T07:30:00+09:00 | commit c5a0c9f（per 1897 行 + 1183515）|
| WF-0.5-3 | Phase 0.5 Step 4 部署（mTLS + Secret + fail-closed） | Ulysses（per DEC-008）| done | 100% | 2026-08-24T07:30:00+09:00 | commit 765930a（per 1457 行 + 2b70b0b）|
| **WF-1-55.27** | **ReserveHandler OCC cleanup + reservation release 失败路径真修（per RGS-REV-009 CR-1）** | **Ulysses（per DEC-008）** | **done** | **100%** | **2026-08-24T18:00:00+09:00** | **merge commit 49d93b5（per phase-0-5/feedback-handler worktree `--no-ff` merge）**；原 branch commit `c96efe8` 真修（reservation.rs + saga_orchestrator.rs +159/-4）+ `a80fa94` 6 域 outbox migration（CR-2）一并合并；`f6a6f3f` PgTestDatabase fixture（CR-3，#[sqlx::test] 强约束）+ `14036d6` tag/marker 收尾同时入 main。**与原 branch 验证一致**：`cargo test -p economy-service --lib` 50/50 pass。 |
| **WF-0.5-8** | **handoff §11 状态同步(§11.6/§11.7/§11.8 标 Closed + 修订历史加 1 行 + 当前状态 3→10 closed 修正,per phase-0-5 反馈单 Issue 5 anti-pattern 修复)** | **Ulysses(per DEC-008)** | **done** | **100%** | **2026-08-24T20:42:00+09:00** | **feature commit `6b4a98c`(per codex/wt-handoff-state-sync worktree,3 段改 Closed + 加 commit 引用 + 修订历史);follow-up commit `b21e470`(per codex/wt-final-touchups worktree,WBS §4 占位 → `6b4a98c` + handoff §11 line 359 "3→10 closed" 修正)** |
| **WF-1-55.38** | **新建 DTL-043 v0.1 消息分发（per RGS-OPEN-QA-001 Q-D-01 + ACTIONS-v0.3 A-01）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **新建 3 张主表 DDL（messages/message_recipients/conversations）+ 4 渠道抽象归属说明；直接进 1.0 状态** |
| **WF-1-55.39** | **新建 DTL-044 v0.1 player 主表 + 0001 反向 doc + 0004 migration（per RGS-OPEN-QA-001 Q-D-02 + ACTIONS-v0.3 A-02）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **DTL-044 含 players/player_characters/player_inventory 字段级 DDL；0001_init.sql 已有 players/player_sessions 补文档说明；0004 migration 补 player_characters/inventory** |
| **WF-1-55.40** | **新建 RGS-DEC-019 PFAU RTO 分级（per RGS-OPEN-QA-001 Q-D-05 + ACTIONS-v0.3 A-05）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **13min 公式拆解 + RTO 分级方案（5min 自动化 / 15min 跨域 PFAU 兜底）+ DTL-031 §4.3 冻结 300s/120s** |
| **WF-1-55.41** | **ADR-0052 v0.2 修订（per RGS-OPEN-QA-001 Q-D-06 + ACTIONS-v0.3 A-06）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **单副本 50-70k DAU / 5-7k QPS 容量 + all-reachable PFAU 仲裁机制（leader lease / 分布式锁 / CRDT）** |
| **WF-1-55.42** | **DTL-026 §4.1 benchmark 子任务（per RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **撮合 benchmark 实测 n 上限 + DTL-026 §4.1 补 n≤500 占位 + 降级/熔断策略** |
| **WF-1-55.43** | **RGS-DEC-Q003 跨 DB Saga 审批包（per RGS-OPEN-QA-001 Q-M-01 + ACTIONS-v0.3 B-02）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **依赖 B-01（WF-1-55.43 应在 DTL-015/016 §3.4 步骤编号完成后 start）；6 场景决议 + RACI + DTL-031 §8.2 解除阻断** |
| **WF-1-55.44** | **4 域 rgs-testkit dev-dep + 集成测试骨架（per RGS-OPEN-QA-001 Q-M-02 + ACTIONS-v0.3 B-03）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **player/match/social/admin 4 域各补 rgs-testkit dev-dep + 1 份 tests/integration_*.rs（参考 economy 现有模板）；~4×0.5 人·天** |
| **WF-1-55.45** | **OTel 启用 + NATS traceparent + sqlx-tracing + 5 域 OTLP 出口（per RGS-OPEN-QA-001 Q-M-03 + ACTIONS-v0.3 B-04）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **核实 53.12/54.13 状态 + NATS header 注入 traceparent + sqlx-tracing 10-20% 采样 + 5 域各自直接 OTLP** |
| **WF-1-55.46** | **verify_probe_consistency.ps1 CI 脚本（per RGS-OPEN-QA-001 Q-M-04 + ACTIONS-v0.3 B-05）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **6 份 manifest probe 段结构化 diff + 阈值一致性全 6 份核对（不是抽查 2 份）+ CI 接入** |
| **WF-1-55.47** | **reservation IT + 混沌测试 + span 断言（per RGS-OPEN-QA-001 Q-M-07 + ACTIONS-v0.3 B-08）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **it_reservation_*.rs（create→conflict→release/cleanup）+ 混沌测试（DB 断开/死锁 P1，row 外部 DELETE P2）+ span 三层断言** |
| **WF-1-55.48** | **verify_fail_closed.ps1 + CI 接入 + RGS-TS-001 §5 状态改（per RGS-OPEN-QA-001 Q-M-08 + Q-M-10 + ACTIONS-v0.3 B-09）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **固化 fail-closed 验证 + 接入 CI（每次 manifest/RBAC 变更 PR 触发）+ TS-001 §5 改"已决策：NATS JetStream"** |
| **WF-1-55.49** | **新建 RGS-ADR-0055 DEC-005/008 兼容论证 + RACI 简表（per RGS-OPEN-QA-001 Q-G-01 + Q-G-02 + ACTIONS-v0.3 C-01）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **ADR-0055 v0.1 + RGS-PLAN-001 v1.0 §1.2 补 RACI 简表（4 类决策：代码合并/DTL 升版/生产发布/资金相关）** |
| **WF-1-55.50** | **WBS-001 §8 log 模板 + 11 份 log 逐份核验（per RGS-OPEN-QA-001 Q-G-04 + ACTIONS-v0.3 C-03）** | **Ulysses(per DEC-008)** | **pending** | **0%** | — | **B-CODE/C-CODE log 新模板（强制验证证据字段）+ 7 G-CODE + 4 B-CODE = 11 份 log 逐份核验报告** |
| _（其余 WF-0.5-4/5/6/7 + WF-1 旧 112 个 + 新 13 个启动后由 wbs_task_progress.ps1 填充）_ | | | | | | |

#### §2A.2.55.续3  P2 阶段 3 个新 L4 任务（v0.8 升版新增，per Ulysses 2026-08-26 09:27 JST "完成剩余工作到 P2"）

| L4 # | 任务摘要 | owner | status | progress | 启动时间 | 备注 |
|---|---|---|---|---|---|---|
| **WF-1-55.77** | **WBS-001 v0.7 → v0.8 升版（P2 3 L4 入表 + §2A.2.55.续3 段新增）** | **架构师（Mavis 接手 agent per DEC-008）** | **pending** | **0%** | — | **承接 P0/P1 8 commit 已落地的下一步：3 个 P2 L4 入表 + 修订历史 v0.8 行 + 变化说明 + 编号 55.77 起点（v0.7 末位 55.50 之后）；§3 汇总 142 → 145 L4 pending 137 → 140** |
| **WF-1-55.78** | **5 域 Lead RACI v1.0（player / economy / match / social / admin 5 份真实签字版）** | **架构师（Mavis 接手 agent per DEC-008）** | **pending** | **0%** | — | **每域 1 份 5-Lead 签字栏（player / economy / match / social / admin + 架构师 + SRE/DBA/安全）× 6 治理任务 = 30 单元；修订历史 v0.1 审批者 = 架构师（Mavis 接手 agent per DEC-008）；引用 RGS-ADR-0055 v0.1 4 治理角色 + RGS-RACI-001 v0.1 160 单元矩阵** |
| **WF-1-55.79** | **CROSS-008~012 二次校正（基于 5 域 Lead 反馈的修正）** | **架构师（Mavis 接手 agent per DEC-008）** | **pending** | **0%** | — | **5 份 CROSS SPEC 各开 §A.5 二次校正段（基于 WF-1-55.78 5 域 Lead 反馈的具体修订点）；不引入新设计，仅汇编 Lead 修正意见；修订历史 v0.X 审批者 = 架构师（Mavis 接手 agent per DEC-008）** |

## 5. 跨会话恢复 SOP（per RGS-WBS-001 v0.3 §13）

**场景**：agent 会话中断（断电 / 网络断开 / 进程 kill）后重启

**恢复步骤**：
1. 列出现有 worktree：`git worktree list`
2. 找未完成 worktree 的 `.wbs-task-marker`
3. 读 marker 知道当前 status / progress
4. 继续工作后调 `wbs_task_progress.ps1 -L4Id <id> -Status progress -Progress <N>` 更新
5. 完成时调 `wbs_task_progress.ps1 -L4Id <id> -Status done`

**marker 损坏或丢失的恢复**：
- 重新创建 marker：手工编辑 JSON 写最小字段
- 重新调 `wbs_task_progress.ps1 -Status start` 重建时间戳

## 6. 强制约束

- ❌ **L4 任务的 status 不是 pending** → 不允许在 L4 worktree 中 push / merge
- ❌ **status 是 blocked** 超过 7 天 → 升级为 RGS-PLAN-001 v0.8 §3.3 风险
- ❌ **done 任务未跑 3 脚本验证** → 不允许 wbs_merge.ps1 合并（脚本强制）

## 7. 关联文档

- 父文档：RGS-WBS-001 v0.3
- worktree 规范：RGS-WT-001 v0.2 §11（WBS L4 任务模式）
- 脚本：`scripts/wbs_list.ps1` / `wbs_create_worktree.ps1` / `wbs_task_progress.ps1` / `wbs_merge.ps1`
- DAG 依赖：RGS-WBS-001_DAG_v0.3.md
- 治理：RGS-PLAN-001 v0.8 §3.4.4 + RGS-QA-001 v0.13
- **11 份 log 核验**：[`docs/deploy/code-logs-verification-report.md`](..\..\deploy\code-logs-verification-report.md)（per WF-1-55.50 + ACTIONS-v0.3 C-03，v0.7 加）

---

## 8. WBS 状态维护 SOP（per phase-0-5 反馈单 Issue 5，v0.5 加）

> **背景**：v0.3 → v0.4 期间，本表与实际状态脱节约 1 周（per 反馈单 Issue 5 现象）。根因：① 手工编辑 v0.X 进度表直接写 "done 100%" 而不跑 `wbs_task_progress.ps1` ② 攒到「后续 Phase」再补 ③ 「已合并进 main」与「任务实质完成」混为一谈（per handoff §11.3 仍写「CR-1/2/3 修复仅 mock 验证」是反模式样本）。

### 8.1 正确流程（preferred）

走 `scripts/wbs_merge.ps1 -L4Id <ID>`，脚本内建 [1/3] 步骤**自动**跑：

```bash
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id <ID> -Status done
```

不要绕过本步骤（per 反馈单 Issue 5 末段「wbs_merge.ps1 [1/3] 步骤里已经内建校验脚本的意义所在——不要绕过它」）。

### 8.2 手工 merge 时补救（escape hatch）

仅在 `wbs_merge.ps1` 不可用时（例如手工 `git merge --no-ff` 后）：

```bash
# 1. 确认 marker 存在
ls -la <worktree>/.wbs-task-marker
# 2. 跑脚本
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id <ID> -Status done
# 3. 验证 v0.X 进度表自动更新
git diff docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.X.md
```

注意：脚本依赖 `.wbs-task-marker` 必须在 worktree 内能找到；如 marker 缺失（per 反馈单 Issue 3），先补建。

### 8.3 反模式（明令禁止）

- ❌ **手工编辑 v0.X 进度表直接写 "done 100%" 而不跑脚本**——这是 v0.3 → v0.4 期间发生的事，导致表/实脱节 1 周
- ❌ **攒到「后续 Phase」再补**——v0.3 → v0.4 累积 1 周无人维护就是反模式样本
- ❌ **「已合并进 main」但「任务实质未完成」的状态**——per 反馈单 Issue 4 描述：handoff §11.3 写「CR-1/2/3 修复仅 mock 验证」时，CR-1 实际已在 `wbs/WF-1-55.27-retry` 分支**已合并**到 main，但**实质未修**——合并 ≠ 任务完成
- ❌ **`status: done` 但** progress < 100%**——v0.5 加：`status: done` 必须 progress = 100% 同步，**否则视为状态机非法**

### 8.4 升级路径

- **脚本失败**：`wbs_task_progress.ps1` throw 时，先看 handoff §11「已知未完成事项」+ RGS-TS-001 §7「工具链 Bug 登记」是否有同 issue 已登记
- **marker 缺失**：走 RGS-WT-001 §11.3 跨会话恢复 SOP，**不**直接改 v0.X 进度表
- **进度表与 .wbs-task-marker 冲突**：以 **marker 为准**（marker 是 L4 任务状态的"原始数据源"，进度表是"视图"），重跑 §8.2 步骤 2 让脚本同步

### 8.5 B-CODE / C-CODE log 强制验证证据模板（v0.7 加，per OPEN-QA-001 Q-G-04）

> **背景**：per RGS-OPEN-QA-001 v0.2 Q-G-04 答复 + ACTIONS-v0.3 C-03：`现有 11 份 log 需逐份核验，不能一刀切判定 done；一刀切正是本疑问要防止的反模式本身`。**"已合并进 main"与"任务实质完成"混为一谈**是 phase-0-5 反馈单 Issue 4 描述的 anti-pattern。本节定义所有 G-CODE / B-CODE / C-CODE log 必须在文件**头部**含以下 5 个验证证据字段，缺一不可：

```markdown
# <CODE-ID> <CODE 名称> log

| 字段 | 内容 |
|---|---|
| **CODE 编号** | G-CODE-XX / B-CODE-XX / C-CODE-XX |
| **实测日期** | YYYY-MM-DDTHH:MM:SS±HH:MM |
| **实测责任人** | Ulysses（per DEC-008 一人公司兼任）/ AI worker 子代理 |
| **commit hash** | <git commit hash>（必填，per §6 合并 ≠ 任务完成）|
| **CI run 链接** | <CI run URL>（如适用；本地图测则填 `本地实测 + 测试输出文件名`）|
| **测试输出摘要** | <关键 pass/fail 数 + 关键 log 行引用，≥ 5 行> |

## 1. 实测目的
[明确该 CODE 的判定标准]

## 2. 前置条件
[列出所有依赖 + 当前状态]

## 3. 实测步骤
[每步可重复执行]

## 4. 实测结果
[关键输出 + commit hash 引用 + CI 链接]

## 5. 结论
[✅ done / ⚠️ partial / ❌ blocked，附理由]
```

**反模式（明令禁止）**：
- ❌ **log 只有 "已完成" 文字描述** —— 无 commit hash / 无测试输出 = 视为未完成
- ❌ **log 引用"已合并到 main"作为完成判据** —— 合并 ≠ 任务完成（per 反馈单 Issue 4）
- ❌ **log 把 4 份 B-CODE 当成 11 份** —— 实际是 7 G-CODE + 4 B-CODE = 11 份（per ACTIONS-v0.3 C-03 修正）

**现有 11 份 log 核验报告**：[`docs/deploy/code-logs-verification-report.md`](..\..\deploy\code-logs-verification-report.md)（per WF-1-55.50 + ACTIONS-v0.3 C-03）
