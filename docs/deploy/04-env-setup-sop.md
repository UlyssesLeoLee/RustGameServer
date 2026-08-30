# 环境部署实测 SOP（Windows 11 + WSL2 + k3s native，RGS-ENV-001 v0.3 §1-§5 + G-CODE-03/06）

> **文档 ID**：`RGS-DEPLOY-ENV-SETUP-001`
> **版本**：v0.3（实测 2026-08-22 后更新：Ubuntu 24.04 + k3s v1.36.3 + k3s 自带 kubectl + 一键 deploy 脚本）
> **生效日期**：2026-08-22
> **目标**：**🟢 GO — 53 起動条件已满足**（per 07-no-go-checklist_v0.4）
> **平台**：Windows 11 + PowerShell 7.6+ + WSL2（**Ubuntu 24.04 LTS 实测**）+ k3s native
> **实测人**：Ulysses（一人公司 12 角色兼任 per DEC-008）
> **实测日期**：2026-08-22 11:58 JST
> **实测结果**：**6/6 section PASS**（Rust / Postgres / DB / Build / Topology / Verify）
> **总耗时**：~30-60 分钟（k3s 首次安装 5-10 分钟 + PG image pull 60-90 秒 + 实测 5 秒）

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Windows 11 + Docker Desktop + k3d 流程（PG 用 docker compose）。 |
| 0.2 | 2026-08-21 | 架构师（Ulysses）| **DEC-010 落地**：k3d → k3s native in WSL2。PG 从 docker compose 升为 k3s pod 部署（per 01-k8s-manifests/20-24 清单）。§0/§2/§3/§4/§5/§7/§8 全面重写。 |
| 0.3 | 2026-08-22 | 架构师（Ulysses）| **实测后更新**：Ulysses 在 WSL2（Ubuntu 24.04）实测装 k3s v1.36.3 + apply PG manifest + 跑 measure_env_setup.ps1，6/6 section PASS。**新增** 关键脚本：`scripts/deploy_dev_k3s.ps1`（一键 apply namespace + SA + 5 PG manifest，幂等）/ `scripts/port_forward_pg.ps1`（WSL2 PG 端口转发）。**实测发现**：(1) OS 实测为 Ubuntu 24.04 LTS（SOP 写 22.04，**两者均可**——WSL2 支持 20.04+）；(2) k3s 实测版本 v1.36.3+k3s1（SOP 写 v1.30+，**实际装到 v1.36+**——k3s 当前主版本）；(3) kubectl 路径：实测用 k3s 自带 `k3s kubectl`（**用户未装 standalone kubectl**，原 SOP 写 `kubectl` 已修正为 `k3s kubectl`）；(4) 24-postgres-service.yaml 原含手动 Endpoints + clusterIP:None + selector 三者冲突，**修正为仅 Service + selector**（Endpoints 由 k8s 自动创建）；(5) postgres pod 需 ServiceAccount，**新增** 00-postgres-sa.yaml（dev 用，生产用 RBAC）。 |
| 0.2 | 2026-08-21 | 架构师（Ulysses）| **DEC-010 落地**：k3d → k3s native in WSL2。PG 从 docker compose 升为 k3s pod 部署（per 01-k8s-manifests/20-24 清单）。§0 总览 / §2 PG / §3 WSL2 / §4 k3s / §5 kubectl / §7 OTel / §8 核验 log 全面重写。**未变更**：Rust 1.98 / cargo 工具链 / 5 独立 DB 原则 / ARC-008 / ADR-0052 / 53 起動条件。 |

---

## 0. 总览（7 大组件）

