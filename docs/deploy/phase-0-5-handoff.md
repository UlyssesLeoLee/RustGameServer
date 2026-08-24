# Phase 0.5 SRE Handoff 提示词 v0.1

> **给接手 SRE 的快速上下文 + 待办清单**。读完这份 + 跑完 5 步接力 + 重跑 4 B-CODE 验证 = Phase 0.5 实质闭环。

---

## 0. 一句话当前状态

**NO-GO 形式上解除(per `RGS-DEC-NOGO-001 v0.1` 一人公司 12 角色全签),Phase 0.5 形式上完成(52 文件 / 6065 行入 main),4 B-CODE 实质 1 🟡 + 3 🔴,需 SRE 接力完成镜像推送 + K3s 实际部署。**

**2026-08-24 追加(主对话)**:部署验证中发现的 CrashLoopBackOff 结构性根因(k8s 原生 `grpc:` 探针无法出示 mTLS 客户端证书 + Health 服务未注册)已定位并修复,详见 §12。**修复代码尚未 commit**,SRE 接力前请先确认 §12 改动已入库(或由主对话在你接手前补 commit)。

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

**Step 2.1:获取 GHCR_PAT(5 分钟)**

```bash
# 1. 打开 https://github.com/settings/tokens
# 2. 点 "Generate new token" → "Generate new token (classic)"
# 3. 配置:
#    Note:        rgs-phase-0.5-image-push
#    Expiration:  90 days
#    Scopes:      ✅ write:packages  (push 镜像)
#                 ✅ read:packages   (K3s node pull)
# 4. 点 "Generate token" → 立即复制(只显示一次!)
#    token 格式: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**Step 2.2:登录 ghcr.io + 跑 build 脚本**

```bash
# 登录 ghcr.io(在 WSL2 内,不是 Windows)
export GHCR_PAT='ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx'
echo $GHCR_PAT | docker login ghcr.io -u <github-username> --password-stdin
# 期望: Login Succeeded

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
| Pod 起了但探针一直不过(CrashLoopBackOff,非 migration/DB 原因) | k8s 原生 `grpc:` 探针无法出示 mTLS 客户端证书,与 fail-closed mTLS 结构性冲突 | **已于 2026-08-24 修复**,见 §12(`RGS-OPS-101`)。若 SRE 拉到的代码没有这次修复,现象是:日志显示 `binding gRPC server` 成功但探针始终超时——按 §12 方案重新应用即可,勿误判为证书或 DB 问题 |

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
- **gRPC 健康探针 mTLS 兼容性修复设计**(2026-08-24 追加):`docs/09-部署运维/RGS-OPS-101_gRPC健康探针mTLS兼容性修复设计_v0.1.md`(需求/基本设计/详细设计/实装规格/实施计划,见 §12)

---

## 9. 联系上下文

| 角色 | 姓名 | 备注 |
|---|---|---|
| 项目所有者 / 全 12 角色 | **Ulysses** | per DEC-008 一人公司治理基线 |
| 工具 | pwsh 7.0+ / git / cargo / kubectl / helm / docker | |
| K3s 节点 | ulyssespc / 172.28.176.169 | control-plane 1.36.3+k3s1 |
| Postgres | postgres-5bb9bb647d-6wfv4 | Running 42h(baseline) |
| 命名空间 | rust-game-server(实测,非 `rgs` 占位)| |
| 工作时间窗 | 2026-08-23 06:30 ~ 2026-08-24 08:30 UTC+9(约 26 小时)| |

---

## 10. 12 角色全签(per DEC-008 一人公司治理基线)

> **本 handoff 文档 + 本次 Phase 0.5 全部交付物** 由 Ulysses 兼任 12 类角色实际签署。

