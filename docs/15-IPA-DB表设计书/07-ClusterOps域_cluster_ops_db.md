# 07-ClusterOps 域（cluster_ops_db）

> **本文件定位**：ClusterOps 域 3 张表的詳細表設計書。覆盖 2 基础表（cluster_nodes / feature_flags）+ 1 公共 outbox。**注意**：LCM 6 表归 admin_db（per FR-LCM-001），不在本文件展开 — 见 [06-Admin 域](06-Admin域_admin_db.md) §6.4-6.9。

| 项目 | 内容 |
|---|---|
| 物理库 | `cluster_ops_db` |
| 担当 crate | `cluster-ops` |
| DBMS | PostgreSQL 18 |
| 表数 | 3（含 outbox） |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/cluster-ops/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 7.1 | `cluster_nodes` | クラスタノード / Cluster Nodes | 永久事实表 | 百级 | 3 |
| 7.2 | `feature_flags` | 機能フラグ / Feature Flags | 永久事实表 | 千级 | 2 |
| 7.3 | `outbox` | アウトボックス（公共） / Outbox | 时序短期表 | 千万级/日 | 3 |

---

## 7.1 `cluster_nodes` クラスタノード

### 概要

集群节点注册表（per RGS-CAP-001 §ClusterOps 容量规划）。`hostname` UNIQUE 唯一。3 角色（primary / replica / candidate）。4 健康状态（healthy / degraded / unhealthy / maintenance）。`last_heartbeat_at` 用于节点存活判定。

| 项目 | 内容 |
|---|---|
| 物理表名 | `cluster_nodes` |
| 論理名 | クラスタノード / Cluster Nodes |
| 出典 | `crates/cluster-ops/migrations/0001_init.sql:5-16` |
| 父文档 | RGS-CAP-001 §ClusterOps 容量规划 |
| 関連表 | (无 FK) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `hostname` | ホスト名 / Hostname | TEXT | 1-253 字符 | — | — | ✅ | ✅ | — | — | 节点主机名（全局唯一）|
| 3 | `ip` | IP アドレス / IP Address | TEXT | 1-45 字符 (IPv4/IPv6) | — | — | — | ✅ | — | — | 节点 IP |
| 4 | `role` | 役割（クラスタ） / Cluster Role | TEXT | — | — | — | — | ✅ | — | `role IN ('primary', 'replica', 'candidate')` | 3 选 1 集群角色 |
| 5 | `status` | ノード状態 / Node Status | TEXT | — | — | — | — | ✅ | `'healthy'` | `status IN ('healthy', 'degraded', 'unhealthy', 'maintenance')` | 4 健康状态 |
| 6 | `last_heartbeat_at` | 最終ハートビート / Last Heartbeat | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 节点存活判定 |
| 7 | `version` | ノードバージョン / Node Version | TEXT | 1-32 字符 | — | — | — | ✅ | — | — | 节点运行的 RGS 版本（语义化版本）|
| 8 | `registered_at` | 登録日時 / Registered At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 注册时间 |
| 9 | `enabled_at` | 有効化日時 / Enabled At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 启用时间（NULL = 待启用）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `cluster_nodes_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `cluster_nodes_hostname_key` | B-tree (UNIQUE) | `(hostname)` | 主机名唯一 |
| 3 | `idx_nodes_status` | B-tree | `(status)` | 按状态筛选（健康/异常节点列表）|
| 4 | `idx_nodes_role` | B-tree | `(role)` | 按角色筛选（primary 列表）|
| 5 | `idx_nodes_heartbeat` | B-tree | `(last_heartbeat_at)` | 存活判定（超时节点扫描）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `cluster_nodes_pkey` | `(id)` |
| UNIQUE | `cluster_nodes_hostname_key` | `(hostname)` |
| CHECK | (隐式) `role_check` | `role IN ('primary', 'replica', 'candidate')` |
| CHECK | (隐式) `status_check` | `status IN ('healthy', 'degraded', 'unhealthy', 'maintenance')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| (无) | — | — | — | — |

### 既知偏差

