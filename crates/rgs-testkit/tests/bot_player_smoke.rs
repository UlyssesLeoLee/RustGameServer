//! player 域 BotAi 集成测试 (per DDD Review v0.3.1 §7.3 wave 3 mTLS 真实接入 +
//! DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! 验证目标:
//! - `PlayerBotAi` impl BotAi (init / act_list / handle)
//! - wave 3 升级: 真实 `tonic::transport::Channel` + mTLS config 框架替换 stub
//! - wave 4 升级: 真实 gRPC 调用 (`PlayerServiceClient::heartbeat` / `get_player_profile`),
//!   2s timeout, 失败容忍 (Unreachable → Ok, 不 panic)
//! - skip_verify 模式 (默认, k3s baseline 0/12) → lazy Channel 构造成功
//! - 真实 mTLS 模式 (cert 缺失) → channel 降级 stub, handle 仍 Ok
//! - 凭据不读 env, 不打印 (per 8/27 11:06 JST 硬 ban)
//!
//! **真实 RPC 预期失败** (per DDD Review v0.3.2 §7.3 L1.2 + k3s baseline 0/12, 9/10 16:36 JST
//! 拍板"接受 baseline 等 SRE 介入"), 走 `RpcCallOutcome::Unreachable` 分支 + warn log,
//! 不影响业务 (handle 仍 Ok).
//!
//! **不依赖 player-service 二进制** (per 任务简报范围外约束), 仅验证 bot 框架内
//! tonic mTLS Channel + generated client + RPC 调用流程.

use rgs_testkit::bot::ai::player::{PlayerBotAi, PlayerMtlsConfig, RpcCallOutcome};
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};

#[tokio::test]
async fn bot_player_full_lifecycle_with_mtls() {
    // 1. 构造 bot + PlayerBotAi (默认 skip_verify 模式)
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-mtls-001", "player", stats.clone());
    let ai = PlayerBotAi::new();

    // 2. init 构造真实 mTLS Channel (lazy, skip_verify)
    ai.init(&bot).await.expect("player ai init");
    assert!(ai.is_real().await, "默认 skip_verify 应构造真实 Channel");

    // 3. start bot (注册到 BotStats)
    bot.start().await.expect("bot start");
    assert_eq!(stats.count().get("player").copied().unwrap_or(0), 1);

    // 4. handle 全 3 acts (Init / Heartbeat / RandProto)
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle act");
    }

    // 5. stop bot (offline)
    bot.stop().await;
    assert!(stats.offline().contains(&"bot-player-mtls-001".to_string()));
}

#[tokio::test]
async fn bot_player_real_grpc_client_init() {
    // 核心测试: 验证 wave 3 真实 tonic Channel 构造, 替换 wave 2 stub
    // (per DDD Review v0.3.1 §7.3)
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-mtls-init", "player", stats);
    let ai = PlayerBotAi::new();

    // init 不 panic
    ai.init(&bot).await.expect("init with real grpc client");

    // skip_verify 模式 → 真实 Channel 构造成功
    // (k3s baseline 0/12 状态, lazy Channel 不做实际握手)
    assert!(ai.is_real().await, "skip_verify 模式应构造真实 Channel");

    // 验证内部 PlayerMtlsClient 拿到正确 endpoint + domain (凭据不打印)
    let client = ai.mtls_client().await.expect("client should be set after init");
    let cfg = client.config();
    assert_eq!(cfg.endpoint.as_deref(), Some("https://127.0.0.1:50051"));
    assert_eq!(cfg.domain.as_deref(), Some("player-service"));
    assert!(cfg.skip_verify, "默认应 skip_verify = true");
    assert!(client.is_real(), "PlayerMtlsClient 自身应 is_real");
}

