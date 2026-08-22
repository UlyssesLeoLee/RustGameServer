# RGS-TST-100 Saga 事务系统测试设计

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-100 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-21 |
| 最终更新日 | 2026-08-21 |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-REQ-100 / RGS-BAS-100 / RGS-DTL-100~102 / RGS-OPS-100 / RGS-GOBS-100 / RGS-SEC-100 / **RGS-IMPL-100** (V 模型对子) |
| 配套标准 | IPA 共通フレーム 2013 + 150 工程日本 SI 业界标准；V 模型映射：**ST ↔ REQ-100** / **IT ↔ BAS-100 + DTL-100~102** / **UT ↔ DTL-100~102** |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。UT 状态机/补偿/重试 3 大类 + IT 4 业务 Saga 端到端 + ST 4 类故障恢复 + 性能 NFR + 测试矩阵。 |

---

## 0. 文档目的

定义 Saga 事务系统的**测试设计**，V 模型对应：

| V 模型 | 测试类型 | 覆盖 | 测试设计文档 |
|---|---|---|---|
| **ST** ↔ REQ-100 | 系统测试 | 端到端 + NFR + 故障恢复 | 本文档 §3 |
| **IT** ↔ BAS-100 | 集成测试 | 微服务集成 + Saga 端到端 | 本文档 §2 |
| **UT** ↔ DTL-100~102 | 单元测试 | 状态机 + 补偿 + 重试 + DB + gRPC | 本文档 §1 |

**测试环境**（per RGS-IMPL-100）：

- Rust 1.98 stable + cargo test
- testcontainers（PostgreSQL 18.6 + NATS JetStream 真实容器）
- K3s dev cluster（WSL2 native, per DEC-010）跑系统测试
- cargo-llvm-cov 覆盖率 ≥ 80%

---

## 1. UT（单元测试）

### 1.1 状态机测试（state_machine_test.rs）

```rust
// per RGS-DTL-102 §1 状态机

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SagaState;

    #[test]
    fn test_pending_to_running() {
        let mut saga = SagaInstance::new_pending(...);
        saga.acquire(...);
        assert_eq!(saga.state, SagaState::Running);
    }

    #[test]
    fn test_running_to_waiting_on_command_publish() {
        // Running → Waiting: command published, waiting for response
    }

    #[test]
    fn test_waiting_to_running_on_step_success() {
        // Waiting → Running: step succeeded, ready for next step
    }

    #[test]
    fn test_waiting_to_retrying_on_timeout() {
        // Waiting → Retrying: step timeout
    }

    #[test]
    fn test_retrying_to_waiting_on_retry() {
        // Retrying → Waiting: retry published
    }

    #[test]
    fn test_running_to_compensating_on_step_failure() {
        // 任一 step 失败 → 触发补偿
    }

    #[test]
    fn test_compensating_to_compensated_on_all_success() {
        // 所有补偿成功 → Compensated (终态)
    }

    #[test]
    fn test_compensating_to_failed_on_compensation_failure() {
        // 补偿自身失败 → Failed (Manual Intervention)
    }

    #[test]
    fn test_running_to_timeout_on_expires() {
        // expires_at 已过 → Timeout
    }

    #[test]
    fn test_terminal_states() {
        for state in [Compensated, Completed, Failed, Canceled, Expired] {
            assert!(state.is_terminal());
            assert!(!state.is_recoverable());
        }
    }

    #[test]
    fn test_recoverable_states() {
        for state in [Running, Waiting, Retrying, Compensating] {
            assert!(state.is_recoverable());
            assert!(!state.is_terminal());
        }
    }

    // Property-based test
    proptest! {
        #[test]
        fn test_state_invariants(state: SagaState) {
            // 状态机不变量：终态不可变
            if state.is_terminal() {
                prop_assert!(!state.is_recoverable());
            }
        }
    }
}
```

**覆盖率目标**：状态机逻辑 100%，含全部合法 + 非法转移。

### 1.2 补偿逻辑测试（compensation_test.rs）

