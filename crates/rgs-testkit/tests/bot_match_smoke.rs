//! match 域 BotAi 派生 smoke 集成测试 (per DDD Review v0.2 §5.1 M2 + §5.2)
//!
//! 验证目标 (per erlang C6: tester_ai_base.erl:54-58 + 205-239):
//! - match 域 BotAi init 成功
//! - act_list 包含 Init / Heartbeat / RandProto(50) / Arena / Boss
//! - handle 所有 act 都返回 Ok
//!
//! **不依赖 5 域真 gRPC** (per PoC stub 设计), 走 mTLS 业务级 ST 留 wave 3.
//! 用 `#[tokio::test]` (不是 `#[pg_test]`, bot 框架本身无 DB 交互).

use rgs_testkit::bot::ai::r#match::MatchBotAi;
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi};

#[tokio::test]
async fn bot_match_ai_inits_cleanly() {
    let ai = MatchBotAi;
    let bot = Bot::new("bot-match-001", "match", BotStats::new());
    ai.init(&bot).await.expect("match ai init ok");
}

#[tokio::test]
async fn bot_match_ai_act_list_contains_arena_and_boss() {
    let ai = MatchBotAi;
    let acts = ai.act_list();
    assert!(acts.contains(&ActKind::Init), "act_list 应含 Init");
    assert!(acts.contains(&ActKind::Heartbeat), "act_list 应含 Heartbeat");
    assert!(
        acts.contains(&ActKind::RandProto(50)),
        "act_list 应含 RandProto(50) 5% 协议随机化"
    );
    assert!(acts.contains(&ActKind::Arena), "act_list 应含 Arena (erlang C6 竞技场)");
    assert!(acts.contains(&ActKind::Boss), "act_list 应含 Boss (erlang C6 世界 Boss)");
}

#[tokio::test]
async fn bot_match_handle_all_acts_ok() {
    let ai = MatchBotAi;
    let bot = Bot::new("bot-match-002", "match", BotStats::new());
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("match handle all acts ok");
    }
}

#[tokio::test]
async fn bot_match_full_lifecycle() {
    // 完整生命周期: init + 遍历所有 act + verify 共享 stats
    let stats = BotStats::new();
    let bot = Bot::new("bot-match-003", "match", stats.clone());
    let ai = MatchBotAi;
    ai.init(&bot).await.expect("init");

    // 跑 2 轮 (init + arena + boss + heartbeat)
    for round in 0..2 {
        for act in ai.act_list() {
            ai.handle(&bot, act.clone()).await.expect("handle");
        }
        // 2 轮结束, 验证 Bot 仍在线 (init 不会 mark_offline)
        assert!(
            !stats.offline().contains(&"bot-match-003".to_string()),
            "round {round}: bot 应仍在线"
        );
    }
    // init 不应 mark online (走 BotCore::start, 跟 PoC 一致), 但 offline 应空
    assert!(stats.offline().is_empty(), "未 stop 应无 offline");
}
