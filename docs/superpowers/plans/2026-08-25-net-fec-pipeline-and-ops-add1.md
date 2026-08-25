# NET/FEC 新流水线 + OPS-ADD1 附录式修订 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **文档性质说明**：本计划的产出是治理文档（BAS/DTL/SPEC-DTL 规格书），不是可执行代码。"测试"步骤对应"交叉引用核验"：(a) `run_check5.py` 自动化引用完整性检查；(b) 人工核对新文档条款是否与源 REQ/BAS/DTL 逐条对应，不得杜撰未在源文档出现的决定。每个 Step 给出的表格行/条款值均为最终内容，不是占位符——执行者应直接采用，不得自行"待补充"。

**Goal:** 为 RGS-REQ-038（NET 域，QUIC Datagram FEC 防丢包，ARC-047）建立完整的 BAS→DTL→SPEC-DTL 三级流水线（新文档），并为 RGS-REQ-007-ADD1（OPS 域，GM 后台 DNS/CDN 基础设施可视化）向既有 RGS-BAS-003／RGS-DTL-003 追加附录式修订（原地修订，不新建文件）。

**Architecture:** NET 域走全新编号 RGS-BAS-038 / RGS-DTL-045 / RGS-SPEC-DTL-045（BAS 与 DTL/SPEC-DTL 编号序列彼此独立分配，不假设 1:1 对应，参见既有 BAS-036↔DTL-041 先例）。OPS-ADD1 走"原地修订+修订历史行"模式（参见既有 DTL-022/DTL-025 ADD1 先例），在 RGS-BAS-003 新增 §3.5 与 RGS-DTL-003 新增 §3.5，追溯性表同步扩展，不新建独立文件。

**Tech Stack:** Markdown（GFM），Mermaid（组件图/时序图），Protobuf 风格协议线格式（沿用 RGS-DTL-003 §1.3 记述规则），Rust 伪代码（沿用既有 DTL 记述规则）。

**Spec:**
- `docs/02-运维安全与网络/RGS-REQ-038_核心传输防丢包强化与周边协议选型_需求定义书.md`（NET/FEC 源需求）
- `docs/02-运维安全与网络/RGS-REQ-007_addendum_GM后台基础设施可视化_需求定义书.md`（OPS-ADD1 源需求）
- `docs/00-基准与治理/RGS-REQ-004_附件C_可追溯性矩阵.md`（已完成 NET/OPS-ADD1 域登记，v3.11，§4.1 P-NET-001／§7 NET域／§8 AC-NET/AC-OPS 登记均已就位，本计划不再改动附件C，仅在完成后二次核验其登记与新文档内容一致）

## Global Constraints

- 编号：NET 走 `RGS-BAS-038` / `RGS-DTL-045` / `RGS-SPEC-DTL-045`（新文件）；OPS-ADD1 走 `RGS-BAS-003` v0.3 / `RGS-DTL-003` v0.2（原地修订，新增修订历史行，不新建文件）。
- 审批栏：一人公司治理基线（DEC-008），全部新文档审批栏用语固定为 `Ulysses(角色兼 per DEC-008)`，签字日期 `2026-08-25`。
- 不整体反转 ARC-003（QUIC 双路径）——NET 域全部设计必须在"新增 FEC 编解码层"范围内，不得引入 KCP 或任何 ARQ 式重传逻辑。
- FEC 具体编解码库选型（自研 XOR parity vs. `fastnet`）是 TBD-NET-001，须经 ARC-014 判定；BAS/DTL 阶段**只能**设计"编解码层的接口契约与可插拔点"，**不得**在 BAS/DTL 阶段替 TBD-NET-001 提前拍板选型结论。
- FEC 冗余带宽必须落在既有 NFR-PE-006 预算内；解码延迟必须落在 50ms tick 周期内且不突破 NFR-PE-004。这两条数值约束在 BAS/DTL/SPEC-DTL 三层必须逐层出现（不能只在 REQ 提一次就不再重复校验）。
- OPS-ADD1 范围外：DNS/CDN 写路径（不新增管理功能，只读展示）；不新增独立 CDN 指标采集 SDK（复用 `DistributionBackend` 既有指标）；不新增独立告警链路（复用既有 FR-GM-004／IF-008 Webhook 通道）。
- 每份新文档／修订完成后必须跑一次 `run_check5.py`（脚本路径见 Task 6），新增的 [ISSUE]/[HEADER-FAIL] 数量必须为 0（允许沿用既有的 12 处遗留 header 问题与既有的 "RGS-REQ-004§3.10" 模板级引用问题，这两类是文档库既有已知缺陷，不因本计划新增）。

---

### Task 1: RGS-BAS-038 基本设计书（NET/FEC 基本设计）

**Files:**
- Create: `docs/02-运维安全与网络/RGS-BAS-038_核心传输防丢包强化与周边协议选型_基本设计书.md`

**Interfaces:**
- Consumes：RGS-REQ-038 全部 FR-NET-001〜012／NFR-NET-001〜003／ARC-047／AC-NET-001〜004／TBD-NET-001/002／RSK-NET-001（已读入本计划，见下方各 Step 引用）
- Produces：`FecCodec` 接口契约（trait 方法签名，供 RGS-DTL-045 §2/§3 与 RGS-SPEC-DTL-045 §2 直接引用，不得改名）；§ 编号供 RGS-DTL-045 追溯性表引用

- [ ] **Step 1: 写文档头 + 修订历史 + 审批栏**

沿用 RGS-BAS-003 的头部结构（表格式文档编号/版本/父文档/依据标准/制定日/制定者/保密级别），具体值：

```markdown
# 基本设计书（基本設計書 / Basic Design Document）

**核心传输防丢包强化与周边协议选型 Core Transport Loss-Recovery Hardening & Peripheral Protocol Selection**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-038 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-038 需求定义书 §9 ARC-047 |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』基本设计工程 |
| 制定日 | 2026-08-25 |
| 制定者 | Ulysses(架构师兼 per DEC-008) |
| 保密级别 | 内部限定（Internal Use Only） |

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 修订内容 | 影响章节 |
|---|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses(架构师兼 per DEC-008) | 初版制定。将RGS-REQ-038 ARC-047展开为FEC编解码层组件设计、`FecCodec`可插拔接口契约、与ARC-003既有QUIC双路径的挂接点、冗余率动态调整设计方针、TBD-NET-001选型判定的输入准备。 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 确认不反转ARC-003 |
| 评审（性能） | Ulysses(性能兼 per DEC-008) | 2026-08-25 | 确认FEC解码延迟不突破NFR-PE-004与50ms tick预算 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **完整12角色兼任清单见 RGS-WBS-001 §17 集体签字声明。** |
```

- [ ] **Step 2: 写 §1 前言 + §2 与 ARC-003 的关系（不反转声明）**

```markdown
## 1. 前言

### 1.1 本文档的定位

本文档是RGS-REQ-038 §9 ARC-047（QUIC Datagram路径FEC编解码层）的系统级展开。本文档遵循RGS-BAS-001既有记述规则，不重复定义；本文档是RGS-BAS-001§3（ARC-003 QUIC双路径既有设计）的**增量扩展**，而非替代——Stream可靠路径与Datagram不可靠路径的既有行为保持不变，本文档仅在Datagram路径之上新增一个可插拔的编解码层。

## 2. 与ARC-003的关系（不变更范围声明）

对应RGS-REQ-038§9"不变更范围"栏。本设计**不得**：

- 变更QUIC作为核心传输协议的既有选择
- 变更Stream路径的可靠传输语义
- 引入任何形式的ARQ式重传逻辑到Datagram路径
- 使FEC解码失败时的行为偏离"静默丢弃，等待下一帧覆盖"（FR-NET-001）
```

- [ ] **Step 3: 写 §3 FecCodec 组件设计（含 mermaid 组件图与接口契约）**