```rust
// per RGS-DTL-100 §1.3 失败补偿

#[cfg(test)]
mod tests {
    #[test]
    fn test_reverse_order_compensation() {
        // 步骤 0/1/2/3/4 中第 3 失败 → 补偿 0/1/2 逆序
        let mut saga = setup_saga_with_steps(5);
        saga.execute_step(0, Ok).await;
        saga.execute_step(1, Ok).await;
        saga.execute_step(2, Ok).await;
        saga.execute_step(3, Err("inventory_full")).await;
        // 期望: 补偿 2, 1, 0 逆序
        saga.run_compensation().await;
        assert_eq!(saga.compensation_log, vec![2, 1, 0]);
    }

    #[test]
    fn test_compensation_uses_comp_action() {
        // 步骤 1 的补偿 action 应是定义里 step.compensation
        let definition = SagaDefinition {
            steps: vec![
                Step { action: "ReserveCurrency", compensation: Some("ReleaseCurrencyReserve"), .. },
                Step { action: "GrantItem", compensation: Some("RevokeItem"), .. },
            ],
            ..Default::default()
        };
        let mut saga = SagaInstance::new(definition);
        saga.execute_step(0, Ok).await;
        saga.execute_step(1, Err("...")).await;
        saga.run_compensation().await;
        // 期望: 调用 ReleaseCurrencyReserve (来自 step 0)
        assert_eq!(saga.compensation_actions_called, vec!["ReleaseCurrencyReserve"]);
    }

    #[test]
    fn test_compensation_failure_marks_manual_intervention() {
        // 补偿自身失败 → state=FAILED + Manual Intervention Queue
        let mut saga = setup_saga_with_failing_compensation();
        saga.execute_step(0, Ok).await;
        saga.execute_step(1, Err("...")).await;
        saga.run_compensation().await; // comp 0 失败
        assert_eq!(saga.state, SagaState::Failed);
        assert!(saga.requires_manual_intervention());
    }

    #[test]
    fn test_idempotent_compensation() {
        // 重复调用补偿不应有副作用
        let mut saga = setup_saga();
        saga.execute_step(0, Ok).await;
        saga.execute_step(1, Err("...")).await;
        saga.run_compensation().await;
        let state1 = saga.clone();
        saga.run_compensation().await; // 第二次
        assert_eq!(saga.state, state1.state);
    }
}
```

### 1.3 重试 / 退避测试（retry_test.rs）

```rust
// per RGS-DTL-102 §4 重试 + §1 状态机 RETRYING

#[cfg(test)]
mod tests {
    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy {
            max_retries: 5,
            initial_backoff: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(60),
        };
        assert_eq!(policy.backoff(0), Duration::from_secs(1));
        assert_eq!(policy.backoff(1), Duration::from_secs(2));
        assert_eq!(policy.backoff(2), Duration::from_secs(4));
        assert_eq!(policy.backoff(3), Duration::from_secs(8));
        assert_eq!(policy.backoff(4), Duration::from_secs(16));
    }

    #[test]
    fn test_max_backoff_cap() {
        let policy = RetryPolicy { max_backoff: Duration::from_secs(60), .. };
        assert_eq!(policy.backoff(100), Duration::from_secs(60));
    }

    #[test]
    fn test_retry_until_max_then_fail() {
        let mut saga = setup_saga();
        for i in 0..5 {
            saga.execute_step(0, Err("transient")).await;
            assert_eq!(saga.state, SagaState::Retrying);
        }
        // 第 5 次重试后，state 应为 FAILED
        assert_eq!(saga.state, SagaState::Failed);
    }
}
```

### 1.4 数据库 + OCC 测试（db_test.rs）

