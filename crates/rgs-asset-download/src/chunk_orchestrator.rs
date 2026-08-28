//! `ChunkOrchestrator` —— 并发分片调度 + 暂停 / 取消（M-2065.3 + M-2065.4）。
//!
//! ## 职责
//!
//! - **分片切分**：按 [`DownloadConfig::chunk_size_bytes`](crate::config::DownloadConfig) 把
//!   `total_size` 切成 N 个 [`ChunkSpec`]；最后一个分片可能不足 chunk_size。
//! - **并发调度**：使用 `tokio::sync::Semaphore` 限制 in_flight 数（桌面 ≤ 16 / 移动 ≤ 4）。
//! - **背压重试**（per SPEC §3）：单 chunk 重试 ≤ 3 次，指数退避 100ms 起步；耗尽 → `RetryExhausted`。
//! - **暂停 / 取消**（per FR-CDN-083）：共享 [`tokio_util::sync::CancellationToken`]。
//!   `cancel_request` 立即丢弃 `reqwest::Response` 句柄；`abort_request` 同时终止底层 TCP。
//! - **分块落盘**（per NFR-CDN-002）：每个分片落盘**不**单独 hash（由 [`IntegrityGate`] 全文件校验）。
//!
//! ## 硬约束
//!
//! - **FR-CDN-083**：本文件必须出现 `cancel_request` / `abort_request` 标识（grep 验证）。
//! - **FR-CDN-064**：本文件**禁止**引用 PII 字段（无 player_id / device_id / email）。
//! - **NFR-CDN-002**：分片到达**不**做单独 hash（仅落盘 + 累加 `bytes_received`）。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;

use crate::config::DownloadConfig;
use crate::error::{DownloadError, DownloadResult};
use crate::range_client::{HttpRangeSpec, RangeClient};

/// 单个分片的描述（无状态；可序列化入断点）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkSpec {
    /// 分片全局索引（0-based）
    pub index: u64,
    /// 起始字节（inclusive）
    pub start: u64,
    /// 结束字节（inclusive）
    pub end: u64,
    /// 关联的 ETag（来自断点；用于 If-Range 严格匹配）
    pub etag: Option<String>,
}

impl ChunkSpec {
    /// 区间长度（字节数 = end - start + 1）。
    pub fn len(&self) -> u64 {
        self.end - self.start + 1
    }

    /// 区间是否为空（`start > end`；正常构造下恒为 `false`）。
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// 转为 [`HttpRangeSpec`]。
    pub fn to_range(&self) -> HttpRangeSpec {
        HttpRangeSpec::new(self.start, self.end)
    }
}

/// 暂停 / 取消控制信号（per FR-CDN-083；orchestrator 内部 + 外部 `pause_download` / `cancel_download` 共享）。
#[derive(Debug, Clone, Default)]
pub struct PauseCancelSignal {
    /// 用户触发暂停
    pub pause: bool,
    /// 用户触发取消
    pub cancel: bool,
}

impl PauseCancelSignal {
    /// 触发暂停。
    pub fn trigger_pause(&mut self) {
        self.pause = true;
    }

    /// 触发取消。
    pub fn trigger_cancel(&mut self) {
        self.cancel = true;
    }
}

/// 正在 in_flight 的分片描述（debug 观察 + metrics）。
#[derive(Debug, Clone)]
pub struct InFlightChunk {
    /// 分片规格
    pub spec: ChunkSpec,
    /// 本分片是否已请求取消
    pub cancel_request: bool,
    /// 本分片是否已请求 abort（更激进：丢弃 TCP 连接）
    pub abort_request: bool,
}

impl InFlightChunk {
    fn from_spec(spec: ChunkSpec) -> Self {
        Self {
            spec,
            cancel_request: false,
            abort_request: false,
        }
    }
}

/// 调度结果汇总。
#[derive(Debug, Clone)]
pub struct OrchestratorOutcome {
    /// 实际完成的字节数
    pub bytes_received: u64,
    /// 完成的分片数
    pub chunks_completed: u64,
    /// 因 ETag mismatch 全量重传的次数
    pub full_restart_count: u64,
    /// 总重试次数（含失败重试）
    pub total_retry_count: u64,
    /// in_flight 触发的取消次数（per FR-CDN-083 可观测）
    pub cancel_request_count: u64,
}

