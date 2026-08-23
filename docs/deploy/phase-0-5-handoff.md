# Phase 0.5 SRE Handoff 提示词 v0.1

> **给接手 SRE 的快速上下文 + 待办清单**。读完这份 + 跑完 5 步接力 + 重跑 4 B-CODE 验证 = Phase 0.5 实质闭环。

---

## 0. 一句话当前状态

**NO-GO 形式上解除(per `RGS-DEC-NOGO-001 v0.1` 一人公司 12 角色全签),Phase 0.5 形式上完成(52 文件 / 6065 行入 main),4 B-CODE 实质 1 🟡 + 3 🔴,需 SRE 接力完成镜像推送 + K3s 实际部署。**

---

## 1. 你接手时的 git 状态(2026-08-24 07:30 main HEAD)

```
b44c589 [phase-0.5] step-6 总报告(主对话接手版,worker timeout 失败后整合)
48101aa [phase-0.5] RGS-PLAN-001 v0.8 → v0.9 升版 + 07-no-go-checklist_business v0.1 → v0.2
765930a [merge] WF-0.5-3: Phase 0.5 Step 4 (mTLS 7 Secret + 5/5 fail-closed PASS)
c5a0c9f [merge] WF-0.5-2: Phase 0.5 Step 2+3 (NATS + OTel/Prom/Grafana 18 manifest)
7046936 [merge] WF-0.5-1: Phase 0.5 Step 1+5 (5 域 manifest 实际值 + docker image 脚本)
fa6b07e [phase-0.5] NO-GO 解除决议(一人公司 12 角色全签) + 4 B-CODE 实测 log
```

**worktree 状态**:`git worktree list` 只剩主工作区 + 3 个你之前决定保留的 detached `rev-010-V{1..3}`。

---

## 2. 4 B-CODE 实际状态(2026-08-24 实测)

| B-CODE | 描述 | 状态 | 阻塞 |
|---|---|---|---|
| **B-CODE-01** | OTel + Prom + Grafana 3 套 K3s 部署 | 🟡 **部分** | 14 K8s resources apply OK / 3 Deployment Scaled / 3 PVC Bound / 0/3 Pod Running |
| **B-CODE-02** | player gRPC HealthCheck | 🔴 **失败** | 5 业务域镜像未推 + OTel 不 Running |
| **B-CODE-03** | login → session_epoch → player_db 落库 | 🔴 **失败** | 同 B-CODE-02 |
| **B-CODE-04** | 跨域 trace 串联 | 🔴 **失败** | OTel Collector ImagePullBackOff + 5 业务域镜像未推 |

---

## 3. 4 BLOCK 失败原因(per `docs/deploy/phase-0-5-step-6-report.md` §4)

| BLOCK | 描述 | 解决 |
|---|---|---|
| **BLOCK-001** | gcr.io:443 + docker.io:443 防火墙拦截 | 改推 ghcr.io(已可达)|
| **BLOCK-002** | ghcr.io:443 OK 但 docker login 无 PAT | 需 GITHUB_TOKEN 或 GHCR_PAT |
| **BLOCK-003** | Step 6 worker `Request timed out` | 已由主对话接手补完,无需处理 |
| **BLOCK-004** | 工具链 5 项缺失(cargo-deny/audit/llvm-cov/helm/kubectl)| 见 §5.1 |

---

## 4. 已交付资产(52 文件 / 6065 行,全在 main)

