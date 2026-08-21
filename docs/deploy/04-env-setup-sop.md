# 环境部署实测 SOP（Windows 11 适配，RGS-ENV-001 v0.3 §1-§5 + G-CODE-03/06）

> **文档 ID**：`RGS-DEPLOY-ENV-SETUP-001`
> **版本**：v0.1
> **生效日期**：2026-08-21
> **目标**：NO-GO 解除（per RGS-PLAN-001 v0.8 §3.3 + 07-no-go-checklist_v0.3）
> **平台**：Windows 11 + PowerShell 7.6+ + Docker Desktop（WSL2 后端）
> **实测人**：Ulysses（一人公司 12 角色兼任 per DEC-008）
> **总耗时**：~45-75 分钟（含 K3s 启动 5-10 分钟）

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Windows 11 适配的 7 大组件实测 SOP。依据 RGS-OPS-001 §1.3 + RGS-ENV-001 v0.3 §1-§5 + G-CODE-03/06 关闭条件。 |

---

## 0. 总览（7 大组件）

| # | 组件 | 触发 G-CODE / ENV § | 优先级 | 预计耗时 |
|---|---|---|---|---|
| 1 | Rust 1.98 stable | G-CODE-06 + ENV-§1 | 🔴 必填 | 5 分钟 |
| 2 | PostgreSQL 18.6 + 5 独立 DB | G-CODE-03 + ENV-§2 | 🔴 必填 | 10 分钟 |
| 3 | Docker Desktop（含 WSL2）| ENV-§3.3 / §5.3 | 🟡 高 | 5 分钟（已装跳过）|
| 4 | K3s（用 k3d 在 Docker 内跑）| ENV-§3 | 🟡 高 | 10 分钟 |
| 5 | kubectl + Helm | ENV-§3.4 | 🟡 高 | 5 分钟 |
| 6 | sqlx-cli + cargo-deny + cargo-audit + cargo-llvm-cov + protoc | ENV-§1.3 + §5.1 | 🟡 中 | 15 分钟 |
| 7 | OTel Collector + Prometheus + Grafana（Docker compose）| ENV-§5 | 🟢 低 | 5 分钟 |

**串行依赖**：1 → 2 → 6（cargo 工具链需 Rust 1.98）→ 3 → 4 → 5 → 7

---

## 1. Rust 1.98 stable（触发 G-CODE-06）

**安装**：
```powershell
# PowerShell 7（不要用 powershell 5.1）
pwsh -NoProfile -Command "irm https://win.rustup.rs/x86_64 | iex"
# 选项：1) Proceed with installation (default)
# 选项：2) Customize installation（可选：改 host triple）

# 安装后关闭 PS，重开 pwsh 验证
pwsh -NoProfile -Command "rustc --version; cargo --version; rustup show"
```

**预期输出**：
```
rustc 1.98.0 (xxxxx 2026-08-XX)
cargo 1.98.0 (xxxxx 2026-08-XX)
```

**最小 workspace 验证**（per G-CODE-06）：
```bash
# 在 D:\RustGameServer 创建最小 Rust workspace
mkdir -p crates/rgs-hello/src
cd D:\RustGameServer

# Cargo.toml（最小）
cat > Cargo.toml <<'EOF'
[workspace]
members = ["crates/rgs-hello"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.98"
EOF

# crates/rgs-hello/Cargo.toml
mkdir -p crates/rgs-hello
cat > crates/rgs-hello/Cargo.toml <<'EOF'
[package]
name = "rgs-hello"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
EOF

# crates/rgs-hello/src/main.rs
cat > crates/rgs-hello/src/main.rs <<'EOF'
fn main() {
    println!("RGS Rust 1.98 OK");
}
EOF

# rust-toolchain.toml 锁版本
cat > rust-toolchain.toml <<'EOF'
[toolchain]
channel = "1.98"
profile = "minimal"
EOF

# 编译 + 测试
cargo build --locked 2>&1 | tee docs/deploy/06-rust-198-build.log
cargo test --locked --workspace 2>&1 | tee -a docs/deploy/06-rust-198-build.log
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

## 2. PostgreSQL 18.6 + 5 独立 DB（触发 G-CODE-03）

**安装**（per RGS-OPS-001 §1.3 官方源 + RGS-ENV-001 v0.3 §2）：

**方案 A（推荐，Windows 原生）**：
```powershell
# 1. 下载 PostgreSQL 18.6 Windows installer
#    来自 https://www.postgresql.org/download/windows/
#    或 EnterpriseDB 安装包

# 2. 安装时：
#    - Port: 5432（默认）
#    - 密码：设一个本地密码（Ulysses 自行保存到密码管理器）
#    - Locale: C / UTF-8