| # | 角色 | 姓名 + 职能 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人(Architect) | **Ulysses(架构师)** | 2026-08-24 | ✅ 实际签:确认 Phase 0.5 形式完成 + handoff 闭环 |
| 2 | SRE Lead(运维) | **Ulysses(SRE)** | 2026-08-24 | ✅ 实际签:K3s 1.36.3 + 5 DB + Phase 0.5 SRE 接力清单 |
| 3 | DBA Lead(数据库) | **Ulysses(DBA)** | 2026-08-24 | ✅ 实际签:PG 18.6 6 库 + migration 0 失败 |
| 4 | QA Lead(测试) | **Ulysses(QA)** | 2026-08-24 | ✅ 实际签:4 B-CODE 实测 log 全部生成(1 🟡 + 3 🔴) |
| 5 | Platform Engineer(平台) | **Ulysses(Platform)** | 2026-08-24 | ✅ 实际签:18 套可观测性 manifest + 5 域 binary 编译 OK |
| 6 | Player 域 Lead(独立) | **Ulysses(player 域 Lead)** | 2026-08-24 | ✅ 实际签:player DTL-018 边界冻结 + B-CODE-02/03 镜像依赖 |
| 7 | Economy 域 Lead(独立) | **Ulysses(economy 域 Lead)** | 2026-08-24 | ✅ 实际签:Q-003 Saga 决策 + DTL-015/016 + B-CODE-04 跨域 trace |
| 8 | Match 域 Lead(独立) | **Ulysses(match 域 Lead)** | 2026-08-24 | ✅ 实际签:DTL-026 边界冻结 |
| 9 | Social 域 Lead(独立) | **Ulysses(social 域 Lead)** | 2026-08-24 | ✅ 实际签:DTL-019/020 边界冻结 |
| 10 | Admin 域 Lead(独立) | **Ulysses(admin 域 Lead)** | 2026-08-24 | ✅ 实际签:DTL-031 v0.2 已审 + ClusterOps 代码就位 |
| 11 | 评审主持人(RGS-REV-003) | **Ulysses(评审主持人)** | 2026-08-24 | ✅ 实际签:REV-003 §7.3 12 类签字闭环 |
| 12 | 项目负责人(PM) | **Ulysses(PM)** | 2026-08-24 | ✅ 实际签:范围 + 风险接受 + 资源(含 5 域独立 Lead 编制)+ Phase 1 实施授权 |

**依据**:`docs/00-基准与治理/RGS-DEC-NOGO-001_v0.1.md` §2 12 类签字(per DEC-008 一人公司 = 1 人 12 职责 = 真实人真实职责)。
**关联**:`RGS-PLAN-001 v0.9` §3.3 7 G-CODE 全部 Closed + `07-no-go-checklist_business v0.2` §4 4 B-CODE 实际状态。
**接受代价**:Q-003 跨域事务"1 人自审自批"已知风险 → CI 强约束 + 自动化测试 ≥ 80% + 自我 PR review + OTel 链路 覆盖。

---

## 11. 已知未完成事项(per 2026-08-24 18:25 主对话盘点后)

> **当前状态**(per 2026-08-24 20:53 主对话盘点,5 worker + 4 follow-up merge commit 落地):**12 项子事项(11.1~11.12)**,其中 **10 项 closed**(§11.3/11.4/11.5/11.6/11.7/11.8/11.9/11.10/11.11/11.12 per `195bfae` + `6ae469b` + `7c2db70` + `32333d2` + `7d68f73` + `452c3b2` + `65b11a7` + `77d06c2` + `f9512cc` 共 9 commit)+ **2 项仍 open**。
>
> **修订历史**:
> - 2026-08-24 20:42 主对话盘点:§11.6/§11.7/§11.8 标 Closed(per 6ae469b + 7c2db70 + 32333d2,3 项已在 phase-0-5 期间完成但 handoff 文档未同步);剩余 §11.1/§11.2 真阻塞,等 SRE 接力 + G-CODE-06 实测
> - 2026-08-24 19:00 修订:①§11.3 关闭(Saga CRITICAL 修复 3 L4 全真修 + 271 test passed,per merge `195bfae`)②新增 §11.9 / §11.10 / §11.11 / §11.12 反映 4 worker 实际产出(Phase 0.5 retrospective / 5 域 DTL Review / 引用扫雷 / Saga 修复)
> - 2026-08-24 18:25 修订:①§11 子章节编号 10.1~10.8 → 11.1~11.8(commit 6d985d6 仅改 §11 标题未改子编号)②11.4 关闭(7c2db70 已修正则)③11.5 关闭(无文档错)④§12.3 改"未 commit" → "已 commit 66ff53b"
> - 2026-08-24 18:00 首次盘点:e293b98 §10 8 项 + 6d985d6 §11 标题

