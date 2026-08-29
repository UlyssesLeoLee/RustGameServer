-- match-service migration 0040_game_sessions (per RGS-DTL-038 v0.1 §7.1 卡牌游戏 schema)
-- 卡牌游戏 8 张新表 之 2 张 (game_sessions + moves), 仅 match-service 域
-- 注: 完整 8 张表 (cards / card_series / card_instances / decks / game_sessions / moves / replays / auctions)
-- 跨 5 域 (card / player / match / replay / trade), 各自域负责自己的 migration.
-- 本 migration 仅承担 match-service 域内 2 张 (game_sessions + moves).
-- 上游: RGS-REQ-038 §FR-004 session / RGS-DTL-038 §5 状态机 / §7.1 数据库 schema

-- 1. game_sessions (对战 session, per §5.1 状态机 + §7.1)
CREATE TABLE IF NOT EXISTS game_sessions (
    match_id              UUID PRIMARY KEY,
    mode                  SMALLINT NOT NULL,            -- GameMode: 0=unspec 1=ranked 2=casual 3=room 4=pve_ai
    status                SMALLINT NOT NULL DEFAULT 1,  -- 状态机: 0=unspec 1=creating 2=waiting 3=starting 4=running 5=turn_n 6=paused 7=ending 8=ended 9=canceled
    players               JSONB NOT NULL DEFAULT '[]'::jsonb,   -- [{player_id, display_name, rank_score, level, deck_ref, team}, ...]
    host_id               TEXT,                                  -- 房主 player_id (CreateMatchRequest.host)
    room_code             TEXT,                                  -- 房间码 (ROOM 模式)
    room_password_hash    TEXT,                                  -- 房间密码 hash (ROOM 模式, 敏感字段)
    max_players           INT NOT NULL DEFAULT 2,
    min_players           INT NOT NULL DEFAULT 2,
    turn_index            INT NOT NULL DEFAULT 0,
    current_player_id     TEXT,                                  -- 当前回合 player_id
    next_turn_deadline_ms BIGINT,                                -- 当前 turn 截止 (epoch ms)
    board_snapshot        JSONB NOT NULL DEFAULT '{}'::jsonb,    -- 战牌状态 (本表存放, 减少对象存储往返)
    board_snapshot_ref    TEXT,                                  -- 对象存储引用 (per §4.2 Match.board_snapshot_ref)
    winner_id             TEXT,                                  -- 胜者 player_id (status=ended 时)
    end_reason            TEXT,                                  -- 投降 / 超时 / 胜负判定 / 取消 / 强制踢出
    ai_difficulty         SMALLINT NOT NULL DEFAULT 0,           -- 0=无 1=随机 2=简单 3=中等 4=困难
    timeout_count         INT NOT NULL DEFAULT 0,                -- 当前玩家累计超时次数
    pending_moves         JSONB NOT NULL DEFAULT '[]'::jsonb,    -- 待执行 move 队列 (per §4.2 GetMatchStateResponse.pending_moves)
    started_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at              TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_game_sessions_status ON game_sessions (status);
CREATE INDEX IF NOT EXISTS idx_game_sessions_host_id ON game_sessions (host_id);
CREATE INDEX IF NOT EXISTS idx_game_sessions_room_code ON game_sessions (room_code);
CREATE INDEX IF NOT EXISTS idx_game_sessions_players_gin ON game_sessions USING GIN (players);
CREATE INDEX IF NOT EXISTS idx_game_sessions_created_at ON game_sessions (created_at);

-- 2. moves (操作日志, per §4.2 Move message + §7.1)
CREATE TABLE IF NOT EXISTS moves (
    move_id      UUID PRIMARY KEY,
    match_id     UUID NOT NULL REFERENCES game_sessions(match_id) ON DELETE CASCADE,
    player_id    TEXT NOT NULL,
    turn_index   INT NOT NULL,
    move_type    SMALLINT NOT NULL,                              -- MoveType: 0=unspec 1=play_card 2=attack 3=end_turn 4=surrender 5=use_ability
    payload_json JSONB NOT NULL DEFAULT '{}'::jsonb,             -- move 输入 (业务层解析)
    result_json  JSONB,                                          -- 业务层返回结果 (可空, 拒绝时无结果)
    accepted     BOOLEAN NOT NULL DEFAULT TRUE,
    reject_reason TEXT,                                          -- accepted=false 时填
    occurred_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_moves_match_id ON moves (match_id);
CREATE INDEX IF NOT EXISTS idx_moves_match_turn ON moves (match_id, turn_index);
CREATE INDEX IF NOT EXISTS idx_moves_player_id ON moves (player_id);
CREATE INDEX IF NOT EXISTS idx_moves_occurred_at ON moves (occurred_at);

-- ticket_id -> match_id 映射表 (EnqueueMatchmaking 队列, per §4.2 + §5.2)
-- 注: 简单实现, 后续桶 (匹配 saga) 可替换为更复杂队列 (Redis / NATS)
CREATE TABLE IF NOT EXISTS matchmaking_tickets (
    ticket_id          UUID PRIMARY KEY,
    player_id          TEXT NOT NULL,
    mode               SMALLINT NOT NULL,
    rank_score_min     INT NOT NULL DEFAULT 0,
    rank_score_max     INT NOT NULL DEFAULT 0,
    deck_ref_card_id   TEXT,
    deck_ref_inst_id   TEXT,
    status             SMALLINT NOT NULL DEFAULT 1,  -- 1=queued 2=matched 3=cancelled 4=expired
    match_id           UUID,                          -- MATCHED 时填 (REFERENCES game_sessions)
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    matched_at         TIMESTAMPTZ,
    cancelled_at       TIMESTAMPTZ,
    expires_at         TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '5 minutes')
);

CREATE INDEX IF NOT EXISTS idx_tickets_player ON matchmaking_tickets (player_id);
CREATE INDEX IF NOT EXISTS idx_tickets_status ON matchmaking_tickets (status);
CREATE INDEX IF NOT EXISTS idx_tickets_mode_score ON matchmaking_tickets (mode, rank_score_min, rank_score_max);
CREATE INDEX IF NOT EXISTS idx_tickets_expires ON matchmaking_tickets (expires_at);

-- session 事件订阅表 (per §4.2 SubscribeMatch 流式 RPC)
-- 注: 简单实现: NATS subject 派发事件, 此表仅记录订阅元数据 (供 GM 审计 / 重启恢复)
CREATE TABLE IF NOT EXISTS session_subscriptions (
    sub_id         UUID PRIMARY KEY,
    match_id       UUID NOT NULL REFERENCES game_sessions(match_id) ON DELETE CASCADE,
    player_id      TEXT NOT NULL,
    full_first     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at      TIMESTAMPTZ,
    UNIQUE (match_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_session_subs_match ON session_subscriptions (match_id);
