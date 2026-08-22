-- admin-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-019 §3 + ARC-051 COC/CEM + SEC-100 §7）
-- 5 域管理域 admin_db schema 初始
-- 54.6 实化：admin_users（RBAC）+ audit_log（hash 链 + UPDATE/DELETE 触发器禁）

CREATE TABLE IF NOT EXISTS admin_users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('super_admin', 'domain_admin', 'auditor', 'support')),
    domain_scope TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ,
    disabled_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_admin_users_role ON admin_users (role);
CREATE INDEX IF NOT EXISTS idx_admin_users_disabled_at ON admin_users (disabled_at);

-- 审计日志（per RGS-SEC-100 §7 hash 链防篡改）
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    actor_id UUID NOT NULL,
    action TEXT NOT NULL,
    target TEXT NOT NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_actor_id ON audit_log (actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log (action);
CREATE INDEX IF NOT EXISTS idx_audit_created_at ON audit_log (created_at);

-- 禁 UPDATE/DELETE 触发器（per RGS-SEC-100 §7）
CREATE OR REPLACE FUNCTION audit_log_no_modify() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit_log is append-only (per RGS-SEC-100 §7)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS audit_log_no_update ON audit_log;
CREATE TRIGGER audit_log_no_update
    BEFORE UPDATE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_log_no_modify();

DROP TRIGGER IF EXISTS audit_log_no_delete ON audit_log;
CREATE TRIGGER audit_log_no_delete
    BEFORE DELETE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_log_no_modify();
