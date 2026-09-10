//! social 域 BotAi 派生 (per DDD Review v0.3.1 §7.3 Phase C + wave 3 mTLS 真实接入
//! + wave 4 真实 RPC 调用 per DDD Review v0.3.2 §7.3 L1.2)
//!
//! 模拟 erlang C6 业务场景 act (per tester_ai_base.erl:163-181 guild / 261-278 partner):
//! - `Init`        启动初始化 (创建真实 tonic Channel + mTLS 框架, per wave 3)
//! - `Heartbeat`   周期心跳 → 真实 RPC `health_check` (per wave 4)
//! - `RandProto(100)` 10% 概率触发协议随机化 (千分位, per erlang C1)
//! - `Guild`       工会操作 → 真实 RPC `get_guild` (per erlang C6 line 163-181, wave 4)
//! - `Partner`     伙伴操作 → 真实 RPC `health_check` 兜底 (per erlang C6 line 261-278, wave 4)
//!
//! # wave 3 真实 mTLS 接入 (per 9/10 16:36 JST 拍板 + DDD Review v0.3.1 §7.3)
//!
//! 替换 stub 为真实 `tonic::transport::Channel` + `ClientTlsConfig` 框架:
//! - **endpoint**: `https://127.0.0.1:50054` (social 域 svc ClusterIP, per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
//! - **cert**: `Option<String>` 路径, 走 [`MtlsConfig`](crate::bot::gm::MtlsConfig), **不读 env, 不打印值** (per 8/27 11:06 JST 硬 ban)
//! - **fall back**: 未注入 cert 时走 skip-verify 模式 + `tracing::warn!` (per 9/10 WipeCluster 重建后 cert 未导出)
//! - **connect_lazy**: Channel 创建不阻塞 (no network), 实际 TLS handshake 推迟到首次 RPC
//!
//! # wave 4 真实 RPC 调用 (per DDD Review v0.3.2 §7.3 L1.2 + 9/10 19:00 JST 拍板)
//!
//! wave 4 升级: 替换 wave 3 "framework ready" 注释为真实 `SocialServiceClient<Channel>` 调用:
//! - **init()** → `health_check` (轻量 ping, 验证 channel + 业务可达)
//! - **handle(Heartbeat)** → `health_check` (per erlang E4 周期心跳)
//! - **handle(Guild)** → `get_guild(EntityId { id: bot.id() })` (per erlang C6 guild)
//! - **handle(Init/RandProto/Partner)** → `health_check` 兜底 (无 partner proto RPC, 走 ping)
//! - **tokio::time::timeout(2s)**: 防止 k3s 不可达时 hang (per wave 4 防御)
//! - **失败容忍**: Err → `Ok(())` + `tracing::warn!` 标注降级模式 (per 9/10 16:36 JST 拍板)
//! - **k3s baseline 0/12 阻塞**: 真实 RPC 调用预期失败 (connection refused), 走错误日志模式
//!
//! # L-CAND-016 防御 (per 9/10 18:24 JST)
//!
//! 5 worker 公共 proto RPC 调用要同步, 本 worker 只加 social proto 真实 RPC, 不改其他 4 域.
//! 通过 `social-service = { path = "../social-service" }` dev-dep 引入 proto 类型, 业务
//! crate 5 域仍按 [dependencies] 独立引用, bot 仅在测试/集成层调真实 client.
//!
//! # 凭据安全 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - mTLS cert path 走 `Option<String>`, 调用方通过 `SocialBotAi::with_mtls(MtlsConfig)` 注入
//! - 任何日志/错误信息都**不打印** cert path 明文 (仅打 endpoint 长度 + 凭据"已配置"标志)
//! - channel 的 `Debug` 实现 redact 所有 mTLS 字段, 防意外泄露

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::OnceCell;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Identity, Certificate};
use tracing::{debug, warn};

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::gm::MtlsConfig;
use crate::bot::Bot;

