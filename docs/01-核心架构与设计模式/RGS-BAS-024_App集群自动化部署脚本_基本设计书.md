# 基本设计书（基本設計書 / Basic Design Document）

**App集群自动化部署脚本 Automated Cluster Deployment Scripts for Atomic Apps**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-024 |
| 版本 | 0.2 |
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

# 3. 依赖图构建与校验

## 3.1 构建

编排层读取清单中全部`app_id`与`depends_on`，构建有向图：`app_id → depends_on中的每个app_id`的边表示"前者依赖后者"。

## 3.2 校验规则（对应FR-DEP-002/FR-DEP-004）

1. **无环校验**：对构建的图执行环检测（如Tarjan强连通分量或简单DFS染色法）；发现环则报告完整环路径（如`TRD → EVT → TRD`），编排运行在执行前以非零退出码终止，不发起任何部署
2. **基础设施前置校验**：预置的基础设施层App集合（GW/EVT/CFG/可观测性基座/密钥管理，与`tier: foundation`标记的App集合比对）须满足：每个`tier: domain`的App，其`depends_on`的传递闭包必须包含全部`tier: foundation`的App；不满足则清单不合法
3. **孤儿引用校验**：`depends_on`中引用的`app_id`必须存在于清单的`apps`列表中，否则报错

## 3.3 拓扑排序

使用Kahn算法对DAG做拓扑排序，产出"执行层级"列表（`level_0`：入度为0的App，即基础设施层；`level_1`：仅依赖level_0的App；依此类推）。同一层级内的App在执行阶段并行处理，跨层级严格串行等待上一层级全部成功。

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

# 5. 幂等性设计

- 部署步骤本身的幂等性完全依赖底层Helm Release的声明式特性（`helm upgrade --install`语义），编排层不引入任何自定义的"先删后建"类非幂等操作
- 续跑（对已存在的`run_id`重新调用编排）时，编排层先读取状态表，仅对`PENDING`/`FAILED`/`BLOCKED`状态的App重新计算是否可执行，`SUCCEEDED`的App直接跳过，不重复调用其Helm Release（即使重复调用本身也是幂等的，跳过只是为了缩短续跑时长）
- 健康检查（判定`RUNNING → SUCCEEDED`）复用该App既有的readiness探针/健康检查端点（RGS-BAS-002脚手架检查清单已要求各App提供），编排层不重新定义健康判定逻辑

# 6. Dry-run与回滚

## 6.1 Dry-run

Dry-run模式执行§3的构建与校验、§3.3的拓扑排序，输出：各层级顺序、每个App当前版本与`target_version`的diff（通过既有Helm的`--dry-run`/`helm diff`插件获取，不新增diff引擎），但不进入§4状态机的`RUNNING`阶段，不产生任何实际部署副作用。

## 6.2 回滚

给定一个`run_id`，回滚流程：

1. 取该run中状态为`SUCCEEDED`的App列表
2. 按依赖图的**逆拓扑序**排列（业务域App在前，基础设施层App在后）
3. 对每个App调用其Helm Release的回滚能力（`helm rollback`到该App在本次run执行前的revision），逐个执行并等待成功后再回滚下一个
4. 全部完成后，该run下涉及的App状态迁移为`ROLLED_BACK`，写入审计记录

回滚顺序刻意与部署顺序相反：避免在业务域App仍在运行、仍依赖某基础设施层App时，过早把该基础设施层App回滚掉。

# 7. 与ARC-018挂载脚手架的强制联动

为解决RGS-REQ-027 RSK-DEP-001（新增域App时集群清单可能被遗漏），在RGS-BAS-002的挂载脚手架检查清单中追加一项强制步骤：

> **【新增检查项】新App挂载完成前，必须在`cluster-manifest.yaml`中登记该App的`app_id`/`depends_on`/`scaffold_ref`/`capacity_tier`，且CI须运行本文档§3.2的依赖图校验通过，方可视为挂载完成。**

该检查项通过`scripts/check-docs-consistency.sh`同类思路新增一条独立CI校验脚本（`scripts/check-cluster-manifest.sh`，本文档仅设计其校验规则，具体实现留待实施阶段）：解析`cluster-manifest.yaml`，比对附件C§7域注册表中的全部已登记域，若存在已注册域但未出现在清单`apps`列表中的情况，CI失败并报告缺失的`app_id`。

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

# 9. 高可用与审计

- 编排CLI/服务进程本身无状态，状态表持久化于PostgreSQL（复用既有生产实例的HA配置：同步复制+Multi-AZ，不引入新HA机制），进程崩溃后重启可从状态表恢复继续（等价于对同一`run_id`的续跑）
- 每条状态迁移记录（App、旧状态、新状态、时间、触发者）写入既有审计留痕存储（RGS-BAS-003§7），不新建独立审计表，仅新增一个事件类型`cluster_deploy_state_change`

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

# 10. 选型建议（回应TBD-DEP-001）

建议采用 **Rust CLI工具 + 现有CI平台Pipeline调用** 的组合，而非二选一：

- CLI工具（Rust编写，与本仓库技术栈一致）承载§3依赖图算法、§4状态机逻辑、§8命令行接口，可在本地或CI runner中执行，逻辑集中、可单元测试
- CLI工具**不直接持有**CI平台的凭据/触发权限，实际的Helm Release调用通过CI平台既有的部署Job/Pipeline触发（CLI生成参数化的Pipeline触发请求），复用既有CI/CD平台的权限模型与审批门禁，避免编排层绕过既有变更管控

此建议在RGS-REQ-027 TBD-DEP-001基础上给出方向性结论，具体CI平台绑定细节（如触发API、凭据范围）留待实施阶段的技术评审最终确定，TBD-DEP-001状态标记为"部分决议"（方向已定，实现细节待定）。
