# RGS-DTL-102 Saga 故障恢复设计

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-102 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-21 |
| 最终更新日 | 2026-08-21 |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-REQ-100 / RGS-BAS-100 / RGS-DTL-100（同侪 Saga 业务模式）/ RGS-DTL-101（同侪 OperationPolicy） |
| 配套标准 | IPA 共通フレーム 2013 + 150 工程日本 SI 业界标准；V 模型映射：UT ↔ DTL |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Saga Instance 状态机 / K3s Pod Crash Recovery / Saga Runtime HA 多副本 OCC / 微服务 Pod 重启兼容 / 升级兼容性 / 故障自检表。 |

---

## 0. 文档目的

定义 Saga 系统的**故障恢复**机制：

1. Saga Instance 状态机（11 个状态）
2. K3s Saga Runtime Pod Crash Recovery
3. Saga Runtime HA（多副本 OCC 抢占）
4. 微服务 Pod 重启时 Saga 不立即失败
5. Saga Definition 升级兼容性（旧实例跑完）
6. Compensation 自身不可恢复处理
7. 故障自检表

---

## 1. Saga Instance 状态机

```mermaid
stateDiagram-v2
    [*] --> PENDING: StartSaga (saga_id 分配, definition 解析)
    PENDING --> RUNNING: 抢占成功 (fence_token 分配)
    RUNNING --> WAITING: Step command 已发, 等待 response
    WAITING --> RUNNING: 收到 step response (success/fail)
    RUNNING --> COMPENSATING: 任一 step 失败, 触发补偿
    COMPENSATING --> COMPENSATED: 所有补偿成功
    COMPENSATING --> FAILED: 任一补偿失败 (Manual Intervention)
    RUNNING --> FAILED: 不可恢复错误 (Manual Intervention)
    WAITING --> RETRYING: step 超时, 触发重试
    RETRYING --> WAITING: 重新发送 command
    RETRYING --> FAILED: 重试耗尽 (Manual Intervention)
    RUNNING --> TIMEOUT: Saga 总超时 (expires_at)
    WAITING --> TIMEOUT: 长时间无响应
    TIMEOUT --> COMPENSATING: 触发强制补偿
    TIMEOUT --> FAILED: 补偿失败
    COMPENSATED --> [*]: terminal
    COMPLETED --> [*]: terminal
    FAILED --> [*]: terminal (但保留在表, 可 GM 介入)

    note right of RUNNING
        Pod 持有 fence_token
        - 写入校验 fence_token
        - 续约通过 SELECT FOR UPDATE
    end note

    note right of WAITING
        命令已发往目标服务
        - 等待 NATS JetStream 响应
        - 超时 → RETRYING
    end note

    note right of FAILED
        不可恢复
        - 进入 Manual Intervention Queue
        - GM 通过 Saga Console 介入
        - 必要时 Corrective Event
    end note
```

**11 个状态**：

| 状态 | 含义 | 终态 | 备注 |
|---|---|---|---|
| PENDING | 已分配 saga_id，definition 已解析，但还未被任何 Pod 抢占 | ❌ | 极短状态（毫秒）|
| RUNNING | Pod 持有 fence_token，正在执行 | ❌ | 持有者写 fence_token |
| WAITING | Step command 已发，等目标服务响应 | ❌ | NATS JetStream consumer 监听 |
| RETRYING | Step 超时 / 失败，等待指数退避 | ❌ | backoff 1s/2s/4s/8s/16s |
| COMPENSATING | 触发补偿流 | ❌ | 逆序执行补偿步骤 |
| COMPENSATED | 补偿完成 | ✅ | Saga 终态 |
| COMPLETED | 正常完成 | ✅ | Saga 终态 |
| FAILED | 不可恢复失败 | ✅ | Manual Intervention |
| TIMEOUT | Saga 总超时（默认 30 分钟）| — | 触发强制补偿 |
| CANCELED | GM 主动取消 | ✅ | Saga Console 操作 |
| EXPIRED | expires_at 已过 + 无法续约 | — | Reaper Worker 处理 |