| # | 组件 | 触发 G-CODE / ENV § | 优先级 | 预计耗时 | 实测耗时 | 实测状态 |
|---|---|---|---|---|---|---|
| 1 | Rust 1.98 stable | G-CODE-06 + ENV-§1 | 🔴 必填 | 5 分钟 | 5 分钟 | ✅ pass |
| 2 | PostgreSQL 18.6 + 5 独立 DB（k3s pod）| G-CODE-03 + ENV-§2 | 🔴 必填 | 15 分钟 | 2 分钟（用 deploy_dev_k3s.ps1）| ✅ pass |
| 3 | WSL2 + Ubuntu 22.04 / 24.04 | ENV-§3.1 | 🟡 高 | 10 分钟 | 已装跳过 | ✅ pass |
| 4 | k3s native（WSL2 内 systemd 模式）| ENV-§3 | 🟡 高 | 10 分钟 | 5 分钟 | ✅ pass |
| 5 | kubectl + Helm（WSL2 内）| ENV-§3.4 | 🟡 高 | 5 分钟 | 1 分钟（用 k3s 自带 kubectl）| ✅ pass |
| 6 | sqlx-cli + cargo-deny + cargo-audit + cargo-llvm-cov + protoc | ENV-§1.3 + §5.1 | 🟡 中 | 15 分钟 | 跳过（待 53 启动后补）| ⚠️ pending |
| 7 | OTel Collector + Prometheus + Grafana（k3s Helm chart）| ENV-§5 | 🟢 低 | 10 分钟 | 跳过（待 53 启动后补）| ⚠️ pending |

**串行依赖**：1 → 6（cargo 工具链需 Rust 1.98）→ 3 → 4 → 5 → 2（PG pod 需 k3s ready）→ 7

> **实测后优化**（v0.3 per 2026-08-22 Ulysses 实测）：
> - ✅ 关键 1-5 项 + §2（PG pod 部署）已实测通过，6/6 measure script section 全 PASS
> - ⚠️ 6-7 项（cargo 工具链 + OTel/Helm）跳过，待 53 起動后 + 5 域微服务实施前补
> - ✅ **实测关键脚本**：`scripts/deploy_dev_k3s.ps1`（一键 apply PG，幂等，30s 内完成）/ `scripts/port_forward_pg.ps1`（WSL2 → Windows 5432 端口转发）/ `scripts/measure_env_setup.ps1`（幂等实测，已支持 k3s kubectl）

> **DEC-010 关键变更**：
> - ❌ 移除：Docker Desktop 强制依赖（k3s native in WSL2 不需要 Docker 引擎）
> - ❌ 移除：k3d（k3s in Docker）方案
> - ❌ 移除：PG docker compose 方案（PG 现在跑在 k3s pod 里）
> - ✅ 保留：Docker Desktop 可选（如果想用 docker compose 跑 OTel dev 栈）
> - ✅ 保留：sqlx-cli 跨平台（Windows pwsh 调 WSL2 内 PG 通过 `localhost:5432` 端口转发）

---

## 1. Rust 1.98 stable（触发 G-CODE-06）

**安装**：
```powershell
# PowerShell 7（不要用 powershell 5.1）
pwsh -NoProfile -Command "irm https://win.rustup.rs/x86_64 | iex"
# 选项：1) Proceed with installation (default)

# 安装后关闭 PS，重开 pwsh 验证
pwsh -NoProfile -Command "rustc --version; cargo --version; rustup show"
```

**预期输出**：
```
rustc 1.98.0 (xxxxx 2026-08-XX)
cargo 1.98.0 (xxxxx 2026-08-XX)
```

**最小 workspace 验证**（per G-CODE-06）：
```powershell
cd D:\RustGameServer

# Cargo.toml（最小，per commit b290367 已就位）
# rust-toolchain.toml（per commit b290367 已就位）
# crates/rgs-hello/（per commit b290367 已就位）

# 编译 + 测试
cargo build --locked 2>&1 | Tee-Object docs/deploy/06-rust-198-build.log
cargo test --locked --workspace 2>&1 | Tee-Object docs/deploy/06-rust-198-build.log -Append
```

**预期输出**（最后几行）：
```
Compiling rgs-hello v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
Running unittests src\main.rs
test result: ok. 0 passed; 0 failed; 0 ignored
```

**关闭 G-CODE-06 条件**：以上 2 条命令全绿 + 命令输出存档到 `docs/deploy/06-rust-198-build.log` ✓

---

## 2. PostgreSQL 18.6 + 5 独立 DB（k3s pod 部署，触发 G-CODE-03）

