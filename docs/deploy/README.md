# 部署目录（Deployment Directory）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEPLOY-README |
| 版本 | 0.1（占位 + 部署准备文档化）|
| 依据 | RGS-PLAN-001 v0.8 §1.2 不可变约束 + handoff §1 NO-GO 原则 + RGS-EXEC-001 v0.3 §8 所有者背书机制 + RGS-IMPL-001 §5 部署约定 + RGS-TS-001 v0.6 §3.11 部署与编排 |
| 状态 | **🟠 NO-GO 状态（per RGS-PLAN-001 v0.8 §3.3）** |
| 保密级别 | 内部限定（Internal Use Only）|

---

## ⚠️ NO-GO 状态（强制声明）

> **本目录所有内容均处于 NO-GO 状态**（per RGS-PLAN-001 v0.8 §3.3 + handoff §1）：
> 1. 7 个 G-CODE 中 6 个仍 **🟠 Open / Blocker**：
>    - G-CODE-01：36 SPEC 字段级评审
>    - G-CODE-02：DTL-031 字段 Review（Q-025）
>    - G-CODE-03：ADR-0052 联审
>    - G-CODE-04：Q-003 Saga 6 场景演练
>    - G-CODE-05：5 域 DTL 边界冻结
>    - G-CODE-07：OLU 重算
> 2. RGS-REV-003 12 类签字栏：**8 项 Ulysses 实际签 + 4 项所有者背书占位**（per §3.4.4 / RGS-EXEC-001 v0.3 §8）
> 3. RGS-ENV-001 12 类签字栏：**2 项 Ulysses 实际签 + 10 项所有者背书占位**
> 4. §1.2 不可变约束："未完成 RGS-QA-001 四类 Gate 前，**只进行文档、契约、原型和测试设计；不提交业务实现**。禁止业务编码、迁移与部署。"
> 5. 所有者背书**不替代具名责任人签字**（per §3.4.4.2 + RGS-EXEC-001 v0.3 §8.3）
> 6. 53 启动推迟：具名责任人到位前，53 不可启动
>
> **本目录内全部为"部署准备文档化"——配置 / 文档 / 占位骨架；不含实际业务 Rust 代码 / sqlx migration / 真实 K8s deploy**。

---

## §1 目录结构

```
docs/deploy/
├── README.md                           # 本文件（总览 + NO-GO 声明）
├── 00-prerequisites/                   # 部署前置条件
│   ├── README.md                       # 前置条件总览
│   ├── 00-no-go-checklist_v0.2.md            # NO-GO 解除 checklist（7 G-CODE + 12 类签字 + 5 域 Lead 到位）
│   ├── 01-environment-verification.md  # RGS-ENV-001 v0.3 引用
│   ├── 02-domain-leads-onboard.md      # 5 域 Lead + 3 配套到位 checklist
│   ├── 03-rust-198-environment.md       # Rust 1.98 + Cargo.lock + CI 基线
│   └── 04-postgresql-184-setup.md       # PG 18.4 + 5 DB 划分
├── 01-k8s-manifests/                   # K8s 占位 manifest（不含真实镜像 / namespace）
│   ├── README.md
│   ├── namespace.yaml                  # 5 域 namespace 占位
│   ├── rbac/                           # RBAC 占位
│   │   ├── README.md
│   │   ├── player-service-sa.yaml      # ServiceAccount 占位
│   │   └── cluster-ops-rbac.yaml       # 集群级 RBAC 占位
│   ├── network-policies/               # NetworkPolicy 占位
│   │   ├── README.md
│   │   └── default-deny.yaml           # 默认拒绝 + 显式允许
│   └── resource-quotas/                # ResourceQuota 占位
│       ├── README.md
│       └── namespace-quota.yaml
├── 02-helm-charts/                     # Helm chart 占位（不含真实 values）
│   ├── README.md
│   ├── shared-library/                 # 共享 Chart 库占位
│   │   ├── Chart.yaml
│   │   ├── values.yaml
│   │   └── templates/_helpers.tpl
│   ├── player-service/
│   ├── economy-service/
│   ├── match-service/
│   ├── social-service/
│   ├── admin-service/
│   ├── cluster-ops-service/
│   └── gateway-service/
├── 03-db-migrations/                   # sqlx migration 占位（不含实际 schema）
│   ├── README.md
│   ├── player_db/                      # 5 DB 独立目录占位
│   ├── economy_db/
│   ├── match_db/
│   ├── social_db/
│   └── admin_db/
├── 04-ci-cd/                           # CI/CD 占位（不实际触发）
│   ├── README.md
│   ├── github-actions/
│   │   ├── rgs-ci.yaml                 # 主 CI pipeline 占位
│   │   ├── rgs-release.yaml            # 发布 pipeline 占位
│   │   └── rgs-nightly.yaml            # 夜间 pipeline 占位
│   └── image-build/
│       ├── Dockerfile.rgs             # 镜像构建占位
│       └── docker-bake.hcl            # 镜像编排占位
├── 05-deploy-sop.md                    # 部署 SOP 步骤清单（NO-GO 解除后才执行）
├── 06-rollback-sop.md                  # 回滚 SOP 步骤清单
└── 07-no-go-checklist_v0.2.md               # NO-GO 解除 checklist（v0.1 占位 + 完整 7 G-CODE 状态）
```

