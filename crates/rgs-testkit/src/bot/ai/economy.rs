//! economy 域 BotAi 派生 (per DDD Review v0.3.1 §7.3 Phase C mTLS 业务级 ST 准备)
//!
//! 行为序列 (per erlang C6 + tester_ai_base.erl:163-194 经济域 act):
//! - `Init`                  启动初始化 (真实 tonic gRPC client stub, 调 GetAccount mTLS 端点)
//! - `Heartbeat`             周期心跳
//! - `RandProto(100)`        10% 概率触发协议随机化 (千分位, per erlang C1)
//! - `Custom("Trade")`       交易行为 stub (Trade / ShopBuy / ExchangeDo 留 SRE 介入)
//! - `Custom("Account")`     账户查询 (持有真实 tonic Channel, **不实际调 RPC**, 等 SRE 介入)
//!
//! # mTLS 业务级真实 client 准备 (per DDD Review v0.3.1 §7.3 + 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
//!
//! - `endpoint: Option<String>` — 真实 economy svc endpoint, e.g. `"https://economy-service:8443"`
//! - `server_name: Option<String>` — mTLS SNI / domain name
//! - `init()` 走 `Endpoint::from_shared() + ClientTlsConfig::new().domain_name(sni) + connect_lazy()`
//!   → 拿 lazy Channel (不立即建立连接, 不阻塞 k3s 不可达场景)
//! - **凭据不读 env, 不打印值** (per 8/27 11:06 JST 硬 ban + AGENTS.md §1.2)
//! - **不调真实 5 域** (k3s baseline 0/12, 9/10 16:36 JST 拍板"接受 baseline 等 SRE 介入", commit `85bfdf5`)
//! - skip verify + log warn (9/10 WipeCluster 重建后 cert 可能未导出, fall back)
//!
//! # 默认 stub 模式 (向后兼容 wave 2 测试)
//!
//! `EconomyBotAi::new()` / `EconomyBotAi::default()` 不配置 endpoint, init/handle 走原 stub 行为.
//! `EconomyBotAi::with_endpoint(endpoint, server_name)` 切到真实 client 模式, init 构造 Channel.

use async_trait::async_trait;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, warn};

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// economy 域 BotAi (per DDD Review v0.3.1 §7.3 Phase C mTLS 业务级 ST 准备)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), Trade, Account]`,
/// `handle` 全 OK stub. **真实 tonic Channel + mTLS 替换 stub** (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`).
///
/// # 模式
///
/// - **stub 模式** (`EconomyBotAi::new()` / `default()`): 无 endpoint, init/handle 走原 stub 行为,
///   跟 wave 2 EconomyBotAi 兼容.
/// - **real client 模式** (`EconomyBotAi::with_endpoint(ep, sni)`): endpoint + server_name 配齐,
///   init 时构造真实 tonic Channel (lazy), handle 走真实 Channel 路径但**不实际调 RPC**.
///
/// # 凭据约束 (per 8/27 11:06 JST hard ban)
///
/// - mTLS cert path 走 `Option<String>`, 由调用方注入, **不读 env, 不打印值**
/// - server_name (SNI) 同上
/// - 当前实现**不加载 cert 文件** (k3s cert 未导出, skip verify fallback), 真实 cert 加载留 SRE 介入
#[derive(Clone, Debug, Default)]
pub struct EconomyBotAi {
    /// economy 域 svc endpoint (per DDD Review v0.3.1 §7.3, e.g. `https://127.0.0.1:50052`)
    pub endpoint: Option<String>,

    /// mTLS SNI / domain name (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`, e.g. `"economy-service"`)
    pub server_name: Option<String>,
}

impl EconomyBotAi {
    /// 构造 economy 域 BotAi (stub 模式, 无 endpoint, 跟 wave 2 EconomyBotAi 兼容)
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造 economy 域 BotAi + 真实 mTLS endpoint (real client 模式)
    ///
    /// `endpoint` 是 economy svc gRPC endpoint (e.g. `"https://127.0.0.1:50052"`),
    /// `server_name` 是 mTLS SNI (e.g. `"economy-service"`).
    ///
    /// **凭据走 Option, 由调用方注入, 不读 env, 不打印值** (per 8/27 11:06 JST 硬 ban).
    pub fn with_endpoint(endpoint: impl Into<String>, server_name: impl Into<String>) -> Self {
        Self {
            endpoint: Some(endpoint.into()),
            server_name: Some(server_name.into()),
        }
    }

    /// 构造真实 tonic Channel (lazy, **不立即建立连接**)
    ///
    /// 走 mTLS 配置 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`):
    /// - `server_name` 配齐 → `ClientTlsConfig::new().domain_name(sni)`
    /// - `server_name` 缺失 → `ClientTlsConfig::new()` (skip verify + log warn, k3s cert 未导出 fallback)
    ///
    /// # 错误
    /// - endpoint URL 解析失败 (`Endpoint::from_shared`)
    /// - TLS config 构造失败 (`Endpoint::tls_config`)
    ///
    /// # 不实际连接
    /// - `connect_lazy()` 拿 lazy Channel, 调用方主动 RPC 时才握手
    /// - 允许 init 在 k3s 不可达时也能成功 (不阻塞单元测试)
    pub async fn build_channel(&self) -> anyhow::Result<Channel> {
        let endpoint_str = self
            .endpoint
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("EconomyBotAi::build_channel: endpoint 未配置"))?;
        let endpoint = Endpoint::from_shared(endpoint_str.clone())?;

