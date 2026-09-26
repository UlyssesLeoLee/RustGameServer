# 详细设计书（詳細設計書 / Detailed Document）

**GM 扩展域（GM-Extra Domain）详细设计 — 批量操作 / 报表 / 审计物理 DDL + 批量补偿 EC 事务边界**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-051 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-044 v0.1 GM 扩展域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/gm-extra-service/` (gm_extra_db, 416 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-052 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-044 §3〜§7 为：gm_extra_db 物理 DDL（`batch_operations`/`batch_progress`/`ops_reports`/`audit_log_index`/`batch_configs`）、批量补偿 EC 事务边界（per FR-GOV-001）、批量进度追踪伪代码、审计日志索引设计（per LC-005 合规要求） | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 批量补偿是否严格走 EC 单点；进度追踪是否完整 |
| 评审（DBA） | | | gm_extra_db 索引是否覆盖高频路径（按时间/操作人查询） |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：gm_extra_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

RGS-BAS-044 给出了 GM 扩展域 1 service 的组件划分、接口契约、核心时序、ARC-052 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-044 已确定的任何结构性选择。
- 不覆盖单玩家 GM 操作（属 gm-backend 既有）。
- 不覆盖 GM 后台 UI。
- 不覆盖 RBAC 权限矩阵（属 RGS-BAS-003 既有）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：gm_extra_db

gm_extra_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 5 张表。

## 2.1 batch_operations（批量操作实例表）

```sql
CREATE TABLE batch_operations (
    batch_op_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_type     SMALLINT NOT NULL,  -- 0=BAN 1=COMPENSATE 2=UNBAN
    filter_json        JSONB NOT NULL,     -- 条件筛选参数
    target_count       INTEGER NOT NULL,
    operator_id        UUID NOT NULL,      -- GM 操作人
    status             SMALLINT NOT NULL DEFAULT 0,  -- 0=Pending 1=Running 2=Done 3=Failed 4=Cancelled
    request_id         VARCHAR(64) NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at       TIMESTAMPTZ,
    audit_level        SMALLINT NOT NULL DEFAULT 0,  -- 0=SUMMARY 1=FULL
    UNIQUE (operator_id, request_id),
    CONSTRAINT chk_batch_ops_type CHECK (operation_type BETWEEN 0 AND 2),
    CONSTRAINT chk_batch_ops_status CHECK (status BETWEEN 0 AND 4)
);
CREATE INDEX idx_batch_operations_operator_time
    ON batch_operations (operator_id, created_at DESC);
```

## 2.2 batch_progress（批量操作进度表）

```sql
CREATE TABLE batch_progress (
    batch_op_id        UUID NOT NULL REFERENCES batch_operations(batch_op_id),
    succeeded          INTEGER NOT NULL DEFAULT 0,
    failed             INTEGER NOT NULL DEFAULT 0,
    last_update_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_processed_index INTEGER NOT NULL DEFAULT 0,  -- 用于断点续传
    PRIMARY KEY (batch_op_id)
);
```

## 2.3 ops_reports（运营报表表）

```sql
CREATE TABLE ops_reports (
    report_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type        SMALLINT NOT NULL,  -- 0=DAILY 1=WEEKLY 2=MONTHLY
    period_start       TIMESTAMPTZ NOT NULL,
    period_end         TIMESTAMPTZ NOT NULL,
    metrics_json       JSONB NOT NULL,     -- 业务指标聚合
    export_csv_uri     TEXT,               -- CSV 导出文件 URI
    export_xlsx_uri    TEXT,               -- Excel 导出文件 URI
    generated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    olu_cost           INTEGER NOT NULL DEFAULT 0,  -- ARC-026 OLU 预算消耗
    CONSTRAINT chk_ops_reports_type CHECK (report_type BETWEEN 0 AND 2)
);
CREATE INDEX idx_ops_reports_period
    ON ops_reports (period_start DESC);
