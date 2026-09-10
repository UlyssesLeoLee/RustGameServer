//! admin 域 BotAi 派生 (per DDD Review v0.2 §5.1 M2 + M4)
//!
//! PoC 行为序列 (per erlang C2 GM 注入 + A1 admin ban 场景):
//! - `Init`           启动初始化 (内部跑 1 个 stub GmCommand, 演示 GM 链路, per M4)
//! - `Heartbeat`      周期心跳
//! - `RandProto(100)` 10% 概率触发协议随机化 (千分位, per erlang C1)
//! - `GmCommand`      GM 命令注入 (走 `GmClient::issue` stub, per erlang C2 + M4)
//! - `BanAccount`     admin 封号场景 (走 `GmClient::issue` stub, per erlang A1)
//!
//! 真实 admin mTLS 调用 (走 `admin-service::issue_gm_command` + `BanAccount` 等
//! RPC) 留 wave 3 接. 本 PoC 走 `GmClient` stub, 立即返回 `Ok(GmResponse { ok: true, .. })`.
//!
//! # 强约束 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - 凭据 (mTLS cert path) 走 `Option<String>` 配置, **不读 env, 不打印值**
//! - GM 指令不打印明文, 只 `cmd_len` 长度 (per DDD Review v0.2 §3 erlang C2 中文指令)

use async_trait::async_trait;
use tracing::debug;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::gm::GmClient;
use crate::bot::Bot;

/// admin 域 BotAi (per DDD Review v0.2 §5.1 M2 + M4)
///
/// 6 派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), GmCommand, BanAccount]`,
/// `handle` 全 OK stub (走 `GmClient::issue`). 真实 admin mTLS gRPC 调用留 wave 3.
///
/// # GM 注入链路 (per DDD Review v0.2 §5.1 M4 + erlang C2)
///
/// `AdminBotAi` 内部持 1 个 `GmClient` (走 `GmClient::new(admin_endpoint)`),
/// 演示完整 GM 注入链路: `Bot` → `BotAi::handle(GmCommand)` → `GmClient::issue` →
///
/// (未来) `admin-service::issue_gm_command` mTLS gRPC. 当前 stub 立即返回 `Ok`,
/// 真实连接留待 wave 3 worker 接 admin mTLS.
#[derive(Clone, Debug)]
pub struct AdminBotAi {
    /// GM 客户端 (per DDD Review v0.2 §5.1 M4)
    ///
    /// 字段保留为 `Option`, 默认 `None` 时构造默认 stub `GmClient`. 调用方
    /// 可在 `AdminBotAi::with_gm_client(...)` 注入真实 (future) 端点.
    gm_client: Option<GmClient>,
}

impl Default for AdminBotAi {
    fn default() -> Self {
        // 默认 endpoint = admin-service:8443 (跟 player 域 bot_smoke.rs 一致)
        Self {
            gm_client: Some(GmClient::new("https://admin-service:8443")),
        }
    }
}

impl AdminBotAi {
    /// 构造 admin 域 BotAi (用默认 endpoint, 走 GmClient stub)
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造 admin 域 BotAi + 注入自定义 GmClient (真实 mTLS 配置走 Option, 不读 env)
    pub fn with_gm_client(gm_client: GmClient) -> Self {
        Self {
            gm_client: Some(gm_client),
        }
    }

    /// 内部 helper: 跑 1 个 stub GM 命令 (用于 init 阶段演示 GM 链路)
    async fn issue_gm_stub(&self, bot: &Bot, cmd: &str) -> anyhow::Result<()> {
        if let Some(gm) = &self.gm_client {
            // PoC: 不打印 cmd 明文 (中文 + 凭据走 Option, per 8/27 11:06 JST 硬 ban)
            // 只打 cmd_len + bot_id, 供调试可见但不泄露
            let resp = gm.issue(cmd).await?;
            debug!(
                bot_id = bot.id(),
                cmd_len = cmd.len(),
                gm_ok = resp.ok,
                "AdminBotAi GM stub issue"
            );
        } else {
            debug!(bot_id = bot.id(), "AdminBotAi GM client not configured, skip");
        }
        Ok(())
    }
}

#[async_trait]
impl BotAi for AdminBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "AdminBotAi::init");
        // 演示 GM 注入链路: init 阶段跑 1 个 stub GmCommand (per M4)
        self.issue_gm_stub(bot, "设等级 1").await?;
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
                debug!(bot_id = bot.id(), "AdminBotAi::handle Init");
                Ok(())
            }
            ActKind::Heartbeat => {
                debug!(bot_id = bot.id(), "AdminBotAi::handle Heartbeat");
                // PoC stub: 真实 admin health check 留 wave 3
                Ok(())
            }
            ActKind::RandProto(p) => {
                debug!(bot_id = bot.id(), rand_proto_prob = p, "AdminBotAi::handle RandProto");
                Ok(())
            }
            ActKind::Custom(name) if name == "GmCommand" => {
                // 演示 GM 注入链路 (per DDD Review v0.2 §5.1 M4)
                self.issue_gm_stub(bot, "加经验 100").await
            }
            ActKind::Custom(name) if name == "BanAccount" => {
                // admin ban 场景 (per erlang A1) - 通过 GM 通道下 ban 指令
                debug!(bot_id = bot.id(), "AdminBotAi::handle BanAccount via GM");
                self.issue_gm_stub(bot, "ban_account 3600 违规").await
            }
            // 未识别 act: 不 panic, 记 debug + 返 Ok (PoC 宽容)
            other => {
                debug!(bot_id = bot.id(), ?other, "AdminBotAi::handle unknown act, skip");
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
        let gm = GmClient::new("https://admin-staging:8443");
        let ai = AdminBotAi::with_gm_client(gm);
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init with custom gm");
    }
}