#[tokio::test]
async fn bot_player_handle_heartbeat_with_real_channel() {
    // 验证 wave 3 Heartbeat 走真实 mTLS Channel 框架
    // (per DDD Review v0.3.1 §7.3 PoC: 仅记录状态, 实际 RPC 留 wave 4 SRE)
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-heartbeat", "player", stats);
    let ai = PlayerBotAi::new();
    ai.init(&bot).await.expect("init");

    // Heartbeat 走 mTLS 框架, 返 Ok (PoC 不实际 RPC)
    let r = ai.handle(&bot, ActKind::Heartbeat).await;
    assert!(
        r.is_ok(),
        "Heartbeat 走 mTLS framework 应返 Ok (PoC, 实际 RPC 留 wave 4)"
    );
}

#[tokio::test]
async fn bot_player_ai_3_acts_layout() {
    // 验证 act_list 跟 DDD Review v0.3.1 §7.3 wave 3 描述一致:
    // [Init, Heartbeat, RandProto(100)]
    let ai = PlayerBotAi::new();
    let acts = ai.act_list();
    assert_eq!(acts.len(), 3, "act_list 应为 3 acts");
    assert!(matches!(acts[0], ActKind::Init));
    assert!(matches!(acts[1], ActKind::Heartbeat));
    assert!(matches!(acts[2], ActKind::RandProto(100)));
}

#[tokio::test]
async fn bot_player_with_real_mtls_config_falls_back_on_missing_cert() {
    // 真实 mTLS 模式 + cert 路径不存在 → channel 降级 stub, init 仍 Ok
    // (per 9/10 WipeCluster 后 cert 未导出场景)
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-real-mtls", "player", stats);
    let cfg = PlayerMtlsConfig {
        endpoint: Some("https://player-service:50051".to_string()),
        domain: Some("player-service".to_string()),
        ca_cert_path: Some("/nonexistent/ca.pem".to_string()),
        client_cert_path: Some("/nonexistent/client.pem".to_string()),
        client_key_path: Some("/nonexistent/client.key".to_string()),
        skip_verify: false,
    };
    let ai = PlayerBotAi::with_config(cfg);

    // init 不 panic, channel 降级 stub
    ai.init(&bot)
        .await
        .expect("init should not panic on missing cert");

    // is_real = false (降级)
    assert!(
        !ai.is_real().await,
        "cert 缺失应降级 stub, is_real = false"
    );

    // handle 仍 Ok (走 stub 路径)
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle stub path");
    }
}

#[tokio::test]
async fn bot_player_with_custom_endpoint_skip_verify() {
    // 自定义 endpoint + skip_verify 模式 → 真实 Channel 构造
    // (per 5 域 ST 多环境路由, M10 P1 准备)
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-custom-ep", "player", stats);
    let cfg = PlayerMtlsConfig {
        endpoint: Some("https://player-staging:50051".to_string()),
        domain: Some("player-staging".to_string()),
        skip_verify: true,
        ..Default::default()
    };
    let expected_endpoint = cfg.endpoint.clone();
    let expected_domain = cfg.domain.clone();
    let ai = PlayerBotAi::with_config(cfg);
    ai.init(&bot).await.expect("init with custom endpoint");

    assert!(
        ai.is_real().await,
        "skip_verify + custom endpoint 应构造真实 Channel"
    );
    let client = ai.mtls_client().await.expect("client");
    assert_eq!(expected_endpoint, client.config().endpoint);
    assert_eq!(expected_domain, client.config().domain);
}

#[tokio::test]
async fn bot_player_init_twice_is_idempotent() {
    // 多次 init 不 panic, Channel 状态被覆盖
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-init-twice", "player", stats);
    let ai = PlayerBotAi::new();

    ai.init(&bot).await.expect("first init");
    ai.init(&bot).await.expect("second init (idempotent)");

    assert!(ai.is_real().await, "二次 init 后仍 is_real = true");
}

