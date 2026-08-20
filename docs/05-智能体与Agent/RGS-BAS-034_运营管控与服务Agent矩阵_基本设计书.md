# 基本设计书（基本設計書 / Basic Design Document）

**运营管控与服务 Agent 矩阵 — Operations & Service Agent Matrix**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-034 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-034 需求定义书、RGS-BAS-033 Agent平台底座 |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-20 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

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
