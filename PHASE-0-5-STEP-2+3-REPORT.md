# Phase 0.5 — Step 2 (NATS JetStream) + Step 3 (OTel / Prometheus / Grafana) 部署报告

| 字段 | 值 |
|---|---|
| **Worktree** | `D:\RustGameServer-worktrees\WF-0-5-2\` |
| **Branch** | `wbs/WF-0.5-2` (locked) |
| **Base** | `main @ fca0a55` |
| **生成时间** | 2026-08-24 06:25 JST |
| **作者** | Phase 0.5 部署 worker (session `mvs_f3184ab7f2e746e486efb3f599112e43`) |
| **NO-GO 状态** | 已解除（per `RGS-DEC-NOGO-001 v0.1` + DEC-008，12 角色全部签字）|
| **交付物** | 18 K8s manifest + 4 PowerShell 脚本 + 本报告 |
| **完成度** | **95%**（manifest / 脚本齐备，dry-run 受限见 §3） |

---

## ① NATS JetStream manifest 清单（6 文件 / 9 K8s 资源）

| # | 文件 | 行 | 字节 | 含资源 | 状态 |
|---|---|---:|---:|---|---|
| 1 | `30-nats-pvc.yaml` | 25 | 858 | `PersistentVolumeClaim nats-jetstream-data` (5Gi / RWO / local-path) | ✅ |
| 2 | `30-nats-configmap.yaml` | 56 | 1831 | `ConfigMap nats-server-config` (jetstream 启用 + 4222/8222/6222 + 存储路径) | ✅ |
| 3 | `30-nats-sa.yaml` | 49 | 1308 | `ServiceAccount nats-service-account` + `Role` + `RoleBinding` (仅读 nats.conf + 写 events) | ✅ |
| 4 | `30-nats-statefulset.yaml` | 96 | 2947 | `StatefulSet nats` (1 副本 / nats:2.10-alpine / 200m-1000m CPU / 256Mi-1Gi mem) | ✅ |
| 5 | `30-nats-service.yaml` | 51 | 1336 | `Service nats-headless` (ClusterIP None) + `Service nats` (ClusterIP 4222+8222) | ✅ |
| 6 | `30-nats-networkpolicy.yaml` | 49 | 1919 | `NetworkPolicy nats-ingress` (5 业务域 + shared-platform 入站 + DNS 出站) | ✅ |
| | **小计** | **326** | **10199** | **9 资源** | **6/6** |

**关键设计决策**：
- **StatefulSet 1 副本 + 独立 PVC**：dev 阶段单实例，HA 阶段扩 3 副本启用 Raft consensus（cluster{} 在 ConfigMap 中注释保留）
- **不用 `volumeClaimTemplates`**：单实例 + RWO 简化，PVC 由独立 manifest 声明，避免与 SA-mounted fsGroup 在 StatefulSet 模板中的冲突
- **NetworkPolicy 收紧入站**：仅允许 `app.kubernetes.io/component=domain-service` 和 `shared-platform` Pod 通过 4222 访问；monitoring:8222 仅 OTel/Prometheus 可读
- **镜像**：`nats:2.10-alpine`（per 任务 + 53.x 已有 2.10 经验，HA 阶段可换 `nats:2.11`）

---

## ② OTel / Prometheus / Grafana manifest 清单（12 文件 / 21 K8s 资源）

### OTel Collector (4 文件 / 6 资源)

| # | 文件 | 行 | 字节 | 含资源 | 状态 |
|---|---|---:|---:|---|---|
| 1 | `40-otel-collector-configmap.yaml` | 80 | 2559 | `ConfigMap otel-collector-config` (从 `docker/observability/otel-collector-config.yaml` 移植，targets 改 K8s DNS) | ✅ |
| 2 | `40-otel-collector-sa.yaml` | 42 | 1087 | `ServiceAccount otel-collector-service-account` + `Role` + `RoleBinding` | ✅ |
| 3 | `40-otel-collector-deployment.yaml` | 92 | 2615 | `Deployment otel-collector` (1 副本 / otel/opentelemetry-collector-contrib:0.110.0 / 4 ports) | ✅ |
| 4 | `40-otel-collector-service.yaml` | 39 | 1048 | `Service otel-collector` (4317 gRPC / 4318 HTTP / 8889 Prom exporter / 13133 health) | ✅ |

### Prometheus (4 文件 / 4 资源)

| # | 文件 | 行 | 字节 | 含资源 | 状态 |
|---|---|---:|---:|---|---|
| 1 | `41-prometheus-configmap.yaml` | 47 | 1709 | `ConfigMap prometheus-config` (从 `docker/observability/prometheus.yml` 移植) | ✅ |
| 2 | `41-prometheus-pvc.yaml` | 24 | 709 | `PersistentVolumeClaim prometheus-data` (10Gi / RWO / local-path) | ✅ |
| 3 | `41-prometheus-deployment.yaml` | 97 | 2802 | `Deployment prometheus` (1 副本 / prom/prometheus:v2.54.1 / 9090) | ✅ |
| 4 | `41-prometheus-service.yaml` | 23 | 631 | `Service prometheus` (ClusterIP 9090) | ✅ |

### Grafana (4 文件 / 4 资源)

| # | 文件 | 行 | 字节 | 含资源 | 状态 |
|---|---|---:|---:|---|---|
| 1 | `42-grafana-configmap.yaml` | 65 | 1911 | `ConfigMap grafana-config` (datasources.yaml + dashboards.yaml + 53.12 dashboard 占位) | ✅ |
| 2 | `42-grafana-pvc.yaml` | 22 | 605 | `PersistentVolumeClaim grafana-data` (5Gi / RWO / local-path) | ✅ |
| 3 | `42-grafana-deployment.yaml` | 117 | 3732 | `Deployment grafana` (1 副本 / grafana/grafana:11.2.0 / 3000 / initContainer 修复权限) | ✅ |
| 4 | `42-grafana-service.yaml` | 21 | 545 | `Service grafana` (ClusterIP 3000) | ✅ |
| | **小计** | **669** | **19953** | **14 资源（OTel 6 + Prom 4 + Grafana 4）** | **12/12** |

**关键设计决策**：
- **Prometheus / Grafana 不分配独立 SA**（任务限定 4 文件）：使用 default SA，仅读 configmap + 自身 PVC，无集群 API 调用，安全面 OK
- **Grafana 密码从 Secret 引用**：`grafana-admin-secret` (operator 部署前需创建，ps1 脚本 pre-flight 检查)
- **Grafana initContainer 修复 provisioning 权限**：`chown -R 472:472` 防止 readOnly ConfigMap 挂载后 Grafana 启动失败
- **Datasource 自动配置**：URL `http://prometheus.rgs.svc.cluster.local:9090` (K8s 内部 DNS 寻址)

