//! S5 §3 真 NATS e2e 测试 (W3) - 简化版
//!
//! 用 k3s 内 nats-0 pod (port-forward 14222) 真跑 NATS 客户端
//! 覆盖: connect / publish / subscribe / request-reply
//!
//! 关联: docs/00-基准与治理/RGS-TST-S5-outbox-NATS-IT-设计书.md
//!
//! 前置: k3s kubectl port-forward -n rust-game-server nats-0 14222:4222 已建立
//! 跑测: WSL 端
//!   source scripts/db-url.sh postgres-superuser 15432
//!   cargo test -p gm-backend --test it_outbox_nats_e2e -- --include-ignored
//!
//! 已知缺口 (per 8/28 设计): 7 个 mock 测试已实装 (it_outbox_nats.rs),
//! JetStream 持久化测试待 Step 3+ 补 (依赖更复杂 API)

use futures_util::StreamExt;
use std::time::Duration;

const NATS_URL: &str = "nats://localhost:14222";

/// helper: 检查 NATS 可达性
async fn nats_available() -> bool {
    matches!(
        tokio::time::timeout(Duration::from_secs(2), async_nats::connect(NATS_URL)).await,
        Ok(Ok(_))
    )
}

#[tokio::test]
async fn nats_connect_succeeds() {
    if !nats_available().await {
        eprintln!("SKIP: k3s nats-0 port-forward not available (run: k3s kubectl port-forward -n rust-game-server nats-0 14222:4222)");
        return;
    }
    let client = async_nats::connect(NATS_URL).await.expect("nats connect");
    // server_info() 直接返 ServerInfo, 验证 max_payload > 0 即认为 NATS 就绪
    let info = client.server_info();
    assert!(info.max_payload > 0, "max_payload must be > 0");
    assert!(!info.server_id.is_empty(), "server_id must be non-empty");
}

#[tokio::test]
async fn nats_publish_and_subscribe() {
    if !nats_available().await {
        eprintln!("SKIP: nats not available");
        return;
    }
    let client = async_nats::connect(NATS_URL).await.expect("nats connect");
    let subject = format!("test.pubsub.{}", uuid::Uuid::new_v4());
    let mut sub = client.subscribe(subject.clone()).await.expect("subscribe");
    client
        .publish(subject.clone(), "hello-nats".into())
        .await
        .expect("publish");
    let received = tokio::time::timeout(Duration::from_secs(3), sub.next())
        .await
        .expect("subscribe timeout")
        .expect("subscribe message");
    assert_eq!(&received.payload[..], b"hello-nats");
}

#[tokio::test]
async fn nats_request_reply() {
    if !nats_available().await {
        eprintln!("SKIP: nats not available");
        return;
    }
    let client = async_nats::connect(NATS_URL).await.expect("nats connect");
    let subject = format!("test.reqrep.{}", uuid::Uuid::new_v4());
    let mut sub = client.subscribe(subject.clone()).await.expect("subscribe");

    // responder
    let responder_client = client.clone();
    let responder = tokio::spawn(async move {
        if let Some(msg) = sub.next().await {
            let reply_subject = msg.reply.unwrap_or_else(|| "test.anon".into());
            let reply = format!("reply:{}", String::from_utf8_lossy(&msg.payload[..]));
            responder_client
                .publish(reply_subject, reply.into())
                .await
                .ok();
        }
    });

    // requester
    let response = client
        .request(subject, "ping".into())
        .await
        .expect("request");
    let response_text = String::from_utf8_lossy(&response.payload[..]).to_string();
    assert!(response_text.starts_with("reply:"), "got: {response_text}");
    responder.abort();
}
