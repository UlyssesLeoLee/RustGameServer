-- cluster-ops migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-020 §3）
-- 5 域集群运营域 cluster_ops_db schema 初始
-- 54.4 占位：仅最小 schema；54.5-54.7 业务实施时按 DTL-020 详细 entity 扩展

CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 54.4 接受：占位最小 schema；跨服 Active-Active、CEM、PFAU 节点 详细字段待 54.6 entity 实施