## §2 部署准备状态

| 目录 | 状态 | 责任方 | NO-GO 解除后激活条件 |
|---|---|---|---|
| 00-prerequisites | 🟡 占位文档化 | 架构师 + PM | 7 G-CODE 全部 Closed |
| 01-k8s-manifests | 🟡 占位（不含真实镜像 / namespace）| Platform Engineer + SRE | 12 类签字齐全 |
| 02-helm-charts | 🟡 占位（不含真实 values）| Platform Engineer | 12 类签字齐全 |
| 03-db-migrations | 🟡 占位（不含实际 schema）| DBA Lead + 5 域 Lead | Q-003 + DTL-031 签字 |
| 04-ci-cd | 🟡 占位（不实际触发）| Platform Engineer | Rust 1.98 实测通过 |
| 05-deploy-sop | 🟡 步骤清单 | SRE + Platform | 12 类签字齐全 |
| 06-rollback-sop | 🟡 步骤清单 | SRE + Platform | 12 类签字齐全 |
| 07-no-go-checklist | 🟢 实时更新 | PM | 7 G-CODE 全部 Closed |

## §3 责任分工（per RGS-PLAN-001 v0.8 §3.4.4 所有者背书机制）

| 责任方 | 角色 | Ulysses 状态 | 部署目录职责 |
|---|---|---|---|
| 架构师（Ulysses）| 架构师 | ✅ **Ulysses 实际签** | 整体架构 + 跨工具集成 |
| PM（Ulysses）| 项目负责人 | ✅ **Ulysses 实际签** | 资源决策 + 范围 + 53 启动授权 |
| SRE Lead | K3s / chaos / OLU | ⏳ 所有者背书 + 待具名 | 01-k8s-manifests + 05-deploy-sop + 06-rollback-sop |
| DBA Lead | PG 18.4 / 5 DB | ⏳ 所有者背书 + 待具名 | 03-db-migrations + 04-prerequisites |
| Platform Engineer | Rust / Cargo / 镜像 | ⏳ 所有者背书 + 待具名 | 02-helm-charts + 04-ci-cd + Dockerfile |
| Player 域 Lead | player 域 | ⏳ 所有者背书 + 待具名 | player-service + 03-db-migrations/player_db |
| Economy 域 Lead | economy + Q-003 | ⏳ 所有者背书 + 待具名 | economy-service + 03-db-migrations/economy_db |
| Match 域 Lead | match | ⏳ 所有者背书 + 待具名 | match-service + 03-db-migrations/match_db |
| Social 域 Lead | social | ⏳ 所有者背书 + 待具名 | social-service + 03-db-migrations/social_db |
| Admin 域 Lead | admin + COC | ⏳ 所有者背书 + 待具名 | admin-service + 03-db-migrations/admin_db |
| cluster-ops 域 Lead | ClusterOpsService | ⏳ 所有者背书 + 待具名 | cluster-ops-service + 01-k8s-manifests/rbac |
| QA Lead | 测试 + 覆盖率 | ⏳ 所有者背书 + 待具名 | 04-ci-cd/rgs-ci.yaml 测试阶段 |

## §4 关联文档

- [RGS-PLAN-001 v0.8](../12-工作流/RGS-PLAN-001_项目实施计划_v0.8.md) — 项目实施计划（§1.2 不可变约束 + §3.4.4 所有者背书机制）
- [RGS-IMPL-001](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md) — 实施约定 + §5 部署约定
- [RGS-TS-001 v0.6](../10-技术选型/RGS-TS-001_主要技术选型报告.md) — §3.11 部署与编排
- [RGS-WBS-001 v0.3](../12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md) — L4 任务清单（部署步骤补全）
- [RGS-EXEC-001 v0.3](../00-基准与治理/reviews/RGS-EXEC-001_G-CODE专题突破操作手册_v0.3.md) — §8 所有者背书机制
- [RGS-ENV-001 v0.3](../00-基准与治理/reviews/RGS-ENV-001_环境核验记录模板_v0.3.md) — 12 类签字栏（含 2 项 Ulysses 实际签 + 10 项所有者背书）
- [RGS-ENV-CALIB-001 v0.1](../00-基准与治理/reviews/RGS-ENV-CALIB-001_OLU校准记录模板_v0.1.md) — OLU 校准

## §5 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。创建 docs/deploy/ 目录骨架 + NO-GO 声明 + 责任分工；全部子目录为占位文档化（不实际部署）。**per user decision 2026-08-21 C 折中**：所有者背书机制下，部署准备可文档化，但 53 启动仍待具名责任人到位。 |
