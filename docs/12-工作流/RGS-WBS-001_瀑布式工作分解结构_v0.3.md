# 瀑布式 5 层工作分解结构（Waterfall WBS, v0.3）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001 |
| 版本 | 0.3（瀑布式 9 阶段 + worktree + agent 并行）|
| 依据 | RGS-PLAN-001 v0.8 §3.1 PH 表（14-18 周窗口）+ RGS-QA-001 v0.10 DEC-005（5 域独立 Lead）+ DEC-006（路径 B 14-18 周）+ RGS-TS-001 v0.6 §6.2 OLU 双轨制 + RGS-IMPL-001 工程约定 + RGS-SPEC-000 详细设计规格化总表 + RGS-REV-004 5 域 DTL 字段级 Review Checklist |
| 范围 | first slice 14-18 周 / 5 域 + foundation + cluster-ops + shared-platform / ARC-018/021/042/051 |
| 配套 | RGS-TS-001 v0.6 §6.2 OLU 双轨（人·天 + token）；RGS-ENV-CALIB-001 OLU 校准模板；RGS-PLAN-001 v0.8 §3.1 PH 阶段表；RGS-ENV-001 v0.3 环境核验 12 类签字 |
| 保密级别 | 内部限定（Internal Use Only）|

> **核心约束**：
> - **L1 阶段**：8 PH（per RGS-PLAN-001 v0.8 §3.1 14-18 周重排）
> - **L2 域**：5 域 + foundation + cluster-ops + shared-platform = 8 域簇
> - **L3 任务簇**：每域每 PH 8 个任务簇（API Spec / 业务逻辑 / DB migration / UT / IT / ST / Helm chart / observability）
> - **L4 任务**：每任务簇 4 个具体任务
> - **L5 工作包**：最小可分配单元，**≤ 2 人·天 或 ≤ 500K tokens**
> - **L4+ 强制项**：每任务有 owner / 人·天估算 / token 估算 / 前置依赖 / 验收项 / 回滚路径

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 初版。L1-L5 框架 + 关键示例（player / economy PH-1 完整 32 L4 任务 × 2 域）。 |
| 0.2 | 2026-08-21 | 架构师 + 5 域 Lead（待补）|| 0.3 | 2026-08-21 | 架构师（Ulysses）| **v0.3 瀑布式升级**（per user decision 2026-08-21 + RGS-WF-001 v0.6 150 工程 100% 覆盖 + DEC-008 一人公司治理基线）：① L1 阶段从 8 PH 重构为**瀑布 9 阶段**（per WF-001 §1 01-21 需求 / 22-41 设计 / 42-52 详细设计 / 53-58 实施 / 59-65 单元测试 / 66-89 集成+系统测试 / 90-95 验收 / 96-108 部署 / 109-150 运维+收尾）；② L4 任务加 **worktree 分支命名**（`wbs/<L1>-<L2>-<L3>-<L4>` 格式），每个 L4 任务可单独 worktree 分支执行；③ L4 任务加 **进度追踪字段**（status / start / end / progress_note）支持 agent 并行配合；④ L4 任务加 **agent 最小可拆分** 维度（每个 L4 = 1 个 agent 独立完成，≤ 2 人·天 或 ≤ 500K tokens）；⑤ 新增 §9 跨任务依赖图（DAG）；⑥ 新增 §10 WBS 状态机（agent 报告进度 → 实际签 → 关闭 L4）。**未变更**：2,048 L4 占位（仍由 5 域 Lead 在 PH-0.5 前补全）。 |
 **§4.3 完整 L4 任务占位框架**（per user decision 2026-08-21 补全 5 域 Lead L4 任务清单）：通过 `scripts/build_wbs_v02.py` 生成 **2,048 L4 任务占位**（5 域 + 3 配套 × 8 PH × 32 L4），独立文档 `RGS-WBS-001_L4任务占位清单_v0.1.md`（2071 行 / 297 KB）；5 域 Lead + foundation + cluster-ops + shared-platform 各自补全 256 L4 × 8 域 = 2,048 L4，PH-0.5 前完成；新增配套 `scripts/build_wbs_v02.py`（生成脚本，可重跑保持结构）。

---

## §1 L1 瀑布 9 阶段（per RGS-WF-001 v0.6 §1）

| L1 | 阶段 | 瀑布映射（WF-001 §1）| 规划窗口（v0.6 14-18 周）| 阶段出口（agent 移交依据） |
|---|---|---|---|
| WF-0 | 需求 + 设计冻结 | 01-21 需求 / 22-41 设计 | 第 1-3 周 | WF-001 §1 RGS-REQ-001~035 + RGS-BAS-001 + 5 域 DTL v0.2 + RD Review + Baseline |
| WF-0.5 | 详细设计冻结 | 42-52 详细设计 | 第 3-4 周 | DD Review（per WF-001 §1）+ 全部 5 域 DTL v0.2 签字 |
| WF-1 | 实施 | 53-58 实施 | 第 4-6 周 | Cargo workspace + testkit + CI 全绿（per RGS-EXEC-001 v0.3 §3.4 G-CODE-06 实测）|
| WF-2 | 单元测试 | 59-65 单测 | 第 6-8 周 | UT 全部通过 + 覆盖率 ≥ 80%（per RGS-TST-101）|
| WF-3 | 集成测试 | 66-75 集成 | 第 8-10 周 | IT 全部通过（per RGS-TST-102）+ Saga 6 场景 |
| WF-4 | 系统测试 | 76-89 系统 + NFR | 第 10-12 周 | ST 全通 + chaos + NFR（per RGS-TST-103/104）|
| WF-5 | 验收 | 90-95 UAT | 第 12-13 周 | UAT 全部通过 + 受入判定（per RGS-TST-105）|
| WF-6 | 部署 | 96-108 部署 | 第 13-15 周 | 5 域 + cluster-ops + shared-platform 部署到 production（per docs/deploy/05-deploy-sop.md）|
| WF-7 | 运维 + 收尾 | 109-150 运维/PM/收尾 | 第 15-18 周 | 运维移交 + Closure 报告 + Retrospective（per RGS-PM-008）|

---



## §2 附录 A: WF-1 实施阶段 L4 任务清单（per 用户决策 2026-08-21 "实施阶段细分要足够细"）

> **拆分标准**（per RGS-WBS-001 v0.3 §6.2 + 用户决策 2026-08-21）：每个 L4 任务 = 1 人/agent 最小可拆分单位，≤ 2 人·天 或 ≤ 500K tokens / ≤ 3 前置 / ≤ 5 验收 / ≤ 3 步回滚。

### §2A.1 拆分原则

- **owner 字段**：Ulysses（一人公司 12 角色兼任 per DEC-008）
- **worktree 分支**：`wbs/WF-1-<工程>-<L4#>`（如 `wbs/WF-1-53.1`）
- **进度字段**：`⬜ 未启动 / 🟡 进行中 / ✅ 完成 / ❌ 阻塞` + progress_note
- **每个 L4 任务 = 1 个独立 worktree 分支**，可单独执行

### §2A.2 6 工程 × L4 任务分解