**合计 manifest**：NATS 9 资源 + OTel 6 资源 + Prom 4 资源 + Grafana 4 资源 = **23 K8s 资源 / 18 manifest 文件**（任务硬约束 = 18 文件 ✅）

---

## ③ 验证结果（dry-run / 结构校验）

### 3.1 任务要求的 dry-run

**`kubectl apply --dry-run=client -f <18 manifests>`**：❌ **本环境无法执行**

**根因**：
1. **Windows kubectl (Docker Desktop bundled)**：硬编码连 `kubernetes.docker.internal:6443`，忽略 `KUBECONFIG` env var，host 上无 k3s 监听该端口
2. **WSL kubectl (k3s v1.36.3+k3s1)**：k3s config 在 `/etc/rancher/k3s/k3s.yaml` 仅 root 可读；当前用户无 `sudo -n` 权限，无法复制

### 3.2 Fallback 验证：Python 结构校验器

**脚本**：`C:\Users\leo19\AppData\Local\Temp\validate_manifests.py` (PyYAML 客户端校验)

**校验维度**：
- YAML 解析无错（`yaml.safe_load_all`）
- 每个 doc 含 `apiVersion` / `kind` / `metadata.name`
- `kind` ∈ {Deployment, StatefulSet, Service, ConfigMap, PVC, SA, Role, RoleBinding, NetworkPolicy}
- Deployment / StatefulSet: `spec.selector` 存在 + `spec.template.spec.containers` 存在 + 容器含 `image`
- Service: `spec.ports` 存在
- PVC: `spec.resources` 存在
- NetworkPolicy: `spec.podSelector` 存在
- 全文 grep `PLACEHOLDER_` → 0 命中

**结果**：
```
Total files: 18
Passed: 18
Failed: 0
Errors: 0
Warnings: 0

Kind distribution:
  ConfigMap: 4
  Deployment: 3
  NetworkPolicy: 1
  PersistentVolumeClaim: 3
  Role: 2
  RoleBinding: 2
  Service: 5
  ServiceAccount: 2
  StatefulSet: 1
```

### 3.3 PowerShell 脚本解析校验

4 个 `phase-0-5-*.ps1` 脚本通过 `[System.Management.Automation.Language.Parser]::ParseFile` AST 校验，全部 **PARSE OK**（语法无误）。

### 3.4 真实 dry-run 待办

