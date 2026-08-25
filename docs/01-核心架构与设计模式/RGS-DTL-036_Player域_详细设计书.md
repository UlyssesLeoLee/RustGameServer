# 详细设计书（詳細設計書 / Detailed Design Document）

**Player 域 Atomic App 契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-036 |
| 版本 | 1.4 |
| 状态 | **契约骨架・待评审・不得作为实施授权** |
| 父文档 | RGS-REQ-001、RGS-BAS-001（v1.4）、RGS-DTL-031 |
| App/DB | `player-service` / `player_db` |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 修订历史 | 见文末"修订历史"小节 |
| 审批 | 未审批；字段级 DD Review 前不得作为实施授权 |

> 本文档只冻结 Player 域与五域集群的边界契约；物理 DDL、字段级 IDL 和容量参数须在 Gate 通过后补齐。版本号 1.4 表示对父 BAS-001 v1.4（权威源分级 / Tier-1・Tier-2，per RGS-ADR-0057）的同步对齐，但本文档仍为契约骨架，不是完整详细设计——完整物理设计属于 DTL-001。

## 1. 领域职责与非职责

- 负责账号、角色、会话 epoch、玩家状态和玩家侧查询。
- 不直接写 `economy_db`、`match_db`、`social_db` 或 `admin_db`。
- 永久事实变更经 Player 本地事务和 Outbox；跨域使用已发布 gRPC/event contract。
- **`player_db` 持久化语义遵循 BAS-001 §5.3 聚合根/聚合边界**：`Account` 为聚合根，`Character` 虽独立生命周期但 sessionEpoch 修改权仅属自身（`issueSessionEpoch()`，ARC-005 对象设计落地）；账号封禁以 `Account` 为唯一入口（不存在角色级封禁）。
- **`player_db` 不直接落在 BAS-001 §5.4.3 的 Tier-1 / Tier-2 权威源分级框架内**：该框架作用域为 SceneActor 内存持有的角色状态字段（Tier-1 = `economy_db` 强一致不可逆资产；Tier-2 = SceneActor 权威的过程态），业务域持久化（`player_db` / `match_db` / `social_db` / `admin_db`）由各自聚合根按本地事务语义维护，不复用该分级标签。本节升版仅同步 BAS-001 v1.4 的边界澄清，不将 player_db 字段标 Tier-X。

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

**背压与死锁防止引用**：player-service 对 `event-bus` 的订阅/消费与对其他域的同步调用，背压配置（队列上限、降级策略、优先级通道划分）须遵循 **BAS-001 §7.2.1（ARC-013 完整落地）**。Player 域不引入新背压点；方向性分类（东西向调用是否构成循环）的判定与证明由 BAS-001 §7.2.1 集中维护，Player 仅在新增跨域同步通道时回查该节。

## 3. API 与事件骨架

| 类型 | 契约 | 规则 |
|---|---|---|
| gRPC | `GetPlayer`、`CreatePlayer`、`UpdatePlayerState` | 业务写入带 `request_id` 和 OCC `expected_version` |
| Event | `PlayerRegistered`、`PlayerStateChanged`、`SessionEpochIssued` | Outbox 发布；消费者按 event_id 幂等 |
| Health | `/healthz`、`/readyz` | Readiness 不等于业务事务成功 |

**字段级契约引用**：上表仅列方法名 / 事件名（per BAS-001 v1.1 接口目录升版前的形态）。字段级契约（gRPC 请求/响应字段、事件 payload 字段、错误枚举、兼容窗口）遵循 **BAS-001 §6（外部接口设计，含 §6.3 gRPC 方法与字段级设计）**。Player 域字段级 IDL 的具体落地（DTL 物理层）在 `crates/contracts/player.proto` + DDD Review 阶段产出，本文档保持契约骨架不展开字段——避免与 BAS-001 §6 重复定义、也避免在证据不足时细化（ARC-014）。

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

---

## 7. 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 修订内容 | 影响章节 |
|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 初版制定：建立 Player 域 Atomic App 契约骨架（领域职责、集群契约、API/事件骨架、插件边界、迁移回滚、待补齐项） | 全部 §1〜§6 |
| 1.4 | 2026-08-26 | 架构师 | 同步父 BAS-001 升版（v1.0〜v1.4）：§1 补 `player_db` 持久化语义遵循 BAS-001 §5.3 聚合根/聚合边界，并澄清 §5.4.3 权威源分级（Tier-1/Tier-2）作用域不覆盖业务域持久化；§2 补 player-service 背压配置遵循 BAS-001 §7.2.1（ARC-013 死锁防止）引用；§3 补字段级契约遵循 BAS-001 §6 引用，明确字段级 IDL 在 `crates/contracts/player.proto` + DDD Review 阶段产出；头部版本 0.1 → 1.4 对齐父 BAS 最终版本；新增§7 修订历史、§8 审批栏。本文档仍为契约骨架，未膨胀为完整详细设计（属 DTL-001 职责） | §1、§2、§3、头部元数据、§7、§8 |

> **不可代签声明**：本表"修订者"列为本次升版的实际执行人。审批栏（§8）须由对应评审/审批角色在字段级 DD Review 之后补签，本升版不代签任何审批。

## 8. 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | — | — | 待补：v0.1（2026-08-21）起草与 v1.4（2026-08-26）升版的签署 |
| 评审（技术） | — | — | 字段级 DD Review 后填写（与 BAS-001 v1.4 一致性、§6 字段级契约是否完备、§7.2.1 引用是否准确） |
| 评审（业务） | — | — | 是否遗漏账号/角色/会话 epoch 业务规则（与 REQ-001 FR-PL-nnn 对账） |
| 审批（负责人） | — | — | Player 域契约骨架的基准化 |

> 本文档经审批后仅作为 Player 域实施（PH-1）的契约输入。物理 DDL、字段级 IDL、容量参数、契约测试夹具仍按 §6 待补齐项在 DDD Review 阶段产出。


