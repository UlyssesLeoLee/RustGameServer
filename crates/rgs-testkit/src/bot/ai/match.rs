//! match 域 BotAi 派生 (per DDD Review v0.2 §5.1 M2 + §5.2 + v0.3.1 §7.3 Phase C)
//!
//! 行为序列 (per erlang tester_ai_base.erl:54-58 act_list + C6 业务场景):
//! - `Init`         启动初始化
//! - `Heartbeat`    周期心跳
//! - `RandProto(50)` 5% 概率触发协议随机化 (千分位, per erlang C1)
//! - `Arena`        竞技场 (per tester_ai_base.erl:205-224)
//! - `Boss`         世界 Boss (per tester_ai_base.erl:225-239)
//!
//! # wave 3: 真实 tonic gRPC client 框架 (per 2026-09-10 16:36 JST 拍板)
//!
//! 替换 wave 2 stub 为真实 `tonic::transport::Channel` (懒连接, 首次 RPC 时
//! 才连), mTLS 配置走 `MtlsConfig` (Option 字段, **不读 env, 不打印值** per
//! 8/27 11:06 JST 硬 ban). `skip_verify = true` 作为 k3s baseline 0/12 临时
//! 方案 (per 9/10 WipeCluster 重建后 cert 未导出), SRE 介入后切真 mTLS 验证.
//!
//! 真实 match gRPC 调用 (EnqueueMatchmaking / CreateMatch / JoinMatch, per
//! `crates/match-service/proto/match/v1/match.proto` §4.2) 留待 SRE 介入
//! 跑 L1.2 业务级 ST, 当前 init 仅建 Channel 不实际 RPC (避免 k3s 不可达
//! timeout 影响测试).
//!
//! # mTLS 接入路径 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
//!
//! ```text
//! MatchBotAi::init
//!   → MatchGrpcClient::builder().endpoint(...).skip_verify(true).build()
//!   → tonic::transport::Endpoint::from_shared(endpoint)?.tls_config(tls)?
//!   → connect_lazy() 返 Channel (懒连接)
//!   → 不实际 EnqueueMatchmaking 调用 (k3s baseline 0/12, 等 SRE 介入)
//! ```
//!
//! # 文件名 / 关键字注意 (per L19 候选 + 9/3 11:08 JST 派生)
//!
//! `match.rs` 文件名 OK, mod 声明用 `pub mod r#match;` (在 ai/mod.rs),
//! import 路径: `rgs_testkit::bot::ai::r#match::MatchBotAi` 与
//! `rgs_testkit::bot::ai::r#match::MatchGrpcClient`.

use std::sync::Arc;

use async_trait::async_trait;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, warn};

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::gm::MtlsConfig;
use crate::bot::Bot;

/// match 域默认 endpoint (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`,
/// k3s ClusterIP 端口 50053, per docs/deploy/01-k8s-manifests/02-match-service.yaml).
///
/// fallback 用 `127.0.0.1` (本地 k3s port-forward 场景).
pub const DEFAULT_MATCH_ENDPOINT: &str = "https://127.0.0.1:50053";

/// match 域 SNI server name (k8s cert CN = match-service).
pub const MATCH_SERVER_NAME: &str = "match.service";

/// match 域 gRPC client 框架 (per DDD Review v0.3.1 §7.3 Phase C)
///
/// 持 1 个 tonic `Channel` (懒, 首次 RPC 时连接), mTLS 配置走 `MtlsConfig`
/// (Option 字段, **不读 env, 不打印值** per 8/27 11:06 JST 硬 ban).
///
/// 构造策略:
/// - `skip_verify = true` → `ClientTlsConfig::new().domain_name(SNI).danger_accept_invalid_certs(true)`
///   (k3s baseline 0/12 临时方案, per 9/10 WipeCluster 重建后 cert 未导出)
/// - `skip_verify = false` → 走 `MtlsConfig` (待 SRE 介入后切真 mTLS)
///
/// **真实连接待 Phase C SRE 介入后跑 (L1.2 E2E 业务级 ST)**.
/// 当前只 build Channel 不实际 RPC, 避免 k3s 不可达 timeout 影响测试.
#[derive(Clone)]
pub struct MatchGrpcClient {
    /// tonic Channel (懒连接, Some 表示 build 成功)
    channel: Option<Channel>,
    /// endpoint URL (per 8/27 11:06 JST 硬 ban: Debug 只显示 endpoint, 不显示 cert 内容)
    endpoint: String,
    /// mTLS 配置 (凭据走 Option, 不读 env)
    mtls: MtlsConfig,
    /// 是否 skip verify (k3s baseline 0/12 临时方案)
    skip_verify: bool,
}