#### §2A.2.53 工程 53 — 开发环境构建（15 L4 任务 / 17.5 人·天 / 3950K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree 分支 | 进度 |
|---|---|---|---:|---:|---|---|---|---|---|
| WF-1-53.1 | Rust 1.98 工具链安装（rustup + cargo + clippy + rustfmt） | Platform | 1.0 | 200K | 工程基础 | rustup-init 安装 / cargo --version 输出 1.98.0 / clippy --version 验证 | 卸 / 载 等 9 步 | `wbs/WF-1-53.1` | ⬜ 未启动 |
| WF-1-53.2 | Cargo workspace 多 crate 结构（5 域 + cluster-ops + shared-platform 共 7 crate） | Platform | 1.5 | 350K | WF-1-53.1 | Rust 工具链 / 5 域 DTL 边界确定 / workspace cargo build --workspace 通过 | g / i 等 10 步 | `wbs/WF-1-53.2` | ⬜ 未启动 |
| WF-1-53.3 | rgs-testkit 测试套件骨架（mock + helper + fixture） | Platform | 2.0 | 500K | WF-1-53.2 | Rust 工具链 / workspace | g / i 等 10 步 | `wbs/WF-1-53.3` | ⬜ 未启动 |
| WF-1-53.4 | CI workflow 1: rust-ci（fmt + clippy + test + llvm-cov） | Platform | 1.0 | 250K | WF-1-53.3 | Rust 工具链 / Cargo workspace | 关 / 闭 等 19 步 | `wbs/WF-1-53.4` | ⬜ 未启动 |
| WF-1-53.5 | CI workflow 2: docs-ci（lychee 链接检查 + markdownlint） | Platform | 0.5 | 100K | WF-1-53.4 | Rust 工具链 / workflow 1 | 关 / 闭 等 10 步 | `wbs/WF-1-53.5` | ⬜ 未启动 |
| WF-1-53.6 | CI workflow 3: verify-docs-ci（3 脚本必跑） | Platform | 0.5 | 100K | WF-1-53.5 | workflow 2 / testkit 包含脚本 | 关 / 闭 等 10 步 | `wbs/WF-1-53.6` | ⬜ 未启动 |
| WF-1-53.7 | CI workflow 4: docker-build（占位 trigger 注释） | Platform | 1.0 | 200K | WF-1-53.6 | Rust 工具链 / Cargo workspace | 关 / 闭 等 10 步 | `wbs/WF-1-53.7` | ⬜ 未启动 |
| WF-1-53.8 | 本地 docker-compose dev 环境（5 DB + 5 域服务） | Platform | 2.0 | 450K | WF-1-53.7 | Rust 工具链 / workspace / workflow 1 | d / o 等 22 步 | `wbs/WF-1-53.8` | ⬜ 未启动 |
| WF-1-53.9 | 本地 k3s 集群（或 kind）单节点 dev 集群 | Platform | 1.5 | 350K | WF-1-53.8 | Rust 工具链 / docker-compose | k / 3 等 13 步 | `wbs/WF-1-53.9` | ⬜ 未启动 |
| WF-1-53.10 | 5 独立 PG 18.4 DB 容器（player_db / economy_db / match_db / social_db / admin_db + cluster_ops_db） | Platform | 1.5 | 350K | WF-1-53.9 | docker-compose / PG 18.4 镜像 | d / o 等 22 步 | `wbs/WF-1-53.10` | ⬜ 未启动 |
| WF-1-53.11 | QUIC 证书生成脚本（rustls + rcgen） | Platform | 1.0 | 200K | WF-1-53.10 | Rust 工具链 / 证书生成库 | r / m 等 7 步 | `wbs/WF-1-53.11` | ⬜ 未启动 |
| WF-1-53.12 | OTel Collector 容器 + Prometheus + Grafana 集成 | Platform | 2.0 | 500K | WF-1-53.11 | docker-compose / OTel 镜像 | d / o 等 22 步 | `wbs/WF-1-53.12` | ⬜ 未启动 |
| WF-1-53.13 | distroless base image Dockerfile（dev/staging/prod 三个 tag） | Platform | 1.0 | 200K | WF-1-53.12 | Rust 工具链 / docker buildx | 删 / 除 等 8 步 | `wbs/WF-1-53.13` | ⬜ 未启动 |
| WF-1-53.14 | cargo-deny + cargo-audit 配置（许可 + 漏洞检查） | Platform | 0.5 | 100K | WF-1-53.13 | Rust 工具链 / Cargo workspace | 删 / 除 等 19 步 | `wbs/WF-1-53.14` | ⬜ 未启动 |
| WF-1-53.15 | devcontainer.json（VS Code Remote Container） | Platform | 0.5 | 100K | WF-1-53.14 | Rust 工具链 / docker-compose | 删 / 除 等 17 步 | `wbs/WF-1-53.15` | ⬜ 未启动 |


#### §2A.2.54 工程 54 — 编码实现（15 L4 任务 / 23.0 人·天 / 5750K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree 分支 | 进度 |
|---|---|---|---:|---:|---|---|---|---|---|
| WF-1-54.1 | 5 域 Cargo crate 骨架（player-service / economy-service / match-service / social-service / admin-service / cluster-ops-service / shared-platform 7 个） | 5 域 | 1.5 | 400K | 工程基础 | workspace / 5 域 DTL | g / i 等 10 步 | `wbs/WF-1-54.1` | ⬜ 未启动 |
| WF-1-54.2 | 5 域 gRPC Proto 定义（player/v1/player.proto 等 7 个 proto 文件） | 5 域 | 1.0 | 250K | WF-1-54.1 | crate 骨架 / workspace / 5 域 DTL §3 API 设计 | g / i 等 10 步 | `wbs/WF-1-54.2` | ⬜ 未启动 |
| WF-1-54.3 | tonic-build 配置（build.rs + OUT_DIR + module 暴露） | Platform | 1.0 | 200K | WF-1-54.2 | Proto 定义 / Cargo workspace | g / i 等 10 步 | `wbs/WF-1-54.3` | ⬜ 未启动 |
| WF-1-54.4 | sqlx 集成（每个域独立 DATABASE_URL + migration runner） | 5 域 | 2.0 | 500K | WF-1-54.3 | crate 骨架 / 5 DB 容器 | g / i 等 10 步 | `wbs/WF-1-54.4` | ⬜ 未启动 |
| WF-1-54.5 | error 类型定义（thiserror + 域特定 error） | 5 域 | 1.0 | 200K | WF-1-54.4 | crate 骨架 / 5 域 DTL | g / i 等 10 步 | `wbs/WF-1-54.5` | ⬜ 未启动 |
| WF-1-54.6 | domain entity + Repository trait 定义（per 5 域 DTL） | 5 域 | 1.5 | 400K | WF-1-54.5 | Proto 定义 / sqlx 集成 / 5 域 DTL §2 entity | g / i 等 10 步 | `wbs/WF-1-54.6` | ⬜ 未启动 |
| WF-1-54.7 | Service 层业务逻辑（player 登录/状态机/economy 交易/补偿） | 5 域 | 2.0 | 500K | WF-1-54.6 | entity 定义 / 5 域 DTL §4 业务 | g / i 等 10 步 | `wbs/WF-1-54.7` | ⬜ 未启动 |
| WF-1-54.8 | Q-003 Saga 状态机实现（per DTL-015/016） | economy | 2.0 | 500K | WF-1-54.7 | Service 层 / Q-003 设计 / Outbox 模式 | g / i 等 10 步 | `wbs/WF-1-54.8` | ⬜ 未启动 |
| WF-1-54.9 | Outbox pattern 实现（事件外发） | economy | 1.5 | 400K | WF-1-54.8 | sqlx 集成 / Saga 实现 | g / i 等 10 步 | `wbs/WF-1-54.9` | ⬜ 未启动 |
| WF-1-54.10 | CEM 事件订阅 + 转发逻辑 | cluster-ops | 2.0 | 500K | WF-1-54.9 | Outbox 实现 / CEM 主题路由 | g / i 等 10 步 | `wbs/WF-1-54.10` | ⬜ 未启动 |
| WF-1-54.11 | PFAU 每功能原子升级（节点异常委托 K8s） | cluster-ops | 2.0 | 500K | WF-1-54.10 | CEM 订阅 / k3s 集群 / ADR-0052 | g / i 等 10 步 | `wbs/WF-1-54.11` | ⬜ 未启动 |
| WF-1-54.12 | RBAC 中间件（per 域 RBAC） | admin | 1.5 | 400K | WF-1-54.11 | Service 层 / RBAC 设计 | g / i 等 10 步 | `wbs/WF-1-54.12` | ⬜ 未启动 |
| WF-1-54.13 | OTel span 注入（每个 gRPC method） | Platform | 1.5 | 400K | WF-1-54.12 | tonic-build / OTel Collector | g / i 等 10 步 | `wbs/WF-1-54.13` | ⬜ 未启动 |
| WF-1-54.14 | Prometheus metrics 暴露（每个域 /metrics 端点） | Platform | 1.5 | 400K | WF-1-54.13 | OTel 注入 / Prometheus 库 | g / i 等 10 步 | `wbs/WF-1-54.14` | ⬜ 未启动 |
| WF-1-54.15 | tracing 日志 + 结构化输出（JSON + 上下文） | Platform | 1.0 | 200K | WF-1-54.14 | OTel 注入 | g / i 等 10 步 | `wbs/WF-1-54.15` | ⬜ 未启动 |


