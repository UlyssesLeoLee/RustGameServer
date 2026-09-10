//! economy 域 BotAi 集成测试 (per DDD Review v0.2 §5.1 M2)
//!
//! 验证目标:
//! - EconomyBotAi act_list 跟 erlang C6 经济域 act 对应 (Init / Heartbeat / RandProto / Trade / Account)
//! - init / handle 全 OK stub
//! - 走 `#[tokio::test]` (bot 框架本身无 DB 交互, per bot_smoke.rs 模式)
//!
//! **不依赖 economy 域真 gRPC** (per PoC stub 设计), 走 mTLS 业务级 ST 留 wave 2.

use rgs_testkit::bot::ai::economy::EconomyBotAi;
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};

#[tokio::test]
async fn bot_economy_full_lifecycle() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-001", "economy", stats.clone());
    let ai = EconomyBotAi;

    // 启动 + init
    bot.start().await.expect("bot start");
    ai.init(&bot).await.expect("ai init");
    assert_eq!(stats.count().get("economy").copied().unwrap_or(0), 1);

    // handle 全 act OK
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle ok");
    }

    // 停止
    bot.stop().await;
    assert!(stats.offline().contains(&"bot-economy-001".to_string()));
}

#[tokio::test]
async fn bot_economy_ai_inits_cleanly() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-002", "economy", stats);
    let ai = EconomyBotAi;

    // 多次 init 幂等 (per erlang B1 callback 协议)
    for _ in 0..3 {
        ai.init(&bot).await.expect("init ok");
    }
}

#[tokio::test]
async fn bot_economy_handle_all_acts_ok() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-003", "economy", stats);
    let ai = EconomyBotAi;

    let acts = ai.act_list();
    // 验 act_list 5 项 (Init / Heartbeat / RandProto(100) / Trade / Account)
    assert_eq!(acts.len(), 5);
    assert!(acts.contains(&ActKind::Init));
    assert!(acts.contains(&ActKind::Heartbeat));
    assert!(acts.contains(&ActKind::RandProto(100)));
    assert!(acts.contains(&ActKind::Custom("Trade".to_string())));
    assert!(acts.contains(&ActKind::Custom("Account".to_string())));

    for act in acts {
        ai.handle(&bot, act).await.expect("handle ok");
    }
}
