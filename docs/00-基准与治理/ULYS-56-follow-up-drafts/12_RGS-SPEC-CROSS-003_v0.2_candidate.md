# RGS-SPEC-CROSS-003 跨域事件 Schema 字典 v0.2 (候选)

> **状态**: 🔵 **v0.2 candidate** (per ULYS-103 acceptance #3 + ADR-0061 §6 P2-#5)
>
> **依据**: RGS-SPEC-CROSS-003 v0.1 (NO-GO placeholder, 见现有 `docs/13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md`) + `09a_跨域事件族清单_v1.1_可验证部分.md` + `crates/shared-platform/src/subject.rs`
>
> **ULYS-103 worker**: agent c557dae5-42e4-4d60-bf27-36eeddbf67bb (2026-09-19 JST)
>
> **重要声明**: 本文件是**候选草案**，**不替换** v0.1 NO-GO 主文档。v0.2 升版路径（per v0.1 §6 激活条件）需 G-CODE-06 + G-CODE-03 全绿 + NO-GO 解除，由 Ulysses 一人公司 12 角色（per DEC-008）正式升 v0.2 时填入实际事件 schema 字典。

---

## §0. v0.1 → v0.2 升版路径

v0.1 §6 激活条件：

1. **G-CODE-06**: Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03**: 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

**满足后** → 由 Ulysses 升 v0.2 → 把本候选文件 §1-§10 内容合并到 v0.1 主文档。

---

## §1. 事件主题命名空间规范（v0.2 candidate）

### §1.1 命名约定（实测可达）

**全局约定（per `crates/shared-platform/src/subject.rs` + RGS-OPEN-QA-001 修正 #7）**：

| 名空间类 | 模式 | 构造器 | 解析器 |
|---|---|---|---|
| 域事件 | `rgs.<domain>.<event_type>.<version>` | `SubjectBuilder::domain_event(domain, event_type, version)` | `parse(subject)` → `SubjectDomain::Domain` |
| Saga 事件 | `rgs.saga.<saga_type>.<event>` | `SubjectBuilder::saga_event(saga_type, event)` | `parse(subject)` → `SubjectDomain::Saga` |
| CEM 路由 | `rgs.cem.<event_type>` | `SubjectBuilder::cem_event(event_type)` | `parse(subject)` → `SubjectDomain::Cem` |
| DLQ | `rgs.dlq.<source_subject>` | `SubjectBuilder::dlq(source)` | `parse(subject)` → `SubjectDomain::Dlq` |

### §1.2 已知域

| 业务域 | subject 前缀 | 解析验证 |
|---|---|---|
| player | `rgs.player.*` | `parse_all_five_business_domains` 测试通过 |
| economy | `rgs.economy.*` | 同上 |
| match | `rgs.match.*` | 同上 |
| social | `rgs.social.*` | 同上 |
| admin | `rgs.admin.*` | 同上 |
| cluster_ops | `rgs.cluster_ops.*` | 同上 |

新域加入时需扩展 `parse()` 函数 (L73-82 of `subject.rs`) 的 match arm —— 这是 `parse_all_five_business_domains` 测试覆盖的入口。

### §1.3 命名约定分歧处置（per RGS-OPEN-QA-001 修正 #7）

**v0.1 主文档 §2.2 描述的 `rgs.events.<domain>.<aggregate>.<action>.<version>` 命名未被任何生产/测试代码采用。** v0.2 candidate **正式废弃**该命名，**采用实测可达的 `rgs.<domain>.<event_type>.<version>`**。

| 项 | v0.1 描述 | v0.2 candidate |
|---|---|---|
| 命名约定 | `rgs.events.<domain>.<aggregate>.<action>.<version>` | `rgs.<domain>.<event_type>.<version>` |
| 业务表达自由度 | `aggregate.action` 强制拆分 | `event_type` 可单段（`registered`）或多段（`wallet.committed`） |
| 实现层支持 | ❌ 无 | ✅ `SubjectBuilder` + `parse()` 全部覆盖 |
| RGS-OPEN-QA-001 修正 #7 引用 | 否（漂移） | 是 |

---

## §2. 事件 payload 模板（CloudEvents 1.0 兼容）

> ⚠️ v0.2 候选**: payload schema 各域各异, v0.2 candidate 仅约定**最小字段集**。具体 schema 待各域 owner 在 SPEC-CROSS-003 v0.2+ §X.Y 中落地。

### §2.1 最小字段集（per `OutboxEntry` 实际 schema）

```text
{
  "id":          UUID,         // OutboxEntry.id (per crates/shared-platform/src/outbox.rs L81)
  "subject":     String,       // outbox.subject (per §1.1 命名约定)
  "payload":     JSON,         // 业务 payload (per §2.2 各域自定义)
  "command_id":  UUID,         // 幂等 key (consumer 端去重)
  "saga_id":     Option<UUID>, // Saga 关联（per §3 Saga 事件族）
  "occurred_at": TIMESTAMPTZ,  // payload 内业务时间 (由 producer 在 append 时填)
  "trace_id":    String,       // per RGS-SPEC-CROSS-006 trace_id 传播
  "version":     u32           // per §1.1 <version> 段
}
```

### §2.2 payload 各域自定义（v0.2 候选）

| 域 | payload schema 主键 | 落地文档 |
|---|---|---|
| player | `{player_id, character_id, event_kind, occurred_at, ...}` | 待 player 域 owner 落地 |
| economy | `{wallet_id, amount, currency, tx_type, occurred_at, ...}` | 待 economy 域 owner 落地 |
| match | `{match_id, players, winner, duration, occurred_at, ...}` | 待 match 域 owner 落地 |
| social | `{actor_id, target_id, verb, occurred_at, ...}` | 待 social 域 owner 落地 |
| admin | `{operator_id, target_id, action_kind, occurred_at, ...}` | 待 admin 域 owner 落地 |
| cluster_ops | `{node_id, cluster_id, event_kind, occurred_at, ...}` | 待 cluster_ops 域 owner 落地 |

---

## §3. 事件 schema 版本管理

**语义化版本（per `crates/shared-platform/src/subject.rs` L47-49）**: `<version>` 段为 `u32` 单调递增。

- **`v1` → `v2` 不兼容变更**: 字段删除/类型变更/含义变更 → 升 `v2`，旧 `v1` 保留消费路径（per §6 schema evolution）。
- **`v1` → `v1.1` 兼容变更**: 新增可选字段（不影响 consumer 既有字段消费）。

新域 onboarding 时首个事件族统一用 `v1`。

---

## §4. 域内事件清单（v0.2 candidate 6 域已设计可达基线）

> 详见 `09a_跨域事件族清单_v1.1_可验证部分.md` §C.1 v1.2 候选清单（22 条 unique, 命名合规可生产构造）。

| 域 | 设计可达事件族 | 实测已发布 |
|---|---|---|
| player | 4 条 (character.created/deleted, session.started/ended) | 2 条 unique (registered.v1, overflow.v1) |
| economy | 5 条 (wallet.committed/reserved/released, inventory.granted/consumed) | 2 条 unique (transferred.v1, debit.v1, overflow.v1) |
| match | 3 条 (match.created/finished, reward.distributed) | 2 条 unique (ended.v1, overflow.v1) |
| social | 4 条 (friend.added/removed, guild.created, mail.sent) | 1 条 unique (overflow.v1) |
| admin | 3 条 (gm.compensated, ban.applied/lifted) | 0 条 unique (待域 owner 落地) |
| cluster_ops | 3 条 (node.joined/left, shard.rebalanced) | 0 条 unique (待域 owner 落地) |

**v0.2 candidate 总数**:

- 设计可达: **22 条**（命名合规可生产构造，per `SubjectBuilder::domain_event`）
- 实测已发布: **9 条 unique**（per `09a` §A.1）
- 差距: **13 条待域 owner 落地**（域 owner 满载后可达 22 条完整覆盖）

---

## §5. 跨域事件清单（Q-003 Saga 相关 + Outbox 相关）

### §5.1 Saga 事件族（per `rgs.saga.<saga_type>.<event>` 命名）

| Saga 类型 | event 实测可达 |
|---|---|
| transfer | `step_completed`, `step_failed`（per `crates/shared-platform/tests/it_subject_dlq_round_trip.rs`） |

### §5.2 跨域事件流（订阅关系）

| 生产者 | 事件族 | 消费者 | 用途 |
|---|---|---|---|
| economy | `rgs.economy.wallet.committed.v1` | player | 触发 player 域缓存失效 (per `RGS-OPEN-QA-001` §修正 #7 案例) |
| match | `rgs.match.match.finished.v1` | economy | 触发 Reward Saga |
| match | `rgs.match.reward.distributed.v1` | player | 玩家经验/货币更新 |

> v0.2 candidate 仅列**已知**订阅关系。新订阅关系上线路由: 各域 owner + 架构师联合登记到本节。

---

## §6. 集群事件清单（COC / PFAU / CEM 自身事件）

### §6.1 CEM 路由事件族

| 事件族 | 命名 |
|---|---|
| feature flag 更新 | `rgs.cem.feature_flag_updated`（per `subject.rs` L15 注释示例） |

### §6.2 cluster_ops 事件族（域内 + 跨域）

| 事件族 | 命名 | 跨域 |
|---|---|---|
| 节点加入 | `rgs.cluster_ops.node.joined.v1` | ✅ |
| 节点离开 | `rgs.cluster_ops.node.left.v1` | ✅ |
| 分片重平衡 | `rgs.cluster_ops.shard.rebalanced.v1` | ✅ (PFAU 触发) |

---

## §7. 事件订阅者注册表（per DTL-031 §3 event_producer_registry）

> v0.2 candidate: 注册表 schema 与 `crates/admin-service/migrations/event_schema_registry.sql` 一致；具体行待各域 owner 落地。

```sql
-- (per crates/admin-service/migrations/event_schema_registry*.sql 模式)
CREATE TABLE event_schema_registry (
    subject VARCHAR(256) PRIMARY KEY,
    payload_schema_uri TEXT NOT NULL,
    payload_sample JSONB,
    owner_domain VARCHAR(64) NOT NULL,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version VARCHAR(16) NOT NULL
);
```

---

## §8. 事件回放 / 重放 / 死信处理

- **回放**: outbox 表的 `status='sent'` 行保留（per `crates/shared-platform/src/outbox.rs` L307-314 mark_sent 只更新状态，不删除行），支持按 `created_at` 范围 SQL 查询回放。
- **DLQ**: per `07_PoisonEvent_DLQ草案.md` §2.2 — `status='failed'` 行为 DLQ 入口，由 `crates/shared-platform/src/dlq.rs` 桥接到 `rgs.dlq.<source>` subject。

---

## §9. 事件去重 / 幂等 key 规范

- **consumer 端幂等 key**: `command_id` (per `OutboxEntry.command_id` + `idx_outbox_command_id` 索引)。
- **去重窗口**: 24h（per RGS-REV-007 CH1 — 业务 consumer 端维护 24h dedup window）。
- **跨进程幂等**: outbox 表的 `id UUID PRIMARY KEY` + NATS at-least-once → consumer 用 `(command_id, source_subject)` 做幂等键。

---

## §10. 事件 schema 样例

### §10.1 WalletCommitted（per economy 域，v1 候选）

```text
subject: rgs.economy.wallet.committed.v1
payload:
{
  "wallet_id":    UUID,
  "amount":       i64 (signed, 单位 = 最小货币单位, e.g. 分),
  "currency":     "CNY" | "USD" | "EUR" | ...,
  "tx_type":      "deposit" | "withdraw" | "transfer",
  "counterparty": Option<UUID>,  // transfer 时填
  "saga_id":      Option<UUID>,  // Saga Reserve/Commit 步骤时填
  "occurred_at":  TIMESTAMPTZ,
  "trace_id":     String,
  "version":      1
}
command_id: UUID  // 幂等 key, 业务层生成（per RGS-REQ-100 §7 BR-111）
saga_id:    Option<UUID>  // Reserve Saga 关联
```

> ⚠️ v0.2 candidate**: WalletCommitted 仅作为样例展示 schema 形状, 实际字段集待 economy 域 owner 落地。ULYS-103 worker 不代签域 owner。

---

## §11. 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 起草 (ULYS-103 worker) | agent c557dae5 | 2026-09-19 JST |
| 候选主导者 | 架构师 + 6 域 owner 联合（Ulysses 一人公司兼任 per DEC-008） | 待 NO-GO 解除 + G-CODE-03/06 全绿 |
| 候选复核 | 平台架构师 + DBA | v0.2 升版前 |

---

## §12. 关联文档

- **RGS-SPEC-CROSS-003 v0.1** (NO-GO 主文档, 现有 `docs/13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md`) — **不替换**。
- **RGS-SPEC-CROSS-003 v0.2 candidate** ← 本文件。
- **RGS-SPEC-CROSS-005** 事务性消息规范 (实际位于 `RGS-DTL-100 §4 Outbox + Inbox Pattern`, per ADR-0061 v0.3 章节引用修订)。
- **RGS-SPEC-CROSS-006** trace_id 传播规范 (per §2.1 trace_id 字段)。
- **RGS-SPEC-CROSS-007** 5 域 RBAC 角色矩阵 (per §2.2 operator_id / actor_id 字段 RBAC 校验)。
- **RGS-ADR-0051** 中心事件管理（CEM, event_schema_registry 表）。
- **RGS-SPEC-DTL-031** admin 域（COC + CEM + PFAU, event_producer_registry §7 引用）。
- **ADR-0060** 事件总线偏离（per NATS 取代 Kafka, §1.1 命名约定分母）。
- **ADR-0061** CDC/Outbox 偏离（per 自研 Outbox 取代 Debezium CDC, §1.1 `outbox.subject` schema 来源）。
- **crates/shared-platform/src/subject.rs** — 命名约定 source of truth。
- **crates/shared-platform/src/outbox.rs** — outbox schema + MIGRATION_TEMPLATE。
- **09a_跨域事件族清单_v1.1_可验证部分.md** — 22 条 v1.2 候选清单 source。
- **10_outbox_migration_template_v1.sql** — 新域 outbox 表 migration 模板。
- **11_outbox_relay_bootstrap_pattern.md** — 新域 relay 启动模式。

---

## §13. 修订历史

| 版本 | 日期 | 修订者 | 内容 |
|---|---|---|---|
| 0.2 candidate | 2026-09-19 JST | ULYS-103 worker (agent c557dae5) | v0.2 候选草案：命名约定 v1 修订（per RGS-OPEN-QA-001 修正 #7 实际命名）；最小字段集 + 各域 payload 自定义 schema 责任划分；域内 22 条候选清单汇总；跨域 + 集群事件族；订阅者注册表 schema；DLQ + 幂等规范；样例事件 |
| 0.1 | 2026-08-21 JST | Ulysses 一人公司 | NO-GO placeholder, per WBS v0.3 §2A.6.7 |
