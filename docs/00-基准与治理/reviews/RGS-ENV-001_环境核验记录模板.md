# 环境核验记录模板

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-ENV-001 |
| 版本 | 0.1（草稿）|
| 依据 | RGS-HANDOFF-001 §5 Step 2 + §4 G-CODE-06 |
| 目的 | 在 53 開発環境構築 启动前，对工具链 / 数据库 / 容器编排 / CI 4 层环境做"无业务实现"的核验 |
| 输出 | 已签名的核验记录（不签 → 53 仍为 NO-GO）|
| 频次 | 1 次（环境首次就绪时）；后续重大变更时复跑 |

---

## §0 核验元数据

| 项 | 值 |
|---|---|
| 核验日期 | YYYY-MM-DD |
| 操作人 | _______（具名人类责任人）|
| 环境名称 | dev / staging / prod-preview |
| 节点清单 | node-1: <IP>, node-2: <IP>, node-3: <IP> |
| 操作系统 | <uname -a 输出> |
| 内核版本 | <uname -r 输出> |

---

## §1 工具链核验（Rust 1.98 stable）

### §1.1 rustc / cargo 版本

```bash
$ rustc --version
# 期望输出: rustc 1.98.0 (<commit-hash> <date>)

$ cargo --version
# 期望输出: cargo 1.98.0 (<commit-hash> <date>)
```

- [ ] **1.1.1** rustc = 1.98.0（**不允许** beta / nightly / 1.97 或更早）
- [ ] **1.1.2** cargo = 1.98.0
- [ ] **1.1.3** MSRV 锁定：仓库根 `rust-toolchain.toml` 写明 `channel = "1.98"`

### §1.2 关键工具链组件

```bash
$ rustup component list --installed
# 期望包含: rustfmt, clippy, rust-std, rust-src (用于 IDE/工具链)
```

- [ ] **1.2.1** clippy 启用：`cargo clippy --version` 存在
- [ ] **1.2.2** rustfmt 启用：`cargo fmt --version` 存在
- [ ] **1.2.3** rust-src 安装（供 rust-analyzer）

### §1.3 工作空间依赖工具

```bash
$ cargo install --list | grep -E 'cargo-deny|cargo-audit|cargo-llvm-cov|sqlx-cli|tonic-build'
```

- [ ] **1.3.1** `cargo-deny` 安装（依赖审计）
- [ ] **1.3.2** `cargo-audit` 安装（CVE 扫描）
- [ ] **1.3.3** `cargo-llvm-cov` 安装（覆盖率）
- [ ] **1.3.4** `sqlx-cli` 安装（migration 工具）
- [ ] **1.3.5** `tonic` 相关 protoc 工具可用

---

## §2 PostgreSQL 18.4 核验

### §2.1 客户端版本

```bash
$ psql --version
# 期望输出: psql (PostgreSQL) 18.4
```

- [ ] **2.1.1** psql = 18.4
- [ ] **2.1.2** libpq 与 psql 版本一致

### §2.2 服务器连接

```bash
$ psql -h <host> -p 5432 -U <user> -c "SELECT version();"
# 期望输出: PostgreSQL 18.4 ...
```

- [ ] **2.2.1** 服务器版本 = 18.4
- [ ] **2.2.2** SSL/TLS 连接（per ADR-0052 容错哲学）
- [ ] **2.2.3** pg_hba.conf 配置核验（md5/scram-sha-256）

### §2.3 5 DB 划分验证（per ARC-008）

```bash
$ psql -h <host> -U <user> -c "\l" | grep -E 'player_db|economy_db|match_db|social_db|admin_db'
```

- [ ] **2.3.1** `player_db` 存在
- [ ] **2.3.2** `economy_db` 存在
- [ ] **2.3.3** `match_db` 存在
- [ ] **2.3.4** `social_db` 存在
- [ ] **2.3.5** `admin_db` 存在

### §2.4 sqlx 编译期校验

```bash
$ DATABASE_URL=postgres://... cargo check --features sqlx/runtime-tokio-rustls
# 期望: 编译通过，无 SQL 错误
```

- [ ] **2.4.1** `cargo check` 成功（编译期 SQL 校验）
- [ ] **2.4.2** `.sqlx/` 目录生成（offline mode 元数据）
- [ ] **2.4.3** 至少 1 张表（示例 players）的 prepared statement 编译通过

### §2.5 Migration 演练（per G-CODE-06）

```bash
$ sqlx migrate run --source crates/rgs-player/migrations
# 期望: 所有 migration 应用成功
$ sqlx migrate revert --source crates/rgs-player/migrations
# 期望: 回滚成功
$ sqlx migrate run --source crates/rgs-player/migrations
# 期望: 再次应用成功（双向演练）
```

- [ ] **2.5.1** forward migration 成功
- [ ] **2.5.2** reverse migration 成功
- [ ] **2.5.3** 重新 forward migration 成功
- [ ] **2.5.4** migration 历史表 (`_sqlx_migrations`) 状态正确

---

## §3 K3s / Kubernetes 核验

### §3.1 kubectl 版本

```bash
$ kubectl version --client
# 期望输出: Client Version: v1.30+ (K3s 默认配套版本)
$ kubectl version --short
# 期望输出: Server Version: v1.30+
```

- [ ] **3.1.1** kubectl client ≥ v1.30
- [ ] **3.1.2** K3s server ≥ v1.30
- [ ] **3.1.3** k3s server 与 client 版本一致（±1 minor）

### §3.2 节点就绪