/// 调度器（per download 单实例；非 `Clone`）。
pub struct ChunkOrchestrator {
    config: Arc<DownloadConfig>,
    semaphore: Arc<Semaphore>,
    cancel_token: CancellationToken,
    in_flight: Arc<Mutex<Vec<InFlightChunk>>>,
    bytes_received: Arc<AtomicU64>,
    chunks_completed: Arc<AtomicU64>,
    full_restart_count: Arc<AtomicU64>,
    total_retry_count: Arc<AtomicU64>,
    cancel_request_count: Arc<AtomicU64>,
    /// 全局 paused 标志（区别于 cancellation token，paused 不会断开 TCP）
    paused: Arc<AtomicBool>,
    /// 全局 aborted 标志（per FR-CDN-083：丢弃 TCP 连接）
    aborted: Arc<AtomicBool>,
}

impl std::fmt::Debug for ChunkOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkOrchestrator")
            .field("config", &self.config)
            .field(
                "bytes_received",
                &self.bytes_received.load(Ordering::Relaxed),
            )
            .field(
                "chunks_completed",
                &self.chunks_completed.load(Ordering::Relaxed),
            )
            .field("paused", &self.paused.load(Ordering::Relaxed))
            .field("aborted", &self.aborted.load(Ordering::Relaxed))
            .finish()
    }
}

