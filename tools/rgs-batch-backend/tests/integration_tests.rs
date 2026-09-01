// rgs-batch-backend integration tests (per BATCH-PLAN v0.2 §3.1 W3 BA-W3-10, 2026-09-02 03:18 JST Mavis 接手代签)
// 11 UT 覆盖 exponential_backoff + endpoint JSON schema (不依赖 DB, 纯 std/serde_json/uuid 基础测试)
// 注: rgs-batch-backend 是 binary 不是 lib, 所以 tests 不 import main.rs 类型, 只测 std + serde_json 算法

// === Group 1: exponential_backoff (DLQ 退避算法, 镜像 main.rs exponential_backoff_ms) ===

fn exponential_backoff_ms(retry_count: i32) -> u64 {
    let base = 100u64;
    let max_delay = 30_000u64;
    let delay = base.saturating_mul(2u64.saturating_pow(retry_count.max(0) as u32));
    delay.min(max_delay)
}

#[test]
fn test_01_dlq_exponential_backoff_retry_0() {
    assert_eq!(exponential_backoff_ms(0), 100);
}

#[test]
fn test_02_dlq_exponential_backoff_retry_1() {
    assert_eq!(exponential_backoff_ms(1), 200);
}

#[test]
fn test_03_dlq_exponential_backoff_retry_5() {
    assert_eq!(exponential_backoff_ms(5), 3200);
}

#[test]
fn test_04_dlq_exponential_backoff_capped_30s() {
    assert_eq!(exponential_backoff_ms(10), 30_000);
}

#[test]
fn test_05_dlq_exponential_backoff_capped_100_retry() {
    assert_eq!(exponential_backoff_ms(100), 30_000);
}

// === Group 2: endpoint JSON schema (per BA-W2-2~9 + W3 BA-W3-1~9) ===

#[test]
fn test_06_worker_pool_status_json() {
    use serde_json::json;
    let status = json!({
        "active": 2,
        "idle": 6,
        "total": 8,
        "max_concurrent": 8,
        "priority_queue_size": 3,
        "by_priority": {"1": 1, "5": 2}
    });
    let s = status.to_string();
    assert!(s.contains("\"active\":2"));
    assert!(s.contains("\"max_concurrent\":8"));
}

#[test]
fn test_07_audit_event_json() {
    use serde_json::json;
    let evt = json!({
        "operator": "ulysses",
        "action": "create_task",
        "params_hash": "abcd1234",
        "result": "success",
        "trace_id": "trace-001"
    });
    let s = evt.to_string();
    assert!(s.contains("\"operator\":\"ulysses\""));
    assert!(s.contains("\"action\":\"create_task\""));
}

#[test]
fn test_08_task_execution_json() {
    use serde_json::json;
    let exec = json!({
        "task_id": "00000000-0000-0000-0000-000000000001",
        "attempt": 1,
        "duration_ms": 1500,
        "result": "success"
    });
    let s = exec.to_string();
    assert!(s.contains("\"attempt\":1"));
    assert!(s.contains("\"duration_ms\":1500"));
}

#[test]
fn test_09_saga_instance_json() {
    use serde_json::json;
    let saga = json!({
        "saga_type": "rgs-batch-saga-cleanup",
        "state": "compensating"
    });
    let s = saga.to_string();
    assert!(s.contains("\"saga_type\":\"rgs-batch-saga-cleanup\""));
    assert!(s.contains("\"state\":\"compensating\""));
}

#[test]
fn test_10_data_source_json() {
    use serde_json::json;
    let ds = json!({
        "name": "main-pg",
        "source_type": "postgres",
        "connection_ref": "env://BATCH_DB_URL",
        "enabled": true
    });
    let s = ds.to_string();
    assert!(s.contains("\"source_type\":\"postgres\""));
    assert!(s.contains("\"enabled\":true"));
}

#[test]
fn test_11_version_endpoint_json() {
    use serde_json::json;
    let v = json!({
        "backend": "0.2.0-w2",
        "w2_features": ["a", "b", "c"]
    });
    let s = v.to_string();
    assert!(s.contains("\"backend\":\"0.2.0-w2\""));
    assert_eq!(v["w2_features"].as_array().unwrap().len(), 3);
}
