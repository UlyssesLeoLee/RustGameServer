-- leaderboard-service migration 0001_init (per RGS-REQ-038 §FR-007 + RGS-DTL-038 §3 DEC-038-02)
-- 卡牌游戏 4 类排行榜 leaderboard_db schema 初始
-- 表设计: leaderboard_entries (玩家在 (type, period, season_id) 维度上的条目)
-- 一条记录 = 一个玩家在一个榜单一个周期内的当前 rank/score/wins/losses
-- rank 通过 (type, period, season_id) partition + score DESC 索引获得

CREATE TABLE IF NOT EXISTS leaderboard_entries (
    id UUID PRIMARY KEY,
    leaderboard_type TEXT NOT NULL CHECK (leaderboard_type IN ('ranked', 'casual', 'collection')),
    period TEXT NOT NULL CHECK (period IN ('weekly', 'monthly', 'seasonal', 'all_time')),
    -- season_id 仅 ranked 必填, 其他可空字符串
    season_id TEXT NOT NULL DEFAULT '',
    player_id UUID NOT NULL,
    display_name TEXT NOT NULL,
    score BIGINT NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    -- 排名 1-based; 0 = 尚未入榜
    rank INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- (type, period, season_id, player_id) 唯一
    UNIQUE (leaderboard_type, period, season_id, player_id)
);

-- 高频读路径索引: 按 (type, period, season_id) 范围 + score DESC 排序
CREATE INDEX IF NOT EXISTS idx_lb_type_period_season_score
    ON leaderboard_entries (leaderboard_type, period, season_id, score DESC);

-- 按玩家查所有榜单位置 (GetPlayerRank 走此索引)
CREATE INDEX IF NOT EXISTS idx_lb_player
    ON leaderboard_entries (player_id);
