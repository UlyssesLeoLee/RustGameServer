-- player-service migration 0005_decks.sql (per DTL-038 §7.1 #4 + FR-002 + DEC-038-01)
--
-- 目的: 卡牌游戏 桶 11 增量 (per RGS-REQ-038 §FR-002 卡组 CRUD + share)
--       卡组归 player-service v2 (per DEC-038-01, NOT 新增 deck-service)
--       复用 player_db (per ARC-008 5 独立 DB 原则)
--
-- 设计原则 (per DTL-038 §7.1):
--   1. JSONB 存 slots (卡组是低频写, 高频读整个 deck, JSONB 减少 JOIN)
--      - 业务层校验 30-60 张 + 同卡 ≤ 2 张 (per DTL-038 规则引擎占位)
--      - 应用层在 service.update_deck 校验; SQL 仅做 schema 兜底
--   2. owner_id 不物化 FK (per DTL-038 §7.1 跨域弱引用原则: 应用层保证存在性)
--      - 玩家删除 = CASCADE 清 decks (per DTL-018 §3.1 玩家级联清理)
--      - ON DELETE CASCADE 在 application 层实现 (非物化 FK)
--   3. share_code UNIQUE (per DTL-038 §7.1 share_code 公开分享码唯一)
--   4. is_public BOOL 索引 (查询公开 deck, list_decks_public 走索引)
--   5. (owner_id, updated_at DESC) 复合索引 (per FR-002 ListDecks 排序)
--
-- 兼容性:
--   - CREATE TABLE IF NOT EXISTS 兼容 fresh DB + 已部署 0001-0004 四种环境
--   - 不修改 0001-0004 任何已有表 (per RGS-IMPL-001 §3.2 历史 migration 不可变)
--   - 该 migration 仅在 0001-0004 之后运行, 不依赖其他表
--
-- 关联:
--   - DTL-038 v0.1 §4.3 (proto v2 Deck/DeckSlot message)
--   - DTL-038 v0.1 §7.1 (decks 表 DDL 草图, 本文件落地)
--   - DTL-018 §3.1 (玩家删除级联清理规则)
--   - DTL-036 §3 (Player 域契约登记, proto 在 v2 增加)

-- ============================================================================
-- 表: decks 卡组 (per DTL-038 §4.3 + §7.1 + FR-002)
-- ============================================================================

CREATE TABLE IF NOT EXISTS decks (
    -- 主键: deck_id (UUID, per DTL-038 §7.1)
    deck_id     UUID PRIMARY KEY,

    -- 所属玩家 ID (UUID, per DTL-018 §3.1 玩家强属性)
    -- 不物化 FK: 玩家删除走 application 层 CASCADE (per DTL-018 §3.1)
    owner_id    UUID NOT NULL,

    -- 卡组名
    name        TEXT NOT NULL,

    -- 模式: 1=ranked 2=casual 3=room 4=ai (per DTL-038 §7.1, TODO 迁 common.v1.GameMode 枚举)
    mode        SMALLINT NOT NULL
                    CHECK (mode IN (1, 2, 3, 4)),

    -- 卡槽列表: JSONB [{card_id: String, count: u32}]
    -- 业务层校验 30-60 张 + 同卡 ≤ 2 张 (per DTL-038 规则引擎占位)
    slots       JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- 状态: 1=draft 2=active 3=archived (per DTL-038 §7.1)
    status      SMALLINT NOT NULL DEFAULT 1
                    CHECK (status IN (1, 2, 3)),

    -- 是否公开分享
    is_public   BOOLEAN NOT NULL DEFAULT FALSE,

    -- 公开分享码 (UUIDv4 string, 仅 is_public=true 时非 NULL, UNIQUE)
    share_code  TEXT UNIQUE,

    -- 点赞数 (per DTL-038 §4.3 Deck.like_count)
    like_count  INTEGER NOT NULL DEFAULT 0
                    CHECK (like_count >= 0),

    -- 元信息
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- decks 索引 (per DTL-038 §7.1 + FR-002 ListDecks/ShareDeck/GetSharedDeck 访问模式)
-- 1. PK 索引 (自动, deck_id)
-- 2. UNIQUE 索引: share_code (ShareDeck 唯一性 + GetSharedDeck 拉取路径)
--    share_code UNIQUE 约束自动创建, 不需要显式 CREATE INDEX
-- 3. 二级索引: owner_id (ListDecks 按玩家过滤)
CREATE INDEX IF NOT EXISTS idx_decks_owner_id ON decks (owner_id);

-- 4. 复合索引: (owner_id, updated_at DESC) (ListDecks 默认排序)
CREATE INDEX IF NOT EXISTS idx_decks_owner_updated ON decks (owner_id, updated_at DESC);

-- 5. 二级索引: is_public (ListDecksPublic 公开 deck 列表)
CREATE INDEX IF NOT EXISTS idx_decks_is_public ON decks (is_public);

-- 6. JSONB GIN 索引: slots (按 card_id 查询某卡出现在哪些 deck)
CREATE INDEX IF NOT EXISTS idx_decks_slots_gin ON decks USING GIN (slots);

-- ============================================================================
-- Migration 结束
-- ============================================================================
--
-- 落地后状态:
--   player_db schema 应包含 6 张表 (per 0001-0005 累计):
--     1. players              (per 0001,  DTL-018 §3.1)
--     2. player_sessions      (per 0001,  DTL-018 §3.2)
--     3. outbox               (per 0002,  DTL-100 §5.3)
--     4. player_characters    (per 0004,  DTL-044 §2.2)
--     5. player_inventory     (per 0004,  DTL-044 §2.4)
--     6. decks                (per 0005 本文件, DTL-038 §7.1)
--   + outbox 上的 CHECK 约束 chk_outbox_status (per 0003 幂等补强)
--
-- 验证 SQL (开发/测试环境手工跑一遍):
--   SELECT table_name FROM information_schema.tables
--   WHERE table_schema = 'public'
--   ORDER BY table_name;
--   -- 预期: 6 行 (含 outbox)
--
--   SELECT indexname FROM pg_indexes
--   WHERE schemaname = 'public'
--   ORDER BY indexname;
--   -- 预期: 至少 16 行 (5 张业务表 + outbox 索引, 含本文件 5 个新索引)
--
-- 后续 L4 任务 (本 migration 不涉及, 仅记录):
--   - entity.rs: 已加 Deck/DeckSlot/PlayerProfile struct (桶 11 完成)
--   - repository.rs: 加 7 CRUD 方法 (create_deck/get_deck/update_deck/delete_deck/list_decks/share_deck/get_shared_deck)
--   - service.rs: 加 7 RPC handler (CreateDeck/GetDeck/UpdateDeck/DeleteDeck/ListDecks/ShareDeck/GetSharedDeck)
--   - 未来: 玩家删除级联清理 decks (per DTL-018 §3.1, 走 application 层)
--   - 未来: 规则引擎 (per DTL-038 §9.1 P2 风险, 留给业务层 game-logic)
