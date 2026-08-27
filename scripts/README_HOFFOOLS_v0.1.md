# RGS Handoff Tools v0.1（HOFFOOLS）

> **文档编号**：RGS-TOOL-HOFFOOLS-001
> **版本**：v0.1
> **任务**：M4 = RGS-PLAN-002 §1.1「主对话退场后 接收-恢复 工具链」
> **L4 ID**：`WF-1-D-15`
> **作者**：Ulysses（一人公司 12 角色 per DEC-008）— 子代理 4 号（M4）
> **日期**：2026-08-27（v0.1 初版）
> **依据**：RGS-PLAN-002 v0.1 §1.1 + Issue #11 + WBS-001 v0.11 §5 跨会话恢复 SOP

---

## 1. 目的

主对话（Ulysses 直连的 root session）退场后（断电 / 网络断开 / 进程 kill / 主动 close session），
任何 Mavis 子代理都可以跑 `rgs_handoff_recover.ps1` + `rgs_handoff_snapshot.ps1` 重建上下文：
- 看见当前 WBS L4 进度汇总
- 看见所有 worktree 状态（含孤儿标记）
- 验证所有 `.wbs-task-marker` 可被现有 `wbs_*.ps1` 处理
- 必要时把状态 dump 成 JSON 离线留底

## 2. 工具集（3 脚本 + 1 README）

| 脚本 | 角色 | 行为 | 是否写回 |
|---|---|---|---|
| `rgs_handoff_recover.ps1` | 接收-恢复总入口 | `-Mode summary` / `worktree-list` / `wbs-verify` | **只读 + echo**（不动 .wbs-task-marker、不动 v0.4.md、不动 git） |
| `rgs_handoff_snapshot.ps1` | 快照主对话状态 | dump JSON 到 `-OutputPath` | **只写目标 JSON 文件**，不写仓库 |
| `README_HOFFOOLS_v0.1.md` | 本文档 | — | — |

## 3. 用法

### 3.1 `rgs_handoff_recover.ps1`

```bash
# 3 个 mode 互斥
pwsh -NoProfile -File scripts/rgs_handoff_recover.ps1 -Mode summary
pwsh -NoProfile -File scripts/rgs_handoff_recover.ps1 -Mode worktree-list
pwsh -NoProfile -File scripts/rgs_handoff_recover.ps1 -Mode wbs-verify
```

#### `-Mode summary`
- 跑 `scripts/wbs_list.ps1 -Summary`，stdout 透传
- 另外扫描所有 worktree 的 `.wbs-task-marker`，按 status 分桶汇总（done / in_progress / pending / blocked / no-marker）
- **不**写回 `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md`（v0.4.md 由 `wbs_task_progress.ps1` 调用触发维护）
- 退出码：0 = 成功

#### `-Mode worktree-list`
- 跑 `git worktree list --porcelain`
- 对每个 wt 检查是否含 `.wbs-task-marker`：
  - `[MAIN]` = 主 wt
  - `[OK]` = 在 `.worktrees/` 或 `<repo>-worktrees/` 下且含 marker
  - `[ORPHAN]` = 在 `.worktrees/` 或 `<repo>-worktrees/` 下但**没有** marker
  - `[EXTERNAL]` = 在其他位置（不在托管根下）
- **不**执行 `git worktree remove` / `prune`（仅 list）
- 退出码：0 = 无孤儿；2 = 有孤儿（手工清理请走 `git worktree remove <path>` per RGS-WT-001 v0.2 §11.4）

#### `-Mode wbs-verify`
- 对每个 `.wbs-task-marker` 的 L4 ID，模拟 `wbs_task_progress.ps1 -L4Id <id> -Status done -WorktreePath <wt>` 调用
- **只 echo 模拟命令，不实际写**
- 退出码：0 = 全部 OK；3 = 有损坏 marker / 缺失 l4_id

### 3.2 `rgs_handoff_snapshot.ps1`

```bash
pwsh -NoProfile -File scripts/rgs_handoff_snapshot.ps1 -OutputPath C:/Users/leo19/.minimax/handoff/2026-08-27T10-30.json
```

快照内容（per 缺标比错标安全，只快照**已 git 实证**的事实）：
1. `snapshot_meta.generated_at`（ISO 8601 JST）
2. `snapshot_meta.repo_root` + `main_worktree`
3. `main_head`：`git log -1 --oneline` 完整一行
4. `main_status`：`git status --short --branch` 全部行
5. `wbs_summary`：`wbs_list.ps1 -Summary` 全部 stdout（行数组）
6. `task_markers`：所有 `.wbs-task-marker` 的核心字段（worktree / l4_id / status / progress / 时间戳）

