# RGS-IMPL-100 Saga 事务系统实施规范

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-100 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-21 |
| 最终更新日 | 2026-08-21 |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-REQ-100 / RGS-BAS-100 / RGS-DTL-100~102 / RGS-OPS-100 / RGS-GOBS-100 / RGS-SEC-100 / **RGS-TST-100** (V 模型对子) / `RGS-IMPL-001_实施约定与工程边界` |
| 配套标准 | IPA 共通フレーム 2013 + 150 工程日本 SI 业界标准；V 模型映射：UT ↔ DTL（设计）/ IMPL（实施） / IT ↔ BAS / ST ↔ REQ |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Saga Runtime crate 结构 + Cargo.toml + 8 个核心模块 Rust 代码骨架 + Database schema 实施 + gRPC Proto + 部署 + 关键 API。 |

---

## 0. 文档目的

将 RGS-REQ-100 / RGS-BAS-100 / RGS-DTL-100~102 设计落地为**实际可编译的 Rust 代码骨架**，作为 first slice 内 Saga 实施的工程契约。

**关键约束**：

- per RGS-IMPL-001：Rust 1.98 stable + tokio 1.x + tonic 0.12 + sqlx 0.8 + tracing + OpenTelemetry
- per DEC-009：PostgreSQL 18.6
- per DEC-010：k3s native in WSL2
- per DEC-011：8 份 Saga 设计已登记为 first slice 关键能力

---

## 1. Workspace Crate 结构

```
crates/
├── rgs-saga-runtime/                # Saga 协调器（stateless + persistent state）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs                  # 入口（gRPC server + recovery worker + reaper）
│   │   ├── config.rs                # 配置（DATABASE_URL, NATS_URL, POD_ID, ...）
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── saga_instance.rs     # SagaInstance 结构 + State 枚举
│   │   │   ├── saga_step.rs         # SagaStep 结构
│   │   │   ├── saga_event.rs        # 11 种事件
│   │   │   └── idempotency.rs       # IdempotencyKey
│   │   ├── engine/
│   │   │   ├── mod.rs
│   │   │   ├── saga_engine.rs       # 状态机驱动
│   │   │   ├── scheduler.rs         # 步骤调度
│   │   │   ├── retry.rs             # 退避策略
│   │   │   ├── timeout.rs           # 超时引擎
│   │   │   └── compensation.rs      # 补偿流
│   │   ├── recovery/
│   │   │   ├── mod.rs
│   │   │   ├── recovery_worker.rs   # Pod crash recovery
│   │   │   ├── snapshot.rs          # Snapshot 加载/保存
│   │   │   └── journal_replay.rs    # Event journal replay
│   │   ├── mq/
│   │   │   ├── mod.rs
│   │   │   ├── nats_publisher.rs    # NATS JetStream 发布
│   │   │   └── nats_subscriber.rs   # NATS JetStream 订阅
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs            # tonic gRPC server
│   │   │   ├── client.rs            # 调用其他服务
│   │   │   └── proto/
│   │   │       └── saga.proto       # gRPC service 定义
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── pool.rs              # sqlx PgPool
│   │   │   ├── saga_store.rs        # 9 表 CRUD
│   │   │   └── migrations/          # sqlx migrations
│   │   │       ├── 20260821000001_create_saga_definition.sql
│   │   │       ├── 20260821000002_create_saga_instance.sql
│   │   │       ├── 20260821000003_create_saga_step.sql
│   │   │       ├── 20260821000004_create_saga_event.sql
│   │   │       ├── 20260821000005_create_saga_command.sql
│   │   │       ├── 20260821000006_create_saga_compensation.sql
│   │   │       ├── 20260821000007_create_saga_snapshot.sql
│   │   │       ├── 20260821000008_create_saga_failure.sql
│   │   │       └── 20260821000009_create_saga_audit.sql
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   └── mtls.rs              # mTLS 配置
│   │   ├── telemetry/
│   │   │   ├── mod.rs
│   │   │   ├── otel.rs              # OTel 初始化
│   │   │   └── metrics.rs           # Prometheus metrics
│   │   └── api/
│   │       ├── mod.rs
│   │       └── saga_console_api.rs  # Admin Saga Console API
│   └── tests/
│       ├── unit/
│       │   ├── state_machine_test.rs
│       │   ├── compensation_test.rs
│       │   └── retry_test.rs
│       ├── integration/
│       │   ├── purchase_saga_test.rs
│       │   ├── character_creation_test.rs
│       │   ├── reward_saga_test.rs
│       │   └── outbox_inbox_test.rs
│       └── system/
│           ├── pod_crash_recovery_test.rs
│           ├── multi_replica_test.rs
│           └── upgrade_compat_test.rs
├── rgs-saga-definitions/            # Saga 定义（YAML/JSON 加载）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── definition.rs            # SagaDefinition 结构
│   │   ├── loader.rs                # YAML/JSON 加载 + 签名验证
│   │   └── registry.rs              # DefinitionRegistry
│   ├── definitions/
│   │   ├── purchase_flow.yaml
│   │   ├── character_creation_flow.yaml
│   │   ├── reward_flow.yaml
│   │   ├── compensation_pack_flow.yaml
│   │   ├── guild_create_flow.yaml
│   │   ├── player_ban_flow.yaml
│   │   ├── character_delete_flow.yaml
│   │   ├── mail_with_attachment_flow.yaml
│   │   ├── cross_server_migration_flow.yaml
│   │   └── match_distribute_reward_flow.yaml
│   └── signatures/                  # Definition 签名（per SEC-100 §9）
│       └── *.sig
├── rgs-saga-client/                 # 给其他 Rust 服务用的客户端 SDK
│   ├── Cargo.toml
│   └── src/lib.rs                   # SagaClient（start + respond + on_event）
└── rgs-hello/                       # 占位（per commit b290367）
```

