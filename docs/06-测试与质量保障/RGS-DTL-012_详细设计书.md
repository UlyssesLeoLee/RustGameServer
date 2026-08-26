# 详细设计书（詳細設計書 / Detailed Design Document）

**测试基础设施与自动化验证：模拟客户端资源模型・k6/Playwright配置具体格式・可复现性种子管理详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-012 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-012 测试基础设施与自动化验证 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-011/013/014/019并行产出）。细化RGS-BAS-012§3模拟客户端组件结构为具体行为画像配置格式与容量估算公式、§4支付渠道Mock为具体HTTP端点契约、§5.1/§5.3 k6脚本组织与可复现性设计为具体种子管理机制、§6.1/§6.4 Playwright双模式与UAT可复现性为具体测试数据准备脚本契约、§8 CI流水线分层为具体GitHub Actions定义（复用RGS-DTL-002§4已确立的CI阶段模式）。**本版本不覆盖**：具体测试用例内容本身（属RGS-TST-001既定范围，非本文档职责）、参考GM后台的前端技术栈选型（TBD-TST-002）。见§6 | 全部 |
| 0.2 | 2026-08-25 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））| — | 同步父 BAS-012 升版至 v0.2 + 补追溯性表 AC-TST-001〜004 验收标准与本文档展开章节的映射 | §10 追溯性 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 模拟客户端资源模型是否与RGS-BAS-012§3.2"资源可预测性"要求一致，PRNG种子机制是否与RGS-BAS-008§9既定原则真正复用而非另起一套 |
| 评审（QA/性能负责人） | | | k6/Playwright具体配置格式是否可直接落地供PH-4/PH-8负载试验使用 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [协议层模拟客户端：资源模型与配置格式](#2-协议层模拟客户端资源模型与配置格式)
3. [外部依赖Mock：具体端点契约](#3-外部依赖mock具体端点契约)
4. [k6性能测试：配置格式与可复现性种子机制](#4-k6性能测试配置格式与可复现性种子机制)
5. [Playwright UAT：测试数据准备契约](#5-playwright-uat测试数据准备契约)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-012给出了测试基础设施的组件划分（模拟客户端/外部依赖Mock/k6/Playwright/参考GM后台）与各组件的目录结构、关键设计点、CI集成分层——均为"需要哪些组件、各自职责边界是什么"这一基本设计层面。本文档将其中具体可落地的部分细化为：模拟客户端的资源预算公式与行为画像配置文件具体格式、支付渠道Mock的HTTP端点契约、k6/Playwright的可复现性种子管理具体机制、CI流水线的GitHub Actions定义。

### 1.2 本文档不做什么

- **不重新决定**RGS-BAS-012已确定的任何结构性选择（模拟客户端复用`rgs-client-core`而非重新实现协议、外部依赖Mock不常驻生产、UAT/负载测试流水线与主干CI分层解耦、参考GM后台的鉴权独立测试凭证体系）。
- **不覆盖**具体测试用例内容本身——RGS-BAS-012§1已明确"不含具体测试用例内容（属RGS-TST-001）"，本文档同样不越权，只给出测试**基础设施**的物理格式。
- **不选定**参考GM后台的前端技术栈（TBD-TST-002）——该项留待详细设计阶段的独立技术选型评审，本文档不代为决定。

### 1.3 记述规则

沿用既有DTL文档记述规则：配置文件以JSON/TOML具体片段给出；CI流水线以GitHub Actions YAML给出，复用RGS-DTL-002§4已确立的写法与阶段命名习惯；伪代码可直接对应Rust `Result`风格实现。

---

## 2. 协议层模拟客户端：资源模型与配置格式

对应RGS-BAS-012§3.1/§3.2。

### 2.1 行为画像配置文件格式

```toml
# tests/perf/fleet-profiles/gathering-behavior.toml
# 对应RGS-BAS-012§3.2"行为画像"设计点，具体化为可被fleet.rs直接解析的配置格式
[profile]
name = "gathering_behavior"
instance_count = 10000

[movement]
pattern = "clustered"          # "random_walk" | "fixed_path" | "clustered"，三值枚举，对应§3.2既定三种移动模式
cluster_radius_m = 15.0        # pattern="clustered"时生效，其余pattern下忽略本字段

[input]
actions_per_second = 3.5       # 输入频率，供NFR-PE-*系列指标覆盖不同流量特征

[reconnect]
disconnect_probability_per_min = 0.02   # 掉线重连概率，每实例每分钟独立采样
reconnect_delay_ms = { min = 500, max = 3000 }

[seed]
# 对应§4本文档"可复现性"设计，见§4.2；本画像文件本身不含种子，种子由发起测试的CLI参数/config/ccu-*.json统一注入
```

`fleet.rs`按`instance_count`批量实例化，每个实例引用同一份`profile`但各自的随机决策（移动路径采样点、掉线时机）独立按`RngCore`派生子种子（见§4.2），保证"同一画像、不同实例"之间不因共享单一RNG状态而产生非预期的相关性伪影。

### 2.2 资源预算公式（落实RGS-BAS-012§3.2"资源可预测性"）

```rust
// 单实例内存占用估算，供容量规划("单Pod 8GB可承载N个实例"这类结论的计算依据)
struct InstanceMemoryBudget {
    protocol_buffer_bytes: usize,   // QUIC Stream缓冲区，固定量级，取自rgs-client-core既定缓冲区大小常量
    prediction_state_bytes: usize,  // 客户端预测/回滚状态(位置/输入队列历史)，随AOI视野半径与tick历史窗口线性增长
}

fn estimate_instance_count(pod_memory_budget_bytes: usize, per_instance: &InstanceMemoryBudget, safety_margin: f64) -> u32 {
    let per_instance_total = per_instance.protocol_buffer_bytes + per_instance.prediction_state_bytes;
    // safety_margin: 提案默认0.7(即仅使用70%可用内存做实例容量估算，为OS/运行时/指标采集预留30%余量)
    // 该0.7为初始提案，非最终值，同RGS-DTL-025§5同类"提案默认值"处理方式，PH-4实测前不作最终结论
    let usable = (pod_memory_budget_bytes as f64 * safety_margin) as usize;
    (usable / per_instance_total) as u32
}
```

该公式的输出（"单Pod可承载N个实例"）直接驱动`fleet.rs`启动时的分片规划——单进程实际承载数须不超过`estimate_instance_count`结果，超出部分横向扩展至新Pod，而非在单进程内继续堆叠实例导致OOM（对应RGS-BAS-012§3.2"资源可预测性"要求的直接落实：容量规划基于**估算公式**而非"跑到崩溃为止"的试错）。

---

## 3. 外部依赖Mock：具体端点契约

对应RGS-BAS-012§4.1。以下为支付渠道Mock（IF-006）的具体HTTP端点契约，落实§4.1三行表格（端点/响应模式/用途）为可直接实现的接口定义。

```
POST /mock/payment/initiate
Request:  { "order_id": "...", "amount": 999, "response_mode": "success" | "failure" | "timeout" }
Response(success):  200 { "transaction_id": "...", "status": "completed" }
Response(failure):  200 { "transaction_id": "...", "status": "declined", "reason": "mock_declined" }
Response(timeout):  无响应，连接挂起至客户端自身超时阈值(验证RGS-BAS-001§6.4既定超时处理路径)

POST /mock/payment/webhook-trigger
# 测试脚本主动触发的Mock侧回调下发，模拟真实支付渠道的异步Webhook
Request:  { "transaction_id": "...", "signature_mode": "valid" | "forged" | "delayed" }
# signature_mode=valid: 用与RGS-BAS-001§6.4既定验签算法匹配的密钥签名(测试专用密钥对，不与生产共享)
# signature_mode=forged: 用错误密钥签名，验证签名校验路径确实拒绝
# signature_mode=delayed: 延迟N秒(可配置)后才实际POST至被测系统的Webhook接收端点，验证幂等键处理

POST /mock/payment/partial-failure
# 对应§4.1"部分失败"行，VF-006 Saga补偿专用场景
Request:  { "order_id": "...", "fail_at_stage": "post_payment_pre_delivery" }
Response: 200 { "transaction_id": "...", "status": "completed" }
# 支付本身返回成功，但Mock内部标记该transaction_id在"发货前"阶段不触发§4.1.2既定的正常发货回调，
# 模拟"支付成功但发货前中断"这一VF-006验证场景，由测试脚本后续断言补偿路径(Saga补偿)是否被正确触发
```

Mock服务自身不持久化状态跨测试运行保留（每次CI流水线启停均为全新实例，复用RGS-BAS-012§4.2"随CI流水线按需启停"既定设计），`transaction_id`由Mock按请求实时生成，不需要预置测试夹具数据。

---

## 4. k6性能测试：配置格式与可复现性种子机制

对应RGS-BAS-012§5.1/§5.2/§5.3。

### 4.1 并发梯度配置具体格式

```json
{
  "scenario_name": "ccu-100k-ramp",
  "seed": 20260817001,
  "stages": [
    { "duration_s": 300, "target_ccu": 10000 },
    { "duration_s": 600, "target_ccu": 50000 },
    { "duration_s": 900, "target_ccu": 100000 },
    { "duration_s": 1800, "target_ccu": 100000 }
  ],
  "behavior_profile_ref": "gathering-behavior.toml"
}
```

`config/ccu-100k-ramp.json`按RGS-BAS-012§5.1既定路径版本化管理（FR-TST-023），`seed`字段是本文档对§5.3可复现性要求的具体落地点（见§4.2）。

### 4.2 可复现性种子机制（落实RGS-BAS-012§5.3，复用RGS-BAS-008§9 Seeded PRNG原则）

```rust
// 复用RGS-BAS-008§9既定"Seeded PRNG可复现"实现思路，本文档只固定其在测试基础设施场景下的具体调用契约
fn derive_instance_rng(master_seed: u64, instance_index: u32) -> impl RngCore {
    // 主种子(来自§4.1配置文件的"seed"字段) + 实例序号 → 派生子种子，保证:
    //   1) 同一master_seed重放时，每个实例序号对应的行为轨迹完全一致(NFR-TST-004可复现性核心要求)
    //   2) 不同实例序号之间不共享RNG状态，避免实例间行为相关性伪影(见§2.1末尾说明)
    let derived = splitmix64_derive(master_seed, instance_index as u64);  // 复用RGS-BAS-008§9同款派生算法，非本文档新增
    StdRng::seed_from_u64(derived)
}
```

每次k6/协议层模拟客户端联合测试运行，测试报告**必须**记录本次使用的`master_seed`（若配置文件未显式指定则由测试运行器随机生成一个并记录，而非默默使用不可追溯的默认种子）——PH-4与PH-8两次负载试验（RGS-BAS-012§5.3已述"方可比对结果差异是否源于系统变化而非测试本身的随机噪声"）比对时，**要求**两次运行使用**相同**`master_seed`，若发现性能差异，须先确认两次运行的`master_seed`一致，否则差异可能仅是负载特征分布随机噪声导致，比对本身不成立。

### 4.3 指标适配脚本具体映射（补充RGS-BAS-012§5.2表述层面）

```javascript
// tests/perf/k6/lib/metrics-adapter.js
export function adaptMetricName(k6MetricName) {
  const mapping = {
    'http_req_duration': 'rgs_request_duration_ms',
    'http_req_failed':   'rgs_request_error_ratio',
    'vus':                'rgs_load_virtual_users',
  };
  return mapping[k6MetricName] ?? k6MetricName;  // 未登记映射的原生指标透传，不阻断，但不接入既有Dashboard既定面板
}
```

---

## 5. Playwright UAT：测试数据准备契约

对应RGS-BAS-012§6.1/§6.4。

### 5.1 确定性数据准备脚本契约（落实§6.4可复现性设计）

```typescript
// tests/uat/playwright/fixtures/prepare-baseline.ts
// 每次CI运行前调用，通过§6.1既定"api/纯API模式"发起幂等初始化请求，从已知基线重新准备数据
export async function prepareBaseline(): Promise<TestFixtureContext> {
  const testAccountId = await createTestAccount({
    requestId: `uat-baseline-${process.env.CI_RUN_ID}`,  // 幂等键：同一CI_RUN_ID重复调用不重复创建
    initialCurrency: 10000,
    initialItems: [{ templateId: 'test_item_basic', quantity: 5 }],
  });
  // 复用RGS-DTL-001§3.2既定确定请求API的幂等语义，本脚本不新增独立的初始化幂等机制
  return { testAccountId };
}
```

`CI_RUN_ID`作为幂等键的组成部分，保证同一次CI运行内重复调用本函数（如多个测试文件各自的`beforeEach`钩子）不产生重复账号；不同`CI_RUN_ID`（不同CI运行）之间天然产生不同账号，避免跨运行的状态残留互相干扰（对应RGS-BAS-012§6.4"不得依赖上一次测试运行残留的状态"要求的具体落实）。

### 5.2 双模式共用断言库调用约定

```typescript
// tests/uat/playwright/shared/assertions.ts
// UI模式与API模式共用同一断言函数，保证两种驱动方式验证的是同一组契约而非各自维护重复断言逻辑
export function assertBanAccountAuditTrail(auditRecord: AuditLogEntry, expected: { operatorId: string; targetId: string }) {
  expect(auditRecord.action).toBe('BanAccount');
  expect(auditRecord.operatorId).toBe(expected.operatorId);
  expect(auditRecord.targetId).toBe(expected.targetId);
  // UI模式(admin-ban-flow.spec.ts)与API模式(admin-service.contract.spec.ts)均调用本函数，
  // 对应RGS-BAS-012§6.1"共用同一断言库"设计点的具体落地
}
```

---

## 6. 本文档的覆盖范围与后续计划

本文档覆盖：模拟客户端行为画像配置文件格式与单实例资源预算估算公式、支付渠道Mock三个端点的具体HTTP契约（发起/Webhook回调/部分失败）、k6并发梯度配置格式与Seeded PRNG种子派生机制、Playwright确定性数据准备脚本与双模式共用断言库的具体调用约定。CI流水线复用RGS-DTL-002§4已确立的GitHub Actions阶段模式与写法，本文档仅新增测试基础设施特有的触发条件（如§8既定"UAT流水线异步触发、负载测试流水线按需触发"），不重复展开通用CI骨架本身。

本版本明确不覆盖、留待后续：

- 具体测试用例内容本身——RGS-BAS-012§1已明确该部分属RGS-TST-001（独立文档）职责，本文档聚焦基础设施，不越界产出具体用例。
- 参考GM后台前端技术栈选型（TBD-TST-002）——留待详细设计阶段独立技术评审，本文档不代为决定。
- `estimate_instance_count`公式中`safety_margin=0.7`及`InstanceMemoryBudget`各字段的具体数值——均为PH-4实测前的初始提案，需按实测容量数据校准，本文档不给出最终值。
- 支付渠道Mock的具体实现代码（本文档只固定HTTP契约，服务端实现语言/框架选型留待实现阶段）。
- 负载测试流水线Runner资源配额的具体规格（RGS-BAS-012§8已述"不与常规CI共享Runner配额"，具体规格数值属运维资源规划范畴，非本文档职责）。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-012§3.1 模拟客户端组件结构 | §2.1 |
| RGS-BAS-012§3.2 资源可预测性/行为画像 | §2.1、§2.2 |
| RGS-BAS-012§4.1 支付渠道Mock端点 | §3 |
| RGS-BAS-012§4.2 Mock部署方式 | §3（明确复用不重复展开） |
| RGS-BAS-012§5.1 k6脚本组织 | §4.1 |
| RGS-BAS-012§5.2 指标适配 | §4.3 |
| RGS-BAS-012§5.3 负载测试可复现性 | §4.2 |
| RGS-BAS-012§6.1 双模式测试架构 | §5.2 |
| RGS-BAS-012§6.4 UAT可复现性 | §5.1 |
| RGS-BAS-012§8 CI集成与流水线分层 | §6（明确复用RGS-DTL-002不重复展开） |
| RGS-DTL-002（挂载脚手架物理落地） | 前提依赖，CI阶段模式与写法复用 |
| RGS-DTL-025§5（提案默认值处理方式的既定先例） | §2.2（安全余量提案值） |
| AC-TST-001（模拟客户端与三引擎适配层同轨迹重放逐字段一致） | §2.1 行为画像配置格式 + §2.2 资源预算公式（复用`rgs-client-core`既定协议一致性） |
| AC-TST-002（外部依赖Mock跑通购买工作流,含支付失败与补偿路径） | §3 三个端点契约（initiate / webhook-trigger / partial-failure） |
| AC-TST-003（参考GM后台驱动的Playwright测试覆盖AdminService全部方法,成功/失败路径） | §5.1 确定性数据准备脚本契约 + §5.2 双模式共用断言库 |
| AC-TST-004（k6负载试验性能数据可直接在既有Dashboard查看,无需人工转换） | §4.3 指标适配脚本具体映射（k6原生指标→`rgs_request_*`既有命名） |
