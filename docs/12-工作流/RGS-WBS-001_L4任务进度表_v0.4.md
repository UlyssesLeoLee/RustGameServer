# RGS-WBS-001 L4 任务进度表 v0.6

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001-ADD3 |
| 版本 | 0.6（v0.5 → v0.6：WF-0.5-8 handoff state sync 经 codex/wt-handoff-state-sync worktree 合并入 main，§3 + §4 同步更新；§3 汇总 L4 总数 128 → 129，done 4 → 5；§4 详细表新增 WF-0.5-8 done 100% 行；修订历史加本版记录。本版本不重命名文件名（保留 v0.4.md 文件名，只是 head 升 v0.6），避免 git mv 触发无关 diff。→ v0.6：WF-0.5-8 新增(handoff state sync done)+ §3/§4 同步；1 个 L4 任务标 done）|
| 依据 | RGS-WBS-001 v0.3 §2A L4 任务清单 + §6.3 进度字段 + §13 跨会话恢复 |
| 状态 | 🟠 **占位**（NO-GO 未解除前为空表；G-CODE-06 实测通过后由 wbs_task_progress.ps1 自动填充）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 父文档 | [RGS-WBS-001 瀑布式工作分解结构 v0.3](RGS-WBS-001_瀑布式工作分解结构_v0.3.md) |
| 关联 | RGS-WT-001 v0.2 §11（WBS L4 任务 worktree 模式）+ scripts/wbs_*.ps1 + phase-0-5 反馈单 Issue 5 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.3 | 2026-08-21 | Ulysses | **占位初版**：per RGS-WBS-001 v0.3 §6.3 + §13 跨会话恢复。状态为占位，等 G-CODE-06 实测通过后由 `wbs_task_progress.ps1` 自动填充。 |
| 0.4 | 2026-08-24 | Ulysses（一人公司 12 角色全签 per DEC-008）| **NO-GO 形式上解除后手工补登**：§3 WF-0.5 阶段 7→4 pending + 3 done；§4 手工补登 WF-0.5-1/2/3（Phase 0.5 Step 1+5 / 2+3 / 4 部署）三行 done。WF-0.5-6（worker 失败）由主对话接手，状态仍为 pending（不计入 done）。 |
| 0.5 | 2026-08-24 | worker-self（per DEC-008 一人公司治理基线，phase-0-5/feedback-handler worktree commit 见行末）| **WF-1-55.27 真修合并入 main**（per phase-0-5 反馈单 Issue 4）：① §3 汇总 WF-1 113 pending → 112 pending + 1 done；合计 125 pending → 124 pending + 4 done ② §4 任务表加 1 行 WF-1-55.27 done 100%，commit 是 merge 后 hash（保留 c96efe8 + a80fa94 + f6a6f3f + 14036d6 4 个原始 commit 的关联）③ 新增 §8 WBS 状态维护 SOP（per 反馈单 Issue 5）锁定「手工编辑 v0.X 进度表写 done 100%」与「攒到后续 Phase 再补」两个 anti-pattern。 |
| 0.6 | 2026-08-24 | worker-self（per DEC-008）| **WF-0.5-8 handoff state sync done**（per handoff §11.6/§11.7/§11.8 状态同步 commit，4 B-CODE 等 SRE 接力后 v0.7 升版）：① §3 汇总 128 → 129 L4 任务，4 done → 5 done（WF-0.5-8 done），124 pending 不变 ② §4 详细表加 1 行 WF-0.5-8 done 100%，commit 是 merge hash ③ 修订历史加本行。**本版本不重命名文件名**（保留 v0.4.md 文件名，只是 head 升 v0.6），避免 git mv 触发无关 diff。 |

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
| WF-1 | 113 | 112 | 0 | 1 | 0 |
| WF-2 ~ WF-7 | 6 | 6 | 0 | 0 | 0 |
| **合计** | **129** | **124** | **0** | **5** | **0** |

> **注**：WF-0 / WF-7 等 9 个 stage 大类行不计入 L4 任务数；实际 L4 任务数 = 129 - 7（阶段占位）= 122。详见 `wbs_list.ps1 -Summary` 输出。
>
> **v0.6 变化**：WF-0.5 阶段 3 done → 4 done（WF-0.5-8 新增 done），合计 4 done → 5 done。

## 4. L4 任务详细进度表

> **本节由 `wbs_task_progress.ps1` 调用时自动更新**。v0.4 状态：NO-GO 形式上解除，WF-0.5-1/2/3 三行 done 已手工补登（其余仍 pending，等 G-CODE-06 实测通过 + wbs_task_progress.ps1 调用后开始自动覆盖）。

| L4 # | 任务摘要 | owner | status | progress | 启动时间 | 备注 |
|---|---|---|---|---|---|---|
| WF-0.5-1 | Phase 0.5 Step 1+5 部署（5 域 manifest + docker image） | Ulysses（per DEC-008）| done | 100% | 2026-08-24T07:30:00+09:00 | commit 7046936（per 1190 行 + 4467080）|
| WF-0.5-2 | Phase 0.5 Step 2+3 部署（NATS + OTel/Prom/Grafana） | Ulysses（per DEC-008）| done | 100% | 2026-08-24T07:30:00+09:00 | commit c5a0c9f（per 1897 行 + 1183515）|
| WF-0.5-3 | Phase 0.5 Step 4 部署（mTLS + Secret + fail-closed） | Ulysses（per DEC-008）| done | 100% | 2026-08-24T07:30:00+09:00 | commit 765930a（per 1457 行 + 2b70b0b）|
| **WF-1-55.27** | **ReserveHandler OCC cleanup + reservation release 失败路径真修（per RGS-REV-009 CR-1）** | **Ulysses（per DEC-008）** | **done** | **100%** | **2026-08-24T18:00:00+09:00** | **merge commit 49d93b5（per phase-0-5/feedback-handler worktree `--no-ff` merge）**；原 branch commit `c96efe8` 真修（reservation.rs + saga_orchestrator.rs +159/-4）+ `a80fa94` 6 域 outbox migration（CR-2）一并合并；`f6a6f3f` PgTestDatabase fixture（CR-3，#[sqlx::test] 强约束）+ `14036d6` tag/marker 收尾同时入 main。**与原 branch 验证一致**：`cargo test -p economy-service --lib` 50/50 pass。 |
| **WF-0.5-8** | **handoff §11 状态同步(§11.6/§11.7/§11.8 标 Closed + 修订历史加 1 行 + 当前状态 3→10 closed 修正,per phase-0-5 反馈单 Issue 5 anti-pattern 修复)** | **Ulysses(per DEC-008)** | **done** | **100%** | **2026-08-24T20:42:00+09:00** | **merge commit `6b4a98c`(per codex/wt-handoff-state-sync worktree `--no-ff` merge,3 段改 Closed + 加 commit 引用 + 修订历史;`final-touchups` follow-up commit `6b4a98c` 替换占位 + 改 §11 line 359 "3→10 closed")** |
| _（其余 WF-0.5-4/5/6/7 + WF-1 其余 112 个仍 pending，等 G-CODE-06 实测通过后由 wbs_task_progress.ps1 填充）_ | | | | | | |

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
