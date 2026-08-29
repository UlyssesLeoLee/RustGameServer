//! match-service → replay-service gRPC client (per W36 2026-08-30 SaveReplay saga)
//!
//! ## 职责
//! - 包装 tonic Channel + ReplayServiceClient
//! - 提供 `save_replay(...)` 调 replay-service SaveReplay RPC
//! - mTLS fail-closed (per BAS-003 / RGS-REV-007): 默认强制 mTLS, RGS_ALLOW_INSECURE_GRPC=1 显式 opt-out
//! - 简单 retry: 失败 1 次后立即放弃, 不级联 (上游 saga fire-and-forget)
//! - 抽象 `ReplayClientTrait` 用于 UT 注入 mock (per 6 UT 任务要求)
//!
//! ## 设计
//! - `try_connect_lazy`: 用 `Endpoint::connect_lazy()` 构造 Channel (不阻塞)
//! - 实际连接在第一次 RPC 时发生, 失败返 Err
//! - 跨域 SaveReplay 触发点在 `matchmaker_v2` (session Ending → Ended)
//!
//! ## 关联
//! - 上游: RGS-DTL-038 §3 DEC-038-03 (replay-service 推荐 A: cluster-ops 对象存储)
//! - 上游: RGS-DTL-038 §6 (跨域 saga 编排)
//! - 上游: 桶 13 收尾 TODO (match-service session 结束自动 SaveReplay)
//! - 上游: 桶 9 99 测试 0 破坏 (matchmaker_v2 行为保持兼容)

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use uuid::Uuid;

use crate::error::Error;
use crate::replay::v1 as replay_proto;
use crate::Result;

// ============================================================================
// ReplayClientTrait — 抽象接口 (供 UT 注入 mock, 6 UT 任务要求)
// ============================================================================

/// SaveReplay 抽象接口 (UT mock + 真实 tonic client 都实现)
#[async_trait::async_trait]
pub trait ReplayClientTrait: Send + Sync {
    /// 触发 SaveReplay (fire-and-forget 语义, 失败仅记录 warn)
    /// 真实实现: 调 replay-service SaveReplay gRPC
    /// Mock 实现: 捕获请求供 UT 验证
    async fn save_replay(
        &self,
        req: SaveReplayRequest,
    ) -> std::result::Result<SaveReplayOutcome, tonic::Status>;
}

// ============================================================================
// SaveReplay 输入 (与 replay.proto SaveReplayRequest 字段一一对应)
// ============================================================================

/// SaveReplay 请求 (业务层 DTO, 屏蔽 proto 依赖便于 UT + 跨域迁移)
#[derive(Debug, Clone)]
pub struct SaveReplayRequest {
    /// 关联的 match_id (UUID v4)
    pub match_id: Uuid,
    /// 玩家 A (UUID 字符串)
    pub player_a: String,
    /// 玩家 B (可选, 单人 / PvE 可空)
    pub player_b: Option<String>,
    /// 模式 (天梯 / 休闲 / 房间 / PvE)
    pub mode: i32, // 0=unspecified 1=ranked 2=casual 3=room 4=pve_ai
    /// 回放数据 (board snapshot + move log, 序列化 bytes)
    pub data: Vec<u8>,
    /// 比赛时长 (秒)
    pub duration_secs: u32,
    /// 自定义 TTL (秒, 0 = 用 mode 默认)
    pub custom_ttl_secs: i64,
    /// 关联 saga_id (留空 = 不参与 saga)
    pub saga_id: Option<String>,
}

/// SaveReplay 结果
#[derive(Debug, Clone)]
pub struct SaveReplayOutcome {
    /// 服务端生成的 replay_id
    pub replay_id: Uuid,
    /// 对象存储 key (客户端可拉流)
    pub object_key: String,
    /// 对象大小 (bytes)
    pub object_size: i64,
}

// ============================================================================
// ReplayClient — 真实 tonic client (mTLS + lazy connect + 简单 retry)
// ============================================================================

/// mTLS 客户端配置 (per RGS-REV-007 CH4 / DEC-015 P1)
#[derive(Debug, Clone)]
pub struct ReplayClientTlsConfig {
    /// 目标域名 (SAN 或 CN)
    pub domain: String,
    /// CA 证书 PEM 路径
    pub ca_cert_path: String,
    /// 客户端证书 PEM 路径
    pub client_cert_path: String,
    /// 客户端私钥 PEM 路径
    pub client_key_path: String,
}

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ReplayClientConfig {
    /// replay-service gRPC endpoint (e.g. "https://replay-service:50057")
    pub endpoint: String,
    /// RPC 超时 (ms, 默认 2000)
    pub timeout_ms: u64,
    /// mTLS 配置 (None = 明文, 仅 RGS_ALLOW_INSECURE_GRPC=1 dev 模式)
    pub tls: Option<ReplayClientTlsConfig>,
}

impl ReplayClientConfig {
    /// 工厂: 简化构造 (无 mTLS, dev only)
    pub fn insecure(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout_ms: 2000,
            tls: None,
        }
    }

    /// 工厂: 完整 mTLS 配置
    pub fn mtls(
        endpoint: impl Into<String>,
        domain: impl Into<String>,
        ca: impl Into<String>,
        client_cert: impl Into<String>,
        client_key: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout_ms: 2000,
            tls: Some(ReplayClientTlsConfig {
                domain: domain.into(),
                ca_cert_path: ca.into(),
                client_cert_path: client_cert.into(),
                client_key_path: client_key.into(),
            }),
        }
    }
}

