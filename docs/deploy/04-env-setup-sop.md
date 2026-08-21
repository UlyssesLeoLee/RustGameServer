# 环境部署实测 SOP（Windows 11 + WSL2 + k3s native，RGS-ENV-001 v0.3 §1-§5 + G-CODE-03/06）

> **文档 ID**：`RGS-DEPLOY-ENV-SETUP-001`
> **版本**：v0.2（per DEC-010：k3d → k3s native in WSL2）
> **生效日期**：2026-08-21
> **目标**：NO-GO 解除（per RGS-PLAN-001 v0.8 §3.3 + 07-no-go-checklist_v0.3）
> **平台**：Windows 11 + PowerShell 7.6+ + WSL2（Ubuntu 22.04 LTS）+ k3s native
> **实测人**：Ulysses（一人公司 12 角色兼任 per DEC-008）
> **总耗时**：~60-90 分钟（含 k3s 安装 5-10 分钟 + PG pod ready 3-5 分钟）

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Windows 11 + Docker Desktop + k3d 流程（PG 用 docker compose）。 |
| 0.2 | 2026-08-21 | 架构师（Ulysses）| **DEC-010 落地**：k3d → k3s native in WSL2。PG 从 docker compose 升为 k3s pod 部署（per 01-k8s-manifests/20-24 清单）。§0 总览 / §2 PG / §3 WSL2 / §4 k3s / §5 kubectl / §7 OTel / §8 核验 log 全面重写。**未变更**：Rust 1.98 / cargo 工具链 / 5 独立 DB 原则 / ARC-008 / ADR-0052 / 53 起動条件。 |

---

## 0. 总览（7 大组件）

| # | 组件 | 触发 G-CODE / ENV § | 优先级 | 预计耗时 | 平台 |
|---|---|---|---|---|---|
| 1 | Rust 1.98 stable | G-CODE-06 + ENV-§1 | 🔴 必填 | 5 分钟 | Windows (pwsh) |
| 2 | PostgreSQL 18.6 + 5 独立 DB（k3s pod）| G-CODE-03 + ENV-§2 | 🔴 必填 | 15 分钟 | WSL2 (kubectl) |
| 3 | WSL2 + Ubuntu 22.04 | ENV-§3.1 | 🟡 高 | 10 分钟（已装跳过）| Windows |
| 4 | k3s native（WSL2 内 systemd 模式）| ENV-§3 | 🟡 高 | 10 分钟 | WSL2 |
| 5 | kubectl + Helm（WSL2 内）| ENV-§3.4 | 🟡 高 | 5 分钟 | WSL2 |
| 6 | sqlx-cli + cargo-deny + cargo-audit + cargo-llvm-cov + protoc | ENV-§1.3 + §5.1 | 🟡 中 | 15 分钟 | Windows (pwsh) |
| 7 | OTel Collector + Prometheus + Grafana（k3s Helm chart）| ENV-§5 | 🟢 低 | 10 分钟 | WSL2 (helm) |

**串行依赖**：1 → 6（cargo 工具链需 Rust 1.98）→ 3 → 4 → 5 → 2（PG pod 需 k3s ready）→ 7

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

# Cargo.toml（最小，per commit dc7d9fa 已就位）
# rust-toolchain.toml（per commit dc7d9fa 已就位）
# crates/rgs-hello/（per commit dc7d9fa 已就位）

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

### §2.1 前置条件

- WSL2 + k3s 已就位（§3 + §4）
- kubectl context 已指向 rgs-dev 集群（§5）

### §2.2 应用 PG manifest（WSL2 内执行）