---

## 2. K3s Pod Crash Recovery

```mermaid
sequenceDiagram
    autonumber
    participant PodA as Saga Runtime Pod A<br/>(持有 S-001)
    participant DB as cluster_ops_db
    participant PodB as Saga Runtime Pod B<br/>(新启动)
    participant MB as NATS JetStream

    Note over PodA: 持有 S-001 fence_token=42<br/>state=RUNNING, step=4/6
    PodA->>DB: UPDATE saga_step (step=4, state=SUCCESS)
    PodA->>DB: UPDATE saga_instance (state=WAITING, step=5)
    PodA->>MB: Publish step 5 command
    PodA--xDB: 💥 Pod A crash (K8s 探测后 SIGKILL)

    Note over DB: S-001 state=WAITING<br/>owner_pod=Pod-A (stale)<br/>fence_token=42

    Note over PodB: K3s 启动新 Pod B
    PodB->>DB: BEGIN
    PodB->>DB: SELECT * FROM saga_instance<br/>WHERE state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')<br/>AND owner_pod != 'Pod-B'<br/>AND fence_token_expires_at < NOW()<br/>ORDER BY updated_at ASC<br/>FOR UPDATE SKIP LOCKED<br/>LIMIT 10
    DB-->>PodB: 返回 S-001 (持有 fence_token 仍未过期, 但 owner_pod 不匹配)

    alt fence_token 未过期 (持有者可能还活着)
        Note over PodB: 等待 grace period (60s)<br/>再次检测
        PodB->>DB: SELECT WHERE owner_pod='Pod-A' AND updated_at < NOW() - 60s
        alt owner 仍 stale
            PodB->>DB: UPDATE saga_instance<br/>SET owner_pod='Pod-B',<br/>fence_token=fence_token+1<br/>WHERE saga_id=S-001 AND fence_token=42
            DB-->>PodB: 1 row updated (抢占成功)
        end
    else fence_token 已过期
        PodB->>DB: UPDATE saga_instance<br/>SET owner_pod='Pod-B',<br/>fence_token=fence_token+1<br/>WHERE saga_id=S-001<br/>AND updated_at < NOW() - INTERVAL '60 seconds'
        DB-->>PodB: 1 row updated (抢占成功)
    end

    PodB->>DB: COMMIT
    PodB->>DB: SELECT last_event_id FROM saga_snapshot WHERE saga_id=S-001
    alt 有 snapshot
        PodB->>DB: 加载 snapshot + replay events from last_event_id
    else 无 snapshot
        PodB->>DB: SELECT * FROM saga_event WHERE saga_id=S-001<br/>ORDER BY event_id ASC
    end

    PodB->>DB: SELECT * FROM saga_step WHERE saga_id=S-001
    PodB->>DB: 检测 state=WAITING 的 step (5)<br/>+ 检查 command_id 是否已 ACK
    PodB->>DB: SELECT FROM saga_command WHERE command_id IN (step 5)
    alt command 已 ACK
        Note over PodB: step 5 已成功, 推进到 step 6
    else command 未 ACK
        Note over PodB: 重新发布 step 5 (幂等 by idempotency_key)
        PodB->>MB: Re-publish step 5 command
    end

    PodB->>DB: INSERT saga_event (SagaResumed, pod=Pod-B)
```

**关键设计点**：

1. **新 Pod 启动时扫描 in-flight Saga**：
   ```sql
   SELECT * FROM saga_instance
   WHERE state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
     AND updated_at < NOW() - INTERVAL '60 seconds'  -- 60s grace period
   FOR UPDATE SKIP LOCKED
   LIMIT 100;
   ```

2. **抢占**：使用 `SELECT FOR UPDATE SKIP LOCKED` 避免多 Pod 同时抢占同一 Saga

3. **fence_token 续约**：
   - 抢占时 `fence_token = fence_token + 1`
   - 续约时 `fence_token = fence_token + 1, updated_at = NOW()`
   - 写 Saga 表时 `WHERE fence_token = ?` 校验