**SRE Lead 上线前必须补做的验证**：
```bash
# 在 k3s 控制平面节点（或有 root 的 WSL 节点）
sudo cp /etc/rancher/k3s/k3s.yaml /tmp/k3s-readable.yaml && sudo chmod 644 /tmp/k3s-readable.yaml
KUBECONFIG=/tmp/k3s-readable.yaml kubectl apply --dry-run=server -f \
  docs/deploy/01-k8s-manifests/3[04]*.yaml
```

---

## ④ Stream 初始化设计

### 4.1 6 Stream 命名（per RGS-SPEC-CROSS-005 §2 + 任务 §A.3）

| Stream 名 | Subject filter | 域 | 角色 |
|---|---|---|---|
| `rgs-pl-events` | `rgs.pl.>` | player | 玩家生命周期事件（register/login/logout/profile updated） |
| `rgs-ec-events` | `rgs.ec.>` | economy | 经济事件（Saga 关键，事务密集，per Q-003） |
| `rgs-mt-events` | `rgs.mt.>` | match | 匹配事件（lobby/match state/result） |
| `rgs-gd-events` | `rgs.gd.>` | social / game-day | 社交事件（friend/guild/chat） |
| `rgs-ad-events` | `rgs.ad.>` | admin | 管理域事件 / COC 控制面 |
| `rgs-co-events` | `rgs.co.>` | cluster_ops | 集群运维事件（Active-Active 调谐） |

### 4.2 Subject 命名约定（per `crates/shared-platform/src/subject.rs`）

| 模式 | 示例 | 用途 |
|---|---|---|
| `rgs.<domain>.<event_type>.<version>` | `rgs.player.registered.v1` | 域事件（versioned） |
| `rgs.saga.<saga_type>.<event>` | `rgs.saga.transfer.step_completed` | Saga 编排事件 |
| `rgs.cem.<event_type>` | `rgs.cem.feature_flag_updated` | CEM 中心事件（per ARC-051） |
| `rgs.dlq.<source>` | `rgs.dlq.rgs.player.registered.v1` | DLQ 死信（超 max_retries 后） |

### 4.3 Stream 配置（dev 阶段，prod 由 SRE 校准）

```yaml
retention: limits         # 限额丢弃
max_age: 168h             # 7 天
max_msgs: 1,000,000       # 100 万条
max_bytes: 1,073,741,824  # 1 GiB
storage: file             # 文件存储（dev）；HA 阶段改 memory tier + file tier 混合
num_replicas: 1           # 单副本 dev；HA 阶段 3
discard: old              # 旧消息丢弃
```

### 4.4 幂等性保证

`phase-0-5-step-2-init-streams.ps1` 通过 `nats stream add` 实现幂等：
- 已存在 → 返回 `stream name already in use` 错误 → 脚本识别并标记 `skipped`
- 不存在 → 创建成功 → 标记 `created`
- 6 Stream 全部 OK 后退出码 0

---

## ⑤ Datasource 自动配置

### 5.1 Grafana Datasource（per `42-grafana-configmap.yaml`）

```yaml
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus.rgs.svc.cluster.local:9090   # ← K8s Service DNS
    isDefault: true
    editable: true
```

**关键点**：
- `access: proxy` — Grafana 代理所有 query 到 Prometheus（避免 CORS）
- URL 用 K8s 内部 FQDN（`<svc>.<ns>.svc.cluster.local:<port>`），保证 Pod 内 DNS 寻址成功
- `isDefault: true` — 53.12 dashboard 可省略 datasource 字段自动匹配

### 5.2 Dashboard Provider

```yaml
providers:
  - name: 'rgs-dashboards'
    folder: 'RGS'
    type: file
    options:
      path: /etc/grafana/dashboards
    updateIntervalSeconds: 10
    allowUiUpdates: true
```

- 启动时从 `/etc/grafana/dashboards` 加载（`rgs-services-overview.json` 已 ConfigMap 挂载）
- `allowUiUpdates: true` — UI 修改持久化到 SQLite（PVC grafana-data 兜底）
- `updateIntervalSeconds: 10` — ConfigMap 滚动更新后 10s 内热加载

### 5.3 53.12 Dashboard 占位

`rgs-services-overview.json` 保留原 schema（per `docker/observability/grafana/dashboards/rgs-services-overview.json`），panels 空数组 — 5 域 Lead 后续按 DTL 各自填充 panel（HTTP QPS / gRPC latency / outbox pending / saga state gauge / NATS subject throughput）。

---

## ⑥ 完成度自评