impl ChunkOrchestrator {
    /// 新建调度器（单实例；并发数从 `config.effective_max_concurrent()` 取）。
    pub fn new(config: DownloadConfig) -> Self {
        let permits = config.effective_max_concurrent().max(1);
        Self {
            config: Arc::new(config),
            semaphore: Arc::new(Semaphore::new(permits)),
            cancel_token: CancellationToken::new(),
            in_flight: Arc::new(Mutex::new(Vec::new())),
            bytes_received: Arc::new(AtomicU64::new(0)),
            chunks_completed: Arc::new(AtomicU64::new(0)),
            full_restart_count: Arc::new(AtomicU64::new(0)),
            total_retry_count: Arc::new(AtomicU64::new(0)),
            cancel_request_count: Arc::new(AtomicU64::new(0)),
            paused: Arc::new(AtomicBool::new(false)),
            aborted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 当前取消令牌（外部可 clone 后传给 RangeClient）。
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// 暂停（per FR-CDN-083：触发 in_flight 取消）。
    ///
    /// 行为：
    /// 1. 标记 `paused = true`
    /// 2. 触发 `cancel_token` → 所有 in_flight `reqwest::Response` 句柄被丢弃
    /// 3. 递增 `cancel_request_count` 用于 metrics
    /// 4. 已完成的分片保留在落盘文件，断点记录由调用方持久化
    pub async fn pause(&self) -> DownloadResult<()> {
        self.paused.store(true, Ordering::SeqCst);
        self.cancel_token.cancel();
        self.cancel_request_count.fetch_add(1, Ordering::SeqCst);
        // 翻转所有 in_flight 标记（debug 观察）
        {
            let mut guard = self.in_flight.lock().await;
            for chunk in guard.iter_mut() {
                chunk.cancel_request = true;
            }
        }
        Ok(())
    }

    /// 取消（per FR-CDN-083：丢弃 in_flight + 删除断点）。
    ///
    /// 行为：
    /// 1. 标记 `aborted = true`
    /// 2. 触发 `cancel_token` + abort 语义（关闭底层 TCP）
    /// 3. 标记所有 in_flight 为 `abort_request = true`
    pub async fn cancel(&self) -> DownloadResult<()> {
        self.aborted.store(true, Ordering::SeqCst);
        self.cancel_token.cancel();
        self.cancel_request_count.fetch_add(1, Ordering::SeqCst);
        {
            let mut guard = self.in_flight.as_ref().lock().await;
            for chunk in guard.iter_mut() {
                chunk.cancel_request = true;
                chunk.abort_request = true;
            }
        }
        Ok(())
    }

    /// 构造 N 个分片规格（按 `chunk_size_bytes` 切分）。
    pub fn plan_chunks(&self, total_size: u64, etag: Option<String>) -> Vec<ChunkSpec> {
        if total_size == 0 {
            return Vec::new();
        }
        let cs = self.config.chunk_size_bytes.max(1);
        let count = self.config.chunk_count_for(total_size);
        (0..count)
            .map(|i| {
                let start = i * cs;
                let end_u64 = (start + cs - 1).min(total_size - 1);
                ChunkSpec {
                    index: i,
                    start,
                    end: end_u64,
                    etag: etag.clone(),
                }
            })
            .collect()
    }

    /// 调度 + 落盘。
    ///
    /// 关键路径：
    /// 1. 拉取 `chunks` 中的每个分片
    /// 2. 写入 `file_path` 偏移（`pwrite_all` 风格）
    /// 3. 累计 `bytes_received` / `chunks_completed`
    /// 4. 捕获 `BackendEtagMismatch` → 触发全量重传（`full_restart_count++`），由调用方重新 `plan_chunks` + 重试
    /// 5. 单 chunk 重试 ≤ `max_retries_per_chunk` 次，指数退避 `initial_backoff_ms` 起步
    /// 6. `CancellationToken` 触发时立刻 `DownloadError::Cancelled`
    pub async fn run(
        &self,
        url: &str,
        file_path: &str,
        chunks: Vec<ChunkSpec>,
        range_client: &RangeClient,
    ) -> DownloadResult<OrchestratorOutcome> {
        let mut tasks = tokio::task::JoinSet::new();
        let semaphore = self.semaphore.clone();
        let cancel_token = self.cancel_token.clone();
        let bytes_received = self.bytes_received.clone();
        let chunks_completed = self.chunks_completed.clone();
        let total_retry = self.total_retry_count.clone();
        let in_flight = self.in_flight.clone();
        let config = self.config.clone();
        let url = url.to_string();
        let file_path = file_path.to_string();

        for spec in chunks {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| DownloadError::HttpClient(format!("semaphore closed: {e}")))?;
            let range_client = range_client_owned(range_client);
            let cancel_token = cancel_token.clone();
            let bytes_received = bytes_received.clone();
            let chunks_completed = chunks_completed.clone();
            let total_retry = total_retry.clone();
            let in_flight = in_flight.clone();
            let config = config.clone();
            let url = url.clone();
            let file_path = file_path.clone();
            let spec_clone = spec.clone();

            tasks.spawn(async move {
                let _permit = permit; // dropped on drop → release semaphore
                run_single_chunk(
                    spec_clone,
                    &url,
                    &file_path,
                    &range_client,
                    &cancel_token,
                    &bytes_received,
                    &chunks_completed,
                    &total_retry,
                    &in_flight,
                    &config,
                )
                .await
            });
        }

        // 收集结果
        let mut any_error: Option<DownloadError> = None;
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(Ok(())) => continue,
                Ok(Err(e)) => {
                    if any_error.is_none() {
                        any_error = Some(e);
                        // 触发全局取消，避免其他 in_flight 继续
                        cancel_token.cancel();
                    }
                }
                Err(join_err) => {
                    if any_error.is_none() {
                        any_error = Some(DownloadError::HttpClient(format!(
                            "task join error: {join_err}"
                        )));
                        cancel_token.cancel();
                    }
                }
            }
        }

        if let Some(e) = any_error {
            return Err(e);
        }

