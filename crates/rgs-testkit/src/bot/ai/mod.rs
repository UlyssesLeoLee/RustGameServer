//! AI 行为框架 (per DDD Review v0.2 §5.1 M2)
//!
//! 模拟 erlang B1-B4: 可插拔 AI callback 协议 (init / act_list / handle).
//! - `BotAi` trait   callback 协议 (init + act_list + handle)
//! - `DefaultBotAi` 空壳实现, 5 域派生时填充 (per DDD Review v0.2 §5.2 ai/)
//!
//! 5 域派生 (wave 2 worker): player / economy / match / social / admin / quest
//! 当前 PoC 只派生 `player` (见 [`player::PlayerBotAi`]).

pub mod player;
pub mod social;

use async_trait::async_trait;

use crate::bot::act::ActKind;
use crate::bot::Bot;

/// AI 行为 callback 协议 (per erlang B1 tester_ai_base.erl -callback init/1 等)
///
/// 3 个回调:
/// - `init`     初始化 (在 Bot::start 内部调用)
/// - `act_list` 行为序列 (per erlang B3 act_list)
/// - `handle`   处理单个 act
///
/// 默认实现见 [`DefaultBotAi`], 5 域派生见 [`player::PlayerBotAi`].
#[async_trait]
pub trait BotAi: Send + Sync {
    /// 初始化 (Bot 启动时调用 1 次)
    async fn init(&self, _bot: &Bot) -> anyhow::Result<()> {
        Ok(())
    }

    /// 行为序列 (per erlang B3 act_list)
    fn act_list(&self) -> Vec<ActKind> {
        Vec::new()
    }

    /// 处理单个 act (返回 Err 时, supervisor 触发掉线自愈)
    async fn handle(&self, _bot: &Bot, _act: ActKind) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 默认 AI (空壳, 5 域派生时填充, per DDD Review v0.2 §5.2)
///
/// PoC 行为: init OK, act_list 为空, handle 全部 OK. 真实 5 域 Bot 派生
/// (wave 2) 替换为各域 `act_list` + 真实 gRPC `handle` 实现.
#[derive(Clone, Debug, Default)]
pub struct DefaultBotAi;

#[async_trait]
impl BotAi for DefaultBotAi {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::stats::BotStats;

    fn dummy_bot() -> Bot {
        Bot::new("bot-default", "default", BotStats::new())
    }

    #[tokio::test]
    async fn default_ai_init_ok() {
        let ai = DefaultBotAi;
        let bot = dummy_bot();
        ai.init(&bot).await.expect("init ok");
    }

    #[tokio::test]
    async fn default_ai_act_list_empty() {
        let ai = DefaultBotAi;
        let acts = ai.act_list();
        assert!(acts.is_empty());
    }

    #[tokio::test]
    async fn default_ai_handle_ok_for_any_act() {
        let ai = DefaultBotAi;
        let bot = dummy_bot();
        for act in [
            ActKind::Init,
            ActKind::Heartbeat,
            ActKind::Guild,
            ActKind::Custom("test".to_string()),
        ] {
            ai.handle(&bot, act).await.expect("handle ok");
        }
    }
}
