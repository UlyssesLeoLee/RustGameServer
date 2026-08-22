-- admin-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-019 §3）
-- 5 域管理域 admin_db schema 初始
-- 54.4 占位：仅最小 schema；54.5-54.7 业务实施时按 DTL-019 详细 entity 扩展

CREATE TABLE IF NOT EXISTS admin_ops (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 54.4 接受：占位最小 schema；GM 命令、公告、封禁 详细字段待 54.6 entity 实施