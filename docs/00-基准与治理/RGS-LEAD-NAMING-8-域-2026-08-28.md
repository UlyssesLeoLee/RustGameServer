# 8 域 Lead 具名草案(per Q2 OPEN-QA)

> **目的**:为 8 域 + cluster-ops + gm-backend + 工具集 配置独立 Lead 实名(per DEC-005 兼任拒绝原则)
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:30 JST)
> **状态**:🟡 OPEN — 待 Ulysses 终审(一人公司 12 角色,决策权在 Ulysses)
> **关联**:Q2 OPEN-QA(per RGS-OPEN-QA-2026-08-27-k3s-deploy §Q2)+ DEC-005(5 域独立 Lead 拒绝兼任)+ DEC-008(一人公司 12 角色)

---

## 0. 背景与约束

### 0.1 兼任拒绝(per DEC-005,2026-08-21)

Ulysses 在 2026-08-21 强证据(per user_profile):
- 5 域 + cluster-ops + shared-platform 等多域架构,每域配独立 Lead,拒绝兼任
- 理由:兼任会把责任矩阵和 RACI 模糊化
- 适用:5 域及以上分布式系统架构设计、Q-031 WBS 资源估算、OLU 预算重算

### 0.2 一人公司 12 角色(per DEC-008)

Ulysses 一人公司模式 12 角色,本表"8 域 Lead 具名"采用以下映射规则:
- 每个 Lead 是 1 个**角色**(非真人),由 Ulysses 代签
- 角色之间**独立**(per DEC-005),不兼任
- 代签透明:author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008)+ 自审 + 日期

### 0.3 当前状态(per OPEN-QA Q2)

- RACI 5 份文档(per 5 域)在 `docs/14-项目管理/RGS-RACI-{PLAYER,ECONOMY,MATCH,SOCIAL,ADMIN}-V1_*.md`
- 当前签字人:"架构师(Mavis 接手 agent per DEC-008)"代签,不是真实 5 个 Lead 实际签字
- OLU 报告 §6.5 评估人·天 21 略超 NFR-OP-010 20 上限(因兼任)

## 1. 8 域 Lead 具名草案

### 1.1 5 域(per 5 域 DTL)

| # | 域 | 角色(per DEC-008 一人公司) | 负责范围 | RACI 关联 |
|---|---|---|---|---|
| 1 | **player-service** | 玩家域 Lead | DTL-015 (玩家域 详细设计) + 5 域 4 域对称骨架 | RGS-RACI-PLAYER-V1 |
| 2 | **economy-service** | 经济域 Lead | DTL-018/037 (经济域) + OCC + outbox + saga 编排 | RGS-RACI-ECONOMY-V1 |
| 3 | **match-service** | 对战域 Lead | DTL-026/038 (Match 域) + 房间 + 撮合 + 扩圈算法 + 跨分片 OCC | RGS-RACI-MATCH-V1 |
| 4 | **social-service** | 社交域 Lead | DTL-019/020/039 (社交+聊天+消息分发) + 好友 + 推送 | RGS-RACI-SOCIAL-V1 |
| 5 | **admin-service** | Admin 域 Lead | DTL-031 (COC 集群运营中心) + DTL-003 协议 + PFAU 7 阶段 | RGS-RACI-ADMIN-V1 |

### 1.2 跨域编排(per 6 域)

| # | 域 | 角色 | 负责范围 |
|---|---|---|---|
| 6 | **cluster-ops** | 集群运营 Lead | DTL-042 (集群全生命周期管理) + realm_lifecycle 6 阶段 + Drill 演练 + 跨域编排 |

### 1.3 第 8 域(per 2026-08-27 Ulysses 指令)

| # | 域 | 角色 | 负责范围 |
|---|---|---|---|
| 7 | **gm-backend** | GM 后台域 Lead | BAS-003 (运维与 GM 后台管控) + DTL-040 (Admin 域契约骨架) + 5 GM endpoint + JWT + audit_log + mTLS |

### 1.4 工具集(per 09 编号域)

