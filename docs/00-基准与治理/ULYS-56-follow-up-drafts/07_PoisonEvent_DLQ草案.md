# ULYS-56 P2-#3 草案: Poison Event 处理流程 (Failed 状态 4 状态机的 DLQ 路径)

> **状态**: 草案 (per ADR-0061 §6 P2-#3)。本文件是**候选 DLQ 流程草案**, 待 ADR-0061 审批通过后正式立项。
>
> **依据**: RGS-ADR-0061 §6 P2-#3 (Poison event 处理流程)
>
> **ULYS-56 工作范围声明**: 本文件由 ULYS-56 worker (agent c557dae5) 2026-09-16 起草, 提供 DLQ 候选流程, **不直接修改 outbox.rs 代码或部署文档**。

---

## 1. 当前状态与缺口 (per ADR-0061 §6 P2-#3)

**当前** (`crates/shared-platform/src/outbox.rs`): 4 状态机 `Failed` 状态仅 `last_error` 字段记录错误信息, **无 DLQ 表 / 重试策略 / 告警阈值**。

**缺口**:
1. 无独立 `outbox_dlq` 表承载 Poison event (永久失败的 outbox 行)
2. 无重试次数上限（当前可无限次 InFlight → Pending 循环）
3. 无告警阈值与人工介入流程（与监控指标草案 §06 P2-#2 联动）
4. 无 DLQ 事件 replay / purge 工具

---

## 2. 候选方案: 重试次数上限 + DLQ 表 + 人工 replay

### 2.1 重试次数上限 (5 次)

**新增字段**: `crates/shared-platform/src/outbox.rs::OutboxEntry` 增加 `retry_count: u32` 字段。

**状态机修订**:

```
Pending → InFlight (Worker 持锁)
InFlight → Sent (NATS ACK 成功)
InFlight → Failed (retry_count >= 5 时直接 Failed, 否则 → Pending)
InFlight → Pending (lease 过期 / 发布失败无 ACK, retry_count += 1)
任意状态 → Failed (重试超阈值)
```

**迁移路径**: 现有 6 域 outbox 表 schema migration `0005_outbox_retry_count.sql`, 默认值 `0`。

### 2.2 DLQ 表 (`outbox_dlq`)

**Schema** (per service, 6 域各自):

```sql
-- crates/{service}/migrations/0006_outbox_dlq.sql
CREATE TABLE outbox_dlq (
    dlq_id BIGSERIAL PRIMARY KEY,
    original_outbox_id BIGINT NOT NULL,    -- 引用原 outbox 行 ID (per service)
    event_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    trace_id UUID,
    retry_count INT NOT NULL,
    first_failed_at TIMESTAMPTZ NOT NULL,
    last_failed_at TIMESTAMPTZ NOT NULL,
    last_error TEXT NOT NULL,
    disposition TEXT NOT NULL DEFAULT 'pending',  -- pending / replayed / discarded / escalated
    disposition_at TIMESTAMPTZ,
    disposition_actor TEXT,
    disposition_note TEXT
);

CREATE INDEX idx_outbox_dlq_disposition_pending
    ON outbox_dlq(disposition, last_failed_at DESC)
    WHERE disposition = 'pending';
```

**触发**: 原 outbox 行 `status = 'Failed' AND retry_count >= 5` 时, 由 `outbox_worker` 同步 INSERT 至 `outbox_dlq` + UPDATE 原行 `status = 'Failed', dlq_inserted_at = NOW()`。

**索引**: `(disposition, last_failed_at DESC)` 部分索引, 加速"待处置"列表查询。

### 2.3 告警阈值（与监控指标草案 P2-#2 联动）

| 触发条件 | 告警级别 | 通知渠道 |
|---|---|---|
| `rgs_outbox_failed_count > 0` 持续 1min | critical | SRE on-call + 域 owner |
| `rgs_outbox_dlq_pending_count > 10` | warning | 域 owner |
| `rgs_outbox_dlq_pending_age_seconds > 3600` | critical | 域 owner + 架构师 |

### 2.4 人工介入流程 (CLI 工具 + 文档)

**CLI 工具** (候选):

```bash
# 列出待处置 DLQ 事件
rgs-admin outbox dlq list --service economy --status pending

# 处置选项:
rgs-admin outbox dlq replay --service economy --dlq-id 12345 --actor ulf   # 重发至 NATS
rgs-admin outbox dlq discard --service economy --dlq-id 12345 --reason "schema invalid, no remediation" --actor ulf
rgs-admin outbox dlq escalate --service economy --dlq-id 12345 --reason "manual investigation required" --actor ulf
```

**审计**: 所有 disposition 操作记录 `disposition_actor` + `disposition_at` + `disposition_note`, 满足 NFR-SE-010 「不可变审计」要求。

---

## 3. 候选实施路径

| 阶段 | 内容 | 责任方 |
|---|---|---|
| Stage 1 | `crates/shared-platform/src/outbox.rs` 增加 `retry_count` 字段 + 状态机修订 (5 次上限) | 架构师 |
| Stage 2 | 6 域 migrations 新增 `0005_outbox_retry_count.sql` + `0006_outbox_dlq.sql` | 架构师 + DBA |
| Stage 3 | `outbox_worker` 实现 DLQ 同步插入逻辑 (Failed → outbox_dlq) | 架构师 |
| Stage 4 | `rgs-admin` CLI 新增 `outbox dlq` 子命令 (list / replay / discard / escalate) | 架构师 + SRE |
| Stage 5 | 监控指标草案 P2-#2 增加 DLQ 指标 + 告警规则 | SRE Lead |
| Stage 6 | RGS-OB-DOC-002 新增"Poison event 处置 SOP" (≤ 4 页) | SRE Lead + 域 owner |

---

## 4. 与其他决策的关联

- **ADR-0061 §6 P2-#2** (监控指标): DLQ 指标 `rgs_outbox_dlq_pending_count` / `rgs_outbox_dlq_pending_age_seconds` 须加入指标清单
- **NFR-SE-010** (不可变审计): `outbox_dlq` 表 disposition 操作须追加至 `operation_audit`
- **RGS-BAS-001 §5.8** (Outbox 通用表结构范式): 扩展 `outbox_dlq` 至通用范式
- **RGS-DTL-100 §4** (Outbox + Inbox Pattern): §4.1 增加 DLQ 章节
- **RGS-REV-007 CH1+CH2+AH1** (Outbox 升级记录): 历史已识别 poison event 风险, 本草案是落地路径

---

## 5. 候选操作者签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 起草 (ULYS-56 worker) | worker (agent c557dae5) | 2026-09-16 JST |
| 候选主导者 | 架构师（Ulysses 一人公司兼任 per DEC-008） | 待 ADR-0061 具名审批通过后 |
| 候选复核 | SRE Lead + DBA + 域 owner | Stage 1-6 顺序 |

---

## 6. 关联文档

- **RGS-ADR-0061 §6 P2-#3**: 后续工作项
- **RGS-ADR-0061 §6 P2-#2**: 监控指标 (联动)
- **RGS-ADR-0061 §2 决定 2**: 事务内强制 outbox 写入约束 (DLQ 是其失效兜底)
- **`crates/shared-platform/src/outbox.rs`**: L52-75 (OutboxStatus 枚举) + L162-195 (`append`)
- **`crates/economy-service/tests/integration_outbox.rs`**: 集成测试 (DLQ 测试待扩展)
- **NFR-SE-010**: 不可变审计要求