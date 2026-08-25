# 基本设计书（基本設計書 / Basic Design Document）

**运营管控与服务 Agent 矩阵 — Operations & Service Agent Matrix**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-034 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-034 v0.2 需求定义书、RGS-BAS-033 v0.2 Agent平台底座 |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-25 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-20 | 架构师 | — | 初版制定。落实 RGS-REQ-034 全部 BR-AGO-001~003 / FR-AGO-001~005 / NFR-AGO-001~003 + AC-AGO-001~004；包含运营管控 + 服务 Agent 矩阵的协同设计（基于 BAS-033 平台层 + 复用 BAS-032 双 Agent 体系）；SRE 运维自愈 Agent（告警聚类 + RCA + Quarantine）+ 7x24h 智能客服 Agent（工单对账 + 小额补偿 Intent）+ GM 风控合规审查 Agent（异常行为画像 + 证据链打包），全部受 L0 Action Gate 强校验。 | RGS-REQ-034 v0.1 |
| 0.2 | 2026-08-25 | 架构师 | — | **同步 REQ 升版**：父文档 RGS-REQ-034 v0.1 → v0.2（per `RGS-DOCS-HEALTH-2026-08-25` 接手 agent 2026-08-25 调查 / 用户 2026-08-25 22:35 JST 指令"依照最新版需求文档更新基本设计文档"）。**正文功能层无变化**——REQ v0.2 修订内容为"补齐与附件 C 已登记 ID 一致的 NFR、验收标准、测试映射及未决/风险项；ARC-055 仅为待具名人类审批的提案"，BAS 运营管控矩阵（3 类 Agent + L0 Action Gate）已落实该约束。**审批栏状态**：本 v0.2 升版**仅为草案**（per DEC-008 治理基线 + `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态,非文档缺陷,agent 不可代签"），未填写 12 角色签字栏——签字动作需 Ulysses 本人在场执行（per ADR-0058 §6 已留出空签字栏，候选提案正文已由 Mavis 起草 per `RGS-DOCS-HEALTH-2026-08-25 §4` 反馈单 commit `7ce0c67`）。 | RGS-REQ-034 v0.2 |

> **签字状态说明**：本 BAS-034 v0.2 升版**未签字**。ARC-055 待具名人类审批（per `RGS-DOCS-HEALTH-2026-08-25 §4` 反馈单 issue #13 跟踪）尚未完成，故只做版本对齐。**Mavis 不代签**（per DEC-008）。

---

## 1. 运营与服务 Agent 拓扑与协同流

```mermaid
sequenceDiagram
    autonumber
    actor Player as 玩家
    participant CS as 客服 Agent
    participant SRE as SRE 运维 Agent
    participant Gate as L0 确定性闸门
    participant Core as Rust 核心系统

    Player->>CS: 提交工单 (充值 648 未到账)
    CS->>Core: 只读查询三方支付流水 & Outbox
    Core-->>CS: 返回: 支付已成功，但网关下发超时
    CS->>CS: 评估补偿额度 (在单人白名单限额内)
    CS->>Gate: 提交 IssueCompensationIntent (带Ed25519签名)
    Gate->>Gate: 校验签名、单日配额、订单防重放
    Gate->>Core: 原子加款并记录审计
    Core-->>Gate: 加款成功 Receipt
    Gate-->>CS: 确认执行
    CS-->>Player: 秒级答复: 资产已补发到游戏邮箱，附致歉礼包
```
