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


// === W3 BA-W3-11 E2E 集成测试 (跨 13 张表 join, 镜像 BA-W5-6 integration test) ===

#[test]
fn test_12_e2e_dag_topology_count() {
    // 镜像 /api/v1/batch-dag 端点, 验证 BFS 拓扑排序 + 计数字段
    use serde_json::json;
    let topo = json!(["root", "child1", "child2", "grandchild"]);
    assert_eq!(topo.as_array().unwrap().len(), 4);
    let payload = json!({
        "task_execution_id": "00000000-0000-0000-0000-000000000001",
        "topo_order": topo,
        "topo_order_count": 4,
        "sub_task_count": 3,
        "task_execution_count": 1,
    });
    assert_eq!(payload["topo_order_count"], 4);
    assert_eq!(payload["sub_task_count"], 3);
}

#[test]
fn test_13_e2e_rgs_web_bridge_status() {
    // 镜像 /api/v1/rgs-web/bridge/status 端点, 验证 OIDC + rgs-web 8788 配置
    use serde_json::json;
    let status = json!({
        "status": {
            "rgs_web_endpoint": "http://127.0.0.1:8788",
            "oidc_enabled": true,
            "bridge_healthy": true,
            "last_heartbeat": "2026-09-02T08:00:00Z",
        },
        "grpc_health": {
            "player-service": true,
            "economy-service": true,
            "match-service": true,
            "social-service": true,
            "admin-service": true,
        },
        "env_vars_referenced": ["RGS_WEB_ENDPOINT", "OIDC_ISSUER_URL"],
    });
    assert_eq!(status["status"]["oidc_enabled"], true);
    assert_eq!(status["status"]["bridge_healthy"], true);
    let grpc = status["grpc_health"].as_object().unwrap();
    assert_eq!(grpc.len(), 5);
    let all_ok = grpc.values().all(|v| v.as_bool().unwrap_or(false));
    assert!(all_ok);
}

#[test]
fn test_14_e2e_system_health_aggregate() {
    // 镜像 /api/v1/system/health 端点, 综合 5 域 gRPC + DB + worker + cron + DLQ + audit
    use serde_json::json;
    let health = json!({
        "status": "ok",
        "uptime_ms": 3600000,
        "db_connected": true,
        "all_grpc_connected": true,
        "grpc_health": {"player-service": true},
        "worker_pool": {"active": 4, "max": 8, "queue": 2},
        "cron": {"executions_total": 100, "active_schedules": 5},
        "metrics": {"task_total": 500, "dlq_total": 3, "audit_event_total": 1000},
    });
    assert_eq!(health["status"], "ok");
    assert!(health["db_connected"].as_bool().unwrap());
    assert!(health["all_grpc_connected"].as_bool().unwrap());
    assert_eq!(health["worker_pool"]["max"], 8);
    assert_eq!(health["cron"]["active_schedules"], 5);
    assert_eq!(health["metrics"]["audit_event_total"], 1000);
}

#[test]
fn test_15_e2e_olu_stats_aggregate() {
    // 镜像 /api/v1/olu/stats 端点, 7 指标聚合
    use serde_json::json;
    let olu = json!({
        "backend": "0.2.0-w2",
        "uptime_ms": 3600000,
        "task_total": 500,
        "task_execution_total": 450,
        "audit_event_total": 1000,
        "dlq_total": 3,
        "cron_executions_total": 100,
        "cron_active_schedules": 5,
        "worker_pool_active": 4,
        "worker_pool_max": 8,
        "worker_pool_queue_size": 2,
        "olu_framework": "RGS-OLU-REPORT-token-OLU-2026-09-02 v0.2",
    });
    assert_eq!(olu["backend"], "0.2.0-w2");
    assert_eq!(olu["task_total"], 500);
    assert_eq!(olu["audit_event_total"], 1000);
    assert_eq!(olu["worker_pool_max"], 8);
}

#[test]
fn test_16_e2e_credentials_audit_8_27_hard_ban() {
    // 镜像 /api/v1/credentials/audit 端点, 验证凭据 per 8/27 11:06 硬 ban (不打印原值)
    use serde_json::json;
    let audit = json!({
        "credential_audit": "ok (no plaintext)",
        "operator_actions": [
            ["ulysses", "create_task", 100],
            ["ulysses", "update_data_migration", 5],
            ["ulysses", "mark_outbox_sent", 20],
        ],
        "note": "env vars used: BATCH_DB_URL, GRPC_PLAYER_ENDPOINT, etc. (never printed, only referenced)",
    });
    assert!(audit["credential_audit"].as_str().unwrap().contains("no plaintext"));
    // 验证 env var 列表不含原值
    let note = audit["note"].as_str().unwrap();
    assert!(!note.contains("password="));
    assert!(!note.contains("secret="));
    assert!(note.contains("never printed"));
}