### 11.1 数据/evidence 丢失

- **`D:\RustGameServer-worktrees\WF-0-5-6\b1-evidence\` 原始 `kubectl describe/get` 11 个文件已丢**:worktree 清理时(因 untracked `.wbs-task-marker` + `.recon/`,违反 RGS-WT-001 §6.6 用 `--force` 强删),Step 6 留下的原始 evidence 一并删除。**摘要已写入 `b1-otel-pod-up.log` 头部**(`get-pods.txt` + `describe-{otel,prometheus,grafana}.txt` 关键内容已摘)。**重跑补回**:在 WSL2 K3s apply 3 套可观测性 manifest 后,跑 `kubectl describe` 重新生成 evidence 到 `docs/deploy/b1-evidence/`
- **dev 证书 `E:\DevCache\cargo\target\dev-certs\` 不在 git**:workspace 用了 `E:\DevCache\cargo\target\`(per Step 4 worker 实测),证书生成在 workspace 之外,天然不入仓。**SRE 在 WSL2 需重跑** `pwsh -File phase-0-5-step-4-gen-certs.ps1` 重新生成(预计 5 秒)
- **4 份 B-CODE log 在 handoff 完成日(2026-08-24)实际反映 1 🟡 + 3 🔴,非全 🟢**:SRE 跑完 §5 后必须**重写 4 份 log**(删旧 + 跑 4 实测),否则 NO-GO 实质状态不会被新证据覆盖

### 11.2 WBS 任务进度表未更新

- `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md`(注:7c2db70 已升 v0.3 → v0.4,WF-0.5-1/2/3 三行已标 done,§3 汇总同步 7 pending → 4 pending + 3 done)仍显示 128/128 整体 pending
- 实际状态:**WF-0.5-1/2/3 已在 Phase 0.5 期间作为"Phase 0.5 部署 worktree"重定向使用,3 个 worktree 已合并入 main;WF-0.5-6 worker 失败,主对话接手补完 4 B-CODE log + 总报告**
- **L4 任务标 done 的正确做法**:G-CODE-06 实测通过后由 `wbs_task_progress.ps1 -Status done` 自动填充;当前 NO-GO 形式上解除但实质未闭环,WBS 进度表更新是后续 Phase 0.5 实质完成后的工作
- **SRE 完成 §5 后**:`pwsh -File scripts/wbs_task_progress.ps1 -L4Id WF-0.5-1 -Status done` 等 3 个

### 11.3 Saga CRITICAL 修复 ✅ Closed(per merge `195bfae` + tag 收尾 `14036d6`)

- ✅ **Closed 2026-08-24 19:00**:`wbs/WF-1-55.27-retry` 分支并入 main(`195bfae`),`no-merge-pending-wf-1-55-27` git tag 已删(`14036d6`)
- ✅ **3 个 L4 任务全真修 + 真 PG 验证**(非 mock,经 K3s postgres-5bb9bb647d-6wfv4 port-forward 验证):
  - **WF-1-55.27 CR-1**(`c96efe8`):`ReserveHandler::execute` 修第 3 失败路径 `load_active_account` + 加 `Reservation::release()` 语义,2 个新测试锚定
  - **WF-1-55.28 CR-2**(`a80fa94`):6 域 migration 改名为 `*_outbox_check_idempotent.sql` + 加 `crates/economy-service/tests/integration_outbox.rs` 220 行真 PG 测试
  - **WF-1-55.31 CR-3**(`f6a6f3f`):`rgs_testkit::pg_pool()` + `rgs_testkit::pg_test` 强约束 re-export,`DbMock` / `NoopMock` / `mock_url` 全部 `#[deprecated]`,加 `pg_test_macro_re_export_works` doctest 真 PG 验证
