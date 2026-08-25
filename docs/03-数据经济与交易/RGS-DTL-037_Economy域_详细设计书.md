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
| 修订历史 | 0.1（2026-08-21）：建立 Economy 域契约骨架；0.2（2026-08-25）：新增 §7 economy_db 现行 DDL 反向登记（`accounts`/`transaction_ledger`），偿还附件D ISS-128/TBD-111 技术债（per RGS-OPEN-QA-001 v0.2 + ACTIONS-v0.3 §3 A-02 偿还模式，仿 RGS-DTL-044 反向文档先例） |
| 审批 | 未审批；Q-003 与字段级 DD Review 前不得作为实施授权（**§7 例外**：§7 仅登记 economy_db 单库内已实施 DDL 的事实现状，不涉及 Q-003 所辖跨 DB 编排，不受本审批门禁约束） |

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

- [x] 账本物理 DDL：`accounts`/`transaction_ledger` 已反向登记于 §7（per ISS-128/TBD-111，2026-08-25）。
- [ ] 库存物理 DDL：`inventory_items` 在 economy-service **未实现**（见 §7.4 缺口说明，非已登记状态）；订单物理 DDL 与分区策略**仍未补齐**。
- [ ] Q-003 审批材料和补偿延迟 p99 指标。
- [ ] 跨域 ID 与 event schema 权威清单。
- [ ] Economy 与 Player/Match/Social 的契约测试。

---

## 7. economy_db 现行 DDL 反向登记（`accounts` / `transaction_ledger`）

> **本节偿还附件D ISS-128/TBD-111 技术债**：RGS-DTL-001 §3.1（2026-08-17 v0.1 起）定义的 `wallets`／`inventory_items`／`transaction_ledger`（角色级 `character_id` 主键、`request_id`+`operation`+`payload` 账本字段）是初版"应然"设计，**从未实施**。`crates/economy-service/migrations/0001_init.sql`（per WBS v0.3 §2A.5 WF-1-54.6）实际落地为下述 `accounts`／`transaction_ledger`（玩家级 `player_id`+币种行、`idempotency_key`+`kind`+`amount` 账本字段），且**未实现** `inventory_items`。
>
> **本节仿 RGS-DTL-044（player_db 反向文档先例）**，以代码为现行基线登记字段级 DDL；**不修改** `crates/economy-service/migrations/` 任何既有 migration，**不**替代人类对"是否将代码迁回 DTL-001 §3.1 设计"的决策权——本节只登记**已选定方案 (a)**（沿 DTL-044 模式，以代码为基线）的既成事实，方案 (a)/(b) 的抉择本身已由项目负责人于 2026-08-25 拍板"按推荐（方案 a）处理"。

### 7.1 `accounts` 表（per `0001_init.sql`）

> **状态**：✅ **已存在**（`crates/economy-service/migrations/0001_init.sql`）

```sql
CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY,
    player_id UUID NOT NULL,
    currency TEXT NOT NULL CHECK (currency IN ('gold', 'diamond', 'token')),
    balance BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    version BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'frozen', 'closed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (player_id, currency)
);

CREATE INDEX IF NOT EXISTS idx_accounts_player_id ON accounts (player_id);
CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts (status);
```

**字段语义**：

