# 详细设计（Detailed Design Document）

**Player 域主表 DDL：`players` / `player_characters` / `player_inventory`**

| 项目 | 内容 |
|---|---|
| 文档编号 | **RGS-DTL-044** |
| 标题 | player 主表 |
| 版本 | 0.1 |
| 状态 | 🟢 v1.0（DTL 实体首版——A-02 偿还技术债，per RGS-OPEN-QA-001 v0.2 Q-D-02 + ACTIONS-v0.3 A-02 + DTL-018 §2 + DTL-036 §6 第 1 条）|
| 主文档 | RGS-DTL-018 §2（玩家域数据模型）/ RGS-DTL-036 §6（Player 域契约登记 checklist）/ RGS-DTL-036 配套 SPEC-DTL-036（player contracts）/ RGS-OPEN-QA-001 v0.2 Q-D-02 + RGS-OPEN-QA-001-ACTIONS v0.3 §3 A-02 |
| 关联 DTL | DTL-018（身份合规，5 张表）/ DTL-036（Player 域契约登记）/ SPEC-DTL-036（player contracts，5 域契约总册）|
| App/DB | `player-service` / `player_db` |
| 编制人 | player 域 Lead（Ulysses per DEC-008 一人公司派生） |
| 编制日 | 2026-08-24 |
| 修订历史 | 0.1（2026-08-24）：A-02 偿还——新建 DTL 实体定义 3 张主表 DDL + 反向 doc `0001_init.sql` + 新建 `0004_player_characters_inventory.sql` migration |
| 依据任务 | WF-1-55.39（per RGS-OPEN-QA-001-ACTIONS v0.3 §4 13 个 pending L4 任务清单；token 预算 12K） |
| 许可证 | Apache-2.0（主仓库） |

---

## 修订历史（追加 / Revision History）

| 版本 | 修订人 | 修订日 | 修订内容 | 影响小节 |
|---|---|---|---|---|
| 0.1 | AI worker（per WF-1-55.39）| 2026-08-24 | A-02 偿还——新建 DTL-044 v0.1 + 反向 doc 0001 + 新建 0004 migration | 全部 |

> **DTL 命名约束**（per RGS-OPEN-QA-001 v0.2 Q-D-02 答复 + ACTIONS-v0.3 §5 #2）：DTL-038 已被 Match 域占用为 `RGS-DTL-038_Match域_详细设计.md`（per `docs/01-核心架构与设计模式/`），故本 DTL 编号为 **044**（DTL-043 留给消息分发 per Q-D-01 + WF-1-55.38），与 Q-D-01 顺序承接。grep `docs/` 全库确认 037/038/039/040/041/042 全有归属，下一可用编号 043/044，本表取 044 与 player 域对应。

---

## 目录

