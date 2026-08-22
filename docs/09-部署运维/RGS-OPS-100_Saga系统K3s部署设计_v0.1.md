# RGS-OPS-100 Saga 系统 K3s 部署设计

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OPS-100 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-21 |
| 最终更新日 | 2026-08-21 |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-REQ-100 / RGS-BAS-100 / RGS-DTL-100~102 / RGS-GOBS-100（同侪 可观测性）/ RGS-SEC-100（同侪 安全审计）/ `docs/deploy/01-k8s-manifests/` |
| 配套标准 | IPA 共通フレーム 2013 + 150 工程日本 SI 业界标准；V 模型映射：UT ↔ DTL |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。3 部署 Profile（Minimal/Standard/HA）+ K3s NetworkPolicy + 资源配比 + DB 隔离 + ServiceAccount/RBAC。 |

---

## 0. 文档目的

定义 Saga 系统的 K3s 部署：

1. 3 种部署 Profile（Minimal / Standard / HA）+ 资源清单
2. 每个组件的 K8s 资源（Deployment / Service / ConfigMap / Secret / PDB / HPA / PVC / NetworkPolicy / ServiceAccount）按需
3. NetworkPolicy 东西向隔离
4. Database 隔离（per Service 独立 schema/user）
5. ServiceAccount + RBAC

**约束**：

- per DEC-010：k3s native in WSL2（dev）/ 可迁移到生产 k3s
- per BR-111：纯开源 Apache-2.0/MIT/BSD
- 不绑云厂商 / 不绑 Redis Enterprise / 不绑闭源事务协调器

---

## 1. 3 部署 Profile 总览

| 维度 | Minimal (dev / 1 人公司) | Standard (staging / 小生产) | High Availability (正式生产) |
|---|---|---|---|
| **K3s 节点数** | 1 (单节点 control-plane) | 3 (1 control + 2 agent) | 5+ (3 control + 2+ agent) |
| **CPU 总额** | 4 | 16 | 32+ |
| **内存总额** | 8 GB | 32 GB | 64+ GB |
| **存储总额** | 50 GB SSD | 500 GB SSD | 1+ TB SSD + WAL-G S3 |
| **PostgreSQL** | 1 instance (k3s pod) | 1 primary + 1 streaming replica | 1 primary + 2 replicas + WAL-G |
| **NATS JetStream** | 1 node | 3 nodes (Raft) | 3 nodes (Raft) |
| **Saga Runtime** | 1 replica | 3 replicas | 5+ replicas |
| **每个域微服务** | 1 replica | 2 replicas | 3+ replicas + HPA |
| **cluster-ops** | 1 replica | 3 replicas (Active-Active per ADR-0052) | 3 replicas (禁 HPA) |
| **Prometheus** | 1 instance (轻量) | 1 instance + 远程存储 | 完整 Prometheus + Thanos |
| **Loki / Tempo** | (可选, 关闭) | 单实例 | 集群模式 |
| **AlertManager** | (可选) | 单实例 | HA 3 实例 |
| **网络插件** | Flannel VXLAN | Calico VXLAN | Calico BGP |
| **Storage Class** | local-path | local-path + NFS | local-path + Longhorn / Rook-Ceph |
| **入口** | (可选) | Traefik | Traefik + cert-manager + external-dns |
| **备份** | pg_dump 本地 | pg_dump + S3 | pg_basebackup + WAL-G + PITR |

**资源公式**（per profile）：

```
CPU = 2 * (num_pods) + 2 (control plane overhead)
Mem = 1 GB * (num_pods) + 2 GB (control plane) + 4 GB (PG + NATS)
```

**Minimal profile 实际资源**：

```
Pods ≈ 12 (5 域 + cluster-ops + shared-platform + 2 gateway + Saga + NATS + PG)
CPU = 2*12 + 2 = 26 (但 K3s 实际只占 1-2 核) → 4 CPU 总
Mem = 1*12 + 2 + 4 = 18 GB (但实际只占 4-6 GB) → 8 GB 总
```

---

## 2. Minimal Profile K3s 资源清单

### 2.1 Namespace

```yaml
# 00-namespace.yaml (per existing docs/deploy/01-k8s-manifests/00-namespace.yaml)
apiVersion: v1
kind: Namespace
metadata:
  name: rust-game-server
  labels:
    app.kubernetes.io/part-of: rust-game-server
    app.kubernetes.io/managed-by: kustomize
```

