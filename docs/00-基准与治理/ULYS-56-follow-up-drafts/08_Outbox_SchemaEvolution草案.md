# ULYS-56 P2-#4 草案: Outbox Schema Evolution 流程

> **状态**: 草案 (per ADR-0061 §6 P2-#4)。本文件是**候选 schema 演进流程草案**, 待 ADR-0061 审批通过后正式立项。
>
> **依据**: RGS-ADR-0061 §6 P2-#4 (Outbox schema evolution 流程)
>
> **ULYS-56 工作范围声明**: 本文件由 ULYS-56 worker (agent c557dae5) 2026-09-16 起草, 提供流程候选清单, **不直接修改 DDL 或 migration 工具**。

---

## 1. 当前状态与缺口 (per ADR-0061 §6 P2-#4)

**当前**: 6 域 outbox 表通过逐次 `0XXX_outbox*.sql` migration 演进 (per `crates/admin-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql` 模式)。

**缺口**:

1. 无显式 schema 演进流程文档（migration 是隐式约定）
2. 无回滚策略（如何撤销破坏性 schema 变更）
3. 无兼容性策略（旧 outbox 行 schema 与新 schema 不匹配时如何处理）
4. 无跨域协调机制（6 域独立迁移时如何保持 outbox 表结构一致）

---

## 2. 候选方案: Expand-Contract 模式 + 跨域 migration 协调

### 2.1 Expand-Contract 模式 (per ARC-015 已采纳)

**Expand 阶段**:

1. 新增列（向后兼容, 旧代码不感知）
2. 部署：先迁移 schema, 再部署新代码（读旧列 + 写新列）
3. 验证：新列数据正确填充

**Contract 阶段**:

1. 移除旧列（破坏性, 但确认无代码再读旧列后再执行）
2. 部署：先部署不再读旧列的代码, 再迁移 schema 移除旧列
3. 验证：旧列引用检查无残留

**示例** (outbox.rs L52-75 OutboxStatus 扩展):

```sql
-- Expand: 增加 OutboxStatus 枚举值
-- crates/shared-platform/src/outbox.rs::OutboxStatus 添加新变体
-- 但 PostgreSQL ENUM 类型需 migration 配合

-- 示例: 增加 'Paused' 状态 (暂停某 aggregate_type 的 relay)
ALTER TYPE outbox_status ADD VALUE 'Paused';
-- 注: PostgreSQL ENUM ADD VALUE 在事务内有约束, 需单独事务或 ALTER TYPE ... COMMIT
```

### 2.2 跨域 migration 协调机制 (候选)

**问题**: 6 域 outbox 表结构由 `crates/shared-platform/src/outbox.rs` 驱动, 但各域独立 migration。`shared_platform` 库升级时, 6 域 migration 必须同步更新。

**候选机制**:

| 机制 | 描述 | 实施复杂度 |
|---|---|---|
| **共享 migration 模板** | `crates/shared-platform/migrations/` 提供 SQL 模板, 各域 `migrations/` 通过 build.rs 或 include_str! 引用 | 中 (需修改构建系统) |
| **CI 校验** (推荐) | `scripts/check-outbox-schema-consistency.sh` 对比 6 域 `outbox` 表 DDL, 确保一致 | 低 (脚本化) |
| **schema 契约测试** | `crates/shared-platform/tests/outbox_schema_contract.rs` 集成测试, 各域 CI 跑通 | 中 (需新增测试) |
| **共享 schema registry** | 升级 `shared_platform` 时, 由架构师同步 push 6 域 PR | 高 (流程约束, 无技术保障) |

**推荐**: 机制 (CI 校验) + (schema 契约测试) 双轨, 短期 CI 校验落地, 中期契约测试覆盖。

### 2.3 回滚策略 (候选)

| 变更类型 | 回滚方法 |
|---|---|
| 新增列 | 直接 DROP COLUMN (无数据丢失风险, 新列无旧数据) |
| 新增 ENUM 值 | PostgreSQL ENUM 不支持 DROP VALUE, 需重建 ENUM 类型 + 数据迁移 (per Expand-Contract 撤销) |
| 修改列类型 | 双写期: 旧列保留 + 新列填充; 撤销时 DROP 新列 + 回滚代码 |
| 删除列 | 不可直接回滚, 需从备份恢复 (破坏性操作, 须 DBA + 架构师双签) |

**关键原则**: 任何破坏性 schema 变更须 DBA + 架构师具名审批 (per DEC-008), 且仅在维护窗口执行。

### 2.4 兼容性策略

**消费者侧** (per §4.2 Inbox): 消费者按 `event_id` 去重, schema 演进通过事件 envelope 的 `version` 字段 (per RGS-SPEC-CROSS-003 v0.2 主题命名空间 `rgs.events.<domain>.<aggregate>.<action>.<version>`)。

**生产者侧**: 新字段写入时保留旧字段（双写期）, 验证无误后撤回旧字段。

---

## 3. 候选实施路径

| 阶段 | 内容 | 责任方 |
|---|---|---|
| Stage 1 | `scripts/check-outbox-schema-consistency.sh` CI 校验脚本 (对比 6 域 outbox DDL 一致性) | 架构师 + SRE |
| Stage 2 | `crates/shared-platform/tests/outbox_schema_contract.rs` 集成测试 (6 域 outbox 表结构契约) | 架构师 |
| Stage 3 | RGS-DTL-100 §4.3 新增"Schema Evolution 流程"章节 (Expand-Contract + 跨域协调 + 回滚) | 架构师 |
| Stage 4 | DBA 维护窗口手册新增"Outbox 表回滚 SOP" (≤ 3 页) | DBA + 架构师 |
| Stage 5 | 共享 migration 模板机制 (机制 1) 评估 + 实施（如决定采纳） | 架构师 + DBA |

---

## 4. 候选操作者签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 起草 (ULYS-56 worker) | worker (agent c557dae5) | 2026-09-16 JST |
| 候选主导者 | 架构师（Ulysses 一人公司兼任 per DEC-008） | 待 ADR-0061 具名审批通过后 |
| 候选复核 | DBA + SRE | Stage 1-5 顺序 |

---

## 5. 关联文档

- **RGS-ADR-0061 §6 P2-#4**: 后续工作项
- **ARC-015** (Expand-Contract 模式, 已采纳 per §3 ADR-0014)
- **RGS-SPEC-CROSS-003 v0.2** (事件 envelope version 字段 + 主题命名空间)
- **RGS-DTL-100 §4** (Outbox + Inbox Pattern, 待扩展 §4.3)
- **RGS-REV-007** (Outbox 升级记录, 历史 migration 模式参考)