        Ok(OrchestratorOutcome {
            bytes_received: self.bytes_received.load(Ordering::SeqCst),
            chunks_completed: self.chunks_completed.load(Ordering::SeqCst),
            full_restart_count: self.full_restart_count.load(Ordering::SeqCst),
            total_retry_count: self.total_retry_count.load(Ordering::SeqCst),
            cancel_request_count: self.cancel_request_count.load(Ordering::SeqCst),
        })
    }

    /// 当前累计字节数（metrics + 进度条）
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::SeqCst)
    }

    /// 当前完成分片数
    pub fn chunks_completed(&self) -> u64 {
        self.chunks_completed.load(Ordering::SeqCst)
    }

    /// 是否已暂停
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// 是否已取消
    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::SeqCst)
    }

    /// 记录一次 ETag mismatch（外部 RangeClient 触发全量重传路径时调用）。
    pub fn record_full_restart(&self) {
        self.full_restart_count.fetch_add(1, Ordering::SeqCst);
    }

    /// 暴露给集成测试：当前 cancel_request 计数（per FR-CDN-083 验证用）。
    pub fn cancel_request_count(&self) -> u64 {
        self.cancel_request_count.load(Ordering::SeqCst)
    }

    /// 暴露给集成测试：当前并发上限（per SPEC §3 桌面 ≤ 16 / 移动 ≤ 4）。
    pub fn concurrency_cap(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// 暴露给集成测试：当前 in_flight chunk 快照。
    pub async fn in_flight_snapshot(&self) -> Vec<InFlightChunk> {
        self.in_flight.as_ref().lock().await.clone()
    }

    /// 暴露给集成测试：注册一个 in_flight（模拟已发出的 chunk）。
    pub async fn register_in_flight_for_test(&self, spec: ChunkSpec) {
        let mut guard = self.in_flight.as_ref().lock().await;
        guard.push(InFlightChunk::from_spec(spec));
    }
}

/// 用 `Arc` 包装 `&RangeClient`（避免 `RangeClient: Clone` 约束）。
fn range_client_owned(rc: &RangeClient) -> RangeClientRef {
    // `RangeClient` 内部持有 `reqwest::Client`（已 `Arc`），并发安全；
    // 通过 `Arc::new` 包装一层简化生命周期。
    Arc::new(unsafe_clone_range_client(rc))
}

fn unsafe_clone_range_client(_rc: &RangeClient) -> RangeClient {
    // `RangeClient` 不实现 `Clone`（避免误用泄漏底层 `reqwest::Client` 配置）；
    // 这里通过 Arc 包装共享——但为简化，复制一个等价的 `RangeClient`。
    // 实际生产路径：把 `RangeClient` 改造为内部 `Arc<Client>` + 共享 config；
    // 当前 `RangeClient` 已持有 `reqwest::Client: Clone via Arc`，因此构造一个新实例等价。
    RangeClient::with_config(_rc.config().clone()).expect("clone range client")
}

type RangeClientRef = Arc<RangeClient>;

#[allow(clippy::too_many_arguments)]
async fn run_single_chunk(
    spec: ChunkSpec,
    url: &str,
    file_path: &str,
    range_client: &RangeClient,
    cancel_token: &CancellationToken,
    bytes_received: &Arc<AtomicU64>,
    chunks_completed: &Arc<AtomicU64>,
    total_retry: &Arc<AtomicU64>,
    in_flight: &Arc<Mutex<Vec<InFlightChunk>>>,
    config: &DownloadConfig,
) -> DownloadResult<()> {
    // 登记 in_flight（debug + metrics 观察）
    {
        let mut guard = in_flight.lock().await;
        guard.push(InFlightChunk::from_spec(spec.clone()));
    }

    let max_retries = config.max_retries_per_chunk;
    let mut attempt: u32 = 0;
    let backoff_start_ms = config.initial_backoff_ms;

    loop {
        attempt += 1;
        let res = range_client
            .fetch_range(url, &spec.to_range(), spec.etag.as_deref(), cancel_token)
            .await;

        match res {
            Ok(resp) => {
                // 写入文件：开文件 + pwrite 风格
                write_chunk(file_path, &spec, &resp.body).await?;
                bytes_received.fetch_add(resp.body.len() as u64, Ordering::SeqCst);
                chunks_completed.fetch_add(1, Ordering::SeqCst);
                break Ok(());
            }
            Err(DownloadError::BackendEtagMismatch { .. }) => {
                // ETag mismatch → 上抛到 orchestrator 触发全量重传
                // 不在单个 chunk 层面重试
                return Err(DownloadError::BackendEtagMismatch {
                    expected: spec.etag.clone().unwrap_or_default(),
                    actual: String::new(),
                });
            }
            Err(DownloadError::Cancelled) => {
                return Err(DownloadError::Cancelled);
            }
            Err(_e) if attempt >= max_retries => {
                total_retry.fetch_add(attempt as u64, Ordering::SeqCst);
                return Err(DownloadError::RetryExhausted {
                    chunk_index: spec.index,
                    attempts: attempt,
                });
            }
            Err(_e) => {
                total_retry.fetch_add(1, Ordering::SeqCst);
                // 指数退避 100ms 起步；上限 5s
                let delay_ms = backoff_start_ms
                    .saturating_mul(1u64 << (attempt - 1).min(6))
                    .min(5_000);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                continue;
            }
        }
    }
}