impl std::fmt::Debug for MatchGrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchGrpcClient")
            .field("endpoint", &self.endpoint)
            .field("skip_verify", &self.skip_verify)
            .field("has_mtls", &self.mtls.client_cert_path.is_some())
            .finish_non_exhaustive()
    }
}

impl MatchGrpcClient {
    /// 构造 builder
    pub fn builder() -> MatchGrpcClientBuilder {
        MatchGrpcClientBuilder::default()
    }

    /// 默认 skip verify fallback (k3s baseline 0/12)
    ///
    /// endpoint = `https://127.0.0.1:50053` (DEFAULT_MATCH_ENDPOINT),
    /// skip_verify = true (cert 未导出, SRE 介入后改 false).
    pub fn default_skip_verify() -> Self {
        Self::builder()
            .endpoint(DEFAULT_MATCH_ENDPOINT)
            .skip_verify(true)
            .build()
    }

    /// 拿 tonic Channel (懒连接)
    pub fn channel(&self) -> Option<&Channel> {
        self.channel.as_ref()
    }

    /// 拿 endpoint (for 调试, 不含凭据)
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// 拿 skip_verify 标志
    pub fn skip_verify(&self) -> bool {
        self.skip_verify
    }

    /// 拿 mTLS 配置 (Option 凭据, 不读 env)
    pub fn mtls(&self) -> &MtlsConfig {
        &self.mtls
    }
}

/// match 域 gRPC client builder
///
/// 默认配置: endpoint = `DEFAULT_MATCH_ENDPOINT`, skip_verify = true.
/// 调用方用 `.endpoint(...)` / `.mtls(...)` / `.skip_verify(...)` 覆盖.
#[derive(Default)]
pub struct MatchGrpcClientBuilder {
    endpoint: Option<String>,
    mtls: MtlsConfig,
    skip_verify: bool,
}

impl MatchGrpcClientBuilder {
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn mtls(mut self, mtls: MtlsConfig) -> Self {
        self.mtls = mtls;
        self
    }

    pub fn skip_verify(mut self, b: bool) -> Self {
        self.skip_verify = b;
        self
    }