```markdown
## 3. FecCodec 组件设计

### 3.1 组件图

\`\`\`mermaid
flowchart LR
    subgraph Sender["发送端（网关 GW / 客户端 CS）"]
        APP1[应用层状态同步数据] --> ENC[FecCodec::encode]
        ENC --> DG1[QUIC Datagram 发送]
    end
    subgraph Receiver["接收端（客户端 CS / 网关 GW）"]
        DG2[QUIC Datagram 接收] --> DEC[FecCodec::decode]
        DEC -->|恢复成功| APP2[应用层状态同步数据]
        DEC -->|恢复失败| DROP[静默丢弃,等待下一帧覆盖<br/>不触发重传,不阻塞后续帧]
    end
    DG1 -.->|不可靠传输,ARC-003既有路径不变| DG2

    classDef codec fill:#c8e6c9,stroke:#1b5e20
    class ENC,DEC codec
\`\`\`

**挂接点**：`FecCodec::encode`/`decode` 挂接于QUIC Datagram发送/接收路径的应用层出入口，**不修改**QUIC协议栈本身、不修改Stream路径。网关侧（GW）与客户端侧（CS）**必须**运行对称的编解码实现（同一 `redundancy_scheme` 版本），否则解码必然失败（归属子系统对应附件C§7 NET域"GW、CS"划分）。

### 3.2 FecCodec 接口契约

\`\`\`rust
/// FEC 编解码可插拔接口，具体实现由 TBD-NET-001 选型结果决定
/// （自研 XOR parity 或经 ARC-014 判定通过的第三方 crate）
pub trait FecCodec: Send + Sync {
    /// 编码：为原始 Datagram 附加冗余数据
    /// redundancy_rate 由 §4 动态调整算法给出，范围 [0.0, 1.0)
    fn encode(&self, payload: &[u8], redundancy_rate: f32) -> EncodedDatagram;

    /// 解码：尝试从（可能残缺的）冗余数据恢复原始 payload
    /// 解码失败返回 None，调用方必须静默丢弃（FR-NET-001），不得重传
    fn decode(&self, received: &[EncodedDatagram]) -> Option<Vec<u8>>;

    /// 解码延迟上界声明，供 §5 NFR-NET-001 校验使用
    /// 实现必须保证该值与输入块大小无关或近似常数（FR-NET-002 硬约束）
    fn max_decode_latency(&self) -> Duration;
}

pub struct EncodedDatagram {
    pub sequence: u32,       // 用于接收端识别丢失的序号
    pub redundancy_group: u32, // 冗余分组标识（自研XOR parity场景下为奇偶校验组号）
    pub data: Vec<u8>,
}
\`\`\`

**设计要点**：`max_decode_latency()` 是接口层面的硬性契约——任何未来接入的 FEC 实现（含 TBD-NET-001 选型结果）都必须能声明一个与块大小无关的延迟上界，这是 FR-NET-002"解码延迟与块大小无关"要求在接口设计上的落地，**不是**留给实现阶段自由决定的软约束。
```

- [ ] **Step 4: 写 §4 冗余率动态调整设计方针**

```markdown
## 4. 冗余率动态调整设计方针

对应FR-NET-003"应当根据实测丢包率动态调整冗余率"。

| 设计点 | 方针 |
|---|---|
| 输入信号 | 复用既有QUIC连接层已暴露的丢包率统计（ACK缺口／Datagram序号缺口检测），**不新增**独立的丢包探测机制 |
| 调整方向 | 丢包率上升→提高`redundancy_rate`（更多冗余，更强恢复能力，更高带宽开销）；丢包率下降→降低`redundancy_rate`（节省带宽，同NFR-NET-002预算约束） |
| 调整时机 | 具体的采样窗口与调整曲线参数属TBD-NET-002（依赖PH-4负载试验数据），本设计只固定"输入信号来源"与"调整方向"两项结构性决定，不预先固定数值曲线 |
| 上下界 | `redundancy_rate`必须有硬编码上界（防止极端丢包场景下冗余开销无限增长突破NFR-NET-002带宽预算），具体上界数值同属TBD-NET-002 |
```

- [ ] **Step 5: 写 §5 性能预算校验表（NFR-NET-001/002/003 逐条对应设计点）**

```markdown
## 5. 性能预算校验（NFR-NET-001〜003 落地点）

| NFR | 校验点 | 本设计如何满足 |
|---|---|---|
| NFR-NET-001（解码延迟落在50ms tick周期内，不突破NFR-PE-004） | §3.2 `max_decode_latency()` 接口契约强制声明延迟上界；FR-NET-002 禁止块式Reed-Solomon（延迟随块大小增长） | 编解码实现选型（TBD-NET-001）时，候选方案必须先通过该接口声明延迟上界，再据此判定是否满足50ms预算，不满足者不得采纳 |
| NFR-NET-002（冗余带宽落在既有NFR-PE-006预算内） | §4冗余率动态调整设计"上下界"约束 | `redundancy_rate`上界数值须保证"冗余开销峰值 × Datagram发送频率"不突破NFR-PE-006峰值20KB/s，具体上界数值留待TBD-NET-002 |
| NFR-NET-003（不得使重连恢复p99<3s劣化） | §2"不变更范围声明"——FEC层不介入重连路径（FR-NET-005现状确认：重连继续沿用`session_epoch`机制） | FEC编解码层只作用于已建连状态下的Datagram收发，重连握手阶段不经过`FecCodec`，两条路径物理隔离 |
```

- [ ] **Step 6: 写 §6 与 TBD-NET-001/RSK-NET-001 的关系（BAS 阶段不得越权拍板选型）**

```markdown
## 6. 与TBD-NET-001／RSK-NET-001的关系

本设计**只**交付§3.2的可插拔接口契约，**不**在基本设计阶段选定`FecCodec`的具体实现。理由：

- TBD-NET-001明确要求"须经ARC-014中间件导入判定"，该判定基准不属基本设计书职责范围。
- 若考虑采纳`fastnet`等第三方crate，RSK-NET-001已标注其成熟度风险（单一维护者/低下载量/创建仅数月），须在选型判定时重新核实，本设计不预先假定选型结果。
- §3.2接口契约的设计目标之一即是让选型结果（自研XOR parity 或 `fastnet`）可以在**不改变上层调用点**的前提下切换，降低TBD-NET-001判定周期对上层设计的阻塞。
```

- [ ] **Step 7: 写 §7 追溯性表**

```markdown
## 7. 追溯性（ARC-047 → 本设计书章节）

| ARC/FR/NFR编号 | 决定摘要 | 本文档展开章节 |
|---|---|---|
| ARC-047 | QUIC Datagram路径FEC编解码层 | §3、§4 |
| FR-NET-001 | FEC编解码层新增，解码失败静默丢弃不重传 | §3.1、§3.2 |
| FR-NET-002 | 禁用块式Reed-Solomon，延迟须与块大小无关 | §3.2 `max_decode_latency()` |
| FR-NET-003 | 冗余率动态调整 | §4 |
| FR-NET-004 | 编解码库选型待定，须经ARC-014判定 | §6 |
| FR-NET-005/006 | 重连与必达事件路径现状确认（不变更） | §2、§5（NFR-NET-003行） |
| NFR-NET-001〜003 | 延迟/带宽/重连性能预算 | §5 |
| AC-NET-001〜004 | 验收标准（故障注入/带宽开销/重连回归/可靠路径回归） | 本文档不重复定义验收流程，验收标准的设计前提均已在§3〜§5落地 |
| TBD-NET-001 | FEC编解码库选型 | §6 |
| TBD-NET-002 | 丢包率阈值/冗余率曲线具体参数 | §4"调整时机"行 |
| RSK-NET-001 | 第三方crate成熟度风险 | §6 |
```

- [ ] **Step 8: 运行 run_check5.py 核验新文件**

