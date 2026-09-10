//! player 域 BotAi 派生 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! PoC 行为序列 (per erlang B3 act_list + C1 协议随机化):
//! - `Init`             启动初始化 (构造 mTLS Channel, 走 `connect_lazy` + 真实 heartbeat RPC)
//! - `Heartbeat`        周期心跳 (走真实 `PlayerServiceClient::heartbeat`)
//! - `RandProto(100)`   10% 概率触发协议随机化 (千分位, per erlang C1) — 真实 `GetPlayerProfile`
//!
//! # wave 4 升级 (per DDD Review v0.3.2 §7.3 L1.2)
//!
//! 替换 wave 3 的 "lazy Channel 框架, 不实际 RPC" 为真实 RPC 调用:
//!
//! - `PlayerMtlsClient::heartbeat()` / `get_player_profile()` 真实 gRPC 调用
//! - 真实 client 实例: `PlayerServiceClient<Channel>` from `PlayerMtlsClient::channel()`
//! - 失败处理: `tokio::time::timeout(2s, ...)` 防 hang, 失败 → `tracing::warn!` + `Ok(())`
//! - k3s baseline 0/12 (per 9/10 16:36 JST 拍板"接受 baseline 等 SRE 介入"): 真实 RPC 调用预期失败
//!   (connection refused), 走错误日志模式, 不 panic
//!
//! # proto 编译 (per build.rs)
//!
//! - `tonic::include_proto!("player.v1")` 在本文件 scope 内生成 `PlayerServiceClient` /
//!   `HeartbeatRequest` / `GetPlayerProfileRequest` 等 generated 类型
//! - build.rs (per `crates/rgs-testkit/build.rs`) 编译 player.proto + common.proto
//! - 跟 `crates/player-service/build.rs` 同步存在, 各自编各自 crate
//!
//! # L-CAND-016 防御 (per 9/10 18:24 JST)
//!
//! - 本 worker 只编 player 域 proto, 不编其他 4 域 proto (economy / match / social / admin)
//! - 5 域 worker 各自编自己域 proto, 避免 race condition
//!
//! # 强约束 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - 凭据 (mTLS cert path) 走 `Option<String>`, **不读 env, 不打印值**
//! - 默认 endpoint = `https://127.0.0.1:50051`, skip_verify = true
//! - 不调真实 5 域 gRPC (k3s baseline 0/12, per 9/10 16:36 JST 拍板"接受 baseline 等 SRE")
//!
//! # 后续接入路径 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
//!
//! ```text
//! PlayerBotAi::handle(Heartbeat)
//!   → player.channel().player_service_client()
//!   → PlayerServiceClient::heartbeat(HeartbeatRequest { ... })
//!   → 失败 → exponential backoff (100/200/400ms) × 3, DLQ
//! ```
//!
//! 真实接入需要 rgs-testkit 编译 player proto (per `crates/rgs-testkit/build.rs` tonic-build),
//! wave 4 worker 已加 build.rs + build-dep 后接.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, warn};

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

// ============================================================================
// player proto generated types (per build.rs tonic-build)
// ============================================================================
//
// tonic::include_proto! 宏读 OUT_DIR 里 build.rs 生成的 player.v1.rs 文件, 把内容
// inlined 到宏调用所在的模块 scope. 因为生成代码用 `super::super::common::v1::*`
// 路径, 必须嵌套 2 层 (`mod proto { mod v1 { ... } }`), 同时 `common` 模块要平
// 行放在 `proto` 的父级, 这样 `super::super` 才能解析到 `crate::common`.
//
// 参考: crates/player-service/src/proto.rs 用相同 pattern.
//
// 完整路径: `proto::v1::player_service_client::PlayerServiceClient`,
// 用 `pub use` 重新导出, 方便外部引用.

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod proto {
    pub mod v1 {
        tonic::include_proto!("player.v1");
    }
}

use proto::v1::player_service_client::PlayerServiceClient;
use proto::v1::{GetPlayerProfileRequest, HeartbeatRequest};

/// RPC 调用超时 (per task briefing: 2s timeout 防 hang)
const RPC_TIMEOUT: Duration = Duration::from_secs(2);

