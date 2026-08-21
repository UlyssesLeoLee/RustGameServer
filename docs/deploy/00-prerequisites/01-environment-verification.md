# 01-环境核验引用（Environment Verification Reference）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEPLOY-00-01 |
| 版本 | 0.1（引用 + 不重复内容）|
| 依据 | RGS-ENV-001 v0.2 + RGS-ENV-CALIB-001 v0.1 |
| 状态 | **🟠 NO-GO 状态** |
| 保密级别 | 内部限定（Internal Use Only）|

---

> **本文件不重复 RGS-ENV-001 v0.2 内容**——所有核验项、命令、通过标准均在主文档。本文件仅作引用 + 部署前置视角。

## §1 主文档

- **[RGS-ENV-001 v0.2 环境核验记录模板](../../00-基准与治理/reviews/RGS-ENV-001_环境核验记录模板_v0.2.md)**：5 层核验（工具链 / PG 18.4 / K3s / 锁定依赖 CI / 跨工具集成）+ 12 类签字栏
- **[RGS-ENV-CALIB-001 v0.1 OLU 校准记录模板](../../00-基准与治理/reviews/RGS-ENV-CALIB-001_OLU校准记录模板_v0.1.md)**：5 域 Lead × 1-2 周 人·天 + token 双轨实测

## §2 5 层核验范围

| 层 | 核验内容 | 责任方 | 当前状态 |
|---|---|---|---|
| §1 工具链 | rustc/cargo 1.98 + clippy + rustfmt + sqlx-cli + cargo-deny/audit/llvm-cov | Platform Engineer | 🟡 占位 |
| §2 PG 18.4 | psql + 服务器连接 + 5 DB 划分 + sqlx 编译期 + migration 双向演练 | DBA Lead | 🟡 占位 |
| §3 K3s | kubectl + 节点就绪 + CoreDNS/Traefik + Helm + 镜像仓库 | SRE Lead | 🟡 占位 |
| §4 锁定依赖 CI | Cargo.lock 入仓 + --locked 构建 + fmt/clippy/deny/audit/llvm-cov | Platform Engineer | 🟡 占位 |
| §5 跨工具集成 | sqlx 编译期 + tonic gRPC + tracing + distroless 容器 | 架构师 | 🟡 占位 |

## §3 12 类签字栏（per RGS-ENV-001 v0.2 §6）

| # | 角色 | 责任方 | Ulysses 状态 |
|---|---|---|---|
| 1 | DBA Lead | DBA Lead | ⏳ 所有者背书 |
| 2 | SRE Lead | SRE Lead | ⏳ 所有者背书 |
| 3 | Player 域 Lead | Player 域 Lead | ⏳ 所有者背书 |
| 4 | Economy 域 Lead | Economy 域 Lead | ⏳ 所有者背书 |
| 5 | Match 域 Lead | Match 域 Lead | ⏳ 所有者背书 |
| 6 | Social 域 Lead | Social 域 Lead | ⏳ 所有者背书 |
| 7 | Admin 域 Lead | Admin 域 Lead | ⏳ 所有者背书 |
| 8 | 架构师 | 架构师（Ulysses）| ✅ **Ulysses 实际签 2026-08-21** |
| 9 | Q-003 二次 | Economy 域 Lead | ⏳ 所有者背书 |
| 10 | Platform Engineer | Platform Engineer | ⏳ 所有者背书 |
| 11 | QA Lead | QA Lead | ⏳ 所有者背书 |
| 12 | PM | PM（Ulysses）| ✅ **Ulysses 实际签 2026-08-21** |

## §4 部署前置视角

> **部署启动条件**：12 类签字 100% 具名责任人补全 + 5 层核验全部通过。

| 状态 | 含义 | 行动 |
|---|---|---|
| 🟢 全部通过 | 53 启动可批准 | PM 按 handoff §5 Step 4 |
| 🟡 部分通过 | 部分签字 / 部分核验 | 等具名责任人到位 |
| 🔴 失败 | 核验不通过 | 3 次修复后升级 NO-GO |

## §5 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。引用 RGS-ENV-001 v0.2 + RGS-ENV-CALIB-001 v0.1；不重复内容。 |
