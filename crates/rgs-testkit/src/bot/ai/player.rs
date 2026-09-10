//! player 域 BotAi 派生 (per DDD Review v0.2 §5.1 PoC)
//!
//! PoC 行为序列 (per erlang B3 act_list + C1 协议随机化):
//! - `Init`         启动初始化
//! - `Heartbeat`    周期心跳
//! - `RandProto(100)` 10% 概率触发协议随机化 (千分位, per erlang C1)
//!
//! 真实 player gRPC 调用 (Heartbeat / GetCharacterProfile / ...) 留 wave 2 接.

use async_trait::async_trait;
use tracing::debug;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// player 域 BotAi (per DDD Review v0.2 §5.1 M2 PoC)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100)]`,
/// `handle` 全 OK stub. 真实实现留 wave 2.
#[derive(Clone, Debug, Default)]
pub struct PlayerBotAi;

#[async_trait]
impl BotAi for PlayerBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "PlayerBotAi::init");
        Ok(())
    }

    fn act_list(&self) -> Vec<ActKind> {
        vec![ActKind::Init, ActKind::Heartbeat, ActKind::RandProto(100)]
    }

    async fn handle(&self, bot: &Bot, act: ActKind) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), ?act, "PlayerBotAi::handle");
        // PoC stub: 真实 player gRPC 调用留 wave 2
        Ok(())
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
        let ai = PlayerBotAi;
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn player_ai_act_list_contains_heartbeat_and_rand_proto() {
        let ai = PlayerBotAi;
        let acts = ai.act_list();
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::RandProto(100)));
    }

    #[tokio::test]
    async fn player_ai_handle_all_acts_ok() {
        let ai = PlayerBotAi;
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }
}