```rust
#[cfg(test)]
mod tests {
    /// 真实 PostgreSQL 容器（testcontainers）
    #[tokio::test]
    async fn test_acquire_saga_with_skip_locked() {
        let pool = setup_pg_container().await;
        run_migrations(&pool).await;
        let saga_id = create_test_saga(&pool, "RUNNING").await;

        // Pod A 抢占
        let token_a = SagaInstance::try_acquire(&pool, saga_id, "pod-a", 60).await.unwrap();
        assert!(token_a.is_some());

        // Pod B 同时抢占（同 grace period 内）应失败
        let token_b = SagaInstance::try_acquire(&pool, saga_id, "pod-b", 60).await.unwrap();
        assert!(token_b.is_none());
    }

    #[tokio::test]
    async fn test_fence_token_increases_monotonically() {
        let pool = setup_pg_container().await;
        let saga_id = create_test_saga(&pool, "RUNNING").await;

        let token1 = SagaInstance::try_acquire(&pool, saga_id, "pod-a", 60).await.unwrap().unwrap();
        let token2 = SagaInstance::renew(&pool, saga_id, "pod-a").await.unwrap().unwrap();
        let token3 = SagaInstance::renew(&pool, saga_id, "pod-a").await.unwrap().unwrap();

        assert!(token2 > token1);
        assert!(token3 > token2);
    }

    #[tokio::test]
    async fn test_write_with_stale_fence_token_fails() {
        // Pod A 持 token=42，写 saga_step 应成功
        // Pod A 失去 leadership（stale），token 仍是 42
        // Pod B 抢占，token=43
        // Pod A 写 saga_step WHERE fence_token=42 → 0 rows
    }
}
```

### 1.5 gRPC 测试

```rust
#[tokio::test]
async fn test_grpc_start_saga() {
    let mut client = SagaServiceClient::connect("http://localhost:50051").await.unwrap();
    let response = client.start_saga(Request::new(StartSagaRequest {
        saga_type: "PurchaseFlow".into(),
        payload: serde_json::json!({"player_id": "P-1", "item_id": "I-1"}),
        initiator: "test".into(),
        trace_id: "ABC".into(),
    })).await.unwrap();
    let saga_id = response.into_inner().saga_id;
    assert!(!saga_id.is_empty());
}

#[tokio::test]
async fn test_mtls_required() {
    // 尝试无 mTLS 连接 → 应被拒绝
    let result = SagaServiceClient::connect("http://localhost:50051").await;
    // 期望: TLS handshake 失败
}
```

### 1.6 Outbox + Inbox 单元测试

```rust
#[tokio::test]
async fn test_outbox_publish_atomic_with_domain_update() {
    // BEGIN; UPDATE items; INSERT outbox; COMMIT
    // 期望: 业务更新 + outbox event 一次 COMMIT
    // 即使 Worker crash，outbox 仍能 retry publish
}

#[tokio::test]
async fn test_inbox_dedup_by_event_id() {
    // 同一 event_id 投递 2 次 → Inbox 表 PRIMARY KEY 拒绝
    // 第二次视为已处理
}

#[tokio::test]
async fn test_idempotency_key_uniqueness() {
    // (saga_id, step_index) 唯一 → 重复 command 视为已处理
}
```

---

## 2. IT（集成测试）

### 2.1 Purchase Saga 端到端（purchase_saga_test.rs）