#### §2A.2.55 工程 55 — 静态分析（10 L4 任务 / 7.1 人·天 / 1550K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree 分支 | 进度 |
|---|---|---|---:|---:|---|---|---|---|---|
| WF-1-55.1 | clippy 配置（clippy.toml + -D warnings 严格模式） | Platform | 0.5 | 100K | 工程基础 | crate 骨架 / workspace | 删 / 除 等 15 步 | `wbs/WF-1-55.1` | ⬜ 未启动 |
| WF-1-55.2 | rustfmt 配置（rustfmt.toml + 强制） | Platform | 0.3 | 50K | WF-1-55.1 | crate 骨架 / workspace | 删 / 除 等 15 步 | `wbs/WF-1-55.2` | ⬜ 未启动 |
| WF-1-55.3 | CI 集成 clippy --all-targets --all-features | Platform | 0.5 | 100K | WF-1-55.2 | clippy 配置 / CI workflow 1 | 关 / 闭 等 10 步 | `wbs/WF-1-55.3` | ⬜ 未启动 |
| WF-1-55.4 | CI 集成 rustfmt --check（不通过则 fail） | Platform | 0.3 | 50K | WF-1-55.3 | rustfmt 配置 / CI workflow 1 | 关 / 闭 等 10 步 | `wbs/WF-1-55.4` | ⬜ 未启动 |
| WF-1-55.5 | cargo-deny 集成（许可 + 重复 + 漏洞） | Platform | 1.0 | 200K | WF-1-55.4 | cargo-deny 配置 | 删 / 除 等 12 步 | `wbs/WF-1-55.5` | ⬜ 未启动 |
| WF-1-55.6 | cargo-audit 集成（CVE 数据库） | Platform | 0.5 | 100K | WF-1-55.5 | cargo-deny 配置 | 删 / 除 | `wbs/WF-1-55.6` | ⬜ 未启动 |
| WF-1-55.7 | 自定义 clippy lint（域特定规则） | Platform | 1.5 | 400K | WF-1-55.6 | clippy 配置 | 删 / 除 等 16 步 | `wbs/WF-1-55.7` | ⬜ 未启动 |
| WF-1-55.8 | secrecy + secret 扫描（gitleaks/trufflehog） | Platform | 1.0 | 200K | WF-1-55.7 | CI workflow 1 | 关 / 闭 等 10 步 | `wbs/WF-1-55.8` | ⬜ 未启动 |
| WF-1-55.9 | dependency 锁定（cargo update --locked CI 检查） | Platform | 0.5 | 100K | WF-1-55.8 | crate 骨架 | 关 / 闭 等 11 步 | `wbs/WF-1-55.9` | ⬜ 未启动 |
| WF-1-55.10 | code coverage 报告（cargo-llvm-cov + codecov.io 上传） | Platform | 1.0 | 250K | WF-1-55.9 | CI workflow 1 / testkit | 关 / 闭 等 13 步 | `wbs/WF-1-55.10` | ⬜ 未启动 |


#### §2A.2.56 工程 56 — 代码审查（10 L4 任务 / 3.6 人·天 / 660K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree 分支 | 进度 |
|---|---|---|---:|---:|---|---|---|---|---|
| WF-1-56.1 | PR 模板定义（.github/PULL_REQUEST_TEMPLATE.md） | Platform | 0.3 | 50K | 工程基础 | Rust 工具链 / GitHub 仓库 | 删 / 除 等 4 步 | `wbs/WF-1-56.1` | ⬜ 未启动 |
| WF-1-56.2 | CODEOWNERS 文件（按域分配 owner） | Platform | 0.3 | 50K | WF-1-56.1 | 5 域 Lead / GitHub 仓库 | 删 / 除 等 13 步 | `wbs/WF-1-56.2` | ⬜ 未启动 |
| WF-1-56.3 | review checklist 模板（功能 / 测试 / 文档 / 安全 / 性能） | Platform | 0.5 | 100K | WF-1-56.2 | CI workflow 1 | 删 / 除 等 12 步 | `wbs/WF-1-56.3` | ⬜ 未启动 |
| WF-1-56.4 | branch protection 规则配置（main 必须 review + CI 通过） | Platform | 0.3 | 50K | WF-1-56.3 | PR 模板 / CI workflow 1 | 关 / 闭 等 20 步 | `wbs/WF-1-56.4` | ⬜ 未启动 |
| WF-1-56.5 | required checks 配置（CI 4 workflow 必通） | Platform | 0.3 | 50K | WF-1-56.4 | CI 4 workflows / branch protection | 删 / 除 等 18 步 | `wbs/WF-1-56.5` | ⬜ 未启动 |
| WF-1-56.6 | PR 自动 label（按域 / 按文件） | Platform | 0.5 | 100K | WF-1-56.5 | PR 模板 / GitHub labeler | 删 / 除 等 22 步 | `wbs/WF-1-56.6` | ⬜ 未启动 |
| WF-1-56.7 | review SLA 约定（24h 内 review） | Ulysses | 0.1 | 10K | WF-1-56.6 | DEC-008 一人公司 | 无 /   等 8 步 | `wbs/WF-1-56.7` | ⬜ 未启动 |
| WF-1-56.8 | self-review 模板（一人公司 = Ulysses 自审） | Ulysses | 0.5 | 100K | WF-1-56.7 | review checklist | 删 / 除 等 4 步 | `wbs/WF-1-56.8` | ⬜ 未启动 |
| WF-1-56.9 | 审查记录归档（GitHub PR 评论 + 决议） | Ulysses | 0.3 | 50K | WF-1-56.8 | PR 模板 / GitHub 仓库 | 无 /   等 8 步 | `wbs/WF-1-56.9` | ⬜ 未启动 |
| WF-1-56.10 | merge 后自动关闭关联 Issue（per WBS L4 任务） | Ulysses | 0.5 | 100K | WF-1-56.9 | branch protection / GitHub Actions | 关 / 闭 等 11 步 | `wbs/WF-1-56.10` | ⬜ 未启动 |


#### §2A.2.57 工程 57 — 构建（10 L4 任务 / 7.8 人·天 / 1750K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree 分支 | 进度 |
|---|---|---|---:|---:|---|---|---|---|---|
| WF-1-57.1 | cargo build --release 配置（[profile.release] LTO + strip） | Platform | 0.3 | 50K | 工程基础 | crate 骨架 / workspace | 删 / 除 等 4 步 | `wbs/WF-1-57.1` | ⬜ 未启动 |
| WF-1-57.2 | cargo build --workspace 验证（5 域 + 3 配套 7 crate 全编） | Platform | 0.5 | 100K | WF-1-57.1 | release 配置 / crate 骨架 | g / i 等 10 步 | `wbs/WF-1-57.2` | ⬜ 未启动 |
| WF-1-57.3 | build 缓存策略（sccache + CI cache） | Platform | 1.0 | 200K | WF-1-57.2 | CI workflow 1 / build 验证 | 删 / 除 等 13 步 | `wbs/WF-1-57.3` | ⬜ 未启动 |
| WF-1-57.4 | binary 大小优化（strip + lto + codegen-units） | Platform | 0.5 | 100K | WF-1-57.3 | release 配置 / cargo-bloat | 删 / 除 等 13 步 | `wbs/WF-1-57.4` | ⬜ 未启动 |
| WF-1-57.5 | docker buildx 多架构（amd64 + arm64） | Platform | 2.0 | 500K | WF-1-57.4 | crate 骨架 / distroless | 删 / 除 等 8 步 | `wbs/WF-1-57.5` | ⬜ 未启动 |
| WF-1-57.6 | 镜像标签策略（semver + git-sha + latest） | Platform | 0.5 | 100K | WF-1-57.5 | buildx 多架构 | 删 / 除 等 10 步 | `wbs/WF-1-57.6` | ⬜ 未启动 |
| WF-1-57.7 | 镜像 SBOM 生成（syft + cyclonedx） | Platform | 1.0 | 250K | WF-1-57.6 | buildx 多架构 / syft 安装 | 关 / 闭 等 12 步 | `wbs/WF-1-57.7` | ⬜ 未启动 |
| WF-1-57.8 | 镜像签名（cosign keyless） | Platform | 1.0 | 250K | WF-1-57.7 | buildx 多架构 / SBOM 生成 | 删 / 除 等 12 步 | `wbs/WF-1-57.8` | ⬜ 未启动 |
| WF-1-57.9 | 本地 build 验证（cargo build --release + docker build） | Platform | 0.5 | 100K | WF-1-57.8 | release 配置 / buildx 多架构 | 无 /   等 8 步 | `wbs/WF-1-57.9` | ⬜ 未启动 |
| WF-1-57.10 | 构建产物归档（GitHub Releases + registry） | Platform | 0.5 | 100K | WF-1-57.9 | buildx 多架构 / CI workflow 1 | 删 / 除 等 19 步 | `wbs/WF-1-57.10` | ⬜ 未启动 |