# 3. 验证
pwsh -NoProfile -Command "& { \$env:PGPASSWORD='你的密码'; & 'C:\Program Files\PostgreSQL\18\bin\psql.exe' -U postgres -c 'SELECT version();' }"
```

**方案 B（Docker 化，per RGS-OPS-001 §2.3）**：
```bash
# 在 D:\RustGameServer\docker-compose-dev.yml 创建
mkdir -p docs/deploy
cat > docs/deploy/pg-compose.yml <<'EOF'
version: "3.8"
services:
  postgres:
    image: postgres:18.6
    container_name: rgs-pg-186
    environment:
      POSTGRES_PASSWORD: ulysses_local
    ports:
      - "5432:5432"
    volumes:
      - rgs_pg_data:/var/lib/postgresql/data
volumes:
  rgs_pg_data:
EOF

docker compose -f docs/deploy/pg-compose.yml up -d
sleep 5
docker exec rgs-pg-186 psql -U postgres -c "SELECT version();"
```

**5 独立 DB 创建**（per ARC-008 + RGS-SPEC-CROSS-005）：
```bash
# 选 B 方案继续
for db in player_db economy_db match_db social_db admin_db cluster_ops_db; do
  docker exec rgs-pg-186 psql -U postgres -c "CREATE DATABASE $db;"
done
docker exec rgs-pg-186 psql -U postgres -c "\l" | grep -E 'player_db|economy_db|match_db|social_db|admin_db|cluster_ops_db'
```

**预期输出**（6 个 DB 都存在）：
```
 player_db     | ulysses | UTF8     | ...
 economy_db    | ulysses | UTF8     | ...
 match_db      | ulysses | UTF8     | ...
 social_db     | ulysses | UTF8     | ...
 admin_db      | ulysses | UTF8     | ...
 cluster_ops_db| ulysses | UTF8     | ...
```

**G-CODE-03 5 独立 DB 拓扑图**：
- 工具：draw.io / Excalidraw / Mermaid / 手画 PNG 均可
- 提交到：`docs/deploy/05-db-topology.png`（或 .svg / .drawio / .mmd）
- 必备元素：6 个 DB 框 + Schema 命名（per RGS-SPEC-CROSS-005） + 跨 DB 访问箭头（标"禁止 JOIN"） + Outbox + CEM 跨域协调路径

**关闭 G-CODE-03 条件**：6 个 DB 全部创建 + 拓扑图存档到 `docs/deploy/05-db-topology.png` ✓

---

## 3. Docker Desktop（含 WSL2）

**安装**：
```powershell
# 下载 Docker Desktop for Windows
# https://www.docker.com/products/docker-desktop/

# 安装时勾选 "Use WSL 2 instead of Hyper-V"

# 验证
pwsh -NoProfile -Command "docker --version; docker compose version; wsl --status"
```

**预期输出**：
```
Docker version 24.x.x, build xxxxx
Docker Compose version v2.x.x
默认发行版: Ubuntu
```

---

## 4. K3s（用 k3d 在 Docker 内跑）

**安装 k3d**（per RGS-ENV-001 v0.3 §3 + k3s 推荐方案）：
```powershell
# 用 choco 或 scoop
scoop install k3d 2>&1
# 备选：winget install k3d

# 验证
k3d --version
```

**创建 dev K3s 集群**（1 control-plane 节点足够 dev）：
```bash
k3d cluster create rgs-dev \
  --servers 1 \
  --agents 2 \
  --port 8080:80@loadbalancer \
  --port 8443:443@loadbalancer \
  --wait

# 验证
kubectl get nodes
```

**预期输出**：
```
NAME                  STATUS   ROLES                  AGE   VERSION
k3d-rgs-dev-server-0   Ready    control-plane,master   30s   v1.30.x+k3s1
k3d-rgs-dev-agent-0    Ready    <none>                 20s   v1.30.x+k3s1
k3d-rgs-dev-agent-1    Ready    <none>                 20s   v1.30.x+k3s1
```

**部署 OTel Collector + Prometheus + Grafana**（per RGS-OPS-001 §4.2）：
```bash
# 简化方案：用 Bitnami Helm chart
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

kubectl create namespace observability
helm install prometheus bitnami/kube-prometheus \
  --namespace observability \
  --set prometheus.enabled=true \
  --set alertmanager.enabled=false

# 验证
kubectl -n observability get pods
```

**预期输出**：
```
NAME                                              READY   STATUS
alertmanager-kube-prometheus-alertmanager-0       0/1     Pending
kube-prometheus-grafana-...                       0/1     Pending
prometheus-kube-prometheus-prometheus-0            0/1     Pending
```

---

## 5. kubectl + Helm

**安装**：
```powershell
# 用 scoop 一键
scoop install kubectl helm 2>&1

# 备选：winget
winget install Kubernetes.kubectl
winget install Helm.Helm

# 验证
kubectl version --client
helm version
```

**预期输出**：
```
Client Version: v1.30.x
version.BuildInfo{Version:"v3.16.x", GitCommit:"...", GoVersion:"go1.22.x"}
```

---

## 6. sqlx-cli + cargo-deny + cargo-audit + cargo-llvm-cov + protoc

**安装**（per RGS-ENV-001 v0.3 §1.3）：
```bash
# cargo 工具链（依赖 Rust 1.98）
cargo install sqlx-cli --version "^0.8" --no-default-features --features rustls,postgres 2>&1
cargo install cargo-deny --locked 2>&1
cargo install cargo-audit --locked 2>&1
cargo install cargo-llvm-cov --locked 2>&1
cargo install cargo-nextest --locked 2>&1  # 可选，更快的测试 runner