- 缺 `role='primary'` 唯一性约束（一个集群只能有一个 primary）——建议 PH-2 加 `CREATE UNIQUE INDEX idx_nodes_one_primary ON cluster_nodes (role) WHERE role = 'primary'`
- `last_heartbeat_at` 在 status='maintenance' 时仍持续刷新？建议 PH-2 加应用层逻辑：maintenance 节点不更新 heartbeat

---

## 7.2 `feature_flags` 機能フラグ

### 概要

特性开关表（per RGS-CAP-001 §ClusterOps 容量规划）。**复合主键** `(key, scope_value)` 允许同一 flag 在不同 scope（global / domain / node）有不同值。3 种 scope（global / domain / node）。`version` OCC。

| 项目 | 内容 |
|---|---|
| 物理表名 | `feature_flags` |
| 論理名 | 機能フラグ / Feature Flags |
| 出典 | `crates/cluster-ops/migrations/0001_init.sql:18-26` |
| 父文档 | RGS-CAP-001 §ClusterOps |
| 関連表 | (无 FK) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `key` | フラグキー / Flag Key | TEXT | 1-128 字符 | ✅ (联合) | — | — | ✅ | — | — | 标志名（如 `enable_new_auction_logic`）|
| 2 | `scope` | スコープ / Scope | TEXT | — | — | — | — | ✅ | — | `scope IN ('global', 'domain', 'node')` | 3 选 1 作用域 |
| 3 | `scope_value` | スコープ値 / Scope Value | TEXT | 1-128 字符 | ✅ (联合) | — | — | ✅ | — | — | scope 对应值（global="", domain="player", node=hostname）|
| 4 | `enabled` | 有効フラグ / Enabled Flag | BOOLEAN | — | — | — | — | ✅ | FALSE | — | 是否启用 |
| 5 | `version` | フラグバージョン（OCC） / Flag Version (OCC) | BIGINT | >= 0 | — | — | — | ✅ | 0 | — | 乐观锁版本号 |
| 6 | `updated_by` | 更新者 ID / Updated By | UUID | 128-bit | — | — (跨域弱引用 → admin_users) | — | ✅ | — | — | 更新者 admin_users.id |
| 7 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `feature_flags_pkey` | B-tree (PK) | `(key, scope_value)` | 联合主键 |
| 2 | `idx_flags_scope_value` | B-tree | `(scope_value)` | 按 scope_value 筛选（按域/按节点查）|
| 3 | `idx_flags_enabled` | B-tree | `(enabled)` | 按启用状态筛选（查所有启用的 flag）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `feature_flags_pkey` | `(key, scope_value)` |
| CHECK | (隐式) `scope_check` | `scope IN ('global', 'domain', 'node')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (跨域) | `admin_users` (admin_db) | `feature_flags.updated_by = admin_users.id` | app-layer | ❌ 弱引用（跨 DB）|

### 既知偏差

- **`scope_value` 与 `scope` 一致性约束缺**：scope='global' 时 scope_value 应为 `''`、scope='domain' 时 scope_value 应在已知域列表内、scope='node' 时 scope_value 应在 cluster_nodes.hostname 列表内——建议 PH-2 加应用层校验 + 文档化 SOP
- 缺 `created_at`（只有 updated_at）

---

## 7.3 `outbox` アウトボックス（公共 / ClusterOps 域）

> 完整模板見 [13-Outbox 跨域模板](13-Outbox跨域模板.md)。

- **位置**：`cluster_ops_db.outbox`
- **结构**：与模板 1:1 一致
- **特有应用层**：
  - `cluster_ops.node.heartbeat_lost` / `node_recovered`
  - `cluster_ops.node.promoted` / `demoted`
  - `cluster_ops.feature_flag.toggled` / `scope_added` / `scope_removed`
  - `cluster_ops.lcm.realm_lifecycle.<subtype>` (LCM 事件转发，可选)

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/cluster-ops/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` |
| CAP-001 | `docs/10-技术选型/RGS-CAP-001_ClusterOps容量规划_v0.1.md` |

> 任何实际 schema 与本文档不一致之处，以 `crates/cluster-ops/migrations/*.sql` 实际 SQL 为准。