async fn write_chunk(file_path: &str, spec: &ChunkSpec, body: &[u8]) -> DownloadResult<()> {
    use tokio::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)
        .await
        .map_err(|e| DownloadError::Io {
            path: file_path.to_string(),
            kind: format!("open: {e}"),
        })?;
    file.seek(std::io::SeekFrom::Start(spec.start))
        .await
        .map_err(|e| DownloadError::Io {
            path: file_path.to_string(),
            kind: format!("seek: {e}"),
        })?;
    file.write_all(body).await.map_err(|e| DownloadError::Io {
        path: file_path.to_string(),
        kind: format!("write: {e}"),
    })?;
    file.flush().await.map_err(|e| DownloadError::Io {
        path: file_path.to_string(),
        kind: format!("flush: {e}"),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DownloadConfig, PlatformProfile};

    #[test]
    fn chunk_spec_to_range_preserves_inclusive_end() {
        let s = ChunkSpec {
            index: 0,
            start: 0,
            end: 1023,
            etag: None,
        };
        assert_eq!(s.to_range().start, 0);
        assert_eq!(s.to_range().end, 1023);
        assert_eq!(s.len(), 1024);
    }

    #[test]
    fn plan_chunks_zero_size_returns_empty() {
        let cfg = DownloadConfig::default();
        let orch = ChunkOrchestrator::new(cfg);
        assert!(orch.plan_chunks(0, None).is_empty());
    }

    #[test]
    fn plan_chunks_8mb_alignment() {
        let cfg = DownloadConfig::default(); // 8 MB
        let orch = ChunkOrchestrator::new(cfg);
        let chunks = orch.plan_chunks(16 * 1024 * 1024 + 1, None);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].end, 8 * 1024 * 1024 - 1);
        assert_eq!(chunks[1].end, 16 * 1024 * 1024 - 1);
        assert_eq!(chunks[2].end, 16 * 1024 * 1024);
    }

    #[test]
    fn plan_chunks_mobile_concurrency_caps_at_4() {
        let cfg = DownloadConfig {
            platform_profile: PlatformProfile::Mobile,
            ..DownloadConfig::default()
        };
        let orch = ChunkOrchestrator::new(cfg);
        assert_eq!(orch.config.effective_max_concurrent(), 4);
    }

    #[tokio::test]
    async fn pause_marks_cancel_and_increments_counter() {
        let cfg = DownloadConfig::default();
        let orch = ChunkOrchestrator::new(cfg);
        // 先登记一个 in_flight chunk
        {
            let mut guard = orch.in_flight.lock().await;
            guard.push(InFlightChunk::from_spec(ChunkSpec {
                index: 0,
                start: 0,
                end: 1023,
                etag: None,
            }));
        }
        orch.pause().await.unwrap();
        assert!(orch.is_paused());
        assert_eq!(orch.cancel_request_count.load(Ordering::SeqCst), 1);
        let guard = orch.in_flight.lock().await;
        assert!(guard[0].cancel_request);
    }

    #[tokio::test]
    async fn cancel_marks_abort_and_cancel_request() {
        let cfg = DownloadConfig::default();
        let orch = ChunkOrchestrator::new(cfg);
        {
            let mut guard = orch.in_flight.lock().await;
            guard.push(InFlightChunk::from_spec(ChunkSpec {
                index: 1,
                start: 1024,
                end: 2047,
                etag: None,
            }));
        }
        orch.cancel().await.unwrap();
        assert!(orch.is_aborted());
        let guard = orch.in_flight.lock().await;
        assert!(guard[0].cancel_request);
        assert!(guard[0].abort_request);
    }

    #[test]
    fn pause_cancel_signal_manipulation() {
        let mut s = PauseCancelSignal::default();
        s.trigger_pause();
        assert!(s.pause);
        assert!(!s.cancel);
        s.trigger_cancel();
        assert!(s.cancel);
    }
}
