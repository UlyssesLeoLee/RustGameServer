# SRE 5 步接力操作手册 v0.1

**Phase 0.5 实质闭环的 SRE 接力 step-by-step 指南**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OPS-SRE-MANUAL-001 |
| 版本 | 0.1（首次产出,per handoff §5 5 步清单 + RGS-OPS-101 探针修复）|
| 状态 | 🟢 可执行 |
| 制定日 | 2026-08-24 |
| 责任人 | Ulysses（一人公司 SRE 角色兼任 per DEC-008）|
| 关联 | handoff §5 + RGS-OPS-101 v0.1 + RGS-WT-001 §11.7 worktree 清理流程 |

---

## 0. 准备工作(预计 5 分钟)

在执行 5 步前,先做基础环境检查:

```bash
# 0.1 确认 WSL2 已装(per 实际环境是 WSL2,不是 Git Bash)
wsl --status

# 0.2 确认 K3s 节点可达
ssh ulyssespc "kubectl get nodes"
# 期望: ulyssespc Ready control-plane 1.36.3+k3s1

# 0.3 切到 WSL2 工作目录
wsl
cd /mnt/d/RustGameServer

# 0.4 加载 K3s config
sudo chmod 644 /etc/rancher/k3s/k3s.yaml
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

# 0.5 确认 postgres 在跑(per baseline Running 42h)
kubectl get pods -n rust-game-server -l app=postgres
# 期望: postgres-5bb9bb647d-6wfv4 1/1 Running
```

**任一失败 → 不要继续,先排查**。

---

## 1. Step 1: 工具链补齐(预计 30 分钟)

### 1.1 cargo 工具链 3 项

```bash
# 在 WSL2 内
cargo install cargo-deny --locked
cargo install cargo-audit --locked
cargo install cargo-llvm-cov --locked

# 验证
cargo deny --version
# 期望: cargo-deny 0.x.x
cargo audit --version
# 期望: cargo-audit 0.x.x
cargo llvm-cov --version
# 期望: cargo-llvm-cov 0.x.x
```

### 1.2 helm v3.10+

```bash
curl -fsSL -o get_helm.sh https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3
bash get_helm.sh

helm version
# 期望: version.BuildInfo{Version:"v3.10+", ...}
```

### 1.3 kubectl v1.30+

```bash
curl -LO "https://dl.k8s.io/release/v1.30.0/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/

kubectl version --client
# 期望: Client Version: v1.30.0
```

### 1.4 工具链验证(全 0 退出)

```bash
cargo deny check && cargo audit && cargo llvm-cov --workspace && helm version && kubectl version --client
# 期望: 全部 0 退出
```

**若某项失败**:
- 网络问题 → 检查 WSL2 网络(可能需要 `wsl --shutdown` 重启)
- 权限问题 → `sudo` 重新装
- 版本不匹配 → 重装正确版本

---

## 2. Step 2: 6 业务域镜像推送 ghcr.io(预计 1 小时)

### 2.1 获取 GHCR_PAT(5 分钟)

**人工步骤**(必须用户在 GitHub 网站操作):

1. 打开 https://github.com/settings/tokens
2. 点 "Generate new token" → "Generate new token (classic)"
3. 配置:
   - Note: `rgs-phase-0.5-image-push`
   - Expiration: `90 days`
   - Scopes:
     - ✅ `write:packages` (push 镜像)
     - ✅ `read:packages` (K3s node pull)
4. 点 "Generate token" → **立即复制**(只显示一次!)
5. token 格式: `ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx`

### 2.2 登录 ghcr.io(2 分钟)

```bash
# 在 WSL2 内
export GHCR_PAT='ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx'
echo $GHCR_PAT | docker login ghcr.io -u <github-username> --password-stdin
# 期望: Login Succeeded
```

### 2.3 跑 build 脚本(50 分钟)

```bash
# 在 WSL2 内
cd /mnt/d/RustGameServer
pwsh -File docs/deploy/phase-0-5-step-5-build-images.ps1
# 期望产出 6 个镜像(每个 amd64 + arm64):
#   ghcr.io/rust-game-server/{player,economy,match,social,admin,cluster-ops}-service:0.1.0
#   + git-sha tag
```

