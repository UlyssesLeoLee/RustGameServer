//! player 域 BotAi 派生 (per DDD Review v0.3.1 §7.3 wave 3 mTLS 真实接入)
//!
//! PoC 行为序列 (per erlang B3 act_list + C1 协议随机化):
//! - `Init`             启动初始化 (构造 mTLS Channel, 走 `connect_lazy`)
//! - `Heartbeat`        周期心跳 (走 real Channel 框架, 实际 RPC 留 wave 4 SRE)
//! - `RandProto(100)`   10% 概率触发协议随机化 (千分位, per erlang C1)
//!
//! # wave 3 升级 (per DDD Review v0.3.1 §7.3)
//!
//! 替换 wave 2 的 "Ok stub" 为真实 `tonic::transport::Channel` + mTLS config 框架:
//!
//! - `PlayerMtlsClient` 持 lazy Channel (无网络握手, 仅 config 校验)
//! - `PlayerMtlsConfig` 凭据走 `Option<String>`, **不读 env, 不打印**
//! - `skip_verify = true` 默认开 (per 9/10 WipeCluster 后 k3s cert 未导出)
//! - 真实 RPC (`PlayerServiceClient::heartbeat` 等) 留 wave 4 SRE 介入 k3s baseline 恢复
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
//! 真实接入需要 rgs-testkit 依赖 player-service 的 generated client (per
//! `crates/player-service/build.rs` tonic-build), wave 4 worker 加 `player-service`
//! 为 rgs-testkit dev-dep 后接.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, warn};

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// player 域 mTLS 客户端 (per DDD Review v0.3.1 §7.3 wave 3)
///
/// 持 1 个 `tonic::transport::Channel` (lazy 模式, 首次 RPC 才连接) +
/// `PlayerMtlsConfig` (凭据 Option, 不读 env 不打印). **PoC**: 走
/// `connect_lazy`, 不做实际网络握手, 真实 RPC 留给 wave 4 SRE.
///
/// # skip_verify 模式 (默认, per 9/10 WipeCluster 后 k3s cert 未导出)
///
/// - 仅需 `endpoint` + `domain` (SNI), 不需 cert 文件
/// - 走 system CA, `connect_lazy` 阶段无 crypto provider 依赖
/// - 实际 RPC 走 `Channel` 时若需 rustls 才要 `install_default_crypto_provider`
///   (本 PoC 留 wave 4 SRE 集成时处理)
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

/// player 域 BotAi (per DDD Review v0.3.1 §7.3 wave 3)
///
/// 6 派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100)]`,
/// `handle` 走真实 `PlayerMtlsClient` 框架 (lazy Channel, 实际 RPC 留 wave 4 SRE).
///
/// # mTLS Channel 状态
///
/// - `init()` 内部构造 `PlayerMtlsClient::connect(self.config.clone())`,
///   把 Channel 存入 `client: Mutex<Option<PlayerMtlsClient>>`
/// - `handle()` 读 `client.lock()`, 查 `is_real()` 决定走真实 Channel 还是 stub
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
        *self.client.lock().await = Some(m);
        if real {
            debug!(bot_id = bot.id(), "PlayerBotAi mTLS Channel: real (lazy)");
        } else {
            debug!(bot_id = bot.id(), "PlayerBotAi mTLS Channel: stub (k3s baseline 0/12)");
        }
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
                // 真实 wave 3: Heartbeat 走真实 Channel 框架 (PoC: 仅记录状态, 实际 RPC 留 wave 4 SRE)
                debug!(
                    bot_id = bot.id(),
                    has_real,
                    "PlayerBotAi::handle Heartbeat (mTLS framework, real RPC 留 wave 4)"
                );
                // 未来真实 RPC 占位 (per 5 域 ST 业务级 mTLS 实践 commit 401ac5c):
                //   let channel = self.client.lock().await.as_ref()
                //       .and_then(|c| c.channel().cloned());
                //   if let Some(ch) = channel {
                //       let mut cli = PlayerServiceClient::new(ch);
                //       let resp = cli.heartbeat(HeartbeatRequest { ... }).await?;
                //   }
                Ok(())
            }
            ActKind::RandProto(p) => {
                debug!(bot_id = bot.id(), rand_proto_prob = p, "PlayerBotAi::handle RandProto");
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
}