Run: `cd "D:\RustGameServer" && python "<scratchpad>/run_check5.py" 2>&1 | grep "BAS-038"`
Expected: 除既有模板级"引用 RGS-REQ-004§3.10"类问题外（如本文档头部未引用该章节则应为 0 条），无新增 [ISSUE]/[HEADER-FAIL]。

- [ ] **Step 9: Commit**

```bash
git add "docs/02-运维安全与网络/RGS-BAS-038_核心传输防丢包强化与周边协议选型_基本设计书.md"
git commit -m "docs: add RGS-BAS-038 NET/FEC basic design (ARC-047)"
```

---

### Task 2: RGS-DTL-045 详细设计书（NET/FEC 详细设计）

**Files:**
- Create: `docs/02-运维安全与网络/RGS-DTL-045_核心传输防丢包强化与周边协议选型_详细设计书.md`

**Interfaces:**
- Consumes：Task 1 产出的 `FecCodec` trait（方法名/签名不得变更）、`EncodedDatagram` struct
- Produces：XOR parity 参考实现的具体数据结构（供 RGS-SPEC-DTL-045 §2/§3 引用）；`redundancy_rate` 调整算法伪代码（供 SPEC-DTL-045 §3 引用）

- [ ] **Step 1: 写文档头 + 修订历史 + 审批栏**

```markdown
# 详细设计书（詳細設計書 / Detailed Design Document）

**核心传输防丢包强化：FecCodec 参考实现・冗余率调整算法・与QUIC Datagram发送路径挂接的具体设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-045 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-038 核心传输防丢包强化与周边协议选型 基本设计书（本文档为其详细化，不改变任何既有决定） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-25 |
| 制定者 | Ulysses(架构师兼 per DEC-008) |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses(架构师兼 per DEC-008) | — | 初版制定。细化RGS-BAS-038§3 FecCodec接口为XOR parity参考实现具体数据结构、§4冗余率调整算法伪代码、与QUIC Datagram发送/接收路径的挂接点代码级设计。**本版本不覆盖**：TBD-NET-001最终选型判定本身（若最终选型非自研XOR parity，本文档的参考实现需相应更新）、TBD-NET-002丢包率阈值/冗余率曲线的最终参数值。 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 协议字段与RGS-BAS-038§3是否逐一对应 |
| 评审（性能） | Ulysses(性能兼 per DEC-008) | 2026-08-25 | XOR parity解码延迟是否确实与块大小无关 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **完整12角色兼任清单见 RGS-WBS-001 §17 集体签字声明。** |
```

- [ ] **Step 2: 写 §1 前言（本文档不做什么）**

```markdown
## 1. 前言

### 1.1 定位

RGS-BAS-038给出了`FecCodec`接口契约、冗余率动态调整的结构性方针（输入信号/调整方向/上下界）、与ARC-003既有拓扑的挂接点。本文档将其落实为：XOR parity参考实现的具体数据结构与算法伪代码、与QUIC Datagram发送/接收路径挂接的代码级设计。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-038已确定的任何结构性选择（`FecCodec`接口方法签名、不介入重连路径、不引入ARQ重传）。
- 不做TBD-NET-001的最终选型判定——本文档给出的XOR parity是**参考实现**，用于验证§3.2接口契约的可行性与延迟特性，若ARC-014判定最终选用`fastnet`等第三方crate，替换实现需保持§3接口不变，具体替换设计留待选型判定完成后的文档修订。
- 不固定TBD-NET-002的丢包率阈值/冗余率曲线具体数值——§4给出算法结构与可调参数占位，数值本身依赖PH-4负载试验。
```

- [ ] **Step 3: 写 §2 XOR parity 参考实现数据结构**

```markdown
## 2. XOR Parity 参考实现：数据结构

对应RGS-BAS-038§3.2接口契约与§6"参考实现验证可行性"目的。

\`\`\`rust
/// 单包级XOR异或校验参考实现：每 N 个原始 Datagram 分为一组，
/// 组内异或产生 1 个校验包；接收端若组内恰好丢失 1 包，
/// 可用剩余 N-1 包 + 校验包异或还原，解码开销为 O(N) 异或操作，
/// 与"块大小"无关（不需要等待完整块到齐再解码，逐包到达即可增量异或）
pub struct XorParityCodec {
    group_size: u32,   // 每组原始包数量 N，来自 redundancy_rate 换算：group_size = ceil(1 / redundancy_rate)
}

pub struct XorParityGroupState {
    group_id: u32,
    received: Vec<Option<Vec<u8>>>,  // 长度 group_size，None 表示尚未收到/已丢失
    parity: Option<Vec<u8>>,          // 本组校验包，收到即存
}

impl FecCodec for XorParityCodec {
    fn encode(&self, payload: &[u8], redundancy_rate: f32) -> EncodedDatagram {
        // 实现要点：将 payload 归入当前组，组内累积异或；
        // 组满 group_size 个原始包后，额外发送 1 个校验 EncodedDatagram（redundancy_group 标记该组）
        // 具体分组/发送时机的状态机在 §3 展开
        unimplemented!("状态机细节见 §3，此处仅声明数据结构")
    }

    fn decode(&self, received: &[EncodedDatagram]) -> Option<Vec<u8>> {
        // 若组内 group_size 个原始包全部收到：直接返回，无需借助 parity
        // 若恰好缺 1 个原始包且 parity 已收到：用其余包异或 parity 还原缺失包
        // 若缺 ≥2 个包：XOR parity 无法恢复（这是该方案的已知能力上限，非bug）
        //   → 按 FR-NET-001 返回 None，调用方静默丢弃
        unimplemented!("完整状态机见 §3")
    }

    fn max_decode_latency(&self) -> Duration {
        // 与 group_size 无关的常数：一次异或运算耗时（微秒级），
        // 不依赖"等待整组到齐"——解码只在"组内恰好缺1包"场景触发，
        // 且触发时刻是"收到该组第 group_size-1 个包（含parity）"，
        // 不需要额外等待，故延迟上界为固定的 O(1) 异或耗时，与 group_size 无关，
        // 满足 RGS-BAS-038 §3.2 硬约束（FR-NET-002）
        Duration::from_micros(50)  // 参考实现的保守估计值，具体实测值见 RGS-SPEC-DTL-045 §8 Gate 证据
    }
}
\`\`\`

**关键设计说明（回应 FR-NET-002"延迟与块大小无关"）**：XOR parity 的解码不是"等整组 N 个包都到齐再统一处理"，而是**逐包增量异或**——每收到一个包就立即与已累积的异或值做一次异或运算，组内恰好缺 1 包时，用"parity ⊕ 已收到的 N-1 个包"直接还原，单次异或运算耗时是常数，不随 `group_size`（对应 KCP 块式方案的"块大小"概念）增长而增长。这与 RGS-BAS-038 否决"块式 Reed-Solomon"（解码延迟随块大小增长）的理由形成对照。
```

- [ ] **Step 4: 写 §3 编解码状态机（发送/接收两侧时序）**