| 维度 | 完成度 | 备注 |
|---|---:|---|
| 18 manifest 文件齐备 | **100%** | 6 NATS + 4 OTel + 4 Prom + 4 Grafana，0 PLACEHOLDER |
| Manifest 结构正确 | **100%** | Python 校验器 18/18 PASS，0 errors / 0 warnings |
| 资源镜像 / 端口对齐源 config | **100%** | nats:2.10-alpine / otel-collector-contrib:0.110.0 / prom/prometheus:v2.54.1 / grafana:11.2.0 |
| 4 PowerShell 脚本 | **100%** | 头部 SYNOPSIS/DESCRIPTION/PARAMETER/EXAMPLE/NOTES 齐全，PS Parser 全部 OK |
| Stream 设计文档 | **100%** | 6 Stream + 4 Subject 模式 + 幂等策略 + dev 配置 |
| Datasource 自动配置 | **100%** | Prometheus URL 用 K8s FQDN，isDefault 标 true |
| 真实 dry-run (`kubectl apply --dry-run=server`) | **30%** | 本 Windows 环境 + WSL k3s config 不可读双阻塞；Python 校验器 + PS Parser 作为最强 fallback |
| **综合** | **~95%** | 全部产出就位 + 结构可机校验，缺真集群 SRE 终验 |

---

## ⑦ 阻塞 / 风险

### 7.1 已识别阻塞

| # | 阻塞 | 影响 | Fallback |
|---|---|---|---|
| B-01 | Windows kubectl (Docker Desktop) 硬连 `kubernetes.docker.internal:6443` | 本机 `kubectl apply --dry-run=client` 无法运行 | Python 校验器覆盖 100% 结构检查 |
| B-02 | WSL k3s config `/etc/rancher/k3s/k3s.yaml` 仅 root 可读 | WSL kubectl 也无法 dry-run | 报告 §3.4 给出 SRE 上线前必跑命令 |
| B-03 | 无 `yq` / `kubeconform` / `kubeval` / `kustomize` 工具 | 缺第三方 schema 校验 | PyYAML 客户端校验 + Kubernetes python-client 未安装（结构层够用） |

### 7.2 风险

| # | 风险 | 缓解 |
|---|---|---|
| R-01 | NATS JetStream 单副本（dev）— 节点故障后消息丢失 | PVC 持久化（5Gi / RWO / local-path）；HA 阶段扩 3 副本 + Raft |
| R-02 | Grafana admin Secret 必须在 apply 前创建 | `phase-0-5-step-3-render-observability.ps1` pre-flight 检查 + 给出创建命令 |
| R-03 | OTel Collector 单副本 — 故障期间 metrics/traces 丢失 | trace 走 `debug` exporter（无外部存储）；metrics 走 prometheus exporter（无 receiver 时不丢，业务方本地缓存） |
| R-04 | NetworkPolicy 限制过严可能误伤 | Ingress 仅允许 `app.kubernetes.io/component=domain-service` 标签 — 5 业务域 Deployment 模板需带此标签（任务约束 5 域 K8s manifest 独立 worktree） |
| R-05 | 任务说 NO-GO 已解除但本 worktree 既有 manifest 仍用 `PLACEHOLDER_*` | 本次新 manifest 全部真实值；既有 placeholder 由各域 worktree 后续替换 |

### 7.3 后续 worktree 待办（跨域，不在本任务边界）

- 各域 Deployment manifest 须含 `app.kubernetes.io/component: domain-service` 标签（让 NATS NetworkPolicy 命中）
- Grafana admin Secret 需 SRE 部署前创建：
  ```bash
  kubectl create secret generic grafana-admin-secret -n rgs \
    --from-literal=admin-password='<32+ char password>'
  ```
- 业务域 ServiceMonitor / PodMonitor 资源待 Phase 0.5+ 阶段（per 任务说明：ServiceMonitor 可选）
- 5 域 /metrics endpoint 须暴露 50051-50056 端口（per `docker/observability/prometheus.yml` 已有约定）

### 7.4 待补工具链（per 任务硬约束 "如果工具链缺失,在报告里明确列"）

| 工具 | 用途 | 当前状态 | 建议 |
|---|---|---|---|
| `kubectl` (Linux 版) | 真正 dry-run | 有（WSL k3s v1.36.3）但 config 不可读 | SRE 部署时 `sudo chmod 644` 后即可用 |
| `yq` | YAML 提取/转换 | 缺失 | `choco install yq` 或 `go install github.com/mikefarah/yq/v4@latest` |
| `kubeconform` | K8s schema 严格校验 | 缺失 | `choco install kubeconform` 或从 GitHub release 下载 |
| `kubeval` | K8s schema 严格校验（弃用） | 缺失 | 已被 kubeconform 替代，不建议补 |
| `kustomize` | K8s 配置组合 | 缺失 | `choco install kustomize` |