```rust
/// 真实 PostgreSQL 18.6 + NATS JetStream 容器
/// 真实 5 域微服务 mock（接收 gRPC + 处理 Inbox）
#[tokio::test]
async fn test_purchase_saga_happy_path() {
    let ctx = SagaTestContext::setup().await;
    ctx.create_player_with_balance("P-1", 1000).await;
    ctx.create_item_in_shop("I-1", 100).await;

    // Start saga
    let saga_id = ctx.runtime.start_saga("PurchaseFlow", json!({
        "player_id": "P-1",
        "item_id": "I-1",
        "qty": 1,
    }), "test").await.unwrap();

    // Wait for completion
    let state = ctx.wait_for_state(saga_id, SagaState::Completed, Duration::from_secs(10)).await;
    assert_eq!(state, SagaState::Completed);

    // Verify
    assert_eq!(ctx.economy.get_balance("P-1").await, 900); // 扣 100
    let items = ctx.inventory.get_items("P-1").await;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "I-1");
    let mails = ctx.mail.get_mails("P-1").await;
    assert_eq!(mails.len(), 1);
    assert!(mails[0].attach_item_ids.contains("I-1"));
}

#[tokio::test]
async fn test_purchase_saga_insufficient_funds_aborts() {
    // 余额不足 → 步骤 1 fail → SagaFailed（无补偿需要）
    let ctx = SagaTestContext::setup().await;
    ctx.create_player_with_balance("P-1", 50).await;  // 不够买 100 的
    ctx.create_item_in_shop("I-1", 100).await;

    let saga_id = ctx.runtime.start_saga("PurchaseFlow", json!({
        "player_id": "P-1",
        "item_id": "I-1",
    }), "test").await.unwrap();

    let state = ctx.wait_for_state(saga_id, SagaState::Failed, Duration::from_secs(10)).await;
    assert_eq!(state, SagaState::Failed);

    // 余额不变
    assert_eq!(ctx.economy.get_balance("P-1").await, 50);
    // 无新物品
    assert_eq!(ctx.inventory.get_items("P-1").await.len(), 0);
}

#[tokio::test]
async fn test_purchase_saga_inventory_full_compensates() {
    // 步骤 1 成功（货币预留），步骤 2 失败（库存满）→ 逆序补偿
    let ctx = SagaTestContext::setup().await;
    ctx.create_player_with_balance("P-1", 1000).await;
    ctx.create_item_in_shop("I-1", 100).await;
    ctx.inventory.set_capacity("P-1", 0).await;  // 满

    let saga_id = ctx.runtime.start_saga("PurchaseFlow", json!({
        "player_id": "P-1",
        "item_id": "I-1",
    }), "test").await.unwrap();

    let state = ctx.wait_for_state(saga_id, SagaState::Compensated, Duration::from_secs(10)).await;
    assert_eq!(state, SagaState::Compensated);

    // 货币预留已释放（balance 不变）
    assert_eq!(ctx.economy.get_balance("P-1").await, 1000);
    // 预留表无残留
    assert_eq!(ctx.economy.get_reserved("P-1").await, 0);
}

#[tokio::test]
async fn test_purchase_saga_idempotent() {
    // 重复执行同一 saga_id → 第二次视为 no-op（由 Inbox + idempotency_key 阻止）
}
```

### 2.2 Character Creation Saga 端到端（character_creation_test.rs）

```rust
#[tokio::test]
async fn test_character_creation_happy_path() {
    let ctx = SagaTestContext::setup().await;
    let account_id = ctx.create_account("A-1").await;

    let saga_id = ctx.runtime.start_saga("CharacterCreationFlow", json!({
        "account_id": account_id,
        "name": "Alice",
        "class": "warrior",
    }), "test").await.unwrap();

    let state = ctx.wait_for_state(saga_id, SagaState::Completed, Duration::from_secs(10)).await;
    assert_eq!(state, SagaState::Completed);

    // 验证：角色 + 初始装备 + 初始货币 + 欢迎邮件
    let character = ctx.character.get_by_account(account_id).await;
    assert_eq!(character.name, "Alice");
    assert_eq!(character.class, "warrior");
    assert_eq!(ctx.inventory.get_items(character.id).await.len(), 3); // weapon + armor + potion
    assert_eq!(ctx.economy.get_balance(character.id).await, 1000); // gold
    assert!(ctx.mail.get_mails(character.id).await.iter().any(|m| m.subject == "欢迎来到游戏"));
}

#[tokio::test]
async fn test_character_creation_mail_failure_compensates() {
    // 步骤 4（mail）失败 → 逆序补偿 3/2/1
    ctx.mail.set_failing(true);
    let saga_id = ctx.runtime.start_saga("CharacterCreationFlow", ...).await;
    let state = ctx.wait_for_state(saga_id, SagaState::Compensated, ...).await;
    // 角色被删除 + 物品被回收 + 货币预留被释放
}
```

### 2.3 Reward Saga 端到端（reward_saga_test.rs）

```rust
#[tokio::test]
async fn test_reward_saga_happy_path() {
    // MatchFinished → 4 步骤全成功 → SagaCompleted
}

#[tokio::test]
async fn test_reward_saga_compensation_fails_manual_intervention() {
    // 比赛已结束不可回滚（per RGS-DTL-100 §3.3）
    // 任意步骤失败 → Manual Intervention Queue（不补偿 match）
}

#[tokio::test]
async fn test_reward_saga_corrective_event() {
    // GM 介入后，发出 Corrective Event（手工补发）
    // 验证：玩家收到 Corrective 通知
}
```