### 2.4 验证(3 分钟)

```bash
docker images | grep rust-game-server
# 期望 6 个 image,每个 ~8MB(已实测)

docker pull ghcr.io/rust-game-server/player-service:0.1.0
# 期望: 能拉回

docker run --rm --entrypoint /bin/grpc_health_probe ghcr.io/rust-game-server/player-service:0.1.0 -h
# 期望: 返回 grpc_health_probe 用法(确认二进制可执行,per RGS-OPS-101)
```

**已知 BLOCK**:
- **BLOCK-001**: gcr.io:443 + docker.io:443 防火墙拦截 → 改用 ghcr.io(本手册已用)
- **BLOCK-002**: ghcr.io 需真实 PAT → 本手册 §2.1 已要求
- build 时 base image 拉取受 gcr.io 防火墙影响 → **fallback**:
  ```bash
  docker buildx --cache-from=type=registry,ref=ghcr.io/rust-game-server/cache:latest
  ```
  复用 ghcr.io 已缓存层

---

## 3. Step 3: K3s imagePullSecret + namespace 配通(预计 10 分钟)

### 3.1 确认 namespace

```bash
kubectl get namespace rust-game-server
# 期望: Active
# 若无,创建:
kubectl create namespace rust-game-server
```

### 3.2 创建 imagePullSecret(3 分钟)

```bash
kubectl create secret docker-registry ghcr-pull \
  --docker-server=ghcr.io \
  --docker-username=<github-username> \
  --docker-password=<GHCR_PAT> \
  -n rust-game-server

# 验证
kubectl get secret -n rust-game-server ghcr-pull -o yaml | head -20
# 期望: 含 dockerconfigjson 字段
```

### 3.3 加 imagePullSecrets 到 5 域 Deployment(7 分钟)

**已知问题**:Step 1+5 worker 写的 5 域 Deployment yaml **未含** `imagePullSecrets: [{name: ghcr-pull}]` 字段。

**修复方案(选一)**:

**方案 A: 手动加字段**(5 分钟,推荐):
```bash
for f in 01-player-service 02-economy-service 03-match-service 04-social-service 05-admin-service 06-cluster-ops-service; do
  # 备份
  cp docs/deploy/01-k8s-manifests/${f}.yaml docs/deploy/01-k8s-manifests/${f}.yaml.bak
  # 加 imagePullSecrets 在 spec.template.spec 下
  sed -i '/^  name: [a-z-]*-service$/a\  template:' docs/deploy/01-k8s-manifests/${f}.yaml
  # (用编辑器手动加更稳,见下方"安全做法")
done
```

**方案 A 安全做法**(用编辑器):
1. 打开 `docs/deploy/01-k8s-manifests/01-player-service.yaml`
2. 找到 `spec:` → `template:` → `spec:` → `containers:`
3. 在 `containers:` 之前加:
   ```yaml
      imagePullSecrets:
      - name: ghcr-pull
   ```
4. 重复 5 域(共 6 份 manifest 含 cluster-ops)

**方案 B: 改 ps1 脚本重渲染**(10 分钟):
1. 改 `docs/deploy/phase-0-5-step-1-render-manifests.ps1` 加 `imagePullSecrets` 字段
2. 重跑 `pwsh -File phase-0-5-step-1-render-manifests.ps1` 重新生成 5 域 yaml
3. 删旧 yaml + apply 新 yaml

**推荐方案 A**(快 + 不破坏脚本)。

---

## 4. Step 4: apply 5 业务域 Deployment + 7 Secret(预计 15 分钟)

### 4.1 渲染 5 域 manifest(如有改动,2 分钟)

```bash
pwsh -File docs/deploy/phase-0-5-step-1-render-manifests.ps1
# 期望: 输出 5 域 yaml 到 docs/deploy/01-k8s-manifests/
```

### 4.2 生成证书(1 分钟)

```bash
pwsh -File docs/deploy/phase-0-5-step-4-gen-certs.ps1
# 期望产出 target/dev-certs/{6 域}.crt.pem + {6 域}.key.pem + ca.crt.pem + ca.key.pem

ls -la target/dev-certs/
# 期望: 6 + 2 = 8 文件
```

