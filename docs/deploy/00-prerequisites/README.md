# 00-prerequisites 部署前置条件

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEPLOY-00 |
| 版本 | 0.1（占位 + 文档化）|
| 依据 | RGS-PLAN-001 v0.7 §3 + RGS-ENV-001 v0.2 + RGS-ENV-CALIB-001 v0.1 + handoff §5 |
| 状态 | **🟠 NO-GO 状态（per RGS-PLAN-001 v0.7 §3.3）** |
| 保密级别 | 内部限定（Internal Use Only）|

---

## §1 部署前置条件总览

> **53 启动前必须满足的 4 大类前置条件**（per RGS-PLAN-001 v0.7 §1.2 不可变约束）：

### §1.1 7 G-CODE 全部 Closed

| G-CODE | 主题 | 当前状态 | 关闭证据 | 责任方 |
|---|---|---|---|---|
| G-CODE-01 | 36 SPEC 字段级评审 | 🟠 Open | DD Review + SPEC 映射完整 | 架构 + QA |
| G-CODE-02 | DTL-031 字段 Review | 🟠 Open / Blocker | 接口 / 状态机 / fencing / CEM/PFAU 审批栏具名签署 | 架构 + Platform + DBA + cluster-ops + Admin |
| G-CODE-03 | ADR-0052 联审 | 🟠 Open | ADR 审批栏 + 拓扑核验 + 故障注入 + 风险接受 | 架构 + SRE + DBA + Platform |
| G-CODE-04 | Q-003 Saga 6 场景 | 🟠 Open / Blocker | Saga/Outbox/补偿 + 6 业务场景验收 | 架构 + DBA + Economy 域 Lead |
| G-CODE-05 | 5 域 DTL 边界冻结 | 🟠 Open | 5 域 DD Review + 接口/事件/DB/插件依赖矩阵 | 5 域独立 Lead + 架构 |
| G-CODE-06 | 工具链 + 开发环境基线 | 🟠 Open | Rust 1.98 实测 + PG 18.4 + K3s + 锁定 CI | Platform + DBA + SRE |
| G-CODE-07 | OLU + 测试基础前置 | 🟠 Open | OLU 重算（含 5 域独立 Lead）+ Q-031 WBS + testkit | SRE + QA + PM |

### §1.2 RGS-ENV-001 v0.2 12 类签字齐全

- **2 项 Ulysses 实际签**（架构师 #8 + PM #12）
- **10 项所有者背书 + 待具名责任人**（DBA / SRE / 5 域 Lead / Platform / QA / Q-003 二次）

### §1.3 RGS-REV-003 §7.3 12 类签字齐全

- **8 项 Ulysses 实际签**（架构师 #1 + 评审主持人 + PM）
- **10+ 项所有者背书 + 待具名责任人**（DBA / SRE / 5 域 Lead / Platform / QA / Q-003 二次）

### §1.4 5 域独立 Lead 到位（per DEC-005）

- player / economy / match / social / admin 5 域各自配独立 Lead
- 不接受架构师兼任 player 域 Lead；不接受 SRE 兼任 admin 域 Lead
- 具名责任人签字补全 12 类"所有者背书"占位

---

## §2 子文档索引

| # | 文档 | 主题 | 当前状态 |
|---|---|---|---|
| 1 | [00-no-go-checklist_v0.2.md](00-no-go-checklist_v0.2.md) | NO-GO 解除 7+12 checklist | 🟡 实时更新 |
| 2 | [01-environment-verification.md](01-environment-verification.md) | RGS-ENV-001 v0.2 引用 | 🟡 工具就位 |
| 3 | [02-domain-leads-onboard.md](02-domain-leads-onboard.md) | 5 域 Lead 到位 checklist | 🟡 占位 |
| 4 | [03-rust-198-environment.md](03-rust-198-environment.md) | Rust 1.98 + Cargo.lock + CI 基线 | 🟡 占位 |
| 5 | [04-postgresql-184-setup.md](04-postgresql-184-setup.md) | PG 18.4 + 5 DB 划分 | 🟡 占位 |

---

## §3 部署准备 SOP 概要

> **NO-GO 解除前的所有"部署"动作 = 文档化 + 占位骨架**，不实际执行。

1. **PH-0**（W1-2）：架构师 + PM 实际签 + 5 域 Lead 招聘启动
2. **PH-0.5**（W2 末）：RGS-ENV-001 v0.2 §6 12 类签字 100% 具名责任人补全
3. **PH-1**（W3-4）：Rust 1.98 实测 + Cargo workspace 占位 + Cargo.lock 锁定
4. **PH-2**（W5-6）：PG 18.4 + 5 DB 划分 + K3s cluster 占位
5. **PH-3**（W7-9）：ClusterOpsService + CEM/PFAU + AdminService 占位
6. **PH-4**（W9-12）：player 端到端 + Saga 契约
7. **PH-5**（W12-14）：5 域联调
8. **PH-6**（W14-16）：Active-Active + 100k CCU 演练
9. **PH-7**（W17-18）：发布 Gate

## §4 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。5 份前置条件占位文档（00-no-go-checklist / 01-env-verify / 02-leads-onboard / 03-rust-198 / 04-pg-184）。**per user decision 2026-08-21 C 折中**：NO-GO 状态保留，部署准备仅文档化。 |
