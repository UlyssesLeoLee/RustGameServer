# 详细设计书（詳細設計書 / Detailed Design Document）

**仿生分层架构与智能决策层：全局开关物理存储与只读强制机制・分析图注册数据模型详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-011 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-011 仿生分层架构与智能决策层 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档是继RGS-DTL-001/002/025/026/027之后的一批详细设计文档之一，与RGS-DTL-012/013/014/019并行产出）。细化RGS-BAS-011§4.1/§4.1.1全局开关设计为具体配置存储行格式与双层（IAM＋NetworkPolicy）只读强制的具体机制、§5A分析图生命周期管理的逻辑数据模型为`AnalysisGraphDefinition`／`AnalysisGraphAuditLog`具体DDL（RGS-DTL-025§1.2此前已引用本框架为"anticheat-fusion"图的注册载体但本框架自身此前无物理schema，本文档补齐）、§7A.2确定性闸门的组件设计为`AdminService`入口侧具体校验代码路径。**本版本不覆盖**：LangGraph分析图内部的节点图结构本身、Prompt设计、各`feature_domain`具体业务分析逻辑——均属各业务域自身范围，非本框架（基础设施）职责。见§7 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-011§5A.1/§5A.1.1逻辑模型完全一致，闸门伪代码是否遗漏部署位置约束的强制点 |
| 评审（安全） | | | 开关只读强制的IAM/NetworkPolicy双层机制描述是否可直接落地为具体策略文件，闸门1枚举白名单是否确为编译期常量而非可配置项 |
| 审批（负责人） | | | 本文档的基准化；`AnalysisGraphDefinition`物理schema是否可直接供RGS-DTL-025"anticheat-fusion"图注册使用 |

---

## 目录

