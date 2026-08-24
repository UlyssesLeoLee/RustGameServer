//! InMemoryNatsMock 实质实现的集成测试
//!
//! 验证:
//! 1. publish 存到 in-memory store
//! 2. subscribe 取出累积消息 (FIFO)
//! 3. received_count 计数正确
//! 4. NoopMock backward compat (新方法返回空 / 0, 不破坏现有调用方)
//!
//! per RGS-SPEC-000 §2.4 + RGS-IMPL-001 §3
//! 规范来源: NATS 跨域事件总线 (DTL-021~025 + ARC-051)

use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};
// NoopMock 已 deprecated (PG fixture 替代), 仅 backward-compat test 引用, 需允许警告
#[allow(deprecated)]
use rgs_testkit::mock::NoopMock;

#[tokio::test]
async fn inmemory_nats_publish_then_subscribe() {
    let m = InMemoryNatsMock::new();
    m.publish("player.events", br#"{"event":"login"}"#)
        .await
        .unwrap();
    m.publish("player.events", br#"{"event":"logout"}"#)
        .await
        .unwrap();

    let msgs = m.subscribe("player.events").await.unwrap();
    assert_eq!(msgs.len(), 2, "应累积 2 条消息 (FIFO)");
    assert_eq!(msgs[0], br#"{"event":"login"}"#);
    assert_eq!(msgs[1], br#"{"event":"logout"}"#);
}

#[tokio::test]
async fn inmemory_nats_subscribe_empty_subject() {
    let m = InMemoryNatsMock::new();
    let msgs = m.subscribe("nonexistent").await.unwrap();
    assert_eq!(
        msgs.len(),
        0,
        "未 publish 过的 subject 应返回空 vec, 不应 panic"
    );
}

#[tokio::test]
async fn inmemory_nats_received_count() {
    let m = InMemoryNatsMock::new();
    m.publish("economy.tx", b"{}").await.unwrap();
    m.publish("economy.tx", b"{}").await.unwrap();
    m.publish("economy.tx", b"{}").await.unwrap();
    assert_eq!(m.received_count("economy.tx"), 3);
    assert_eq!(m.received_count("nonexistent"), 0);
}

#[tokio::test]
async fn noop_nats_backward_compat() {
    // NoopMock 已 deprecated (PG fixture 替代), 但新加的 NATS 方法
    // 必须 backward compat: 不实际存, 计数 / 订阅都返 0 / 空, 不 panic.
    #[allow(deprecated)]
    let m = NoopMock;
    m.publish("test", b"{}").await.unwrap();
    let msgs = m.subscribe("test").await.unwrap();
    assert_eq!(msgs.len(), 0, "NoopMock 不实际存消息");
    assert_eq!(m.received_count("test"), 0, "NoopMock 计数始终为 0");
}