**不**快照：
- 任何 secrets / env / `.env` / `~/.kube/config`
- 任何推测性数据（per DTL-036 hotfix 教训：禁"per X 历史形态"等回溯叙事）

退出码：0 = 成功；1 = 失败（参数 / 路径 / git 错误等）

## 4. 与现有 `wbs_*.ps1` 的关系

| 现有脚本 | HOFFOOLS 如何使用 |
|---|---|
| `scripts/wbs_list.ps1` | `-Mode summary` 直接 invoke `-Summary`，stdout 透传 |
| `scripts/wbs_task_progress.ps1` | `-Mode wbs-verify` **只 echo** 模拟命令，不实际 invoke（避免写 marker） |
| `scripts/wbs_create_worktree.ps1` | 不调用（M4 不创建新 worktree） |
| `scripts/wbs_merge.ps1` | 不调用（M4 不合并分支） |

**重要边界**：HOFFOOLS 严格只读 + echo，**绝不**调任何会写 `.wbs-task-marker` 或 git 状态的命令。
这是「缺标比错标安全」原则的执行：恢复工具的失败不应污染主对话状态。

## 5. 退出码速查

| 退出码 | 含义 | 触发条件 |
|---|---|---|
| 0 | 全部成功 | — |
| 1 | 内部错误 | 参数 / 路径 / git / pwsh 异常 |
| 2 | 有孤儿 worktree | `-Mode worktree-list` 找到 `[ORPHAN]` 标记 |
| 3 | 有损坏 marker | `-Mode wbs-verify` 找到 l4_id 缺失 / 解析失败 |

## 6. 常见错误

| 现象 | 原因 | 修复 |
|---|---|---|
| `wbs_list.ps1 -Summary` 退出码非 0 | PowerShell < 7.0 或脚本缺失 | 用 `pwsh` 调，确认 `scripts/wbs_list.ps1` 存在 |
| `git worktree list --porcelain` 失败 | 不在 git 仓库根 | `cd` 到 RGS 仓库根，或显式 `-RepoRoot` |
| 大量 `[ORPHAN]` 标记 | 历史 worktree 未清理（merge 后没 remove） | 手工 `git worktree remove <path>`（per RGS-WT-001 v0.2 §11.4） |
| 大量 `[FAIL] l4_id 缺失` | marker 损坏或手工改坏 | 重建 marker：手工编辑 JSON 写最小字段 → `wbs_task_progress.ps1 -Status start` |
| `Cannot find path` (snapshot) | `-OutputPath` 父目录不存在 | 工具会自动 `New-Item -ItemType Directory -Force`，如仍失败检查权限 |

## 7. 已知缺口（per DTL-036 hotfix 教训「缺标比错标安全」原则必须显式列）

1. **`wbs_task_progress.ps1` 对 array 格式 marker 支持不完整**：
   现状：Update 阶段 `$content | ConvertFrom-Json` 把 array 解析为 `PSCustomObject[]`，
   再 `$marker.status = ...` 会因 array 不支持 set 元素属性而失败。
   影响：merge 多 task 的 worktree 跑 `wbs_task_progress.ps1` 会报错。
   workaround（本任务期间采用）：临时把 marker 改为单条 entry 跑脚本，再合并回 array。
   待办（per 强约束 7：HOFFOOLS 不修 wbs_*.ps1，**留待 M 后续任务 / 单独 ticket 修**）。
2. **HOFFOOLS 不查 secrets / .env / `~/.kube/config`**（per 强约束 6）：snapshot 也不会包含这些路径的内容。
3. **HOFFOOLS 不调 `git worktree remove` / `prune`**：发现 `[ORPHAN]` 后**只报告**，不自动清理（避免误删未保存改动）。
4. **本 README 的「与 wbs_*.ps1 关系」节可能随 wbs_*.ps1 演化失同步**：v0.1 baseline 2026-08-27，后续升级请同步本节。

## 8. 关联文档

- 父文档：[RGS-PLAN-002 v0.1 §1.1]（主对话退场后 接收-恢复 工具链定义）
- Issue: #11（M4 子代理任务单）
- 父任务：[RGS-WBS-001 v0.11 §5 跨会话恢复 SOP](D:/RustGameServer/docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md)
- 治理：DEC-008（一人公司 12 角色）+ 2026-08-26 08:40 JST 代签新规则
- 关联风险：DTL-036 v1.4 hotfix 教训（禁"per X 历史形态"等回溯叙事）
