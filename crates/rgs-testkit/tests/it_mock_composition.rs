//! 集成测试: 跨模块 NoOp 组合场景 (per rgs-testkit 强约束 WF-1-55.31)
//!
//! 禁止依赖真 DB / InMemory mock PG; 用 NoOp stub + 真 socket 启 mock server
//! 验证跨模块组合行为. 覆盖:
//! - 5 域 fixture 注入 NATS subject 跨域事件流 (player.events / economy.tx / match.event)
//! - TonicGrpcMock + InMemoryNatsMock 组合 (5 域 gRPC + 跨域事件总线同框)
//! - pg_test_db fixture gate (env 未设置时 graceful skip, 不 panic)
//! - 多线程并发 publish InMemoryNatsMock (线程安全 + 计数正确)
//! - FixtureBuilder + serde 序列化 5 域 fixture 跨 crate JSON 字符串传递
//!
//! 规范: RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4 + DTL-021~025 + ARC-051

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::mock::{GrpcMock, InMemoryNatsMock, NatsMock, TonicGrpcMock};
use rgs_testkit::pg_test_db;
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// 5 域 fixture 注入 NATS subject: 跨域事件流 (DTL-021~025)
// ============================================================================

#[tokio::test]
async fn five_domain_fixtures_into_nats_subject_stream() {
    // 5 域 fixture 工厂 + FixtureBuilder 串联, 把 5 域事件 payload 推 NATS subject
    let nats = InMemoryNatsMock::new();

    // player 域: 登录事件 → player.events subject
    let p = FixtureBuilder::new(fixture::player())
        .with_name("Alice")
        .with_level(10)
        .build();
    nats.publish(
        "player.events",
        json!({"event": "login", "player_id": p.id, "level": p.level })
            .to_string()
            .as_bytes(),
    )
    .await
    .unwrap();

    // economy 域: 转账事件 → economy.tx subject
    let e = fixture::economy(&p.id);
    nats.publish(
        "economy.tx",
        json!({"player_id": e.player_id, "delta": -100, "currency_after": e.currency })
            .to_string()
            .as_bytes(),
    )
    .await
    .unwrap();

    // match 域: 比赛开始 → match.event subject
    let m = fixture::match_game(&p.id);
    nats.publish(
        "match.event",
        json!({"match_id": m.match_id, "player_id": m.player_id, "status": m.status })
            .to_string()
            .as_bytes(),
    )
    .await
    .unwrap();

    // social 域: 私信事件 → social.message subject
    let s = FixtureBuilder::new(fixture::social_message(&p.id, "bob"))
        .with_message("hi from rgs-testkit")
        .build();
    nats.publish(
        "social.message",
        json!({"from": s.player_id, "to": s.friend_id, "body": s.message })
            .to_string()
            .as_bytes(),
    )
    .await
    .unwrap();

    // admin 域: 审计事件 → admin.audit subject
    let a = FixtureBuilder::new(fixture::admin_action("admin1", "audit", &p.id))
        .with_action("view_profile")
        .with_target(&p.id)
        .build();
    nats.publish(
        "admin.audit",
        json!({"admin": a.admin_id, "action": a.action, "target": a.target_id })
            .to_string()
            .as_bytes(),
    )
    .await
    .unwrap();

    // 5 个 subject 各有 1 条, 总 5 条
    assert_eq!(nats.received_count("player.events"), 1);
    assert_eq!(nats.received_count("economy.tx"), 1);
    assert_eq!(nats.received_count("match.event"), 1);
    assert_eq!(nats.received_count("social.message"), 1);
    assert_eq!(nats.received_count("admin.audit"), 1);

    // 验证 player.events 取出内容含 player_id
    let player_msgs = nats.subscribe("player.events").await.unwrap();
    let player_event: serde_json::Value =
        serde_json::from_slice(&player_msgs[0]).expect("parse json");
    assert_eq!(player_event["event"], "login");
    assert_eq!(player_event["player_id"], p.id);
    assert_eq!(player_event["level"], 10);
}

// ============================================================================
// TonicGrpcMock + InMemoryNatsMock 组合: 5 域 gRPC + 跨域事件同框
// ============================================================================