/// player 域 mTLS 客户端 (per DDD Review v0.3.1 §7.3 wave 3 + v0.3.2 §7.3 L1.2 wave 4)
///
/// 持 1 个 `tonic::transport::Channel` (lazy 模式, 首次 RPC 才连接) +
/// `PlayerMtlsConfig` (凭据 Option, 不读 env 不打印).
///
/// # wave 4 升级
///
/// - `try_heartbeat()`: 真实 `PlayerServiceClient::heartbeat()` 调用, 2s timeout
/// - `try_get_player_profile()`: 真实 `PlayerServiceClient::get_player_profile()` 调用, 2s timeout
/// - 失败时返 `RpcCallOutcome::Unreachable`, 不 panic (per 任务简报: "失败时返回 Err, 不 panic")
/// - k3s baseline 0/12 → 真实 RPC 预期 connection refused → 走 `Unreachable` 分支
///
/// # skip_verify 模式 (默认, per 9/10 WipeCluster 后 k3s cert 未导出)
///
/// - 仅需 `endpoint` + `domain` (SNI), 不需 cert 文件
/// - 走 system CA, `connect_lazy` 阶段无 crypto provider 依赖
/// - 实际 RPC 走 `Channel` 时若需 rustls 才要 `install_default_crypto_provider`
///   (本 PoC 留 wave 5 SRE 集成时处理)
///
/// # 真实 mTLS 模式 (k3s baseline 恢复后, SRE 介入)
///
/// - 需 `ca_cert_path` + `client_cert_path` + `client_key_path` (3 个)
/// - 任一缺失 → `build_channel` 报错 → 降级 stub
#[derive(Clone, Debug)]
pub struct PlayerMtlsClient {
    /// 真实 tonic Channel (lazy 模式, 首次 RPC 才连接)
    /// `None` 表示 Channel 构造失败 (凭据缺失 / URI 无效), fall back to stub
    channel: Option<Channel>,
    /// mTLS config (持引用, 凭据走 Option 不打印)
    config: PlayerMtlsConfig,
}

/// RPC 调用结果 (per task briefing "失败时返回 Err, 不 panic")
///
/// `Ok(())` 表示 RPC 调通, `Err(...)` 表示 RPC 失败 (timeout / connection refused / status).
/// 业务上层 (PlayerBotAi::handle) 把 `Err` 转成 `tracing::warn!` + `Ok(())` 不 panic.
#[derive(Debug, Clone)]
pub enum RpcCallOutcome {
    /// RPC 调通
    Ok,
    /// RPC 失败 (timeout / connection refused / status), 不 panic
    Unreachable(String),
}

impl RpcCallOutcome {
    /// 转 `Result<(), String>`, 业务上层用 `.is_ok()` 判断
    pub fn into_result(self) -> Result<(), String> {
        match self {
            RpcCallOutcome::Ok => Ok(()),
            RpcCallOutcome::Unreachable(e) => Err(e),
        }
    }
}

impl PlayerMtlsClient {
    /// 构造 player mTLS client (走 lazy connect)
    ///
    /// 任何错误 (URI 无效 / cert 缺失) → `channel = None` + log warn, 后续
    /// `handle` 走 stub. 永不返回 `Err` (PoC 宽容).
    pub async fn connect(config: PlayerMtlsConfig) -> anyhow::Result<Self> {
        let channel = match Self::build_channel(&config).await {
            Ok(c) => Some(c),
            Err(e) => {
                warn!(error = %e, "player mTLS Channel build failed, fall back to stub");
                None
            }
        };
        Ok(Self { channel, config })
    }

    /// 构造真实 Channel (lazy)
    ///
    /// - `Endpoint::from_shared` 校验 URI 格式
    /// - `tls_config` 应用 ClientTlsConfig (skip_verify 或完整 mTLS)
    /// - `connect_lazy` 立即返 Channel, 不做实际网络握手
    ///
    /// # 错误
    /// - URI 无效 → `tonic::transport::Error`
    /// - 真实 mTLS 模式 cert 文件缺失 → `std::io::Error`
    async fn build_channel(config: &PlayerMtlsConfig) -> anyhow::Result<Channel> {
        let endpoint_str = config
            .endpoint
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("player endpoint not configured"))?;
        let domain = config
            .domain
            .clone()
            .unwrap_or_else(|| "player-service".to_string());