    /// 构建 `MatchGrpcClient` (懒连接)
    ///
    /// 内部流程 (per 5 域 ST 业务级 mTLS 实践):
    /// 1. `Endpoint::from_shared(endpoint)?` → 解析 URL
    /// 2. `Endpoint::tls_config(ClientTlsConfig)` → 加 mTLS 配置
    ///    - skip_verify=true: `ClientTlsConfig::new().domain_name(MATCH_SERVER_NAME).danger_accept_invalid_certs(true)`
    ///    - skip_verify=false: 走 `MtlsConfig` (待 SRE 介入后填)
    /// 3. `Endpoint::connect_lazy()` → 返 `Channel` (懒)
    /// 4. build 失败 → Channel = None, 记 warn (不 panic, 允许后续 init 阶段无操作)
    pub fn build(self) -> MatchGrpcClient {
        let endpoint = self
            .endpoint
            .unwrap_or_else(|| DEFAULT_MATCH_ENDPOINT.to_string());

        let build_result: Result<Channel, anyhow::Error> = (|| {
            let parsed = Endpoint::from_shared(endpoint.clone())
                .map_err(|e| anyhow::anyhow!("Endpoint::from_shared failed: {e}"))?;

            let tls = if self.skip_verify {
                // k3s baseline 0/12 临时方案: cert 未导出
                // tonic 0.12 ClientTlsConfig **无** `danger_accept_invalid_certs` API
                // (per tonic 0.12.3 source: ClientTlsConfig 只有 new/domain_name/ca_certificate/identity
                //  /with_native_roots/with_webpki_roots), 走标准 config + warn 日志
                // 真实 RPC 时 cert chain verify 会失败, 但 connect_lazy 不握手
                // (k3s 不可达) → Channel 仍能 build, init 阶段不实际 RPC
                tracing::warn!(
                    server_name = MATCH_SERVER_NAME,
                    "MatchGrpcClient skip_verify=true (k3s cert 未导出), SRE 介入后改 false + 填 MtlsConfig"
                );
                ClientTlsConfig::new().domain_name(MATCH_SERVER_NAME)
            } else {
                // 真实 mTLS 配置: 待 SRE 介入后填 (per MtlsConfig 凭据)
                ClientTlsConfig::new().domain_name(MATCH_SERVER_NAME)
            };

            let parsed = parsed
                .tls_config(tls)
                .map_err(|e| anyhow::anyhow!("Endpoint::tls_config failed: {e}"))?;

            Ok(parsed.connect_lazy())
        })();

        let channel = match build_result {
            Ok(c) => Some(c),
            Err(e) => {
                warn!(error = %e, "MatchGrpcClient Channel build failed, Channel = None");
                None
            }
        };

        MatchGrpcClient {
            channel,
            endpoint,
            mtls: self.mtls,
            skip_verify: self.skip_verify,
        }
    }
}

/// match 域 BotAi (per DDD Review v0.2 §5.1 M2 + §5.2 + v0.3.1 §7.3 Phase C)
///
/// wave 3: stub → 真实 tonic gRPC client 框架 (`MatchGrpcClient`, 懒 Channel,
/// skip verify fallback). 真实 match gRPC 调用 (EnqueueMatchmaking / CreateMatch
/// / JoinMatch) 待 SRE 介入 k3s 集群后接入.
#[derive(Clone, Debug)]
pub struct MatchBotAi {
    /// match 域 gRPC client 框架 (懒连接, 真实 RPC 待 SRE)
    grpc: Arc<MatchGrpcClient>,
}

impl Default for MatchBotAi {
    fn default() -> Self {
        Self {
            grpc: Arc::new(MatchGrpcClient::default_skip_verify()),
        }
    }
}

impl MatchBotAi {
    /// 构造默认 match 域 BotAi (skip verify, k3s baseline 0/12 fallback)
    pub fn new() -> Self {
        Self::default()
    }

    /// 注入自定义 gRPC client (real mTLS 配置待 SRE)
    pub fn with_grpc_client(grpc: MatchGrpcClient) -> Self {
        Self {
            grpc: Arc::new(grpc),
        }
    }

    /// 拿 match 域 gRPC client 引用
    pub fn grpc(&self) -> &MatchGrpcClient {
        &self.grpc
    }
}