```bash
$ kubectl get nodes
NAME           STATUS   ROLES                  AGE   VERSION
node-1         Ready    control-plane,master   30d   v1.30.x+k3s1
node-2         Ready    <none>                 30d   v1.30.x+k3s1
node-3         Ready    <none>                 30d   v1.30.x+k3s1

$ kubectl get nodes -o json | jq '.items[].status.conditions[] | select(.type=="Ready")'
```

- [ ] **3.2.1** 至少 3 节点（per DEC-001 all-reachable 需要 quorum）
- [ ] **3.2.2** 所有节点 Ready
- [ ] **3.2.3** 节点间网络互通（ping/curl 内网 IP）

### §3.3 核心组件

```bash
$ kubectl -n kube-system get pods | grep -E 'coredns|traefik|local-path|metrics-server'
```

- [ ] **3.3.1** CoreDNS 运行
- [ ] **3.3.2** Traefik Ingress 运行
- [ ] **3.3.3** local-path StorageClass 可用
- [ ] **3.3.4** metrics-server 运行

### §3.4 Helm 能力

```bash
$ helm version
# 期望输出: version.BuildInfo{Version:"v3.x", ...}

$ helm list -A
# 期望输出: 空或仅系统 chart
```

- [ ] **3.4.1** helm ≥ v3.10
- [ ] **3.4.2** ArgoCD / Flux 准备就绪（如果用 GitOps）

### §3.5 镜像仓库

```bash
$ crictl images | grep -E 'rgs-|distroless'
# 或
$ nerdctl images | grep -E 'rgs-|distroless'
```

- [ ] **3.5.1** 内网镜像仓库可达
- [ ] **3.5.2** distroless/cc-debian12 基础镜像可拉取
- [ ] **3.5.3** RGS-* 业务镜像 registry 配置就绪

---

## §4 锁定依赖 CI 核验

### §4.1 仓库级

```bash
$ git ls-files Cargo.lock | head
# 期望: 仓库根存在 Cargo.lock（per RGS-IMPL-001 §3）

$ cargo --locked build
# 期望: 编译成功（CI 必须用 --locked）
```

- [ ] **4.1.1** `Cargo.lock` 入仓（**应用 crate** 必入，**library crate** 可不入）
- [ ] **4.1.2** `cargo --locked build` 成功

### §4.2 静态检查

```bash
$ cargo fmt --all -- --check
# 期望: 无 diff

$ cargo clippy --all-targets --all-features -- -D warnings
# 期望: 无 warning

$ cargo deny check
# 期望: licenses/ban/sources/advisories 全部通过
```

- [ ] **4.2.1** `cargo fmt --check` 通过
- [ ] **4.2.2** `cargo clippy -D warnings` 通过
- [ ] **4.2.3** `cargo deny check` 通过

### §4.3 单元测试与覆盖率

```bash
$ cargo test --workspace --locked
# 期望: 编译 + 测试全部通过（即使无业务代码，至少 workspace 元数据可编译）

$ cargo llvm-cov --workspace --html
# 期望: 覆盖率报告生成
```

- [ ] **4.3.1** `cargo test --locked` 通过
- [ ] **4.3.2** coverage 报告生成（即使 0% 也允许 — 这只是环境核验）
- [ ] **4.3.3** 测试可在 5 分钟内完成（per RGS-TST §10 V-model 配对）

### §4.4 安全扫描

```bash
$ cargo audit
# 期望: 无未修复的 RUSTSEC 警告
```

- [ ] **4.4.1** `cargo audit` 通过
- [ ] **4.4.2** 已知 CVE 全部在白名单 / 已修复

---

## §5 跨工具集成核验

### §5.1 Rust 1.98 + sqlx 编译期

```bash
$ DATABASE_URL=postgres://localhost/test_db cargo check --features sqlx/runtime-tokio-rustls
```

- [ ] **5.1.1** sqlx 编译期类型检查通过
- [ ] **5.1.2** 与 1.98 stable 兼容

### §5.2 tonic gRPC + tracing

```bash
$ cargo check --features rgs-coc/grpc,rgs-coc/observability
```

- [ ] **5.2.1** tonic 编译通过
- [ ] **5.2.2** tracing-opentelemetry 链接通过
- [ ] **5.2.3** OTel collector 集成（如果本地有）

### §5.3 distroless 容器构建

```bash
$ docker build -f Dockerfile.rgs -t rgs:test .
# 或
$ buildctl build ... -f Dockerfile.rgs
```

- [ ] **5.3.1** 镜像构建成功
- [ ] **5.3.2** 镜像基于 `gcr.io/distroless/cc-debian12:nonroot`
- [ ] **5.3.3** 镜像大小 < 100MB（粗略目标）

---

## §6 签字栏

| 角色 | 责任人 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 工具链（§1, §4, §5）| Platform Engineer | | | |
| PostgreSQL 18.4（§2）| DBA | | | |
| K3s / Kubernetes（§3）| SRE Lead | | | |
| 跨工具集成（§5）| 架构师 | | | |
| **总签字** | **PM** | | | |

---

## §7 异常处理

### §7.1 失败重试

- 任何 ❌ 项需在本表下方附"修复记录"，注明：
  - 失败现象
  - 根因分析
  - 修复方案
  - 重新核验日期
- 修复记录由 Platform Engineer 签

### §7.2 升级路径

- 3 次修复仍未通过 → 升级为 NO-GO
- 触发 handoff §1 评估 53 启动推迟
- 记录在 RGS-QA-001 风险登记表

---

## §8 核验完成声明

> **本核验记录是 53 開発環境構築 的前置条件之一（per handoff §4 G-CODE-06）。** 未签字的核验记录等同 NO-GO。
>
> 核验通过后，须在 30 天内启动 53；超过 30 天须重新核验。
