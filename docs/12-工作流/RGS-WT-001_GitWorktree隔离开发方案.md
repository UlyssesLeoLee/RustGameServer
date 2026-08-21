# Git Worktree 隔离开发方案（RGS-WT-001）

**RustGameServer — 面向 5 域 Atomic App、ClusterOpsService 与可热插拔 Plugin 的本地并行开发隔离约定**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WT-001 |
| 版本 | 0.1 |
| 状态 | 工程约定 / 可执行；不授予业务编码、数据库迁移或部署授权 |
| 制定日 | 2026-08-21 |
| 适用范围 | RustGameServer 的 Git worktree、本地开发依赖、Compose、测试数据库与本地密钥注入 |
| 关联 | RGS-WF-001 v0.5、RGS-PLAN-001 v0.8、RGS-IMPL-001、RGS-OPS-001、RGS-QA-001 v0.10 |

> 本文只定义并行工作的物理隔离与回收方式。它不替代 `RGS-WF-001` §9 Gate，亦不将目前的实施 NO-GO 改为 GO。任何 worktree 在开始业务代码、迁移、集群部署前，仍须满足 `RGS-PLAN-001` 的 `G-CODE-*` 条件与具名审批。

---

## 1. 目标与边界

本项目包含 player、economy、match、social、admin 五个独立域，以及位于域上层的 ClusterOpsService（COC）。它们必须能够由不同 Lead 并行推进，但不得因本地目录、端口、数据库或容器资源复用而相互污染。Plugin 及其宿主的原子化解耦要求同样适用于开发环境：一个任务的未提交实现、运行时状态和本地机密不能成为另一个任务的隐式输入。

本方案的目标是：

1. 每项可并行工作拥有独立 Git `HEAD`、index、未提交改动和 `target/` 输出；
2. 每项会启动依赖的工作拥有唯一 Compose 项目、端口块、数据库命名空间以及本地状态卷；
3. worktree 的创建和回收只由受控脚本执行，避免误删主工作区、误复用分支或残留 Git 管理记录；
4. 共享的仅限于安全且只读/可再生成的资源：Git 对象库、Cargo registry/git cache、已提交的规范和依赖锁定文件。

不在本方案范围内的事项：生产、预生产或共享集成环境的访问；真实密钥分发；实际 Docker Compose、Helm、数据库迁移脚本的实现。这些仍由 `RGS-OPS-001`、`RGS-IMPL-001` 与对应 SPEC/DTL 约束。

## 2. 标准目录与分支模型

主工作区仅用于集成、评审、文档基线、创建/回收 worktree 和最终合并：

```text
D:\RustGameServer                         # primary worktree, branch main
D:\RustGameServer-worktrees\
  player-first-slice\                     # branch codex/wt-player-first-slice
  economy-saga\                           # branch codex/wt-economy-saga
  clusterops-contract\                    # branch codex/wt-clusterops-contract
```

受控脚本固定使用仓库外的兄弟目录 `D:\RustGameServer-worktrees`；禁止在 `D:\RustGameServer` 内建立嵌套 worktree。默认分支为 `codex/wt-<task-name>`，任务名仅允许 2–48 位小写字母、数字和连字符。脚本不复用、重置、强制覆盖或删除已有分支。

推荐 worktree 类型与边界如下：

| 类型 | 典型名称 | 允许的主要范围 | 不应并入同一 worktree 的范围 |
|---|---|---|---|
| Gate / 文档 | `docs-gate-review` | SPEC、DTL、QA、评审证据 | 业务实现和真实迁移 |
| 单域纵切片 | `player-first-slice` | 一个域 crate、对应 proto、迁移、测试 | 其他域的业务语义变更 |
| COC / 平台 | `clusterops-contract` | COC、共享契约、部署/CI 基础设施 | 任一域的业务实现 |
| 可观测性 | `observability-local` | OTel、日志、指标、本地观测依赖 | 业务事务策略 |
| 集成 | `integration-slice-01` | 已评审变更的组合验证 | 未评审的跨域设计探索 |

