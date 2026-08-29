-- replay-service migration 0001_init (per WBS 桶 13 + RGS-DTL-038 §3 DEC-038-03 + §7.1 #7)
-- 卡牌游戏回放 replay_db schema 初始
-- 桶 13 实化: replays 表 (回放元数据, 数据在对象存储)
--
-- 设计原则 (per DEC-038-03 推荐 A):
-- - PostgreSQL 仅存元数据 (replay_id, match_id, players, mode, object_key, expires_at)
-- - 回放数据 (move log, board snapshots) 存对象存储 (cluster-ops S3-兼容)
--   本地用 LocalFs 模拟, 生产替换为 S3 / MinIO
--
-- 跨域引用 (per ARC-008 5 独立 DB 原则):
-- - match_id 跨域引用 game_sessions.match_id (match-service), 不物化 FK
-- - player_a / player_b 跨域引用 players.id (player-service), 不物化 FK
--
-- 生命周期:
-- - 默认 90 天 (天梯 RANKED) / 7 天 (休闲 CASUAL) / 30 天 (房间 ROOM)
-- - 过期清理: 启动时检查 + cron job (TODO 推 W36+)

CREATE TABLE IF NOT EXISTS replays (
    replay_id       UUID PRIMARY KEY,
    match_id        UUID NOT NULL,
    player_a        TEXT NOT NULL,
    player_b        TEXT,
    mode            SMALLINT NOT NULL,
    object_key      TEXT NOT NULL,
    object_size     BIGINT,
    duration_secs   INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL
);

-- 按 player_a 查该玩家的回放 (ListReplays 玩家维度, 走此索引)
CREATE INDEX IF NOT EXISTS idx_replays_player_a ON replays(player_a);

-- 按 player_b 查 (同上, 二者均为可选)
CREATE INDEX IF NOT EXISTS idx_replays_player_b ON replays(player_b);

-- 按 match_id 查 (一个 match_id 可能有多次回放保存重试 / 校验, 走此索引)
CREATE INDEX IF NOT EXISTS idx_replays_match_id ON replays(match_id);

-- 过期清理扫描 (cron job 走 expires_at < now() 范围, 高频)
CREATE INDEX IF NOT EXISTS idx_replays_expires ON replays(expires_at);