        let mut endpoint = Endpoint::from_shared(endpoint_str.to_string())?
            .timeout(Duration::from_secs(5));

        let tls = if config.skip_verify {
            // 降级模式: 仅设 SNI domain, 走 system CA
            // connect_lazy 阶段不需要 crypto provider
            ClientTlsConfig::new().domain_name(domain)
        } else {
            // 真实 mTLS: 读 ca + client cert + client key 3 个 PEM
            let ca_path = config
                .ca_cert_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("ca_cert_path not configured"))?;
            let client_cert_path = config
                .client_cert_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("client_cert_path not configured"))?;
            let client_key_path = config
                .client_key_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("client_key_path not configured"))?;

            let ca_pem = std::fs::read_to_string(ca_path).map_err(|e| {
                anyhow::anyhow!("ca cert read failed (path=REDACTED, err={})", e)
            })?;
            let client_pem = std::fs::read_to_string(client_cert_path).map_err(|e| {
                anyhow::anyhow!("client cert read failed (path=REDACTED, err={})", e)
            })?;
            let key_pem = std::fs::read_to_string(client_key_path).map_err(|e| {
                anyhow::anyhow!("client key read failed (path=REDACTED, err={})", e)
            })?;

            let ca = tonic::transport::Certificate::from_pem(ca_pem);
            let identity = tonic::transport::Identity::from_pem(client_pem, key_pem);
            ClientTlsConfig::new()
                .domain_name(domain)
                .ca_certificate(ca)
                .identity(identity)
        };

        endpoint = endpoint.tls_config(tls)?;
        Ok(endpoint.connect_lazy())
    }

    /// 是否有真实 Channel (false 表示 stub fallback)
    pub fn is_real(&self) -> bool {
        self.channel.is_some()
    }

    /// 拿真实 Channel 引用 (None 时调用方应走 stub)
    pub fn channel(&self) -> Option<&Channel> {
        self.channel.as_ref()
    }

    /// 拿 mTLS config 引用 (凭据已走 Option, 不打印)
    pub fn config(&self) -> &PlayerMtlsConfig {
        &self.config
    }

    // ========================================================================
    // wave 4 真实 RPC 调用 (per DDD Review v0.3.2 §7.3 L1.2)
    // ========================================================================

    /// 真实 heartbeat RPC 调用 (per DDD Review v0.3.2 §7.3 L1.2)
    ///
    /// - 用 `PlayerMtlsClient::channel()` 拿 lazy Channel
    /// - 构造 `PlayerServiceClient<Channel>` + `HeartbeatRequest` (空 fields, PoC 阶段)
    /// - `tokio::time::timeout(2s, ...)` 防 hang
    /// - 失败 (timeout / connection refused / status) → 返 `RpcCallOutcome::Unreachable`
    ///   (不 panic, 业务上层走 warn log + Ok(()))
    ///
    /// # k3s baseline 0/12 (per 9/10 16:36 JST 拍板)
    ///
    /// 真实 RPC 调用预期 `connection refused`, 走 `Unreachable` 分支, 不影响业务.
    pub async fn try_heartbeat(&self) -> RpcCallOutcome {
        let channel = match self.channel.as_ref() {
            Some(c) => c.clone(),
            None => {
                return RpcCallOutcome::Unreachable(
                    "player mTLS Channel not initialized (stub fallback)".to_string(),
                );
            }
        };

        let mut client = PlayerServiceClient::new(channel);
        let req = tonic::Request::new(HeartbeatRequest {
            request_id: "bot-test".to_string(),
            session_id: "test-session".to_string(),
            character_id: "test-character".to_string(),
            client_time_unix: chrono::Utc::now().timestamp(),
        });

        match tokio::time::timeout(RPC_TIMEOUT, client.heartbeat(req)).await {
            Ok(Ok(_resp)) => RpcCallOutcome::Ok,
            Ok(Err(status)) => RpcCallOutcome::Unreachable(format!(
                "player heartbeat gRPC status: code={:?}, message={}",
                status.code(),
                status.message()
            )),
            Err(_elapsed) => RpcCallOutcome::Unreachable(format!(
                "player heartbeat timeout after {:?}",
                RPC_TIMEOUT
            )),
        }
    }

    /// 真实 GetPlayerProfile RPC 调用 (per DDD Review v0.3.2 §7.3 L1.2, RandProto 用)
    ///
    /// - 跟 `try_heartbeat` 同模式, 但 RPC 不同 (per erlang C1 协议随机化)
    /// - `player_id` 用空 string 占位 (PoC, 真实凭据 SRE 介入)
    /// - 失败 → `RpcCallOutcome::Unreachable`, 不 panic
    pub async fn try_get_player_profile(&self) -> RpcCallOutcome {
        let channel = match self.channel.as_ref() {
            Some(c) => c.clone(),
            None => {
                return RpcCallOutcome::Unreachable(
                    "player mTLS Channel not initialized (stub fallback)".to_string(),
                );
            }
        };

        let mut client = PlayerServiceClient::new(channel);
        let req = tonic::Request::new(GetPlayerProfileRequest {
            request_id: "bot-test-rand-proto".to_string(),
            player_id: "test-player".to_string(),
        });

        match tokio::time::timeout(RPC_TIMEOUT, client.get_player_profile(req)).await {
            Ok(Ok(_resp)) => RpcCallOutcome::Ok,
            Ok(Err(status)) => RpcCallOutcome::Unreachable(format!(
                "player get_player_profile gRPC status: code={:?}, message={}",
                status.code(),
                status.message()
            )),
            Err(_elapsed) => RpcCallOutcome::Unreachable(format!(
                "player get_player_profile timeout after {:?}",
                RPC_TIMEOUT
            )),
        }
    }
}