### 2.2 Saga Runtime

```yaml
# 30-saga-runtime.yaml (per existing convention)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: saga-runtime
  namespace: rust-game-server
  labels:
    app.kubernetes.io/name: saga-runtime
    app.kubernetes.io/component: orchestration
spec:
  # Minimal: 1 replica; Standard+: 3+ replicas
  replicas: 1
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0  # 保证 saga-runtime 永远可用
  selector:
    matchLabels:
      app.kubernetes.io/name: saga-runtime
  template:
    metadata:
      labels:
        app.kubernetes.io/name: saga-runtime
        app.kubernetes.io/component: orchestration
    spec:
      serviceAccountName: saga-runtime
      containers:
        - name: saga-runtime
          image: PLACEHOLDER_SAGA_RUNTIME_IMAGE
          imagePullPolicy: IfNotPresent
          ports:
            - name: grpc
              containerPort: 50051
            - name: metrics
              containerPort: 9090
          env:
            - name: RUST_LOG
              value: info
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: cluster-ops-db-secret
                  key: url
            - name: NATS_URL
              valueFrom:
                configMapKeyRef:
                  name: nats-config
                  key: url
            - name: POD_ID
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
          resources:
            requests:
              cpu: 250m
              memory: 512Mi
            limits:
              cpu: 1000m
              memory: 1Gi
          livenessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 5
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: saga-runtime
  namespace: rust-game-server
spec:
  type: ClusterIP
  selector:
    app.kubernetes.io/name: saga-runtime
  ports:
    - name: grpc
      port: 50051
      targetPort: 50051
    - name: metrics
      port: 9090
      targetPort: 9090
---
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: saga-runtime-pdb
  namespace: rust-game-server
spec:
  minAvailable: 1  # Minimal: 1; Standard+: 2
  selector:
    matchLabels:
      app.kubernetes.io/name: saga-runtime
```

### 2.3 NATS JetStream

```yaml
# 40-nats-jetstream.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: nats
  namespace: rust-game-server
spec:
  serviceName: nats
  replicas: 1  # Minimal: 1; Standard+: 3 (Raft)
  selector:
    matchLabels:
      app.kubernetes.io/name: nats
  template:
    metadata:
      labels:
        app.kubernetes.io/name: nats
    spec:
      containers:
        - name: nats
          image: nats:2.10-alpine
          args:
            - --jetstream
            - --store_dir=/data
            - --cluster_name=rgs-nats
            - --cluster=nats-0.nats:6222
            - --http_port=8222
          ports:
            - name: client
              containerPort: 4222
            - name: cluster
              containerPort: 6222
            - name: monitor
              containerPort: 8222
            - name: metrics
              containerPort: 7777
          volumeMounts:
            - name: data
              mountPath: /data
          resources:
            requests:
              cpu: 100m
              memory: 256Mi
            limits:
              cpu: 1000m
              memory: 1Gi
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes: [ReadWriteOnce]
        storageClassName: local-path
        resources:
          requests:
            storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: nats
  namespace: rust-game-server
spec:
  type: ClusterIP
  clusterIP: None  # Headless for StatefulSet
  selector:
    app.kubernetes.io/name: nats
  ports:
    - name: client
      port: 4222
    - name: cluster
      port: 6222
    - name: monitor
      port: 8222
```

### 2.4 5 域 + cluster-ops + shared-platform + 2 Gateway

```yaml
# 50-account-service.yaml (示例, 其余域类似)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: account-service
  namespace: rust-game-server
spec:
  replicas: 1  # Minimal: 1; Standard+: 2-3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app.kubernetes.io/name: account-service
  template:
    metadata:
      labels:
        app.kubernetes.io/name: account-service
        app.kubernetes.io/component: domain-service
    spec:
      serviceAccountName: account-service
      containers:
        - name: account
          image: PLACEHOLDER_ACCOUNT_IMAGE
          ports:
            - name: grpc
              containerPort: 50051
            - name: metrics
              containerPort: 9090
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: account-db-secret
                  key: url
            - name: NATS_URL
              value: nats://nats:4222
          resources:
            requests:
              cpu: 250m
              memory: 256Mi
            limits:
              cpu: 1000m
              memory: 1Gi
---
# 50b-economy-service.yaml — 略（同模式）
# 50c-character-service.yaml — 略
# 50d-inventory-service.yaml — 略
# 50e-match-service.yaml — 略
# 50f-mail-service.yaml — 略
# 50g-guild-service.yaml — 略
# 50h-cluster-ops.yaml — ActiveActive 3 replicas (per ADR-0052)
# 50i-shared-platform.yaml — QUIC edge / gRPC ingress / OTel collector

# Gateway 模式相同
# 60-game-gateway.yaml
# 60-admin-gateway.yaml
```