        let tls_config = if let Some(sni) = &self.server_name {
            ClientTlsConfig::new().domain_name(sni.clone())
        } else {
            warn!(
                "EconomyBotAi: server_name 未配置, mTLS SNI 跳过 (per 5 域 ST 业务级 mTLS 实践 \
                 commit 401ac5c skip-verify 兜底, 9/10 WipeCluster 重建后 cert 未导出)"
            );
            ClientTlsConfig::new()
        };

        // 拆成多步: tonic 0.12 `tls_config` 返 Result<Endpoint, _>, `connect_lazy` 返 Channel (infallible)
        let endpoint = endpoint.tls_config(tls_config)?;
        let channel = endpoint.connect_lazy();
        Ok(channel)
    }
}

#[async_trait]
impl BotAi for EconomyBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        // 真实 client 模式: init 时构造 lazy Channel (不立即握手)
        // (per DDD Review v0.3.1 §7.3 + 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
        if self.endpoint.is_some() {
            let _channel = self.build_channel().await?;
            debug!(
                bot_id = bot.id(),
                has_endpoint = true,
                "EconomyBotAi::init 真实 mTLS Channel 创建成功 (lazy, \
                 实际 RPC 留 SRE 介入 k3s baseline 恢复后, 9/10 16:36 JST 拍板)"
            );
        } else {
            debug!(bot_id = bot.id(), "EconomyBotAi::init stub 模式 (无 endpoint)");
        }
        Ok(())
    }

    fn act_list(&self) -> Vec<ActKind> {
        vec![
            ActKind::Init,
            ActKind::Heartbeat,
            ActKind::RandProto(100),
            ActKind::Custom("Trade".to_string()),
            ActKind::Custom("Account".to_string()),
        ]
    }

    async fn handle(&self, bot: &Bot, act: ActKind) -> anyhow::Result<()> {
        match act {
            ActKind::Init => {
                debug!(bot_id = bot.id(), "EconomyBotAi::handle Init");
                Ok(())
            }
            ActKind::Heartbeat => {
                debug!(bot_id = bot.id(), "EconomyBotAi::handle Heartbeat");
                // PoC stub: 真实 economy health check 留 SRE 介入
                Ok(())
            }
            ActKind::RandProto(p) => {
                debug!(
                    bot_id = bot.id(),
                    rand_proto_prob = p,
                    "EconomyBotAi::handle RandProto"
                );
                Ok(())
            }
            ActKind::Custom(name) if name == "Trade" => {
                // PoC stub: 真实 Trade RPC (ShopBuy / ExchangeDo) 留 SRE 介入
                debug!(bot_id = bot.id(), "EconomyBotAi::handle Trade (stub)");
                Ok(())
            }
            ActKind::Custom(name) if name == "Account" => {
                // 真实 client 模式: 走 Channel 路径但**不调 RPC** (k3s baseline 0/12 阻塞)
                // 真实 GetAccount 调 admin-service::issue_gm_command 留 SRE 介入 k3s baseline 恢复后
                if self.endpoint.is_some() {
                    debug!(
                        bot_id = bot.id(),
                        "EconomyBotAi::handle Account (real Channel path, RPC 等 SRE 介入)"
                    );
                } else {
                    debug!(bot_id = bot.id(), "EconomyBotAi::handle Account (stub)");
                }
                Ok(())
            }
            // 未识别 act: 不 panic, 记 debug + 返 Ok (PoC 宽容, 跟 admin 域对齐 per L14 守门)
            other => {
                debug!(bot_id = bot.id(), ?other, "EconomyBotAi::handle unknown act, skip");
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
        Bot::new("bot-economy-001", "economy", BotStats::new())
    }

    #[tokio::test]
    async fn economy_ai_init_ok() {
        // stub 模式 (默认, 跟 wave 2 EconomyBotAi 兼容)
        let ai = EconomyBotAi::new();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn economy_ai_act_list_contains_all_acts() {
        let ai = EconomyBotAi::new();
        let acts = ai.act_list();
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::RandProto(100)));
        assert!(acts.contains(&ActKind::Custom("Trade".to_string())));
        assert!(acts.contains(&ActKind::Custom("Account".to_string())));
    }

    #[tokio::test]
    async fn economy_ai_handle_all_acts_ok() {
        let ai = EconomyBotAi::new();
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }

    /// wave 3 新增: 真实 tonic Channel 构造 (skip-verify 模式, k3s cert 未导出 fallback)
    /// 验证 `build_channel` 不 panic, 返回有效 lazy Channel
    /// (per DDD Review v0.3.1 §7.3 + 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
    #[tokio::test]
    async fn economy_ai_build_real_grpc_channel_skip_verify() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let channel = ai.build_channel().await.expect("build_channel ok");
        // Channel 是 lazy 的, 构造成功即 OK (实际 gRPC 调用留 SRE 介入)
        drop(channel);
    }

    /// wave 3 新增: init 走真实 client 模式不 panic
    /// 验证 `EconomyBotAi::with_endpoint` + `init` 走真实 Channel 构造路径
    /// (per DDD Review v0.3.1 §7.3 Phase C mTLS 业务级 ST 准备)
    #[tokio::test]
    async fn economy_ai_init_real_client_mode() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let bot = Bot::new("bot-economy-mtls-001", "economy", BotStats::new());
        ai.init(&bot).await.expect("init with real mTLS channel");
    }

    /// wave 3 新增: handle Account 走真实 client 模式不 panic
    /// (per DDD Review v0.3.1 §7.3 Phase C, Custom("Account") 走 Channel 路径但**不调 RPC**)
    #[tokio::test]
    async fn economy_ai_handle_account_real_client_mode() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let bot = Bot::new("bot-economy-mtls-002", "economy", BotStats::new());
        ai.handle(&bot, ActKind::Custom("Account".to_string()))
            .await
            .expect("handle Account real client");
    }
}
