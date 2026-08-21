# 保姆级部署说明（保姆的——全步骤、命令、配置、验证）

**RustGameServer 分布式游戏服务器基础设施**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OPS-001 |
| 版本 | 0.3 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师 |
| 依据标准 | RGS-REQ-027 App集群自动化部署脚本 + RGS-BAS-024 集群部署基本设计 + RGS-DTL-024 详细设计 |
| 适用对象 | 首次部署 / 新成员 / 应急恢复 |
| 预计时长 | 本地 30 min，CI 1 h，预发布 4 h，生产 8 h |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版。覆盖本地→CI→预发布→生产 全链路 |
| **0.2** | 2026-08-20 | 架构师 | CI 示例改为不可变 Action SHA、最小权限与仅受保护主分支的发布/签名；多区域 runbook 增加独立 context、复制仲裁、fencing、DNS 切流及演练证据要求 |
| **0.3** | 2026-08-21 | 架构师 | 同步 RGS-IMPL-001：Rust 1.98 stable 目标与 GA Gate、PostgreSQL 18.4、cargo-llvm-cov、域 migration owner、distroless nonroot、不可变镜像标签、Helm/Argo/OTel 约定。 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师 | 2026-08-19 | — |
| 评审（SRE） |  |  |  |
| 审批（负责人） |  |  |  |

---

> **实施前限制**：本文件中的命令和模板是获 PH-0.5 书面授权后的执行说明，不授权当前仓库创建 workspace、迁移、Kubernetes/Helm 制品或发布。版本、目录、密钥与发布边界以 [RGS-IMPL-001](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md) 为准；Rust 1.98 必须在 stable GA 且 CI 核验后才可使用。

---

## 目录（目次）