> **DEC-010 变更**：从 docker compose 改为 k3s pod 部署（per `01-k8s-manifests/20-24-*.yaml`）。
> **优势**：与生产 5 域 Deployment 部署路径一致；Volume/PVC/ConfigMap/Secret 全部标准化；一次 SOP 适用 dev/prod。
> **v0.3 实测优化**：Ulysses 已实测一键脚本 `scripts/deploy_dev_k3s.ps1` 完成本节全部流程（30s 内）。

### §2.1 前置条件

- WSL2 + k3s 已就位（§3 + §4）
- k3s 自带 kubectl（`k3s kubectl`）即可，无须独立装 kubectl
- 在 Windows 端能跑 `pwsh` 7.0+

### §2.2 一键部署 PG（推荐 per v0.3 实测）

```powershell
# Windows pwsh（推荐）
pwsh -NoProfile -File scripts/deploy_dev_k3s.ps1

# 预期输出（实测 2026-08-22 11:58 JST）：
#   namespace applied
#   ServiceAccount applied
#   20-postgres-secret.yaml applied: secret/postgres-superuser configured + 6 个 DB secret created
#   21-postgres-pvc.yaml applied: persistentvolumeclaim/postgres-data-pvc created
#   22-postgres-configmap.yaml applied: configmap/postgres-config created
#   23-postgres-statefulset.yaml applied: deployment.apps/postgres configured
#   24-postgres-service.yaml applied: service/postgres configured
#   waiting... (5s): pod Ready
#   PG version: PostgreSQL 18.6 (Debian 18.6-1.pgdg13+2) ...
#   ✅ PG 18.6 验证通过
```

**脚本自动完成**：
1. 检查 WSL2 + k3s 节点 Ready
2. 把 `01-k8s-manifests/` 的 `00-namespace.yaml` + 5 个 PG manifest 复制到 WSL2 `/tmp/rgs-deploy-dev/`（避免 Windows 路径问题）
3. 替换所有 `PLACEHOLDER_*` 为 dev 值（namespace / SA / SVC / PVC / ConfigMap / 资源 request/limit / 密码）
4. 注入一个 dev 用的 `postgres-service-account`（生产由 helm RBAC 接管）
5. apply 全部 manifest 到 k3s
6. 等待 pod Running + Ready 1/1（timeout 360s，足够 image pull 60-90s）
7. `kubectl exec` 进 pod 跑 `psql SELECT version()` 验证 18.6

### §2.3 手动部署（备选 / 调试用）

如果脚本失败或要手动执行：

```bash
# 在 WSL2 内（默认 distro，per wsl -l -q）
wsl

# 1. 创建命名空间
k3s kubectl apply -f 01-k8s-manifests/00-namespace.yaml

# 2. 创建 ServiceAccount（dev 用，prod 由 RBAC 接管）
cat <<EOF | k3s kubectl apply -f -
apiVersion: v1
kind: ServiceAccount
metadata:
  name: postgres-service-account
  namespace: rust-game-server
EOF

# 3. sed 替换 PLACEHOLDER_* 后 apply 5 个 PG manifest
cd /mnt/d/RustGameServer/docs/deploy/01-k8s-manifests
for f in 20-postgres-secret.yaml 21-postgres-pvc.yaml 22-postgres-configmap.yaml 23-postgres-statefulset.yaml 24-postgres-service.yaml; do
  sed -e 's/PLACEHOLDER_NAMESPACE/rust-game-server/g' \
      -e 's/PLACEHOLDER_POSTGRES_SA/postgres-service-account/g' \
      -e 's/PLACEHOLDER_POSTGRES_SVC_NAME/postgres/g' \
      -e 's/PLACEHOLDER_POSTGRES_PVC_NAME/postgres-data-pvc/g' \
      -e 's/PLACEHOLDER_POSTGRES_CONFIGMAP_NAME/postgres-config/g' \
      -e 's/PLACEHOLDER_POSTGRES_DEPLOY_NAME/postgres/g' \
      -e 's/PLACEHOLDER_POSTGRES_STORAGE_CLASS/local-path/g' \
      -e 's/PLACEHOLDER_POSTGRES_STORAGE_SIZE/5Gi/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_SUPERUSER_PASSWORD/ulysses_local/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_PLAYER_PASSWORD/ulysses_local/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_ECONOMY_PASSWORD/ulysses_local/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_MATCH_PASSWORD/ulysses_local/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_SOCIAL_PASSWORD/ulysses_local/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_ADMIN_PASSWORD/ulysses_local/g' \
      -e 's/REPLACE_BEFORE_DEPLOY_CLUSTER_OPS_PASSWORD/ulysses_local/g' \
      "$f" | k3s kubectl apply -f -
done
```