| 列 | 类型 | 约束 | 语义 | 与 DTL-001 §3.1 `wallets` 的差异 |
|---|---|---|---|---|
| `id` | UUID | PK | 账户行业务主键 | DTL-001 `wallets` 以 `character_id` 为 PK，无独立 `id` |
| `player_id` | UUID | NOT NULL | 所属玩家（**非** `character_id`）| 见 §7.3 分片键理由 |
| `currency` | TEXT | NOT NULL, enum('gold','diamond','token') | 币种，**以行区分**而非独立币种字段 | DTL-001 `wallets.balance` 单列即代表单一货币，多币种需多行 `wallets` 表（隐含）；`accounts` 显式 `UNIQUE(player_id, currency)` 使多币种模型显性化 |
| `balance` | BIGINT | NOT NULL DEFAULT 0, >= 0 | 余额 | 与 `wallets.balance` 语义一致 |
| `version` | BIGINT | NOT NULL DEFAULT 0 | OCC 乐观并发版本号 | 与 `wallets.version` 语义一致 |
| `status` | TEXT | NOT NULL DEFAULT 'active', enum | 账户状态（active/frozen/closed）| DTL-001 `wallets` 无此列 |
| `created_at`/`updated_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 标准审计时间戳 | 一致 |

**索引**：`idx_accounts_player_id`（FK 等效索引，按玩家查全部币种账户）、`idx_accounts_status`（风控/冻结账户筛选）、`UNIQUE(player_id, currency)`（隐式索引，防重复开户）。

### 7.2 `transaction_ledger` 表（per `0001_init.sql`；`saga_id`/`command_id` 字段随表一并在 0001 建立，`0002_saga_init.sql` 仅新增 `sagas`/`reservations`/`inbox` 三表，未 ALTER 本表）

> **状态**：✅ **已存在**

```sql
CREATE TABLE IF NOT EXISTS transaction_ledger (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL UNIQUE,
    saga_id UUID,
    command_id UUID,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL CHECK (currency IN ('gold', 'diamond', 'token')),
    kind TEXT NOT NULL CHECK (kind IN ('deposit', 'spend', 'transfer', 'refund', 'compensation')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'reversed', 'failed')),
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ledger_saga_id ON transaction_ledger (saga_id);
CREATE INDEX IF NOT EXISTS idx_ledger_account_id ON transaction_ledger (account_id);
CREATE INDEX IF NOT EXISTS idx_ledger_status ON transaction_ledger (status);
```

**字段语义与 DTL-001 §3.1 `transaction_ledger` 的差异**：

| 列 | 类型 | 约束 | 语义 | 与 DTL-001 §3.1 的差异 |
|---|---|---|---|---|
| `id` | UUID | PK | 账目行主键 | 一致 |
| `account_id` | UUID | NOT NULL FK→`accounts.id` ON DELETE CASCADE | 所属账户 | DTL-001 以 `character_id` 关联 `wallets`，本表以 `account_id` 关联 `accounts` |
| `idempotency_key` | TEXT | NOT NULL UNIQUE | 幂等键（per RGS-DTL-100 §6）| DTL-001 用 `request_id`；语义等价，命名不同 |
| `saga_id` / `command_id` | UUID | NULL | Saga 关联（per RGS-DTL-100 Saga 关键能力）| DTL-001 无此二列（v0.1 早于 Saga 设计定稿） |
| `amount` | BIGINT | NOT NULL | 变动金额 | 与 DTL-001 `payload` 内隐含金额字段等价，但**结构化为独立列**而非 JSONB |
| `currency` | TEXT | NOT NULL, enum | 币种 | DTL-001 无此列（v0.1 单币种假设） |
| `kind` | TEXT | NOT NULL, enum('deposit','spend','transfer','refund','compensation') | 交易类型 | DTL-001 用 `operation` TEXT 自由字段；本表**收窄为枚举** CHECK 约束 |
| `status` | TEXT | NOT NULL DEFAULT 'pending', enum | 账目状态（pending/confirmed/reversed/failed）| DTL-001 无此列 |
| `memo` | TEXT | NULL | 备注 | DTL-001 无此列 |
| `created_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 创建时间 | 一致 |
| **DTL-001 有而本表无** | — | — | `payload JSONB`（自由格式交易载荷，可携带道具明细等） | 见 §7.4 `inventory_items` 缺口说明——`payload` 在 DTL-001 设计中部分承担"货币+道具统一审计载荷"职责，本表**无对应列**，道具变动无法通过本表审计 |

**索引**：`idx_ledger_saga_id`（Saga 补偿查询）、`idx_ledger_account_id`（按账户查流水）、`idx_ledger_status`（对账/补偿失败筛选）、`UNIQUE(idempotency_key)`（隐式索引，幂等去重）。

### 7.3 为什么是 `player_id` 而非 `character_id`：分片路由约束

> **本节回应 ISS-128 论点中"角色级 vs 玩家级"分歧的技术侧论据**，不代替产品侧"多角色账号是否共享经济"的决策（见 §7.5）。

`accounts.player_id`（而非 DTL-001 `wallets.character_id`）并非随意实现偏差，而是与以下**已 Accepted 的集群路由约束**一致：