### K8s manifest(36 文件)
- `docs/deploy/01-k8s-manifests/00-namespace.yaml` + `00-postgres-sa.yaml` + `20-24-postgres-*.yaml`(已部署:postgres pod Running 42h)
- `docs/deploy/01-k8s-manifests/{01..05,06}-*-{deployment,service}.yaml`(5 业务域 + cluster-ops 实际值,Step 1 落地)
- `docs/deploy/01-k8s-manifests/07-shared-platform.yaml` + `08-configmap-template.yaml` + `09-secret-template.yaml` + `10-rbac-template.yaml`
- `docs/deploy/01-k8s-manifests/30-nats-{pvc,configmap,sa,statefulset,service,networkpolicy}.yaml`(6 文件,已 apply 但 Pod ImagePullBackOff)
- `docs/deploy/01-k8s-manifests/40-otel-collector-{configmap,sa,deployment,service}.yaml`(4 文件,已 apply 但 Pod ImagePullBackOff)
- `docs/deploy/01-k8s-manifests/41-prometheus-{configmap,pvc,deployment,service}.yaml`(4 文件,已 apply 但 Pod ImagePullBackOff)
- `docs/deploy/01-k8s-manifests/42-grafana-{configmap,pvc,deployment,service}.yaml`(4 文件,已 apply 但 Pod ImagePullBackOff)
- `docs/deploy/01-k8s-manifests/50-secret-{ca,player-tls,economy-tls,match-tls,social-tls,admin-tls,cluster-ops-tls}.yaml`(7 文件,占位,需 `phase-0-5-step-4-render-secrets.ps1` 注入真实证书)

### ps1 部署脚本(11 文件,保留在 `docs/deploy/`)
- `phase-0-5-step-1-render-manifests.ps1`(决策矩阵 → 5 域 manifest 渲染)
- `phase-0-5-step-1-validate-manifests.ps1` + `phase-0-5-step-1-validate-helper.py`(YAML 结构 + Python PyYAML 验证)
- `phase-0-5-step-5-build-images.ps1`(6 业务域 docker buildx multi-arch + ghcr.io push)
- `phase-0-5-step-2-render-nats.ps1` + `phase-0-5-step-2-init-streams.ps1`(NATS manifest 渲染 + 6 Stream 初始化)
- `phase-0-5-step-3-render-observability.ps1` + `phase-0-5-step-3-validate-observability.ps1`(OTel/Prom/Grafana 渲染 + 验证)
- `phase-0-5-step-4-gen-certs.ps1`(rgs-certgen 730 天,6 域 + CA)
- `phase-0-5-step-4-render-secrets.ps1`(7 Secret yaml base64 注入)
- `phase-0-5-step-4-patch-deployments.ps1`(6 域 env+volumes+volumeMounts 增量 + merge guide)
- `phase-0-5-step-4-validate-fail-closed.ps1`(5 域 binary fail-closed 验证,**已实跑 5/5 PASS**)

### 文档
- `docs/00-基准与治理/RGS-DEC-NOGO-001_一人公司NO-GO解除决议_v0.1.md`
- `docs/12-工作流/RGS-PLAN-001_项目实施计划_v0.9.md`(v0.8 → v0.9 升版,7 G-CODE 全 Closed)
- `docs/deploy/07-no-go-checklist_business_v0.2.md`(4 B-CODE 实际状态)
- `docs/deploy/phase-0-5-step-6-report.md`(Step 6 总报告)

### 实测 log
- `docs/deploy/b1-otel-pod-up.log` / `b2-player-grpc-healthcheck.log` / `b3-session-pg-trace.log` / `b4-cross-domain-trace.log`(反映 2026-08-24 实际状态)

---

## 5. SRE 5 步接力清单(预计 2-3 小时)

### Step 1:工具链补齐(30 分钟)

```bash
# cargo 工具链 3 项
cargo install cargo-deny --locked
cargo install cargo-audit --locked
cargo install cargo-llvm-cov --locked

# helm v3.10+
curl -fsSL -o get_helm.sh https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3
bash get_helm.sh
helm version  # 期望 ≥ v3.10

# kubectl ≥ v1.30
curl -LO "https://dl.k8s.io/release/v1.30.0/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/
kubectl version --client  # 期望 ≥ v1.30

# K3s config 权限(WSL2 默认仅 root 可读)
sudo chmod 644 /etc/rancher/k3s/k3s.yaml
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
kubectl get nodes  # 期望 ulyssespc Ready
```

**验证**:`cargo deny check` + `cargo audit` + `cargo llvm-cov --workspace` + `helm version` + `kubectl version --client` 全 0 退出 + 期望版本。

