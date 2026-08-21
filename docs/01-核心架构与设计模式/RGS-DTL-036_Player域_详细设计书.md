# 详细设计书（詳細設計書 / Detailed Design Document）

**Player 域 Atomic App 契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-036 |
| 版本 | 0.1 |
| 状态 | **契约骨架・待评审・不得作为实施授权** |
| 父文档 | RGS-REQ-001、RGS-BAS-001、RGS-DTL-031 |
| App/DB | `player-service` / `player_db` |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 修订历史 | 0.1（2026-08-21）：建立 Player 域契约骨架 |
| 审批 | 未审批；字段级 DD Review 前不得作为实施授权 |

> 本文档只冻结 Player 域与五域集群的边界契约；物理 DDL、字段级 IDL 和容量参数须在 Gate 通过后补齐。

## 1. 领域职责与非职责

- 负责账号、角色、会话 epoch、玩家状态和玩家侧查询。
- 不直接写 `economy_db`、`match_db`、`social_db` 或 `admin_db`。
- 永久事实变更经 Player 本地事务和 Outbox；跨域使用已发布 gRPC/event contract。

## 2. 集群契约

```yaml
app_id: player-service
db: player_db
depends_on: [gateway, event-bus, config, observability, secrets]
scaffold_ref: services/player-service/deploy/helm
feature_host: true
health: [/healthz, /readyz]
```

Player 是五域中第一条业务纵向切片，但其依赖必须从集群 manifest 读取，不得在代码中硬编码部署顺序。

## 3. API 与事件骨架

| 类型 | 契约 | 规则 |
|---|---|---|
| gRPC | `GetPlayer`、`CreatePlayer`、`UpdatePlayerState` | 业务写入带 `request_id` 和 OCC `expected_version` |
| Event | `PlayerRegistered`、`PlayerStateChanged`、`SessionEpochIssued` | Outbox 发布；消费者按 event_id 幂等 |
| Health | `/healthz`、`/readyz` | Readiness 不等于业务事务成功 |

## 4. 插件边界

插件只能作为 Player 宿主 Feature 注册，使用白名单 API 和 tick/request 边界切换；不得访问 DB 连接、文件系统、网络或其他域客户端。插件异常必须被捕获、熔断并记录审计。

## 5. 迁移、回滚与测试

- `player_db` migration 遵循 Expand-Contract，向前迁移和回滚演练均由 CI 执行。
- App 回滚不回滚已发布永久事实；版本恢复通过兼容 API、事件和显式数据迁移处理。
- 必须覆盖：OCC 冲突、重复 request、Outbox 重放、会话 epoch 过期、插件异常和独立 NetworkPolicy。

## 6. 待补齐项

- [ ] 账号/角色/会话物理 DDL 与索引。
- [ ] `.proto` 字段号、错误枚举和兼容窗口。
- [ ] Player 与 Economy/Match/Social 的字段权威清单。
- [ ] 第一条端到端路径的契约测试夹具（`crates/testkit`）。
