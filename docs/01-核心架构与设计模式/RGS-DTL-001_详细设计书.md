# 详细设计书（詳細設計書 / Detailed Design Document）

**核心架构：物理数据库设计・协议线格式・核心算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-001 |
| 版本 | 0.6 |
| 父文档 | RGS-BAS-001 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"推进制作更新详细设计"）。本项目此前26个域全部止步于需求+基本设计两层，本文档是**第一份详细设计文档**，作为后续其余域详细设计的模板与先例。范围：RGS-BAS-001§5物理数据库设计的两个最核心限界上下文（player_db／economy_db）落实为具体DDL、§6接口设计落实为具体协议线格式（.proto风格）、§4.2 tick循环与§4.3 AOI算法落实为可直接翻译为Rust实现的伪代码级算法。**本版本不覆盖BAS-001全部章节**，其余限界上下文（match_db／social_db／admin_db）与MT/GD/EV/WF/OB/AD模块详细设计留待后续版本或独立DTL文档补充，见§7 |
| 0.2 | 2026-08-17 | 架构师 | — | 续接详细设计阶段，补齐本文档v0.1自己在§7声明的遗留缺口：新增match_db／social_db／admin_db核心表物理DDL（§6〜§8）、MatchService／SocialService／AdminService协议线格式（§9）、RGS-BAS-001§4.6〜4.8（对局状态机、社交并发控制、事件工作流Outbox分发器、购买Saga补偿路径、可观测性Trace传播）算法详细设计（§10）。**触发原因**：RGS-DTL-025（反作弊）已扩展`admin_db`新增三表、RGS-DTL-026（匹配）已扩展`match_db`新增三表，两文档均在其覆盖范围声明中指出"核心架构自身的DTL-001不尽快补齐，将出现业务域DTL引用的库由谁最终定义全貌的文档权责模糊风险"——本次修订补齐该两库（及social_db）各自的核心表，消除该权责模糊。原§6/§7章节相应重编号为§11/§12，新增§13追溯性表 | 全部 |
| 0.3 | 2026-08-17 | 架构师 | — | 负责人指示"开子代理完成剩余的"（技术选型/遗留不一致收尾）。新增§12解决v0.2自述的`player_db`（UUID）与`match_db`/`admin_db`（BIGINT，RGS-DTL-025/026既定）主键风格不一致：决定保留`player_db`自身UUID主键不变（新增`accounts.player_seq`/`characters.character_seq`两个BIGSERIAL列作为跨库权威数值身份，供BIGINT风格的库直接引用，避免额外维护影子映射表）。原§12/§13章节相应重编号为§13/§14 | §2.1（新增两列）、§12（新增）、原§12→§13、原§13→§14 |
| 0.4 | 2026-08-25 | 架构师 | — | 补记与 RGS-DTL-044 的交叉引用（§2.1 前置段落） + economy_db 实际实现与本节 DDL 的差异说明（§3.1 前置段落），并将上述两处悬置状态登记为附件D ISS-127、ISS-128/TBD-111。**不改变本文档任何既有 DDL 定义、章节编号或既有 ADR 关联**——本版本仅作为"既有应然设计与现行实现/反向文档并存"的双重描述基线 | §2.1（前置说明）、§3.1（前置说明） |
| 0.5 | 2026-08-25 | 架构师 | — | 项目负责人就 ISS-128/TBD-111 拍板方案(a)（沿 DTL-044 模式，以代码为现行基线）：RGS-DTL-037 v0.2 §7 已完成 economy_db `accounts`/`transaction_ledger` 现行 DDL 反向登记，本版本同步更新 §3.1 前置段落指向该反向文档。**不改变本文档任何既有 DDL 定义**——`wallets`/`inventory_items`/`transaction_ledger` 仍为原始应然设计记录，不删除、不修改 | §3.1（前置说明） |
| 0.6 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008） | — | 同步父 BAS-001 升版至 v1.4 + 补 §7.2.1 ARC-013 死锁防止/背压八边界落实 + §3.4 ADR-0057 权威源分级 Tier-1/Tier-2 落实。**不引入新设计**：仅落实 BAS-001 v1.3（§7.2.1 背压设置点八边界一览 + 死锁防止调用图证明）和 v1.4（§5.4.3 Tier-1 强一致不可逆资产 = economy_db 权威源；Tier-2 最终一致过程态 = SceneActor 内存权威源）已确定的内容至详细设计层级；BAS-001 v1.0〜v1.2 涉及章节（§1.1/§1.2/§1.3/§4/§4.5.1/§5.2/§5.3/§5.7/§6.3/§10/§11）已在 v0.1〜v0.5 落实，本版本不重写 | §3.4（新增）、§7.2.1（新增）、§14（追溯性表追加行） |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-001§5逻辑ER图/RGS-BAS-007命名规范完全一致，协议线格式是否与RGS-BAS-001§6字段设计一一对应 |
| 评审（DBA） | | | 索引/约束设计是否满足RGS-BAS-007既定的索引/分区标准，是否有遗漏的高频查询路径未建索引 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：player_db](#2-物理数据库设计playerdb)
3. [物理数据库设计：economy_db](#3-物理数据库设计economydb)
3.1 DDL
3.2 确定请求API的物理执行语义
3.4 权威源分级落实（v0.6 新增）
4. [协议线格式：PlayerService／EconomyService](#4-协议线格式playerserviceeconomyservice)
5. [核心算法详细设计](#5-核心算法详细设计)
5.1 tick循环算法
5.2 AOI计算算法
5.3 死锁防止与背压八边界（v0.6 新增）
6. [物理数据库设计：match_db核心表](#6-物理数据库设计matchdb核心表)
7. [物理数据库设计：social_db](#7-物理数据库设计socialdb)
8. [物理数据库设计：admin_db核心表](#8-物理数据库设计admindb核心表)
9. [协议线格式：MatchService／SocialService／AdminService](#9-协议线格式matchservicesocialserviceadminservice)
10. [§4.6〜4.8算法详细设计](#10-4648算法详细设计)
11. [错误码一览](#11-错误码一览)
12. [跨库标识映射](#12-跨库标识映射v03新增解决与rgs-dtl-025026的主键风格不一致)
13. [本文档的覆盖范围与后续计划](#13-本文档的覆盖范围与后续计划)
14. [追溯性](#14-追溯性)

---

# 1. 前言

## 1.1 本文档的定位

IPA SLCP-JCF2013将设计工程分为**基本设计（システム外部から見た仕様）**与**详细设计（内部構造・実装レベルの仕様）**两个阶段。RGS-BAS-001已完成前者：限界上下文划分、组件职责、逻辑ER图（字段用`string`/`long`/`datetime`等抽象类型表述）、UML接口视图（方法签名但无线格式）。本文档完成后者：**逻辑ER图→物理DDL**（具体SQL类型、约束、索引）、**UML接口→协议线格式**（具体wire类型、字段编号，可直接生成代码）、**流程图→算法伪代码**（可直接翻译为Rust实现，而非仅表达"做什么"）。

## 1.2 本文档不做什么

- **不重新决策**：本文档不引入任何RGS-BAS-001未决定的架构选择，逻辑设计与物理设计之间若出现表面差异（如字段拆分），须能一一追溯回原逻辑设计条目，不得借详细设计之名做架构变更
- **不是实现本身**：本文档仍是设计文档，伪代码用于表达算法逻辑与边界条件，不是可编译的Rust源码；.proto风格片段用于固定线格式，实际是否采用Protobuf或其他IDL留给实现阶段，本文档固定的是**字段编号与类型**这一契约本身

## 1.3 记述规则

物理类型标注遵循PostgreSQL语法；协议字段编号规则：1〜15为高频字段（varint单字节编码），16以上为低频/可选字段，编号一经分配**不得**在后续版本变更或复用（同Protobuf既定最佳实践，即便本项目最终不采用Protobuf本身，该编号纪律仍保留作为契约稳定性的通用原则）。

---

# 2. 物理数据库设计：player_db

## 2.1 DDL

> **与 RGS-DTL-044 的关系（2026-08-25 补记）**：本节 `accounts` / `characters` DDL 为初版"应然"设计（2026-08-17 v0.1 起，v0.3 仅新增 `player_seq` / `character_seq` 两列）。实际实现中，`player-service` 落地为 `players` / `player_characters` / `player_inventory`（表名、主键命名风格、`status` 物理类型、是否存在 `session_epoch` 等均与本节不同），该实现已由 **`RGS-DTL-044_player主表_v0.1.md`**（2026-08-24，状态 🟢 v1.0，A-02 偿还技术债的反向文档）正式登记为现行设计基线，逐列标注来源 `entity.rs`。
>
> 本节下述 `player_seq` / `credential_hash` / `session_epoch` / OCC `version` / `ban_records` 等字段／表，DTL-044 未覆盖，在 `crates/player-service/migrations/0001_init.sql` 与 `crates/player-service/src/entity.rs` 中亦**尚未存在**，视为本节仍为有效设计意图但**尚未实现**的部分（其中 `player_seq` / `character_seq` 作为 v0.3 新增的跨库权威数值身份，本身就需待 `accounts` / `characters` 表最终被实现后才有意义——与 DTL-044 的反向登记并存于同一文档体系，形成"应然 vs 现行"双重描述）。该悬置状态已登记至附件D **ISS-127**，由架构师跟进实现路径决策；本文档不修改任何既有 DDL 定义，仅在此处补记与 DTL-044 的关系以消除两份设计文档对同一张表给出不同表名／主键／字段的治理缺口。

```sql
-- 复用RGS-BAS-007既定命名规范：表名snake_case复数、主键统一为<entity>_id、
-- 乐观并发列统一命名version、审计时间戳统一created_at/updated_at

CREATE TABLE accounts (
    player_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_seq      BIGSERIAL NOT NULL UNIQUE,      -- v0.3新增,跨库标识映射的权威数值ID,见§12"跨库标识映射"
    credential_hash TEXT NOT NULL,
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0=Registered 1=Active 2=Suspended 3=Banned 4=Deleted（枚举值见ST-005，故意用SMALLINT而非TEXT：高频WHERE条件，避免字符串比较开销）
    version         BIGINT NOT NULL DEFAULT 0,     -- OCC，DR-007
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_accounts_status CHECK (status BETWEEN 0 AND 4)
);

CREATE TABLE characters (
    character_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    character_seq    BIGSERIAL NOT NULL UNIQUE,     -- v0.3新增,跨库标识映射的权威数值ID,见§12"跨库标识映射"
    player_id        UUID NOT NULL REFERENCES accounts(player_id) ON DELETE RESTRICT,
    -- ON DELETE RESTRICT而非CASCADE：账号删除走FR-GOV-010〜013既定的跨库编排流程
    -- （RGS-BAS-009§5.2），不得由数据库外键级联静默删除，避免绕过审计留痕
    name              VARCHAR(32) NOT NULL,
    level             INT NOT NULL DEFAULT 1,
    current_scene_id  UUID,                        -- 可空，未在场景内时为NULL
    session_epoch     BIGINT NOT NULL DEFAULT 0,    -- ARC-005 Single-Writer核心机制
    version           BIGINT NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_characters_name UNIQUE (name)     -- 角色名全局唯一，BR-001既定
);

CREATE TABLE ban_records (
    ban_id      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id   UUID NOT NULL REFERENCES accounts(player_id) ON DELETE RESTRICT,
    reason      TEXT NOT NULL,
    issued_by   UUID NOT NULL,                      -- 逻辑引用admin_db操作者，跨库不建物理FK（同RGS-BAS-007既定跨限界上下文引用规则）
    issued_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ                          -- NULL=永久封禁
);

-- 索引设计（对应高频查询路径）
CREATE INDEX idx_characters_player_id ON characters (player_id);
    -- 支撑§4.4.1 GetCharacterList：按player_id列出全部角色
CREATE INDEX idx_characters_current_scene_id ON characters (current_scene_id)
    WHERE current_scene_id IS NOT NULL;
    -- 部分索引：支撑"某场景当前有哪些角色"查询（运维/GM工具），NULL值不入索引减小体积
CREATE INDEX idx_ban_records_player_id_active ON ban_records (player_id)
    WHERE expires_at IS NULL OR expires_at > now();
    -- 部分索引：登录鉴权路径（FR-PL-001）需要快速判定"当前是否有生效中的封禁"，
    -- 只索引未过期/永久记录，历史已过期封禁不占索引空间
```

## 2.2 与逻辑设计的对应关系

| RGS-BAS-001§5.3逻辑字段 | 物理实现 | 差异说明 |
|---|---|---|
| `Account.status`（字符串枚举） | `accounts.status SMALLINT` | 高频查询列改用整数枚举，逻辑语义不变，仅物理编码优化（同RGS-BAS-007既定"高频WHERE列避免TEXT比较"原则） |
| `Character.name`唯一性 | `uq_characters_name UNIQUE` | BAS-001 UML图未显式标注唯一性，本文档补充物理约束（角色名全局唯一是既有BR-001隐含要求，此前逻辑设计遗漏显式声明，此处不算新决策，是对既有需求的物理落实） |
| 其余字段 | 类型直译 | `string`→`UUID`/`VARCHAR`/`TEXT`视语义而定，`long`→`BIGINT`，`datetime`→`TIMESTAMPTZ`（复用RGS-BAS-007既定"时间戳一律带时区"规范） |

---

# 3. 物理数据库设计：economy_db

## 3.1 DDL

> **与 economy-service 实际实现的关系（2026-08-25 补记）**：本节 `wallets` / `inventory_items` / `transaction_ledger` DDL 为初版"应然"设计（2026-08-17 v0.1 起）。实际实现 `crates/economy-service/migrations/0001_init.sql` 落地为 `accounts`（PK `id`、列 `player_id`，货币作为**行**而非独立表）+ `transaction_ledger`（字段 `idempotency_key` / `kind TEXT` / `amount BIGINT`，**无** `payload JSONB`），且**未实现** `inventory_items` 表。两套表名／主键／字段命名／角色级 vs 玩家级＋币种行的整体模型均存在结构性分歧。
>
> **2026-08-25 更新（v0.5）**：项目负责人已就上述悬置状态拍板方案(a)——沿 DTL-044 模式为 economy_db 写反向文档，以代码为现行基线。**RGS-DTL-037 v0.2 §7** 已完成 `accounts`/`transaction_ledger` 字段级反向登记，含 `player_id`（而非 `character_id`）分片键的技术合理性论据（per RGS-DTL-022 v0.2 + RGS-REQ-025-ADD1 ARC-040-2/AC-CAP-101 + RGS-ADR-0057 §2.2）与 `inventory_items` 能力缺口的显式记录。附件D **ISS-128/TBD-111** 状态同步更新为"部分已修正"——文档治理缺口（无反向文档）已消除；`inventory_items` 能力缺口与"多角色账号经济是否共享"的产品侧问题（见 DTL-037 §7.5）仍未修正，继续追踪。本文档（DTL-001 §3.1）不修改任何既有 DDL 定义，`wallets`/`inventory_items`/`transaction_ledger` 仍完整保留为原始应然设计记录。

```sql
CREATE TABLE wallets (
    character_id  UUID PRIMARY KEY,   -- 逻辑引用player_db.characters，跨库不建物理FK
    balance        BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    version         BIGINT NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE inventory_items (
    item_instance_id  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    character_id      UUID NOT NULL,
    item_template_id  VARCHAR(64) NOT NULL,   -- 引用静态配置表（ARC-016数值表），非动态数据
    quantity          INT NOT NULL DEFAULT 1 CHECK (quantity > 0),
    version           BIGINT NOT NULL DEFAULT 0,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE transaction_ledger (
    -- 落地FR-EC-003确定请求API的幂等键与审计追溯，仅追加，复用RGS-BAS-007§4幂等去重表设计标准
    ledger_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id    UUID NOT NULL,          -- 幂等键，客户端/上游生成
    character_id  UUID NOT NULL,
    operation     SMALLINT NOT NULL,      -- 0=grant_item 1=consume_item 2=grant_currency 3=consume_currency
    payload       JSONB NOT NULL,         -- 操作明细（item_template_id/quantity或currency delta），结构随operation变体
    expected_version BIGINT NOT NULL,     -- OCC校验时使用的版本号
    result_version    BIGINT NOT NULL,    -- 执行成功后的新版本号
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_transaction_ledger_request_id UNIQUE (request_id)
    -- 唯一约束是幂等性的物理强制层：重复request_id的INSERT在数据库层即失败，
    -- 不依赖应用层查重逻辑单独兜底（纵深防御，同ARC-009既定幂等设计精神）
);

-- 分区：按RGS-BAS-007§4既定标准，交易流水表按时间范围月度分区
CREATE TABLE transaction_ledger_template (LIKE transaction_ledger INCLUDING ALL);
-- 实际分区创建由既有分区滚动创建自动化流程（RGS-BAS-007§4）执行，本文档仅声明分区键

CREATE INDEX idx_inventory_items_character_id ON inventory_items (character_id);
CREATE INDEX idx_transaction_ledger_character_id_created ON transaction_ledger (character_id, created_at);
    -- 支撑GM工具/玩家自查"我的交易历史"按角色+时间范围查询
```

## 3.2 确定请求API的物理执行语义

`CommitTransaction`（RGS-BAS-001§4.5.1既有时序）对应的物理事务边界：

```sql
BEGIN;
  -- 1. 幂等检查：request_id已存在则直接返回既有结果，不重复执行（应用层先查一次可选，
  --    但真正的幂等保证来自下方INSERT的唯一约束，查询只是避免不必要的失败重试）
  -- 2. OCC校验：更新前先确认version未变
  UPDATE wallets SET balance = balance + $delta, version = version + 1, updated_at = now()
    WHERE character_id = $cid AND version = $expected_version;
  -- 若UPDATE影响行数为0，说明version已被并发修改，事务回滚，向上层返回OCC冲突错误（ST-004状态转移）
  -- 3. 写入流水（幂等键的物理强制点）
  INSERT INTO transaction_ledger (request_id, character_id, operation, payload, expected_version, result_version)
    VALUES ($request_id, $cid, $op, $payload, $expected_version, $expected_version + 1);
    -- 若request_id冲突，本INSERT失败，事务整体回滚——但此时上一步wallets的UPDATE也会一并回滚，
    -- 不会出现"钱包已扣但流水未记"的不一致态，这是"同一事务边界"设计的直接物理保证
COMMIT;
```

## 3.4 权威源分级落实（per RGS-BAS-001 §5.4.3 / RGS-ADR-0057 §2.1，v0.6 同步父 BAS-001 v1.4）

RGS-BAS-001 v1.4 在 §5.4.3 将 `economy_db` 与 SceneActor 内存的权威源分两级，本节将这两级权威源在详细设计层的具体落位落实——**不引入新设计**，仅落实 BAS 已确定的分级语义到具体表/字段/恢复路径。

### 3.4.1 Tier-1：economy_db 权威（强一致不可逆资产）

`wallets.balance`（§3.1）、`transaction_ledger`（§3.1，含 `Wallet`/`Inventory` 操作流水）、`inventory_items`（§3.1，保留事件溯源语义下的当前持有快照）：

- **权威源**：`economy_db`。SceneActor 侧持有的 `Wallet` / `Inventory` 字段仅为**读缓存**（tick 内高频读取避免跨服务往返）。
- **写路径**：依 BAS-001 §4.5.2 确定请求，SceneActor 通过 `Out2` 异步调用 economy-service，**不阻塞当前 tick**（不改变 §5.1 tick 阶段预算，NFR-PE-002 25ms 总预算）。
- **客户端确认时序**：SceneActor 在收到 `economy_db` 提交成功回执之前，**不得**向客户端下发该操作成功确认——这是 Tier-1 权威源语义的物理表现：DB 未提交的资产变更对客户端不可见。
- **崩溃恢复语义**：`economy_db` 自身崩溃恢复时，Tier-1 字段天然完整（事务日志/WAL 保证），**不依赖**任何 SceneActor 侧 Checkpoint 机制。

### 3.4.2 Tier-2：SceneActor 内存权威（最终一致过程态）

坐标、技能冷却、任务计数、临时 Buff 等——**不落于** `economy_db`，而是 SceneActor 内存独有：

- **权威源**：SceneActor 内存。DB 仅作周期性 Checkpoint（暂定 ≤30 秒，per 附件 D ISS-010 探讨中，RPO 上界 30 秒）。
- **崩溃恢复语义**：SceneActor 崩溃后从最近一次 Checkpoint 恢复，可能丢失至多一个 Checkpoint 周期内的 Tier-2 状态变化——这是 BAS-001 v1.4 §5.4.3 既定的可接受代价。
- **与 Tier-1 的零冲突声明**：两者区别落在"权威源在哪"（DB 权威 vs Actor 权威），**而非**"写入时机是否落在同一 tick"——写路径均为异步、均不阻塞 tick，与 §5.1 tick 循环结构、CON-007、ARC-007 三项既有约束零冲突（落实 BAS-001 v1.4 修订历史声明）。

### 3.4.3 详细设计层不引入的内容

- 不为 Tier-1/Tier-2 设计独立的数据通路——两者复用 §3.1 既定 DDL + §4 既定协议线格式 + §5.1 既定 tick 阶段，只是写路径时序与崩溃恢复语义按本节分级落实。
- 不修改 `wallets` / `inventory_items` / `transaction_ledger` 任何既有 DDL 定义。
- 不在 `economy_db` 新增 Checkpoint 表（Tier-2 的 Checkpoint 属 SceneActor 进程内机制，不落 economy_db）。

### 3.4.4 待跟进项

- Tier-2 Checkpoint 周期 ≤30 秒的初始值在详细设计阶段提供可配置参数项（便于 PH-4 调参，NFR-PE-002 与 BAS-001 §7.2 背压参数表风格一致），具体数值由运行时实现阶段给出。
- 附件 D ISS-010（RPO 30 秒）仍由架构师跟进，本节不替父文档决定具体数值。

---

# 4. 协议线格式：PlayerService／EconomyService

## 4.1 设计说明

RGS-BAS-001§6.3已定义方法签名与字段名，本节固定**字段编号**（协议演进的兼容性锚点，一旦分配不可变更/复用，同ARC-015 Expand-Contract精神在协议层的对应物）。以下用Protobuf风格表达线格式契约，不代表实现阶段必须采用Protobuf本身。

## 4.2 PlayerService

```protobuf
message AuthenticateRequest {
  string credential_token = 1;
}
message AuthenticateResponse {
  string player_id = 1;
  repeated CharacterSummary character_list = 2;
  ResultCode result_code = 3;
}
message CharacterSummary {
  string character_id = 1;
  string name = 2;
  int32 level = 3;
}

message SelectCharacterRequest {
  string player_id = 1;
  string character_id = 2;
}
message SelectCharacterResponse {
  int64 session_epoch = 1;      // 新发行的epoch，ARC-005核心字段，字段号低位优先编码
  string current_scene_id = 2;
  ResultCode result_code = 3;
}
```

## 4.3 EconomyService

```protobuf
message CommitTransactionRequest {
  string request_id = 1;        // 幂等键，编号1（最高频访问字段）
  string character_id = 2;
  int64 session_epoch = 3;      // ARC-005：请求必须携带当前epoch供服务端校验陈旧请求
  oneof operation {
    GrantItem grant_item = 10;
    ConsumeItem consume_item = 11;
    GrantCurrency grant_currency = 12;
    ConsumeCurrency consume_currency = 13;
  }
  int64 expected_version = 20;  // OCC字段编号统一置于20+区间，与业务字段区分
}
message CommitTransactionResponse {
  ResultCode result_code = 1;
  int64 new_version = 2;
  string ledger_id = 3;
}
```

## 4.4 通用`ResultCode`枚举（跨全部服务复用，不各自定义）

```protobuf
enum ResultCode {
  OK = 0;
  UNKNOWN_ERROR = 1;
  OCC_CONFLICT = 2;           // 对应ST-004乐观并发冲突
  STALE_SESSION_EPOCH = 3;    // ARC-005：epoch已被更新的SelectCharacter请求作废
  INVALID_REQUEST = 4;
  ACCOUNT_BANNED = 5;
  INSUFFICIENT_BALANCE = 6;
  DUPLICATE_REQUEST_ID = 7;   // 幂等键冲突但内容不一致（正常重放应直接返回原结果而非此错误）
}
```

---

# 5. 核心算法详细设计

## 5.1 tick循环算法（落实RGS-BAS-001§4.2.2流程图为伪代码）

```
fn scene_actor_tick(scene: &mut SceneState, tick_no: u64) {
    let tick_start = Instant::now();

    // 阶段1：输入应用（ECS System顺序执行，同一场景内严格串行，ARC-001既定）
    for input in scene.pending_inputs.drain_up_to(tick_no) {
        // 乱序/重复输入的排除：sequence_no单调性检查（FR-RT-004）
        if input.sequence_no <= scene.last_applied_sequence[input.entity_id] {
            continue;  // 静默丢弃，不报错（客户端重传是正常现象，非异常）
        }
        apply_input_system(scene, input);
        scene.last_applied_sequence[input.entity_id] = input.sequence_no;
    }

    // 阶段2：移动模拟（含FR-SEC-050未信任输入解析安全边界——本阶段全部读取均来自
    // 阶段1已校验通过的输入，不重新解析原始字节，避免重复的panic风险面）
    movement_system(scene);

    // 阶段3：战斗判定（服务器权威，ARC-002）
    combat_system(scene);

    // 阶段4：AOI更新（§5.2另行详述）
    aoi_update_system(scene);

    // 阶段5：复制生成（差分快照）
    let snapshot = generate_delta_snapshot(scene, tick_no);
    broadcast_to_clients(scene, snapshot);

    // 耗时预算校验（NFR-PE-002）
    let elapsed = tick_start.elapsed();
    if elapsed > TICK_BUDGET_SOFT_LIMIT {
        emit_metric("tick_overrun", elapsed);  // 复用RGS-BAS-004既定指标体系，不新建监控通道
    }
    if elapsed > TICK_BUDGET_HARD_LIMIT {
        // 硬预算超支：本tick已产生的快照仍然发送（不丢弃已完成的工作），
        // 但触发降级判定（是否需要临时降低AOI半径/快照频率，同ARC-013既定原则）
        trigger_degradation_check(scene);
    }
}
```

**关键边界条件说明**：
- 输入应用阶段的乱序/重复排除是**幂等的必要条件**——若不做此排除，客户端网络抖动导致的重传会被反复应用，产生移动量翻倍等错误
- 硬预算超支**不得**导致丢弃已完成的快照生成结果（宁可这一tick延迟发送，不得让客户端出现"完全没收到状态更新"的更差体验）

## 5.2 AOI计算算法（落实RGS-BAS-001§4.3.1，含G-003无饿死性设计）

```
fn aoi_update_system(scene: &mut SceneState) {
    for observer in scene.entities_with_view() {
        let mut candidates: Vec<(EntityId, f64)> = Vec::new();

        for target in scene.entities_in_grid_range(observer.position, VIEW_DISTANCE) {
            if target.id == observer.id { continue; }

            let distance_score = 1.0 - (distance(observer.position, target.position) / VIEW_DISTANCE);
            let importance_score = target.importance_weight;  // 静态权重，ARC-016数值表配置

            // G-003无饿死性设计：老化因子随"距上次更新的tick数"单调增长，
            // 权重下限恒正，保证任意实体等待足够久后必被纳入
            let ticks_since_update = scene.current_tick - target.last_aoi_update_tick;
            let aging_score = AGING_BASE_WEIGHT + AGING_GROWTH_RATE * (ticks_since_update as f64);

            let final_score = distance_score * W_DISTANCE
                             + importance_score * W_IMPORTANCE
                             + aging_score * W_AGING;
            candidates.push((target.id, final_score));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let selected = candidates.into_iter().take(observer.aoi_budget).collect();

        update_observer_view(observer, selected, scene.current_tick);
    }
}
```

**无饿死性证明要点**：`aging_score`是`ticks_since_update`的严格单调递增函数且下限为`AGING_BASE_WEIGHT > 0`，故任一实体若持续未被选中，其`ticks_since_update`持续增长，`final_score`最终必然超过任何"新近更新过"的竞争实体的得分上限（`W_DISTANCE + W_IMPORTANCE`为常数上界），保证有限时间内必被选中——这是G-003"老化机制"要求的形式化落实，而非仅停留在文字描述。

## 5.3 死锁防止与背压八边界（per RGS-BAS-001 §7.2.1 / ARC-013，v0.6 同步父 BAS-001 v1.3）

RGS-BAS-001 v1.3 在 §7.2.1 给出"背压设置点一览（八边界）"与"死锁防止的具体证明"。本节将这两部分从系统级设计落实为**可执行的边界配置表 + 可重跑的调用图核查代码**——**不引入新设计**，仅将 BAS 既定机制落位到具体配置项与可验证算法。

### 5.3.1 背压八边界配置表（落实 BAS-001 §7.2.1 八边界一览）

ARC-013 要求在"客户端连接、网关、场景mailbox、gRPC、事件消费者、数据库连接池、工作流Activity、日志路径"八处边界均须设置上限。本节给出每条边界的**配置项定义**与**初始默认值**（PH-4 负载试验后调整）：

| 背压边界 | 配置项 | 类型 | 初始默认值 | 拒绝策略 | 落位位置 |
|---|---|---|---|---|---|
| 客户端连接 | `gw.max_conns_per_ip` | u32 | 8 | 超限拒绝新连接 | gateway-service |
| 客户端连接 | `gw.input_rate_per_conn_hz` | u32 | 60 | 超出降级为心跳 | gateway-service |
| 网关 | `gw.route_pending_max` | u32 | 4096 | 满则返回重连提示 | gateway-service（§4.1.4输入路由） |
| 网关 | `gw.hpa_max_replicas` | u32 | 32 | HPA 上限 | K8s HPA 配置（BAS-001 §3.2） |
| 场景mailbox | `rt.scene_actor_mailbox_cap` | u32 | 256 | 满则按 BAS-001 §4.1.4 step4 直接拒绝 | runtime SceneActor（ARC-001） |
| gRPC（东西向） | `rpc.pool_max_per_target` | u32 | 16 | 排队等待；超时则 fail-fast | tonic client pool |
| gRPC（东西向） | `rpc.request_timeout_ms` | u64 | 500 | 超时则取消 | tonic client |
| 事件消费者 | `evt.dispatcher_retry_backoff_ms` | u64 | 1000 | 下轮重试，不无限堆积 | outbox 分发器（§10.2） |
| 事件消费者 | `evt.consumer_concurrency` | u32 | 32 | 消费者并发度上限 | 工作流 Activity 容器 |
| 数据库连接池 | `db.pool_max_conns`（per service） | u32 | 32 | 满则排队；超时则 fail | sqlx pool |
| 工作流Activity | `wf.activity_max_retries` | u32 | 3 | 超出后进补偿（§10.3） | Temporal Activity |
| 工作流Activity | `wf.activity_timeout_ms` | u64 | 3000 | 超时则 Activity 重试 | Temporal Activity |
| 日志路径 | `log.error_sampling = 1.0` | f64 | 1.0 | 错误全量 | observability |
| 日志路径 | `log.normal_sampling = 0.01` | f64 | 0.01 | 正常路径 1% 采样 | observability（BAS-001 §7.2 引 RGS-BAS-004） |

**表的使用约束**（落实 BAS-001 §7.2.1 背压参数"须可配置"要求）：上表所有项**均**通过 `crates/<service>/src/config.rs` 的 `serde::Deserialize` 加载，PH-1 阶段交付具体数值，PH-4 负载试验后调参。本节不替实现阶段决定具体生产值（除"必须存在的配置项"本身）。

### 5.3.2 死锁防止的可重跑核查代码（落实 BAS-001 §7.2.1 死锁防止证明）

BAS-001 §7.2.1 在系统级给出"对全部东西向调用边核查反向是否存在同步等待边"的方法。本节将其落实为**可在测试/CI 阶段重跑的 Rust 函数**——输入为系统调用图（从 build 期的依赖注入 + tonic service 描述静态生成），输出为"是否存在环"的布尔结论。

```rust
/// 死锁防止核查：对调用图所有"等待应答"边，逐一验证其反向不存在同步等待边
/// 落实 RGS-BAS-001 §7.2.1 ARC-013 死锁防止证明方法为可重跑算法
///
/// 关键不变量：调用图中"调用方在发起后未收到应答前阻塞"的边（wait_for_reply=true）
/// 集合中，任意一条边的反向若存在 wait_for_reply=true 的另一条边，则形成环——违反 ARC-013。
fn assert_no_deadlock_cycle(call_graph: &CallGraph) -> Result<(), DeadlockError> {
    use std::collections::HashMap;

    // 1. 提取所有"等待应答"边
    let wait_edges: Vec<&Edge> = call_graph.edges.iter()
        .filter(|e| e.wait_for_reply)
        .collect();

    // 2. 按"目标节点"分组：同一被调用方有多个调用方在等待时，按 ARC-013 须为优先级
    //    不同的独立通道；本检查仅做环检测，优先级由各服务的并发度配置保证（不属本节范围）
    let mut by_target: HashMap<&str, Vec<&Edge>> = HashMap::new();
    for e in &wait_edges {
        by_target.entry(&e.to).or_default().push(*e);
    }

    // 3. 逐条等待边的反向核查：对每条 (A→B, wait)，检查是否存在 (B→A, wait)
    for e in &wait_edges {
        let reverse_exists = wait_edges.iter().any(|other| {
            other.from == e.to && other.to == e.from && other.wait_for_reply
        });
        if reverse_exists {
            return Err(DeadlockError::Cycle {
                edge_a: (e.from.clone(), e.to.clone()),
                edge_b: (e.to.clone(), e.from.clone()),
            });
        }
    }

    // 4. 调用图整体环路检测（DFS），覆盖多跳环
    if let Some(cycle) = call_graph.detect_cycle_dfs() {
        return Err(DeadlockError::Cycle { edge_a: (cycle[0].clone(), cycle[1].clone()), edge_b: (String::new(), String::new()) });
    }

    Ok(())
}
```

**对当前系统调用图的核查结论**（按 BAS-001 §7.2.1 表格逐行落实为代码可验证事实）：

| 调用方向 | `wait_for_reply` | 状态 |
|---|---|---|
| 网关 → 运行时（§4.1.4 输入路由） | `false`（fire-and-forward，mailbox 满则直接拒绝，不阻塞） | ✅ |
| 网关 → 玩家服务（§4.1.2 鉴权） | `true`（单向请求-应答） | ✅（玩家服务不反向调用网关） |
| 运行时 → 经济服务（§4.5.2 确定请求） | `false`（不阻塞 tick，结果在后续 tick 反映） | ✅ |
| 经济服务 → 运行时 | 不存在（BAS-001 §6.3 接口图无 `EconomyService → RuntimeCaller`） | ✅ |
| 业务服务 → 事件基础设施（§4.7.1 Outbox） | `false`（事务提交后立即返回，不等待消费） | ✅ |
| 事件基础设施 → 消费者 | `false`（消费者不产生反向同步应答） | ✅ |

**若详细设计阶段新增任何服务间同步调用**（wait_for_reply=true 的边），须在新增的 DTL 文档中以同款表格核查并附 `assert_no_deadlock_cycle` 的测试用例通过证据，否则不得合并——这是 ARC-013 在详细设计层留下的可执行检查方法，对应 BAS-001 §7.2.1"重新执行本表的核查并更新本节"要求。

---

# 6. 物理数据库设计：match_db核心表

对应RGS-BAS-001§5.5 `MATCH`/`MATCH_PARTICIPANT`/`MATCH_RESULT`逻辑ER图与需求定义书§8.3 ST-002对局状态机。`match_db`同库内已由RGS-DTL-026§2新增`queue_entries`/`match_ratings`/`match_quality_metrics`三表（匹配侧），本节补齐对局侧核心表——两份文档合并起来才是`match_db`的完整物理设计，RGS-DTL-026§2的`queue_entries.match_ref BIGINT`已隐含约定`matches.match_id`为`BIGINT`（非本文档§2 `player_db`/`economy_db`使用的`UUID`风格），本节DDL遵循该已发布约定，不引入第三种不一致的主键风格（`match_db`/`admin_db`与`player_db`间的BIGINT/UUID跨库映射机制见§12）。

```sql
-- 对局主表，落实需求定义书§8.3 ST-002状态机
CREATE TABLE matches (
    match_id     BIGSERIAL PRIMARY KEY,   -- 与RGS-DTL-026 queue_entries.match_ref的BIGINT假设保持一致
    status       TEXT NOT NULL DEFAULT 'Created'
                   CHECK (status IN ('Created', 'Waiting', 'Running', 'Finished', 'Archived', 'Cancelled')),
    mode         TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at   TIMESTAMPTZ,
    finished_at  TIMESTAMPTZ,
    version      BIGINT NOT NULL DEFAULT 0   -- OCC，同RGS-DTL-001§3.2/RGS-DTL-026§5既定模式
);
CREATE INDEX idx_matches_active ON matches (status) WHERE status IN ('Waiting', 'Running');
    -- 支撑§10.1状态机驱动逻辑的"当前活跃对局"扫描路径

-- 对局参与者表
CREATE TABLE match_participants (
    match_id      BIGINT NOT NULL REFERENCES matches(match_id),
    character_id  BIGINT NOT NULL,   -- 与RGS-DTL-026 match_ratings.character_id的BIGINT假设保持一致（跨库映射机制见§12）
    team          TEXT NOT NULL,
    joined_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (match_id, character_id)
);
CREATE INDEX idx_match_participants_character ON match_participants (character_id);
    -- 支撑"某角色参与过哪些对局"查询（GM工具/玩家历史）

-- 对局结果表
CREATE TABLE match_results (
    match_id         BIGINT PRIMARY KEY REFERENCES matches(match_id),
    outcome          TEXT NOT NULL,
    rewards_granted  BOOLEAN NOT NULL DEFAULT FALSE,   -- 是否已经过§4.5.1确定请求授予奖励
    finalized_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**状态迁移与物理更新的对应**（落实需求定义书§8.3表格与RGS-BAS-001§8状态迁移详细设计中的触发来源表为可执行SQL，OCC模式同§3.2）：

```sql
-- 例：Running → Finished（场景Actor判定结束条件成立后调用，§10.1详述触发点）
UPDATE matches
SET status = 'Finished', finished_at = now(), version = version + 1
WHERE match_id = $match_id AND status = 'Running' AND version = $expected_version;
-- 影响行数=0：并发的重复结束判定或版本已被其他路径变更，调用方按OCC_CONFLICT处理（不视为异常，重新读取当前状态即可）
```

不允许的迁移（如`Archived`之后的任何写入、`Terminating`阶段之外直接写`Cancelled → Running`）由`CHECK`约束保证的取值集合与应用层状态机共同兜底：`CHECK`约束只保证落在合法状态取值集合内，**不保证**迁移路径合法（PostgreSQL `CHECK`无法表达"仅允许从X迁移到Y"），故迁移路径的合法性校验职责在应用层（§10.1伪代码），这是本文档必须明确记录的物理约束局限，避免实现者误以为数据库层已完整兜底。

---

# 7. 物理数据库设计：social_db

对应RGS-BAS-001§5.6 `FRIEND_LINK`/`GUILD`/`GUILD_MEMBER`逻辑ER图。`social_db`目前尚无任何其他DTL文档扩展，本节是该库的首次物理落地，沿用本文档§2/§3已确立的`UUID`主键风格（`player_db`/`economy_db`同源，`character_id`逻辑引用`player_db.characters.character_id`）。

```sql
-- 好友关系表：无向关系以有序对(a,b)存储避免重复行，插入前由应用层规范化"较小UUID排在a"
CREATE TABLE friend_links (
    character_id_a  UUID NOT NULL,
    character_id_b  UUID NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (character_id_a, character_id_b),
    CONSTRAINT chk_friend_links_ordered CHECK (character_id_a < character_id_b)
    -- 应用层规范化职责：写入前必须按UUID字节序排序两端，数据库仅做事后校验兜底，防止(a,b)与(b,a)重复行
);
CREATE INDEX idx_friend_links_b ON friend_links (character_id_b);
    -- PRIMARY KEY(a,b)已提供以a为前缀的索引；本索引提供"以b查询"方向，满足双向好友列表查询

CREATE TABLE guilds (
    guild_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(32) NOT NULL,
    version      BIGINT NOT NULL DEFAULT 0,   -- OCC，成员变更等操作的并发控制
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_guilds_name UNIQUE (name)   -- 公会名全局唯一，同RGS-DTL-001§2.2 characters.name唯一性判定原则
);

CREATE TABLE guild_members (
    guild_id      UUID NOT NULL REFERENCES guilds(guild_id) ON DELETE RESTRICT,
    -- ON DELETE RESTRICT而非CASCADE：公会解散走应用层编排流程（成员清退通知等），
    -- 不由数据库外键级联静默删除，理由同RGS-DTL-001§2.1 characters.player_id外键设计
    character_id  UUID NOT NULL,
    role          TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('member', 'officer', 'leader')),
    joined_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, character_id)
);
CREATE INDEX idx_guild_members_character ON guild_members (character_id);
    -- 支撑"某角色所属公会"查询（单角色同时只能属于一个公会由应用层规则保证，非本表约束职责——
    -- 若需要数据库层强制"每角色至多一个公会"，须额外加character_id上的UNIQUE约束，
    -- 本文档未见RGS-BAS-001明确该业务规则，故不擅自添加，留待RGS-BAS-001后续确认）
```

---

# 8. 物理数据库设计：admin_db核心表

对应RGS-BAS-001§5.7 `OPERATION_AUDIT`/`COMPENSATION_BATCH`逻辑ER图。`admin_db`同库内已由RGS-DTL-025§2新增`detection_signals`/`anticheat_cases`/`case_signal_links`三表（反作弊侧，`player_id BIGINT`，逻辑对应`player_db.accounts.player_seq`，跨库不设物理FK，见§12），本节补齐运营治理核心表，沿用RGS-DTL-025已确立的`BIGINT`主键/外键风格，不在同一库内引入第三种不一致的类型约定（同§6对`match_db`的处理原则，跨库映射机制见§12）。

```sql
-- 操作审计表，NFR-SE-010"仅追加不可变"是RGS-BAS-001§5.7已明确标注的唯一强约束
CREATE TABLE operation_audits (
    audit_id           BIGSERIAL PRIMARY KEY,
    operator_id         BIGINT NOT NULL,     -- 逻辑引用运营/GM账号，非玩家account_id体系，跨库不建物理FK
    action_type          TEXT NOT NULL,
    target_player_id       BIGINT,           -- 可空：部分操作类型（如系统级维护开关）不针对特定玩家
    detail                  JSONB NOT NULL DEFAULT '{}',
    occurred_at              TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (occurred_at);
-- 月度分区，复用RGS-DTL-025§2已使用的admin_db既有分区管理脚本(G-005模式)，不新建另一套分区机制

CREATE INDEX idx_operation_audits_target_player
    ON operation_audits (target_player_id, occurred_at) WHERE target_player_id IS NOT NULL;
CREATE INDEX idx_operation_audits_operator
    ON operation_audits (operator_id, occurred_at);
    -- 两个查询方向：按被操作玩家追溯("谁对该玩家做过什么") / 按操作者追溯("该GM做过什么")，均为运营审计常见查询路径

-- 补偿批次表
CREATE TABLE compensation_batches (
    batch_id      BIGSERIAL PRIMARY KEY,
    created_by     BIGINT NOT NULL,          -- 逻辑引用运营/GM账号，同operator_id体系
    reason           TEXT NOT NULL,
    item_grants        JSONB NOT NULL,        -- {character_ids: [...], item_template_id, quantity}结构，对应§9.3 GrantCompensation请求体
    status               TEXT NOT NULL DEFAULT '待执行'
                            CHECK (status IN ('待执行', '执行中', '已完成', '部分失败')),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`compensation_batches`与`operation_audits`是"一对多生成"关系（RGS-BAS-001§5.7 ER图`COMPENSATION_BATCH ||--o{ OPERATION_AUDIT : generates`）：每个批次实际执行时，对批次内每个受益角色各生成一条`operation_audits`记录（`action_type='COMPENSATION_GRANT'`，`detail`内含`batch_id`），而非用物理外键关联——`operation_audits.detail`是`JSONB`，`batch_id`作为其中一个键存在，不建独立外键列，理由是`operation_audits`已经是全`admin_db`范围内单一的追加日志表，各类操作生成审计记录的关联信息结构不同（`detail`本身即为承载该差异的字段，同RGS-DTL-025§2`detection_signals.context_ref`语义随`signal_type`变体的设计精神一致）。

---

# 9. 协议线格式：MatchService／SocialService／AdminService

对应RGS-BAS-001§6.3.3/§6.3.4的字段级设计（`SocialService`此前仅有UML接口图§6.3类图，无独立字段表，本节按同一图上已列出的三个方法一并落实字段编号）。字段编号规则沿用§1.3/§4.1既定纪律。

## 9.1 MatchService

```protobuf
message EnqueueMatchRequest {
  string character_id = 1;
  string mode          = 2;
}
message EnqueueMatchResponse {
  string queue_ticket_id = 1;   // 对应RGS-DTL-026 queue_entries.entry_id的字符串形式
  ResultCode result_code  = 2;
}

message GetMatchStatusRequest {
  int64 match_id = 1;   // 与§6 matches.match_id物理类型一致(BIGINT)
}
message GetMatchStatusResponse {
  string status = 1;    // 取值同§6 matches.status CHECK约束(ST-002状态机)
  repeated MatchParticipant participants = 2;
}
message MatchParticipant {
  string character_id = 1;
  string team          = 2;
}
```

## 9.2 SocialService

```protobuf
message AddFriendRequest {
  string from_character_id = 1;
  string to_character_id    = 2;
}
message AddFriendResponse {
  ResultCode result_code = 1;
}

message JoinGuildRequest {
  string character_id = 1;
  string guild_id       = 2;
}
message JoinGuildResponse {
  ResultCode result_code = 1;
  int64 new_version        = 2;   // 对应§7 guilds.version，加入后成员数变化不改动guilds.version本身，
                                   -- 此字段实为guild_members写入后的确认回执，非OCC校验字段(加入操作本身非OCC路径)
}

message SendChatRequest {
  string channel              = 1;   // 取值：world｜guild｜whisper，对应RGS-BAS-001§6.2.2 ChatMessage.channel语义
  string sender_character_id   = 2;
  string text                    = 3;
  int64 sent_at_ms                = 4;
}
message SendChatResponse {
  ResultCode result_code = 1;
}
```

## 9.3 AdminService

```protobuf
message BanAccountRequest {
  string player_id    = 1;
  string reason         = 2;
  int64 expires_at_ms    = 3;   // 0表示永久封禁(proto3不区分未设置与0，与RGS-DTL-001§4.3 CommitTransactionRequest raw_value同款约定)
  string operator_id      = 4;
}
message BanAccountResponse {
  string ban_id           = 1;
  ResultCode result_code   = 2;
}

message GrantCompensationRequest {
  repeated string character_ids = 1;
  string item_template_id        = 2;
  int32 quantity                   = 3;
  string reason                     = 4;
}
message GrantCompensationResponse {
  int64 batch_id            = 1;   // 对应§8 compensation_batches.batch_id
  ResultCode result_code     = 2;
}

message SetMaintenanceModeRequest {
  bool enabled          = 1;
  string message          = 2;
  string operator_id       = 3;
}
message SetMaintenanceModeResponse {
  ResultCode result_code = 1;
}
```

`ResultCode`枚举复用§4.4已定义的全服务通用枚举，不为三个新服务另行定义。

---

# 10. §4.6〜4.8算法详细设计

对应RGS-BAS-001§4.6（MT／GD概要）、§4.7（EV／WF）、§4.8（OB／AD），落实为可翻译为Rust实现的伪代码。RGS-BAS-001§4.6原文声明"仅模块划分，处理时序留PH-5/PH-6开始前补充"——本节仅落实**已经在§8.3 ST-002状态机与RGS-BAS-001§4.7〜4.8流程图/时序图中给出的部分**，不越权替BAS-001做§4.6尚未做出的处理时序决策（社交模块聊天/公会的完整处理时序仍留待，见§13）。

## 10.1 对局状态机驱动逻辑（落实ST-002与RGS-BAS-001§8状态迁移详细设计中的触发来源表）

```rust
// 场景Actor判定对局结束条件成立后调用(Running→Finished)，对应§6状态迁移SQL
fn on_match_finished(match_id: MatchId, expected_version: i64) -> Result<(), MatchError> {
    let rows = exec_occ_update(
        "UPDATE matches SET status='Finished', finished_at=now(), version=version+1
         WHERE match_id=$1 AND status='Running' AND version=$2",
        (match_id, expected_version),
    )?;
    if rows == 0 {
        // 并发的重复结束判定，或已被其他路径(如强制终止)修改状态：重新读取当前状态，
        // 若已是Finished/Archived则视为幂等成功，不重复报错；否则记录异常供人工核查
        return reconcile_unexpected_state(match_id);
    }
    Ok(())
}

// Finished→Archived，须等待§4.5.1确定请求机制完成奖励发放后才可迁移(match_results.rewards_granted=true)
fn on_settlement_completed(match_id: MatchId, expected_version: i64) -> Result<(), MatchError> {
    // 前置校验：结算与奖励发放是两个独立事务(match_results写入 vs matches状态迁移)，
    // 顺序不可颠倒——必须先确认match_results.rewards_granted=true，再迁移matches.status，
    // 避免"已归档但奖励未发"的不可挽回状态(同RGS-BAS-001§4.5.2"不得虚构已确定结果"精神)
    if !query_rewards_granted(match_id)? {
        return Err(MatchError::RewardsNotYetGranted);
    }
    let rows = exec_occ_update(
        "UPDATE matches SET status='Archived', version=version+1
         WHERE match_id=$1 AND status='Finished' AND version=$2",
        (match_id, expected_version),
    )?;
    if rows == 0 { return reconcile_unexpected_state(match_id); }
    Ok(())
}
```

## 10.2 事件工作流：Outbox分发器（落实RGS-BAS-001§4.7.1流程图）

```rust
// 分发器周期性调用，对应ARC-009/010
fn outbox_dispatch_cycle(db: &OutboxTable) -> Result<(), DispatchError> {
    let pending = db.select_pending();  // WHERE published_at IS NULL

    // 按aggregate_id分组，组内保序发布(ARC-010)，组间可并行
    let groups = group_by_aggregate_id(pending);
    for (aggregate_id, events) in groups {
        for event in events {  // 组内严格按序，不并行发同一aggregate的事件
            match publish_to_event_bus(&event, /*partition_key=*/&aggregate_id) {
                Ok(()) => db.mark_published(event.outbox_id, now()),
                Err(_) => {
                    // 保留published_at=NULL，本轮不再处理该aggregate后续事件(保序要求)，
                    // 下一轮重试；消费者侧幂等吸收因重试产生的重复投递(ARC-009)
                    break;
                }
            }
        }
    }
    Ok(())
}
```

## 10.3 购买工作流Saga状态转移（落实RGS-BAS-001§4.7.2时序图）

```rust
enum PurchaseState { Initiated, PaymentPending, PaymentCompleted, PaymentFailed,
                      Delivered, DeliveryFailed, Refunding, Refunded, Completed }

fn purchase_saga_step(wf: &mut PurchaseWorkflow, event: PurchaseEvent) -> Result<(), SagaError> {
    match (wf.state, event) {
        (PurchaseState::Initiated, PurchaseEvent::Start) => {
            wf.state = PurchaseState::PaymentPending;
            request_payment(wf.request_id, wf.amount)?;
        }
        (PurchaseState::PaymentPending, PurchaseEvent::PaymentSucceeded) => {
            wf.state = PurchaseState::PaymentCompleted;
            // 发货请求携带同一request_id体系(幂等)，复用§4.5.1确定请求机制
            request_delivery(wf.request_id, &wf.item)?;
        }
        (PurchaseState::PaymentPending, PurchaseEvent::PaymentFailedOrExpired) => {
            wf.state = PurchaseState::PaymentFailed;   // 终态，无需补偿(未曾发货)
        }
        (PurchaseState::PaymentCompleted, PurchaseEvent::DeliverySucceeded) => {
            wf.state = PurchaseState::Delivered;
            wf.state = PurchaseState::Completed;
        }
        (PurchaseState::PaymentCompleted, PurchaseEvent::DeliveryFailed) => {
            wf.state = PurchaseState::DeliveryFailed;
            if wf.delivery_retry_count < MAX_DELIVERY_RETRIES {
                wf.delivery_retry_count += 1;
                request_delivery(wf.request_id, &wf.item)?;   // Activity级重试，状态不变
            } else {
                // 重试耗尽：进入补偿路径，发货最终判定失败，必须退款
                wf.state = PurchaseState::Refunding;
                request_refund(wf.request_id, wf.amount)?;
            }
        }
        (PurchaseState::Refunding, PurchaseEvent::RefundCompleted) => {
            wf.state = PurchaseState::Refunded;   // 终态：购买失败已退款
        }
        (state, event) => {
            // 非法迁移(如已Completed状态收到DeliveryFailed)：拒绝并告警，不静默忽略，
            // 这类情况意味着上游事件重复投递到了已终结的工作流实例，需人工核查而非自动吞掉
            return Err(SagaError::IllegalTransition { state, event });
        }
    }
    Ok(())
}
```

## 10.4 可观测性：Trace传播字段的具体落位（落实RGS-BAS-001§4.8.1表格为可执行结构）

```rust
// 各阶段trace_id的具体读写点，对应§4.8.1表格逐行落实
struct TraceContext {
    trace_id: TraceId,
    span_id: SpanId,
}

// 网关→内部gRPC：标准W3C Trace Context header，不新增自定义头（复用既有otel库，不重新实现propagator）
fn propagate_grpc(ctx: &TraceContext, req: &mut GrpcRequest) {
    req.metadata.insert("traceparent", ctx.to_w3c_traceparent());
}

// 业务服务→PostgreSQL：trace_id作为outbox表列持久化，随事件继续传播(DR-013)
fn build_outbox_row(ctx: &TraceContext, aggregate_id: &str, payload: &[u8]) -> OutboxRow {
    OutboxRow { trace_id: ctx.trace_id, aggregate_id: aggregate_id.into(), payload: payload.into(), published_at: None }
}

// 事件消费者：从事件header取出trace_id延续span，而非开启全新根span(否则链路断裂)
fn consume_event(event: &BusEvent) -> TraceContext {
    TraceContext { trace_id: event.header.trace_id, span_id: SpanId::new_child_of(event.header.trace_id) }
}
```

指标采集拓扑（§4.8.2）本身不含算法级细节（OTLP推送/暴露是标准库行为，非本项目自定义逻辑），故本节不重复展开为伪代码，仅在此明确：`emit_metric`（本文档§5.1 tick循环已使用的既有调用点）与本节`TraceContext`共享同一OTel SDK实例，两者不是两套独立的可观测性接入路径。

---

# 11. 错误码一览

| `ResultCode` | 触发条件 | 对应既有设计 |
|---|---|---|
| `OCC_CONFLICT` | `expected_version`与数据库当前`version`不一致 | RGS-BAS-001§4.5.1（OCC更新，受影响行数=0） |
| `STALE_SESSION_EPOCH` | 请求携带的`session_epoch`低于该角色当前生效值 | ARC-005 Single-Writer保证 |
| `ACCOUNT_BANNED` | `accounts.status`处于封禁态且存在生效中的`ban_records` | FR-AD-001 |
| `INSUFFICIENT_BALANCE` | `ConsumeCurrency`请求的扣减量超过当前`wallets.balance` | FR-EC-002 |
| `DUPLICATE_REQUEST_ID` | 幂等键已存在但**请求内容与首次不一致**（正常重放应命中相同`payload`，直接返回原`ledger_id`而非此错误——本错误码专指内容冲突这一异常情形） | §3.2确定请求物理执行语义 |
| `REWARDS_NOT_YET_GRANTED` | `on_settlement_completed`调用时`match_results.rewards_granted`仍为`false` | §10.1，防止"已归档但奖励未发"不可挽回状态 |
| `ILLEGAL_SAGA_TRANSITION` | 购买工作流收到与当前状态不匹配的事件（如已`Completed`收到`DeliveryFailed`） | §10.3，通常意味着事件重复投递到已终结的工作流实例 |

---

# 12. 跨库标识映射（v0.3新增，解决与RGS-DTL-025/026的主键风格不一致）

`player_db`（§2/§3使用`UUID`主键，`player_id`/`character_id`）与`match_db`/`admin_db`（RGS-DTL-025/026独立选型均为`BIGINT`）之间的类型差异，最终决定为：**保留`player_db`自身的`UUID`主键不变**（不做迁移——`UUID`作为对外暴露标识符已有其自身价值：不因枚举而泄露账号规模、可在客户端本地生成幂等操作的关联ID等），同时在`accounts`/`characters`表新增`player_seq`/`character_seq`两个`BIGSERIAL`列（§2.1已加入）作为**权威数值身份**，供`match_db`/`admin_db`等以`BIGINT`为主键风格的库直接引用。

**决策依据**：本项目尚处需求/设计阶段，无生产数据，"迁移`player_db`本身DDL到BIGINT"不涉及真实数据迁移风险——但即便如此，仍选择"新增映射列"而非"改类型"，因为：(a) `UUID`主键在`player_db`内部已被大量FK/索引依赖（`characters.player_id`、`ban_records.player_id`等），全面改型影响面大于新增一列；(b) `match_db`/`admin_db`两份文档已发布，反过来改它们的`BIGINT`风格成本相同甚至更高（两份文档、更多下游引用）；(c) `UUID`对外暴露、`BIGINT`内部高频关联各有其适用场景，允许两者并存、以显式映射衔接，是比强行统一为单一类型更贴合实际需求的设计。

**映射机制**（具体、非"应用层处理"式的空泛表述）：

```sql
-- player_db.accounts.player_seq 与 characters.character_seq 已是BIGSERIAL UNIQUE(§2.1)，
-- match_db/admin_db的BIGINT外键(如RGS-DTL-025 detection_signals.player_id、
-- RGS-DTL-026 match_ratings.character_id)在语义上直接对应这两列的值，
-- 而非另建一张独立的跨库映射表——BIGSERIAL本身就是权威映射源，无需额外维护一份影子映射数据
```

```rust
// 应用层在任何跨库写入路径中，统一从player_db查询获得*_seq值后再写入match_db/admin_db，
// 不在match_db/admin_db内部自行生成或猜测BIGINT值
async fn resolve_character_seq(character_id: Uuid, player_db: &PlayerDbPool) -> Result<i64, DbError> {
    sqlx::query_scalar!("SELECT character_seq FROM characters WHERE character_id = $1", character_id)
        .fetch_one(player_db)
        .await
}
// RGS-DTL-025/026涉及character_id/player_id写入admin_db/match_db的路径，均在写入前调用本函数
// （或等价的批量版本），确保BIGINT值的唯一权威来源始终是player_db，杜绝"两边各自维护一份编号"的分叉风险
```

**反向查询**（`match_db`/`admin_db`记录关联回`player_db`的`UUID`，如GM后台按`BIGINT`案件记录查询玩家详情时）：`character_seq`/`player_seq`列已建`UNIQUE`索引（§2.1`BIGSERIAL NOT NULL UNIQUE`隐含唯一索引），反向查询`SELECT character_id FROM characters WHERE character_seq = $1`同样是O(1)索引查询，无需额外维护反向映射表。

---

# 13. 本文档的覆盖范围与后续计划

本文档v0.1是本项目**第一份详细设计文档**，覆盖范围曾**刻意限定**在核心架构中最基础的两个限界上下文（player_db／economy_db）与最核心的两个算法（tick循环／AOI计算）。v0.2补齐了v0.1自己声明的遗留缺口：

- match_db核心表（`matches`/`match_participants`/`match_results`，§6）／social_db（`friend_links`/`guilds`/`guild_members`，§7）／admin_db核心表（`operation_audits`/`compensation_batches`，§8）物理DDL
- MatchService／SocialService／AdminService协议线格式细化（§9）
- §4.6〜4.8中**已有明确流程图/时序图/状态机依据**的部分：对局状态机（ST-002）、事件工作流Outbox分发器、购买Saga补偿路径、Trace传播字段落位（§10）

v0.3（负责人指示"开子代理完成剩余的"）补齐：

- 跨库标识映射机制（§12），解决v0.2自述的UUID/BIGINT风格不一致遗留问题

**仍明确不覆盖、留待后续**：

- RGS-BAS-001§4.6原文本身声明"社交模块（好友/聊天/公会）仅模块划分，处理时序留PH-6开始前补充"——BAS-001尚未给出该部分的流程图/时序图，本文档不能在父文档未决策的情况下自行编造处理时序，故§7的social_db DDL仅是数据结构落地，**不含**好友申请/聊天/公会权限变更的算法级处理逻辑，须等RGS-BAS-001自身先补充§4.6社交处理时序后，本文档再跟进一版
- §8.1指标采集拓扑（RGS-BAS-001§4.8.2）本身不含本项目自定义算法，故未展开为伪代码，见§10.4末尾说明
- 全部其余业务域（RGS-REQ-006〜030对应的BAS-002〜027中，尚无对应DTL文档者）的详细设计

**后续计划**（留待负责人确认优先级排期，本文档不代为决定）：按既有RGS-REQ-001§11.2.1"领域文档工作的阶段归属表"的PH-1〜PH-8顺序，逐域产出对应DTL文档。RGS-BAS-001§4.6社交处理时序的补齐，建议作为本文档下一版本的优先输入。

---

# 14. 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-001§5.3 player_db逻辑模型 | §2 |
| RGS-BAS-001§5.4 economy_db逻辑模型 | §3 |
| RGS-BAS-001§6.3.1〜6.3.2 PlayerService/EconomyService字段设计 | §4 |
| RGS-BAS-001§4.2.2 tick循环流程图 | §5.1 |
| RGS-BAS-001§4.3.1 AOI算法（G-003无饿死性） | §5.2 |
| RGS-BAS-001§5.5 match_db逻辑ER图 | §6 |
| RGS-REQ-001§8.3 ST-002对局状态机 | §6、§10.1 |
| RGS-BAS-001§5.6 social_db逻辑ER图 | §7 |
| RGS-BAS-001§5.7 admin_db逻辑ER图（NFR-SE-010仅追加约束） | §8 |
| RGS-BAS-001§6.3.3 MatchService字段占位 | §9.1 |
| RGS-BAS-001§6.3类图 SocialService方法签名 | §9.2 |
| RGS-BAS-001§6.3.4 AdminService字段设计 | §9.3 |
| RGS-BAS-001§4.7.1 Outbox分发器流程图 | §10.2 |
| RGS-BAS-001§4.7.2 购买工作流时序（含补偿路径） | §10.3 |
| RGS-BAS-001§4.8.1 Trace传播载体设计表 | §10.4 |
| RGS-BAS-001§4.8.2 指标采集拓扑 | §10.4（声明不展开为伪代码的理由） |
| RGS-DTL-025§2 admin_db反作弊三表（本文档§8核心表与其同库） | §8、§12（跨库标识映射机制） |
| RGS-DTL-026§2 match_db匹配三表（本文档§6核心表与其同库） | §6、§12（跨库标识映射机制） |
| RGS-BAS-001§5.4.3 权威源分级 Tier-1/Tier-2（per RGS-ADR-0057 §2.1，v0.6 同步父 BAS-001 v1.4） | §3.4 |
| RGS-BAS-001§7.2.1 背压设置点八边界 + 死锁防止证明（ARC-013，v0.6 同步父 BAS-001 v1.3） | §5.3 |
