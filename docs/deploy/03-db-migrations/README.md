# 03-db-migrations — PostgreSQL 18.4 数据库迁移占位

> **状态：🔴 NO-GO 占位**（per `RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6`）
>
> 本目录所有 `.sql` 文件在 **53 開発環境構築 启动条件**全部满足前**不得执行迁移**。
>
> 当前文件全部为**结构骨架**（仅注释，无 DDL），仅用于提前铺好 5 域 + cluster-ops 各自独立 DB 的迁移目录。**禁止在 NO-GO 解除前向本目录提交实际 schema、索引、约束、种子数据。**

---

## 1. 5 独立 DB 原则（per ARC-008）

| DB 名 | 域 | 用途 | 状态 |
|---|---|---|---|
| `player_db` | player | 玩家账号 / 角色 / 状态 | 占位 |
| `economy_db` | economy | 货币 / 道具 / 交易 / Q-003 Saga | 占位 |
| `match_db` | match | 房间 / 匹配 / tick | 占位 |
| `social_db` | social | 好友 / 聊天 / 群组 | 占位 |
| `admin_db` | admin | COC 控制面 / 审计 / 配载 | 占位 |
| `cluster_ops_db` | cluster-ops | 集群状态 / PFAU 记录 / 节点调谐 | 占位 |

> **5 独立 DB + 1 cluster_ops_db = 6 个独立 PostgreSQL 实例/Schema**（per ARC-008）
>
> 跨域事务通过 **Q-003 Saga 状态机** 协调（per DTL-015/016），**不通过跨 DB 事务**（不引入 2PC）。

---

## 2. 目录组织

```
03-db-migrations/
├── README.md                                  # 本文件
├── _status.md                                 # NO-GO 状态详情
├── player_db/
│   └── 0001_initial_placeholder.sql           # 占位（无 DDL）
├── economy_db/
│   ├── 0001_initial_placeholder.sql           # 占位
│   └── 0002_q003_saga_state_placeholder.sql   # 占位（Q-003 Saga 状态机）
├── match_db/
│   └── 0001_initial_placeholder.sql           # 占位
├── social_db/
│   └── 0001_initial_placeholder.sql           # 占位
├── admin_db/
│   ├── 0001_initial_placeholder.sql           # 占位
│   └── 0002_coc_audit_log_placeholder.sql     # 占位（COC 审计）
└── cluster_ops_db/
    ├── 0001_initial_placeholder.sql           # 占位
    └── 0002_pfau_history_placeholder.sql      # 占位（PFAU 升级历史，per ADR-0052）
```

---

## 3. 迁移工具（待选型）

候选迁移工具：

| 工具 | 备注 | 决策状态 |
|---|---|---|
| **sqlx-migrate**（推荐） | 与 sqlx 集成，纯 SQL 文件，Rust 生态原生 | 待 DBA + SRE 联合选型 |
| refinery | 纯 Rust，支持内嵌迁移 | 备选 |
| flyway | JVM 工具，跨语言 | 备选（需 JVM 运行时） |
| golang-migrate | 跨语言，但 Go 生态优先 | 备选 |

> **实际选型由 DBA + SRE 联合签字后确定**，写在 `RGS-ENV-001 v0.3` 实际核验记录中。

---

## 4. 5 独立 DB 拓扑（待 DBA 落地）

```
┌─────────────────┐
│   player 域     │──→ player_db     (独立 PG 实例或独立 Schema)
│   economy 域    │──→ economy_db    (独立 PG 实例或独立 Schema) — Q-003 Saga 核心
│   match 域      │──→ match_db      (独立 PG 实例或独立 Schema)
│   social 域     │──→ social_db     (独立 PG 实例或独立 Schema)
│   admin 域      │──→ admin_db      (独立 PG 实例或独立 Schema) — COC
│   cluster-ops 域│──→ cluster_ops_db (独立 PG 实例或独立 Schema) — PFAU 历史
└─────────────────┘
        ↕ 跨域协同
   Q-003 Saga 状态机
   (per DTL-015/016)
```

**约束**：
- 每域 DB 单独的连接池（不共享）
- 跨域操作通过 gRPC + Saga，不通过跨 DB 视图
- Schema 命名规范：`{domain}_v{major}`（如 `player_v1`），后续升级用新 schema + 视图切换

---

## 5. PG 18.4 关键特性利用（per RGS-TS-001 §5.2）

- **逻辑复制**：cluster_ops_db 订阅各域 CDC，驱动 PFAU all-reachable
- **分区表**：match_db 房间状态按时间分区（高频写入）
- **JSONB 索引**：social_db 消息 payload 用 GIN 索引
- **审计触发器**：admin_db 自动写入 audit_log（per ARC-051）
- **PGAudit**：admin_db + cluster_ops_db 开启（高权限域）

---

## 6. NO-GO 解除条件

本目录从占位升级为实际迁移，必须满足：

1. **7 G-CODE 全部 Closed**（per `RGS-EXEC-001 v0.3`），特别：
   - G-CODE-03 DBA 具名 + 5 独立 DB 拓扑图签字
2. **RGS-ENV-001 v0.3 §6 12 类签字栏全部具名签字**（当前 2/12 实际签 + 10/12 所有者背书占位）
3. **DBA 联合 5 域 Lead 完成 6 个 DB 的 schema 设计**（DTL → DDL 落地）
4. **迁移工具选型确定**（DBA + SRE 联合签字）

满足后由架构师出 v0.8 删除"所有者背书"占位 → 本目录 `_status.md` 升 `🟢 GO` → 由 DBA 主导 `sqlx migrate run` 在 staging 验证 → 实际部署。

---

## 7. 关联文档

- 上游：`RGS-TS-001 v0.6 §5.2`（PG 18.4 选型）+ `RGS-TS-001 v0.6 §5.1`（sqlx 选型）
- 并行：`01-k8s-manifests/`（Secret 引用）+ `02-helm-charts/`（values 引用）
- 设计：`ARC-008 5 独立 DB 原则` + `DTL-015/016/018/019/020/026/031`
- 治理：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6`
- SOP：`../05-deploy-sop.md` §3（DB 迁移步骤）
- 自检表：`../07-no-go-checklist_v0.2.md`