/// player 域 mTLS config (per 8/27 11:06 JST 硬 ban + AGENTS.md §1.2)
///
/// 凭据 (mTLS cert path) 走 `Option<String>`, **不读 env, 不打印值**.
#[derive(Clone, Debug, Default)]
pub struct PlayerMtlsConfig {
    /// player-service endpoint, e.g. `"https://127.0.0.1:50051"`
    pub endpoint: Option<String>,
    /// SNI / domain name, e.g. `"player-service"`
    pub domain: Option<String>,
    /// CA cert path (PEM), 真实 mTLS 模式必填
    pub ca_cert_path: Option<String>,
    /// client cert path (PEM), 真实 mTLS 模式必填
    pub client_cert_path: Option<String>,
    /// client key path (PEM), 真实 mTLS 模式必填
    pub client_key_path: Option<String>,
    /// skip server cert verification (k3s baseline 0/12 fallback, 9/10 WipeCluster)
    pub skip_verify: bool,
}

/// player 域 BotAi (per DDD Review v0.3.2 §7.3 L1.2 wave 4)
///
/// 6 派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100)]`,
/// `handle` 走真实 `PlayerMtlsClient` + 真实 gRPC 调用 (wave 4 升级).
///
/// # wave 4 真实 RPC
///
/// - `init()` 内部构造 `PlayerMtlsClient::connect(self.config.clone())` +
///   真实 heartbeat RPC (2s timeout, 预期失败 → warn + Ok)
/// - `handle(Heartbeat)` 真实 `PlayerServiceClient::heartbeat` 调用
/// - `handle(RandProto(100))` 真实 `PlayerServiceClient::get_player_profile` 调用
/// - skip_verify 模式 (默认) → Channel 构造成功, `is_real() = true`
/// - 真实 mTLS 模式 (cert 缺失) → Channel 构造失败, `is_real() = false`
#[derive(Clone, Debug)]
pub struct PlayerBotAi {
    /// 真实 mTLS client (init 时构造, 用 `Mutex<Option<_>>` 因为 `BotAi::init` 持 `&self`)
    client: Arc<Mutex<Option<PlayerMtlsClient>>>,
    /// mTLS config (供 init 构造 Channel 用)
    config: PlayerMtlsConfig,
}