#### §2A.2.58 工程 58 — CI（12 L4 任务 / 10.5 人·天 / 2300K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree 分支 | 进度 |
|---|---|---|---:|---:|---|---|---|---|---|
| WF-1-58.1 | CI workflow 模板（GitHub Actions 通用结构） | Platform | 0.5 | 100K | 工程基础 | Rust 工具链 / GitHub Actions | 删 / 除 等 4 步 | `wbs/WF-1-58.1` | ⬜ 未启动 |
| WF-1-58.2 | CI workflow: rust-ci 完善（matrix: 5 域 + clippy + fmt + test） | Platform | 1.5 | 400K | WF-1-58.1 | rust-ci workflow / SAST 配置 | 关 / 闭 等 10 步 | `wbs/WF-1-58.2` | ⬜ 未启动 |
| WF-1-58.3 | CI workflow: docs-ci 完善（lychee + markdownlint） | Platform | 1.0 | 200K | WF-1-58.2 | docs-ci workflow / SAST 配置 | 关 / 闭 等 10 步 | `wbs/WF-1-58.3` | ⬜ 未启动 |
| WF-1-58.4 | CI workflow: verify-docs-ci 完善（3 脚本必跑） | Platform | 1.0 | 200K | WF-1-58.3 | verify-docs-ci workflow / scripts/ | 关 / 闭 等 10 步 | `wbs/WF-1-58.4` | ⬜ 未启动 |
| WF-1-58.5 | CI workflow: docker-build 触发条件激活（仅 main + tag） | Platform | 1.0 | 250K | WF-1-58.4 | docker-build workflow / buildx 多架构 | 关 / 闭 等 10 步 | `wbs/WF-1-58.5` | ⬜ 未启动 |
| WF-1-58.6 | CI 缓存策略（sccache + cargo cache） | Platform | 1.0 | 200K | WF-1-58.5 | sccache 配置 / CI 4 workflows | 删 / 除 等 13 步 | `wbs/WF-1-58.6` | ⬜ 未启动 |
| WF-1-58.7 | CI 矩阵测试（ubuntu-latest + macos-latest） | Platform | 1.0 | 200K | WF-1-58.6 | rust-ci workflow | 删 / 除 等 9 步 | `wbs/WF-1-58.7` | ⬜ 未启动 |
| WF-1-58.8 | CI 必通才合并（branch protection + required checks） | Platform | 0.5 | 100K | WF-1-58.7 | required checks 配置 | 关 / 闭 等 18 步 | `wbs/WF-1-58.8` | ⬜ 未启动 |
| WF-1-58.9 | CI 失败通知（Slack webhook） | Platform | 0.5 | 100K | WF-1-58.8 | CI 4 workflows / Slack workspace | 关 / 闭 等 15 步 | `wbs/WF-1-58.9` | ⬜ 未启动 |
| WF-1-58.10 | CI 性能监控（workflow 时长 dashboard） | Platform | 1.0 | 200K | WF-1-58.9 | CI 4 workflows / Grafana | 删 / 除 等 17 步 | `wbs/WF-1-58.10` | ⬜ 未启动 |
| WF-1-58.11 | CI secrets 管理（GitHub Secrets + sealed-secrets） | Platform | 1.0 | 250K | WF-1-58.10 | QUIC 证书 / GitHub Secrets | 删 / 除 等 10 步 | `wbs/WF-1-58.11` | ⬜ 未启动 |
| WF-1-58.12 | CI 离线运行（act 本地模拟） | Platform | 0.5 | 100K | WF-1-58.11 | CI workflow 模板 | 删 / 除 等 9 步 | `wbs/WF-1-58.12` | ⬜ 未启动 |

### §2A.3 6 工程 L4 任务统计

| 工程 | 名称 | L4 任务数 | 累计人·天 | 累计 token/周 |
|---|---|---:|---:|---:|
| 53 | 开发环境构建 | 15 | 15.8 | 3,650K |
| 54 | 编码实现 | 15 | 21.0 | 5,250K |
| 55 | 静态分析 | 10 | 7.4 | 1,750K |
| 56 | 代码审查 | 10 | 3.6 | 850K |
| 57 | 构建 | 10 | 9.3 | 2,200K |
| 58 | CI | 12 | 11.0 | 2,550K |
| **合计** | **WF-1 实施** | **72** | **69.5 人·天** | **15960K tokens/周** |

**一人公司约束**（per DEC-008）：Ulysses 1 人 12 角色，按 14 周窗口 5 工作日/周 = 70 人·天总容量。WF-1 阶段 69.5 人·天 **几乎占满全部容量**。已知代价（per RGS-QA-001 v0.10 §9.5.7）。

### §2A.4 agent 协作模式

1. **单 agent 串行**（默认）：Ulysses 1 人按 L4 任务顺序逐个完成
2. **多 agent 并行**（worktree 隔离）：Ulysses 可在多个 worktree 间切换"伪并行"
3. **进度追踪**：`scripts/wbs_task_progress.sh <L4#>` 报告进度 → `.wbs-task-marker` JSON → 跨会话恢复

### §2A.5 后续（v0.4 待办）

- WF-2 单元测试（59-65）→ L4 任务分解
- WF-3 集成测试（66-75）→ L4 任务分解
- WF-4 系统测试（76-89）→ L4 任务分解
- WF-5 验收（90-95）→ L4 任务分解
- WF-6 部署（96-108）→ L4 任务分解
- WF-7 运维 + 收尾（109-150）→ L4 任务分解

### §2A.6 文档关联表（per 用户决策 2026-08-21 "wbs 和 spec 的关联性正确添加"）

> **per RGS-WBS-001 v0.3 §6.3 前置依赖字段规范** + RGS-SPEC-000 总表 + 5 域 DTL 详细设计 + RGS-IMPL-001/002~006 实施规范

#### §2A.6.1 6 工程 → SPEC 文档映射

| 工程 | 名称 | 主要关联 SPEC 文档 | 次要关联 SPEC |
|---|---|---|---|
| 53 | 開発環境構築 | RGS-IMPL-001 §2.1 工具链 + RGS-SPEC-000 §3 总体架构 + RGS-IMPL-006 CI 规范 | RGS-SPEC-DTL-018/015/016/026/019/020/031（5 域 DTL 边界）|
| 54 | 编码实现 | **RGS-SPEC-DTL-018**（player 域）+ **RGS-SPEC-DTL-015/016**（economy 域）+ **RGS-SPEC-DTL-026**（match 域）+ **RGS-SPEC-DTL-019/020**（social 域）+ **RGS-SPEC-DTL-031**（admin 域）| RGS-SPEC-000 §3.2 gRPC + §3.4 DB + RGS-SPEC-DTL-021~025/032~040（跨域 + shared-platform） |
| 55 | 静态分析 | RGS-IMPL-003 静态分析规范（per §6.3） | RGS-SPEC-000 §4 编码约束 |
| 56 | 代码审查 | RGS-IMPL-004 代码审查规范（per §6.4） | RGS-SPEC-000 §4 编码约束 |
| 57 | 构建 | RGS-IMPL-005 构建规范（per §6.5） | RGS-SPEC-000 §6 部署约束 |
| 58 | CI | RGS-IMPL-006 CI 规范（per §6.6） | RGS-SPEC-000 §4 编码约束 |

#### §2A.6.2 54 编码实现任务 ↔ 5 域 SPEC 详细映射

| L4 # | 任务 | owner | 主要 SPEC | 关联 DTL |
|---|---|---|---|---|
| WF-1-54.1 | 5 域 Cargo crate 骨架 | 5 域 | RGS-SPEC-000 §3 + RGS-IMPL-001 §2.3 | DTL-018/015/016/026/019/020/031（7 个）|
| WF-1-54.2 | 5 域 gRPC Proto 定义 | 5 域 | RGS-SPEC-000 §3.2 + §3.3 | DTL-018 §3 / DTL-015 §3 / DTL-016 §3 / DTL-026 §3 / DTL-019 §3 / DTL-020 §3 / DTL-031 §6 |
| WF-1-54.3 | tonic-build 配置 | Platform | RGS-SPEC-000 §3.2 + RGS-IMPL-001 §2.4 | — |
| WF-1-54.4 | sqlx 集成 | 5 域 | RGS-SPEC-000 §3.4 + RGS-IMPL-001 §4 | DTL-018 §4 / DTL-015 §4 / DTL-016 §4 / DTL-026 §4 / DTL-019 §4 / DTL-020 §4 / DTL-031 §7 |
| WF-1-54.5 | error 类型定义 | 5 域 | RGS-IMPL-001 §3 + RGS-SPEC-000 §3.5 | DTL-018 §5 / DTL-015 §5 / DTL-016 §5 / DTL-026 §5 / DTL-019 §5 / DTL-020 §5 / DTL-031 §8 |
| WF-1-54.6 | domain entity + Repository trait | 5 域 | RGS-SPEC-000 §3.4 | DTL-018 §2 / DTL-015 §2 / DTL-016 §2 / DTL-026 §2 / DTL-019 §2 / DTL-020 §2 / DTL-031 §2 |
| WF-1-54.7 | Service 层业务逻辑 | 5 域 | RGS-SPEC-000 §3.5 | DTL-018 §4 / DTL-015 §4 / DTL-016 §4 / DTL-026 §4 / DTL-019 §4 / DTL-020 §4 / DTL-031 §4 |
| **WF-1-54.8** | **Q-003 Saga 状态机** | **economy** | **RGS-SPEC-DTL-015 §6 + RGS-SPEC-DTL-016 §6** | **DTL-015/016（Q-003 跨域核心）** |
| **WF-1-54.9** | **Outbox pattern** | **economy** | **RGS-SPEC-DTL-015 §7** | **DTL-015（Outbox 模式）** |
| **WF-1-54.10** | **CEM 事件订阅 + 转发** | **cluster-ops** | **RGS-SPEC-DTL-031 §9** | **DTL-031（CEM + COC + admin 域）** |
| **WF-1-54.11** | **PFAU 每功能原子升级** | **cluster-ops** | **RGS-SPEC-DTL-031 §10 + RGS-ADR-0052** | **DTL-031（PFAU + all-reachable）** |
| **WF-1-54.12** | **RBAC 中间件** | **admin** | **RGS-SPEC-DTL-031 §8** | **DTL-031（RBAC + 角色）** |
| WF-1-54.13 | OTel span 注入 | Platform | RGS-SPEC-000 §5.1 | RGS-SPEC-DTL-021~025 跨域 + RGS-SPEC-DTL-032~040 shared-platform |
| WF-1-54.14 | Prometheus metrics | Platform | RGS-SPEC-000 §5.2 | RGS-SPEC-DTL-021~025/032~040 |
| WF-1-54.15 | tracing 日志 | Platform | RGS-SPEC-000 §5.3 | RGS-SPEC-DTL-021~025/032~040 |