```markdown
## 3. 编解码状态机

### 3.1 发送端状态机

\`\`\`mermaid
stateDiagram-v2
    [*] --> Accumulating: 新分组开始 (group_id++)
    Accumulating --> Accumulating: encode(payload) 调用<br/>累积异或 running_xor ^= payload<br/>正常发送该原始 Datagram
    Accumulating --> SendParity: 组内已发送 group_size 个原始包
    SendParity --> Accumulating: 发送校验 EncodedDatagram<br/>(redundancy_group=group_id, data=running_xor)<br/>重置 running_xor, 新分组开始
\`\`\`

### 3.2 接收端状态机

\`\`\`mermaid
stateDiagram-v2
    [*] --> Waiting: 新分组到达首个包
    Waiting --> Waiting: 收到组内原始包/校验包<br/>写入 XorParityGroupState.received[i]
    Waiting --> Complete: 组内 group_size 个原始包全部收到<br/>(无需借助parity)
    Waiting --> Recoverable: 组内恰好缺1个原始包<br/>且 parity 已收到
    Waiting --> Unrecoverable: 组内缺≥2个原始包<br/>或 parity 未收到而原始包也不全
    Recoverable --> Complete: 用剩余包异或parity还原缺失包<br/>(decode() 返回 Some)
    Unrecoverable --> Dropped: decode() 返回 None<br/>调用方静默丢弃(FR-NET-001)<br/>不重传,不阻塞后续分组
    Complete --> [*]
    Dropped --> [*]
\`\`\`

**边界条件说明**：

- 分组超时：若一个分组长时间未收全（如网络抖动导致乱序），接收端**必须**设置分组存活窗口（具体窗口时长属实现调优参数，本文档不预先固定数值，留待 RGS-SPEC-DTL-045 §3 或实现阶段），超窗后即使后续包到达也不再尝试恢复该组，直接进入 `Dropped`——这是"不阻塞后续帧"（FR-NET-001）在乱序场景下的具体落地。
- `group_id` 单调递增，接收端按 `group_id` 而非到达顺序归组，容忍 QUIC Datagram 乱序到达（Datagram 路径本身不保证顺序，这是 ARC-003 既有前提，本设计不改变）。
```

- [ ] **Step 5: 写 §4 冗余率动态调整算法伪代码**

```markdown
## 4. 冗余率动态调整算法

对应RGS-BAS-038§4"输入信号来源"与"调整方向"两项结构性决定的代码级落地。

\`\`\`rust
struct RedundancyAdjuster {
    current_rate: f32,        // 当前 redundancy_rate，初始值 TBD-NET-002
    min_rate: f32,             // 下界，TBD-NET-002
    max_rate: f32,              // 上界，TBD-NET-002（须满足 RGS-BAS-038§5 NFR-NET-002 校验）
    sample_window: Duration,     // 丢包率采样窗口，TBD-NET-002
}

impl RedundancyAdjuster {
    /// 复用既有 QUIC 连接层丢包率统计（不新增独立探测机制，同 RGS-BAS-038§4）
    fn adjust(&mut self, observed_loss_rate: f32) {
        // 结构性方向：丢包率上升→提高冗余；下降→降低冗余
        // 具体的映射函数（线性/阶梯/其他）与调整幅度步长同属TBD-NET-002，
        // 此处给出结构，不预先固定映射函数形态
        let target_rate = compute_target_rate(observed_loss_rate); // 具体函数留待TBD-NET-002确定
        self.current_rate = target_rate.clamp(self.min_rate, self.max_rate);
    }
}
\`\`\`

**与 RGS-BAS-038§5 的联动**：`max_rate` 的具体数值必须先满足"冗余开销峰值 × Datagram 发送频率 ≤ NFR-PE-006 峰值 20KB/s"的约束方程，再据此反推允许的最大 `group_size`（`redundancy_rate ≈ 1/group_size`）——该反推过程属TBD-NET-002负载试验的一部分，本文档只固定约束方程的结构。
```

- [ ] **Step 6: 写 §5 与 QUIC Datagram 路径的挂接点**

```markdown
## 5. 与QUIC Datagram发送/接收路径的挂接点

对应RGS-BAS-038§3.1组件图"挂接点"说明，落实为代码级挂接位置。

\`\`\`rust
// 发送端：既有 QUIC Datagram 发送路径的应用层出口新增 FecCodec::encode 调用
fn send_state_sync_datagram(conn: &QuicConnection, payload: &[u8], adjuster: &RedundancyAdjuster) {
    let encoded = fec_codec.encode(payload, adjuster.current_rate);
    conn.send_datagram(encoded.into_wire_bytes());  // 既有 ARC-003 Datagram 发送 API，不变更
}

// 接收端：既有 QUIC Datagram 接收路径的应用层入口新增 FecCodec::decode 调用
fn on_datagram_received(conn: &QuicConnection, wire_bytes: &[u8]) {
    let encoded = EncodedDatagram::from_wire_bytes(wire_bytes);
    group_buffer.insert(encoded);
    if let Some(payload) = fec_codec.decode(group_buffer.current_group_packets()) {
        deliver_to_application(payload);  // 既有应用层状态同步处理路径，不变更
    }
    // decode 返回 None 时不调用 deliver_to_application，等价于静默丢弃（FR-NET-001）
}
\`\`\`

**不变更声明**：`conn.send_datagram` / `deliver_to_application` 均为 ARC-003 既有 API，本文档不修改其签名或行为，`FecCodec` 只插入在"应用层 payload"与"QUIC wire bytes"之间的编解码步骤。
```

- [ ] **Step 7: 写 §6 本文档覆盖范围与后续计划**

```markdown
## 6. 本文档的覆盖范围与后续计划

本文档覆盖：XOR parity 参考实现数据结构（§2）、编解码状态机（§3，含分组超时边界条件）、冗余率动态调整算法结构（§4）、与QUIC Datagram路径的代码级挂接点（§5）。

本版本明确不覆盖、留待后续：

- TBD-NET-001最终选型判定——若判定结果非自研XOR parity，需要新增对应实现设计（本文档§2〜§3作为接口可行性参考，不因选型结果被推翻）。
- TBD-NET-002的具体数值（分组存活窗口时长、`min_rate`/`max_rate`/`sample_window`、`compute_target_rate`映射函数形态）——依赖PH-4负载试验数据。
- RSK-NET-001第三方crate成熟度复核——若TBD-NET-001选定`fastnet`，须在采纳前重新核实其crates.io最新状态。
```

- [ ] **Step 8: 写追溯性表**

```markdown
## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-038§3 FecCodec组件设计 | §2、§5 |
| RGS-BAS-038§4 冗余率动态调整设计方针 | §4 |
| RGS-BAS-038§5 性能预算校验 | §2"关键设计说明"、§4"与RGS-BAS-038§5的联动" |
| RGS-BAS-038§6 与TBD-NET-001/RSK-NET-001的关系 | §1.2、§6 |
```

- [ ] **Step 9: 运行 run_check5.py 核验新文件，并核对 §3.2/§4 接口签名与 RGS-BAS-038 §3.2 逐字一致**

Run: `cd "D:\RustGameServer" && python "<scratchpad>/run_check5.py" 2>&1 | grep "DTL-045"`
Expected: 无新增 [ISSUE]/[HEADER-FAIL]（模板级§3.10问题除外）。
人工核对：`FecCodec` trait 的 `encode`/`decode`/`max_decode_latency` 三个方法名与参数类型在 RGS-BAS-038 与 RGS-DTL-045 中完全一致（禁止如 `decode()` vs `try_decode()` 的漂移）。

- [ ] **Step 10: Commit**

```bash
git add "docs/02-运维安全与网络/RGS-DTL-045_核心传输防丢包强化与周边协议选型_详细设计书.md"
git commit -m "docs: add RGS-DTL-045 NET/FEC detailed design"
```

---

### Task 3: RGS-SPEC-DTL-045 实现规格书

**Files:**
- Create: `docs/13-实现规格/RGS-SPEC-DTL-045_实现规格书.md`

**Interfaces:**
- Consumes：Task 2 的 `FecCodec`/`XorParityCodec`/`RedundancyAdjuster` 类型定义
- Produces：无下游任务消费（流水线末端）

- [ ] **Step 1: 写文档头（沿用既有 SPEC-DTL-04x 系列模板，见本会话此前创建的 SPEC-DTL-043/044/100/101/102）**

