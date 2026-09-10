//! social 域 BotAi 派生 (per DDD Review v0.2 §5.1 M2 + 5 域 BotAi 派生)
//!
//! 模拟 erlang C6 业务场景 act (per tester_ai_base.erl:163-181 guild / 261-278 partner):
//! - `Init`        启动初始化 (调 1 个 stub GetFriendList)
//! - `Heartbeat`   周期心跳
//! - `RandProto(100)` 10% 概率触发协议随机化 (千分位, per erlang C1)
//! - `Guild`       工会操作 (per erlang C6 line 163-181)
//! - `Partner`     伙伴操作 (per erlang C6 line 261-278)
//!
//! 真实 social gRPC 调用 (GetGuild / CreateGuild / JoinGuild / ...) 留 wave 3 接.

use async_trait::async_trait;
use tracing::debug;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// social 域 BotAi (per DDD Review v0.2 §5.1 M2 + 5 域 BotAi 派生)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), Guild, Partner]`,
/// `handle` 全 OK stub. 真实 social gRPC 调用 (GetGuild / CreateGuild /
/// JoinGuild / PromoteMember / LeaveGuild / DissolveGuild) 走 wave 3.
#[derive(Clone, Debug, Default)]
pub struct SocialBotAi;

#[async_trait]
impl BotAi for SocialBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "SocialBotAi::init (stub GetFriendList)");
        // PoC stub: 真实 GetFriendList gRPC 调用留 wave 3
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
        debug!(bot_id = bot.id(), ?act, "SocialBotAi::handle");
        // PoC stub: 真实 social gRPC 调用 (GetGuild / CreateGuild / ...) 留 wave 3
        match act {
            ActKind::Guild => {
                // 模拟 erlang tester_ai_base.erl:163-181 guild 协议序列
                debug!(bot_id = bot.id(), "Guild act stub (GetGuild)");
            }
            ActKind::Partner => {
                // 模拟 erlang tester_ai_base.erl:261-278 partner 协议序列
                debug!(bot_id = bot.id(), "Partner act stub (PartnerList)");
            }
            _ => {}
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
        let ai = SocialBotAi;
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn social_ai_act_list_contains_erlang_c6_acts() {
        let ai = SocialBotAi;
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
        let ai = SocialBotAi;
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }
}
