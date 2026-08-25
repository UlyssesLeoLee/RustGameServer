//! ResumeTokenStore —— 断点记录持久化抽象
//!
//! 实现规格：RGS-SPEC-DTL-041 §3 + §6 + RGS-DTL-041 §3
//! 任务来源：RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.3~5
//!
//! ## 三种实现
//!
//! - [`ResumeTokenStore`]：trait（put / get / delete / list / cleanup_expired）
//! - [`JsonFileResumeTokenStore`]：每个 token 一个 `.json` 文件，原子写（tmp + rename + 内存索引）
//! - [`SqliteResumeTokenStore`]：单个 SQLite DB，LRU 100MB 上限（per RGS-SPEC-DTL-041 §8 `lru_max_bytes`）
//!
//! ## FR-CDN-064 不变式
//!
//! 所有 store 写入的 payload 都是 `ResumeToken` 序列化结果，类型层保证不含 PII。
//!
//! ## 原子写保证
//!
//! - JSON file：tmp + rename，crash-safe（POSIX rename 原子）
//! - SQLite：单事务（`rusqlite::Transaction`），自动 rollback on error

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::AssetDownloadError;
use crate::resume_token::ResumeToken;

// ---------------------------------------------------------------------------
// ResumeTokenStore trait
// ---------------------------------------------------------------------------

/// 断点记录存储 trait
///
/// 所有方法都是 `async` 以便未来切换到远程 store。
/// 错误语义：未找到不算错误，返回 `Ok(None)`；只有 IO / 序列化 / 后端错误才返回 `Err`。
#[async_trait]
pub trait ResumeTokenStore: Send + Sync {
    /// 写入 / 覆盖一条断点记录
    ///
    /// 写入语义：upsert（按 `token_id`）
    /// 原子性：必须原子（要不成功，要不不变）
    async fn put(&self, token: &ResumeToken) -> Result<(), AssetDownloadError>;

    /// 按 `token_id` 查询一条断点记录
    ///
    /// 未找到返回 `Ok(None)`；找到但已过期返回 `Ok(Some(token))`，由调用方决定是否清理。
    async fn get(&self, token_id: &str) -> Result<Option<ResumeToken>, AssetDownloadError>;

    /// 按 `token_id` 删除一条断点记录
    ///
    /// 返回是否实际删除（true = 已删除；false = 不存在）
    async fn delete(&self, token_id: &str) -> Result<bool, AssetDownloadError>;

    /// 列出所有断点记录（不做过期过滤，由调用方决定）
    async fn list(&self) -> Result<Vec<ResumeToken>, AssetDownloadError>;

    /// 清理所有已过期的断点记录
    ///
    /// 返回清理条数
    async fn cleanup_expired(&self) -> Result<usize, AssetDownloadError>;
}

// ===========================================================================
// JsonFileResumeTokenStore —— 文件系统 + 内存索引
// ===========================================================================

/// 文件系统实现的断点记录存储
///
/// 设计：
/// - 每个 token 一个 `<dir>/<token_id>.json` 文件
/// - 写入：先写 `.json.tmp.<rand>`，再 `rename` 到 `.json`（POSIX 原子）
/// - 内存 `HashMap<token_id, PathBuf>` 索引；启动时全量加载
/// - `list` / `get` 直接走内存索引（O(1) lookup / O(n) list）
/// - 适用场景：单设备 token 数量 < 1000 的轻量场景
#[derive(Debug)]
pub struct JsonFileResumeTokenStore {
    dir: PathBuf,
    index: Arc<Mutex<HashMap<String, PathBuf>>>,
}

impl JsonFileResumeTokenStore {
    /// 在 `dir` 下创建 / 加载 store
    pub async fn new(dir: impl Into<PathBuf>) -> Result<Self, AssetDownloadError> {
        let dir = dir.into();
        if !dir.exists() {
            tokio::fs::create_dir_all(&dir).await.map_err(|e| {
                AssetDownloadError::StoreIoError {
                    path: dir.display().to_string(),
                    cause: e.to_string(),
                }
            })?;
        }
        let store = Self {
            dir,
            index: Arc::new(Mutex::new(HashMap::new())),
        };
        store.reload_index().await?;
        Ok(store)
    }