```markdown
# RGS-DTL-045 实现规格书

**RGS-SPEC-DTL-045**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-045 |
| 版本 | 0.1 |
| 状态 | 规格草案，待 RGS-DTL-045 具名 DD Review |
| 源详细设计 | RGS-DTL-045（核心传输防丢包强化：FecCodec 参考实现・冗余率调整算法） |
| 实现范围 | `rgs-net-fec` crate（或等效模块，沿用既有 workspace 命名约定）：`FecCodec` trait + `XorParityCodec` 参考实现 + `RedundancyAdjuster` + 网关/客户端两侧 QUIC Datagram 路径挂接点 |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、既有 `quinn`/`s2n-quic` 等 QUIC 库基线（沿用既有网关/客户端实现依赖，不新增） |
| 规格真源 | 源 DTL 的 XOR parity 数据结构（§2）、编解码状态机（§3）、冗余率调整算法（§4）、QUIC 挂接点（§5） |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-25 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **完整12角色兼任清单见 RGS-WBS-001 §17 集体签字声明。** |
```

- [ ] **Step 2: 写 §1 使用规则**

```markdown
## 1. 使用规则

本规格把 RGS-DTL-045 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-045 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 `FecCodec` trait（DTL §2 接口，方法名/签名逐字一致）、`XorParityCodec` 参考实现（DTL §2/§3 完整状态机）、`RedundancyAdjuster`（DTL §4）、网关侧与客户端侧对称挂接（DTL §5）；**不得**在实现阶段引入 DTL 未定义的额外重传/ACK 机制（回到 KCP-ARQ 反模式，违反 RGS-REQ-038§4 判定）；**不得**跳过 TBD-NET-001 选型判定直接把 XOR parity 参考实现当作生产终态硬编码——参考实现的可替换性（trait 抽象）必须保留。
```

- [ ] **Step 3: 写 §2 实现单元**

```markdown
## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 编解码 trait | `rgs-net-fec` crate 内 `codec` 模块：`FecCodec` trait | 方法签名与 DTL §2 逐字一致 |
| XOR parity 参考实现 | `rgs-net-fec::xor_parity`：`XorParityCodec`/`XorParityGroupState` | 数据结构与 DTL §2 一致；状态机（Accumulating/SendParity/Waiting/Complete/Recoverable/Unrecoverable/Dropped）与 DTL §3 mermaid 图逐状态对应 |
| 冗余率调整 | `rgs-net-fec::adjuster`：`RedundancyAdjuster` | 字段/`adjust()`方法与 DTL §4 一致；`min_rate`/`max_rate`/`sample_window` 数值待 TBD-NET-002 输入，实现阶段留可配置项，不硬编码占位数值 |
| 网关侧挂接 | 网关（GW）QUIC Datagram 发送/接收路径（既有网关服务内，具体文件路径待网关服务代码结构确定） | 挂接点与 DTL §5 一致，不修改既有 `conn.send_datagram`/应用层交付函数签名 |
| 客户端侧挂接 | 客户端 SDK（CS）QUIC Datagram 发送/接收路径（既有客户端 SDK crate 内） | 与网关侧运行**对称**的 `redundancy_scheme` 版本（DTL §3.1 边界条件），版本不匹配须有显式协商或降级为不启用 FEC 的兜底路径（协商机制细节属实现阶段设计，非 DTL 已固定决策，本规格标注为待实现阶段细化项，不作为 Gate 条件） |
| CI | fmt、clippy、test、deny checks | 负例必须阻断合并 |
```

- [ ] **Step 4: 写 §3 实现契约**

```markdown
## 3. 实现契约

- `FecCodec::decode` 返回 `None` 时，调用方**必须**静默丢弃且**不得**触发任何重传或阻塞后续 Datagram 处理（DTL §3.2 `Unrecoverable → Dropped` 路径，FR-NET-001 硬约束）。
- `XorParityCodec` 的分组**必须**按 `group_id` 单调递增归组，**不得**假设 Datagram 到达顺序（QUIC Datagram 路径本身不保证顺序，ARC-003 既有前提）。
- 分组存活窗口超时后，**不得**再尝试恢复该组（即使后续包到达），**必须**直接判定为 `Dropped`（DTL §3.2 边界条件，防止无界等待累积内存/阻塞后续分组处理）。
- `RedundancyAdjuster::adjust` 的丢包率输入**必须**复用既有 QUIC 连接层统计，**不得**新增独立丢包探测逻辑（DTL §4"复用既有..."硬约束）。
- `current_rate` 每次调整后**必须**经 `clamp(min_rate, max_rate)`，**不得**允许冗余率突破 `max_rate`（防止极端丢包场景下带宽开销失控，突破 NFR-NET-002）。
- 网关侧与客户端侧**必须**运行对称的编解码实现版本；版本不匹配时**不得**产生"部分包能解码、部分包不能解码"的不确定行为——须有明确的版本协商或整体降级为不启用 FEC（透传原始 Datagram，等价于 ARC-003 既有行为，不引入新的失败模式）。
- `max_decode_latency()` 返回值**必须**通过实测校验（不得仅凭理论推导），实测值须录入 §8 Gate 证据，且必须满足 RGS-BAS-038§5 的 50ms tick 周期约束。
```

- [ ] **Step 5: 写 §4 可观测性规格**

```markdown
## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 指标（`net_fec_*`）：编码/解码调用计数、解码成功率（按 `Recoverable`/`Unrecoverable` 分组）、当前 `redundancy_rate`（按连接分组的瞬时值，用于验证 §3 动态调整生效）、分组超时丢弃计数。
- 指标标签：仅 `outcome`（success/unrecoverable/timeout）等低基数标签；连接标识（`connection_id`）等高基数信息**不**作为 metric label。
- 解码延迟（`max_decode_latency` 实测分布）须纳入现有 p99 延迟监控体系，用于持续校验 NFR-NET-001 未被劣化。
```

- [ ] **Step 6: 写 §5 安全、容错与发布**

```markdown
## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 容错 | XOR parity 组内缺≥2包场景下 `decode()` 正确返回 `None`（不崩溃、不 panic）；分组存活窗口超时清理逻辑不产生内存泄漏（长期运行的分组缓冲区大小有界） |
| 一致性 | 网关/客户端版本不对称时的降级路径（透传原始 Datagram）必须有集成测试覆盖，防止"静默产生错误恢复结果"这一比"不恢复"更危险的失败模式 |
| 性能回归 | `max_decode_latency()` 实测值的 CI 基准测试（`cargo bench` 或等效），防止未来实现变更悄悄突破 50ms tick 预算 |
| 数据治理 | FEC 冗余数据本身不携带业务语义（纯异或字节），无 PII，无需脱敏 |
| 发布 | FEC 层默认**关闭**（feature flag 或配置开关），需先在非生产环境验证 AC-NET-001〜004 通过后再逐步灰度开启，不得一次性全量启用（与既有 QUIC Datagram 路径的现有行为形成对照基线，便于回归判定） |
```

- [ ] **Step 7: 写 §6 测试规格**

```markdown
## 6. 测试规格

- UT：覆盖 `XorParityCodec` 编解码状态机全部转移路径（Complete/Recoverable/Unrecoverable）+ 分组存活窗口超时清理 + `RedundancyAdjuster::adjust` 的 clamp 边界。
- IT：覆盖网关↔客户端对称编解码全链路（模拟丢包场景，验证 AC-NET-001 恢复比例）+ 版本不对称降级路径。
- Performance：`max_decode_latency()` 实测基准测试，验证与 `group_size`（冗余率）无关（多组不同 `redundancy_rate` 下延迟应近似常数，验证 FR-NET-002 核心主张）。
- ST（对应 AC-NET-001〜004）：
  - AC-NET-001：既定丢包率区间故障注入，验证不触发重传的前提下恢复约定比例的丢失 Datagram，且延迟满足 NFR-NET-001。
  - AC-NET-002：带宽开销试验，验证冗余开销落在 NFR-NET-002 预算内且随丢包率下降而降低（验证 §3 `RedundancyAdjuster` 动态调整生效）。
  - AC-NET-003：重连回归试验，验证新增 FEC 层后重连恢复时延不劣于 NFR-NET-003 既定基线（FEC 层不介入重连路径的隔离性验证）。
  - AC-NET-004：可靠路径回归试验，验证必达事件（Stream 路径）传输行为不受本变更影响。

测试必须回填 RGS-REQ-004 追踪矩阵（AC-NET-001〜004）和 DTL-045 各章节验收项；不能只证明"服务启动"。
```

