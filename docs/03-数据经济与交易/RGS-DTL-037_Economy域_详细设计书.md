# 详细设计书（詳細設計書 / Detailed Design Document）

**Economy 域 Atomic App 契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-037 |
| 版本 | 0.1 |
| 状态 | **契约骨架・待评审・Q-003 未批准前禁止跨 DB 实施** |
| 父文档 | RGS-REQ-011、RGS-BAS-007、RGS-ADR-0015、RGS-DTL-031 |
| App/DB | `economy-service` / `economy_db` |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 修订历史 | 0.1（2026-08-21）：建立 Economy 域契约骨架 |
| 审批 | 未审批；Q-003 与字段级 DD Review 前不得作为实施授权 |

## 1. 领域职责与非职责

- 负责货币、道具、购买/交易账本和 Economy 永久事实。
- `CommitTransaction` 是产生货币/道具永久事实的唯一宿主入口。
- 不直接写 `player_db`、`match_db`、`social_db` 或 `admin_db`。

## 2. 集群契约

```yaml
app_id: economy-service
db: economy_db
depends_on: [player-service, event-bus, config, observability, secrets]
scaffold_ref: services/economy-service/deploy/helm
feature_host: true
health: [/healthz, /readyz]
```

## 3. API 与事件骨架

| 类型 | 契约 | 规则 |
|---|---|---|
| gRPC | `CommitTransaction`、`GetBalance`、`GetInventory` | 单 DB 本地事务；必须有 `request_id`、幂等键和 session epoch |
| Event | `EconomyTransactionCommitted`、`CompensationRequired` | Outbox 发布；事件只表达已提交事实 |
| Compensation | Saga 反向操作候选接口 | Q-003 审批前只定义契约，不实现跨 DB 编排 |

## 4. Q-003 与插件边界

跨 DB 购买、转账、跨域奖励采用“每库本地事务 + Saga + Outbox”候选方案，补偿失败进入人工对账；最终补偿上限和升级路径待 Q-003 具名审批。经济插件只能通过 `CommitTransaction` 白名单 API，禁止脚本/插件直写表。

## 5. 迁移、回滚与测试

- 账本和库存写入必须可审计、幂等、可重放校验。
- 业务回滚不删除已提交账本；使用补偿交易和审计关联。
- 必须覆盖：重复请求、余额 OCC、Outbox 重放、补偿失败、插件越权、Q-003 三个真实跨 DB 场景。

## 6. 待补齐项

- [ ] 账本/库存/订单物理 DDL 与分区策略。
- [ ] Q-003 审批材料和补偿延迟 p99 指标。
- [ ] 跨域 ID 与 event schema 权威清单。
- [ ] Economy 与 Player/Match/Social 的契约测试。