1. [环境准备](#1-环境准备)
2. [本地开发环境](#2-本地开发环境)
3. [CI/CD 配置](#3-cicd-配置)
4. [预发布/测试环境](#4-预发布测试环境)
5. [生产环境](#5-生产环境)
6. [灰度发布与回滚](#6-灰度发布与回滚)
7. [日常运维](#7-日常运维)
8. [故障排查](#8-故障排查)
9. [应急恢复](#9-应急恢复)
10. [附录](#10-附录)

---

## 1. 环境准备

## 1.1 硬件最低要求

| 角色 | CPU | 内存 | 磁盘 | 网络 |
|---|---|---|---|---|
| **本地开发机** | 4 核 | 16 GB | 50 GB SSD | 1 Gbps |
| **CI Runner** | 4 核 | 8 GB | 100 GB SSD | 1 Gbps |
| **预发布节点**（每节点） | 8 核 | 32 GB | 200 GB SSD | 10 Gbps |
| **生产网关节点** | 16 核 | 64 GB | 200 GB NVMe | 25 Gbps |
| **生产运行时节点** | 32 核 | 128 GB | 500 GB NVMe | 25 Gbps |
| **生产 DB 主** | 32 核 | 128 GB | 1 TB NVMe（RAID-10） | 25 Gbps |

> **为什么这么定**：实时运行时 20Hz tick + 100k CCU 性能预算（参照 RGS-REQ-001 §9.2 NFR-PE-001~006）。CPU 决定 tick 处理能力，内存决定实体容量，NVMe 决定检查点写盘。

## 1.2 操作系统

| 环境 | 操作系统 |
|---|---|
| 本地开发 | macOS 14+ / Ubuntu 22.04 LTS / Windows 11 + WSL2 |
| 服务器 | **Ubuntu 22.04 LTS**（统一） / 容器内为 `ubuntu:22.04` |

> **为什么统一 Ubuntu**：所有运维脚本、systemd 单元、Prometheus exporter 均针对 Ubuntu 优化。混 OS 会导致 cross-compile 与故障排查成本翻倍。

## 1.3 软件依赖清单

| 软件 | 版本 | 安装方式 | 用途 |
|---|---|---|---|
| **Git** | 2.40+ | `apt install git` | 代码管理 |
| **Rust toolchain** | 1.98 stable（用户目标；GA 前不可用） | `rustup` | 编译 |
| **PostgreSQL** | 18.4 | 官方源 | 5 个独立 DB |
| **Redis** | 7.2+ | 官方源 | 缓存 / 限流 / 会话 |
| **Docker** | 24+ | 官方源 | 容器化（CI / 预发布） |
| **Kubernetes** | 1.28+ | kubeadm / EKS / GKE | 生产编排 |
| **Helm** | 3.13+ | 官方脚本 | 模板渲染 |
| **k6** | 0.49+ | 二进制 | 负载测试 |
| **OpenResty** | 1.21+ | 官方源 | CDN 边缘 + WAF |
| **OTel Collector** | 0.96+ | 容器 | 可观测性收集 |
| **Grafana** | 10+ | 容器 | 仪表盘 |
| **Prometheus** | 2.48+ | 容器 | 指标存储 |
| **MinIO** | RELEASE.2024-08+ | 容器 | 自托管对象存储（CDN 源） |
| **NATS** | 2.10+ | 容器 | 内部消息（可选） |

## 1.4 网络与端口规划

| 服务 | 端口（容器内） | 协议 | 暴露 |
|---|---|---|---|
| 网关（GW） | 7000/UDP, 7001/TCP | QUIC | NodePort/LoadBalancer |
| API 网关 | 8080/TCP | HTTP/2 | ClusterIP |
| 运行时（RT） | 9000/TCP | gRPC | ClusterIP |
| 业务服务 | 9100-9105/TCP | gRPC | ClusterIP |
| PostgreSQL | 5432/TCP | PG | ClusterIP |
| Redis | 6379/TCP | RESP | ClusterIP |
| OTel Collector | 4317/gRPC, 4318/HTTP | OTLP | ClusterIP |
| Prometheus | 9090/TCP | HTTP | ClusterIP |
| Grafana | 3000/TCP | HTTP | Ingress |
| MinIO | 9000-9001/TCP | HTTP | ClusterIP |
| NATS | 4222/TCP | NATS | ClusterIP |

> **为什么这样规划**：QUIC 必须用 UDP 暴露，gRPC 走 ClusterIP 即可（由 API 网关或网关转发），可观测性端口仅内网暴露，PostgreSQL 不直接对外。

---

## 2. 本地开发环境

## 2.1 克隆代码

```bash
# 1. 克隆仓库
git clone https://github.com/<org>/RustGameServer.git
cd RustGameServer

# 2. 查看项目结构
ls -la
# 应该看到：
# - Cargo.toml         （workspace 根）
# - crates/            （子 crate 目录）
# - services/          （业务服务）
# - docs/              （本文档所在）
# - scripts/           （运维脚本）
# - k8s/               （K8s manifests）
# - tests/             （集成测试 + 模拟客户端）
```

## 2.2 安装 Rust 工具链

```bash
# 1. 安装 rustup（Linux/macOS）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 2. 验证
rustc --version    # 期望：经 CI 核验的 rustc 1.98.x stable
cargo --version    # 期望：与 rustc 1.98.x 配套；GA 前本步骤不得宣告通过

# 3. 安装组件
rustup component add clippy rustfmt
cargo install sqlx-cli --version 0.8
cargo install cargo-llvm-cov    # 覆盖率
cargo install k6                # 负载测试
```

> **为什么这些组件**：`clippy`/`rustfmt` 是 CI 必跑；`sqlx-cli` 仅由 DB owner 的受控 migration runner 使用；`llvm-cov` 计算覆盖率；`k6` 跑 TL-6 负载。实际执行须在 Gate 关闭后进行。

## 2.3 启动 PostgreSQL + Redis（Docker 方式）

```bash
# 1. 创建 docker-compose.yml
cat > docker-compose.dev.yml <<'EOF'
version: '3.8'
services:
  postgres:
    image: postgres:18.4
    container_name: rgs-postgres
    environment:
      POSTGRES_USER: rgs
      POSTGRES_PASSWORD: ${RGS_DEV_POSTGRES_PASSWORD:?set_in_local_env}
      POSTGRES_DB: rgs
    ports:
      - "5432:5432"
    volumes:
      - rgs-pg-data:/var/lib/postgresql/data
      - ./scripts/sql/init-databases.sql:/docker-entrypoint-initdb.d/01-init.sql:ro
    command: ["postgres", "-c", "shared_buffers=256MB", "-c", "max_connections=200"]
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rgs"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: rgs-redis
    command: ["redis-server", "--appendonly", "yes", "--maxmemory", "512mb", "--maxmemory-policy", "allkeys-lru"]
    ports:
      - "6379:6379"
    volumes:
      - rgs-redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  rgs-pg-data:
  rgs-redis-data:
EOF

# 2. 启动
docker compose -f docker-compose.dev.yml up -d

# 3. 验证
docker compose -f docker-compose.dev.yml ps
# 应该看到 postgres 和 redis 都 healthy

# 4. 验证连接
docker exec -it rgs-postgres psql -U rgs -c '\l'
# 应该看到 5 个 DB：player_db, economy_db, match_db, social_db, admin_db
```

## 2.4 初始化 5 个数据库

```bash
# 创建 SQL 脚本
mkdir -p scripts/sql
cat > scripts/sql/init-databases.sql <<'EOF'
-- 创建 5 个独立 DB（ARC-008 限界上下文隔离）
CREATE DATABASE player_db;
CREATE DATABASE economy_db;
CREATE DATABASE match_db;
CREATE DATABASE social_db;
CREATE DATABASE admin_db;

-- 每个 DB 启用扩展
\c player_db
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

\c economy_db
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

\c match_db
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

\c social_db
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

\c admin_db
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
EOF

# 重启 postgres 让 init 脚本执行
docker compose -f docker-compose.dev.yml restart postgres
```

## 2.5 配置环境变量

```bash
# .env.development（提交到 .gitignore）
cat > .env.development <<'EOF'
# 数据库
DATABASE_URL_PLAYER=postgres://rgs:rgs_dev_password@localhost:5432/player_db
DATABASE_URL_ECONOMY=postgres://rgs:rgs_dev_password@localhost:5432/economy_db
DATABASE_URL_MATCH=postgres://rgs:rgs_dev_password@localhost:5432/match_db
DATABASE_URL_SOCIAL=postgres://rgs:rgs_dev_password@localhost:5432/social_db
DATABASE_URL_ADMIN=postgres://rgs:rgs_dev_password@localhost:5432/admin_db

# Redis
REDIS_URL=redis://localhost:6379

# 服务发现
GATEWAY_BIND=0.0.0.0:7000
RUNTIME_BIND=0.0.0.0:9000
ECONOMY_BIND=0.0.0.0:9102

# 可观测性
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
RUST_LOG=info,rgs=debug

# 模式
RGS_MODE=development
RGS_SHUTDOWN_GRACE_SEC=10
EOF

# 加载
export $(cat .env.development | xargs)
```

## 2.6 运行数据库迁移

```bash
# 对每个 DB 跑迁移
for db in player_db economy_db match_db social_db admin_db; do
    echo ">>> Migrating $db"
    sqlx migrate run --database-url "postgres://rgs:rgs_dev_password@localhost:5432/$db" --source migrations/$db
done

# 验证
psql postgres://rgs:rgs_dev_password@localhost:5432/economy_db -c "\dt"
# 应该看到 inventory, inventory_item, wallet, ledger, outbox 等表
```

> **为什么用 sqlx-cli**：sqlx 编译期检查 SQL 合法性（`sqlx::query!` 宏），避免运行时 SQL 错误。CI 也用同一工具。

## 2.7 跑测试

```bash
# 1. 单元测试（UT）
cargo test --workspace --lib
# 期望：所有 24 份 UT 设计书的用例通过
# 耗时：< 5 min（QA-006 约束）

# 2. 集成测试（IT）
cargo test --workspace --test '*'
# 期望：所有 24 份 IT 设计书的用例通过
# 耗时：< 12 min

# 3. 覆盖率报告
cargo llvm-cov --workspace --html --output-dir coverage/
# 打开 coverage/index.html
# 期望：核心区域 ≥ 80%（QA-001）

# 4. 静态检查
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# 5. 文档生成（可选）
cargo doc --workspace --no-deps --open
```

## 2.8 启动开发服务器

```bash
# 终端 1：网关
cargo run -p rgs-gateway

# 终端 2：运行时
cargo run -p rgs-runtime

# 终端 3：经济服务
cargo run -p rgs-economy

# 终端 4：玩家服务
cargo run -p rgs-player

# 终端 5：模拟客户端（用于本地调试）
cargo run -p load-mock-client -- --profile development --target gateway:7000

# 看到日志：tick=20Hz, CCU=10
```

## 2.9 验证

```bash
# 1. 健康检查
curl http://localhost:9000/health
# 期望：{"status":"ok","version":"0.1.0"}

# 2. 注册测试账号
curl -X POST http://localhost:8080/api/v1/account/register \
  -H "Content-Type: application/json" \
  -d '{"device_id":"test-device-001","platform":"test","client_version":"0.1.0"}'
# 期望：返回 account_id

# 3. 查看日志（应看到 trace_id）
tail -f logs/rgs-gateway.log
```

---

## 3. CI/CD 配置

## 3.1 GitHub Actions 主流程

```bash
mkdir -p .github/workflows
```

**文件 `.github/workflows/ci.yml`**：

```yaml
name: CI
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

# 工作流默认只读；不得依赖仓库默认权限。
permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

jobs:
  # PR 仅执行无密钥、无 Registry 登录的校验。
  lint:
    runs-on: ubuntu-22.04
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - name: 安装固定 Rust 工具链
        run: rustup toolchain install 1.80.0 --profile minimal && rustup default 1.80.0
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo build --workspace --locked

  test-ut:
    needs: lint
    runs-on: ubuntu-22.04
    permissions:
      contents: read
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: rgs
          POSTGRES_PASSWORD: rgs_test
          POSTGRES_DB: rgs_test
        ports: ['5432:5432']
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 3s
          --health-retries 5
      redis:
        image: redis:7-alpine
        ports: ['6379:6379']
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - name: 安装固定 Rust 工具链
        run: rustup toolchain install 1.80.0 --profile minimal && rustup default 1.80.0
      - name: 创建 5 个 DB
        run: |
          for db in player_db economy_db match_db social_db admin_db; do
            PGPASSWORD=rgs_test psql -h localhost -U rgs -d rgs_test -c "CREATE DATABASE $db;"
          done
      - name: 跑迁移
        run: |
          for db in player_db economy_db match_db social_db admin_db; do
            sqlx migrate run --database-url "postgres://rgs:rgs_test@localhost:5432/$db" --source migrations/$db
          done
      - run: cargo test --workspace --lib
      - run: cargo tarpaulin --workspace --out Xml --output-dir coverage/
      - uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
        with:
          name: coverage
          path: coverage/

  test-it:
    needs: test-ut
    runs-on: ubuntu-22.04
    permissions:
      contents: read
    services:
      postgres:
        image: postgres:16-alpine
        env: { POSTGRES_USER: rgs, POSTGRES_PASSWORD: rgs_test, POSTGRES_DB: rgs_test }
        ports: ['5432:5432']
      redis:
        image: redis:7-alpine
        ports: ['6379:6379']
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - name: 安装固定 Rust 工具链
        run: rustup toolchain install 1.80.0 --profile minimal && rustup default 1.80.0
      - name: 准备 DB
        run: |
          for db in player_db economy_db match_db social_db admin_db; do
            PGPASSWORD=rgs_test psql -h localhost -U rgs -d rgs_test -c "CREATE DATABASE $db;"
            sqlx migrate run --database-url "postgres://rgs:rgs_test@localhost:5432/$db" --source migrations/$db
          done
      - run: cargo test --workspace --test '*'

  # 只有受保护 main 的 push 可以接触 Registry 或 OIDC；任何 PR（含同仓 PR）均不会运行本 job。
  build-image:
    needs: test-it
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: [self-hosted, linux, rgs-release]
    permissions:
      contents: read
      packages: write
      id-token: write
    strategy:
      matrix:
        service: [gateway, runtime, economy, player, match, social, admin]
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - uses: docker/setup-buildx-action@e468171a9de216ec08956ac3ada2f0791b6bd435 # v3.11.1
      - uses: docker/login-action@9780b0c442fbb1117ed29e0efdff1e18412f7567 # v3.3.0
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ github.token }}
      - uses: docker/metadata-action@369eb591f429131d6889c46b94e711f089e6ca96 # v5.6.1
        id: meta
        with:
          images: ghcr.io/${{ github.repository_owner }}/rgs-${{ matrix.service }}
          tags: |
            type=sha,prefix=
            type=ref,event=branch
      - uses: docker/build-push-action@263435318d21b8e681c14492fe198d362a7d2c83 # v6.18.0
        id: build
        with:
          context: .
          file: services/${{ matrix.service }}/Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
          sbom: true
          provenance: mode=max
      - name: 对镜像摘要做无密钥签名
        # rgs-release 基线镜像必须预装并验证 cosign；禁止在工作流中 curl | sh 安装签名器。
        env:
          IMAGE: ghcr.io/${{ github.repository_owner }}/rgs-${{ matrix.service }}@${{ steps.build.outputs.digest }}
        run: |
          test -n "${{ steps.build.outputs.digest }}"
          cosign sign --yes "$IMAGE"
```

上述 SHA 是不可变 Git 提交，不使用可变 `@vN`/`@stable` 标签。`build-image` 使用短期 `GITHUB_TOKEN` 推送和 GitHub OIDC 无密钥签名；PR 作业没有 Registry 登录步骤、没有 `secrets.*` 输入、也没有 `packages: write` 或 `id-token: write` 权限。Buildx 生成的 SBOM 与 provenance 必须随镜像摘要保存，进入部署仓库前由准入策略校验签名、SBOM 和 provenance 三者均存在且与同一 digest 绑定。

## 3.2 Dockerfile 模板

**文件 `services/gateway/Dockerfile`**（多阶段构建）：

```dockerfile
# ===== 阶段 1: builder =====
# 仅由已通过 G-CODE-06 的 CI 注入已发布、已核验的 stable 版本。
ARG RUST_VERSION=1.98
FROM rust:${RUST_VERSION}-slim-bookworm AS builder

WORKDIR /app

# 系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 复制 manifests（缓存优化）
COPY Cargo.toml Cargo.lock ./
COPY proto proto
COPY crates crates
COPY services services

# 编译（release 模式）
RUN cargo build --locked --release -p rgs-gateway-service

# ===== 阶段 2: runtime =====
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

# 复制二进制
COPY --from=builder /app/target/release/rgs-gateway-service /usr/local/bin/rgs-gateway-service

EXPOSE 7000/udp 7001/tcp

ENTRYPOINT ["/usr/local/bin/rgs-gateway-service"]
```

> **为什么多阶段构建**：运行时镜像必须按 digest 固定并以 nonroot 运行；健康检查由 Kubernetes probe 执行。生产制品只能使用 Git SHA/digest 与 OCI revision/version labels，dirty worktree 只能生成本地开发镜像。

## 3.3 触发条件总览

| 触发 | 阶段 | 时长 |
|---|---|---|
| PR 推送 | lint → test-ut → test-it → build-image | 约 30 min |
| main 推送 | + deploy-staging | 约 1 h |
| 标签推送（v*.*.*） | + deploy-prod（需手动 approval） | 约 2 h |

---

## 4. 预发布/测试环境

## 4.1 K8s 集群准备

```bash
# 1. 集群要求（PH-4 验证环境）
# - 至少 3 节点（1 control-plane + 2 worker）
# - 每节点 8 核 32 GB
# - 网络：Calico / Cilium（NetworkPolicy 必须）

# 2. 初始化（kubeadm 方式，仅 dev 集群）
sudo kubeadm init --pod-network-cidr=10.244.0.0/16
mkdir -p $HOME/.kube
sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config

# 3. 安装 CNI（Calico）
kubectl apply -f https://raw.githubusercontent.com/projectcalico/calico/v3.27.0/manifests/calico.yaml

# 4. 加入 worker 节点
# 在 worker 上：
sudo kubeadm join <control-plane-ip>:6443 --token <token> --discovery-token-ca-cert-hash <hash>
```

## 4.2 部署 OTel + Prometheus + Grafana

```bash
# 1. 创建命名空间
kubectl create namespace rgs-monitoring

# 2. 部署 OTel Collector
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm install otel-collector open-telemetry/opentelemetry-collector \
  -n rgs-monitoring \
  -f k8s/monitoring/otel-collector-values.yaml

# 3. 部署 Prometheus
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack \
  -n rgs-monitoring \
  -f k8s/monitoring/prometheus-values.yaml

# 4. 部署 Grafana（用上面 stack 自带）
# 导入仪表盘：Grafana → + → Import → 上传 k8s/monitoring/dashboards/*.json

# 5. 验证
kubectl port-forward -n rgs-monitoring svc/prometheus-operated 9090
# 浏览器打开 http://localhost:9090  应能看到 Prometheus
```

## 4.3 部署 MinIO（自托管对象存储）

```bash
# 1. 部署
helm repo add minio https://charts.min.io/
helm install minio minio/minio \
  -n rgs-storage \
  --create-namespace \
  --set rootUser=rgs-admin \
  --set rootPassword=$(openssl rand -base64 32) \
  --set persistence.size=200Gi \
  --set replicas=4 \
  --set mode=distributed

# 2. 初始化 Bucket
kubectl port-forward -n rgs-storage svc/minio 9000 &
mc alias set local http://localhost:9000 rgs-admin <password>
mc mb local/rgs-cdn
mc mb local/rgs-rules     # 风控规则仓库
mc anonymous set download local/rgs-cdn  # manifest 可公开下载

# 3. 验证
mc ls local
```

## 4.4 部署 PostgreSQL + Redis

```bash
# 1. PostgreSQL（主从）
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres bitnami/postgresql \
  -n rgs-db --create-namespace \
  --set auth.postgresPassword=$(openssl rand -base64 32) \
  --set primary.persistence.size=200Gi \
  --set readReplicas.replicaCount=2 \
  --set metrics.enabled=true

# 2. 初始化 5 个 DB
kubectl exec -n rgs-db postgres-primary-0 -- psql -U postgres -c "CREATE DATABASE player_db;"
kubectl exec -n rgs-db postgres-primary-0 -- psql -U postgres -c "CREATE DATABASE economy_db;"
# ... 其他 3 个

# 3. Redis
helm install redis bitnami/redis \
  -n rgs-db \
  --set auth.password=$(openssl rand -base64 32) \
  --set master.persistence.size=50Gi \
  --set replica.replicaCount=2
```

## 4.5 部署 RGS 服务（挂载脚手架）

```bash
# 1. 创建命名空间
kubectl create namespace rgs

# 2. 配置 secrets
kubectl create secret generic rgs-secrets -n rgs \
  --from-literal=database-url-player="postgres://..." \
  --from-literal=database-url-economy="postgres://..." \
  --from-literal=database-url-match="postgres://..." \
  --from-literal=database-url-social="postgres://..." \
  --from-literal=database-url-admin="postgres://..." \
  --from-literal=redis-url="redis://:password@redis-master:6379" \
  --from-literal=otel-endpoint="http://otel-collector:4317"

# 3. 用挂载脚手架生成新 App（如果新增）
cargo run -p rgs-scaffold -- new --name rgs-shop --type business

# 4. 部署核心服务（gateway + runtime + 5 业务服务）
helm install rgs-gateway ./charts/rgs-gateway -n rgs \
  -f values.staging.yaml
helm install rgs-runtime ./charts/rgs-runtime -n rgs \
  -f values.staging.yaml
# 5 个业务服务同上

# 5. 验证
kubectl get pods -n rgs
# 应该看到所有 Pod 状态 Running
kubectl get svc -n rgs
# 应该看到 ClusterIP 服务
```

## 4.6 验证端到端

```bash
# 1. 通过 Ingress 访问
curl https://rgs-staging.example.com/health
# 期望：{"status":"ok"}

# 2. 跑模拟客户端（10 CCU）
cargo run -p load-mock-client -- \
  --target gateway:7000 \
  --profile development \
  --instance-count 10 \
  --duration-sec 300

# 3. 跑 k6 HTTP 负载（100 RPS）
k6 run tests/load/api-gateway-baseline.js

# 4. 看 Grafana 仪表盘
# 打开 https://grafana-staging.example.com
# 应能看到 tick_rate, p99 latency, CCU 等指标
```

---

## 5. 生产环境

## 5.1 容量规划（参照 RGS-REQ-025）

| 阶段 | CCU | 单节点配置 | 节点数 | 备注 |
|---|---|---|---|---|
| **T0 上线** | 1 万 | 4 核 16 GB | gateway×2, runtime×2, biz×3 | 单可用区 |
| **T1 6 个月** | 5 万 | 8 核 32 GB | gateway×4, runtime×4, biz×5 | 单可用区 |
| **T2 12 个月** | 50 万 | 16 核 64 GB | gateway×8, runtime×8, biz×8 | 多可用区 |
| **T3 24 个月** | 1000 万 | 32 核 128 GB | gateway×32, runtime×32, biz×16 | 多区域 |

## 5.2 部署（多区域灾备）

本节的 RPO/RTO 均为**待演练验证的目标**，不是仅凭部署完成即可对外承诺的 SLO。区域 A 与 B 必须是不同 Kubernetes context、不同故障域；任何命令未显式指定 `--context` 均不得用于生产灾备操作。

### 5.2.1 部署前：context 与版本确认

```bash
export REGION_A_CONTEXT='rgs-prod-region-a'  # 仅示例；先由 SRE 从受控清单核对真实名称
export REGION_B_CONTEXT='rgs-prod-region-b'
export NAMESPACE='rgs-prod'

# 两个 context 必须存在、指向不同 cluster/server；输出保存到变更单。
kubectl config get-contexts -o name | grep -Fx "$REGION_A_CONTEXT"
kubectl config get-contexts -o name | grep -Fx "$REGION_B_CONTEXT"
kubectl --context "$REGION_A_CONTEXT" cluster-info
kubectl --context "$REGION_B_CONTEXT" cluster-info
kubectl --context "$REGION_A_CONTEXT" auth can-i create deployments -n "$NAMESPACE"
kubectl --context "$REGION_B_CONTEXT" auth can-i create deployments -n "$NAMESPACE"

# 镜像必须按 digest 部署；记录 Helm chart、values 的 Git 提交和所有镜像 digest。
helm template rgs-gateway ./charts/rgs-gateway -f values.prod.region-a.yaml > /tmp/rgs-region-a.rendered.yaml
helm template rgs-gateway ./charts/rgs-gateway -f values.prod.region-b.yaml > /tmp/rgs-region-b.rendered.yaml
```

### 5.2.2 双区域发布与复制校验

```bash
# 区域 A（唯一初始写主）
kubectl --context "$REGION_A_CONTEXT" create namespace "$NAMESPACE" --dry-run=client -o yaml \
  | kubectl --context "$REGION_A_CONTEXT" apply -f -
helm upgrade --install rgs-gateway ./charts/rgs-gateway \
  --kube-context "$REGION_A_CONTEXT" --namespace "$NAMESPACE" \
  -f values.prod.region-a.yaml --wait --atomic
kubectl --context "$REGION_A_CONTEXT" rollout status deployment/rgs-gateway -n "$NAMESPACE" --timeout=10m

# 区域 B（只读/待切换，不得取得写租约）
kubectl --context "$REGION_B_CONTEXT" create namespace "$NAMESPACE" --dry-run=client -o yaml \
  | kubectl --context "$REGION_B_CONTEXT" apply -f -
helm upgrade --install rgs-gateway ./charts/rgs-gateway \
  --kube-context "$REGION_B_CONTEXT" --namespace "$NAMESPACE" \
  -f values.prod.region-b.yaml --wait --atomic
kubectl --context "$REGION_B_CONTEXT" rollout status deployment/rgs-gateway -n "$NAMESPACE" --timeout=10m

# 在区域 A 主库确认区域 B 复制副本处于 streaming + sync；不满足时不得把 RPO 写为 0。
psql "$REGION_A_PRIMARY_DSN" -c "SELECT application_name,state,sync_state,write_lag,flush_lag,replay_lag FROM pg_stat_replication;"
# 在区域 B 确认只读副本仍在恢复且回放 LSN 可取；输出同样归档为变更证据。
psql "$REGION_B_STANDBY_DSN" -c "SELECT pg_is_in_recovery(), pg_last_wal_replay_lsn(), now() - pg_last_xact_replay_timestamp() AS replay_lag;"
# Redis 必须明确主从角色和复制健康；缓存/会话的复制方案不应被默认为跨区域强一致。
redis-cli -h "$REGION_B_REDIS_HOST" INFO replication
```

### 5.2.3 仲裁、fencing 与 DNS 切换

写主仲裁必须由预置、强一致的 DR 控制器持有单一 `write_epoch`。以下 `rgs-drctl` 代表该受控控制器；若未部署、无审计权限或无法取得其状态，**停止切换，不得手工 promote PostgreSQL 或仅改 DNS**。

```bash
# 正常状态：A 是唯一 writer，B 仅为 standby；把结果写入变更单。
rgs-drctl status --primary region-a --secondary region-b

# 区域故障切换的顺序：先切走写流量并 fence 旧写主，再以新的 epoch promote B。
rgs-drctl fence --region region-a --reason '<incident-or-drill-id>' --next-write-epoch '<monotonic-epoch>'
rgs-drctl wait-fence --region region-a --epoch '<monotonic-epoch>' --timeout 10m
rgs-drctl promote --region region-b --write-epoch '<monotonic-epoch>' --require-synchronous-replica
rgs-drctl status --primary region-b --expect-write-epoch '<monotonic-epoch>'

# 只有上一步确认 A 不能再取得写凭据、B 已持有唯一 epoch 后，才允许修改全局 DNS/GTM。
rgs-dnsctl set-weight --record gateway-prod.example.com --region region-a --weight 0
rgs-dnsctl set-weight --record gateway-prod.example.com --region region-b --weight 100
dig +short gateway-prod.example.com
```

`fence` 必须同时撤销旧区域写数据库凭据/写租约、将旧区域网关写流量置零，并留下 epoch 与操作者审计；DNS 只做流量路由，绝不能充当仲裁。DNS/GTM 记录 TTL、健康检查域名、切换前后权重和至少三个递归解析点的观察时间必须写入演练或事故证据。

### 5.2.4 演练验收与证据

每季度、数据库复制拓扑变更后、以及上线前至少执行一次受控区域切换演练。演练必须采用可追踪的合成写入，记录从停止旧主写入到新主可服务的实际 RTO，以及最后确认写入和新主可见写入之间的实际 RPO。以下证据缺一不可：

| 证据 | 最低内容 |
|---|---|
| 环境与版本 | 演练/事故 ID、两个 context/cluster UID、chart/values 提交、镜像 digest、开始/结束 UTC 时间 |
| 复制与仲裁 | 主从 LSN、`sync_state`、旧/新 `write_epoch`、旧主 fencing 成功证明、B promotion 记录 |
| 数据与流量 | 合成写入 ID 的前后查询结果、DNS/GTM 权重和多解析点观察、读写健康检查 |
| 结论 | 实测 RTO/RPO、目标是否通过、未通过项、负责人、复测日期与关联变更单 |

未达到目标、无法证明无 split-brain、或证据不完整时，RPO/RTO 状态为“未验证”，不得在 SLA、发布公告或事件总结中写成已达成。

## 5.3 100k CCU 负载验证（PH-8）

```bash
# 1. 协议层模拟客户端 100k 连接
cargo run -p load-mock-client -- \
  --target gateway-prod.region-a:7000 \
  --profile prod-100k \
  --instance-count 100000 \
  --duration-sec 3600

# 2. k6 HTTP 负载
k6 run --vus 1000 --duration 1h tests/load/api-gateway-100k.js

# 3. 收集指标
# 期望：tick p99 < 25ms（NFR-PE-002）
# 期望：100k CCU 稳定运行 1 小时（AC-005）
```

## 5.4 准入门禁

- ✅ 全部 NFR Lv.3/4 实测达标
- ✅ 100k CCU 稳定运行 ≥ 1 小时
- ✅ FT-001~014 故障注入 14/14 通过
- ✅ AC-001~019 验收标准 19/19 通过
- ✅ 缺陷密度 ≤ 1.0 件/KLOC
- ✅ 全部 TBD 决议完毕（AC-016）

---

## 6. 灰度发布与回滚

## 6.1 灰度发布（CDN + 服务两端）

```bash
# 1. CDN 资源灰度（参照 RGS-REQ-030-ADD1）
mc cp local/rgs-cdn/manifest-v2.json \
       local/rgs-cdn/manifest-v2-canary.json
# 在 manifest-v2-canary.json 中标记 channel=canary

# 2. 服务灰度（用 Argo Rollouts）
kubectl argo rollouts set image rgs-gateway \
  --container rgs-gateway=ghcr.io/org/rgs-gateway:v2.0.0-canary

# 3. 监控灰度指标
# - 错误率增量 ≤ 10%（NFR-MNT-002）
# - p99 latency 增量 ≤ 5%
# 5 分钟观察期

# 4. 全量发布
kubectl argo rollouts promote rgs-gateway

# 5. 或回滚
kubectl argo rollouts abort rgs-gateway
kubectl argo rollouts undo rgs-gateway --to-revision=<n>
```

## 6.2 DB 迁移（Expand-Contract）

```bash
# Phase 1: Expand（添加新列）
sqlx migrate run --database-url $PROD_DB_URL --source migrations/v2.0
# 部署新代码（读老列，向新列写）

# Phase 2: 验证所有消费者已迁移
psql $PROD_DB_URL -c "SELECT count(*) FROM migration_check WHERE target_version >= 2.0;"

# Phase 3: Contract（删除老列）
sqlx migrate run --database-url $PROD_DB_URL --source migrations/v2.0 --target-version 2.1
```

> **为什么 Expand-Contract**：NFR-AV-007 要求无停机滚动更新。直接 DROP COLUMN 会导致新旧版本同时运行时新版本 INSERT 失败。

## 6.3 紧急回滚

```bash
# 1. 立即回滚服务（30s 内）
kubectl argo rollouts abort rgs-gateway
kubectl argo rollouts undo rgs-gateway --to-revision=last-stable

# 2. 立即回滚 CDN（30s 内）
mc cp local/rgs-cdn/manifest-stable.json \
       local/rgs-cdn/manifest-current.json
# 边缘节点 30s 内全部流量切回 stable

# 3. 紧急 DB 回滚（仅可逆迁移）
sqlx migrate revert --database-url $PROD_DB_URL
```

---

## 7. 日常运维

## 7.1 监控关键指标

| 指标 | 类型 | 阈值 | 告警 |
|---|---|---|---|
| `rgs_tick_duration_p99_ms` | Gauge | < 25ms | > 35ms 持续 1min |
| `rgs_ccu_active` | Gauge | < 100k | > 80k 持续 5min |
| `rgs_request_error_rate` | Ratio | < 0.1% | > 0.5% 持续 1min |
| `rgs_db_connection_pool_wait` | Gauge | < 5s | > 10s |
| `rgs_outbox_lag_seconds` | Gauge | < 5s | > 30s |
| `rgs_redis_memory_usage` | Gauge | < 80% | > 90% |
| `rgs_disk_io_utilization` | Gauge | < 80% | > 90% |

## 7.2 每日检查清单

```bash
# 1. 所有 Pod 健康
kubectl get pods -n rgs-prod | grep -v Running

# 2. 数据库主从延迟
psql $PROD_DB_PRIMARY_URL -c "SELECT now() - pg_last_xact_replay_timestamp() AS replay_lag;"
# 期望：< 1s

# 3. Redis 内存
redis-cli -h $REDIS_HOST INFO memory | grep used_memory_human

# 4. 当日错误日志
kubectl logs -n rgs-prod -l app=rgs-gateway --since=24h | grep ERROR | wc -l

# 5. OLU 工时
# 登录 Grafana → RGS-OLU Dashboard → 看本周消耗
# 期望：< 2.0 SRE·d/wk（NFR-OP-010）
```

## 7.3 每周检查清单

```bash
# 1. 数据库 VACUUM（防止事务 ID 回卷）
psql $PROD_DB_URL -c "VACUUM (ANALYZE, VERBOSE);"

# 2. 备份验证
# 拉取昨日全量备份到测试环境，做一次恢复演练
# 期望：恢复时间 < 30min（NFR-AV-010）

# 3. 容量趋势
# 看 Grafana → 容量仪表盘
# 增长 20%/月时考虑扩容

# 4. OLU 复盘
# 每月一次，复盘 OLU 预算使用情况
```

## 7.4 配置变更

```bash
# 1. 配置文件变更（Helm values）
helm upgrade rgs-gateway ./charts/rgs-gateway -n rgs-prod -f values.prod.yaml

# 2. Secret 变更
kubectl edit secret rgs-secrets -n rgs-prod
# 或：
kubectl create secret generic rgs-secrets -n rgs-prod \
  --from-literal=... --dry-run=client -o yaml | kubectl apply -f -

# 3. ConfigMap 变更
kubectl create configmap rgs-config -n rgs-prod \
  --from-file=config.toml --dry-run=client -o yaml | kubectl apply -f -
```

---

## 8. 故障排查

## 8.1 常见故障速查

| 症状 | 可能原因 | 排查命令 |
|---|---|---|
| Pod 启动失败 | 配置错误 / 镜像拉取失败 | `kubectl describe pod <pod> -n rgs-prod` |
| 数据库连接耗尽 | 连接泄漏 / max_connections 过小 | `SELECT count(*) FROM pg_stat_activity;` |
| tick 抖动 | IO 阻塞 / GC 暂停 | `kubectl top pod` + `perf top` |
| Outbox 堆积 | 消费慢 / 消费者故障 | `SELECT count(*) FROM outbox WHERE published_at IS NULL;` |
| Redis 内存爆 | 数据未 TTL / 容量不足 | `redis-cli INFO memory` |
| 玩家登录失败 | session_epoch 错配 | `grep "epoch_mismatch" logs/` |

## 8.2 紧急命令

```bash
# 1. 查看实时日志
kubectl logs -n rgs-prod -f -l app=rgs-gateway --tail=100

# 2. 进入 Pod 调试
kubectl exec -it <pod> -n rgs-prod -- /bin/sh

# 3. 抓取 trace
curl http://<gateway>:8080/api/v1/admin/trace/<trace_id>

# 4. 强制重启 Pod
kubectl delete pod <pod> -n rgs-prod

# 5. 暂停自动扩缩
kubectl scale deployment/rgs-gateway -n rgs-prod --replicas=0
# ⚠️ 紧急用，会中断服务
```

## 8.3 性能调优

```bash
# 1. PostgreSQL 慢查询
psql $PROD_DB_URL -c "SELECT pid, query, state, wait_event FROM pg_stat_activity WHERE state='active';"
# 看是否有锁等待

# 2. CPU profile
cargo build --release --features profiling
./rgs-gateway --cpu-profile=/tmp/profile.out
# 用 pprof 工具分析

# 3. 火焰图
# 用 cargo-flamegraph
cargo flamegraph --bin rgs-gateway
# 浏览器打开 flamegraph.svg
```

---

## 9. 应急恢复

## 9.1 灾难场景与恢复

下表是**经 §5.2.4 演练验证后才可声明的目标**。在最近一次同范围演练的证据中没有实测值时，状态一律为“未验证”；特别是 `RPO=0` 仅在同步复制、唯一写主仲裁和 fencing 均被证明有效时成立。

| 场景 | 验证后目标 RTO | 验证后目标 RPO | 恢复流程与证据 |
|---|---|---|---|
| 单 Pod 故障 | < 30s | 0 | K8s 自动重启；记录探针失败、替换 Pod 就绪和业务健康检查时间 |
| 单节点故障 | < 5min | 0 | K8s 调度 Pod 到其他节点；证明无单节点持久状态 |
| DB 主节点故障 | < 30min | 0（仅同步副本+fencing 验证后） | 依 §5.2.3 由仲裁控制器切换；归档 LSN、epoch 与合成写入结果 |
| 整个可用区故障 | < 30min | 0（同上） | 先 fence 旧写主，再经 DNS/GTM 切到另一可用区；保留多解析点证据 |
| 整个区域故障 | < 2h | 0（同上；否则报告实测丢失量） | 依 §5.2.3 跨区域切换；证明无 split-brain、复制状态和 DNS 切流 |
| 误删数据 | 依备份实测 | 依最近恢复点实测 | 从备份恢复；演练必须记录备份时间、恢复点和数据校验 |
| 配置错误 | < 5min | 0 | Helm 回滚到上一版本；记录部署版本、回滚版本和健康检查 |

> 每次演练或真实事故都必须更新受控证据，而不是把这里的目标值当作自动成立的结果。

## 9.2 应急联系

| 角色 | 联系方式 | 响应时间 |
|---|---|---|
| 架构师 | （内部） | 5min |
| SRE | （内部） | 5min |
| 项目负责人 | （内部） | 15min |

---

## 10. 附录

## 10.1 关键文件位置速查

| 文件 | 路径 |
|---|---|
| 父 Cargo manifest | `Cargo.toml` |
| 工作空间 member | `crates/*/Cargo.toml`, `services/*/Cargo.toml` |
| 数据库迁移 | `crates/rgs-<domain>/migrations/YYYYMMDDHHMMSS_description.sql`（仅 DB owner 执行） |
| Helm charts | `charts/<service>/` |
| K8s manifests | `k8s/` |
| 监控配置 | `k8s/monitoring/` |
| 仪表盘 | `k8s/monitoring/dashboards/*.json` |
| 测试设计书 | `docs/*/RGS-TST-*.md`（33 份） |
| 需求定义书 | `docs/*/RGS-REQ-*.md`（33 份） |
| 基本设计书 | `docs/*/RGS-BAS-*.md`（27 份） |
| 详细设计书 | `docs/*/RGS-DTL-*.md`（26 份） |
| ADR | `docs/08-架构决策记录/RGS-ADR-*.md`（14 份） |
| 部署说明（本文件） | `docs/09-部署运维/RGS-OPS-001_*.md` |

## 10.2 端口速查

| 端口 | 服务 |
|---|---|
| 5432 | PostgreSQL |
| 6379 | Redis |
| 7000/UDP | QUIC 网关 |
| 7001/TCP | QUIC fallback |
| 8080 | HTTP API 网关 |
| 9000-9105 | gRPC 服务 |
| 4222 | NATS |
| 4317/4318 | OTel |
| 9090 | Prometheus |
| 3000 | Grafana |

## 10.3 命令速查

| 任务 | 命令 |
|---|---|
| 跑全部测试 | `cargo test --workspace` |
| 看覆盖率 | `cargo llvm-cov --workspace --html` |
| 跑 1k CCU 负载 | `cargo run -p load-mock-client -- --instance-count 1000` |
| 看实时日志 | `kubectl logs -f -l app=rgs-gateway` |
| 数据库迁移 | `sqlx migrate run --database-url <url> --source migrations/<db>` |
| 灰度发布 | `kubectl argo rollouts set image rgs-gateway --container=...` |
| 紧急回滚 | `kubectl argo rollouts undo rgs-gateway --to-revision=<n>` |
| 看 OLU | 登录 Grafana → RGS-OLU Dashboard |
| 跑 ADR 治理 CI | 提交 ADR → git push → CI 自动验证 |

## 10.4 关联文档

| 文档 | 关系 |
|---|---|
| RGS-REQ-001 | 主需求 |
| RGS-REQ-027 | 集群自动化部署 |
| RGS-BAS-001 | 主基本设计 |
| RGS-BAS-024 | 部署基本设计 |
| RGS-DTL-024 | 部署详细设计 |
| RGS-REQ-015 | 测试基础设施 |
| RGS-REQ-013 | 体系治理（OLU / 治理 CI） |
| RGS-ADR-0008 | 中间件导入判定 |
| RGS-ADR-0025 | OLU 预算 |
| RGS-OPS-001 | 本文档 |
| 24 份 RGS-TST-UT/IT/ST | 测试设计书 |

---

> 本文档是 RustGameServer 项目的**唯一权威部署指南**。任何部署相关操作前请先查阅本文档相应章节。
>
> 维护责任人：SRE 团队
>
> 最后更新：2026-08-19
