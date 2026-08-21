# 02-5 域 Lead 到位 Checklist（Domain Leads Onboard Checklist）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEPLOY-00-02 |
| 版本 | 0.1（占位 + 文档化）|
| 依据 | RGS-QA-001 v0.12 DEC-005 + RGS-PLAN-001 v0.8 §3.4.4 + RGS-WBS-001 v0.2 §2 |
| 状态 | **🟠 5 域 Lead 待具名到位** |
| 保密级别 | 内部限定（Internal Use Only）|

---

## §1 5 域独立 Lead 到位要求（per DEC-005）

> **核心原则**（per user decision 2026-08-21 DEC-005）：5 域 Lead 必须**独立**配置，不可兼任。
> - 架构师**不**兼任 player 域 Lead
> - SRE**不**兼任 admin 域 Lead
> - Q-003 跨域核心问题需 Economy 域 Lead 独立决策权
> - COC 控制面属 admin 域独立控制面

## §2 5 域 + 3 配套 Lead 配置表

| # | 域 / 域簇 | 角色 | 状态 | 到位后职责 |
|---|---|---|---|---|
| 1 | **Player 域** | Player 域 Lead | ⏳ 待具名 | 256 L4 任务补全 + 域签字 |
| 2 | **Economy 域** | Economy 域 Lead | ⏳ 待具名 | 256 L4 任务 + Q-003 二次确认 |
| 3 | **Match 域** | Match 域 Lead | ⏳ 待具名 | 256 L4 任务 + NFR-PT 100ms |
| 4 | **Social 域** | Social 域 Lead | ⏳ 待具名 | 256 L4 任务 + 异步路径 |
| 5 | **Admin / COC 域** | Admin 域 Lead | ⏳ 待具名 | 256 L4 任务 + RBAC + 审计 |
| 6 | **cluster-ops 域** | cluster-ops 域 Lead | ⏳ 待具名 | 256 L4 任务 + ClusterOpsService |
| 7 | **foundation** | 架构师（兼） | ✅ 架构师（Ulysses）| 64 L4 任务 |
| 8 | **shared-platform** | Platform Engineer | ⏳ 待具名 | 96 L4 任务 + 镜像构建 |

## §3 5 域 Lead 招聘 / 分配 Checklist

| 步骤 | 内容 | 责任方 | 截止 | 状态 |
|---|---|---|---|---|
| 1 | Player 域 Lead 候选人确定 | Ulysses（PM）| PH-0 末 | ⏳ |
| 2 | Economy 域 Lead 候选人确定（Q-003 决策权）| Ulysses（PM）| PH-0 末 | ⏳ |
| 3 | Match 域 Lead 候选人确定（Gameplay Engineer）| Ulysses（PM）| PH-0 末 | ⏳ |
| 4 | Social 域 Lead 候选人确定（Messaging Engineer）| Ulysses（PM）| PH-0 末 | ⏳ |
| 5 | Admin 域 Lead 候选人确定（COC 控制面）| Ulysses（PM）| PH-0 末 | ⏳ |
| 6 | cluster-ops 域 Lead 候选人确定（独立，非 SRE 兼任）| Ulysses（PM）| PH-0 末 | ⏳ |
| 7 | Platform Engineer 候选人确定 | Ulysses（PM）| PH-0 末 | ⏳ |
| 8 | DBA Lead 候选人确定 | Ulysses（PM）| PH-0 末 | ⏳ |
| 9 | SRE Lead 候选人确定 | Ulysses（PM）| PH-0 末 | ⏳ |
| 10 | QA Lead 候选人确定 | Ulysses（PM）| PH-0 末 | ⏳ |
| 11 | 业务方代表确定 | Ulysses（PM）| PH-0 末 | ⏳ |

## §4 5 域 Lead 到位后立即触发的工作

1. **L4 任务补全**（per RGS-WBS-001 v0.2 §4.3）：每域 Lead 补 256 L4 任务的 6 字段
2. **RGS-ENV-001 v0.3 §6 签字**：5 域 Lead 各自签 §1-§5 域相关核验
3. **RGS-REV-003 §7.3 签字**：5 域 Lead 各自签 G-CODE-02 + G-CODE-05
4. **RGS-EXEC-001 v0.3 签字**：5 域 Lead 签 §2.4 / §3.4 / §4.4 域相关栏位
5. **所有者背书解除**（per §3.4.4.3）：RGS-PLAN-001 升 v0.8 移除"所有者背书"占位

## §5 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。5 域 + 3 配套 Lead 到位 checklist + 招聘/分配 11 步。 |