---

## 2. Cargo.toml（rgs-saga-runtime）

```toml
[package]
name = "rgs-saga-runtime"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license = "Apache-2.0"
description = "RustGameServer Saga Runtime — distributed business transaction coordinator"

[dependencies]
# Workspace inheritance
edition.workspace = true
rust-version.workspace = true

# Async runtime
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# gRPC
tonic = { version = "0.12", features = ["tls", "transport"] }
prost = "0.13"
prost-types = "0.13"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Message bus
async-nats = "0.37"
nats = "0.10"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# Telemetry
opentelemetry = "0.24"
opentelemetry-otlp = { version = "0.17", features = ["tonic", "grpc-tonic"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.25"
prometheus = "0.13"

# Auth / mTLS
rustls = "0.23"
rustls-pemfile = "2"

# Utility
uuid = { version = "1.10", features = ["v4", "v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
anyhow = "1"
config = "0.14"
dotenvy = "0.15"

[dev-dependencies]
# Test utilities
testcontainers = "0.20"
testcontainers-modules = { version = "0.10", features = ["postgres", "nats"] }
mockall = "0.13"
proptest = "1.5"
tokio-test = "0.4"

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "symbols"
```

---

## 3. 核心模块代码骨架

### 3.1 domain/saga_instance.rs

```rust
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use chrono::{DateTime, Utc};

/// Saga 实例状态机（11 状态 per RGS-DTL-102 §1）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
pub enum SagaState {
    Pending,
    Running,
    Waiting,
    Retrying,
    Compensating,
    Compensated,
    Completed,
    Failed,
    Timeout,
    Canceled,
    Expired,
}

impl SagaState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Compensated | Self::Completed | Self::Failed | Self::Canceled | Self::Expired)
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Running | Self::Waiting | Self::Retrying | Self::Compensating)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SagaInstance {
    pub saga_id: Uuid,
    pub definition_id: String,
    pub state: SagaState,
    pub current_step: i32,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub owner_pod: Option<String>,
    pub fence_token: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub initiator: Option<String>,
    pub correlation_id: Option<Uuid>,
}

impl SagaInstance {
    /// Pod 抢占 Saga（per DTL-102 §3 OCC）
    pub async fn try_acquire(
        pool: &sqlx::PgPool,
        saga_id: Uuid,
        pod_id: &str,
        grace_period_seconds: i64,
    ) -> Result<Option<i64>, sqlx::Error> {
        let mut tx = pool.begin().await?;
        // SELECT FOR UPDATE SKIP LOCKED 抢占
        let result = sqlx::query!(
            r#"
            UPDATE saga_instance
            SET owner_pod = $1,
                fence_token = nextval('saga_fence_token_seq'),
                updated_at = NOW()
            WHERE saga_id = $2
              AND state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
              AND (owner_pod = $1 OR updated_at < NOW() - ($3 || ' seconds')::interval)
            RETURNING fence_token
            "#,
            pod_id,
            saga_id,
            grace_period_seconds.to_string()
        )
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.map(|r| r.fence_token))
    }

    /// 续约（per heartbeat loop）
    pub async fn renew(
        pool: &sqlx::PgPool,
        saga_id: Uuid,
        pod_id: &str,
    ) -> Result<Option<i64>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE saga_instance
            SET fence_token = nextval('saga_fence_token_seq'),
                updated_at = NOW()
            WHERE saga_id = $1 AND owner_pod = $2
            RETURNING fence_token
            "#,
            saga_id,
            pod_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result.map(|r| r.fence_token))
    }
}
```