// =============================================================================
// Wave 4 真实 RPC 接入测试 (per DDD Review v0.3.2 §7.3 L1.2)
// =============================================================================
//
// k3s baseline 0/12 (per 9/10 16:36 JST 拍板"接受 baseline 等 SRE 介入"):
// 真实 RPC 调用预期 connection refused, 走 `RpcCallOutcome::Unreachable` 分支,
// handle 仍 Ok, 不 panic. SRE 介入 + k3s baseline 恢复后, `Unreachable` 转 `Ok`.

#[tokio::test]
async fn bot_player_real_rpc_call_heartbeat_returns_err_on_k3s_unreachable() {
    // wave 4 核心测试 (per task briefing "Step 3: 集成测试 ~30 min"):
    // 真实 heartbeat RPC 调用预期 Err (k3s 0/12, connection refused).
    // 2s timeout 防 hang, 失败容忍: 不 panic, 走 Unreachable 分支.
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-real-rpc-hb", "player", stats);
    let ai = PlayerBotAi::new();

    // init 构造 Channel + 内部调一次真实 heartbeat (预期 Unreachable 但 Ok(()))
    ai.init(&bot)
        .await
        .expect("init should not panic on k3s unreachable");
    assert!(ai.is_real().await, "skip_verify 模式应构造真实 Channel");

    // 拿 mTLS client 直接调 try_heartbeat, 验证 Unreachable 分支
    let client = ai
        .mtls_client()
        .await
        .expect("client should be set after init");
    let outcome = client.try_heartbeat().await;
    match outcome {
        RpcCallOutcome::Ok => {
            // 极小概率: 127.0.0.1:50051 真的有人在 listen (k3s 恢复), 接受
            eprintln!("heartbeat RPC succeeded (k3s baseline 已恢复?)");
        }
        RpcCallOutcome::Unreachable(e) => {
            // 预期: connection refused 或 timeout, error message 应非空
            assert!(
                !e.is_empty(),
                "Unreachable error message 应非空, got: {}",
                e
            );
            eprintln!("expected: heartbeat RPC unreachable (k3s baseline 0/12): {}", e);
        }
    }

    // handle(Heartbeat) 走真实 RPC, 仍 Ok, 不 panic
    let r = ai.handle(&bot, ActKind::Heartbeat).await;
    assert!(
        r.is_ok(),
        "handle(Heartbeat) 真实 RPC 失败时仍 Ok, got: {:?}",
        r
    );

    // 完整 lifecycle 不 panic
    bot.start().await.expect("bot start");
    for act in ai.act_list() {
        ai.handle(&bot, act).await.expect("handle all acts");
    }
    bot.stop().await;
}

#[tokio::test]
async fn bot_player_real_rpc_call_get_player_profile_returns_err_on_k3s_unreachable() {
    // wave 4 真实 RPC 调用 GetPlayerProfile 测试 (per task briefing "Step 2: handle(RandProto)"):
    // RandProto 走 GetPlayerProfile, 预期 Err (k3s 0/12), handle 仍 Ok.
    let stats = BotStats::new();
    let bot = Bot::new("bot-player-real-rpc-gpp", "player", stats);
    let ai = PlayerBotAi::new();
    ai.init(&bot).await.expect("init");

    let client = ai
        .mtls_client()
        .await
        .expect("client should be set after init");
    let outcome = client.try_get_player_profile().await;
    match outcome {
        RpcCallOutcome::Ok => {
            eprintln!("get_player_profile RPC succeeded (k3s baseline 已恢复?)");
        }
        RpcCallOutcome::Unreachable(e) => {
            assert!(!e.is_empty(), "Unreachable error message 应非空");
            eprintln!(
                "expected: get_player_profile RPC unreachable (k3s baseline 0/12): {}",
                e
            );
        }
    }

    // handle(RandProto) 走真实 RPC, 仍 Ok, 不 panic
    let r = ai.handle(&bot, ActKind::RandProto(100)).await;
    assert!(
        r.is_ok(),
        "handle(RandProto) 真实 RPC 失败时仍 Ok, got: {:?}",
        r
    );
}
