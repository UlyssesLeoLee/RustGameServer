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

#[tokio::test]
async fn bot_admin_real_grpc_client_init() {
    // wave 3 mTLS 真实接入集成测试 (per DDD Review v0.3.1 §7.3 Phase C)
    //
    // 验证目标:
    // - AdminBotAi 默认持真实 tonic Channel (lazy, 0 网络往返)
    // - endpoint = https://127.0.0.1:50055 (admin-service default port)
    // - skip_verify = true (k3s baseline 0/12 阶段, per 9/10 16:36 JST 拍板)
    // - issue(cmd) 不 panic, 返 Ok + GmResponse { ok: false, error: Some }
    //   (k3s baseline 0/12 阶段真实 RPC 必失败, 不 panic 不静默吞)
    //
    // 真实连接待 SRE 介入 k3s baseline 恢复后跑 (per DDD Review v0.3.1 §7.3)

    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-mtls-001", "admin", stats);
    let ai = AdminBotAi::new();

    // 1. 默认 AdminBotAi 持真实 lazy mTLS Channel
    let gm = ai
        .gm_client()
        .expect("AdminBotAi default should have gm_client");
    assert!(
        gm.channel().is_some(),
        "默认 GmClient 应建 lazy mTLS Channel (wave 3 升级)"
    );
    assert!(gm.skip_verify(), "k3s baseline 0/12 阶段默认 skip verify");
    assert_eq!(gm.endpoint(), Some("https://127.0.0.1:50055"));

    // 2. init 走真实 mTLS 通道, 不 panic
    //    (k3s baseline 0/12 阶段真实 RPC 必失败, 返 Ok + error 字段填充)
    ai.init(&bot).await.expect("init with real mTLS channel");

    // 3. GmCommand handle 走真实 mTLS 通道, 不 panic
    let r = ai
        .handle(&bot, ActKind::Custom("GmCommand".to_string()))
        .await;
    assert!(
        r.is_ok(),
        "GmCommand handle 应返 Ok (real mTLS, k3s baseline 0/12 必失败但不 panic)"
    );

    // 4. BanAccount handle 走真实 mTLS 通道, 不 panic
    let r = ai
        .handle(&bot, ActKind::Custom("BanAccount".to_string()))
        .await;
    assert!(
        r.is_ok(),
        "BanAccount handle 应返 Ok (real mTLS, k3s baseline 0/12 必失败但不 panic)"
    );
}

#[tokio::test]
async fn bot_admin_5_acts_layout_with_mtls_channel() {
    // 静态验证: AdminBotAi 默认 = 真实 lazy mTLS Channel
    // 注: tonic 0.12 connect_lazy 需要 tokio runtime, 用 #[tokio::test]
    let ai = AdminBotAi::new();
    let acts = ai.act_list();
    assert_eq!(acts.len(), 5, "act_list 应为 5 acts");

    // 1 个 GmClient 默认 = 真实 lazy mTLS Channel
    let gm = ai
        .gm_client()
        .expect("AdminBotAi default should have gm_client");
    assert!(
        gm.channel().is_some(),
        "默认 GmClient 应建 lazy mTLS Channel"
    );
}

#[tokio::test]
async fn bot_admin_real_rpc_call_ban_account_returns_err_on_k3s_unreachable() {
    // wave 4 真实 RPC 调用集成测试 (per DDD Review v0.3.2 §7.3 L1.2)
    //
    // 验证目标:
    // - AdminBotAi 默认 GmClient 走 issue_real 真实 admin proto RPC 调用
    //   (admin_service_client::AdminServiceClient<Channel>::ban_account(...))
    // - 2s timeout 防 hang (per L11 + 5 域 ST 业务级 mTLS 实践)
    // - k3s baseline 0/12 阶段, 真实 RPC 必失败 (connection refused), 但不 panic
    // - 返 Ok + GmResponse { ok: false, error: Some }, 错误容忍模式
    // - L-CAND-016 防御: 只加 admin proto RPC, 不改其他 4 域

    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-real-rpc-001", "admin", stats);
    let ai = AdminBotAi::new();

    // 1. 默认 GmClient 持真实 lazy mTLS Channel
    let gm = ai
        .gm_client()
        .expect("AdminBotAi default should have gm_client");
    assert!(
        gm.channel().is_some(),
        "默认 GmClient 应建 lazy mTLS Channel (wave 3 升级)"
    );

    // 2. GmCommand 走 issue_real 真实 RPC 调用 (wave 4 升级)
    //    2s timeout 防 hang, 错误容忍模式
    let r = ai
        .handle(&bot, ActKind::Custom("GmCommand".to_string()))
        .await;
    assert!(
        r.is_ok(),
        "GmCommand handle 应返 Ok (issue_real 真实 RPC, k3s baseline 0/12 必失败但不 panic)"
    );

    // 3. BanAccount 走 issue_real 真实 RPC 调用 (wave 4 升级)
    let r = ai
        .handle(&bot, ActKind::Custom("BanAccount".to_string()))
        .await;
    assert!(
        r.is_ok(),
        "BanAccount handle 应返 Ok (issue_real 真实 RPC, k3s baseline 0/12 必失败但不 panic)"
    );
}

#[tokio::test]
async fn bot_admin_real_rpc_call_init_does_not_panic() {
    // wave 4 真实 RPC init 阶段集成测试 (per DDD Review v0.3.2 §7.3 L1.2)
    //
    // 验证目标:
    // - AdminBotAi::init 走 issue_real 真实 RPC (admin proto ban_account)
    // - 2s timeout 防 hang
    // - 错误容忍模式: 真实 RPC 失败时返 Ok(()) 不 panic

    let stats = BotStats::new();
    let bot = Bot::new("bot-admin-real-rpc-init-001", "admin", stats);
    let ai = AdminBotAi::new();

    // init 阶段走真实 RPC, 不 panic
    ai.init(&bot)
        .await
        .expect("init with real RPC should not panic");
}

#[tokio::test]
async fn bot_admin_issue_real_direct_call_returns_error_on_k3s_unreachable() {
    // wave 4 直接 issue_real 调用测试 (per DDD Review v0.3.2 §7.3 L1.2)
    //
    // 验证目标:
    // - GmClient::issue_real 直接调用走 admin proto client ban_account 真实通路
    // - k3s baseline 0/12 阶段, 真实 RPC 必失败, 返 Ok + GmResponse { ok: false, error: Some }
    // - 2s timeout 防 hang
    let c = GmClient::new("https://placeholder:8443")
        .with_endpoint("https://127.0.0.1:50055")
        .with_skip_verify(true);

    let r = c
        .issue_real("ban_account 3600 违规")
        .await
        .expect("issue_real should not panic");
    assert!(!r.ok, "k3s baseline 0/12 阶段 ok 应 false");
    assert!(r.error.is_some(), "error 字段应填充");
}