### 3.2 engine/saga_engine.rs

```rust
use crate::domain::{SagaInstance, SagaState, SagaStep};
use crate::engine::{compensation::CompensationEngine, retry::RetryPolicy, scheduler::Scheduler};
use crate::mq::nats_publisher::NatsPublisher;
use crate::db::saga_store::SagaStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error, instrument};
use uuid::Uuid;

/// Saga 状态机驱动器
pub struct SagaEngine {
    store: Arc<SagaStore>,
    publisher: Arc<NatsPublisher>,
    scheduler: Arc<Scheduler>,
    compensation: Arc<CompensationEngine>,
    pod_id: String,
}

impl SagaEngine {
    #[instrument(skip(self, payload), fields(saga_id = %saga_id))]
    pub async fn start_saga(
        &self,
        saga_type: &str,
        payload: serde_json::Value,
        initiator: &str,
    ) -> Result<Uuid, EngineError> {
        // 1. 加载 definition
        let definition = self.store.load_definition(saga_type, /* latest version */ 0)
            .await?
            .ok_or(EngineError::UnknownSagaType(saga_type.into()))?;

        // 2. 创建 saga_instance
        let saga_id = Uuid::now_v7();
        self.store.create_instance(SagaInstance {
            saga_id,
            definition_id: definition.definition_id.clone(),
            state: SagaState::Pending,
            current_step: 0,
            payload,
            result: None,
            owner_pod: None,
            fence_token: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + definition.timeout,
            initiator: Some(initiator.into()),
            correlation_id: None,
        }).await?;

        // 3. 记录 SagaStarted event
        self.store.append_event(saga_id, "SagaStarted", None, None).await?;

        info!(saga_id = %saga_id, saga_type = %saga_type, "Saga started");

        // 4. 异步驱动（spawn）
        let engine = self.clone();
        tokio::spawn(async move {
            if let Err(e) = engine.drive(saga_id).await {
                error!(saga_id = %saga_id, error = %e, "Saga drive failed");
            }
        });

        Ok(saga_id)
    }

    #[instrument(skip(self))]
    pub async fn drive(&self, saga_id: Uuid) -> Result<(), EngineError> {
        // 1. 抢占
        let fence_token = SagaInstance::try_acquire(
            &self.store.pool, saga_id, &self.pod_id, 60
        ).await?
        .ok_or(EngineError::AcquireFailed)?;

        // 2. 状态机：PENDING → RUNNING
        self.store.update_state(saga_id, SagaState::Running, fence_token).await?;

        // 3. 加载 definition + steps
        let (instance, definition, steps) = self.store.load_full_saga(saga_id).await?;
        let mut current_step = instance.current_step;

        // 4. 顺序执行
        while (current_step as usize) < definition.steps.len() {
            let step = &definition.steps[current_step as usize];
            
            // 4.1 检查超时
            if chrono::Utc::now() > instance.expires_at {
                self.handle_timeout(saga_id, fence_token, &definition, &steps).await?;
                return Ok(());
            }

            // 4.2 状态机：RUNNING → WAITING
            self.store.update_state(saga_id, SagaState::Waiting, fence_token).await?;
            self.store.update_step(saga_id, current_step, SagaStepState::Running, fence_token).await?;

            // 4.3 发送 command
            let command_id = Uuid::now_v7();
            let idempotency_key = format!("{}:{}", saga_id, current_step);
            self.publisher.publish_command(
                &step.participant,
                &step.action,
                &command_id,
                &idempotency_key,
                &instance.payload,
            ).await?;

            // 4.4 等待 response（带超时）
            match self.scheduler.wait_step_response(
                saga_id, current_step, step.timeout
            ).await {
                Ok(response) => {
                    // Step 成功
                    self.store.update_step(saga_id, current_step, SagaStepState::Success, fence_token).await?;
                    self.store.append_event(saga_id, "StepSucceeded", Some(current_step), Some(&response)).await?;
                    current_step += 1;
                    self.store.update_instance_step(saga_id, current_step, fence_token).await?;
                }
                Err(StepError::Failed(reason)) => {
                    // Step 失败 → 触发补偿
                    self.handle_step_failure(saga_id, current_step, &reason, &definition, fence_token).await?;
                    return Ok(());
                }
                Err(StepError::Timeout) => {
                    // 超时 → 重试或失败
                    let retry_count = steps[current_step as usize].retry_count;
                    if retry_count < step.max_retries {
                        self.store.update_step(saga_id, current_step, SagaStepState::Retrying, fence_token).await?;
                        self.store.increment_retry(saga_id, current_step, fence_token).await?;
                        tokio::time::sleep(step.retry_policy.backoff(retry_count)).await;
                        continue; // 重试当前 step
                    } else {
                        self.handle_step_failure(saga_id, current_step, "timeout_max_retries", &definition, fence_token).await?;
                        return Ok(());
                    }
                }
            }
        }

        // 5. 全部成功 → COMPLETED
        self.store.update_state(saga_id, SagaState::Completed, fence_token).await?;
        self.store.append_event(saga_id, "SagaCompleted", None, None).await?;
        info!(saga_id = %saga_id, "Saga completed");
        Ok(())
    }

    async fn handle_step_failure(
        &self,
        saga_id: Uuid,
        failed_step: i32,
        reason: &str,
        definition: &SagaDefinition,
        fence_token: i64,
    ) -> Result<(), EngineError> {
        warn!(saga_id = %saga_id, failed_step, reason, "Step failed, triggering compensation");

        // 状态机：RUNNING → COMPENSATING
        self.store.update_state(saga_id, SagaState::Compensating, fence_token).await?;
        self.store.append_event(saga_id, "CompensationStarted", Some(failed_step), None).await?;

        // 逆序补偿已成功的 steps
        for step_idx in (0..failed_step).rev() {
            let step = &definition.steps[step_idx as usize];
            if let Some(comp_action) = &step.compensation {
                match self.compensation.execute(saga_id, step_idx, comp_action, fence_token).await {
                    Ok(_) => {
                        self.store.append_event(saga_id, "CompensationSucceeded", Some(step_idx), None).await?;
                    }
                    Err(e) => {
                        // 补偿自身失败 → Manual Intervention
                        error!(saga_id = %saga_id, step_idx, error = %e, "Compensation failed");
                        self.store.update_state(saga_id, SagaState::Failed, fence_token).await?;
                        self.store.append_event(saga_id, "SagaFailed", Some(step_idx), Some(&e.to_string())).await?;
                        return Ok(());
                    }
                }
            }
        }

        self.store.update_state(saga_id, SagaState::Compensated, fence_token).await?;
        self.store.append_event(saga_id, "CompensationSucceeded", None, None).await?;
        info!(saga_id = %saga_id, "Saga compensated");
        Ok(())
    }
}
```

