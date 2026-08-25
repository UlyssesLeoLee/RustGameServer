# 详细设计书（詳細設計書 / Detailed Design Document）

**网络拓扑容灾与数据分析管线：分析存储物理表设计・消费者游标格式・脱敏规则与资源隔离算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-017 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-017 网络基础设施拓扑、容灾与数据分析管线 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定，本文档是RGS-DTL-001/002/025/026/027之后本批次继续推进详细设计阶段的一部分。细化RGS-BAS-017§3.1组件的`AnalyticsEventConsumer`消费者游标与脱敏规则配置为具体DDL、§3.2数据流与§3.2.1异常分支落实为可直接翻译为Rust实现的伪代码、§3.4资源隔离的连接配额落实为具体NetworkPolicy/配额参数（含TBD数值的初始提案）。**本版本不覆盖**：`AnalyticsStore`选型（TBD-INF-002）本身的对比评审、`AnalyticsQueryUI`所复用开源BI工具的具体部署配置。见§7 | 全部 |
| 0.2 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | **同步父 BAS-017 升版至 v0.2**（1 次升版，BAS-017 v0.2 装饰性升版）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-017 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 游标表DDL是否与既有事件基础设施消费者组模式一致，异常分支伪代码是否覆盖游标丢失场景 |
| 评审（安全） | | | 脱敏规则表是否可阻止未配置规则的事件类型被默认接入（§2.3 CHECK约束是否真正生效阻断） |
| 审批（负责人） | | | 本文档的基准化；连接配额初始提案数值是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：分析管线元数据表](#2-物理数据库设计分析管线元数据表)
3. [AnalyticsEventConsumer消费流程详细设计](#3-analyticseventconsumer消费流程详细设计)
4. [资源隔离强制手段的具体参数](#4-资源隔离强制手段的具体参数)
5. [独立访问权限的具体实现](#5-独立访问权限的具体实现)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)
7. [追溯性](#7-追溯性)

---

# 1. 前言

## 1.1 定位

RGS-BAS-017给出了分析管线的组件划分、数据流时序、脱敏字段清单（文字性分类）与资源隔离的原则性描述。本文档将这些落实为：脱敏规则与消费者游标的具体DDL（落位于分析管线自身的元数据存储，非`AnalyticsStore`本身——后者的表结构随TBD-INF-002选型确定，不属于本文档范围）、消费/脱敏/异常处理流程的可直接翻译为Rust实现的算法伪代码、资源隔离连接配额与NetworkPolicy的具体参数提案。

## 1.2 本文档不做什么

- 不重新决定RGS-BAS-017已确定的任何结构性选择（`AnalyticsStore`与可观测性存储物理隔离、分析管线消费组独立于可观测性消费组、脱敏在写入前执行、分析管线权限独立评审不复用运维/GM后台RBAC）。
- 不选定`AnalyticsStore`本身（TBD-INF-002）——本文档给出的DDL仅覆盖分析管线自身需要的元数据（脱敏规则配置表、消费者游标表），不涉及`AnalyticsStore`内部的OLAP表结构，后者随选型完成后另行补充。
- 不覆盖`AnalyticsQueryUI`所复用开源BI工具的部署形态——RGS-BAS-017§3.1已确定"复用开源BI工具"，其Helm/部署细节留待该工具选型确定后按RGS-DTL-002既定模板挂载，不在本文档展开。

## 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准（元数据表依附既有可观测性/事件基础设施数据库，不新建独立库，理由见§2），算法伪代码可直接对应Rust `Result`实现，TBD数值以标注"提案，非最终值"的形式给出初始默认值。

---

# 2. 物理数据库设计：分析管线元数据表

对应RGS-BAS-017§3.1/§3.3。两张元数据表依附**既有事件基础设施的元数据库**（与ARC-009 Outbox分发器既有的消费者游标存储同库，复用其既有连接池，不为分析管线新挂载独立库——分析管线的"物理隔离"要求（ARC-035）针对的是`AnalyticsStore`本体，元数据表本身不持有业务/个人数据，不在隔离范围内）。

```sql
-- 分析消费者游标表：与可观测性消费组的游标物理隔离(独立table,不共享既有可观测性游标表)，
-- 对应RGS-BAS-017§3.2"各自的消费组/游标"要求
CREATE TABLE analytics_consumer_cursors (
    partition_key   TEXT NOT NULL,           -- 复用RGS-BAS-004既定字段规范
    consumer_group  TEXT NOT NULL DEFAULT 'analytics_pipeline',
    last_committed_offset  BIGINT NOT NULL DEFAULT 0,
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0=Running 1=Paused（对应§3.2.1"暂停该分区消费"分支）
    paused_reason   TEXT,                     -- 暂停原因（写入失败/脱敏异常），供运维排查
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (partition_key, consumer_group)
);

-- 脱敏规则配置表：按事件类型声明字段路径处理方式，对应§3.3
CREATE TABLE analytics_redaction_rules (
    event_type      TEXT PRIMARY KEY,          -- 与既有事件线格式的message类型名一致
    field_rules     JSONB NOT NULL,             -- [{ "field_path": "...", "action": "drop|truncate|hash" }, ...]
    reviewed_by     TEXT NOT NULL,              -- 评审人，对应"未配置脱敏规则不得默认接入"评审记录
    reviewed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active       BOOLEAN NOT NULL DEFAULT TRUE
);
```

`analytics_redaction_rules`未主动新建`event_type`行即视为**未配置**——`AnalyticsEventConsumer`在消费循环中对查不到规则的事件类型采取"直接丢弃、不写入`AnalyticsStore`、记录告警"而非"未脱敏直接写入"（§3.1算法体现该判定），这是RGS-BAS-017§3.3"未配置脱敏规则的事件类型不得默认接入"的物理实现落点——由消费者代码逻辑强制而非数据库约束强制（脱敏规则本身是JSONB自由结构，数据库层无法对"是否已正确脱敏"做约束校验，只能保证"存在规则行"这一前置条件，实际脱敏正确性依赖§3.1代码逻辑与评审）。

---

# 3. AnalyticsEventConsumer消费流程详细设计

对应RGS-BAS-017§3.2/§3.2.1。

## 3.1 主消费循环

```rust
fn consume_analytics_event(event: RawEvent, partition_key: &str) -> Result<(), AnalyticsConsumeError> {
    let cursor = load_or_init_cursor(partition_key, "analytics_pipeline")?;
    if cursor.status == CursorStatus::Paused {
        // 对应§3.2.1"暂停该分区消费并保留游标位置"：暂停期间不消费新事件，直到人工/自动修复后恢复
        return Err(AnalyticsConsumeError::PartitionPaused { reason: cursor.paused_reason.clone() });
    }

    let rule = lookup_redaction_rule(&event.event_type);
    let rule = match rule {
        Some(r) if r.is_active => r,
        _ => {
            // 未配置或已停用规则: 不写入AnalyticsStore,记录告警,游标仍前移(该事件类型被有意排除,
            // 非异常,不应反复重试同一条不可能通过的事件——区别于下方"写入异常"的可重试语义)
            emit_metric("analytics_event_type_unconfigured", &event.event_type);
            advance_cursor(partition_key, event.offset)?;
            return Ok(());
        }
    };

    let redacted = apply_redaction(&event, &rule)?;  // §3.1脱敏字段清单落地：drop/truncate/hash三种action

    match write_to_analytics_store(&redacted) {
        Ok(()) => {
            advance_cursor(partition_key, event.offset)?;
            Ok(())
        }
        Err(write_err) => {
            // 写入异常：不前移游标，按ARC-009标准重试策略重试(§3.2.1)
            Err(AnalyticsConsumeError::WriteFailed(write_err))
        }
    }
}
```

## 3.2 重试耗尽后的暂停与恢复

```rust
fn on_retry_exhausted(partition_key: &str, err: AnalyticsConsumeError) {
    pause_cursor(partition_key, "analytics_pipeline", format!("{:?}", err));  // §3.2.1"暂停该分区消费并保留游标位置"
    emit_alert("analytics_consumer_paused", partition_key);
    // 不影响可观测性消费组: pause_cursor仅操作analytics_consumer_cursors表，
    // 与可观测性既有游标表物理隔离，二者互不可见，故障不传导(§3.2.1既定设计)
}

fn resume_after_fix(partition_key: &str) -> Result<(), AnalyticsConsumeError> {
    let cursor = load_cursor(partition_key, "analytics_pipeline")?;
    if !cursor_offset_still_within_replay_window(&cursor) {
        // 游标已超出ARC-010事件流可重放窗口: 走全量重建路径(§3.2.1)
        return Err(AnalyticsConsumeError::CursorLost {
            gap_note: "超出可重放窗口的历史数据视为不可恢复缺口，需在分析报表中标注".into(),
        });
    }
    set_cursor_status(partition_key, "analytics_pipeline", CursorStatus::Running)?;
    // 从暂停位置(last_committed_offset)继续消费，不重放全部历史(§3.2.1"避免与AnalyticsStore已有数据产生重复聚合口径偏差")
    Ok(())
}
```

**关键边界条件说明**："未配置脱敏规则"与"写入异常"两种失败在游标前移策略上**刻意不同**——前者游标照常前移（该事件类型的这一条记录本就不该进入`AnalyticsStore`，重试无意义且会导致该分区永久卡在同一条"必然失败"的事件上）；后者游标保留不前移（写入异常通常是暂时性的存储侧问题，值得重试，前移游标会丢失这条本应写入的数据）。这一区分是RGS-BAS-017§3.2.1文字描述"消费失败"与"未配置规则不得默认接入"两条独立设计点在同一循环中共存时必须显式区分的边界条件，原基本设计文字未明确交待两者交织时的游标处理差异，本文档在此处做出的是**实现细节补充**而非结构性变更（不改变"未配置不得接入"与"失败要重试"两条既有规则本身，只是把两者放入同一循环时的顺序/游标处理具体化）。

---

# 4. 资源隔离强制手段的具体参数

对应RGS-BAS-017§3.4，"具体数值详细设计确定"在此落实为初始提案。

## 4.1 NetworkPolicy规则片段

复用RGS-DTL-002§3已确立的default-deny+allow-list两段式模板，`AnalyticsStore`作为独立组件按该模板挂载，`allowedIngressFrom`**仅**含`analytics-event-consumer`与`analytics-query-ui`两个来源，不含任何运维可观测性组件的Pod selector——这是§3.4"运维查询组件无权限连接AnalyticsStore"的物理落地方式，直接复用RGS-DTL-002模板，不新增机制。

## 4.2 连接配额与超时（提案初始值，非最终值）

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| `AnalyticsQueryUI`单用户并发查询数上限 | 3 | 防止单一分析用户的多标签页/仪表盘并行查询耗尽存储实例资源，3为常见BI工具默认并发量级的保守取值 |
| 单查询超时 | 60秒 | 覆盖典型大范围扫描聚合查询耗时，超出视为异常查询模式，超时后连接强制释放 |
| 全局并发查询数上限 | 20 | 与`AnalyticsStore`选型（TBD-INF-002）实测吞吐相关，此值为选型前的保守占位，选型完成后应结合实测重新校准 |

以上数值应在`AnalyticsStore`选型完成、进行NFR-INF-003验证测试后按实测吞吐校准，本文档提案仅供初始上线使用。

---

# 5. 独立访问权限的具体实现

对应RGS-BAS-017§3.5，落实为角色定义的具体存储位置。

```sql
-- 分析管线角色定义表(依附既有GM后台AD限界上下文数据库，复用RGS-BAS-003既有RBAC存储结构，
-- 但角色定义本身独立于既有GM角色矩阵，不与之共享同一张角色表——同RGS-BAS-017§3.5"独立评审与分配"要求)
CREATE TABLE analytics_access_roles (
    gm_user_id      BIGINT NOT NULL,     -- 逻辑引用RGS-BAS-003既有GM用户身份，不建物理FK(跨限界上下文既定规则)
    role            TEXT NOT NULL CHECK (role IN ('分析只读用户', '分析管理员')),
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    granted_by      BIGINT NOT NULL,
    PRIMARY KEY (gm_user_id, role)
);
```

`analytics_access_roles`的授权/变更流程复用RGS-BAS-003§7审计设计的同类留痕原则（本表本身即是权限记录，变更即新增/删除行，配合既有审计日志基础设施记录`granted_by`/`granted_at`已足够留痕，不额外重复建审计表）。

---

# 6. 本文档的覆盖范围与后续计划

本文档覆盖：分析管线消费者游标表与脱敏规则配置表的物理DDL、`AnalyticsEventConsumer`主消费循环与异常/暂停/恢复的完整伪代码（含"未配置规则"与"写入异常"两种失败的游标处理差异化边界条件）、资源隔离NetworkPolicy规则的具体复用方式与连接配额初始提案数值、独立访问权限角色表的物理设计。

本版本明确不覆盖、留待后续：

- `AnalyticsStore`本体的选型（TBD-INF-002）与内部OLAP表结构——选型确定后需要独立DTL章节或新版本本文档补充其表结构，本文档仅覆盖分析管线自身元数据。
- `AnalyticsQueryUI`所复用开源BI工具的具体部署配置——待该工具选型确定后按RGS-DTL-002既定Helm/CI模板挂载，本文档不预先设计。
- §4.2连接配额三项数值的正式校准——当前为初始提案，需等待`AnalyticsStore`选型完成后的实测数据。
- 多区域评估门禁（RGS-BAS-017§2.3，TBD-INF-001）的具体阈值——该门禁与OLU申领流程集成的机制已由RGS-BAS-017§2.3与RGS-DTL-009§4.4的OLU台账检查共同覆盖，本文档不重复展开具体阈值本身（阈值评审属独立TBD，非物理/协议格式细化对象）。

后续详细设计建议顺序：与RGS-DTL-018/020/021同批次并行推进，均为02-运维安全与网络域现存BAS文档中此前止步于基本设计层的部分。

---

# 7. 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-017§3.1 组件划分 | §2 |
| RGS-BAS-017§3.2 数据流时序 | §3.1 |
| RGS-BAS-017§3.2.1 消费异常分支 | §3.2 |
| RGS-BAS-017§3.3 脱敏字段清单 | §2、§3.1 |
| RGS-BAS-017§3.4 资源隔离强制手段 | §4 |
| RGS-BAS-017§3.5 独立访问权限 | §5 |
| RGS-DTL-002（挂载脚手架/NetworkPolicy模板复用） | §4.1 |