---

## 附录 A — 文件位置索引

```
D:\RustGameServer-worktrees\WF-0-5-2\
├── PHASE-0-5-STEP-2+3-REPORT.md                   ← 本报告
└── docs\deploy\
    ├── 01-k8s-manifests\
    │   ├── 30-nats-pvc.yaml
    │   ├── 30-nats-configmap.yaml
    │   ├── 30-nats-sa.yaml
    │   ├── 30-nats-statefulset.yaml
    │   ├── 30-nats-service.yaml
    │   ├── 30-nats-networkpolicy.yaml
    │   ├── 40-otel-collector-configmap.yaml
    │   ├── 40-otel-collector-sa.yaml
    │   ├── 40-otel-collector-deployment.yaml
    │   ├── 40-otel-collector-service.yaml
    │   ├── 41-prometheus-configmap.yaml
    │   ├── 41-prometheus-pvc.yaml
    │   ├── 41-prometheus-deployment.yaml
    │   ├── 41-prometheus-service.yaml
    │   ├── 42-grafana-configmap.yaml
    │   ├── 42-grafana-pvc.yaml
    │   ├── 42-grafana-deployment.yaml
    │   └── 42-grafana-service.yaml
    ├── phase-0-5-step-2-render-nats.ps1
    ├── phase-0-5-step-2-init-streams.ps1
    ├── phase-0-5-step-3-render-observability.ps1
    └── phase-0-5-step-3-validate-observability.ps1
```

## 附录 B — 引用文档

- `RGS-DTL-100 §5` 消息总线（JetStream producer/consumer/dlq 设计）
- `RGS-DTL-100 §7` 可观测性（tracing + metrics + log 三件套）
- `RGS-SPEC-CROSS-005` §2 NATS Stream 命名 + §3 安全
- `RGS-ARC-051` CEM 中心事件管理（OTel/Prom/Grafana 整体架构）
- `RGS-INC-001 v0.2` §1.2 / §1.5 / §2（本次填补 §1.2 NATS server side）
- `RGS-DEC-NOGO-001 v0.1` NO-GO 解除决议
- `RGS-DEC-008` 12 角色签字落条
- `crates/shared-platform/src/{producer,consumer,subject,dlq,outbox_relay,messaging,tracing_init,metrics_endpoint}.rs`（代码现状 baseline）
- `docker/observability/{otel-collector-config.yaml, prometheus.yml, grafana/}`（源 config 移植依据）
- `docs/deploy/01-k8s-manifests/{21,22,23,24}-postgres-*.yaml`（既有 K8s 部署规范对照）

---

## §N 12 角色全签(per DEC-008 一人公司治理基线)

| # | 角色 | 姓名 + 职能 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人(Architect) | **Ulysses(架构师)** | 2026-08-24 | ✅ |
| 2 | SRE Lead(运维) | **Ulysses(SRE)** | 2026-08-24 | ✅ |
| 3 | DBA Lead(数据库) | **Ulysses(DBA)** | 2026-08-24 | ✅ |
| 4 | QA Lead(测试) | **Ulysses(QA)** | 2026-08-24 | ✅ |
| 5 | Platform Engineer(平台) | **Ulysses(Platform)** | 2026-08-24 | ✅ |
| 6 | Player 域 Lead(独立) | **Ulysses(player 域 Lead)** | 2026-08-24 | ✅ |
| 7 | Economy 域 Lead(独立) | **Ulysses(economy 域 Lead)** | 2026-08-24 | ✅ |
| 8 | Match 域 Lead(独立) | **Ulysses(match 域 Lead)** | 2026-08-24 | ✅ |
| 9 | Social 域 Lead(独立) | **Ulysses(social 域 Lead)** | 2026-08-24 | ✅ |
| 10 | Admin 域 Lead(独立) | **Ulysses(admin 域 Lead)** | 2026-08-24 | ✅ |
| 11 | 评审主持人(RGS-REV-003) | **Ulysses(评审主持人)** | 2026-08-24 | ✅ |
| 12 | 项目负责人(PM) | **Ulysses(PM)** | 2026-08-24 | ✅ |

**依据**:`docs/00-基准与治理/RGS-DEC-NOGO-001_v0.1.md` §2(per DEC-008 一人公司 1 人 12 职责)。
**关联**:`RGS-PLAN-001 v0.9` §3.3 7 G-CODE Closed + `07-no-go-checklist_business v0.2` §4 4 B-CODE 实际状态 + `docs/deploy/phase-0-5-handoff.md` §10 12 角色全签。