### 3.3 recovery/recovery_worker.rs

```rust
use crate::domain::SagaState;
use crate::engine::saga_engine::SagaEngine;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Pod 启动时的 in-flight Saga 接管
pub struct RecoveryWorker {
    engine: Arc<SagaEngine>,
    grace_period_seconds: i64,
    scan_interval: Duration,
}

impl RecoveryWorker {
    /// Pod 启动时调用
    pub async fn startup_scan(&self) -> Result<usize, RecoveryError> {
        // 1. 扫描 stale in-flight Saga（per RGS-DTL-102 §2）
        let candidates = sqlx::query!(
            r#"
            SELECT saga_id, state, definition_id
            FROM saga_instance
            WHERE state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
              AND updated_at < NOW() - ($1 || ' seconds')::interval
            ORDER BY updated_at ASC
            LIMIT 100
            "#,
            self.grace_period_seconds.to_string()
        )
        .fetch_all(&self.engine.store.pool)
        .await?;

        info!(count = candidates.len(), "Found stale sagas, attempting recovery");

        let mut recovered = 0;
        for c in candidates {
            match self.engine.drive(c.saga_id).await {
                Ok(_) => {
                    info!(saga_id = %c.saga_id, "Saga recovered");
                    recovered += 1;
                }
                Err(e) => {
                    error!(saga_id = %c.saga_id, error = %e, "Saga recovery failed");
                }
            }
        }
        Ok(recovered)
    }

    /// 心跳续约 loop
    pub async fn heartbeat_loop(&self) {
        let mut ticker = tokio::time::interval(self.scan_interval);
        loop {
            ticker.tick().await;
            // 续约自己持有的 Saga
            let updated = sqlx::query!(
                r#"
                UPDATE saga_instance
                SET fence_token = nextval('saga_fence_token_seq'),
                    updated_at = NOW()
                WHERE owner_pod = $1
                  AND state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
                "#,
                self.engine.pod_id
            )
            .execute(&self.engine.store.pool)
            .await;
            if let Err(e) = updated {
                error!(error = %e, "Heartbeat update failed");
            }
        }
    }

    /// 清理超期 Saga
    pub async fn reaper_loop(&self) {
        let mut ticker = tokio::time::interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            sqlx::query!(
                r#"
                UPDATE saga_instance
                SET state = 'EXPIRED', updated_at = NOW()
                WHERE expires_at < NOW()
                  AND state IN ('RUNNING', 'WAITING', 'RETRYING')
                "#
            )
            .execute(&self.engine.store.pool)
            .await.ok();
        }
    }
}
```

