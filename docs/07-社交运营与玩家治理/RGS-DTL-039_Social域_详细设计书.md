# 详细设计书（詳細設計書 / Detailed Design Document）

**Social 域 Atomic App 契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-039 |
| 版本 | 0.1 |
| 状态 | **契约骨架・待评审・不得作为实施授权** |
| 父文档 | RGS-REQ-017、RGS-BAS-013、RGS-DTL-013、RGS-DTL-031 |
| App/DB | `social-service` / `social_db` |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 修订历史 | 0.1（2026-08-21）：建立 Social 域契约骨架 |
| 审批 | 未审批；字段级 DD Review 前不得作为实施授权 |

## 1. 领域职责与非职责

- 负责社交关系、队伍/公会、消息、举报与玩家治理流程。
- 只消费 Player 公开契约，不直接访问 `player_db`；经济事实只经 Economy API/event。
- 治理操作必须有 RBAC、审计和可回滚的状态迁移。

## 2. 集群契约

```yaml
app_id: social-service
db: social_db
depends_on: [player-service, event-bus, config, observability, secrets]
scaffold_ref: services/social-service/deploy/helm
feature_host: true
health: [/healthz, /readyz]
```

## 3. API 与事件骨架

| 类型 | 契约 | 规则 |
|---|---|---|
| gRPC | `CreateRelationship`、`UpdateTeam`、`SubmitReport`、`GetSocialState` | request_id、RBAC、OCC |
| Event | `RelationshipChanged`、`TeamChanged`、`ReportSubmitted` | Outbox 发布；敏感数据按日志/事件规范脱敏 |
| Admin | `ModerateReport`、`DisableSocialFeature` | 经 AdminService 统一入口并落审计 |

## 4. 插件边界

活动、治理规则和消息过滤可作为编译期 Feature 或受限沙箱脚本；白名单 API 不得产生绕过 Economy `CommitTransaction` 的永久事实。脚本只能在安全点热重载，超时/异常进入熔断。

## 5. 迁移、回滚与测试

- 关系和治理状态使用显式状态机；删除/禁用不得绕过审计。
- 内容/规则版本回滚不删除举报、审计或已产生的治理事实。
- 必须覆盖：RBAC 越权、事件重放、脚本资源超限、敏感日志脱敏、NetworkPolicy 和 PFAU 回滚。

## 6. 待补齐项

- [ ] 关系、队伍、举报和消息物理 DDL。
- [ ] 与 Player/Economy/Admin 的事件 schema 和数据最小化规则。
- [ ] 沙箱白名单 API 与规则版本 fencing。
- [ ] Social 集成测试和故障注入矩阵。
