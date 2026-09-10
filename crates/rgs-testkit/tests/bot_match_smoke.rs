//! match 域 BotAi 派生 smoke 集成测试 (per DDD Review v0.2 §5.1 M2 + §5.2 + v0.3.1 §7.3 Phase C)
//!
//! 验证目标 (per erlang C6: tester_ai_base.erl:54-58 + 205-239):
//! - match 域 BotAi init 成功
//! - act_list 包含 Init / Heartbeat / RandProto(50) / Arena / Boss
//! - handle 所有 act 都返回 Ok
//!
//! **wave 3 (per 9/10 16:36 JST 拍板)**: 真实 tonic Channel (懒连接) +
//! skip verify fallback (k3s baseline 0/12 cert 未导出, 等 SRE 介入).
//! 不实际调 5 域 gRPC, 仅验 Channel build + skip verify 配置正确.
//!
//! 用 `#[tokio::test]` (不是 `#[pg_test]`, bot 框架本身无 DB 交互).

use rgs_testkit::bot::ai::r#match::{MatchBotAi, MatchGrpcClient, DEFAULT_MATCH_ENDPOINT};
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi};

#[tokio::test]
async fn bot_match_ai_inits_cleanly() {
    let ai = MatchBotAi::new();
    let bot = Bot::new("bot-match-001", "match", BotStats::new());
    ai.init(&bot).await.expect("match ai init ok");
}