/// replay-service gRPC client (per W36 SaveReplay saga)
///
/// - 包装 tonic Channel + ReplayServiceClient<Channel>
/// - 懒连接 (try_connect_lazy 不阻塞)
/// - mTLS fail-closed (per BAS-003)
/// - 简单 retry: 失败 1 次立即放弃, 不级联 (fire-and-forget)
pub struct ReplayClient {
    config: ReplayClientConfig,
    client: replay_proto::replay_service_client::ReplayServiceClient<Channel>,
    /// 简单失败计数 (用于快速熔断, 避免每次都尝试拨号)
    /// - 真实生产应使用 gm-backend 那种 CircuitBreaker; 这里只做轻量级
    consecutive_failures: Arc<AtomicU32>,
    /// 上次失败时间 (用于 cooldown)
    last_failure_at: Arc<Mutex<Option<std::time::Instant>>>,
    /// cooldown 窗口 (失败后 30s 内不重试)
    cooldown: Duration,
}

impl ReplayClient {
    /// 构造客户端 (懒连接, 不阻塞)
    ///
    /// mTLS fail-closed (per RGS-REV-007 CH4 / DEC-015 P1):
    /// - tls = Some → load_client_tls 走 rustls, 失败返 Err
    /// - tls = None + RGS_ALLOW_INSECURE_GRPC=1 → 明文 (dev/test only)
    pub fn try_connect_lazy(config: ReplayClientConfig) -> Result<Self> {
        let endpoint = Endpoint::from_shared(config.endpoint.clone())
            .map_err(|e| Error::Internal(anyhow::anyhow!("invalid replay endpoint: {}", e)))?
            .timeout(Duration::from_millis(config.timeout_ms))
            .connect_timeout(Duration::from_millis(config.timeout_ms));

        let endpoint = if let Some(tls_cfg) = &config.tls {
            // mTLS 模式 (per RGS-REV-007 CH4)
            let input = shared_platform::tls::ClientTlsConfigInput {
                domain: tls_cfg.domain.clone(),
                ca_cert_path: tls_cfg.ca_cert_path.clone(),
                client_cert_path: tls_cfg.client_cert_path.clone(),
                client_key_path: tls_cfg.client_key_path.clone(),
            };
            let client_tls: ClientTlsConfig = shared_platform::tls::load_client_tls(&input)
                .map_err(|e| {
                    Error::Internal(anyhow::anyhow!("replay client TLS load failed: {}", e))
                })?;
            endpoint.tls_config(client_tls).map_err(|e| {
                Error::Internal(anyhow::anyhow!("replay endpoint TLS config failed: {}", e))
            })?
        } else {
            // 明文 (dev only)
            endpoint
        };

        let channel = endpoint.connect_lazy();
        let client = replay_proto::replay_service_client::ReplayServiceClient::new(channel);

        Ok(Self {
            config,
            client,
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            last_failure_at: Arc::new(Mutex::new(None)),
            cooldown: Duration::from_secs(30),
        })
    }

    /// 检查是否在 cooldown 窗口内 (避免短时间内反复拨号失败)
    async fn is_in_cooldown(&self) -> bool {
        let last = self.last_failure_at.lock().await;
        if let Some(at) = *last {
            at.elapsed() < self.cooldown
        } else {
            false
        }
    }

    /// 记录失败 (用于 cooldown)
    async fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        let mut last = self.last_failure_at.lock().await;
        *last = Some(std::time::Instant::now());
    }

    /// 记录成功 (重置失败计数)
    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    /// 返回当前连续失败次数 (供监控/UT 验证)
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// 返回 endpoint (供监控/UT 验证)
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }
}

#[async_trait::async_trait]
impl ReplayClientTrait for ReplayClient {
    async fn save_replay(
        &self,
        req: SaveReplayRequest,
    ) -> std::result::Result<SaveReplayOutcome, tonic::Status> {
        // cooldown 检查
        if self.is_in_cooldown().await {
            return Err(tonic::Status::unavailable(
                "replay-service in cooldown window, skipping",
            ));
        }

        // 构造 proto 请求
        let proto_req = replay_proto::SaveReplayRequest {
            request_id: Uuid::new_v4().to_string(),
            match_id: req.match_id.to_string(),
            player_a: req.player_a.clone(),
            player_b: req.player_b.clone().unwrap_or_default(),
            mode: req.mode,
            data: req.data.clone(),
            duration_secs: req.duration_secs,
            custom_ttl_secs: req.custom_ttl_secs,
            saga_id: req.saga_id.clone().unwrap_or_default(),
        };

        // 单次 RPC (fire-and-forget, 失败不重试 — 简化实现)
        let mut client = self.client.clone();
        let result = client.save_replay(proto_req).await;

        match &result {
            Ok(resp) => {
                self.record_success();
                let inner = resp.get_ref();
                let object_key = inner.object_key.clone();
                let object_size = if let Some(meta) = &inner.meta {
                    meta.object_size
                } else {
                    0
                };
                let replay_id = if let Some(meta) = &inner.meta {
                    Uuid::parse_str(&meta.replay_id).unwrap_or_default()
                } else {
                    Uuid::nil()
                };
                Ok(SaveReplayOutcome {
                    replay_id,
                    object_key,
                    object_size,
                })
            }
            Err(e) => {
                self.record_failure().await;
                Err(e.clone())
            }
        }
    }
}

// ============================================================================
// Tests — 单元测试在 tests/ut_replay_client.rs (独立文件以避免污染主 lib)
// ============================================================================
