//! admin 域 BotAi 派生 (per DDD Review v0.2 §5.1 M2 + M4 + DDD Review v0.3.1 §7.3 Phase C)
//!
//! PoC 行为序列 (per erlang C2 GM 注入 + A1 admin ban 场景):
//! - `Init`           启动初始化 (内部跑 1 个 mTLS GmCommand, 演示真实 tonic Channel, per M4)
//! - `Heartbeat`      周期心跳
//! - `RandProto(100)` 10% 概率触发协议随机化 (千分位, per erlang C1)
//! - `GmCommand`      GM 命令注入 (走 `GmClient::issue` 真实 mTLS 通道, per erlang C2 + M4)
//! - `BanAccount`     admin 封号场景 (走 `GmClient::issue` 真实 mTLS 通道, per erlang A1)
//!
//! 真实 admin mTLS 调用 (走 `tonic::transport::Channel` + 客户端 mTLS 凭据) 已替换
//! stub. 当前 **k3s baseline 0/12 阶段** (per 9/10 16:36 JST 拍板 "接受 baseline
//! 等 SRE 介入"), 真实 RPC 必失败, 但 client 框架已就位 + 返 `Ok + GmResponse
//! { ok: false, error }` 不 panic.
//!
//! # 强约束 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - 凭据 (mTLS cert path) 走 `Option<String>` 配置, **不读 env, 不打印值**
//! - GM 指令不打印明文, 只 `cmd_len` 长度 (per DDD Review v0.2 §3 erlang C2 中文指令)
//! - k3s baseline 0/12 阶段走 `with_skip_verify(true)`, cert 未导出 fallback,
//!   1 行 warn (不打 cert path / endpoint 内容)

use async_trait::async_trait;
use tracing::debug;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::gm::GmClient;
use crate::bot::Bot;

/// admin 域 BotAi (per DDD Review v0.2 §5.1 M2 + M4 + DDD Review v0.3.1 §7.3)
///
/// 6 派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), GmCommand, BanAccount]`,
/// `handle` 走 `GmClient::issue` 真实 mTLS (k3s baseline 0/12 阶段必失败, 返
/// `GmResponse { ok: false, error }` 不 panic).
///
/// # GM 注入链路 (per DDD Review v0.2 §5.1 M4 + erlang C2 + DDD Review v0.3.1 §7.3)
///
/// `AdminBotAi` 内部持 1 个 `GmClient` (默认 `GmClient::with_endpoint("https://127.0.0.1:50055")
/// .with_skip_verify(true)`, 走 lazy tonic Channel), 演示完整 GM 注入链路:
/// `Bot` → `BotAi::handle(GmCommand)` → `GmClient::issue` → (lazy) `tonic Channel`
/// → (k3s baseline 0/12 阶段) connection refused → `Ok(GmResponse { ok: false, error })`.
#[derive(Clone, Debug)]
pub struct AdminBotAi {
    /// GM 客户端 (per DDD Review v0.2 §5.1 M4 + v0.3.1 §7.3)
    ///
    /// 字段保留为 `Option`, 默认 `None` 时构造默认 stub `GmClient`. 调用方
    /// 可在 `AdminBotAi::with_gm_client(...)` 注入真实端点.
    gm_client: Option<GmClient>,
}

impl Default for AdminBotAi {
    fn default() -> Self {
        // 默认 endpoint = https://127.0.0.1:50055 (admin-service default port, per
        // crates/admin-service/src/main.rs:62) + skip_verify (k3s baseline 0/12 阶段,
        // per 9/10 16:36 JST 拍板)
        Self {
            gm_client: Some(
                GmClient::new("https://placeholder:8443")
                    .with_endpoint("https://127.0.0.1:50055")
                    .with_skip_verify(true),
            ),
        }
    }
}

impl AdminBotAi {
    /// 构造 admin 域 BotAi (用默认 endpoint + skip verify, 走真实 lazy mTLS Channel)
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造 admin 域 BotAi + 注入自定义 GmClient (真实 mTLS 配置走 Option, 不读 env)
    pub fn with_gm_client(gm_client: GmClient) -> Self {
        Self {
            gm_client: Some(gm_client),
        }
    }

    /// 拿内部 GmClient 引用 (供集成测试断言 channel 状态, per DDD Review v0.3.1 §7.3)
    ///
    /// 默认 `GmClient` 应持真实 lazy mTLS Channel (per wave 3 升级). 测试用
    /// `bot_admin_real_grpc_client_init` + `bot_admin_5_acts_layout_with_mtls_channel`
    /// 验证 channel 存在 + endpoint + skip_verify 配置正确.
    pub fn gm_client(&self) -> Option<&GmClient> {
        self.gm_client.as_ref()
    }

    /// 内部 helper: 跑 1 个真实 RPC GM 命令 (wave 4, per DDD Review v0.3.2 §7.3 L1.2)
    ///
    /// **wave 4 升级** — 走 `GmClient::issue_real` 真实 admin proto RPC 调用
    /// (admin_service_client::AdminServiceClient<Channel>::ban_account(...))
    /// 而不是 wave 3 的 `issue` 模拟调用. k3s baseline 0/12 阶段, 真实 RPC
    /// **会失败** (connection refused), 但调用通路已就位 + 2s timeout + 不
    /// panic + 走 `Ok(())` 错误容忍模式.
    ///
    /// 返 `Ok(())` 即使 RPC 失败, 让 bot supervisor 决定重试/掉线.
    async fn issue_gm_real(&self, bot: &Bot, cmd: &str) -> anyhow::Result<()> {
        if let Some(gm) = &self.gm_client {
            // PoC: 不打印 cmd 明文 (中文 + 凭据走 Option, per 8/27 11:06 JST 硬 ban)
            // 只打 cmd_len + bot_id, 供调试可见但不泄露
            //
            // 走 `issue_real` 而非 `issue`: issue_real 真实调用 admin proto
            // RPC, 走真实 tonic Channel 通路, 失败时返 GmResponse { ok: false }.
            let resp = gm.issue_real(cmd).await?;
            debug!(
                target: "rgs_testkit::bot::ai::admin",
                bot_id = bot.id(),
                cmd_len = cmd.len(),
                gm_ok = resp.ok,
                has_error = resp.error.is_some(),
                "AdminBotAi GM real RPC issue (k3s baseline 0/12: gm_ok=false 预期)"
            );
        } else {
            debug!(
                target: "rgs_testkit::bot::ai::admin",
                bot_id = bot.id(),
                "AdminBotAi GM client not configured, skip"
            );
        }
        Ok(())
    }
}