### Step 2:6 业务域镜像推送 ghcr.io(1 小时)

```bash
# 登录 ghcr.io
docker login ghcr.io -u <github-username> -p <GHCR_PAT>
# 或:echo $GHCR_PAT | docker login ghcr.io -u <user> --password-stdin

# 跑 build 脚本
cd /d/RustGameServer
pwsh -File docs/deploy/phase-0-5-step-5-build-images.ps1
# 期望产出 6 个镜像(每个 amd64 + arm64):ghcr.io/rust-game-server/{player,economy,match,social,admin,cluster-ops}-service:0.1.0 + git-sha tag

# 验证
docker images | grep rust-game-server
# 期望 6 个 image,每个 ~8MB(已实测)
```

**已知 BLOCK**:build 时 base image 拉取可能受 gcr.io 防火墙影响——**fallback**:用 `docker buildx --cache-from=type=registry,ref=ghcr.io/rust-game-server/cache:latest` 复用 ghcr.io 已缓存层。

**验证**:6 镜像 push 成功(`docker push` 0 退出)+ `docker pull ghcr.io/rust-game-server/player-service:0.1.0` 能拉回。

### Step 3:K3s imagePullSecret + namespace 配通(10 分钟)

```bash
# 实际 namespace 是 rust-game-server(per Step 6 worker 实测)
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
kubectl create namespace rust-game-server --dry-run=client -o yaml | kubectl apply -f -

# 创建 imagePullSecret
kubectl create secret docker-registry ghcr-pull \
  --docker-server=ghcr.io \
  --docker-username=<github-username> \
  --docker-password=<GHCR_PAT> \
  -n rust-game-server

# 验证
kubectl get secret -n rust-game-server ghcr-pull -o yaml
```

**已知问题**:Step 1+5 worker 写的 5 域 Deployment yaml **未含** `imagePullSecrets: [{name: ghcr-pull}]` 字段——需要在 apply 前**手动加**这一段(或改 `phase-0-5-step-1-render-manifests.ps1` 加 `imagePullSecrets` 字段重渲染)。

**修复方案(选一)**:
- A. 编辑 `01-player-deployment.yaml` 等 5 域 yaml,在 `spec.template.spec` 加:
  ```yaml
  imagePullSecrets:
  - name: ghcr-pull
  ```
- B. 改 `phase-0-5-step-1-render-manifests.ps1` 加 `imagePullSecrets` 字段 → 重跑 `pwsh -File phase-0-5-step-1-render-manifests.ps1` 重新生成 5 域 yaml

### Step 4:apply 5 业务域 Deployment + 7 Secret(15 分钟)

```bash
# 1. 渲染 5 域 manifest(如有改动)
pwsh -File docs/deploy/phase-0-5-step-1-render-manifests.ps1

# 2. 渲染 7 Secret(注入 rgs-certgen 生成的真实证书)
# 先生成证书
pwsh -File docs/deploy/phase-0-5-step-4-gen-certs.ps1
# 期望产出 target/dev-certs/{6 域}.crt.pem + {6 域}.key.pem + ca.crt.pem + ca.key.pem
pwsh -File docs/deploy/phase-0-5-step-4-render-secrets.ps1
# 期望产出 7 个真实注入证书的 Secret yaml

# 3. apply 所有
kubectl apply -f docs/deploy/01-k8s-manifests/01-player-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/01-player-service.yaml
# ... 重复 5 域
kubectl apply -f docs/deploy/01-k8s-manifests/50-secret-ca.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/50-secret-player-tls.yaml
# ... 重复 6 域

# 4. 验证 Pod 启动
kubectl get pods -n rust-game-server -l app=player-service
# 期望 1/1 Running
sleep 60  # 等所有 5 业务域 Pod 起来
kubectl get pods -n rust-game-server
# 期望 5 业务域 + NATS + OTel + Prom + Grafana + postgres 全部 Running
```