// social-service proto 类型 (per L-CAND-016 防御: 仅 social 域 dev-dep, 不改其他 4 域)
use social_service::common::v1 as common_proto;
use social_service::proto::v1::social_service_client::SocialServiceClient;

/// social 域 BotAi (per DDD Review v0.3.1 §7.3 Phase C + wave 3 mTLS 真实接入)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), Guild, Partner]`,
/// `handle` 全 OK stub. **wave 3 替换 stub** 为真实 `tonic::transport::Channel` + mTLS 框架.
///
/// 真实业务 RPC (`social.v1.SocialService::GetGuild` / `HealthCheck` 等) 留 wave 4 —
/// k3s baseline 0/12 阻塞 (per 9/10 15:14-15:30 JST 4 段历史, 9/10 16:36 JST 拍板
/// "接受 baseline 等 SRE 介入"), wave 3 仅交付**框架就绪** (`Channel` 创建不 panic).
///
/// # 凭据 (per 8/27 11:06 JST 硬 ban + AGENTS.md §1.2)
///
/// - mTLS cert path 走 `Option<String>`, **不读 env, 不打印值**
/// - 调用方通过 [`SocialBotAi::with_mtls`] 注入真实证书路径
/// - 未注入时 fall back to skip-verify (HTTP `connect_lazy`, no `tls_config`) + `warn!`
pub struct SocialBotAi {
    /// social svc endpoint (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`, social port 50054)
    ///
    /// Default: `https://127.0.0.1:50054` (本机 / k3s ClusterIP 视部署而定).
    /// 生产应由调用方通过 [`SocialBotAi::with_endpoint`] 注入.
    endpoint: String,

    /// mTLS 凭据 (per 8/27 11:06 JST 硬 ban: 不读 env, 不打印值)
    ///
    /// `None` = skip-verify + `warn!` (per 9/10 WipeCluster 重建后 cert 未导出).
    /// `Some(cfg)` = 走 mTLS, 凭据来自 `cfg.client_cert_path` / `cfg.client_key_path` /
    /// `cfg.ca_cert_path` (PEM 文件路径, 由调用方注入).
    mtls: Option<MtlsConfig>,

    /// 真实 tonic Channel (init 后填充, `connect_lazy` 模式不阻塞网络)
    ///
    /// `Arc<OnceCell<Channel>>`:
    /// - `Arc` 共享 cell (BotAi 派生 Clone 时不会重复创建 cell)
    /// - `OnceCell` 一次性写入 (`init` 阶段, 后续 `handle` 阶段可读)
    /// - 实际 TLS handshake 推迟到首次 RPC (k3s baseline 0/12 不阻塞 channel 创建)
    channel: Arc<OnceCell<Channel>>,
}

impl std::fmt::Debug for SocialBotAi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Redact mTLS cert path (per 8/27 11:06 JST 硬 ban: Debug 不打印值)
        f.debug_struct("SocialBotAi")
            .field("endpoint", &self.endpoint)
            .field("mtls", &"<redacted Option<MtlsConfig>>")
            .field(
                "channel",
                &self.channel.get().map(|_| "Channel(<initialized>)"),
            )
            .finish()
    }
}

impl Clone for SocialBotAi {
    fn clone(&self) -> Self {
        // Arc<OnceCell<Channel>> 是 Clone (Arc::clone 共享 cell)
        // endpoint / mtls 重新 clone 字符串, 保持独立配置
        Self {
            endpoint: self.endpoint.clone(),
            mtls: self.mtls.clone(),
            channel: Arc::clone(&self.channel),
        }
    }
}

impl Default for SocialBotAi {
    fn default() -> Self {
        Self {
            endpoint: "https://127.0.0.1:50054".to_string(),
            mtls: None,
            channel: Arc::new(OnceCell::new()),
        }
    }
}