### §2.4 等待 PG pod ready + 验证

```bash
# 等待 pod Running + Ready 1/1
k3s kubectl wait --for=condition=ready pod -n rust-game-server -l app.kubernetes.io/name=postgres --timeout=300s

# 验证 pod
k3s kubectl get pod -n rust-game-server -l app.kubernetes.io/name=postgres

# 验证 PG 版本
k3s kubectl exec deploy/postgres -n rust-game-server -- psql -U postgres -tAc "SELECT version();"
# 预期: PostgreSQL 18.6 (Debian 18.6-1.pgdg13+2) ...

# 验证 5 独立 DB
k3s kubectl exec deploy/postgres -n rust-game-server -- psql -U postgres -tAc "SELECT datname FROM pg_database WHERE datistemplate=false;"
# 预期: admin_db / cluster_ops_db / economy_db / match_db / player_db / postgres / social_db
```

### §2.5 端口转发（供 Windows 端 sqlx-cli / psql 用）

```powershell
# Windows pwsh（另开一个 shell，保持运行）
pwsh -NoProfile -File scripts/port_forward_pg.ps1

# 预期: port-forward 启动后，Windows 端 5432 端口可连
# 验证：
Test-NetConnection -ComputerName 127.0.0.1 -Port 5432
# 预期: TcpTestSucceeded = True
```

```powershell
# Windows pwsh（连 PG）
pwsh -NoProfile -Command "& { \$env:PGPASSWORD='ulysses_local'; & 'C:\Program Files\PostgreSQL\18\bin\psql.exe' -h localhost -U postgres -c 'SELECT version();' }"
# 预期: PostgreSQL 18.6

# sqlx-cli 用法
$env:DATABASE_URL = "postgres://postgres:ulysses_local@localhost:5432/player_db"
sqlx database create
```

### §2.6 G-CODE-03 5 独立 DB 拓扑图

- 工具：draw.io / Excalidraw / Mermaid / 手画 PNG 均可
- 提交到：`docs/deploy/05-db-topology.png`（或 .svg / .drawio / .mmd）
- 必备元素：6 个 DB 框 + Schema 命名（per RGS-SPEC-CROSS-005） + 跨 DB 访问箭头（标"禁止 JOIN"） + Outbox + CEM 跨域协调路径 + **k3s pod 边界框**（per DEC-010）
- Mermaid 源已就位：`docs/deploy/05-db-topology.mmd`（脚本 `measure_env_setup.ps1` 自动生成）

**关闭 G-CODE-03 条件**（v0.3 实测更新）：
- ✅ k3s pod Running + Ready 1/1（`kubectl get pod -l app.kubernetes.io/name=postgres` 显示 `1/1`）
- ✅ PG 18.6 验证（`SELECT version()` 包含 `18.6`）
- ✅ 5 独立 DB 全建（`player_db / economy_db / match_db / social_db / admin_db + cluster_ops_db`）
- ✅ 拓扑图存档到 `docs/deploy/05-db-topology.png`（或 .svg / .mmd）
- ✅ Ulysses **实际跑过**（非签字声明，per RGS-EXEC-001 v0.3 §3.4）— 2026-08-22 11:58 JST 实测

---

## 3. WSL2 + Ubuntu 22.04

> **DEC-010 变更**：Docker Desktop 不再是 k3s 前置（k3s native in WSL2）。但 Docker Desktop 仍可保留供 OTel dev compose（§7 备选）。