跨域事项（尤其 Saga、边界事件和 Plugin 契约）先在一个显式的 `integration-*` worktree 中完成 ADR/DTL/SPEC 对齐，再按域拆分。不得把“便于联调”当作跨域直接修改的理由。

## 3. 隔离矩阵

| 资源 | 隔离规则 | 执行方式 |
|---|---|---|
| Git 分支、HEAD、index、未提交改动 | 每项任务独立 | `git worktree add -b codex/wt-<task>` |
| Git 对象与历史 | 可以共享 | Git worktree 的正常共享；不得直接编辑 `.git` 管理文件 |
| Rust 构建输出 | 必须隔离 | 每个 worktree 保持自己的 `target/`；禁止把 `CARGO_TARGET_DIR` 指向共享目录 |
| Cargo registry / git cache | 可以共享 | 使用 Cargo 默认全局缓存；缓存被视为可再生成、不可携带业务状态 |
| Compose 项目、容器、卷、网络 | 必须隔离 | 所有 Compose 调用使用 `COMPOSE_PROJECT_NAME=rgs_<task>` |
| 本地端口 | 必须隔离 | 每项任务分配 1–99 的端口块；`RGS_PORT_OFFSET = PortBlock × 100` |
| PostgreSQL 与其他状态服务 | 必须隔离 | 数据库名/Schema、Redis key 前缀、消息 topic 前缀均以 `RGS_DATABASE_NAMESPACE=rgs_<task>` 派生 |
| `.worktree.env` | 每项任务独立，且不入库 | 脚本生成非机密 namespace 值；已由 `.gitignore` 忽略 |
| 密钥、令牌、连接字符串 | 绝不共享、绝不入库 | 各 task 从个人/本地 secret store 注入；禁止生产与预生产凭据 |
| Plugin 二进制及宿主运行状态 | 必须隔离 | 不共享 plugin build 目录、socket、watcher、容器卷或临时数据库 |

端口块只提供**命名空间**，不是端口映射本身。后续 Compose/Actix 配置必须以 `RGS_PORT_OFFSET` 计算对外端口，并将最终映射写入对应 SPEC/OPS 文档。例如基准端口 `8100` 在端口块 2 中应使用 `8300`。不得在 Compose 文件中硬编码所有 worktree 都会争用的宿主端口。

## 4. 受控工具

仓库中的 [`scripts/worktree.ps1`](../../scripts/worktree.ps1) 是唯一允许的日常创建与回收入口。它执行以下保护：

- 仅可从 primary worktree 运行，且只操作受管兄弟目录；
- 创建前验证 base commit、目标目录、Git 管理记录和目标分支均不存在；
- 为新 worktree 创建 `codex/wt-<task>` 分支，使用 Git `--lock` 防止被清理；
- 在 Git 的 per-worktree config 中记录 `rgs.worktree.id`、`rgs.worktree.portBlock` 和 Compose 项目名；
- 生成不含机密的 `.worktree.env`；
- `remove` 仅处理路径、ID 都匹配的干净 worktree，且不用 `--force`，保留分支供审计或恢复；
- `doctor` 检查锁、身份、端口块和元数据冲突。

标准操作：

```powershell
# 创建文档/评审任务；自动选择未占用端口块
.\scripts\worktree.ps1 create -Name docs-gate-review

# 创建单域任务；显式占用端口块 2
.\scripts\worktree.ps1 create -Name player-first-slice -Base main -PortBlock 2

# 查看受管任务与其分支/端口块
.\scripts\worktree.ps1 list

# 开发前与交接前执行；失败即不得继续使用该 task 环境
.\scripts\worktree.ps1 doctor

# 仅在状态干净、PR/交接已完成后回收；分支仍会保留
.\scripts\worktree.ps1 remove -Name docs-gate-review

# 仅检查，不改写 Git 管理数据
git worktree prune --dry-run
```

禁止事项：

- 不得在资源管理器中删除 worktree 目录；
- 不得对 worktree 执行 `git worktree remove --force`、`git clean -fdx` 或在未经确认的路径运行递归删除；
- 不得手工编辑 `.git/worktrees/*`、复用其他任务的 `.worktree.env`、数据库、卷或端口；
- 不得通过一个 worktree 访问/修改另一个 worktree 的未提交源文件或 `target/`；
- 不得将本地开发便利性解释为绕过 `G-CODE-*`、测试、Review、发布或变更管理的授权。