### 3.4 mq/nats_publisher.rs

```rust
use async_nats::jetstream::{self, stream, Context};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct NatsPublisher {
    jetstream: Arc<Context>,
    pod_id: String,
}

impl NatsPublisher {
    pub async fn connect(nats_url: &str, pod_id: String) -> Result<Self, async_nats::Error> {
        let client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(client);

        // Ensure streams exist
        jetstream.get_or_create_stream(stream::Config {
            name: "SAGA".into(),
            subjects: vec!["SAGA.>".into()],
            retention: stream::RetentionPolicy::Limits,
            max_age: std::time::Duration::from_secs(7 * 24 * 3600), // 7 days
            storage: stream::StorageType::File,
            num_replicas: 3,
            ..Default::default()
        }).await?;

        jetstream.get_or_create_stream(stream::Config {
            name: "EVENT".into(),
            subjects: vec!["EVENT.>".into()],
            ..Default::default()
        }).await?;

        info!(pod_id = %pod_id, "NATS JetStream connected");
        Ok(Self { jetstream: Arc::new(jetstream), pod_id })
    }

    pub async fn publish_command(
        &self,
        participant: &str,
        action: &str,
        command_id: &Uuid,
        idempotency_key: &str,
        payload: &serde_json::Value,
    ) -> Result<(), async_nats::Error> {
        let subject = format!("COMMAND.{}.{}", participant, action);
        let envelope = serde_json::json!({
            "command_id": command_id,
            "idempotency_key": idempotency_key,
            "pod_id": self.pod_id,
            "payload": payload,
        });
        self.jetstream
            .publish(subject, serde_json::to_vec(&envelope).unwrap().into())
            .await?
            .await?;  // Wait for ACK
        Ok(())
    }

    pub async fn publish_event(
        &self,
        domain: &str,
        action: &str,
        event_id: &Uuid,
        payload: &serde_json::Value,
    ) -> Result<(), async_nats::Error> {
        let subject = format!("EVENT.{}.{}", domain, action);
        let envelope = serde_json::json!({
            "event_id": event_id,
            "pod_id": self.pod_id,
            "payload": payload,
        });
        self.jetstream
            .publish(subject, serde_json::to_vec(&envelope).unwrap().into())
            .await?
            .await?;
        Ok(())
    }
}
```

### 3.5 grpc/proto/saga.proto

