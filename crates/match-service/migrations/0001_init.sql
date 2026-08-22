-- match-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-016 §3）
-- 5 域匹配域 match_db schema 初始
-- 54.4 占位：仅最小 schema；54.5-54.7 业务实施时按 DTL-016 详细 entity 扩展

CREATE TABLE IF NOT EXISTS matches (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 54.4 接受：占位最小 schema；房间匹配、对战撮合、Match Slot 详细字段待 54.6 entity 实施