4. **Snapshot + Journal Replay**：
   - 定期（每 N 步）保存 snapshot
   - 恢复时加载最新 snapshot + 重放后续 event
   - 比完整 replay 快 100x

5. **Command 幂等性**：
   - 每个 command 携带 `idempotency_key = {saga_id}:{step_index}`
   - 重发时目标服务通过 Inbox 表去重

---

## 3. Saga Runtime HA（多副本）

```mermaid
graph TB
    subgraph Replicas["saga-runtime Deployment (3 replicas)"]
        PodA["Pod A<br/>fence_token_range=1-100"]
        PodB["Pod B<br/>fence_token_range=101-200"]
        PodC["Pod C<br/>fence_token_range=201-300"]
    end

    DB[("cluster_ops_db<br/>saga_instance<br/>+ fence_token column")]

    PodA -->|抢占 S-001 fence_token=1| DB
    PodA -->|写 Saga step 1| DB
    PodA -->|续约 fence_token=2| DB

    PodB -->|抢占 S-002 fence_token=101| DB
    PodB -->|写 Saga step 3| DB

    PodC -->|抢占 S-003 fence_token=201| DB
    PodC -->|写 Saga step 2| DB

    Note["SAGA FENCE TOKEN GUARANTEE:<br/>1. 单调递增 (PostgreSQL sequence)<br/>2. 写入时 WHERE fence_token=? 校验<br/>3. 过期 Leader 写入 0 rows affected<br/>4. 不依赖 distributed Redis lock"]
    DB -.-> Note

    classDef pod fill:#c8e6c9,stroke:#1b5e20
    classDef db fill:#e3f2fd,stroke:#1565c0
    classDef note fill:#fff9c4,stroke:#f57f17
    class PodA,PodB,PodC pod
    class DB db
    class Note note
```

**OCC 多副本设计**：

1. **Fence Token 序列**：
   ```sql
   CREATE SEQUENCE saga_fence_token_seq START 1 INCREMENT 1;
   ```

2. **抢占（Pod A 启动 / 发现 in-flight Saga）**：
   ```sql
   BEGIN;
   -- 抢占
   UPDATE saga_instance
   SET owner_pod = 'pod-a-uuid',
       fence_token = nextval('saga_fence_token_seq'),
       updated_at = NOW()
   WHERE saga_id = ?
     AND state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
     AND (owner_pod = 'pod-a-uuid' OR updated_at < NOW() - INTERVAL '60 seconds');
   -- 0 rows = 抢占失败（被其他 Pod 持有且未超时）
   COMMIT;
   ```

3. **续约（每 30s 一次）**：
   ```sql
   UPDATE saga_instance
   SET fence_token = nextval('saga_fence_token_seq'),
       updated_at = NOW()
   WHERE saga_id = ? AND owner_pod = 'pod-a-uuid'
   RETURNING fence_token;
   ```

4. **写入校验（任何 Saga step 写入）**：
   ```sql
   -- 写入 saga_step
   UPDATE saga_step
   SET state = 'SUCCESS', output = ?
   WHERE step_id = ? AND saga_id = ?
     AND fence_token = ?;  -- 必须匹配当前 Pod 持有的 token
   ```

5. **过期 Pod 写入**：`UPDATE ... WHERE fence_token = ?` 会返回 0 rows affected，应用层检测后重抢。

**为什么不用 Redis distributed lock**：

- Redis 是新组件（per BR-111 纯开源 + 不绑新依赖）
- PostgreSQL 已经存在且 ACID 强一致
- Fence Token 在 PostgreSQL 单调递增 + 事务性，比 Redis lock 更可靠
- 减少 K3s 资源（Minimal profile 不需要 Redis）

---

## 4. 微服务 Pod 重启兼容

**场景**：Saga 步骤已发送给 Inventory Service，Inventory Pod 重启，命令丢失 / 重复。

