-- player-service migration 0006_characters.sql
--
-- 目的: 补上 "桶 12 占位" 缺口 —— PgCharacterRepository 的 6 个方法
--       (create / find_by_id / find_by_account_id / find_by_name / update /
--        delete_by_id) 此前全部 `Err(Internal("not yet implemented"))`,
--       导致 13 个 gRPC RPC 在端到端探测时全部 INTERNAL:
--         CreateCharacter / LoginCharacter / ReconnectCharacter /
--         GetCharacterProfile / GetCharacterAssets / GetCharacterInfo /
--         RenameCharacter / GuestModeTimeout / AntiAddictionCheck /
--         ForceDisconnect / EnterBackground / GetAvatarList / SetAvatar / Heartbeat
--
-- 为什么新建 characters 表而不复用 0004 的 player_characters:
--   两张表服务的不是一回事, 字段集几乎没有交集。
--     player_characters (0004, 战斗属性): id / player_id / char_class / level /
--                                  hp / atk / def / crit_rate / stats /
--                                  primary_weapon_id
--     characters      (本表, 账号角色): id / account_id / name / class_id /
--                                  faction_id / level / vip_level /
--                                  current_avatar_id / signature / status /
--                                  in_background / last_login_at
--   entity::Character 要的是 name / account_id / faction_id / vip_level /
--   signature / status / in_background —— player_characters 一个都没有;
--   反过来 hp/atk/def/crit_rate 也不是 Character 的字段。
--   强行复用会让一张表同时承担两种生命周期不同的聚合, 违反单一职责。
--
-- 设计要点 (per DTL-038 §7.1 / ARC-008 5 域分库):
--   1. name UNIQUE —— CharacterRepository::find_by_name 的契约前提,
--      CreateCharacter 的重名检查 (Error::NicknameTaken) 依赖它。
--      用普通 UNIQUE 而非 partial index: 表本身无软删除列, 全表唯一即可。
--   2. account_id 不加 FK —— 跨域外键 (per 0005_decks.sql 同样理由),
--      应用层保证存在性, 避免跨 crate migration 顺序耦合。
--   3. (account_id) 索引 —— find_by_account_id 是 1 账号 1 角色的热路径。
--   4. status CHECK 限定 PlayerStatus 的 4 个 serde 值 (snake_case):
--      active / banned / disabled / pending。
--   5. class_id / faction_id / vip_level / current_avatar_id 保留 CHECK 之外
--      的宽松整数域 —— 桶 12 占位说明里明确写了职业/阵营枚举本身待定,
--      这里不提前锁死业务枚举, 避免下一个桶还要改 CHECK。
--      level 保留 CHECK >= 1 (等级 0 无意义, 且 Character::new 默认 1)。
--   6. 幂等: CREATE TABLE IF NOT EXISTS, fresh DB 与已跑 0001-0005 的库都能用。
--   7. created_at / updated_at 由 DEFAULT now() 兜底, 应用层仍可显式覆盖。

CREATE TABLE IF NOT EXISTS characters (
    id                 uuid        PRIMARY KEY,
    account_id         uuid        NOT NULL,
    name               text        NOT NULL,
    class_id           integer     NOT NULL,
    faction_id         integer     NOT NULL,
    level              integer     NOT NULL DEFAULT 1,
    vip_level          integer     NOT NULL DEFAULT 0,
    current_avatar_id  integer     NOT NULL DEFAULT 1,
    signature          text        NOT NULL DEFAULT '',
    status             text        NOT NULL DEFAULT 'active',
    in_background      boolean     NOT NULL DEFAULT false,
    last_login_at      timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT characters_name_key      UNIQUE (name),
    CONSTRAINT characters_level_check    CHECK (level >= 1),
    CONSTRAINT characters_vip_check      CHECK (vip_level >= 0),
    CONSTRAINT characters_avatar_check   CHECK (current_avatar_id >= 1),
    CONSTRAINT characters_status_check   CHECK (status IN ('active', 'banned', 'disabled', 'pending'))
);

-- find_by_account_id 热路径 (v0.1 契约: 1 账号 1 角色)
CREATE INDEX IF NOT EXISTS characters_account_id_idx ON characters (account_id);
