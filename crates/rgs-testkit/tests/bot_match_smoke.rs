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