### 2.5 PostgreSQL（per existing 01-k8s-manifests/23-postgres-statefulset.yaml）

```yaml
# 23-postgres-statefulset.yaml (Minimal 1 replica, Standard+ 用 Bitnami PostgreSQL HA)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: rust-game-server
spec:
  replicas: 1
  strategy:
    type: Recreate
  selector:
    matchLabels:
      app.kubernetes.io/name: postgres
  template:
    spec:
      serviceAccountName: postgres
      containers:
        - name: postgres
          image: postgres:18.6
          ports:
            - name: postgres
              containerPort: 5432
          env:
            - name: POSTGRES_USER
              valueFrom:
                secretKeyRef: ...
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef: ...
            - name: PGDATA
              value: /var/lib/postgresql/data/pgdata
          volumeMounts:
            - name: postgres-data
              mountPath: /var/lib/postgresql/data
            - name: postgres-config
              mountPath: /etc/postgresql/postgresql.conf
              subPath: postgresql.conf
          resources:
            requests:
              cpu: 500m
              memory: 1Gi
            limits:
              cpu: 2000m
              memory: 4Gi
      volumes:
        - name: postgres-data
          persistentVolumeClaim:
            claimName: postgres-data-pvc
        - name: postgres-config
          configMap:
            name: postgres-config
---
# 6 DBs created by init SQL
# - player_db / economy_db / match_db / social_db / admin_db / cluster_ops_db
```

### 2.6 可观测性栈（Minimal 简化）

```yaml
# 70-otel-collector.yaml
# 71-prometheus.yaml (1 instance)
# 72-grafana.yaml (1 instance, embedded Loki + Tempo)
```

---

## 3. NetworkPolicy 东西向隔离

```yaml
# 80-networkpolicy.yaml
---
# Default: deny all (除已声明)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: rust-game-server
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
# 5 域 + cluster-ops 可以出到 PostgreSQL
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-services-to-postgres
  namespace: rust-game-server
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/component: domain-service
  policyTypes:
    - Egress
  egress:
    - to:
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: postgres
      ports:
        - port: 5432
          protocol: TCP
---
# Saga Runtime 可以出到所有 domain + NATS
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-saga-runtime-egress
  namespace: rust-game-server
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: saga-runtime
  policyTypes:
    - Egress
  egress:
    - to:
        - podSelector:
            matchLabels:
              app.kubernetes.io/component: domain-service
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: nats
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: postgres
    - to:
        - namespaceSelector: {}  # DNS
      ports:
        - port: 53
          protocol: UDP
---
# Gateway 接受 Game Client / Admin UI
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-game-gateway-ingress
  namespace: rust-game-server
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: game-gateway
  policyTypes:
    - Ingress
  ingress:
    - ports:
        - port: 50051
          protocol: TCP
        - port: 443
          protocol: TCP
        - port: 443
          protocol: UDP  # QUIC
```

---

## 4. Database 隔离（per Service 独立 schema/user）

```sql
-- 5 域 + cluster-ops 各自独立 schema + user
-- 实际部署：shared cluster + separate database + separate user

-- player_db
CREATE DATABASE player_db;
CREATE USER player_user WITH PASSWORD 'PLACEHOLDER_PLAYER_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE player_db TO player_user;

-- economy_db
CREATE DATABASE economy_db;
CREATE USER economy_user WITH PASSWORD 'PLACEHOLDER_ECONOMY_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE economy_db TO economy_user;

-- match_db
CREATE DATABASE match_db;
CREATE USER match_user WITH PASSWORD 'PLACEHOLDER_MATCH_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE match_db TO match_user;

-- social_db
CREATE DATABASE social_db;
CREATE USER social_user WITH PASSWORD 'PLACEHOLDER_SOCIAL_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE social_db TO social_user;

-- admin_db
CREATE DATABASE admin_db;
CREATE USER admin_user WITH PASSWORD 'PLACEHOLDER_ADMIN_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE admin_db TO admin_user;

-- cluster_ops_db (含 saga_store 9 表 + PFAU + COC + 审计)
CREATE DATABASE cluster_ops_db;
CREATE USER cluster_ops_user WITH PASSWORD 'PLACEHOLDER_CLUSTER_OPS_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE cluster_ops_db TO cluster_ops_user;
```