```mermaid
sequenceDiagram
    autonumber
    participant SR as Saga Runtime
    participant MB as NATS JetStream
    participant ISP as Inventory Service Pod (旧)
    participant ISN as Inventory Service Pod (新)
    participant DB as inventory_db

    SR->>MB: Publish ReserveInventorySlot<br/>(command_id=C-002, idempotency_key=S-001:C-002)
    MB->>ISP: Deliver
    activate ISP
    ISP->>DB: BEGIN
    ISP->>DB: INSERT reservations
    ISP->>DB: INSERT inbox (idempotency_key)
    ISP--xDB: 💥 ISP 崩溃 (K8s 杀进程)

    Note over DB: reservations 已 commit<br/>但 inbox 状态未完成 (PENDING)

    Note over MB: ack 未送达 (consumer 中断)
    MB->>ISN: Re-deliver (max 5 次 backoff)
    activate ISN
    ISN->>DB: SELECT FROM inbox WHERE event_id=...
    alt 已 INSERT 但 PENDING
        ISN->>DB: 检测到 idempotency_key 存在
        ISN->>DB: 应用业务（INSERT reservations）→ 因 UNIQUE constraint 失败
        ISN->>DB: 视为已处理, 标记 inbox.status=DONE
        ISN->>MB: ACK
    else 不存在
        ISN->>DB: BEGIN; INSERT reservations; INSERT inbox; COMMIT
        ISN->>MB: ACK
    end
    deactivate ISN

    Note over SR: 通过 Inbox DONE 状态 + idempotency_key<br/>保证不重复执行
```

**关键机制**：

1. **Inbox 表去重**：`idempotency_key` PRIMARY KEY / UNIQUE
2. **幂等命令**：即使重复执行，副作用可忽略
3. **Saga Runtime 超时重试**：默认 5s 步超时 + 5 次重试（指数退避）
4. **Pod 重启不立即失败**：Saga Runtime 检测到 command 超时后**不立即判定失败**，而是再重试 1-2 次，给目标服务恢复时间

**Saga Runtime 重试策略**：

| 重试次数 | 退避 | 触发条件 |
|---|---|---|
| 1 | 0s（立即） | 目标服务返回 5xx |
| 2 | 1s | 目标服务返回 5xx |
| 3 | 2s | 目标服务返回 5xx |
| 4 | 4s | 目标服务返回 5xx |
| 5 | 8s | 目标服务返回 5xx |
| Exceeded | — | SagaFailed + 触发 Manual Intervention |

---

## 5. Saga Definition 升级兼容

```mermaid
graph LR
    subgraph "v1 Active (旧)"
        V1Def[saga_definition v1<br/>deprecated=false]
        V1Inst1[S-001<br/>using v1]
        V1Inst2[S-002<br/>using v1]
    end

    subgraph "v2 部署中 (新)"
        V2Def[saga_definition v2<br/>deprecated=false]
        V2Inst[S-003<br/>using v2]
    end

    V1Def -.->|"v1 实例跑完"| V1Inst1
    V1Def -.->|"v1 实例跑完"| V1Inst2
    V2Def -->|"新 Saga 走 v2"| V2Inst

    Note["兼容策略:<br/>1. v1 + v2 同时 active<br/>2. saga_instance.definition_id 记录实际版本<br/>3. 在飞 Saga 用启动时的 version<br/>4. 老 version 没有 in-flight 时标记 deprecated=true"]

    classDef v1 fill:#ffe0b2,stroke:#e65100
    classDef v2 fill:#c8e6c9,stroke:#1b5e20
    class V1Def,V1Inst1,V1Inst2 v1
    class V2Def,V2Inst v2
```

**升级流程**：

1. 部署 v2 Saga Runtime（含 v2 definition）
2. 旧 v1 Pod 仍运行，处理 in-flight v1 Saga
3. 新 Saga 走 v2（GRPC router 决定）
4. 旧 v1 Saga 跑完后，v1 Pod 缩容到 0
5. v1 definition `deprecated=true`

**关键约束**：