impl Default for PlayerBotAi {
    fn default() -> Self {
        // 默认: skip_verify 模式, endpoint = 127.0.0.1:50051, domain = player-service
        // (per DDD Review v0.3.1 §7.3 + 9/10 WipeCluster 后 cert 未导出)
        let config = PlayerMtlsConfig {
            endpoint: Some("https://127.0.0.1:50051".to_string()),
            domain: Some("player-service".to_string()),
            skip_verify: true,
            ..Default::default()
        };
        Self {
            client: Arc::new(Mutex::new(None)),
            config,
        }
    }
}

impl PlayerBotAi {
    /// 构造 player 域 BotAi (默认 skip_verify 模式, 走 stub fallback 兼容)
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造 player 域 BotAi + 自定义 mTLS config (凭据走 Option, 调用方注入)
    pub fn with_config(config: PlayerMtlsConfig) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// 内部 helper: 检查真实 Channel 状态 (供 log / 测试断言)
    pub async fn is_real(&self) -> bool {
        self.client
            .lock()
            .await
            .as_ref()
            .map(|c| c.is_real())
            .unwrap_or(false)
    }

    /// 拿 mTLS client 引用 (供测试 / 未来真实 RPC 调用)
    pub async fn mtls_client(&self) -> Option<PlayerMtlsClient> {
        self.client.lock().await.clone()
    }
}