| 依据 | 内容 |
|---|---|
| RGS-REQ-025-ADD1 ARC-040-2 | "分片策略唯一采用 `jump_consistent_hash_v1(stable_hash_v1(player_id), active_shard_ids)`" —— 分片键**唯一**为 `player_id`，无 `character_id` 分片路径 |
| RGS-REQ-025-ADD1 FR-CAP-004 | 库内分片路由以 `stable_hash_v1(player_id)` 为输入，**禁止** `player_id % num_shards` 等其他路由方式 |
| RGS-REQ-025-ADD1 AC-CAP-101（lint 强制）| 路由入口仅调用 `jump_consistent_hash_v1`；lint 阻断直接以 `player_id % ...` 选 shard 或以 `character_id` 路由 |
| RGS-ADR-0057 §2.2（已 Accepted）| "接受 `player-service` 与 `economy-service` 按玩家ID一致性哈希同节点部署，避免玩法内跨节点 RPC" —— economy-service 的物理部署即以 `player_id` 为协同键 |

若 `accounts` 以 `character_id` 为路由/分片键，将与 economy-service 实际按 `player_id` 一致性哈希同节点部署的物理事实（ADR-0057 §2.2）及集群唯一分片算法（ARC-040-2）产生**路由键不一致**——账户查询需先由 `character_id` 反查 `player_id` 才能定位 shard，增加一次跨分片/跨节点查找，与 ADR-0057 §2.2 "避免跨节点 RPC" 的初衷相悖。**`accounts.player_id` 与集群分片路由键保持一致，是现行实现相对 DTL-001 `wallets.character_id` 设计的一个技术合理性优势**，而非纯粹的实现漂移。

**未完全否定角色级隔离的价值**：若未来产品侧确认需要角色级经济隔离（见 §7.5），可考虑 `UNIQUE(player_id, character_id, currency)` 复合键的增量扩展（仍以 `player_id` 为分片路由键，`character_id` 仅作账户内二级维度），无需推翻分片路由约束或改回纯 `character_id` 主键模型。此增量方案不在本次登记范围内，留待未来产品决策后立项。

### 7.4 `inventory_items`（可堆叠物资）缺口 —— 真实未实现能力，非命名差异

DTL-001 §3.1 `inventory_items`（`item_template_id` + `quantity`，可堆叠消耗品/材料，引用静态数值配置表 per ARC-016）在 economy-service **完全未实现**——`crates/economy-service/migrations/` 现有 4 个 migration（`0001_init.sql`/`0002_saga_init.sql`/`0003_outbox.sql`/`0004_outbox_check_idempotent.sql`）均无对应表。

**注意与 player-service 的 `player_inventory` 区分**：`crates/player-service/migrations/0004_player_characters_inventory.sql` 的 `player_inventory` 是**装备槽位模型**（`UNIQUE(player_id, slot)`、`primary_weapon_id` 外键），与 DTL-001 `inventory_items` 的**可堆叠材料模型**（`item_template_id`+`quantity`）在业务语义上是两套不同能力，**不能相互替代**。当前代码库**没有任何一张表**承载"可堆叠消耗品/材料"这一能力。

**本节不代为决策是否排期实现** `inventory_items`；该缺口继续追踪于附件D TBD-111（见 §7.6），留待 economy 域 Lead 结合游戏内是否存在可堆叠道具（药水/材料等）的产品需求决定。

### 7.5 未决的产品侧问题（本节明确不代为决策）

RGS-BAS-001 §5 UML 声明 `Account "1" *-- "0..*" Character`（一账号多角色聚合），且 `session_epoch` 明确归属角色级（§5.3）。**多角色账号是否共享货币/道具，还是各角色独立经济**，是尚未有具名人类决策的产品问题——DTL-001 原始设计（角色级 `wallets`）与现行实现（玩家级 `accounts`）分别隐含了两种不同答案，但均**非**经过产品侧显式确认的结论。本节登记现行实现为技术基线，**不代表该产品问题已解决**；该问题继续独立于 §7.3 的分片路由技术论据存在。

### 7.6 附件D 登记同步

本节内容已同步登记至 `RGS-REQ-005_附件D_问题风险管理表.md` §1.2 ISS-128/TBD-111（状态由"未着手"更新为"部分已修正"）：结构性分歧的**文档治理缺口**（"无反向文档"）已消除；`inventory_items` **能力缺口**与 §7.5 产品侧问题仍**未修正**，继续追踪。