    /// 全量扫描目录重建内存索引
    async fn reload_index(&self) -> Result<(), AssetDownloadError> {
        let mut index = self.index.lock().await;
        index.clear();

        let mut entries = tokio::fs::read_dir(&self.dir).await.map_err(|e| {
            AssetDownloadError::StoreIoError {
                path: self.dir.display().to_string(),
                cause: e.to_string(),
            }
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AssetDownloadError::StoreIoError {
                path: self.dir.display().to_string(),
                cause: e.to_string(),
            }
        })? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.ends_with(".json") || name.contains(".tmp.") {
                continue;
            }
            let token_id = name.trim_end_matches(".json").to_string();
            index.insert(token_id, path);
        }
        Ok(())
    }

    /// 读文件 → 反序列化
    async fn read_file(&self, path: &Path) -> Result<ResumeToken, AssetDownloadError> {
        let bytes = tokio::fs::read(path).await.map_err(|e| {
            AssetDownloadError::StoreIoError {
                path: path.display().to_string(),
                cause: e.to_string(),
            }
        })?;
        serde_json::from_slice(&bytes).map_err(|e| {
            AssetDownloadError::StoreSerializationError(format!(
                "failed to deserialize token at {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// 原子写文件：tmp + rename
    async fn write_file_atomic(
        &self,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), AssetDownloadError> {
        // tmp 文件名加随机后缀，避免并发写冲突
        let tmp_name = format!(
            "{}.tmp.{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("token"),
            uuid::Uuid::new_v4().simple()
        );
        let tmp_path = path.with_file_name(tmp_name);
        tokio::fs::write(&tmp_path, bytes).await.map_err(|e| {
            AssetDownloadError::StoreIoError {
                path: tmp_path.display().to_string(),
                cause: e.to_string(),
            }
        })?;
        // POSIX rename 原子；Windows 上 ReplaceFile/POSIX rename 也原子
        if let Err(e) = tokio::fs::rename(&tmp_path, path).await {
            // 清理 tmp 残留
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(AssetDownloadError::StoreIoError {
                path: path.display().to_string(),
                cause: format!("rename from tmp failed: {e}"),
            });
        }
        Ok(())
    }
}

#[async_trait]
impl ResumeTokenStore for JsonFileResumeTokenStore {
    async fn put(&self, token: &ResumeToken) -> Result<(), AssetDownloadError> {
        let path = self.dir.join(format!("{}.json", token.token_id));
        let bytes = serde_json::to_vec_pretty(token).map_err(|e| {
            AssetDownloadError::StoreSerializationError(format!("serialize token: {e}"))
        })?;
        self.write_file_atomic(&path, &bytes).await?;
        // 写后更索引
        let mut index = self.index.lock().await;
        index.insert(token.token_id.clone(), path);
        debug!(token_id = %token.token_id, "JsonFileResumeTokenStore::put ok");
        Ok(())
    }

    async fn get(&self, token_id: &str) -> Result<Option<ResumeToken>, AssetDownloadError> {
        let path = {
            let index = self.index.lock().await;
            index.get(token_id).cloned()
        };
        let Some(path) = path else {
            return Ok(None);
        };
        let token = self.read_file(&path).await?;
        Ok(Some(token))
    }

    async fn delete(&self, token_id: &str) -> Result<bool, AssetDownloadError> {
        let path = {
            let mut index = self.index.lock().await;
            index.remove(token_id)
        };
        let Some(path) = path else {
            return Ok(false);
        };
        if path.exists() {
            tokio::fs::remove_file(&path).await.map_err(|e| {
                AssetDownloadError::StoreIoError {
                    path: path.display().to_string(),
                    cause: e.to_string(),
                }
            })?;
        }
        debug!(token_id = %token_id, "JsonFileResumeTokenStore::delete ok");
        Ok(true)
    }

    async fn list(&self) -> Result<Vec<ResumeToken>, AssetDownloadError> {
        let paths: Vec<PathBuf> = {
            let index = self.index.lock().await;
            index.values().cloned().collect()
        };
        let mut out = Vec::with_capacity(paths.len());
        for path in paths {
            match self.read_file(&path).await {
                Ok(t) => out.push(t),
                Err(e) => {
                    // 单个文件损坏不中断列表，只 warn
                    warn!(path = %path.display(), error = %e, "skip corrupted token file");
                }
            }
        }
        Ok(out)
    }

    async fn cleanup_expired(&self) -> Result<usize, AssetDownloadError> {
        let now = Utc::now();
        let all = self.list().await?;
        let mut removed = 0;
        for token in all {
            if token.is_expired(now) && self.delete(&token.token_id).await? {
                removed += 1;
            }
        }
        if removed > 0 {
            info!(count = removed, "JsonFileResumeTokenStore::cleanup_expired done");
        }
        Ok(removed)
    }
}

// ===========================================================================
// SqliteResumeTokenStore —— SQLite + LRU 100MB
// ===========================================================================

/// 100MB LRU 上限（per RGS-SPEC-DTL-041 §8 `lru_max_bytes`）
pub const DEFAULT_LRU_MAX_BYTES: u64 = 100 * 1024 * 1024;

/// SQLite 默认 schema 初始化语句
const SCHEMA_SQL: &str = r#"
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
CREATE INDEX IF NOT EXISTS idx_resume_tokens_asset_id ON resume_tokens(asset_id);
CREATE INDEX IF NOT EXISTS idx_resume_tokens_expires_at ON resume_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_resume_tokens_updated_at ON resume_tokens(updated_at);
"#;

/// SQLite 实现的断点记录存储（per M-2064.5）
///
/// 设计：
/// - 单个 SQLite DB 文件
/// - 表 schema 见 `SCHEMA_SQL`（同步见 `migrations/0001_resume_token_index.sql`）
/// - payload 字段是 `ResumeToken` 的 JSON 序列化（**不**走 BLOB opaque 路径，便于调试 / 跨 schema 兼容）
/// - 100MB LRU 上限：每次 put 后检查总 payload 字节数；超限按 `updated_at ASC` 驱逐，直到 < 90% 上限
/// - rusqlite 是同步 API，包装在 `spawn_blocking` 里跑
#[derive(Debug)]
pub struct SqliteResumeTokenStore {
    /// SQLite DB 路径（保留供诊断 / 迁移使用；M-2065 可能读取）
    #[allow(dead_code)]
    db_path: PathBuf,
    conn: Arc<Mutex<Connection>>,
    lru_max_bytes: u64,
}

impl SqliteResumeTokenStore {
    /// 在 `db_path` 上创建 / 加载 store，默认 100MB LRU
    pub async fn new(db_path: impl Into<PathBuf>) -> Result<Self, AssetDownloadError> {
        Self::with_lru(db_path, DEFAULT_LRU_MAX_BYTES).await
    }

    /// 在 `db_path` 上创建 / 加载 store，指定 LRU 上限
    pub async fn with_lru(
        db_path: impl Into<PathBuf>,
        lru_max_bytes: u64,
    ) -> Result<Self, AssetDownloadError> {
        let db_path = db_path.into();
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    AssetDownloadError::StoreIoError {
                        path: parent.display().to_string(),
                        cause: e.to_string(),
                    }
                })?;
            }
        }

        let path_for_blocking = db_path.clone();
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection, rusqlite::Error> {
            let conn = Connection::open(&path_for_blocking)?;
            // WAL 模式提升并发读性能
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.execute_batch(SCHEMA_SQL)?;
            Ok(conn)
        })
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite open: {e}")))?;

        Ok(Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
            lru_max_bytes,
        })
    }

    /// 估算总 payload 字节数
    async fn total_payload_bytes(&self) -> Result<u64, AssetDownloadError> {
        let conn = self.conn.clone();
        let total: i64 = tokio::task::spawn_blocking(move || -> Result<i64, rusqlite::Error> {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare("SELECT COALESCE(SUM(payload_size), 0) FROM resume_tokens")?;
            let total: i64 = stmt.query_row([], |row| row.get(0))?;
            Ok(total)
        })
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite sum: {e}")))?;
        Ok(total.max(0) as u64)
    }

    /// 驱逐最旧的 token 直到总 payload < 90% 上限（防抖）
    async fn evict_until_under_limit(&self) -> Result<usize, AssetDownloadError> {
        let target = self.lru_max_bytes.saturating_mul(9) / 10; // 90%
        let mut evicted = 0;
        loop {
            let total = self.total_payload_bytes().await?;
            if total < self.lru_max_bytes {
                break;
            }
            // 取最旧的一条
            let conn = self.conn.clone();
            let oldest: Option<(String, i64)> = tokio::task::spawn_blocking(
                move || -> Result<Option<(String, i64)>, rusqlite::Error> {
                    let conn = conn.blocking_lock();
                    let mut stmt = conn.prepare(
                        "SELECT token_id, payload_size FROM resume_tokens \
                         ORDER BY updated_at ASC LIMIT 1",
                    )?;
                    let row = stmt
                        .query_row([], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                        })
                        .optional()?;
                    Ok(row)
                },
            )
            .await
            .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
            .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite select: {e}")))?;

            let Some((token_id, size)) = oldest else {
                break;
            };
            // 删除这一条
            let conn = self.conn.clone();
            let token_id_clone = token_id.clone();
            let deleted: usize = tokio::task::spawn_blocking(move || -> Result<usize, rusqlite::Error> {
                let conn = conn.blocking_lock();
                let n = conn.execute(
                    "DELETE FROM resume_tokens WHERE token_id = ?1",
                    params![token_id_clone],
                )?;
                Ok(n)
            })
            .await
            .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
            .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite delete: {e}")))?;
            if deleted == 0 {
                break;
            }
            evicted += 1;
            warn!(
                token_id = %token_id,
                size = size,
                target = target,
                "SqliteResumeTokenStore LRU eviction"
            );
            // 防抖：避免 hot loop
            if total.saturating_sub(size as u64) < target {
                break;
            }
        }
        Ok(evicted)
    }
}

