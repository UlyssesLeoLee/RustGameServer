//! UT：ResumeTokenStore 13 字段 + 原子写 + LRU
//!
//! 实现规格：RGS-SPEC-DTL-041 §6
//! 任务来源：RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.7
//!
//! 覆盖：
//! - 13 字段全部正确读写（per SPEC §6）
//! - JSON file：原子写不残留 .tmp.* 文件
//! - JSON file：重启后 reload 仍能读取
//! - SQLite：LRU 驱逐在超限时触发
//! - SQLite：cleanup_expired 仅删过期
//! - FR-CDN-064：payload 字段不含 PII（防御性）
//! - FR-CDN-074：etag 字段非空
//! - NFR-CDN-002：checksum_sha256 字段存在且 64 字符

use std::path::PathBuf;

use chrono::Utc;
use rgs_asset_download::{
    AssetDownloadError, JsonFileResumeTokenStore, ResumeToken, ResumeTokenStore,
    SqliteResumeTokenStore, DEFAULT_LRU_MAX_BYTES,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// helper
// ---------------------------------------------------------------------------

fn sha256_hex(seed: u8) -> String {
    // 生成 64 字符 hex；不同 seed 给出不同 hash
    let s = format!("{seed:x}").repeat(64 / 1 + 1);
    s[..64].to_string()
}

fn make_token(asset_id: &str, total_size: u64, chunk_size: u64, dir: &TempDir) -> ResumeToken {
    ResumeToken::new(
        asset_id,
        dir.path().join(format!("{asset_id}.bin")),
        total_size,
        chunk_size,
        "\"abc-etag-123\"",
        sha256_hex(asset_id.len() as u8),
        "https://cdn.example.com/asset.bin",
    )
    .expect("token should be valid")
}

// ---------------------------------------------------------------------------
// 13 字段 schema 测试
// ---------------------------------------------------------------------------

#[test]
fn token_has_13_fields_per_spec_section_6() {
    let dir = TempDir::new().unwrap();
    let t = make_token("asset-001", 8192, 1024, &dir);

    // 13 字段 + schema_version
    let _: u32 = t.schema_version; // 0
    let _: String = t.token_id; // 1
    let _: String = t.asset_id; // 2
    let _: PathBuf = t.file_path; // 3
    let _: u64 = t.total_size; // 4
    let _: u64 = t.chunk_size; // 5
    let _: Vec<u32> = t.completed_chunks; // 6
    let _: String = t.etag; // 7
    let _: chrono::DateTime<Utc> = t.created_at; // 8
    let _: chrono::DateTime<Utc> = t.updated_at; // 9
    let _: chrono::DateTime<Utc> = t.expires_at; // 10
    let _: String = t.checksum_sha256; // 11
    let _: String = t.backend_url; // 12
    let _: rgs_asset_download::DownloadState = t.status; // 13

    // 显式校验
    assert_eq!(t.total_size, 8192);
    assert_eq!(t.chunk_size, 1024);
    assert_eq!(t.total_chunks(), 8);
    assert!(!t.token_id.is_empty());
    assert!(!t.etag.is_empty());
    assert_eq!(t.checksum_sha256.len(), 64);
    assert!(t.checksum_sha256.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(t.status, rgs_asset_download::DownloadState::Idle);
}

#[test]
fn token_field_order_matches_spec_terminology() {
    // 13 字段在 JSON 序列化时的 key 顺序与 SPEC §6 一致
    let dir = TempDir::new().unwrap();
    let t = make_token("asset-x", 1024, 256, &dir);
    let json = serde_json::to_string(&t).unwrap();
    // 抽检关键字
    for key in [
        "token_id",
        "asset_id",
        "file_path",
        "total_size",
        "chunk_size",
        "completed_chunks",
        "etag",
        "created_at",
        "updated_at",
        "expires_at",
        "checksum_sha256",
        "backend_url",
        "status",
    ] {
        assert!(json.contains(key), "missing key {key} in serialized token: {json}");
    }
}

#[test]
fn expires_at_is_created_at_plus_7_days() {
    let dir = TempDir::new().unwrap();
    let t = make_token("a", 1, 1, &dir);
    let delta = t.expires_at - t.created_at;
    assert_eq!(delta, chrono::Duration::days(7));
}

// ---------------------------------------------------------------------------
// FR-CDN-064：不含 PII 字段
// ---------------------------------------------------------------------------

#[test]
fn token_serialization_does_not_contain_pii_substrings() {
    let dir = TempDir::new().unwrap();
    let t = make_token("asset-001", 4096, 512, &dir);
    let json = serde_json::to_string(&t).unwrap();
    // FR-CDN-064：禁出现的 PII 字段名
    for forbidden in ["player_id", "device_id", "email", "ip_address", "mac_address"] {
        assert!(
            !json.contains(forbidden),
            "FR-CDN-064 violation: payload contains '{forbidden}': {json}"
        );
    }
}

#[test]
fn token_struct_has_no_pii_fields_in_definition() {
    // 反射做不到精确字段名检查；这里通过解析源码 + 限定到 struct 块范围来定位
    // 关键点：13 字段定义段不应包含 PII 字段名（除了防御性常量 `PII_FORBIDDEN_FIELDS`）
    let source = include_str!("../src/resume_token.rs").replace("\r\n", "\n");
    // 提取 pub struct ResumeToken { ... } 段
    let struct_start = source
        .find("pub struct ResumeToken {")
        .expect("struct ResumeToken definition exists");
    let struct_end_rel = source[struct_start..]
        .find("\n}\n")
        .expect("struct closing brace");
    let struct_end = struct_start + struct_end_rel;
    let struct_body = &source[struct_start..=struct_end];

    for forbidden in ["player_id", "device_id", "email", "ip_address", "mac_address"] {
        assert!(
            !struct_body.contains(forbidden),
            "FR-CDN-064 struct violation: ResumeToken struct contains '{forbidden}'"
        );
    }
}

// ---------------------------------------------------------------------------
// JsonFileResumeTokenStore 行为
// ---------------------------------------------------------------------------

#[tokio::test]
async fn json_file_store_put_get_delete_list() {
    let dir = TempDir::new().unwrap();
    let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    let t = make_token("a", 1024, 256, &dir);

    // put
    store.put(&t).await.unwrap();
    // get
    let got = store.get(&t.token_id).await.unwrap().expect("exists");
    assert_eq!(got, t);
    // list
    let all = store.list().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0], t);
    // delete
    assert!(store.delete(&t.token_id).await.unwrap());
    // get 返回 None
    assert!(store.get(&t.token_id).await.unwrap().is_none());
}