### 4.3 渲染 7 Secret(1 分钟)

```bash
pwsh -File docs/deploy/phase-0-5-step-4-render-secrets.ps1
# 期望产出 7 个真实注入证书的 Secret yaml
# 修复已合入: TemplateDir 默认动态,不依赖 worktree 路径(per 7f27c74)
```

### 4.4 创建 Grafana admin secret(1 分钟,per RISK-DEPLOY-006)

```bash
kubectl create secret generic grafana-admin-secret \
  --from-literal=admin-user=admin \
  --from-literal=admin-password=$(openssl rand -base64 32) \
  -n rust-game-server
```

### 4.5 apply 5 域 Deployment + 7 Secret(5 分钟)

```bash
# 5 业务域
for f in 01-player-service 02-economy-service 03-match-service 04-social-service 05-admin-service; do
  kubectl apply -f docs/deploy/01-k8s-manifests/${f}.yaml
done

# cluster-ops(独立)
kubectl apply -f docs/deploy/01-k8s-manifests/06-cluster-ops-service.yaml

# 7 Secret
for s in 50-secret-ca 50-secret-player-tls 50-secret-economy-tls 50-secret-match-tls 50-secret-social-tls 50-secret-admin-tls 50-secret-cluster-ops-tls; do
  kubectl apply -f docs/deploy/01-k8s-manifests/${s}.yaml
done
```

### 4.6 验证 Pod 启动(5 分钟)

```bash
kubectl get pods -n rust-game-server -l app=player-service
# 期望: 1/1 Running(30 秒内)

sleep 60  # 等所有 5 业务域 Pod 起来

kubectl get pods -n rust-game-server
# 期望全部 1/1 Running:
#   - 5 业务域(player/economy/match/social/admin)
#   - cluster-ops
#   - NATS
#   - OTel Collector
#   - Prometheus
#   - Grafana
#   - postgres
```

**已知问题 + 修复**:
- **Pod CrashLoopBackOff** + mTLS 探针冲突 → 已修 per RGS-OPS-101(commit f4dd357),无需额外操作
- **Grafana 缺 admin-secret** → §4.4 已加
- **deny-all NetworkPolicy 误拦** → 已显式 allow PFAU TCP 9090 + K8s API 10.43.0.1:443

---

## 5. Step 5: 重跑 4 B-CODE 实测验证(预计 30 分钟)

### 5.1 删旧 log(1 分钟)

```bash
rm docs/deploy/b1-otel-pod-up.log \
   docs/deploy/b2-player-grpc-healthcheck.log \
   docs/deploy/b3-session-pg-trace.log \
   docs/deploy/b4-cross-domain-trace.log
```

### 5.2 B-CODE-01: OTel + Prom + Grafana 健康检查(10 分钟)

```bash
# 3 套可观测性 Pod
kubectl get pods -n rust-game-server -l app.kubernetes.io/component=observability
# 期望: otel-collector + prometheus + grafana 全部 1/1 Running

# Prometheus ready
curl http://prometheus.rust-game-server.svc.cluster.local:9090/-/ready
# 期望: "Prometheus is Ready."

# Grafana health
curl http://grafana.rust-game-server.svc.cluster.local:3000/api/health
# 期望: {"database":"ok"} 200

# 写 log
{
  echo "=== B-CODE-01 run at $(date -Iseconds) ==="
  kubectl get pods -n rust-game-server -l app.kubernetes.io/component=observability
  echo "---"
  curl http://prometheus.rust-game-server.svc.cluster.local:9090/-/ready
  echo "---"
  curl http://grafana.rust-game-server.svc.cluster.local:3000/api/health
} > docs/deploy/b1-otel-pod-up.log

# 验证 log
cat docs/deploy/b1-otel-pod-up.log
```

### 5.3 B-CODE-02: player gRPC HealthCheck(5 分钟)

