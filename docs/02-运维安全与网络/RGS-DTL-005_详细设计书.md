# 详细设计书（詳細設計書 / Detailed Design Document）

**插件热插拔与生命周期管理：PLUGIN_REGISTRY物理数据库设计・沙箱脚本白名单API线格式・经济类插件单点判定算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-005 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-005 插件热插拔与生命周期管理 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-003／004同批次产出，覆盖02-运维安全与网络域第三份文档）。细化RGS-BAS-005§3.1逻辑ER图为`PLUGIN_REGISTRY`/`PLUGIN_AUDIT`具体DDL、§5白名单API与永久事实强制路由落实为具体Rust接口签名与调用伪代码、§6生命周期状态机落实为状态转移表与SQL、§7经济类插件单点判定与ARC-016分发通道复用落实为可直接翻译为Rust实现的伪代码、§9故障隔离的指数退避熔断落实为具体算法。**本版本不覆盖**：沙箱脚本引擎具体选型（TBD-PLG-001）本身的技术选型评审结论、脚本语言绑定层的完整API清单。见§7 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-005§3.1逻辑模型一致，白名单API伪代码是否确实杜绝了绕过`CommitTransaction`直接写库的路径 |
| 评审（安全） | | | §3永久事实路由的epoch/request_id注入点是否真正不可被脚本参数覆盖（对应RGS-BAS-005§5"不得由脚本提供"的强约束） |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：PLUGIN_REGISTRY与PLUGIN_AUDIT](#2-物理数据库设计plugin_registry与plugin_audit)
3. [沙箱脚本白名单API接口设计与永久事实强制路由](#3-沙箱脚本白名单api接口设计与永久事实强制路由)
4. [生命周期状态转移详细设计](#4-生命周期状态转移详细设计)
5. [经济类插件单点判定与跨节点分发算法详细设计](#5-经济类插件单点判定与跨节点分发算法详细设计)
6. [故障隔离：熔断与指数退避算法详细设计](#6-故障隔离熔断与指数退避算法详细设计)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-005给出了`PLUGIN_REGISTRY`/`PLUGIN_AUDIT`的逻辑ER图、生命周期状态机图与传播时序图、白名单API与永久事实强制路由的文字约束、跨节点数据同步的方式选择说明（复用ARC-016通道）、故障隔离的熔断+指数退避文字描述。本文档将其落实为：可执行DDL、白名单API的具体Rust trait签名（含epoch/request_id注入点的类型级强制）、状态转移的完整实现（含OCC并发处理）、经济类插件单点判定的算法伪代码、熔断与指数退避的具体计算公式。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-005已确定的任何结构性选择（不引入动态链接库加载、插件不独立拥有数据库、经济类插件生效判定收归EC单点、跨节点同步复用ARC-016既有分发通道而非独立轮询）。
- 不选定沙箱脚本引擎本身（TBD-PLG-001）——RGS-BAS-005已标注"待详细设计确定"但同时列出了满足性判定标准（内存/时间受限、无文件系统/网络访问、仅白名单API），本文档确认这些标准并落实白名单API接口契约本身，但**不**在本文档内完成引擎选型评审（Rhai/Lua子集等具体产品比较、许可证核对等需要独立评审记录，非本文档单方面可代为决定，同RGS-DTL-027对TBD-CDN-002的处理方式一致：给出的是使用该引擎所需满足的接口契约，不是引擎选型结论）。
- 不覆盖脚本语言绑定层的完整白名单函数清单——具体注册哪些宿主函数是随业务功能迭代持续增长的运营配置事项，本文档只固定"白名单函数注册"这一机制本身的接口形状与"永久事实必须走CommitTransaction"这一强约束，不逐一列举当前已注册的函数集合。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，接口以Rust trait签名给出（`PLUGIN_REGISTRY`跨节点分发走ARC-016既有版本化产物通道，非独立Protobuf协议，故本文档不新增协议消息定义，仅在§5引用该通道的既有产物格式），算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：PLUGIN_REGISTRY与PLUGIN_AUDIT

对应RGS-BAS-005§3.1。两表依附插件所依附限界上下文的既有DB（如`admin_db`或`economy_db`，取决于`bounded_context`字段值所指域），本文档给出通用DDL模板，实际建表时按依附的限界上下文选择目标库。

```sql
-- 插件注册表，对应FR-PLG-001/010〜013，落地RGS-BAS-005§3.1 ER图
CREATE TABLE plugin_registry (
    plugin_id             TEXT PRIMARY KEY,   -- 业务定义的稳定标识符，非UUID（便于代码中硬编码引用，同RGS-BAS-005§4"plugin_id到处理函数的映射表"）
    version                TEXT NOT NULL,
    kind                    SMALLINT NOT NULL CHECK (kind IN (0, 1)),
                            -- 0=feature_flag 1=sandbox_script
    bounded_context           TEXT NOT NULL,   -- 所依附限界上下文缩写，如'EC'/'AD'，复用RGS-REQ-001既有缩写
    state                      SMALLINT NOT NULL DEFAULT 0
                              CHECK (state BETWEEN 0 AND 4),
                              -- 0=已注册 1=已启用 2=已禁用 3=已弃用 4=已移除
    declared_dependencies        JSONB NOT NULL DEFAULT '[]',  -- 声明的API/事件白名单，结构为字符串数组
    script_ref                     TEXT NULL,   -- kind=sandbox_script时必填，指向脚本内容存储引用（对象存储key或版本化产物ID，随ARC-016既有分发通道格式）
    is_economic                      BOOLEAN NOT NULL DEFAULT FALSE,  -- FR-GOV-030判定标记，判定权归EC单点（见§5）
    created_by                         TEXT NOT NULL,
    updated_at                           TIMESTAMPTZ NOT NULL DEFAULT now(),
    version_seq                            BIGINT NOT NULL DEFAULT 0,  -- OCC乐观锁，同ARC-009既有模式
    CONSTRAINT chk_plugin_registry_script_ref
        CHECK (kind = 0 OR script_ref IS NOT NULL)
        -- sandbox_script类型必须有script_ref，feature_flag类型该列恒为NULL（约束落实ER图隐含的类型判别式完整性）
);

CREATE INDEX idx_plugin_registry_bounded_context_state
    ON plugin_registry (bounded_context, state);
    -- 支撑"某限界上下文当前启用插件列表"查询，运行时WATCH组件（RGS-BAS-005§2组件图）拉取时的核心查询路径
CREATE INDEX idx_plugin_registry_is_economic
    ON plugin_registry (is_economic) WHERE is_economic = TRUE;
    -- 部分索引：支撑EC单点判定服务（§5）快速枚举全部经济类插件，非经济类插件不占索引空间

-- 插件生命周期审计表，对应FR-PLG-*留痕要求，仅追加，与OPERATION_AUDIT独立分表（RGS-BAS-005§3.1既定理由：变更频率差异）
CREATE TABLE plugin_audit (
    audit_id       BIGSERIAL PRIMARY KEY,
    plugin_id       TEXT NOT NULL REFERENCES plugin_registry(plugin_id),
    from_state       SMALLINT NOT NULL,
    to_state           SMALLINT NOT NULL,
    operator_id          TEXT NOT NULL,
    occurred_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_plugin_audit_plugin_id_occurred
    ON plugin_audit (plugin_id, occurred_at);
    -- 支撑"某插件历史状态变更记录"查询（AC-PLG-004"移除演练无孤儿表遗留"核对时的取证路径）
```

`plugin_registry`不建立跨库物理FK指向`admin_db.OPERATION_AUDIT`或其依附限界上下文的其他表——同RGS-DTL-001§2/§3多库物理隔离既定约束，`bounded_context`列仅为逻辑标记，实际物理位置由部署时选择的目标DB决定。

---

## 3. 沙箱脚本白名单API接口设计与永久事实强制路由

对应RGS-BAS-005§5表格，尤其"永久事实的强制路由"一行的强约束，落实为宿主侧Rust接口设计，使该约束在类型系统层面而非仅靠代码评审生效。

### 3.1 白名单API注册接口

```rust
// 宿主(所依附限界上下文的服务)显式注册可供脚本调用的函数集合
// 脚本引擎(具体选型见TBD-PLG-001)仅能调用经由本接口注册的函数,无法反射/枚举宿主进程内其他任何符号
pub trait HostFunctionRegistry {
    fn register<F>(&mut self, name: &'static str, func: F)
    where
        F: Fn(&ScriptCallContext, &[ScriptValue]) -> Result<ScriptValue, ScriptError> + 'static;
}

// 脚本调用时宿主自动构造，脚本代码无法自行构造或篡改本结构体的任何字段——
// 这是"session_epoch由宿主注入、不得由脚本提供"（RGS-BAS-005§5）的类型级强制点
pub struct ScriptCallContext {
    pub plugin_id: String,
    pub session_epoch: i64,        // 宿主从当前会话上下文注入，脚本参数中不存在同名可覆盖字段
    pub caller_character_id: String,
    request_id_seed: Uuid,          // 私有字段，脚本无法读取/传入，仅供§3.2内部生成request_id时使用
}
```

### 3.2 永久事实白名单函数的强制封装

```rust
// 示例:"发放声明范围内的道具"这一类白名单API的宿主实现骨架
// 凡产生永久事实(DR-002:道具/货币/购买/交易)的白名单函数,宿主实现一律遵循本模板,不得旁路
fn host_fn_grant_item(ctx: &ScriptCallContext, args: &[ScriptValue]) -> Result<ScriptValue, ScriptError> {
    let (item_template_id, quantity) = parse_grant_item_args(args)?;  // 参数解析走§7A.1既定未信任输入安全规则(RGS-BAS-006同源约束)

    // 关键点: request_id由宿主基于ctx.request_id_seed确定性派生,不接受脚本传入的任何request_id参数
    // (即便脚本参数列表中出现"request_id"这一名字的入参,宿主实现也必须忽略它,不得采纳)
    let request_id = derive_request_id(&ctx.request_id_seed, "grant_item");

    let result = economy_service::commit_transaction(CommitTransactionRequest {
        request_id,
        character_id: ctx.caller_character_id.clone(),
        session_epoch: ctx.session_epoch,   // 同样只能来自ctx,不接受脚本参数覆盖
        operation: TransactionOp::GrantItem { item_template_id, quantity },
        expected_version: fetch_current_wallet_version(&ctx.caller_character_id)?,
    })?;
    // 复用RGS-DTL-001§4.3 CommitTransactionRequest既定线格式,不新设经济操作旁路协议

    Ok(ScriptValue::from(result.new_version))
}
```

**设计要点**（落实RGS-BAS-005§5"不得包含任何直接数据库写入，不得新设旁路"）：`ScriptCallContext`的`session_epoch`字段与`request_id_seed`字段均无对应的公开setter，脚本引擎绑定层在构造`ScriptCallContext`实例时由宿主进程一次性填充，脚本侧`ScriptValue`参数列表中即便恰好包含名为`session_epoch`的键值，`host_fn_grant_item`的实现签名也不从`args`中解析该字段——**结构上不存在**脚本覆盖epoch的输入通道，而非"实现时注意不要读取"这一容易被后续维护者破坏的约定。CI侧应对全部白名单函数实现做静态扫描，确认涉及永久事实的函数体内未出现从`args`解析`session_epoch`/`request_id`同名字段的模式（同RGS-DTL-004§3.3绕过检测的同类思路，扫描规则本身留待实现阶段细化）。

---

## 4. 生命周期状态转移详细设计

对应RGS-BAS-005§6状态机图，落实为状态转移合法性表与SQL实现。

| 当前`state` | 触发操作 | 目标`state` | 前置校验 |
|---|---|---|---|
| （不存在记录） | `CreatePlugin` | 0（已注册） | `plugin_id`唯一（主键约束天然保证） |
| 0（已注册） | `EnablePlugin` | 1（已启用） | — |
| 1（已启用） | `DisablePlugin` | 2（已禁用） | — |
| 2（已禁用） | `EnablePlugin` | 1（已启用） | — |
| 2（已禁用） | `DeprecatePlugin` | 3（已弃用） | — |
| 3（已弃用） | `RemovePlugin` | 4（已移除） | 数据归档决定已完成（FR-PLG-012，具体归档流程本文档不展开，业务侧调用前置校验） |

```rust
fn transition_plugin_state(
    plugin_id: &str,
    expected_from: PluginState,
    to: PluginState,
    expected_version_seq: i64,
    operator_id: &str,
) -> Result<(), PluginError> {
    // 合法性校验:目标状态是否是expected_from的合法后继(按上表),非法转移直接拒绝,不触达数据库
    if !is_valid_transition(expected_from, to) {
        return Err(PluginError::InvalidTransition { from: expected_from, to });
    }

    // 单条UPDATE同时校验state与version_seq,OCC+状态机校验合一,避免TOCTOU(同RGS-DTL-001§3.2既定OCC模式)
    let rows_affected = execute_sql(
        "UPDATE plugin_registry SET state = $1, version_seq = version_seq + 1, updated_at = now()
         WHERE plugin_id = $2 AND state = $3 AND version_seq = $4",
        &[to as i16, plugin_id, expected_from as i16, expected_version_seq],
    )?;

    if rows_affected == 0 {
        // 并发冲突:状态已被其他操作者改变,或version_seq已过期,不区分二者,统一要求调用方重新查询最新状态后重试
        return Err(PluginError::ConcurrentStateChange);
    }

    // 与状态更新在同一事务内追加审计记录(不可分离为两次独立提交,否则可能出现"状态已变但审计缺失"的不一致)
    insert_plugin_audit(plugin_id, expected_from, to, operator_id)?;
    Ok(())
}
```

---

## 5. 经济类插件单点判定与跨节点分发算法详细设计

对应RGS-BAS-005§7表格核心要点："经济类插件生效判定必须由EC单点执行，各节点本地状态仅可用于表现层"。

### 5.1 非经济类插件：ARC-016既有分发通道复用

```rust
// PLUGIN_REGISTRY变更触发本函数(而非各节点独立轮询该表),生成新版本配置产物,
// 复用ARC-016数值表热更新既有的版本化分发+tick边界原子切换+一致性检查+可回退基础设施
fn on_plugin_registry_changed(plugin: &PluginRegistryRow) -> Result<(), PublishError> {
    if plugin.is_economic {
        // 经济类插件不走本函数的"直接影响判定"路径,详见§5.2,本函数仍需为其生成配置产物
        // (供表现层节点读取展示"活动进行中"提示),但该产物不作为道具/货币计算依据
    }
    let config_artifact = build_plugin_config_artifact(plugin);  // 复用ARC-016既有产物构建流程,不新建独立构建逻辑
    let consistency_ok = run_consistency_check(&config_artifact);  // 复用ARC-016既有一致性检查,不合格版本不得反映
    if !consistency_ok {
        return Err(PublishError::ConsistencyCheckFailed);
    }
    publish_new_version(config_artifact)  // 复用既有分发通道,各节点在下一tick边界原子切换到新版本
}
```

### 5.2 经济类插件：EC单点判定

```rust
// 与§5.1不同,经济类插件"是否生效"的判定不发生在各节点独立读取本地插件状态的时刻,
// 而是内联在EconomyService.CommitTransaction的事务执行路径内,与结算同一事务(ARC-006对齐)
fn commit_transaction_with_plugin_check(req: &CommitTransactionRequest) -> Result<CommitTransactionResponse, EcError> {
    // 事务内查询当前生效的经济类插件集合(EC自身直接查plugin_registry当前state,而非依赖各节点本地缓存/ARC-016分发产物)
    let active_economic_plugins = query_active_economic_plugins_in_tx(req.character_id)?;

    let mut adjusted_op = req.operation.clone();
    for plugin in &active_economic_plugins {
        // 插件对本次交易的影响(如活动加成倍率)在同一事务内计算并应用,不产生独立的第二次写入
        adjusted_op = apply_economic_plugin_effect(adjusted_op, plugin)?;
    }

    // 复用RGS-DTL-001§3.2既定"OCC校验+流水写入同一事务边界"物理执行语义,不重复展开
    execute_commit_transaction_tx(req.character_id, adjusted_op, req.expected_version)
}
```

**边界条件说明**（落实RGS-BAS-005§7"不得作为道具/货币计算的判定依据"）：`§5.1`的`build_plugin_config_artifact`产出的配置产物即便包含经济类插件的表现层数据（如活动名称/图标状态），运行时/客户端读取该产物只能驱动UI展示，实际经济计算路径（`§5.2`）**不读取**该分发产物，而是在`CommitTransaction`事务内直接查询`plugin_registry`当前状态——这是两条独立的读取路径，即便两者短暂不一致（分发产物尚未收敛到最新版本，但EC事务内查询已读到最新`plugin_registry`状态），也不构成正确性问题，因为经济结算路径从不信任分发产物，只信任EC自身事务内的实时查询，天然避免RGS-BAS-005原文所述"跨节点套利面"。

---

## 6. 故障隔离：熔断与指数退避算法详细设计

对应RGS-BAS-005§9"重启退避"表格，落实为具体计算公式（复用RGS-BAS-001§4.2.3 Supervisor机制/RGS-BAS-010§4 G-013既有原则，本文档只针对插件场景给出应用形式）。

```rust
struct PluginCircuitState {
    plugin_id: String,
    consecutive_failures: u32,
    last_failure_at: Option<Instant>,
    circuit_open: bool,          // true=已熔断(状态置为已禁用)
}

const FAILURE_THRESHOLD: u32 = 5;             // 提案默认值:连续5次异常触发熔断,与RGS-DTL-025/026同类TBD提案先例一致的做法,非最终值
const BASE_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(300);  // 提案默认值:退避上限5分钟

fn on_plugin_execution_failure(state: &mut PluginCircuitState) -> Option<PluginAction> {
    state.consecutive_failures += 1;
    state.last_failure_at = Some(Instant::now());

    if state.consecutive_failures >= FAILURE_THRESHOLD && !state.circuit_open {
        state.circuit_open = true;
        // 熔断触发:自动置为已禁用(复用§4状态转移,expected_from=已启用),并告警(复用RGS-DTL-003§4/RGS-BAS-003§6告警通道)
        return Some(PluginAction::DisableAndAlert);
    }
    None
}

// 指数退避:是否允许本次重试/重新启用尝试
fn next_retry_allowed_at(state: &PluginCircuitState) -> Instant {
    let backoff_secs = BASE_BACKOFF.as_secs() * 2u64.saturating_pow(state.consecutive_failures.min(10));
    // saturating_pow(10)封顶,避免consecutive_failures持续增长导致2^n溢出u64(未信任输入安全同款纪律,RGS-BAS-006§7A.1精神的复用)
    let backoff = Duration::from_secs(backoff_secs).min(MAX_BACKOFF);
    state.last_failure_at.unwrap_or_else(Instant::now) + backoff
}
```

**关键边界条件**（落实RGS-BAS-005§9"不得在检测到连续异常后立即重试"）：`on_plugin_execution_failure`本身不执行重试动作，只更新状态与判定是否需要熔断——实际的重新启用尝试（无论是自动化流程还是人工`EnablePlugin`调用）必须先查询`next_retry_allowed_at`确认已过退避窗口，未过窗口的重新启用请求应在应用层直接拒绝（而非依赖调用方自觉遵守间隔），超过既定重试次数上限后转入人工介入分支（本文档不展开人工介入的具体流程，复用RGS-BAS-001§4.2.3既有分支）。

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：`plugin_registry`/`plugin_audit`两表物理DDL（含经济类标记的部分索引）、沙箱脚本白名单API的Rust接口设计与永久事实强制路由的类型级强制机制、生命周期状态转移的完整合法性表与OCC实现、经济类插件单点判定与非经济类插件ARC-016通道复用的双路径算法、熔断与指数退避的具体计算公式。

本版本明确不覆盖、留待后续：

- 沙箱脚本引擎最终选型（TBD-PLG-001）——本文档只固定该引擎需满足的接口契约（§3.1），不代为完成Rhai/Lua子集等具体产品的选型评审，评审需独立进行并核对许可证/资源限制实现细节。
- 白名单函数的完整清单——随业务功能持续增长的运营配置事项，非架构层面一次性决策，本文档不逐一列举。
- §6熔断阈值（`FAILURE_THRESHOLD`=5、`MAX_BACKOFF`=5分钟）的正式校准——当前为初始提案，需PH-4真实故障率数据支撑校准。
- `PLUGIN_AUDIT`的具体查询接口协议线格式（GM后台/运维工具如何检索该表）——RGS-BAS-005原文未展开该查询接口本身的方法签名，本文档同样不代为新增（若需要，应先由BAS层决定是否新增对应`AdminService`方法）。

后续详细设计建议顺序：本文档§5经济类插件单点判定依赖`EconomyService.CommitTransaction`既有事务边界（RGS-DTL-001§3.2已详细设计），两者已具备一致的物理执行语义基础；§3永久事实强制路由涉及的具体白名单函数实现，建议在RGS-BAS-009（若存在对应经济治理域详细设计）中按业务功能逐项补充，本文档只提供模板与强制约束点。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-005§3.1/§3.2 插件注册表设计与一致性机制 | §2 |
| RGS-BAS-005§4 特性开关插件设计 | §5.1 |
| RGS-BAS-005§5 沙箱脚本插件设计（永久事实强制路由） | §3 |
| RGS-BAS-005§6 生命周期状态机与触发时序 | §4 |
| RGS-BAS-005§7 跨节点数据同步设计（经济类插件单点判定） | §5 |
| RGS-BAS-005§8 回滚设计 | §4（状态转移表涵盖回滚路径：已禁用→已启用等） |
| RGS-BAS-005§9 故障隔离设计 | §6 |
| RGS-DTL-001§3.2 CommitTransaction物理执行语义 | §3.2（复用） |
| RGS-DTL-001§4.3 CommitTransactionRequest线格式 | §3.2（复用） |