## 5. 本地依赖契约

未来的 Compose、测试 fixture、Actix 启动配置和数据库初始化必须读取以下变量；未读取即视为未完成本方案的接入：

| 变量 | 示例 | 用途 |
|---|---|---|
| `RGS_WORKTREE_ID` | `player-first-slice` | 日志、容器标签、临时目录与测试报告标识 |
| `RGS_PORT_BLOCK` | `2` | 人可读的端口块编号 |
| `RGS_PORT_OFFSET` | `200` | 宿主端口映射偏移量 |
| `COMPOSE_PROJECT_NAME` | `rgs_player-first-slice` | Docker Compose 的容器、网络、卷隔离 |
| `RGS_DATABASE_NAMESPACE` | `rgs_player-first-slice` | PostgreSQL 数据库/Schema、Redis 与事件资源前缀 |

对于 PostgreSQL 18.4，每一个域仍遵守自身数据库边界；`RGS_DATABASE_NAMESPACE` 只是在本地运行时对数据库实例/名称再分区，不能被解释成跨域共享数据库。对 Saga + Outbox 的测试，产生者、消费者和 outbox 表必须均处在同一 task namespace 中，避免测试读取其他 worktree 的事件残留。

对于 Actix Web 与 Plugin 宿主，监听地址和 socket/IPC 临时路径也必须由 `RGS_WORKTREE_ID` 和 `RGS_PORT_OFFSET` 派生。禁止固定使用 `localhost:8080`、固定 Unix socket 名称或全局临时文件名。

## 6. 生命周期、交接与回收

1. 在 primary worktree 确认当前 `main`、SPEC/DTL 版本和任务边界；先完成应有 Gate，不以 worktree 创建替代批准。
2. 使用脚本创建 task；进入新目录后先运行 `doctor`，再根据任务范围实现或起草。
3. 每次启动本地依赖时，加载当前 task 的 `.worktree.env`，并确认容器项目、端口和数据库前缀与 task 一致。
4. 交接时记录：task 名、分支、base commit、port block、数据库/Compose namespace、运行中的依赖、未关闭风险、验证命令及结果。不得记录密钥。
5. 合并或停止后，确保 worktree 干净，运行 `doctor`，再用 `remove` 回收。保留 `codex/wt-<task>` 分支直到 PR、评审证据与变更管理允许删除。
6. 发现目录已被人为删除、Git 报告 `prunable` 或元数据不一致时，先运行 `git worktree prune --dry-run` 并保留输出；由仓库管理员确认后再处理。不得直接执行强制清理。

## 7. 进入实施前的验收

本方案在以下条件全部满足时，视为“并行开发环境隔离已就绪”；它仍只是 53 開発環境構築 的一项环境证据：

- [ ] 所有活跃 task 均位于受管兄弟目录，且 `git worktree list --porcelain` 可见；
- [ ] `scripts/worktree.ps1 doctor` 成功，所有活跃 worktree 均 locked，并拥有唯一 ID/port block；
- [ ] 每个会启动状态依赖的 task 都通过 `.worktree.env` 派生 Compose、端口、数据库/缓存/事件命名空间；
- [ ] `target/`、Plugin 运行时目录、数据库卷、日志与临时 socket 未跨 task 共享；
- [ ] 本地密钥不在 Git、`.worktree.env`、日志、handoff 或测试 fixture 中；
- [ ] 该 task 的 SPEC、DTL、QA、Review 和 `G-CODE-*` 证据满足其实现入口条件。

## 修订历史

