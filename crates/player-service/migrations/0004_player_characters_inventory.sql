-- player-service migration 0004_player_characters_inventory（per WF-1-55.39 + DTL-044 v0.1 §2.2 §2.3 §2.4 + RGS-OPEN-QA-001 v0.2 Q-D-02 答复 #1）
--
-- 目的: 偿还 0001_init.sql 先实施但 DTL 未跟进的技术债 (per Q-D-02 答复 #1)
--       0001_init.sql 已建 players/player_sessions 两表, 缺 player_characters/player_inventory 两表
--       DTL-036 §6 第 1 条 checklist 写明"账号/角色/会话三表 DDL 跟进", 本 migration 落地 player_characters/inventory 两表
--
-- 设计原则 (per DTL-044 §2.3):
--   1. JSONB + 字段化混合 (per Q-D-02 答复 #3)
--      - HP/ATK/DEF/crit_rate 等高频/需索引属性拆列 (热路径 O(log n) B-tree)
--      - stats JSONB 留低频/扩展属性 (抗性/buffs/enchantments_applied/talents 等)
--   2. 反范式禁令 (per Q-D-02 答复 #2)
--      - 装备不走 equipment_json, 走 player_inventory 独立表 + 外键引用
--      - primary_weapon_id UUID FK → player_inventory.id (ON DELETE SET NULL)
--   3. FK 必加索引 (per DTL-044 §4 索引原则 #2)
--      - player_id FK 索引避免 ON DELETE CASCADE 性能塌方
--   4. 槽位唯一 (per DTL-044 §2.4 复合约束)
--      - UNIQUE (player_id, slot) 防重入 bug
--   5. item_id 不物化 FK (per DTL-044 §2.4 跨域弱引用原则)
--      - 物品 master 表跨域, DB 层不强制
--
-- 兼容性:
--   - CREATE TABLE IF NOT EXISTS 兼容 fresh DB + 已部署 0001/0002/0003 三种环境
--   - 不修改 0001/0002/0003 任何已有表 (per RGS-IMPL-001 §3.2 历史 migration 不可变)
--   - 该 migration 仅在 0001/0002/0003 之后运行, 不依赖它们的具体 schema, 只依赖 players.id 存在
--
-- ⚠️ 已知反 pattern 防御 (per RGS-REV-009 CR-2 / WF-1-55.28 经验):
--   - 0002 outbox 的 CHECK 约束写在 CREATE TABLE IF NOT EXISTS 块内导致已部署环境 CHECK 失效
--   - 本 migration 所有 CHECK 约束直接在 CREATE TABLE 块内 (fresh DB 有效)
--   - 已部署 0001/0002/0003 环境: 本文件 CREATE TABLE IF NOT EXISTS 块被 sqlx 静默跳过 → 新表/约束永不创建
--   - **缓解**: 0001/0002/0003 部署后 player_characters/player_inventory 表应**不存在**, CREATE TABLE 块一定会执行
--   - 若生产环境已存在 player_characters/player_inventory (极不可能), 需要单独 0005 migration 加 DO 块
--     (per 0003_outbox_check_idempotent.sql 的 DO $$ EXCEPTION WHEN duplicate_object 模式)
--
-- 关联:
--   - DTL-044 v0.1 (本 migration 字段级定义来源)
--   - DTL-018 §3.1 (玩家删除级联清理规则)
--   - DTL-018 §3.2 (player_sessions 引用, 本文件不涉及)
--   - DTL-036 §3 (Player 域契约登记, 本文件不涉及 proto)
--   - DTL-100 §5.3 (Outbox, 本文件不涉及)

-- ============================================================================
-- 表 1: player_characters 玩家角色档案 (per DTL-044 §2.2)
-- ============================================================================

CREATE TABLE IF NOT EXISTS player_characters (
    -- 主键
    id                  UUID PRIMARY KEY,

    -- 外键: 所属玩家 (ON DELETE CASCADE 玩家删除级联清角色, per DTL-018 §3.1)
    player_id           UUID NOT NULL
                            REFERENCES players(id) ON DELETE CASCADE,

    -- 职业 (5 选 1 枚举, per DTL-044 §2.2 字段语义)
    char_class          TEXT NOT NULL
                            CHECK (char_class IN ('warrior', 'mage', 'archer', 'assassin', 'support')),

    -- 角色等级 (独立于 players.level, per DTL-044 §2.2 字段语义)
    level               INTEGER NOT NULL DEFAULT 1
                            CHECK (level >= 1 AND level <= 999),

    -- 高频战斗属性拆列 (per DTL-044 §2.3 JSONB + 字段化混合, 热路径 B-tree 索引)
    hp                  INTEGER NOT NULL DEFAULT 100
                            CHECK (hp >= 0),
    atk                 INTEGER NOT NULL DEFAULT 10
                            CHECK (atk >= 0),
    def                 INTEGER NOT NULL DEFAULT 5
                            CHECK (def >= 0),
    crit_rate           REAL NOT NULL DEFAULT 0.05
                            CHECK (crit_rate >= 0.0 AND crit_rate <= 1.0),

    -- 扩展属性 JSONB (per DTL-044 §2.3 低频/扩展位)
    --   推荐 schema (per DTL-044 §2.3.2):
    --     {
    --       "resistances": {"fire": 0.0, "ice": 0.0, ...},
    --       "buffs": [...], "debuffs": [...],
    --       "enchantments_applied": {...},
    --       "talents": {...},
    --       "achievements": [...],
    --       "custom": {}
    --     }
    stats               JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- 主武器引用: 不内嵌 JSON, 走 player_inventory 独立表 + 外键 (per DTL-044 §3 反范式禁令)
    -- ON DELETE SET NULL: 玩家丢武器时角色保留, 武器槽位置空
    primary_weapon_id   UUID,

    -- 元信息
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 复合约束 (表级, per DTL-044 §2.2)
    CONSTRAINT fk_pc_player FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    CONSTRAINT fk_pc_weapon FOREIGN KEY (primary_weapon_id) REFERENCES player_inventory(id) ON DELETE SET NULL
);

-- player_characters 索引 (per DTL-044 §4.2)
-- 1. PK 索引 (自动, id)
-- 2. FK 索引: player_id (必需! ON DELETE CASCADE 性能 + 查某玩家所有角色)
CREATE INDEX IF NOT EXISTS idx_pc_player_id ON player_characters (player_id);

-- 3. 复合索引: char_class + level (匹配池按职业+等级分桶, match-service 跨域查询)
CREATE INDEX IF NOT EXISTS idx_pc_class_level ON player_characters (char_class, level);

-- 4. FK 索引: primary_weapon_id (武器删除时定位角色)
CREATE INDEX IF NOT EXISTS idx_pc_weapon ON player_characters (primary_weapon_id);

-- 5. JSONB GIN 索引: stats (查所有火抗 > 0.5 的角色等路径查询)
CREATE INDEX IF NOT EXISTS idx_pc_stats_gin ON player_characters USING GIN (stats);

-- ============================================================================
-- 表 2: player_inventory 玩家背包物品 (per DTL-044 §2.4)
-- ============================================================================

CREATE TABLE IF NOT EXISTS player_inventory (
    -- 主键
    id                  UUID PRIMARY KEY,

    -- 外键: 所属玩家 (ON DELETE CASCADE, per DTL-018 §3.1)
    player_id           UUID NOT NULL
                            REFERENCES players(id) ON DELETE CASCADE,

    -- 物品 master ID (UUID, 跨域引用, 不在本 DDL 物化 FK, per DTL-044 §2.4)
    --   物品 master 表可能跨 schema/跨域/跨 service 边界
    --   强 FK 会导致物品改版/下架影响玩家背包
    --   应用层做存在性校验 + 缓存, DB 层不强制
    item_id             UUID NOT NULL,

    -- 数量: 同槽位堆叠数, 1 = 非堆叠
    quantity            INTEGER NOT NULL DEFAULT 1
                            CHECK (quantity > 0),

    -- 槽位: 0..199 (背包上限 200 格, per DTL-044 §2.4 字段语义)
    slot                INTEGER NOT NULL
                            CHECK (slot >= 0 AND slot < 200),

    -- 获取时间
    acquired_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 物品实例属性 JSONB (附魔词条/绑定角色/到期时间等, per DTL-044 §2.4 字段语义)
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- 元信息
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 复合约束 (per DTL-044 §2.4)
    CONSTRAINT fk_pi_player FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,

    -- 一玩家一槽位一物 (slot 唯一 per player, 防重入 bug)
    CONSTRAINT uq_pi_player_slot UNIQUE (player_id, slot)
);

-- player_inventory 索引 (per DTL-044 §4.3)
-- 1. PK 索引 (自动, id)
-- 2. UNIQUE 索引: (player_id, slot) (防重入 + 查某玩家某槽位)
--    uq_pi_player_slot 约束自动创建, 不需要显式 CREATE INDEX
-- 3. FK 索引: player_id (必需! ON DELETE CASCADE 性能 + 查某玩家所有物品)
CREATE INDEX IF NOT EXISTS idx_pi_player_id ON player_inventory (player_id);

-- 4. 二级索引: item_id (查持有某物品的所有玩家, 运营活动/补偿)
CREATE INDEX IF NOT EXISTS idx_pi_item_id ON player_inventory (item_id);

-- 5. 二级索引: acquired_at (按获取时间排序/限时物品筛选)
CREATE INDEX IF NOT EXISTS idx_pi_acquired_at ON player_inventory (acquired_at);

-- 6. JSONB GIN 索引: metadata (查所有附魔=火属性的武器等路径查询)
CREATE INDEX IF NOT EXISTS idx_pi_metadata_gin ON player_inventory USING GIN (metadata);

-- ============================================================================
-- Migration 结束
-- ============================================================================
--
-- 落地后状态:
--   player_db schema 应包含 5 张表 (per 0001/0002/0003/0004 累计):
--     1. players              (per 0001 line 5-15,   DTL-044 §2.1 反向 doc)
--     2. player_sessions      (per 0001 line 22-32,  DTL-018 §3.2 引用)
--     3. outbox               (per 0002,            DTL-100 §5.3)
--     4. player_characters    (per 0004 本文件,     DTL-044 §2.2)
--     5. player_inventory     (per 0004 本文件,     DTL-044 §2.4)
--   + outbox 上的 CHECK 约束 chk_outbox_status (per 0003 幂等补强)
--
-- 验证 SQL (开发/测试环境手工跑一遍):
--   SELECT table_name FROM information_schema.tables
--   WHERE table_schema = 'public'
--   ORDER BY table_name;
--   -- 预期: 5 行 (含 outbox)
--
--   SELECT indexname FROM pg_indexes
--   WHERE schemaname = 'public'
--   ORDER BY indexname;
--   -- 预期: 至少 13 行 (4 张业务表 + outbox 索引)
--
-- 后续 L4 任务 (本 migration 不涉及, 仅记录):
--   - entity.rs 加 PlayerCharacter / PlayerInventory struct (DTL-036 §6 第 1 条)
--   - repository.rs 加 CRUD (per DTL-002 §4)
--   - service.rs 加业务逻辑 (per DTL-036 §3 gRPC)
--   - 0005 migration (未来): players.metadata JSONB + CHECK 约束补强 (per DTL-044 §2.1.1 + §5.2)
--   - 0006+ migration (未来): 性能优化 (分区/物化视图等, PH-2 实施)