- [ ] **Step 8: 写 §7 Definition of Done**

```markdown
## 7. Definition of Done

- RGS-DTL-045 的 XOR parity 数据结构、状态机、冗余率调整算法、QUIC 挂接点与实现逐项对账。
- Cargo fmt、clippy、test、deny 检查通过。
- AC-NET-001〜004 全部通过且有实测证据（非理论推导）。
- 网关/客户端版本对称性校验与降级路径集成测试通过。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。
```

- [ ] **Step 9: 写 §8 Gate 证据与实测参数**

```markdown
## 8. Gate 证据与实测参数

进入实现前必须取得：① 源 DTL RGS-DTL-045 的具名 DD Review；② **TBD-NET-001 选型判定完成**（ARC-014 判定通过，若非自研 XOR parity 则本规格的§2/§3 实现单元需相应更新后才可进入生产实现，当前 XOR parity 仅为参考实现，未经 ARC-014 判定前不得直接用于生产路径）；③ `max_decode_latency()` 实测值（须录入具体数值，验证是否满足 RGS-BAS-038§5 的 50ms tick 约束）；④ AC-NET-001〜004 在非生产环境的完整故障注入实测报告。**本规格不覆盖**：TBD-NET-002 丢包率阈值/冗余率曲线的最终参数值——该值依赖 PH-4 负载试验数据，本规格的 `min_rate`/`max_rate`/`sample_window` 仅提供可配置结构，最终数值由 PH-4 试验结果回填，不作为本规格的 Gate 阻塞条件；RSK-NET-001 第三方 crate 成熟度复核——仅在 TBD-NET-001 选型结果为第三方 crate 时才适用，若选型结果为自研 XOR parity 则该风险不适用。
```

- [ ] **Step 10: 运行 run_check5.py 核验；核对 §3 实现契约的每一条"必须/不得"均可在 DTL-045 中找到逐条对应原文（不得杜撰）**

Run: `cd "D:\RustGameServer" && python "<scratchpad>/run_check5.py" 2>&1 | grep "SPEC-DTL-045"`
Expected: 无新增 [ISSUE]/[HEADER-FAIL]（模板级§3.10问题除外）。

- [ ] **Step 11: Commit**

```bash
git add "docs/13-实现规格/RGS-SPEC-DTL-045_实现规格书.md"
git commit -m "docs: add RGS-SPEC-DTL-045 NET/FEC implementation spec"
```

---

### Task 4: RGS-BAS-003 v0.3 修订（新增 DNS/CDN 状态只读查询方法）

**Files:**
- Modify: `docs/02-运维安全与网络/RGS-BAS-003_运维与GM后台管控_基本设计书.md`

**Interfaces:**
- Consumes：既有 `AdminService` 接口定义（§3，不得修改既有方法签名）
- Produces：`QueryDnsStatus`/`QueryCdnEdgeStatus` 方法定义（供 RGS-DTL-003 §3.5 与既有 SPEC-DTL-003 引用）

- [ ] **Step 1: 在修订历史表追加 v0.3 行**

Modify `docs/02-运维安全与网络/RGS-BAS-003_运维与GM后台管控_基本设计书.md` 第 19-20 行之后（既有修订历史表内），新增一行：

```markdown
| 0.3 | 2026-08-25 | Ulysses(架构师兼 per DEC-008) | 依RGS-REQ-007-ADD1新增：DNS解析状态只读查询、CDN边缘节点状态只读查询（复用既有`DistributionBackend`指标）、异常告警复用既有FR-GM-004通道。范围外：DNS/CDN写路径不新增。 | §3.4新增子节§3.5、§6、§8、§13 |
```

同时更新文档头版本号：`| 版本 | 0.2 |` → `| 版本 | 0.3 |`。

- [ ] **Step 2: 更新目录，插入 §3.5 条目**

Modify 目录列表第 37 行 `3. [AdminService 字段级API扩展设计](#3-adminservice-字段级api扩展设计)` 之后，无需新增顶级目录项（§3.5 是 §3 的子节，沿用既有目录粒度不逐子节列出，与既有 §3.1〜3.4 一致处理）。

- [ ] **Step 3: 在 §3（AdminService 字段级API扩展设计）末尾、§4 之前新增 §3.5**

Modify：在文件第 169-171 行（§3.4 表格结束、`---` 分隔符、`# 4. 运行时受限控制通道设计` 标题之间）插入：

```markdown
## 3.5 DNS/CDN 基础设施状态只读查询方法（RGS-REQ-007-ADD1，v0.3新增）

对应RGS-REQ-007-ADD1 FR-OPS-005〜007。以下方法**新增**至`AdminService`，均为只读、无侧作用查询，不新增任何DNS/CDN写路径。

| Method | 请求字段 | 响应字段 | 对应功能 |
|---|---|---|---|
| `QueryDnsStatus` | `request_id`（可选，幂等键非必需——纯查询无副作用）／`domain_filter`（可选，指定域名子集，为空则查全部既定域名集合） | `results[]`（`domain`／`region`／`resolved_ips[]`／`baseline_ips[]`／`is_anomalous`（布尔，与基线不一致时为true）／`checked_at`） | FR-OPS-005 |
| `QueryCdnEdgeStatus` | `request_id`（可选）／`backend_filter`（可选，指定`DistributionBackend`子集） | `results[]`（`backend_name`／`edge_hit_rate`／`origin_fetch_success_rate`（均取自既有NFR-CDN-101/102指标）／`is_anomalous`（布尔，任一指标低于NFR-CDN-101/102阈值时为true）／`checked_at`） | FR-OPS-006 |

**设计要点**：两个方法均遵循与`QueryHealthView`（§3.4）相同的设计原则——**不直接探活**DNS/CDN基础设施本身，而是从既有探测/指标数据源聚合展示：`QueryDnsStatus`的数据源是FR-OPS-005新增的多地理位置探测任务（探测节奏复用FR-OPS-001既有轮询机制，探测间隔≤5分钟，NFR-OPS-009），`QueryCdnEdgeStatus`的数据源是既有`DistributionBackend`已暴露的指标（NFR-CDN-101/102），**不新增**独立的CDN指标采集SDK（FR-OPS-006硬约束）。探测流量与生产玩家流量隔离（NFR-OPS-010），具体隔离机制（独立探测节点/独立网络路径）属详细设计阶段展开（见RGS-DTL-003 v0.2修订）。

**权限边界**：两个方法默认仅SRE/运维角色可见（NFR-OPS-011，沿用ARC-019控制平面统一入口原则），RBAC矩阵扩展见§8修订。
```

- [ ] **Step 4: 在 §6（告警与事件推送设计）末尾追加 DNS/CDN 告警复用说明**

Modify：在文件第 260-262 行（§6.2设计原则表格结束、`---`分隔符之前）插入：

```markdown
## 6.3 DNS/CDN异常告警（RGS-REQ-007-ADD1，v0.3新增）

对应FR-OPS-007。DNS解析异常（与基线不一致）或CDN边缘异常（命中率/回源成功率低于NFR-CDN-101/102阈值）**复用**本节§6.1既有告警数据流与Webhook推送通道（IF-008），**不新增**独立告警链路——具体而言，§3.5新增的探测任务在检出异常时，按既有"规则命中→告警事件→Webhook分发"路径推送，异常检出本身即等价于§6.1"规则命中"节点，不需要额外的告警规则引擎接入。
```

- [ ] **Step 5: 在 §8（RBAC 角色矩阵扩展）追加 DNS/CDN 查询权限行**