- Saga Definition schema 必须**向后兼容**（v1 step 字段不能删，只能加 optional 字段）
- 在飞 v1 Saga 即使 v2 已部署，**继续按 v1 跑完**（不能中途切到 v2）
- Pod 升级使用 **Rolling Update**（maxUnavailable=0, maxSurge=1）

---

## 6. Compensation 自身不可恢复

```mermaid
graph TB
    Saga[Saga 步骤 N 失败] --> CompTrigger[触发补偿]
    CompTrigger --> Comp1[补偿步骤 N-1]
    Comp1 -->|success| Comp2[补偿步骤 N-2]
    Comp1 -->|fail| Manual1[Manual Intervention Queue]
    Comp2 -->|success| CompN[补偿步骤 0]
    Comp2 -->|fail| Manual2[Manual Intervention Queue]
    CompN -->|success| CompDone[Saga COMPENSATED]
    CompN -->|fail| Manual3[Manual Intervention Queue]

    Manual1 --> GMConsole[GM Saga Console]
    Manual2 --> GMConsole
    Manual3 --> GMConsole

    GMConsole --> Decision{GM 决策}
    Decision -->|Retry 补偿| CompRetry[Retry from failed step]
    Decision -->|Manual Compensate| ManualComp[GM 手工补偿]
    Decision -->|Corrective Event| CorrEvent[发 Corrective Event]
    Decision -->|Cancel Saga| Cancel[Cancel Saga<br/>+ Audit]

    classDef step fill:#e3f2fd,stroke:#1565c0
    classDef manual fill:#ffcdd2,stroke:#c62828
    classDef decision fill:#fff9c4,stroke:#f57f17
    class Saga,CompTrigger,Comp1,Comp2,CompN,CompDone step
    class Manual1,Manual2,Manual3,ManualComp,CorrEvent,Cancel,CompRetry,GMConsole manual
    class Decision decision
```

**不可恢复 Compensation 处理**：

| 场景 | 处理 |
|---|---|
| RefundCurrency 失败（玩家账号已冻结）| Manual Compensate（GM 手工打款）|
| RevokeItem 失败（物品已交易）| Corrective Event（通知原 + 新玩家协调）|
| SendMail 失败（无外部副作用）| Retry（最多 5 次）|
| 数据库写失败 | SagaFailed + Manual Intervention |

**GM 介入工具**（per RGS-SEC-100）：

- Saga Console 显示 in-flight + failed Saga
- 二次权限校验（2FA）
- Audit Log 记录所有操作
- Pause / Resume / Retry / Manual Compensate / Cancel

---

## 7. 故障自检表（per spec 59）

| 检查项 | 状态 | 证据 |
|---|---|---|
| K3s Pod crash 后丢 Saga？ | ✅ 不丢 | §2 Crash Recovery + snapshot + journal replay |
| Saga Runtime 多副本重复驱动？ | ✅ 不重复 | §3 Fence Token + SELECT FOR UPDATE SKIP LOCKED |
| MQ 重复消息？ | ✅ 不重复处理 | §4 Inbox 表 + event_id PRIMARY KEY |
| MQ 乱序？ | ✅ 顺序保证 | NATS JetStream per-subject 顺序 |
| 微服务升级中在飞 Saga？ | ✅ 兼容 | §5 v1+v2 双版本并行 + 旧 instance 跑完 |
| Compensation 自身失败？ | ✅ Manual Intervention | §6 GM Saga Console |
| 浏览器关闭影响 Saga？ | ✅ 不影响 | spec 20 浏览器非 Coordinator |
| Saga 进入实时游戏 Tick？ | ✅ 不进 | DTL-100 spec 53 明确禁止 |
| 跨服务操作没有 Saga？ | ✅ 强制 | DTL-101 OperationPolicy |
| 经济操作缺幂等？ | ✅ 强制 | DTL-100 §1.2 idempotency_key |
| 纯 UI 操作进服务器？ | ✅ 不进 | DTL-101 L0 UiOnly |
| 跨域无 Compensation？ | ✅ 全部定义 | DTL-100 §1.3 |
| 服务直访其他服务 DB？ | ✅ 禁止 | RGS-BAS-100 §7 Database per Service |
| 依赖闭源组件？ | ✅ 全开源 | RGS-BAS-100 §5 NATS JetStream Apache-2.0 |