| # | 域 | 角色 | 负责范围 |
|---|---|---|---|
| 8 | **rgs-certgen** (工具集) | 工具链 Lead | RGS-IMPL-001 §4 工具链 + 后续 rgs-archive-tool 等工具类 crate |

### 1.5 共享支持(per Open-QA Q2 待具名)

| 角色 | 负责范围 | 当前代签 |
|---|---|---|
| SRE Lead | 部署 + 监控 + oncall | Ulysses / Mavis |
| Platform Lead | 共享内核 (shared-platform) + TLS 集成 | Ulysses / Mavis |
| QA Lead | UT/IT/ST 文档 + 测试覆盖率 | Ulysses / Mavis |
| 评审主持人 (PM) | DDD Review 主持 + 跨域协调 | Ulysses / Mavis |
| DBA | 5 域 + cluster-ops + admin_db schema 管理 | Ulysses / Mavis |
| 架构师 | 5 域 + cluster-ops 整体架构 + DDD 主持 | Ulysses / Mavis (per DEC-008) |

## 2. 一人公司 12 角色 ↔ 8 域 Lead 映射

| 1 人公司 12 角色 | 8 域 + 共享支持 |
|---|---|
| 1. 架构师 | 整体架构 (非域 Lead)|
| 2. PM | 评审主持人 |
| 3. DBA | DB schema 治理 |
| 4. 5 域 Lead (×5)| player / economy / match / social / admin |
| 5. cluster-ops Lead | cluster-ops |
| 6. GM 后台 Lead | gm-backend |
| 7. SRE Lead | 部署 + 监控 |
| 8. Platform Lead | shared-platform |
| 9. QA Lead | 测试设计 + 覆盖率 |
| 10. 工具链 Lead | 09 工具集 |
| 11. 业务方代表 | (RGS-OPEN-QA Q6 代签边界) |
| 12. 变更控制委员会 | DDD Review |

## 3. 代签透明(per 2026-08-27 19:39/20:56/21:59 JST 三次强化)

每域 Lead 文档签字格式(per DEC-008 + 19:39 三次强化):
- author = Ulysses(一人公司 12 角色)
- 审批 = 架构师(Mavis 接手 agent per DEC-008) + 自审 + 日期
- 修订人 = Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手

**非代签(仍 ⏳)**:
- SRE Lead / Platform Lead / 评审 / PM 真实具名(per 8/27 21:59 JST 决议)

## 4. 阻塞影响(per Q2 §阻塞影响)

### 4.1 OLU 报告 §6.5 评估人·天 21 略超 NFR-OP-010 20

- 原因:5 域 Lead 兼任导致维护工作量叠加
- 缓解(本具名):8 域独立 Lead,工作量可分配到各域

### 4.2 RACI v1.1 → v1.2 升级

- 待 8 域 Lead 具名后,5 份 RACI 升级到 v1.2
- v1.2 字段:实际具名 + 代签范围(per Q2 决议 §决策项 3)
- 8 域独立 RACI 文档新增:cluster-ops / gm-backend / 工具集 + 共享支持角色

## 5. 决策项

| 决策点 | 选项 | 推荐 |
|---|---|---|
| 8 域 Lead 角色映射 | 上表 §1 | **采纳** |
| 共享支持角色 | 上表 §1.5 | **采纳** |
| RACI v1.2 升级 | 立即 / DDD Review 阶段 | **DDD Review 阶段**(per Q2 决议)|
| 一人公司 12 角色 vs 8 域 Lead 数量 | 12 角色 vs 8 域 + 共享 4 | **合理**(共享支持 + 8 域) |

## 6. 待 Ulysses 终审

- [ ] 是否同意 §1 8 域 Lead 角色映射
- [ ] 是否同意 §1.5 共享支持角色
- [ ] RACI v1.2 升级窗口(DDD Review 阶段 vs 立即)
- [ ] 12 角色 vs 8 域 数量是否合理

## 7. 关闭条件(per Q2)

8 域 Lead 实际具名 + RACI v1.2 升级 + OLU 报告 §6.5 重算(预计人·天 21 → 16-18 per 8 域分配)→ Q2 可关闭。

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:30 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