§8.1角色矩阵表当前最后一行是`| 运维工单发起（新增） | ... |`（第287行），表格之后是空行（288行）、`## 8.2 高危操作判定与二次确认流程`标题（289行）。**不得**在表格中间插入新表格（会破坏既有表格的 Markdown 结构）——正确做法是：① 在既有表格**末尾**（第287行之后、288行空行之前）追加一行新角色；② 在表格结束、`## 8.2`标题之前插入一段说明。

Modify：在文件第287行`| 运维工单发起（新增） | \`CreateOpsTicket\` | ... |`行之后追加表格新行：

```markdown
| **DNS/CDN状态查看（新增，v0.3）** | `QueryDnsStatus`／`QueryCdnEdgeStatus` | 默认仅SRE/运维角色，与既有"只读查看"角色权限范围不重叠（NFR-OPS-011），非SRE/运维角色调用返回权限拒绝（AC-OPS-008验证点） |
```

再在该表格结束、`## 8.2`标题之前（即原288行空行处）插入说明段落：

```markdown
> **v0.3修订说明**：`QueryDnsStatus`／`QueryCdnEdgeStatus`（§3.5）**不**归入既有"只读查看"角色的默认权限范围——NFR-OPS-011要求默认仅SRE/运维角色可见，与"只读查看"角色（面向更广泛的GM操作者）的既有权限粒度不同，故单独新增一行角色而非扩展"只读查看"行。
```

- [ ] **Step 6: 在 §13（追溯性表）追加 RGS-REQ-007-ADD1 映射行**

Modify：在文件第 371 行 `| FR-OPS-001〜004 | ...` 行之后插入：

```markdown
| **FR-OPS-005〜007（RGS-REQ-007-ADD1，v0.3新增）** | **DNS/CDN基础设施状态只读可观测性** | **§3.5、§6.3** |
| **NFR-OPS-009〜011（RGS-REQ-007-ADD1，v0.3新增）** | **探测间隔/流量隔离/RBAC** | **§3.5** |
| **AC-OPS-006〜008（RGS-REQ-007-ADD1，v0.3新增）** | **DNS异常检出告警、CDN异常检出告警、RBAC拒绝验证** | **§3.5、§6.3、§8** |
```

- [ ] **Step 7: 运行 run_check5.py 核验修订**

Run: `cd "D:\RustGameServer" && python "<scratchpad>/run_check5.py" 2>&1 | grep "BAS-003"`
Expected: 无新增 [ISSUE]（既有 §3.10 模板问题若本文档此前未触发则应继续保持 0）。

- [ ] **Step 8: Commit**

```bash
git add "docs/02-运维安全与网络/RGS-BAS-003_运维与GM后台管控_基本设计书.md"
git commit -m "docs: extend RGS-BAS-003 v0.3 with DNS/CDN read-only status queries (REQ-007-ADD1)"
```

---

### Task 5: RGS-DTL-003 v0.2 修订（DNS/CDN 协议线格式与探测隔离设计）

**Files:**
- Modify: `docs/02-运维安全与网络/RGS-DTL-003_详细设计书.md`

**Interfaces:**
- Consumes：Task 4 的 `QueryDnsStatus`/`QueryCdnEdgeStatus` 方法定义（字段名不得变更）
- Produces：Protobuf 线格式（供未来实现阶段直接引用；本计划不修改 RGS-SPEC-DTL-003，因其当前 85 行内容未涉及 §3.4/§3.5 查询类方法的协议细节，追加内容留待 SPEC-DTL-003 独立修订，不在本计划范围内——见 Task 5 Step 6 说明）

- [ ] **Step 1: 在修订历史表追加 v0.2 行**

Modify 文件第 22 行既有修订历史表之后，新增一行：

```markdown
| 0.2 | 2026-08-25 | Ulysses(架构师兼 per DEC-008) | — | 依RGS-BAS-003 v0.3新增：`QueryDnsStatus`/`QueryCdnEdgeStatus`协议线格式（§3.5）、DNS/CDN探测流量隔离的具体设计（§8，落地NFR-OPS-010）。**本版本不覆盖**：DNS探测的具体实现方式（自建探测节点 vs. 第三方DNS监控服务，TBD-OPS-004）、CDN边缘指标接口形式的最终确定（TBD-OPS-005）。 | §3.5（新增）、§8（新增） |
```

同时更新文档头版本号：`| 版本 | 0.1 |` → `| 版本 | 0.2 |`。

- [ ] **Step 2: 更新目录，追加 §8 条目**

Modify 文件第 42 行 `7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)` 之后插入：

```markdown
8. [DNS/CDN状态查询协议线格式与探测隔离设计（RGS-REQ-007-ADD1）](#8-dnscdn状态查询协议线格式与探测隔离设计rgs-req-007-add1)
```

- [ ] **Step 3: 在 §3.4（查询/审计方法）Protobuf 代码块末尾追加 DNS/CDN 消息定义**

Modify：在文件第 271-282 行（`QueryHealthViewRequest`/`QueryHealthViewResponse` 定义之后、代码块结束 ``` 之前）追加：

```protobuf
message QueryDnsStatusRequest {
  string request_id     = 1;   // 可选，纯查询无副作用
  repeated string domain_filter = 2;  // 可选，为空则查全部既定域名集合
}
message QueryDnsStatusResponse {
  repeated DnsStatusEntry results = 1;
}
message DnsStatusEntry {
  string domain               = 1;
  string region                  = 2;
  repeated string resolved_ips      = 3;
  repeated string baseline_ips         = 4;
  bool   is_anomalous                     = 5;
  int64  checked_at_ms                       = 6;
}

message QueryCdnEdgeStatusRequest {
  string request_id        = 1;   // 可选
  repeated string backend_filter = 2;  // 可选，为空则查全部DistributionBackend
}
message QueryCdnEdgeStatusResponse {
  repeated CdnEdgeStatusEntry results = 1;
}
message CdnEdgeStatusEntry {
  string backend_name                  = 1;
  double edge_hit_rate                    = 2;   // 取自既有NFR-CDN-101指标
  double origin_fetch_success_rate           = 3;  // 取自既有NFR-CDN-102指标
  bool   is_anomalous                           = 4;
  int64  checked_at_ms                             = 5;
}
```

紧随其后在正文（代码块之外）追加说明：

```markdown
字段编号规则延续§1.3：`request_id`/`domain_filter`/`backend_filter`等高频字段占1〜N，`is_anomalous`/`checked_at_ms`等低频判定字段紧随其后，不预留16+区间（本组消息字段数量少，不适用§3.4`QueryOnlineStatusRequest`分页场景的10+/11+分组惯例）。
```

- [ ] **Step 4: 新增 §8 DNS/CDN 探测流量隔离设计**

Modify：在文件第 419-421 行（§7"后续详细设计建议顺序"段落结束、`---`分隔符之前）插入新章节：

```markdown
---

## 8. DNS/CDN状态查询协议线格式与探测隔离设计（RGS-REQ-007-ADD1）

对应RGS-BAS-003§3.5、NFR-OPS-009〜011。

### 8.1 探测流量隔离设计（NFR-OPS-010落地）

| 设计点 | 方针 |
|---|---|
| 网络路径隔离 | DNS多地理位置探测与CDN边缘指标读取**均不经过**生产玩家流量路径（既有网关/运行时集群）——DNS探测是独立的出站解析请求（探测节点向公网DNS服务器发起查询，不途经本系统网关），CDN边缘指标是**读取**既有`DistributionBackend`已采集的存量指标（数据源见RGS-BAS-027§6.1 `DistributionBackend`接口契约 + RGS-REQ-030_addendum_CDN边缘策略已定义的NFR-CDN-101/102指标采集管线，非新建），不产生额外的CDN边缘请求流量 |
| 探测节奏 | 复用FR-OPS-001既有轮询机制，探测间隔**不得**超过5分钟（NFR-OPS-009），具体探测节点部署方式（自建 vs. 第三方）为TBD-OPS-004，本文档不预先决定 |
| 资源配额 | DNS探测任务须有独立资源配额（探测节点或探测进程），**不得**与生产路径共享连接池/线程池，防止探测任务异常（如探测目标不可达导致的重试风暴）反向影响生产性能 |

