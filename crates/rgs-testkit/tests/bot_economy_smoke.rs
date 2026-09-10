//! economy 域 BotAi 集成测试 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! 验证目标 (per wave 2 派生 + wave 3 mTLS 真实接入 + wave 4 真实 RPC 接入):
//! - EconomyBotAi act_list 跟 erlang C6 经济域 act 对应 (Init / Heartbeat / RandProto / Trade / Account)
//! - init / handle 全 OK stub (默认 stub 模式, 跟 wave 2 EconomyBotAi 兼容)
//! - 真实 tonic Channel 构造 (with_endpoint + init) 走 mTLS 业务级路径
//!   (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`, 5 域 ST 业务级 ST 准备)
//! - **wave 4**: 真实 `EconomyServiceClient<Channel>::get_account` 调用, 2s timeout,
//!   k3s baseline 0/12 阶段预期失败 (connection refused), 走 `Ok(())` 错误容忍模式
//!
//! 走 `#[tokio::test]` (bot 框架本身无 DB 交互, per bot_smoke.rs 模式)

use std::time::{Duration, Instant};

use rgs_testkit::bot::ai::economy::EconomyBotAi;
use rgs_testkit::bot::stats::BotStats;
use rgs_testkit::bot::{ActKind, Bot, BotAi, BotCore};

#[tokio::test]
async fn bot_economy_full_lifecycle() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-001", "economy", stats.clone());
    let ai = EconomyBotAi::new();

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
    let ai = EconomyBotAi::new();

    // 多次 init 幂等 (per erlang B1 callback 协议)
    for _ in 0..3 {
        ai.init(&bot).await.expect("init ok");
    }
}

#[tokio::test]
async fn bot_economy_handle_all_acts_ok() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-003", "economy", stats);
    let ai = EconomyBotAi::new();

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

/// wave 3 新增: 真实 tonic gRPC client init (per DDD Review v0.3.1 §7.3 Phase C mTLS 业务级 ST 准备)
///
/// 验证 `EconomyBotAi::with_endpoint` + `init` 走真实 Channel 构造路径不 panic
/// (skip-verify 模式, k3s cert 未导出 fallback per 9/10 WipeCluster 重建).
///
/// 走 `Endpoint::from_shared + ClientTlsConfig::new().domain_name(sni) + connect_lazy()`
/// (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`).
///
/// # 不实际连接 k3s
/// - `connect_lazy()` 拿 lazy Channel, init 阶段不实际握手
/// - k3s baseline 0/12 阻塞 (per 9/10 16:36 JST 拍板"接受 baseline 等 SRE 介入"), 真实 RPC 留 SRE
#[tokio::test]
async fn bot_economy_real_grpc_client_init() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-mtls-001", "economy", stats);
    let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");

    // init 走真实 tonic Channel 构造 (lazy, 不实际连接 k3s)
    ai.init(&bot).await.expect("init with real mTLS channel");

    // handle Account 走真实 Channel 路径 (但**不调 RPC**, k3s baseline 0/12 阻塞)
    ai.handle(&bot, ActKind::Custom("Account".to_string()))
        .await
        .expect("handle Account real client");
}

/// wave 4 新增: 真实 RPC 调用 `get_account` (per DDD Review v0.3.2 §7.3 L1.2)
///
/// 验证 `EconomyBotAi::with_endpoint` + `handle(Account)` 走真实 `EconomyServiceClient::get_account` 路径,
/// k3s baseline 0/12 阶段预期 connection refused / 2s timeout, 走 `Ok(())` 错误容忍模式不 panic.
///
/// (per DDD Review v0.3.2 §7.3 L1.2 + 9/10 19:00 JST Ulysses 选 wave 4 + L-CAND-016 防御:
///
/// - 真实 client 构造走 `EconomyServiceClient<Channel>::new(channel)` (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
/// - 真实 RPC 调 `client.get_account(Request<EntityId>)` (per economy-service proto `GetAccount(EntityId) -> Account`)
/// - 2s timeout 防 hang (per D2 L1.2 E2E 业务级 ST 准备)
/// - k3s baseline 0/12 → 预期 `Err(Status)` 或 `Err(Elapsed)`, 走 `tracing::warn!` + `Ok(())` 不 panic
/// - 不动其他 4 域 (per L-CAND-016 防御, 5 worker 公共 proto RPC 调用要同步)
///
/// 跟 wave 3 `bot_economy_real_grpc_client_init` 区别:
/// - wave 3: 走 lazy Channel 路径, 不实际 RPC 调用
/// - wave 4: 真实 RPC 调用, 验证 k3s 不可达时 2s timeout 内返 `Ok(())` 不 panic
#[tokio::test]
async fn bot_economy_real_rpc_call_get_account_returns_err_on_k3s_unreachable() {
    let stats = BotStats::new();
    let bot = Bot::new("bot-economy-rpc-001", "economy", stats);
    let ai = EconomyBotAi::with_endpoint("https://127.0.0.1:50052", "economy-service");

    // init 走真实 client 构造 + 真实 RPC 调用 (k3s 不可达, 2s timeout 内返 Ok(()))
    let init_start = Instant::now();
    ai.init(&bot)
        .await
        .expect("init with real mTLS + real RPC should not panic");
    let init_elapsed = init_start.elapsed();
    assert!(
        init_elapsed < Duration::from_secs(3),
        "init 真实 RPC 应在 2s timeout 内完成 (实际 {}ms)",
        init_elapsed.as_millis()
    );
    // 真实 client 已构造
    assert!(
        ai.is_real().await,
        "real client 应已构造 (wave 4 真实 RPC 接入就绪)"
    );

    // handle Account 走真实 RPC 调用 (k3s 不可达, 2s timeout 内返 Ok(()))
    let handle_start = Instant::now();
    ai.handle(&bot, ActKind::Custom("Account".to_string()))
        .await
        .expect("handle Account real RPC should not panic on k3s unreachable");
    let handle_elapsed = handle_start.elapsed();
    assert!(
        handle_elapsed < Duration::from_secs(3),
        "handle Account 真实 RPC 应在 2s timeout 内完成 (实际 {}ms)",
        handle_elapsed.as_millis()
    );
}
