# 详细设计书（詳細設計書 / Detailed Design Document）

**分布式游戏服务器基础设施 RustGameServer**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DET-001 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-001 基本设计书（全章） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程（日文原标准） |
| 制定日 | 2026-08-15 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 修订内容 | 影响章节 |
|---|---|---|---|---|
| 0.1 | 2026-08-15 | 架构师 | 初版制定。填补基本设计书§1.1未分配的"函数签名・算法・模块内数据结构"设计层，覆盖GW／RT／SY／PL／EC五个子系统的模块级详细设计（完整颗粒度），MT／GD／EV／WF四个子系统维持模块划分级颗粒度（继承基本设计书§1.2声明）。新增错误码体系（本书首次决定）。发现EC侧`session_epoch`防护机制在基本设计书§5.4未落地存储位置，登记为ISS-012并给出本书暂定方案 | 全部 |
| 0.2 | 2026-08-15 | 架构师 | 应用户要求，明确"App群组"边界：§2.1由示意性crate划分改为正式决定的独立部署单元清单（不变更基本设计书§3.1／3.2部署拓扑）；新增§2.7跨App API互通设计（东西向gRPC/Protobuf维持不变，南北向HTTPS类接口新决定采用JSON并给出兼容性纪律）；新增§2.8并发控制总则（锁顺序、锁超时、死锁重试）、§2.9回滚与补偿总则、§2.10排他手段选用基准；§7.1标注复合操作的锁顺序引用；§10/§11/§12同步补充暂定参数、测试观点与追溯性。基本设计书与需求定义书未变更 | §2、§7.1、§10、§11、§12 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-15 | — |
| 评审（技术） | | | 与基本设计书的一致性；是否存在与RGS-IFS-001／RGS-DBS-001职责重叠 |
| 评审（业务） | | | 错误码体系是否覆盖全部业务规则（BZ-nnn） |
| 审批（负责人） | | | 本文档的基准化 |

> 本文档经审批后成为**实现阶段（PH-1以降各子系统开工时）的输入基准**。本文档**不包含**物理数据类型／索引／约束（DDL，见RGS-DBS-001）与字节级线路格式（量化精度・位打包布局・`.proto`字段编号，见RGS-IFS-001）。

---

## 目录