### 2.4 Outbox + Inbox 集成（outbox_inbox_test.rs）

```rust
#[tokio::test]
async fn test_outbox_publishes_to_nats() {
    let ctx = SagaTestContext::setup().await;
    ctx.economy.grant_currency("P-1", 100, "test").await;

    // Outbox 应有 PENDING 行
    let pending = ctx.outbox_repo.get_pending("economy", 10).await;
    assert!(pending.iter().any(|e| e.event_type == "CurrencyGranted"));

    // Worker 发布
    ctx.outbox_worker.run_once().await;

    // Outbox 应改为 PUBLISHED
    let pending_after = ctx.outbox_repo.get_pending("economy", 10).await;
    assert!(pending_after.is_empty());

    // NATS 应有 event
    let msgs = ctx.nats.drain_messages("EVENT.economy.granted").await;
    assert_eq!(msgs.len(), 1);
}

#[tokio::test]
async fn test_inbox_dedup_prevents_double_processing() {
    // 同一 event 投递 2 次 → Inbox PRIMARY KEY 拒绝
    // 业务处理仅 1 次
}

#[tokio::test]
async fn test_atomic_outbox_with_business() {
    // BEGIN; business; INSERT outbox; COMMIT
    // Worker crash → restart → outbox PENDING 行被重新发布
}
```

---

## 3. ST（系统测试）

### 3.1 K3s Pod Crash Recovery（pod_crash_recovery_test.rs）

```rust
/// 在 K3s dev cluster 上跑
#[tokio::test]
async fn test_pod_crash_recovery_continues_saga() {
    let ctx = SystemTestContext::setup_k3s().await;
    
    // 启动 PurchaseFlow saga
    let saga_id = ctx.runtime.start_saga("PurchaseFlow", json!({"player_id": "P-1", "item_id": "I-1"}), "test").await.unwrap();
    
    // 等待到 step 3/6（grant-currency 已成功，正在 grant-item）
    ctx.wait_for_step(saga_id, 2, Duration::from_secs(5)).await;
    
    // K3s 杀 saga-runtime pod（强制 SIGKILL）
    let pod_name = ctx.runtime_pod_name();
    ctx.kubectl("delete pod", &pod_name, "--grace-period=0", "--force").await;
    
    // 等待新 pod 启动
    ctx.wait_for_runtime_ready(Duration::from_secs(60)).await;
    
    // Saga 应继续
    let final_state = ctx.wait_for_state(saga_id, SagaState::Completed, Duration::from_secs(60)).await;
    assert_eq!(final_state, SagaState::Completed);
    
    // 验证：所有 4 步骤都被执行
    let steps = ctx.get_saga_steps(saga_id).await;
    assert!(steps.iter().all(|s| s.state == "SUCCESS"));
}

#[tokio::test]
async fn test_pod_crash_recovery_releases_fence_token() {
    // 杀 pod → 新 pod 接管 → fence_token 增加
}

#[tokio::test]
async fn test_two_pods_no_duplicate_driving() {
    // 启动 2 个 pod A + B（手动 deployment replicas=2）
    // 启动 saga → 期望仅 1 个 pod 持有
    // 杀持有 pod → 另一个接管
}
```

### 3.2 多副本 OCC（multi_replica_test.rs）

```rust
#[tokio::test]
async fn test_no_duplicate_driving_with_3_replicas() {
    let ctx = SystemTestContext::setup_k3s_with_replicas(3).await;
    
    // 并发启动 100 个 saga
    let saga_ids: Vec<Uuid> = (0..100)
        .map(|i| ctx.runtime.start_saga("RewardFlow", json!({"match_id": format!("M-{}", i)}), "test").unwrap())
        .collect();
    
    // 等待全部完成
    ctx.wait_all_completed(&saga_ids, Duration::from_secs(120)).await;
    
    // 验证：每个 saga 仅被驱动 1 次（无重复 step 副作用）
    for saga_id in &saga_ids {
        let events = ctx.get_saga_events(saga_id).await;
        let grant_events: Vec<_> = events.iter().filter(|e| e.event_type == "StepSucceeded").collect();
        // 4 个 step 各 1 次
        assert_eq!(grant_events.len(), 4);
    }
}

#[tokio::test]
async fn test_grace_period_prevents_immediate_steal() {
    // Pod A 持有 saga，60s grace period 内 Pod B 抢占应失败
}
```

