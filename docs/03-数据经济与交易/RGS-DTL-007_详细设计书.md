# 详细设计书（詳細設計書 / Detailed Design Document）

**数据库设计标准落地示例：命名/索引/分区/迁移标准的字面化DDL与工具链配置详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-007 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-007 数据库设计标准与存储过程使用规范 基本设计书（本文档为其详细化，不改变任何既有决定，仅将标准条文落实为字面可执行示例） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档接续RGS-DTL-001/002/025/026/027批次，与RGS-DTL-015/016同批次产出）。RGS-BAS-007本身是**标准文档**而非某一限界上下文的逻辑设计，故本文档不重新推导标准本身，只给出：①一份满足§2〜§4全部规则的模板`CREATE TABLE`（示范命名/索引/分区/OCC/审计列的字面写法，供其余DTL文档在引用RGS-BAS-007时可直接对照）②§5迁移流程实际使用的迁移工具具体调用命令与CI校验脚本骨架③§7存储过程例外的登记表具体schema。**本版本不覆盖**任何具体限界上下文的实际业务表（分别由各自DTL文档给出，如RGS-DTL-001§2/§3、RGS-DTL-025§2、RGS-DTL-026§2、RGS-DTL-015/016本身）。见§6 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 模板DDL是否字面满足RGS-BAS-007§2〜§4全部规则条目，是否存在示例本身违反标准的疏漏 |
| 评审（DBA） | | | 迁移工具调用命令与既有CI"migrations校验"阶段（RGS-BAS-002§4.2）的实际兼容性 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [标准落地：模板表DDL](#2-标准落地模板表ddl)
3. [标准落地：分区滚动创建工具化](#3-标准落地分区滚动创建工具化)
4. [标准落地：迁移工具具体调用](#4-标准落地迁移工具具体调用)
5. [标准落地：存储过程例外登记表](#5-标准落地存储过程例外登记表)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-007给出命名规范表、索引/分区设计标准、迁移流程设计、备份恢复标准、存储过程例外评审流程——均以规则条文与流程图形式表达，本身只规定标准、不提供具体限界上下文的DDL。具体DDL由对应分域RGS-DTL章节产出（当前为RGS-DTL-001§2.1／§3.1、RGS-DTL-015§2、RGS-DTL-016§2、RGS-DTL-025§2与RGS-DTL-026§2），而非集中于单一文档；故本文档承担的角色是：给出一份**字面示范**——不属于任何真实业务限界上下文，但逐条对照RGS-BAS-007规则可验证合规的模板`CREATE TABLE`，供各业务域DTL文档在编写自己的DDL时可直接参照句法，以及RGS-BAS-007§5/§7中仅以文字/流程图描述、尚未给出具体命令行/表结构的两处补齐为可执行形式。

### 1.2 本文档不做什么

- **不重新决定标准本身**：命名规则、索引选型原则、分区粒度选择、Expand-Contract纪律、存储过程允许边界，均为RGS-BAS-007§2〜§7已确定内容，本文档逐条引用而非重新论证。
- **不是任何业务域的物理设计**：模板表本身不对应任何真实限界上下文的业务实体，仅用于示范句法；各业务域的实际DDL已分别由RGS-DTL-001（player_db/economy_db）、RGS-DTL-025（admin_db反作弊三表）、RGS-DTL-026（match_db匹配三表）、RGS-DTL-015/016（本批次另两份文档）给出，本文档不重复。
- **不选定RGS-BAS-007未点名的具体迁移工具**——本文档给出的迁移工具调用示例假定采用`sqlx-cli`（Rust生态原生、与ARC-023"业务逻辑不入库"精神一致的最小化工具），但最终工具选型若与实现阶段既有CI基础设施冲突，以实现阶段实际配置为准，本文档不构成强制指定。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，命令行示例以POSIX shell语法给出。

---

## 2. 标准落地：模板表DDL

以下模板表`example_domain_events`不对应任何真实业务实体，逐条注释标注其对应RGS-BAS-007的具体规则条目，供其余DTL文档编写者对照句法（不得直接照抄表名/字段名用于真实业务）：

```sql
-- 模板：一张"审计/事件类"表的标准写法，示范RGS-BAS-007§2命名规范、§3索引标准、
-- §4分区标准、§5迁移幂等写法、DR-007 OCC列、审计时间戳规范的字面组合

-- 数据库名规则（RGS-BAS-007§2）：<限界上下文缩写英文全称>_db，此处以example_db示意
-- CREATE DATABASE example_db;  -- 数据库创建属限界上下文挂载阶段(RGS-DTL-002)职责，本文档不重复

-- 表名规则（§2）：snake_case复数或领域名词
CREATE TABLE IF NOT EXISTS example_domain_events (
    -- 主键规则（§2 "<entity>_id"）：此表实体为event，故主键为event_id
    event_id        BIGSERIAL PRIMARY KEY,

    -- 列名规则（§2）：snake_case，与既有API/日志字段同名概念保持一致拼写
    actor_player_id BIGINT NOT NULL,           -- 逻辑引用player_db.accounts，跨库不建物理FK
                                                  -- （同RGS-DTL-001/025/026既定跨库引用规则）
    event_type      TEXT NOT NULL CHECK (event_type IN ('created', 'updated', 'archived')),
    payload         JSONB NOT NULL,

    -- 乐观并发列规则（DR-007，RGS-DTL-001§2.1已示范）：统一命名version
    version         INTEGER NOT NULL DEFAULT 0,

    -- 审计时间戳规则（§2，时间戳一律带时区）：created_at/updated_at
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (created_at);
-- 分区规则（§4）：审计/事件类表按月range分区，此处以created_at为分区键示范

-- 外键约束名规则（§2 "fk_<表名>_<引用表名>"，此表无同库外键，规则以注释形式重申）：
-- 若example_domain_events需引用同库另一张表example_domain_categories，
-- 约束名应写作：CONSTRAINT fk_example_domain_events_example_domain_categories FOREIGN KEY (...) REFERENCES ...

-- 索引命名规则（§2 "idx_<表名>_<列名或用途简写>"）与索引设计标准（§3步骤1〜2：
-- 识别高频查询模式→匹配复合索引，字段顺序按选择性由高到低）
CREATE INDEX IF NOT EXISTS idx_example_domain_events_actor_created
    ON example_domain_events (actor_player_id, created_at);
    -- 对应查询场景（§3步骤3要求逐条标注）：示范GetEventsByActor(actor_player_id, time_range)方法

CREATE INDEX IF NOT EXISTS idx_example_domain_events_type
    ON example_domain_events (event_type)
    WHERE event_type = 'archived';
    -- 部分索引示范：仅索引低基数但高频WHERE命中的取值，同RGS-DTL-001 idx_ban_records_player_id_active的既定手法
```

**索引复核（§3步骤4）承诺声明**：本模板表不进入生产，不适用PH-4复核流程；各业务域DTL文档在引用本模板句法时，须各自在实现阶段纳入PH-4复核范围，本文档仅提供句法参照，不代为承诺复核义务。

---

## 3. 标准落地：分区滚动创建工具化

RGS-BAS-007§4给出分区策略表与滚动创建的流程图（"新分区自动创建→当前写入落入最新分区→超出保留期DETACH→归档或DROP"），但未给出该定时任务的具体实现形式。本文档补充：

```sql
-- 月度分区滚动创建函数模板（PL/pgSQL，属§7"极窄允许边界"外的运维自动化脚本，
-- 非业务逻辑存储过程——本函数不查询/修改业务表内容，只操作分区元数据，
-- 不属于§7评审范围（§7评审对象是"表内数据的赋值级触发器"，本函数是运维脚本，
-- 由外部定时任务调用而非数据库内部触发器机制触发，两者性质不同，特此在此明确区分，
-- 避免与§7例外评审混淆）
CREATE OR REPLACE FUNCTION ensure_monthly_partition(
    parent_table   TEXT,
    partition_month DATE  -- 传入该月第一天
) RETURNS VOID AS $$
DECLARE
    partition_name TEXT := parent_table || '_' || to_char(partition_month, 'YYYY_MM');
    start_bound    TIMESTAMPTZ := partition_month;
    end_bound      TIMESTAMPTZ := partition_month + INTERVAL '1 month';
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF %I FOR VALUES FROM (%L) TO (%L)',
        partition_name, parent_table, start_bound, end_bound
    );
END;
$$ LANGUAGE plpgsql;
```

调度侧（外部定时任务，非数据库内部）伪代码：

```
fn partition_maintenance_job(registered_tables: &[PartitionedTableConfig]) {
    for table in registered_tables {
        // 提前创建下月分区，避免月初边界写入落空（"新分区自动创建"对应流程图节点A）
        ensure_monthly_partition(table.name, next_month_first_day());

        // 超期分区DETACH（流程图节点C/D，保留期取各表既定值，如admin_db.operation_audit=3年）
        for expired_partition in table.partitions_older_than(table.retention_period) {
            detach_and_archive_or_drop(expired_partition, table.archive_policy);
            // archive_policy: 归档到冷存储（走既有备份基础设施）或直接DROP，
            // 依§4表"超期DETACH+归档或DROP"，具体二选一由各业务域DTL文档按自身保留合规要求声明
        }
    }
}
```

`registered_tables`的注册来源：各业务域DTL文档在其DDL章节声明分区表时，须在实现阶段将该表加入本调度任务的配置清单（本文档不代为登记各域具体表名，登记动作属各业务域DTL文档自身职责，如RGS-DTL-025§2已声明`detection_signals`复用本机制、RGS-DTL-026§2已声明`queue_entries`清理复用G-005模式脚本——两者是同一调度基础设施的不同调用方）。

---

## 4. 标准落地：迁移工具具体调用

RGS-BAS-007§5.1要求"使用`IF NOT EXISTS`/`IF EXISTS`等幂等语法，或迁移工具自带的已执行记录表"，但未点名具体工具。本文档提出`sqlx-cli`（Rust生态原生、与本项目服务实现语言一致）作为提案：

```bash
# 迁移文件命名（§2既定规则：<序号>_<动词>_<对象>.sql）
sqlx migrate add -r add_column_mail_read_at
# -r 生成up/down成对文件，落实§5.1"每个迁移脚本应当配对一个回滚脚本"

# 生成结果示例：
#   migrations/0007_add_column_mail_read_at.up.sql
#   migrations/0007_add_column_mail_read_at.down.sql

# CI"migrations校验"阶段（复用RGS-BAS-002§4.2既有CI骨架，此处补充具体命令）
sqlx migrate run --dry-run   # 语法/依赖校验，不实际执行
sqlx migrate run             # 向前迁移演练（隔离测试库）
sqlx migrate revert          # 回滚演练，验证down脚本可用
```

**Expand-Contract分离的CI层面强制**（对应RGS-BAS-007§5.2"同一次迁移脚本不得同时包含Expand与Contract动作"）：

```bash
# CI脚本骨架片段：静态扫描单个迁移文件是否同时含新增(ADD COLUMN/CREATE TABLE)
# 与删除(DROP COLUMN/DROP TABLE)语句，命中则CI标红，要求拆分为两个迁移版本
if grep -qiE '\bADD (COLUMN|TABLE)\b' "$migration_file" && \
   grep -qiE '\bDROP (COLUMN|TABLE)\b' "$migration_file"; then
    echo "CI FAIL: migration mixes Expand and Contract actions (RGS-BAS-007 §5.2)"
    exit 1
fi
```

**不可逆迁移声明格式**（§5.1要求复杂的数据迁移若回滚代价过高，须在对应分域RGS-DTL物理DDL章节中显式声明；声明格式统一如下）：

```sql
-- ⚠ IRREVERSIBLE MIGRATION（不可逆迁移声明，RGS-BAS-007§5.1）
-- 原因：<迁移内容摘要，如"历史payload字段JSON结构变更，旧结构在Contract阶段已不再产生，
--        down脚本理论可还原但会丢失Expand期间新写入的字段数据">
-- 评审记录：<ADR编号或评审会议纪要引用>
```

---

## 5. 标准落地：存储过程例外登记表

RGS-BAS-007§7要求全部已批准的例外可审计。RGS-DTL-007§5给出统一登记表schema；各业务域DTL文档若确有触发器例外，应在**自己的DDL章节内**按该schema登记，以避免登记位置分散、失去统一的核对结构（同RGS-BAS-016§3.1"跨限界上下文表结构扩展须同步更新原表文档"确立的同类精神）。不新建独立汇总数据库表，登记本身是文档层面的表格，非运行时数据：

| 列 | 说明 |
|---|---|
| `trigger_name` | 触发器名，命名同§2索引/约束命名精神：`trg_<表名>_<用途简写>` |
| `table_name` | 所在表 |
| `logic_summary` | 触发逻辑摘要（须满足§7"允许"判定标准：单表内、赋值级操作） |
| `reviewer` | 评审架构师 |
| `adr_ref` | 对应ADR编号 |

本项目截至本文档制定时（2026-08-17），尚未有任何业务域DTL文档登记过触发器例外——RGS-DTL-001/025/026/027均未使用存储过程/触发器，§7例外机制迄今零命中，这是ARC-023"业务逻辑不入库"原则被严格遵守的正面信号，非文档疏漏。

---

## 6. 本文档的覆盖范围与后续计划

本文档覆盖：满足RGS-BAS-007§2〜§4全部命名/索引/分区规则的模板`CREATE TABLE`字面示范、分区滚动创建的PL/pgSQL函数与调度伪代码、迁移工具（`sqlx-cli`提案）具体调用命令与CI Expand-Contract静态检查脚本骨架、存储过程例外登记表的统一schema。

本版本明确不覆盖、留待后续：

- 任何真实业务限界上下文的实际DDL——已分别由RGS-DTL-001（player_db/economy_db）、RGS-DTL-025（admin_db反作弊三表）、RGS-DTL-026（match_db匹配三表）、RGS-DTL-015/016（EC限界上下文交易/工单/对账表，见同批次另两份文档）覆盖，本文档不重复。
- `sqlx-cli`之外候选迁移工具（如`diesel_cli`/`refinery`）的对比评估——本文档只给出一个可行提案，未做工具选型评审本身，若实现阶段团队已有既定工具链选择，以实际选择为准。
- 备份与恢复设计（RGS-BAS-007§6）的具体演练脚本与RTO实测记录——该演练记录属RGS-OPS-001归档范围（RGS-BAS-007§6已注明），不属于本文档（设计文档）职责。
- 分区调度任务（§3）本身的部署形态（K8s CronJob具体manifest等）——留待实现阶段按既有运维基础设施（RGS-DTL-002已确立的Helm/CI模式）补充，本文档只给出调度逻辑伪代码。

后续详细设计建议顺序：本文档与RGS-DTL-015（玩家间交易系统）、RGS-DTL-016（客服工单与支付对账）同批次产出，三者均属03域（数据经济与交易），建议后续DTL文档编写者优先参照本文档§2模板句法而非各自独立摸索命名/索引写法，减少跨域DDL风格漂移。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-007§2 命名规范 | §2 |
| RGS-BAS-007§3 索引设计标准 | §2 |
| RGS-BAS-007§4 分区设计标准 | §2、§3 |
| RGS-BAS-007§5 迁移流程设计 | §4 |
| RGS-BAS-007§6 备份与恢复设计 | §6（明确排除具体演练脚本） |
| RGS-BAS-007§7 存储过程例外评审流程 | §5 |
| RGS-BAS-007§8 连接池标准 | §6（明确排除，无新增设计点，RGS-BAS-007原文已声明"不重新设定具体数值"） |
| RGS-DTL-001/025/026/027（既有DTL文档对RGS-BAS-007规则的实际引用） | 前提依赖，本文档的模板句法与既有文档已采用的写法保持一致，非另立新规则 |