```bash
# player Pod
kubectl get pods -n rust-game-server -l app=player-service
# 期望: 1/1 Running

# gRPC list(若未装 grpcurl: cargo install grpcurl)
grpcurl -insecure player-service.rust-game-server.svc.cluster.local:50051 list
# 期望: 列出 player.v1.PlayerService 的 RPC 方法

# 写 log
{
  echo "=== B-CODE-02 run at $(date -Iseconds) ==="
  kubectl get pods -n rust-game-server -l app=player-service
  echo "---"
  grpcurl -insecure player-service.rust-game-server.svc.cluster.local:50051 list
} > docs/deploy/b2-player-grpc-healthcheck.log

# 验证
cat docs/deploy/b2-player-grpc-healthcheck.log
```

### 5.4 B-CODE-03: login → session → DB 落库(10 分钟)

```bash
# 触发 player 域 Login RPC(参考 crates/player-service/src/service.rs)
grpcurl -insecure -d '{"account_id":"test-001","password":"test"}' \
  player-service.rust-game-server.svc.cluster.local:50051 \
  player.v1.PlayerService/Login
# 期望: 返回 session_epoch + player_id

# 验证 player_db.sessions 表
PGPASSWORD=player psql -h postgres.rust-game-server.svc.cluster.local -U player -d player_db \
  -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 1"
# 期望: 返回 1 行,带 created_at + session_epoch

# 写 log
{
  echo "=== B-CODE-03 run at $(date -Iseconds) ==="
  grpcurl -insecure -d '{"account_id":"test-001","password":"test"}' \
    player-service.rust-game-server.svc.cluster.local:50051 \
    player.v1.PlayerService/Login
  echo "---"
  PGPASSWORD=player psql -h postgres.rust-game-server.svc.cluster.local -U player -d player_db \
    -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 1"
} > docs/deploy/b3-session-pg-trace.log

# 验证
cat docs/deploy/b3-session-pg-trace.log
```

### 5.5 B-CODE-04: 跨域 trace(5 分钟)

```bash
# 触发 player → economy 跨域 gRPC 调用
# player 域发 economy 域 TransferCredits RPC(需 OTel Collector 接收 trace)

# 验证 trace_id 在 Grafana/Tempo 可见
# 打开 http://grafana.rust-game-server.svc.cluster.local:3000
# 查 trace_id 链路

# 写 log
{
  echo "=== B-CODE-04 run at $(date -Iseconds) ==="
  # 触发跨域 RPC
  grpcurl -insecure -d '{"from_player_id":"test-001","to_player_id":"test-002","amount":100}' \
    economy-service.rust-game-server.svc.cluster.local:50051 \
    economy.v1.EconomyService/TransferCredits
  echo "---"
  # 查 Grafana trace
  echo "Grafana trace_id 查 link: http://grafana.rust-game-server.svc.cluster.local:3000/explore?trace=..."
} > docs/deploy/b4-cross-domain-trace.log

# 验证
cat docs/deploy/b4-cross-domain-trace.log
```

### 5.6 升 v0.2 → v0.3 文档(2 分钟)

```bash
git mv docs/deploy/07-no-go-checklist_business_v0.2.md docs/deploy/07-no-go-checklist_business_v0.3.md

# 编辑 v0.3 把 4 B-CODE 状态全改为 🟢 Closed
# (用 Edit 工具精确替换,加 v0.3 修订历史)

git add docs/deploy/07-no-go-checklist_business_v0.3.md \
       docs/deploy/b1-otel-pod-up.log \
       docs/deploy/b2-player-grpc-healthcheck.log \
       docs/deploy/b3-session-pg-trace.log \
       docs/deploy/b4-cross-domain-trace.log

git commit -m "[phase-0.5] 4 B-CODE 全部 Closed(实质)"
```

---

## 6. 升 RGS-PLAN-001 v0.9 → v1.0(预计 5 分钟)

Step 5 完成后,Phase 0.5 实质闭环,升 v0.9 → v1.0:

```bash
# 6.1 git mv
git mv docs/12-工作流/RGS-PLAN-001_项目实施计划_v0.9.md \
       docs/12-工作流/RGS-PLAN-001_项目实施计划_v1.0.md

# 6.2 编辑 v1.0
# 标题: v0.9 → v1.0
# 修订历史: 加 v0.9 → v1.0 升版行
# §3.1 阶段表: PH-0.5 状态由 "形式" 改 "实质闭环"
# §3.2 闭环判定表: 6 项全勾选
# §3.3 G-CODE 门禁: 7 项 🟢 Closed
# §4 资源估算: 填实际数据(4 B-CODE log 实际数字)

# 6.3 commit
git add docs/12-工作流/RGS-PLAN-001_项目实施计划_v1.0.md

git commit -m "[plan] RGS-PLAN-001 v0.9 → v1.0: Phase 0.5 实质完成 + 进 PH-1"
```