1. [前言](#1-前言)
2. [通用设计约定](#2-通用设计约定)
3. [GW：接入网关详细设计](#3-gw接入网关详细设计)
4. [RT：实时运行时详细设计](#4-rt实时运行时详细设计)
5. [SY：同步・AOI详细设计](#5-sy同步aoi详细设计)
6. [PL：玩家・账号详细设计](#6-pl玩家账号详细设计)
7. [EC：玩家经济详细设计](#7-ec玩家经济详细设计)
8. [MT／GD／EV／WF：模块划分（概要级）](#8-mtgdevwf模块划分概要级)
9. [错误码体系](#9-错误码体系)
10. [暂定参数一览](#10-暂定参数一览)
11. [单体测试观点](#11-单体测试观点)
12. [追溯性](#12-追溯性)

---

# 1. 前言

## 1.1 本文档的定位

基本设计书（RGS-BAS-001）§1.1将"详细设计书"应回答的问题定义为**代码级如何实现（函数签名、DDL、字节级线路格式）**，并将其中DDL与字节级线路格式分别移交**RGS-DBS-001（数据库设计书）**与**RGS-IFS-001（外部接口规格书）**——但**函数签名、内部算法、模块内数据结构、异常处理的具体实现**并未被分配给任何既定文档。本文档（RGS-DET-001）正是填补这一空白：

| 关注点 | 归属文档 | 本文档是否涉及 |
|---|---|---|
| 组件划分、组件间交互时序、逻辑数据模型、API字段级设计 | RGS-BAS-001（已完成） | 否，仅引用 |
| 模块内部结构、函数／方法签名、算法伪代码、内部数据结构（Rust层） | **本文档（RGS-DET-001）** | **是** |
| 物理数据类型・索引・约束・分区・DDL | RGS-DBS-001（PH-2另行制定） | 否 |
| 字节级线路格式（量化精度・位打包布局・`.proto`字段编号） | RGS-IFS-001（PH-1另行制定） | 否 |

本文档**不新增独立的设计ID体系**，沿用需求定义书（RGS-REQ-001）1.5.3节的ID体系（ARC/FR/NFR/DR/BZ/ST/ISS）与本文档§9新定义的错误码体系（该体系本身是本文档在§9.2范围内被明确授权决定的产出物，见RGS-BAS-001§9.2："具体错误码编号体系属详细设计范围"）。

**文档间优先级**：本文档与RGS-BAS-001记述矛盾时，**以RGS-BAS-001为准**——本文档是基本设计书的展开，不得变更基本设计书已确定的组件划分、交互时序或逻辑数据模型。若详细设计阶段发现基本设计书需要补充或修正（如§7.4所述epoch防护字段问题），**必须**通过登记ISS并在基本设计书下一修订版中吸收，不得在本文档中径自改写基本设计书的既有图表。

## 1.2 适用范围与颗粒度声明

本文档覆盖RGS-BAS-001§4已展开的全部子系统。颗粒度继承RGS-BAS-001§1.2的声明：

- **GW／RT／SY／PL／EC**（PH-1〜PH-4对应子系统）：给出**模块内函数签名级**详细设计（本文档§3〜§7）。
- **MT／GD／EV／WF**（PH-5／PH-6对应子系统）：仅给出**模块划分**，函数签名级详细设计留待该阶段开始前补充本文档（本文档§8）。

此颗粒度差异的理由与RGS-BAS-001§1.2相同：避免在ARC-014证据不足时对远期功能做过度设计（需求定义书PP-001）。

## 1.3 关联文档

| 文档编号 | 文档名 | 与本文档的分工 |
|---|---|---|
| RGS-REQ-001〜005 | 需求定义书及附件 | 本文档全部设计决定必须可追溯至ARC/FR/NFR/DR/BZ/ST/ISS |
| RGS-BAS-001 | 基本设计书 | 本文档的直接父文档，逐章展开 |
| RGS-DBS-001（待制定，PH-2） | 数据库设计书 | 本文档给出Rust侧的仓储（Repository）接口与逻辑字段引用；RGS-DBS-001给出DDL |
| RGS-IFS-001（待制定，PH-1） | 外部接口规格书 | 本文档给出量化前的逻辑接口（§5.5）；RGS-IFS-001给出字节布局 |
| RGS-TST-001（待制定，PH-2） | 试验计划书 | 依据本文档§11单体测试观点及§3〜§7各算法设计试验用例 |

## 1.4 记述规则

沿用需求定义书1.5.1节的强度用语（必须／应当／可以／不得）。本文档新增以下记述约定：

| 约定 | 说明 |
|---|---|
| 代码块语言标注为`rust` | 表示**示意性伪代码**，用于表达函数签名与算法结构，不代表最终可编译源码。字段类型使用Rust惯用类型（`u64`／`i64`／`Uuid`／`DateTime<Utc>`等），但具体使用的第三方crate版本与API细节不在本文档决定范围 |
| 字段命名 | 沿用RGS-BAS-001§6已定义的字段名（如`character_id`、`session_epoch`、`request_id`、`expected_version`），不重复造词 |
| `character_id` vs `player_id` | 沿用RGS-BAS-001 1.2版的澄清：`session_epoch`归属角色（Character），涉及运行时权威写入与经济确定请求的API**一律使用`character_id`**。需求定义书§5.3.2表格中FR-EC-003的输入项仍写作`player_id`，与RGS-BAS-001§4.5.1／§6.3.2不一致，本文档以RGS-BAS-001为准（其为需求定义书§5.3.2在此处的架构级澄清），该措辞差异建议在RGS-REQ-001下次修订时同步更正，本文档不代为修改需求定义书正文 |
| 暂定参数标注 | 本文档给出的数值型参数，除非明确注明"已按需求定义书确定"，否则均为**暂定初始值**，汇总于§10，并标注推导依据与复核期限 |

---

# 2. 通用设计约定

本章给出跨全部子系统的共通设计约定，避免在§3〜§8中重复定义。

## 2.1 App群组：独立部署单元的划分（正式决定）

RGS-BAS-001§3.1／§3.2已确定部署拓扑（哪些组件各自是独立的K8s Deployment／StatefulSet），本节在**不变更该拓扑**的前提下，将其落实为详细设计层面的**App边界决定**——即哪些功能点**必须**各自打包为独立的二进制／进程／crate，具备独立的构建、发布、扩缩容节奏，彼此之间**不共享内存、不共享数据库连接池、不越过API直接调用对方内部函数**。这是对RGS-BAS-001部署拓扑的细化落地，不是新的架构决定。

| App | 对应子系统 | 对应RGS-BAS-001部署单元 | 数据库归属 | 说明 |
|---|---|---|---|---|
| `rgs-gateway` | GW | Deployment，HPA（§3.2） | 无（会话状态在缓存基础设施，DR-003） | 独立App，§3 |
| `rgs-runtime` | RT＋SY | StatefulSet，不自动缩容（§3.2） | 无（DR-001实时状态，检查点写入独立存储） | SY（FR-SY-001〜009）**不是独立App**，是`rgs-runtime`进程内场景Actor tick循环的ECS System集合（RGS-BAS-001§4.2.1、本文档§5.0）。这是**唯一**未被拆分为独立App的子系统，原因是ARC-001要求同一场景状态在单一task内无锁访问，跨进程会引入本文档§2.9要严禁的分布式锁 |
| `rgs-player` | PL | Deployment，HPA（业务服务，§3.2） | `player_db`（DR-004排他所有） | 独立App |
| `rgs-economy` | EC | Deployment，HPA | `economy_db` | 独立App |
| `rgs-match` | MT | Deployment，HPA | `match_db` | 独立App，§8模块级颗粒度 |
| `rgs-social` | GD | Deployment，HPA | `social_db` | 独立App，§8模块级颗粒度 |
| `rgs-admin` | AD | Deployment，HPA | `admin_db` | 独立App，本版未展开函数签名 |
| `rgs-eventing` | EV | Deployment（Outbox分发器，FR-EV-001） | 无自有DB，读取各App的`outbox`表 | 独立App，§8模块级颗粒度 |
| `rgs-workflow` | WF | Deployment（购买Saga编排，FR-WF-001） | 工作流基础设施自身状态存储 | 独立App，§8模块级颗粒度 |

共用库（不是App，不独立部署，以依赖形式被上表各App引用）：

| 库 | 用途 |
|---|---|
| `rgs-common` | 共通类型：ID包装类型（§2.2）、错误基础设施（§2.3）、trace上下文（§2.6） |
| `rgs-observability` | 各App共用的trace/metrics/log初始化与传播工具 |

**App边界的约束性规则**（本节新增，落地本次详细设计要求）：

1. 上表每个App**必须**是独立的可执行二进制与独立的Cargo crate，**不得**将两个App的业务逻辑编译进同一二进制（`rgs-runtime`内含SY除外，因其非独立App，理由见上表）。
2. App间通信**只能**通过§2.8定义的API方式（gRPC或HTTPS），**不得**共享数据库连接、**不得**直接import对方crate的内部（非公开API）模块。
3. 每个App对自身数据库的访问**必须**通过自身进程内的连接池，池的上限**必须**可配置（呼应RGS-BAS-001§7.2），**不得**与其他App共享连接池实例——这是防止§2.9死锁分析可判定范围局限在单App、单数据库连接池内的前提。
4. 新增App或合并既有App，**必须**先确认是否影响RGS-BAS-001§3.1／§3.2的部署拓扑；若影响，须先修订RGS-BAS-001（本文档§1.1优先级规则复述）。本节仅是拓扑已确定前提下的细化，不创设新拓扑。

> 具体使用的第三方crate（HTTP框架、gRPC框架版本等）选型不在本文档决定范围，属实现阶段技术选型。

## 2.2 通用类型约定

```rust
// 逻辑标识类型。物理类型（是否UUID、是否有额外校验位）由RGS-DBS-001决定，
// 此处仅为表达函数签名的需要而给出Rust包装类型示意。
pub struct PlayerId(pub Uuid);
pub struct CharacterId(pub Uuid);
pub struct RequestId(pub Uuid);      // UUIDv7，RGS-BAS-001§6.1
pub struct SessionEpoch(pub i64);    // 单调递增，RGS-BAS-001§4.4.2
pub struct Version(pub i64);         // OCC版本号，DR-007
pub struct TraceId(pub String);      // W3C Trace Context，NFR-OP-002
```

## 2.3 错误处理约定

- 各模块定义**自身的错误枚举**（建议使用`thiserror`表达，具体crate选型不在本文档决定范围），错误枚举的每个变体**必须**映射至本文档§9错误码体系中的一个码值。
- 模块对外暴露的函数**必须**返回`Result<T, ModuleError>`，**不得**使用`panic!`表达业务错误或系统错误的正常路径（`panic!`仅用于不可恢复的编程错误，由RT监督机制处理，见§4.10）。
- 三种错误分类（RGS-BAS-001§9.1：业务错误／系统错误／基础设施错误）在类型系统上**应当**可区分（如通过错误枚举的顶层variant区分），使调用方能依分类决定是否重试。

## 2.4 有界队列约定（CON-008落地）

**全部**跨task／跨线程的消息传递**必须**使用有界（bounded）通道，**不得**使用无界（unbounded）通道。

```rust
// 正确：显式容量，容量来源于配置（§10暂定参数）
let (tx, rx) = tokio::sync::mpsc::channel::<SceneMessage>(config.mailbox_capacity);

// 禁止：CI静态检查（clippy自定义lint或grep规则）拒绝以下调用出现在源码中
// tokio::sync::mpsc::unbounded_channel(...)
```

CI**必须**包含一条针对`unbounded_channel`／`UnboundedSender`／`UnboundedReceiver`字面量出现的静态检查规则，检出即构建失败。

## 2.5 实时路径非阻塞约定（CON-007落地）

场景Actor tick循环内**不得**同步调用数据库、消息中间件、工作流引擎（对应RGS-BAS-001§4.2.1 `Out2`与§4.5.2）。凡tick循环内触发的外部调用，**必须**遵循以下模式：

```rust
// 反模式（禁止）：tick循环内 await 一个跨进程调用，阻塞至下一tick
async fn system_apply_reward_WRONG(economy_client: &EconomyClient, req: CommitRequest) {
    let _ = economy_client.commit_transaction(req).await; // 阻塞当前tick
}

// 正确模式：tick循环内仅"登记"，实际调用由独立task异步执行，
// 结果通过有界channel在后续tick被"轮询式"消费（不阻塞等待）
fn system_apply_reward(pending_tx: &mpsc::Sender<CommitRequest>, req: CommitRequest) {
    let _ = pending_tx.try_send(req); // 非阻塞发送，队满则按§4.14丢弃策略处理
}

// 独立task（scene actor创建时随之spawn，生命周期与scene actor一致）
async fn economy_dispatch_task(
    mut pending_rx: mpsc::Receiver<CommitRequest>,
    economy_client: EconomyClient,
    result_tx: mpsc::Sender<EconomyResult>,
) {
    while let Some(req) = pending_rx.recv().await {
        let outcome = economy_client.commit_transaction(req).await; // 阻塞发生在此task，不影响tick循环
        let _ = result_tx.send(outcome.into()).await;
    }
}
```

`result_tx`的接收端即为RGS-BAS-001§4.2.1 `SceneMessage::EconomyResult`，在下一次可用的tick被mailbox消费（见§4.1）。

## 2.6 时刻与追踪约定

- 全部持久化与日志时刻使用UTC（`DateTime<Utc>`），呼应NFR-EN-005。
- 跨组件边界的公开async函数**应当**标注`#[tracing::instrument]`（或等价的span传播机制），以满足NFR-OP-001/002对`trace_id`全路径传播的要求。本文档后续各函数签名省略该标注以保持简洁，但视为默认存在。

## 2.7 跨App API互通设计

本节是§2.1所划分各App之间**如何互相调用**的详细设计，不变更RGS-BAS-001§3.3／§7已确定的协议归属（东西向gRPC、南北向HTTPS、客户端实时通道QUIC），仅在RGS-BAS-001未规定序列化格式的层面（HTTPS类API的请求/响应体）给出本文档的决定。

### 2.7.1 协议与序列化格式（按接口分类）

| 接口 | 协议（RGS-BAS-001已定，不变） | 序列化格式 | 决定归属 |
|---|---|---|---|
| IF-001 客户端⇔GW（实时） | QUIC Datagram／Stream | 二进制量化/位打包 | 不变，字节布局属RGS-IFS-001 |
| IF-003 GW⇔RT、RT⇔业务App、业务App⇔业务App（东西向内部） | gRPC（tonic），mTLS | Protobuf（gRPC标准序列化） | 不变，`.proto`字段编号属RGS-IFS-001 |
| IF-002 客户端⇔业务API（经API网关，PH-6起） | HTTPS | **JSON**（本文档决定） | 本文档新决定 |
| IF-006 工作流⇔外部支付（Webhook） | HTTPS | **JSON**（本文档决定，签名字段随JSON体一同传输；签名**必须**基于原始请求体字节验证，**不得**先反序列化为JSON对象再重新序列化后计算/校验签名——JSON无规范的规范化(canonicalization)规则，重新序列化可能改变字段顺序/空白/数字格式，导致与支付服务商侧签名不一致。具体签名算法仍依赖ISS-007与服务商选型，RGS-BAS-001§6.4，PH-6前确定） | 本文档新决定（序列化格式），签名算法本身仍移交 |
| IF-007 运营工具⇔运营API | HTTPS，RBAC | **JSON**（本文档决定） | 本文档新决定 |

**结论**：App群组内部（东西向，App与App之间的调用，如RT调用EC的`CommitTransaction`）**保持RGS-BAS-001已定的gRPC/Protobuf不变**——这类调用在NFR-PE-008（p99<20ms）等严格延迟预算下，Protobuf的序列化开销与强类型契约仍是必要的。**JSON仅适用于面向HTTPS客户端/运营工具的南北向接口**，这些接口本身对延迟不敏感（NFR-PE-009 p99<200ms，NFR-PE-012 p99<3秒），JSON带来的可读性、调试便利性、与Web生态的兼容性收益大于其序列化开销。此划分与RGS-BAS-001§3.3的协议归属完全一致，不构成矛盾。

### 2.7.2 JSON API的兼容性纪律

Protobuf原生支持未知字段跳过与`optional`语义（NFR-OP-006／ARC-015 Expand-Contract天然受益），JSON不具备这类保障，**必须**由本节的编码纪律弥补：

| 规则 | 内容 |
|---|---|
| 只增不改 | 新增字段**必须**为可选（反序列化时缺省），**不得**复用已废弃字段的名称给新含义 |
| 未知字段容忍 | 反序列化器**必须**配置为忽略未识别字段（而非报错拒绝），供Expand阶段新旧版本混跑（RGS-BAS-001§7.4） |
| 字段类型不变 | 已发布字段的JSON类型（字符串／数字／布尔／数组／对象）**不得**变更，变更须走新字段名 |
| 版本号 | 每个JSON API响应体**必须**携带`schema_version`字段（复用DR-013 outbox表已用字段名），供NFR-OP-006版本占比监控 |

### 2.7.3 App间调用的客户端约定

App间发起的每一类外部调用（无论gRPC或HTTPS），**必须**遵循统一的客户端结构，而不是在各App内各自零散实现：

```rust
pub struct ApiClient<T> {
    inner: T,                      // 具体传输层客户端（tonic生成的gRPC stub，或HTTP客户端）
    pool: ConnectionPool,          // 每App独立连接池，池上限见§10（呼应§2.1规则3）
    timeout: Duration,             // 单次调用超时，按被调App的NFR响应时间预算设定，见§10
    retry_policy: RetryPolicy,     // 仅对"幂等操作"生效，见下
}

pub enum RetryPolicy {
    /// 业务拒绝（如余额不足、版本冲突已达上限）：不重试，直接向上返回§9错误码
    NoRetry,
    /// 瞬时基础设施错误（超时/不可达）：有界重试，退避策略与次数见§10，
    /// 重试**必须**携带与首次调用相同的`request_id`（幂等前提，ARC-009）
    BoundedRetry { max_attempts: u32, backoff: BackoffPolicy },
}
```

**分类判定规则**（决定一次失败该不该重试）：错误码分类（§9：业务错误／系统错误／基础设施错误）直接决定`RetryPolicy`分支——业务错误**不得**重试（重试不会改变结果，且可能违反BZ-nnn语义），系统错误／基础设施错误**可以**有界重试。这一判定规则同时适用于gRPC与HTTPS两类调用，不因序列化格式而异。

## 2.8 并发控制总则（锁顺序・超时・死锁重试）

本节给出跨全部App通用的并发控制规则，适用范围是**单个App自身数据库内、涉及多行或多聚合根的同一事务**。跨App场景**不使用**本节的锁机制，而是§2.10表格给出的防护令牌机制。

| 规则 | 内容 | 适用示例 |
|---|---|---|
| **锁顺序规则（单一复合排序键，易错点见下方说明）** | 同一事务需要锁定多行/多表时，**必须**按以下单一复合键的升序依次加锁，**不得**将"表优先"与"主键优先"当成两条独立规则分别套用：<br>①先锁定本次涉及的**全部**`character_epoch_fence`行（先按`character_id`字节序排序，逐一锁定），使旧epoch请求能尽早失败，避免对业务表做无谓加锁；<br>②再锁定其余业务行，排序键为`(character_id字节序, 表名字典序)`——即先按角色分组（`character_id`升序），同一角色内部再按表名（如`inventory`先于`wallet`）升序 | §7.1 `commit_transaction`同时涉及Inventory与Wallet时；FR-EC-008玩家间交易（BZ-007原子双向转移，两个角色X<Y）的加锁顺序为：`fence(X), fence(Y), inventory(X), wallet(X), inventory(Y), wallet(Y)`——无论请求方向是X→Y还是Y→X，均产生同一物理顺序，不形成互等环路 |
| **排序键必须是字节序，不得是文本字典序（易错点）** | `character_id`为`Uuid`（§2.2），比较**必须**取其16字节二进制表示（`as_bytes()`）升序，**不得**取`to_string()`的文本字典序——两者对同一组UUID可能给出不同顺序（连字符位置、大小写字母都会影响文本排序），若应用层用文本序排序、而数据库原生`uuid`类型比较用的是字节序，两条并发事务可能各自"认为"自己遵守了顺序规则，实际却选择了相反的物理加锁顺序，规则形同虚设 | 全部涉及`character_id`排序的加锁场景 |
| **锁等待超时** | 数据库会话级设置`lock_timeout`上限（暂定值见§10，须明显小于§2.7.3调用方设置的超时，否则调用方会先于数据库放弃等待，导致锁超时这一"快速失败重试"机制永远无法触发），超时视为本次尝试失败，按§7.1已建立的"整个尝试作为一次性事务重开"模式处理，**不得**无界等待挤占连接池 | 全部持锁事务 |
| **死锁检测与重试** | PostgreSQL死锁检测返回的错误码（`40P01`）**必须**归类为"可重试的系统错误"而非业务错误，复用§7.1的重试结构（全新事务、有界次数，具体次数见§10） | 全部持锁事务 |
| **禁止跨`.await`持锁** | 进程内共享可变状态**优先**使用无锁并发结构（如§3.3会话表已用的并发映射），确需显式锁（`Mutex`等）时**不得**在持锁临界区跨越一次`.await`——临界区内只做同步操作 | GW `SessionManager`、RT `SceneSupervisor.crash_history`等进程内共享状态 |

## 2.9 回滚与补偿总则

| 失败层级 | 处理机制 | 依据 |
|---|---|---|
| 单App单事务内失败（含§2.8死锁/锁超时） | 直接`ROLLBACK`，无需额外补偿逻辑——本事务尚未提交，天然原子撤销 | 已是§6/§7各`commit_transaction`默认行为 |
| 跨App调用失败，且下游操作幂等 | 幂等重试（复用同一`request_id`），不引入补偿 | ARC-009 |
| 跨App调用失败，下游是不可逆副作用（如已扣款）且重试无法收敛 | 具名、幂等的补偿Activity（如`RefundActivity`），走工作流基础设施 | ARC-011、RGS-BAS-001§4.7.2 |
| 跨App调用失败，下游仅为复制/通知类操作 | 不补偿，任其自然过期或被下次全量同步覆盖 | DR-001可容忍丢失 |

**禁止事项**：**不得**在跨App调用失败时，通过"反向调用对方另一个正向API"来模拟事务回滚——这是未经隔离性保证的手写分布式事务，正是CON-004禁止2PC所要防止的反模式。跨App的撤销**必须**是具名的补偿Activity，不得复用正向API反向调用充当撤销。

## 2.10 排他（互斥）手段选用基准

| 场景 | 推荐手段 | 不得使用 |
|---|---|---|
| 单App单事务内、单聚合根写冲突（如Wallet余额更新） | 乐观并发控制（`version`列，DR-007／008），§7.1默认路径 | 悲观行锁作为默认路径（无冲突证据前不引入额外锁等待，NFR-PE-008预算紧张） |
| 单App单事务内、需要先确认某标识的最新状态（如epoch防护判定） | 短临界区悲观行锁（`SELECT…FOR UPDATE`），§7.2 | 持锁跨越网络调用或跨越tick循环 |
| 单App单事务内、涉及多个聚合根或多个角色 | §2.8固定锁顺序＋悲观行锁 | 不固定顺序的多行加锁 |
| 单App进程内、跨task共享可变状态 | 无锁并发结构（§3.3同款） | 跨App共享该结构；持锁跨`.await`（§2.8） |
| 跨App、跨数据库的互斥需求（如"同一角色不能同时被两个连接写入"） | 防护令牌（Fencing Token，ARC-005／`session_epoch`／§7.2`CHARACTER_EPOCH_FENCE`）——用单调标识"拒绝旧写入者的结果"，而非用锁"阻止旧写入者行动" | 引入分布式锁中间件（未过ARC-014判定基准；ARC-012已声明缓存不得作为仲裁者） |

**总原则**：本项目**不使用**跨App/跨库的分布式锁。全部跨App互斥效果，由（a）单App内的OCC/短临界区行锁与（b）防护令牌两种机制之一实现，两者均已在本文档给出具体设计（§7.1、§7.2），本节仅是将其提升为全App群组通用的选用基准，供PH-5以降新增App（MT/GD/EV/WF）设计涉及并发写入的功能时遵循。

---

# 3. GW：接入网关详细设计

对应RGS-BAS-001§4.1（模块构成表见§4.1.1）。

## 3.1 连接终结模块

```rust
pub struct ConnectionTerminator {
    endpoint: QuicEndpointHandle, // 具体QUIC实现（quinn／s2n-quic选型不在本文档范围）
}

impl ConnectionTerminator {
    /// 接受一个新的QUIC连接尝试（TLS 1.3握手在此完成，NFR-SE-002）
    pub async fn accept(&self) -> Result<IncomingConnection, GwError>;

    /// 握手完成后返回可用于后续鉴权流程的连接句柄
    pub async fn complete_handshake(&self, incoming: IncomingConnection)
        -> Result<ClientConnection, GwError>;
}

pub struct ClientConnection {
    pub connection_id: ConnectionId, // GW内部标识，网关生命周期内唯一，不在API字段中对外暴露
    pub established_at: DateTime<Utc>,
}
```

## 3.2 鉴权模块

对应RGS-BAS-001§4.1.2时序图。

```rust
pub struct AuthModule {
    player_client: PlayerServiceClient, // 对应RGS-BAS-001§6.3.1 PlayerService
}

impl AuthModule {
    /// 实现RGS-BAS-001§4.1.2全部步骤：调用PlayerService.Authenticate，
    /// 成功后建立会话记录（转交SessionManager），失败则不建立
    pub async fn authenticate(
        &self,
        conn: &ClientConnection,
        token: AuthToken,
    ) -> Result<AuthOutcome, GwError> {
        // ①调用PlayerService.Authenticate（RGS-BAS-001§6.3.1）
        // 网络层瞬时错误（deadline exceeded／unavailable）允许1次重试，
        // 重试预算须控制在NFR-PE-012（登录处理p99<3秒）之内；
        // 业务拒绝（凭证无效／BZ-006封禁）不重试。
        // ②成功：返回player_id与character_list；失败：返回原因码，连接由调用方断开
        unimplemented!()
    }
}

pub enum AuthOutcome {
    Success { player_id: PlayerId, characters: Vec<CharacterSummary> },
    Rejected { reason: AuthRejectReason }, // 对应E-GW-1xxx，见§9
}
```

## 3.3 会话管理模块

对应FR-GW-003。会话表为网关进程内的并发映射，键为`ConnectionId`。

```rust
pub struct SessionEntry {
    pub character_id: Option<CharacterId>, // 账号鉴权完成但未选择角色时为None
    pub session_epoch: Option<SessionEpoch>,
    pub current_scene_id: Option<SceneId>,
    pub last_heartbeat_at: DateTime<Utc>,
}

pub struct SessionManager {
    sessions: DashMapLike<ConnectionId, SessionEntry>, // 并发哈希表，具体crate不在本文档范围
    heartbeat_interval: Duration, // 暂定值见§10
    heartbeat_timeout: Duration,  // 暂定值见§10
}

impl SessionManager {
    pub fn register(&self, conn_id: ConnectionId, entry: SessionEntry);
    pub fn touch_heartbeat(&self, conn_id: ConnectionId);

    /// 周期性扫描任务（独立task，非阻塞GW主接收循环）：
    /// 对`now - last_heartbeat_at > heartbeat_timeout`的会话触发断线处理（FR-GW-003）
    pub async fn heartbeat_sweep_loop(&self);
}
```

## 3.4 路由模块

对应FR-GW-004，实现RGS-BAS-001§3.5缓存优先、权威回退的路由查询。

```rust
pub struct RoutingModule {
    scene_location_cache: CacheClient,       // 缓存基础设施客户端（ARC-012：仅性能优化路径）
    player_service: PlayerServiceClient,     // 权威回退查询
}

impl RoutingModule {
    /// 实现RGS-BAS-001§3.5时序图的"缓存命中/未命中"分支
    pub async fn resolve_scene_route(&self, scene_id: SceneId) -> Result<NodeAddr, GwError> {
        if let Some(addr) = self.scene_location_cache.get(scene_id).await {
            return Ok(addr); // 缓存命中，性能路径
        }
        let addr = self.player_service.query_scene_assignment(scene_id).await?; // 权威回退
        self.scene_location_cache.set(scene_id, addr.clone()).await; // 回填
        Ok(addr)
    }

    /// 对应RGS-BAS-001§4.1.4步骤3〜4：转发至运行时节点。
    /// mailbox已满（背压）时**不重试**，直接丢弃并返回，
    /// 由客户端下一次输入自然覆盖（ARC-013，高频状态可替代性）
    pub async fn forward_input(
        &self,
        target: &NodeAddr,
        msg: SceneMessage,
    ) -> Result<(), GwError> {
        match self.send_to_runtime(target, msg).await {
            Err(RuntimeSendError::MailboxFull) => Ok(()), // 静默丢弃，非错误
            other => other.map_err(Into::into),
        }
    }
}
```

## 3.5 限流模块

对应FR-GW-006。采用令牌桶算法，每连接独立计量。

```rust
pub struct RateLimiter {
    capacity: u32,      // 暂定值见§10
    refill_per_sec: u32,// 暂定值见§10
}

pub struct TokenBucket {
    tokens: f64,
    last_refill_at: Instant,
}

impl RateLimiter {
    /// O(1)算法：按经过时间线性补充令牌，扣减1枚令牌表示放行1条输入。
    /// 令牌不足时**丢弃当前输入、不断开连接**（RGS-BAS-001§4.1.4步骤2），
    /// 除非在NFR-SE-008窗口内持续超限（转交§3.6之外的账号级限流，PH-4范围）
    pub fn check_and_consume(&self, bucket: &mut TokenBucket) -> RateLimitDecision {
        let elapsed = bucket.last_refill_at.elapsed().as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_sec as f64)
            .min(self.capacity as f64);
        bucket.last_refill_at = Instant::now();
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            RateLimitDecision::Allow
        } else {
            RateLimitDecision::Throttle
        }
    }
}
```

## 3.6 排空控制模块

对应FR-GW-009。状态机：`Running → Draining → Terminated`。

```rust
pub enum GwLifecycleState { Running, Draining, Terminated }

pub struct DrainController {
    state: AtomicLifecycleState,
    drain_timeout: Duration, // 暂定值见§10
}

impl DrainController {
    /// 收到SIGTERM时调用。
    /// ①切换状态为Draining：readinessProbe即刻失败（不再接受新连接），
    ///   livenessProbe维持通过（进程未死锁）——呼应RGS-BAS-001§7.1
    /// ②既有连接不主动断开，等待其自然结束（客户端登出／场景转移／心跳超时）
    /// ③超过drain_timeout仍有残留连接时强制断开，触发客户端重连（§3.7）
    pub async fn on_sigterm(&self) {
        self.state.store(GwLifecycleState::Draining);
        // readiness探针読取该state即可实现①，无需额外通知逻辑
        // 等待"全部连接已自然结束"与"超时"两者之一先发生，
        // 不得无条件睡满drain_timeout——否则残留连接早已清空时仍会白白拖满整个超时时长，
        // 与NFR-AV-007"无停机滚动更新"的意图相悖（排空应尽快完成，超时只是兜底上限）
        tokio::select! {
            _ = self.wait_until_no_remaining_connections() => {}
            _ = tokio::time::sleep(self.drain_timeout) => {
                self.force_close_remaining().await;
            }
        }
        self.state.store(GwLifecycleState::Terminated);
    }
}
```

## 3.7 连接建立与鉴权处理流程（细化RGS-BAS-001§4.1.2）

| 步骤 | 调用 | 失败时行为 |
|---|---|---|
| 1 | `ConnectionTerminator::accept` → `complete_handshake` | TLS握手失败：连接层直接拒绝，不产生会话记录 |
| 2 | 客户端在首个Stream发送鉴权令牌 | 格式错误：`E-GW-1001` |
| 3 | `AuthModule::authenticate` | 凭证无效／封禁：`E-GW-1002`／`E-GW-1003`（BZ-006），断开连接 |
| 4 | `SessionManager::register` | — |
| 5 | 向客户端返回`SessionHandshake`响应（RGS-BAS-001§6.2.2） | — |

## 3.8 重连处理流程（细化RGS-BAS-001§4.1.3、§3.5）

重连与首次连接共享步骤1〜3，差异点：

```rust
impl AuthModule {
    /// 重连专用路径：token验证成功后，必须调用PlayerService.SelectCharacter
    /// 重新发行epoch（RGS-BAS-001§4.4.2），而非复用旧epoch。
    /// 新epoch发行完成前，禁止执行到运行时的路由绑定（RGS-BAS-001§4.1.3约束）。
    pub async fn reauthenticate(
        &self,
        conn: &ClientConnection,
        token: AuthToken,
        character_id: CharacterId,
    ) -> Result<ReauthOutcome, GwError> {
        let auth = self.verify_token(token).await?;
        let epoch = self.player_client.select_character(auth.player_id, character_id).await?; // 新epoch
        // 此后才允许RoutingModule::forward_input绑定该连接至运行时（含旧epoch连接失效，由RT侧§4.1校验实现）
        Ok(ReauthOutcome { new_epoch: epoch, ... })
    }
}
```

---

# 4. RT：实时运行时详细设计

对应RGS-BAS-001§4.2。本章是全书篇幅最大的部分，因场景Actor是ARC-001的核心落地对象。

## 4.0 SY与RT的部署关系（澄清）

RGS-BAS-001§4.3（SY：同步・AOI）在功能需求编号上是独立子系统（FR-SY-001〜009），但**不是独立部署单元**——RGS-BAS-001§3.1／§3.2的部署拓扑中不存在"SY节点池"，AOI更新与复制生成是场景Actor tick循环内的System（RGS-BAS-001§4.2.1图中的`S4`／`S5`）。因此SY的详细设计（本文档§5）与RT共享同一进程、同一`rgs_runtime`模块，仅在职责上分节描述。

## 4.1 场景Actor整体结构

```rust
pub struct SceneActor {
    scene_id: SceneId,
    world: EcsWorld,               // ECS实体存储，具体crate（如bevy_ecs）不在本文档强制范围，但已列入RGS-REQ-005 OSS许可表
    mailbox: mpsc::Receiver<SceneMessage>, // 有界，容量见§10（CON-008）
    tick_interval: Duration,       // 50ms，NFR-PE-001
    current_tick: TickId,
    config: arc_swap::ArcSwap<SceneConfigTable>, // 数值表，ARC-016原子切换
    checkpoint_state: CheckpointScheduler, // §4.9
}

pub enum SceneMessage {
    PlayerInput { character_id: CharacterId, session_epoch: SessionEpoch, input: PlayerInputMessage },
    PlayerJoin { character_id: CharacterId, session_epoch: SessionEpoch },
    PlayerLeave { character_id: CharacterId },
    AdjacentSceneEvent { payload: CrossSceneEvent }, // FR-RT-008场景间转移用
    EconomyResult { request_id: RequestId, outcome: EconomyOutcome }, // §2.5非阻塞回调
    Shutdown, // 排空用，§4.13
}
```

`SceneActor::run`是场景Actor的唯一入口，**必须**运行于单一Tokio task内（ARC-001，RGS-BAS-001§4.2.1约束复述），场景状态**不得**跨task共享可变引用（不引入锁）。

```rust
impl SceneActor {
    pub async fn run(mut self) {
        let mut interval = tokio::time::interval(self.tick_interval);
        loop {
            interval.tick().await;
            let tick_start = Instant::now();

            // 非阻塞排空mailbox（上限N条/tick，防止单tick内消息风暴挤占模拟预算）
            let messages = self.drain_mailbox_nonblocking(MAX_MESSAGES_PER_TICK);
            if messages.iter().any(|m| matches!(m, SceneMessage::Shutdown)) {
                self.handle_shutdown().await; // §4.13
                return;
            }

            self.system_apply_input(&messages);          // S1，§4.4
            self.system_simulate_movement();              // S2，§4.5
            self.system_simulate_combat();                // S3，§4.6
            self.system_update_aoi();                      // S4，§5.1
            self.system_generate_replication();             // S5，§5.3

            if self.checkpoint_state.due(self.current_tick) {
                self.system_checkpoint_async();             // §4.9，非阻塞
            }

            self.current_tick.advance();
            record_tick_duration_metric(tick_start.elapsed()); // NFR-OP-003
        }
    }
}
```

## 4.2 tick循环预算分配（细化RGS-BAS-001§4.2.2）

NFR-PE-002规定tick处理时间p99须<25ms（周期50ms的50%）。将RGS-BAS-001§4.2.2给出的百分比套用至该25ms预算，得到各阶段的推导上限（**推导自已批准的NFR数值，非新决定**）：

| 阶段 | 占比（RGS-BAS-001§4.2.2） | 推导上限（25ms×占比） |
|---|---|---|
| 输入应用（S1） | 20% | 5.00ms |
| 移动模拟（S2） | 25% | 6.25ms |
| 战斗模拟（S3） | 25% | 6.25ms |
| AOI更新（S4） | 15% | 3.75ms |
| 复制生成（S5） | 15% | 3.75ms |

各阶段**应当**记录独立耗时指标（NFR-OP-003），供PH-2实测后调整占比或总预算本身（本表数值属§10暂定参数）。

## 4.3 ECS组件与实体分类（细化FR-RT-002）

| 组件 | 字段 | 适用实体 |
|---|---|---|
| `Position` | `x, y, z: f32` | 玩家／NPC／投射物 |
| `Velocity` | `vx, vy, vz: f32` | 同上 |
| `Health` | `current, max: i32` | 玩家／NPC |
| `AnimState` | `state: AnimStateId` | 玩家／NPC |
| `PlayerLink` | `character_id, session_epoch` | 仅玩家实体，将ECS实体与永久身份关联 |
| `AoIState` | `cell: CellCoord, interest_set: HashSet<EntityId>` | 玩家（AOI计算的主体，§5.1） |
| `ReplicationMeta` | `last_sent_tick: TickId, priority_hint: f32` | 全部需要复制的实体 |
| `Cooldowns` | `map: HashMap<SkillId, Instant>` | 玩家／NPC |

实体分类（`EntityKind::Player／Npc／Projectile`）决定哪些System对其生效（如`PlayerLink`仅玩家持有，`Cooldowns`仅可释放技能者持有）。

## 4.4 输入应用（S1，细化FR-RT-004）

```rust
pub struct InputBuffer {
    pending: VecDeque<PlayerInputMessage>, // 按sequence_no排序
    last_applied_seq: u64,
}

impl InputBuffer {
    /// 乱序与重复排除算法：
    /// - sequence_no <= last_applied_seq：视为重复／过期，丢弃
    /// - sequence_no在合理窗口内（未来N个序号以内）：按序号排序后插入
    /// - sequence_no超出未来窗口：视为异常（可能是重放攻击，NFR-SE-007），丢弃并计入FR-AD-004异常计数
    pub fn ingest(&mut self, input: PlayerInputMessage) -> Result<(), RtError>;

    /// 每tick调用一次：按序号顺序取出全部可应用的输入
    pub fn drain_applicable(&mut self) -> Vec<PlayerInputMessage>;
}
```

物理合法性校验（NFR-SE-001，RGS-BAS-001§7.3输入校验分层的"运行时层"）在应用输入时执行：速度不得超过实体上限、位置不得越过场景边界，超限值**不得**采纳（钳制或拒绝，不信任客户端申告值，BZ-004）。

## 4.5 移动模拟（S2，细化FR-RT-005）

```rust
fn system_simulate_movement(world: &mut EcsWorld, config: &SceneConfigTable) {
    for (position, velocity, ..) in world.query_movable_entities() {
        let clamped_velocity = clamp_to_max_speed(velocity, config.max_speed); // NFR-SE-001
        let proposed = position + clamped_velocity * TICK_DELTA_SECONDS;
        if !collides_with_static_geometry(proposed) {
            *position = proposed;
        }
        // 碰撞检测的具体算法（网格/包围盒）由实现阶段选型，此处仅定义接口边界
    }
}
```

## 4.6 战斗模拟（S3，细化FR-RT-006）

```rust
fn system_simulate_combat(world: &mut EcsWorld, pending_economy_tx: &mpsc::Sender<CommitRequest>) {
    for skill_use in world.drain_pending_skill_uses() {
        if !is_off_cooldown(&skill_use) { continue; } // 业务拒绝，不产生副作用
        let targets = resolve_targets(&skill_use); // 服务器权威判定，NFR-SE-001
        for target in targets {
            apply_damage(target, compute_damage(&skill_use, target));
            if is_dead(target) {
                handle_death(target);
                if let Some(drop) = roll_drop(target) {
                    // 掉落=永久事实变更，必须经确定请求API，且不得阻塞当前tick（§2.5）
                    let _ = pending_economy_tx.try_send(build_commit_request(drop));
                }
            }
        }
    }
}
```

## 4.7 延迟补偿（FR-RT-007，○优先级，PH-3）

维护每场景最近N个tick的实体位置历史环形缓冲区（`HistoryRingBuffer<TickId, Vec<(EntityId, Position)>>`），判定命中时按客户端声明的观测时刻回溯查找对应tick的历史位置。N的具体值属§10暂定参数。本功能为Should级，PH-3阶段视资源决定是否实现。

## 4.8 场景间转移（细化FR-RT-008）

```rust
impl SceneActor {
    /// 源场景侧：移除实体，确定检查点后，向目标场景发送AdjacentSceneEvent携带实体快照
    async fn system_transition_out(&mut self, character_id: CharacterId, target_scene: SceneId);
    /// 目标场景侧：接收AdjacentSceneEvent后在世界中重建实体
    fn system_transition_in(&mut self, event: CrossSceneEvent);
}
```

对客户端下发`SceneTransitionCommand`（RGS-BAS-001§6.2.2）触发客户端侧的场景切换。

## 4.9 检查点（细化FR-RT-009）

检查点用于恢复DR-001实时状态（RPO目标30秒，NFR-AV-006），**不是**永久事实的权威来源（永久事实仍以EC/PL等业务服务的PostgreSQL为权威）。

```rust
pub struct CheckpointScheduler {
    interval_ticks: u32, // 由暂定周期（ISS-010，暂定30秒）÷tick周期(50ms)换算，见§10
    last_checkpoint_tick: TickId,
}

impl SceneActor {
    /// 非阻塞：仅在独立task内执行序列化与落盘/落库，tick循环本身不await该结果（§2.5同款模式）
    fn system_checkpoint_async(&mut self) {
        let snapshot = SceneSnapshot::capture(&self.world, self.current_tick);
        let store = self.checkpoint_store.clone();
        tokio::spawn(async move {
            if let Err(e) = store.save(snapshot).await {
                tracing::error!(error = ?e, "checkpoint save failed"); // NFR-OP-005告警
            }
        });
    }
}
```

保留代数**已由需求定义书§6.5确定为"最新3代，世代管理"**（非本文档暂定值），检查点存储的物理形态（文件／对象存储／数据库表）不在本文档决定范围。

## 4.10 Actor监督与恢复（细化FR-RT-010，RGS-BAS-001§4.2.3）

```rust
pub struct SceneSupervisor {
    scenes: HashMap<SceneId, JoinHandle<()>>,
    crash_history: HashMap<SceneId, VecDeque<Instant>>, // 用于判定"持续性panic"
}

impl SceneSupervisor {
    /// 每当被监督的task结束（正常关闭或panic）时触发
    async fn on_scene_task_exit(&mut self, scene_id: SceneId, exit: TaskExit) {
        match exit {
            TaskExit::Panic(_) => {
                self.record_crash(scene_id);
                if self.is_recoverable(scene_id) {
                    self.restart_from_checkpoint(scene_id).await; // 从最新检查点重建
                } else {
                    self.mark_unavailable(scene_id); // NFR-OP-005告警，等待人工介入
                }
            }
            TaskExit::Graceful => { /* 排空流程正常结束，无需处理 */ }
        }
    }

    /// 判定基准（暂定，§10）：滑动窗口内（如5分钟）崩溃次数超过阈值（如3次）视为持续性，不再自动恢复
    fn is_recoverable(&self, scene_id: SceneId) -> bool;
}
```

## 4.11 场景容量限制（细化FR-RT-013）

软上限300／硬上限500**已由NFR-PE-016确定**（非本文档暂定值）。

```rust
impl SceneActor {
    fn can_accept_new_player(&self) -> CapacityDecision {
        let count = self.world.player_entity_count();
        if count >= HARD_CAP_500 { CapacityDecision::Reject }
        else if count >= SOFT_CAP_300 { CapacityDecision::RouteToSiblingOrQueue } // FR-RT-013
        else { CapacityDecision::Accept }
    }
}
```

## 4.12 数值表热更新原子切换（细化FR-RT-014，ARC-016）

```rust
impl SceneActor {
    /// 可在任意时刻由外部（配置中心推送）调用，仅"提交"新版本，不立即生效
    pub fn stage_config_update(&self, new_config: Arc<SceneConfigTable>) {
        self.pending_config.store(Some(new_config));
    }
}

// 在run()的tick循环最开始处（interval.tick().await之后、任何System执行之前）：
if let Some(pending) = self.pending_config.swap(None) {
    self.config.store(pending); // 原子替换，本tick内全部System读取到同一版本
}
```

此实现保证RGS-BAS-001§4.2.2的约束："同一tick内所有System必须看到同一版本"。

## 4.13 Actor排空流程（细化FR-RT-012，RGS-BAS-001§4.2.4）

```rust
impl SceneActor {
    async fn handle_shutdown(&mut self) {
        self.stop_accepting_new_players();                         // 步骤2
        self.notify_players_pending_maintenance().await;            // 步骤3（发送提示，可靠通道）
        self.system_checkpoint_async();                             // 步骤3（确定检查点）
        self.wait_for_checkpoint_ack().await;                       // 确保步骤4前检查点已落地
        self.mark_scene_pending_reassignment().await;               // 步骤4（缓存基础设施失效）
        // 步骤5〜6由客户端重连流程（§3.8）与SceneSupervisor共同完成，本函数返回后task结束
    }
}
```

## 4.14 mailbox容量与drain超时（推导说明）

mailbox容量：以NFR-PE-016软上限300玩家为基准，预留2个tick的突发缓冲（每玩家每tick至多1条输入消息的保守估计），推导值为300×2=600，向上取2的幂次得到**暂定容量1024**（§10）。

---

# 5. SY：同步・AOI详细设计

对应RGS-BAS-001§4.3。如§4.0所述，本章内容运行于RT场景Actor的S4/S5阶段内。

## 5.1 AOI网格算法（细化FR-SY-001，RGS-BAS-001§4.3.1）

```rust
pub struct AoiGrid {
    cell_size: f32, // 暂定值，待ISS-009决议，见§10
    cells: HashMap<CellCoord, Vec<EntityId>>,
}

impl AoiGrid {
    fn cell_of(&self, pos: &Position) -> CellCoord {
        CellCoord((pos.x / self.cell_size).floor() as i32, (pos.y / self.cell_size).floor() as i32)
    }

    /// 每tick仅对"所在格子发生变化"的实体做增量更新，而非全量重建（性能优化，服务于§4.2 AOI更新阶段3.75ms预算）
    fn update(&mut self, world: &EcsWorld) {
        for (entity_id, pos) in world.query_positioned_entities() {
            let new_cell = self.cell_of(pos);
            if world.get_cached_cell(entity_id) != Some(new_cell) {
                self.move_entity_cell(entity_id, new_cell);
            }
        }
    }

    /// 兴趣集合＝玩家所在格子＋周围8格（3×3邻域），RGS-BAS-001§4.3.1节点C
    fn compute_interest_set(&self, player_cell: CellCoord) -> HashSet<EntityId> {
        neighbors_3x3(player_cell).flat_map(|c| self.cells.get(&c).into_iter().flatten().copied()).collect()
    }
}
```

**格子大小与视野距离的确定基准**（ISS-009）：3×3邻域需完整覆盖玩家视野半径，故格子边长**应当**约等于视野距离本身（半径R的视野落在中心格＋周围8格构成的3R×3R区域内即可保证不漏判）。ISS-009最终决议前，本文档给出PH-2开发用暂定值，见§10。

## 5.2 视野事件判定（细化FR-SY-002）

```rust
fn diff_interest_sets(prev: &HashSet<EntityId>, curr: &HashSet<EntityId>) -> (Vec<EntityId>, Vec<EntityId>) {
    let entered: Vec<_> = curr.difference(prev).copied().collect(); // 触发enter_view（RGS-BAS-001§6.2.1）
    let left: Vec<_> = prev.difference(curr).copied().collect();     // 触发leave_view
    (entered, left)
}
```

## 5.3 差分快照生成与基线管理（细化FR-SY-003／004，RGS-BAS-001§4.3.2）

```rust
pub struct ClientReplicationState {
    baseline_tick: TickId,     // 客户端已确认的最新基线
    acked_tick: TickId,
}

impl SceneActor {
    fn system_generate_replication(&mut self) {
        for (character_id, repl_state) in self.replication_states.iter_mut() {
            let interest_set = self.aoi.compute_interest_set(/* 该玩家所在格 */);
            let (entered, left) = diff_interest_sets(&repl_state.prev_interest_set, &interest_set);
            let updates = self.select_within_budget(&interest_set, repl_state); // §5.4带宽预算裁剪
            let snapshot = StateDeltaSnapshot {
                base_tick: repl_state.baseline_tick,
                current_tick: self.current_tick,
                entity_updates: updates,
                enter_view: entered,
                leave_view: left,
            };
            self.send_datagram(character_id, snapshot); // 不可靠通道，RGS-BAS-001§6.2.1
            repl_state.prev_interest_set = interest_set;
        }
    }

    /// 收到客户端ACK时调用：基线单调推进，不回退（丢包时基线不前进，下次仍相对旧基线计算差分，自愈）
    fn on_input_ack(&mut self, character_id: CharacterId, last_processed_tick: TickId) {
        let state = self.replication_states.get_mut(&character_id).unwrap();
        state.baseline_tick = state.baseline_tick.max(last_processed_tick);
    }
}
```

## 5.4 优先级控制与带宽预算裁剪（细化FR-SY-005）

```rust
fn priority_score(entity: &ReplicationMeta, distance: f32, staleness_ticks: u32) -> f32 {
    // 距离越近、重要度越高、越久未更新（staleness）优先级越高——staleness项防止长期低优先级实体被无限期饿死
    importance_weight(entity) / (1.0 + distance) + staleness_ticks as f32 * STALENESS_FACTOR
}

fn select_within_budget(candidates: Vec<(EntityId, f32 /* priority */)>, budget_bytes: usize) -> Vec<EntityId> {
    let mut sorted = candidates;
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // 降序
    let mut used = 0usize;
    let mut selected = Vec::new();
    for (id, _) in sorted {
        let cost = estimate_update_size(id); // 精确字节估算属§5.5量化范围，此处为估算值
        if used + cost > budget_bytes { continue; } // 本tick跳过，下tick因staleness提升优先级
        used += cost;
        selected.push(id);
    }
    selected
}
```

**每tick带宽预算推导**：NFR-PE-006峰值20KB/s，按NFR-PE-005配送速率区间的下限10Hz折算（保守估计，避免高估预算），得每tick预算 ≈ 20000 / 10 = **2000字节**（暂定值，见§10，供PH-4负载试验前的开发期参考）。

## 5.5 量化・位打包的接口边界（细化FR-SY-006）

本文档**仅**定义量化前的逻辑接口，不涉及具体位宽与打包布局（移交RGS-IFS-001，参见RGS-BAS-001§1.1边界声明）：

```rust
/// 逻辑量化：将浮点值转换为一个中间表示，具体位宽由RGS-IFS-001决定
pub trait Quantize {
    type Quantized;
    fn quantize(&self, precision: QuantizationPrecision) -> Self::Quantized;
}
```

## 5.6 时钟同步（细化FR-SY-007）

```rust
/// 标准NTP式往返估算：客户端记录t0发送、服务器记录t1接收/t2响应、客户端记录t3接收
/// offset = ((t1 - t0) + (t2 - t3)) / 2；rtt = (t3 - t0) - (t2 - t1)
fn estimate_clock_offset(t0: Instant, t1: Instant, t2: Instant, t3: Instant) -> (Duration, Duration);
```

握手时执行一次，此后按暂定周期性重新探测（周期值见§10），修正长时间连接的时钟漂移。

## 5.7 预测・和解支持（细化FR-SY-008，RGS-BAS-001§4.3.3）

服务器侧职责仅限于：tick循环处理输入后，在下一份差分快照中回送`last_processed_sequence`（RGS-BAS-001§6.2.1 `InputAck`）。和解本身（预测结果比较、回滚重放）是客户端职责，不在服务器详细设计范围内。

## 5.8 可靠性通道分离（细化FR-SY-009）

沿用RGS-BAS-001§6.2.1／§6.2.2既有的消息-通道映射表，不重复定义。本模块的职责仅是：产生`StateDeltaSnapshot`／`InputAck`的System写入不可靠通道发送队列，产生`ItemGrantNotice`／`SceneTransitionCommand`的路径写入可靠通道发送队列——两个发送队列在GW侧对应不同的QUIC通道类型（Datagram／Stream），具体绑定属RGS-IFS-001范围。

---

# 6. PL：玩家・账号详细设计

对应RGS-BAS-001§4.4。

## 6.1 仓储接口

```rust
#[async_trait::async_trait]
pub trait AccountRepository {
    async fn authenticate(&self, credential: CredentialToken) -> Result<AuthResult, PlError>;
}

#[async_trait::async_trait]
pub trait CharacterRepository {
    async fn list_characters(&self, player_id: PlayerId) -> Result<Vec<CharacterSummary>, PlError>;
    async fn issue_session_epoch(&self, character_id: CharacterId) -> Result<SessionEpoch, PlError>;
}
```

> 具体使用的PostgreSQL客户端crate选型（如sqlx／tokio-postgres／deadpool-postgres）不在本文档决定范围，属实现阶段的技术选型，不影响本节定义的trait签名。

## 6.2 会话世代（epoch）发行（细化FR-PL-003，RGS-BAS-001§4.4.2，**ARC-005核心落地点**）

```rust
impl CharacterRepository for PgCharacterRepository {
    async fn issue_session_epoch(&self, character_id: CharacterId) -> Result<SessionEpoch, PlError> {
        let mut tx = self.pool.begin().await?;
        // 关键约束：递增与读取必须是同一条SQL语句内的原子操作（UPDATE...RETURNING），
        // 不得先SELECT当前值、在应用层+1后再UPDATE——
        // 后者在并发重连场景下会产生竞态：两次并发重连可能读到相同旧值，各自+1后得到相同"新"epoch，
        // 破坏Single-Writer保证（ARC-005，RGS-BAS-001§4.4.2设计要点原文）。
        let new_epoch: i64 = sqlx_like_query!(
            &mut tx,
            "UPDATE character SET session_epoch = session_epoch + 1 WHERE character_id = $1 RETURNING session_epoch",
            character_id
        ).await?;
        tx.commit().await?;
        Ok(SessionEpoch(new_epoch))
    }
}
```

## 6.3 账号注册・认证（细化FR-PL-001）

```rust
impl AccountRepository for PgAccountRepository {
    async fn authenticate(&self, credential: CredentialToken) -> Result<AuthResult, PlError> {
        // ①校验凭证 ②检查BZ-006封禁状态（AccountStatus::Banned/Suspended则拒绝）
        // ③加载角色列表 ④返回鉴权结果 —— 需求定义书§5.3.1 FR-PL-001/002处理步骤原文
        unimplemented!()
    }
}
```

---

# 7. EC：玩家经济详细设计

对应RGS-BAS-001§4.5。本章是ARC-006（ACK须在持久化后）与ARC-009（Effectively Once）在代码路径上的直接体现。

## 7.1 确定请求处理（细化FR-EC-003，RGS-BAS-001§4.5.1）

```rust
pub struct CommitRequest {
    pub request_id: RequestId,
    pub character_id: CharacterId, // 沿用RGS-BAS-001§4.5.1/§6.3.2用词，见§1.4关于player_id/character_id的说明
    pub session_epoch: SessionEpoch,
    pub operation: EconomyOperation, // GrantItem | ConsumeItem | GrantCurrency | ConsumeCurrency
    pub expected_version: Version,
}

pub struct TxResult {
    pub success: bool,
    pub new_version: Version,
    pub ledger_id: LedgerId,
}

const MAX_OCC_RETRIES: u32 = 3; // 已由需求定义书§5.3.2确定，非本文档暂定值

impl EconomyService {
    /// 关键约束（**易错点，实现时须遵守**）：OCC冲突后的"重新读取"（需求定义书§5.3.2原文）
    /// 意味着**开启一个全新事务**重新读取当前version，而不是在同一个尚未提交的事务内原地重读——
    /// 同一事务的快照看不到导致冲突的那次并发提交（PostgreSQL Read Committed下语句级重读虽能看到，
    /// 但Repeatable Read下整个事务会因串行化冲突直接失败），两种隔离级别下"事务内重试"都不成立。
    /// 因此重试循环包裹的是**整个"开事务→校验→更新→提交/回滚"周期**，而非仅重试更新语句本身。
    pub async fn commit_transaction(&self, req: CommitRequest) -> Result<TxResult, EcError> {
        // ①幂等判定（循环外只读一次，纯粹为减少无谓事务开销，非正确性必需——
        // 即使跳过本判定直接进入下方循环，尚未提交成功的request_id在每次尝试的②中依然查得"未处理"）
        if let Some(prior) = lookup_processed_request_readonly(req.request_id).await? {
            return Ok(prior.result_snapshot); // ARC-009幂等重放，不产生新副作用
        }

        let mut attempt = 0;
        loop {
            let mut tx = self.pool.begin().await?; // 每次尝试都是全新事务，全新快照

            // ②事务内二次幂等判定（防止①只读检查之后、本次BEGIN之前，
            // 另一并发请求已抢先完成处理）
            if let Some(prior) = lookup_processed_request(&mut tx, req.request_id).await? {
                tx.commit().await?; // 无新写入，仅提交只读判定
                return Ok(prior.result_snapshot);
            }

            // ③session_epoch合法性判定（ARC-005，细节见§7.2）。
            // 棘轮的推进**必须**与本次尝试的OCC更新共享同一事务，随其一同提交或一同回滚，
            // 不得独立持久化——否则会出现"防护判定已推进但对应写入从未发生"的不一致状态（见§7.2末段）
            if validate_epoch_fence(&mut tx, req.character_id, req.session_epoch).await.is_err() {
                tx.rollback().await?;
                return Err(EcError::EpochExpired);
            }

            // ④OCC更新。若`operation`是复合操作（同时影响Wallet与Inventory，
            // 例如"扣钱+发货"，RGS-BAS-001§5.4.2已预留此可能性），
            // `apply_occ_update`内部对多个聚合根的加锁顺序必须遵循§2.8锁顺序规则
            // （复合排序键：本函数③已先行按character_id字节序锁定fence行，
            // 此处对inventory/wallet的加锁须按"同角色内表名字典序"排序，inventory先于wallet）
            match apply_occ_update(&mut tx, &req).await? {
                OccOutcome::Applied { new_version, ledger_id } => {
                    // ⑤流水记录（BZ-003可完全复原）
                    insert_ledger_entry(&mut tx, &req, new_version).await?;
                    // ⑥Outbox登记（DR-011，同一事务）
                    insert_outbox_event(&mut tx, &req).await?;
                    let result = TxResult { success: true, new_version, ledger_id };
                    // 已处理记录（供未来幂等重放命中①/②）
                    insert_processed_request(&mut tx, req.request_id, req.character_id, &result).await?;
                    tx.commit().await?; // ②〜⑥在此单一事务内原子提交——ARC-006/ARC-009的唯一交汇点
                    return Ok(result);
                }
                OccOutcome::Conflict => {
                    tx.rollback().await?; // 本次尝试（含epoch棘轮的推进）整体回滚，不留痕迹
                    attempt += 1;
                    if attempt > MAX_OCC_RETRIES {
                        return Err(EcError::VersionConflict); // 业务错误，需求定义书§5.3.2
                    }
                    continue; // 下一轮循环以全新事务、全新读取的version重试
                }
            }
        }
    }
}
```

## 7.2 session_epoch防护机制（本文档新增设计，登记为ISS-012）

RGS-BAS-001§4.5.1的时序图中，`EC->>DB: 校验session_epoch合法性（ARC-005）`一步未说明EC**依据什么本地状态**做出该判定。经分析：

- 依DR-004（禁止跨限界上下文直接SQL访问），EC**不得**直接查询`player_db`的`character.session_epoch`列。
- 若改为在确定请求路径中同步调用PlayerService查询当前epoch，将违反CON-007（业务事务不得依赖额外的跨服务同步调用）并挤占NFR-PE-008（p99<20ms）预算。
- RT在路由建立时已校验epoch为最新（RGS-BAS-001§3.5），但RGS-BAS-001§4.5.1仍要求EC侧独立校验，体现纵深防御原则（防止RT一侧的badge逻辑缺陷或竞态导致旧连接的请求流入）。

**本文档给出的方案**：EC在自身限界上下文内维护一个**防护令牌棘轮（Fencing Token Ratchet）**，不依赖对`player_db`的直接读取，仅依据"EC自己见过的最大epoch"单调判定：

```rust
/// 逻辑实体（economy_db内新增，尚未被RGS-BAS-001§5.4 ER图记载，见下方说明）
/// CHARACTER_EPOCH_FENCE { character_id PK, last_seen_epoch, updated_at }
async fn validate_epoch_fence(tx: &mut Tx, character_id: CharacterId, presented: SessionEpoch) -> Result<(), EcError> {
    let fence = select_epoch_fence_for_update(tx, character_id).await?; // 行锁，防止同世代内的并发请求互相踩踏
    match fence {
        None => { insert_epoch_fence(tx, character_id, presented).await?; Ok(()) } // 首次见到该角色
        Some(last) if presented.0 < last.0 => Err(EcError::EpochExpired), // 旧epoch，拒绝（ARC-005核心判定）
        Some(last) if presented.0 > last.0 => { update_epoch_fence(tx, character_id, presented).await?; Ok(()) } // 棘轮前进
        Some(_) => Ok(()), // 与当前世代一致的正常请求
    }
}
```

**事务边界约束（易错点）**：`validate_epoch_fence`**必须**与调用方（§7.1 `commit_transaction`）当次尝试的OCC更新共享同一事务，棘轮的推进只能随该次尝试一同提交或一同回滚，**不得**作为独立于该事务之外的语句执行。若脱离该约束单独提交棘轮推进，会出现"防护判定已推进但对应的经济写入从未成功落地"的不一致状态——`FOR UPDATE`行锁的持有范围与OCC更新语句必须在同一次`BEGIN…COMMIT`内。

**处置**：`CHARACTER_EPOCH_FENCE`是一个RGS-BAS-001§5.4 ER图未包含的新逻辑实体，超出本文档"不得变更基本设计书逻辑数据模型"的边界（本文档§1.1）。已登记为**ISS-012**（见RGS-REQ-005本次修订），状态"决议完毕（方案已确定，需在RGS-BAS-001下次修订版中正式吸收入§5.4 ER图）"。在RGS-BAS-001正式吸收前，本方案作为PH-3实现的权威依据；`CHARACTER_EPOCH_FENCE`的物理形态（是否独立表、是否合并入现有WALLET/INVENTORY行）由RGS-DBS-001决定。

## 7.3 EC降级路径的重试队列（细化RGS-BAS-001§4.5.2）

```rust
pub struct EconomyDegradeQueue {
    pending: VecDeque<CommitRequest>, // 有界，容量见§10（CON-008）
    retry_interval: Duration,          // 暂定值见§10
}
```

`RT→EC`调用超时（服务不可用）时，请求进入该本地队列（RGS-BAS-001§4.5.2节点F），客户端展示"获取中"而非"已获得"，**不得**虚构一个已确定的结果（RGS-BAS-001原文约束）。持续不可用超过阈值（§10暂定值）后提示玩家稍后查看背包。

---

# 8. MT／GD／EV／WF：模块划分（概要级）

依据本文档§1.2颗粒度声明，本章仅给出模块划分，不展开函数签名与算法。函数签名级详细设计留待各子系统开始前（PH-5／PH-6）补充本文档。

| 子系统 | 模块 | 职责一句话 |
|---|---|---|
| MT 对局・匹配 | 匹配队列模块 | 队列投入・撮合・成立通知（FR-MT-002） |
| MT 对局・匹配 | 对局生命周期模块 | 落地ST-002状态机（FR-MT-001） |
| MT 对局・匹配 | 结算模块 | 结果确定与奖励发放，经§7同款确定请求机制（FR-MT-003） |
| GD 社交・公会 | 好友模块 | 好友申请・确认・删除（FR-GD-001） |
| GD 社交・公会 | 聊天模块 | 频道・私聊・禁言（FR-GD-002） |
| GD 社交・公会 | 公会模块 | 创建・加入・退出・权限（FR-GD-003） |
| EV 事件基础设施 | Outbox分发器 | 轮询各服务outbox表并发布（FR-EV-001，RGS-BAS-001§4.7.1） |
| EV 事件基础设施 | Schema管理模块 | 登记・兼容性检查・版本管理（FR-EV-003） |
| EV 事件基础设施 | 消费者幂等模块 | 依`event_id`去重（FR-EV-004） |
| WF 工作流基础设施 | 购买Saga编排模块 | 落地ST-003状态机含补偿路径（FR-WF-001，RGS-BAS-001§4.7.2） |
| WF 工作流基础设施 | 补偿处理模块 | 各Activity的补偿实现（FR-WF-003） |

---

# 9. 错误码体系

本节是本文档在RGS-BAS-001§9.2授权范围内**唯一自主决定**的编号体系（"具体错误码编号体系属详细设计范围"）。编码格式：`E-<SS>-<分类位><序号>`，`<SS>`为需求定义书§5.1子系统符号，分类位延续RGS-BAS-001§9.1三分类（1=业务错误，5=系统错误，9=基础设施错误）。

| 错误码 | 分类 | 含义 | 对应需求 |
|---|---|---|---|
| `E-GW-1001` | 业务错误 | 鉴权令牌格式错误 | NFR-SE-006 |
| `E-GW-1002` | 业务错误 | 凭证无效 | FR-PL-001 |
| `E-GW-1003` | 业务错误 | 账号封禁中 | BZ-006 |
| `E-GW-1004` | 业务错误 | 输入超出速率限制 | NFR-SE-008、FR-GW-006 |
| `E-GW-5001` | 系统错误 | 会话表内部不一致 | FR-GW-003 |
| `E-GW-9001` | 基础设施错误 | 目标运行时节点不可达 | §3.4 |
| `E-RT-1001` | 业务错误 | 场景已达硬上限 | FR-RT-013、NFR-PE-016 |
| `E-RT-1002` | 业务错误 | 输入序号超出合法窗口（疑似重放） | NFR-SE-007 |
| `E-RT-1003` | 业务错误 | 物理合法性校验失败（速度／位置越界） | NFR-SE-001、BZ-004 |
| `E-RT-5001` | 系统错误 | 场景Actor task异常终止（已由监督者处理） | FR-RT-010 |
| `E-RT-9001` | 基础设施错误 | 检查点存储写入失败 | FR-RT-009 |
| `E-SY-1001` | 业务错误 | 客户端确认的基线tick早于服务器保留窗口 | FR-SY-004 |
| `E-PL-1001` | 业务错误 | 角色不存在或不属于该账号 | FR-PL-002 |
| `E-PL-1002` | 业务错误 | 账号封禁中，拒绝发行epoch | BZ-006、ARC-005 |
| `E-PL-9001` | 基础设施错误 | player_db不可达 | NFR-AV-009 |
| `E-EC-1001` | 业务错误 | 货币余额不足 | BZ-001 |
| `E-EC-1002` | 业务错误 | 背包容量已满 | FR-EC-001 |
| `E-EC-1003` | 业务错误 | 版本冲突，重试已达上限 | DR-008、需求定义书§5.3.2 |
| `E-EC-1004` | 业务错误 | session_epoch已过期 | ARC-005、§7.2 |
| `E-EC-9001` | 基础设施错误 | economy_db不可达，降级至本地重试队列 | NFR-AV-009、§7.3 |

> 本表为初版示例集合，不要求穷尽全部业务规则（BZ-nnn）在PH-1即分配码值。PH-2〜PH-4各模块开工时**应当**依本节格式补充完整错误码，纳入本文档下一修订版。

---

# 10. 暂定参数一览

| 参数 | 暂定值 | 推导依据 | 关联ISS | 复核时机 |
|---|---|---|---|---|
| tick阶段预算（输入应用／移动／战斗／AOI／复制） | 5.00／6.25／6.25／3.75／3.75 ms | 推导自NFR-PE-002（25ms）× RGS-BAS-001§4.2.2占比 | 无 | PH-2实测后调整 |
| SceneActor mailbox容量 | 1024 | 推导自NFR-PE-016（软上限300）×2 tick突发缓冲，向上取2的幂 | 无 | PH-2〜PH-4负载试验 |
| `MAX_MESSAGES_PER_TICK`（单tick排空mailbox上限） | 1024（等于mailbox容量） | 取值等于mailbox容量本身，使单tick可在mailbox满载时一次性排空，避免更低的取值让输入被跨tick延迟处理 | 无 | PH-2〜PH-4负载试验 |
| GW心跳间隔／超时 | 15秒／45秒（3次未响应） | 工程惯例值，与既有NFR无直接换算关系 | 无 | PH-1实现时确认 |
| GW限流令牌桶容量／补充速率 | 40／20每秒 | 补充速率对齐NFR-PE-001（20Hz），容量留2秒突发余量 | 无 | PH-3实现时确认 |
| AOI格子大小 | 25米 | 假设典型视野半径约25米，3×3邻域覆盖75米直径视野；最终值待策划确认 | **ISS-009** | PH-2 |
| 检查点周期 | 30秒 | 沿用RGS-BAS-001§4.2.3／§10已给出的暂定值，未变更 | **ISS-010** | PH-3 |
| GW排空超时 | 5分钟 | 与NFR-AV-003（游戏节点RTO目标5分钟）量级对齐，非该指标本身 | 无 | PH-4负载试验 |
| RT→EC调用超时 | 200ms | 为NFR-PE-008（p99<20ms）预留约10倍网络与排队余量；因调用为§2.5非阻塞模式，超过一个tick周期（50ms）不违反CON-007 | 无 | PH-3实现时确认 |
| EC确定请求OCC重试上限 | 3次 | **已由需求定义书§5.3.2确定**，非暂定值 | 无（已决） | — |
| 检查点保留代数 | 3代 | **已由需求定义书§6.5确定**，非暂定值 | 无（已决） | — |
| 场景容量软／硬上限 | 300／500 | **已由NFR-PE-016确定**，非暂定值 | 无（已决） | — |
| EC降级重试队列容量／重试间隔 | 待定 | 需PH-3结合NFR-AV-009降级运行试验确定 | 无 | PH-3 |
| CHARACTER_EPOCH_FENCE逻辑实体 | 方案已定（§7.2） | 本文档新增，需RGS-BAS-001吸收 | **ISS-012** | PH-3前完成RGS-BAS-001修订 |
| 数据库会话`lock_timeout`（§2.8） | 50ms | 工程惯例值：**必须**明显小于调用方超时（本表"业务App间gRPC调用超时100ms"），使锁等待在调用方放弃等待之前就先由数据库以`55P03`报错快速失败并触发重试，而不是让调用方超时掩盖真实原因；若取值接近或超过调用方超时，本参数将永远无法在调用方放弃前触发，形同虚设 | 无 | PH-2〜PH-4负载试验 |
| 死锁重试上限／退避 | 3次，与EC OCC重试同款退避策略 | 复用§7.1已确定的重试结构，不新增一套参数体系 | 无 | PH-2实现时确认 |
| 业务App间gRPC调用超时（PL/EC/MT/GD/AD互调，非RT→EC路径） | 100ms | 为NFR-PE-009（p99<200ms）预留约2倍余量；此类调用发生在业务App自身的请求处理路径中（非tick循环），可同步等待 | 无 | PH-3〜PH-5各App开工时确认 |
| App间调用连接池上限（每App、每被调方） | 32 | 工程惯例值，需与RGS-DBS-001的数据库连接池上限（§7.2设计要点提及"须设上限"）分开配置，避免App间调用饱和挤占数据库连接预算 | 无 | PH-2〜PH-4负载试验 |
| HTTPS类App间调用（业务App间的有界重试）退避策略 | 初始100ms，指数退避×2，最多3次 | 工程惯例值，与gRPC路径的重试结构（§2.7.3）保持同一量级 | 无 | PH-3实现时确认 |

---

# 11. 单体测试观点

供RGS-TST-001（试验计划书）接续，本节仅列出各模块**须验证的不变量**，不展开具体测试用例。

| 模块 | 不变量 |
|---|---|
| GW鉴权 | 封禁账号（BZ-006）任何情况下不得建立会话 |
| GW限流 | 令牌不足时输入被丢弃但连接不断开（§3.5） |
| RT mailbox | 队满时`try_send`失败不得阻塞tick循环（CON-008／§2.4） |
| RT tick循环 | 任何tick内不得出现同步跨进程I/O调用（CON-007／§2.5，可通过静态分析或tokio-console实测验证） |
| RT数值表切换 | 同一tick内全部System读取到同一版本配置，不出现"半新半旧" | 
| RT检查点恢复 | 从检查点重建的场景状态与检查点保存时刻的状态一致（幂等重放） |
| SY AOI | 实体跨越格子边界时enter_view／leave_view事件各恰好触发一次，不重复不遗漏 |
| SY带宽裁剪 | 连续多tick未被选中的实体，staleness因子应使其优先级单调上升，不发生无限期饿死 |
| PL epoch发行 | 并发调用`issue_session_epoch`不产生重复epoch值（竞态测试，§6.2） |
| EC确定请求 | 同一`request_id`重放任意次数，返回结果与副作用均与首次一致（幂等，§7.1） |
| EC OCC重试 | 版本冲突重试3次后仍冲突，返回业务错误而非无限重试 |
| EC epoch防护 | presented_epoch严格小于已记录的棘轮值时必须拒绝（§7.2） |
| 并发（跨聚合根） | 并发发起"同时涉及Inventory与Wallet"的复合确定请求，最终状态与串行执行等价，且不产生死锁（§2.8规则1、§7.1） |
| 并发（多角色） | 并发发起A→B与B→A两笔涉及不同角色对的操作，双方均不发生死锁等待（§2.8规则2，模拟FR-EC-008场景的压力测试） |
| 死锁重试 | 人为构造死锁（两个事务以相反顺序加锁同一对行）时，数据库返回`40P01`后应用层能自动重试并最终成功，不留下部分写入 | 
| App间调用重试 | 业务错误（如`E-EC-1001`）不触发§2.7.3的`BoundedRetry`；基础设施错误（如超时）触发有界重试且不超过配置上限 |

---

# 12. 追溯性

## 12.1 RGS-BAS-001章节 → 本文档章节

| RGS-BAS-001章节 | 本文档展开章节 |
|---|---|
| §4.1（GW） | §3 |
| §4.2（RT） | §4 |
| §4.3（SY） | §5 |
| §4.4（PL） | §6 |
| §4.5（EC） | §7 |
| §4.6／4.7（MT／GD／EV／WF） | §8（概要级，颗粒度继承） |
| §9（异常・错误处理设计方针） | §9（错误码体系，本文档首次给出具体码值） |
| §3.1／3.2（部署构成・部署单元一览） | §2.1（App群组：独立部署单元的划分，正式决定，不变更部署拓扑本身） |
| §3.3（网络区域设计） | §2.7（跨App API互通设计：协议归属不变，新决定HTTPS类接口的JSON序列化） |
| §4.5.1／§4.7.2（确定请求事务、购买工作流补偿） | §2.8〜§2.10（并发控制・回滚・排他总则，将既有个案机制提升为全App群组通用规则） |

## 12.2 RGS-BAS-001§10「详细设计移交事项一览」逐项处理情况

| 编号 | 事项 | 本文档处理情况 |
|---|---|---|
| 1 | 高频路径字节级线路格式 | 仍移交**RGS-IFS-001**；本文档§5.5仅给出量化前的逻辑接口 |
| 2 | AOI格子大小与视野距离具体值 | 本文档§5.1／§10给出PH-2开发用暂定值25米，最终由**ISS-009**决议 |
| 3 | 检查点周期具体值 | 本文档§4.9／§10沿用RGS-BAS-001已给出的暂定30秒，未变更，最终由**ISS-010**决议 |
| 4 | tick预算各阶段具体毫秒数 | 本文档§4.2已给出推导值（5.00／6.25／6.25／3.75／3.75ms），PH-2实测后调整 |
| 5 | mailbox容量、经济服务待处理请求数上限 | 本文档§4.14／§7.3／§10给出暂定初始值，PH-2〜PH-4负载试验后调整 |
| 6 | gRPC方法`.proto`文件化 | 仍移交**RGS-IFS-001**；本文档维持RGS-BAS-001§6.3已定义的方法名／字段名不变，未新增字段 |
| 7 | 数据库物理设计（DDL） | 仍移交**RGS-DBS-001**；本文档§7.2新增的`CHARACTER_EPOCH_FENCE`逻辑实体供其物理化参考 |
| 8 | 错误码编号体系 | **本文档§9已给出**——本文档职责范围内解决的唯一一项 |
| 9 | 支付服务商选型与Webhook签名算法 | 不涉及，属WF/EC的PH-6范围，本文档§8维持模块划分级颗粒度 |
| 10 | MT／GD子系统详细处理时序 | 仍未展开，本文档§8维持模块划分级别，与RGS-BAS-001§1.2颗粒度声明一致 |

## 12.3 本文档新发现事项

| 编号 | 内容 | 处置 |
|---|---|---|
| ISS-012 | EC侧`session_epoch`防护机制未在RGS-BAS-001§5.4 ER图落地存储位置 | 本文档§7.2给出方案（棘轮式`CHARACTER_EPOCH_FENCE`），已登记RGS-REQ-005，需RGS-BAS-001下次修订版吸收 |

---

**以上**
