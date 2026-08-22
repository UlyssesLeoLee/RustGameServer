# RGS-WBS-001 L4 任务进度表 v0.3

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001-ADD3 |
| 版本 | 0.3（per RGS-WBS-001 v0.3 §6.3 + §13 跨会话恢复）|
| 依据 | RGS-WBS-001 v0.3 §2A L4 任务清单 + §6.3 进度字段 + §13 跨会话恢复 |
| 状态 | 🟠 **占位**（NO-GO 未解除前为空表；G-CODE-06 实测通过后由 wbs_task_progress.ps1 自动填充）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 父文档 | [RGS-WBS-001 瀑布式工作分解结构 v0.3](RGS-WBS-001_瀑布式工作分解结构_v0.3.md) |
| 关联 | RGS-WT-001 v0.2 §11（WBS L4 任务 worktree 模式）+ scripts/wbs_*.ps1 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.3 | 2026-08-21 | Ulysses | **占位初版**：per RGS-WBS-001 v0.3 §6.3 + §13 跨会话恢复。状态为占位，等 G-CODE-06 实测通过后由 `wbs_task_progress.ps1` 自动填充。 |

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

> **本节由 `wbs_list.ps1 -Summary` 自动生成**。当前 NO-GO 状态，所有任务为 `pending`。

| Stage | 任务数 | pending | in_progress | done | blocked |
|---|---|---|---|---|---|
| WF-0 | 2 | 2 | 0 | 0 | 0 |
| WF-0.5 | 7 | 7 | 0 | 0 | 0 |
| WF-1 | 113 | 113 | 0 | 0 | 0 |
| WF-2 ~ WF-7 | 6 | 6 | 0 | 0 | 0 |
| **合计** | **128** | **128** | **0** | **0** | **0** |

> **注**：WF-0 / WF-7 等 9 个 stage 大类行不计入 L4 任务数；实际 L4 任务数 = 128 - 7（阶段占位）= 121。详见 `wbs_list.ps1 -Summary` 输出。

## 4. L4 任务详细进度表

> **本节由 `wbs_task_progress.ps1` 调用时自动更新**。当前 NO-GO 状态，所有任务为 `pending`。G-CODE-06 实测通过后开始填充。

| L4 # | 任务摘要 | owner | status | progress | 启动时间 | 备注 |
|---|---|---|---|---|---|---|
| _（NO-GO 解除后由 wbs_task_progress.ps1 填充）_ | | | | | | |

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