```

## 2.4 audit_log_index（审计日志索引表）

```sql
CREATE TABLE audit_log_index (
    audit_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_op_id        UUID,                -- 关联 batch_operations (可为 NULL, 单玩家操作无 batch)
    operator_id        UUID NOT NULL,
    target_player_id   UUID,                -- 操作目标玩家
    operation_type     VARCHAR(32) NOT NULL,
    operation_detail   JSONB NOT NULL,
    occurred_at        TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (occurred_at);
-- 月度分区, 复用 RGS-BAS-007 既定短生命周期表分区/保留期标准
CREATE INDEX idx_audit_log_operator_time
    ON audit_log_index (operator_id, occurred_at DESC);
CREATE INDEX idx_audit_log_target_time
    ON audit_log_index (target_player_id, occurred_at DESC);
CREATE INDEX idx_audit_log_operation_time
    ON audit_log_index (operation_type, occurred_at DESC);
-- 3 索引覆盖 3 种过滤 (per FR-GMX-030)
```

## 2.5 batch_configs（批量配置表）

```sql
CREATE TABLE batch_configs (
    operation_type     SMALLINT PRIMARY KEY,  -- 0=BAN 1=COMPENSATE 2=UNBAN
    max_batch_size     INTEGER NOT NULL,
    require_2fa        BOOLEAN NOT NULL DEFAULT TRUE,
    audit_level        SMALLINT NOT NULL DEFAULT 1,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_batch_configs_op CHECK (operation_type BETWEEN 0 AND 2)
);
```

---

# 3. 关键算法详细设计

## 3.1 批量补偿 EC 单点（per FR-GOV-001 落实）

```rust
// 落实 RGS-BAS-044 §5.2 批量补偿经 EC 单点
// 每目标一次 commit_transaction, 保证:
//   1. 单点发放 (per FR-GOV-001)
//   2. 失败可重试 (Saga 补偿)
//   3. 全程审计
fn batch_compensate(
    target_players: Vec<PlayerId>,
    reward_spec: RewardSpec,
    operator_id: OperatorId,
    request_id: &str,
) -> Result<BatchOpId, GmError> {
    // 1. 初始化 batch_operations
    let batch_op_id = insert_batch_operation(BatchOp::Compensate, &target_players, operator_id, request_id)?;
    insert_batch_progress(batch_op_id, target_players.len())?;
    // 2. 校验单次上限 (per NFR-GMX-004)
    let config = load_batch_config(OperationType::Compensate)?;
    if target_players.len() > config.max_batch_size as usize {
        return Err(GmError::BatchTooLarge { limit: config.max_batch_size });
    }
    // 3. 逐目标经 EC 单点 (per CON-GMX-003)
    let mut succeeded = 0;
    let mut failed = 0;
    for (i, player_id) in target_players.iter().enumerate() {
        match commit_transaction(CommitTransactionRequest {
            request_id: deterministic_hash(&[batch_op_id.as_bytes(), player_id.as_bytes()]),
            character_id: (*player_id).into(),
            operation: Operation::GrantItems(reward_spec.clone()),
            session_epoch: current_session_epoch(*player_id)?,
            expected_version: current_wallet_version(*player_id)?,
        }) {
            Ok(_) => succeeded += 1,
            Err(e) => {
                failed += 1;
                log::warn!("batch_compensate player={} failed: {}", player_id, e);
            }
        }
        // 进度更新 (断点续传关键)
        update_batch_progress(batch_op_id, succeeded, failed, last_processed_index: i)?;
    }
    // 4. 审计落痕 × N
    for player_id in &target_players {
        insert_audit_log(AuditEntry {
            batch_op_id, operator_id, target_player_id: *player_id,
            operation_type: "BATCH_COMPENSATE",
            operation_detail: json!({"reward_spec": reward_spec}),
        })?;
    }
    // 5. 完成
    complete_batch_operation(batch_op_id, succeeded, failed)?;
    Ok(batch_op_id)
}
```

## 3.2 批量进度查询 + 断点续传

```rust
// 落实 FR-GMX-003 进度查询 + 断点续传
fn query_batch_progress(batch_op_id: BatchOpId) -> Result<BatchProgress, GmError> {
    let op = load_batch_operation(batch_op_id)?;
    let prog = load_batch_progress(batch_op_id)?;
    Ok(BatchProgress {
        batch_op_id,
        total: op.target_count,
        succeeded: prog.succeeded,
        failed: prog.failed,
        status: op.status,
    })
}

// 断点续传: 从 last_processed_index 继续
async fn resume_batch_if_needed(batch_op_id: BatchOpId) -> Result<(), GmError> {
    let op = load_batch_operation(batch_op_id)?;
    if op.status != BatchStatus::Running as i16 {
        return Ok(());  // 已完成或失败, 不需要续传
    }
    let prog = load_batch_progress(batch_op_id)?;
    let remaining = op.target_count as i32 - prog.last_processed_index - 1;
    if remaining <= 0 {
        complete_batch_operation(batch_op_id, prog.succeeded, prog.failed)?;
        return Ok(());
    }
    // 从 last_processed_index + 1 继续处理
    // (具体处理逻辑取决于操作类型, 此处省略)
    Ok(())
}
```

## 3.3 玩家画像 8 维度并行聚合

```rust
// 落实 RGS-BAS-044 §5.3 玩家画像 8 维度聚合
// 8 维度并行查询, 不串行 (per NFR-GMX-001 性能)
async fn get_player_profile(player_id: PlayerId) -> Result<PlayerProfile, GmError> {
    let (account, characters, wallet, txn, battle, social, inventory, behavior) = tokio::try_join!(
        call_player_get_account(player_id),
        call_player_get_characters(player_id),
        call_economy_get_wallet(player_id),
        call_economy_get_transaction_summary(player_id),
        call_battle_get_battle_stats(player_id),
        call_social_get_social_graph(player_id),
        call_economy_get_inventory_summary(player_id),
        call_player_get_behavior_timeline(player_id),
    )?;
    // 隐私过滤 (per NFR-SE-005)
    let profile = PlayerProfile {
        account, characters, wallet, transaction_summary: txn,
        battle_stats: battle,
        social_graph: filter_social_privacy(social),
        inventory, behavior_timeline: behavior,
    };
    Ok(profile)
}
```

## 3.4 审计日志导出（per FR-GMX-031 合规）

```rust
// 落实 FR-GMX-031 审计导出 (per LC-005 合规要求)
// 导出格式: CSV + 签名
fn export_audit_log(filter: AuditFilter, format: ExportFormat) -> Result<ExportUri, GmError> {
    let entries: Vec<AuditEntry> = query_audit_log_with_filter(filter)?;
    match format {
        ExportFormat::Csv => {
            let csv = generate_csv(&entries)?;
            let signed_uri = sign_and_store(csv, "audit_export.csv")?;
            Ok(signed_uri)
        }
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&entries)?;
            let signed_uri = sign_and_store(json, "audit_export.json")?;
            Ok(signed_uri)
        }
    }
}
```

---

# 4. 对接点

## 4.1 与 gm-backend 既有的对接（CON-GMX-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 单玩家封号 × N | `BanPlayer` | gm-backend 既有 | 批量循环调用 |
| 单玩家补偿 | 走 EC 单点 | RGS-DTL-001 §3.2 | 不走 gm-backend |

## 4.2 与 EC 的对接（CON-GMX-003 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 批量补偿 | `commit_transaction(GrantItems)` × N | RGS-DTL-001 §3.2 | 走 EC 既有路径 |

## 4.3 与 RGS-BAS-003 RBAC 的对接（CON-GMX-002 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| RBAC 校验 | `CheckPermission(operator_id, op)` | RGS-BAS-003 §3 | 本域**不**自建 RBAC |

## 4.4 与 ARC-026 OLU 预算的对接（CON-GMX-004 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 报表生成 | `CheckOLUBudget(report_type)` | ARC-026 既有 | 超限拒绝 |

## 4.5 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| BatchConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 仅订阅 config_updated |

## 4.6 与既有审计日志的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 审计写入 | `InsertAuditLog` | gm-backend 既有 | 本域索引 |
| 审计查询 | `QueryAuditLog` | 本域索引表 | 高频查询路径 |

---

# 5. 数据库 DDL 权威边界

gm_extra_db 共 5 张表，以本文档为唯一权威。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| 批量封号（每批 1000 人） | < 30s | per NFR-GMX-001 | PH-4 |
| 玩家画像 8 维度并行 | < 200ms | tokio::try_join! 并行 | PH-4 |
| 审计日志按时间过滤 | < 100ms | idx_audit_log_*_time | PH-2 |
| 报表生成 | < 60s | PH-4 性能校准 | PH-4 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：

- gm_extra_db 5 张表的物理 DDL（含 audit_log_index 月度分区）
- 批量补偿 EC 事务边界（per FR-GOV-001）
- 批量进度查询 + 断点续传（per FR-GMX-003）
- 玩家画像 8 维度并行聚合
- 审计日志导出（per LC-005）
- 6 项对接点

本版本明确不覆盖、留待后续：

- 单玩家 GM 操作 — 属 gm-backend 既有
- RBAC 权限矩阵 — 属 RGS-BAS-003 既有
- OLU 预算台账 — 属 ARC-026 既有
- TBD-GMX-001（单次批量上限）— 留待 PH-2 性能校准

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-044 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-044 §3 业务需求 | §2, §3 |
| RGS-REQ-044 §4 功能需求 | §3.1, §3.3 |
| RGS-REQ-044 §5 NFR | §6 |
| RGS-REQ-044 §6 ARC-052 | §2.5, §3 |
| RGS-REQ-044 §7 AC-GMX-001〜005 | §3.1, §3.2, §3.3 |
| RGS-REQ-044 §8 RSK-GMX-001 | §3.1 |
| RGS-BAS-044 §3 架构总览 | §2 |
| RGS-BAS-044 §4 组件设计 | §3.1〜3.4 |
| RGS-BAS-044 §5 数据流时序 | §3.1, §3.3 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.1 |
| LC-005 合规要求 | §2.4, §3.4 |
