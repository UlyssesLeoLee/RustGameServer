# 详细设计书（詳細設計書 / Detailed Design Document）

**客户端资源分发的断点续传与可恢复下载 Resumable & Recoverable Asset Download**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-041 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-036 客户端资源分发的断点续传与可恢复下载 基本设计书 |
| 父需求 | RGS-REQ-036 |
| 配套实现规格 | RGS-SPEC-CROSS-006 日志 trace_id 传播规范；RGS-SPEC-CROSS-004 DTO 领域实体映射规则；RGS-IMPL-002 PG 编码规范 |
| 决策依据 | RGS-ADR-0023 客户端核心逻辑单一实现多引擎薄适配层；RGS-ADR-0044 客户端资源分发默认自托管开源 |
| 协同文档 | RGS-BAS-008 客户端引擎适配层与 SDK；RGS-BAS-027 客户端资源分发与热更新；RGS-REQ-001 NFR-PE 系列 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 适用许可 | Apache-2.0（本仓库） |

> 本文档落实 RGS-BAS-036 全部组件、状态机、Schema、契约、补足 Rust 类型签名 / 错误语义 / Saga 步骤 / 客户端 SDK crate 划分 / 平台特定实现要点。详细到可被直接编码的程度，但仍保留少量"由详细编码阶段决定"的 TBD（仅限具体超时阈值、并发数动态调参算法等无法在文档层级预定的实现细节）。

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 首版草案。落实 RGS-BAS-036：①客户端 SDK `asset_download` 模块的 Rust crate 边界（`rgs-asset-download`）；②`DownloadStateMachine` 状态机的类型与转移合法性；③`ResumeTokenStore` 的 SQLite + JSON 双层存储；④`RangeClient` 的 HTTP 客户端选型与协议实现；⑤`ChunkOrchestrator` 的并发分片调度伪代码；⑥`IntegrityGate` 的整文件 hash 计算与 Manifest 比对；⑦平台特定实现（Unix sparse file / Windows SetFileValidData / 移动平台 pre-allocate）；⑧与既有 `asset_update` / `version` 模块的集成时序；⑨可观测性埋点（`metrics` crate 计数器/直方图）| 全文 |
| 0.2 | 2026-08-21 | Ulysses(一人公司 12 角色兼任 per DEC-008) | Ulysses(同) | 具名人类审批完成(per RGS-WBS-001 §17 集体签字声明):一人公司兼任体制下,Ulysses 在本表审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17。审批栏细化角色意见与 DEC-008 兼任对应关系见 RGS-REQ-004 §3.10。**升 v0.2**: 文档从 v0.1 草案转为 v0.2 具名审批版,生产基线化仍需 G-CODE-06 实测通过(per RGS-WF-001) | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 评审（架构） | 待指定 | — | 确认状态机、错误语义、跨平台实现边界 |
| 评审（客户端/平台） | 待指定 | — | 确认 SDK crate 划分与既有 `asset_update` / `version` 模块集成；移动平台 pre-allocate 实现 |
| 评审（SRE/可观测性） | 待指定 | — | 确认埋点指标接入既有 `RGS-BAS-004` 体系 |
| 评审（安全/合规） | 待指定 | — | 确认断点记录不含 PII；Range 协议不绕过完整性校验 |
| 审批（项目负责人） | 待指定 | — | 确认风险、范围、回滚条件与实施授权 |

| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 目录