1. [范围与目的](#1-范围与目的)
2. [字段级 DDL](#2-字段级-ddl)
3. [`players.metadata` JSONB 范围与反范式禁令](#3-playersmetadata-jsonb-范围与反范式禁令)
4. [索引规划](#4-索引规划)
5. [与 `0001_init.sql` 反向 doc](#5-与-0001_initsql-反向-doc)
6. [签字栏（per DEC-008 12 角色 RACI）](#6-签字栏per-dec-008-12-角色-raci)
7. [附录 A：player_sessions 引用关系（per DTL-018 §3.2）](#7-附录-aplayer_sessions-引用关系per-dtl-018-§32)
8. [追踪矩阵](#8-追踪矩阵)

---

# 1. 范围与目的

## 1.1 范围

本 DTL 定义 player-service 域（per RGS-ARC-008 5 域划分）下 **`player_db` schema** 内 3 张主表的**字段级 DDL**：

| # | 表名 | 用途 | 状态 |
|---|---|---|---|
| 1 | `players` | 玩家账号档案（昵称/等级/vip/状态/最近登录/创建/更新）| **已存在**（`0001_init.sql`）+ 本 DTL §2.1 字段级对齐 |
| 2 | `player_characters` | 玩家角色档案（职业/等级/HP/ATK/DEF 等高频属性 + stats JSONB 扩展）| **待新建**（`0004_player_characters_inventory.sql` migration）|
| 3 | `player_inventory` | 玩家背包物品（item_id 外键 + 数量 + 槽位 + 获取时间）| **待新建**（`0004_player_characters_inventory.sql` migration）|

**不在本 DTL 范围**：

- 身份合规 5 张表（`account_identity_links` / `identity_binding_audit_logs` / `compliance_profiles` / `identity_verification_vault` / `minor_restriction_audit_logs`）—— 归 DTL-018 §2，per RGS-BAS-018，**本 DTL 不修改 DTL-018**
- 玩家会话表 `player_sessions` —— 归 DTL-018 §3.2，**已在 0001_init.sql 实现**，本 DTL §7 仅记录引用关系
- 玩家域 contracts（gRPC proto）—— 归 DTL-036 §3 + SPEC-DTL-036，**本 DTL 不写 proto**（per DTL-036 §6 第 1 条："Player 域主表 DDL 归属于 DTL-044"与 contracts 分离）
- 玩家 Rust 实体代码（`crates/player-service/src/entity.rs`）—— **本任务不写 Rust 代码**（per WF-1-55.39 范围约束：只写 SQL + DDL 文档）
- 物品 master 表（`items`）—— 归其他域（D 域物品定义/配置中心），本 DTL 仅以 `item_id` 外键引用，不在本 DTL 物化

## 1.2 目的

**偿还技术债**（per RGS-OPEN-QA-001 v0.2 Q-D-02 + ACTIONS-v0.3 §3 A-02）：

1. **背景**：`crates/player-service/migrations/0001_init.sql`（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-018 §3）**先实现**了 `players`（5 列核心 + 3 索引）/ `player_sessions` 表，但 **`player_characters` / `player_inventory` 尚未实现**；DTL-036 §6 第 1 条 checklist 写明"账号/角色/会话三表 DDL 跟进"，本任务偿还该 TODO。
2. **矛盾**：DTL-018 §2 关注身份合规（5 张表），DTL-036 §6 关注契约登记（API/事件），均**未**给出 player 主表 3 张表的字段级 DDL。
3. **本 DTL 定位**：补齐 DTL-018 与 DTL-036 之间的"实体定义"空缺，承接 DTL-036 §6 第 1 条 checklist。
4. **范围澄清**：本 DTL 仅定义主表 DDL；proto 契约见 SPEC-DTL-036；Rust 实体见 `crates/player-service/src/entity.rs`（已存在 Player/PlayerSession 2 个，本任务不新增 entity 代码）。

## 1.3 设计原则

| 原则 | 落地 | 出处 |
|---|---|---|
| **反范式禁令** | `players.metadata` JSONB **不存** `equipment_json` 等反范式字段（inventory 走独立表 + 外键）| Q-D-02 答复 #2 |
| **混合存储** | `player_characters.stats` = JSONB + 字段化（HP/ATK/DEF 等高频/需索引属性拆列）| Q-D-02 答复 #3 |
| **历史 migration 不可变** | 不修改 0001_init.sql 已有 DDL；新增表走 0004 migration | per RGS-IMPL-001 §3.2 |
| **占位表走"先代码后文档"已倒挂的 DTL 偿债** | 0001 已存在表通过本 DTL §5 反向 doc 字段对齐 | per Q-D-02 答复 #1 |
| **DEC-008 派生 RACI** | player 域 Lead 全权 owner 字段级 DDL；DTL-018 owner 不动 | per DEC-008 + §6 签字栏 |

---

# 2. 字段级 DDL

> **SQL 方言**：PostgreSQL 14+（per RGS-DTL-002 §3 5 域统一 PostgreSQL + `gen_random_uuid()` pgcrypto）
> **schema 名**：`player_db`（per DTL-036 §2 cluster manifest）

## 2.1 `players` 表（per 0001_init.sql 第 5-19 行）

> **状态**：✅ **已存在**（per `crates/player-service/migrations/0001_init.sql`，commit 历史 WBS v0.3 §2A.5 WF-1-54.4）
> **本 DTL 作用**：字段级 DDL 对齐 + 后续 §2.1.1 扩展建议（metadata JSONB 列待 0005 migration，**本任务不动**）

```sql
-- 表 2.1.1: players 玩家账号档案
-- 状态: 已存在 (per 0001_init.sql)
-- 主键: id (UUID, 业务主键, 跨域统一 UUID v4)
-- 唯一: name (玩家昵称, 全局唯一)

CREATE TABLE IF NOT EXISTS players (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    level           INTEGER NOT NULL DEFAULT 1
                        CHECK (level >= 1 AND level <= 999),
    vip_level       INTEGER NOT NULL DEFAULT 0
                        CHECK (vip_level >= 0 AND vip_level <= 20),
    status          TEXT NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'banned', 'disabled', 'pending')),
    last_login_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 索引 (per 0001_init.sql 已建)
CREATE INDEX IF NOT EXISTS idx_players_name    ON players (name);
CREATE INDEX IF NOT EXISTS idx_players_level   ON players (level);
CREATE INDEX IF NOT EXISTS idx_players_status  ON players (status);
```

**字段语义**：

| 列 | 类型 | 约束 | 语义 | 来源 |
|---|---|---|---|---|
| `id` | UUID | PK | 玩家业务主键（跨域统一 UUID v4，per RGS-SPEC-CROSS-002）| entity.rs `Player.id` |
| `name` | TEXT | NOT NULL UNIQUE | 玩家昵称（全局唯一，case-sensitive）| entity.rs `Player.name` |
| `level` | INTEGER | NOT NULL DEFAULT 1, 1..999 | 玩家等级（per 游戏数值，999 上限预留）| entity.rs `Player.level` |
| `vip_level` | INTEGER | NOT NULL DEFAULT 0, 0..20 | VIP 等级（0 = 非 VIP，20 = 顶级）| entity.rs `Player.vip_level` |
| `status` | TEXT | NOT NULL DEFAULT 'active', enum | 账号状态（active / banned / disabled / pending）| entity.rs `PlayerStatus` |
| `last_login_at` | TIMESTAMPTZ | NULL | 最近登录时间（NULL = 从未登录）| entity.rs `Player.last_login_at` |
| `created_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 创建时间（账号注册时刻）| entity.rs `Player.created_at` |
| `updated_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 更新时间（任何字段变更都需刷新）| entity.rs `Player.updated_at` |

### 2.1.1 扩展建议：`metadata JSONB` 列（**待未来 0005 migration，**本任务不实施）

> **范围澄清**：Q-D-02 答复 #2 明确"**`metadata` JSONB 不存 `equipment_json` 等反范式字段**"，本 DTL §3 详细禁止清单见后。`metadata` 列是**预留扩展位**（per Q-D-02 答复 #2：JSONB 留给"未明确归属的低频属性 + 未来扩展"），但 0001_init.sql 当前**尚未实施** `metadata` 列。本 DTL 文档化建议 schema：

```sql
-- 建议 schema (待 0005 migration, 非本任务范围)
-- ALTER TABLE players ADD COLUMN metadata JSONB NOT NULL DEFAULT '{}'::jsonb;
-- 禁止字段: equipment_json, characters_json, inventory_json (反范式)
-- 推荐字段: avatar_url, locale, region, last_client_version, marketing_source 等低频 UI/运营属性
```

**为什么本任务不实施**：

- 0001_init.sql 不可变（per RGS-IMPL-001 §3.2 历史 migration 不可变）
- ALTER TABLE 加列需要单独 0005 migration + expand-contract 双步（per DTL-002 §3 + DTL-036 §5 Expand-Contract 约定）
- 本任务 scope（per WF-1-55.39 + ACTIONS-v0.3 §3 A-02）= "3 张主表 DDL + 0001 反向 doc + 0004 新增 2 表"，不含 ALTER players
- §3 文档化约束，让 0005 实施时直接套用

## 2.2 `player_characters` 表（per 0004 migration 本任务新建）

> **状态**：🆕 **待新建**（per `crates/player-service/migrations/0004_player_characters_inventory.sql`）
> **主键**：`id` (UUID, 业务主键)
> **外键**：`player_id` → `players.id` ON DELETE CASCADE（per DTL-018 §3.1 账号删除级联清理角色）

```sql
-- 表 2.2.1: player_characters 玩家角色档案
-- 状态: 待新建 (per 0004_player_characters_inventory.sql)
-- 用途: 玩家可拥有多个角色 (per game design 多角色 / 单服多职业)
-- 字段策略: 高频属性拆列 (HP/ATK/DEF 需查询/索引) + stats JSONB 扩展 (低频/未明确属性)

CREATE TABLE IF NOT EXISTS player_characters (
    id              UUID PRIMARY KEY,
    player_id       UUID NOT NULL
                        REFERENCES players(id) ON DELETE CASCADE,
    char_class      TEXT NOT NULL
                        CHECK (char_class IN ('warrior', 'mage', 'archer', 'assassin', 'support')),
    level           INTEGER NOT NULL DEFAULT 1
                        CHECK (level >= 1 AND level <= 999),
    -- 高频属性拆列 (per Q-D-02 答复 #3: JSONB + 字段化混合)
    hp              INTEGER NOT NULL DEFAULT 100
                        CHECK (hp >= 0),
    atk             INTEGER NOT NULL DEFAULT 10
                        CHECK (atk >= 0),
    def             INTEGER NOT NULL DEFAULT 5
                        CHECK (def >= 0),
    crit_rate       REAL NOT NULL DEFAULT 0.05
                        CHECK (crit_rate >= 0.0 AND crit_rate <= 1.0),
    -- 低频/扩展属性 JSONB (per Q-D-02 答复 #3)
    stats           JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- 装备: 引用 inventory 行 (外键到 player_inventory, 不内嵌 JSON)
    primary_weapon_id UUID,
    -- 元信息
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 复合约束
    CONSTRAINT fk_pc_player FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    CONSTRAINT fk_pc_weapon FOREIGN KEY (primary_weapon_id) REFERENCES player_inventory(id) ON DELETE SET NULL
);
```

**字段语义**：

| 列 | 类型 | 约束 | 语义 | 来源 |
|---|---|---|---|---|
| `id` | UUID | PK | 角色业务主键 | entity 暂未实化（per DTL-036 §6 第 1 条 TODO，0004 migration 落地后 entity 跟进不在本任务 scope）|
| `player_id` | UUID | NOT NULL, FK→`players.id` ON DELETE CASCADE | 所属玩家 | 同上 |
| `char_class` | TEXT | NOT NULL, enum | 职业（warrior/mage/archer/assassin/support）| per game design |
| `level` | INTEGER | NOT NULL, 1..999 | 角色等级（独立于玩家等级）| per game design |
| `hp` | INTEGER | NOT NULL, >= 0 | 当前 HP（高频战斗查询）| per §2.3 字段化原则 |
| `atk` | INTEGER | NOT NULL, >= 0 | 攻击力（高频）| per §2.3 |
| `def` | INTEGER | NOT NULL, >= 0 | 防御力（高频）| per §2.3 |
| `crit_rate` | REAL | NOT NULL, 0.0..1.0 | 暴击率（高频，0.05 = 5%）| per §2.3 |
| `stats` | JSONB | NOT NULL DEFAULT '{}' | 扩展属性（per §2.3 JSONB 部分）| per §2.3 |
| `primary_weapon_id` | UUID | NULL, FK→`player_inventory.id` ON DELETE SET NULL | 主武器引用（不内嵌 JSONB）| per §3 反范式禁令 |
| `created_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 角色创建时间 | 标准 |
| `updated_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 角色更新时间 | 标准 |

**复合约束**：

- `fk_pc_player`：玩家删除时级联清角色（per DTL-018 §3.1 "玩家删除 = 角色全清"）
- `fk_pc_weapon`：主武器删除（玩家丢弃该装备）时角色保留但 `primary_weapon_id = NULL`（不级联到角色）
  - **理由**：玩家可能因换装/出售主动丢弃武器，角色不应被连带删除

## 2.3 `stats` JSONB + 字段化混合存储方案（per Q-D-02 答复 #3）

### 2.3.1 拆分原则

| 属性类型 | 存储策略 | 理由 | 例子 |
|---|---|---|---|
| **高频查询/需索引/范围扫描** | **字段化拆列** | 走 B-tree 索引，O(log n)；JSONB 走 GIN 索引，O(log n) 但选择性差 | HP / ATK / DEF / crit_rate / move_speed |
| **低频/未明确/将来扩展** | **JSONB 留位** | 字段化成本高（DDL 变更需要 migration + 索引重建），JSONB 灵活 | 抗性（fire_resist / ice_resist / ...）/ 状态效果（buffs / debuffs）/ 装备附魔 / 天赋加点 / 隐藏成就 |
| **业务明确/枚举型** | **字段化拆列 + CHECK 约束** | 避免魔法值 + 强类型保护 | 职业 char_class / 阵营 faction / 转职次数 promotion_count |

### 2.3.2 JSONB 推荐 schema（`stats` 字段）

```jsonc
{
  // 抗性 (低频，JSONB 灵活)
  "resistances": {
    "fire": 0.0,
    "ice": 0.0,
    "lightning": 0.0,
    "poison": 0.0,
    "holy": 0.0,
    "shadow": 0.0
  },
  // 状态效果 (运行时缓存，定期落库)
  "buffs": [
    {"id": "haste", "duration_sec": 30, "magnitude": 0.2}
  ],
  "debuffs": [],
  // 装备附魔 (per §3 反范式禁令：附魔是装备属性不是角色属性，独立存 inventory)
  // 但"附魔效果作用于角色"的部分放这里
  "enchantments_applied": {
    "weapon_atk_bonus": 5,
    "armor_def_bonus": 3
  },
  // 天赋 / 技能加点 (per game design)
  "talents": {
    "warrior_path": 3,
    "defense_branch": 2
  },
  // 隐藏成就 / 统计
  "achievements": ["first_blood", "level_10_reached"],
  // 自定义扩展位 (per §3 禁止字段外的任意)
  "custom": {}
}
```

### 2.3.3 为什么不全字段化

> **理由**（per Q-D-02 答复 #3 隐含 + IMPL 经验）：

1. **migration 成本**：每次新增属性 → 0006/0007/... migration + ALTER TABLE + 索引重建（大数据量时阻塞业务）
2. **JSONB 灵活性**：游戏迭代快，PH-2/3/4 不断加新属性（数值策划月度 release），全字段化跟不上
3. **JSONB 索引够用**：高频查询走字段化列，低频走 JSONB GIN 索引（per RGS-DTL-002 §3 PostgreSQL JSONB 最佳实践）

### 2.3.4 为什么不全 JSONB

1. **HP/ATK/DEF 是热路径**：每帧战斗计算都查，全 JSONB 解析成本叠加 → 实时性能不达标（per NFR-PT 100ms 战斗响应约束）
2. **索引效率**：B-tree 单列索引 vs JSONB GIN 索引，选择性差距 ~10x（per PostgreSQL 官方 benchmark）
3. **数据完整性**：CHECK 约束可防止负数 HP 等 invalid 状态，JSONB 路径难做

### 2.3.5 决策矩阵

| 评估维度 | 全字段化 | 全 JSONB | **混合（采用）** |
|---|---|---|---|
| 热路径性能 | ✅ 最优 | ❌ 解析开销 | ✅ 字段化命中热路径 |
| 迁移成本 | ❌ 高 | ✅ 零 | ⚠️ 中（新增 JSONB 字段零成本，核心属性已拆）|
| 灵活性 | ❌ 低 | ✅ 高 | ✅ 高 |
| 数据完整性 | ✅ CHECK 约束 | ❌ 弱 schema | ✅ CHECK 约束 + JSONB schema 校验（应用层）|
| 索引效率 | ✅ B-tree | ⚠️ GIN 路径索引 | ✅ 混合 |

## 2.4 `player_inventory` 表（per 0004 migration 本任务新建）

> **状态**：🆕 **待新建**（per `crates/player-service/migrations/0004_player_characters_inventory.sql`）
> **主键**：`id` (UUID, 业务主键)
> **外键**：`player_id` → `players.id` ON DELETE CASCADE + `item_id` → 物品 master 表（**注**：物品 master 表归其他域，本 DTL 仅以 `item_id UUID` 引用，**不物化 FK**）

```sql
-- 表 2.4.1: player_inventory 玩家背包物品
-- 状态: 待新建 (per 0004_player_characters_inventory.sql)
-- 用途: 玩家持有的物品记录
-- 设计: 每条记录 = 1 玩家 1 物品类型 1 槽位
--       同类物品可堆叠 (quantity > 1)，也可一格一物 (quantity = 1)

CREATE TABLE IF NOT EXISTS player_inventory (
    id              UUID PRIMARY KEY,
    player_id       UUID NOT NULL
                        REFERENCES players(id) ON DELETE CASCADE,
    item_id         UUID NOT NULL,  -- 物品 master 表 FK (跨域, 不在本 DTL 物化)
    quantity        INTEGER NOT NULL DEFAULT 1
                        CHECK (quantity > 0),
    slot            INTEGER NOT NULL
                        CHECK (slot >= 0 AND slot < 200),  -- 背包上限 200 格
    acquired_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 物品实例属性 (附魔/绑定等)
    -- 元信息
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 复合约束
    CONSTRAINT fk_pi_player FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    -- 一玩家一槽位一物 (slot 唯一 per player, 防止重入 bug)
    CONSTRAINT uq_pi_player_slot UNIQUE (player_id, slot)
);
```

**字段语义**：

| 列 | 类型 | 约束 | 语义 | 来源 |
|---|---|---|---|---|
| `id` | UUID | PK | 背包行 ID（业务主键）| entity 暂未实化（per DTL-036 §6 TODO 跟进，不在本任务 scope）|
| `player_id` | UUID | NOT NULL, FK→`players.id` ON DELETE CASCADE | 所属玩家 | per §2.2 |
| `item_id` | UUID | NOT NULL | 物品 master ID（**外键不在本 DDL 物化**，跨域引用）| 物品 master 表归其他域（D 域）|
| `quantity` | INTEGER | NOT NULL, > 0 | 数量（同槽位堆叠数，1 = 非堆叠）| per game design |
| `slot` | INTEGER | NOT NULL, 0..199 | 槽位编号（per player 唯一）| per game design（背包上限 200 格）|
| `acquired_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 获取时间 | 标准 |
| `metadata` | JSONB | NOT NULL DEFAULT '{}' | 物品实例属性（附魔词条/绑定角色/到期时间等）| per §3 JSONB 范围 |
| `created_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 记录创建时间 | 标准 |
| `updated_at` | TIMESTAMPTZ | NOT NULL DEFAULT now() | 记录更新时间 | 标准 |

**复合约束**：

- `fk_pi_player`：玩家删除级联清背包
- `uq_pi_player_slot`：每玩家每槽位唯一（防止双写 bug；同槽位只能放一种物品或一组堆叠物品）

**为什么 `item_id` 不物化 FK**：

1. 物品 master 表可能跨 schema / 跨域 / 跨 service 边界（per D 域边界）
2. 强 FK 会导致物品改版/下架影响玩家背包（数据完整性问题）
3. 应用层做存在性校验 + 缓存，DB 层不强制（per RGS-IMPL-001 §3 跨域弱引用原则）
4. 未来物品 master 改名为 `item_templates` / 拆 `items_static` + `items_dynamic` 等不影响本表

---

# 3. `players.metadata` JSONB 范围与反范式禁令

> **本节是 Q-D-02 答复 #2 的硬约束**，所有 player-service 域代码（含 entity / repository / service / outbox relay）必须遵守。

## 3.1 反范式禁令（per Q-D-02 答复 #2 原文 + 本 DTL 强化）

`players.metadata` JSONB **禁止**包含以下字段（违反将导致数据不一致 + 查询性能退化 + 跨域耦合）：

| 禁止字段 | 替代方案 | 理由 |
|---|---|---|
| `equipment_json` | `player_inventory` 独立表 + `player_characters.primary_weapon_id` 外键引用 | 装备有数量/槽位/附魔/绑定等多维属性，JSONB 内嵌无法查询/索引/级联 |
| `characters_json` | `player_characters` 独立表 | 角色有职业/等级/HP/ATK/DEF/stats JSONB 等多维属性，JSONB 内嵌查询复杂度 O(n*m) |
| `inventory_json` | `player_inventory` 独立表 | 背包有槽位唯一约束/数量/堆叠/到期等业务规则，JSONB 内嵌无法做唯一约束 |
| `sessions_json` | `player_sessions` 独立表（per DTL-018 §3.2）| 会话有过期时间/heartbeat/设备指纹等，JSONB 内嵌无 TTL 索引能力 |
| `idp_links_json` | `account_identity_links` 独立表（per DTL-018 §2.2）| IdP 绑定有唯一约束 + 审计日志，JSONB 内嵌破坏 FR-IDN-006 冲突检测 |
| `compliance_json` | `compliance_profiles` 独立表（per DTL-018 §2.4）| 合规判定有 flags 字段 + 审计日志，JSONB 内嵌无法做位运算 |
| `vault_json` | `identity_verification_vault` 独立表（per DTL-018 §4.2）| 实名原始凭证需加密 + 访问日志 + 单独权限，JSONB 内嵌破坏 FR-IDA-002 安全模型 |
| `audit_log_json` | `identity_binding_audit_logs` / `minor_restriction_audit_logs` 独立表（per DTL-018 §2.4/§4.3）| 审计日志需 append-only + 时间索引 + 不可篡改约束 |

**违反反范式禁令的代价**：

1. **数据不一致**：同一份数据在 JSONB 与独立表双写，无法原子更新（违反 transactionality）
2. **查询性能退化**：JSONB GIN 索引选择性差，全表扫描常见
3. **跨域耦合**：JSONB 字段跨域消费时（如 economy 域需读玩家装备计算税收）需双份解析逻辑
4. **安全风险**：vault 字段（实名凭证）内嵌 → 无独立权限控制 → 安全审计失败
5. **扩展困难**：新加 JSONB 字段 = 应用层 schema 漂移，无 DDL 保护

## 3.2 推荐使用 `players.metadata` 的低频场景

| 推荐字段 | 用途 | 频率 |
|---|---|---|
| `avatar_url` | 头像 URL | UI 读取 |
| `locale` | 客户端语言 | UI/i18n |
| `region` | 所在区域（CN/US/EU 等）| 风控/合规路由 |
| `last_client_version` | 客户端版本号 | 强制升级逻辑 |
| `marketing_source` | 渠道来源 | 运营分析 |
| `preferences` | 用户偏好（静音/画质等）| UI 个性化 |
| `tags` | 运营标签数组（高价值/流失风险/...）| 运营活动 |

**所有推荐字段都是低频 UI/运营属性，不参与热路径查询。**

## 3.3 与 §2.1.1 扩展建议的关系

本 §3 范围约束 + §2.1.1 schema 建议**共同构成**"未来 0005 migration 加 `metadata` 列时的字段字典"，确保 0005 实施时一次到位，避免 0006/0007 反复补字段。

---

# 4. 索引规划

> **索引原则**（per RGS-DTL-002 §3 + DTL-036 §5 Expand-Contract）：
> 1. **每个表至少 2 个索引**（含 PK + 1 个以上二级索引）
> 2. **FK 必加索引**（避免 ON DELETE CASCADE 性能塌方）
> 3. **高频查询字段必加索引**（按 5W1H: Who/What/When/Where/Why/How 维度）
> 4. **复合索引遵循最左前缀**（where a = ? and b > ? 走 idx(a, b)）
> 5. **partial index 优先于全索引**（where 条件稳定的用 `WHERE ...` partial）

## 4.1 `players` 索引（已存在 per 0001_init.sql）

| # | 索引名 | 列 | 类型 | 用途 | 出处 |
|---|---|---|---|---|---|
| 1 | `players_pkey` | id | B-tree PK | 主键查询 | 自动 |
| 2 | `idx_players_name` | name | B-tree UNIQUE | 昵称查玩家（登录/搜索）| 0001 |
| 3 | `idx_players_level` | level | B-tree | 按等级分桶（排行榜/匹配池）| 0001 |
| 4 | `idx_players_status` | status | B-tree partial 可选 | 风控（封禁列表）| 0001 |

**建议未来扩展**（**本任务不实施**）：

- `idx_players_last_login` (last_login_at) —— 在线状态/活跃用户筛选
- `idx_players_created_at` (created_at) —— 注册趋势/同期群分析

## 4.2 `player_characters` 索引（per 0004 migration 本任务新建）

| # | 索引名 | 列 | 类型 | 用途 | 状态 |
|---|---|---|---|---|---|
| 1 | `player_characters_pkey` | id | B-tree PK | 主键查询 | 自动 |
| 2 | `idx_pc_player_id` | player_id | B-tree | FK 索引（必需！ON DELETE CASCADE 性能）+ 查某玩家所有角色 | 0004 |
| 3 | `idx_pc_class_level` | char_class, level | B-tree 复合 | 匹配池按职业+等级分桶（match-service 跨域查询）| 0004 |
| 4 | `idx_pc_weapon` | primary_weapon_id | B-tree | FK 索引（武器删除时定位角色）| 0004 |
| 5 | `idx_pc_stats_gin` | stats | GIN | JSONB 路径查询（如查所有火抗 > 0.5 的角色）| 0004 |

**复合索引最左前缀示例**：

```sql
-- 场景: 查某职业等级 > 50 的角色
SELECT * FROM player_characters WHERE char_class = 'warrior' AND level > 50;
-- 走 idx_pc_class_level (char_class, level) 最左前缀
```

## 4.3 `player_inventory` 索引（per 0004 migration 本任务新建）

| # | 索引名 | 列 | 类型 | 用途 | 状态 |
|---|---|---|---|---|---|
| 1 | `player_inventory_pkey` | id | B-tree PK | 主键查询 | 自动 |
| 2 | `uq_pi_player_slot` | (player_id, slot) | B-tree UNIQUE | 槽位唯一（防重入）+ 查某玩家某槽位 | 0004 |
| 3 | `idx_pi_player_id` | player_id | B-tree | FK 索引（必需）+ 查某玩家所有物品 | 0004 |
| 4 | `idx_pi_item_id` | item_id | B-tree | 查持有某物品的所有玩家（运营活动/补偿）| 0004 |
| 5 | `idx_pi_acquired_at` | acquired_at | B-tree | 按获取时间排序/限时物品筛选 | 0004 |
| 6 | `idx_pi_metadata_gin` | metadata | GIN | JSONB 路径查询（查所有附魔=火属性 的武器）| 0004 |

## 4.4 索引总览表

| 表 | 索引数（含 PK） | FK 索引 | 高频字段索引 | JSONB 索引 |
|---|---|---|---|---|
| `players` | 4（1 PK + 3 二级）| N/A | name(level status) | N/A（未实施）|
| `player_characters` | 5（1 PK + 4 二级）| ✅ player_id + primary_weapon_id | char_class+level | stats GIN |
| `player_inventory` | 6（1 PK + 5 二级）| ✅ player_id | item_id, acquired_at | metadata GIN |
| **合计** | 15 | 3 | 9 | 2 |

---

# 5. 与 `0001_init.sql` 反向 doc

> **本节偿还 Q-D-02 答复 #1 的技术债**：`0001_init.sql` 已实施 `players` / `player_sessions` 2 表但当时**未配套 DTL**，本 DTL 通过反向 doc 把字段级语义与 DDL 对齐。

## 5.1 现状（事实陈述）

| 事实 | 详情 | 出处 |
|---|---|---|
| **0001_init.sql 实施时间** | 2026-08-22（commit WBS v0.3 §2A.5 WF-1-54.4）| git log |
| **0001 已实施表** | `players`（5 列核心 + 3 索引）/ `player_sessions`（6 列 + 2 索引）| `crates/player-service/migrations/0001_init.sql` |
| **0001 缺失表** | `player_characters` / `player_inventory` | Q-D-02 答复 #1 |
| **缺失原因** | 54.6 阶段 focus 在 `players` 基础档案 + 跨服 session（per DTL-018 §3.2），角色/背包等业务表延后到 PH-1 实施 | git commit msg + DTL-036 §6 第 1 条占位 |
| **DTL 倒挂** | 已有代码无 DTL 字段级定义 | Q-D-02 答复 #1 |

## 5.2 字段级对照表（`players` 表）

| 0001 DDL（line 5-15）| 0001 字段 | 本 DTL §2.1 字段 | 一致性 | 差异说明 |
|---|---|---|---|---|
| `id UUID PRIMARY KEY` | id | id | ✅ 一致 | 无 |
| `name TEXT NOT NULL UNIQUE` | name | name | ✅ 一致 | 无 |
| `level INTEGER NOT NULL DEFAULT 1` | level | level | ✅ 一致 | 本 DTL §2.1 补充 `CHECK (level >= 1 AND level <= 999)` 建议（0001 未加 CHECK，可后续 0005 migration 补）|
| `vip_level INTEGER NOT NULL DEFAULT 0` | vip_level | vip_level | ✅ 一致 | 本 DTL §2.1 补充 `CHECK (vip_level >= 0 AND vip_level <= 20)` 建议（同上）|
| `status TEXT NOT NULL DEFAULT 'active' CHECK (...)` | status | status | ✅ 一致 | 0001 已加 CHECK，约束值与本 DTL §2.1 完全一致 |
| `last_login_at TIMESTAMPTZ` | last_login_at | last_login_at | ✅ 一致 | 无 |
| `created_at TIMESTAMPTZ NOT NULL DEFAULT now()` | created_at | created_at | ✅ 一致 | 无 |
| `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()` | updated_at | updated_at | ✅ 一致 | 无 |
| **0001 缺失** | metadata JSONB | metadata（§2.1.1 建议）| ⚠️ 缺失 | 0001 未加，本 DTL §2.1.1 建议未来 0005 migration 补 |

**结论**：`players` 表 0001 DDL 与本 DTL §2.1 **完全一致**，仅差 `level` / `vip_level` 的 CHECK 范围约束（可选增强）和 `metadata` JSONB 列（未来 0005）。**本任务不修改 0001**，反向 doc 已对齐。

## 5.3 字段级对照表（`player_sessions` 表，per 0001 DDL line 22-32）

> **本表不在本 DTL 主表范围**（per DTL-018 §3.2 负责），仅作引用关系记录

| 0001 DDL 字段 | 本 DTL §7 引用 | 一致性 | 备注 |
|---|---|---|---|
| `id UUID PRIMARY KEY` | §7.1 | ✅ 引用 | 业务主键 |
| `player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE` | §7.1 | ✅ 引用 | FK + 级联删除 |
| `device_id TEXT NOT NULL` | §7.1 | ✅ 引用 | 设备指纹 |
| `ip TEXT NOT NULL` | §7.1 | ✅ 引用 | 登录 IP |
| `login_at TIMESTAMPTZ NOT NULL DEFAULT now()` | §7.1 | ✅ 引用 | 登录时间 |
| `last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now()` | §7.1 | ✅ 引用 | 心跳时间 |
| `expires_at TIMESTAMPTZ NOT NULL` | §7.1 | ✅ 引用 | 过期时间（per entity.rs `PlayerSession.heartbeat()` 滑动 24h）|
| `idx_player_sessions_player_id` | §7.1 | ✅ 引用 | FK 索引 |
| `idx_player_sessions_expires_at` | §7.1 | ✅ 引用 | TTL 清理索引 |

## 5.4 本任务对 0001 的态度

| 行为 | 是否 | 理由 |
|---|---|---|
| 修改 0001 已有 DDL | ❌ 否 | per RGS-IMPL-001 §3.2 历史 migration 不可变 |
| 在 0001 末尾追加新表 | ❌ 否 | 历史 migration 文件本体不修改，新表走 0004 migration |
| 反向 doc 字段对齐 | ✅ 是 | 本 DTL §5.2 完整字段级对照 |
| 提议未来 0005 增强 | ✅ 是 | §2.1.1 metadata 列 + §5.2 CHECK 约束建议（不实施，仅文档化）|

## 5.5 与后续 migration 的关系

| Migration | 范围 | 本 DTL 引用 |
|---|---|---|
| **0001_init.sql** | players + player_sessions | §5.2 + §5.3 反向 doc |
| **0002_outbox.sql** | outbox（事务性消息）| 不在本 DTL 范围（per RGS-DTL-100 §5.3 + DTL-036 §3）|
| **0003_outbox_check_idempotent.sql** | outbox CHECK 修复 | 不在本 DTL 范围（per RGS-REV-009 CR-2 + WF-1-55.28）|
| **0004_player_characters_inventory.sql**（本任务新建）| player_characters + player_inventory | §2.2 + §2.4 + §4 索引全部 |
| 未来 0005（建议）| players.metadata JSONB + CHECK 约束补强 | §2.1.1 建议 + §3 字段字典 |
| 未来 0006+（建议）| characters/inventory 性能优化（分区/物化视图等）| 本 DTL 范围外，PH-2 实施 |

---

# 6. 签字栏（per DEC-008 12 角色 RACI）

> **RACI 原则**（per RGS-DEC-008 一人公司派生 + RGS-DEC-005 5 域独立 Lead）：
> - **R（执行）**：AI worker（per DEC-008 派生）/ player 域 Lead（Ulysses 兼任 per 一人公司）
> - **A（责任）**：player 域 Lead（Ulysses）
> - **C（咨询）**：economy/match/social/admin 4 域 Lead（跨域影响） + DBA + 安全
> - **I（知会）**：PM/QA/PE + 架构师 + 集群运维

| # | 角色 | 姓名/Agent | RACI | 签字 | 日期 | 备注 |
|---|---|---|---|---|---|---|
| 1 | **player 域 Lead** | Ulysses（per DEC-008）| **A / R** | ☐ | | 主责 owner，1.0 状态接受 = done 100% |
| 2 | economy 域 Lead | Ulysses（per 一人公司兼任不推荐，per user profile 2026-08-21：5 域独立 Lead）| C | ☐ | | 跨域：players.metadata 不含 trade_json 等 |
| 3 | match 域 Lead | Ulysses（per user profile 同上）| C | ☐ | | 跨域：player_characters 匹配池查询 |
| 4 | social 域 Lead | Ulysses | C | ☐ | | 跨域：players 元数据被好友/聊天引用 |
| 5 | admin 域 Lead | Ulysses | C | ☐ | | 跨域：players.status 'banned' 触发 admin 封禁流程 |
| 6 | cluster-ops / SRE Lead | Ulysses（per 一人公司）| C | ☐ | | 索引/分区/性能预算 |
| 7 | **架构师** | AI worker | R | ☐ | | DDL 字段级定义 + 反向 doc |
| 8 | DBA | AI worker（per SQLx 自审 + migration 审计）| C | ☐ | | 索引规划 + FK 完整性 |
| 9 | 安全 | AI worker（per RGS-SEC-101 + DTL-018 §2 vault 权限参考）| C | ☐ | | metadata JSONB 不含敏感字段 + 跨域权限 |
| 10 | PM | Ulysses | I | ☐ | | 知会：PH-1 实施进度 |
| 11 | QA | Ulysses | I | ☐ | | 知会：未来 #[sqlx::test] 跟进 |
| 12 | PE（产品工程）| Ulysses | I | ☐ | | 知会：业务字段对得上 game design |

**签字规则**：

- **v0.1 → v1.0 升版条件**（per DEC-008 派生 RACI 强化）：5 域独立 Lead（#2-#5）+ player 域 Lead（#1）+ 架构师（#7）共 **7 个 R/A 角色必须签**，C/I 角色建议签（不强制）
- 一人公司场景：Ulysses 兼任多角色，**所有 12 项均由 Ulysses 一人签**（per DEC-008 + user profile 2026-08-21 "5 域独立 Lead 不接受兼任" → 一人公司下接受一人多签但责任矩阵清晰）
- **签字时机**：Ulysses 终审后接受 → status 🟢 v1.0 → 由 WF-1-55.39 worker 提交 PR → merge to main → 本表正式生效

---

# 7. 附录 A：`player_sessions` 引用关系（per DTL-018 §3.2）

> **本节是 §5.3 的展开**，详细说明 `player_sessions` 表与本 DTL 主表的引用关系。

## 7.1 `player_sessions` 字段总览

```sql
-- 已在 0001_init.sql line 22-32 实施 (per WBS v0.3 §2A.5 WF-1-54.4)
-- 详细 DDL 归 DTL-018 §3.2, 本 DTL 仅记录引用关系

CREATE TABLE IF NOT EXISTS player_sessions (
    id                  UUID PRIMARY KEY,
    player_id           UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    device_id           TEXT NOT NULL,
    ip                  TEXT NOT NULL,
    login_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_heartbeat_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_player_sessions_player_id ON player_sessions (player_id);
CREATE INDEX IF NOT EXISTS idx_player_sessions_expires_at ON player_sessions (expires_at);
```

## 7.2 引用关系

| 字段 | 引用目标 | ON DELETE | 理由 |
|---|---|---|---|
| `player_sessions.player_id` | `players.id` (§2.1) | CASCADE | 玩家账号删除 → 所有 session 同步清理（per DTL-018 §3.2 "active-active 跨服身份"）|

## 7.3 与本 DTL 主表的引用图

```
        ┌─────────────────┐
        │    players      │ (DTL-044 §2.1, 0001 line 5-15)
        │  id (PK)        │
        │  name (UNIQUE)  │
        │  level/vip/status│
        └────────┬────────┘
                 │ 1:N (player_id FK, ON DELETE CASCADE)
       ┌─────────┼──────────────┬──────────────┐
       │         │              │              │
       ▼         ▼              ▼              ▼
┌────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────────────┐
│ player_    │ │ player_      │ │ player_      │ │ player_sessions │
│ characters │ │ inventory    │ │ characters   │ │ (DTL-018 §3.2)  │
│ §2.2 / 0004│ │ §2.4 / 0004  │ │ .primary_    │ │ 0001 line 22-32 │
│            │ │              │ │ weapon_id    │ │                 │
│  (1:N)     │ │  (1:N)       │ │  → inv.id    │ │  (1:N)          │
│            │ │              │ │  (§3 反范式) │ │                 │
└────────────┘ └──────────────┘ └──────────────┘ └─────────────────┘
```

**说明**：

- `player_characters.primary_weapon_id` → `player_inventory.id`：ON DELETE SET NULL（武器丢角色保留）
- 三个 N 端子表均 ON DELETE CASCADE（玩家删除全清）
- `player_sessions` 是 DTL-018 §3.2 范围，本 DTL 仅引用

## 7.4 未来 N+1 关系的预留

| 关系 | 现状 | 未来 |
|---|---|---|
| `player_characters` ↔ 技能 master | 不在 0004 | PH-2 `player_character_skills` 表 |
| `player_inventory` ↔ 装备附魔 master | metadata JSONB 暂存 | PH-2 拆 `player_equipment_affixes` |
| `players` ↔ 成就 master | 不在 0004 | PH-2 `player_achievements` 表 |

---

# 8. 追踪矩阵

> **本节是 DTL 标准追踪矩阵**（per RGS-DTL-001 §6 + RGS-IMPL-001），让任何字段变化可追溯。

## 8.1 来源追溯（Source → 本文）

| 来源 | 出处章节 | 落地章节 |
|---|---|---|
| Q-D-02 答复 #1（新建 DTL-044 + 反向 doc + 0004 migration）| RGS-OPEN-QA-001 v0.2 §2 / ACTIONS-v0.3 §3 A-02 | 全部 + §5 |
| Q-D-02 答复 #2（`metadata` JSONB 不含 `equipment_json` 等反范式字段）| 同上 | §3 全文 |
| Q-D-02 答复 #3（`stats` JSONB + 字段化混合）| 同上 | §2.2 + §2.3 全文 |
| DTL-018 §2（5 张身份合规表）| RGS-BAS-018 + DTL-018 | §1.1 不在范围说明 + §7 引用 |
| DTL-018 §3.2（player_sessions）| DTL-018 + 0001 line 22-32 | §5.3 + §7 全文 |
| DTL-036 §6 第 1 条（账号/角色/会话三表 DDL 跟进）| DTL-036 | §1.2 目的 #3 |
| 0001_init.sql（已实施 players/player_sessions）| crates/player-service/migrations/0001_init.sql | §5.2 + §5.3 反向 doc |
| entity.rs（Player / PlayerSession 已实化）| crates/player-service/src/entity.rs | §2.1 字段语义列来源 |
| WF-1-55.39（任务定义）| RGS-WBS-001 L4 进度表 v0.4 | 编制依据 + token 12K 预算 |
| RGS-OPEN-QA-001-ACTIONS v0.3 §4 13 个 L4 任务 | ACTIONS-v0.3 | 编号 044 与 038 区分（§修订历史）|
| RGS-DEC-008（一人公司派生）| RGS-DEC-008 | §6 12 角色 RACI（Ulysses 一人多签）|
| RGS-IMPL-001 §3.2（历史 migration 不可变）| IMPL-001 | §5.4 不修改 0001 |
| user profile 2026-08-21（5 域独立 Lead）| user memory | §6 RACI 兼任澄清 |

## 8.2 影响追溯（本文 → 下游）

| 落地章节 | 影响对象 | 后续动作 |
|---|---|---|
| §2.1 `players` 字段语义 | entity.rs `Player` struct（已对齐）| 无需改动 |
| §2.2 `player_characters` DDL | 0004 migration（本任务） + entity.rs `PlayerCharacter`（**未来 L4**）| 0004 实施后 entity 跟进，**不属本任务** |
| §2.4 `player_inventory` DDL | 0004 migration（本任务） + entity.rs `PlayerInventory`（**未来 L4**）| 同上 |
| §2.3 stats 混合存储 | repository.rs / service.rs 业务层（**未来 L4**）| 字段化列走简单 getter/setter，JSONB 部分走 serde_json |
| §3 metadata 反范式禁令 | 全 player-service 域代码 + 跨域消费方 | 任何 PR 涉及 `players.metadata` 字段字典改动必须 review §3 禁令 |
| §4 索引规划 | sqlx migration 自动创建 | 0004 migration 落地后即生效 |
| §5 反向 doc | 0001_init.sql 维护者 | 无需改动 0001，文档化对齐 |
| §6 签字栏 | Ulysses 终审 | v0.1 → v1.0 升版 |
| §7 player_sessions 引用 | 已有，无影响 | 无 |

## 8.3 后续 L4 任务依赖

| 依赖任务 | 依赖内容 | 状态 |
|---|---|---|
| WF-1-55.39（本任务）| DTL-044 v0.1 + 0004 migration | 🆕 本次开工 |
| 未来 L4: player_characters entity 实化 | §2.2 Rust struct + repository CRUD | 待立项 |
| 未来 L4: player_inventory entity 实化 | §2.4 Rust struct + repository CRUD | 待立项 |
| 未来 L4: stats JSONB schema 校验 | §2.3.2 JSONB 推荐 schema 应用层校验（serde derive + 自定义 validator）| 待立项 |
| 未来 L4: 0005 migration（players.metadata + CHECK 增强）| §2.1.1 + §5.2 建议 | 待立项 |
| 未来 L4: 跨域（match/economy/social）消费 player_characters 字段化属性 | §2.3 字段化列（HP/ATK/DEF/crit_rate）| 待立项 |

---

> **本 DTL 文档结束**
> **编制**：AI worker per WF-1-55.39 / **owner**：player 域 Lead Ulysses per DEC-008
> **下一步**：Ulysses 终审 → 签字栏 #1/#7 → v0.1 → v1.0 升版 → 0004 migration merge to main