### 3.3 Definition 升级兼容（upgrade_compat_test.rs）

```rust
#[tokio::test]
async fn test_v1_definition_during_upgrade() {
    // 启动 v1 PurchaseFlow
    // 部署 v2 PurchaseFlow（追加 step 5）
    // 旧 saga 用 v1 跑完
    // 新 saga 用 v2
}

#[tokio::test]
async fn test_old_definition_runs_to_completion() {
    // 模拟：v1 in-flight，v2 部署
    // 期望：v1 saga 跑完不被中断
}
```

### 3.4 微服务重启兼容（microservice_restart_test.rs）

```rust
#[tokio::test]
async fn test_inventory_restart_during_grant_item() {
    // GrantItem command 已 publish，Inventory Pod 重启
    // 期望：Inbox dedup + 重试 → step 仍成功
}

#[tokio::test]
async fn test_economy_restart_during_reserve() {
    // 预留后 restart → 验证预留仍存在（committed to DB before crash）
}
```

---

## 4. 性能 NFR 测试（per RGS-REQ-100 §5）

### 4.1 同步 Saga 延迟

```rust
#[tokio::test]
async fn test_sync_saga_p95_under_200ms() {
    let ctx = SystemTestContext::setup_k3s().await;
    
    // 预热 10 次
    for _ in 0..10 {
        ctx.runtime.start_saga("PurchaseFlow", sample_payload(), "warmup").await.unwrap();
        ctx.wait_all_completed(..., Duration::from_secs(5)).await;
    }
    
    // 测 100 次
    let mut durations = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let saga_id = ctx.runtime.start_saga("PurchaseFlow", sample_payload(), "perf").await.unwrap();
        ctx.wait_for_state(saga_id, SagaState::Completed, Duration::from_secs(5)).await;
        durations.push(start.elapsed());
    }
    
    durations.sort();
    let p95 = durations[95];
    let p99 = durations[99];
    
    // NFR-PT: p95 < 200ms, p99 < 500ms
    assert!(p95 < Duration::from_millis(200), "p95 = {:?}", p95);
    assert!(p99 < Duration::from_millis(500), "p99 = {:?}", p99);
}
```

### 4.2 异步 Saga 延迟

```rust
#[tokio::test]
async fn test_async_saga_p95_under_2s() {
    // 含外部副作用（mail）的 saga
    // NFR-PT: p95 < 2s
}
```

### 4.3 吞吐量

```rust
#[tokio::test]
async fn test_throughput_100_sagas_per_second() {
    // 100 sagas/s 持续 60s
    // NFR-SC: 10 replicas 时支持 100 sagas/s
}
```

### 4.4 Saga Runtime HA

```rust
#[tokio::test]
async fn test_3_replicas_24h_stability() {
    // 3 replicas 跑 24h，无内存泄漏，无 dead-lock
    // NFR-AV: 99.95% （年停机 < 4.4h）
}
```

### 4.5 实时游戏 Tick 不进 Saga

```rust
#[tokio::test]
async fn test_realtime_event_does_not_trigger_saga() {
    // 发送 1000 个 position update
    // 期望：0 saga 创建
    // 验证 BR-103
}
```

---

## 5. 测试矩阵

