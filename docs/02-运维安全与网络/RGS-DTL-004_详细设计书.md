# 详细设计书（詳細設計書 / Detailed Design Document）

**埋点与日志规范：统一埋点SDK接口设计・结构化日志线格式・脱敏与强制全采集算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-004 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-004 埋点与日志规范 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-003同批次产出，覆盖02-运维安全与网络域第二份文档）。细化RGS-BAS-004§4字段规范为统一埋点SDK的具体Rust trait/结构体接口、§5脱敏规则表落实为可直接翻译为Rust实现的字段拦截算法伪代码、§6.2强制全量采集判定落实为具体算法、§3高基数注记的`scene_id`分桶方案给出初始提案（TBD-LOG-004）。**本版本不覆盖**：日志聚合基础设施的物理选型与索引策略、`trace_sample_ratio`具体数值（TBD-LOG-001）。见§7 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | SDK接口设计是否真正让业务代码无法绕过（RGS-BAS-004§2"唯一入口"要求） |
| 评审（安全/合规） | | | 脱敏算法伪代码（§3）是否存在可被字符串拼接等手法绕过的路径（对应RGS-BAS-004§9"脱敏黑名单绕过检测"CI项） |
| 审批（负责人） | | | 本文档的基准化；TBD-LOG-004分桶方案提案是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [统一埋点SDK接口设计](#2-统一埋点sdk接口设计)
3. [脱敏算法详细设计](#3-脱敏算法详细设计)
4. [结构化日志线格式](#4-结构化日志线格式)
5. [强制全量采集判定算法详细设计](#5-强制全量采集判定算法详细设计)
6. [TBD-LOG-004：scene_id高基数分桶方案提案](#6-tbd-log-004scene_id高基数分桶方案提案)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-004给出了指标目录、span/日志字段命名规范的表格、脱敏规则表与"SDK内置黑名单优先于清洗"的原则、采样与强制全量采集的判定条件文字描述。本文档将其落实为：统一埋点SDK的具体Rust接口签名（使"业务代码不得直接调用裸OTel API"这一约束在编译期/CI层面可强制）、脱敏拦截的算法级伪代码、结构化日志的具体线格式（JSON schema）、强制全量采集判定的可执行算法、`scene_id`高基数分桶的初始提案。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-004已确定的任何结构性选择（业务代码不得直接调用裸OTel SDK、脱敏优先于清洗而非事后清洗、高频tick路径不产生span只用指标）。
- 不覆盖日志聚合基础设施的物理选型与索引策略——RGS-BAS-004原文已明确该项"留待详细设计阶段确定"，属独立技术选型（依ARC-014判定基准），本文档只定义SDK产出的日志格式契约，不选定接收端存储。
- 不覆盖`trace_sample_ratio`具体数值（TBD-LOG-001）——RGS-BAS-004已注明"待PH-4负载试验确定"，本文档不预先给出数值提案（不同于TBD-LOG-004，该项缺乏可推导的合理初始值，贸然提案的风险高于留白）。
- 不覆盖具体的CI lint规则实现代码（如`clippy`自定义lint的Rust源码本身）——本文档固定的是lint**检查什么**（黑名单模式、字段命名比对逻辑），不是lint工具自身的实现细节。

### 1.3 记述规则

沿用既有DTL文档记述规则：协议/接口以Rust trait签名与JSON schema给出（本文档不涉及Protobuf——日志/指标/trace走既有OTel OTLP协议，OTLP本身已是既定线格式，本文档只固定SDK暴露给业务代码的封装层接口与日志JSON字段），算法伪代码可直接对应Rust `Result`实现。

---

## 2. 统一埋点SDK接口设计

对应RGS-BAS-004§2"业务代码不得直接调用裸OTel SDK"与§8脚手架集成设计，落实为具体trait签名。

### 2.1 日志API

```rust
// 统一日志入口，脚手架生成的业务代码模板只暴露本trait，不导出`tracing`/`log`原始宏
pub trait RgsLogger {
    // message: 人类可读简述,不得拼接结构化数据(RGS-BAS-004§4.3.1既定)
    // fields: 业务扩展字段(§4.3.2),类型受限为可序列化的基础类型集合,避免误传大对象拖慢日志管道
    fn info(&self, message: &str, fields: &[LogField]);
    fn warn(&self, message: &str, fields: &[LogField]);
    fn error(&self, message: &str, fields: &[LogField]);   // 调用error()自动触发§5强制全量采集判定
    fn debug(&self, message: &str, fields: &[LogField]);   // 生产环境默认丢弃,不经网络发送(§4.2既定)
    fn trace(&self, message: &str, fields: &[LogField]);   // 同debug,仅开发/预发布环境
}

pub enum LogFieldValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}
pub struct LogField {
    pub key: &'static str,      // 静态字符串,强制snake_case由CI lint检查(RGS-BAS-004§9),而非运行时校验
    pub value: LogFieldValue,
    pub sensitive_hint: bool,   // §3.2字段级标注机制,默认false,业务代码对已知敏感自定义字段显式置true
}
```

`service.name`／`trace_id`／`span_id`／`timestamp`／`level`五个基础字段（§4.3.1）**不**出现在`LogField`调用参数中——由SDK实现在`info`/`warn`等方法内部自动从当前OTel上下文与进程启动配置注入，业务代码无法覆盖，这是"脚手架产出物业务代码无需手写"（RGS-BAS-004§8）在接口设计上的强制落地方式。

### 2.2 Span API

```rust
pub trait RgsTracer {
    // name须符合`<限界上下文缩写>.<动词短语>`格式,SDK内部在debug构建下assert格式,
    // release构建仅CI lint检查(§4.1既定"span命名格式CI警告"级别,不在运行时panic阻断线上流量)
    fn start_span(&self, name: &'static str) -> RgsSpanGuard;
}
pub struct RgsSpanGuard {
    // Drop实现自动关闭span,业务代码无需显式调用end(),避免遗漏关闭导致的span泄漏
}
```

高频tick路径（场景Actor tick循环）**不**提供便捷的`start_span`调用点——脚手架生成的场景Actor模板不在`scene_actor_tick`函数体内注入span中间件（同RGS-DTL-001§5.1 tick循环结构一致，本文档不重复该函数体），仅提供§4指标上报API，这是"高频路径不产生span"（RGS-BAS-004§4.1/§7）在代码结构层面的强制落地（而非仅靠约定）。

### 2.3 指标API

```rust
pub trait RgsMetrics {
    fn record_histogram(&self, name: &'static str, value: f64, labels: &[(&'static str, &str)]);
    fn record_counter(&self, name: &'static str, delta: u64, labels: &[(&'static str, &str)]);
    fn record_gauge(&self, name: &'static str, value: f64, labels: &[(&'static str, &str)]);
}
```

`labels`的value侧类型固定为`&str`而非允许任意值拼接——防止业务代码误将`character_id`等高基数标识符直接作为标签值传入（§3高基数注记的接口层防线，CI侧仍需§6分桶方案作为语义层面的第二道防线，接口类型系统本身无法阻止"传入一个语义上是`character_id`但字面上是字符串"的误用）。

---

## 3. 脱敏算法详细设计

对应RGS-BAS-004§5.1脱敏规则表与§5.2"SDK内置黑名单，不依赖开发者主动调用"。

### 3.1 黑名单拦截主算法

```rust
// LogField/span属性写入前统一经过本函数,SDK内部调用点收敛于info/warn/error/start_span实现内部,
// 业务代码无法绕过(RGS-BAS-004§2"唯一入口"要求在此处的算法级体现)
fn sanitize_field(field: &LogField) -> LogFieldValue {
    if field.sensitive_hint {
        // 显式标注的业务专属敏感字段,直接按类别应用脱敏规则,不做黑名单模式匹配(已明确敏感,无需猜测类别)
        return redact_value(&field.value);
    }

    let key_lower = field.key.to_ascii_lowercase();
    for pattern in SENSITIVE_KEY_PATTERNS {
        if key_lower.contains(pattern) {
            // 命中黑名单:替换为[REDACTED]并记录一次内部指标提示开发者误用(RGS-BAS-004§5.1"告警提示开发者误用")
            emit_metric("rgs_log_field_redacted_total", 1.0);
            return LogFieldValue::Str("[REDACTED]".to_string());
        }
    }
    field.value.clone()
}

// 黑名单模式,覆盖RGS-BAS-004§5.1"账号凭证/密码/Token"类别
const SENSITIVE_KEY_PATTERNS: &[&str] = &["token", "password", "credential", "secret", "pin"];
```

### 3.2 分类脱敏规则

```rust
fn redact_value(value: &LogFieldValue) -> LogFieldValue {
    match value {
        // IP地址:保留网段,末段掩码(§5.1),精确到位数为TBD-LOG-002,本文档提案IPv4掩码最后一段(/24等价)
        // 作为TBD-LOG-002最终合规裁定前的过渡默认值(与RGS-BAS-004"精确到位数留待TBD-LOG-002"一致,
        // 本文档提案的具体位数与TBD-LOG-002本身不冲突——若裁定结果与本提案不同,只需替换本函数实现)
        LogFieldValue::Str(s) if looks_like_ipv4(s) => LogFieldValue::Str(mask_last_octet(s)),
        // 邮箱/手机号:不可逆哈希化(§5.1),使用确定性哈希(同一输入恒定输出同一哈希,支持客服查重场景)
        LogFieldValue::Str(s) if looks_like_email_or_phone(s) => {
            LogFieldValue::Str(deterministic_hash_hex(s))
        }
        // 兜底:无法识别具体类别但被标注为敏感,禁止记录(同"账号凭证"类别的处理方式,宁可过度脱敏不可漏判)
        _ => LogFieldValue::Str("[REDACTED]".to_string()),
    }
}
```

### 3.3 绕过检测的CI侧算法依据

对应RGS-BAS-004§9"脱敏黑名单绕过检测"CI项。本文档明确该检测的判定逻辑（供CI lint实现参考，非CI工具本身的代码）：

```
扫描规则: 检测源码中是否存在
  (a) 敏感字段名变量与字符串字面量的format!/+连接操作，其结果作为LogField.value传入
  (b) 敏感字段名变量在传入LogField前经过非SDK提供的自定义"脱敏"函数处理
      （即绕开§3.1 sanitize_field，自行实现一遍脱敏逻辑——即便动机是"这个字段黑名单没覆盖"，
      正确路径是走§3.2 sensitive_hint标注机制，而非在业务代码内自建脱敏实现）
命中(a)或(b): CI失败,阻断合并(同RGS-BAS-004§9既定阻断级别)
```

---

## 4. 结构化日志线格式

对应RGS-BAS-004§4.3字段规范表格，落实为SDK实际输出的JSON schema。

```json
{
  "timestamp": "2026-08-17T10:23:45.123Z",
  "level": "INFO",
  "service.name": "economy-service",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "message": "commit transaction succeeded",
  "player_id": "8f14e...",
  "character_id": "3ac21...",
  "request_id": "req-9f2c...",
  "operator_id": null
}
```

- 五个基础字段（`timestamp`／`level`／`service.name`／`trace_id`／`span_id`）恒定出现在字段名固定位置（`trace_id`/`span_id`在当前上下文不可得时——如启动阶段尚无活跃trace——省略该字段而非置空字符串，避免下游检索按空字符串误匹配）。
- 业务扩展字段（`player_id`等）按§4.3.2"依上下文可得性附加"，不可得时**不出现**该key（而非`null`），JSON体积随日志上下文动态变化，日志聚合存储的字段索引应按"字段可能不存在"设计（这是留给日志聚合选型阶段的约束条件，本文档在此明确声明该约束，但不选定满足该约束的具体存储产品）。
- 字段名书写与本文档§4.3.1/4.3.2表格逐一对应，`snake_case`；`service.name`是唯一例外（含`.`），因其直接复用OTel resource attribute既定命名（RGS-BAS-004§4.3.1已注明"与OTel resource attributes一致"），CI字段命名检查（RGS-BAS-004§9）应对此字段单独放行，不误报为命名违规。

---

## 5. 强制全量采集判定算法详细设计

对应RGS-BAS-004§6.2表格与"采样判定逻辑内置于SDK层，业务代码无需感知"。

```rust
fn should_force_full_capture(ctx: &EmitContext) -> bool {
    // 四类判定条件,与RGS-BAS-004§6.2表格逐项对应,任一命中即强制全采集
    if ctx.level == LogLevel::Error {
        return true;   // 类别1: ERROR级别日志与对应span
    }
    if ctx.service_name == "admin-service" && ctx.is_admin_service_method_call {
        return true;   // 类别2: 全部GM指令(AdminService全部方法调用)
    }
    if ctx.matches_high_risk_operation() {
        return true;   // 类别3: 高危操作(含二次确认流程),判定依据RGS-DTL-003§6高危操作方法名白名单
    }
    if ctx.is_degradation_or_backpressure_reject {
        return true;   // 类别4: 降级/背压拒绝路径(ARC-007降级,ARC-013背压拒绝)
    }
    false
}

// span/日志实际发出前的统一判定入口(SDK内部,业务代码不可见)
fn should_emit(ctx: &EmitContext, sample_ratio: f64) -> bool {
    if should_force_full_capture(ctx) {
        return true;   // 强制路径:忽略sample_ratio
    }
    // 非强制路径:按sample_ratio概率采样(具体随机数生成使用trace_id派生的确定性采样,
    // 保证同一trace内的多个span采样决策一致,避免"链路一半被采样一半未被采样"的割裂视图)
    deterministic_sample(ctx.trace_id, sample_ratio)
}
```

**边界条件说明**：`matches_high_risk_operation`的判定集合（方法名白名单）应与RGS-DTL-003§6"高危操作阈值"覆盖的方法（`ConfirmSceneRestart`、超阈值批量`BanAccount`/`GrantCompensation`）保持同步——若RGS-DTL-003后续新增高危操作类型，本文档的白名单判定集合须同步更新，两份文档在此处存在维护耦合，本文档在§7中列为已知留意事项而非缺陷。

---

## 6. TBD-LOG-004：scene_id高基数分桶方案提案

RGS-BAS-004§3.2标注`scene_id`直接作为指标标签会导致基数爆炸，分桶方案留待详细设计确定（TBD-LOG-004）。本文档提出以下初始提案，非最终值：

```rust
// scene_id分桶: 场景类型 + 区域分片ID的组合,而非scene_id本身
// 有限基数: 场景类型数(数十级,随玩法内容增长但增长缓慢) × 分片数(部署规模级,同样有限)
// 相比scene_id本身(与在线场景实例数同量级,可达数千至数万)基数下降数个数量级
fn scene_metric_label(scene: &SceneState) -> String {
    format!("{}#{}", scene.scene_type, scene.shard_id)
    // 示例: "dungeon_boss_01#shard-3"
}
```

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| 分桶维度 | `场景类型`（如"新手村"/"副本-BOSS-01"）＋`分片ID` | 两者均为部署/内容配置级有限基数量，运维排查"某类场景整体是否偏慢"或"某分片是否过载"是最常见诉求，均可由该组合维度满足 |
| 逐场景实例诊断 | 不通过指标标签实现，改用§5"慢tick诊断快照"（RGS-BAS-004§7既定机制）以日志形式记录具体`scene_id`，经`trace_id`/日志检索定位单个实例 | 与RGS-BAS-004§3高基数注记"逐实体/逐玩家的维度分析应通过日志/trace的关联ID检索实现"原文要求直接对应，本文档只是把该原则应用到`scene_id`场景 |

以上提案应在PH-4阶段结合实际场景类型数量与分片规模验证基数是否确实可控，校准结果回写本文档新版本。

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：统一埋点SDK的日志/Span/指标三组Rust trait接口签名、脱敏拦截的完整算法伪代码（黑名单匹配+分类脱敏+绕过检测判定逻辑依据）、结构化日志的具体JSON线格式、强制全量采集判定的可执行算法、TBD-LOG-004 `scene_id`分桶方案的初始提案。

本版本明确不覆盖、留待后续：

- 日志聚合基础设施的物理选型与索引策略——RGS-BAS-004原文已明确留待详细设计阶段确定，属独立技术选型评审，本文档只固定其消费方看到的日志格式契约（§4），不选定存储产品。
- `trace_sample_ratio`具体数值（TBD-LOG-001）——需PH-4负载试验数据支撑，当前缺乏可推导的合理初始值，本文档不强行给出提案。
- CI lint工具本身的实现代码（`cargo clippy`自定义lint的Rust源码）——本文档只固定检查逻辑的语义（§3.3），不实现lint工具。
- Webhook/告警链路对本文档指标的具体消费逻辑——已由RGS-DTL-003覆盖（RGS-BAS-003§6/§7消费本文档§3指标目录），本文档不重复。

后续详细设计建议顺序：本文档§5强制全量采集的高危操作白名单与RGS-DTL-003§6高危操作阈值提案存在同步维护关系，两份文档应在后续修订中交叉核对；`scene_id`分桶提案（§6）确定后，应同步检查RGS-DTL-001§5.1 tick循环伪代码中`emit_metric("tick_overrun", ...)`调用点是否需要补充分桶标签参数。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-004§2 整体埋点与日志数据流（唯一入口原则） | §2 |
| RGS-BAS-004§3 指标目录 | §2.3、§6 |
| RGS-BAS-004§4.1 Span命名规范 | §2.2 |
| RGS-BAS-004§4.2 日志级别 | §2.1 |
| RGS-BAS-004§4.3 日志字段规范 | §2.1、§4 |
| RGS-BAS-004§5 脱敏设计 | §3 |
| RGS-BAS-004§6 采样设计 | §5 |
| RGS-BAS-004§7 高频路径可观测性设计 | §2.2（不提供tick内span调用点） |
| RGS-BAS-004§8 标准化脚手架集成 | §2（接口即脚手架产出物契约） |
| RGS-BAS-004§9 CI静态检查设计 | §3.3、§4 |
| TBD-LOG-002（IP脱敏精确位数） | §3.2（过渡默认值提案，非最终裁定） |
| TBD-LOG-004（scene_id分桶方案） | §6 |
