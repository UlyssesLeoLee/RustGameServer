//! economy 域 BotAi 派生 (per DDD Review v0.2 §5.1 M2)
//!
//! PoC 行为序列 (per erlang C6 + tester_ai_base.erl:163-194 经济域 act):
//! - `Init`                  启动初始化 (跑 1 个 stub GetAccount)
//! - `Heartbeat`             周期心跳
//! - `RandProto(100)`        10% 概率触发协议随机化 (千分位, per erlang C1)
//! - `Custom("Trade")`       交易行为 stub (per tester_ai_base.erl 经济域)
//! - `Custom("Account")`     账户查询 stub (per tester_ai_base.erl 经济域)
//!
//! 真实 economy gRPC 调用 (GetAccount / ShopBuy / ExchangeDo / ...) 留 wave 2 接.

use async_trait::async_trait;
use tracing::debug;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// economy 域 BotAi (per DDD Review v0.2 §5.1 M2 PoC)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(100), Trade, Account]`,
/// `handle` 全 OK stub. 真实实现留 wave 2 (mTLS 业务级 ST 跑通后接 tonic client).
#[derive(Clone, Debug, Default)]
pub struct EconomyBotAi;

#[async_trait]
impl BotAi for EconomyBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "EconomyBotAi::init");
        // PoC stub: 真实 GetAccount gRPC 调用留 wave 2
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
        debug!(bot_id = bot.id(), ?act, "EconomyBotAi::handle");
        // PoC stub: 真实 economy gRPC 调用留 wave 2
        Ok(())
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
        let ai = EconomyBotAi;
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn economy_ai_act_list_contains_all_acts() {
        let ai = EconomyBotAi;
        let acts = ai.act_list();
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::RandProto(100)));
        assert!(acts.contains(&ActKind::Custom("Trade".to_string())));
        assert!(acts.contains(&ActKind::Custom("Account".to_string())));
    }

    #[tokio::test]
    async fn economy_ai_handle_all_acts_ok() {
        let ai = EconomyBotAi;
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }
}
