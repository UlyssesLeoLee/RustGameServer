# 详细设计书（詳細設計書 / Detailed Design Document）

**Player 域 Atomic App 契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-036 |
| 版本 | 1.4.1 |
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
| gRPC | `GetPlayer`、`CreatePlayer`、`UpdatePlayerState`（**待 DDD Review 阶段用 BAS-001 §6.3.1 PlayerService 现有方法名重写**：`Authenticate` / `SelectCharacter` / `GetCharacterList` 等；DTL-036 v1.4 此处方法名为占位，**非父 BAS 升版基线**） | 业务写入带 `request_id`、OCC `expected_version`、**`session_epoch`**（per BAS-001 §6.1 ARC-005："凡受 Single-Writer 保护的方法，请求必须携带 session_epoch"）。`session_epoch` 由本域 `issueSessionEpoch()` 签发并在每次单写者切换时强制刷新（详见 §1） |
| Event | `PlayerRegistered`、`PlayerStateChanged`、`SessionEpochIssued` | Outbox 发布；消费者按 `event_id` 幂等。`SessionEpochIssued` 事件必须含新 epoch + 旧 epoch 范围（FR-PL-002 关联） |
| Health | `/healthz`、`/readyz` | Readiness 不等于业务事务成功 |

**字段级契约引用**：**§3 表格方法名 / 事件名均为占位，需在 DDD Review 阶段与 BAS-001 §6.3.1 PlayerService、REQ-001 §FR-PL-001〜006 业务规则逐条对账后重写**。本次 v1.4 升版仅澄清"§3 表格与父文档现状**未对齐**"——这是 v1.4 的已知缺口，**不是 v1.4 已经与父文档对齐**。字段级契约（gRPC 请求/响应字段、事件 payload 字段、错误枚举、兼容窗口）最终遵循 **BAS-001 §6（外部接口设计，含 §6.3 gRPC 方法与字段级设计）**。Player 域字段级 IDL 的具体落地（DTL 物理层）在 `crates/contracts/player.proto` + DDD Review 阶段产出，本文档保持契约骨架不展开字段——避免与 BAS-001 §6 重复定义、也避免在证据不足时细化（ARC-014）。

**§3 已知缺口**（DDD Review 阶段必查项）：
- gRPC 方法名与 BAS-001 §6.3.1 PlayerService 现有方法名对账（当前是 `GetPlayer`/`CreatePlayer`/`UpdatePlayerState`，父 BAS 是 `Authenticate`/`SelectCharacter`/`GetCharacterList`，**两者不一致**）
- 与 REQ-001 §FR-PL-004（玩家永久状态读写，PH-1 ◎）、FR-PL-005（封禁/制裁）、FR-PL-006（在线状态）三条业务规则对账（**当前 §3 未覆盖**，见 §8 评审（业务）栏备注）
- `session_epoch` 必填规则的伪代码级强制（`unary.rs` 中间件层校验），与 BAS-001 §6.1 + ARC-005 一致性
- `PlayerRegistered` 事件应包含 `player_id` / `account_id` / `initial_session_epoch` 三字段（per FR-PL-001）
- 错误枚举（`StaleSessionEpoch` / `ExpectedVersionMismatch` / `PlayerNotFound` 等）由 DDD Review 阶段定义

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
| 1.4.1 | 2026-08-26 | 架构师 | **hotfix**：用户 review 反馈指出 v1.4 升版存在 3 项治理基线违规，立即撤回并修正：（1）§3 第 50 行原文 "per BAS-001 v1.1 接口目录升版前的形态" 属**伪造出处**——`GetPlayer`/`CreatePlayer`/`UpdatePlayerState` 这三个方法名在 BAS-001 全部 git 历史中 0 次出现，父 BAS 自始是 `Authenticate`/`SelectCharacter`/`GetCharacterList`（per `git log --all -p --follow RGS-BAS-001_基本设计书.md` 实证）。已替换为"占位 + 显式声明 §3 与父 BAS 现状未对齐"诚实表述；（2）§3 规则列漏 `session_epoch`，违反 BAS-001 §6.1 ARC-005 强制要求（"凡受 Single-Writer 保护的方法，请求必须携带 session_epoch"），已补回；（3）§3 表格方法名/事件名与 REQ-001 §FR-PL-004/005/006 三条业务规则未对账（§8 评审（业务）栏自身备注"待对账"），已显式列入 §3 末尾"已知缺口"清单。**触发根因复盘**：v1.4 升版在 worker 授权范围"不引入新设计、只引用 BAS 已确定内容"下，§3 引用处"per BAS-001 v1.1 接口目录升版前的形态"是 worker 在 BAS 升版脉络未充分求证时编造的回溯叙事，违反 DEC-008 真实性原则。**修正式**: 升版一律禁止使用"per X 历史形态""per X 升版前/后"这类**无 git 历史证据**的回溯叙事，统一改为"待 DDD Review 与父文档 X.Y §Z 对齐"的诚实缺标。| §3（修正引用+补规则列+列已知缺口） |

> **不可代签声明**：本表"修订者"列为本次升版的实际执行人。审批栏（§8）须由对应评审/审批角色在字段级 DD Review 之后补签，本升版不代签任何审批。

## 8. 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | — | — | 待补：v0.1（2026-08-21）起草与 v1.4（2026-08-26）升版的签署 |
| 评审（技术） | — | — | 字段级 DD Review 后填写（与 BAS-001 v1.4 一致性、§6 字段级契约是否完备、§7.2.1 引用是否准确） |
| 评审（业务） | — | — | **DDD Review 必查**: ① §3 gRPC 方法名与 BAS-001 §6.3.1 对账（v1.4.1 已显式标缺口）；② 与 REQ-001 §FR-PL-004（玩家永久状态读写, PH-1 ◎）/ FR-PL-005（封禁/制裁）/ FR-PL-006（在线状态）三条业务规则逐条对账（v1.4.1 已显式标缺口） |
| 审批（负责人） | — | — | Player 域契约骨架的基准化 |

> 本文档经审批后仅作为 Player 域实施（PH-1）的契约输入。物理 DDL、字段级 IDL、容量参数、契约测试夹具仍按 §6 待补齐项在 DDD Review 阶段产出。