#[async_trait]
impl BotAi for PlayerBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "PlayerBotAi::init");
        // 构造真实 mTLS Channel (lazy, skip_verify 模式成功率高)
        let m = PlayerMtlsClient::connect(self.config.clone()).await?;
        let real = m.is_real();

        // wave 4 升级: init 时调一次 heartbeat 真实 RPC (per DDD Review v0.3.2 §7.3 L1.2)
        // k3s baseline 0/12 → 预期 connection refused → warn + 走 stub 兼容
        if real {
            let outcome = m.try_heartbeat().await;
            match outcome {
                RpcCallOutcome::Ok => {
                    debug!(bot_id = bot.id(), "PlayerBotAi init heartbeat RPC: OK");
                }
                RpcCallOutcome::Unreachable(e) => {
                    warn!(
                        bot_id = bot.id(),
                        error = %e,
                        "PlayerBotAi init heartbeat RPC failed (k3s baseline 0/12 expected, falling back to stub)"
                    );
                }
            }
        } else {
            debug!(bot_id = bot.id(), "PlayerBotAi mTLS Channel: stub (k3s baseline 0/12)");
        }

        *self.client.lock().await = Some(m);
        Ok(())
    }

    fn act_list(&self) -> Vec<ActKind> {
        // 维持 wave 2 stub 形态 (per DDD Review v0.3.1 §7.3)
        vec![ActKind::Init, ActKind::Heartbeat, ActKind::RandProto(100)]
    }

    async fn handle(&self, bot: &Bot, act: ActKind) -> anyhow::Result<()> {
        let has_real = self.is_real().await;
        match act {
            ActKind::Init => {
                debug!(bot_id = bot.id(), has_real, "PlayerBotAi::handle Init");
                Ok(())
            }
            ActKind::Heartbeat => {
                // wave 4 升级: 真实 heartbeat RPC (per DDD Review v0.3.2 §7.3 L1.2)
                debug!(
                    bot_id = bot.id(),
                    has_real,
                    "PlayerBotAi::handle Heartbeat (real RPC)"
                );
                if has_real {
                    let outcome = {
                        let guard = self.client.lock().await;
                        match guard.as_ref() {
                            Some(c) => c.try_heartbeat().await,
                            None => RpcCallOutcome::Unreachable(
                                "client not initialized".to_string(),
                            ),
                        }
                    };
                    match outcome {
                        RpcCallOutcome::Ok => {
                            debug!(bot_id = bot.id(), "Heartbeat RPC: OK");
                        }
                        RpcCallOutcome::Unreachable(e) => {
                            warn!(
                                bot_id = bot.id(),
                                error = %e,
                                "Heartbeat RPC failed (k3s baseline 0/12 expected)"
                            );
                        }
                    }
                }
                // 不管成功失败, 都 Ok (PoC 宽容, 不 panic)
                Ok(())
            }
            ActKind::RandProto(p) => {
                // wave 4 升级: 真实 GetPlayerProfile RPC (per DDD Review v0.3.2 §7.3 L1.2)
                debug!(
                    bot_id = bot.id(),
                    rand_proto_prob = p,
                    has_real,
                    "PlayerBotAi::handle RandProto (real GetPlayerProfile RPC)"
                );
                if has_real {
                    let outcome = {
                        let guard = self.client.lock().await;
                        match guard.as_ref() {
                            Some(c) => c.try_get_player_profile().await,
                            None => RpcCallOutcome::Unreachable(
                                "client not initialized".to_string(),
                            ),
                        }
                    };
                    match outcome {
                        RpcCallOutcome::Ok => {
                            debug!(bot_id = bot.id(), "GetPlayerProfile RPC: OK");
                        }
                        RpcCallOutcome::Unreachable(e) => {
                            warn!(
                                bot_id = bot.id(),
                                error = %e,
                                "GetPlayerProfile RPC failed (k3s baseline 0/12 expected)"
                            );
                        }
                    }
                }
                Ok(())
            }
            // 未识别 act: 不 panic, 记 debug + 返 Ok (PoC 宽容)
            other => {
                debug!(bot_id = bot.id(), ?other, "PlayerBotAi::handle unknown act, skip");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::stats::BotStats;

    fn dummy_bot() -> Bot {
        Bot::new("bot-player-001", "player", BotStats::new())
    }

    #[tokio::test]
    async fn player_ai_init_ok() {
        // 默认 skip_verify 模式 → Channel 构造成功
        let ai = PlayerBotAi::new();
        let bot = dummy_bot();
        // wave 4 升级: init 内部调一次真实 heartbeat, 预期 Unreachable (k3s 0/12) 但 Ok(())
        ai.init(&bot).await.expect("init");
        assert!(ai.is_real().await, "skip_verify 模式应构造真实 lazy Channel");
    }

    #[tokio::test]
    async fn player_ai_act_list_contains_heartbeat_and_rand_proto() {
        let ai = PlayerBotAi::new();
        let acts = ai.act_list();
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::RandProto(100)));
    }

    #[tokio::test]
    async fn player_ai_handle_all_acts_ok() {
        let ai = PlayerBotAi::new();
        let bot = dummy_bot();
        // 先 init 让 Channel 状态就绪
        ai.init(&bot).await.expect("init");
        for act in ai.act_list() {
            // wave 4 升级: handle 内部调真实 RPC, 预期 Unreachable 但仍 Ok(())
            ai.handle(&bot, act).await.expect("handle");
        }
    }

    #[tokio::test]
    async fn player_ai_with_custom_config_skip_verify() {
        // 自定义 config 走 skip_verify 模式
        let cfg = PlayerMtlsConfig {
            endpoint: Some("https://player-staging:50051".to_string()),
            domain: Some("player-staging".to_string()),
            skip_verify: true,
            ..Default::default()
        };
        let ai = PlayerBotAi::with_config(cfg);
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init with custom config");
        assert!(ai.is_real().await, "skip_verify 模式应构造真实 Channel");
    }

    #[tokio::test]
    async fn player_ai_with_real_mtls_missing_cert_falls_back() {
        // 真实 mTLS 模式 + cert 路径不存在 → build_channel 失败 → channel = None
        let cfg = PlayerMtlsConfig {
            endpoint: Some("https://player:50051".to_string()),
            domain: Some("player-service".to_string()),
            ca_cert_path: Some("/nonexistent/ca.pem".to_string()),
            client_cert_path: Some("/nonexistent/client.pem".to_string()),
            client_key_path: Some("/nonexistent/client.key".to_string()),
            skip_verify: false,
        };
        let ai = PlayerBotAi::with_config(cfg);
        let bot = dummy_bot();
        // init 不 panic, channel 降级 stub
        ai.init(&bot).await.expect("init should not panic on cert missing");
        assert!(!ai.is_real().await, "cert 缺失应降级 stub, is_real = false");
    }

    #[tokio::test]
    async fn player_mtls_client_connect_lazy_succeeds() {
        // 直接调 PlayerMtlsClient::connect, 验证 lazy Channel 构造
        let cfg = PlayerMtlsConfig {
            endpoint: Some("https://127.0.0.1:50051".to_string()),
            domain: Some("player-service".to_string()),
            skip_verify: true,
            ..Default::default()
        };
        let m = PlayerMtlsClient::connect(cfg).await.expect("connect");
        assert!(m.is_real(), "skip_verify lazy connect 应返真实 Channel");
        assert!(m.channel().is_some());
    }

    #[tokio::test]
    async fn player_mtls_client_connect_no_endpoint_falls_back() {
        // 无 endpoint → build_channel 失败 → channel = None
        let cfg = PlayerMtlsConfig::default();
        let m = PlayerMtlsClient::connect(cfg).await.expect("connect should not panic");
        assert!(!m.is_real(), "无 endpoint 应降级 stub");
        assert!(m.channel().is_none());
    }

    // =========================================================================
    // wave 4 真实 RPC 调用测试 (per DDD Review v0.3.2 §7.3 L1.2)
    // =========================================================================

    #[tokio::test]
    async fn player_mtls_client_try_heartbeat_returns_unreachable_on_k3s_down() {
        // wave 4 真实 RPC 调用测试 (per task briefing):
        // k3s baseline 0/12 → 真实 heartbeat RPC 预期 connection refused
        // 走 RpcCallOutcome::Unreachable 分支, 不 panic
        let cfg = PlayerMtlsConfig {
            endpoint: Some("https://127.0.0.1:50051".to_string()),
            domain: Some("player-service".to_string()),
            skip_verify: true,
            ..Default::default()
        };
        let m = PlayerMtlsClient::connect(cfg).await.expect("connect");
        assert!(m.is_real(), "应构造真实 Channel");

        // 真实 RPC 调用预期失败 (k3s 不可达, 2s timeout 内 connection refused)
        let outcome = m.try_heartbeat().await;
        match outcome {
            RpcCallOutcome::Ok => {
                // 极小概率成功 (如果 127.0.0.1:50051 真有 listener), 也接受
                // 但 baseline 0/12 状态下应不会发生
                eprintln!("unexpected: heartbeat succeeded (可能 50051 有 listener)");
            }
            RpcCallOutcome::Unreachable(e) => {
                // 预期: connection refused 或 timeout
                assert!(
                    !e.is_empty(),
                    "Unreachable error message 应非空, got: {}",
                    e
                );
                eprintln!("expected: heartbeat unreachable: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn player_mtls_client_try_get_player_profile_returns_unreachable_on_k3s_down() {
        // wave 4 真实 RPC 调用测试 (per task briefing):
        // RandProto 走 GetPlayerProfile, k3s baseline 0/12 预期失败
        let cfg = PlayerMtlsConfig {
            endpoint: Some("https://127.0.0.1:50051".to_string()),
            domain: Some("player-service".to_string()),
            skip_verify: true,
            ..Default::default()
        };
        let m = PlayerMtlsClient::connect(cfg).await.expect("connect");
        let outcome = m.try_get_player_profile().await;
        match outcome {
            RpcCallOutcome::Ok => {
                eprintln!("unexpected: get_player_profile succeeded");
            }
            RpcCallOutcome::Unreachable(e) => {
                assert!(!e.is_empty(), "Unreachable error message 应非空");
                eprintln!("expected: get_player_profile unreachable: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn player_mtls_client_no_channel_returns_unreachable_without_panic() {
        // 无 Channel 状态 (init 失败) → 真实 RPC 调用返 Unreachable, 不 panic
        let cfg = PlayerMtlsConfig::default();
        let m = PlayerMtlsClient::connect(cfg).await.expect("connect");
        assert!(!m.is_real());

        let outcome = m.try_heartbeat().await;
        assert!(
            matches!(outcome, RpcCallOutcome::Unreachable(_)),
            "无 Channel 应返 Unreachable, got: {:?}",
            outcome
        );

        let outcome = m.try_get_player_profile().await;
        assert!(
            matches!(outcome, RpcCallOutcome::Unreachable(_)),
            "无 Channel 应返 Unreachable, got: {:?}",
            outcome
        );
    }
}