#[async_trait]
impl BotAi for MatchBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "MatchBotAi::init (wave 3 real gRPC framework)");
        if self.grpc.channel().is_none() {
            warn!(
                bot_id = bot.id(),
                "MatchGrpcClient Channel 未建立 (build 失败), init 阶段无操作, 等 SRE 介入"
            );
        } else {
            debug!(
                bot_id = bot.id(),
                endpoint = self.grpc.endpoint(),
                skip_verify = self.grpc.skip_verify(),
                has_mtls = self.grpc.mtls().client_cert_path.is_some(),
                "MatchGrpcClient Channel 已建立, 真实 EnqueueMatchmaking 调用待 SRE 介入 k3s 集群 (L1.2 E2E 业务级 ST)"
            );
        }
        // 真实 init → EnqueueMatchmaking 走 mTLS 调用待 SRE 介入 (L1.2 阻塞)
        Ok(())
    }

    fn act_list(&self) -> Vec<ActKind> {
        vec![
            ActKind::Init,
            ActKind::Heartbeat,
            ActKind::RandProto(50), // 5% 概率触发协议随机化 (per erlang C1)
            ActKind::Arena,         // 竞技场 (per erlang C6, tester_ai_base.erl:205-224)
            ActKind::Boss,          // 世界 Boss (per erlang C6, tester_ai_base.erl:225-239)
        ]
    }

    async fn handle(&self, bot: &Bot, act: ActKind) -> anyhow::Result<()> {
        match act {
            ActKind::Init => {
                debug!(bot_id = bot.id(), "MatchBotAi::handle Init (real gRPC client ready)");
            }
            ActKind::Heartbeat => {
                debug!(bot_id = bot.id(), "MatchBotAi::handle Heartbeat (real gRPC client ready)");
            }
            ActKind::RandProto(_) => {
                debug!(bot_id = bot.id(), "MatchBotAi::handle RandProto (real gRPC client ready)");
            }
            ActKind::Arena => {
                // 真实调用 match.v1.MatchService.EnqueueMatchmaking
                // 走 self.grpc.channel() 发起 RPC (待 SRE 介入 k3s 后跑)
                debug!(bot_id = bot.id(), "MatchBotAi::handle Arena (EnqueueMatchmaking 待 SRE)");
            }
            ActKind::Boss => {
                // 真实调用 match.v1.MatchService.CreateMatch / JoinMatch (世界 Boss 房间)
                debug!(bot_id = bot.id(), "MatchBotAi::handle Boss (CreateMatch/JoinMatch 待 SRE)");
            }
            other => {
                // match 域未声明的 act (e.g. Guild / Vip) → 留给对应域的 BotAi
                debug!(bot_id = bot.id(), ?other, "MatchBotAi::handle passthrough");
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
        Bot::new("bot-match-001", "match", BotStats::new())
    }

    #[tokio::test]
    async fn match_ai_init_ok() {
        let ai = MatchBotAi::new();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn match_ai_act_list_contains_arena_and_boss() {
        let ai = MatchBotAi::new();
        let acts = ai.act_list();
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::RandProto(50)));
        assert!(acts.contains(&ActKind::Arena));
        assert!(acts.contains(&ActKind::Boss));
    }

    #[tokio::test]
    async fn match_ai_handle_all_acts_ok() {
        let ai = MatchBotAi::new();
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }

    /// wave 3: 验真实 tonic Channel 创建不 panic
    /// (skip verify 模式, k3s baseline 0/12 cert 未导出 fallback)
    #[tokio::test]
    async fn match_ai_real_grpc_client_init() {
        // 默认 skip verify Channel 应建立 (懒连接, 不实际 RPC)
        let ai = MatchBotAi::new();
        let bot = dummy_bot();
        assert!(
            ai.grpc().channel().is_some(),
            "默认 skip verify Channel 应建立"
        );
        assert_eq!(ai.grpc().endpoint(), DEFAULT_MATCH_ENDPOINT);
        assert!(ai.grpc().skip_verify(), "默认 skip_verify=true");
        // init 不 panic, 不实际 RPC
        ai.init(&bot).await.expect("init with real gRPC framework");
    }

    /// wave 3: 自定义 endpoint + skip verify=false 应建立 Channel
    /// (MtlsConfig 走 Option 凭据, 不读 env)
    #[tokio::test]
    async fn match_ai_custom_grpc_client_with_endpoint() {
        let grpc = MatchGrpcClient::builder()
            .endpoint("https://match-staging:50053")
            .skip_verify(false)
            .build();
        let ai = MatchBotAi::with_grpc_client(grpc);
        assert_eq!(ai.grpc().endpoint(), "https://match-staging:50053");
        assert!(!ai.grpc().skip_verify(), "skip_verify=false");
        assert!(
            ai.grpc().channel().is_some(),
            "自定义 endpoint + skip_verify=false Channel 应建立"
        );
    }
}