#[async_trait]
impl BotAi for AdminBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(target: "rgs_testkit::bot::ai::admin", bot_id = bot.id(), "AdminBotAi::init");
        // 演示 GM 注入链路: init 阶段跑 1 个真实 RPC GmCommand (per M4 + v0.3.2 §7.3 wave 4)
        // wave 4 升级: 走 issue_gm_real (真实 admin proto RPC 调用) 而非 issue_gm_stub
        self.issue_gm_real(bot, "设等级 1").await?;
        Ok(())
    }

    fn act_list(&self) -> Vec<ActKind> {
        // 5 acts: Init / Heartbeat / RandProto(100) / GmCommand / BanAccount
        // - Init + Heartbeat + RandProto: 跟 player 域对齐 (per erlang B3 + C1)
        // - GmCommand: per erlang C2 + DDD Review v0.2 §5.1 M4
        // - BanAccount: per erlang A1 admin ban 场景 (admin 域特化)
        vec![
            ActKind::Init,
            ActKind::Heartbeat,
            ActKind::RandProto(100),
            ActKind::Custom("GmCommand".to_string()),
            ActKind::Custom("BanAccount".to_string()),
        ]
    }

    async fn handle(&self, bot: &Bot, act: ActKind) -> anyhow::Result<()> {
        match act {
            ActKind::Init => {
                debug!(target: "rgs_testkit::bot::ai::admin", bot_id = bot.id(), "AdminBotAi::handle Init");
                Ok(())
            }
            ActKind::Heartbeat => {
                debug!(target: "rgs_testkit::bot::ai::admin", bot_id = bot.id(), "AdminBotAi::handle Heartbeat");
                // PoC stub: 真实 admin health check 走 mTLS lazy channel (k3s baseline 0/12 阶段失败)
                Ok(())
            }
            ActKind::RandProto(p) => {
                debug!(target: "rgs_testkit::bot::ai::admin", bot_id = bot.id(), rand_proto_prob = p, "AdminBotAi::handle RandProto");
                Ok(())
            }
            ActKind::Custom(name) if name == "GmCommand" => {
                // 演示 GM 注入链路 (per DDD Review v0.2 §5.1 M4 + v0.3.1 §7.3 + v0.3.2 §7.3 wave 4)
                // wave 4 升级: 走 issue_gm_real 真实 RPC 调用
                self.issue_gm_real(bot, "加经验 100").await
            }
            ActKind::Custom(name) if name == "BanAccount" => {
                // admin ban 场景 (per erlang A1) - 通过 GM 通道下 ban 指令
                debug!(target: "rgs_testkit::bot::ai::admin", bot_id = bot.id(), "AdminBotAi::handle BanAccount via GM");
                // wave 4 升级: 走 issue_gm_real 真实 RPC 调用 (admin proto ban_account)
                self.issue_gm_real(bot, "ban_account 3600 违规").await
            }
            // 未识别 act: 不 panic, 记 debug + 返 Ok (PoC 宽容)
            other => {
                debug!(target: "rgs_testkit::bot::ai::admin", bot_id = bot.id(), ?other, "AdminBotAi::handle unknown act, skip");
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
        Bot::new("bot-admin-001", "admin", BotStats::new())
    }

    #[tokio::test]
    async fn admin_ai_init_ok() {
        let ai = AdminBotAi::new();
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn admin_ai_act_list_contains_5_acts() {
        let ai = AdminBotAi::new();
        let acts = ai.act_list();
        assert_eq!(acts.len(), 5, "act_list 长度应为 5");
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::RandProto(100)));
        assert!(acts.contains(&ActKind::Custom("GmCommand".to_string())));
        assert!(acts.contains(&ActKind::Custom("BanAccount".to_string())));
    }

    #[tokio::test]
    async fn admin_ai_handle_all_acts_ok() {
        let ai = AdminBotAi::new();
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle ok");
        }
    }

    #[tokio::test]
    async fn admin_ai_with_gm_client_works() {
        let gm = GmClient::new("https://placeholder:8443")
            .with_endpoint("https://admin-staging:8443")
            .with_skip_verify(true);
        let ai = AdminBotAi::with_gm_client(gm);
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init with custom gm");
    }

    #[tokio::test]
    async fn admin_ai_default_has_real_mtls_channel() {
        // wave 3 升级验证: 默认 AdminBotAi 持真实 lazy mTLS Channel
        // 注: tonic 0.12 connect_lazy 需要 tokio runtime (hyper-util executor),
        // 所以用 #[tokio::test] 而不是 #[test]
        let ai = AdminBotAi::new();
        let gm = ai.gm_client().expect("default gm_client");
        assert!(gm.channel().is_some(), "默认 GmClient 应建 lazy mTLS Channel");
        assert!(gm.skip_verify(), "k3s baseline 0/12 阶段默认 skip verify");
        assert_eq!(gm.endpoint(), Some("https://127.0.0.1:50055"));
    }
}