#### §2A.6.3 53 開発環境構築任务 ↔ SPEC 映射

| L4 # | 任务 | 主要 SPEC | 关联 DTL |
|---|---|---|---|
| WF-1-53.1 | Rust 1.98 工具链 | RGS-IMPL-001 §2.1 + RGS-IMPL-002 §2 | — |
| WF-1-53.2 | Cargo workspace | RGS-IMPL-001 §2.2 + RGS-SPEC-000 §3 | DTL-018/015/016/026/019/020/031 |
| WF-1-53.3 | rgs-testkit | RGS-IMPL-001 §6 + RGS-SPEC-000 §4.5 | DTL-018/015/016/026/019/020/031 |
| WF-1-53.4 | CI workflow rust-ci | RGS-IMPL-006 §3 + RGS-IMPL-001 §2.5 | — |
| WF-1-53.5 | CI workflow docs-ci | RGS-IMPL-006 §4 | — |
| WF-1-53.6 | CI workflow verify-docs-ci | RGS-IMPL-006 §5 + RGS-EXEC-001 v0.3 §3.4 | — |
| WF-1-53.7 | CI workflow docker-build | RGS-IMPL-005 + RGS-IMPL-006 §6 | — |
| WF-1-53.8 | docker-compose dev | RGS-SPEC-000 §6 + RGS-OPS-001 §3 | DTL-018/015/016/026/019/020/031 |
| WF-1-53.9 | k3s 集群 | RGS-SPEC-000 §6 + RGS-OPS-001 §4 | — |
| WF-1-53.10 | 5 独立 PG 18.4 DB | RGS-SPEC-000 §3.4 + RGS-ARC-008 | DTL-018 §4 / DTL-015 §4 / DTL-016 §4 / DTL-026 §4 / DTL-019 §4 / DTL-020 §4 / DTL-031 §4 |
| WF-1-53.11 | QUIC 证书 | RGS-SPEC-000 §3.6 + RGS-IMPL-001 §5 | — |
| WF-1-53.12 | OTel + Prometheus | RGS-SPEC-000 §5 + RGS-GOBS-004 | RGS-SPEC-DTL-021~025/032~040 |
| WF-1-53.13 | distroless base image | RGS-IMPL-005 + RGS-OPS-001 §6 | — |
| WF-1-53.14 | cargo-deny + cargo-audit | RGS-IMPL-003 §5 | — |
| WF-1-53.15 | devcontainer | RGS-IMPL-001 §2.1 | — |

#### §2A.6.4 55-58 工程 ↔ RGS-IMPL 规范映射

| 工程 | 任务数 | 主要规范文档 |
|---|---|---|
| 55 静态分析 | 10 | RGS-IMPL-003 静态分析规范（含 clippy / rustfmt / cargo-deny / cargo-audit / secret 扫描 / 自定义 lint / 依赖锁定 / coverage 全部 8 个工具配置）|
| 56 代码审查 | 10 | RGS-IMPL-004 代码审查规范（含 PR 模板 / CODEOWNERS / checklist / branch protection / required checks / labeler / SLA / self-review / 审查归档 / 关联 Issue 关闭全部 10 个流程）|
| 57 构建 | 10 | RGS-IMPL-005 构建规范（含 release 配置 / workspace 验证 / sccache / binary 优化 / buildx / 标签 / SBOM / cosign / 本地验证 / 产物归档全部 10 个步骤）|
| 58 CI | 12 | RGS-IMPL-006 CI 规范（含 template / 4 workflow 完善 / sccache / 矩阵 / required checks / 通知 / dashboard / secrets / act 全部 9-12 个配置）|

#### §2A.6.5 RGS-SPEC-000 总表索引

**`RGS-SPEC-000_详细设计规格化总表.md`** 是 37 个 RGS-SPEC-DTL-XXX 的总索引。引用方式：

- **总表** → 列 36 份 DTL + 7 份 SPEC-DTL-001~009 + 11 份 SPEC-DTL-011~027 + 11 份 SPEC-DTL-031~040
- **L4 任务引用 SPEC** → 优先引用对应域的 RGS-SPEC-DTL-XXX（不是 SPEC-000）
- **跨域任务** → 引用 RGS-SPEC-DTL-021~025（cross-domain）+ RGS-SPEC-DTL-032~040（shared-platform）

#### §2A.6.6 RGS-IMPL-001 父文档引用规则

**所有 6 工程的 L4 任务前置**都隐含引用 `RGS-IMPL-001 实施约定与工程边界`（项目基线），但不在每行重复——在 `RGS-WBS-001 v0.3 §2 L2 域 / 域簇` 中 foundation 域是"workspace / testkit / CI / DAG validator / manifest"基线。

---

### §2A.7 激活条件

🔴 → 🟢：G-CODE-06（Rust 1.98 实测）+ G-CODE-03（5 独立 DB 拓扑图）+ NO-GO 完全解除

---

## §2 L2 域 / 域簇（8 个）

| L2 | 域 / 域簇 | 域 Lead（独立 per DEC-005）| 主要职责 |
|---|---|---|---|
| 1 | **foundation** | 架构师（兼）| workspace / testkit / CI / DAG validator / manifest |
| 2 | **player** | Player 域 Lead（独立）| 账号 / 角色 / 会话 epoch / 玩家状态 |
| 3 | **economy** | Economy 域 Lead（独立 + Q-003 二次确认）| 货币 / 道具 / 交易 / 补偿 / Outbox |
| 4 | **match** | Match 域 Lead（独立）| 匹配队列 / 对局 / 评分 / 100ms 性能 |
| 5 | **social** | Social 域 Lead（独立）| 社交关系 / 消息 / 活动 / 异步通知 |
| 6 | **admin / COC** | Admin 域 Lead（独立，不兼任 SRE）| GM / RBAC / 审计 / ClusterOps 控制面 |
| 7 | **cluster-ops** | cluster-ops 域 Lead（独立）| ClusterOpsService + CEM + PFAU + 状态机 |
| 8 | **shared-platform** | Platform Engineer（独立）| Rust 工具链 / Cargo.lock / 镜像 / K3s / OTel |

> **DEC-005 不兼任原则**：架构师不兼任 player / SRE 不兼任 admin 域 Lead；架构师可独立负责 foundation（per §3.1 PH-0 + PH-1 阶段）；SRE 不兼任 cluster-ops Lead（cluster-ops 域 Lead 独立配置）。

---

## §3 L3 任务簇框架（每域每 PH 8 个）

### §3.1 通用 8 任务簇模板

每域每 PH 阶段，按以下 8 个 L3 任务簇组织：

| L3 # | 任务簇 | 适用范围 |
|---|---|---|
| 1 | **API Spec** | gRPC 方法 / Proto 文件 / tonic-build / 编译期校验 |
| 2 | **业务逻辑** | 核心算法 / 状态机 / 错误码 / 边界条件 |
| 3 | **DB migration** | Schema / 索引 / 约束 / 双向迁移演练 |
| 4 | **UT 单元测试** | testkit helpers / mock / 覆盖率 ≥ 80% |
| 5 | **IT 集成测试** | 跨组件 / 跨 DB / Saga 步骤 |
| 6 | **ST 系统测试** | 端到端 / 性能 / chaos / RPO/RTO 演练 |
| 7 | **Helm chart** | template / values / NetworkPolicy / HPA |
| 8 | **observability** | OTel spans / Prometheus metrics / 仪表盘 |

### §3.2 任务簇适配

- **foundation 域**：8 任务簇替换为（workspace 骨架 / testkit / CI 工具链 / DAG validator / cargo-deny / manifest schema / 文档生成 / 工程约定）
- **cluster-ops 域**：8 任务簇替换为（Control Plane API / CEM / PFAU / 状态机 / RBAC / fencing / 审计 / OCC）
- **shared-platform 域**：8 任务簇替换为（Rust 工具链 / Cargo.lock 锁定 / 镜像构建 / K3s / OTel Collector / Helm / 密钥 / 灾备）

---

## §4 L4 任务清单（关键示例：player 域 PH-1 + economy 域 PH-1）

> **完整 L4 任务清单每域每 PH 32 个（8 任务簇 × 4 任务）**。本节给 player 域 + economy 域 PH-1 的完整 L4 任务清单作为模板；其他域/阶段由各域 Lead 在 PH-0 末出完整 L4 任务清单 + 签字。

### §4.1 player 域 PH-1 工程基础 L4 任务（8 任务簇 × 4 任务 = 32 L4）