**已知问题**:`Grafana` 缺 `grafana-admin-secret`(per Step 2+3 worker 报告 RISK-DEPLOY-006)——apply 前手动创建:
```bash
kubectl create secret generic grafana-admin-secret \
  --from-literal=admin-user=admin \
  --from-literal=admin-password=$(openssl rand -base64 32) \
  -n rust-game-server
```

**deny-all NetworkPolicy 误拦**(per Step 1+5 worker RISK-DEPLOY-005)——已显式 allow PFAU TCP 9090 + K8s API 10.43.0.1:443,无需额外操作。

### Step 5:重跑 4 B-CODE 实测验证(30 分钟)

```bash
# 删旧 log
rm docs/deploy/b1-otel-pod-up.log docs/deploy/b2-player-grpc-healthcheck.log \
   docs/deploy/b3-session-pg-trace.log docs/deploy/b4-cross-domain-trace.log

# 跑 B-CODE-01: OTel + Prom + Grafana 健康检查
kubectl get pods -n rust-game-server -l app.kubernetes.io/component=observability
# 期望 3/3 Running
curl http://prometheus.rust-game-server.svc.cluster.local:9090/-/ready
# 期望 "Prometheus is Ready."
curl http://grafana.rust-game-server.svc.cluster.local:3000/api/health
# 期望 {"database":"ok"} 200
# 把输出写到 docs/deploy/b1-otel-pod-up.log

# 跑 B-CODE-02: player gRPC HealthCheck
kubectl get pods -n rust-game-server -l app=player-service
# 期望 1/1 Running
# 用 grpcurl(若未装:cargo install grpcurl):
grpcurl -insecure player-service.rust-game-server.svc.cluster.local:50051 list
# 期望列出 player.v1.PlayerService 的 RPC 方法
# 把输出写到 docs/deploy/b2-player-grpc-healthcheck.log

# 跑 B-CODE-03: login → session → DB 落库
# 触发 login(需有 player 域 Login RPC,见 crates/player-service/src/service.rs)
# 验证 player_db.sessions 表
PGPASSWORD=player psql -h postgres.rust-game-server.svc.cluster.local -U player -d player_db \
  -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 1"
# 期望返回 1 行,带 created_at + session_epoch
# 把输出写到 docs/deploy/b3-session-pg-trace.log

# 跑 B-CODE-04: 跨域 trace
# player → economy 跨域 gRPC 调用(需 OTel Collector 接收 trace)
# 验证 trace_id 在 Grafana/Tempo 可见
# 把输出写到 docs/deploy/b4-cross-domain-trace.log
```

**4 B-CODE 全部 🟢 Closed** → 升文档:
```bash
# 升 07-no-go-checklist_business v0.2 → v0.3
# 4 B-CODE 状态: B-CODE-01/02/03/04 全部 🟢
git add docs/deploy/07-no-go-checklist_business_v0.2.md
git mv docs/deploy/07-no-go-checklist_business_v0.2.md docs/deploy/07-no-go-checklist_business_v0.3.md
# 编辑 v0.3 把 4 B-CODE 状态全改为 🟢 Closed,加 v0.3 修订历史
git add docs/deploy/07-no-go-checklist_business_v0.3.md
git commit -m "[phase-0.5] 4 B-CODE 全部 Closed(实质)"

# 升 RGS-PLAN-001 v0.9 → v1.0
# 4 B-CODE Closed + Phase 0.5 实质完成 + 进 PH-1
git mv docs/12-工作流/RGS-PLAN-001_项目实施计划_v0.9.md \
       docs/12-工作流/RGS-PLAN-001_项目实施计划_v1.0.md
# 编辑 v1.0:Phase 0.5 实质完成 + 进入 PH-1 授权
git add docs/12-工作流/RGS-PLAN-001_项目实施计划_v1.0.md
git commit -m "[plan] RGS-PLAN-001 v0.9 → v1.0: Phase 0.5 实质完成 + 进 PH-1"
```

---

## 6. 验证 checklist(Phase 0.5 闭环判定)