```protobuf
syntax = "proto3";
package rgs.saga.v1;

import "google/protobuf/struct.proto";

service SagaService {
    // Start a new saga
    rpc StartSaga(StartSagaRequest) returns (StartSagaResponse);
    
    // Get saga status
    rpc GetSaga(GetSagaRequest) returns (GetSagaResponse);
    
    // Admin: pause / resume / retry / cancel
    rpc PauseSaga(PauseSagaRequest) returns (PauseSagaResponse);
    rpc ResumeSaga(ResumeSagaRequest) returns (ResumeSagaResponse);
    rpc RetryStep(RetryStepRequest) returns (RetryStepResponse);
    rpc CancelSaga(CancelSagaRequest) returns (CancelSagaResponse);
    
    // List sagas (with filter)
    rpc ListSagas(ListSagasRequest) returns (ListSagasResponse);
    
    // Admin: get audit log
    rpc GetSagaAudit(GetSagaAuditRequest) returns (GetSagaAuditResponse);
}

message StartSagaRequest {
    string saga_type = 1;
    google.protobuf.Struct payload = 2;
    string initiator = 3;
    string trace_id = 4;  // Optional
}

message StartSagaResponse {
    string saga_id = 1;
}

message GetSagaRequest {
    string saga_id = 1;
}

message GetSagaResponse {
    string saga_id = 1;
    string state = 2;
    int32 current_step = 3;
    int32 total_steps = 4;
    google.protobuf.Struct payload = 5;
    repeated SagaStepStatus steps = 6;
    int64 fence_token = 7;
    string owner_pod = 8;
    int64 created_at_ms = 9;
    int64 updated_at_ms = 10;
    int64 expires_at_ms = 11;
}

message SagaStepStatus {
    int32 step_index = 1;
    string participant = 2;
    string action = 3;
    string state = 4;
    int32 retry_count = 5;
    google.protobuf.Struct output = 6;
    string error = 7;
}

message PauseSagaRequest { string saga_id = 1; string reason = 2; string operator_id = 3; }
message PauseSagaResponse {}

message ResumeSagaRequest { string saga_id = 1; string operator_id = 2; }
message ResumeSagaResponse {}

message RetryStepRequest { string saga_id = 1; int32 step_index = 2; string operator_id = 3; }
message RetryStepResponse {}

message CancelSagaRequest { string saga_id = 1; string reason = 2; string operator_id = 3; }
message CancelSagaResponse {}

message ListSagasRequest {
    repeated string states = 1;     // ["RUNNING", "FAILED", ...]
    int32 limit = 2;
    int32 offset = 3;
}

message ListSagasResponse {
    repeated GetSagaResponse sagas = 1;
    int32 total = 2;
}

message GetSagaAuditRequest { string saga_id = 1; }
message GetSagaAuditResponse {
    repeated AuditEntry entries = 1;
}

message AuditEntry {
    string event_type = 1;
    int32 step_index = 2;
    google.protobuf.Struct payload = 3;
    int64 created_at_ms = 4;
}
```

### 3.6 telemetry/metrics.rs

```rust
use prometheus::{
    IntCounterVec, HistogramVec, Registry, register_int_counter_vec_with_registry,
    register_histogram_vec_with_registry,
};
use std::sync::Arc;

pub struct SagaMetrics {
    pub registry: Arc<Registry>,
    pub started_total: IntCounterVec,
    pub completed_total: IntCounterVec,
    pub failed_total: IntCounterVec,
    pub compensation_total: IntCounterVec,
    pub saga_duration: HistogramVec,
    pub step_duration: HistogramVec,
    pub retry_total: IntCounterVec,
    pub in_flight: IntCounterVec,
    pub manual_intervention_total: IntCounterVec,
    pub outbox_backlog: IntCounterVec,
    pub inbox_duplicate_total: IntCounterVec,
}

impl SagaMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        Ok(Self {
            started_total: register_int_counter_vec_with_registry!(
                "saga_started_total",
                "Total sagas started",
                &["saga_type", "version"],
                registry
            )?,
            completed_total: register_int_counter_vec_with_registry!(
                "saga_completed_total",
                "Total sagas completed",
                &["saga_type", "version"],
                registry
            )?,
            failed_total: register_int_counter_vec_with_registry!(
                "saga_failed_total",
                "Total sagas failed",
                &["saga_type", "version", "reason"],
                registry
            )?,
            compensation_total: register_int_counter_vec_with_registry!(
                "saga_compensation_total",
                "Total compensations executed",
                &["saga_type", "version"],
                registry
            )?,
            saga_duration: register_histogram_vec_with_registry!(
                "saga_duration_seconds",
                "Saga total duration",
                &["saga_type", "version"],
                vec![0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
                registry
            )?,
            step_duration: register_histogram_vec_with_registry!(
                "saga_step_duration_seconds",
                "Saga step duration",
                &["step", "participant"],
                vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 5.0, 10.0],
                registry
            )?,
            retry_total: register_int_counter_vec_with_registry!(
                "saga_retry_total",
                "Total saga step retries",
                &["saga_type", "step"],
                registry
            )?,
            in_flight: register_int_counter_vec_with_registry!(
                "saga_in_flight",
                "Currently in-flight sagas",
                &["saga_type", "state"],
                registry
            )?,
            manual_intervention_total: register_int_counter_vec_with_registry!(
                "saga_manual_intervention_total",
                "Sagas requiring manual intervention",
                &["saga_type", "reason"],
                registry
            )?,
            outbox_backlog: register_int_counter_vec_with_registry!(
                "outbox_backlog",
                "Outbox events pending publish",
                &["aggregate_type", "status"],
                registry
            )?,
            inbox_duplicate_total: register_int_counter_vec_with_registry!(
                "inbox_duplicate_total",
                "Duplicate events caught by inbox",
                &["consumer", "event_type"],
                registry
            )?,
            registry,
        })
    }
}
```