| 版本 | 日期 | 内容 |
|---|---|---|
| 0.1 | 2026-08-21 | 初版：定义仓库外受管 worktree、分支、锁、端口块、Compose/数据库 namespace、密钥边界与安全回收脚本。 |
| 0.2 | 2026-08-21 | **WBS L4 任务 worktree 模式**（per RGS-WBS-001 v0.3 §6.1 + §6.3 + §13）：① 新增 `wbs/<L4-ID>` 分支命名规则（替代通用 `codex/wt-<task>`）② 新增 4 个 WBS 脚本（wbs_list / wbs_create_worktree / wbs_task_progress / wbs_merge）③ 新增 `.wbs-task-marker` 跨会话恢复机制 ④ 显式 PowerShell 7.0+ 依赖声明 ⑤ 5 域 DTL 边界 + 跨域 DTL-021~025 / shared-platform DTL-032~040 的 worktree 分配规则。 |

---

## 11. WBS L4 任务 worktree 模式（v0.2 新增，per RGS-WBS-001 v0.3 §6.1 + §6.3 + §13）

### 11.1 分支命名

L4 任务 worktree 的分支命名规则（**替代 §2 通用 `codex/wt-<task>` 规则**）：

```
分支名：wbs/<L4-ID>
示例：wbs/WF-1-54.1、wbs/WF-1-54.2、wbs/WF-0.5-1、wbs/WF-0.5-7
```

**L4-ID 格式**（per RGS-WBS-001 v0.3 §6.1）：
- `WF-<L1>-<L3>-<L4>`（实施阶段，如 `WF-1-54.1`）
- `WF-<L1>.<L2>-<L4>`（跨阶段，如 `WF-0.5-1`）
- L1 = 阶段（0 / 0.5 / 1 / 2 / 3 / 4 / 5 / 6 / 7）
- L3 = 工程号（53 / 54 / 55 / 56 / 57 / 58），L2 = 子阶段（仅 0.5 用）
- L4 = 任务序号（1-15）

**worktree 目录名**（L4 ID 的 `.` → `-`）：

```
D:\RustGameServer-worktrees\
  WF-1-54-1\                    # 分支 wbs/WF-1-54.1
  WF-1-54-2\                    # 分支 wbs/WF-1-54.2
  WF-0-5-1\                     # 分支 wbs/WF-0.5-1
  WF-0-5-7\                     # 分支 wbs/WF-0.5-7
```

### 11.2 4 个 WBS 专用脚本（per RGS-WBS-001 v0.3 §6.4）

| 脚本 | 作用 | 命令 |
|---|---|---|
| `scripts/wbs_list.ps1` | 列出 WBS L4 任务 + 按 stage/domain/status 过滤 | `pwsh -File scripts/wbs_list.ps1 [-Stage WF-1] [-Domain player] [-Status pending] [-Summary]` |
| `scripts/wbs_create_worktree.ps1` | 为 L4 任务创建 worktree + 写 `.wbs-task-marker` | `pwsh -File scripts/wbs_create_worktree.ps1 -L4Id WF-1-54.1` |
| `scripts/wbs_task_progress.ps1` | 进度追踪（start / progress / done / blocked）| `pwsh -File scripts/wbs_task_progress.ps1 -L4Id WF-1-54.1 -Status progress -Progress 50` |
| `scripts/wbs_merge.ps1` | 跑 3 脚本验证 + 合并回 main + 清理 worktree | `pwsh -File scripts/wbs_merge.ps1 -L4Id WF-1-54.1` |

**与通用 `scripts/worktree.ps1` 的关系**：
- 通用 `worktree.ps1`：自由 task 名 + `codex/wt-<task>` 分支 + 端口块管理
- WBS 专用 `wbs_*.ps1`：固定 L4 ID → `wbs/<L4-ID>` 分支 + 自带 `.wbs-task-marker` + 跨会话恢复
- 两者并行存在，**WBS L4 任务优先用 `wbs_*.ps1`**，非 WBS 自由探索任务仍可用 `worktree.ps1`

### 11.3 `.wbs-task-marker` 跨会话恢复机制（per §13）

每个 WBS L4 worktree 根目录写一个 `.wbs-task-marker` JSON 文件（含 7 字段）：