| 业务场景 | UT | IT | ST | NFR | 文档引用 |
|---|---|---|---|---|---|
| Saga 状态机 | ✅ | ✅ | ✅ | — | DTL-102 §1 |
| 补偿流 | ✅ | ✅ | ✅ | — | DTL-100 §1.3 |
| 重试 + 退避 | ✅ | ✅ | ✅ | — | DTL-102 §4 |
| 多副本 OCC | ✅ | — | ✅ | — | DTL-102 §3 |
| Pod Crash Recovery | — | — | ✅ | — | DTL-102 §2 |
| Outbox + Inbox | ✅ | ✅ | — | — | DTL-100 §4 |
| 幂等性 | ✅ | ✅ | — | — | DTL-100 §1.2 |
| Purchase Saga | ✅ | ✅ | ✅ | ✅ | DTL-100 §1 |
| Character Creation | — | ✅ | ✅ | — | DTL-100 §2 |
| Reward Saga | — | ✅ | ✅ | — | DTL-100 §3 |
| 不可逆事件 | — | ✅ | — | — | DTL-100 §3.3 |
| 跨服务调用契约 | — | ✅ | ✅ | — | DTL-100 §6 |
| 实时游戏 Tick 不进 Saga | — | — | ✅ | ✅ | BAS-100 §1 + REQ BR-103 |
| OperationPolicy 决策 | ✅ | — | — | — | DTL-101 §6 |
| AuthorityBoundary 检查 | ✅ | — | — | — | DTL-101 §7 |
| GM RBAC | ✅ | — | ✅ | — | SEC-100 §2 |
| 2FA 高风险 | — | — | ✅ | — | SEC-100 §3 |
| 审计 hash 链 | ✅ | — | — | — | SEC-100 §4 |
| mTLS 跨服务 | ✅ | — | ✅ | — | SEC-100 §5 |
| NetworkPolicy | — | — | ✅ | — | SEC-100 §6 |
| Secret 加密 | ✅ | — | ✅ | — | SEC-100 §7 |
| OTel 追踪 | — | — | ✅ | — | GOBS-100 §1 |
| Prometheus metrics | ✅ | — | ✅ | — | GOBS-100 §4 |
| Loki 日志字段 | — | — | ✅ | — | GOBS-100 §5 |
| Tempo trace 存储 | — | — | ✅ | — | GOBS-100 §6 |
| Admin Saga Console | — | — | ✅ | — | GOBS-100 §7 |
| K3s 部署 | — | — | ✅ | — | OPS-100 §2 |
| PodDisruptionBudget | — | — | ✅ | — | OPS-100 §2.2 |
| 升级兼容 | — | — | ✅ | — | OPS-100 §7 |

**测试覆盖率目标**：

- UT: ≥ 80%（核心模块 100%）
- IT: 100% 业务 Saga（4 个）+ 100% Outbox/Inbox
- ST: 100% 故障恢复场景 + 100% NFR + 100% K3s 集成

---

## 6. 测试工具

| 工具 | 用途 | 来源 |
|---|---|---|
| `cargo test` | UT | Rust 标准 |
| `testcontainers` | IT 真实 PG + NATS | crates.io |
| `proptest` | 状态机不变量 property test | crates.io |
| `mockall` | 5 域微服务 mock | crates.io |
| `cargo-llvm-cov` | 覆盖率 | crates.io |
| K3s dev cluster (WSL2) | ST | per DEC-010 |
| `kubectl` | 杀 pod / 部署验证 | 系统工具 |
| `nats` CLI | NATS 调试 | nats.io |
| `psql` | PG 验证 | 系统工具 |

---

## 7. 关联文档

- **设计**：`RGS-REQ-100` / `RGS-BAS-100` / `RGS-DTL-100~102` / `RGS-OPS-100` / `RGS-GOBS-100` / `RGS-SEC-100`
- **V 模型对子**：`RGS-IMPL-100` Saga 实施规范
- **现有测试**：
  - `RGS-TST-101` 单元测试类型集
  - `RGS-TST-102` 集成测试类型集
  - `RGS-TST-103` 系统测试类型集
  - `RGS-TST-104` 性能测试类型集
  - `RGS-TST-105` 安全测试类型集

---

## 8. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。UT 5 类（状态机/补偿/重试/DB+OCC/gRPC/Outbox+Inbox）+ IT 4 业务 Saga（Purchase/CharacterCreation/Reward/Outbox+Inbox）+ ST 4 类故障恢复（Pod crash / 多副本 OCC / Definition 升级 / 微服务重启）+ 性能 NFR 4 项（p95 / throughput / 24h / 实时 Tick 不进 Saga）+ 28 行测试矩阵。 |