### 8.2 与既有CDN指标管线的关系

`QueryCdnEdgeStatus`的数据源复用RGS-BAS-027§6.1已定义的`DistributionBackend`抽象接口所暴露的、RGS-REQ-030_addendum_CDN边缘策略_需求定义书已定义的NFR-CDN-101（边缘命中率）/NFR-CDN-102（回源成功率）指标，本文档**不重复**定义该指标的采集/暴露机制，仅新增GM后台侧的只读查询接口（§3.4 Protobuf定义）与异常判定阈值引用（判定阈值复用NFR-CDN-101/102既定值，本文档不重新定义阈值本身）。**订正说明**：RGS-BAS-036/RGS-DTL-041（客户端断点续传设计）虽然也引用`DistributionBackend`，但只是该抽象的另一个消费方（Range支持扩展），并非`DistributionBackend`或NFR-CDN-101/102的定义来源，本文档不应引用它们作为数据源依据。

### 8.3 与TBD-OPS-004/005的关系

本文档**不**对TBD-OPS-004（DNS探测具体实现方式）、TBD-OPS-005（CDN边缘指标接口形式最终确定）作出选型判定，理由与§7"本版本不覆盖"一致——两项均待独立评审/选型判定完成后再补充实现级细节，本节§8.1/§8.2给出的是不依赖该判定结果的结构性设计（隔离原则、数据源复用关系），选型结果确定后不需要推翻本节已有内容，只需补充具体实现方式。
```

- [ ] **Step 5: 更新追溯性表**

Modify：在文件第 434 行 `| RGS-DTL-025§5 / RGS-DTL-026§4.1... | §6 |` 行之后插入：

```markdown
| RGS-BAS-003§3.5（v0.3新增，DNS/CDN只读查询） | §3.4（Protobuf定义追加）、§8 |
| RGS-BAS-003§6.3（v0.3新增，DNS/CDN告警复用） | 前提依赖，本文档不重复定义告警路径本身 |
```

- [ ] **Step 6: 说明 SPEC-DTL-003 是否需要联动修订（决策记录，非本计划任务）**

读取现有 `docs/13-实现规格/RGS-SPEC-DTL-003_实现规格书.md`（85行），确认其 §2/§3 是否逐条列举了 `AdminService` 全部方法的实现契约。若其内容是方法级枚举式契约（类似本计划 Task 3 SPEC-DTL-045 §3 的写法），则 DNS/CDN 新方法应追加对应契约行；若其内容是通用性契约（不逐方法枚举），则本次修订不需要触碰 SPEC-DTL-003。**本步骤只做判断记录，不在本计划内执行 SPEC-DTL-003 的修改**——若判断结果为"需要修订"，应作为独立后续任务提出，不在当前计划范围内擅自展开（避免计划范围蔓延）。

- [ ] **Step 7: 运行 run_check5.py 核验修订**

Run: `cd "D:\RustGameServer" && python "<scratchpad>/run_check5.py" 2>&1 | grep "DTL-003"`
Expected: 无新增 [ISSUE]。

- [ ] **Step 8: Commit**

```bash
git add "docs/02-运维安全与网络/RGS-DTL-003_详细设计书.md"
git commit -m "docs: extend RGS-DTL-003 v0.2 with DNS/CDN protocol wire format and probe isolation design (REQ-007-ADD1)"
```

---

### Task 6: 全量交叉核验 + 附件C一致性复核

**Files:**
- Read-only verification, no new file changes expected (附件C已在此前"自审"阶段完成v3.11登记，本任务只做核验，不修改)

**Interfaces:**
- Consumes：Task 1〜5 全部产出文件
- Produces：核验结论（若发现不一致，转为新的修订任务，不在本计划内直接展开）

- [ ] **Step 1: 全量运行 run_check5.py，确认新增文件与修订的净增 issue 数为 0**

Run:
```bash
cd "D:\RustGameServer"
python "<scratchpad>/run_check5.py" > "<scratchpad>/check_after_net_pipeline.txt" 2>&1
grep -c "\[ISSUE\]" "<scratchpad>/check_after_net_pipeline.txt"
grep -c "\[HEADER-FAIL\]" "<scratchpad>/check_after_net_pipeline.txt"
```
Expected: `[ISSUE]` 计数 = 此前基线 80 + 本计划 5 份新文件各自的模板级"§3.10"引用问题（若沿用模板则为 +5，若移除该引用行则为 +0）− 0（不应产生本计划内容本身导致的新增结构性问题，如字段名不一致、编号重复等）。`[HEADER-FAIL]` 计数应保持 12（不变）。

- [ ] **Step 2: 核对附件C §4.1 P-NET-001 行、§7 NET域行、§8 AC-NET/AC-OPS行与新文档内容一致**

Run: `cd "D:\RustGameServer" && grep -n "P-NET-001\|AC-NET-\|AC-OPS-00[6-8]" "docs/00-基准与治理/RGS-REQ-004_附件C_可追溯性矩阵.md"`
Expected: 已核实的既有登记（本会话此前已确认，见 Task 描述引用的 v3.11 登记行）与 Task 1〜5 新文档中的 ARC-047/FR-NET/AC-NET/AC-OPS 编号完全一致，不需要修改附件C（新文档只是把附件C已登记的需求"展开成设计"，不改变附件C本身的登记范围）。

- [ ] **Step 3: 人工核对 BAS-038→DTL-045→SPEC-DTL-045 三层的 `FecCodec`/`XorParityCodec`/`RedundancyAdjuster` 类型签名一致性**

对照 Task 1 Step 3、Task 2 Step 3/5、Task 3 Step 3 中的 Rust 代码块，逐字段确认无漂移（方法名、参数类型、返回类型）。

- [ ] **Step 4: 若 Step 1〜3 发现不一致，记录问题清单；若全部通过，标记本计划完成**

不新增文件；若发现问题，在会话中向用户报告具体不一致点，由用户决定是否追加修订任务（不在本计划内擅自扩大范围）。

---

## Self-Review（写完计划后自查，非任务本身）

**Spec 覆盖**：RGS-REQ-038 全部 FR-NET-001〜012／NFR-NET-001〜003／ARC-047／AC-NET-001〜004／TBD-NET-001/002／RSK-NET-001 均已在 Task 1〜3 中有对应章节展开（见各 Task 的追溯性表 Step）。RGS-REQ-007-ADD1 全部 FR-OPS-005〜007／NFR-OPS-009〜011／AC-OPS-006〜008／TBD-OPS-004/005 均已在 Task 4〜5 中有对应章节展开。

**占位符扫描**：TBD-NET-001（选型判定）、TBD-NET-002（负载试验数值）、TBD-OPS-004/005（选型判定）均**有意**保留为"结构已定，数值待定"，这不是计划占位符缺陷，而是如实反映源 REQ 文档本身标注的未决事项——已在每个相关 Step 中明确注明"这是待定项，本设计只固定结构"，符合"不得越权替 TBD 拍板"的 Global Constraint。

**类型一致性**：`FecCodec` trait 的三个方法（`encode`/`decode`/`max_decode_latency`）在 Task 1 Step 3（BAS-038 首次定义）、Task 2 Step 3/5（DTL-045 参考实现与挂接点引用）、Task 3 Step 3/4（SPEC-DTL-045 实现单元与契约引用）中签名保持一致。`QueryDnsStatus`/`QueryCdnEdgeStatus` 方法在 Task 4 Step 3（BAS-003 字段级表格）与 Task 5 Step 3（DTL-003 Protobuf 线格式）中字段名一一对应。