# protoc（protoc 编译器，per RGS-SPEC-CROSS-002 §3）
# Windows：下载 https://github.com/protocolbuffers/protobuf/releases/latest
# 解压到 C:\protoc\bin，加 PATH
scoop install protoc 2>&1  # 备选

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

## 7. OTel Collector + Prometheus + Grafana（per §4 已部署）

如果 §4 已 helm install prometheus 成功，本节跳过。否则用 Docker compose 简化方案：
```bash
# 在 D:\RustGameServer\docs\deploy\observability-compose.yml
cat > docs/deploy/observability-compose.yml <<'EOF'
version: "3.8"
services:
  otel-collector:
    image: otel/opentelemetry-collector-contrib:0.110.0
    container_name: rgs-otel-collector
    command: ["--config=/etc/otelcol/config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otelcol/config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP

  prometheus:
    image: prom/prometheus:v2.55.0
    container_name: rgs-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:11.3.0
    container_name: rgs-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=ulysses_local
EOF

docker compose -f docs/deploy/observability-compose.yml up -d
sleep 5
docker ps | grep -E 'rgs-otel|rgs-prometheus|rgs-grafana'
```

**验证**：
- 浏览器打开 `http://localhost:3000`（Grafana，默认 admin/ulysses_local）
- 浏览器打开 `http://localhost:9090`（Prometheus）

---

## 8. 12 类环境核验（per RGS-ENV-001 v0.3 §1-§5，63 个 checkbox）

跑完后**手动勾选** RGS-ENV-001 v0.3 §1-§5 全部 63 个 checkbox，或生成自动核验 log：

```bash
# 12 类环境核验 log 模板
cat > docs/deploy/07-env-verification.log <<'EOF'
=== RGS-ENV-001 v0.3 §1-§5 12 类环境核验 log ===
核验人：Ulysses（一身公司 12 角色兼任 per DEC-008）
核验日期：$(Get-Date)
环境名称：dev

§1 工具链核验（Rust 1.98 stable）
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

§2 PostgreSQL 18.6 核验
[✅] 2.1.1 psql = 18.6
[✅] 2.1.2 libpq 与 psql 版本一致
[✅] 2.2.1 服务器版本 = 18.6
[✅] 2.2.2 SSL/TLS 连接
[✅] 2.2.3 pg_hba.conf 配置核验
[✅] 2.3.1-5 5 独立 DB（player_db / economy_db / match_db / social_db / admin_db）
[✅] 2.4.1 cargo check 成功
[✅] 2.4.2 .sqlx/ 目录生成
[✅] 2.4.3 至少 1 张表 prepared statement 编译通过
[✅] 2.5.1-4 migration 双向演练（forward + reverse + forward）

§3 K3s / Kubernetes 核验（k3d）
[✅] 3.1.1 kubectl client ≥ v1.30
[✅] 3.1.2 K3s server ≥ v1.30
[✅] 3.1.3 k3s server 与 client 版本一致
[✅] 3.2.1 至少 3 节点（1 server + 2 agent）
[✅] 3.2.2 所有节点 Ready
[✅] 3.2.3 节点间网络互通
[✅] 3.3.1-4 CoreDNS / Traefik / local-path / metrics-server
[✅] 3.4.1 helm ≥ v3.10
[✅] 3.5.1-3 内网镜像仓库可达 + distroless/cc-debian12 可拉取

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
[✅] 5.3.1 distroless 镜像构建成功

总览：63/63 ✅ 通
EOF
```

**注意**：上面 log 模板是**预期格式**，Ulysses 实测后用 `[✅]` / `[❌]` 替换 `[✅]` 真实值。

---

## 9. NO-GO 解除 SOP（实测完成后）

实测全部通过后，I 帮你：
1. **RGS-EXEC-001 v0.3 §3.4 签字栏** 实测数据填入
2. **RGS-ENV-001 v0.3 §1-§5** 63 个 checkbox 全勾
3. **07-no-go-checklist v0.3** 升 v0.4 GO（实测 ✅ 状态更新）
4. **7 G-CODE 全 Closed**（G-CODE-03/06 实测通过）
5. **53 開発環境構築 起動**（pwsh -File scripts/wbs_create_worktree.ps1 -L4Id WF-0.5-1）

---

## 10. 关联文档

- RGS-OPS-001 保姆级部署说明
- RGS-ENV-001 v0.3 环境核验记录模板
- RGS-EXEC-001 v0.3 G-CODE 突破操作手册
- RGS-PLAN-001 v0.8 §3.3 NO-GO 解除进度
- 07-no-go-checklist v0.3 顶层 summary
- RGS-TS-001 v0.6 主要技术选型报告
- RGS-SPEC-CROSS-005 数据库命名约定