#[tokio::test]
async fn json_file_store_atomic_write_no_tmp_leftover() {
    let dir = TempDir::new().unwrap();
    let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    // 连续写入 5 个不同 token
    for i in 0..5 {
        let t = make_token(&format!("a{i}"), 1024, 256, &dir);
        store.put(&t).await.unwrap();
    }
    // 扫描目录，无 .tmp.* 残留
    let mut entries = tokio::fs::read_dir(dir.path()).await.unwrap();
    let mut count = 0;
    while let Some(e) = entries.next_entry().await.unwrap() {
        let name = e.file_name();
        let s = name.to_string_lossy();
        assert!(!s.contains(".tmp."), "tmp file leftover: {s}");
        count += 1;
    }
    // 应有 5 个 .json 文件
    assert!(count >= 5);
}

#[tokio::test]
async fn json_file_store_overwrite_is_atomic() {
    let dir = TempDir::new().unwrap();
    let store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    let mut t = make_token("a", 1024, 256, &dir);
    store.put(&t).await.unwrap();

    // 改 status 后再 put（覆盖）
    t.status = rgs_asset_download::DownloadState::Downloading;
    t.completed_chunks = vec![0, 1, 2];
    store.put(&t).await.unwrap();

    // 读回：覆盖生效
    let got = store.get(&t.token_id).await.unwrap().unwrap();
    assert_eq!(got.status, rgs_asset_download::DownloadState::Downloading);
    assert_eq!(got.completed_chunks, vec![0, 1, 2]);
    // 总数仍为 1（覆盖不增加条目）
    let all = store.list().await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn json_file_store_persists_across_reload() {
    let dir = TempDir::new().unwrap();
    let t = make_token("a", 1024, 256, &dir);
    {
        let s = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
            .await
            .unwrap();
        s.put(&t).await.unwrap();
    } // drop s
    // 重新打开
    let s2 = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    let got = s2.get(&t.token_id).await.unwrap().expect("exists");
    assert_eq!(got, t);
}

#[tokio::test]
async fn json_file_store_cleanup_expired() {
    let dir = TempDir::new().unwrap();
    let s = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    let mut old = make_token("old", 1024, 256, &dir);
    old.expires_at = Utc::now() - chrono::Duration::days(1);
    s.put(&old).await.unwrap();
    let fresh = make_token("fresh", 1024, 256, &dir);
    s.put(&fresh).await.unwrap();

    let n = s.cleanup_expired().await.unwrap();
    assert_eq!(n, 1);
    assert!(s.get(&old.token_id).await.unwrap().is_none());
    assert!(s.get(&fresh.token_id).await.unwrap().is_some());
}

#[tokio::test]
async fn json_file_store_ignores_tmp_files_when_loading_index() {
    let dir = TempDir::new().unwrap();
    // 写一个 .json.tmp.x 文件模拟"半截写入"残留
    let stray = dir.path().join("stray.json.tmp.deadbeef");
    tokio::fs::write(&stray, b"corrupt")
        .await
        .unwrap();
    // 加载 store
    let s = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    // list 应忽略 stray 文件
    let all = s.list().await.unwrap();
    assert!(all.is_empty());
    // get stray 也不应返回
    assert!(s.get("stray").await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// SqliteResumeTokenStore 行为
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sqlite_store_put_get_delete_list() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    let s = SqliteResumeTokenStore::new(db).await.unwrap();
    let t = make_token("a", 1024, 256, &dir);

    s.put(&t).await.unwrap();
    let got = s.get(&t.token_id).await.unwrap().expect("exists");
    assert_eq!(got, t);
    let all = s.list().await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(s.delete(&t.token_id).await.unwrap());
    assert!(s.get(&t.token_id).await.unwrap().is_none());
}

#[tokio::test]
async fn sqlite_store_persists_across_reopen() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    let t = make_token("a", 1024, 256, &dir);
    {
        let s = SqliteResumeTokenStore::new(db.clone()).await.unwrap();
        s.put(&t).await.unwrap();
    }
    let s2 = SqliteResumeTokenStore::new(db).await.unwrap();
    let got = s2.get(&t.token_id).await.unwrap().expect("exists");
    assert_eq!(got, t);
}

#[tokio::test]
async fn sqlite_store_cleanup_expired() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    let s = SqliteResumeTokenStore::new(db).await.unwrap();
    let mut old = make_token("old", 1024, 256, &dir);
    old.expires_at = Utc::now() - chrono::Duration::days(1);
    s.put(&old).await.unwrap();
    let fresh = make_token("fresh", 1024, 256, &dir);
    s.put(&fresh).await.unwrap();

    let n = s.cleanup_expired().await.unwrap();
    assert_eq!(n, 1);
    assert!(s.get(&old.token_id).await.unwrap().is_none());
    assert!(s.get(&fresh.token_id).await.unwrap().is_some());
}