#### §4.1.1 API Spec 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.1.1 | 列出 player 域 gRPC 方法（per RGS-SPEC-DTL-018）| Player Lead | 0.5 | 100K | RGS-DTL-018 v0.2 | gRPC 方法清单（含 request/response）| git revert |
| 1.1.2 | 定义 Proto 文件 `proto/rgs/player/v1/*.proto` | Player Lead | 1.0 | 200K | 1.1.1 | Proto 编译通过 + field 编号固定 | git revert |
| 1.1.3 | 配置 tonic-build (build.rs) | foundation 域 | 0.5 | 100K | 1.1.2 | cargo build 成功生成 Rust 代码 | git revert |
| 1.1.4 | 编译期校验 sqlx query + tonic method 一致 | foundation 域 | 0.5 | 100K | 1.1.3 | CI 阻断不一致 | 关闭 CI 检查 |

#### §4.1.2 业务逻辑任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.2.1 | `players` 表 + 实体定义 | Player Lead | 1.0 | 250K | 1.1.2 | Entity + Repository trait 定义 | git revert |
| 1.2.2 | `player_characters` / `player_inventory` 索引策略 | Player Lead | 1.0 | 250K | 1.2.1 | 索引按 player_id 分区 + UT 覆盖 | 删索引 |
| 1.2.3 | 登录态 JWT / session 字段 | Player Lead | 1.0 | 200K | 1.2.1 | 与 RGS-REQ-007 一致 | git revert |
| 1.2.4 | 状态机：登录 / 在线 / 离线 | Player Lead | 1.0 | 250K | 1.2.3 | 状态转移图 + UT 覆盖 | git revert |

#### §4.1.3 DB migration 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.3.1 | `20260821000001_create_players.sql` 迁移 | DBA + Player Lead | 0.5 | 100K | 1.2.1 | sqlx migrate run 成功 | sqlx migrate revert |
| 1.3.2 | `20260821000002_player_characters.sql` | DBA + Player Lead | 0.5 | 100K | 1.2.2 | 迁移成功 | sqlx migrate revert |
| 1.3.3 | `20260821000003_player_inventory.sql` | DBA + Player Lead | 0.5 | 100K | 1.2.2 | 迁移成功 | sqlx migrate revert |
| 1.3.4 | 双向迁移演练（forward + revert）| DBA | 0.5 | 50K | 1.3.1-3 | 双向 CI 通过 | 关 CI |

#### §4.1.4 UT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.4.1 | testkit helper for player 域 | foundation + Player Lead | 1.0 | 200K | RGS-TS-001 §3.14 | testkit crate 公开 API | git revert |
| 1.4.2 | UT 覆盖 players 表 CRUD | Player Lead | 1.0 | 200K | 1.4.1 | 覆盖率 ≥ 80% | git revert |
| 1.4.3 | UT 覆盖状态机（登录/在线/离线）| Player Lead | 1.0 | 250K | 1.2.4 | 状态转移 100% 覆盖 | git revert |
| 1.4.4 | cargo llvm-cov 报告 | foundation | 0.5 | 100K | 1.4.2-3 | CI 报告 ≥ 80% | 关检查 |

#### §4.1.5 IT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.5.1 | IT：player_service 启动 + health | Player Lead | 1.0 | 200K | 1.4 | Cargo build + health check 200 | git revert |
| 1.5.2 | IT：DB 集成（testcontainers PG 18.4）| Player Lead | 1.0 | 250K | 1.5.1 | testcontainers PG 启动 + migration | git revert |
| 1.5.3 | IT：登录态端到端 | Player Lead | 1.5 | 300K | 1.5.2 | JWT 创建 + 验证 + 刷新 | git revert |
| 1.5.4 | IT：跨域契约测试（player 事件被 social 订阅）| Social Lead | 1.5 | 300K | 1.5.3 | gRPC event 发送 + 接收 | git revert |

#### §4.1.6 ST 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.6.1 | ST：player_service 在 K8s 部署 | SRE Lead | 1.0 | 200K | 1.5 | helm install + kubectl get pods Ready | helm uninstall |
| 1.6.2 | ST：NFR-PT latency 验证 | Match Lead + SRE | 1.5 | 300K | 1.6.1 | p99 < 100ms | helm rollback |
| 1.6.3 | ST：chaos 演练（pod kill）| SRE Lead | 1.0 | 250K | 1.6.1 | 故障注入通过 | helm rollback |
| 1.6.4 | ST：RPO/RTO 验证 | SRE Lead | 1.5 | 300K | 1.6.3 | RPO < 5s / RTO < 60s | helm rollback |

#### §4.1.7 Helm chart 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.7.1 | `helm/rgs-player-service/Chart.yaml` | Platform + Player Lead | 0.5 | 100K | RGS-TS-001 §3.11 | Chart 解析 | helm uninstall |
| 1.7.2 | `values.yaml` 默认配置 | Platform + Player Lead | 0.5 | 100K | 1.7.1 | helm template 通过 | helm uninstall |
| 1.7.3 | `templates/deployment.yaml` | Platform | 0.5 | 100K | 1.7.2 | 5 副本 + HPA | helm uninstall |
| 1.7.4 | `templates/networkpolicy.yaml` | Platform | 0.5 | 100K | 1.7.3 | 仅 ClusterOps 可访问 | 删除 NetworkPolicy |

#### §4.1.8 observability 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.8.1 | OTel spans（login / logout / refresh）| Player Lead | 1.0 | 200K | 1.2.3 | 4 个 span 采集 | git revert |
| 1.8.2 | Prometheus metrics（qps / latency / error）| Player Lead | 1.0 | 200K | 1.8.1 | 3 个指标导出 | git revert |
| 1.8.3 | Grafana 仪表盘 player-overview | SRE Lead | 1.0 | 200K | 1.8.2 | 仪表盘 5 个 panel | 删除 dashboard |
| 1.8.4 | Loki 日志（JSON + trace_id）| Player Lead | 0.5 | 100K | 1.8.1 | 日志采集 100% | git revert |

**player 域 PH-1 L4 任务合计**：32 任务 / **~26 人·天** / **~5.6M tokens**（per §6.2.1.2 + §6.2.2.3 估算）

### §4.2 economy 域 PH-1 L4 任务（8 任务簇 × 4 任务 = 32 L4）

#### §4.2.1 API Spec 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.1.1 | 列出 economy 域 gRPC 方法（per RGS-SPEC-DTL-015/016）| Economy Lead | 1.0 | 200K | RGS-DTL-015/016 v0.2 | gRPC 方法清单 | git revert |
| 2.1.2 | 定义 Proto 文件 `proto/rgs/economy/v1/*.proto` | Economy Lead | 1.5 | 350K | 2.1.1 | Proto 编译通过 | git revert |
| 2.1.3 | 配置 tonic-build (build.rs) | foundation 域 | 0.5 | 100K | 2.1.2 | cargo build 成功 | git revert |
| 2.1.4 | Q-003 Saga 步骤定义（player/economy/social 跨域）| Economy Lead + 架构师 | 2.0 | 500K | 2.1.1 | Saga 步骤图（6 场景 per REV-005）| git revert |

#### §4.2.2 业务逻辑任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.2.1 | `accounts` + `account_balance` + `currency_types` 表 | Economy Lead | 1.5 | 400K | 2.1.2 | 3 表 schema + 实体 | git revert |
| 2.2.2 | `transactions` 表（事务日志 + request_id 幂等键）| Economy Lead | 1.5 | 400K | 2.2.1 | 事务日志 + 幂等键 | git revert |
| 2.2.3 | `CommitTransaction` 接口（永久事实）| Economy Lead | 2.0 | 500K | 2.1.4 | Saga 永久事实 commit | git revert |
| 2.2.4 | 货币精度（DECIMAL + f64 vs Decimal 决策）| Economy Lead | 1.0 | 250K | 2.2.1 | DECIMAL 类型 + 决策记录 | git revert |

#### §4.2.3 DB migration 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.3.1 | `20260821000010_accounts.sql` | DBA + Economy Lead | 0.5 | 100K | 2.2.1 | 迁移成功 | sqlx migrate revert |
| 2.3.2 | `20260821000011_transactions.sql` | DBA + Economy Lead | 1.0 | 200K | 2.2.2 | 迁移成功 + 索引 | sqlx migrate revert |
| 2.3.3 | `20260821000012_outbox.sql`（per RGS-IMPL-001 §3 Saga）| DBA + Economy Lead | 1.0 | 250K | 2.2.3 | Outbox 表 + 索引 | sqlx migrate revert |
| 2.3.4 | 双向迁移演练 + 锁等待回归 | DBA | 0.5 | 50K | 2.3.1-3 | 双向 CI 通过 | 关 CI |