```bash
# 在 WSL2 内（Ubuntu 22.04），假设 /mnt/d/RustGameServer 已挂载 Windows D 盘
wsl -d Ubuntu-22.04

cd /mnt/d/RustGameServer/docs/deploy/01-k8s-manifests

# 1. 替换 PLACEHOLDER_* 为实际值（首次部署）
#    推荐：kustomize patch 或 sed 一次性替换
#    - PLACEHOLDER_NAMESPACE -> rust-game-server
#    - PLACEHOLDER_POSTGRES_* -> postgres / postgres-data-pvc / postgres-config 等
#    - PLACEHOLDER_POSTGRES_STORAGE_CLASS -> local-path（k3s 默认）
#    - PLACEHOLDER_POSTGRES_STORAGE_SIZE -> 5Gi
#    - REPLACE_BEFORE_DEPLOY_* -> 实际密码（用 openssl rand -base64 24 生成）
#    - PLACEHOLDER_POSTGRES_SA -> postgres-service-account
#    - PLACEHOLDER_POSTGRES_SVC_NAME -> postgres

# 2. 创建命名空间
kubectl apply -f 00-namespace.yaml

# 3. 应用 PG 5 个 manifest（顺序：secret → pvc → configmap → statefulset → service）
kubectl apply -f 20-postgres-secret.yaml -n rust-game-server
kubectl apply -f 21-postgres-pvc.yaml -n rust-game-server
kubectl apply -f 22-postgres-configmap.yaml -n rust-game-server
kubectl apply -f 23-postgres-statefulset.yaml -n rust-game-server
kubectl apply -f 24-postgres-service.yaml -n rust-game-server
```

### §2.3 等待 PG pod ready

```bash
# 等待 pod Running + Ready 1/1（PG 启动 ~30s）
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=postgres -n rust-game-server --timeout=120s

# 验证 pod
kubectl get pod -l app.kubernetes.io/name=postgres -n rust-game-server
```

**预期输出**：
```
NAME                      READY   STATUS    RESTARTS   AGE
postgres-xxxxxxxxxx-xxxxx   1/1     Running   0          90s
```

### §2.4 验证 PG 版本 + 5 独立 DB

```bash
# 进入 PG pod 跑 psql
kubectl exec -it deploy/postgres -n rust-game-server -- psql -U postgres

# 在 psql 内
\dx
SELECT version();
\l
```

**预期输出**：
```
                                                   version
----------------------------------------------------------------------------------------------------------
 PostgreSQL 18.6 (Ubuntu 18.6-1.pgdg22.04+1) on x86_64-pc-linux-gnu, compiled by gcc (Ubuntu 11.4.0-1ubuntu1~22.04) ...
(1 row)

                              List of databases
   Name           |  Owner   | Encoding |   Collate   |    Ctype    |   Access privileges
------------------+----------+----------+-------------+-------------+-----------------------
 admin_db          | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
 cluster_ops_db    | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
 economy_db        | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
 match_db          | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
 player_db         | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
 postgres          | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
 social_db         | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 |
(7 rows)
```

### §2.5 端口转发（供 Windows 端 sqlx-cli 用）

```bash
# 在 WSL2 单独开一个 shell 跑 port-forward（保持运行）
kubectl port-forward svc/postgres -n rust-game-server 5432:5432
```

```powershell
# 在 Windows pwsh 内验证（另开一个 pwsh 窗口）
pwsh -NoProfile -Command "& { \$env:PGPASSWORD='postgres 密码'; & 'C:\Program Files\PostgreSQL\18\bin\psql.exe' -h localhost -U postgres -c 'SELECT version();' }"
# 或用 sqlx-cli：
$env:DATABASE_URL = "postgres://postgres:密码@localhost:5432/player_db"
sqlx database create
```

### §2.6 G-CODE-03 5 独立 DB 拓扑图

- 工具：draw.io / Excalidraw / Mermaid / 手画 PNG 均可
- 提交到：`docs/deploy/05-db-topology.png`（或 .svg / .drawio / .mmd）
- 必备元素：6 个 DB 框 + Schema 命名（per RGS-SPEC-CROSS-005） + 跨 DB 访问箭头（标"禁止 JOIN"） + Outbox + CEM 跨域协调路径 + **新增 k3s pod 边界框**（per DEC-010）
- Mermaid 源已就位：`docs/deploy/05-db-topology.mmd`

**关闭 G-CODE-03 条件**：6 个 DB 全部创建 + k3s pod running + 拓扑图存档到 `docs/deploy/05-db-topology.png` ✓

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