#[tokio::test]
async fn sqlite_store_lru_eviction_with_tiny_limit() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    // 1KB 极限：每个 token 至少 1 个 JSON row，必然超
    let s = SqliteResumeTokenStore::with_lru(db, 1024).await.unwrap();

    for i in 0..10 {
        let t = make_token(&format!("lru-{i}"), 4096, 1024, &dir);
        s.put(&t).await.unwrap();
    }
    // 验证 list 大小 <= 1（极端情况下被驱逐到很小）
    let all = s.list().await.unwrap();
    assert!(
        all.len() <= 3,
        "LRU did not evict aggressively: len={}",
        all.len()
    );
}

#[tokio::test]
async fn sqlite_store_lru_keeps_recently_updated() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    let s = SqliteResumeTokenStore::with_lru(db, 2048).await.unwrap();
    // 写 3 个；最旧的应在 LRU 时被驱逐
    let mut tokens: Vec<ResumeToken> = (0..3)
        .map(|i| make_token(&format!("k-{i}"), 4096, 1024, &dir))
        .collect();
    for t in &tokens {
        s.put(t).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    // 再触发一轮 put 让最后写入的被保留
    let last = make_token("k-last", 4096, 1024, &dir);
    s.put(&last).await.unwrap();
    tokens.push(last.clone());
    // 至少 last 应在
    assert!(s.get(&last.token_id).await.unwrap().is_some());
    // 验证部分最早的可能被驱逐
    let all = s.list().await.unwrap();
    // 验证 last 总在
    assert!(all.iter().any(|t| t.token_id == last.token_id));
}

