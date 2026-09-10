//! admin 域 BotAi smoke 集成测试 (per DDD Review v0.2 §5.1 M2 + M4 wave 2 派工)
//!
//! 验证目标:
//! - `AdminBotAi` impl BotAi (init / act_list / handle)
//! - act_list 包含 5 acts: Init / Heartbeat / RandProto(100) / GmCommand / BanAccount
//! - GM 注入链路走 `GmClient::issue` (per erlang C2 + M4), 演示不直接调 admin gRPC
//! - 真实 admin mTLS gRPC 调用留 wave 3
//!
//! **不依赖 5 域真 gRPC** (per PoC stub 设计), 跟 player 域 `bot_smoke.rs` 模式一致.

use rgs_testkit::bot::ai::admin::AdminBotAi;
use rgs_testkit::bot::gm::GmClient;
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};

#[tokio::test]
async fn bot_admin_full_lifecycle() {
    // 1. 构造 bot + AdminBotAi
    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-001", "admin", stats.clone());
    let ai = AdminBotAi::new();

    // 2. init (内部跑 1 个 stub GmCommand)
    ai.init(&bot).await.expect("admin ai init");

    // 3. start bot (注册到 BotStats)
    bot.start().await.expect("bot start");
    assert_eq!(stats.count().get("admin").copied().unwrap_or(0), 1);

    // 4. handle 全 5 acts
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle act");
    }

    // 5. stop bot (offline)
    bot.stop().await;
    assert!(stats.offline().contains(&"bot-admin-001".to_string()));
}

#[tokio::test]
async fn bot_admin_ai_inits_cleanly() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-init-001", "admin", stats);
    let ai = AdminBotAi::new();
    // init 应该不 panic + 内部 GmClient stub 走通
    ai.init(&bot).await.expect("init cleanly");
}

#[tokio::test]
async fn bot_admin_handle_all_acts_ok() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-handle-001", "admin", stats);
    let ai = AdminBotAi::new();
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle ok");
    }
}

#[tokio::test]
async fn bot_admin_gm_inject_via_gm_client() {
    // 演示 GM 注入链路: AdminBotAi 内部 GmClient::issue, 不直接走 admin gRPC
    // (per DDD Review v0.2 §5.1 M4 + erlang C2)
    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-gm-001", "admin", stats);
    let ai = AdminBotAi::new();

    // GmCommand 走 GmClient stub
    let r = ai
        .handle(&bot, ActKind::Custom("GmCommand".to_string()))
        .await;
    assert!(r.is_ok(), "GmCommand handle 应返 Ok (走 GmClient stub)");

    // BanAccount 也走 GmClient stub
    let r = ai
        .handle(&bot, ActKind::Custom("BanAccount".to_string()))
        .await;
    assert!(r.is_ok(), "BanAccount handle 应返 Ok (走 GmClient stub)");
}

#[tokio::test]
async fn bot_admin_ai_with_injected_gm_client() {
    // 验证 AdminBotAi 接受外部注入的 GmClient (per AGENTS.md §1.2 凭据走 Option)
    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-injected-001", "admin", stats);
    let custom_gm = GmClient::new("https://admin-staging:8443");
    let ai = AdminBotAi::with_gm_client(custom_gm);

    // init 走自定义 GmClient
    ai.init(&bot).await.expect("init with custom gm");
    // GmCommand 走自定义 GmClient
    ai.handle(&bot, ActKind::Custom("GmCommand".to_string()))
        .await
        .expect("GmCommand via custom gm");
}

#[tokio::test]
async fn bot_admin_ai_5_acts_layout() {
    // 验证 act_list 跟 DDD Review v0.2 §5.1 M2 描述一致:
    // [Init, Heartbeat, RandProto(100), GmCommand, BanAccount]
    let ai = AdminBotAi::new();
    let acts = ai.act_list();
    assert_eq!(acts.len(), 5, "act_list 应为 5 acts");
    assert!(matches!(acts[0], ActKind::Init));
    assert!(matches!(acts[1], ActKind::Heartbeat));
    assert!(matches!(acts[2], ActKind::RandProto(100)));
    assert_eq!(acts[3], ActKind::Custom("GmCommand".to_string()));
    assert_eq!(acts[4], ActKind::Custom("BanAccount".to_string()));
}