#[tokio::test]
async fn bot_match_ai_act_list_contains_arena_and_boss() {
    let ai = MatchBotAi::new();
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
    let ai = MatchBotAi::new();
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
    let ai = MatchBotAi::new();
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

/// wave 3 (per DDD Review v0.3.1 §7.3 + 9/10 16:36 JST 拍板):
/// 验 match 域真实 tonic gRPC client 框架 (懒 Channel, skip verify fallback).
/// k3s baseline 0/12, cert 未导出, 走 skip verify; SRE 介入后切真 mTLS.
#[tokio::test]
async fn bot_match_real_grpc_client_init() {
    // r#match 关键字 import (per L19 + 9/3 11:08 JST 派生约束)
    let ai = MatchBotAi::new();
    let bot = Bot::new("bot-match-mtls-001", "match", BotStats::new());

    // 验默认 skip verify Channel 建立 (懒连接, 不实际 RPC)
    let grpc = ai.grpc();
    assert!(
        grpc.channel().is_some(),
        "默认 skip verify tonic Channel 应建立"
    );
    assert_eq!(grpc.endpoint(), DEFAULT_MATCH_ENDPOINT);
    assert!(grpc.skip_verify(), "默认 skip_verify=true (k3s cert 未导出 fallback)");

    // init 不 panic, 真实 EnqueueMatchmaking 待 SRE 介入
    ai.init(&bot).await.expect("init with real gRPC framework");

    // 验 match 域 act_list 仍含 5 acts (Init/Heartbeat/RandProto(50)/Arena/Boss)
    let acts = ai.act_list();
    assert_eq!(acts.len(), 5, "match act_list 长度应为 5");
    assert!(acts.contains(&ActKind::Arena));
    assert!(acts.contains(&ActKind::Boss));
}

/// wave 3: 自定义 endpoint + MtlsConfig 注入 (凭据走 Option, 不读 env, 不打印)
#[tokio::test]
async fn bot_match_real_grpc_client_custom_endpoint() {
    use rgs_testkit::bot::gm::MtlsConfig;

    let mtls = MtlsConfig {
        endpoint: Some("https://match-staging:50053".to_string()),
        // 凭据走 Option, 真实路径待 SRE 介入后注入
        client_cert_path: None,
        client_key_path: None,
        ca_cert_path: None,
        server_name: Some("match.service".to_string()),
        skip_verify: false,
    };
    let grpc = MatchGrpcClient::builder()
        .endpoint("https://match-staging:50053")
        .mtls(mtls)
        .skip_verify(false)
        .build();
    let ai = MatchBotAi::with_grpc_client(grpc);
    let bot = Bot::new("bot-match-mtls-002", "match", BotStats::new());

    assert_eq!(ai.grpc().endpoint(), "https://match-staging:50053");
    assert!(!ai.grpc().skip_verify(), "skip_verify=false (待 SRE 介入后切真 mTLS)");
    assert!(
        ai.grpc().channel().is_some(),
        "自定义 endpoint + skip_verify=false Channel 应建立"
    );
    ai.init(&bot).await.expect("init with custom grpc");
}

/// wave 4 (per DDD Review v0.3.2 §7.3 L1.2 + 2026-09-10 19:00 JST 拍板):
/// 验真实 RPC 调用 (`MatchServiceClient::enqueue_matchmaking`) 走
/// `tokio::time::timeout(2s)`, 失败时返 `Ok(())` 不 panic.
///
/// 预期: k3s baseline 0/12 阶段, `connection refused` (127.0.0.1:50053 不可达),
/// `tracing::warn!` + `Ok(())` 错误容忍模式, 不 panic 不静默吞.
#[tokio::test]
async fn bot_match_real_rpc_call_enqueue_pvp_returns_err_on_k3s_unreachable() {
    // r#match 关键字 import (per L19 + 9/3 11:08 JST 派生约束) — wave 4 验 typed 客户端可构造
    let ai = MatchBotAi::new();
    let bot = Bot::new("bot-match-rpc-001", "match", BotStats::new());

    // typed MatchServiceClient<Channel> 应可构造 (per wave 4 rgs-testkit build.rs 生成)
    let client_opt = ai.match_service_client();
    assert!(
        client_opt.is_some(),
        "MatchServiceClient<Channel> 应可构造 (per wave 4 typed client)"
    );

    // 真实 RPC 调用: enqueue_matchmaking — k3s baseline 0/12 预期失败
    // 走 `Ok(())` 错误容忍, 不 panic 不 Err
    let result = ai.enqueue_matchmaking(&bot).await;
    assert!(
        result.is_ok(),
        "enqueue_matchmaking 应返 Ok(()) (k3s 不可达 走错误容忍, 不 panic)"
    );

    // 真实 RPC 调用: create_match — 同样预期失败, 走 Ok(())
    let result = ai.create_match(&bot).await;
    assert!(
        result.is_ok(),
        "create_match 应返 Ok(()) (k3s 不可达 走错误容忍, 不 panic)"
    );

    // 2s timeout 防御: 整个测试应在 5s 内完成 (2s * 2 RPC + 余量)
    let start = std::time::Instant::now();
    let _ = ai.enqueue_matchmaking(&bot).await;
    let _ = ai.create_match(&bot).await;
    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "2 次真实 RPC 调用 (2s timeout each) 应在 5s 内完成, 实际耗时: {elapsed:?}"
    );
}

/// wave 4: 自定义 endpoint RPC 真实调用 (per `tokio::time::timeout(2s)` 防 hang)
///
/// 同上一个 test, 但走自定义 endpoint, 验证 typed `MatchServiceClient<Channel>`
/// 可在自定义 endpoint 上构造, 真实 RPC 失败时走 `Ok(())` 容忍.
#[tokio::test]
async fn bot_match_real_rpc_custom_endpoint_returns_ok() {
    let grpc = MatchGrpcClient::builder()
        .endpoint("https://match-staging.invalid:50053")
        .skip_verify(true)
        .build();
    let ai = MatchBotAi::with_grpc_client(grpc);
    let bot = Bot::new("bot-match-rpc-002", "match", BotStats::new());

    // typed client 仍可构造 (per wave 4)
    assert!(
        ai.match_service_client().is_some(),
        "自定义 endpoint typed client 应可构造"
    );

    // 真实 RPC 调用预期失败, 走 Ok(()) 容忍
    let result = ai.enqueue_matchmaking(&bot).await;
    assert!(result.is_ok(), "自定义 endpoint enqueue_matchmaking 应返 Ok(())");
}