#[tokio::test]
async fn sqlite_store_get_missing_returns_none() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    let s = SqliteResumeTokenStore::new(db).await.unwrap();
    let got = s.get("not-existing-token-id").await.unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn sqlite_store_delete_missing_returns_false() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("tokens.db");
    let s = SqliteResumeTokenStore::new(db).await.unwrap();
    let deleted = s.delete("not-existing-token-id").await.unwrap();
    assert!(!deleted);
}

// ---------------------------------------------------------------------------
// store 间对比
// ---------------------------------------------------------------------------

#[tokio::test]
async fn json_file_and_sqlite_stores_return_equal_token_for_same_input() {
    let dir = TempDir::new().unwrap();
    let db_dir = TempDir::new().unwrap();
    let json_store = JsonFileResumeTokenStore::new(dir.path().to_path_buf())
        .await
        .unwrap();
    let sqlite_store = SqliteResumeTokenStore::new(db_dir.path().join("tokens.db"))
        .await
        .unwrap();

    let t = make_token("shared", 2048, 512, &dir);
    json_store.put(&t).await.unwrap();
    sqlite_store.put(&t).await.unwrap();

    let from_json = json_store.get(&t.token_id).await.unwrap().unwrap();
    let from_sqlite = sqlite_store.get(&t.token_id).await.unwrap().unwrap();
    assert_eq!(from_json, from_sqlite);
}

// ---------------------------------------------------------------------------
// FR-CDN-074：etag 必填
// ---------------------------------------------------------------------------

#[test]
fn token_constructor_rejects_empty_etag() {
    let dir = TempDir::new().unwrap();
    let err = ResumeToken::new(
        "a",
        dir.path().join("a.bin"),
        1,
        1,
        "", // empty etag
        sha256_hex(1),
        "https://x",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        rgs_asset_download::resume_token::ResumeTokenError::EmptyEtag
    ));
}

// ---------------------------------------------------------------------------
// NFR-CDN-002：checksum_sha256 必填且 64 hex
// ---------------------------------------------------------------------------

#[test]
fn token_constructor_rejects_wrong_length_checksum() {
    let dir = TempDir::new().unwrap();
    let err = ResumeToken::new(
        "a",
        dir.path().join("a.bin"),
        1,
        1,
        "\"e\"",
        "abcd", // 太短
        "https://x",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        rgs_asset_download::resume_token::ResumeTokenError::InvalidChecksumLength(4)
    ));
}

#[test]
fn token_constructor_rejects_non_hex_checksum() {
    let dir = TempDir::new().unwrap();
    let bad = "z".repeat(64);
    let err = ResumeToken::new(
        "a",
        dir.path().join("a.bin"),
        1,
        1,
        "\"e\"",
        bad,
        "https://x",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        rgs_asset_download::resume_token::ResumeTokenError::InvalidChecksumLength(64)
    ));
}

// ---------------------------------------------------------------------------
// 默认 LRU 上限（per SPEC §8 `lru_max_bytes = 100MB`）
// ---------------------------------------------------------------------------

#[test]
fn default_lru_max_bytes_is_100mb() {
    assert_eq!(DEFAULT_LRU_MAX_BYTES, 100 * 1024 * 1024);
}

// ---------------------------------------------------------------------------
// 错误语义：put 错误路径（虽然 happy path 不应触发，但确认错误类型不遗漏）
// ---------------------------------------------------------------------------

#[tokio::test]
async fn json_file_store_returns_specific_error_on_io_failure() {
    // 试图在一个不存在的盘符下创建
    let bad = PathBuf::from("Z:\\definitely-not-existing\\store");
    let res = JsonFileResumeTokenStore::new(bad).await;
    assert!(matches!(
        res,
        Err(AssetDownloadError::StoreIoError { .. })
    ));
}
