# 详细设计书（詳細設計書 / Detailed Design Document）

**GM后台拓扑可视化无限画布：视图配置物理数据库设计・聚合服务查询协议格式・颗粒度聚合与缓存算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-021 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-021 GM后台拓扑可视化——无限画布 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定，本文档是RGS-DTL-001/002/025/026/027之后本批次继续推进详细设计阶段的一部分。细化RGS-BAS-021§6.1/§6.3 `ViewPreset`/`UserViewPreference`逻辑字段为具体DDL、§2.2查询模式与快照缓存落实为可直接翻译为Rust实现的伪代码、§3三级颗粒度数据映射与§4边构造落实为具体聚合查询协议格式、§5.2版本新鲜度标注落实为响应字段。**本版本不覆盖**：画布前端渲染库本身的选型代码（TBD-VIZ-001）、节点自动分组/聚类算法（TBD-VIZ-002）。见§7 | 全部 |
| 0.2 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | **同步父 BAS-021 升版至 v0.2**（1 次升版，BAS-021 v0.2 装饰性升版）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-021 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 聚合查询协议是否切实只读，快照缓存失效策略是否满足NFR-VIZ-002/003滞后与高频交互约束 |
| 评审（安全） | | | 个人偏好过滤是否在物理查询层面无法绕过`default_visible_roles`角色过滤（§6.3边界条件） |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：视图配置与个人偏好](#2-物理数据库设计视图配置与个人偏好)
3. [拓扑聚合查询协议格式](#3-拓扑聚合查询协议格式)
4. [三级颗粒度聚合与快照缓存算法详细设计](#4-三级颗粒度聚合与快照缓存算法详细设计)
5. [视图过滤与角色可见性合成算法详细设计](#5-视图过滤与角色可见性合成算法详细设计)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)
7. [追溯性](#7-追溯性)

---

# 1. 前言

## 1.1 定位

RGS-BAS-021给出了拓扑聚合服务的组件图、三级颗粒度数据映射表、边分类规则、LangGraph可视化映射、`ViewPreset`/`UserViewPreference`逻辑字段表。本文档将其落实为可执行DDL、聚合服务对前端画布提供的查询协议格式、颗粒度聚合与快照缓存的算法级伪代码、个人偏好与角色可见性合成的具体判定逻辑。

## 1.2 本文档不做什么

- 不重新决定RGS-BAS-021已确定的任何结构性选择（拓扑聚合服务依附AD限界上下文不新建独立限界上下文、聚合适配层仅只读权限、大范围查询优先走分析管线读端点、个人偏好不得绕过角色可见性限制）。
- 不覆盖画布前端渲染库本身的选型与集成代码（TBD-VIZ-001）——该选型待CON-001评审完成后确定，本文档的协议格式设计对具体前端库保持中立。
- 不覆盖节点自动分组/聚类算法（TBD-VIZ-002）——RGS-BAS-021§7已明确"留待详细设计阶段确定"但同时"本文档不预先约定具体聚类算法"，本文档遵循该既定范围声明，不越权在此展开。

## 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，聚合服务对前端的查询协议以HTTP+JSON风格给出（画布前端为浏览器环境，非既有内部gRPC路径，同RGS-DTL-027§3清单查询协议的选型理由），算法伪代码可直接对应Rust `Result`实现。

---

# 2. 物理数据库设计：视图配置与个人偏好

对应RGS-BAS-021§6.1/§6.3。依附既有GM后台AD限界上下文数据库，不新建独立库。

```sql
-- 全局声明式视图预设，对应§6.1，管理员/架构师维护
CREATE TABLE view_presets (
    view_id                TEXT PRIMARY KEY,
    display_name             TEXT NOT NULL,
    default_granularity        TEXT NOT NULL CHECK (default_granularity IN ('app', 'plugin', 'method')),
    node_filter                  JSONB NOT NULL DEFAULT '{}',
    edge_filter                    JSONB NOT NULL DEFAULT '{}',
    color_scheme                     JSONB NOT NULL DEFAULT '{}',
    default_visible_roles              TEXT[] NOT NULL,   -- 复用RGS-BAS-003§8角色矩阵的角色标识
    updated_at                           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 个人自定义偏好，对应§6.3，与view_presets非同一存储对象(逻辑设计已确定，本处物理落实为独立表)
CREATE TABLE user_view_preferences (
    preference_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gm_user_id          BIGINT NOT NULL,   -- 逻辑引用RGS-BAS-003既有GM用户身份，跨限界上下文不建物理FK
    base_view_id           TEXT NOT NULL REFERENCES view_presets(view_id),
    custom_node_filter        JSONB NOT NULL DEFAULT '{}',
    custom_edge_filter          JSONB NOT NULL DEFAULT '{}',
    updated_at                    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_user_view_pref UNIQUE (gm_user_id, base_view_id)
    -- 每个GM用户对每个base_view_id至多一条个人偏好记录(覆盖式保存，非历史版本追加)
);
```

---

# 3. 拓扑聚合查询协议格式

对应RGS-BAS-021§2.1/§3/§5.2。前端画布向拓扑聚合服务发起的查询接口：

```
GET /v1/topology/snapshot?granularity={app|plugin|method}&view_id={V}&drill_from={NodeId?}
```

响应体（JSON，落实§3三级颗粒度数据映射与§5.2版本新鲜度标注）：

```json
{
  "granularity": "app",
  "snapshot_generated_at": "2026-08-17T09:00:00Z",
  "cache_lag_seconds": 12,
  "nodes": [
    {
      "node_id": "anticheat-service",
      "node_type": "app",
      "source": "mount_record",
      "metrics_overlay": { "qps": 120.5, "error_rate": 0.002, "p99_ms": 45 }
    }
  ],
  "edges": [
    {
      "from": "gateway-service",
      "to": "anticheat-service",
      "edge_type": "control_flow",
      "highlight_level": "normal"
    }
  ],
  "langgraph_overlays": [
    {
      "graph_definition_id": "anticheat-fusion",
      "version": 3,
      "confirmed_fresh": true
    }
  ]
}
```

- `cache_lag_seconds`：`snapshot_generated_at`距当前请求时刻的秒数，落实§5.2"显式标注...CACHE快照的生成时间"要求，前端据此可自行计算展示"数据新鲜度"，不要求聚合服务额外计算一个衍生字段。
- `langgraph_overlays[].confirmed_fresh`：聚合服务比对`snapshot_generated_at`与`AnalysisGraphDefinition.version`最近变更时间得出，`false`时前端**必须**在画布上显式标注"非最新版本"（落实§5.2"供安全评审人员判断当前所见是否为最新生效版本"）。

---

# 4. 三级颗粒度聚合与快照缓存算法详细设计

对应RGS-BAS-021§2.2/§3。

## 4.1 快照生成（周期性，不响应每次前端请求实时聚合）

```rust
fn refresh_topology_snapshot(granularity: Granularity) -> Result<(), SnapshotError> {
    let nodes = match granularity {
        Granularity::App => collect_from_mount_records(),      // 只读适配子模块，§2.2"仅具备只读查询权限"
        Granularity::Plugin => collect_from_plugin_registry(),
        Granularity::Method => collect_from_trace_spans(),      // 大范围查询优先走分析管线读端点(RGS-DTL-017 AnalyticsStore)
    };
    let edges = derive_edges_for_granularity(granularity, &nodes);  // §4.1边分类：控制流/数据流/LangGraph建议提交
    let metrics = fetch_golden_metrics_overlay(&nodes);              // 复用RGS-BAS-004黄金指标，§4.2"不得由前端直接连接指标系统"

    write_snapshot_cache(granularity, TopologySnapshot { nodes, edges, metrics, generated_at: now() })?;
    Ok(())
}
// 触发方式: 复用NFR-VIZ-002既定滞后阈值的周期性定时任务(与既有可观测性/分析管线周期性任务同一调度框架，
// 不新建独立调度机制)，具体周期秒数详细设计不预设固定值——不同颗粒度的刷新成本差异较大，
// 由运维按NFR-VIZ-002/003综合实测配置，本文档只固定"周期性生成、不逐请求触发"这一结构性约束
```

## 4.2 前端请求处理（读缓存，不触发实时聚合）

```rust
fn handle_snapshot_request(req: &SnapshotQuery) -> Result<TopologySnapshotResponse, QueryError> {
    let cached = read_snapshot_cache(req.granularity)
        .ok_or(QueryError::SnapshotNotYetGenerated)?;  // 冷启动边界：首次上线尚无缓存时的明确错误，而非阻塞等待生成

    let view = resolve_effective_view(req.gm_user_id, &req.view_id)?;  // §5合成算法
    let filtered = apply_view_filter(&cached, &view);

    if let Some(drill_from) = &req.drill_from {
        // §3"下钻操作携带来源节点上下文"：本函数只负责返回新颗粒度数据，
        // 锚点动画由前端消费drill_from自行处理，聚合服务不掺入渲染层决策
        return Ok(TopologySnapshotResponse::with_drill_anchor(filtered, drill_from.clone()));
    }
    Ok(TopologySnapshotResponse::plain(filtered))
}
```

**"不得每次平移/缩放触发新聚合查询"的落实位置**：本函数是前端在视口变化时调用的**唯一**数据获取入口，其内部只读`snapshot_cache`（§4.1周期性写入），不含任何对`mount_records`/`trace_spans`等原始数据源的直接查询路径——这一结构本身即保证了NFR-VIZ-003约束，而非依赖前端"克制调用频率"这一软约束。

---

# 5. 视图过滤与角色可见性合成算法详细设计

对应RGS-BAS-021§6.3，核心边界条件是"个人偏好不得绕过角色可见性限制"。

```rust
fn resolve_effective_view(gm_user_id: GmUserId, view_id: &str) -> Result<EffectiveView, ViewError> {
    let preset = load_view_preset(view_id).ok_or(ViewError::UnknownView)?;
    let user_role = load_gm_user_role(gm_user_id);  // 复用RGS-BAS-003§8既有角色矩阵

    if !preset.default_visible_roles.contains(&user_role) {
        // 用户角色本就不在该视图的default_visible_roles内: 直接拒绝，不进入个人偏好合成环节
        // (即便该用户此前保存过对应base_view_id的个人偏好，角色变更后也不得继续访问)
        return Err(ViewError::RoleNotPermitted);
    }

    let preference = load_user_view_preference(gm_user_id, view_id);
    let effective_filter = match preference {
        None => preset.node_filter.clone(),  // 无个人偏好: 回退至ViewPreset默认(§6.3)
        Some(pref) => {
            // 关键实现点: 个人偏好是在preset.node_filter基础上的"追加/覆盖"，
            // 语义为逻辑AND(收窄)而非替换——即便custom_node_filter本身试图放宽范围，
            // 最终生效范围仍是 preset.node_filter ∩ custom_node_filter，物理上不存在越过preset的路径
            intersect_filters(&preset.node_filter, &pref.custom_node_filter)
        }
    };

    Ok(EffectiveView {
        node_filter: effective_filter,
        edge_filter: preference.as_ref()
            .map(|p| intersect_filters(&preset.edge_filter, &p.custom_edge_filter))
            .unwrap_or_else(|| preset.edge_filter.clone()),
        color_scheme: preset.color_scheme.clone(),  // 着色规则不受个人偏好影响，仅过滤条件可自定义(§6.1/§6.3既定范围)
    })
}
```

**"不得借由个人偏好绕过角色可见性限制"的物理保证**：`intersect_filters`实现为**逻辑求交**而非"个人偏好覆盖预设"，即无论`custom_node_filter`内容如何，合成结果的可见节点集合恒为`preset`定义范围的子集——这是RGS-BAS-021§6.3该约束在算法层面唯一正确的实现方式（若实现为"存在个人偏好则完全采用个人偏好"，将产生可绕过角色过滤的路径，是本文档在细化过程中特别排除的错误实现）。

---

# 6. 本文档的覆盖范围与后续计划

本文档覆盖：`view_presets`/`user_view_preferences`两表物理DDL、拓扑聚合查询的HTTP+JSON协议格式（含版本新鲜度标注字段）、快照生成与前端请求处理的完整伪代码（周期性生成、请求只读缓存的结构性隔离）、个人偏好与角色可见性合成算法（逻辑求交而非覆盖，物理防止绕过角色限制）。

本版本明确不覆盖、留待后续：

- 画布前端渲染库本身的选型与集成代码（TBD-VIZ-001）——待CON-001许可评审完成后确定，本文档协议格式设计对具体前端库保持中立，不预设依赖。
- 节点自动分组/聚类算法（TBD-VIZ-002）——按RGS-BAS-021§7既定范围声明，本文档不预先约定具体聚类算法。
- §4.1快照刷新周期的具体秒数——由运维按NFR-VIZ-002/003综合实测配置，本文档只固定"周期性、非逐请求"的结构性约束。
- 三级颗粒度各自的聚合查询在走RGS-DTL-017分析管线读端点时的具体查询语句——依赖`AnalyticsStore`选型（TBD-INF-002）完成后才可具体化，本文档仅声明该读取路径的复用关系。

后续详细设计建议顺序：与RGS-DTL-017/018/020同批次并行推进；本文档§3/§4的方法级聚合路径依赖RGS-DTL-017分析管线元数据设计已在同批次完成，二者应视为互相引用的配套文档。

---

# 7. 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-021§2.1/§2.2 组件划分与查询模式 | §3、§4.1 |
| RGS-BAS-021§3 三级颗粒度数据映射 | §4.1 |
| RGS-BAS-021§4 数据流/控制流边构造 | §3、§4.1 |
| RGS-BAS-021§5 LangGraph可视化设计 | §3 |
| RGS-BAS-021§5.2 版本与新鲜度标注 | §3 |
| RGS-BAS-021§6.1/§6.2 业务视图声明式配置 | §2、§5 |
| RGS-BAS-021§6.3 个人自定义偏好 | §2、§5 |
| RGS-DTL-017（分析管线读端点复用） | §4.1（方法级颗粒度聚合路径） |