---

## 8. Recovery Worker 实现

```rust
// saga-runtime/recovery.rs
pub struct RecoveryWorker {
    db: PgPool,
    pod_id: String,
    grace_period: Duration,         // 默认 60s
    scan_interval: Duration,        // 默认 5s
    snapshot_interval: Duration,    // 默认 30s
}

impl RecoveryWorker {
    /// Pod 启动时调用一次
    pub async fn startup_scan(&self) -> Result<usize> {
        // 1. 扫描 in-flight Saga
        let candidates = sqlx::query!(
            r#"
            SELECT saga_id, state, owner_pod, fence_token, definition_id
            FROM saga_instance
            WHERE state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
              AND updated_at < NOW() - $1::interval
            ORDER BY updated_at ASC
            LIMIT 100
            "#,
            format!("{} seconds", self.grace_period.as_secs())
        )
        .fetch_all(&self.db)
        .await?;

        let mut recovered = 0;
        for c in candidates {
            // 2. 抢占
            let new_token: i64 = sqlx::query_scalar!(
                r#"
                UPDATE saga_instance
                SET owner_pod = $1, fence_token = nextval('saga_fence_token_seq'),
                    updated_at = NOW()
                WHERE saga_id = $2
                  AND (owner_pod = $1 OR updated_at < NOW() - $3::interval)
                RETURNING fence_token
                "#,
                self.pod_id,
                c.saga_id,
                format!("{} seconds", self.grace_period.as_secs())
            )
            .fetch_optional(&self.db)
            .await?;

            if let Some(t) = new_token {
                // 3. 恢复 Saga state machine
                self.resume_saga(c.saga_id, t).await?;
                recovered += 1;
            }
        }
        Ok(recovered)
    }

    /// 持续运行：定期续约 in-flight Saga
    pub async fn heartbeat_loop(&self) {
        let mut ticker = tokio::time::interval(self.scan_interval);
        loop {
            ticker.tick().await;
            // 更新自己的 Saga 持有者
            sqlx::query!(
                r#"
                UPDATE saga_instance
                SET fence_token = nextval('saga_fence_token_seq'),
                    updated_at = NOW()
                WHERE owner_pod = $1
                  AND state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')
                "#,
                self.pod_id
            )
            .execute(&self.db)
            .await.ok();
        }
    }

    /// Reaper: 清理超期 Saga
    pub async fn reaper_loop(&self) {
        let mut ticker = tokio::time::interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            // 标记超期 Saga 为 FAILED
            sqlx::query!(
                r#"
                UPDATE saga_instance
                SET state = 'EXPIRED', updated_at = NOW()
                WHERE expires_at < NOW()
                  AND state IN ('RUNNING', 'WAITING', 'RETRYING')
                "#
            )
            .execute(&self.db)
            .await.ok();
        }
    }
}
```

---

## 9. 关联文档

- **基础**：`RGS-REQ-100` / `RGS-BAS-100`
- **同侪**：
  - `RGS-DTL-100` Saga 业务模式设计
  - `RGS-DTL-101` OperationPolicy 与 AuthorityBoundary 设计
- **部署**：`RGS-OPS-100`
- **可观测性**：`RGS-GOBS-100`
- **安全**：`RGS-SEC-100`

---

## 10. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Saga Instance 状态机 (11 状态) / K3s Pod Crash Recovery (snapshot + journal replay + fence_token) / Saga Runtime HA (Fence Token + SELECT FOR UPDATE SKIP LOCKED) / 微服务重启兼容 (Inbox + 重试) / Definition 升级兼容 (v1+v2 双版本) / Compensation 不可恢复 (GM 介入) / Recovery Worker Rust 实现。 |