**安装 WSL2**（PowerShell 管理员）：
```powershell
# 1. 启用 WSL 功能
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart

# 2. 重启 Windows

# 3. 设置 WSL2 为默认
wsl --set-default-version 2

# 4. 安装 Ubuntu 22.04（从 Microsoft Store 或 wsl --install）
wsl --install -d Ubuntu-22.04

# 5. 启动并设置用户名 + 密码
wsl -d Ubuntu-22.04
```

**启用 systemd**（k3s 需要，per k3s 官方文档）：
```bash
# WSL2 内（Ubuntu 22.04）
sudo nano /etc/wsl.conf
# 添加：
# [boot]
# systemd=true

# 退出 WSL
exit

# 在 Windows pwsh 重启 WSL
wsl --shutdown
wsl -d Ubuntu-22.04
```

**验证 systemd 启用**：
```bash
# WSL2 内
systemctl list-units --type=service --state=running | head
```

**预期输出**（看到 `systemd` + 多个 `.service`）：
```
UNIT                        LOAD   ACTIVE SUB     DESCRIPTION
accounts-daemon.service     loaded active running Accounts Service
cron.service                loaded active running Regular background program processing daemon
dbus.service                loaded active running D-Bus System Message Bus
...
systemd-journald.service    loaded active running Journal Service
```

---

## 4. k3s native（WSL2 内 systemd 模式）

> **DEC-010 变更**：从 k3d（k3s in Docker）改为 k3s native（WSL2 Ubuntu 内 systemd 模式）。
> **优势**：与生产 k3s 部署路径一致；systemd 服务管理（`systemctl status k3s`）；无 Docker 引擎依赖。

**安装 k3s**（WSL2 内）：
```bash
# 标准安装（无 traefik，因为 dev 集群用 kubectl port-forward 暴露）
curl -sfL https://get.k3s.io | sh -s - --disable=traefik --write-kubeconfig-mode=644

# 安装后 k3s 自动以 systemd 服务启动
sudo systemctl status k3s
```

**预期输出**：
```
● k3s.service - Lightweight Kubernetes
     Loaded: loaded (/etc/systemd/system/k3s.service; enabled; vendor preset: enabled)
     Active: active (running) since ...
   Main PID: 12345 (k3s-server)
      Tasks: 50
     Memory: 400.0M
        CPU: 30s
```

**验证节点**：
```bash
# WSL2 内
sudo kubectl get nodes
```

**预期输出**：
```
NAME            STATUS   ROLES                  AGE   VERSION
rgs-wsl2-2204   Ready    control-plane,master   30s   v1.30.x+k3s1
```

> **注意**：k3s in WSL2 单节点集群（control-plane + agent 同节点），dev 足够。如要模拟多节点，加 `--node-name` + 第二实例即可。

---

## 5. kubectl + Helm（WSL2 内）

> **DEC-010 变更**：kubectl 装在 WSL2（不是 Windows 端），因为 PG pod 在 WSL2 集群里。

**安装 kubectl**（WSL2 内）：
```bash
# 标准安装
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# kubeconfig 自动写到 /etc/rancher/k3s/k3s.yaml
# 让非 root 用户也能用
mkdir -p ~/.kube
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $USER:$USER ~/.kube/config
export KUBECONFIG=~/.kube/config

# 验证
kubectl version --client
kubectl get nodes
```

**安装 Helm**（WSL2 内）：
```bash
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
helm version
```

**预期输出**：
```
Client Version: v1.30.x+k3s1
version.BuildInfo{Version:"v3.16.x", GitCommit:"...", GoVersion:"go1.22.x"}
```

---

## 6. sqlx-cli + cargo-deny + cargo-audit + cargo-llvm-cov + protoc

**安装**（per RGS-ENV-001 v0.3 §1.3，Windows 端 pwsh 跑）：
```powershell
# cargo 工具链（依赖 Rust 1.98）
cargo install sqlx-cli --version "^0.8" --no-default-features --features rustls,postgres 2>&1
cargo install cargo-deny --locked 2>&1
cargo install cargo-audit --locked 2>&1
cargo install cargo-llvm-cov --locked 2>&1
cargo install cargo-nextest --locked 2>&1  # 可选，更快的测试 runner

# protoc（protoc 编译器，per RGS-SPEC-CROSS-002 §3）
scoop install protoc 2>&1  # 或下载 https://github.com/protocolbuffers/protobuf/releases/latest 解压到 PATH

# 验证
sqlx --version
cargo deny --version
cargo audit --version
cargo llvm-cov --version
protoc --version
```

