# 详细设计书（詳細設計書 / Detailed Design Document）

**请求处理链标准化：`tower::Layer`管道具体实现・字段级线格式・脚手架生成物骨架代码详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-023 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-023 请求处理链标准化——前处理/后处理管道 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定。细化RGS-BAS-023§2管道结构图为具体`tower::Layer`组合的Rust伪代码、§3`RequestContext`逻辑字段为具体结构体定义、§3.2校验规则声明为具体`.proto`字段扩展格式、§4.1脱敏/序列化并行支线为具体伪代码、§4.2审计判定表为可执行判定函数、§5统一错误响应结构为具体协议格式、§6.1脚手架生成物为具体代码模板骨架。**本版本不覆盖**：`tower`生态各中间件内部实现细节（如具体限流算法选型）、既有服务迁移的具体排期表（RGS-BAS-023§6.3已声明"按各限界上下文自身排期，无统一强制截止时间"，排期属于项目管理范畴非本文档职责）。见§8 | 全部 |
| 0.2 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | **同步父 BAS-023 升版至 v0.2**（1 次升版，BAS-023 v0.2 装饰性升版）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-023 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | `tower::Layer`组合顺序是否与RGS-BAS-023§2.1流程图完全一致，`RequestContext`字段是否与§3.1表格逐字段对应 |
| 评审（安全） | | | 前处理阶段是否存在绕过§6.4同步依赖边界的隐蔽I/O路径（如日志中意外记录未脱敏字段） |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [管道的`tower::Layer`组合实现](#2-管道的towerlayer组合实现)
3. [`RequestContext`与前处理字段级实现](#3-requestcontext与前处理字段级实现)
4. [后处理字段级实现](#4-后处理字段级实现)
5. [统一错误响应协议格式](#5-统一错误响应协议格式)
6. [脚手架生成物骨架代码](#6-脚手架生成物骨架代码)
7. [既有服务迁移的技术判定协议](#7-既有服务迁移的技术判定协议)
8. [本文档的覆盖范围与后续计划](#8-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-023给出了管道分层结构图、各阶段职责与复用对象表、`RequestContext`逻辑字段表、统一错误响应逻辑字段表、脚手架集成的生成物清单。本文档将其落实为：`tower::Service`/`Layer`组合的具体Rust伪代码（可直接翻译为实现）、`RequestContext`的具体结构体定义、错误响应的具体协议格式、脚手架生成物的骨架代码模板，使实现人员可直接依此产出可编译代码，不必再对RGS-BAS-023的流程图做二次设计判断。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-023已确定的任何结构性选择（管道运行在服务进程内而非独立网关层、前后处理固定顺序不可重排、序列化先于脱敏且二者并行而非串行、审计判定集中在管道后处理⑤阶段）。若细化过程中发现基本设计本身有缺陷，修正应回写RGS-BAS-023，不在本文档内悄悄改写。
- 不选定`tower`生态之外的具体限流算法/令牌桶参数——RGS-BAS-023§2.2已将限流阶段的具体算法标注为"复用NFR-SEC-008既有速率限制标准"，本文档只给出该阶段在管道中的接入点，不重新设计限流算法本身。
- 不覆盖既有服务迁移的具体排期——RGS-BAS-023§6.3已明确排期"按各限界上下文自身排期"，属项目管理范畴，本文档§7只给出"某服务是否可被判定为迁移完成"的技术判定协议，不代为排期。

### 1.3 记述规则

沿用既有DTL文档记述规则：Rust伪代码可直接对应`tower::Service`/`Layer` trait实现；协议格式以Protobuf风格给出（管道内部结构，非跨服务网络协议，但沿用同一字段编号纪律以便未来若需要跨进程传播`RequestContext`片段时兼容）。

---

## 2. 管道的`tower::Layer`组合实现

对应RGS-BAS-023§2.1管道结构图，落实为`tower::Layer`组合顺序的具体伪代码。RGS-BAS-023§1已明确"以既有Rust服务框架的中间件/拦截器机制（如`tower::Service`/`Layer`组合模式）实现"，本节是该决定的直接落地。

```rust
// 前处理层顺序固定(FR-PPL-001)，ServiceBuilder按声明顺序从外到内包裹，
// 请求方向：最外层Layer最先执行，与§2.1流程图P1→P2→P3→P4→P5顺序一致
fn build_service_pipeline<S>(inner: S) -> impl Service<Request, Response = Response>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
{
    ServiceBuilder::new()
        .layer(TraceContextLayer::new())        // ①追踪上下文
        .layer(AuthenticationLayer::new())       // ②鉴权
        .layer(RateLimitLayer::new())            // ③限流
        .layer(InputValidationLayer::new())      // ④输入校验
        .layer(IdempotencyKeyLayer::new())       // ⑤幂等键提取
        .service(inner)                          // 业务逻辑(开发者填充)
        // 后处理层不通过tower::Layer包裹业务服务实现(响应方向的横切关注点)，
        // 而是在响应路径显式串联,对应§2.1的Q1→Q2→Q3→Q4→Q5，见下方response_pipeline
}

// 后处理管道：对业务逻辑返回的Result<Response, BizError>做统一后处理
// 对应§2.1 "P2/P3/P4拒绝时直接进入Q3脱敏/Q4埋点/Q5审计,不执行业务逻辑"这一短路语义
async fn response_pipeline(
    outcome: Result<BizResponse, PipelineShortCircuit>,
    ctx: &RequestContext,
) -> WireResponse {
    // ①结果规范化：无论是业务成功返回值,还是前处理短路产生的错误,统一映射为内部结果类型
    let normalized = normalize_result(outcome);
    // ②序列化：面向客户端的响应体，不脱敏(§4.1"客户端本就有权查看自己请求的完整合法数据")
    let wire_response = serialize_for_client(&normalized);
    // ③脱敏 与 ④埋点上报：与②并行的两条支线，共享同一份normalized，但目的地不同(§4.1)
    let redacted_for_log = redact_for_logging(&normalized);
    emit_golden_metrics(ctx, &normalized);
    // ⑤审计留痕：判定逻辑见§4.2，集中在此处，业务方法不得自行调用审计接口
    if should_audit(ctx, &normalized) {
        write_audit_record(ctx, &redacted_for_log);
    }
    wire_response
}
```

`PipelineShortCircuit`携带前处理阶段拒绝的具体阶段与原因（鉴权失败／限流拒绝／校验失败），供`normalize_result`映射为§5统一错误响应结构对应的`error_code`；该类型不暴露给业务逻辑层，业务逻辑只处理`Result<BizResponse, BizError>`，两种错误来源在响应生成前已统一收敛。

---

## 3. `RequestContext`与前处理字段级实现

对应RGS-BAS-023§3.1逻辑字段表，落实为具体结构体：

```rust
#[derive(Clone, Default)]
struct RequestContext {
    trace_id: TraceId,                          // ①写入，复用RGS-BAS-004既定字段(同RGS-DTL-001§10.4 TraceContext)
    span_id: SpanId,                             // ①写入
    account_id: Option<AccountId>,                // ②写入，鉴权失败为None
    session_epoch: Option<i64>,                    // ②写入，鉴权失败为None
    authenticated: bool,                            // ②写入，鉴权失败则为false(§3.1既定标记)
    rate_limit_decision: Option<RateLimitDecision>,  // ③写入
    idempotency_key: Option<String>,                  // ⑤写入，仅确定请求路径方法填充
}
```

### 3.1 输入校验规则的具体协议扩展（对应RGS-BAS-023§3.2）

```protobuf
// 附加在既有.proto字段定义之上的自定义option扩展，对应§3.2"值域/格式/必填性"三个维度
extend google.protobuf.FieldOptions {
  ValidationRule rule = 50001;   // 编号取自自定义option保留区间,不与业务字段编号冲突(同RGS-DTL-001§1.3字段编号纪律精神)
}
message ValidationRule {
  bool required          = 1;
  int32 min_length         = 2;   // 字符串类字段适用，0表示不限制
  int32 max_length           = 3;
  string regex_pattern         = 4;  // 复用既有日志脱敏模式库的字段格式定义(§3.2"避免重复定义")
  repeated string enum_values     = 5;  // 枚举取值集合校验
}

// 使用示例：CommitTransactionRequest.request_id字段附加校验规则(与RGS-DTL-001§4.3字段编号1对应)
// message CommitTransactionRequest {
//   string request_id = 1 [(rule) = { required: true, regex_pattern: "^[0-9a-f-]{36}$" }];
// }
```

```rust
// ④输入校验阶段的执行逻辑，对应§3.2"校验失败时必须指明具体是哪个字段、哪条规则未通过"
fn validate_input(msg: &dyn ProtoMessage) -> Result<(), Vec<FieldError>> {
    let mut errors = vec![];
    for field in msg.declared_fields() {
        if let Some(rule) = field.validation_rule() {
            if rule.required && field.is_empty() {
                errors.push(FieldError { field: field.name(), reason: "required field missing".into() });
                continue;   // 必填性未通过时不再对该字段做后续规则校验(避免重复报错同一字段)
            }
            if !field.is_empty() {
                check_length_and_pattern(field, &rule, &mut errors);
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

---

## 4. 后处理字段级实现

### 4.1 脱敏与序列化并行支线（对应RGS-BAS-023§4.1）

```rust
// 序列化：产出返回给客户端的响应，不脱敏
fn serialize_for_client(result: &NormalizedResult) -> WireResponse {
    encode_wire_format(result)   // 复用RGS-REQ-012/BAS-008既有协议编解码层，本文档不重复设计编解码本身
}

// 脱敏：产出写入日志/指标系统的记录，与serialize_for_client互不依赖、互不调用对方产出
fn redact_for_logging(result: &NormalizedResult) -> RedactedRecord {
    let mut record = result.to_loggable_record();
    for field in record.fields_mut() {
        if let Some(pattern) = redaction_pattern_for(field.name()) {   // 复用RGS-BAS-004§5既定脱敏规则
            field.value = apply_redaction_pattern(&field.value, pattern);
        }
    }
    record
    // 关键约束(§4.1明确要求)：本函数产出的RedactedRecord类型与serialize_for_client的WireResponse类型
    // 在类型系统层面即不同，编译期即可防止"脱敏后的数据被误用作客户端响应体"这一实现错误
}
```

### 4.2 审计留痕判定函数（对应RGS-BAS-023§4.2判定表）

```rust
fn should_audit(ctx: &RequestContext, result: &NormalizedResult) -> bool {
    if is_high_risk_operation(result.method_name()) {   // 复用RGS-BAS-003§8既定高危操作分类
        return true;   // 强制留痕，且走既定二次确认流程(若适用，二次确认本身不属于本函数职责)
    }
    if is_deterministic_request_path(result.method_name()) {   // FR-EC-003类确定请求路径
        return true;   // 强制留痕(价值发放类操作)
    }
    false   // 其余(普通只读查询)：不留痕，同RGS-BAS-004采样精神
}
```

该函数是§4.2判定表的唯一实现入口，管道后处理⑤阶段调用它，业务方法不得绕过管道自行调用审计写入接口——落实RGS-BAS-023§4.2"判定逻辑集中在管道后处理⑤阶段的一个判定组件"这一强制要求的具体方式是：审计写入接口本身在代码可见性上仅对管道内部模块开放（Rust `pub(crate)`级别），业务逻辑所在的crate无法直接引用，从语言机制层面而非仅约定层面阻止绕过。

---

## 5. 统一错误响应协议格式

对应RGS-BAS-023§5逻辑字段表：

```protobuf
message StandardErrorResponse {
  string error_code    = 1;   // 枚举取值：AUTH_FAILED｜RATE_LIMITED｜VALIDATION_FAILED｜INTERNAL_ERROR
  string message          = 2;   // 面向人类描述，不得包含未脱敏敏感信息(生成前经§4.1同款脱敏规则校验)
  repeated FieldError field_errors = 3;   // 可选，§3.2校验失败时的逐字段错误
  string trace_id                    = 4;   // 与RequestContext.trace_id同值，供客户端/GM后台按此检索链路
}
message FieldError {
  string field   = 1;
  string reason    = 2;
}
```

三引擎客户端SDK（RGS-REQ-012/BAS-008既定范围）对该结构的通用错误处理逻辑不属于本文档职责（本文档只固定服务端产出的协议格式契约本身），客户端侧具体实现留待RGS-BAS-008对应的DTL文档（若尚未存在）另行覆盖。

---

## 6. 脚手架生成物骨架代码

对应RGS-BAS-023§6.1生成内容表，落实为具体骨架代码模板（与RGS-DTL-002§2 Helm模板同一"挂载脚手架产物"性质，本节补充的是RGS-DTL-002未覆盖的**应用层**骨架代码，两者互补而非重复——RGS-DTL-002覆盖K8s/CI资源，本节覆盖Rust源码骨架）：

```rust
// scaffold-template/src/pipeline.rs（挂载脚手架生成，占位符<Context>由挂载脚本替换）
// 前处理⑤阶段与后处理⑤阶段默认接线完成，对应§6.1"生成物"表格
pub fn build_<context>_pipeline() -> impl Service<Request, Response = Response> {
    build_service_pipeline(<Context>BizService::default())
}

// 校验规则模板占位(对应§6.1"校验规则模板")，开发者在此为方法新增校验注解，
// 除本文件外的管道阶段代码对开发者不暴露修改入口(§6.2既定)
pub mod validation_rules {
    // 示例占位：pub const EXAMPLE_METHOD_REQUEST_ID_RULE: ValidationRule = ValidationRule { required: true, ... };
}

// 统一错误响应类型：直接引用§5既定结构，脚手架生成时import，不生成新定义(§6.1"不得由开发者重新定义")
pub use rgs_pipeline_common::StandardErrorResponse;
```

`<Context>BizService::default()`是唯一要求开发者填充的部分（RGS-BAS-023§6.2"业务逻辑本体"），其余骨架文件按§6.2定制点表格声明为不可修改区域，脚手架生成时可附加代码生成器注释标记（如`// GENERATED: DO NOT EDIT BELOW THIS LINE`）供代码评审工具据此自动识别越权修改，落实§7.2代码评审检查清单"定制点之外的管道代码未被修改"这一要求的可自动化部分。

---

## 7. 既有服务迁移的技术判定协议

对应RGS-BAS-023§6.3迁移策略中"不得允许半迁移状态"这一约束的可执行判定：

```rust
// 服务自检工具调用，判定该服务是否处于§6.3明确禁止的"半迁移"中间态
fn check_migration_state(service_layers: &ServiceLayerInventory) -> MigrationState {
    let standard_layers = [
        LayerKind::Trace, LayerKind::Auth, LayerKind::RateLimit,
        LayerKind::Validation, LayerKind::Idempotency,
        LayerKind::ResultNormalize, LayerKind::Serialize,
        LayerKind::Redact, LayerKind::Metrics, LayerKind::Audit,
    ];
    let adopted_count = standard_layers.iter().filter(|l| service_layers.uses_standard(**l)).count();
    match adopted_count {
        0 => MigrationState::NotMigrated,                 // 完全维持原实现,合法
        n if n == standard_layers.len() => MigrationState::FullyMigrated,   // 完全采用标准管道,合法
        _ => MigrationState::PartiallyMigrated,             // 半迁移，§6.3明确禁止的中间态
    }
}
```

CI流水线（复用RGS-DTL-002§4已确立的CI阶段模式，新增一个校验步骤而非另起一套CI机制）在既有服务的构建阶段调用此函数，`MigrationState::PartiallyMigrated`时构建失败并报告具体缺失的标准阶段，防止"只接入鉴权、未接入脱敏"这类难以判定实际行为的状态被静默合入主干。

---

## 8. 本文档的覆盖范围与后续计划

本文档覆盖：管道的`tower::Layer`组合实现伪代码（含前处理短路语义）、`RequestContext`具体结构体定义、输入校验规则的`.proto`自定义option扩展格式与执行逻辑、脱敏与序列化并行支线的类型级隔离实现、审计留痕判定函数（含访问控制层面的绕过防护）、统一错误响应协议格式、脚手架生成物的Rust骨架代码模板、既有服务"半迁移"状态的可执行判定协议。

本版本明确不覆盖、留待后续：

- `tower`生态之外的具体限流算法与令牌桶/滑动窗口参数——RGS-BAS-023§2.2标注为"复用NFR-SEC-008既有速率限制标准"，本文档只给出接入点（`RateLimitLayer`），具体算法与阈值参数不属于本文档范围，应由NFR-SEC-008对应的既有设计或其DTL文档覆盖。
- 既有服务迁移的具体排期表——RGS-BAS-023§6.3已明确"按各限界上下文自身排期，无统一强制截止时间"，本文档§7只给出技术判定协议（能否判定为已完成/半迁移），不代为制定排期计划。
- 管道自身性能开销的具体负载试验数值——RGS-BAS-023§7.1检查清单"符合既定阈值（TBD-PPL-001确定后回填）"标注为TBD，本文档不预先给出数值。
- RGS-REQ-012/RGS-BAS-008对应DTL文档尚未存在时，三引擎客户端SDK对§5统一错误响应结构的具体消费实现——留待该DTL文档产出时覆盖，本文档只固定服务端契约。

后续详细设计建议顺序：与RGS-DTL-001§12/RGS-DTL-022§7建议一致，本文档与RGS-DTL-022（弹性容量）、RGS-DTL-024（集群部署编排）三者互不阻塞，可并行推进；限流具体算法选型若需要独立设计，建议尽早明确归属（本文档不代为决定应归入哪份文档）。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-023§2.1 管道结构图 | §2 |
| RGS-BAS-023§2.2 各阶段职责与复用对象 | §2、§3、§4 |
| RGS-BAS-023§3.1 RequestContext字段表 | §3 |
| RGS-BAS-023§3.2 输入校验声明式定义 | §3.1 |
| RGS-BAS-023§4.1 脱敏与序列化顺序 | §4.1 |
| RGS-BAS-023§4.2 审计留痕判定表 | §4.2 |
| RGS-BAS-023§5 统一错误响应结构 | §5 |
| RGS-BAS-023§6.1 挂载脚手架生成内容扩展 | §6 |
| RGS-BAS-023§6.2 定制点 | §6 |
| RGS-BAS-023§6.3 既有服务迁移策略 | §7 |
| RGS-BAS-023§6.4 前处理阶段同步依赖边界 | §3（校验阶段仅结构化Schema校验的实现即体现该边界） |
| RGS-DTL-002（挂载脚手架物理落地，本文档§6前提依赖） | §6 |
