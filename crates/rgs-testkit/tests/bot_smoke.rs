//! bot 框架 smoke 集成测试 (per DDD Review v0.2 §5.1 M1-M6 PoC)
//!
//! 验证目标:
//! - M1 Bot 子模块 (Bot struct + BotCore trait)
//! - M2 BotAi callback 协议 (init / act_list / handle)
//! - M3 ActKind + ActList 概率触发
//! - M4 GmClient stub
//! - M5 BotStats 实时统计 (count / offline)
//! - M6 BotSupervisor 错峰 + 掉线自愈
//!
//! **不依赖 5 域真 gRPC** (per PoC stub 设计), 走 mTLS 业务级 ST 留 wave 2.
//! 用 `#[tokio::test]` (不是 `#[pg_test]`, bot 框架本身无 DB 交互).

use rgs_testkit::bot::ai::player::PlayerBotAi;
use rgs_testkit::bot::supervisor::BotSupervisor;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::gm::{GmClient, GmResponse};
use std::time::Duration;

#[tokio::test]
async fn bot_smoke_create_start_stop() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-001", "player", stats.clone());
    bot.start().await.expect("bot start");
    assert_eq!(stats.count().get("player").copied().unwrap_or(0), 1);
    bot.stop().await;
    assert!(stats.offline().contains(&"bot-001".to_string()));
}

#[tokio::test]
async fn bot_supervisor_stagger_spawn() {
    let mut sup = BotSupervisor::new(10, 100); // 10 bots, 100ms stagger
    sup.spawn_with_stagger().await.expect("spawn");
    // 简单验: spawn 完成后所有 bot 在线
    let online = sup.online_count();
    assert!(online >= 8, "期望 ≥8 bot 在线, 实测 {}", online); // 留 stagger 容差
    sup.stop_all().await;
    assert_eq!(sup.online_count(), 0);
}

#[tokio::test]
async fn bot_ai_player_act_list() {
    let ai = PlayerBotAi;
    let acts = ai.act_list();
    assert!(acts.contains(&ActKind::Heartbeat));
    assert!(acts.contains(&ActKind::RandProto(100)));
    assert!(acts.contains(&ActKind::Init));
}

#[tokio::test]
async fn bot_ai_player_handle_all_acts() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-001", "player", stats);
    let ai = PlayerBotAi;
    ai.init(&bot).await.expect("init");
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle ok");
    }
}

#[tokio::test]
async fn bot_gm_client_stub_returns_ok() {
    let c = GmClient::new("https://admin-service:8443");
    let r: GmResponse = c.issue("加经验 100").await.expect("gm stub ok");
    assert!(r.ok);
}

#[tokio::test]
async fn bot_stats_count_and_offline() {
    let s = BotStats::new();
    s.mark_online("bot-1", "player");
    s.mark_online("bot-2", "player");
    s.mark_online("bot-3", "economy");
    s.mark_offline("bot-2");
    let c = s.count();
    assert_eq!(c.get("player").copied(), Some(1));
    assert_eq!(c.get("economy").copied(), Some(1));
    let off = s.offline();
    assert_eq!(off, vec!["bot-2".to_string()]);
}

#[tokio::test]
async fn bot_act_list_pick_random_within_weight() {
    use rgs_testkit::bot::ActList;
    let al = ActList::new()
        .push(ActKind::Heartbeat)
        .push(ActKind::RandProto(1000)); // 100% 概率 RandProto
    for _ in 0..10 {
        assert_eq!(al.pick_random(), Some(ActKind::RandProto(1000)));
    }
}

#[tokio::test]
async fn bot_supervisor_max_clamped() {
    // max 超过白名单 4500 → 截断
    let sup = BotSupervisor::new(10_000, 50);
    assert_eq!(sup.max, BotSupervisor::MAX_BOTS);
}

#[tokio::test]
async fn bot_supervisor_relogin_offline_recovers() {
    let mut sup = BotSupervisor::new(3, 10);
    sup.spawn_with_stagger().await.expect("spawn");
    assert_eq!(sup.online_count(), 3);
    // 手动 stop 一个 bot → offline
    if let Some(b) = sup.bots.first() {
        // clone 因为 stop 是 &self
        let _ = b; // 不动, 用 stop_all 替代
    }
    sup.stop_all().await;
    assert_eq!(sup.online_count(), 0);
    let recovered = sup.relogin_offline().await.expect("relogin");
    assert_eq!(recovered, 3, "3 个 bot 全部 relogin 成功");
    assert_eq!(sup.online_count(), 3);
}

#[tokio::test]
async fn bot_uses_default_mod_when_not_specified() {
    // 验证 spawn_with_stagger 默认 mod_name = "default"
    let mut sup = BotSupervisor::new(2, 10);
    sup.spawn_with_stagger().await.expect("spawn");
    let c = sup.stats.count();
    assert_eq!(c.get("default").copied(), Some(2));
    sup.stop_all().await;
    // 防 stop_all 期间 task panic 导致卡死
    tokio::time::sleep(Duration::from_millis(50)).await;
}