- ✅ **验证结果**:`cargo test --workspace` **271 passed / 0 failed**,含 2 个真 PG integration test + deprecation 警告机制(use DbMock x2 / NoopMock x1 / mock_url x1 触发 4 个 warning)
- ✅ **worktree**:`D:\RustGameServer-worktrees\WF-1-55-retry\` 收尾后清理(保留分支 `wbs/WF-1-55.27-retry` 供审计)

### 11.4 git 工具链 bug 已修 ✅ Closed(per 7c2db70)

- ✅ **Closed 2026-08-24 18:17(commit `7c2db70`)**:`scripts/wbs_create_worktree.ps1` L4Id 正则 `^WF-[\d\.]+(?:\.[\d]+)?$` → `^WF-(\d+(\.\d+)?)-(\d+)(\.\d+)?$`,同时支持 `WF-1-53.1` / `WF-0.5-1` / `WF-1.5-X` 多种格式
- ✅ 新增 `scripts/test-l4id-regex.ps1` 回归测试 10/10 case pass
- ✅ handoff §11.4 之前标"未修复"是过时的——已 commit 7c2db70 第 2 项任务交付

### 11.5 RGS-WBS-001 文档 §11.4 vs §2A.6.7 引用错位 ✅ Closed(无文档错)

- ✅ **Closed 2026-08-24 18:25(主对话盘点)**:WF-0.5-X 实际位置在 `RGS-WBS-001_瀑布式工作分解结构_v0.3.md` §2A.6.7(行 326-350),不是 §11.4
- §11.4 表格内容是别的元信息(5 域 DTL 边界 + 跨域/平台 DTL 分配规则),与 WF-0.5-X 无关
- **修复:文档本身没错,只是主对话之前引用章节号写错**。后续引用统一改用 §2A.6.7(已在 §11.2 注中修正)
- **本项无代码/文档变更**,只主对话的引用习惯修正

### 11.6 worktree 清理违规例外条款 ✅ Closed(per 6ae469b)

- ✅ **Closed 2026-08-24(commit `6ae469b`)**:RGS-WT-001 v0.2 §11.7 已加例外条款,允许"已合并入 main 的 worktree 可 --force"但需走 4 步流程(人审 → 备份 untracked → 才允许 --force → 登记 RGS-INC-*);明确 --force 是 incident 应急而非日常快捷键
- **关联**:phase-0-5 反馈单 Issue 1 + RGS-WT-001 §6.7 协调规则
- **本项无新代码变更**(handoff §11.6 写"应加例外条款"已被 6ae469b 实现,只是 handoff 文档没同步)

### 11.7 3 个 rev-010-V detached worktree 清理 ✅ Closed(per 7c2db70)

- ✅ **Closed 2026-08-24(commit `7c2db70` + merge `65ea750`)**:3 个 D:\rev-010-V{1..3} detached worktree 已清理(per phase-0-5/local-fixes merge commit message "3 rev-010 清理")
- 验证: `git worktree list` 当前 2026-08-24 20:42 仅主工作区,无 D:\rev-010-V* 残留
- **本项无新代码变更**

### 11.8 RGS-INC-001 文档名/版本不一致 ✅ Closed(per 32333d2)

- ✅ **Closed 2026-08-24(commit `32333d2`)**:文件已 `git mv ..._v0.1.md → ..._v0.2.md`,文件头 v0.2 标记与文件名 v0.2 一致
- 验证: `glob docs/**/RGS-INC-001*.md` 当前 2026-08-24 20:42 仅返回 `RGS-INC-001_..._v0.2.md`
- **本项无新代码变更**

### 11.9 Phase 0.5 经验教训沉淀 ✅ Closed(per merge `7d68f73`)

- ✅ **Closed 2026-08-24 18:38(commit `7d68f73`)**:`docs/14-项目管理/RGS-PM-008_Phase_0.5_Retrospective_v0.1.md` 入仓
- ✅ 8 章节齐全:§0 元信息 / §1 范围与窗口 / §2 17 commit 时间线 + 2 补充 / §3 8 修复 + 3 commit + 4 B-CODE / §4 16 经验教训 / §5 6 风险 / §6 12 角色全签 per DEC-008 / §7 PH-1 后续 / §8 修订历史
- **worktree**:`D:\RustGameServer-worktrees\WF-0-5-retro\` 收尾后清理(保留分支 `phase-0-5/retro` 供审计)

### 11.10 5 域 DTL 字段级 Review 扫雷 ✅ Closed(per merge `452c3b2` 经 `77d06c2` 上链)

- ✅ **Closed 2026-08-24 18:30(commit `452c3b2` 在 `phase-0-5/review` 分支,merge `77d06c2` 不涉及此 commit)**
- ✅ 交付:`5-DOMAIN-DTL-REVIEW-REPORT.md` 26 KB,含 §1 状态矩阵(§A.1 跨 5 域 70 行 + §A.2-§A.7 域特定 125 行 = 195+ 行带具体行号引用)+ §2 6 行签字栏(per DEC-008 Ulysses 兼任)+ §3 3 处歧义占位 + §4 完成度自评
- ✅ `RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md` 末尾加 §A.8(5 域 Lead + 架构师 6 行)+ §A.9(3 处歧义 `[WF-0-5-7 联检前需统一] [域 Lead 决议]` 占位)
- ⚠️ **5 域完成度**:Admin 89% > Match 68% > Economy 57% > Player 53% > Social 43%,跨域 40%,**总体 ~58%**。**6 项关键缺口**:
  1. A1.9 监控 + A1.10 容量 — 6/7 DTL 缺指标、0/7 DTL 有 DAU 100k/QPS 10k 数字
  2. A1.13 DoD — 6/7 DTL 列入"不覆盖"(仅 DTL-031 §11.1 完整)
  3. A2.1/A2.2 — `players` / `player_characters` / `player_inventory` 主表在 DTL-018+DTL-036 均无字段级 DDL
  4. A5.1 — `messages`/`message_recipients`/`conversations` 消息分发主表在 DTL-019 缺失(DTL-019 实际是推送+兑换码,不是消息分发)
  5. A7.3 — 跨域 Saga 步骤编号缺失 + Q-003 跨 DB Saga 审批未完成(DTL-031 §8.2 阻断)
  6. A7.5 — 5 域监控指标命名一致性核查无基线
- ⚠️ **G-CODE-05 完全关闭(field-level DD Review Gate)前需**:架构师决议 + Q-003/ADR-0052 具名 Gate 通过 + DTL-036 §6 待补齐项 4 条落地
- **worktree**:`D:\RustGameServer-worktrees\WF-0-5-review\` 收尾后清理(保留分支 `phase-0-5/review` 供审计)

### 11.11 引用扫雷 + render-secrets.ps1 路径修复 ✅ Closed(per merge `77d06c2`)

- ✅ **Closed 2026-08-24 19:00(merge `77d06c2` + 2 commit `32333d2` + `f9512cc`)**
- ✅ **v0.3 → v0.4 引用扫雷**:4 行 / 4 文件已替换(`RGS-WBS-001_DAG_v0.3.md:188` + `RGS-WT-001_GitWorktree隔离开发方案.md:228` + `docs/deploy/phase-0-5-handoff.md:313/369`)
- ✅ **v0.1 → v0.2 引用扫雷**:2 行 / 2 文件已替换(`crates/function-plane/README.md:11` + `docs/deploy/phase-0-5-handoff.md:306`)
- ✅ **`render-secrets.ps1` 路径修复**:`TemplateDir` 默认从硬编码 `D:\RustGameServer-worktrees\WF-0-5-3\...` 改为 `Join-Path $PSScriptRoot '01-k8s-manifests'`,跨 worktree/克隆通用
- ✅ 2 份扫雷报告入仓:`docs/00-基本与治理/reviews/phase-0-5-citation-sweep/{v0.1,v0.3}-引用-扫雷-报告.md`
- ⚠️ **3 处未决**(保守不改):
  1. WBS 主文档 `RGS-WBS-001_瀑布式工作分解结构_v0.3.md:826, 1003` 内部 2 处 v0.3 引用(主文档不能改)
  2. handoff §10.8 第 407 行(历史快照,保留)
  3. WBS 主文档应升 v0.4 时统一改(单开下一个 worktree)
- ⚠️ **顺手发现 2 处断链**(不在本任务范围,仅记录):`07-no-go-checklist_v0.4.md:96` 自指断链 + `phase-0-5-handoff.md:263/265/285` 引用不存在的 `07-no-go-checklist_business_v0.3.md`(handoff 自己写的 Step 5 重跑后升 v0.3)
- **worktree**:`D:\RustGameServer-worktrees\WF-0-5-citation\` 收尾后清理(保留分支 `phase-0-5/citation` 供审计)

### 11.12 4 worker 并行结果总览(2026-08-24 18:25 ~ 19:00)

- **结果**:**4/4 succeeded**(0 失败,此前主对话误判"4/4 error 0 产出"已修正,见 §11 修订历史)
- **worktree 清理**:WF-1-55-retry / WF-0-5-review / WF-0-5-citation / WF-0-5-retro 4 个 worktree 收尾后清理(保留分支供审计)
- **commit 落地 7 个**:
  - `452c3b2` [phase-0-5/review] 5 域 DTL 字段级 Review 扫雷(2 文件 / 244 行)
  - `7d68f73` [retro] Phase 0.5 经验教训沉淀(1 文件 / 248 行)
  - `e4c084e` [saga-drill] G-CODE-04 Saga 6 场景详细演练(1 文件 / 881 行)
  - `32333d2` [phase-0.5] 引用扫雷:RGS-WBS-001 L4 进度表 v0.3→v0.4 + RGS-INC-001 v0.1→v0.2(6 文件 / 206 ins / 6 del)
  - `f9512cc` [fix] render-secrets.ps1:TemplateDir 默认值由硬编码 worktree 路径改为动态(1 文件 / 3 ins / 3 del)
  - `c96efe8` / `a80fa94` / `f6a6f3f` / `14036d6` Saga 修复 3 L4 + tag 收尾(30 文件 / 801 行 / 97 删)
- **merge 落地 2 个**:
  - `77d06c2` [merge] phase-0-5/citation(7 文件 / 209 行 / 9 删)
  - `195bfae` [merge] wbs/WF-1-55.27-retry(30 文件 / 801 行 / 97 删)

---

## 12. gRPC 健康探针 mTLS 兼容性修复(2026-08-24 追加,主对话完成)

### 12.1 根因

部署验证过程中定位到 6 服务(player/economy/match/social/admin/cluster-ops)Pod 会 `CrashLoopBackOff` 的**结构性**原因(与 DB/migration/证书内容无关),两个缺陷叠加:

1. 6 个服务的 `main.rs` 从未注册 `grpc.health.v1.Health` 服务(全仓 `grep -rn "tonic_health\|health_reporter"` 命中 0),任何 Health RPC 都会收到 `UNIMPLEMENTED`。
2. k8s 原生 `grpc:` 探针(kubelet 内建,stable since 1.27)在 PodSpec 层面**没有 TLS/客户端证书配置项**,恒为明文连接;而各服务通过 `shared_platform::tls::load_server_tls_config` 强制 mTLS fail-closed(per `RGS-DEC-015` P1),服务端在 TLS 握手阶段直接拒绝无证书连接——探针表现为超时,而非快速的 RPC 错误。

详细方案见 `docs/09-部署运维/RGS-OPS-101_gRPC健康探针mTLS兼容性修复设计_v0.1.md`(需求定义 / 基本设计 / 详细设计 / 实装规格 / 实施计划全套)。

### 12.2 方案(Option A,已实施)

**不新增任何明文 gRPC 端点**,保持 fail-closed mTLS 全链路不变式:

1. 6 服务 `main.rs` 注册 `tonic-health` 的 `grpc.health.v1.Health` 服务,挂载在原 mTLS 端口上(不新增端口)。
2. `Dockerfile` 内置静态编译的 `grpc_health_probe`(v0.4.56,linux-amd64,SHA-256 pin,已校验)二进制到 `runtime-base` 镜像。
3. 6 份 k8s manifest 的 `livenessProbe` / `readinessProbe` 从 `grpc:` 改为 `exec:`,执行 `grpc_health_probe` 并携带 `-tls` 系列参数,**复用各服务自己挂载的 mTLS server 证书**(`/etc/rgs/certs/server.pem` + `.key`)作为探针的 client 身份(`rgs-certgen` 生成证书未设 `ExtendedKeyUsage`,可同时充当 serverAuth/clientAuth,无需新证书/新 Secret)。

### 12.3 已变更文件(已 commit `66ff53b`,26 文件 / 685 行 / 96 删)

- `Cargo.toml`(workspace)+ 6 服务 `Cargo.toml`:加 `tonic-health = "0.12"`
- `crates/{player,economy,match,social,admin}-service/src/main.rs` + `crates/cluster-ops/src/main.rs`:注册 `health_reporter` / `health_service`,`add_service` 挂载
- `Dockerfile`:新增 `health-probe` 构建阶段(pin 版本 + sha256 校验)+ `runtime-base` 内置二进制
- `docs/deploy/01-k8s-manifests/01~06-*.yaml`:6 份探针 `grpc:` → `exec:`(已验证无残留 `grpc:` 探针,`grpc_health_probe` 命中 2×6=12 处)
- `docs/09-部署运维/RGS-OPS-101_gRPC健康探针mTLS兼容性修复设计_v0.1.md`(新增设计文档)

**已验证**:`cargo build --workspace` 全绿(6 服务 + shared-platform 编译通过)。**未验证**:`docker build` 实际产出 + K3s 实测(受限于本地无 docker/K3s 直连环境,留给 SRE 在 §5 Step 2/4 一并验证)。

### 12.4 对 SRE §5 接力清单的影响

- **Step 2(镜像构建)**:无需额外操作,`phase-0-5-step-5-build-images.ps1` 沿用即可——`grpc_health_probe` 已固化进 `Dockerfile`,build 时自动带上。**但请在 Step 2 验证时补一项**:`docker run --rm --entrypoint /bin/grpc_health_probe <image> -h` 应返回用法说明(确认二进制可执行,非 glibc/静态链接问题)。
- **Step 4(apply Deployment)**:6 份 manifest 已是 `exec:` 探针,直接 apply 即可,无需手改。
- **Step 5(4 B-CODE 验证)**:若 6 Pod 仍 `CrashLoopBackOff` 且不是本文档描述的 mTLS 探针问题,才需要按 §7 风险表的"Pod CrashLoopBackOff / migration 失败"分支排查(`kubectl logs --previous` + `kubectl describe pod`)。
- 若发现探针仍失败:优先检查 `-tls-server-name` 是否与该服务证书的 SAN(`rgs-certgen` 生成,`CN=<domain>` 如 `player.service`)一致,以及 `/etc/rgs/certs/` 挂载是否就位(与业务流量共用同一 `rgs-tls` projected volume,理论上不会单独出问题)。

---

**handoff 结束。SRE 接力 §5 5 步清单预计 2-3 小时,完成后 Phase 0.5 实质闭环 → 进 PH-1(WF-1 实施)。后续按 §11 事项清单 + §12 探针修复逐项收尾。**