#[async_trait]
impl ResumeTokenStore for SqliteResumeTokenStore {
    async fn put(&self, token: &ResumeToken) -> Result<(), AssetDownloadError> {
        let payload = serde_json::to_vec(token).map_err(|e| {
            AssetDownloadError::StoreSerializationError(format!("serialize: {e}"))
        })?;
        let payload_size = payload.len() as i64;
        let token_id = token.token_id.clone();
        let asset_id = token.asset_id.clone();
        let status = token.status.as_str().to_string();
        let created_at = token.created_at.to_rfc3339();
        let updated_at = token.updated_at.to_rfc3339();
        let expires_at = token.expires_at.to_rfc3339();

        let conn = self.conn.clone();
        let payload_arc = Arc::new(payload);
        let payload_for_thread = Arc::clone(&payload_arc);

        tokio::task::spawn_blocking(move || -> Result<(), rusqlite::Error> {
            let conn = conn.blocking_lock();
            conn.execute(
                "INSERT INTO resume_tokens \
                 (token_id, asset_id, status, payload, payload_size, created_at, updated_at, expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
                 ON CONFLICT(token_id) DO UPDATE SET \
                   asset_id=excluded.asset_id, \
                   status=excluded.status, \
                   payload=excluded.payload, \
                   payload_size=excluded.payload_size, \
                   updated_at=excluded.updated_at, \
                   expires_at=excluded.expires_at",
                params![
                    token_id,
                    asset_id,
                    status,
                    payload_for_thread,
                    payload_size,
                    created_at,
                    updated_at,
                    expires_at,
                ],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite upsert: {e}")))?;

        // LRU 触发检查
        let total = self.total_payload_bytes().await?;
        if total > self.lru_max_bytes {
            self.evict_until_under_limit().await?;
        }
        debug!(token_id = %token.token_id, payload_size, total, "SqliteResumeTokenStore::put ok");
        Ok(())
    }

    async fn get(&self, token_id: &str) -> Result<Option<ResumeToken>, AssetDownloadError> {
        let conn = self.conn.clone();
        let token_id_owned = token_id.to_string();
        let payload: Option<Vec<u8>> = tokio::task::spawn_blocking(
            move || -> Result<Option<Vec<u8>>, rusqlite::Error> {
                let conn = conn.blocking_lock();
                let mut stmt =
                    conn.prepare("SELECT payload FROM resume_tokens WHERE token_id = ?1")?;
                let row: Option<Vec<u8>> = stmt
                    .query_row(params![token_id_owned], |row| row.get(0))
                    .optional()?;
                Ok(row)
            },
        )
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite select: {e}")))?;

        let Some(bytes) = payload else {
            return Ok(None);
        };
        let token = serde_json::from_slice(&bytes).map_err(|e| {
            AssetDownloadError::StoreSerializationError(format!("deserialize: {e}"))
        })?;
        Ok(Some(token))
    }

    async fn delete(&self, token_id: &str) -> Result<bool, AssetDownloadError> {
        let conn = self.conn.clone();
        let token_id_owned = token_id.to_string();
        let n: usize = tokio::task::spawn_blocking(move || -> Result<usize, rusqlite::Error> {
            let conn = conn.blocking_lock();
            let n = conn.execute(
                "DELETE FROM resume_tokens WHERE token_id = ?1",
                params![token_id_owned],
            )?;
            Ok(n)
        })
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite delete: {e}")))?;
        Ok(n > 0)
    }

    async fn list(&self) -> Result<Vec<ResumeToken>, AssetDownloadError> {
        let conn = self.conn.clone();
        let payloads: Vec<Vec<u8>> = tokio::task::spawn_blocking(
            move || -> Result<Vec<Vec<u8>>, rusqlite::Error> {
                let conn = conn.blocking_lock();
                let mut stmt = conn.prepare("SELECT payload FROM resume_tokens")?;
                let rows = stmt
                    .query_map([], |row| row.get::<_, Vec<u8>>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            },
        )
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite list: {e}")))?;

        let mut out = Vec::with_capacity(payloads.len());
        for bytes in payloads {
            match serde_json::from_slice::<ResumeToken>(&bytes) {
                Ok(t) => out.push(t),
                Err(e) => {
                    warn!(error = %e, "skip corrupted sqlite resume_token payload");
                }
            }
        }
        Ok(out)
    }

    async fn cleanup_expired(&self) -> Result<usize, AssetDownloadError> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.clone();
        let n: usize = tokio::task::spawn_blocking(move || -> Result<usize, rusqlite::Error> {
            let conn = conn.blocking_lock();
            let n = conn.execute(
                "DELETE FROM resume_tokens WHERE expires_at <= ?1",
                params![now],
            )?;
            Ok(n)
        })
        .await
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("join error: {e}")))?
        .map_err(|e| AssetDownloadError::StoreBackendError(format!("sqlite cleanup: {e}")))?;
        if n > 0 {
            info!(count = n, "SqliteResumeTokenStore::cleanup_expired done");
        }
        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// 测试模块
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resume_token::{ResumeToken, RESUME_TOKEN_TTL_DAYS};
    use crate::state_machine::DownloadState;
    use chrono::Duration;
    use tempfile::TempDir;

    fn sha256_hex() -> String {
        "a".repeat(64)
    }

    fn make_token(asset_id: &str, path: PathBuf) -> ResumeToken {
        ResumeToken::new(
            asset_id,
            path,
            1024 * 1024,
            64 * 1024,
            "\"abc-etag\"",
            sha256_hex(),
            "https://cdn.example.com/asset.bin",
        )
        .expect("token")
    }

    #[tokio::test]
    async fn json_file_store_round_trip() {
        let dir = TempDir::new().unwrap();
        let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        let token = make_token("asset-001", dir.path().join("a.bin"));
        store.put(&token).await.unwrap();
        let got = store.get(&token.token_id).await.unwrap().expect("exists");
        assert_eq!(got, token);
        assert!(dir.path().join(format!("{}.json", token.token_id)).exists());
    }

    #[tokio::test]
    async fn json_file_store_get_missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        let got = store.get("nonexistent").await.unwrap();
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn json_file_store_delete_returns_true_then_false() {
        let dir = TempDir::new().unwrap();
        let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        let token = make_token("asset-x", dir.path().join("x.bin"));
        store.put(&token).await.unwrap();
        assert!(store.delete(&token.token_id).await.unwrap());
        // 再次 delete 返回 false
        assert!(!store.delete(&token.token_id).await.unwrap());
        // get 不存在
        assert!(store.get(&token.token_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn json_file_store_list_returns_all() {
        let dir = TempDir::new().unwrap();
        let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        for i in 0..5 {
            let t = make_token(&format!("asset-{i:03}"), dir.path().join(format!("a{i}.bin")));
            store.put(&t).await.unwrap();
        }
        let all = store.list().await.unwrap();
        assert_eq!(all.len(), 5);
    }

    #[tokio::test]
    async fn json_file_store_cleanup_expired_removes_only_expired() {
        let dir = TempDir::new().unwrap();
        let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        // 写入 1 个即将过期（人为设 expires_at 过去）
        let mut old = make_token("old", dir.path().join("old.bin"));
        old.expires_at = Utc::now() - Duration::days(1);
        store.put(&old).await.unwrap();
        // 写入 1 个未过期
        let fresh = make_token("fresh", dir.path().join("fresh.bin"));
        store.put(&fresh).await.unwrap();
        // cleanup
        let removed = store.cleanup_expired().await.unwrap();
        assert_eq!(removed, 1);
        // old 没了，fresh 还在
        assert!(store.get(&old.token_id).await.unwrap().is_none());
        assert!(store.get(&fresh.token_id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn json_file_store_atomic_write_no_tmp_leftover() {
        // 验证写完后没有 .tmp.* 残留
        let dir = TempDir::new().unwrap();
        let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        let t = make_token("a", dir.path().join("a.bin"));
        store.put(&t).await.unwrap();
        // 扫描目录
        let mut entries = tokio::fs::read_dir(dir.path()).await.unwrap();
        while let Some(e) = entries.next_entry().await.unwrap() {
            let name = e.file_name();
            let s = name.to_string_lossy();
            assert!(!s.contains(".tmp."), "tmp leftover: {s}");
        }
    }

    #[tokio::test]
    async fn json_file_store_idempotent_reload_after_restart() {
        let dir = TempDir::new().unwrap();
        let token = make_token("a", dir.path().join("a.bin"));
        {
            let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
                .await
                .unwrap();
            store.put(&token).await.unwrap();
        }
        // 重新打开
        let store2 = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        let got = store2.get(&token.token_id).await.unwrap().expect("exists");
        assert_eq!(got, token);
    }

    #[tokio::test]
    async fn sqlite_store_round_trip() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("tokens.db");
        let store = SqliteResumeTokenStore::new(db).await.unwrap();
        let token = make_token("asset-001", dir.path().join("a.bin"));
        store.put(&token).await.unwrap();
        let got = store.get(&token.token_id).await.unwrap().expect("exists");
        assert_eq!(got, token);
    }

    #[tokio::test]
    async fn sqlite_store_list_and_cleanup() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("tokens.db");
        let store = SqliteResumeTokenStore::new(db).await.unwrap();
        for i in 0..3 {
            let t = make_token(&format!("a-{i}"), dir.path().join(format!("a{i}.bin")));
            store.put(&t).await.unwrap();
        }
        assert_eq!(store.list().await.unwrap().len(), 3);
        // 全部未过期
        assert_eq!(store.cleanup_expired().await.unwrap(), 0);
        // 人工标记一个过期
        let all = store.list().await.unwrap();
        let mut t = all.into_iter().next().unwrap();
        t.expires_at = Utc::now() - Duration::days(1);
        t.set_status(DownloadState::Expired);
        store.put(&t).await.unwrap();
        assert_eq!(store.cleanup_expired().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn sqlite_store_lru_eviction_when_over_limit() {
        // 强制小 LRU 上限（1KB）→ 触发驱逐
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("tokens.db");
        let store = SqliteResumeTokenStore::with_lru(db, 1024)
            .await
            .unwrap();
        // 连续 put 多个 token；每个 token 的 payload ~ 1KB+ → 触发 LRU
        for i in 0..5 {
            let t = make_token(
                &format!("asset-lru-{i}"),
                dir.path().join(format!("a{i}.bin")),
            );
            store.put(&t).await.unwrap();
        }
        let total = store.total_payload_bytes().await.unwrap();
        // 应当已驱逐到 < 1MB
        assert!(total <= 1024, "total={total}, lru did not evict");
    }

    #[test]
    fn lru_default_is_100mb() {
        assert_eq!(DEFAULT_LRU_MAX_BYTES, 100 * 1024 * 1024);
    }

    #[test]
    fn resume_token_ttl_is_7_days() {
        assert_eq!(RESUME_TOKEN_TTL_DAYS, 7);
    }
}