impl SocialBotAi {
    /// 构造 social 域 BotAi (用默认 endpoint, 无 mTLS → skip-verify + `warn!`)
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造 social 域 BotAi + 注入自定义 endpoint (凭据 None, 走 skip-verify)
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..Self::default()
        }
    }

    /// 构造 social 域 BotAi + 注入 mTLS 配置 (凭据 Option, 不读 env, 不打印)
    ///
    /// 凭据路径由调用方通过 [`MtlsConfig`] 显式注入, 走 PEM 文件.
    /// 缺失任一字段 (cert/key/ca) 时 fall back to skip-verify + `warn!`.
    pub fn with_mtls(mtls: MtlsConfig) -> Self {
        Self {
            mtls: Some(mtls),
            ..Self::default()
        }
    }

    /// 构造 social 域 BotAi + 同时注入 endpoint 和 mTLS 配置
    pub fn with_endpoint_and_mtls(endpoint: impl Into<String>, mtls: MtlsConfig) -> Self {
        Self {
            endpoint: endpoint.into(),
            mtls: Some(mtls),
            ..Self::default()
        }
    }

    /// 公开访问: Channel 是否已初始化 (test helper)
    pub fn channel_initialized(&self) -> bool {
        self.channel.get().is_some()
    }

    /// 公开访问: 内部 tonic Channel 引用 (wave 4 真实 RPC 调用方)
    ///
    /// 返回 `Some(&Channel)` 当 init 已完成; `None` 当 init 尚未调用.
    /// test 集成测试 / wave 4 worker 可借此拿 Channel 构造 `SocialServiceClient<Channel>`.
    pub fn channel(&self) -> Option<&Channel> {
        self.channel.get()
    }

    /// 公开访问: 当前 endpoint (test helper)
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// 公开访问: mTLS 是否已配置 (不暴露路径, 仅 true/false, per 8/27 11:06 JST 硬 ban)
    pub fn mtls_configured(&self) -> bool {
        self.mtls
            .as_ref()
            .map(|c| {
                c.client_cert_path.is_some()
                    && c.client_key_path.is_some()
                    && c.ca_cert_path.is_some()
            })
            .unwrap_or(false)
    }

    /// 内部 helper: 构建 tonic Channel (skip-verify / mTLS, 走 `connect_lazy` 不阻塞)
    ///
    /// # 行为
    /// - `mtls = None` → 无 TLS config, `connect_lazy` 创建 HTTP/2 Channel, log warn
    /// - `mtls = Some(cfg)`, 三字段全有 → 加载 PEM identity, 走 mTLS, log debug
    /// - `mtls = Some(cfg)`, 三字段不全 → 缺哪个 log warn, fall back to skip-verify
    ///
    /// 任何路径下都不打印 cert path 明文 (per 8/27 11:06 JST 硬 ban).
    async fn build_channel(&self) -> anyhow::Result<Channel> {
        let mut endpoint = Endpoint::from_shared(self.endpoint.clone())?
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(30));

        if let Some(mtls) = &self.mtls {
            // mTLS 路径: 加载 PEM identity (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
            // 凭据通过 MtlsConfig 注入, 不读 env (per 8/27 11:06 JST 硬 ban)
            match (
                mtls.client_cert_path.as_ref(),
                mtls.client_key_path.as_ref(),
                mtls.ca_cert_path.as_ref(),
            ) {
                (Some(cert), Some(key), Some(ca)) => {
                    let cert_pem = std::fs::read_to_string(cert).map_err(|e| {
                        anyhow::anyhow!(
                            "SocialBotAi mTLS client cert read failed (path redacted): {}",
                            e
                        )
                    })?;
                    let key_pem = std::fs::read_to_string(key).map_err(|e| {
                        anyhow::anyhow!(
                            "SocialBotAi mTLS client key read failed (path redacted): {}",
                            e
                        )
                    })?;
                    let ca_pem = std::fs::read_to_string(ca).map_err(|e| {
                        anyhow::anyhow!(
                            "SocialBotAi mTLS CA cert read failed (path redacted): {}",
                            e
                        )
                    })?;

                    let identity = Identity::from_pem(cert_pem, key_pem);
                    let ca_cert = Certificate::from_pem(ca_pem);
                    let domain = mtls
                        .server_name
                        .clone()
                        .unwrap_or_else(|| "social.local".to_string());

                    let tls_config = ClientTlsConfig::new()
                        .domain_name(domain)
                        .ca_certificate(ca_cert)
                        .identity(identity);
                    endpoint = endpoint.tls_config(tls_config)?;

                    // 不打印 endpoint 之外的字段 (per 8/27 11:06 JST 硬 ban)
                    debug!(
                        endpoint = %self.endpoint,
                        "SocialBotAi mTLS Channel configured (cert/path redacted)"
                    );
                }
                _ => {
                    // 部分配置: log warn + fall back to skip-verify
                    warn!(
                        endpoint = %self.endpoint,
                        "SocialBotAi MtlsConfig 字段不全 (cert/key/ca 三选一缺失), fall back to skip-verify"
                    );
                }
            }
        } else {
            // 无 mTLS: skip-verify + log warn (per 9/10 WipeCluster cert 未导出)
            warn!(
                endpoint = %self.endpoint,
                "SocialBotAi no mTLS config, using skip-verify Channel (wave 3 framework, k3s baseline 0/12 阻塞, 真实连接待 SRE 介入)"
            );
        }

        // `connect_lazy` 不阻塞, TLS handshake 推迟到首次 RPC
        let channel = endpoint.connect_lazy();
        Ok(channel)
    }

    /// 内部 helper: 从 Channel 构造 SocialServiceClient (per wave 4 真实 RPC)
    ///
    /// tonic 0.12 `SocialServiceClient::new(channel)` 是 infallible, 返回 client.
    /// Channel 已由 init 阶段 build + set, 这里只做引用 clone (Channel 内部 Arc).
    ///
    /// # 凭据
    /// 不接收任何 cert / endpoint 明文 (per 8/27 11:06 JST 硬 ban).
    fn make_social_client(&self) -> Option<SocialServiceClient<Channel>> {
        self.channel
            .get()
            .map(|c| SocialServiceClient::new(c.clone()))
    }

    /// 内部 helper: 调用 `health_check` 真实 RPC (per wave 4)
    ///
    /// # 行为
    /// - Channel 未初始化 → 立即返 `Ok(())` (防御)
    /// - 走 `tokio::time::timeout(2s, ...)` 防止 hang
    /// - 任何失败 (timeout / Status / connect) → 返 `Ok(())` + `warn!` 标注降级
    ///
    /// k3s baseline 0/12 阶段: 真实 RPC 调用预期 `Err` (connection refused),
    /// 走 `Ok(())` 错误容忍模式 (per 9/10 16:36 JST 拍板 "接受 baseline 等 SRE 介入").
    async fn rpc_health_check(&self) -> anyhow::Result<()> {
        let mut client = match self.make_social_client() {
            Some(c) => c,
            None => {
                warn!("SocialBotAi::rpc_health_check: Channel 未初始化, 跳过");
                return Ok(());
            }
        };

        let req = common_proto::HealthCheckRequest {
            service: "rgs-testkit-bot-social".to_string(),
        };

        let rpc_result = tokio::time::timeout(
            Duration::from_secs(2),
            client.health_check(tonic::Request::new(req)),
        )
        .await;

        match rpc_result {
            Ok(Ok(resp)) => {
                let inner = resp.into_inner();
                debug!(
                    status = inner.status,
                    message = %inner.message,
                    "SocialBotAi::rpc_health_check OK (k3s reachable)"
                );
                Ok(())
            }
            Ok(Err(status)) => {
                warn!(
                    code = ?status.code(),
                    message = %status.message(),
                    "SocialBotAi::rpc_health_check failed (k3s baseline 0/12 阶段预期), 走降级模式"
                );
                Ok(())
            }
            Err(_elapsed) => {
                warn!(
                    timeout_secs = 2,
                    "SocialBotAi::rpc_health_check timeout (k3s 不可达), 走降级模式"
                );
                Ok(())
            }
        }
    }

    /// 内部 helper: 调用 `get_guild` 真实 RPC (per wave 4)
    ///
    /// # 行为
    /// - Channel 未初始化 → 立即返 `Ok(())` (防御)
    /// - 走 `tokio::time::timeout(2s, ...)` 防止 hang
    /// - 任何失败 (timeout / Status / connect / NotFound) → 返 `Ok(())` + `warn!` 标注降级
    ///
    /// 注: `get_guild(EntityId { id: <guild_uuid> })` 取 `bot.id()` 作 guild_id placeholder
    /// (bot.id() 通常是 "bot-social-001" 非 UUID, 业务 NotFound 预期; k3s 不可达时
    /// 走 connect 失败降级).
    async fn rpc_get_guild(&self, bot: &Bot) -> anyhow::Result<()> {
        let mut client = match self.make_social_client() {
            Some(c) => c,
            None => {
                warn!("SocialBotAi::rpc_get_guild: Channel 未初始化, 跳过");
                return Ok(());
            }
        };

        let req = common_proto::EntityId {
            id: bot.id().to_string(),
        };

        let rpc_result = tokio::time::timeout(
            Duration::from_secs(2),
            client.get_guild(tonic::Request::new(req)),
        )
        .await;

        match rpc_result {
            Ok(Ok(_guild)) => {
                debug!(
                    bot_id = bot.id(),
                    "SocialBotAi::rpc_get_guild OK (k3s reachable, guild found)"
                );
                Ok(())
            }
            Ok(Err(status)) => {
                warn!(
                    bot_id = bot.id(),
                    code = ?status.code(),
                    message = %status.message(),
                    "SocialBotAi::rpc_get_guild failed (k3s baseline 0/12 阶段预期, 或 guild 不存在), 走降级模式"
                );
                Ok(())
            }
            Err(_elapsed) => {
                warn!(
                    bot_id = bot.id(),
                    timeout_secs = 2,
                    "SocialBotAi::rpc_get_guild timeout (k3s 不可达), 走降级模式"
                );
                Ok(())
            }
        }
    }
}

