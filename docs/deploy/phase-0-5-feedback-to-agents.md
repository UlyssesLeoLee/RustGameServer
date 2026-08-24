# phase-0-5-feedback-to-agents.md

# 角色：给并发/后续 agent session 的问题反馈单，要求对方逐项修改并回填结果
# 生成：主对话（Sonnet 5）2026-08-24 19:xx，基于本 session 实测发现
# 使用方式：接手 agent 逐条核实 → 修改 → 在对应条目下方追加「已处理」段落说明 commit/依据，不要删除原问题描述

---

## 1. 并发 session 修改 main 未协调，导致我方在途工作被静默 stash

- **现象**：本 session 开始时 `git status` 显示 ~30 个文件 M（Dockerfile / 6 个 service main.rs / tls.rs / k8s manifests 等），几十分钟后再查 `git log` 发现 main 已推进 5 个新 commit（`66ff53b` `ad54801` `7c2db70` `65ea750` `6d985d6`），working tree 变回干净，我方之前的编辑消失于 `stash@{0}: On main: handoff-fix-in-progress-20260824-1827`。
- **问题**：没有任何提示/交接说明告知本 session「你正在看的文件已被别的 session 改过/stash 过」，导致核对成本极高，且存在两个 session 同时改同一批文件、互相覆盖的风险。
- **要求**：
  1. 多 session 并发操作同一仓库时，改动前先 `git fetch`/`git status` 确认无他人在途改动，或至少在 handoff 文档里登记「本 session 正在编辑：文件列表 + 时间戳」。
  2. 不要用 `git stash` 静默处理别人 session 的未提交改动——应保留在 working tree 或明确记录 stash 原因和恢复方式（本次 stash 内容尚属无害，但这是运气好，不是流程保证）。

## 2. 4 个并行 worker（Saga 修复 / DTL Review / 引用扫雷 / Retrospective）全部 0 产出

- **现象**：per `docs/deploy/phase-0-5-handoff.md` §11.3 记录：「原计划开 4 worker 并行...实际 4/4 session 在创建后立即被 mavis 框架标 error（"Canonical history target is missing after migration"），HEAD 全部 == main，0 产出」。
- **问题**：这是框架层面的失败（worker 派发机制本身出错），不是任务内容问题，但没有被登记为工具链 bug，也没有重试或降级处理，导致 WF-1-55.27/28/29（P0 merge-blocker）长期挂起。
- **要求**：
  1. 定位「Canonical history target is missing after migration」的根因（worktree/分支基准丢失？migration 步骤未跑？），登记到 `RGS-TS-001` 工具链 bug 清单。
  2. 在根因修复前，明确这条路径不可用，后续任务改走手工 `wbs_create_worktree.ps1` 或主对话直接处理，不要重复派发到会立即失败的机制上浪费 token。

## 3. 多个 worktree 缺 `.wbs-task-marker`，导致 `wbs_merge.ps1` 找不到任务

- **现象**：`git worktree list` 显示 4 个活跃 worktree（`WF-0-5-citation` `WF-0-5-retro` `WF-0-5-review` `WF-1-55-retry`），但逐一检查后 **全部没有** `.wbs-task-marker` 文件。`wbs_merge.ps1` 依赖该 marker 定位分支，marker 缺失时脚本会直接 `throw "L4 任务 marker 未找到"`，工具链形同虚设。
- **问题**：不清楚这些 worktree 是「从未走 `wbs_create_worktree.ps1` 创建」还是「marker 事后被删/漏提交」。无论哪种，当前状态下标准合并流程对这 4 个 worktree 全部失效。
- **要求**：
  1. 排查这 4 个 worktree 各自的创建方式，若是手工 `git worktree add` 绕开了脚本（参考 §11.4 描述的历史先例），需要**补建** `.wbs-task-marker`（schema 见 `wbs_create_worktree.ps1` 的 `$marker` 对象：`l4_id/task/owner/tokens/spec/dtl/branch/status/progress/started_at/updated_at/worktree`）。
  2. 若已合并完成的 worktree（如 `WF-0-5-citation/retro/review`，均已 merge 进 phase-0-5/local-fixes）不再需要，按 §11.6 记录的规范流程清理，不要重复用 `--force` 强删。
  3. 本 session 已为 `WF-1-55-retry` 手工补了 marker（`l4_id: WF-1-55.27`），待确认合并方式后可保留或按需调整。

## 4. `wbs/WF-1-55.27-retry`（commit `c96efe8`）是真实、已验证的修复，但从未合并、也未登记

- **现象**：分支包含真实修复（`crates/economy-service/src/{reservation.rs,saga_orchestrator.rs}`，+159/-4 行），针对 `ReserveHandler::execute` 第三条失败路径（`load_active_account` 失败但 reservation 已落盘，导致 compensate 时幽灵 `+amount` 入账）。本 session 复核：
  - `git merge-base main wbs/WF-1-55.27-retry` == 当前 main tip（`6d985d6`），可直接 fast-forward 式合并，无冲突。
  - `cargo test -p economy-service --lib` 在该分支上对着最新 main 跑：**50/50 通过**（含 2 个新增回归测试）。
- **问题**：`docs/deploy/phase-0-5-handoff.md` §11.3 仍写「CR-1/2/3 修复仅 mock 验证」——这条记录是**过时的**，没有反映这个已存在、已测试通过的真实修复。说明「登记 WBS 状态」和「实际分支里发生了什么」这两者之间存在信息断层，容易导致后续 agent 重复造轮子或错过已有成果。
- **要求**：
  1. 合并 `wbs/WF-1-55.27-retry` 到 main（本 session 已验证可直接合并，卡在 auto-mode classifier 对 `git merge`/`worktree remove --force` 类操作的许可拦截，需要人工确认后执行 `pwsh -File scripts/wbs_merge.ps1 -L4Id WF-1-55.27`）。
  2. 合并后更新 `RGS-WBS-001_L4任务进度表` 与 handoff §11.3，把 WF-1-55.27 标记为 done，并纠正「仅 mock 验证」的过时描述。
  3. **建议**：以后每次开新 worktree/分支前，先 `git branch -a` + `git log --all --oneline | grep <L4Id>` 扫一遍有没有孤立 commit 命中同一个任务，避免重复劳动或遗漏已完成工作。

## 5. `RGS-WBS-001_L4任务进度表` 长期与实际状态脱节

- **现象**：per handoff §11.2，进度表仍显示「128/128 pending」，但实际至少 WF-0.5-1/2/3、WF-1-55.27（若采纳本反馈单第 4 条）等多项已完成/已合并。
- **问题**：进度表是多 agent 协作的唯一状态源，长期不同步会导致后续 session 无法信任它，被迫每次都手工 `git log`/`git branch` 交叉核实（本 session 就是这样做的），效率很低。
- **要求**：每次任务真正 merge 进 main 后，**立即**跑 `pwsh -File scripts/wbs_task_progress.ps1 -L4Id <ID> -Status done`，不要攒到「后续 Phase」再补，这也是 wbs_merge.ps1 [1/3] 步骤里已经内建校验脚本的意义所在——不要绕过它。

---

## 处理登记区（接手 agent 填写）

| 条目 | 处理 agent | 处理时间 | commit/依据 | 状态 |
|---|---|---|---|---|
| 1 | | | | ⬜ |
| 2 | | | | ⬜ |
| 3 | | | | ⬜ |
| 4 | | | | ⬜ |
| 5 | | | | ⬜ |
