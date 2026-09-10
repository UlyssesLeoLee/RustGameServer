//! social 域 BotAi 集成 smoke 测试 (per DDD Review v0.2 §5.1 M2 + 5 域 BotAi 派生)
//! + wave 3 mTLS 真实接入 (per DDD Review v0.3.1 §7.3 Phase C)
//!
//! 验证目标:
//! - `SocialBotAi` init 走通 (stub GetFriendList 返回 OK)
//! - `act_list` 含 erlang C6 业务场景 act (Guild / Partner)
//! - `handle` 对所有 act 返 OK (Guild / Partner stub)
//! - **wave 3**: `init` 创建真实 `tonic::transport::Channel` + mTLS 框架不 panic (skip verify 模式)
//!
//! **不依赖 5 域真 gRPC 连接** (per PoC + k3s baseline 0/12, 9/10 16:36 JST 拍板
//! "接受 baseline 等 SRE 介入"): `init` 走 `Endpoint::connect_lazy`, Channel 创建
//! 不阻塞, TLS handshake 推迟到首次 RPC. 真实业务 RPC 留 wave 4.
//!
//! 用 `#[tokio::test]` (social BotAi 本身无 DB 交互, 跟 player 域 bot_smoke.rs 一致).

use rgs_testkit::bot::ai::social::SocialBotAi;
use rgs_testkit::bot::gm::MtlsConfig;
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};

#[tokio::test]
async fn bot_social_full_lifecycle() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-001", "social", stats.clone());
    let ai = SocialBotAi::default();

    // init 阶段 (wave 3: 创建真实 tonic Channel + mTLS 框架)
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
    let ai = SocialBotAi::default();
    ai.init(&bot).await.expect("init clean");
}

#[tokio::test]
async fn bot_social_handle_all_acts_ok() {
    // 验证 handle 对 erlang C6 业务场景 act + 通用 act 都返 OK
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-003", "social", stats);
    let ai = SocialBotAi::default();

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

// =====================================================================
// wave 3 mTLS 真实接入测试 (per DDD Review v0.3.1 §7.3 Phase C)
// =====================================================================

#[tokio::test]
async fn bot_social_real_grpc_client_init() {
    // 验证真实 tonic Channel 创建不 panic (skip verify 模式)
    //
    // 流程:
    // 1. SocialBotAi::default() → endpoint = https://127.0.0.1:50054, mtls = None
    // 2. init(&bot) → build_channel() 走 skip-verify 分支 (无 mTLS config)
    //    + warn! 日志 (per 9/10 WipeCluster cert 未导出)
    //    + Endpoint::connect_lazy() 返回 Channel (不阻塞, TLS 推迟)
    // 3. 验证 channel_initialized() = true
    //
    // 不调真实 5 域 gRPC (k3s baseline 0/12 阻塞, per 9/10 16:36 JST 拍板).
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-mtls-001", "social", stats);
    let ai = SocialBotAi::default();

    // init 必须成功, 不 panic
    ai.init(&bot).await.expect("init real tonic Channel");

    // Channel 必须就绪
    assert!(
        ai.channel_initialized(),
        "init 后 Channel 应已就绪 (connect_lazy 不阻塞, Cell 写入)"
    );
    // 内部 Channel 引用必须存在
    assert!(
        ai.channel().is_some(),
        "channel() 必须返 Some(&Channel)"
    );
    // 凭据未配置 (skip-verify 模式)
    assert!(!ai.mtls_configured(), "默认无 mTLS 凭据");
    assert_eq!(ai.endpoint(), "https://127.0.0.1:50054");
}

#[tokio::test]
async fn bot_social_real_grpc_client_with_endpoint() {
    // 验证自定义 endpoint 也能正常创建 Channel
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-mtls-002", "social", stats);
    let ai = SocialBotAi::with_endpoint("https://social.svc.cluster.local:50054");

    ai.init(&bot).await.expect("init with custom endpoint");
    assert!(ai.channel_initialized());
    assert_eq!(ai.endpoint(), "https://social.svc.cluster.local:50054");
}

#[tokio::test]
async fn bot_social_real_grpc_client_with_mtls_marks_configured() {
    // 验证 mTLS 凭据全部提供时, mtls_configured() = true
    let mtls = MtlsConfig {
        endpoint: Some("https://social.local:50054".to_string()),
        client_cert_path: Some("/etc/rgs/certs/client.pem".to_string()),
        client_key_path: Some("/etc/rgs/certs/client.key".to_string()),
        ca_cert_path: Some("/etc/rgs/certs/social-ca.pem".to_string()),
        server_name: Some("social.local".to_string()),
        skip_verify: false,
    };
    let ai = SocialBotAi::with_mtls(mtls);
    assert!(
        ai.mtls_configured(),
        "三字段全有 → mtls_configured() 应为 true"
    );
    // 注: 不调 init() (cert 文件不存在, PEM 读取会 fail), 仅验证配置标记
}

// =====================================================================
// wave 4 真实 RPC 调用测试 (per DDD Review v0.3.2 §7.3 L1.2)
// =====================================================================

#[tokio::test]
async fn bot_social_real_rpc_call_health_check_returns_ok_on_k3s_unreachable() {
    // wave 4 真实 RPC 调用验证 (per L-CAND-016 防御: 仅 social proto RPC, 不改其他 4 域)
    //
    // 流程:
    // 1. SocialBotAi::default() + init() 走 wave 3 mTLS 框架
    // 2. init 内部调 wave 4 真实 health_check (per DDD Review v0.3.2 §7.3)
    // 3. k3s baseline 0/12 → connection refused → 降级模式 Ok(()) + warn
    // 4. 测试断言: init 不 panic, 返 Ok, channel_initialized() = true
    //
    // 不调真实业务 5 域 gRPC (k3s 不可达, per 9/10 16:36 JST 拍板"接受 baseline 0/12 等 SRE 介入")
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-wave4-001", "social", stats);
    let ai = SocialBotAi::default();

    // init 必须成功, 走 wave 4 真实 health_check (预期连接失败但走降级)
    ai.init(&bot).await.expect("init with wave 4 real RPC");

    // Channel 必须就绪
    assert!(ai.channel_initialized(), "init 后 Channel 应已就绪");
}

#[tokio::test]
async fn bot_social_real_rpc_call_get_guild_returns_ok_on_k3s_unreachable() {
    // wave 4 真实 RPC: handle(Guild) 调真实 get_guild
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-wave4-002", "social", stats);
    let ai = SocialBotAi::default();

    ai.init(&bot).await.expect("init");
    // handle(Guild) 走真实 get_guild RPC, 预期降级模式 Ok(())
    let r = ai.handle(&bot, ActKind::Guild).await;
    assert!(r.is_ok(), "handle(Guild) wave 4 真实 RPC 应降级 Ok(())");
}

#[tokio::test]
async fn bot_social_real_rpc_full_lifecycle_with_real_rpc_calls() {
    // wave 4 全生命周期 + 真实 RPC 调用
    let stats = BotStats::new();
    let bot = Bot::new("bot-social-wave4-003", "social", stats.clone());
    let ai = SocialBotAi::default();

    // init 走 wave 4 真实 health_check
    ai.init(&bot).await.expect("init");
    assert!(ai.channel_initialized());

    // start 阶段
    bot.start().await.expect("start");
    assert_eq!(stats.count().get("social").copied().unwrap_or(0), 1);

    // 跑全 act_list, 每个 act 走真实 RPC (health_check / get_guild)
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle with real RPC");
    }

    // stop 阶段
    bot.stop().await;
    assert!(stats.offline().contains(&"bot-social-wave4-003".to_string()));
}
