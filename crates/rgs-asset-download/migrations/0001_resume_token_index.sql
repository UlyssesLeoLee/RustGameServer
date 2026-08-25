-- 0001_resume_token_index.sql
--
-- 断点记录表 + 索引（per RGS-SPEC-DTL-041 §6 + RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.5）
--
-- **说明**：
-- - 本 SQL 是显式 migration 形式（与 `src/resume_token_store.rs::SCHEMA_SQL` 内容一致）
-- - 应用启动时 `SqliteResumeTokenStore::new` 会自动跑 `CREATE TABLE IF NOT EXISTS` 建表
-- - 手工部署 / 调试可使用 `sqlite3 downloads.db < 0001_resume_token_index.sql`
--
-- **FR-CDN-064**：表结构**不**包含 PII 字段（player_id / device_id / email / ip / mac 全部 NOT in schema）
--
-- **回滚**：
-- ```sql
-- DROP INDEX IF EXISTS idx_resume_tokens_updated_at;
-- DROP INDEX IF EXISTS idx_resume_tokens_expires_at;
-- DROP INDEX IF EXISTS idx_resume_tokens_asset_id;
-- DROP TABLE IF EXISTS resume_tokens;
-- ```

CREATE TABLE IF NOT EXISTS resume_tokens (
    token_id     TEXT PRIMARY KEY NOT NULL,
    asset_id     TEXT NOT NULL,
    status       TEXT NOT NULL,
    payload      BLOB NOT NULL,
    payload_size INTEGER NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    expires_at   TEXT NOT NULL
);

-- 按 asset_id 查询（同一资产多次下载会话）
CREATE INDEX IF NOT EXISTS idx_resume_tokens_asset_id
    ON resume_tokens(asset_id);

-- 过期清理扫描（cleanup_expired）
CREATE INDEX IF NOT EXISTS idx_resume_tokens_expires_at
    ON resume_tokens(expires_at);

-- LRU 驱逐（按 updated_at 升序驱逐最旧）
CREATE INDEX IF NOT EXISTS idx_resume_tokens_updated_at
    ON resume_tokens(updated_at);