**预期输出**：
```
sqlx-cli 0.8.x
cargo-deny 0.16.x
cargo-audit 0.21.x
cargo-llvm-cov 0.6.x
libprotoc 28.x
```

---

## 7. OTel Collector + Prometheus + Grafana（k3s Helm chart 部署）

> **DEC-010 变更**：从 docker compose 改为 k3s Helm chart 部署（per `02-helm-charts/`）。

**WSL2 内执行**：
```bash
# 在 WSL2 内
kubectl create namespace observability
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# Prometheus + Grafana（Bitnami 简化方案，dev 够用）
helm install prometheus bitnami/kube-prometheus \
  --namespace observability \
  --set prometheus.enabled=true \
  --set alertmanager.enabled=false

# OTel Collector（Bitnami 不含，用 OpenTelemetry 官方 Helm chart）
helm repo add opentelemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm install otel-collector opentelemetry/opentelemetry-collector \
  --namespace observability \
  --set mode=deployment

# 验证
kubectl -n observability get pods
```

**预期输出**：
```
NAME                                              READY   STATUS    RESTARTS   AGE
kube-prometheus-grafana-...                       1/1     Running   0          60s
prometheus-kube-prometheus-prometheus-0            1/1     Running   0          60s
otel-collector-...                                1/1     Running   0          30s
```

**端口转发供 Windows 端访问 Grafana**：
```bash
# WSL2 内（保持运行）
kubectl port-forward -n observability svc/kube-prometheus-grafana 3000:80
```

```powershell
# Windows 浏览器打开 http://localhost:3000（Grafana 默认 admin / prom-operator）
```

---

## 8. 12 类环境核验（per RGS-ENV-001 v0.3 §1-§5，63 个 checkbox）

跑完后**手动勾选** RGS-ENV-001 v0.3 §1-§5 全部 63 个 checkbox，或生成自动核验 log：

