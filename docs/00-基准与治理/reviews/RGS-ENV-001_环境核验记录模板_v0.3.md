# 环境核验记录模板

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-ENV-001 |
| 版本 | 0.3（v0.8 撤销所有者背书，Ulysses 12 类全签 per DEC-008）|
| 依据 | RGS-HANDOFF-001 §5 Step 2 + §4 G-CODE-06 + RGS-PLAN-001 v0.8 §3.4.3 签字顺序（DBA → SRE → 5 域 Lead → 架构师 → Economy 域 Lead（Q-003 二次确认） → Platform → QA → PM）+ RGS-QA-001 v0.12 DEC-005（5 域独立 Lead） |
| 目的 | 在 53 開発環境構築 启动前，对工具链 / 数据库 / 容器编排 / CI 4 层环境做"无业务实现"的核验 |
| 输出 | 已签名的核验记录（不签 → 53 仍为 NO-GO）|
| 频次 | 1 次（环境首次就绪时）；后续重大变更时复跑 |
| 签字栏位 | **12 类签字**（v0.3 全部 Ulysses 实际签 per DEC-008 一人公司 12 角色兼任）|

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 初版。覆盖工具链 / PostgreSQL / K3s / CI / 跨工具集成 5 层核验；5 类签字栏（Platform / DBA / SRE / 架构师 / PM）。 |
| 0.2 | 2026-08-21 | 架构师 | **§6 签字栏 5 类 → 12 类扩**（per RGS-PLAN-001 v0.8 §3.4.3 + RGS-QA-001 v0.12 DEC-005）：新增 5 域独立 Lead 签字（player / economy / match / social / admin）+ Economy 域 Lead Q-003 二次确认 + QA Lead = 12 类；签字顺序不可跳签（DBA → SRE → 5 域 Lead → 架构师 → Economy 域 Lead 二次 → Platform → QA → PM）；新增"5 域 Lead 不享有代表同意机制"说明。**未变更**：§1-§5 核验项 + §7 异常处理 + §8 完成声明。 |
| 0.3 | 2026-08-21 | 架构师（Ulysses）| **DEC-008 落地**（一人公司治理基线 per RGS-QA-001 v0.12 §9.5.7）：撤销 v0.2 所有者背书机制 + Ulysses 12 类全签（一人公司 12 角色兼任 = 真实人真实职责）。**接受代价**：Q-003 跨域事务"1 人自审自批"已知风险，流程化补偿（CI 强约束 + 自动化测试 ≥ 80% + 自我 PR review + OTel 链路）。**未变更**：§1-§5 核验项 + §7 异常处理 + §8 完成声明 + 12 类签字顺序（DBA→SRE→5 域 Lead→架构师→Q-003 二次→Platform→QA→PM，**Ulysses 1 人按顺序走完 12 步**）。**注意**：§1-§5 12 类环境核验仍需 Ulysses 实际跑过才算 ✅ 通（签字不构成证据，per RGS-EXEC-001 v0.3 §3.4）。 |

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

## §2 PostgreSQL 18.6 核验

### §2.1 客户端版本

```bash
$ psql --version
# 期望输出: psql (PostgreSQL) 18.6
```

- [ ] **2.1.1** psql = 18.6
- [ ] **2.1.2** libpq 与 psql 版本一致

### §2.2 服务器连接

```bash
$ psql -h <host> -p 5432 -U <user> -c "SELECT version();"
# 期望输出: PostgreSQL 18.6 ...
```

- [ ] **2.2.1** 服务器版本 = 18.6
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

## §6 签字栏（v0.2：12 类，不可跳签 + 所有者背书机制）

> **签字顺序（per RGS-PLAN-001 v0.8 §3.4.3，不可跳签）**：
> **DBA → SRE → 5 域 Lead（player → economy → match → social → admin）→ 架构师 → Economy 域 Lead（Q-003 二次确认）→ Platform → QA → PM**
>
> **v0.2 所有者背书机制**（per RGS-PLAN-001 v0.8 §3.4.4 + RGS-EXEC-001 v0.3 §8，user decision 2026-08-21 折中方案 C）：
> - **2 项 Ulysses 实际签**（架构师 / PM 角色）
> - **10 项所有者背书 + 待具名责任人**（DBA / SRE / 5 域 Lead / Platform / QA / Q-003 二次 具名责任人位）
> - **风险声明**：所有者背书**不替代具名责任人签字**；NO-GO 仍由 7 G-CODE 全部 Closed 解除
> - **解除路径**：具名责任人到位后升 v0.3，移除"所有者背书"占位 → 全部具名签字补全

| # | 角色 | 责任人 | 签字 | 日期 | 核验范围 | 结论 |
|---|---|---|---|---|---|---|
| 1 | **DBA Lead** | **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §2 PostgreSQL 18.6（5 DB 划分 + 双向迁移演练）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 2 | **SRE Lead** | **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §3 K3s / Kubernetes（节点就绪 + CoreDNS + Traefik + Helm + 镜像仓库）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 3 | **Player 域 Lead**（独立）| **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §1 工具链 + §5 跨工具集成（player 域相关）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 4 | **Economy 域 Lead**（独立）| **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §1 + §5（economy 域相关）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 5 | **Match 域 Lead**（独立）| **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §1 + §5（match 域相关）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 6 | **Social 域 Lead**（独立）| **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §1 + §5（social 域相关）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 7 | **Admin 域 Lead**（独立，不兼任 SRE）| **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §1 + §5（admin / COC 域相关）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 8 | **架构师** | **Ulysses（架构师）** | **Ulysses（架构师）** | **2026-08-21** | §5 跨工具集成（sqlx 编译期 + tonic gRPC + OTel + distroless）| ✅ 通过 |
| 9 | **Economy 域 Lead（Q-003 二次确认）** | **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | Q-003 跨 DB Saga 在新环境可跑 | ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 10 | **Platform Engineer** | **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §1 工具链 + §4 锁定依赖 CI（Rust 1.98 + cargo fmt/clippy/deny/audit/llvm-cov）| ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 11 | **QA Lead** | **Ulysses 实际签**（一人公司 12 角色兼任）| Ulysses（一人公司 12 角色兼任） | 2026-08-21 | §4.3 测试覆盖 + §4 测试可重入 | ☐ 通过 / ☐ 有条件 / ☐ 不通过 |
| 12 | **PM（总签字）** | **Ulysses（PM）** | **Ulysses（PM）** | **2026-08-21** | 12 类签字齐全 + 30 天内启动 53 承诺（含 §8 所有者背书）| ✅ 通过（带 10 项所有者背书 + 待具名责任人补全）|

> **联合评审主持人**（per RGS-REV-003）：架构师（兼）作为评审主持人，主持环境核验联合评审；不单独占签字栏（架构师已在 #8 签字）。
> **5 域 Lead 不享有"代表同意"机制**（per DEC-005 不兼任原则延伸）：任一域 Lead 异议即该域核验不通过。
> **所有者背书风险**（per RGS-EXEC-001 v0.3 §8.3）：#1-#7、#9-#11 标"所有者背书"的 10 项**不构成 G-CODE Closed 的证据**；具名责任人到位后升 v0.3 移除背书占位。

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
