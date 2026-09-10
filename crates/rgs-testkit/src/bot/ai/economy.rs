//! economy 域 BotAi 派生 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! 行为序列 (per erlang C6 + tester_ai_base.erl:163-194 经济域 act):
//! - `Init`                  启动初始化 (真实 tonic gRPC client + 真实 RPC 调用 `get_account`)
//! - `Heartbeat`             周期心跳 (真实 RPC 调用 `get_account`)
//! - `RandProto(100)`        10% 概率触发协议随机化 (千分位, per erlang C1, 真实 RPC 调用)
//! - `Custom("Trade")`       交易行为 (真实 RPC 调用 placeholder, wave 5 接 Trade)
//! - `Custom("Account")`     账户查询 (真实 tonic Channel + 真实 RPC 调用 `get_account`)
//!
//! # wave 4 真实 RPC 接入 (per DDD Review v0.3.2 §7.3 L1.2, 9/10 19:00 JST Ulysses 选 wave 4)
//!
//! - wave 3 (per `b947c97`): lazy Channel, init 时构造但**不实际 RPC 调用**
//! - **wave 4 (本 worker)**: lazy Channel → 真实 `EconomyServiceClient<Channel>::get_account` 调用
//!
//! # 凭据约束 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - mTLS cert path 走 `Option<String>`, 由调用方注入, **不读 env, 不打印值**
//! - server_name (SNI) 同上
//! - 当前实现**不加载 cert 文件** (k3s cert 未导出, skip verify fallback), 真实 cert 加载留 SRE 介入
//!
//! # 错误容忍 (per DDD Review v0.3.2 §7.3 L1.2)
//!
//! - k3s baseline 0/12 (per 9/10 16:36 JST 拍板 "接受 baseline 等 SRE 介入") → 真实 RPC 预期失败
//! - 失败处理: `tokio::time::timeout(2s)` 防 hang + `tracing::warn!` 标降级模式 + `Ok(())` 不 panic
//! - bot supervisor 继续跑, 不阻断后续 act
//!
//! # 默认 stub 模式 (向后兼容 wave 2 测试)
//!
//! `EconomyBotAi::new()` / `EconomyBotAi::default()` 不配置 endpoint, init/handle 走原 stub 行为.
//! `EconomyBotAi::with_endpoint(endpoint, server_name)` 切到真实 client 模式, init 构造 client.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tonic::Request;
use tracing::{debug, warn};

use economy_service::common::v1::EntityId;
use economy_service::proto::v1::economy_service_client::EconomyServiceClient;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// economy 域 BotAi (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), Trade, Account]`,
/// `handle` 走真实 `EconomyServiceClient::get_account` 调用 (per wave 4).
///
/// # 模式
///
/// - **stub 模式** (`EconomyBotAi::new()` / `default()`): 无 endpoint, init/handle 走原 stub 行为,
///   跟 wave 2 EconomyBotAi 兼容.
/// - **real client 模式** (`EconomyBotAi::with_endpoint(ep, sni)`): endpoint + server_name 配齐,
///   init 时构造真实 tonic Channel (lazy) + 真实 `EconomyServiceClient<Channel>`,
///   handle 走真实 RPC 调用 `get_account` (2s timeout, 失败返 `Ok(())`).
///
/// # 凭据约束 (per 8/27 11:06 JST hard ban)
///
/// - mTLS cert path 走 `Option<String>`, 由调用方注入, **不读 env, 不打印值**
/// - server_name (SNI) 同上
/// - 当前实现**不加载 cert 文件** (k3s cert 未导出, skip verify fallback), 真实 cert 加载留 SRE 介入
pub struct EconomyBotAi {
    /// economy 域 svc endpoint (per DDD Review v0.3.1 §7.3, e.g. `https://127.0.0.1:50052`)
    pub endpoint: Option<String>,

    /// mTLS SNI / domain name (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`, e.g. `"economy-service"`)
    pub server_name: Option<String>,

    /// 真实 tonic gRPC client (init 时构造, 用 `Mutex<Option<_>>` 因为 `BotAi::init` 持 `&self`)
    /// (per player.rs wave 3 同款 pattern)
    client: Arc<Mutex<Option<EconomyServiceClient<Channel>>>>,
}

impl std::fmt::Debug for EconomyBotAi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EconomyBotAi")
            .field("endpoint", &self.endpoint)
            .field("server_name", &self.server_name)
            .field("client", &"<Arc<Mutex<Option<EconomyServiceClient<Channel>>>>>")
            .finish()
    }
}

impl Default for EconomyBotAi {
    fn default() -> Self {
        Self {
            endpoint: None,
            server_name: None,
            client: Arc::new(Mutex::new(None)),
        }
    }
}