1. [定位与非目标](#1-定位与非目标)
2. [crate 划分与依赖边界](#2-crate-划分与依赖边界)
3. [状态机类型与转移实现](#3-状态机类型与转移实现)
4. [ResumeTokenStore 持久化实现](#4-resumetokenstore-持久化实现)
5. [RangeClient 实现](#5-rangeclient-实现)
6. [ChunkOrchestrator 并发分片实现](#6-chunkorchestrator-并发分片实现)
7. [IntegrityGate 整文件校验实现](#7-integritygate-整文件校验实现)
8. [平台特定实现要点](#8-平台特定实现要点)
9. [与既有 SDK 模块的集成时序](#9-与既有-sdk-模块的集成时序)
10. [可观测性埋点接入](#10-可观测性埋点接入)
11. [故障、降级与异常处理](#11-故障降级与异常处理)
12. [验收证据与开放项](#12-验收证据与开放项)
13. [追溯性](#13-追溯性)

---

# 1. 定位与非目标

## 1.1 定位

`rgs-asset-download` 是客户端 SDK 新增 crate，实现断点续传与可恢复下载的**全部客户端侧逻辑**。它与既有 `rgs-asset-update`（Manifest/Delta/Rollout 编排）、`rgs-version`（协议版本协商）同级别，作为 SDK 三大模块之一。

**核心职责**：
- 接收来自 `rgs-asset-update` 的"需下载文件清单"，调度下载
- 通过 HTTP Range/HEAD 协议与 `DistributionBackend` 交互
- 在客户端本地维护断点状态机与断点记录
- 整文件下载完成后调用 `rgs-asset-update` 的 `IntegrityGate`（由本文档落地）做完整性校验
- 失败/异常情况下提供降级路径（回退为全量 GET / 取消分片 / 触发 Resuming 校验）

**非职责**（由既有模块负责）：
- Manifest 拉取 / 签名校验 / 灰度判定（`rgs-asset-update`）
- 协议版本协商（`rgs-version`）
- 业务侧应用决策（何时触发下载、何时拒绝使用，应用层负责）
- 网络层握手（QUIC/TCP，由既有 `rgs-network` crate）

## 1.2 非目标与硬禁止

- **不**实现 Manifest 拉取 / 签名校验 / 灰度判定——复用 `rgs-asset-update` 既有实现
- **不**实现协议版本协商——复用 `rgs-version` 既有实现
- **不**修改 `DistributionBackend` 抽象层（HTTP Range 是 HTTP 协议层能力，**不**下沉到抽象）
- **不**在断点记录中存放 PII（FR-CDN-064 硬约束）
- **不**绕过 `rgs-asset-update` 的 `IntegrityGate` 整文件校验（NFR-CDN-002 硬约束）
- **不**假设服务端有状态——断点信息 100% 在客户端本地

# 2. crate 划分与依赖边界

## 2.1 Cargo workspace 集成

`rgs-asset-download` 作为独立 crate 加入既有 `rgs-sdk` workspace（与 `rgs-asset-update` / `rgs-version` / `rgs-network` 同级）：

```toml
# rgs-sdk/Cargo.toml 新增 member
[workspace]
members = [
    "rgs-asset-update",
    "rgs-asset-download",   # ← 新增
    "rgs-version",
    "rgs-network",
    "rgs-quic",
    "rgs-cbindgen",
    # ...
]
```

## 2.2 crate 内部模块

```text
rgs-asset-download/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # 公开 API 入口
│   ├── error.rs                # DownloadError 错误类型
│   ├── state.rs                # DownloadState 状态枚举 + 状态机
│   ├── token.rs                # ResumeToken + ResumeTokenStore
│   ├── range_client.rs         # RangeClient: HTTP Range/HEAD 客户端
│   ├── chunk_orchestrator.rs   # ChunkOrchestrator: 并发分片调度
│   ├── integrity_gate.rs       # IntegrityGate: 整文件 hash 校验
│   ├── downloader.rs           # Downloader: 顶层协调者
│   ├── platform/
│   │   ├── mod.rs
│   │   ├── unix.rs             # Unix sparse file
│   │   ├── windows.rs          # Windows SetFileValidData
│   │   ├── android.rs          # Android pre-allocate
│   │   └── ios.rs              # iOS pre-allocate
│   └── metrics.rs              # 可观测性埋点
└── tests/
    ├── unit/
    │   ├── state_transitions.rs
    │   ├── resume_token_store.rs
    │   ├── range_client.rs
    │   ├── chunk_orchestrator.rs
    │   └── integrity_gate.rs
    └── integration/
        ├── resumable_download.rs
        ├── pause_resume.rs
        ├── etag_invalidation.rs
        └── gray_rollback.rs
```

## 2.3 依赖项

```toml
# rgs-asset-download/Cargo.toml
[dependencies]
rgs-asset-update = { path = "../rgs-asset-update" }
rgs-version = { path = "../rgs-version" }
rgs-network = { path = "../rgs-network" }

# HTTP 客户端（与既有 rgs-asset-update 同款）
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream"] }

# 异步运行时
tokio = { version = "1", features = ["full"] }

# SQLite (断点索引)
rusqlite = { version = "0.31", features = ["bundled"] }

# Hash 算法（与 Manifest 一致）
sha2 = "0.10"
blake3 = "1.5"

# 序列化（断点记录 JSON 化）
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# 时间
chrono = { version = "0.4", features = ["serde"] }

# 指标（接入既有 RGS-BAS-004 体系）
metrics = "0.22"

# 错误处理
thiserror = "1"
anyhow = "1"

# 日志
tracing = "0.1"
```

> **依赖收敛原则**：HTTP 客户端选型（reqwest）**必须**与既有 `rgs-asset-update` 同款，避免 SDK 内部两套 HTTP 客户端实现。

## 2.4 公开 API 入口

```rust
// src/lib.rs
pub use downloader::Downloader;
pub use state::DownloadState;
pub use token::ResumeToken;
pub use error::DownloadError;
pub use integrity_gate::IntegrityReport;

/// 触发下载入口
pub async fn download_asset(
    downloader: &Downloader,
    file_path: &str,
    source_url: &str,
) -> Result<(), DownloadError>;

/// 暂停下载
pub async fn pause_download(
    downloader: &Downloader,
    file_path: &str,
) -> Result<(), DownloadError>;

/// 取消下载（区别于暂停）
pub async fn cancel_download(
    downloader: &Downloader,
    file_path: &str,
) -> Result<(), DownloadError>;

/// 查询当前下载状态
pub async fn get_download_state(
    downloader: &Downloader,
    file_path: &str,
) -> Result<DownloadState, DownloadError>;
```

# 3. 状态机类型与转移实现

## 3.1 状态枚举

```rust
// src/state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DownloadState {
    NotStarted,
    Probing,
    Resuming,
    Downloading,
    Paused,
    Failed,
    Canceled,
    Completed,
}

impl DownloadState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Canceled | Self::Completed)
    }

    /// 状态机对应用：是否可从 `self` 转移到 `next`
    pub fn can_transition_to(self, next: DownloadState) -> bool {
        use DownloadState::*;
        match (self, next) {
            // 正常入口
            (NotStarted, Probing) => true,
            (Probing, Downloading) => true,
            (Probing, Resuming) => true,
            (Probing, Failed) => true,
            (Resuming, Downloading) => true,
            (Resuming, NotStarted) => true,  // 断点失效

            // 正常运行
            (Downloading, Paused) => true,
            (Downloading, Failed) => true,
            (Downloading, Completed) => true,
            (Downloading, Canceled) => true,

            // 暂停/失败恢复
            (Paused, Downloading) => true,  // 注: 实际会先转 Resuming
            (Failed, Resuming) => true,
            (Failed, NotStarted) => true,

            // 终态
            (Canceled, _) => false,
            (Completed, _) => false,

            // 其他
            _ => false,
        }
    }
}
```

## 3.2 状态机实现

```rust
// src/state.rs (续)
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::token::ResumeTokenStore;

pub struct DownloadStateMachine {
    file_path: String,
    current: Arc<RwLock<DownloadState>>,
    token_store: Arc<ResumeTokenStore>,
}

impl DownloadStateMachine {
    /// 状态转移，违反合法性时拒绝
    pub async fn transition(&self, next: DownloadState) -> Result<(), DownloadError> {
        let mut current = self.current.write().await;
        let prev = *current;
        if !prev.can_transition_to(next) {
            return Err(DownloadError::InvalidTransition { from: prev, to: next });
        }
        *current = next;
        // 状态变更同步到 ResumeTokenStore
        self.token_store.update_status(&self.file_path, next).await?;
        Ok(())
    }

    pub async fn current(&self) -> DownloadState {
        *self.current.read().await
    }

    /// 暂停时回到 Resuming 前的临时状态
    /// 设计要点: Paused 玩家恢复时, 实际会先重新 Resuming 校验 ETag/灰度/签名
    /// 这里的 Paused 仅作玩家意图记录, 恢复时由 downloader 重新调度
}
```

## 3.3 错误类型

```rust
// src/error.rs
use thiserror::Error;
use crate::state::DownloadState;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: DownloadState, to: DownloadState },

    #[error("range request failed: {0}")]
    RangeRequestFailed(#[from] reqwest::Error),

    #[error("server returned 416 Range Not Satisfiable: total_size={total_size}, requested_start={requested_start}")]
    RangeNotSatisfiable { total_size: u64, requested_start: u64 },

    #[error("server returned 200 OK (ETag changed): old_etag={old_etag:?}, new_etag={new_etag:?}")]
    ETagChanged { old_etag: String, new_etag: String },

    #[error("integrity check failed: expected={expected}, actual={actual}")]
    IntegrityCheckFailed { expected: String, actual: String },

    #[error("manifest signature invalid: {reason}")]
    ManifestSignatureInvalid { reason: String },

    #[error("gray rollout mismatch: file is no longer accessible to this player")]
    GrayRolledBack,

    #[error("resume token expired: last_updated_at={last_updated_at}")]
    ResumeTokenExpired { last_updated_at: chrono::DateTime<chrono::Utc> },

    #[error("disk space insufficient: required={required_bytes}, available={available_bytes}")]
    DiskSpaceInsufficient { required_bytes: u64, available_bytes: u64 },

    #[error("retry exhausted after {attempts} attempts")]
    RetryExhausted { attempts: u32 },

    #[error("HTTP 429 rate limited")]
    RateLimited,

    #[error("resume token store error: {0}")]
    TokenStoreError(String),

    #[error("server does not support Range (Accept-Ranges: none)")]
    RangeNotSupported,
}
```

# 4. ResumeTokenStore 持久化实现

## 4.1 数据结构

```rust
// src/token.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeToken {
    pub token_id: Uuid,
    pub file_path: String,
    pub source_url: String,
    pub etag: String,
    pub total_size: u64,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub chunk_manifest: Vec<ChunkRecord>,
    pub temp_file_path: PathBuf,
    pub last_updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub status: DownloadState,
    pub retry_count: u32,
    pub last_error: Option<String>,
    // 显式禁止: player_id / device_id / ip / mac 等 PII 字段
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Sha256,
    Blake3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub start: u64,
    pub end: u64,           // inclusive
    pub downloaded: bool,
    pub last_byte_at: DateTime<Utc>,
}
```

## 4.2 SQLite 索引

```sql
-- 在 `rgs-sdk` 启动时执行的 migration
CREATE TABLE IF NOT EXISTS resume_token_index (
    token_id           TEXT PRIMARY KEY,
    file_path          TEXT NOT NULL UNIQUE,
    status             TEXT NOT NULL,
    last_updated_at    INTEGER NOT NULL,    -- unix timestamp
    created_at         INTEGER NOT NULL,
    storage_size_bytes INTEGER NOT NULL
);

CREATE INDEX idx_resume_token_index_status_last_updated
    ON resume_token_index (status, last_updated_at);

CREATE INDEX idx_resume_token_index_last_updated
    ON resume_token_index (last_updated_at);
```

## 4.3 ResumeTokenStore 实现要点

```rust
// src/token.rs (续)
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;

pub struct ResumeTokenStore {
    base_dir: PathBuf,           // ~/.rgs-sdk/downloads/
    sqlite: Arc<Mutex<Connection>>,
    storage_limit_bytes: u64,    // NFR-CDN-113, 默认 100MB
}

impl ResumeTokenStore {
    pub fn new(base_dir: PathBuf) -> Result<Self, DownloadError> {
        std::fs::create_dir_all(&base_dir)?;
        let db_path = base_dir.join("index.sqlite");
        let conn = Connection::open(&db_path)?;
        // 执行 migration (见上 SQL)
        Ok(Self {
            base_dir,
            sqlite: Arc::new(Mutex::new(conn)),
            storage_limit_bytes: 100 * 1024 * 1024,  // 100MB
        })
    }

    /// 写入断点记录 (FR-CDN-061 原子写: 先 SQLite 再 JSON)
    pub async fn upsert(&self, token: &ResumeToken) -> Result<(), DownloadError> {
        // 1. 写 JSON 文件
        let json_path = self.base_dir.join(format!("{}.json", token.token_id));
        let json = serde_json::to_string_pretty(token)
            .map_err(|e| DownloadError::TokenStoreError(e.to_string()))?;
        let tmp_path = json_path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, json.as_bytes()).await?;
        tokio::fs::rename(&tmp_path, &json_path).await?;  // 原子 rename

        // 2. 更新 SQLite 索引
        let storage_size = tokio::fs::metadata(&json_path).await?.len();
        let conn = self.sqlite.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO resume_token_index
             (token_id, file_path, status, last_updated_at, created_at, storage_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                token.token_id.to_string(),
                token.file_path,
                serde_json::to_string(&token.status)?,
                token.last_updated_at.timestamp(),
                token.created_at.timestamp(),
                storage_size,
            ],
        )?;

        // 3. 触发 LRU 清理
        self.maybe_evict().await?;
        Ok(())
    }

    pub async fn get(&self, file_path: &str) -> Result<Option<ResumeToken>, DownloadError> {
        let conn = self.sqlite.lock().await;
        let token_id: Option<String> = conn
            .query_row(
                "SELECT token_id FROM resume_token_index WHERE file_path = ?1",
                rusqlite::params![file_path],
                |row| row.get(0),
            )
            .optional()?;
        drop(conn);

        if let Some(id) = token_id {
            let json_path = self.base_dir.join(format!("{}.json", id));
            let bytes = tokio::fs::read(&json_path).await?;
            let token: ResumeToken = serde_json::from_slice(&bytes)
                .map_err(|e| DownloadError::TokenStoreError(e.to_string()))?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, file_path: &str) -> Result<(), DownloadError> {
        // 1. 读 token_id
        let conn = self.sqlite.lock().await;
        let token_id: Option<String> = conn
            .query_row(
                "SELECT token_id FROM resume_token_index WHERE file_path = ?1",
                rusqlite::params![file_path],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = token_id {
            conn.execute(
                "DELETE FROM resume_token_index WHERE file_path = ?1",
                rusqlite::params![file_path],
            )?;
            drop(conn);
            // 2. 删 JSON 文件
            let json_path = self.base_dir.join(format!("{}.json", id));
            let _ = tokio::fs::remove_file(json_path).await;  // 静默失败
        }
        Ok(())
    }

    pub async fn update_status(&self, file_path: &str, status: DownloadState) -> Result<(), DownloadError> {
        let conn = self.sqlite.lock().await;
        conn.execute(
            "UPDATE resume_token_index SET status = ?1, last_updated_at = ?2 WHERE file_path = ?3",
            rusqlite::params![
                serde_json::to_string(&status)?,
                Utc::now().timestamp(),
                file_path,
            ],
        )?;
        Ok(())
    }

    /// LRU 清理 (NFR-CDN-113 落地)
    async fn maybe_evict(&self) -> Result<(), DownloadError> {
        // 简化: 详细清理策略见 RGS-BAS-036 §5.3
        let conn = self.sqlite.lock().await;
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(storage_size_bytes), 0) FROM resume_token_index",
            [],
            |row| row.get(0),
        )?;

        if total < (self.storage_limit_bytes as i64) * 8 / 10 {
            return Ok(());  // 低于 80% 不清理
        }

        // 优先清理 completed (last_updated_at < 1h)
        // 其次清理 canceled/failed (last_updated_at < 24h)
        // 最后按 last_updated_at 升序淘汰
        // (具体 SQL 略, 详细编码阶段补全)
        Ok(())
    }

    /// 过期判定 (FR-CDN-063, 默认 7 天)
    pub fn is_expired(&self, token: &ResumeToken) -> bool {
        let now = Utc::now();
        now.signed_duration_since(token.last_updated_at).num_days() > 7
    }
}
```

# 5. RangeClient 实现

## 5.1 HTTP 客户端

```rust
// src/range_client.rs
use reqwest::{Client, Request, Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, RANGE, IF_RANGE, ACCEPT_RANGES, ETAG, CONTENT_RANGE, CONTENT_LENGTH, LAST_MODIFIED};

pub struct RangeClient {
    http: Client,
    user_agent: String,
}

impl RangeClient {
    pub fn new() -> Result<Self, DownloadError> {
        let http = Client::builder()
            .user_agent(concat!("rgs-asset-download/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            http,
            user_agent: format!("rgs-asset-download/{}", env!("CARGO_PKG_VERSION")),
        })
    }

    /// HEAD 探测 (FR-CDN-042)
    pub async fn probe(&self, url: &str) -> Result<ProbeResult, DownloadError> {
        let resp = self.http.head(url).send().await?;
        if resp.status() != StatusCode::OK {
            return Err(DownloadError::RangeRequestFailed(resp.error_for_status().unwrap_err()));
        }

        let headers = resp.headers();
        let content_length: u64 = headers
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| DownloadError::TokenStoreError("missing Content-Length".into()))?;

        let etag = headers
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| DownloadError::TokenStoreError("missing ETag".into()))?
            .to_string();

        let accept_ranges = headers
            .get(ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|s| s == "bytes")
            .unwrap_or(false);

        let last_modified = headers
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        Ok(ProbeResult {
            total_size: content_length,
            etag,
            accept_ranges,
            last_modified,
        })
    }

    /// Range 请求 (FR-CDN-040)
    pub async fn range_request(
        &self,
        url: &str,
        start: u64,
        end: Option<u64>,    // None 表示开区间
        if_range_etag: Option<&str>,
        resume_token_id: Option<Uuid>,
    ) -> Result<RangeResponse, DownloadError> {
        let range_header = match end {
            Some(e) => format!("bytes={}-{}", start, e),
            None => format!("bytes={}-", start),
        };

        let mut req = self.http.get(url)
            .header(RANGE, range_header);

        if let Some(etag) = if_range_etag {
            // FR-CDN-074 强制用 ETag 而非 Last-Modified
            req = req.header(IF_RANGE, etag);
        }

        if let Some(token_id) = resume_token_id {
            // 可观测性追踪, 不影响行为
            req = req.header("X-RGS-Resume-Token", token_id.to_string());
        }

        let resp = req.send().await?;
        let status = resp.status();

        match status {
            StatusCode::PARTIAL_CONTENT => {
                // 206
                let content_range = parse_content_range(resp.headers())?;
                let etag = resp.headers()
                    .get(ETAG)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| DownloadError::TokenStoreError("206 missing ETag".into()))?
                    .to_string();
                Ok(RangeResponse::Partial {
                    content_range,
                    etag,
                    body: resp.bytes_stream(),
                })
            }
            StatusCode::OK => {
                // 200 (ETag 不匹配, FR-CDN-041)
                let new_etag = resp.headers()
                    .get(ETAG)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| DownloadError::TokenStoreError("200 missing ETag".into()))?
                    .to_string();
                Ok(RangeResponse::Full {
                    new_etag,
                    body: resp.bytes_stream(),
                })
            }
            StatusCode::RANGE_NOT_SATISFIABLE => {
                // 416
                let total_size = parse_total_from_416(resp.headers())?;
                Err(DownloadError::RangeNotSatisfiable {
                    total_size,
                    requested_start: start,
                })
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(DownloadError::RateLimited)
            }
            _ => {
                Err(DownloadError::RangeRequestFailed(
                    resp.error_for_status().unwrap_err()
                ))
            }
        }
    }
}

pub struct ProbeResult {
    pub total_size: u64,
    pub etag: String,
    pub accept_ranges: bool,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum RangeResponse {
    Partial {
        content_range: (u64, u64, u64),  // (start, end, total)
        etag: String,
        body: reqwest::Body,
    },
    Full {
        new_etag: String,
        body: reqwest::Body,
    },
}

fn parse_content_range(headers: &HeaderMap) -> Result<(u64, u64, u64), DownloadError> {
    let s = headers.get(CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| DownloadError::TokenStoreError("missing Content-Range".into()))?;
    // 格式: "bytes START-END/TOTAL"
    let s = s.strip_prefix("bytes ")
        .ok_or_else(|| DownloadError::TokenStoreError("invalid Content-Range prefix".into()))?;
    let (range, total) = s.split_once('/')
        .ok_or_else(|| DownloadError::TokenStoreError("invalid Content-Range".into()))?;
    let (start, end) = range.split_once('-')
        .ok_or_else(|| DownloadError::TokenStoreError("invalid Content-Range range".into()))?;
    Ok((start.parse().unwrap(), end.parse().unwrap(), total.parse().unwrap()))
}

fn parse_total_from_416(headers: &HeaderMap) -> Result<u64, DownloadError> {
    let s = headers.get(CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| DownloadError::TokenStoreError("416 missing Content-Range".into()))?;
    let s = s.strip_prefix("bytes */")
        .ok_or_else(|| DownloadError::TokenStoreError("invalid 416 Content-Range".into()))?;
    s.parse().map_err(|_| DownloadError::TokenStoreError("invalid 416 total".into()))
}
```

# 6. ChunkOrchestrator 并发分片实现

## 6.1 数据结构

```rust
// src/chunk_orchestrator.rs
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::sync::Notify;

pub struct ChunkOrchestrator {
    config: ChunkConfig,
    state: Arc<Mutex<ChunkState>>,
    pause_notify: Arc<Notify>,
    cancel_notify: Arc<Notify>,
}

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub chunk_size: u64,            // 默认 8MB
    pub concurrency_desktop: u32,   // 默认 8
    pub concurrency_mobile: u32,    // 默认 4
    pub mobile_detection: MobileDetection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileDetection {
    CompileTime,   // 通过编译期 cfg 决定
    Runtime,       // 通过运行时检测 (如 sysinfo crate)
}

struct ChunkState {
    chunks: Vec<ChunkRange>,
    in_flight: HashMap<u64, ChunkRange>,   // chunk_id -> range
    completed: HashSet<u64>,
    failed: VecDeque<FailedChunk>,
    current_concurrency: u32,
    platform: Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Mobile,
}

#[derive(Debug, Clone)]
pub struct ChunkRange {
    pub chunk_id: u64,
    pub start: u64,
    pub end: u64,   // inclusive
}

#[derive(Debug, Clone)]
pub struct FailedChunk {
    pub chunk: ChunkRange,
    pub error: DownloadError,
    pub retry_count: u32,
}
```

## 6.2 调度主循环

```rust
// src/chunk_orchestrator.rs (续)
impl ChunkOrchestrator {
    pub fn new(total_size: u64, config: ChunkConfig, platform: Platform) -> Self {
        let chunk_size = config.chunk_size;
        let mut chunks = Vec::new();
        let mut start = 0u64;
        let mut id = 0u64;
        while start < total_size {
            let end = (start + chunk_size - 1).min(total_size - 1);
            chunks.push(ChunkRange { chunk_id: id, start, end });
            start = end + 1;
            id += 1;
        }
        let initial_concurrency = match platform {
            Platform::Desktop => config.concurrency_desktop,
            Platform::Mobile => config.concurrency_mobile,
        };
        Self {
            config,
            state: Arc::new(Mutex::new(ChunkState {
                chunks,
                in_flight: HashMap::new(),
                completed: HashSet::new(),
                failed: VecDeque::new(),
                current_concurrency: initial_concurrency,
                platform,
            })),
            pause_notify: Arc::new(Notify::new()),
            cancel_notify: Arc::new(Notify::new()),
        }
    }

    /// 启动下载主循环
    pub async fn run(
        &self,
        range_client: Arc<RangeClient>,
        token: ResumeToken,
        on_chunk_complete: impl Fn(ChunkRange) + Send + Sync + 'static,
    ) -> Result<(), DownloadError> {
        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        let state = self.state.clone();

        loop {
            // 检查取消/暂停
            tokio::select! {
                _ = self.cancel_notify.notified() => {
                    // 取消: 取消所有 in_flight, 返回
                    for (_, chunk) in state.lock().await.in_flight.drain() {
                        // 取消请求 (实际由 RangeClient 内部处理)
                    }
                    return Err(DownloadError::RetryExhausted { attempts: 0 });  // 标识为取消
                }
                _ = self.pause_notify.notified() => {
                    // 暂停: 取消所有 in_flight, 返回
                    for (_, chunk) in state.lock().await.in_flight.drain() {
                        // 取消请求
                    }
                    return Ok(());  // 标识为暂停
                }
                else => {}
            }

            // 选择下一个未完成的 chunk
            let next = {
                let mut s = state.lock().await;
                if s.completed.len() == s.chunks.len() {
                    break;  // 全部完成
                }
                if s.in_flight.len() >= s.current_concurrency as usize {
                    continue;  // 已达并发上限
                }
                s.chunks.iter()
                    .find(|c| !s.completed.contains(&c.chunk_id)
                            && !s.in_flight.contains_key(&c.chunk_id)
                            && !s.failed.iter().any(|f| f.chunk.chunk_id == c.chunk_id))
                    .cloned()
            };

            if let Some(chunk) = next {
                let chunk_id = chunk.chunk_id;
                {
                    let mut s = state.lock().await;
                    s.in_flight.insert(chunk_id, chunk.clone());
                }
                let state_clone = state.clone();
                let range_client_clone = range_client.clone();
                let on_complete = Arc::new(on_chunk_complete);
                let handle = tokio::spawn(async move {
                    let result = download_chunk(
                        range_client_clone,
                        chunk,
                        &token,
                    ).await;
                    let mut s = state_clone.lock().await;
                    s.in_flight.remove(&chunk_id);
                    match result {
                        Ok(_) => {
                            s.completed.insert(chunk_id);
                            (on_complete)(/* chunk info */);
                        }
                        Err(e) => {
                            let retry_count = s.failed.iter()
                                .find(|f| f.chunk.chunk_id == chunk_id)
                                .map(|f| f.retry_count + 1)
                                .unwrap_or(1);
                            s.failed.push_back(FailedChunk {
                                chunk: ChunkRange { chunk_id, start: 0, end: 0 },  // 简略
                                error: e,
                                retry_count,
                            });
                        }
                    }
                });
                handles.push(handle);
            } else {
                // 没有可下载的 chunk, 等待 in_flight 完成
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        // 等待所有 handle 完成
        for h in handles {
            let _ = h.await;
        }
        Ok(())
    }

    pub async fn pause(&self) {
        self.pause_notify.notify_waiters();
    }

    pub async fn cancel(&self) {
        self.cancel_notify.notify_waiters();
    }

    /// 恢复时调用: 重试失败 chunk
    pub async fn retry_failed(&self) -> Result<(), DownloadError> {
        let mut s = self.state.lock().await;
        s.failed.clear();  // 简单策略: 清空失败队列, 重新尝试
        Ok(())
    }
}

async fn download_chunk(
    range_client: Arc<RangeClient>,
    chunk: ChunkRange,
    token: &ResumeToken,
) -> Result<(), DownloadError> {
    let mut backoff = tokio::time::Duration::from_millis(100);
    for attempt in 0..3 {
        match range_client.range_request(
            &token.source_url,
            chunk.start,
            Some(chunk.end),
            Some(&token.etag),
            Some(token.token_id),
        ).await {
            Ok(RangeResponse::Partial { content_range, etag: _, body }) => {
                // 写入临时文件 (FR-CDN-084 seek 写入)
                write_chunk_to_temp(&token.temp_file_path, chunk.start, body).await?;
                return Ok(());
            }
            Ok(RangeResponse::Full { .. }) => {
                // 200 OK: ETag 变更, 全量重传由上层处理
                return Err(DownloadError::ETagChanged {
                    old_etag: token.etag.clone(),
                    new_etag: "from_response".into(),  // 实际从 response 提取
                });
            }
            Err(DownloadError::RateLimited) => {
                // 限流: 指数退避
                tokio::time::sleep(backoff).await;
                backoff *= 2;
            }
            Err(e) if attempt < 2 => {
                tokio::time::sleep(backoff).await;
                backoff *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    Err(DownloadError::RetryExhausted { attempts: 3 })
}
```

# 7. IntegrityGate 整文件校验实现

## 7.1 接口

```rust
// src/integrity_gate.rs
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;
use tokio::io::AsyncReadExt;

pub struct IntegrityReport {
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

pub struct IntegrityGate;

impl IntegrityGate {
    /// 整文件 hash 计算 (FR-CDN-012, 不分块单独校验)
    pub async fn verify(
        temp_file_path: &Path,
        expected_hash: &str,
        algorithm: ChecksumAlgorithm,
    ) -> Result<IntegrityReport, DownloadError> {
        let actual = Self::compute_hash(temp_file_path, algorithm).await?;
        let passed = actual.eq_ignore_ascii_case(expected_hash);
        Ok(IntegrityReport {
            expected: expected_hash.to_string(),
            actual,
            passed,
        })
    }

    pub async fn compute_hash(
        file_path: &Path,
        algorithm: ChecksumAlgorithm,
    ) -> Result<String, DownloadError> {
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut hasher = match algorithm {
            ChecksumAlgorithm::Sha256 => Box::new(Sha256::new()) as Box<dyn AsyncHasher>,
            ChecksumAlgorithm::Blake3 => Box::new(Blake3Hasher::new()) as Box<dyn AsyncHasher>,
        };

        let mut buf = vec![0u8; 1024 * 1024];  // 1MB buffer
        loop {
            let n = file.read(&mut buf).await?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        Ok(hasher.finalize_hex())
    }
}
```

> **简化提示**：`AsyncHasher` trait 抽象需要实际实现（同 `digest::DynDigest` 或自定义）。具体编码阶段补全。

# 8. 平台特定实现要点

## 8.1 Unix sparse file

```rust
// src/platform/unix.rs
use std::os::unix::fs::OpenOptionsExt;
use tokio::fs::OpenOptions;

pub async fn preallocate(path: &Path, total_size: u64) -> Result<(), DownloadError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .custom_flags(libc::O_CREAT)
        .open(path)
        .await?;
    file.set_len(total_size).await?;
    // Unix 上 set_len 自动创建 sparse file
    Ok(())
}
```

## 8.2 Windows

```rust
// src/platform/windows.rs
use std::os::windows::fs::OpenOptionsExt;
use tokio::fs::OpenOptions;
use windows::Win32::Storage::FileSystem::{
    SetFileValidData, SetFileInformationByHandle,
    FileAllocationInfo, FILE_ALLOCATION_INFO,
};

pub async fn preallocate(path: &Path, total_size: u64) -> Result<(), DownloadError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .await?;

    // Windows: 先设置 valid data length (类似 sparse)
    // 注: 需要 GENERIC_WRITE 权限
    unsafe {
        let handle = file.as_raw_handle() as *mut std::ffi::c_void;
        SetFileValidData(handle, total_size as i64)?;
    }

    // 设置文件分配信息
    let alloc_info = FILE_ALLOCATION_INFO {
        AllocationSize: total_size as i64,
    };
    unsafe {
        let handle = file.as_raw_handle() as isize;
        SetFileInformationByHandle(
            handle,
            FileAllocationInfo,
            &alloc_info as *const _ as *const _,
            std::mem::size_of::<FILE_ALLOCATION_INFO>() as u32,
        )?;
    }
    Ok(())
}
```

> **安全注意**：`SetFileValidData` 需要 `SeManageVolumePrivilege`,详细编码阶段需评估权限获取策略。

## 8.3 移动平台

```rust
// src/platform/android.rs / src/platform/ios.rs
// 移动平台通常无 sparse file 概念, 简单方案: 顺序写入, 由 ChunkOrchestrator 保证
// seek 写入通过 tokio::fs::File::seek + write 实现
```

# 9. 与既有 SDK 模块的集成时序

## 9.1 完整下载流程

```rust
// src/downloader.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Downloader {
    state_machine: Arc<DownloadStateMachine>,
    range_client: Arc<RangeClient>,
    chunk_orchestrator: Arc<ChunkOrchestrator>,
    token_store: Arc<ResumeTokenStore>,
    integrity_gate: Arc<IntegrityGate>,
    // 来自 rgs-asset-update 的依赖注入
    manifest_client: Arc<rgs_asset_update::ManifestClient>,
    signature_verifier: Arc<rgs_asset_update::SignatureVerifier>,
    gray_rollout_checker: Arc<rgs_asset_update::GrayRolloutChecker>,
}

impl Downloader {
    /// 公开入口: 触发下载一个文件
    pub async fn download(
        &self,
        file_entry: &AssetFileEntry,
        source_url: &str,
    ) -> Result<(), DownloadError> {
        // 状态转移: NotStarted → Probing
        self.state_machine.transition(DownloadState::Probing).await?;

        // 1. HEAD 探测
        let probe = self.range_client.probe(source_url).await?;

        // 2. 检查既有断点
        let existing_token = self.token_store.get(&file_entry.file_path).await?;

        let token = match existing_token {
            Some(t) if !self.token_store.is_expired(&t) => {
                // 走 Resuming 流程
                self.state_machine.transition(DownloadState::Resuming).await?;
                self.resume_flow(t, &probe, file_entry).await?
            }
            _ => {
                // 全新下载
                self.new_download_flow(file_entry, source_url, &probe).await?
            }
        };

        // 3. 整文件校验
        let report = IntegrityGate::verify(
            &token.temp_file_path,
            &file_entry.checksum,
            token.checksum_algorithm,
        ).await?;
        if !report.passed {
            return Err(DownloadError::IntegrityCheckFailed {
                expected: report.expected,
                actual: report.actual,
            });
        }

        // 4. 应用到正式位置 + 清理断点
        self.finalize(&token, file_entry).await?;
        self.state_machine.transition(DownloadState::Completed).await?;
        Ok(())
    }

    async fn resume_flow(
        &self,
        existing: ResumeToken,
        probe: &ProbeResult,
        file_entry: &AssetFileEntry,
    ) -> Result<ResumeToken, DownloadError> {
        // 校验 1: ETag 一致
        if existing.etag != probe.etag {
            // ETag 变更 → 触发 NotStarted 全量重传
            self.state_machine.transition(DownloadState::NotStarted).await?;
            return Err(DownloadError::ETagChanged {
                old_etag: existing.etag.clone(),
                new_etag: probe.etag.clone(),
            });
        }

        // 校验 2: 重新拉取 Manifest 并校验签名 (FR-CDN-071)
        let manifest = self.manifest_client.fetch_latest().await?;
        self.signature_verifier.verify(&manifest)?;

        // 校验 3: 灰度状态 (FR-CDN-072)
        if !self.gray_rollout_checker.is_accessible(&manifest, &file_entry).await? {
            return Err(DownloadError::GrayRolledBack);
        }

        // 校验 4: 过期
        if self.token_store.is_expired(&existing) {
            return Err(DownloadError::ResumeTokenExpired {
                last_updated_at: existing.last_updated_at,
            });
        }

        // 校验通过 → 继续
        self.state_machine.transition(DownloadState::Downloading).await?;
        self.chunk_orchestrator.run(
            self.range_client.clone(),
            existing.clone(),
            |chunk| { /* 更新 chunk_manifest, 原子写断点 */ },
        ).await?;
        Ok(existing)
    }
}
```

# 10. 可观测性埋点接入

```rust
// src/metrics.rs
use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};

pub fn register_metrics() {
    describe_counter!("rgs_asset_download_state_transition_total", "状态机各转移次数");
    describe_gauge!("rgs_asset_download_active_count", "当前 Downloading 状态的资源数");
    describe_counter!("rgs_asset_download_bytes_received_total", "Range 响应实际接收字节数");
    describe_counter!("rgs_asset_download_resume_count", "断点恢复成功次数");
    describe_counter!("rgs_asset_download_resume_failure_total", "断点恢复失败次数");
    describe_counter!("rgs_asset_download_chunk_retry_total", "chunk 失败重试次数");
    describe_histogram!("rgs_asset_download_duration_seconds", "单文件总耗时");
    describe_histogram!("rgs_asset_download_throughput_bytes_per_sec", "单 chunk 实际吞吐");
    describe_counter!("rgs_asset_download_integrity_failure_total", "整文件校验失败次数");
    describe_counter!("rgs_asset_download_etag_mismatch_total", "ETag 变更触发全量重传次数");
    describe_gauge!("rgs_asset_download_resume_token_store_bytes", "断点记录本地存储占用");
}

// 在关键路径调用
fn record_state_transition(from: DownloadState, to: DownloadState) {
    counter!("rgs_asset_download_state_transition_total",
        "from" => format!("{:?}", from),
        "to" => format!("{:?}", to)
    ).increment(1);
}
```

> **指标命名空间**：遵循既有 RGS-BAS-004 §3 命名规范（`rgs_<module>_<metric>`），指标接入既有 OTel Collector 导出。

# 11. 故障、降级与异常处理

## 11.1 异常分类与处理

| 类别 | 触发 | 处理 |
|---|---|---|
| Range 瞬时失败 | 网络抖动 / TCP RST / 5xx | 单 chunk 指数退避重试 3 次 |
| Range 416 | 客户端 Range 越界 | 状态机 `Resuming → NotStarted`，触发全量重传 |
| Range 200 (ETag 变) | 源文件 ETag 变更 | 状态机 `Resuming → NotStarted`，触发全量重传 |
| 整文件校验失败 | IntegrityGate 不通过 | 状态机 `Failed`，玩家重试 |
| Manifest 签名失败 | 恢复时拉 Manifest 签名校验失败 | 状态机 `Failed`，**不**使用既有断点 |
| 灰度回滚 | 玩家被切回旧版本 | 状态机 `Resuming → NotStarted`，从旧 URL 重传 |
| 磁盘满 | 预分配 / 写入 ENOSPC | 状态机 `Failed`，`last_error: disk_full`，**不**自动重试 |
| 断点过期 | `last_updated_at` > 7 天 | 状态机 `Resuming → NotStarted`，全量重传 |
| 服务端不支持 Range | HEAD 返回 `Accept-Ranges: none` | 客户端**不**发 Range，回退单流全量 GET |
| 限流 | HTTP 429 | 指数退避（**不**绕过限流配额）|

## 11.2 降级路径

| 故障 | 降级策略 | 实现 |
|---|---|---|
| Range 请求持续失败 | 自动回退为全量 GET（不分片）| `ChunkOrchestrator` 切到单流模式 |
| 并发分片异常 | 降级为单流串行下载 | 同上 |
| ChunkOrchestrator 自身故障 | 状态机 `Failed` | 玩家重试触发重新调度 |
| IntegrityGate 超时 | 后台异步计算 | 详见 RGS-BAS-036 §10.2 |

# 12. 验收证据与开放项

## 12.1 验收证据

- [ ] `rgs-asset-download` crate 在 workspace 编译通过
- [ ] `cargo test -p rgs-asset-download` 全部单元测试通过（state transitions / token store / range client / chunk orchestrator / integrity gate）
- [ ] `cargo test -p rgs-asset-download --test integration` 全部集成测试通过（resumable / pause_resume / etag_invalidation / gray_rollback）
- [ ] 真实 `DistributionBackend`（MinIO 自托管）Range 行为端到端测试通过
- [ ] 移动平台（Android / iOS）pre-allocate 与 sparse file 实测
- [ ] OTel Collector 实测所有 10 项指标正常导出

## 12.2 开放项

| 编号 | 内容 | 处理 |
|---|---|---|
| TBD-DTL-041-01 | 移动平台 `Platform` 运行时检测（vs 编译期 cfg）| 详细编码阶段补全 |
| TBD-DTL-041-02 | `AsyncHasher` trait 抽象的具体实现（`digest::DynDigest`）| 详细编码阶段补全 |
| TBD-DTL-041-03 | Windows `SetFileValidData` 权限获取策略 | 详细编码阶段评估 |
| TBD-DTL-041-04 | SQLite WAL 模式 + 并发读优化 | 详细编码阶段补全 |
| TBD-DTL-041-05 | 断点记录轻量加密（RSK-CDN-204 缓解）| 详细编码阶段评估算法选型 |
| RSK-DTL-041-01 | 并发分片在大文件（≥4GB）下的内存占用 | 验证阶段测量,必要时改为流式处理 |
| RSK-DTL-041-02 | 移动平台断点记录在系统资源回收下的恢复率 | 实测阶段验证 |

# 13. 追溯性

| 需求 ID | 本设计书章节 |
|---|---|
| FR-CDN-040~045 HTTP Range 协议 | §5, §11 |
| FR-CDN-050~053 状态机 | §3 |
| FR-CDN-060~064 断点记录 | §4 |
| FR-CDN-070~074 与既有模块协同 | §9 |
| FR-CDN-080~084 并发分片 | §6, §8 |
| NFR-CDN-110~114 | §1, §4, §10, §11 |
| AC-CDN-110~118 | §12.1 |

---

> 本文档与 RGS-BAS-036（断点续传 基本设计书）配套使用，详细到 Rust crate 级别可被直接编码。配套的 RGS-SPEC 阶段产出（如 `RGS-SPEC-CROSS-XXX` 通用规范）由 13-实现规格 流程产出，**不**在本文档重复。