1. [前言](#1-前言)
2. [全局开关：物理存储与只读强制机制](#2-全局开关物理存储与只读强制机制)
3. [分析图注册数据模型：物理数据库设计](#3-分析图注册数据模型物理数据库设计)
4. [确定性闸门：部署位置与校验伪代码](#4-确定性闸门部署位置与校验伪代码)
5. [本文档的覆盖范围与后续计划](#5-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-011给出了智能层的组件图、OLU核算、事件订阅边界、LangGraph设计范式、分析图生命周期管理的逻辑数据模型（§5A.1文字字段表）、确定性闸门的组件设计（§7A，含部署位置的强约束但未给出具体校验代码路径）。本文档将其中**基础设施性质**（跨全部`feature_domain`共享、不因具体业务分析逻辑而变化）的三个部分落实为物理/实现级设计：全局开关（`neuro_layer_enabled`）的具体存储行格式与"智能层只读不可写"的双层强制机制、`AnalysisGraphDefinition`/`AnalysisGraphAuditLog`的具体DDL、确定性闸门三重校验在`AdminService`入口侧的具体伪代码路径。

### 1.2 本文档不做什么

- **不重新决定**RGS-BAS-011已确定的任何结构性选择（智能层旁路观察者定位、闸门必须部署于`AdminService`侧而非智能层内、开关默认关闭且写权限唯一收口于`AdminService`、双态OLU核算口径）。
- **不设计LangGraph图内部实现**——节点图结构、边的条件转移、Prompt模板、各`feature_domain`（异常行为识别／经济健康度／匹配质量评估／GM决策辅助，以及RGS-DTL-025已引用的`anticheat-fusion`）具体分析逻辑，均属各业务域自身范围。本文档只给出这些图**如何被注册、版本化、审计**这一治理层，不涉及图跑什么。RGS-DTL-025§1.2已明确将"`anticheat-fusion`分析图内部LangGraph节点图结构"排除在其自身范围之外并指向本框架——本文档正是那个被指向的注册载体，二者此前的引用关系至此闭环。
- **不覆盖**GM后台"分析图目录"查询页的前端UI细节（属参考GM后台/前端自身设计范围，同RGS-DTL-012§7参考GM后台最小实现范围思想）。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准；伪代码可直接对应Rust `Result`风格实现；本文档新增的NetworkPolicy/IAM策略片段以YAML风格给出，沿用RGS-DTL-002§1.3的占位符约定（`<CONTEXT>`等尖括号占位符为挂载时替换项，本文档中固定为`neuro`）。

---

## 2. 全局开关：物理存储与只读强制机制

对应RGS-BAS-011§4.1"全局开关"与§4.1.1"写权限收口"。开关值落位于ARC-016既定的数值表热更新配置存储（本文档不新建存储介质，只固定本开关自身的具体键/行格式）。

### 2.1 配置存储行格式

```sql
-- 复用ARC-016既有配置存储表结构（本文档不重复定义该表本身，仅新增本开关对应的一行数据）
-- 假定既有配置存储表结构近似为 config_entries(config_key TEXT PRIMARY KEY, config_value JSONB, version BIGINT, updated_by TEXT, updated_at TIMESTAMPTZ)
-- 本开关的具体键值：
INSERT INTO config_entries (config_key, config_value, version, updated_by, updated_at)
VALUES ('neuro_layer_enabled', 'false'::jsonb, 0, 'system_bootstrap', now())
ON CONFLICT (config_key) DO NOTHING;
-- ON CONFLICT DO NOTHING：幂等初始化，防止部署脚本重复执行时覆盖已被AdminService合法修改过的值（FR-NEURO-050"全新部署环境初始值为false"仅约束首次创建）
```

`config_value`固定为JSON布尔值`true`/`false`，不使用字符串枚举（该配置项语义单一，不存在第三态，无需保留扩展空间）。`version`列复用既有配置存储的乐观锁/审计列语义，每次`AdminService`写入递增，供§2.3核对任务判定"是否发生过变更"。

### 2.2 只读强制机制：IAM层

对应RGS-BAS-011§4.1"配置存储隔离"表述中"IAM与NetworkPolicy两层均不具备写权限"的IAM侧具体落地。智能层服务账号（`neuro-service`）绑定的角色**不得**包含对`config_entries`表（或其等价的配置存储写API）的`INSERT`/`UPDATE`权限：

```sql
-- neuro服务账号对应的数据库角色权限声明（复用RGS-DTL-025§5A.1.1"只增不改"同类数据库层强制原则）
-- 智能层对配置存储的角色仅授予只读，不授予任何写权限——这是"物理上够不到"而非"约定上不应该"
GRANT SELECT ON config_entries TO neuro_service_role;
-- 显式声明：不执行以下语句，作为本文档的强制性记录（供CI/审计核对"neuro_service_role的权限集合中不包含以下任何一条"）
-- REVOKE-verified-absent: GRANT INSERT ON config_entries TO neuro_service_role;
-- REVOKE-verified-absent: GRANT UPDATE ON config_entries TO neuro_service_role;
-- REVOKE-verified-absent: GRANT DELETE ON config_entries TO neuro_service_role;
```

若配置存储的写入不经数据库直连而是经既有配置管理API（更符合ARC-016"数值表与可执行文件分离"实现常态），则等价约束落在API网关/服务网格的RBAC层：`neuro-service`身份**不得**被授予该API的`PUT`/`POST`/`DELETE`方法权限，仅授予`GET`。两种实现路径（DB直连角色权限 或 API RBAC）二选一，取决于详细设计阶段配置存储的实际访问方式选型，本文档不代为决定该实现细节，仅固定"无论哪种路径，智能层角色的权限集合中都不得出现写方法"这一不变式。

### 2.3 只读强制机制：NetworkPolicy层

对应RGS-BAS-011§4.1"NetworkPolicy出站白名单不含配置存储的写端点"。复用RGS-DTL-002§3已确立的NetworkPolicy两段式default-deny + allow-list模板，本文档固定智能层（`neuro`上下文）的`allowedEgressTo`具体取值：

```yaml
# charts/neuro-service/values.yaml 片段（对应RGS-DTL-002§3模板的.Values.allowedEgressTo取值）
allowedEgressTo:
  - event-bus-consumer-endpoint   # §4.1订阅端点，仅只读订阅
  - admin-service                  # §6.2唯一出口：提交Recommendation
  - otel-collector                 # 可观测性
  - config-store-read-endpoint     # 配置存储只读端点（若配置存储对读/写分别暴露不同端点，仅放行读端点）
# 显式声明不出现于本列表（供RGS-DTL-002§6.3一致性校验脚本核对时的负向断言基准）：
# - config-store-write-endpoint（若配置存储读写共用单一端点而非分离，则该端点本身不得出现在本列表中，
#   即智能层对配置存储的NetworkPolicy出站权限为完全不可达，只能通过§2.2的DB/API只读角色间接读取经由其他中间层转发的数据；
#   具体采用"独立读端点放行"或"完全不可达+经中间层转发"两种拓扑之一，取决于详细设计阶段配置存储的部署形态，本文档不代为决定）
```

`RGS-DTL-002§6.3一致性校验脚本`（`scripts/check-mount-record-consistency.sh`）对`neuro`上下文的校验，除既有的"`values.yaml`与Mount Record依赖登记一致"外，本文档新增一条**否定断言**扩展点：该脚本对`neuro`上下文额外校验`allowedEgressTo`中不含任何标记为"配置存储写端点"的条目——这是FR-NEURO-042"配置存储隔离"在CI阶段的具体落地，复用既有脚本框架，不新建独立校验工具。

### 2.4 消费循环最外层读取（对应RGS-BAS-011§4.1"最外层判定"）

```rust
// 智能层每个消费者实例的事件消费主循环，最外层开关判定
fn on_event_received(event: RawEvent, ctx: &ConsumerContext) -> Result<(), ConsumeError> {
    // §2.1键值，只读订阅既有热更新推送/短轮询通道，不主动发起写请求
    let enabled: bool = ctx.config_store.read_bool("neuro_layer_enabled").unwrap_or(false);
    //   ↑ 读取失败（配置存储瞬时不可用）时默认按false处理而非panic——"读取失败"与"显式关闭"在
    //     "不产生分析副作用"这一后果上等价，符合NFR-NEURO-010"默认安全性"精神：不确定时选择更保守的分支

    commit_offset(&event)?;  // 无论是否处理，均先提交offset（RGS-BAS-011§4.1既定"接收但不处理"）

    if !enabled {
        return Ok(());  // 关闭态：不进入下方任何分析管线，等同于本条事件从未被观察
    }

    dispatch_to_analysis_queue(event, ctx)  // 开启态：进入§2.1图组件的QUEUE→GRAPH→REC管线（RGS-BAS-011§2.1组件图）
}
```

**边界条件**：`commit_offset`必须先于开关判定执行（而非放在函数末尾统一提交），否则关闭态下大量事件因未提交offset而导致消费者组重平衡时反复重新投递，产生RGS-BAS-011§4.1"避免消费者组积压/重平衡异常"要求所指的确切问题。

---

## 3. 分析图注册数据模型：物理数据库设计

对应RGS-BAS-011§5A.1/§5A.1.1。两表依附既有`admin_db`（AD限界上下文数据库，同RGS-DTL-025§2反作弊三表的挂靠方式），本文档只新增表结构。

### 3.1 DDL

```sql
-- 分析图定义表，对应FR-NEURO-043〜047
CREATE TABLE analysis_graph_definitions (
    graph_id          UUID NOT NULL,
    version            INTEGER NOT NULL,   -- 单调递增，FR-NEURO-046参数级更新时递增
    feature_domain      TEXT NOT NULL,      -- 'NEURO'/'GSM'/'SUP'等，§5A.3登记初始目录；新domain追加不改表结构
    status                TEXT NOT NULL DEFAULT '草稿'
                              CHECK (status IN ('草稿', '生效', '已废弃')),
    graph_spec_ref          TEXT NOT NULL,   -- 引用ARC-016配置存储中的实际图定义内容(节点/边/参数)，本表不内嵌具体内容
    subscribed_event_scope    TEXT[] NOT NULL DEFAULT '{}',  -- Topic/partition_key子集，数组存储，供§5A.4状态-订阅一致性核对逐项比对
    olu_review_ref             TEXT,          -- 引用ARC-014/ARC-026评审记录；status='生效'时不得为空(见下方CHECK)
    supersedes_graph_id          UUID,          -- 可选，版本链：结构变更视为新场景时指向被取代的旧graph_id
    supersedes_version             INTEGER,       -- 与supersedes_graph_id成对，指向具体旧版本
    created_at                       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (graph_id, version),
    CONSTRAINT chk_agd_effective_requires_review
        CHECK (status <> '生效' OR olu_review_ref IS NOT NULL)
        -- 数据库层强制FR-NEURO-044"生效状态olu_review_ref必须非空"，不仅依赖应用层校验
);

-- 部分唯一索引：同一graph_id任意时刻至多一个'生效'版本，防止双主导致并发建议产出口径不一致
CREATE UNIQUE INDEX uq_agd_graph_id_effective
    ON analysis_graph_definitions (graph_id) WHERE status = '生效';

-- 支撑FR-NEURO-045目录查询（按域/状态过滤）
CREATE INDEX idx_agd_domain_status
    ON analysis_graph_definitions (feature_domain, status);

-- 分析图审计日志表，复用RGS-BAS-003§7审计设计存储结构，落地FR-NEURO-048
CREATE TABLE analysis_graph_audit_logs (
    log_id            BIGSERIAL PRIMARY KEY,
    graph_id            UUID NOT NULL,
    version_before        INTEGER,   -- NULL表示新增注册(无"之前版本")
    version_after           INTEGER NOT NULL,
    action                    TEXT NOT NULL
                                 CHECK (action IN ('注册', '评审通过转生效', '参数更新', '结构变更', '废弃')),
    operator                    TEXT NOT NULL,
    spec_checksum                  TEXT NOT NULL,   -- graph_spec_ref内容的SHA-256哈希，供§5A.4可核对性比对，不得为空
    occurred_at                      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_agal_version_after
        FOREIGN KEY (graph_id, version_after) REFERENCES analysis_graph_definitions (graph_id, version)
    -- version_before为空(新增注册)时不建外键约束(版本尚未存在于before时点)，version_after始终必须已存在
);

CREATE INDEX idx_agal_graph_occurred
    ON analysis_graph_audit_logs (graph_id, occurred_at);

-- 只增不改：审计表数据库角色权限仅授予INSERT，不授予UPDATE/DELETE
-- (复用RGS-DTL-025§2同类"只增不改"数据库层强制模式，与RGS-BAS-003§7既定审计表权限收紧模式一致)
REVOKE UPDATE, DELETE ON analysis_graph_audit_logs FROM PUBLIC;
GRANT INSERT, SELECT ON analysis_graph_audit_logs TO admin_service_role;
```

`version_before`允许为空但`version_after`不允许，是"新增注册"这一动作在数据模型层面的直接表达——不需要额外的布尔标志位区分"这是否是首次注册"，`version_before IS NULL`本身即是该判定条件，减少一个可能与`action`字段不一致的冗余状态位。

### 3.2 与逻辑设计的对应关系

| RGS-BAS-011§5A.1逻辑字段 | 物理实现 | 差异说明 |
|---|---|---|
| `AnalysisGraphDefinition.graph_id`（唯一标识） | `(graph_id, version)`复合主键 | BAS-011§5A.1文字表述"主键`(graph_id, version)`"已在§5A.1.1明确给出，本文档直译，非新决策 |
| `superseded_by`/`supersedes`（版本链） | `supersedes_graph_id`/`supersedes_version` | 逻辑设计仅描述"引用同graph_id的其他version"这一版本链语义；物理实现进一步区分"结构变更→新graph_id"（本表两列指向的是**不同**graph_id的场景）与"参数更新→同graph_id新version"（此时`supersedes_graph_id`等于自身`graph_id`，`supersedes_version`为旧version号）——两种版本链在同一对列上表达，不新增字段，仅在写入时按§5A.2既定的"结构变更须关联新graph_id"规则决定该列取值 |
| `olu_review_ref`必须非空（`生效`状态） | `chk_agd_effective_requires_review` CHECK约束 | 逻辑设计已用"**必须**非空"文字强调，本文档将其从应用层校验提升为数据库层CHECK约束，属物理落实而非新决策 |
| 其余字段 | 类型直译 | `uuid`→`UUID`，`string`→`TEXT`，`int`→`INTEGER`/`BIGSERIAL`，枚举→`TEXT CHECK IN (...)`（沿用RGS-DTL-025同类枚举编码风格：本表状态值语义重于查询频率，采用可读TEXT而非SMALLINT，与RGS-DTL-001§2.2"高频WHERE列用SMALLINT"原则不冲突——本表非高频热路径，可读性优先） |

---

## 4. 确定性闸门：部署位置与校验伪代码

对应RGS-BAS-011§7A.2三重闸门设计。本节落实"闸门必须部署于`AdminService`入口侧"这一强约束的具体代码路径归属，以及三重闸门的校验伪代码。

### 4.1 部署位置的物理落实

闸门代码**物理位于**`services/admin-service/src/neuro_gate/`模块内（`admin-service`既有服务的一个内部模块，而非独立部署单元），随`admin-service`的既有部署形态（Deployment，见RGS-DTL-002§2.1）一同构建/发布，与智能层（`neuro-service`）**不共享任何构建产物或运行时进程**——这是RGS-BAS-011§7A.2"闸门若在智能层内，其自身就成为L4组件的一部分"这一风险的物理层面消除方式：即便`neuro-service`的容器/进程被完全攻陷，攻击者也无法在该进程内修改闸门逻辑，因为闸门代码根本不存在于该进程的地址空间内。

```
services/admin-service/src/
  neuro_gate/
    mod.rs              # 三重闸门入口，RecommendationReceived RPC处理器调用点
    action_whitelist.rs # 闸门1：编译期常量枚举
    value_domain.rs      # 闸门2：值域校验
    risk_tier.rs           # 闸门3：risk_tier自动继承逻辑
```

### 4.2 三重闸门校验伪代码

```rust
// admin-service侧，Recommendation提交RPC的处理入口（RGS-BAS-011§6.2呈现时序图的AD自校验环节）
fn handle_recommendation(rec: RecommendationSubmission) -> Result<GateOutcome, GateError> {
    // 闸门1: 枚举白名单，全等匹配，来源为编译期常量集合(非运行时可配置列表)
    if !ADMIN_SERVICE_METHOD_WHITELIST.contains(&rec.suggested_action.as_str()) {
        // ADMIN_SERVICE_METHOD_WHITELIST: &'static [&'static str] = &["BanAccount", "MuteChat", "CreateOpsTicket", ...]
        // 全等匹配：不使用.starts_with()/.contains()等前缀/模糊匹配，理由见RGS-BAS-011§7A.2闸门1设计要点
        record_gate_rejection(&rec, GateStage::ActionWhitelist);  // ERROR级别，强制全采集(§7A.2既定)
        return Err(GateError::UnknownAction);
    }

    // 闸门2: 值域校验，越界必须拒绝，不得截断(clamp)
    let domain_spec = value_domain_for(&rec.suggested_action);  // 与既有API字段设计(RGS-BAS-003§3)同源取值
    for (param_name, param_value) in rec.parameters.iter() {
        if !domain_spec.validate(param_name, param_value) {
            record_gate_rejection(&rec, GateStage::ValueDomain);
            return Err(GateError::ParameterOutOfDomain { param: param_name.clone() });
            //  ↑ 直接返回错误，不做param_value.clamp(min, max)之类的静默纠正——
            //    截断会把"明显错误"变成"看似合理"，消除错误可见性(RGS-BAS-011§7A.2闸门2设计要点引用ARC-030)
        }
    }

    // 闸门3: risk_tier自动继承，不得由提交方(智能层)自行申报
    let risk_tier = risk_tier_for_action(&rec.suggested_action);  // 查询RGS-BAS-003§8既定高危操作分类表，忽略rec中若携带的任何risk_tier字段
    // 低风险只读例外(RGS-BAS-011§6.2"补齐设计缺口"条目落地): 该判定同样查表决定，不接受rec自行声明"我是低风险"
    if is_low_risk_read_only(&rec.suggested_action) {
        present_as_notification(&rec);  // 直接通知呈现，跳过审批门槛，但仍不触发任何写操作(§6.2既定)
        return Ok(GateOutcome::NotifiedOnly);
    }

    enqueue_for_manual_approval(&rec, risk_tier);  // 默认路径：走既有二次确认(RGS-BAS-003§8)，与GM自主发起操作同一路径
    Ok(GateOutcome::PendingApproval { risk_tier })
}
```

**关键边界条件**：闸门2的校验循环对`rec.parameters`中**任意一个**字段越界即整体拒绝（不做"部分字段生效、越界字段忽略"的部分接受）——理由与RGS-DTL-026§5"跨分片OCC全有或全无"同类：部分接受同样会把一个本应被拒绝的建议变成看似合法的、但内容被悄悄修改过的建议，与截断问题同构，故一并拒绝而非部分放行。

---

## 5. 本文档的覆盖范围与后续计划

本文档覆盖：全局开关`neuro_layer_enabled`的配置存储行格式、IAM层与NetworkPolicy层的双重只读强制机制的具体策略/权限声明、消费循环最外层判定的伪代码、`analysis_graph_definitions`/`analysis_graph_audit_logs`两表的物理DDL（含部分唯一索引/CHECK约束/只增不改权限收紧的数据库层强制）、确定性闸门三重校验在`AdminService`入口侧的具体部署位置与校验伪代码。

本版本明确不覆盖、留待后续：

- LangGraph分析图内部的节点结构、边的条件转移、Prompt模板——按RGS-BAS-011§5A.2既定评审流程，各`feature_domain`场景（异常行为识别/经济健康度/匹配质量评估/GM决策辅助，以及RGS-DTL-025已提前引用的`anticheat-fusion`反作弊融合场景）须各自完成评审并各自登记为`analysis_graph_definitions`中的一行，图内部实现属各场景自身职责，非本框架范围——本文档只交付它们注册时所需落地的物理表结构。
- `AnalysisGraphAuditLog.spec_checksum`哈希计算与§5A.4"配置内容是否被篡改"定期核对任务的具体调度实现——本文档只固定该列在DDL中的存在与NOT NULL约束，核对任务本身复用既有定时作业基础设施（RGS-BAS-011§5A.4已声明不新建独立调度组件），具体核对脚本留待实现阶段。
- §5A.4"状态与实际订阅是否一致"核对任务中"智能层实际存活的消费者组集合"这一侧的具体获取方式（依赖K8s/消息中间件的运行时内省接口，非本框架数据模型职责）。
- LLM推理后端（自托管）的具体引擎与模型选型——RGS-BAS-011§2.2已明确"详细设计阶段选定推理引擎与模型"，但该选型与本文档聚焦的治理/开关基础设施无直接耦合，故本文档不展开，留待独立的推理后端详细设计（若后续需要单独立项）。
- 各`feature_domain`分析图注册时`subscribed_event_scope`的具体Topic/`partition_key`取值——本文档只固定该字段在DDL中的存储形态（`TEXT[]`），具体取值由各图注册时按§5A.3登记表自行声明。

后续详细设计建议顺序：优先推进已在RGS-DTL-025中提前引用本框架的`anticheat-fusion`场景补齐其自身的图内部评审文档（对应RGS-DTL-025§6"评审通过后另立分析图定义文档"），使反作弊域的智能层接入闭环；GSM（RGS-DTL-014，本批次已并行产出）若涉及§5A.3"未来接入"提及的举报信号场景，届时同样复用本文档§3的表结构注册，不需另立注册基础设施。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-011§4.1 事件订阅与全局开关运行时读取 | §2.4 |
| RGS-BAS-011§4.1.1 全局开关写权限收口 | §2.1、§2.2、§2.3 |
| RGS-BAS-011§5A.1 AnalysisGraphDefinition/AnalysisGraphAuditLog数据模型 | §3.1、§3.2 |
| RGS-BAS-011§5A.1.1 物理落位与约束 | §3.1 |
| RGS-BAS-011§5A.2 CRUD时序 | §3.1（版本链字段设计对应"改/删"分支）、§5（明确排除图内部实现） |
| RGS-BAS-011§5A.3 初始功能场景目录 | §5（登记各场景需自行接入本框架，不预先设计其内容） |
| RGS-BAS-011§5A.4 高可用与可核对性 | §5（明确排除核对任务具体实现，仅固定支撑其核对所需的列） |
| RGS-BAS-011§7A.2 三重闸门组件设计与部署位置约束 | §4.1、§4.2 |
| RGS-BAS-011§7A.3 泄漏路径防护（配置存储写入路径） | §2 |
| RGS-BAS-011§6.2 建议呈现时序与低风险例外判定 | §4.2 |
| RGS-DTL-002（挂载脚手架物理落地） | 前提依赖，本文档假定NEURO域已按RGS-DTL-002完成挂载 |
| RGS-DTL-025§1.2（对本框架图注册模型的既有引用） | §1.1（引用闭环说明），§3 |
