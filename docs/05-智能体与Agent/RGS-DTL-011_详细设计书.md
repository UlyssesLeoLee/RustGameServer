# 详细设计书（詳細設計書 / Detailed Design Document）

**仿生分层架构与智能决策层：全局开关物理存储与只读强制机制・分析图注册数据模型详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-011 |
| 版本 | 1.0 |
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
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档是继RGS-DTL-001/002/025/026/027之后的一批详细设计文档之一，与RGS-DTL-012/013/014/019并行产出）。细化RGS-BAS-011§4.1/§4.1.1全局开关设计为具体配置存储行格式与双层（IAM＋NetworkPolicy）只读强制的具体机制、§5A分析图生命周期管理的逻辑数据模型为`AnalysisGraphDefinition`／`AnalysisGraphAuditLog`具体DDL（RGS-DTL-025§1.2此前已引用本框架为"anticheat-fusion"图的注册载体但本框架自身此前无物理schema，本文档补齐）、§7A.2确定性闸门的组件设计为`AdminService`入口侧具体校验代码路径。**本版本不覆盖**：LangGraph分析图内部的节点图结构本身、Prompt设计、各`feature_domain`具体业务分析逻辑——均属各业务域自身范围，非本框架（基础设施）职责。见§5 | 全部 |
| 1.0 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008） | — | 同步父BAS-011升版至v1.0（10次升版v0.1→v0.2→v0.3→v0.4→v0.5→v0.6→v0.7→v0.8→v0.9→v1.0）+ 补§6技术栈边界与LLM自托管实施细节（落实v0.3 §2.2新增约束）、§7配置存储隔离的IAM/NetworkPolicy双锁细化（落实v0.4 §4.1）、§8分析图高可用与可核对性详细设计（落实v0.5/v0.6 §5A.1.1/§5A.4）、§9双态OLU核算详细化（落实v0.8/v0.9 §3.1/§3.2）、§10隔离与降级扩展（落实v0.2 §7故障注入验证）、§11推理输入快照与离线重放（落实v0.2 §7A.4）、§12检查清单详细化（落实v0.7 §9.1/§9.2/§9.3）。既有§2/§3/§4经核对已落实v0.1~v0.2/v0.5/v0.6/v0.7/v1.0，仅补未覆盖的v0.3~v0.9中§2.2技术栈边界/§5A.4可核对性/§3.1双态OLU/§9检查清单拆分等4类未在v0.1落实的BAS升版内容。**不引入新设计**：仅落实BAS已确定的设计；**不可代签**：本行"审批者"列填"—"由修订者自行核对确认 | §6〜§12新增、§2/§3/§4/§5既有章节元数据同步 |

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
6. [技术栈边界与LLM推理后端自托管实施细节](#6-技术栈边界与llm推理后端自托管实施细节)
7. [配置存储隔离的IAM/NetworkPolicy双锁细化](#7-配置存储隔离的iamnetworkpolicy双锁细化)
8. [分析图高可用与可核对性详细设计](#8-分析图高可用与可核对性详细设计)
9. [双态OLU核算详细化](#9-双态olu核算详细化)
10. [隔离与降级扩展：故障注入验证与背压落地](#10-隔离与降级扩展故障注入验证与背压落地)
11. [推理输入快照与离线重放复核](#11-推理输入快照与离线重放复核)
12. [标准化检查清单详细化](#12-标准化检查清单详细化)

---

## 1. 前言

### 1.1 定位

RGS-BAS-011给出了智能层的组件图、OLU核算、事件订阅边界、LangGraph设计范式、分析图生命周期管理的逻辑数据模型（§5A.1文字字段表）、确定性闸门的组件设计（§7A，含部署位置的强约束但未给出具体校验代码路径）。本文档将其中**基础设施性质**（跨全部`feature_domain`共享、不因具体业务分析逻辑而变化）的三个部分落实为物理/实现级设计：全局开关（`neuro_layer_enabled`）的具体存储行格式与"智能层只读不可写"的双层强制机制、`AnalysisGraphDefinition`/`AnalysisGraphAuditLog`的具体DDL、确定性闸门三重校验在`AdminService`入口侧的具体伪代码路径。

### 1.2 本文档不做什么

- **不重新决定**RGS-BAS-011已确定的任何结构性选择（智能层旁路观察者定位、闸门必须部署于`AdminService`侧而非智能层内、开关默认关闭且写权限唯一收口于`AdminService`、双态OLU核算口径）。
- **不设计LangGraph图内部实现**——节点图结构、边的条件转移、Prompt模板、各`feature_domain`（异常行为识别／经济健康度／匹配质量评估／GM决策辅助，以及RGS-DTL-025已引用的`anticheat-fusion`）具体分析逻辑，均属各业务域自身范围。本文档只给出这些图**如何被注册、版本化、审计**这一治理层，不涉及图跑什么。RGS-DTL-025§1.2已明确将"`anticheat-fusion`分析图内部LangGraph节点图结构"排除在其自身范围之外并指向本框架——本文档正是那个被指向的注册载体，二者此前的引用关系至此闭环。
- **不覆盖**GM后台"分析图目录"查询页的前端UI细节（属参考GM后台/前端自身设计范围，同RGS-DTL-012§6参考GM后台最小实现范围思想）。

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
  - admin-service                  # RGS-BAS-011§6.2唯一出口：提交Recommendation
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
        present_as_notification(&rec);  // 直接通知呈现，跳过审批门槛，但仍不触发任何写操作(RGS-BAS-011§6.2既定)
        return Ok(GateOutcome::NotifiedOnly);
    }

    enqueue_for_manual_approval(&rec, risk_tier);  // 默认路径：走既有二次确认(RGS-BAS-003§8)，与GM自主发起操作同一路径
    Ok(GateOutcome::PendingApproval { risk_tier })
}
```

**关键边界条件**：闸门2的校验循环对`rec.parameters`中**任意一个**字段越界即整体拒绝（不做"部分字段生效、越界字段忽略"的部分接受）——理由与RGS-DTL-026§5"跨分片OCC全有或全无"同类：部分接受同样会把一个本应被拒绝的建议变成看似合法的、但内容被悄悄修改过的建议，与截断问题同构，故一并拒绝而非部分放行。

---

## 5. 本文档的覆盖范围与后续计划

本文档v1.0覆盖（综合v0.1+v1.0两版升版内容）：

- **v0.1已落实**：全局开关`neuro_layer_enabled`的配置存储行格式、IAM层与NetworkPolicy层的双重只读强制机制的具体策略/权限声明、消费循环最外层判定的伪代码、`analysis_graph_definitions`/`analysis_graph_audit_logs`两表的物理DDL（含部分唯一索引/CHECK约束/只增不改权限收紧的数据库层强制）、确定性闸门三重校验在`AdminService`入口侧的具体部署位置与校验伪代码。
- **v1.0新增（同步BAS-011 v0.3~v0.9升版）**：技术栈边界CI静态扫描契约（§6.1）、LLM自托管NetworkPolicy端点命名约定与商业端点负向断言（§6.2）、LLM模型权重许可登记流程（§6.3）、配置存储隔离双锁间的强制关系与单锁失守处置（§7）、分析图高可用与可核对性三项核对任务的实施细节（§8）、双态OLU核算的可观察指标与季度复核时点（§9）、隔离与降级的故障注入试验契约与熔断暂存机制（§10）、推理输入快照的存储格式与离线重放工具形态（§11）、三类检查清单的可执行形态（§12）。

本版本明确不覆盖、留待后续：

- LangGraph分析图内部的节点结构、边的条件转移、Prompt模板——按RGS-BAS-011§5A.2既定评审流程，各`feature_domain`场景（异常行为识别/经济健康度/匹配质量评估/GM决策辅助，以及RGS-DTL-025已提前引用的`anticheat-fusion`反作弊融合场景）须各自完成评审并各自登记为`analysis_graph_definitions`中的一行，图内部实现属各场景自身职责，非本框架范围——本文档只交付它们注册时所需落地的物理表结构。
- `AnalysisGraphAuditLog.spec_checksum`哈希计算与§8.2.2"配置内容是否被篡改"定期核对任务的具体核对脚本实现——本文档§8.2.2已固定该核对任务的实施细节（数据源、比对逻辑、告警分级），但具体核对脚本（读ARC-016配置存储+计算SHA-256+比对`spec_checksum`列）留待实现阶段，调度复用既有定时作业基础设施（RGS-BAS-011§5A.4已声明不新建独立调度组件）。
- §8.2.1"状态与实际订阅一致性核对"中"智能层实际存活的消费者组集合"这一侧的具体获取方式（依赖K8s/消息中间件的运行时内省接口，非本框架数据模型职责）。
- LLM推理后端（自托管）的具体引擎与模型选型——RGS-BAS-011§2.2已明确"详细设计阶段选定推理引擎与模型"，本文档§6.3已固定许可核实登记流程（4步骤），但选型本身（候选模型清单+评估）留待独立的推理后端详细设计（若后续需要单独立项）。
- 各`feature_domain`分析图注册时`subscribed_event_scope`的具体Topic/`partition_key`取值——本文档只固定该字段在DDL中的存储形态（`TEXT[]`），具体取值由各图注册时按§5A.3登记表自行声明。
- §9双态OLU核算的具体指标采集点配置（指标定义已在§9.1列出，**不**为核算目的新增采集点；优先复用既有可观测性数据与`analysis_graph_audit_logs`/`Recommendation`审计字段的聚合统计）。

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
| RGS-BAS-011§2.2 技术栈边界（禁用langgraph-api/FR-NEURO-039） | §6.1 |
| RGS-BAS-011§2.2 LLM推理后端自托管（FR-NEURO-040） | §6.2 |
| RGS-BAS-011§2.2 LLM模型权重许可（FR-NEURO-041） | §6.3 |
| RGS-BAS-011§4.1 配置存储隔离双层强制（FR-NEURO-042，v0.4新增） | §2.2、§2.3、§7（细化） |
| RGS-BAS-011§5A.1.1 物理落位与约束（v0.6新增） | §3.1（已落实） |
| RGS-BAS-011§5A.4 高可用与可核对性（v0.6新增） | §8（详细化） |
| RGS-BAS-011§3.1/§3.2 双态OLU核算（v0.8/v0.9新增） | §9 |
| RGS-BAS-011§7 隔离与降级（v0.2初版） | §10 |
| RGS-BAS-011§7A.4 推理输入快照与离线重放（v0.2初版） | §11 |
| RGS-BAS-011§9.1/§9.2/§9.3 检查清单拆分（v0.7新增） | §12 |

---

## 6. 技术栈边界与LLM推理后端自托管实施细节

对应RGS-BAS-011§2.2"技术栈边界"与"LLM推理后端"两栏约束（同步BAS-011 v0.3开源合规自审修正与v0.4/v0.6中相关落地）。BAS-011§2.2给出的是约束本身（CI扫描不通过即构建失败、NetworkPolicy出站白名单不得包含商业LLM API端点），本节固定该约束在详细设计层的**实施机制**（静态扫描脚本契约、出站白名单的命名约定、模型权重许可登记流程）。

### 6.1 技术栈边界（FR-NEURO-039）的CI静态扫描契约

RGS-BAS-011§2.2"技术栈边界"行明确"部署镜像**仅**安装MIT许可的`langgraph`／`langgraph-core`／`langchain-core`包，**不得**安装`langgraph-api`包，**不得**在CI/CD中出现`langgraph dev`／`langgraph build`命令"。本节固定该约束的CI扫描实现契约：

```yaml
# 复用RGS-BAS-006§6供应链安全流水线阶段产物，neuro域挂载时追加以下扫描项
# scripts/check-neuro-stack-boundary.sh（在CI的"供应链扫描"阶段调用，扫描失败即阻断构建）
negative-assertions:
  pip-dependency-allowlist:
    - langgraph          # MIT
    - langgraph-core     # MIT
    - langchain-core     # MIT
  pip-dependency-denylist:
    - langgraph-api      # 商业组件，禁用
  ci-script-denylist:
    # 扫描 .github/workflows/ 与 .gitlab-ci.yml 中的 step 脚本名/命令关键字
    - 'langgraph dev'
    - 'langgraph build'
    - 'langgraph-api'
  config-denylist:
    # 扫描部署values.yaml/charts与运行时环境变量命名空间
    - 'LANGGRAPH_PLATFORM_API_KEY'
    - 'LANGGRAPH_CLOUD_ENDPOINT'
# 任意denylist命中：CI退出码非零，阻断构建。允许列表外的依赖：必须先在附件D§4登记并经安全评审
# 扫描脚本与RGS-DTL-025§6.2同类（"反作弊三表挂载检查"）使用相同的失败/阻断语义，命名空间统一
```

**边界说明**：扫描仅覆盖BAS-011§2.2"依赖清单静态扫描"所要求的清单/脚本/配置三个层面，**不**对依赖包的实际运行时行为做沙箱验证（该层级验证由RGS-BAS-009§4的治理CI静态分析与故障注入试验AC-NEURO-008承担，不在本框架职责）。本节扫描命中即视为RGS-BAS-011§2.2构建失败条件已触发，无"警告但放行"分支。

### 6.2 LLM推理后端自托管（FR-NEURO-040）的NetworkPolicy端点命名约定

RGS-BAS-011§2.2"LLM推理后端"行要求"NetworkPolicy出站白名单**不得**包含任何已知商业LLM API服务商的域名/端点"。本节固定该约束在NetworkPolicy层的可校验形态——将"商业LLM API端点"以命名约定形式标记为`commercial-llm-*`前缀，使既有RGS-DTL-002§6.3一致性校验脚本可通过命名约定作**正向匹配**反向断言：

```yaml
# charts/neuro-service/values.yaml 的 allowedEgressTo 取值（追加至§2.3已有列表）
allowedEgressTo:
  - event-bus-consumer-endpoint    # §2.3既有
  - admin-service                   # §2.3既有
  - otel-collector                  # §2.3既有
  - config-store-read-endpoint      # §2.3既有
  - self-hosted-llm-inference-endpoint   # 自托管LLM推理服务（K8s内ClusterIP或同VPC内自建GPU节点）
# 命名约束（供RGS-DTL-002§6.3一致性校验脚本与RGS-DTL-002新增的"商业端点负向断言"扩展点核对）：
# - 任何端点名匹配正则 ^commercial-llm- 的条目禁止出现在本列表
# - 本字段新增条目默认须经安全评审通过（与附件D§4 OSS许可盘点流程联动）
# - 端点实际指向的服务须为自托管（K8s内自建或同VPC内GPU节点），不得解析到任何 .openai.com / .anthropic.com / .googleapis.com 等商业服务商域
```

**自托管验证方法**：在CI的NetworkPolicy Lint阶段（`scripts/check-mount-record-consistency.sh`扩展点），除§2.3已声明的"`allowedEgressTo`不含配置存储写端点"负向断言外，本文档新增第二条负向断言：`allowedEgressTo`中任何端点的DNS解析结果不得落入RGS-BAS-006§4已登记的商业LLM API服务商域名列表（该列表由安全评审维护，本框架仅消费）。两条负向断言并列，CI任一失败即阻断部署。

### 6.3 LLM模型权重许可核实（FR-NEURO-041）的登记流程

RGS-BAS-011§2.2"模型权重许可"未在本文档§5 v0.1版展开（明确"LLM推理后端选型留待详细设计"），本节固定模型权重许可核实的**登记流程**（具体选型由独立的后端详细设计文档承担）：

| 步骤 | 责任方 | 产出 | 落地 |
|---|---|---|---|
| 1. 选型评估 | 架构师 | 候选模型清单（≥2个） | 内部评审会议纪要 |
| 2. 许可条款逐项核实 | 架构师 + 安全 | 许可矩阵（商用允许/禁止、衍生要求、署名要求） | 附件D§4 OSS许可盘点表新增行 |
| 3. 评审通过 | 负责人 | 批准决议 | 附件D§4对应行"评审状态"列 |
| 4. 部署后核对 | SRE | 运行时加载的模型权重哈希与登记一致 | RGS-BAS-006§6供应链流水线（运行时指纹校验，与LLM模型权重维度挂钩） |

**强制项**：商用禁止的模型权重**不得**进入生产部署镜像构建阶段。该约束的最终落实点是步骤4的运行时指纹校验——即使选型评审通过、构建阶段合规，运行时若加载的权重哈希与登记不一致，仍视为RGS-BAS-011§2.2"商用禁止条款"被违反，按RSK-NEURO-002既定流程处置。

---

## 7. 配置存储隔离的IAM/NetworkPolicy双锁细化

对应RGS-BAS-011§4.1"配置存储隔离（FR-NEURO-042）"行（同步BAS-011 v0.4补强确定性边界，处置"绕过闸门直接改配置"这一比闸门被绕过本身更隐蔽的通道）。§2.2/§2.3已落实双锁的物理机制，本节固定**双锁之间的强制关系**与**单锁失守时的处置**（该处置本身是BAS-011§4.1"双层强制"文字的逻辑延伸，非新设计）：

### 7.1 双锁间的强制关系

```
  ┌──────────────────────────────────────────────────────────┐
  │  IAM层(§2.2) + NetworkPolicy层(§2.3) 同时生效            │
  │  → 智能层"物理上够不到"配置存储的写端点                  │
  │                                                          │
  │  强制关系:任一层失守,另一层仍能拦截                      │
  │  → 双层非冗余,是纵深防御(同RGS-BAS-011§7A.2闸门设计精神)│
  └──────────────────────────────────────────────────────────┘
                            ↓
  ┌──────────────────────────────────────────────────────────┐
  │  单锁失守 ≠ 闸门被绕过                                   │
  │  但属于 §7A.3 "禁止的泄漏路径"中的高风险通道             │
  │  (唯一能完全跳过三重闸门的路径,FR-NEURO-042设计依据)    │
  └──────────────────────────────────────────────────────────┘
```

| 场景 | 后果 | 处置（按RGS-BAS-011§7A.3对应泄漏路径行的既定防护设计） |
|---|---|---|
| IAM锁失守（智能层角色被错误授予写权限） | NetworkPolicy锁仍生效，**写请求**无法出站到达配置存储写端点 | 配置存储读端点可继续提供只读访问（开关判定不受影响）；运维侧立即触发权限审计与角色回收 |
| NetworkPolicy锁失守（出站白名单被错误放行写端点） | IAM锁仍生效，**写请求**到达配置存储写端点但被DB/API层角色权限拒绝 | 智能层自身的事件消费循环不受影响（开关读取走读端点，不在白名单阻断范围）；运维侧立即触发NetworkPolicy白名单复核 |
| 双锁同时失守 | **写请求**可实际抵达并被DB/API层接受 | 立即视为RSK-NEURO-002（OSS供应链）与RGS-BAS-011§7A.3对应防护的"FR-NEURO-042防护失效"信号，按RGS-BAS-003§6既定告警推送流程处理，等级不低于闸门绕过告警 |
| 双锁均正常 | 智能层仅可读取配置存储的`neuro_layer_enabled`值 | 正常路径——§2.4消费循环最外层判定 |

### 7.2 与三重闸门的关系

RGS-BAS-011§7A.3泄漏路径表将"写入热更新配置绕开闸门"标注为"风险高于闸门被绕过本身"。本文档§2.2/§2.3/§7.1的IAM+NetworkPolicy双锁是该防护在详细设计层的完整落地；§4确定性闸门则是另一独立防线（约束"建议如何被执行"）。二者**不重复**且**不替代**：

- 闸门被绕过：智能层产出的`Recommendation`绕过`AdminService`入口侧的三重校验直接生效于L0/L1。
- 配置存储被绕写：智能层不经`Recommendation`、不经`AdminService`、不留人工审批记录，直接通过修改`neuro_layer_enabled`（开关本身）或更广义地通过修改ARC-016其他热更新配置（若IAM/NP配置错误放行）来改变L0/L1的判定行为。

两者的**共同点**都是"智能层越过了§2.1组件图约定的唯一出口`AdminService`"，但绕过方式与防御层次不同。两条防线均失守的概率（独立失守事件的交集）远低于任一单独失守——这正是RGS-BAS-011§4.1强调"双层强制"而非"单层"的工程理由。

---

## 8. 分析图高可用与可核对性详细设计

对应RGS-BAS-011§5A.4（NFR-NEURO-009落地，同步BAS-011 v0.5/v0.6升版）。§3.1 DDL已落实§5A.1.1的"物理落位与约束"（部分唯一索引/外键完整性/只增不改），本节固定§5A.4的"高可用"与"可核对性"在详细设计层的实施形态。

### 8.1 高可用（不新建独立容灾方案）

RGS-BAS-011§5A.4.1明确"复用既有RGS-BAS-001§7.1 PostgreSQL同步复制/RGS-BAS-017§2单区域Multi-AZ/RGS-BAS-007§6备份恢复，不为`analysis_graph_definitions`/`analysis_graph_audit_logs`两表单独设计容灾方案"。本节固定**不做什么**的边界，避免实现阶段误读为"该两表不需要容灾"：

- **不新建**独立的复制拓扑（两表落在AD数据库，天然继承RPO=0）
- **不新建**独立的Multi-AZ配置（随AD数据库整体的可用区故障切换能力）
- **不新建**独立的备份策略（分区/备份颗粒度以整库为单位）
- **不单列**恢复演练节奏（随整库节奏）

### 8.2 可核对性（对账/一致性验证任务实施细节）

RGS-BAS-011§5A.4.2列出三项定期核对任务。本节固定每项任务的实施细节（核对脚本读取数据源、比对逻辑、调度与告警通道）。任务调度复用既有定时作业基础设施（不新建独立调度组件）。

#### 8.2.1 状态-实际订阅一致性核对

| 维度 | 实施细节 |
|---|---|
| 读取侧A：`status='生效'`集合 | SQL: `SELECT graph_id, version, subscribed_event_scope FROM analysis_graph_definitions WHERE status='生效';`（读admin_service_role的SELECT权限即可，与§3.1既有授权一致） |
| 读取侧B：智能层实际存活的消费者组 | 来源：消息中间件的管理API（具体中间件选型由基础设施域决定，本框架不指定）或K8s的`kubectl get`消费者组内省（仅当智能层消费者组以独立Deployment形式暴露运行时元数据时）。本框架**不**直接采集，**仅**消费既有采集任务已落地的数据 |
| 比对逻辑 | ① `生效`记录数 ≠ 实际消费者组数：记"配置存在但未实际运行"或"实际运行但未注册"的偏差集合；② 任意一个`生效`记录对应的`subscribed_event_scope`中的Topic/partition_key未被该消费者组实际订阅：记"订阅范围偏差" |
| 调度 | 复用既有定时作业（hourly），不新建调度器 |
| 告警分级 | "有消费者组存活但无`生效`记录"——视为**安全事件**（对应RGS-BAS-011§5A.4.2"②类不一致必须视为安全事件"），按RGS-BAS-003§6推送，等级不低于闸门绕过告警；"有`生效`记录但无消费者组存活"——记为运维异常（非安全事件），按一般告警通道 |

#### 8.2.2 配置内容篡改核对

| 维度 | 实施细节 |
|---|---|
| 读取侧A：`AnalysisGraphAuditLog.spec_checksum` | SQL: `SELECT graph_id, version_after, spec_checksum FROM analysis_graph_audit_logs ORDER BY occurred_at DESC;`（取每个`graph_id+version`的最新一次操作的`spec_checksum`） |
| 读取侧B：`graph_spec_ref`实际内容的实时哈希 | 读取ARC-016配置存储中各`graph_id+version`对应的实际内容，计算SHA-256（与§3.1 DDL中`spec_checksum`列同算法） |
| 比对逻辑 | 任一`graph_id+version`的`spec_checksum`与对应`graph_spec_ref`实际内容哈希不一致：记为篡改命中 |
| 调度 | 与§8.2.1同频（hourly） |
| 告警分级 | **必须**视为FR-NEURO-042防护失效信号（即便IAM/NetworkPolicy双锁生效，仍以本核对作为纵深防御第三层），立即告警并**冻结该图**——临时将对应`status`置为`已废弃`级别的订阅暂停（消费者组退出），待人工排查；排查完成确认为误报后方可由架构师经ARC-014评审恢复 |

#### 8.2.3 审计记录完整性核对

| 维度 | 实施细节 |
|---|---|
| 读取侧A：`AnalysisGraphDefinition.status`变更历史 | 通过`AnalysisGraphAuditLog`聚合：按`graph_id+version`分组，序列化为"每次生效/废弃/参数更新/结构变更/注册"的事件流 |
| 读取侧B：`AnalysisGraphDefinition`当前状态 | 直接SQL读取`status`列 |
| 比对逻辑 | 当前`status='生效'`的每一条记录，是否存在**唯一**一条`action='评审通过转生效'`的审计记录（且无对应`action='废弃'`记录先于其发生）？当前`status='已废弃'`的每一条记录，是否存在**唯一**一条`action='废弃'`的审计记录？"有状态变更但无审计记录"——视为`AnalysisGraphAuditLog`"只增不改"约束或应用层写入路径本身存在缺陷 |
| 调度 | 与§8.2.1/§8.2.2同频（hourly） |
| 告警分级 | **不视为安全事件**（区别于篡改），按RGS-BAS-003既定的"缺陷"流程处理——开发侧收到告警后核对`status`变更路径是否遗漏了审计写入，修复后无需回滚历史（只增不改约束本身保证历史记录不被事后篡改） |

#### 8.2.4 核对任务自身的运维负荷

RGS-BAS-011§5A.4.2末行明确"核对任务的运维负荷被视为智能层运维面估算的一部分（ISS-065核算范围内，非独立追加项）"。本节不对该核算做重新申领，仅在§9双态OLU核算详细化时确认该核算口径未被遗漏。

---

## 9. 双态OLU核算详细化

对应RGS-BAS-011§3.1/§3.2（v0.8新增"双态OLU核算"响应ISS-079、v0.9交叉审核修正起算基数）。RGS-BAS-011§3.1/§3.2已给出完整的双态拆分表与台账数字，本节固定该核算在详细设计层的**实施性细节**——智能层运维面与附件D§5台账的对应关系、各运维面的具体可观察指标（供运维台账填报用）、核算结果的复核时点。

### 9.1 双态OLU的运维面与可观察指标

| 运维面 | 适用状态 | OLU/月 | 可观察指标（供附件D§5台账填报与季度复核使用） |
|---|---|---|---|
| Python/LangGraph依赖管理与漏洞响应 | 关闭态基线 | 6 | (a) 依赖更新PR数量/季度；(b) 漏洞扫描告警响应时延（中位数）；(c) 部署镜像重建次数/季度 |
| 独立Namespace的部署与监控（含开关状态一致性核对任务） | 关闭态基线 | 3 | (a) Deployment发布次数/季度；(b) 告警规则数（与既有命名空间同维度对比）；(c) 开关状态一致性核对任务成功率（与§8.2.1的同一定时作业复用指标） |
| 分析图的迭代维护 | 仅开启态 | 4 | (a) 新增/调整分析规则PR数量/季度；(b) `AnalysisGraphDefinition`中`status`变更为`生效`/`参数更新`/`结构变更`/`废弃`的操作次数/季度（与§3.1 DDL中`analysis_graph_audit_logs`对应） |
| 建议质量监控与误报率跟踪 | 仅开启态 | 3 | (a) `Recommendation`产出量/季度（与§6.1数据结构的`recommendation_id`关联）；(b) 采纳率（GM实际审批通过/拒绝比）；(c) 误报回溯（GM拒绝后回填原因分类） |

**关键边界**：上述指标**不**为核算目的而新增采集点——优先复用既有可观测性数据（OTel/RGS-BAS-004）与§3.1/`analysis_graph_audit_logs`/`Recommendation`审计字段的聚合统计，避免"为核算而核算"的额外运维面。

### 9.2 台账数字（2026-08-17起算基数修正后，与BAS-011§3.2一致）

| 项目 | OLU/月 |
|---|---|
| 附件D§5.3回收后余额（210预算口径，仅计入必须执行项R-1〜R-4、R-6，v0.9修正由+52为+50） | +50 |
| 智能层**关闭态基线**申领 | −9 |
| **关闭态部署后余额** | **+41** |
| 智能层**开启态增量**预留申领（仅开关开启时追加） | −7 |
| **若开关开启，核算后余额** | **+34** |

> **与RGS-BAS-011§3.2的差异**：本节数字与BAS-011 v0.9/v1.0完全一致；R-5非必须执行项不计入基准余额（v0.9修正的来由）。本节**不重新核算**，仅在详细设计层固定该数字与RGS-BAS-011§3.2的同步关系，避免详细设计阶段或后续核对文档出现口径漂移。

### 9.3 核算复核时点

- **季度复核**：附件D§5台账的常规季度复核窗口中，智能层的双态核算数字作为独立行接受复核（不混入其他域的核算）。
- **状态变更复核**：开关从`false`翻转为`true`时，立即触发一次复核——确认开启态增量7 OLU的预留申领是否被正确追加（数字上不构成阻断，但状态变更本身须登记）。
- **关闭态回归复核**：开关从`true`翻转为`false`时（应急关闭），不释放已追加的7 OLU（已消耗的运维面不会随开关关闭而消失——例如已部署的分析图定义仍须保留、可被重放），但复核台账中"开启态增量预留"的实际占用数。

---

## 10. 隔离与降级扩展：故障注入验证与背压落地

对应RGS-BAS-011§7（v0.2初版，NFR-NEURO-001）。BAS-011§7给出三条隔离与降级设计要点（队列上限/熔断/全局降级），其中"全局降级"行明确"以故障注入试验（AC-NEURO-001）验证该'自然结果'确实成立，而非想当然地假设"。本节固定该故障注入试验的**详细设计层契约**（试验假设、注入动作、验收标准），以及队列上限的具体落地。

### 10.1 故障注入试验AC-NEURO-001的详细契约

| 维度 | 实施细节 |
|---|---|
| 试验假设 | "智能层整体不可用（Pod崩溃、依赖故障）时，**不得**产生任何对既有实时/业务路径的影响"（RGS-BAS-011§7全局降级行） |
| 注入动作 | (1) Kill智能层所有Pod（模拟进程崩溃）；(2) 切断智能层到事件基础设施的网络（模拟依赖不可达）；(3) 切断智能层到`AdminService`的网络（模拟建议出口不可达）——三种注入分别执行，每种注入后观察既有路径 |
| 验收标准 | (a) 既有业务的P99/P999延迟不出现可观测变化（与基线对比，阈值由性能基线决定，本文不固定具体数字）；(b) 既有业务的事件消费者组的offset提交速率不下降；(c) 既有业务的`AdminService`调用路径无错误率上升；(d) 智能层恢复后，事件消费循环自动重启并恢复（无需人工介入），与`AdminService`的连接自动重连 |
| 试验频次 | 与RGS-BAS-009§4治理CI的"故障注入回归"节奏一致（季度或每次重大变更后），不单独排期 |
| 不通过时处置 | 任一验收标准未达，视为"隔离与降级设计"本身存在缺陷（非测试方法问题），回退至BAS-011§7的设计假设重新审视 |

### 10.2 队列上限的落地

RGS-BAS-011§7"队列上限"行要求"分析队列（§2.1）**必须**有界，超限时按优先级丢弃（复用ARC-013既定背压原则）"。本节固定该上限的具体取值与丢弃策略：

| 维度 | 实施细节 |
|---|---|
| 队列容量 | 复用既有分析队列的容量配置（具体值由基础设施域按节点规格决定，本框架不固定硬编码），但**必须**有界（无界队列不允许——BAS-011§7强约束） |
| 丢弃优先级 | 按RGS-BAS-011§2.1"分析队列ARC-013背压"既定的优先级策略：`confidence`字段（§6.1数据结构）已计算且达到本图`olu_review_ref`中声明的告警阈值的高优先级事件优先保留；新到达的、尚未进入分析管线的事件优先丢弃 |
| 丢弃记录 | 每次丢弃**必须**记录一条降级事件（含事件ID、丢弃时间、丢弃原因——"队列超限"），便于事后追溯"建议缺失"是否由本降级策略导致（区别于"开关关闭未处理"与"分析逻辑判定不产生建议"） |
| 不丢弃 | 已被分析管线接受（已进入LangGraph执行）的事件**不得**因队列超限被丢弃——丢弃仅发生在"队列入队前"环节，分析中事件由`AdminService`熔断（§10.3）保护，不在队列丢弃范围 |

### 10.3 熔断

RGS-BAS-011§7"熔断"行要求"智能层对`AdminService`的建议提交调用**必须**设超时+熔断，`AdminService`不可用时智能层**不得**阻塞其事件消费循环（继续消费并暂存分析结果，待恢复后补交）"。本节固定熔断参数与暂存机制：

| 维度 | 实施细节 |
|---|---|
| 超时 | 复用既有HTTP/gRPC客户端的超时配置基线（具体值由基础设施域决定，本框架不固定硬编码），但**必须**小于消费循环的下一次拉取间隔——否则熔断无意义 |
| 熔断器策略 | 复用既有熔断器（`AdminService`侧已有SLA监控与熔断约定），失败率阈值与冷却时间跟随既有值，**不**为智能层单独调参 |
| 暂存 | `AdminService`不可用时，分析结果暂存于智能层专属的轻量存储（与§2.1分析队列同一存储介质，§2.2技术栈边界行已声明"编排循环的状态持久化**必须**自行实现"）。`AdminService`恢复后，按事件时间戳顺序补交（**不**堆积超过24小时——超过则视为过期建议丢弃，避免历史回灌） |

---

## 11. 推理输入快照与离线重放复核

对应RGS-BAS-011§7A.4（v0.2初版）。本节固定"输入快照"的存储格式、保留期，以及"离线重放"的工具形态与重放结果的解读方法。

### 11.1 输入快照的存储格式

```sql
-- 智能层每次推理（每个产出的recommendation_id对应一次推理）必须写入一条推理输入快照
-- 该表与RGS-BAS-011§7A.4"输入快照属业务数据而非可观测性数据，不得被日志采样（FR-LOG-040）丢弃"对应
CREATE TABLE neuro_inference_snapshots (
    recommendation_id   UUID PRIMARY KEY,        -- 与§6.1 Recommendation.recommendation_id同
    graph_id            UUID NOT NULL,             -- 哪个分析图
    graph_version          INTEGER NOT NULL,         -- 推理时刻生效的version（FR-NEURO-038"当时配置"语义）
    input_event_ids        UUID[] NOT NULL,           -- 推理依赖的输入事件ID列表（用于重放时定位原始事件）
    input_config_snapshot    JSONB NOT NULL,            -- 推理时刻的配置/阈值/模型版本的完整快照（不含PII）
    model_version              TEXT,                    -- 推理时刻的模型版本（若使用多模型版本化部署）
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
    -- 注：本表不挂载"只增不改"约束——它是分析中间结果而非治理审计，
    --    但保留期独立设定（见§11.3），不随日志采样策略变化
);
CREATE INDEX idx_nis_graph_created ON neuro_inference_snapshots (graph_id, created_at);
```

### 11.2 与既有审计/可观测性的边界

- **不**纳入RGS-BAS-004§5日志采样的"采样丢弃"集合（FR-LOG-040）：快照是业务数据，复用RGS-BAS-011§7A.4"属业务数据而非可观测性数据"分类。
- **不**复用`AnalysisGraphAuditLog`表（治理审计语义不同——`AnalysisGraphAuditLog`记录图定义本身的变化，本表记录每次推理的输入）。
- **不**替代`Recommendation`记录本身——`Recommendation`是面向GM的建议输出（FR-NEURO-022依据），本表是面向离线复核的输入证据。

### 11.3 保留期

- 独立于日志保留期、审计保留期。
- 建议保留期：**≥1年**（与RGS-BAS-003§7审计日志的常规保留期对齐），便于跨季度的复核回溯。
- 详细保留期由运维政策决定（具体年数不固定于本文档，避免保留期变更需修改详细设计书）。

### 11.4 离线重放的工具形态

| 维度 | 实施细节 |
|---|---|
| 重放工具 | 复用既有"事件重放"基础设施（与RGS-DTL-019"事件溯源/重放"同类工具），**不**为智能层单独建设重放工具 |
| 重放输入来源 | (1) `neuro_inference_snapshots.input_event_ids`定位原始事件（从事件基础设施的事件存储中按ID检索）；(2) `input_config_snapshot`提供配置/阈值/模型版本的重放时刻取值 |
| 重放输出 | 重放产生的`Recommendation`**不**写入生产`AdminService`（避免与已发生的实际生产建议混淆），**仅**作为复核对照的参考输出 |
| 重放结果的解读 | RGS-BAS-011§7A.4明确"重放**不保证**产生相同输出（L4的固有属性），重放的目的是**复核当时的判断依据是否合理**，而非验证输出可复现——故重放工具的报告输出应同时呈现"当时配置下的建议"与"重放时刻采用相同配置但不同随机种子的建议"，供复核人员对比判断依据而非判断输出 |

---

## 12. 标准化检查清单详细化

对应RGS-BAS-011§9.1/§9.2/§9.3（同步BAS-011 v0.7将原§9拆分为部署/开启/闸门三段）。本节固定三类检查清单在详细设计层的**可执行形态**——每条检查项的核对方法、对应工具/脚本（若已存在）、与RGS-BAS-011§9原文检查项的对应关系。本节**不**重写检查项本身（检查项由BAS-011§9决定），仅将其转化为可被CI/运维侧直接勾选的实施清单。

### 12.1 §9.1部署检查清单（不依赖开关是否开启）

| BAS-011§9.1原文检查项 | 详细设计层实施形态 | 工具/脚本 | 责任方 |
|---|---|---|---|
| NetworkPolicy已验证：智能层无法连接任何业务数据库，无法直接调用`AdminService`高危方法 | §2.3 NetworkPolicy `allowedEgressTo`核对 + §6.2 商业LLM端点负向断言 | `scripts/check-mount-record-consistency.sh`（RGS-DTL-002§6.3）扩展 | SRE |
| 事件订阅权限已验证：智能层消费者身份仅具备订阅权限，无发布权限 | 消费者组的RBAC/Topic ACL核对 | 事件基础设施管理CLI | SRE |
| 建议呈现已验证：`suggested_action`白名单校验生效，非法动作被拒绝 | §4.2闸门1的单元/集成测试覆盖（白名单全等匹配、非前缀/模糊） | 既有测试框架 | 后端 |
| 故障注入试验已验证既有实时/业务路径无影响 | §10.1 AC-NEURO-001的详细契约 | 故障注入平台 | SRE+架构师 |
| LangGraph及Python依赖已完成OSS许可盘点与漏洞扫描接入 | §6.1 依赖清单扫描通过 | `scripts/check-neuro-stack-boundary.sh`（§6.1） | 安全 |
| 部署镜像不含`langgraph-api`包，CI/CD中不含`langgraph dev`／`langgraph build`命令 | §6.1 denylist全部未命中 | `scripts/check-neuro-stack-boundary.sh`（§6.1） | 安全 |
| NetworkPolicy出站白名单确认不含任何商业LLM API端点 | §6.2 商业LLM端点负向断言通过 | `scripts/check-mount-record-consistency.sh`扩展 | SRE |
| LLM模型权重的许可条款已核实允许商用，并登记至附件D§4 | §6.3 许可矩阵步骤1-3完成 | 附件D§4登记 | 架构师+安全 |
| 全局开关默认值验证：全新部署环境`neuro_layer_enabled`初始值为`false` | §2.1 `INSERT ... ON CONFLICT DO NOTHING`幂等初始化逻辑核对 | 部署脚本的数据库初始化阶段 | SRE |
| 开关关闭态零产出验证：开关为`false`时运行一段观测期，确认无新`Recommendation`产生 | §2.4消费循环最外层判定的回归测试 | 既有测试框架 | 后端 |
| 开关读写隔离验证：智能层服务账号/凭证在IAM与NetworkPolicy两层均无法写入开关底层存储 | §2.2 IAM GRANT核对 + §2.3 NetworkPolicy白名单核对 | `scripts/check-mount-record-consistency.sh`扩展 + 数据库角色权限审计脚本 | 安全+SRE |

### 12.2 §9.2开关开启检查清单（须负责人显式决议）

| BAS-011§9.2原文检查项 | 详细设计层实施形态 | 工具/脚本 | 责任方 |
|---|---|---|---|
| §9.1部署检查清单已全部完成（前置条件） | §12.1全部勾选 | 复选 | 负责人 |
| RGS-REQ-014§9 CR-011已获负责人批准（前置条件） | CR-011决议文档存在 | 文档核对 | 负责人 |
| OLU预算台账余额已核实为非负（附件D§5.4，210预算口径下当前为+34） | §9.2数字+34与附件D§5.4一致 | 附件D§5.4台账 | PM |
| 开关翻转生效时延验证：翻转为开启后，在既定热更新时延内全部实例均开始产出建议 | 翻转操作的端到端集成测试（开关写入→所有消费者实例读取→开始产出） | 既有测试框架 | 后端+SRE |

### 12.3 §9.3确定性闸门检查清单（安全关键）

| BAS-011§9.3原文检查项 | 详细设计层实施形态 | 工具/脚本 | 责任方 |
|---|---|---|---|
| 闸门部署于`AdminService`入口侧，确认未部署于智能层内部 | §4.1 闸门代码物理位于`services/admin-service/src/neuro_gate/`的核对（`neuro-service`镜像中不存在该模块） | 镜像构建产物审计 | 安全 |
| 闸门1枚举校验确认为全等匹配，非前缀/模糊/包含匹配 | §4.2 `ADMIN_SERVICE_METHOD_WHITELIST.contains()`的代码审查（确认无`.starts_with`等） | 代码审查 | 安全 |
| 闸门2值域越界确认为拒绝而非截断 | §4.2 闸门2校验循环的代码审查（确认无`.clamp()`调用） | 代码审查 | 安全 |
| 闸门3的`risk_tier`确认由`suggested_action`自动继承，非智能层自行申报 | §4.2 闸门3的`risk_tier_for_action`查表行为审查（确认忽略`rec`中可能携带的`risk_tier`字段） | 代码审查 | 安全 |
| `Recommendation` schema确认不含承载可执行产物的字段 | §6.1数据结构字段审查（确认无`code`/`sql`/`config`类字段） | 代码审查+契约测试 | 安全 |
| 静态分析确认L0/L1路径的同步调用链中不含智能层组件；NetworkPolicy出站白名单已验证 | RGS-BAS-009§4治理CI静态分析通过 | 治理CI | SRE |
| 对抗性测试已通过并纳入常态化回归 | §10.1 + 闸门专项对抗性测试的回归纳入TL-8 | 既有测试框架 | 安全+后端 |
| 闸门实现的测试覆盖率高于QA-001既定80%基线 | 闸门模块的测试覆盖率报告核对 | 覆盖率工具 | QA |
| 推理输入快照持久化已验证，可离线重放复核 | §11.1 推理输入快照写入路径的集成测试 + §11.4 重放工具的端到端验证 | 集成测试 | 后端 |
| 埋点无副作用对比试验已通过 | RGS-BAS-011§7A.3与RGS-BAS-004§9联动项的AC-NEURO-008 | 故障注入平台 | SRE+架构师 |

### 12.4 清单本身的版本对齐

- 本节清单的检查项**与RGS-BAS-011§9.1/§9.2/§9.3的检查项一一对应**，不增不减。
- 若BAS-011§9后续升版新增/调整检查项，本节须同步更新（升版时直接对照BAS-011修订历史的"影响章节"列）。
- 本节**不**对检查项做优先级标注或勾选顺序的强制——执行顺序由运维SOP决定，本框架仅提供"该项如何被验证"的技术回答。