#[tokio::test]
async fn tonic_grpc_mock_plus_inmemory_nats_compose() {
    // 业务场景: player-service gRPC login → 触发 player.events NATS 事件
    let mut grpc = TonicGrpcMock::new().await;
    let body = br#"{"session_epoch":"e1","player_id":"p-compose"}"#;
    grpc.expect("POST", "/player.v1.PlayerService/Login", 200, body);
    assert!(grpc.url().starts_with("http://"), "mock server must expose http url");

    // gRPC mock 起来后, 业务层会 publish 一条 NATS 事件
    let nats = InMemoryNatsMock::new();
    nats.publish(
        "player.events",
        br#"{"event":"login","player_id":"p-compose"}"#,
    )
    .await
    .unwrap();

    // 验证 NATS 收到 + gRPC mock url 可用
    assert_eq!(nats.received_count("player.events"), 1);
    let msgs = nats.subscribe("player.events").await.unwrap();
    assert_eq!(msgs[0], br#"{"event":"login","player_id":"p-compose"}"#);
}

// ============================================================================
// pg_test_db fixture gate: env 未设置时 graceful, 不 panic
// ============================================================================

#[tokio::test]
async fn pg_test_db_gate_when_database_url_unset() {
    // 显式 unset DATABASE_URL, 验证 pg_available() 返回 false 且 pg_pool() Err
    let prev = std::env::var(pg_test_db::DATABASE_URL_ENV).ok();
    std::env::remove_var(pg_test_db::DATABASE_URL_ENV);

    assert!(
        !pg_test_db::pg_available().await,
        "pg_available() must be false when DATABASE_URL unset"
    );
    assert!(
        pg_test_db::pg_pool().await.is_err(),
        "pg_pool() must err when DATABASE_URL unset"
    );

    if let Some(v) = prev {
        std::env::set_var(pg_test_db::DATABASE_URL_ENV, v);
    }
}

#[test]
fn pg_test_db_database_url_env_name_is_stable() {
    // 防 fixture env var 名与 sqlx 默认脱节 (强约束, 改名前需先改 sqlx::test 注入路径)
    assert_eq!(pg_test_db::DATABASE_URL_ENV, "DATABASE_URL");
    assert_eq!(
        pg_test_db::DEFAULT_POOL_SIZE, 8,
        "default pool size must be tuned for CI single-process 6 域 tests"
    );
}

// ============================================================================
// 多线程并发 publish InMemoryNatsMock: 线程安全 + 计数正确
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn inmemory_nats_concurrent_publish_thread_safe() {
    // 4 个 task × 50 次 publish 同 subject, 总 200 条, received_count 必须 200
    let nats = Arc::new(InMemoryNatsMock::new());
    let subject = "concurrent.subject";
    let mut handles = Vec::new();
    for task_id in 0..4 {
        let n = Arc::clone(&nats);
        let s = subject.to_string();
        handles.push(tokio::spawn(async move {
            for i in 0..50 {
                let payload = format!("task={task_id},i={i}");
                n.publish(&s, payload.as_bytes()).await.unwrap();
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(
        nats.received_count(subject),
        200,
        "concurrent publishes must be counted accurately"
    );
    let msgs = nats.subscribe(subject).await.unwrap();
    assert_eq!(msgs.len(), 200);
}

// ============================================================================
// FixtureBuilder + serde JSON 跨 crate 字符串传递
// ============================================================================

#[test]
fn fixture_builder_chained_serde_cross_crate_shape() {
    // 模拟 5 域 crate 边界: 生产端 builder 出 fixture → 序列化为 JSON string
    // → 消费端 (另一域) 反序列化为同类型 → 字段一致
    let p = FixtureBuilder::new(fixture::player())
        .with_name("CrossCrate")
        .with_level(77)
        .build();
    let wire = serde_json::to_string(&p).expect("serialize player");
    let back: fixture::PlayerFixture =
        serde_json::from_str(&wire).expect("deserialize player across crate");
    assert_eq!(p, back);
    assert_eq!(back.name, "CrossCrate");
    assert_eq!(back.level, 77);

    // 5 域 fixture 同框 JSON 数组 (模拟跨域聚合测试 fixture)
    let m = FixtureBuilder::new(fixture::match_game("p1"))
        .with_score(42)
        .with_status("Completed")
        .build();
    let s = FixtureBuilder::new(fixture::social_message("p1", "p2"))
        .with_message("ping")
        .build();
    let a = FixtureBuilder::new(fixture::admin_action("a1", "audit", "p1"))
        .with_action("view")
        .build();
    let arr = json!([
        serde_json::to_value(&m).unwrap(),
        serde_json::to_value(&s).unwrap(),
        serde_json::to_value(&a).unwrap(),
    ]);
    let arr_str = serde_json::to_string(&arr).expect("serialize array");
    let parsed: serde_json::Value =
        serde_json::from_str(&arr_str).expect("deserialize array");
    let items = parsed.as_array().expect("must be array");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["status"], "Completed");
    assert_eq!(items[1]["message"], "ping");
    assert_eq!(items[2]["action"], "view");
}