impl EconomyBotAi {
    /// 构造 economy 域 BotAi (stub 模式, 无 endpoint, 跟 wave 2 EconomyBotAi 兼容)
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造 economy 域 BotAi + 真实 mTLS endpoint (real client 模式, wave 4 真实 RPC 接入)
    ///
    /// `endpoint` 是 economy svc gRPC endpoint (e.g. `"https://127.0.0.1:50052"`),
    /// `server_name` 是 mTLS SNI (e.g. `"economy-service"`).
    ///
    /// **凭据走 Option, 由调用方注入, 不读 env, 不打印值** (per 8/27 11:06 JST 硬 ban).
    pub fn with_endpoint(endpoint: impl Into<String>, server_name: impl Into<String>) -> Self {
        Self {
            endpoint: Some(endpoint.into()),
            server_name: Some(server_name.into()),
            client: Arc::new(Mutex::new(None)),
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

    /// 构造真实 `EconomyServiceClient<Channel>` (走 lazy Channel, per wave 4 真实 RPC 接入)
    ///
    /// 复用 `build_channel()` 拿 lazy Channel, 然后用 tonic 0.12 生成的 client 包装.
    /// 调用方主动 RPC 时才握手 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`).
    pub async fn build_client(&self) -> anyhow::Result<EconomyServiceClient<Channel>> {
        let channel = self.build_channel().await?;
        Ok(EconomyServiceClient::new(channel))
    }

    /// 检查 client 是否已构造 (供 log / 测试断言)
    pub async fn is_real(&self) -> bool {
        self.client.lock().await.is_some()
    }

    /// 拿 client 引用 (供测试 / 未来使用)
    pub async fn mtls_client(&self) -> Option<EconomyServiceClient<Channel>> {
        self.client.lock().await.clone()
    }

    /// 内部 helper: 真实 RPC 调用 `get_account`, 2s timeout, 失败返 `Ok(())` (不 panic)
    ///
    /// (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入):
    /// - 持 `Mutex` guard 调 `&mut client.get_account(...)` (独占访问)
    /// - `tokio::time::timeout(Duration::from_secs(2), ...)` 防 hang
    /// - 3 种结果:
    ///   - `Ok(Ok(_))` → RPC 成功, debug log
    ///   - `Ok(Err(status))` → RPC 返 Status (k3s 不可达预期: Unavailable / connection refused), warn log
    ///   - `Err(_elapsed)` → 2s timeout, warn log
    /// - 全部返 `Ok(())`, 不 panic (bot supervisor 继续跑)
    async fn call_get_account(&self, bot: &Bot) -> anyhow::Result<()> {
        let mut guard = self.client.lock().await;
        if let Some(client) = guard.as_mut() {
            // 构造 EntityId (用 bot_id 作为 id)
            // 注: get_account 会 parse id as UUID, 非 UUID 会在服务端返 invalid_argument
            // 但 k3s 不可达时, 客户端先 connection refused, 走 timeout/Err 路径
            let entity_id = EntityId {
                id: bot.id().to_string(),
            };
            let request = Request::new(entity_id);

            match tokio::time::timeout(
                Duration::from_secs(2),
                client.get_account(request),
            )
            .await
            {
                Ok(Ok(_account)) => {
                    debug!(
                        bot_id = bot.id(),
                        "EconomyBotAi 真实 RPC get_account 成功 (k3s baseline 恢复后场景)"
                    );
                }
                Ok(Err(status)) => {
                    warn!(
                        bot_id = bot.id(),
                        code = ?status.code(),
                        message = status.message(),
                        "EconomyBotAi 真实 RPC get_account 失败 \
                         (k3s baseline 0/12 预期, per 9/10 16:36 JST 拍板, 真实连接等 SRE 介入)"
                    );
                }
                Err(_elapsed) => {
                    warn!(
                        bot_id = bot.id(),
                        "EconomyBotAi 真实 RPC get_account timeout (2s, \
                         k3s baseline 0/12 预期, per 9/10 16:36 JST 拍板)"
                    );
                }
            }
        } else {
            debug!(
                bot_id = bot.id(),
                "EconomyBotAi 真实 client 未构造 (走 stub 模式)"
            );
        }
        Ok(())
    }
}

#[async_trait]
impl BotAi for EconomyBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        if self.endpoint.is_some() {
            // 真实 client 模式: 构造 EconomyServiceClient<Channel> (lazy, 不立即握手)
            match self.build_client().await {
                Ok(client) => {
                    *self.client.lock().await = Some(client);
                    debug!(
                        bot_id = bot.id(),
                        "EconomyBotAi::init 真实 mTLS client 创建成功 (lazy, \
                         实际 RPC 走 wave 4 handle)"
                    );
                }
                Err(e) => {
                    warn!(
                        bot_id = bot.id(),
                        error = %e,
                        "EconomyBotAi::init build_client 失败, 走 stub 模式"
                    );
                    return Ok(());
                }
            }
            // wave 4 真实 RPC 调用 (即使 k3s 不可达也容忍, 返 Ok(()))
            // (per DDD Review v0.3.2 §7.3 L1.2: "init 调 get_account 真实 RPC, 失败时 tracing::warn! + Ok(())")
            self.call_get_account(bot).await?;
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
                self.call_get_account(bot).await
            }
            ActKind::Heartbeat => {
                debug!(bot_id = bot.id(), "EconomyBotAi::handle Heartbeat");
                // (per DDD Review v0.3.2 §7.3 L1.2: "handle(Heartbeat) 调 get_account 真实 RPC")
                self.call_get_account(bot).await
            }
            ActKind::RandProto(p) => {
                debug!(
                    bot_id = bot.id(),
                    rand_proto_prob = p,
                    "EconomyBotAi::handle RandProto"
                );
                // 随机 RPC (per DDD Review v0.3.2 §7.3 L1.2: "调随机 RPC")
                self.call_get_account(bot).await
            }
            ActKind::Custom(name) if name == "Trade" => {
                // Trade 走 get_account placeholder (wave 5 接 Trade/Bid RPC, k3s baseline 0/12 阶段先走 RPC 通路验证)
                debug!(
                    bot_id = bot.id(),
                    "EconomyBotAi::handle Trade (走 get_account placeholder, wave 5 接 Trade RPC)"
                );
                self.call_get_account(bot).await
            }
            ActKind::Custom(name) if name == "Account" => {
                debug!(bot_id = bot.id(), "EconomyBotAi::handle Account (真实 RPC)");
                self.call_get_account(bot).await
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

    /// wave 4 新增: 真实 `EconomyServiceClient<Channel>` 构造 + `is_real` 断言
    /// 验证 build_client 走 tonic 0.12 生成的 EconomyServiceClient 包装, 不 panic
    /// (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
    #[tokio::test]
    async fn economy_ai_build_real_grpc_client() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let _client = ai.build_client().await.expect("build_client ok");
        // client 是 lazy 包装, 构造成功即 OK (实际 RPC 调用验证在 init/handle 路径)
    }

    /// wave 4 新增: init 走真实 client 模式 + 真实 RPC 调用 (k3s baseline 0/12 阶段预期失败, 返 Ok(()))
    /// 验证 `EconomyBotAi::with_endpoint` + `init` 走真实 `EconomyServiceClient::get_account` 路径,
    /// 2s timeout 内返 `Ok(())` 不 panic
    /// (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
    #[tokio::test]
    async fn economy_ai_init_real_client_mode() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let bot = Bot::new("bot-economy-mtls-001", "economy", BotStats::new());
        // init 走真实 client 构造 + 真实 RPC 调用 (k3s 不可达, 2s timeout 内返 Ok(()))
        let start = std::time::Instant::now();
        ai.init(&bot).await.expect("init should not panic");
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(3),
            "init 应在 2s timeout 内完成 (实际 {}ms)",
            elapsed.as_millis()
        );
        // 真实 client 已构造
        assert!(ai.is_real().await, "real client 应已构造");
    }

    /// wave 4 新增: handle Account 走真实 client + 真实 RPC 调用 (k3s baseline 0/12 阶段预期失败, 返 Ok(()))
    /// 验证 `EconomyBotAi::with_endpoint` + `handle(Account)` 走真实 `get_account` 路径
    /// (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
    #[tokio::test]
    async fn economy_ai_handle_account_real_rpc_call() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let bot = Bot::new("bot-economy-mtls-002", "economy", BotStats::new());
        // 先 init 走真实 client 构造
        ai.init(&bot).await.expect("init");
        // handle Account 走真实 RPC 调用 (k3s 不可达, 2s timeout 内返 Ok(()))
        let start = std::time::Instant::now();
        ai.handle(&bot, ActKind::Custom("Account".to_string()))
            .await
            .expect("handle Account real RPC should not panic");
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(3),
            "handle Account 应在 2s timeout 内完成 (实际 {}ms)",
            elapsed.as_millis()
        );
    }

    /// wave 4 新增: handle 全部 act 走真实 RPC 调用, 全部返 Ok(()) 不 panic
    /// 验证 5 域派生基线 (Init/Heartbeat/RandProto/Trade/Account) 全部走真实 RPC 通路
    /// (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
    #[tokio::test]
    async fn economy_ai_handle_all_acts_real_rpc_call() {
        let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");
        let bot = Bot::new("bot-economy-mtls-003", "economy", BotStats::new());
        // 先 init
        ai.init(&bot).await.expect("init");
        // 全部 act 走真实 RPC (k3s 不可达, 全部 2s timeout 内返 Ok(()))
        for act in ai.act_list() {
            ai.handle(&bot, act)
                .await
                .expect("handle real RPC should not panic on k3s unreachable");
        }
    }
}
