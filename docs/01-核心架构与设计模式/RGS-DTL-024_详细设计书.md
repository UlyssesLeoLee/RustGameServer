# 详细设计书（詳細設計書 / Detailed Design Document）

**App集群自动化部署脚本：编排状态表物理数据库设计・CLI/依赖图算法实现・回滚流程详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-024 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-024 App集群自动化部署脚本 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定。细化RGS-BAS-024§4编排状态表为具体PostgreSQL DDL、§3依赖图构建与校验规则为可直接翻译为Rust实现的伪代码（含DFS环检测/Kahn拓扑排序）、§4编排主循环伪代码为完整并发处理与错误路径、§6.2回滚流程为具体伪代码、§7强制联动检查脚本为具体校验逻辑、§8 CLI子命令为具体参数/退出码规范。**本版本不覆盖**：`cluster-manifest.yaml`的完整JSON Schema校验实现代码、与具体CI平台（如GitHub Actions/GitLab CI）绑定的触发API细节（RGS-BAS-024§10已声明"留待实施阶段的技术评审最终确定"）。见§8 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 状态表DDL的并发控制是否覆盖"编排进程崩溃后重启续跑"这一RGS-BAS-024§9既定场景，拓扑排序算法是否与Kahn算法标准实现一致 |
| 评审（运维） | | | 回滚流程的逆拓扑序实现是否真正保证"业务域App在前、基础设施层App在后" |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：编排状态表](#2-物理数据库设计编排状态表)
3. [依赖图构建与校验算法详细设计](#3-依赖图构建与校验算法详细设计)
4. [编排主循环详细设计](#4-编排主循环详细设计)
5. [幂等续跑详细设计](#5-幂等续跑详细设计)
6. [Dry-run与回滚详细设计](#6-dry-run与回滚详细设计)
7. [强制联动检查脚本设计](#7-强制联动检查脚本设计)
8. [CLI命令具体规范](#8-cli命令具体规范)
9. [本文档的覆盖范围与后续计划](#9-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-024给出了集群清单YAML Schema、依赖图算法的文字描述、编排状态机的状态表格与主循环伪代码骨架、幂等性设计要点、Dry-run/回滚的文字流程、CLI子命令清单。本文档将其落实为：编排状态表的可执行DDL、依赖图算法（环检测/拓扑排序）的完整伪代码、编排主循环的并发与错误处理细节、回滚流程的完整伪代码、CLI各子命令的参数与退出码规范，使实现人员可直接依此产出可编译代码。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-024已确定的任何结构性选择（编排层是既有ARC-018脚手架之上的薄编排、不引入新部署执行机制、状态表持久化于既有PostgreSQL、回滚走逆拓扑序）。若细化过程中发现基本设计本身有缺陷，修正应回写RGS-BAS-024，不在本文档内悄悄改写。
- 不覆盖`cluster-manifest.yaml`的完整JSON Schema校验实现代码——本文档§3给出校验**规则**的伪代码，具体Schema校验库选型与完整实现留待实施阶段。
- 不覆盖具体CI平台绑定细节——RGS-BAS-024§10已明确"具体CI平台绑定细节（如触发API、凭据范围）留待实施阶段的技术评审最终确定"，本文档同样不越权预先选定。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，算法伪代码可直接对应Rust `Result`实现，CLI规范以POSIX风格参数/退出码约定给出。

---

## 2. 物理数据库设计：编排状态表

对应RGS-BAS-024§4"每个`run_id`维护一张状态表，每个App一行"与§9"状态表持久化于既有PostgreSQL"。落实为具体DDL，复用既有生产PostgreSQL实例（不新建独立数据库，同RGS-BAS-024§9"不引入新HA机制"精神）：

```sql
-- 编排运行主表：一次deploy-cluster apply对应一行
CREATE TABLE deploy_runs (
    run_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manifest_version INTEGER NOT NULL,     -- 对应cluster-manifest.yaml的manifest_version字段
    cluster_id       TEXT NOT NULL,          -- 对应cluster-manifest.yaml的cluster_id字段
    env                TEXT NOT NULL,          -- dev/staging/prod
    status                TEXT NOT NULL DEFAULT 'RUNNING'
                             CHECK (status IN ('RUNNING', 'COMPLETED', 'FAILED', 'ROLLED_BACK')),
    started_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at                TIMESTAMPTZ,
    triggered_by                  TEXT NOT NULL   -- 操作者标识，供§9审计使用
);

-- App级状态行：对应§4状态表"每个App一行"
CREATE TABLE deploy_run_apps (
    run_id       UUID NOT NULL REFERENCES deploy_runs(run_id),
    app_id         TEXT NOT NULL,
    level            INTEGER NOT NULL,     -- 对应§3.3拓扑排序产出的执行层级(level_0/level_1/...)
    state               TEXT NOT NULL DEFAULT 'PENDING'
                           CHECK (state IN ('PENDING', 'RUNNING', 'SUCCEEDED', 'FAILED', 'BLOCKED', 'ROLLED_BACK')),
    target_version         TEXT NOT NULL,
    retry_count               INTEGER NOT NULL DEFAULT 0,
    version                     INTEGER NOT NULL DEFAULT 0,   -- OCC乐观锁，同RGS-DTL-001§3.2既定模式，
                                                                -- 防止编排进程崩溃重启续跑时与残留旧进程并发写同一行
    updated_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (run_id, app_id)
);
CREATE INDEX idx_deploy_run_apps_run_level ON deploy_run_apps (run_id, level);
    -- 支撑§4主循环"按层级遍历"的核心查询路径

-- 状态变更审计记录：对应§9"每条状态迁移记录写入既有审计留痕存储"，
-- 本表是该要求的物理落地，复用RGS-BAS-003§7既定审计设计的表结构范式(不新建独立审计机制)
CREATE TABLE deploy_run_state_changes (
    change_id     BIGSERIAL PRIMARY KEY,
    run_id          UUID NOT NULL REFERENCES deploy_runs(run_id),
    app_id            TEXT NOT NULL,
    old_state           TEXT,
    new_state             TEXT NOT NULL,
    triggered_by             TEXT NOT NULL,
    occurred_at                 TIMESTAMPTZ NOT NULL DEFAULT now()
    -- event_type固定为'cluster_deploy_state_change'(§9既定)，若与既有admin_db审计表合并存储，
    -- 该固定值作为筛选条件，本表结构本身不显式存储该常量列(避免冗余)
);
CREATE INDEX idx_deploy_run_state_changes_run ON deploy_run_state_changes (run_id, occurred_at);
```

---

## 3. 依赖图构建与校验算法详细设计

对应RGS-BAS-024§3.1构建描述与§3.2三条校验规则文字表述，落实为完整伪代码。

### 3.1 图构建

```rust
struct DependencyGraph {
    nodes: HashMap<AppId, AppManifestEntry>,
    edges: HashMap<AppId, Vec<AppId>>,   // app_id -> depends_on列表
}

fn build_graph(manifest: &ClusterManifest) -> DependencyGraph {
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();
    for app in &manifest.apps {
        nodes.insert(app.app_id.clone(), app.clone());
        edges.insert(app.app_id.clone(), app.depends_on.clone());
    }
    DependencyGraph { nodes, edges }
}
```

### 3.2 三条校验规则（对应§3.2）

```rust
fn validate_manifest(graph: &DependencyGraph, foundation_app_ids: &HashSet<AppId>) -> Result<(), Vec<ManifestViolation>> {
    let mut violations = vec![];

    // 规则1：孤儿引用校验(先做，因后续算法假定图内引用均存在，避免下游panic)
    for (app_id, deps) in &graph.edges {
        for dep in deps {
            if !graph.nodes.contains_key(dep) {
                violations.push(ManifestViolation::OrphanReference { app_id: app_id.clone(), missing_dep: dep.clone() });
            }
        }
    }
    if !violations.is_empty() { return Err(violations); }   // 孤儿引用存在时后续两条规则无法可靠执行，提前返回

    // 规则2：无环校验(DFS染色法：白=未访问，灰=访问中，黑=已完成；灰->灰边即为环)
    let mut color: HashMap<AppId, Color> = graph.nodes.keys().map(|id| (id.clone(), Color::White)).collect();
    let mut cycle_path = vec![];
    for start in graph.nodes.keys() {
        if color[start] == Color::White {
            if let Some(cycle) = dfs_detect_cycle(graph, start, &mut color, &mut cycle_path) {
                violations.push(ManifestViolation::CycleDetected { path: cycle });
            }
        }
    }
    if !violations.is_empty() { return Err(violations); }   // 有环时拓扑排序不可能成功，提前返回，不继续规则3

    // 规则3：基础设施前置校验——每个tier:domain的App，depends_on传递闭包必须包含全部tier:foundation的App
    for app in graph.nodes.values().filter(|a| a.tier == Tier::Domain) {
        let closure = transitive_closure(graph, &app.app_id);
        let missing: Vec<_> = foundation_app_ids.difference(&closure).cloned().collect();
        if !missing.is_empty() {
            violations.push(ManifestViolation::MissingFoundationDependency { app_id: app.app_id.clone(), missing });
        }
    }

    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

// 环检测：灰色节点被再次访问即为环，回溯cycle_path还原完整环路径(对应§3.2"报告完整环路径"要求)
fn dfs_detect_cycle(graph: &DependencyGraph, node: &AppId, color: &mut HashMap<AppId, Color>, path: &mut Vec<AppId>) -> Option<Vec<AppId>> {
    color.insert(node.clone(), Color::Gray);
    path.push(node.clone());
    for dep in &graph.edges[node] {
        match color[dep] {
            Color::Gray => {
                // 找到环：从path中dep首次出现的位置到末尾即为环路径(如 TRD -> EVT -> TRD)
                let start_idx = path.iter().position(|n| n == dep).unwrap();
                return Some(path[start_idx..].to_vec());
            }
            Color::White => {
                if let Some(cycle) = dfs_detect_cycle(graph, dep, color, path) {
                    return Some(cycle);
                }
            }
            Color::Black => {}   // 已完成子树，安全跳过
        }
    }
    path.pop();
    color.insert(node.clone(), Color::Black);
    None
}
```

### 3.3 Kahn算法拓扑排序（对应§3.3）

```rust
fn topological_levels(graph: &DependencyGraph) -> Vec<Vec<AppId>> {
    // 前置条件：调用方必须已通过§3.2规则2无环校验，本函数不重复检测环(职责分离，环检测已在validate_manifest完成)
    let mut in_degree: HashMap<AppId, usize> = graph.nodes.keys().map(|id| (id.clone(), 0)).collect();
    for deps in graph.edges.values() {
        for _dep in deps { /* depends_on方向：app依赖dep，故计入app的入度，而非dep的入度 */ }
    }
    for (app_id, deps) in &graph.edges {
        *in_degree.get_mut(app_id).unwrap() = deps.len();
    }
    // 反向邻接：dep -> 依赖它的app列表，用于dep完成后递减下游入度
    let mut dependents: HashMap<AppId, Vec<AppId>> = graph.nodes.keys().map(|id| (id.clone(), vec![])).collect();
    for (app_id, deps) in &graph.edges {
        for dep in deps { dependents.get_mut(dep).unwrap().push(app_id.clone()); }
    }

    let mut levels = vec![];
    let mut remaining: HashSet<AppId> = graph.nodes.keys().cloned().collect();
    while !remaining.is_empty() {
        let current_level: Vec<AppId> = remaining.iter()
            .filter(|id| in_degree[*id] == 0)
            .cloned()
            .collect();
        // current_level为空但remaining非空：意味着存在环，不应发生(前置条件已排除)，防御性panic而非静默死循环
        assert!(!current_level.is_empty(), "topological_levels called on cyclic graph, violates precondition");
        for id in &current_level {
            remaining.remove(id);
            for dependent in &dependents[id] {
                *in_degree.get_mut(dependent).unwrap() -= 1;
            }
        }
        levels.push(current_level);
    }
    levels
}
```

---

## 4. 编排主循环详细设计

对应RGS-BAS-024§4主循环伪代码骨架，落实为含错误处理与并发细节的完整版本。

```rust
async fn run_orchestration(run_id: RunId, levels: &[Vec<AppId>]) -> Result<(), OrchestrationError> {
    for (level_idx, level) in levels.iter().enumerate() {
        // 层级内并行：对应§3.3"同一层级内的App在执行阶段并行处理"
        let results = futures::future::join_all(
            level.iter().map(|app_id| deploy_single_app(run_id, app_id.clone()))
        ).await;

        let all_succeeded = results.iter().all(|r| r.is_ok());
        if !all_succeeded {
            // 跨层级严格串行等待：本层失败即暂停整体run，不进入下一层级(§4"失败即暂停，不继续下一层级")
            mark_run_status(run_id, RunStatus::Failed)?;
            return Err(OrchestrationError::LevelFailed { level: level_idx });
        }
    }
    mark_run_status(run_id, RunStatus::Completed)?;
    Ok(())
}

async fn deploy_single_app(run_id: RunId, app_id: AppId) -> Result<(), OrchestrationError> {
    let row = fetch_app_state(run_id, &app_id)?;
    if row.state == AppState::Succeeded {
        return Ok(());   // 续跑时跳过已成功App，对应§5幂等续跑设计
    }

    // OCC更新为RUNNING，防止同一App被并发的两个编排进程实例同时调度(如误触发两次apply)
    let updated = occ_update_state(run_id, &app_id, row.version, AppState::Running)?;
    if updated == 0 {
        return Err(OrchestrationError::ConcurrentOrchestrationDetected { app_id });
    }

    let result = invoke_scaffold(&row.scaffold_ref, &row.target_version).await;   // 调用既有Helm Release流程
    match result {
        Ok(_) => {
            occ_update_state(run_id, &app_id, row.version + 1, AppState::Succeeded)?;
            Ok(())
        }
        Err(e) => {
            let retry_count = row.retry_count + 1;
            if retry_count < MAX_RETRIES_FR_DEP_009 {
                // 重试次数未耗尽：状态迁移为FAILED后由续跑机制(§5)驱动RUNNING重试，本函数本身不做递归重试，
                // 避免单次调用栈内无限重试导致的超时/资源占用问题——重试是"下一次续跑"的职责而非本函数职责
                update_retry_count(run_id, &app_id, retry_count)?;
            }
            occ_update_state(run_id, &app_id, row.version + 1, AppState::Failed)?;
            Err(OrchestrationError::AppDeployFailed { app_id, source: e })
        }
    }
}
```

---

## 5. 幂等续跑详细设计

对应RGS-BAS-024§5三条幂等性要点，落实为续跑入口的具体逻辑：

```rust
async fn resume_run(run_id: RunId) -> Result<(), OrchestrationError> {
    let manifest = load_manifest_for_run(run_id)?;   // 复用该run创建时锁定的manifest_version，不重新解析当前HEAD清单，
                                                        // 避免续跑期间清单被并发修改导致的拓扑不一致(补充设计要点，非BAS-024原文明确但为幂等续跑的必要前提)
    let graph = build_graph(&manifest);
    let levels = topological_levels(&graph);

    // 续跑只对PENDING/FAILED/BLOCKED状态重新计算是否可执行(§5第二点)
    let app_states = fetch_all_app_states(run_id)?;
    for (app_id, state) in &app_states {
        if *state == AppState::Succeeded {
            continue;   // 跳过，不重复调用其Helm Release(§5"即使重复调用本身也是幂等的，跳过只是为了缩短续跑时长")
        }
    }
    run_orchestration(run_id, &levels).await   // 复用§4主循环，SUCCEEDED的App在deploy_single_app内部已处理跳过逻辑
}
```

健康检查（`RUNNING → SUCCEEDED`判定）复用各App既有readiness探针（RGS-BAS-002脚手架检查清单要求），`invoke_scaffold`内部轮询该探针直至就绪或超时，本文档不重新定义探针协议本身。

---

## 6. Dry-run与回滚详细设计

### 6.1 Dry-run（对应RGS-BAS-024§6.1）

```rust
fn dry_run(manifest: &ClusterManifest) -> Result<DryRunReport, Vec<ManifestViolation>> {
    let graph = build_graph(manifest);
    validate_manifest(&graph, &foundation_app_ids())?;   // 复用§3.2校验，不重复实现
    let levels = topological_levels(&graph);
    let diffs = manifest.apps.iter()
        .map(|app| compute_version_diff(app))   // 通过既有helm diff插件获取，本文档不新增diff引擎(§6.1既定)
        .collect();
    Ok(DryRunReport { levels, diffs })
    // 不调用deploy_single_app/invoke_scaffold，不产生任何RUNNING状态迁移或实际部署副作用(§6.1既定)
}
```

### 6.2 回滚（对应RGS-BAS-024§6.2四步流程）

```rust
async fn rollback_run(run_id: RunId) -> Result<(), OrchestrationError> {
    let succeeded_apps = fetch_apps_by_state(run_id, AppState::Succeeded)?;   // 步骤1
    let graph = build_graph(&load_manifest_for_run(run_id)?);
    let levels = topological_levels(&graph);

    // 步骤2：按依赖图的逆拓扑序排列——levels本身是正向拓扑序(基础设施层在level_0)，
    // 故回滚顺序=levels.reverse()，业务域App(高level)先回滚，基础设施层(level_0)最后回滚
    for level in levels.iter().rev() {
        for app_id in level {
            if !succeeded_apps.contains(app_id) { continue; }   // 本次run未成功部署的App无需回滚
            // 步骤3：逐个执行并等待成功后再回滚下一个(严格串行，不并行，与部署阶段的层内并行不同——
            // 回滚失败的影响面比部署失败更严重，串行便于每步失败后立即人工介入而非并发放大风险)
            rollback_single_app(run_id, app_id).await?;
        }
    }
    // 步骤4：全部完成后状态迁移为ROLLED_BACK，写入审计记录
    mark_run_status(run_id, RunStatus::RolledBack)?;
    write_audit_record(run_id, AuditEvent::RollbackCompleted)?;
    Ok(())
}

async fn rollback_single_app(run_id: RunId, app_id: &AppId) -> Result<(), OrchestrationError> {
    let row = fetch_app_state(run_id, app_id)?;
    invoke_helm_rollback(&row.scaffold_ref, row.pre_run_revision).await?;   // helm rollback到本次run执行前的revision
    occ_update_state(run_id, app_id, row.version, AppState::RolledBack)?;
    Ok(())
}
```

**回滚顺序正确性说明**（落实§6.2"避免在业务域App仍在运行、仍依赖某基础设施层App时，过早把该基础设施层App回滚掉"）：`levels.iter().rev()`确保`level_N`（最深层业务域）总是先于`level_0`（基础设施层）执行回滚，任一时刻仍在运行的业务域App所依赖的基础设施层App尚未被回滚，避免"依赖悬空"窗口。

---

## 7. 强制联动检查脚本设计

对应RGS-BAS-024§7`scripts/check-cluster-manifest.sh`校验规则，落实为具体校验逻辑（本文档仅设计规则，具体shell/Rust实现留待实施阶段，同RGS-BAS-024§7原文声明）：

```
# scripts/check-cluster-manifest.sh 校验规则(伪代码，非最终实现)
1. 解析 cluster-manifest.yaml 的全部 apps[].app_id
2. 解析 docs/00-基准与治理/RGS-REQ-004_附件C_可追溯性矩阵.md §7域注册表的全部已登记域代码
3. 计算 (已登记域代码集合) - (清单app_id集合) = 缺失集合
4. 缺失集合非空 => CI失败，报告缺失的app_id列表(对应RGS-BAS-024§7"若存在已注册域但未出现在清单apps列表中的情况，CI失败并报告缺失的app_id")
5. 额外调用§3.2 validate_manifest，任何ManifestViolation同样导致CI失败
```

该脚本与RGS-DTL-002§6.3`check-mount-record-consistency.sh`是**两个独立脚本**、校验对象不同（后者校验单App的Mount Record与NetworkPolicy一致性，前者校验集群清单与全局域注册表的完整性），但复用同一"CI阶段调用独立校验脚本"的既有模式，不因两者校验对象不同而在CI流水线组织上产生分歧——两者均可挂载到既有CI阶段模式中（RGS-DTL-002§4），本文档不重复设计CI YAML本身。

---

## 8. CLI命令具体规范

对应RGS-BAS-024§8子命令表，落实为具体参数与退出码规范：

| 子命令 | 参数 | 退出码0 | 退出码非0 |
|---|---|---|---|
| `deploy-cluster validate <manifest>` | `manifest`路径 | 校验全部通过 | 打印全部`ManifestViolation`（§3.2），逐条列出 |
| `deploy-cluster plan <manifest> --env <env>` | `manifest`路径、`--env` | 输出§6.1 `DryRunReport`（层级顺序+diff），不接触实际环境 | 校验失败（先执行§3.2校验，同`validate`） |
| `deploy-cluster apply <manifest> --env <env>` | 同上，额外`--triggered-by <operator>` | 全部App达`SUCCEEDED`，打印生成的`run_id` | 任一层级失败，打印`run_id`与失败App列表，供后续`resume`使用 |
| `deploy-cluster resume <run_id>` | `run_id` | 续跑后全部达`SUCCEEDED` | 同上，可重复调用直至成功或人工判定需要`rollback` |
| `deploy-cluster status <run_id>` | `run_id` | 打印状态表当前快照（每App的`state`/`retry_count`/`updated_at`） | `run_id`不存在 |
| `deploy-cluster rollback <run_id>` | `run_id` | 全部已成功App均已回滚，状态迁移`ROLLED_BACK` | 回滚过程中途失败（§6.2串行执行，某一步失败即停止，不继续回滚下一个App，避免在不确定状态下扩大操作面） |

`apply`/`resume`/`rollback`三个子命令均对应§9"每条状态迁移记录写入既有审计留痕存储"要求，`triggered_by`字段（`apply`显式传入，`resume`/`rollback`复用发起该命令的当前操作者身份）贯穿写入§2 `deploy_run_state_changes.triggered_by`列。

---

## 9. 本文档的覆盖范围与后续计划

本文档覆盖：编排状态表（`deploy_runs`/`deploy_run_apps`/`deploy_run_state_changes`）物理DDL、依赖图构建/三条校验规则（孤儿引用/DFS环检测/基础设施前置）/Kahn拓扑排序的完整伪代码、编排主循环（含OCC并发防护与重试边界）、幂等续跑逻辑、Dry-run与回滚（含逆拓扑序正确性说明）的完整伪代码、集群清单强制联动检查脚本的校验规则、CLI六个子命令的参数与退出码具体规范。

本版本明确不覆盖、留待后续：

- `cluster-manifest.yaml`的完整JSON/YAML Schema校验实现代码——本文档§3给出校验**规则**的伪代码，具体Schema校验库（如`jsonschema`/`serde`自定义`Deserialize`校验）选型与逐字段实现留待实施阶段。
- `scripts/check-cluster-manifest.sh`的完整shell/Rust实现——本文档§7只给出校验规则的步骤描述，同RGS-BAS-024原文声明"具体实现留待实施阶段"，本文档不越权提前实现。
- 与具体CI平台（GitHub Actions/GitLab CI等）绑定的触发API、凭据范围细节——RGS-BAS-024§10已明确"留待实施阶段的技术评审最终确定"，TBD-DEP-001标记为"部分决议"，本文档同样不代为完成剩余部分。
- §9A部署时长基准（P50 10分钟/P99 20分钟）的PH-4实测校准结果——当前为RGS-BAS-024设计阶段估算值，本文档不重复展开该估算过程（已在RGS-BAS-024§9A完整给出，无需详细设计层面的进一步展开），仅在CLI规范（§8）中承接"可度量"这一要求本身（`status`子命令输出的`updated_at`时间戳序列即为该度量的数据来源）。

后续详细设计建议顺序：与RGS-DTL-001§12/RGS-DTL-022§7/RGS-DTL-023§8建议一致，本文档与RGS-DTL-022（弹性容量）、RGS-DTL-023（请求处理链管道）三者互不阻塞，可并行推进；`cluster-manifest.yaml` Schema校验实现建议尽早启动，因其阻塞§7强制联动检查脚本从"规则设计"转为"CI可执行"。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-024§2 集群清单Schema | §2、§3 |
| RGS-BAS-024§3.1 依赖图构建 | §3.1 |
| RGS-BAS-024§3.2 校验规则 | §3.2 |
| RGS-BAS-024§3.3 拓扑排序 | §3.3 |
| RGS-BAS-024§4 编排状态机与主循环 | §2、§4 |
| RGS-BAS-024§5 幂等性设计 | §5 |
| RGS-BAS-024§6.1 Dry-run | §6.1 |
| RGS-BAS-024§6.2 回滚 | §6.2 |
| RGS-BAS-024§7 与ARC-018挂载脚手架强制联动 | §7 |
| RGS-BAS-024§8 CLI工具设计 | §8 |
| RGS-BAS-024§9 高可用与审计 | §2、§4、§8 |
| RGS-BAS-024§9A 部署时长基准 | §9（明确不重复展开） |
| RGS-BAS-024§10 选型建议（TBD-DEP-001） | §9（明确排除） |
| RGS-DTL-001§3.2（OCC模式先例，本文档§2/§4复用） | §2、§4 |
| RGS-DTL-002§4（CI阶段模式，本文档§7复用） | §7 |