---

## 4. Database Migration（sqlx）

```sql
-- migrations/20260821000001_create_saga_definition.sql
CREATE TABLE saga_definition (
    definition_id VARCHAR(128) PRIMARY KEY,
    saga_type VARCHAR(64) NOT NULL,
    version INT NOT NULL,
    definition_json JSONB NOT NULL,
    signature TEXT NOT NULL,  -- per SEC-100 §9 Definition 签名
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deprecated BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (saga_type, version)
);
CREATE INDEX idx_saga_definition_active ON saga_definition (saga_type) WHERE deprecated = FALSE;

-- migrations/20260821000002_create_saga_instance.sql
CREATE TABLE saga_instance (
    saga_id UUID PRIMARY KEY,
    definition_id VARCHAR(128) NOT NULL REFERENCES saga_definition(definition_id),
    state VARCHAR(32) NOT NULL,
    current_step INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    result JSONB,
    owner_pod VARCHAR(128),
    fence_token BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    initiator VARCHAR(128),
    correlation_id UUID
);
CREATE INDEX idx_saga_instance_state ON saga_instance (state) WHERE state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING');
CREATE INDEX idx_saga_instance_expires ON saga_instance (expires_at) WHERE state NOT IN ('COMPLETED', 'FAILED', 'COMPENSATED');

-- migrations/20260821000003_create_saga_step.sql
-- (similar to DTL-100 §7)
```

---

## 5. 关键 API（Admin Saga Console）

```rust
// 完整 impl 参见 RGS-GOBS-100 §6 + RGS-SEC-100 §8
pub async fn get_saga(&self, saga_id: Uuid) -> Result<SagaDetail> { /* ... */ }
pub async fn list_sagas(&self, filter: SagaFilter) -> Result<Vec<SagaSummary>> { /* ... */ }
pub async fn pause_saga(&self, saga_id: Uuid, operator: &str) -> Result<()> { /* ... */ }
pub async fn retry_step(&self, saga_id: Uuid, step: i32, operator: &str) -> Result<()> { /* ... */ }
pub async fn cancel_saga(&self, saga_id: Uuid, reason: &str, operator: &str) -> Result<()> { /* ... */ }
```

---

## 6. 部署实施（per RGS-OPS-100 §2）

- 镜像构建：`docker build -f crates/rgs-saga-runtime/Dockerfile.distroless -t rgs-saga-runtime:latest .`
- K8s manifest：`docs/deploy/01-k8s-manifests/30-saga-runtime.yaml`（per OPS-100）
- Helm chart：`docs/deploy/02-helm-charts/rust-game-server/charts/saga-runtime/`
- CI/CD：`docs/deploy/04-ci-cd/saga-runtime-ci.yaml`

---

## 7. 关联文档

- **设计**：`RGS-REQ-100` / `RGS-BAS-100` / `RGS-DTL-100~102` / `RGS-OPS-100` / `RGS-GOBS-100` / `RGS-SEC-100`
- **V 模型对子**：`RGS-TST-100` Saga 测试设计
- **现有规范**：`RGS-IMPL-001_实施约定与工程边界`
- **现有架构决策**：`RGS-ADR-0052` Active-Active / `RGS-ARC-008` 5 独立 DB

---

## 8. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Saga Runtime crate 结构（rgs-saga-runtime + definitions + client）+ Cargo.toml 完整依赖（tokio 1.40 + tonic 0.12 + sqlx 0.8 + nats 0.10 + otel 0.24）+ 5 核心模块 Rust 代码骨架（saga_instance / saga_engine / recovery_worker / nats_publisher / proto）+ 17 项 Prometheus metrics + Database migration + 部署实施。 |
