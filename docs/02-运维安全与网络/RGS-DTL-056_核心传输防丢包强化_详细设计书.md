# 详细设计书（詳細設計書 / Detailed Design Document）

**核心传输防丢包强化 Core Transport Loss-Recovery Hardening**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-056 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-056（核心传输防丢包强化与周边协议选型 基本设计书） |
| 上游依据 | RGS-REQ-038 v0.1 §5（ARC-047：FEC over QUIC Datagram）/ §7（NFR-NET-001 解码延迟 ≤ 50ms tick 周期）/ §9（方针判定） |
| 关联文档 | RGS-DTL-006（网络安全 详细设计书）；RGS-DTL-001 §4（player_db 物理 DDL 中 Datagram 消息持久化路径）；RGS-DTL-007 §3（数据库设计标准）；RGS-IMPL-001 §3 Q-201〜Q-207（工程约定）；RGS-SPEC-CROSS-002（gRPC/Proto 风格指南） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 + RGS-IMPL-001 工程边界 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 状态 | 待评审（v0.1 草案）；与既有 `RGS-DTL-038_卡牌游戏适配_详细设计书.md`（卡牌游戏）和 `RGS-DTL-038_Match域_详细设计书.md`（Match 域）**同号不同主题**，仅作为 ARC-047 子问题的物理/接口级设计 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 | — | 初版制定。落实 RGS-BAS-056 §4/§5/§6 的物理/接口级设计：FEC 编解码器伪代码、QUIC Datagram 帧格式扩展、KCP 补丁式改造字段映射、TCP/UDP 周边接口形态、rgs-fec 新建 crate 判定与既有 crates 对接点 | 全部 |
| 0.2 | 2026-09-20 | Hermes Agent (c557dae5) per ULYS-87 | — | 编号重命名 RGS-DTL-038 → RGS-DTL-056（ULYS-87 收口 DTL-038 三处冲突：卡牌/Match 域 DTL-038 保留；核心传输防丢包主题 BAS+DTL+SPEC 三层链统一改为 056）。文档主题、父文档 BAS-056、§11 追溯性保持。`docs/document-registry.toml` 注释同步登记 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-09-11 | — |
| 评审（DBA） |  |  | 确认 §5.3 Datagram 持久化路径与 RGS-DTL-001 §4 player_db.outbox 与 RGS-DTL-007 §3 数据库标准不冲突 |
| 评审（平台/SRE） |  |  | 确认 §6 新建 rgs-fec crate 与 RGS-IMPL-001 §2 Q-101〜Q-106 工作空间约定（virtual workspace / resolver=3 / 不引入泛化 rgs-common）一致 |
| 评审（性能） |  |  | 确认 §3 算法伪代码与 §7 性能预算（NFR-NET-001 解码延迟 ≤ 50ms tick 周期 / NFR-NET-002 单包 XOR parity 路径开销 ≤ 8%）一致 |
| 审批（负责人） |  |  | 本文档基准化；ARC-047 物理/接口级设计的批准 |

---

## 目录