#### §4.2.4 UT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.4.1 | testkit helper for economy 域（含 Saga mock）| foundation + Economy | 1.5 | 350K | 2.1.4 | testkit 公开 API | git revert |
| 2.4.2 | UT 覆盖账户 CRUD + 余额变更 | Economy Lead | 1.5 | 350K | 2.4.1 | 覆盖率 ≥ 80% | git revert |
| 2.4.3 | UT 覆盖 Saga 6 场景（正常/补偿/超时/人工/去重/PFAU+Saga）| Economy Lead | 3.0 | 700K | 2.4.1 | 6 场景 100% 覆盖（per REV-005 附件B）| git revert |
| 2.4.4 | cargo llvm-cov 报告 | foundation | 0.5 | 100K | 2.4.2-3 | CI ≥ 80% | 关检查 |

#### §4.2.5 IT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.5.1 | IT：economy_service 启动 + health | Economy Lead | 1.0 | 200K | 2.4 | Cargo build + health 200 | git revert |
| 2.5.2 | IT：DB 集成（独立 economy_db）| Economy Lead | 1.0 | 250K | 2.5.1 | testcontainers economy_db 启动 | git revert |
| 2.5.3 | IT：跨 DB Saga 真实演练（6 场景 per REV-005）| Economy Lead + DBA | 3.0 | 800K | 2.5.2 | 6 场景全部通过 | git revert |
| 2.5.4 | IT：Outbox 重试 + DLQ | Economy Lead | 1.5 | 400K | 2.5.3 | Outbox 消费者幂等 | git revert |

#### §4.2.6 ST 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.6.1 | ST：economy_service 在 K8s 部署 | SRE Lead | 1.0 | 250K | 2.5 | helm install + Ready | helm uninstall |
| 2.6.2 | ST：5 域跨 DB 事务正确性 | DBA + 5 域 Lead | 2.0 | 500K | 2.6.1 | 5 域跨 DB 一致 | helm rollback |
| 2.6.3 | ST：Saga 失败补偿验证 | Economy Lead | 1.5 | 400K | 2.6.1 | 补偿步骤回滚 | helm rollback |
| 2.6.4 | ST：人工升级路径（金额 > 阈值）| Economy Lead + Admin | 1.5 | 350K | 2.6.1 | 人工审核触发 + 审计 | helm rollback |

#### §4.2.7 Helm chart 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.7.1 | `helm/rgs-economy-service/Chart.yaml` | Platform + Economy Lead | 0.5 | 100K | RGS-TS-001 §3.11 | Chart 解析 | helm uninstall |
| 2.7.2 | `values.yaml` 默认配置 | Platform + Economy Lead | 0.5 | 100K | 2.7.1 | helm template 通过 | helm uninstall |
| 2.7.3 | `templates/deployment.yaml` | Platform | 0.5 | 100K | 2.7.2 | 5 副本 + HPA | helm uninstall |
| 2.7.4 | `templates/networkpolicy.yaml` | Platform | 0.5 | 100K | 2.7.3 | 仅 player/social 可访问 | 删除 |

#### §4.2.8 observability 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.8.1 | OTel spans（transaction / commit / compensate）| Economy Lead | 1.5 | 350K | 2.2.3 | 6 场景 span 完整 | git revert |
| 2.8.2 | Prometheus metrics（qps / commit / 补偿）| Economy Lead | 1.0 | 250K | 2.8.1 | 4 指标导出 | git revert |
| 2.8.3 | Grafana 仪表盘 economy-overview | SRE Lead | 1.0 | 200K | 2.8.2 | 5 panel | 删除 dashboard |
| 2.8.4 | Loki 日志（事务 + 补偿 + 人工升级）| Economy Lead | 0.5 | 100K | 2.8.1 | 3 类日志 | git revert |

**economy 域 PH-1 L4 任务合计**：32 任务 / **~38 人·天** / **~8.5M tokens**（per §6.2.1.2 + §6.2.2.3 估算上限）

### §4.3 完整 L4 任务占位清单（5 域 + 3 配套 × 8 PH × 32 L4 = 2,048 行）

> **v0.2 升版**（per user decision 2026-08-21）：v0.1 仅给 player / economy PH-1 完整 64 L4 任务示例；v0.2 通过 `scripts/build_wbs_v02.py` 生成完整 **2,048 L4 任务占位清单**（5 域 + foundation + cluster-ops + shared-platform × 8 PH × 8 任务簇 × 4 任务）。
>
> **占位清单独立文档**：[RGS-WBS-001_L4任务占位清单_v0.1.md](RGS-WBS-001_L4任务占位清单_v0.1.md)（**2071 行 / 297 KB**）
>
> **5 域 + 3 配套 Lead 补全责任**（PH-0.5 前）：
>
> | 责任人 | 补全量 | 截止 |
> |---|---|---|
> | Player 域 Lead | 256 L4（player 域 × 8 PH）| PH-0.5 |
> | Economy 域 Lead | 256 L4 + Q-003 二次确认 | PH-0.5 |
> | Match 域 Lead | 256 L4 | PH-0.5 |
> | Social 域 Lead | 256 L4 | PH-0.5 |
> | Admin 域 Lead | 256 L4 | PH-0.5 |
> | cluster-ops 域 Lead | 256 L4 | PH-0.5 |
> | foundation（架构师）| 256 L4 | PH-0.5 |
> | shared-platform（Platform）| 256 L4 | PH-0.5 |
> | **合计** | **2,048 L4** | — |
>
> **每行 6 字段补全**（人·天 / Tokens / 前置 / 验收 / 回滚 5 字段 + 签字 1 字段）：
>
> | 字段 | 单位 | 来源 |
> |---|---|---|
> | 人·天 | 0.1-5.0 | per RGS-TS-001 v0.6 §6.2.1.2 估算 |
> | Tokens | 50K-1M | per RGS-TS-001 v0.6 §6.2.2.3 估算 |
> | 前置 | L4 # 引用 | 同域前置 PH 任务 |
> | 验收 | 文字 | per RGS-IMPL-001 §3 质量门禁 |
> | 回滚 | git/helm revert | per RGS-IMPL-001 §5 部署约定 |
> | 签字 | 域 Lead / 架构 | PH-0.5 联合评审 |
>
> **维护方式**：
> 1. **编辑**：`docs/12-工作流/RGS-WBS-001_L4任务占位清单_v0.1.md`（5 域 Lead 各自编辑自己的域行；可用 Excel / VS Code 多列编辑）
> 2. **重生成**：`python scripts/build_wbs_v02.py`（保持结构一致；如已补全的行被覆盖，需手动合并）
> 3. **PH-0.5 签字**：5 域 Lead + SRE + 架构 + PM 按域签字
> 4. **PH-1 末**：每域 Lead 出 L5 工作包完整清单（per §5）
> 5. **PH-3 / PH-7 校准**：per RGS-TS-001 v0.6 §6.2.5 校准节点

---

## §5 L5 工作包示例

> **L5 = 最小可分配单元，≤ 2 人·天 或 ≤ 500K tokens**。
> 每个 L4 任务下钻 2-3 个 L5 工作包。

### §5.1 示例：player 域 1.4.1 testkit helper for player 域

| L5 # | 工作包 | owner | 人·天 | token | 验收项 |
|---|---|---|---:|---:|---|
| 1.4.1.1 | testkit PG helper（testcontainers 封装）| foundation | 0.5 | 100K | testkit::pg::test_pg() 函数 |
| 1.4.1.2 | testkit mock player 域 client | foundation | 0.5 | 100K | testkit::player::mock_client() |
| 1.4.1.3 | testkit fixture builder（玩家 / 角色 / 物品）| foundation | 1.0 | 250K | testkit::player::fixture() builder |

**L5 合计**：3 工作包 / **2 人·天** / **450K tokens**（≤ 500K 上限 ✅）

### §5.2 示例：economy 域 2.4.3 UT 覆盖 Saga 6 场景

| L5 # | 工作包 | owner | 人·天 | token | 验收项 |
|---|---|---|---:|---:|---|
| 2.4.3.1 | 场景 1：正常（player → economy → social 成功）| Economy Lead | 0.5 | 150K | 6 步骤全过 |
| 2.4.3.2 | 场景 2：补偿（economy 失败回滚 player / social）| Economy Lead | 0.5 | 150K | 补偿步骤回滚 |
| 2.4.3.3 | 场景 3：超时（economy 30s 未响应）| Economy Lead | 0.5 | 100K | 超时检测 + 补偿 |
| 2.4.3.4 | 场景 4：人工升级（金额 > 阈值）| Economy Lead | 0.5 | 100K | 人工审核触发 |
| 2.4.3.5 | 场景 5：去重（request_id 重复）| Economy Lead | 0.5 | 100K | 幂等保证 |
| 2.4.3.6 | 场景 6：PFAU + Saga（5 节点灰度）| Economy Lead + cluster-ops | 0.5 | 100K | PFAU all-reachable |

**L5 合计**：6 工作包 / **3 人·天** / **700K tokens**（> 500K 上限 ⚠️ → 拆分为 2 个 L5：2.4.3.1-3 + 2.4.3.4-6）

---

## §6 5 域 Lead × 14-18 周 WBS 汇总（粗算）

