-- cluster-ops migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-020 §3 + ARC-051）
-- 5 域集群运营域 cluster_ops_db schema 初始
-- 54.6 实化：cluster_nodes（跨服 Active-Active）+ feature_flags（PFAU all-reachable）

CREATE TABLE IF NOT EXISTS cluster_nodes (
    id UUID PRIMARY KEY,
    hostname TEXT NOT NULL UNIQUE,
    ip TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('primary', 'replica', 'candidate')),
    status TEXT NOT NULL DEFAULT 'healthy'
        CHECK (status IN ('healthy', 'degraded', 'unhealthy', 'maintenance')),
    last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version TEXT NOT NULL,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    enabled_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_nodes_status ON cluster_nodes (status);
CREATE INDEX IF NOT EXISTS idx_nodes_role ON cluster_nodes (role);
CREATE INDEX IF NOT EXISTS idx_nodes_heartbeat ON cluster_nodes (last_heartbeat_at);

-- PFAU 功能开关（per DEC-002 all-reachable）
CREATE TABLE IF NOT EXISTS feature_flags (
    key TEXT NOT NULL,
    scope TEXT NOT NULL CHECK (scope IN ('global', 'domain', 'node')),
    scope_value TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    version BIGINT NOT NULL DEFAULT 0,
    updated_by UUID NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (key, scope_value)
);

CREATE INDEX IF NOT EXISTS idx_flags_scope_value ON feature_flags (scope_value);
CREATE INDEX IF NOT EXISTS idx_flags_enabled ON feature_flags (enabled);