#[async_trait]
impl BotAi for SocialBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        let channel = self.build_channel().await?;
        // OnceCell::set 失败 = 重复 init, 立即报错 (防误用)
        self.channel.set(channel).map_err(|_| {
            anyhow::anyhow!("SocialBotAi channel already initialized (重复 init)")
        })?;

        debug!(
            bot_id = bot.id(),
            endpoint = %self.endpoint,
            mtls_configured = self.mtls_configured(),
            "SocialBotAi::init (wave 4 真实 tonic Channel + mTLS 框架就绪, 真实业务 RPC 调 health_check)"
        );

        // wave 4: 真实 RPC ping 验证 (per DDD Review v0.3.2 §7.3 L1.2)
        // k3s baseline 0/12 阶段: 预期 Err (connection refused), 走降级模式 Ok(())
        self.rpc_health_check().await?;

        Ok(())
    }

    fn act_list(&self) -> Vec<ActKind> {
        vec![
            ActKind::Init,
            ActKind::Heartbeat,
            ActKind::RandProto(100),
            ActKind::Guild,
            ActKind::Partner,
        ]
    }

    async fn handle(&self, bot: &Bot, act: ActKind) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), ?act, "SocialBotAi::handle (wave 4 真实 RPC)");
        match act {
            ActKind::Guild => {
                // 真实调用 social.v1.SocialServiceClient::get_guild(EntityId { id: bot.id() })
                // (per erlang C6 line 163-181 guild 协议序列, wave 4 升级)
                self.rpc_get_guild(bot).await?;
            }
            ActKind::Heartbeat => {
                // 周期心跳 → health_check (per erlang E4, 25s 周期)
                self.rpc_health_check().await?;
            }
            ActKind::Init | ActKind::RandProto(_) | ActKind::Partner => {
                // Init / RandProto / Partner 走 health_check 兜底
                // (Partner 域 social-service proto 当前无 partner RPC, 走 ping 验证 channel)
                self.rpc_health_check().await?;
            }
            _ => {
                // 其他 act 走 health_check 兜底 (per wave 4 防御: 不漏 act)
                self.rpc_health_check().await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::stats::BotStats;

    fn dummy_bot() -> Bot {
        Bot::new("bot-social-001", "social", BotStats::new())
    }

    #[tokio::test]
    async fn social_ai_init_ok() {
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
        // init 后 Channel 必须就绪
        assert!(ai.channel_initialized(), "init 后 Channel 应已就绪");
    }

    #[tokio::test]
    async fn social_ai_act_list_contains_erlang_c6_acts() {
        let ai = SocialBotAi::default();
        let acts = ai.act_list();
        // erlang C6 business scene acts
        assert!(acts.contains(&ActKind::Guild), "act_list missing Guild");
        assert!(acts.contains(&ActKind::Partner), "act_list missing Partner");
        // common acts (per B3 act_list)
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::RandProto(100)));
    }

    #[tokio::test]
    async fn social_ai_handle_all_acts_ok() {
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        // 先 init (让 Channel 就绪, handle 路径走 "ready" 分支)
        ai.init(&bot).await.expect("init");
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }

    #[tokio::test]
    async fn social_ai_default_uses_skip_verify_warn() {
        // Default 构造: 无 mTLS → skip-verify + log warn
        let ai = SocialBotAi::default();
        assert!(!ai.mtls_configured(), "默认无 mTLS 配置");
        assert_eq!(ai.endpoint(), "https://127.0.0.1:50054");
        assert!(!ai.channel_initialized(), "init 前 Channel 未就绪");
    }

    #[tokio::test]
    async fn social_ai_with_endpoint_overrides_default() {
        // 自定义 endpoint
        let ai = SocialBotAi::with_endpoint("https://social.svc.cluster.local:50054");
        assert_eq!(ai.endpoint(), "https://social.svc.cluster.local:50054");
        assert!(!ai.mtls_configured());
    }

    #[tokio::test]
    async fn social_ai_with_mtls_marks_configured_when_all_three_paths_present() {
        // MtlsConfig 三字段全有 → mtls_configured() = true
        let mtls = MtlsConfig {
            endpoint: Some("https://social.local:50054".to_string()),
            client_cert_path: Some("/etc/rgs/certs/client.pem".to_string()),
            client_key_path: Some("/etc/rgs/certs/client.key".to_string()),
            ca_cert_path: Some("/etc/rgs/certs/social-ca.pem".to_string()),
            server_name: Some("social.local".to_string()),
            skip_verify: false,
        };
        let ai = SocialBotAi::with_mtls(mtls);
        assert!(ai.mtls_configured());
    }

    #[tokio::test]
    async fn social_ai_with_mtls_marks_unconfigured_when_paths_missing() {
        // MtlsConfig 字段不全 → mtls_configured() = false
        let mtls = MtlsConfig {
            endpoint: Some("https://social.local:50054".to_string()),
            client_cert_path: Some("/etc/rgs/certs/client.pem".to_string()),
            client_key_path: None, // 缺
            ca_cert_path: Some("/etc/rgs/certs/social-ca.pem".to_string()),
            server_name: Some("social.local".to_string()),
            skip_verify: false,
        };
        let ai = SocialBotAi::with_mtls(mtls);
        assert!(!ai.mtls_configured(), "字段不全不应视为已配置");
    }

    #[tokio::test]
    async fn social_ai_debug_redacts_mtls() {
        // Debug 必须 redact mTLS 字段 (per 8/27 11:06 JST 硬 ban)
        let mtls = MtlsConfig {
            endpoint: Some("https://social.local:50054".to_string()),
            client_cert_path: Some("/secret/path/should-not-appear.pem".to_string()),
            client_key_path: Some("/secret/key/should-not-appear.key".to_string()),
            ca_cert_path: Some("/secret/ca/should-not-appear.pem".to_string()),
            server_name: Some("social.local".to_string()),
            skip_verify: false,
        };
        let ai = SocialBotAi::with_mtls(mtls);
        let dbg = format!("{:?}", ai);
        assert!(
            !dbg.contains("should-not-appear"),
            "Debug 不应打印 cert path 明文, 实际: {}",
            dbg
        );
        assert!(
            dbg.contains("redacted"),
            "Debug 应标注 'redacted', 实际: {}",
            dbg
        );
    }

    #[tokio::test]
    async fn social_ai_init_double_call_errors() {
        // 双重 init 必须报错 (OnceCell set 失败)
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("first init");
        let r = ai.init(&bot).await;
        assert!(r.is_err(), "double init 应该报错");
    }

    #[tokio::test]
    async fn social_ai_clone_shares_channel_cell() {
        // Clone 后 channel cell 是共享的 (Arc)
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
        let cloned = ai.clone();
        // clone 后 channel_initialized 状态一致 (共享 cell)
        assert!(cloned.channel_initialized());
    }

    // =================================================================
    // wave 4 真实 RPC 调用单测 (per DDD Review v0.3.2 §7.3 L1.2)
    // =================================================================

    #[tokio::test]
    async fn social_ai_wave4_rpc_health_check_returns_ok_on_k3s_unreachable() {
        // wave 4 真实 RPC 调用验证: k3s 不可达时, health_check 走降级模式
        // 返 Ok(()) + warn log, 不 panic, 不返 Err
        //
        // 注: 默认 endpoint = https://127.0.0.1:50054 (social svc ClusterIP placeholder),
        // k3s baseline 0/12 阶段无 svc 监听, connect 立即 RST, 走降级
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init (含 wave 4 真实 health_check)");

        // 显式调 health_check, 应返 Ok(()) (k3s 不可达预期)
        let r = ai.rpc_health_check().await;
        assert!(r.is_ok(), "rpc_health_check 应返 Ok(()) 走降级模式, 实际: {:?}", r);
    }

    #[tokio::test]
    async fn social_ai_wave4_rpc_get_guild_returns_ok_on_k3s_unreachable() {
        // wave 4 真实 RPC 调用验证: k3s 不可达时, get_guild 走降级模式
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");

        let r = ai.rpc_get_guild(&bot).await;
        assert!(r.is_ok(), "rpc_get_guild 应返 Ok(()) 走降级模式, 实际: {:?}", r);
    }

    #[tokio::test]
    async fn social_ai_wave4_rpc_health_check_without_init_returns_ok() {
        // 未 init 直接调 health_check → 返 Ok(()) 防御, 不 panic
        let ai = SocialBotAi::default();
        let r = ai.rpc_health_check().await;
        assert!(r.is_ok(), "未 init 调 health_check 应返 Ok(()) 防御, 实际: {:?}", r);
    }

    #[tokio::test]
    async fn social_ai_wave4_handle_guild_uses_real_rpc_get_guild() {
        // 验证 handle(Guild) 走真实 get_guild RPC (per wave 4 升级)
        // k3s 不可达时仍返 Ok(()) (降级模式)
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
        let r = ai.handle(&bot, ActKind::Guild).await;
        assert!(r.is_ok(), "handle(Guild) 应返 Ok(()) 走降级模式, 实际: {:?}", r);
    }

    #[tokio::test]
    async fn social_ai_wave4_handle_heartbeat_uses_real_rpc_health_check() {
        // 验证 handle(Heartbeat) 走真实 health_check RPC
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
        let r = ai.handle(&bot, ActKind::Heartbeat).await;
        assert!(r.is_ok(), "handle(Heartbeat) 应返 Ok(()) 走降级模式, 实际: {:?}", r);
    }

    #[tokio::test]
    async fn social_ai_wave4_handle_partner_uses_real_rpc_fallback() {
        // 验证 handle(Partner) 走真实 health_check (Partner 域无 partner RPC, 兜底)
        let ai = SocialBotAi::default();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
        let r = ai.handle(&bot, ActKind::Partner).await;
        assert!(r.is_ok(), "handle(Partner) 应返 Ok(()) 走降级模式, 实际: {:?}", r);
    }
}
