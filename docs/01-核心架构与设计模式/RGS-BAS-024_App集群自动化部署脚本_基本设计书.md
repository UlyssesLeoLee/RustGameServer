# 基本设计书（基本設計書 / Basic Design Document）

**App集群自动化部署脚本 Automated Cluster Deployment Scripts for Atomic Apps**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-024 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-027 需求定义书（ARC-042） |
| 制定日 | 2026-08-17 |
| 最终更新日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定。将RGS-REQ-027 ARC-042展开为集群清单Schema、依赖图算法、编排状态机、CLI工具设计、与ARC-018脚手架检查清单的强制联动方式 | 全部 |
| 0.2 | 2026-08-17 | 架构师 | — | 补齐设计缺口：NFR-DEP-003要求给出可度量的部署时长具体基准，此前正文无任何数字。新增§9A，给出基于依赖图层级数与单App部署耗时的估算目标（P50 10分钟/P99 20分钟），并声明估算不含灾备重建的基础设施状态恢复耗时（RSK-DEP-002），须PH-4实测校准 | NFR-DEP-003 |
| 0.3 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部36个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§2.1/§3.1.1/§3.2.1/§3.3.1/§4.1/§5.1/§6.1.1/§6.2.1/§7.1/§8.1/§9.1/§9A.1/§10.1 全部13个 ## L2/L3 功能段加"本功能日志设计" 5 列详尽版（字段名/触发条件/频率估算/采样策略/脱敏与成本），引用 BAS-001 v1.5 §4.8.3 模板（commit 32d9eb6）+ BAS-003 v0.3 样板（commit 75a001c）+ BAS-004 v0.3 §4.2 二维矩阵 + §4.3 字段 + §5.1 脱敏 + §6.2 强制全采样（commit 47e26b0/0ee6262）；字段名前缀统一 `deploy.*`（deployment 部署域），与 BAS-001 `log.*` / BAS-002 `mnt.*` / BAS-003 `ops.*` / BAS-010 `pat.*` 区分；显式区分部署任务执行（启动/暂停/恢复/回滚）→ `info!`/`warn!` release 必出 + §6.2 强制全采样（部署动作审计）、镜像拉取/版本校验 → `info!` release 必出（per FR-DEP-001）、灰度/金丝雀发布 → `info!` release 必出 + §6.2 强制全采样（per FR-DEP-007/008）、部署失败/超时 → `error!` §6.2 强制全采样（per FR-DEP-009 + NFR-OP-008 SLA 保障）、部署详细日志（kubectl exec 输出）→ `debug!`/`trace!` debug-only `#[cfg(debug_assertions)]` 守护 release 完全剔除、部署配置变更 → `info!` release 必出（per §9 审计联动 + 状态表持久化）；覆盖 ARC-018 挂载脚手架联动 + ARC-042 DEP 域架构 + ARC-009 状态机 + RGS-BAS-022 容量分档 + RGS-BAS-003 §7 审计留痕 + NFR-DEP-001〜003 + FR-DEP-001〜009 + NFR-PE-008 性能监控 + NFR-OP-008 排查SLA + FR-LOG-010/011/012/013/040 日志规范 + RSK-DEP-001 集群清单遗漏 + RSK-DEP-002 灾备基础设施恢复等全系列相关追溯依据；§11 追溯性新增 AC-DEP-005（`deploy.*` debug-only 宏 release 完全剔除）与 AC-DEP-006（每功能 BAS 文档须含本功能 log 章节），与 BAS-001 v1.5 §4.8.3.4 / BAS-002 v0.4 §13 / BAS-003 v0.3 §13 / BAS-004 v0.3 §12 形成统一规范 | §2.1/§3.1.1/§3.2.1/§3.3.1/§4.1/§5.1/§6.1.1/§6.2.1/§7.1/§8.1/§9.1/§9A.1/§10.1/§11 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 编排层是否严格只调用既有GitOps/Helm Release流程，未重复实现部署机制 |
| 评审（运维） | | | 依赖图预置的基础设施层App清单是否与实际拓扑（RGS-BAS-017）一致 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [集群清单 Schema](#2-集群清单-schema)
3. [依赖图构建与校验](#3-依赖图构建与校验)
4. [编排状态机](#4-编排状态机)
5. [幂等性设计](#5-幂等性设计)
6. [Dry-run与回滚](#6-dry-run与回滚)
7. [与ARC-018挂载脚手架的强制联动](#7-与arc-018挂载脚手架的强制联动)
8. [CLI工具设计](#8-cli工具设计)
9. [高可用与审计](#9-高可用与审计)
10. [选型建议（回应TBD-DEP-001）](#10-选型建议回应tbd-dep-001)
11. [追溯性](#11-追溯性)

---

# 1. 前言

本文档展开RGS-REQ-027定义的DEP域需求，给出集群清单格式、依赖图算法、编排状态机与工具形态的具体设计。核心原则（继承ARC-042）：编排层是**既有ARC-018挂载脚手架之上的一层薄编排**，不引入新的部署执行机制，所有实际部署动作仍是既有GitOps/Helm Release流程。

# 2. 集群清单 Schema

```yaml
# cluster-manifest.yaml
cluster_id: prod-cn-east-1          # 对应RGS-BAS-017拓扑中的具体环境
manifest_version: 3
apps:
  - app_id: GW                       # 网关，基础设施层
    target_version: v2.4.0
    depends_on: []
    tier: foundation
    scaffold_ref: charts/gw
    capacity_tier: T2                # 引用RGS-BAS-022 T0-T3分档，不重新定义容量数值
  - app_id: EVT                      # 事件总线，基础设施层
    target_version: v1.9.0
    depends_on: []
    tier: foundation
    scaffold_ref: charts/evt
    capacity_tier: T2
  - app_id: CFG                      # ARC-016配置中心，基础设施层
    target_version: v1.3.0
    depends_on: []
    tier: foundation
    scaffold_ref: charts/cfg
    capacity_tier: T1
  - app_id: TRD                      # 交易域，业务域App
    target_version: v0.8.0
    depends_on: [GW, EVT, CFG]
    tier: domain
    scaffold_ref: charts/trd
    capacity_tier: T2
  # ……其余全部域App，每新增一个域必须在此登记（见§7强制联动）
environment_overrides:
  dev:
    default_capacity_tier: T0
  staging:
    default_capacity_tier: T1
  prod:
    default_capacity_tier: T2        # 具体档位仍以每个App自身覆盖为准
```

字段说明：

| 字段 | 说明 |
|---|---|
| `app_id` | 对应域简称，须与附件C§7域注册表中的域代码一致 |
| `depends_on` | 该App依赖的其他`app_id`列表，为空表示无依赖（通常为基础设施层） |
| `tier` | `foundation`（基础设施层）或`domain`（业务域），仅用于校验§3.2的强制规则，不影响拓扑排序算法本身 |
| `scaffold_ref` | 指向该App的ARC-018挂载脚手架产物（Helm chart路径），编排层通过此字段调用既有部署机制，不感知chart内部细节 |
| `capacity_tier` | 引用RGS-BAS-022的T0-T3分档，决定该App在目标环境下的初始资源配置 |

### 2.1 本功能日志设计

本节覆盖**集群清单 Schema 加载/解析/校验/环境覆盖**的运行时可观测字段——清单加载、字段解析、版本号校验、`environment_overrides` 解析、清单 schema 违规检测均有 release 必出事件。事件名统一 `deploy.*` 前缀。配置变更是 §9 审计联动 + 状态表持久化的源头，必须 release 必出供 SRE 按 `cluster_id` + `manifest_version` 维度聚合；清单原始 YAML 完整 dump 走 `debug!` 守护（高吞吐配置对象 + 含 secrets/连接串风险）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.manifest.loaded` | 编排层读取 `cluster-manifest.yaml` 完成（IO 成功 + 字节数统计） | 部署命令触发频次（典型 0.1-1/h 全集群） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `cluster_id`／`manifest_version`／`byte_size`；约 280B/条 |
| `deploy.manifest.parsed` | YAML 解析完成，结构化对象已构造 | 同上 | release 必出（`info!` 强制全采样） | 含 `app_count`／`environment_count`；约 220B/条；无敏感字段 |
| `deploy.manifest.environment_override_applied` | `environment_overrides` 段对当前 `--env` 取值已合并（含 `default_capacity_tier`） | 同上 | release 必出（`info!` 强制全采样，**配置变更审计**） | 含 `env`／`default_capacity_tier`；约 240B/条 |
| `deploy.manifest.parse_error` | YAML 语法错误 / 必需字段缺失 / 类型不匹配（per §2 Schema 字段说明） | 极低（配置错） | release 必出（`error!` §6.2 强制全采样，per BAS-004 v0.3 §6.2） | 含 `error_kind`／`line`／`column`／`error`；约 320B/条；**绝不写明文 secrets** |
| `deploy.manifest.schema_violation` | 字段值不在 Schema 允许集内（如 `tier` 取非 `foundation`/`domain`、`capacity_tier` 取非 T0-T3） | 极低（配置错） | release 必出（`error!` 强制全采样） | 含 `app_id`／`field`／`value`／`allowed`；约 280B/条 |
| `deploy.manifest.duplicate_app_id` | 同一 `app_id` 在 `apps` 列表中重复登记 | 极低（配置错） | release 必出（`error!` 强制全采样） | 含 `app_id`／`line_numbers[]`；约 240B/条 |
| `deploy.manifest.debug.raw_yaml_dump` | 清单原始 YAML 完整 dump（含 `environment_overrides` 全集） | 偶发（故障复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 5-50KB/条（清单规模决定；release 剔除零运行时开销） |
| `deploy.manifest.debug.parsed_object_dump` | 解析后的结构化对象 dump（含每 App 完整字段） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 3-30KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `deploy.manifest.debug.raw_yaml_dump` 在大型清单（23 个 App 估算）下可达 30KB+ —— `#[cfg(debug_assertions)]` 守护避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `deploy.manifest.*_error` 系列**绝不写明文 secrets**（per BAS-004 v0.3 §5.1 脱敏黑名单）：`environment_overrides` 段虽不直接含连接串，但**禁止**在 error 字段中回显原始 YAML 片段以免泄露后续追加的 secret 字段
- `deploy.manifest.environment_override_applied` 是 §9 审计联动的关键源（**配置变更** → 状态表 + 审计留痕），必须 release 必出

# 3. 依赖图构建与校验

## 3.1 构建

编排层读取清单中全部`app_id`与`depends_on`，构建有向图：`app_id → depends_on中的每个app_id`的边表示"前者依赖后者"。

### 3.1.1 本功能日志设计

本节覆盖**依赖图构建**的运行时可观测字段——节点注册、边插入、构建完成均有 release 必出事件。事件名统一 `deploy.graph.*` 前缀。构建是 §3.2 校验规则的前置步骤，**必须**在构建失败时强制全采样（per FR-DEP-002）；完整图 dump 走 `debug!` 守护（高吞吐对象 + 含调用链敏感信息）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.graph.construction_started` | 编排层开始从已解析清单构建有向图 | 部署命令触发频次（典型 0.1-1/h 全集群） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `app_count`／`edge_count_estimate`；约 220B/条 |
| `deploy.graph.node_registered` | 单个 App 节点插入图（每 App 一条） | 同上 | release 必出（`info!` 强制全采样，per §6.2 强制全采样） | 含 `app_id`／`tier`；约 200B/条 × 23 = 4.6KB/部署；无敏感字段 |
| `deploy.graph.edge_inserted` | 单条依赖边插入（每依赖关系一条） | 同上 | release 必出（`info!` 强制全采样） | 含 `from_app`／`to_app`；约 220B/条 × N 边 |
| `deploy.graph.construction_completed` | 全部节点 + 边插入完成，准备进入 §3.2 校验 | 同上 | release 必出（`info!` 强制全采样） | 含 `node_count`／`edge_count`／`elapsed_ms`；约 240B/条 |
| `deploy.graph.construction_failed` | 构建过程中未预期异常（如内部数据结构 OOM、节点 ID 冲突） | 极低 | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `error_kind`／`error`／`trace_id`；约 320B/条 |
| `deploy.graph.debug.full_dag_dump` | 完整有向图 dump（节点 + 邻接表 + 反向索引） | 偶发（故障复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB/条（节点数决定；release 剔除零运行时开销） |
| `deploy.graph.debug.adjacency_list_dump` | 邻接表逐行 dump（用于可视化复盘） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-5KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `deploy.graph.debug.full_dag_dump` 邻接表逐行打印在高 App 数（23 个）+ 高边数场景下可能 10KB+ —— `#[cfg(debug_assertions)]` 守护避免 release 误开 RUST_LOG=debug 时淹没生产日志
- `deploy.graph.node_registered` / `edge_inserted` 是**每 App/每边一条**的细粒度事件，在大型清单下累积字节数显著（23 个 App + ~30 条边 ≈ 12KB/部署），但仍 release 必出 —— **配置拓扑**的完整可观测是 RSK-DEP-001 防护的关键（清单被遗漏时通过节点注册事件缺失立即可见）
- `deploy.graph.construction_failed` 必须 `error!` 强制全采样（per FR-DEP-002 + NFR-OP-008 排查 SLA），含 `trace_id` 便于跨服务追踪

## 3.2 校验规则（对应FR-DEP-002/FR-DEP-004）

1. **无环校验**：对构建的图执行环检测（如Tarjan强连通分量或简单DFS染色法）；发现环则报告完整环路径（如`TRD → EVT → TRD`），编排运行在执行前以非零退出码终止，不发起任何部署
2. **基础设施前置校验**：预置的基础设施层App集合（GW/EVT/CFG/可观测性基座/密钥管理，与`tier: foundation`标记的App集合比对）须满足：每个`tier: domain`的App，其`depends_on`的传递闭包必须包含全部`tier: foundation`的App；不满足则清单不合法
3. **孤儿引用校验**：`depends_on`中引用的`app_id`必须存在于清单的`apps`列表中，否则报错

### 3.2.1 本功能日志设计

本节覆盖**依赖图校验规则（无环 / 基础设施前置 / 孤儿引用）**的运行时可观测字段——三类校验通过 / 失败均有 release 必出事件。事件名统一 `deploy.validate.*` 前缀。**所有校验失败**均为 `error!` 强制全采样（per FR-DEP-002 / FR-DEP-004 + NFR-OP-008 SLA 保障 + RSK-DEP-001 集群清单遗漏防护），失败时编排运行**不发起任何部署**并以非零退出码终止（per §3.2 主条款）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.validate.cycle_check_started` | Tarjan SCC / DFS 染色法开始执行无环校验 | 部署命令触发频次（典型 0.1-1/h） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `algorithm`／`node_count`；约 200B/条 |
| `deploy.validate.cycle_detected` | **环检测命中**（如 `TRD → EVT → TRD`），报告完整环路径 | 极低（配置错） | release 必出（`error!` §6.2 强制全采样，per FR-DEP-002） | 含 `cycle_path[]`（如 `[TRD, EVT, TRD]`）／`cycle_length`；约 320B/条 |
| `deploy.validate.foundation_missing` | 某个 `tier: domain` App 的 `depends_on` 传递闭包**未包含全部** `tier: foundation` App | 极低（配置错） | release 必出（`error!` 强制全采样，per FR-DEP-004） | 含 `app_id`／`missing_foundation_apps[]`；约 340B/条 |
| `deploy.validate.foundation_check_passed` | 基础设施前置校验通过 | 部署命令触发频次 | release 必出（`info!` 强制全采样） | 含 `domain_app_count`／`foundation_app_count`；约 220B/条 |
| `deploy.validate.orphan_reference` | `depends_on` 中引用的 `app_id` 不在 `apps` 列表中 | 极低（配置错） | release 必出（`error!` 强制全采样） | 含 `referencing_app`／`missing_app`；约 240B/条 |
| `deploy.validate.passed` | 全部三类校验（无环 / 基础设施前置 / 孤儿引用）通过 | 部署命令触发频次 | release 必出（`info!` 强制全采样） | 含 `elapsed_ms`／`total_app_count`；约 240B/条 |
| `deploy.validate.failed` | 任一校验未通过，编排运行终止（非零退出码） | 极低 | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2 + NFR-OP-008） | 含 `failed_check`（cycle/foundation/orphan）／`error`；约 320B/条 |
| `deploy.validate.debug.check_trace` | 校验算法逐节点 trace（DFS 访问序、SCC 编号等） | 极低（故障复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 200B-2KB/条（节点数决定，release 剔除） |
| `deploy.validate.debug.closure_dump` | `depends_on` 传递闭包 dump（每 App 完整闭包） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `deploy.validate.cycle_detected` 报告**完整环路径**（`[TRD, EVT, TRD]`）以便 SRE 一眼定位循环依赖，但禁止在路径中夹带 secrets —— `depends_on` 字段本身无 secret，但**禁止**未来扩展为带凭据依赖时回显
- `deploy.validate.failed` 是 RSK-DEP-001（新增域 App 时集群清单遗漏）的**关键审计事件**——与 §7 强制联动 CI 校验形成双层防护（CI 拦截 + 运行时拦截），必须 `error!` 强制全采样
- `deploy.validate.debug.closure_dump` 传递闭包在高 App 数下可能 5KB+ —— `#[cfg(debug_assertions)]` 守护避免 release 误开 RUST_LOG=debug 时撑爆日志

## 3.3 拓扑排序

使用Kahn算法对DAG做拓扑排序，产出"执行层级"列表（`level_0`：入度为0的App，即基础设施层；`level_1`：仅依赖level_0的App；依此类推）。同一层级内的App在执行阶段并行处理，跨层级严格串行等待上一层级全部成功。

### 3.3.1 本功能日志设计

本节覆盖**Kahn 拓扑排序产出执行层级**的运行时可观测字段——层级计算、入度更新、层级并行边界均有 release 必出事件。事件名统一 `deploy.topology.*` 前缀。层级数 + 每层 App 数是 §9A 部署时长基准（依赖图层级数估算）的**真实值**，必须 release 必出供 PH-4 实测校准（per NFR-DEP-003 + GOV-OLU-004 同类纪律）；排序过程逐层 trace 走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.topology.computed` | Kahn 算法完成层级列表计算 | 部署命令触发频次（典型 0.1-1/h） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `level_count`／`total_app_count`／`elapsed_ms`；约 240B/条 |
| `deploy.topology.level_summary` | 每层汇总一行（level_index + app_count） | 同上 | release 必出（`info!` 强制全采样，**性能基准**，per NFR-DEP-003 + §9A 估算依据） | 含 `level_index`／`app_count`；约 200B/条 × 4 层 = 0.8KB/部署；无敏感字段 |
| `deploy.topology.parallel_boundary_ready` | 某一层全部 App 入度归零，并行边界已就绪 | 同上 | release 必出（`info!` 强制全采样） | 含 `level_index`／`ready_app_count`；约 200B/条 |
| `deploy.topology.sink_detected` | 排序完成但存在无下游的"汇点"（终态 App，无任何 App 依赖之） | 同上 | release 必出（`info!` 强制全采样，**拓扑完整性审计**） | 含 `sink_app_count`／`sink_app_ids[]`（仅当 ≤ 10 时展开）；约 280B/条 |
| `deploy.topology.kahn_queue_exhausted` | Kahn 队列耗尽但仍有节点未访问（理论上不应发生，是 §3.2 校验漏过的隐藏环） | 极低（防御性） | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `remaining_node_count`／`remaining_apps[]`；约 320B/条 |
| `deploy.topology.debug.per_level_app_list` | 每层完整 App 列表 dump（用于 PH-4 实测校准可视化） | 偶发（PH-4 校准） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-3KB/条（每层 App 数决定，release 剔除） |
| `deploy.topology.debug.kahn_progress_trace` | Kahn 入度更新逐事件 trace（每 pop/push 一次） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 200B-1KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `deploy.topology.level_summary` 是**性能基准**的源头（per NFR-DEP-003 + §9A 估算依据 "App 总数 / 依赖图层级数估算"）—— 必须在 release profile 输出用于 PH-4 实测校准，**不**走 `debug!` 守护
- `deploy.topology.sink_detected` 是**拓扑完整性审计**事件（per §3.2 校验"理论保证" + 此处运行时兜底），用于发现配置中可能的"孤儿分支"（如某域 App 注册了但无任何 App 依赖之，可能为退场未清理）
- `deploy.topology.kahn_queue_exhausted` 是**防御性 error 事件**（理论上 §3.2 校验已拦截环，Kahn 耗尽说明存在并发清单修改或校验漏过）—— 必须 `error!` 强制全采样 + 含 `remaining_apps[]` 用于人工定位

# 4. 编排状态机

每个`run_id`维护一张状态表，每个App一行：

| 状态 | 含义 | 允许的下一状态 |
|---|---|---|
| PENDING | 待执行，尚未轮到其层级 | RUNNING |
| RUNNING | 正在调用其`scaffold_ref`对应的Helm Release流程 | SUCCEEDED / FAILED |
| SUCCEEDED | 部署完成且健康检查通过 | （终态，续跑时跳过） |
| FAILED | 超过FR-DEP-009定义的重试次数仍未成功 | RUNNING（续跑时重试）/ SKIPPED（人工标记跳过，需审批） |
| BLOCKED | 其上游依赖未SUCCEEDED，本层级尚未开始 | RUNNING |
| ROLLED_BACK | 触发整体回滚后的终态 | （终态） |

状态表持久化于既有PostgreSQL（复用现有实例，不新增存储组件），每次状态迁移写一条状态变更记录，供审计（§9）与续跑（§5）读取。

编排主循环伪代码：

```
for level in topological_levels:
    for app in level (并行):
        if app.state == SUCCEEDED: continue          # 续跑时跳过已成功
        transition(app, RUNNING)
        result = invoke_scaffold(app.scaffold_ref, app.target_version)  # 调用既有Helm Release
        if result.ok: transition(app, SUCCEEDED)
        else: transition(app, FAILED); pause_run()    # 失败即暂停，不继续下一层级
    if any(app.state != SUCCEEDED for app in level): break
```

### 4.1 本功能日志设计

本节覆盖**编排状态机（PENDING → RUNNING → SUCCEEDED/FAILED/BLOCKED/ROLLED_BACK）**的运行时可观测字段——状态迁移、暂停、续跑、回滚均有 release 必出事件。事件名统一 `deploy.run.*` / `deploy.app.*` 前缀。**部署任务执行**（启动/暂停/恢复/回滚）是审计关键，per §6.2 强制全采样 + §9 审计留痕联动；**部署失败/超时**走 `error!` 强制全采样（per FR-DEP-009 + NFR-OP-008 SLA 保障）；完整状态表 dump 走 `debug!` 守护（高吞吐持久化对象）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.run.started` | 编排主循环开始（`run_id` 生成 + 状态表初始化） | 部署命令触发频次（典型 0.1-1/h） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `run_id`／`cluster_id`／`manifest_version`／`app_count`；约 320B/条 |
| `deploy.run.resumed` | 续跑已存在的 `run_id`（per §5 幂等性 + §9 进程崩溃恢复） | 偶发（故障续跑） | release 必出（`info!` 强制全采样） | 含 `run_id`／`resumed_from_state`／`skipped_succeeded_count`；约 320B/条 |
| `deploy.run.paused` | 主循环遇 FAILED 后 `pause_run()` 触发（per §4 主循环伪代码） | 偶发 | release 必出（`warn!` 强制全采样，**部署动作审计**） | 含 `run_id`／`paused_at_level`／`failed_app_id`／`error_kind`；约 340B/条 |
| `deploy.run.completed` | 全部层级 App 状态转为 SUCCEEDED | 部署命令触发频次 | release 必出（`info!` 强制全采样，**部署任务执行完成**） | 含 `run_id`／`total_elapsed_ms`／`succeeded_count`；约 320B/条 |
| `deploy.run.failed` | 任一层级全部 App 失败 / 主循环超时 / 整体回滚触发 | 极低 | release 必出（`error!` §6.2 强制全采样，per FR-DEP-009 + NFR-OP-008） | 含 `run_id`／`failure_kind`／`failed_apps[]`／`trace_id`；约 380B/条 |
| `deploy.run.timeout` | 整个 run 超过 §9A 估算上限（如 P99 20 分钟） | 极低 | release 必出（`error!` 强制全采样） | 含 `run_id`／`elapsed_ms`／`timeout_threshold_ms`／`stuck_level`；约 320B/条 |
| `deploy.app.state_transition` | 单 App 状态迁移（PENDING→RUNNING / RUNNING→SUCCEEDED / RUNNING→FAILED / BLOCKED→RUNNING） | 稳态 1/s、峰值 50/s（一个 run 23 App × 状态数） | release 必出（`info!` 强制全采样，per §9 审计留痕 + §6.2） | 含 `run_id`／`app_id`／`from_state`／`to_state`／`attempt`；约 260B/条 × N |
| `deploy.app.attempt_started` | 单 App 某次重试/续跑开始（per FR-DEP-009 重试机制） | 偶发（首次失败后） | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`attempt`／`max_attempts`；约 240B/条 |
| `deploy.app.health_check_timeout` | 单 App 健康检查超时（per §5 复用 readiness 探针） | 极低 | release 必出（`error!` 强制全采样，per FR-DEP-009） | 含 `run_id`／`app_id`／`probe_name`／`elapsed_ms`；约 280B/条 |
| `deploy.app.retry_exhausted` | 单 App 重试次数耗尽（per FR-DEP-009） | 极低 | release 必出（`error!` 强制全采样） | 含 `run_id`／`app_id`／`final_attempt`／`last_error`；约 320B/条 |
| `deploy.app.skipped_succeeded` | 续跑时跳过已 SUCCEEDED 的 App（per §5 幂等性） | 偶发（续跑场景） | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`original_succeeded_at`；约 280B/条 |
| `deploy.run.debug.full_state_table` | 完整状态表 dump（每 App 当前状态 + 状态变更历史） | 偶发（故障复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB/条（App 数 + 历史决定，release 剔除零运行时开销） |
| `deploy.app.debug.helm_release_output` | Helm Release 调用完整 stdout/stderr（含底层 pod event / describe 输出） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-50KB/条（kubectl output 决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + ARC-018 既有部署机制边界）：
- `deploy.app.debug.helm_release_output` 在 Helm Release 失败时 kubectl describe 输出可达 50KB+ —— `#[cfg(debug_assertions)]` 守护避免 RUST_LOG=debug 误开时撑爆生产日志通道（这是用户特别注明的"部署详细日志 kubectl exec 输出 → debug-only"要求）
- `deploy.run.*` 全系列 release 必出（`info!`/`warn!`/`error!`）—— **部署任务执行**是 §9 审计联动（`cluster_deploy_state_change` 事件类型）的核心源头，per §6.2 强制全采样白名单
- `deploy.app.state_transition` 累积字节数显著（23 App × 5-6 状态 = ~120 条/部署 × 260B ≈ 31KB/部署），但仍 release 必出 —— **完整状态轨迹**是事后复盘 RSK-DEP-001/002 唯一可信来源
- `deploy.app.attempt_started` 的 `attempt` 字段由独立 `let` 绑定（per BAS-004 v0.3 §4.3 规则 #4），**不**在 `info!` 宏参数内调用递增函数

# 5. 幂等性设计

- 部署步骤本身的幂等性完全依赖底层Helm Release的声明式特性（`helm upgrade --install`语义），编排层不引入任何自定义的"先删后建"类非幂等操作
- 续跑（对已存在的`run_id`重新调用编排）时，编排层先读取状态表，仅对`PENDING`/`FAILED`/`BLOCKED`状态的App重新计算是否可执行，`SUCCEEDED`的App直接跳过，不重复调用其Helm Release（即使重复调用本身也是幂等的，跳过只是为了缩短续跑时长）
- 健康检查（判定`RUNNING → SUCCEEDED`）复用该App既有的readiness探针/健康检查端点（RGS-BAS-002脚手架检查清单已要求各App提供），编排层不重新定义健康判定逻辑

### 5.1 本功能日志设计

本节覆盖**幂等性设计（续跑跳过 SUCCEEDED + 健康检查复用）**的运行时可观测字段——续跑判定、健康检查结果、Helm Release 重复调用避免均有 release 必出事件。事件名统一 `deploy.idem.*` / `deploy.app.health_*` 前缀。健康检查通过是 SUCCEEDED 的判定依据，per §6.2 强制全采样 + §9 审计联动；健康检查失败 / 超时是 §4.1 `deploy.app.retry_exhausted` 的源头；完整 readiness probe 请求 / 响应 dump 走 `debug!` 守护（含 K8s endpoint 信息）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.idem.resume_detected` | 续跑入口检测到 `run_id` 已存在（per §5 + §9 进程崩溃恢复） | 偶发（故障续跑） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `run_id`／`previous_state_summary`；约 320B/条 |
| `deploy.idem.app_skipped_succeeded` | 续跑时跳过已 SUCCEEDED 的 App（per §5 主条款） | 偶发（续跑场景） | release 必出（`info!` 强制全采样，**幂等性证明**） | 含 `run_id`／`app_id`／`original_succeeded_at`／`target_version`；约 320B/条 |
| `deploy.idem.app_will_retry` | 续跑时识别 PENDING/FAILED/BLOCKED 状态 App 准备重试 | 偶发 | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`current_state`；约 260B/条 |
| `deploy.app.health_check_started` | 单 App 健康检查发起（复用 readiness 探针 / 健康检查端点，per §5） | 稳态 1/s、峰值 50/s | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`probe_endpoint`；约 240B/条 |
| `deploy.app.health_check_passed` | 健康检查通过（判定 RUNNING → SUCCEEDED） | 同上 | release 必出（`info!` 强制全采样，**部署任务执行关键事件**） | 含 `run_id`／`app_id`／`probe_name`／`latency_ms`；约 240B/条 |
| `deploy.app.health_check_failed` | 健康检查失败（探针返回非 2xx / 端点不可达） | 偶发 | release 必出（`warn!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `run_id`／`app_id`／`probe_name`／`http_status`／`error`；约 320B/条 |
| `deploy.app.health_check_timeout` | 健康检查超时（probe 等待响应超时） | 极低 | release 必出（`error!` 强制全采样） | 含 `run_id`／`app_id`／`probe_name`／`elapsed_ms`／`timeout_ms`；约 280B/条 |
| `deploy.app.helm_release_invoke_started` | Helm Release 实际调用开始（per §5 依赖底层声明式幂等） | 稳态 1/s、峰值 50/s | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`chart_ref`／`target_version`；约 320B/条 |
| `deploy.app.helm_release_idempotent_noop` | Helm Release 检测到目标 revision 已存在，跳过实际变更 | 偶发（续跑 / 重复调用） | release 必出（`info!` 强制全采样，**幂等性确认**） | 含 `run_id`／`app_id`／`current_revision`／`target_revision`；约 280B/条 |
| `deploy.app.debug.probe_request_dump` | 完整 readiness probe HTTP request dump（headers + body） | 偶发（健康检查失败复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-3KB/条（release 剔除零运行时开销） |
| `deploy.app.debug.probe_response_dump` | 完整 probe response dump（含 body） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-5KB/条（response body 决定，release 剔除） |
| `deploy.app.debug.helm_diff_dump` | Helm `helm diff` 插件完整输出（per §6.1 dry-run） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-20KB/条（manifest 大小决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + ARC-018 既有机制边界）：
- `deploy.app.debug.helm_diff_dump` 在大型 manifest 下可达 20KB+ —— `#[cfg(debug_assertions)]` 守护避免 release 误开 RUST_LOG=debug 时撑爆生产日志
- `deploy.app.health_check_*` 系列是**部署任务执行关键事件**（per §4.1 `deploy.app.state_transition` RUNNING→SUCCEEDED 判定依据），per §6.2 强制全采样
- `deploy.idem.app_skipped_succeeded` 是**幂等性证明**的审计依据（per §5 主条款 "重复调用本身也是幂等的，跳过只是为了缩短续跑时长"），必须 release 必出供 SRE 验证"为何该 App 在本 run 中未触发实际部署"
- `deploy.app.debug.probe_response_dump` 中的 `http_status` 字段在 §4.1 `deploy.app.health_check_failed` 中以**枚举化摘要**输出，避免在 release profile 中回显完整 body

# 6. Dry-run与回滚

## 6.1 Dry-run

Dry-run模式执行§3的构建与校验、§3.3的拓扑排序，输出：各层级顺序、每个App当前版本与`target_version`的diff（通过既有Helm的`--dry-run`/`helm diff`插件获取，不新增diff引擎），但不进入§4状态机的`RUNNING`阶段，不产生任何实际部署副作用。

### 6.1.1 本功能日志设计

本节覆盖**Dry-run 模式（构建+校验+拓扑排序+Helm diff）**的运行时可观测字段——dry-run 进入、各阶段跳过 RUNNING 状态、Helm diff 产出、drift 检测均有 release 必出事件。事件名统一 `deploy.dry_run.*` 前缀。Dry-run **不**进入 §4 状态机 RUNNING，因此**不**触发 §4.1 系列状态迁移事件，避免误审计；diff 完整输出走 `debug!` 守护（高吞吐 manifest 对象）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.dry_run.started` | `deploy-cluster plan` / `validate` 子命令进入 dry-run 模式 | 部署命令触发频次（典型 0.5-5/h，开发者本地 + CI） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `run_id`（dry-run 临时 ID）／`cluster_id`／`manifest_version`；约 320B/条 |
| `deploy.dry_run.phase_skipped` | 显式声明 dry-run 跳过 §4 状态机 RUNNING 阶段（不产生实际部署副作用，per §6.1） | 同上 | release 必出（`info!` 强制全采样，**dry-run 边界审计**） | 含 `skipped_phase`／`reason`；约 200B/条 |
| `deploy.dry_run.graph_validated` | §3 校验通过（无环 / 基础设施前置 / 孤儿引用） | 同上 | release 必出（`info!` 强制全采样） | 含 `app_count`／`level_count`／`elapsed_ms`；约 240B/条 |
| `deploy.dry_run.topology_computed` | §3.3 拓扑排序完成 | 同上 | release 必出（`info!` 强制全采样，per §3.3.1） | 含 `level_count`／`per_level_count[]`；约 260B/条 |
| `deploy.dry_run.helm_diff_started` | 每个 App `helm diff` / `--dry-run` 调用开始 | 稳态 5/s、峰值 50/s（dry-run 并行调用） | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`current_revision`／`target_version`；约 280B/条 |
| `deploy.dry_run.helm_diff_completed` | 单 App Helm diff 完成（无论是否有差异） | 同上 | release 必出（`info!` 强制全采样） | 含 `run_id`／`app_id`／`has_drift`／`diff_size`／`elapsed_ms`；约 280B/条 |
| `deploy.dry_run.drift_detected` | 单 App `current_revision` 与 `target_version` 不一致（drift 命中） | 偶发 | release 必出（`warn!` 强制全采样，per BAS-004 v0.3 §6.2，**配置漂移审计**） | 含 `run_id`／`app_id`／`current_revision`／`target_version`／`resource_count`；约 320B/条 |
| `deploy.dry_run.completed` | 全部 App diff 完成，dry-run 报告输出 | 部署命令触发频次 | release 必出（`info!` 强制全采样，**dry-run 终止**） | 含 `run_id`／`total_app_count`／`drift_count`／`elapsed_ms`；约 320B/条 |
| `deploy.dry_run.helm_diff_failed` | Helm diff 调用失败（chart 解析错误 / kubeconfig 不可达） | 极低 | release 必出（`error!` 强制全采样） | 含 `run_id`／`app_id`／`error_kind`／`error`；约 320B/条 |
| `deploy.dry_run.debug.full_diff_output` | 完整 Helm diff 输出 dump（per manifest resource 块） | 偶发（开发者复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 5-50KB/条（manifest 规模决定，release 剔除零运行时开销） |
| `deploy.dry_run.debug.per_resource_diff` | 逐 resource 级别 diff dump（Deployment / Service / ConfigMap 等） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 10-100KB/条（resource 数决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + ARC-018 既有部署机制边界）：
- `deploy.dry_run.debug.full_diff_output` / `deploy.dry_run.debug.per_resource_diff` 累计可达 100KB+ —— `#[cfg(debug_assertions)]` 守护避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `deploy.dry_run.phase_skipped` 是**dry-run 边界审计**的关键事件（防止 dry-run 模式被错误地升级为实际部署），必须 release 必出供合规审计（per §6.1 "不产生任何实际部署副作用" 强约束）
- `deploy.dry_run.drift_detected` 是**配置漂移审计**的源头（per §3.2 校验"通过"后，运行时发现 actual 与 target 不一致——可能是 chart 渲染后实际值与期望差异），走 `warn!` 强制全采样供 SRE 排查 Helm value 覆盖 / K8s 实际资源漂移
- `deploy.dry_run.helm_diff_*` 全系列必须 `info!` 级别 release 必出（dry-run 是开发者高频命令，per §6.1 "不进入 RUNNING" 边界）—— 但**不**计入 `deploy.run.*` 状态机审计范围

## 6.2 回滚

给定一个`run_id`，回滚流程：

1. 取该run中状态为`SUCCEEDED`的App列表
2. 按依赖图的**逆拓扑序**排列（业务域App在前，基础设施层App在后）
3. 对每个App调用其Helm Release的回滚能力（`helm rollback`到该App在本次run执行前的revision），逐个执行并等待成功后再回滚下一个
4. 全部完成后，该run下涉及的App状态迁移为`ROLLED_BACK`，写入审计记录

回滚顺序刻意与部署顺序相反：避免在业务域App仍在运行、仍依赖某基础设施层App时，过早把该基础设施层App回滚掉。

### 6.2.1 本功能日志设计

本节覆盖**回滚流程（逆拓扑序 + 逐个 helm rollback + 状态 ROLLED_BACK）**的运行时可观测字段——回滚进入、逐 App 回滚、整体完成、回滚失败均有 release 必出事件。事件名统一 `deploy.rollback.*` 前缀。**部署任务执行（回滚）**是审计关键，per §6.2 强制全采样 + §9 审计留痕联动 + 用户特别注明的"启动/暂停/恢复/回滚 → release 必出 + 强制全采样"要求；**回滚失败/超时**走 `error!` 强制全采样（per FR-DEP-009 + NFR-OP-008 SLA）；Helm rollback 详细输出走 `debug!` 守护（高吞吐 kubectl output）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.rollback.started` | `deploy-cluster rollback <run_id>` 进入回滚流程 | 极低（仅故障 / 主动回滚） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2，**部署任务执行关键事件**） | 含 `run_id`／`rollback_target_run_id`／`succeeded_app_count`／`reverse_topology_levels`；约 380B/条 |
| `deploy.rollback.reverse_topology_computed` | 逆拓扑序层级列表计算完成（业务域 App 在前，基础设施层 App 在后，per §6.2） | 极低 | release 必出（`info!` 强制全采样，per §3.3 拓扑排序延伸） | 含 `run_id`／`rollback_level_count`；约 240B/条 |
| `deploy.rollback.app_rollback_started` | 单个 App helm rollback 调用开始（含目标 revision 选定） | 极低 | release 必出（`info!` 强制全采样，**部署任务执行关键事件**） | 含 `run_id`／`app_id`／`target_revision`／`current_revision`；约 280B/条 |
| `deploy.rollback.app_rollback_completed` | 单 App 回滚完成，状态迁移为 ROLLED_BACK | 极低 | release 必出（`info!` 强制全采样，**部署任务执行完成**） | 含 `run_id`／`app_id`／`new_revision`／`elapsed_ms`；约 280B/条 |
| `deploy.rollback.completed` | 全部回滚完成，该 run 下涉及 App 全部为 ROLLED_BACK 状态 | 极低 | release 必出（`info!` 强制全采样，**部署任务执行完成**） | 含 `run_id`／`rolled_back_count`／`total_elapsed_ms`；约 320B/条 |
| `deploy.rollback.app_rollback_failed` | 单 App helm rollback 调用失败（revision 不存在 / chart 损坏 / K8s 拒绝） | 极低 | release 必出（`error!` §6.2 强制全采样，per FR-DEP-009 + NFR-OP-008） | 含 `run_id`／`app_id`／`target_revision`／`error_kind`／`error`／`trace_id`；约 380B/条 |
| `deploy.rollback.timeout` | 单 App 回滚超时（helm rollback 调用超过 P99 估算） | 极低 | release 必出（`error!` 强制全采样） | 含 `run_id`／`app_id`／`elapsed_ms`／`timeout_threshold_ms`；约 320B/条 |
| `deploy.rollback.app_rollback_skipped` | 单 App 因上游依赖未 SUCCEEDED 而无法回滚（per §6.2 逆拓扑序限制） | 极低 | release 必出（`warn!` 强制全采样，**回滚顺序强制**） | 含 `run_id`／`app_id`／`skipped_reason`；约 280B/条 |
| `deploy.rollback.partial_rollback` | 部分 App 回滚成功 / 部分失败（需人工介入） | 极低 | release 必出（`warn!` 强制全采样） | 含 `run_id`／`succeeded_apps[]`／`failed_apps[]`；约 380B/条 |
| `deploy.rollback.debug.helm_rollback_output` | Helm rollback 完整 stdout/stderr（含 K8s 资源更新详情） | 偶发（回滚失败复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 5-100KB/条（kubectl output 决定，release 剔除零运行时开销） |
| `deploy.rollback.debug.reverse_topology_dump` | 逆拓扑序完整层级列表 dump（含每层 App 排序依据） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |
| `deploy.rollback.debug.pre_rollback_revision_dump` | 回滚前每 App 当前 revision 列表 dump（per §6.2 "取该run中状态为SUCCEEDED的App列表"） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-3KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + 用户特别注明的"部署详细日志 kubectl exec 输出 → debug-only"）：
- `deploy.rollback.debug.helm_rollback_output` 在回滚 K8s 资源更新详情时 kubectl describe 输出可达 100KB+ —— `#[cfg(debug_assertions)]` 守护避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `deploy.rollback.*` 全系列 release 必出（`info!`/`warn!`/`error!`）—— **回滚作为部署任务执行的特殊形态**，per §6.2 强制全采样白名单 + §9 审计联动 + 用户特别注明"启动/暂停/恢复/回滚 → release 必出 + 强制全采样"
- `deploy.rollback.app_rollback_skipped` 是**回滚顺序强制**的审计依据（per §6.2 "回滚顺序刻意与部署顺序相反" 强约束），用于 SRE 排查"为何某 App 未回滚"——可能是上游依赖链已断，需先处理
- `deploy.rollback.app_rollback_failed` 必须在 `error` 字段中显式区分"revision 不存在（修复路径：人工选定目标 revision）" vs "chart 损坏（修复路径：重新挂载）" vs "K8s 拒绝（修复路径：检查 RBAC）"—— 三类失败修复路径完全不同，per NFR-OP-008 排查 SLA 保障
- `deploy.rollback.debug.pre_rollback_revision_dump` 中的 `revision` 由独立 `let` 绑定（per BAS-004 v0.3 §4.3 规则 #4），**不**在 `debug!` 宏参数内调用 K8s API

# 7. 与ARC-018挂载脚手架的强制联动

为解决RGS-REQ-027 RSK-DEP-001（新增域App时集群清单可能被遗漏），在RGS-BAS-002的挂载脚手架检查清单中追加一项强制步骤：

> **【新增检查项】新App挂载完成前，必须在`cluster-manifest.yaml`中登记该App的`app_id`/`depends_on`/`scaffold_ref`/`capacity_tier`，且CI须运行本文档§3.2的依赖图校验通过，方可视为挂载完成。**

该检查项通过`scripts/check-docs-consistency.sh`同类思路新增一条独立CI校验脚本（`scripts/check-cluster-manifest.sh`，本文档仅设计其校验规则，具体实现留待实施阶段）：解析`cluster-manifest.yaml`，比对附件C§7域注册表中的全部已登记域，若存在已注册域但未出现在清单`apps`列表中的情况，CI失败并报告缺失的`app_id`。

### 7.1 本功能日志设计

本节覆盖**ARC-018 挂载脚手架与集群清单校验脚本（`scripts/check-cluster-manifest.sh`）强制联动**的运行时可观测字段——CI 校验进入、缺失 App 检测、校验通过 / 失败均有 release 必出事件。事件名统一 `deploy.ci.*` 前缀。**部署配置变更** + **校验失败**是关键审计事件（per §9 审计联动 + RSK-DEP-001 集群清单遗漏防护 + 用户特别注明的"部署配置变更 → release 必出"），必须 release 必出 + §6.2 强制全采样；清单完整 dump 走 `debug!` 守护（高吞吐配置对象）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.ci.check_started` | `scripts/check-cluster-manifest.sh` 进入 CI 阶段（PR 触发 / 定时） | CI 触发频次（典型 10-100/h 全仓库 PR 合并） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `ci_run_id`／`trigger_kind`（PR/cron/manual）; 约 240B/条 |
| `deploy.ci.manifest_loaded` | CI 脚本读取 `cluster-manifest.yaml` 完成 | 同上 | release 必出（`info!` 强制全采样） | 含 `cluster_id`／`manifest_version`／`app_count`；约 240B/条 |
| `deploy.ci.domain_registry_loaded` | CI 脚本读取附件 C §7 域注册表（已登记域列表） | 同上 | release 必出（`info!` 强制全采样） | 含 `registered_domain_count`；约 200B/条 |
| `deploy.ci.missing_app_detected` | **RSK-DEP-001 命中**：已注册域但未出现在 `cluster-manifest.yaml` `apps` 列表中 | 极低（仅新增域时） | release 必出（`error!` §6.2 强制全采样，per RSK-DEP-001 + 用户特别注明的部署配置变更审计） | 含 `missing_app_id`／`domain_code`；约 240B/条 |
| `deploy.ci.unregistered_app_detected` | 清单中存在但域注册表未登记的 App（孤儿 App，违反 ARC-018 挂载流程） | 极低 | release 必出（`warn!` 强制全采样） | 含 `app_id`；约 200B/条 |
| `deploy.ci.check_passed` | 全部校验通过（清单与域注册表一致） | CI 触发频次 | release 必出（`info!` 强制全采样） | 含 `total_app_count`／`elapsed_ms`；约 240B/条 |
| `deploy.ci.check_failed` | 任一校验未通过，CI 退出非零 | 极低 | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `failed_check`（missing_app/unregistered/diff）/`reason`；约 320B/条 |
| `deploy.ci.dependency_graph_validated` | 部署前 CI 阶段运行 §3.2 依赖图校验（per §7 主条款 "CI须运行本文档§3.2的依赖图校验通过"） | 部署命令触发频次 | release 必出（`info!` 强制全采样，per §3.2.1） | 含 `app_count`／`elapsed_ms`；约 240B/条 |
| `deploy.ci.debug.full_manifest_dump` | 清单原始 YAML dump（含 `environment_overrides` 全集） | 极低（CI 失败复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 5-50KB/条（清单规模决定，release 剔除零运行时开销） |
| `deploy.ci.debug.domain_registry_dump` | 域注册表完整 dump（每域 code + description） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + RSK-DEP-001 防护）：
- `deploy.ci.debug.full_manifest_dump` 在大型清单（23 个 App 估算）下可达 30KB+ —— `#[cfg(debug_assertions)]` 守护避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `deploy.ci.missing_app_detected` 是 **RSK-DEP-001 防护的关键审计事件**——与 §3.2.1 `deploy.validate.failed` 形成**双层防护**（CI 阶段拦截 + 运行时拦截），必须 `error!` 强制全采样
- `deploy.ci.check_passed` 虽高频（10-100/h）但 release 必出 —— **完整审计轨迹**是事后复盘"为何某 PR 通过了 CI 但运行时仍失败"的唯一可信来源（可能 CI 与运行时环境差异 / 域注册表不同步）
- `deploy.ci.*` 全系列必须严格**避免回显 secrets**（per BAS-004 v0.3 §5.1）：`environment_overrides` 段虽不直接含连接串，但**禁止**未来扩展为带凭据配置时在 error 字段回显原始 YAML 片段

# 8. CLI工具设计

提供一个编排CLI（形态见§10选型建议），核心子命令：

| 子命令 | 作用 |
|---|---|
| `deploy-cluster plan <manifest> --env <env>` | 执行dry-run，输出拓扑顺序与diff |
| `deploy-cluster apply <manifest> --env <env>` | 执行实际编排部署，生成`run_id` |
| `deploy-cluster resume <run_id>` | 续跑指定run |
| `deploy-cluster status <run_id>` | 查询当前状态表 |
| `deploy-cluster rollback <run_id>` | 触发§6.2回滚流程 |
| `deploy-cluster validate <manifest>` | 仅执行§3校验规则，不涉及任何实际环境 |

### 8.1 本功能日志设计

本节覆盖**编排 CLI 工具（`deploy-cluster`）6 个子命令（plan/apply/resume/status/rollback/validate）**的运行时可观测字段——子命令进入、完成、失败、参数校验均有 release 必出事件。事件名统一 `deploy.cli.*` 前缀。**CLI 子命令**是 §10 选型建议中"编排层入口"的关键节点（per §10 "CLI工具承载§3依赖图算法、§4状态机逻辑、§8命令行接口"），必须 release 必出 + §6.2 强制全采样（per BAS-004 v0.3 §6.2）；CLI 完整 args dump 走 `debug!` 守护（**可能含敏感参数**如 `--registry-credentials`）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.cli.subcommand_invoked` | 任意 `deploy-cluster` 子命令（plan/apply/resume/status/rollback/validate）入口 | 部署命令触发频次（典型 0.1-5/h 混合，CI + 人工） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `subcommand`／`cli_version`／`manifest_path`／`env`；约 280B/条 |
| `deploy.cli.subcommand_completed` | 子命令正常完成（exit code = 0） | 同上 | release 必出（`info!` 强制全采样，**部署任务执行完成**） | 含 `subcommand`／`run_id`（如生成）／`elapsed_ms`／`exit_code`；约 280B/条 |
| `deploy.cli.subcommand_failed` | 子命令异常退出（exit code ≠ 0） | 极低 | release 必出（`error!` §6.2 强制全采样，per BAS-004 v0.3 §6.2 + NFR-OP-008） | 含 `subcommand`／`exit_code`／`error_kind`／`error`／`trace_id`；约 380B/条 |
| `deploy.cli.subcommand_invalid_args` | CLI 参数解析失败（缺少必需参数 / 参数值非法） | 极低（用户操作错） | release 必出（`warn!` 强制全采样） | 含 `subcommand`／`invalid_arg`／`provided_value`；约 240B/条；**绝不回显凭据类参数值** |
| `deploy.cli.config_loaded` | CLI 启动阶段加载本地配置（如 `~/.deploy-cluster/config.toml`） | 部署命令触发频次 | release 必出（`info!` 强制全采样，**部署配置变更审计**） | 含 `config_path`／`config_version`；约 240B/条 |
| `deploy.cli.ci_pipeline_invocation_started` | CLI 通过 CI 平台 Pipeline 触发（per §10 "CLI工具不直接持有CI平台的凭据/触发权限"） | CI 触发频次 | release 必出（`info!` 强制全采样，**CI/CD 边界审计**） | 含 `ci_run_id`／`pipeline_name`／`trigger_kind`；约 280B/条 |
| `deploy.cli.ci_pipeline_invocation_completed` | CI 平台 Pipeline 触发请求完成 | 同上 | release 必出（`info!` 强制全采样） | 含 `ci_run_id`／`pipeline_status`；约 240B/条 |
| `deploy.cli.local_invocation` | CLI 在本地开发环境直接调用（不经 CI 平台） | 偶发（开发者本地） | release 必出（`info!` 强制全采样，**本地 vs CI 边界审计**） | 含 `hostname`／`user`；约 200B/条 |
| `deploy.cli.permission_denied` | CLI 试图调用需要更高权限的子命令（如 apply 无 RBAC 角色） | 极低 | release 必出（`warn!` 强制全采样，per §8 RBAC 联动） | 含 `subcommand`／`user`／`required_role`；约 240B/条 |
| `deploy.cli.debug.full_args_dump` | CLI 完整 args dump（含所有 --flag 值） | 偶发（命令失败复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-3KB/条（args 决定，release 剔除零运行时开销） |
| `deploy.cli.debug.config_file_dump` | 完整本地配置文件 dump（含所有 section） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |
| `deploy.cli.debug.subcommand_trace` | 子命令逐阶段 trace（参数解析→配置加载→清单读取→§3 构建→§3.2 校验） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-2KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §5.1 脱敏黑名单 + §10 选型建议）：
- `deploy.cli.debug.full_args_dump` 中的 `--registry-credentials` / `--vault-token` 等敏感参数**必须**在 CLI 解析阶段以独立 `let` 绑定（per BAS-004 v0.3 §4.3 规则 #4），**绝不**进入 `debug!` 宏参数—— 避免 release build 误开 RUST_LOG=debug 时凭据泄露
- `deploy.cli.subcommand_invoked` / `completed` 是**部署任务执行**的关键入口/出口节点（per §6.2 强制全采样白名单 + 用户特别注明的"部署任务执行 → release 必出"），必须 release 必出
- `deploy.cli.local_invocation` 是**本地 vs CI 边界审计**的依据（per §10 "CLI工具不直接持有CI平台的凭据/触发权限"）—— 本地调用不走 CI 平台权限模型，必须 release 必出供合规审计
- `deploy.cli.config_loaded` 是**部署配置变更审计**的源头（per §9 审计联动 + 用户特别注明的"部署配置变更 → release 必出"），必须 release 必出 + `info!` 强制全采样
- `deploy.cli.debug.config_file_dump` 中的 `vault_token` / `registry_credentials` 等敏感 section **必须**在 dump 前过滤（per §5.1 脱敏黑名单 + ARC-020 + §5 反模式"日志先明文记录…" 同类禁止），通过独立 `redact_config()` 函数预处理后再 dump

# 9. 高可用与审计

- 编排CLI/服务进程本身无状态，状态表持久化于PostgreSQL（复用既有生产实例的HA配置：同步复制+Multi-AZ，不引入新HA机制），进程崩溃后重启可从状态表恢复继续（等价于对同一`run_id`的续跑）
- 每条状态迁移记录（App、旧状态、新状态、时间、触发者）写入既有审计留痕存储（RGS-BAS-003§7），不新建独立审计表，仅新增一个事件类型`cluster_deploy_state_change`

### 9.1 本功能日志设计

本节覆盖**高可用（进程崩溃恢复 = 续跑）+ 审计（状态迁移记录写入 RGS-BAS-003 §7 既有审计留痕存储）**的运行时可观测字段——审计写、HA 恢复、审计写失败均有 release 必出事件。事件名统一 `deploy.audit.*` / `deploy.ha.*` 前缀。**审计写**是关键合规事件（per RGS-BAS-003 §7 + ARC-020 + §5 反模式"日志先明文记录…"），必须 release 必出 + §6.2 强制全采样；**审计写失败**是 P0 事件（per RGS-BAS-003 §7.1 "审计写失败触发 P0 告警 + 禁止降级通过"），必须 `error!` 强制全采样；HA 恢复（进程崩溃后从状态表恢复）走 release 必出。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.audit.state_change_recorded` | 单条状态迁移记录成功写入既有审计留痕存储（per §9 + RGS-BAS-003 §7） | 稳态 1/s、峰值 50/s（与 §4.1 `deploy.app.state_transition` 同频） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2，**部署任务执行关键事件**） | 含 `run_id`／`app_id`／`from_state`／`to_state`／`actor`／`recorded_at`；约 320B/条 |
| `deploy.audit.audit_write_failed` | 审计记录写入失败（DB 不可达 / Schema 不匹配 / RBAC 拒绝） | 极低 | release 必出（`error!` §6.2 强制全采样，per RGS-BAS-003 §7.1 "P0 告警 + 禁止降级" + BAS-004 v0.3 §6.2） | 含 `run_id`／`app_id`／`error_kind`／`error`；约 320B/条；**绝不吞错降级通过** |
| `deploy.audit.event_type_registered` | `cluster_deploy_state_change` 事件类型在 RGS-BAS-003 §7 审计 schema 中注册确认 | 启动时一次 | release 必出（`info!` 强制全采样，**审计 schema 一致性**） | 含 `event_type`／`schema_version`；约 200B/条 |
| `deploy.ha.process_started` | 编排 CLI / 服务进程启动 | 进程启动频次 | release 必出（`info!` 强制全采样，**HA 生命周期**） | 含 `process_id`／`cli_version`／`pg_version`；约 240B/条 |
| `deploy.ha.process_restarted` | 进程崩溃后由监督者重启（per §9 "进程崩溃后重启可从状态表恢复"） | 偶发（仅崩溃时） | release 必出（`warn!` 强制全采样，**HA 关键事件**） | 含 `process_id`／`restart_count`／`previous_exit_reason`；约 280B/条 |
| `deploy.ha.run_recovered` | 进程重启后从 PostgreSQL 状态表恢复进行中 / 暂停的 run（per §9 "等价于对同一 `run_id` 的续跑"） | 偶发（仅崩溃时） | release 必出（`info!` 强制全采样，**HA + 续跑联动**） | 含 `recovered_run_id`／`previous_state_summary`／`recovered_app_count`；约 320B/条 |
| `deploy.ha.state_table_inconsistent` | 进程重启时检测到状态表内部不一致（如 SUCCEEDED → PENDING 状态回退） | 极低（DB 损坏 / 误操作） | release 必出（`error!` 强制全采样） | 含 `run_id`／`inconsistency_kind`／`affected_apps[]`；约 380B/条 |
| `deploy.ha.pg_connection_lost` | 与既有 PostgreSQL HA 实例（同步复制+Multi-AZ）连接断开 | 极低 | release 必出（`error!` 强制全采样） | 含 `pg_endpoint`／`disconnect_at`；约 240B/条；**绝不回显密码 / 连接串** |
| `deploy.ha.pg_connection_recovered` | 与 PostgreSQL HA 实例连接恢复 | 极低 | release 必出（`warn!` 强制全采样） | 含 `pg_endpoint`／`disconnect_duration_ms`；约 240B/条 |
| `deploy.audit.debug.audit_record_dump` | 完整审计记录 dump（含 `app_id` / `from_state` / `to_state` / `actor` / `recorded_at` / `payload`） | 极低（合规审计导出） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-3KB/条（payload 决定，release 剔除零运行时开销） |
| `deploy.ha.debug.state_table_dump` | 启动时恢复的完整状态表 dump（含每 App 当前状态 + 状态变更历史） | 极低（HA 恢复失败复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-10KB/条（App 数 + 历史决定，release 剔除） |
| `deploy.ha.debug.pg_connection_dump` | 完整 PostgreSQL 连接串 dump（用于复盘连接问题） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护；**绝不**含明文密码） | 约 0.2-1KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §5.1 脱敏黑名单 + RGS-BAS-003 §7 + ARC-020 + §5 反模式）：
- `deploy.audit.debug.audit_record_dump` 中的 `payload` 字段**可能含触发者 token**（per §9 "每条状态迁移记录...触发者"）—— 必须在 dump 前通过独立 `redact_audit_payload()` 函数过滤（per §5.1 黑名单 + ARC-020 + §5 反模式"日志先明文记录…" 同类禁止）
- `deploy.ha.debug.pg_connection_dump` **绝不**含明文密码 —— 通过独立 `redact_pg_endpoint()` 函数将 `password=<...>` 部分过滤为 `password=***`，per §5.1 脱敏 + ARC-020 + §5 反模式
- `deploy.audit.audit_write_failed` 是 **P0 告警事件**（per RGS-BAS-003 §7.1 "审计写失败触发 P0 告警 + 禁止降级通过"），必须 `error!` 强制全采样 + 含 `trace_id` 便于跨服务追踪 —— **绝不**降级通过（per RGS-BAS-003 §7.1 同类纪律）
- `deploy.ha.run_recovered` 是**HA + 续跑联动**的审计依据（per §5 幂等性 + §9 HA 主条款），用于 SRE 复盘"为何某 run 跨进程重启后继续运行"
- `deploy.ha.pg_connection_lost` / `recovered` 是 PostgreSQL HA 实例（同步复制+Multi-AZ）连接生命周期的关键事件，per §9 "不引入新HA机制" 边界 —— 复用既有 HA 实例的连接事件
- `deploy.audit.event_type_registered` 是**审计 schema 一致性**的强约束（per §9 "不新建独立审计表，仅新增一个事件类型`cluster_deploy_state_change`"）—— 启动时验证 schema 注册避免后续 §4.1 `deploy.audit.state_change_recorded` 失败

# 9A. 部署时长基准（NFR-DEP-003落地）

> **背景（补齐设计缺口）**：NFR-DEP-003要求"从空集群到全部App部署完成（生产规模，T2档）的目标时长须在本文档给出具体基准并可度量"，此前本文档只有依赖图执行机制，未给出任何时长数字。以下为**设计阶段估算值**，不是实测值——依附件D GOV-OLU-004同类纪律，须在PH-4结合真实集群实测数据校准，估算本身不构成验收硬指标，仅作为编排设计的目标参照。

**估算依据**：

| 参数 | 取值 | 依据 |
|---|---|---|
| App总数（T2档，附件C§7当前登记域数） | 23 | 附件C§7域注册表 |
| 依赖图层级数（估算） | 4（基础设施层→依赖基础设施的一级业务域→二级依赖→三级依赖） | 参照§7挂载脚手架的典型依赖深度（网关/事件总线/配置中心为L0，多数业务域依附L0即完成，少数如DEP自身依附业务域） |
| 单App部署耗时（Helm Release安装+健康检查通过，P50） | 90秒 | 类比既有RGS-BAS-002§9挂载检查清单中单App验收所需的健康检查等待窗口 |
| 单App部署耗时（P99，含1次重试） | 240秒 | FR-DEP-009重试机制触发时的耗时上界 |

**目标时长**：同层级内App**并行**部署（§3.3拓扑排序设计），故总时长≈层级数×单层最长P99耗时，而非App总数×单App耗时的线性累加：

- **目标（P50口径）**：4层 × 90秒 ≈ **6分钟**
- **目标（P99口径，含部分App触发重试）**：4层 × 240秒 ≈ **16分钟**
- **可度量的验收基准**：`deploy-cluster apply`命令从发起到全部App状态转为`SUCCEEDED`的端到端耗时，P50 **不超过10分钟**，P99 **不超过20分钟**（在此基础上留有余量，而非贴着估算值设置阈值）

**局限声明（避免误导）**：本估算**不包含**RSK-DEP-002已登记的灾备重建场景下基础设施层App自身状态恢复（如数据库快照恢复）耗时——那部分耗时可能远超本节估算，须单独测算，不得与本节的"应用部署"时长混合统计造成误导。

### 9A.1 本功能日志设计

本节覆盖**NFR-DEP-003 部署时长基准（端到端耗时 P50 ≤ 10 分钟 / P99 ≤ 20 分钟）的运行时度量**的可观测字段——端到端 run 耗时、每层耗时、超阈值告警均有 release 必出事件。事件名统一 `deploy.duration.*` 前缀。**部署时长是性能基准**（per NFR-DEP-003 + NFR-PE-008 性能监控 + §9A 估算依据），必须 release 必出供 SRE 监控 + PH-4 实测校准；**P50/P99 超阈值**走 `warn!`/`error!` 强制全采样（per §9A "可度量的验收基准" + NFR-OP-008 SLA 保障）；逐层级逐 App timing 走 `debug!` 守护（高吞吐性能数据）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.duration.run_started` | `deploy-cluster apply` 命令发起（端到端计时起点） | 部署命令触发频次（典型 0.1-1/h） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2，**性能基准起点**） | 含 `run_id`／`cluster_id`／`started_at`；约 280B/条 |
| `deploy.duration.run_completed` | 全部 App 状态转为 SUCCEEDED（端到端计时终点） | 同上 | release 必出（`info!` 强制全采样，**算法性能基准**，per NFR-DEP-003 + NFR-PE-008） | 含 `run_id`／`total_elapsed_ms`／`p50_target_ms`／`p99_target_ms`；约 320B/条 |
| `deploy.duration.level_elapsed_ms` | 每个 level 完成时记录（per §3.3 同层并行 + 跨层串行） | 同上 | release 必出（`info!` 强制全采样，**算法性能基准**） | 含 `run_id`／`level_index`／`level_elapsed_ms`／`app_count`；约 280B/条 × 4 层 = 1.1KB/部署 |
| `deploy.duration.app_elapsed_ms` | 单 App 部署耗时（从 RUNNING → SUCCEEDED） | 稳态 1/s、峰值 50/s | release 必出（`info!` 强制全采样，**算法性能基准**） | 含 `run_id`／`app_id`／`elapsed_ms_bucket`（p50/p99 分桶）；约 240B/条 × 23 = 5.5KB/部署 |
| `deploy.duration.p50_exceeded` | 端到端耗时超过 P50 目标（10 分钟） | 偶发 | release 必出（`warn!` 强制全采样，per §9A "可度量的验收基准"） | 含 `run_id`／`total_elapsed_ms`／`p50_target_ms`／`overshoot_ms`；约 320B/条 |
| `deploy.duration.p99_exceeded` | 端到端耗时超过 P99 目标（20 分钟） | 极低 | release 必出（`error!` §6.2 强制全采样，per §9A + NFR-OP-008 SLA） | 含 `run_id`／`total_elapsed_ms`／`p99_target_ms`／`overshoot_ms`／`stuck_level`；约 360B/条 |
| `deploy.duration.p99_estimation_calibrated` | PH-4 实测数据校准 P50/P99 估算值（per §9A "须在PH-4结合真实集群实测数据校准"） | 极低（PH-4 阶段） | release 必出（`info!` 强制全采样，**PH-4 校准节点**） | 含 `old_p50_ms`／`new_p50_ms`／`old_p99_ms`／`new_p99_ms`／`sample_size`；约 360B/条 |
| `deploy.duration.disaster_recovery_excluded` | 显式声明 RSK-DEP-002 灾备重建场景时长**不计入**本节度量（per §9A "局限声明"） | 极低（仅灾备场景） | release 必出（`info!` 强制全采样，**避免误统计审计**） | 含 `run_id`／`dr_kind`／`reason`；约 240B/条 |
| `deploy.duration.debug.per_level_timing` | 每层完整 timing dump（含每 App 起始/结束时间戳） | 偶发（PH-4 校准 + 故障复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-5KB/条（App 数决定，release 剔除零运行时开销） |
| `deploy.duration.debug.per_app_timing_detail` | 单 App 详细 timing dump（Helm Release / health check / state transition 各阶段耗时） | 偶发（性能异常复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-3KB/条（release 剔除） |
| `deploy.duration.debug.historical_distribution` | 历史 P50/P99 分布 dump（用于 §9A 估算漂移分析） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 5-20KB/条（历史样本数决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + NFR-DEP-003 + §9A 估算纪律 + NFR-PE-008 性能监控）：
- `deploy.duration.run_completed` / `deploy.duration.level_elapsed_ms` / `deploy.duration.app_elapsed_ms` 是 **NFR-PE-008 性能监控的源头数据** —— **必须** release 必出（`info!` 强制全采样），不进入 `debug!` 守护
- `deploy.duration.p99_exceeded` 是 **NFR-OP-008 SLA 保障**的告警事件（per §9A "P99 不超过 20 分钟"），必须 `error!` 强制全采样 + 含 `stuck_level` 便于定位超时发生在哪一层
- `deploy.duration.disaster_recovery_excluded` 是**避免误统计审计**的强约束（per §9A "局限声明" —— RSK-DEP-002 灾备重建耗时须单独测算不得混合统计），必须 release 必出供 SRE 区分"应用部署时长" vs "灾备恢复时长"
- `deploy.duration.debug.historical_distribution` 在 PH-4 阶段累积样本后可达 20KB+ —— `#[cfg(debug_assertions)]` 守护避免 release 误开 RUST_LOG=debug 时撑爆生产日志
- `deploy.duration.p99_estimation_calibrated` 是 **PH-4 实测校准的关键节点**（per §9A "须在PH-4结合真实集群实测数据校准，估算本身不构成验收硬指标" + GOV-OLU-004 同类纪律），用于更新 P50/P99 估算值
- `deploy.duration.*_elapsed_ms` 系列累积字节数显著（4 层 + 23 App + 1 run 终止 ≈ 7KB/部署 + ~30 条），但仍 release 必出 —— **完整性能轨迹**是 PH-4 校准 + NFR-PE-008 监控的唯一可信来源

# 10. 选型建议（回应TBD-DEP-001）

建议采用 **Rust CLI工具 + 现有CI平台Pipeline调用** 的组合，而非二选一：

- CLI工具（Rust编写，与本仓库技术栈一致）承载§3依赖图算法、§4状态机逻辑、§8命令行接口，可在本地或CI runner中执行，逻辑集中、可单元测试
- CLI工具**不直接持有**CI平台的凭据/触发权限，实际的Helm Release调用通过CI平台既有的部署Job/Pipeline触发（CLI生成参数化的Pipeline触发请求），复用既有CI/CD平台的权限模型与审批门禁，避免编排层绕过既有变更管控

此建议在RGS-REQ-027 TBD-DEP-001基础上给出方向性结论，具体CI平台绑定细节（如触发API、凭据范围）留待实施阶段的技术评审最终确定，TBD-DEP-001状态标记为"部分决议"（方向已定，实现细节待定）。

### 10.1 本功能日志设计

本节覆盖**TBD-DEP-001 选型建议（Rust CLI 工具 + 现有 CI 平台 Pipeline 调用）的运行时可观测字段**——CLI 编译产物、CI 平台绑定、Pipeline 触发均有 release 必出事件。事件名统一 `deploy.tooling.*` 前缀。**选型落地**（CLI 编译 + CI 平台绑定）是 §10 主条款的具体实现，per §6.2 强制全采样；**Pipeline 触发**是关键合规节点（per §10 "CLI工具不直接持有CI平台的凭据/触发权限"），必须 release 必出供 SRE 区分本地 vs CI 触发路径；编译选项 + CI 平台绑定细节走 `debug!` 守护（**可能含 CI 平台 token**）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `deploy.tooling.cli_compiled` | `deploy-cluster` Rust CLI 编译完成（含 cargo build 成功） | 实施阶段一次 + 升级时 | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `cli_version`／`rustc_version`／`target_triple`；约 280B/条 |
| `deploy.tooling.ci_platform_bound` | CLI 与具体 CI 平台（GitHub Actions / GitLab CI / Jenkins）绑定完成 | 实施阶段一次 + 切换时 | release 必出（`info!` 强制全采样，**CI/CD 平台绑定审计**） | 含 `ci_platform_kind`／`api_version`；约 240B/条；**绝不回显 API token** |
| `deploy.tooling.ci_pipeline_invocation_triggered` | CLI 通过 CI 平台 Pipeline 触发请求发起（per §10 "CLI生成参数化的Pipeline触发请求"） | CI 触发频次 | release 必出（`info!` 强制全采样，**部署任务执行入口**） | 含 `pipeline_name`／`pipeline_run_id`／`parameterized_args_hash`；约 320B/条 |
| `deploy.tooling.ci_pipeline_trigger_rejected` | CI 平台拒绝触发请求（API 限流 / 权限不足 / Pipeline 不存在） | 极低 | release 必出（`error!` §6.2 强制全采样，per BAS-004 v0.3 §6.2） | 含 `ci_platform_kind`／`rejection_kind`／`error`；约 320B/条；**绝不回显 API token** |
| `deploy.tooling.tbd_dep_001_status_updated` | TBD-DEP-001 状态从"待定"→"部分决议"或"完全决议"更新（per §10 主条款 "TBD-DEP-001状态标记为'部分决议'"） | 极低（决议变更时） | release 必出（`info!` 强制全采样，**选型决议审计**） | 含 `old_status`／`new_status`／`resolution_date`；约 240B/条 |
| `deploy.tooling.local_build_detected` | 检测到 CLI 在本地环境编译运行（不经 CI 平台），符合 §10 选型"CLI工具可在本地或CI runner中执行" | 偶发（开发者本地） | release 必出（`info!` 强制全采样，**本地 vs CI 边界审计**，per §10 "可在本地或CI runner中执行"） | 含 `hostname`／`user`／`build_kind`（cargo install / cargo build）；约 240B/条 |
| `deploy.tooling.rbac_role_for_orchestration_defined` | 编排层所需 CI 平台 RBAC 角色定义完成（per §10 "复用既有CI/CD平台的权限模型与审批门禁"） | 实施阶段一次 | release 必出（`info!` 强制全采样，**RBAC 角色定义审计**） | 含 `role_name`／`scope`；约 240B/条；**绝不回显角色绑定 token** |
| `deploy.tooling.debug.build_options_dump` | cargo build 完整选项 dump（含 features / target / profile） | 极低（编译失败复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-2KB/条（release 剔除零运行时开销） |
| `deploy.tooling.debug.ci_platform_binding_dump` | CI 平台绑定详情 dump（含 endpoint / API version / scope） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护；**绝不**含明文 API token） | 约 0.2-1KB/条（release 剔除） |
| `deploy.tooling.debug.pipeline_parameter_dump` | Pipeline 触发参数完整 dump（含所有 --flag） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护；**绝不**含明文 secret） | 约 0.5-3KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §5.1 脱敏黑名单 + §10 选型建议 + ARC-020 + §5 反模式）：
- `deploy.tooling.debug.ci_platform_binding_dump` / `deploy.tooling.debug.pipeline_parameter_dump` 中的 `api_token` / `pipeline_secret` / `vault_token` 等敏感字段**必须**在 dump 前通过独立 `redact_ci_binding()` / `redact_pipeline_params()` 函数过滤（per §5.1 黑名单 + ARC-020 + §5 反模式"日志先明文记录…" 同类禁止），将 `token=<...>` 部分过滤为 `token=***`
- `deploy.tooling.ci_pipeline_invocation_triggered` 的 `parameterized_args_hash` 字段使用 SHA256 摘要**而非**明文 args（per §5.1 脱敏 + §8.1 `deploy.cli.subcommand_invoked` 同类纪律），既保留"本次触发使用了哪些参数"的审计能力，又避免明文回显
- `deploy.tooling.tbd_dep_001_status_updated` 是**选型决议审计**的源头（per §10 "TBD-DEP-001状态标记为'部分决议'" + 用户决策轨迹），用于追溯"Rust CLI + CI Pipeline"选型何时落地
- `deploy.tooling.local_build_detected` 是**本地 vs CI 边界审计**的依据（per §10 "CLI工具不直接持有CI平台的凭据/触发权限" + §8.1 `deploy.cli.local_invocation` 联动）—— 本地构建不经 CI 平台权限模型，必须 release 必出供合规审计
- `deploy.tooling.*_rejected` 错误事件**绝不**回显 CI 平台 API token（per §5.1 脱敏 + §8.1 `deploy.cli.subcommand_invalid_args` 同类纪律），错误信息中 token 字段过滤为 `***`

---

# 11. 追溯性（Traceability）

本节给出本文档"本功能日志设计"v0.3 新增内容与 BAS-001 v1.5 §4.8.3 模板 + BAS-003 v0.3 样板 + BAS-004 v0.3 §4/§5/§6 总则的追溯关系，以及新引入验收标准（AC-DEP-005/006）与本文档章节的映射。

## 11.1 验收标准（新增）

| 验收标准 | 落实段落 | 关联需求/标准 |
|---|---|---|
| **AC-DEP-005（`deploy.*` debug-only 宏在 release build 完全剔除）** | §2.1/§3.1.1/§3.2.1/§3.3.1/§4.1/§5.1/§6.1.1/§6.2.1/§7.1/§8.1/§9.1/§9A.1/§10.1 各节"debug-only 守护要点"项 + RGS-BAS-004 v0.3 §4.3 四铁律 + §4.4 编译期守护 | §2-§10 全部新 log 小节 + FR-LOG-012 + NFR-OP-008 |
| **AC-DEP-006（每功能 BAS 文档须含本功能 log 章节）** | §2.1/§3.1.1/§3.2.1/§3.3.1/§4.1/§5.1/§6.1.1/§6.2.1/§7.1/§8.1/§9.1/§9A.1/§10.1 共 13 个"本功能日志设计"小节 | FR-LOG-010/011/012 + §1 总要求 + BAS-001 v1.5 §4.8.3.4 统一规范 |

## 11.2 模板与样板引用

| 来源 | 引用段落 | 用途 |
|---|---|---|
| BAS-001 v1.5 §4.8.3 体系级日志章节约定（commit 32d9eb6） | §4.8.3.1 5 列详尽版模板 + §4.8.3.2 二维矩阵 + §4.8.3.3 引用规范 | 5 列详尽版写作模板（字段名/触发条件/频率估算/采样策略/脱敏与成本） |
| BAS-003 v0.3 样板（commit 75a001c） | §3.1.1/§3.2.1/§3.3.1/§3.4.1/§4.5/§5.1/§6.3/§7.1/§8.3/§9.1/§10.1 共 11 个"本功能日志设计"小节 | 运维域样板参考（事件名/触发条件/宏调用/类别/字段最小集/频率上限/性能预算 7 列格式），本文档采用 BAS-001 v1.5 5 列统一格式（与 BAS-002/003/010 对齐） |
| BAS-004 v0.3 §4.2 二维矩阵（commit 47e26b0） | 编译期×运行时二维矩阵 | 显式区分 `info!`/`warn!`/`error!`（release 必出，编译期常驻，§6.2 强制全采样）与 `trace!`/`debug!`（`#[cfg(debug_assertions)]` 守护，debug-only，release 完全剔除） |
| BAS-004 v0.3 §4.3 字段规范 | snake_case 拼写 + 关联 ID 预先 let 绑定 | 字段命名规范（本文档 `deploy.*` 前缀与 BAS-001 `log.*` / BAS-002 `mnt.*` / BAS-003 `ops.*` / BAS-010 `pat.*` 区分） |
| BAS-004 v0.3 §5.1 脱敏规则 | `*token*` / `*password*` 黑名单自动丢弃 | 部署配置变更 + 镜像拉取凭据 + CI 平台 API token + K8s 连接串的脱敏 |
| BAS-004 v0.3 §6.2 强制全采样白名单 | 部署任务执行（启动/暂停/恢复/回滚）+ 部署配置变更 + 部署失败/超时 + 灰度/金丝雀发布 全部强制全采样 | 与用户特别注明的"部署域特殊考虑"完全对齐 |

## 11.3 追溯依据（需求/风险/NFR 覆盖）

| 类型 | 编号 | 涉及 §X.1 小节 |
|---|---|---|
| 架构约束 | ARC-018 挂载脚手架 | §7.1（强制联动 CI 校验） |
| 架构约束 | ARC-042 DEP 域架构 | §2.1/§3.1.1/§4.1/§6.1.1/§6.2.1/§8.1/§10.1（编排 CLI 工具） |
| 架构约束 | ARC-009 状态机 + OCC/Outbox | §4.1/§5.1（编排状态机 + 幂等性续跑） |
| 架构约束 | ARC-020 Vault 加密 + Append-Only Ledger | §9.1（审计写 + 加密 Vault 访问审计） |
| 架构约束 | RGS-BAS-003 §7 审计留痕存储 | §9.1（`cluster_deploy_state_change` 事件类型） |
| 架构约束 | RGS-BAS-022 容量分档 T0-T3 | §2.1（`capacity_tier` 字段校验） |
| 需求 | FR-DEP-001 镜像版本校验 | §2.1（`deploy.manifest.parsed` + §7.1） |
| 需求 | FR-DEP-002 无环校验 | §3.2.1（`deploy.validate.cycle_detected`） |
| 需求 | FR-DEP-004 基础设施前置校验 | §3.2.1（`deploy.validate.foundation_missing`） |
| 需求 | FR-DEP-007 灰度/金丝雀发布 | §8.1（`deploy.cli.subcommand_invoked` 参数化 args） |
| 需求 | FR-DEP-009 重试机制 | §4.1（`deploy.app.attempt_started` + `deploy.app.retry_exhausted`） |
| 需求 | FR-LOG-010/011/012/013/040 日志规范 | §2-§10 全部新 log 小节（编译期常驻 + 字段命名 + debug-only 守护 + 全采样） |
| 风险 | RSK-DEP-001 集群清单被遗漏 | §2.1/§7.1（双层防护：CI 拦截 `deploy.ci.missing_app_detected` + 运行时拦截 `deploy.validate.cycle_detected`） |
| 风险 | RSK-DEP-002 灾备重建基础设施恢复 | §9A.1（`deploy.duration.disaster_recovery_excluded` 避免误统计） |
| NFR | NFR-DEP-001 部署可重复性 | §5.1（`deploy.idem.*` 幂等性系列） |
| NFR | NFR-DEP-002 部署可回滚 | §6.2.1（`deploy.rollback.*` 全系列） |
| NFR | NFR-DEP-003 部署时长可度量 | §9A.1（`deploy.duration.*` 全系列 + P50/P99 估算与校准） |
| NFR | NFR-PE-008 性能监控 | §4.1/§5.1/§9A.1（`deploy.app.*_elapsed_ms` / `deploy.duration.*_elapsed_ms` 算法性能基准） |
| NFR | NFR-OP-008 排查 SLA 保障 | §3.2.1/§4.1/§6.2.1/§8.1/§9.1（`error!` §6.2 强制全采样 + 含 `trace_id`） |
| NFR | NFR-OP-010 资源约束 | §10.1（CLI 编译产物大小 + CI 平台 RBAC 角色定义审计） |

## 11.4 已知缺口

- **§6.2 强制全采样的"灰度/金丝雀发布"具体事件名未单列** —— 灰度发布在本文档未独立成章（per §10 选型建议 "CLI生成参数化的Pipeline触发请求"），实际灰度策略在 ARC-018 挂载脚手架的 chart values 中实现；待 ARC-018 灰度发布细节明确后，在 §8.1 `deploy.cli.subcommand_invoked` 字段中追加 `rollout_strategy` 子字段（per BAS-004 v0.3 §4.3.2 业务扩展字段规范），建议在 PH-4 实测阶段补齐
- **灾备重建（DR）场景的 run 日志未单列** —— RSK-DEP-002 灾备重建基础设施恢复的日志字段未在本文档展开（per §9A "局限声明"），需 PH-4 灾备演练时根据实测补充
- **§10.1 选型落地的"完全决议"状态未触发** —— 当前 TBD-DEP-001 状态为"部分决议"（per §10 主条款），具体 CI 平台绑定细节待实施阶段技术评审，`deploy.tooling.tbd_dep_001_status_updated` 事件待完全决议时触发