> **估算依据**：RGS-TS-001 v0.6 §6.2.1.2（人·天）+ §6.2.2.3（token）；每域 8 PH × 32 L4 任务 + foundation/cluster-ops/shared-platform 配套。

### §6.1 5 域 Lead WBS 汇总（人·天 + token 双轨）

| 域 | L4 任务数 | 人·天 / 周均 | token / 周均 | 14-18 周合计（人·天）| 14-18 周合计（token）|
|---|---:|---:|---:|---:|---:|
| Player 域 | 32 × 8 PH = 256 | ~3-5 / 周 | ~2M-4M / 周 | ~42-90 | ~28M-72M |
| Economy 域 | 32 × 8 PH = 256 | ~5-8 / 周 | ~4M-8M / 周 | ~70-144 | ~56M-144M |
| Match 域 | 32 × 8 PH = 256 | ~4-6 / 周 | ~3M-5M / 周 | ~56-108 | ~42M-90M |
| Social 域 | 32 × 8 PH = 256 | ~3-5 / 周 | ~2M-4M / 周 | ~42-90 | ~28M-72M |
| Admin / COC 域 | 32 × 8 PH = 256 | ~4-6 / 周 | ~3M-5M / 周 | ~56-108 | ~42M-90M |
| **5 域 Lead 合计** | **1,280** | **~19-30 / 周** | **~14M-26M / 周** | **~266-540** | **~196M-468M** |

> **vs RGS-TS-001 v0.6 §6.2.1.2 / §6.2.2.3 估算**：本 WBS 框架 5 域合计与 TS-001 §6.2 双轨估算区间**一致**。

### §6.2 foundation / cluster-ops / shared-platform 配套（不计入 5 域 Lead WBS）

| 域簇 | L4 任务数 | 人·天 / 14-18 周 | token / 14-18 周 |
|---|---:|---:|---:|
| foundation（架构师兼）| ~64 | ~30-50 | ~15M-30M |
| cluster-ops（独立 Lead）| ~128 | ~80-120 | ~50M-90M |
| shared-platform（Platform Engineer 兼）| ~96 | ~50-80 | ~30M-50M |
| **配套合计** | **~288** | **~160-250** | **~95M-170M** |

> **总 WBS（5 域 Lead + 配套）**：~1,568 L4 任务 / **~426-790 人·天** / **~291M-638M tokens**

---

## §7 WBS 维护与校准

### §7.1 校准节点

| 节点 | 校准内容 | 责任方 |
|---|---|---|
| PH-0.5 | RGS-ENV-CALIB-001 校准数据 vs WBS 估算 | 5 域 Lead + SRE + 架构 + PM |
| PH-3 | 进度对账（WBS 完成率 vs 实际）| 5 域 Lead + SRE + PM |
| PH-7 | 最终对账（OLU 实际 vs 估算 + CIR 闭环）| SRE + PM + 架构 |

### §7.2 偏差处理

| 偏差 | 处理 |
|---|---|
| < 30% | 接受 |
| 30-50% | WBS 升 v0.2 + 重新估 |
| > 50% | NO-GO 升级（53 启动条件不满足） |

### §7.3 5 域 Lead L4 任务清单补全

> **PH-0 末（2026-08-21 v0.1 草稿发布）**仅给 player 域 + economy 域 PH-1 完整 L4 任务清单作为模板。
> **PH-0.5 前**：5 域 Lead + foundation + cluster-ops + shared-platform 各自补全其余 7 PH 的 L4 任务清单。
> **PH-1 末**：每域 Lead 出 L5 工作包完整清单。

---

## §8 签字栏

| # | 角色 | 姓名 | 签字 | 日期 | 结论 |
|---|---|---|---|---|---|
| 1 | 架构师（foundation + 监督）| __________ | __________ | ____-__-__ | ☐ L1-L3 框架接受 / ☐ 修订 |
| 2 | Player 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ PH-1 L4 模板接受 |
| 3 | Economy 域 Lead（独立 + Q-003 二次确认）| __________ | __________ | ____-__-__ | ☐ PH-1 L4 模板接受 |
| 4 | Match 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 5 | Social 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 6 | Admin 域 Lead（独立，不兼任 SRE）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 7 | cluster-ops 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 8 | Platform Engineer（shared-platform）| __________ | __________ | ____-__-__ | ☐ 框架接受 |
| 9 | SRE Lead（监督 OLU）| __________ | __________ | ____-__-__ | ☐ OLU 双轨估算一致 |
| 10 | PM | __________ | __________ | ____-__-__ | ☐ 资源决策接受 / ☐ 偏差 > 30% 升 v0.2 |

---

> **本 WBS 与 RGS-PLAN-001 v0.8 §3.1 PH 表 / RGS-TS-001 v0.6 §6.2 双轨制 / RGS-ENV-CALIB-001 校准模板 三方一致**。
> **5 域 Lead L4 任务清单补全由各 Lead 在 PH-0.5 前出**。


## §11 跨任务依赖图（DAG，v0.3 新增）

跨 L4 任务的依赖关系图（部分示例）：

```
WF-0 (需求)
  ├─ 1.1.1 [列出 gRPC 方法]
  │    └─→ WF-1.1.1 [定义 Proto]
  │         └─→ WF-1.1.2 [tonic-build 配置]
  │              └─→ WF-1.1.3 [编译期校验]
  │                   └─→ WF-2.1.1 [UT 编写]
  │                        └─→ WF-3.1.1 [IT 集成]
  │                             └─→ WF-4.1.1 [ST 系统]
  │                                  └─→ WF-5.1.1 [UAT]
  │                                       └─→ WF-6.1.1 [部署]
  │                                            └─→ WF-7.1.1 [运维]
  └─ ... 1.1.4 / 1.1.5 / 1.1.6 并行
```

依赖图存储：`docs/12-工作流/RGS-WBS-001_DAG_v0.3.md`（v0.3 新增，自动生成）

---

## §12 WBS 状态机（v0.3 新增）

每个 L4 任务状态机：

```
[⬜ 未启动] ──start──> [🟡 进行中] ──progress X%──> [🟡 进行中]
                         │                          │
                         │ done                     │ blocked
                         ↓                          ↓
                    [✅ 完成]                  [❌ 阻塞]
                         │                          │
                         │ regress                  │ unblock
                         ↓                          ↓
                    [🟡 进行中]               [🟡 进行中]
```

状态转换由 `scripts/wbs_task_progress.sh` 管理，写入 `RGS-WBS-001_L4任务进度表_v0.3.md`。

---

## §13 跨会话恢复（v0.3 新增）

每个 L4 任务在 worktree 中包含 `.wbs-task-marker` 文件（git 跟踪），格式：

```json
{
  "l4_id": "WF-1-2-1.1.1",
  "branch": "wbs/WF-1-2-1.1.1",
  "status": "in_progress",
  "started_at": "2026-08-21T06:30:00Z",
  "last_progress_at": "2026-08-21T07:15:00Z",
  "progress_pct": 50,
  "progress_note": "已完成 Proto 定义，待 tonic-build",
  "assignee": "Ulysses"
}
```

跨会话恢复：
```bash
# 列出所有进行中的 L4 任务
./scripts/list_wbs_tasks.sh --status in_progress

# 恢复到 worktree
git worktree add ../rgs-task-wbs-WF-1-2-1.1.1 wbs/WF-1-2-1.1.1
cd ../rgs-task-wbs-WF-1-2-1.1.1
cat .wbs-task-marker
# 看到 status=in_progress, progress_pct=50 → 继续任务
```

---

## §14 与 DEC-008 一人公司治理基线的兼容

- L4 任务的 owner 字段 = `Ulysses`（1 人 12 角色）
- 进度签字 = Ulysses 1 人（自审自批，流程化补偿：CI + 自动化 + 自我 PR review + OTel）
- worktree 分支并行 = 1 个 Ulysses 可在多个 worktree 间切换（不是多人协作，但 1 人多任务并行）
- 风险：1 人串行可能比 14-18 周更长（per RGS-QA-001 v0.10 §9.5.7）

**未变更**：5 域 Lead 配置（仍可独立，Ulysses = 1 人 12 角色兼任）

---

## 关联文档

- 上游：RGS-WF-001 v0.6（150 工程 100% 覆盖）
- 下游：
  - `RGS-WBS-001_L4任务占位清单_v0.1.md`（2,048 L4 占位，PH-0.5 前由 5 域 Lead 补全）
  - `RGS-WBS-001_L4任务进度表_v0.3.md`（v0.3 新增，agent 报告进度后自动更新）
  - `RGS-WBS-001_DAG_v0.3.md`（v0.3 新增，跨任务依赖图）
  - `RGS-WT-001`（worktree 规范）
- 工具：
  - `scripts/build_wbs_v02.py`（v0.2 L4 生成器）
  - `scripts/list_wbs_tasks.sh`（v0.3 新增）
  - `scripts/create_worktree_for_task.sh`（v0.3 新增）
  - `scripts/wbs_task_progress.sh`（v0.3 新增）
  - `scripts/merge_wbs_task.sh`（v0.3 新增）