1. [前言](#1-前言)
2. [物理/接口设计总览](#2-物理接口设计总览)
3. [关键算法与数据结构（FEC RS 码 + 决策表）](#3-关键算法与数据结构)
4. [QUIC Datagram 帧格式扩展](#4-quic-datagram-帧格式扩展)
5. [与现有 crate 的对接点](#5-与现有-crate-的对接点)
6. [rgs-fec 新建 crate 判定与目录结构](#6-rgs-fec-新建-crate-判定与目录结构)
7. [性能预算与可观测性埋点](#7-性能预算与可观测性埋点)
8. [后续计划与 TBD](#8-后续计划与-tbd)

---

# 1. 前言

本文档是 RGS-BAS-056 §4〜§6 的物理/接口级详细设计，**仅**落实 ARC-047（FEC over QUIC Datagram 路径）的工程实现，不涉及：
- ARC-003 Stream 路径（由 RGS-DTL-006 §4 网络协议栈设计承担，本文不改其实现）；
- 周边协议（账号/支付/GM/资源分发/实时语音/位置）的协议字段（已在 RGS-BAS-056 §6 选型矩阵中固化）；
- 反作弊 / 限流等业务侧策略（由 RGS-DTL-025 承担）。

> **实施门禁（per RGS-IMPL-001 §1.3）**：在 `G-CODE-01〜G-CODE-07` 未全部通过前，本文 §5/§6 给出的对接点**仅**作为设计层记录；不得依据本文创建 `rgs-fec` crate 的 Rust 代码、`player_db.migrations/*_fec_*` SQL migration 或 k8s 部署制品。代码实现需等待 `G-CODE-06`（Rust 1.98 stable GA + 全量 CI bootstrap）与 `G-CODE-04`（Saga 场景演练）具名批准。

# 2. 物理/接口设计总览

## 2.1 模块切分

```
                        ┌─────────────────────────────────────┐
   Client (Bevy/UE/Unity)                                 │
        │   Datagram 帧（含 FEC parity）                  │
        ▼                                                  │
   quinn::Endpoint::send_datagram(...)  ── ARC-003 不可靠 Datagram 路径
        │                                                  │
        ▼                                                  │
   ┌──────────────┐                                       │
   │ rgs-fec      │  ┌────────────────────────────────┐    │
   │ ├ encoder    │  │ rs_player / rs_outbox        │    │
   │ ├ decoder    │──┤ / rs_session_key             │    │
   │ ├ parity_t   │  │ / rs_redundancy_decision      │    │
   │ └ wire_fmt   │  └────────────────────────────────┘    │
   └──────┬───────┘                                       │
          │  解码后纯负载                                 │
          ▼                                                │
   ┌──────────────┐                                       │
   │ SessionState │  per-player tick 消费                 │
   │ Aggregator   │  (rgs-contracts-player / rgs-         │
   └──────────────┘   session-state-service)              │
                                                             │
   注：Stream 路径（必达事件）不经 FEC，                  │
   直接走 ARC-003 Reliable Stream。                         │
   周边协议（账号/支付/GM）走 TCP/UDP，见 RGS-BAS-056 §6 │
```

## 2.2 crate 边界

| crate | 角色 | 是否新建 | 依据 |
|---|---|---|---|
| `rgs-fec` | FEC 编解码库（encoder/decoder/parity_table/wire_format），纯函数 + trait `FecEncoder`/`FecDecoder` | **新建** | RGS-BAS-056 §5.3；本节 §6 |
| `rgs-contracts-network` | 网络消息契约（含扩展 DatagramFrame 头部） | 已有 crate，**新增** v2 message 变体 | RGS-IMPL-001 §2 Q-105 |
| `network-gateway`（已存在，2,759 LOC） | QUIC endpoint 集成 | 已有 crate，**新增** FEC codec adapter 调用点 | RGS-BAS-056 §4.2 |
| `rgs-session-state`（或 player-service 内部子模块） | 解码后状态聚合 | 不新建 crate，复用既有 `player-service/session_state.rs` | RGS-DTL-001 §4 |
| `rgs-outbox`（已存在，shared-platform） | Datagram 持久化出口 | 已有 crate，**新增** `outbox_datagram_fec` 表 | RGS-DTL-007 §3 + 本节 §5.3 |

> **判定**：**新建** `rgs-fec` crate，**不**汇入泛化的 `rgs-common`（per RGS-IMPL-001 §2 Q-102 强制规则）。FEC 单一职责、最小依赖（仅 `bytes` + `thiserror` + `tracing` + 候选 `reed-solomon-erasure` 或自研）。

# 3. 关键算法与数据结构

## 3.1 单包级 XOR parity FEC（首选实现，per RGS-BAS-056 §5.2 候选①）

### 3.1.1 编码器伪代码

```rust
// rgs-fec/src/encoder.rs (设计伪代码，未经具名 Gate 批准不得编码)

pub const FEC_GROUP_SIZE: usize = 8;          // K=8 数据包一组
pub const FEC_PARITY_PACKETS: usize = 2;       // M=2 冗余包（冗余度 25%，动态调整）
pub const FEC_MAX_PAYLOAD: usize = 1200;       // 单包 MTU 上限（与 QUIC DATAGRAM 默认对齐）

#[derive(Debug, Clone)]
pub struct FecGroup {
    pub group_id: u32,                        // 递增 group_id（per connection 单调递增）
    pub packets: Vec<DatagramPacket>,         // K 个数据包
    pub parity: Vec<DatagramPacket>,          // M 个冗余包
    pub created_at: Instant,
}

pub trait FecEncoder {
    /// 将 K 个数据包编码为 K+M 个数据包（K 数据 + M 冗余）
    /// 编码函数：parity[j] = XOR_{i in K} (data[i] * coeff_matrix[j][i])
    fn encode_group(&mut self, group: &mut FecGroup) -> Result<(), FecError>;
}
```

> **算法选择**：本设计**默认采用单包级 XOR parity**（RGS-BAS-056 §5.2 候选①）。理由：
> - 解码延迟与分组大小无关（或近似常数），满足 NFR-NET-001（≤ 50ms tick 周期）；
> - 无第三方依赖，规避 RSK-NET-001（新兴 crate 不可控）；
> - 25% 冗余度（K=8, M=2）在 ARC-003 既有 2% 丢包率实测下可保证 > 99.99% 投递成功率（per RGS-REQ-038 §7 NFR-NET-004）。

### 3.1.2 解码器伪代码

```rust
// rgs-fec/src/decoder.rs (设计伪代码)

pub trait FecDecoder {
    /// 接收乱序到达的 K+M 个包中任意 ≥ K 个，恢复原始 K 个数据包。
    /// 若收到 K-1 个数据包 + 1 个冗余包：通过矩阵求逆恢复第 K 个；
    /// 若收到 ≥ K+M 个：丢弃多余冗余包，仅保留 K 数据包。
    fn decode_group(&self, group_id: u32, received: &[DatagramPacket]) 
        -> Result<Vec<DatagramPacket>, FecError>;
}

/// 滑动窗口：以 group_id 为键，缓存 (K-1)+M 个以内的到达包，超时未补齐则请求 Stream 路径补发
pub struct FecGroupCache {
    cache: BTreeMap<u32, FecGroupPartial>,
    max_age: Duration,                         // 默认 200ms（4 × tick 周期）
}
```

### 3.1.3 冗余度决策表（per RGS-BAS-056 §5.4）

| 网络状况（per ARC-003 既有 metrics） | 冗余度 (M/K) | 决策来源 |
|---|---|---|
| 丢包率 ≤ 1% | M=1, K=8 (12.5%) | RGS-REQ-038 §7 NFR-NET-004 默认 |
| 1% < 丢包率 ≤ 3% | M=2, K=8 (25%) | RGS-REQ-038 §7 NFR-NET-004 默认 |
| 3% < 丢包率 ≤ 5% | M=3, K=8 (37.5%) | RGS-REQ-038 §7 NFR-NET-004 上限 |
| 丢包率 > 5% | 不再增加冗余，触发告警 + 自动切换到 Stream 路径重传 | RGS-BAS-056 §5.4 RSK-NET-002 |

> 决策依据为 network-gateway 的 `fec_loss_rate_estimate` Prometheus gauge（per RGS-DTL-004 §3.4 指标目录新增）。

## 3.2 错误类型

```rust
// rgs-fec/src/error.rs (设计)

#[derive(Debug, thiserror::Error)]
pub enum FecError {
    #[error("group {0} not found in cache")]
    GroupNotFound(u32),
    
    #[error("group {0} expired after {1:?}")]
    GroupExpired(u32, Duration),
    
    #[error("insufficient packets: received {received}, need {required}")]
    InsufficientPackets { received: usize, required: usize },
    
    #[error("payload size {0} exceeds MTU {1}")]
    PayloadTooLarge(usize, usize),
    
    #[error("decoder matrix inversion failed: {0}")]
    MatrixInversionFailed(String),
}
```

# 4. QUIC Datagram 帧格式扩展

## 4.1 wire format（per RGS-SPEC-CROSS-002 §3 proto 风格）

```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Version (8)  |  Type (8)     |  Group ID (32)                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Seq In Group (8) |  Total K (8) | Total M (8) |  Rsv (8)     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Payload Length (16)                                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Payload (variable, ≤ FEC_MAX_PAYLOAD)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  FEC Parity (32 bytes, only for parity packets)               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

| 字段 | 位宽 | 说明 |
|---|---|---|
| Version | 8 | 当前 0x01；预留扩展位 |
| Type | 8 | 0x01 = DATA, 0x02 = PARITY |
| Group ID | 32 | per connection 单调递增 |
| Seq In Group | 8 | 在本组内的序号（0..K+M-1） |
| Total K / M | 8+8 | 本组的 K 与 M，用于解码时校验 |
| Payload Length | 16 | 0..FEC_MAX_PAYLOAD |
| Payload | var | 业务数据；DATA 包含完整业务 payload；PARITY 包含 32 字节 XOR 结果 |
| FEC Parity | 32 | 仅 PARITY 包存在 |

## 4.2 与既有 proto 的兼容

- `proto/rgs/network/v1/datagram.proto` 新增 `FecFrame` 消息（gRPC streaming 场景下透传，但运行时通过 QUIC native datagram 而非 gRPC）；
- 不引入 `postcard` 等第三种序列化（per RGS-IMPL-001 §3 Q-202）；
- 既有 `DatagramPacket` 消息保留，新加 `option FecHeader fec_header = N;`。

# 5. 与现有 crate 的对接点

## 5.1 `network-gateway` 改造点

```rust
// network-gateway/src/quic_endpoint.rs 改造点（设计伪代码）

impl QuicEndpoint {
    pub async fn send_with_fec(
        &self,
        conn: &quinn::Connection,
        group_id: u32,
        payloads: Vec<Bytes>,
    ) -> Result<(), NetworkError> {
        let mut group = FecGroup::new(group_id, payloads);
        self.fec_encoder.encode_group(&mut group)?;
        
        // 发送 K+M 个 datagram
        for pkt in group.packets.iter().chain(&group.parity) {
            conn.send_datagram(pkt.wire_bytes()?)?;
        }
        Ok(())
    }
}
```

## 5.2 `rgs-session-state`（或 `player-service/session_state.rs`）改造点

- 既有 `SessionStateAggregator` 接收流改为：
  1. 先经过 `FecDecoder::decode_group`；
  2. 解码失败的 group 触发 Stream 路径补偿（`request_reliable_resend(group_id)`）；
  3. 补偿成功的事件写入 `player_db.outbox`，失败则写入 `player_db.outbox_dlq`（per RGS-DTL-007 §3 DLQ 表）。

## 5.3 数据库持久化（仅设计记录，per RGS-IMPL-001 §1.3 实施门禁）

```sql
-- 待 G-CODE-04 批准后实施的 migration（仅设计记录）
CREATE TABLE player_db.outbox_datagram_fec (
    group_id BIGINT NOT NULL,
    seq_in_group SMALLINT NOT NULL,
    payload_type VARCHAR(64) NOT NULL,    -- 'data' | 'parity'
    payload BYTEA NOT NULL,
    fec_parity BYTEA,                     -- NULL for 'data'
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, seq_in_group)
) PARTITION BY RANGE (created_at);

-- 按周分区（per RGS-DTL-007 §3.4 分区策略）
```

> **状态**：本 migration **未**实施。当前所有 Datagram 消息直接走 `player_db.outbox`（已有）。

# 6. rgs-fec 新建 crate 判定与目录结构

## 6.1 判定结论：**新建** rgs-fec crate

理由：
1. RGS-IMPL-001 §2 Q-102 禁止泛化 `rgs-common`；
2. FEC 是单一职责（编码/解码/校验/帧格式），不与现有任何 crate 重叠；
3. 同时被 `network-gateway`、`rgs-session-state`、`player-service/session_state.rs` 三处使用，符合"至少两个已冻结边界"门槛；
4. 不引入反向依赖（仅依赖 `bytes` + `thiserror` + `tracing` + `reed-solomon-erasure` 候选）。

## 6.2 目录结构

```
crates/rgs-fec/
├── Cargo.toml                 # [dependencies] bytes, thiserror, tracing; candidates: reed-solomon-erasure
├── src/
│   ├── lib.rs                 # pub mod encoder, decoder, frame, error
│   ├── encoder.rs             # FecEncoder trait + XorParityEncoder 实现
│   ├── decoder.rs             # FecDecoder trait + XorParityDecoder 实现 + FecGroupCache 滑动窗口
│   ├── frame.rs               # 4.1 节 wire format 解析/序列化
│   ├── error.rs               # FecError
│   └── decision.rs            # 3.1.3 冗余度决策表
├── tests/
│   ├── encoder_test.rs        # UT-DTL038-FEC-ENC-001〜008
│   ├── decoder_test.rs        # UT-DTL038-FEC-DEC-001〜012
│   └── integration_test.rs    # IT-DTL038-FEC-INT-001〜005
└── README.md
```

## 6.3 与其他 crate 的依赖方向

```text
rgs-fec
  ├── bytes
  ├── thiserror
  └── tracing

network-gateway ──> rgs-fec        (FEC encoder adapter)
rgs-session-state (player-service 内部) ──> rgs-fec  (FEC decoder adapter)
rgs-contracts-network ──> rgs-fec  (DatagramFrame 消息定义反向被引用)
```

> 反向依赖禁止：`rgs-fec` **不得**依赖 `network-gateway` / `player-service` / `rgs-contracts-network`。

# 7. 性能预算与可观测性埋点

## 7.1 性能预算

| 指标 | 预算 | 来源 |
|---|---|---|
| 单包编码延迟 | < 0.5ms | RGS-REQ-038 §7 NFR-NET-001 |
| 单包解码延迟 | < 1ms（含矩阵求逆） | RGS-REQ-038 §7 NFR-NET-001 |
| 端到端解码延迟 | ≤ 50ms（与 tick 周期对齐） | RGS-REQ-038 §7 NFR-NET-001 |
| 单包 XOR parity 路径开销 | ≤ 8%（带宽） | RGS-REQ-038 §7 NFR-NET-002 |
| 25% 冗余度下投递成功率 | > 99.99%（2% 丢包率） | RGS-REQ-038 §7 NFR-NET-004 |

## 7.2 可观测性埋点（per RGS-DTL-004 §3.4）

新增指标：

| 指标 | 类型 | 标签 |
|---|---|---|
| `rgs_fec_groups_encoded_total` | Counter | connection_id |
| `rgs_fec_groups_decoded_total` | Counter | connection_id, outcome (success/decode_failed/timeout) |
| `rgs_fec_decode_latency_seconds` | Histogram | connection_id, outcome |
| `rgs_fec_redundancy_ratio` | Gauge | connection_id (当前 M/K) |
| `rgs_fec_loss_rate_estimate` | Gauge | connection_id |

新增 span：`fec.encode_group`、`fec.decode_group`、`fec.fallback_to_stream`。

# 8. 后续计划与 TBD

| 项 | 状态 | 解除条件 |
|---|---|---|
| TBD-NET-FEC-001：候选 `reed-solomon-erasure` crate 的 crates.io 评估与采纳决策 | Open | 调研产物（per RSK-NET-001）；本设计**默认采用自研单包 XOR parity**，仅在后续压测证明 XOR parity 不足以满足 NFR-NET-004 时重新评估 |
| `rgs-fec` crate 的 Rust 代码实施 | 等待 G-CODE-06 | RGS-IMPL-001 §6 全部 G-CODE 通过 |
| `player_db.outbox_datagram_fec` 表的 SQL migration 实施 | 等待 G-CODE-06 | 同上 |
| 与 `network-gateway` 的集成代码实施 | 等待 G-CODE-06 | 同上 |
| 与 `rgs-session-state` 的集成代码实施 | 等待 G-CODE-06 | 同上 |
| 真实压测（4 平台 × 1000 客户端 × 1000 资源样本）下的解码延迟 / 投递成功率验证 | 等待 ST 阶段 | RGS-QA-001 §3.2 实施入口 |
| 与 ARC-003 Stream 路径的回退链路测试（丢包率 > 5% 时自动切换） | 等待 ST 阶段 | 同上 |
| `RGS-SPEC-DTL-056-防丢包` 实现规格书 | 等待 BAS-056 + 本 DTL 评审通过 | RGS-IMPL-001 §1.2 SPEC 模板 |
| 命名冲突说明：在文档注册表 `docs/document-registry.toml` 中明确登记"ARC-047 防丢包子问题展开；与既有 RGS-DTL-038 卡牌/Match 同号但不同主题" | **本设计书同步完成** | — |

## 8.1 跨文档引用

- **父**：[RGS-REQ-038](../RGS-REQ-038_核心传输防丢包强化与周边协议选型_需求定义书.md) §5/§7/§9
- **父**：[RGS-BAS-056](RGS-BAS-056_核心传输防丢包强化与周边协议选型_基本设计书.md) §4/§5/§6
- **相关**：[RGS-DTL-006 §4](RGS-DTL-006_详细设计书.md) 网络协议栈物理设计（不改）
- **相关**：[RGS-DTL-001 §4](RGS-DTL-001_详细设计书.md) player_db.outbox 物理 DDL
- **相关**：[RGS-DTL-004 §3.4](RGS-DTL-004_详细设计书.md) 指标目录（本文 §7.2 新增指标）
- **相关**：[RGS-DTL-007 §3](RGS-DTL-007_详细设计书.md) 数据库设计标准（本文 §5.3 待实施 migration 遵循）
- **相关**：[RGS-IMPL-001 §1.3/§2/§3](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md) 实施门禁与工程边界
- **相关**：[RGS-SPEC-CROSS-002](../13-实现规格/RGS-SPEC-CROSS-002_gRPC_Proto风格指南_v0.1.md) proto 风格

## 8.2 待具名人类审批的 Gate

| Gate | 内容 | 审批者 |
|---|---|---|
| G-NET-001 | 自研 XOR parity 算法的正确性（含矩阵求逆边界条件） | 架构师 + 平台 Lead |
| G-NET-002 | §7.1 性能预算是否接受（尤其 25% 冗余度下的带宽成本） | SRE Lead |
| G-NET-003 | §6 新建 rgs-fec crate 的依赖图无环 | 架构师 + 五域 Lead |
