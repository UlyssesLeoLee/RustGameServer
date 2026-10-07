# RGS Outbox Relay 启动模式 (v1, 2026-09-19 JST)

> **状态**: v1 candidate (per ADR-0061 §6 P2-#5 + ULYS-103 acceptance #2)
>
> **用途**: 新域 onboarding 时, 复制本模式到 `crates/<domain>/src/main.rs`, 替换 `<domain>` 占位符即可。

## §1. 背景

RGS 6 域（admin / cluster-ops / economy / match / player / social）的 `main.rs` 都遵循**同一模式**启动 outbox relay：

1. 启动时建立 NATS JetStream 客户端（per `crates/shared-platform/src/producer.rs`）。
2. 建立 PG Pool，从 pool 构造 `PgOutboxRepository`。
3. 构造 `OutboxRelay` + `RelayConfig::default()` (5s poll, 100 batch, 5 retries)。
4. `tokio::spawn` 后台 `relay.run()`。
5. 保持 NATS Client owner 存活 (`let _nats_keepalive = nats_client` 注释保留, 确保 async_nats::Client 的 Arc 不被 drop)。

## §2. 公共片段 (6 域 main.rs 完全一致)

```rust
// crates/<domain>/src/main.rs —— outbox relay 启动段

use shared_platform::outbox_relay::{OutboxRelay, RelayConfig};
use shared_platform::outbox::PgOutboxRepository;
use shared_platform::producer::{Producer, ProducerConfig};

// ... 在 main() 里, 拿到 pg_pool 和 nats_client 之后 ...

let outbox_repo = Arc::new(PgOutboxRepository::new(pg_pool.clone()));
let producer = Arc::new(Producer::new(js_ctx, ProducerConfig::default()));
let relay = OutboxRelay::new(outbox_repo, producer, RelayConfig::default());
tokio::spawn(async move {
    // 保持 NATS Client 存活（async_nats::Client 内部共享 Arc，但需 owner 存在以维持连接）
    let _nats_keepalive = nats_client;
    Arc::new(relay).run().await;
});
tracing::info!(target: "<domain>-service", "outbox relay started (NATS={})", nats_uri);
```

## §3. 6 域实例（实测一致的 pattern）

| 域 | 文件 | 关键行 |
|---|---|---|
| admin-service | `crates/admin-service/src/main.rs` L19 (use) + ~L? (spawn) | `use shared_platform::outbox_relay::{OutboxRelay, RelayConfig};` |
| cluster-ops | `crates/cluster-ops/src/main.rs` L19 | 同上 |
| economy-service | `crates/economy-service/src/main.rs` L22 | 同上 |
| match-service | `crates/match-service/src/main.rs` L21 | 同上 |
| player-service | `crates/player-service/src/main.rs` L19 | 同上 |
| social-service | `crates/social-service/src/main.rs` L19 | 同上 |

## §4. `RelayConfig::default()` 默认值

```rust
// crates/shared-platform/src/outbox_relay.rs L35-43
impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            batch_size: 100,
            max_retries: 5,
        }
    }
}
```

| 字段 | 默认值 | 说明 |
|---|---|---|
| `poll_interval` | 5s | 每次轮询间隔 |
| `batch_size` | 100 | 每次最多取 100 条 pending (per `FOR UPDATE SKIP LOCKED`) |
| `max_retries` | 5 | 失败 ≥5 次后转 `status='failed'` (per DLQ 草案 §2.2) |

新域如需调整，按域 SLA 调整即可（如降低 `poll_interval` 提高新鲜度，或降低 `max_retries` 加快 DLQ 触发）。

## §5. 失败处理（per `OutboxRelay::tick()` 实现）

```rust
// crates/shared-platform/src/outbox_relay.rs L84-118
for entry in pending {
    match self.publish_entry(&entry).await {
        Ok(()) => self.repo.mark_sent(entry.id).await?,
        Err(e) => {
            if entry.retry_count + 1 >= self.config.max_retries {
                self.repo.mark_giveup(entry.id).await?;  // → status='failed' (DLQ 入口)
            } else {
                self.repo.mark_failed(entry.id, e.to_string()).await?;
                // mark_failed: retry_count+1, last_error, status 保持 in_flight
                // 等 lease 过期被另一副本重试
            }
        }
    }
}
```

- **多 relay 并发安全**: `list_pending` 内部 `FOR UPDATE SKIP LOCKED` + mark `in_flight` + lease 30s。
- **崩溃接管**: relay 崩溃后 lease 过期，另一副本通过 list_pending 重新拿到。
- **DLQ 触发**: `mark_giveup` → `status='failed'`（per P2-#3 DLQ 草案 §2.2 入口）。

## §6. 新域 onboarding 时复制步骤

1. 复制 §2 公共片段到 `crates/<new-domain>/src/main.rs`，替换：
   - `<domain>` → 新域名（仅日志 target 用）
   - `pg_pool` / `nats_client` / `js_ctx` 来源按新域实际连接逻辑
2. 验证编译：`cargo build -p <new-domain>`
3. 启动时日志应输出：`outbox relay started (NATS=...)`
4. （可选）自定义 `RelayConfig`：构造非默认实例（如 `batch_size = 200` 高吞吐场景）。

## §7. 关联文档

- `crates/shared-platform/src/outbox_relay.rs` — Relay 实现 source of truth。
- `crates/shared-platform/src/outbox.rs` — OutboxRepository trait + Pg/InMemory impl。
- `crates/shared-platform/src/producer.rs` — NATS JetStream Producer。
- ADR-0061 §2 决定 2: 事务内强制 outbox 写入。
- ADR-0061 §2 决定 4 第 6 项: relay 启动由 main.rs 触发。
- `09a_跨域事件族清单_v1.1_可验证部分.md` §C v1.2 候选清单（新域的首批 subject）。
- `10_outbox_migration_template_v1.sql` — 新域 outbox 表 migration 模板。
- `06_Outbox监控指标草案.md` — 新域监控指标接入清单。
- `07_PoisonEvent_DLQ草案.md` — 新域 DLQ 接入清单。

## §8. 修订历史

| 版本 | 日期 | 修订者 | 内容 |
|---|---|---|---|
| v1 | 2026-09-19 JST | ULYS-103 worker (agent c557dae5) | 初版：6 域 main.rs 一致的 outbox relay 启动模式抽取 + 新域 onboarding 复制步骤 |