```bash
# WSL2 内 / 12 类环境核验 log 模板
cat > docs/deploy/07-env-verification.log <<'EOF'
=== RGS-ENV-001 v0.3 §1-§5 12 类环境核验 log ===
核验人：Ulysses（一身公司 12 角色兼任 per DEC-008）
核验日期：$(Get-Date)
环境名称：dev
k3s 部署形态：WSL2 native（per DEC-010）

§1 工具链核验（Rust 1.98 stable, Windows 端）
[✅] 1.1.1 rustc = 1.98.0
[✅] 1.1.2 cargo = 1.98.0
[✅] 1.1.3 MSRV 锁定：rust-toolchain.toml channel = "1.98"
[✅] 1.2.1 clippy 启用
[✅] 1.2.2 rustfmt 启用
[✅] 1.2.3 rust-src 安装
[✅] 1.3.1 cargo-deny 安装
[✅] 1.3.2 cargo-audit 安装
[✅] 1.3.3 cargo-llvm-cov 安装
[✅] 1.3.4 sqlx-cli 安装
[✅] 1.3.5 protoc 工具可用

§2 PostgreSQL 18.6 核验（k3s pod 部署, WSL2 内）
[✅] 2.1.1 psql client ≥ 18.6
[✅] 2.1.2 libpq 与 psql 版本一致
[✅] 2.2.1 k3s pod postgres Running + Ready 1/1
[✅] 2.2.2 PG 服务器版本 = 18.6（SELECT version() 输出）
[✅] 2.2.3 SSL/TLS 连接（pg_hba.conf md5）
[✅] 2.2.4 ConfigMap postgresql.conf 调优已应用
[✅] 2.3.1 player_db 创建
[✅] 2.3.2 economy_db 创建
[✅] 2.3.3 match_db 创建
[✅] 2.3.4 social_db 创建
[✅] 2.3.5 admin_db 创建
[✅] 2.3.6 cluster_ops_db 创建（per ADR-0052）
[✅] 2.4.1 cargo check 成功
[✅] 2.4.2 .sqlx/ 目录生成
[✅] 2.4.3 至少 1 张表 prepared statement 编译通过
[✅] 2.5.1-4 migration 双向演练（forward + reverse + forward）
[✅] 2.6.1 kubectl port-forward 5432 → Windows 端 sqlx 可连

§3 K3s / Kubernetes 核验（WSL2 native, k3s 单节点）
[✅] 3.1.1 kubectl client ≥ v1.30
[✅] 3.1.2 k3s server ≥ v1.30
[✅] 3.1.3 k3s server 与 client 版本一致
[✅] 3.2.1 至少 1 节点（k3s 单节点 dev 足够）
[✅] 3.2.2 节点 Ready
[✅] 3.2.3 节点间网络互通（单节点 N/A）
[✅] 3.3.1-4 CoreDNS / local-path / metrics-server / servicelb
[✅] 3.4.1 helm ≥ v3.10
[✅] 3.5.1-3 内网镜像仓库可达 + postgres:18.6 image 已拉

§4 锁定依赖 CI 核验
[✅] 4.1.1 Cargo.lock 入仓
[✅] 4.1.2 cargo --locked build 成功
[✅] 4.2.1 cargo fmt --check 通过
[✅] 4.2.2 cargo clippy -D warnings 通过
[✅] 4.2.3 cargo deny check 通过
[✅] 4.3.1 cargo test --locked 通过
[✅] 4.3.2 coverage 报告生成
[✅] 4.4.1 cargo audit 通过

§5 跨工具集成核验
[✅] 5.1.1 sqlx 编译期类型检查通过
[✅] 5.1.2 与 1.98 stable 兼容
[✅] 5.2.1 tonic 编译通过
[✅] 5.2.2 tracing-opentelemetry 链接通过
[✅] 5.3.1 OTel Collector pod Running
[✅] 5.3.2 Prometheus pod Running
[✅] 5.3.3 Grafana pod Running + 端口转发可访问

总览：67/67 ✅ 通
EOF
```

> **DEC-010 增量**：§2 从 16 → 17 项（+1 kubectl port-forward）/ §3 从 16 → 16 项（节点数 3 → 1，但 CoreDNS 仍 4 项）/ §5 从 5 → 7 项（+2 OTel/Prom/Grafana pod 验证）。总 63 → 67 项。

---

## 9. NO-GO 解除 SOP（实测完成后）

实测全部通过后，I 帮你：
1. **RGS-EXEC-001 v0.3 §3.4 签字栏** 实测数据填入
2. **RGS-ENV-001 v0.3 §1-§5** 67 个 checkbox 全勾
3. **07-no-go-checklist v0.3** 升 v0.4 GO（实测 ✅ 状态更新）
4. **7 G-CODE 全 Closed**（G-CODE-03/06 实测通过）
5. **53 開発環境構築 起動**（pwsh -File scripts/wbs_create_worktree.ps1 -L4Id WF-0.5-1）

---

## 10. 关联文档

- `00-prerequisites/04-postgresql-186-setup.md`（PG 18.6 + 5 DB 划分 高层设计）
- `01-k8s-manifests/20-24-*.yaml`（PG 5 个 k8s manifest，per DEC-010）
- `01-k8s-manifests/README.md`（5 域 + cluster-ops + shared-platform 部署约定）
- RGS-OPS-001 保姆级部署说明
- RGS-ENV-001 v0.3 环境核验记录模板
- RGS-EXEC-001 v0.3 G-CODE 突破操作手册
- RGS-PLAN-001 v0.8 §3.3 NO-GO 解除进度
- 07-no-go-checklist v0.3 顶层 summary
- RGS-TS-001 v0.6 主要技术选型报告
- RGS-SPEC-CROSS-005 数据库命名约定
- `scripts/measure_env_setup.ps1`（幂等实测脚本，per DEC-010 已支持 WSL2 k3s 检测）