- [ ] 工具链 5 项实测 PASS(`cargo deny check` + `cargo audit` + `cargo llvm-cov` + `helm version` + `kubectl version --client`)
- [ ] 6 业务域镜像 push ghcr.io 成功 + K3s imagePullSecret 配通
- [ ] `kubectl get pods -n rust-game-server` 期望 5 业务域 + cluster-ops + NATS + OTel + Prom + Grafana + postgres 全部 1/1 Running
- [ ] 4 份 B-CODE log 重写,内容反映实际 4/4 🟢
- [ ] `07-no-go-checklist_business_v0.3.md` 4 B-CODE 全 🟢
- [ ] `RGS-PLAN-001_v1.0.md` Phase 0.5 实质完成 + 进 PH-1

---

## 7. 回退 / 风险

| 风险 | 触发 | 回退 |
|---|---|---|
| image push 失败 | GHCR_PAT 失效 / ghcr.io 限流 | 改用自建 registry / 暂用 `imagePullPolicy: Never` + 节点预加载 |
| Pod CrashLoopBackOff | migration 失败 / DB 连接错误 | `kubectl logs -n rust-game-server <pod> --previous` + `kubectl describe pod` |
| OTel 链路断裂 | traceparent 注入失败 | 检查 `shared_platform::grpc_tracing` + OTel Collector logs |
| mTLS fail-closed 误拦合法流量 | PEM 文件不匹配 | 重跑 `phase-0-5-step-4-gen-certs.ps1` + 重新 `phase-0-5-step-4-render-secrets.ps1` |

---

## 8. 关键引用(必读)

- **NO-GO 形式解除依据**:`docs/00-基准与治理/RGS-DEC-NOGO-001_一人公司NO-GO解除决议_v0.1.md`
- **Phase 0.5 启动计划**:`docs/01-核心架构与设计模式/RGS-INC-002_Phase_0.5_启动计划_v0.1.md`
- **现状基线 v0.2 勘误**:`docs/01-核心架构与设计模式/RGS-INC-001_增量式架构升级_Function与WASM演进方案_v0.1.md` §1.4/§1.5/§2
- **业务 NO-GO 现状**:`docs/deploy/07-no-go-checklist_business_v0.2.md`
- **Step 1+5 报告**(5 域 manifest 实际值 + docker image 脚本):`docs/deploy/01-k8s-manifests/` 17 文件 + `phase-0-5-step-1+5-*.ps1` + `PHASE-0-5-STEP-1+5-REPORT.md`(worktree 清理时已 commit 进 main)
- **Step 2+3 报告**(NATS + OTel/Prom/Grafana):`PHASE-0-5-STEP-2+3-REPORT.md` + 18 manifest
- **Step 4 报告**(mTLS + Secret + fail-closed):`PHASE-0-5-STEP-4-REPORT.md` + 7 Secret + 4 ps1
- **Step 6 总报告**:`docs/deploy/phase-0-5-step-6-report.md`
- **本次 handoff 提示词**:`docs/deploy/phase-0-5-handoff.md`(本文件)
- **WBS 任务进度表**:`docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.3.md`
- **worktree 规范**:`docs/12-工作流/RGS-WT-001_GitWorktree隔离开发方案.md` §11

---

## 9. 联系上下文

| 角色 | 姓名 | 备注 |
|---|---|---|
| 项目所有者 / 全 12 角色 | **Ulysses** | per DEC-008 一人公司治理基线 |
| 工具 | pwsh 7.0+ / git / cargo / kubectl / helm / docker | |
| K3s 节点 | ulyssespc / 172.28.176.169 | control-plane 1.36.3+k3s1 |
| Postgres | postgres-5bb9bb647d-6wfv4 | Running 42h(baseline) |
| 命名空间 | rust-game-server(实测,非 `rgs` 占位)| |
| 工作时间窗 | 2026-08-23 06:30 ~ 2026-08-24 07:30 UTC+9(约 25 小时)| |

---

**handoff 结束。SRE 接力 5 步清单预计 2-3 小时,完成后 Phase 0.5 实质闭环 → 进 PH-1(WF-1 实施)。**
