# 01-k8s-manifests — Kubernetes 清单占位

> **状态：🔴 NO-GO 占位**（per `RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` 12 类签字栏）
>
> 本目录所有 yaml 在 **53 開発環境構築 启动条件**（7 G-CODE Closed + 12 类签字齐全）**全部满足前**不得替换为实际业务配置。
>
> 当前文件全部为**结构骨架**（namespace / 占位 Deployment / ConfigMap 模板），仅用于提前铺好目录与命名空间边界。**禁止在 NO-GO 解除前向本目录提交实际镜像 tag、副本数、资源 request/limit、Secret 实际值。**

---

## 1. 目录组织

| 序号 | 文件 | 角色 | 状态 |
|---|---|---|---|
| `00-namespace.yaml` | 命名空间 + ResourceQuota + LimitRange | 平台 | 占位 |
| `01-player-service.yaml` | player 域 Deployment + Service | 域 | 占位 |
| `02-economy-service.yaml` | economy 域 Deployment + Service | 域 | 占位 |
| `03-match-service.yaml` | match 域 Deployment + Service | 域 | 占位 |
| `04-social-service.yaml` | social 域 Deployment + Service | 域 | 占位 |
| `05-admin-service.yaml` | admin 域（含 COC 控制面）Deployment + Service | 域 | 占位 |
| `06-cluster-ops-service.yaml` | cluster-ops 域（Active-Active 多副本）Deployment + Service | 集群 | 占位 |
| `07-shared-platform.yaml` | shared-platform（QUIC edge / gRPC ingress / OTel collector） | 平台 | 占位 |
| `08-configmap-template.yaml` | ConfigMap 模板（含 5 域配置 + CEM topic 路由） | 平台 | 占位 |
| `09-secret-template.yaml` | Secret 模板（**仅占位结构，values 全部 `PLACEHOLDER_*`**） | 平台 | 占位 |
| `10-rbac-template.yaml` | ServiceAccount + Role + RoleBinding（5 域 + cluster-ops + shared-platform） | 平台 | 占位 |

---

## 2. 命名空间约定

- 主命名空间：`rust-game-server`（**待 DBA + SRE 具名责任人确认**）
- 5 域共用同一 namespace（**不切分**），由 RBAC + NetworkPolicy 隔离（per ARC-008）
- `cluster-ops` 域使用独立 ServiceAccount（per ADR-0052，Active-Active 跨节点调谐权限）

---

## 3. 镜像与副本数（占位）

| 域 | 镜像 tag | 初始副本 | HPA min/max | 备注 |
|---|---|---|---|---|
| player | `PLACEHOLDER_PLAYER_IMAGE` | 2 | 2/8 | 读多写少 |
| economy | `PLACEHOLDER_ECONOMY_IMAGE` | 2 | 2/6 | 事务密集，CPU-bound |
| match | `PLACEHOLDER_MATCH_IMAGE` | 3 | 3/12 | 实时匹配，弹性需求高 |
| social | `PLACEHOLDER_SOCIAL_IMAGE` | 2 | 2/6 | 中等 |
| admin | `PLACEHOLDER_ADMIN_IMAGE` | 1 | 1/2 | 含 COC 控制面，低流量高权限 |
| cluster-ops | `PLACEHOLDER_CLUSTER_OPS_IMAGE` | 3 | 3/3 | Active-Active，**禁 HPA** |

> **禁 HPA**：cluster-ops 域为 Active-Active 固定 3 副本（per ADR-0052），HPA 弹性扩缩会破坏 all-reachable 假设。

---

## 4. 资源 request/limit（占位）

| 域 | CPU req | CPU lim | Mem req | Mem lim | 备注 |
|---|---|---|---|---|---|
| player | 500m | 2000m | 512Mi | 2Gi | 读路径 |
| economy | 1000m | 4000m | 1Gi | 4Gi | 事务 |
| match | 1000m | 4000m | 1Gi | 4Gi | 实时 |
| social | 500m | 2000m | 512Mi | 2Gi | 中等 |
| admin | 250m | 1000m | 256Mi | 1Gi | 低流量 |
| cluster-ops | 500m | 2000m | 512Mi | 2Gi | 控制面 |
| shared-platform | 1000m | 2000m | 1Gi | 2Gi | 边缘代理 |

> 实际值由 **5 域 Lead + SRE 联合压测后** 校准（per `RGS-ENV-CALIB-001 v0.1` 模板）。

---

## 5. NO-GO 解除条件

本目录从占位升级为实际清单，必须满足：

1. **7 G-CODE 全部 Closed**（per `RGS-EXEC-001 v0.3`）
   - G-CODE-01 业务方代表具名签字
   - G-CODE-02 5 域 Lead 独立具名（不兼任，per DEC-005）
   - G-CODE-03 DBA 具名 + 5 独立 DB 拓扑图签字
   - G-CODE-04 SRE 具名 + 部署 SOP 签字
   - G-CODE-05 Platform 架构师具名 + CI/CD 签字
   - G-CODE-06 Rust 1.98 + Cargo.lock + CI 全绿
   - G-CODE-07 QA Lead 具名 + 验收矩阵签字
2. **RGS-ENV-001 v0.3 §6 12 类签字栏全部具名签字**（当前 2/12 实际签 + 10/12 所有者背书占位）
3. **RGS-REV-003 §7.3 12 类签字栏全部具名签字**（当前 8/12 实际签 + 10+ 所有者背书占位）

满足后由架构师出 v0.8 删除"所有者背书"占位 → 本目录 `_status.md` 升 `🟢 GO` → 由 SRE 主导替换为实际配置。

---

## 6. 关联文档

- 上游：`RGS-PLAN-001 v0.8` §3.3（NO-GO 强约束）+ `RGS-ENV-001 v0.3` §6（12 类签字）
- 并行：`RGS-EXEC-001 v0.3`（G-CODE 突破手册）+ `RGS-REV-003`（联合评审）
- 兄弟目录：`02-helm-charts/`（Chart 包装层）、`03-db-migrations/`（DB schema）
- 顶层：`../README.md`、`../07-no-go-checklist_v0.2.md`
- 前置：`../00-prerequisites/00-no-go-checklist_v0.2.md`
