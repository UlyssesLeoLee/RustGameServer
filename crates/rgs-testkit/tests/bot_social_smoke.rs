//! social 域 BotAi 集成 smoke 测试 (per DDD Review v0.2 §5.1 M2 + 5 域 BotAi 派生)
//!
//! 验证目标:
//! - `SocialBotAi` init 走通 (stub GetFriendList 返回 OK)
//! - `act_list` 含 erlang C6 业务场景 act (Guild / Partner)
//! - `handle` 对所有 act 返 OK (Guild / Partner stub)
//!
//! **不依赖 5 域真 gRPC** (per PoC stub 设计), mTLS 业务级接入留 wave 3.
//! 用 `#[tokio::test]` (social BotAi 本身无 DB 交互, 跟 player 域 bot_smoke.rs 一致).

use rgs_testkit::bot::ai::social::SocialBotAi;
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};

#[tokio::test]
async fn bot_social_full_lifecycle() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-001", "social", stats.clone());
    let ai = SocialBotAi;

    // init 阶段
    ai.init(&bot).await.expect("init");

    // start 阶段
    bot.start().await.expect("start");
    assert_eq!(stats.count().get("social").copied().unwrap_or(0), 1);

    // handle 所有 act
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle");
    }

    // stop 阶段
    bot.stop().await;
    assert!(stats.offline().contains(&"bot-social-001".to_string()));
    assert_eq!(stats.count().get("social").copied().unwrap_or(0), 0);
}

#[tokio::test]
async fn bot_social_ai_inits_cleanly() {
    // 验证 init 不会因缺 gRPC 客户端而 panic
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-002", "social", stats);
    let ai = SocialBotAi;
    ai.init(&bot).await.expect("init clean");
}

#[tokio::test]
async fn bot_social_handle_all_acts_ok() {
    // 验证 handle 对 erlang C6 业务场景 act + 通用 act 都返 OK
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-003", "social", stats);
    let ai = SocialBotAi;

    // erlang C6 business scene acts
    for act in [
        ActKind::Guild,
        ActKind::Partner,
        ActKind::Heartbeat,
        ActKind::Init,
        ActKind::RandProto(100),
    ] {
        ai.handle(&bot, act).await.expect("handle ok");
    }

    // act_list 全集跑一遍
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("act_list handle ok");
    }
}