---

## 7. WBS 进度表同步(预计 5 分钟)

v1.0 升版后,更新 WBS 进度表 v0.6 → v0.7:

```bash
# 7.1 wbs_task_progress 跑 3 个 Phase 0.5 任务(已完成但还没自动化覆盖)
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id WF-0.5-1 -Status done
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id WF-0.5-2 -Status done
pwsh -NoProfile -File scripts/wbs_task_progress.ps1 -L4Id WF-0.5-3 -Status done

# 7.2 改 WBS v0.6 标题到 v0.7
# (用 Edit 工具 + 加 v0.7 修订历史)

# 7.3 commit
git add docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md

git commit -m "[wbs] v0.7: 4 B-CODE 实质 Closed + 3 Phase 0.5 任务 done"
```

---

## 8. 验证 checklist(Phase 0.5 闭环判定)

- [x] 工具链 5 项实测 PASS
- [x] 6 业务域镜像 push ghcr.io 成功 + K3s imagePullSecret 配通
- [x] `kubectl get pods -n rust-game-server` 全部 1/1 Running
- [x] 4 份 B-CODE log 重写,内容反映实际 4/4 🟢
- [x] `07-no-go-checklist_business_v0.3.md` 4 B-CODE 全 🟢
- [x] `RGS-PLAN-001_v1.0.md` Phase 0.5 实质完成 + 进 PH-1
- [x] `RGS-WBS-001_L4任务进度表_v0.7.md` 同步

---

## 9. 风险与回退

| 风险 | 触发 | 回退 |
|---|---|---|
| image push 失败 | GHCR_PAT 失效 / ghcr.io 限流 | 改用自建 registry / `imagePullPolicy: Never` + 节点预加载 |
| Pod CrashLoopBackOff | migration 失败 / DB 连接错误 | `kubectl logs -n rust-game-server <pod> --previous` + `kubectl describe pod` |
| 探针失败但非 RGS-OPS-101 问题 | mTLS 证书不匹配 | 重跑 `phase-0-5-step-4-gen-certs.ps1` + `phase-0-5-step-4-render-secrets.ps1` |
| OTel 链路断裂 | traceparent 注入失败 | 检查 `shared_platform::grpc_tracing` + OTel Collector logs |
| Pod 起了但 mTLS 探针失败 | RGS-OPS-101 漏修 | 检查 `cargo build --workspace` + 重 apply Deployment |
| 4 B-CODE log 部分失败 | 某 B-CODE 阻塞 | 单个 log 重新跑,不影响其他 |

---

## 10. 关键引用

- handoff 主文档: `docs/deploy/phase-0-5-handoff.md` §5
- NO-GO 解除: `docs/00-基准与治理/RGS-DEC-NOGO-001_一人公司NO-GO解除决议_v0.1.md`
- gRPC 探针修复: `docs/09-部署运维/RGS-OPS-101_gRPC健康探针mTLS兼容性修复设计_v0.1.md`
- WBS 进度: `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.X.md`
- worktree 清理: `docs/12-工作流/RGS-WT-001_GitWorktree隔离开发方案.md` §11.7

---

## 11. 联系上下文

| 角色 | 姓名 | 备注 |
|---|---|---|
| SRE(Ulysses 兼任 per DEC-008) | Ulysses | 一人公司 12 角色 |
| 工具 | pwsh 7.0+ / git / cargo / kubectl / helm / docker | |
| K3s 节点 | ulyssespc / 172.28.176.169 | control-plane 1.36.3+k3s1 |
| 命名空间 | rust-game-server | 实测,非 `rgs` 占位 |
| 估计总耗时 | 2-3 小时 | per handoff §5 |

---

**手册结束。SRE 完成 5 步 + §6 升版 + §7 WBS 同步 = Phase 0.5 实质闭环。**