**强约束**（per RGS-BAS-100 §7）：

- ❌ service-to-service direct SQL 访问
- ✅ 通过 gRPC + 业务接口访问其他服务
- ✅ Saga 协调通过 Saga Runtime + Command/Event

---

## 5. ServiceAccount + RBAC

```yaml
# 90-serviceaccounts.yaml
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: saga-runtime
  namespace: rust-game-server
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: account-service
  namespace: rust-game-server
---
# 5 域 + cluster-ops + 2 gateway + saga-runtime 各自独立 SA
# 每个 SA 只能访问自己需要的 Secret / ConfigMap
---
# RBAC: saga-runtime 可以读所有 Secret（需要读各域 DB URL）
# 5 域 SA 只能读自己的 Secret
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: saga-runtime-secret-reader
  namespace: rust-game-server
rules:
  - apiGroups: [""]
    resources: ["secrets"]
    resourceNames:
      - account-db-secret
      - character-db-secret
      - inventory-db-secret
      - economy-db-secret
      - match-db-secret
      - social-db-secret
      - mail-db-secret
      - guild-db-secret
      - cluster-ops-db-secret
    verbs: ["get", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: saga-runtime-secret-reader
  namespace: rust-game-server
subjects:
  - kind: ServiceAccount
    name: saga-runtime
    namespace: rust-game-server
roleRef:
  kind: Role
  name: saga-runtime-secret-reader
  apiGroup: rbac.authorization.k8s.io
```

---

## 6. HPA（仅 Standard+ profile）

```yaml
# 100-hpa.yaml (Minimal 不用)
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: account-service-hpa
  namespace: rust-game-server
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: account-service
  minReplicas: 2
  maxReplicas: 8
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
---
# cluster-ops 禁 HPA (per ADR-0052)
# saga-runtime 用固定 replicas + fence_token HA
```

---

## 7. Upgrade 兼容性

**Rolling Update 配置**（所有 Deployment）：

```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1
    maxUnavailable: 0  # 保证微服务始终可用
```

**Saga Definition 兼容性**：

- Saga Runtime 升级时，**old + new pod 同时运行**（maxSurge=1, maxUnavailable=0）
- 新 Saga 走新 pod（通过 Service endpoint 负载均衡）
- 在飞 Saga 由持有 fence_token 的 pod 继续
- Pod 缩容按 `pod.kubernetes.io/priority` 或 `saga_instance.owner_pod`

**PostgreSQL 升级**：

- 18.4 → 18.6：patch 升级，零停机（per DEC-009）
- 18.6 → 19.0：主版本升级，需要停机 / Logical Replication 蓝绿

---

## 8. 关联文档

- **基础**：`RGS-REQ-100` / `RGS-BAS-100` / `RGS-DTL-100~102`
- **同侪**：
  - `RGS-GOBS-100` Saga 可观测性设计
  - `RGS-SEC-100` GM 审计与 Saga 安全设计
- **现有 K8s 资源**：`docs/deploy/01-k8s-manifests/`（per DEC-010 PG manifest 已就位）
- **部署 SOP**：`docs/deploy/04-env-setup-sop.md`（per DEC-010 WSL2 k3s native）
- **现有架构决策**：`RGS-ADR-0052` Active-Active 固定 3 副本 / PFAU 原则 (per 功能原子升级) / CEM 中心事件管理 (per 事件路由)

---

## 9. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。3 部署 Profile (Minimal 4C8G / Standard 16C32G / HA 32+C64+G) + 完整 K3s 资源清单 (saga-runtime / nats / postgres / 5 域 / cluster-ops / 2 gateway) + NetworkPolicy 东西向隔离 + DB per Service (5 域 + cluster_ops 独立 schema/user) + ServiceAccount + RBAC + HPA + Upgrade 兼容。 |