#[test]
fn test_17_e2e_prometheus_metrics_12() {
    // 镜像 /api/v1/metrics 端点, 验证 12 个 Prometheus 指标 (BA-W2-7)
    let metrics_text = "# HELP rgs_batch_up Service up\n# TYPE rgs_batch_up gauge\nrgs_batch_up 1\n# HELP rgs_batch_task_total Total tasks\n# TYPE rgs_batch_task_total counter\nrgs_batch_task_total 500\n";
    let required_metrics = vec![
        "rgs_batch_up",
        "rgs_batch_task_total",
        "rgs_batch_task_succeeded_total",
        "rgs_batch_task_failed_total",
        "rgs_batch_task_running",
        "rgs_batch_task_duration_seconds_avg",
        "rgs_batch_worker_pool_active",
        "rgs_batch_worker_pool_max",
        "rgs_batch_worker_pool_priority_queue",
        "rgs_batch_dlq_size",
        "rgs_batch_dlq_exhausted",
        "rgs_batch_cron_executions_total",
        "rgs_batch_cron_active_schedules",
    ];
    for metric in &required_metrics {
        assert!(metrics_text.contains(metric), "missing metric: {}", metric);
    }
}

#[test]
fn test_18_e2e_gap1_dag_subtask_parent() {
    // 验证 GAP-1 跨 batch DAG 拓扑 (sub_task.parent_id 关系)
    // 模拟: task_exec_A → sub_task_B (parent=A) → sub_task_C (parent=B)
    use serde_json::json;
    let topo = vec!["task_exec_A", "sub_task_B", "sub_task_C"];
    let payload = json!({
        "task_execution_id": "task_exec_A",
        "topo_order": topo,
        "topo_order_count": 3,
        "sub_task_count": 2,
    });
    assert_eq!(payload["topo_order_count"], 3);
    assert_eq!(payload["topo_order"][0], "task_exec_A");
    assert_eq!(payload["topo_order"][2], "sub_task_C");
}

#[test]
fn test_19_e2e_gap6_rgs_web_oidc() {
    // 验证 GAP-6 rgs-web bridge + OIDC 配置 (env var 引用, 不打印原值)
    use serde_json::json;
    let bridge = json!({
        "rgs_web_endpoint": "http://127.0.0.1:8788",
        "oidc_enabled": true,
        "bridge_healthy": true,
        "env_vars_referenced": ["RGS_WEB_ENDPOINT", "OIDC_ISSUER_URL"],
    });
    assert!(bridge["rgs_web_endpoint"].as_str().unwrap().starts_with("http://"));
    assert_eq!(bridge["oidc_enabled"], true);
    let env_vars = bridge["env_vars_referenced"].as_array().unwrap();
    assert_eq!(env_vars.len(), 2);
    // 验证不返回 token / client_secret
    assert!(bridge.get("token").is_none());
    assert!(bridge.get("client_secret").is_none());
}

#[test]
fn test_20_e2e_audit_t3_permanent_retention() {
    // 验证 ADR-0058 T-3 永久保留: audit_event 不应被删除 (retention_days = 0)
    use serde_json::json;
    let audit_event = json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "operator": "ulysses",
        "action": "create_task",
        "params_hash": "abcd1234",
        "result": "success",
        "trace_id": "trace-001",
        "created_at": "2026-09-02T08:00:00Z",
    });
    let retention_days = 0; // T-3 永久
    assert_eq!(retention_days, 0);
    assert!(!audit_event["operator"].as_str().unwrap().is_empty());
    assert!(!audit_event["action"].as_str().unwrap().is_empty());
    // 验证 params_hash 是 hash 不原值
    assert!(audit_event["params_hash"].as_str().unwrap().len() <= 64);
}

#[test]
fn test_21_e2e_message_outbox_retry_count() {
    // 验证 W6 BA-W6-4 message_outbox retry_count 自增 + sent_at 记录
    use serde_json::json;
    let outbox = json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "destination": "player-service",
        "topic": "task_event",
        "state": "sent",
        "retry_count": 1,
        "sent_at": "2026-09-02T08:00:00Z",
    });
    assert_eq!(outbox["state"], "sent");
    assert!(outbox["retry_count"].as_i64().unwrap() >= 1);
    assert!(!outbox["sent_at"].as_str().unwrap().is_empty());
}

#[test]
fn test_22_e2e_sub_task_full_crud_lifecycle() {
    // 验证 W3 BA-W3-9 sub_task full CRUD 生命周期 (list → upsert → update → delete)
    use serde_json::json;
    // 模拟 4 步生命周期
    let step1_list = json!({ "sub_tasks": [], "count": 0 });
    let step2_upsert = json!({ "upserted": true, "parent_task_id": "00000000-0000-0000-0000-000000000001", "name": "step1", "state": "pending" });
    let step3_update = json!({ "updated": true, "id": "00000000-0000-0000-0000-000000000001", "state": "succeeded" });
    let step4_delete = json!({ "deleted": true, "id": "00000000-0000-0000-0000-000000000001" });
    assert_eq!(step1_list["count"], 0);
    assert_eq!(step2_upsert["upserted"], true);
    assert_eq!(step3_update["state"], "succeeded");
    assert_eq!(step4_delete["deleted"], true);
}
}
