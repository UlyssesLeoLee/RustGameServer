//! match 域 BotAi 派生 (per DDD Review v0.2 §5.1 M2 + §5.2 ai/match.rs)
//!
//! 行为序列 (per erlang tester_ai_base.erl:54-58 act_list + C6 业务场景):
//! - `Init`         启动初始化
//! - `Heartbeat`    周期心跳
//! - `RandProto(50)` 5% 概率触发协议随机化 (千分位, per erlang C1)
//! - `Arena`        竞技场 (per tester_ai_base.erl:205-224)
//! - `Boss`         世界 Boss (per tester_ai_base.erl:225-239)
//!
//! 真实 match gRPC 调用 (EnqueueMatchmaking / GetMatch / CreateMatch / JoinMatch)
//! 留 wave 3 接 mTLS 业务级 ST. PoC 阶段 `handle` 全 OK stub.

use async_trait::async_trait;
use tracing::debug;

use crate::bot::act::ActKind;
use crate::bot::ai::BotAi;
use crate::bot::Bot;

/// match 域 BotAi (per DDD Review v0.2 §5.1 M2 + §5.2)
///
/// 5 域派生基线 — `act_list` = `[Init, Heartbeat, RandProto(50), Arena, Boss]`,
/// `handle` 全 OK stub. 真实 match gRPC 调用 (EnqueueMatchmaking / GetMatch /
/// CreateMatch / JoinMatch) 留 wave 3 接 mTLS 业务级 ST.
///
/// # 文件名注意
///
/// `match.rs` 文件名不是 Rust 关键字 (文件名), 但 `match` 本身是关键字.
/// 调用方 import 路径: `rgs_testkit::bot::ai::match::MatchBotAi`
/// (mod 段 `pub mod match;` 允许这种引用, 跟 std 的 `r#match` 不同).
#[derive(Clone, Debug, Default)]
pub struct MatchBotAi;

#[async_trait]
impl BotAi for MatchBotAi {
    async fn init(&self, bot: &Bot) -> anyhow::Result<()> {
        debug!(bot_id = bot.id(), "MatchBotAi::init");
        // PoC stub: 真实 match gRPC 调用 (e.g. GetMatch) 留 wave 3
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
                // PoC stub: 真实调用 match.v1.MatchService.GetMatch
                debug!(bot_id = bot.id(), "MatchBotAi::handle Init (stub GetMatch)");
            }
            ActKind::Heartbeat => {
                // PoC stub: 真实调用 match.v1.MatchService.HealthCheck
                debug!(bot_id = bot.id(), "MatchBotAi::handle Heartbeat (stub HealthCheck)");
            }
            ActKind::RandProto(_) => {
                // PoC stub: 真实调用 match.v1.MatchService.GetMatch (协议随机化)
                debug!(bot_id = bot.id(), "MatchBotAi::handle RandProto (stub)");
            }
            ActKind::Arena => {
                // PoC stub: 真实调用 match.v1.MatchService.EnqueueMatchmaking (queue 竞技场)
                debug!(bot_id = bot.id(), "MatchBotAi::handle Arena (stub EnqueueMatchmaking)");
            }
            ActKind::Boss => {
                // PoC stub: 真实调用 match.v1.MatchService.CreateMatch / JoinMatch (世界 Boss 房间)
                debug!(bot_id = bot.id(), "MatchBotAi::handle Boss (stub CreateMatch)");
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
        let ai = MatchBotAi;
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init");
    }

    #[tokio::test]
    async fn match_ai_act_list_contains_arena_and_boss() {
        let ai = MatchBotAi;
        let acts = ai.act_list();
        assert!(acts.contains(&ActKind::Init));
        assert!(acts.contains(&ActKind::Heartbeat));
        assert!(acts.contains(&ActKind::RandProto(50)));
        assert!(acts.contains(&ActKind::Arena));
        assert!(acts.contains(&ActKind::Boss));
    }

    #[tokio::test]
    async fn match_ai_handle_all_acts_ok() {
        let ai = MatchBotAi;
        let bot = dummy_bot();
        for act in ai.act_list() {
            ai.handle(&bot, act).await.expect("handle");
        }
    }
}
