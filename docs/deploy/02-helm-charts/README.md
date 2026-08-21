# 02-helm-charts — Helm Chart 占位

> **状态：🔴 NO-GO 占位**（per `RGS-PLAN-001 v0.7 §3.3` + `RGS-ENV-001 v0.2 §6`）
>
> 本目录所有 Chart 在 **53 開発環境構築 启动条件**全部满足前**不得升级为可 helm install 的实际 chart**。
>
> 当前 Chart 仅为**结构骨架**（Chart.yaml + values.yaml 占位），Chart.yaml `version: 0.0.0` + `appVersion: "PLACEHOLDER"`。**禁止在 NO-GO 解除前向本目录提交实际镜像 tag、副本数、resources 校准值、Secret values。**

---

## 1. 目录组织

```
02-helm-charts/
├── README.md                                  # 本文件
├── _status.md                                 # NO-GO 状态详情
├── rust-game-server/                          # Umbrella Chart
│   ├── Chart.yaml
│   ├── values.yaml
│   └── charts/                                # 6 个子 Chart
│       ├── player/
│       │   ├── Chart.yaml
│       │   ├── values.yaml
│       │   └── templates/                     # 实际模板（NO-GO 解除后才填充）
│       │       └── .gitkeep
│       ├── economy/
│       │   ├── Chart.yaml
│       │   ├── values.yaml
│       │   └── templates/.gitkeep
│       ├── match/
│       │   ├── Chart.yaml
│       │   ├── values.yaml
│       │   └── templates/.gitkeep
│       ├── social/
│       │   ├── Chart.yaml
│       │   ├── values.yaml
│       │   └── templates/.gitkeep
│       ├── admin/                              # 含 COC 控制面
│       │   ├── Chart.yaml
│       │   ├── values.yaml
│       │   └── templates/.gitkeep
│       └── cluster-ops/                       # Active-Active（per ADR-0052）
│           ├── Chart.yaml
│           ├── values.yaml
│           └── templates/.gitkeep
```

---

## 2. Chart 版本约定

| Chart | Chart version | appVersion | 升级路径 |
|---|---|---|---|
| `rust-game-server` (umbrella) | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |
| `player` | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |
| `economy` | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |
| `match` | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |
| `social` | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |
| `admin` (COC) | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |
| `cluster-ops` | `0.0.0` | `"PLACEHOLDER"` | NO-GO 解除后升 `0.1.0` |

> Chart 升级规则：
> - patch 修订（values 调整）：0.0.x
> - 域功能变更（新增配置项）：0.x.0
> - 破坏性变更（域拆分、API 变更）：x.0.0
> - 实际版本号在 NO-GO 解除后由 SRE + 5 域 Lead 联合约定

---

## 3. Chart 间依赖（umbrella）

`rust-game-server` 通过 `Chart.yaml` 的 `dependencies` 字段引用 6 个子 Chart：

| 依赖 Chart | condition（默认启用） | 别名 |
|---|---|---|
| `player` | `player.enabled: true` | `player` |
| `economy` | `economy.enabled: true` | `economy` |
| `match` | `match.enabled: true` | `match` |
| `social` | `social.enabled: true` | `social` |
| `admin` | `admin.enabled: true` | `admin` |
| `cluster-ops` | `clusterOps.enabled: true` | `cluster-ops` |

> 单一 Chart 安装（如只装 `player`）需 `helm install rust-game-server/charts/player`。Umbrella 安装需先 `helm dependency update rust-game-server/`。

---

## 4. 关键原则（per 治理文档）

| 原则 | 来源 | 落实位置 |
|---|---|---|
| 5 域共用 namespace，RBAC + NetworkPolicy 隔离 | ARC-008 | umbrella values.yaml + 子 chart |
| cluster-ops 域 Active-Active 固定 3 副本，**禁 HPA** | ADR-0052 | cluster-ops/values.yaml |
| Q-003 Saga 跨域核心：economy 域独立决策权 | DEC-005 | economy/values.yaml（独立 leader 签字栏） |
| 拒动态库加载，仅 Rhai 沙箱 | ARC-020 | 6 个域 values.yaml `rhai.scriptPath` |
| COC 控制面属 admin 域独立控制面 | DEC-005 | admin/values.yaml（独立 SA + COC bind） |
| Secret values 全部加密仓管理 | RGS-IMPL-001 §3.4 | 6 个域 values.yaml（**仅 reference，不存值**） |

---

## 5. NO-GO 解除条件

本目录从占位升级为可 helm install 的实际 Chart，必须满足：

1. **7 G-CODE 全部 Closed**（per `RGS-EXEC-001 v0.2`）
2. **RGS-ENV-001 v0.2 §6 12 类签字栏全部具名签字**（当前 2/12 实际签 + 10/12 所有者背书占位）
3. **RGS-REV-003 §7.3 12 类签字栏全部具名签字**（当前 8/12 实际签 + 10+ 所有者背书占位）
4. **01-k8s-manifests/ 实际配置完成**（SRE + 5 域 Lead 联合校准）
5. **03-db-migrations/ 实际 schema 完成**（DBA + 5 域 Lead 联合校准）

满足后由架构师出 v0.8 删除"所有者背书"占位 → 本目录 `_status.md` 升 `🟢 GO` → 由 SRE 主导 `helm install --dry-run` 验证 → 实际部署。

---

## 6. 关联文档

- 上游：`RGS-PLAN-001 v0.7` §3.3（NO-GO 强约束）
- 并行：`01-k8s-manifests/`（裸 manifest）+ `03-db-migrations/`（schema）+ `04-ci-cd/`（CI）
- SOP：`../05-deploy-sop.md` + `../06-rollback-sop.md`
- 自检表：`../07-no-go-checklist.md` + `../00-prerequisites/00-no-go-checklist.md`