```json
{
  "l4_id": "WF-1-54.1",
  "task": "5 域 Cargo crate 骨架（7 个）",
  "owner": "5 域",
  "tokens": "400K",
  "spec": "RGS-SPEC-000 §2.1 + RGS-IMPL-001 §2",
  "dtl": "DTL-018/015/016/026/019/020/031 §2",
  "branch": "wbs/WF-1-54.1",
  "status": "in_progress",
  "progress": 50,
  "started_at": "2026-08-21T10:30:00+09:00",
  "updated_at": "2026-08-21T14:15:00+09:00",
  "worktree": "D:\\RustGameServer-worktrees\\WF-1-54-1"
}
```

**跨会话恢复**：
- 重新打开 worktree 时，agent 先读 `.wbs-task-marker` 知道当前 status / progress
- 修改后调 `wbs_task_progress.ps1` 更新 marker
- 同时维护 `.wbs-task-log.txt` 追加历史记录
- **RGS-WBS-001_L4任务进度表_v0.3.md** 汇总所有 marker 状态（agent / 人类 review 用）

### 11.4 5 域 DTL 边界 + 跨域/平台 DTL 分配规则（per RGS-WBS-001 v0.3 §2A.6.1 + §2A.7）

WBS L4 任务按 owner 分配 worktree：

| 域 / 类型 | owner | L4 任务示例 | worktree 命名 |
|---|---|---|---|
| 5 业务域 DTL §1-§3（player / economy / match / social / admin）| 5 域 Lead（DEC-008 = Ulysses）| WF-0.5-1 / WF-0.5-2 / WF-0.5-3 | wbs/WF-0.5-1 等 |
| 跨域 DTL-021~025 | **Platform 域 Lead**（Ulysses）| WF-0.5-4 | wbs/WF-0.5-4 |
| shared-platform DTL-032~040 | **Platform 域 Lead**（Ulysses）| WF-0.5-5 | wbs/WF-0.5-5 |
| 7 份 RGS-SPEC-CROSS-001~007 | 各主题 owner（Platform 6/7 + cluster-ops 1/7）| WF-0.5-6 | wbs/WF-0.5-6 |
| 5 域 DTL §1-§3 联检 | 架构师（Ulysses）| WF-0.5-7 | wbs/WF-0.5-7 |
| 54 编码实现（5 域 + Platform）| 各自域 Lead | WF-1-54.1 ~ WF-1-54.15 | wbs/WF-1-54.X |
| 53 開発環境構築 | Platform | WF-1-53.1 ~ WF-1-53.15 | wbs/WF-1-53.X |
| 55-58 静态分析/CR/构建/CI | Platform | WF-1-55.X ~ WF-1-58.X | wbs/WF-1-55.X 等 |

**冲突检测规则**：
- 同一 owner 在同一时间只能 worktree 1 个 L4 任务（避免 1 人多 worktree 状态混淆）
- 跨域 DTL-021~025 的 L4 任务必须等 5 域 DTL §1-§3 联检（WF-0.5-7）通过后才能 start
- shared-platform DTL-032~040 的 L4 任务必须等 CROSS-001~007（WF-0.5-6）v0.2 填完

### 11.5 PowerShell 7.0+ 依赖声明

WBS 脚本（`wbs_*.ps1`）**必须**用 PowerShell 7.0+ 跑（中文路径支持）：

```bash
# 检测 PS 7
where.exe pwsh
# 期望: C:\Program Files\PowerShell\7\pwsh.exe

# 跑 WBS 脚本
pwsh -NoProfile -File scripts/wbs_list.ps1 -Summary
pwsh -NoProfile -File scripts/wbs_create_worktree.ps1 -L4Id WF-1-54.1
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id WF-1-54.1 -Status start
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id WF-1-54.1 -Status done
pwsh -NoProfile -File scripts/wbs_merge.ps1 -L4Id WF-1-54.1
```

**不**兼容 PowerShell 5.1（Windows 默认）— 因为 §2 通用 `worktree.ps1` 用 ANSI 系统编码解析中文路径，PS 5.1 解析中文目录名失败（GBK vs UTF-8 冲突）。

如果只有 PS 5.1：用 `chcp 65001` + 重启 PS，但建议升级 PS 7（Ulysses 环境已装 PS 7.6.3）。
