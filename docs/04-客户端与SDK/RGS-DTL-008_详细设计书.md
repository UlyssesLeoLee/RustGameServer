# 详细设计书（詳細設計書 / Detailed Design Document）

**客户端引擎适配层与SDK：FFI C ABI边界具体签名・版本协商线格式・编解码性能基准配置详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-008 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-008 客户端引擎适配层与SDK 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档接续RGS-DTL-001/002/025/026/027批次，是04域第二份DTL文档，与已有RGS-DTL-027／CDN域构成同域姊妹文档，二者均挂载同一客户端SDK运行环境但归属不同基本设计源）。细化RGS-BAS-008§3.1编解码性能约束为具体零拷贝编解码trait签名与预分配缓冲结构、§4 FFI边界设计落实为具体C ABI导出函数签名（含panic捕获/参数校验/内存归属的可编译级函数原型）、§8协议版本协商时序落实为具体线格式（`SessionHandshake`字段编号）、§9回归测试基础设施落实为具体基准测试与CI性能门禁配置骨架。**本版本不覆盖**：TBD-SDK-001绑定生成工具链的最终选型评审结论（仅给出`cbindgen`提案）、Unity/UE两侧C#/C++绑定代码本身（FFI边界之外，各自引擎生态内的胶水代码，不属于核心SDK详细设计范围）。见§6 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | FFI导出函数签名是否完整覆盖RGS-BAS-008§4.1全部约定（panic捕获/参数校验/内存归属/错误传播），是否存在遗漏的unsafe边界 |
| 评审（客户端团队代表） | | | C ABI签名是否便于Unity/UE两侧生成对应的P/Invoke与C++绑定，是否有不必要的复杂参数结构增加绑定难度 |
| 审批（负责人） | | | 本文档的基准化；TBD-SDK-001绑定生成工具链提案（`cbindgen`）是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [编解码性能设计的具体实现](#2-编解码性能设计的具体实现)
3. [FFI边界：C ABI导出函数具体签名](#3-ffi边界c-abi导出函数具体签名)
4. [协议版本协商线格式](#4-协议版本协商线格式)
5. [回归测试基础设施：基准测试与CI性能门禁配置](#5-回归测试基础设施基准测试与ci性能门禁配置)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-008给出了核心SDK模块结构框图、编解码性能约束（零拷贝/预分配原则的文字表述）、FFI边界设计的**约定**（panic捕获/参数校验/内存归属/错误传播四条规则，未给出具体函数签名）、协议版本协商的时序图（消息名但无字段编号）、回归测试基础设施的文字描述。本文档将其落实为：可编译级C ABI函数原型、具体协议线格式（字段编号）、`cargo bench`基准测试骨架与CI性能门禁的具体阈值比对脚本。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-008已确定的任何结构性选择（核心逻辑单一实现只在`rgs-client-core`、Bevy直接依赖不经FFI、Unity/UE经FFI边界、仅`ffi/`模块允许unsafe）。
- 不选定TBD-SDK-001绑定生成工具链的最终结论——本文档提出`cbindgen`生成C头文件的具体调用方式，但最终是否采纳、C#/C++侧绑定生成的配套脚本仍需详细设计阶段（或本文档后续版本）与客户端团队联合确认，本文档的提案不构成评审本身。
- 不覆盖Unity C#侧/UE C++侧FFI边界**之外**的绑定代码本身（`RgsClientBehaviour`的`MonoBehaviour`实现细节、`URgsClientComponent`的`UActorComponent`实现细节）——这些已分别在RGS-BAS-008§6/§7给出接口层面设计，其内部实现是各引擎适配层自身的实现细节，不属于核心SDK（本文档聚焦对象）的FFI边界详细设计范围。
- 不给出RGS-IFS-001字节格式本身的重新定义——RGS-BAS-008§3.1原文已明确"具体的字节打包/量化精度由RGS-IFS-001给出，本设计仅约束实现该格式时不得引入的性能反模式"，本文档同样只在此约束范围内给出零拷贝trait签名，不重新定义位打包顺序/定点数精度。

### 1.3 记述规则

沿用既有DTL文档记述规则：协议格式以Protobuf风格给出字段编号（复用RGS-DTL-001§1.3已确立的编号纪律：1〜15高频字段，16以上低频/可选字段，一经分配不得变更/复用），FFI签名以Rust `extern "C"`语法给出（可直接编译，非伪代码），算法伪代码可直接对应Rust `Result`实现。

---

## 2. 编解码性能设计的具体实现

对应RGS-BAS-008§3.1，落实"零拷贝优先"与"预分配缓冲"两条约束为具体trait/结构签名。

### 2.1 零拷贝编解码trait

```rust
// codec/mod.rs
// Datagram路径(PlayerInputMessage/StateDeltaSnapshot/InputAck)必须实现该trait，
// 而非使用serde derive的默认owned反序列化路径(后者逐字段分配String/Vec，
// 违反RGS-BAS-008§3.1"避免高频路径产生不必要堆分配"约束)
pub trait ZeroCopyCodec<'a>: Sized {
    /// 从借用的字节切片解码，返回值的生命周期与输入切片绑定，
    /// 不得在实现体内调用to_vec()/to_string()等产生新分配的操作
    fn decode_borrowed(buf: &'a [u8]) -> Result<Self, CodecError>;

    /// 编码到调用方提供的预分配缓冲区(见§2.2)，返回实际写入字节数，
    /// 函数体内不得调用Vec::new()/Vec::with_capacity()——缓冲区生命周期由调用方管理
    fn encode_into(&self, buf: &mut [u8]) -> Result<usize, CodecError>;
}
```

### 2.2 预分配缓冲区结构

```rust
// transport/quic.rs
// 连接建立后为发送/接收路径各分配一次，贯穿连接生命周期复用，
// 不得在每次encode_into/decode_borrowed调用时重新分配(RGS-BAS-008§3.1"应当"级约束)
pub struct PreallocatedIoBuffers {
    send_buf: Box<[u8; SEND_BUF_CAPACITY]>,     // 容量取IF-001最大单帧尺寸的既定上界（由RGS-IFS-001给出，本文档不重复定义）
    recv_buf: Box<[u8; RECV_BUF_CAPACITY]>,
}

impl PreallocatedIoBuffers {
    pub fn new() -> Self {
        // 固定容量数组Box化，一次性堆分配，此后encode_into/decode_borrowed均复用该内存，
        // 不产生新的堆分配——落实"预分配缓冲区并复用"要求的具体内存布局选择
        Self {
            send_buf: Box::new([0u8; SEND_BUF_CAPACITY]),
            recv_buf: Box::new([0u8; RECV_BUF_CAPACITY]),
        }
    }
}
```

**边界条件说明**：若单帧编码结果超出`SEND_BUF_CAPACITY`（理论上不应发生，因IF-001协议本身对单帧尺寸有既定上界），`encode_into`须返回`CodecError::BufferTooSmall`而非panic或截断写入——截断写入会产生静默数据损坏，是本文档要求避免的更严重故障模式，故显式返回错误优先于任何"尽量塞入"的降级尝试。

---

## 3. FFI边界：C ABI导出函数具体签名

对应RGS-BAS-008§4.1四条约定，本节给出全部导出函数的可编译级Rust `extern "C"`原型，逐条对照约定标注落实方式。

### 3.1 连接生命周期函数

```rust
// ffi/mod.rs —— 仅本模块允许unsafe(RGS-BAS-008§3"设计原则"既定约束)

use std::os::raw::c_char;
use std::panic::catch_unwind;

#[repr(C)]
pub enum RgsErrorCode {
    Ok = 0,
    ErrInternalPanic = 1,       // panic捕获约定的具体错误码(RGS-BAS-008§4.1"如RGS_ERR_INTERNAL_PANIC")
    ErrNullPointer = 2,          // 参数校验约定：指针参数非空检查失败
    ErrOutOfRange = 3,            // 参数校验约定：长度/索引参数越界
    ErrConnectionFailed = 4,
    ErrProtocolVersionRejected = 5,  // 对应§4版本协商result_code=协议版本过旧
}

/// 建立连接。返回不透明句柄指针，所有权归核心SDK，调用方须以rgs_connection_free释放。
/// 参数校验：address_utf8为空指针时返回ErrNullPointer，不解引用。
#[no_mangle]
pub extern "C" fn rgs_connection_connect(
    address_utf8: *const c_char,
    protocol_version: u32,
    out_handle: *mut *mut RgsConnectionHandle,
    out_error: *mut RgsErrorCode,
) -> bool {
    // 全部导出函数体内部使用catch_unwind包裹核心逻辑调用(RGS-BAS-008§4.1"panic捕获"约定)
    let result = catch_unwind(|| {
        if address_utf8.is_null() || out_handle.is_null() || out_error.is_null() {
            return Err(RgsErrorCode::ErrNullPointer);
            // 指针参数在解引用前校验非空；本行之后才允许对address_utf8等指针做任何解引用操作
        }
        // SAFETY: 已完成非空校验，且ffi/模块是本代码库内唯一允许出现该类unsafe解引用的位置
        let addr_str = unsafe { std::ffi::CStr::from_ptr(address_utf8) }
            .to_str()
            .map_err(|_| RgsErrorCode::ErrOutOfRange)?;

        let conn = core_connect(addr_str, protocol_version).map_err(|_| RgsErrorCode::ErrConnectionFailed)?;
        let handle = Box::into_raw(Box::new(RgsConnectionHandle(conn)));
        // SAFETY: out_handle已校验非空
        unsafe { *out_handle = handle; }
        Ok(())
    });

    match result {
        Ok(Ok(())) => { unsafe { *out_error = RgsErrorCode::Ok; } true }
        Ok(Err(code)) => { unsafe { *out_error = code; } false }
        Err(_panic) => {
            // panic不得跨越FFI边界(RGS-BAS-008§4.1"Rust panic跨越FFI边界是未定义行为")，
            // catch_unwind已在此处拦截，此分支是该拦截的落地点
            unsafe { *out_error = RgsErrorCode::ErrInternalPanic; }
            false
        }
    }
}

/// 内存归属约定的落地：由核心SDK通过rgs_connection_connect分配的句柄，
/// 必须通过本函数释放，调用方(C#/C++侧)不得直接free(RGS-BAS-008§4.1"内存归属"约定)
#[no_mangle]
pub extern "C" fn rgs_connection_free(handle: *mut RgsConnectionHandle) {
    if handle.is_null() { return; }  // 参数校验：空指针释放是安全的no-op，不panic
    let _ = catch_unwind(|| {
        // SAFETY: handle由rgs_connection_connect通过Box::into_raw产生，
        // 此处Box::from_raw是该配对分配的唯一合法释放路径
        unsafe { drop(Box::from_raw(handle)); }
    });
}
```

### 3.2 数据收发函数（对应§2零拷贝路径）

```rust
/// 发送PlayerInputMessage。caller_buf由调用方(C#/C++侧)分配并持有，
/// 核心SDK不得持有超出本次调用期的引用(RGS-BAS-008§4.1"内存归属"约定后半段)
#[no_mangle]
pub extern "C" fn rgs_send_player_input(
    handle: *mut RgsConnectionHandle,
    caller_buf: *const u8,
    caller_buf_len: usize,
    out_error: *mut RgsErrorCode,
) -> bool {
    let result = catch_unwind(|| {
        if handle.is_null() || caller_buf.is_null() || out_error.is_null() {
            return Err(RgsErrorCode::ErrNullPointer);
        }
        if caller_buf_len == 0 || caller_buf_len > MAX_INPUT_MESSAGE_LEN {
            // 长度参数校验范围，越界返回错误码而非直接触发越界访问
            return Err(RgsErrorCode::ErrOutOfRange);
        }
        // SAFETY: 已校验非空且长度已限定上界
        let slice = unsafe { std::slice::from_raw_parts(caller_buf, caller_buf_len) };
        let msg = PlayerInputMessage::decode_borrowed(slice).map_err(|_| RgsErrorCode::ErrOutOfRange)?;
        // SAFETY: handle已校验非空，且由rgs_connection_connect合法产生
        unsafe { (*handle).0.send_input(msg) }.map_err(|_| RgsErrorCode::ErrConnectionFailed)
    });

    match result {
        Ok(Ok(())) => { unsafe { *out_error = RgsErrorCode::Ok; } true }
        Ok(Err(code)) => { unsafe { *out_error = code; } false }
        Err(_) => { unsafe { *out_error = RgsErrorCode::ErrInternalPanic; } false }
    }
}
```

**错误传播约定的落地**：全部导出函数统一返回`bool`（成功/失败）+ 通过`out_error`输出指针写回`RgsErrorCode`枚举——这是RGS-BAS-008§4.1"统一错误码枚举（非异常/非panic），C#/C++侧适配层将错误码转换为各自语言习惯的异常/返回值类型"的字面落实：C#侧可在P/Invoke包装层将`RgsErrorCode`非`Ok`的情形转换为`RgsException`抛出，C++侧可转换为`std::expected`或错误码返回值，具体转换代码属各引擎适配层自身职责（§1.2已声明不在本文档范围）。

### 3.3 崩溃回归测试集覆盖点（对应RGS-BAS-008§9"畸形输入"测试集，落实为具体测试用例清单）

| 测试用例 | 期望行为 |
|---|---|
| `address_utf8`传入空指针 | 返回`false`，`out_error=ErrNullPointer`，不崩溃 |
| `caller_buf_len`传入`usize::MAX` | 返回`false`，`out_error=ErrOutOfRange`，不发生越界读取 |
| `handle`传入已被`rgs_connection_free`释放的悬空指针 | 属调用方误用（use-after-free），FFI边界本身无法从指针值本身判断其有效性；本文档在此明确记录该已知限制，缓解手段是C#/C++侧绑定层在`free`后将本地句柄变量置空，防止重复传入——这一缓解措施不属于核心SDK（Rust侧）职责范围 |
| 触发`core_connect`内部panic（如构造测试用mock使其panic） | `catch_unwind`捕获，返回`false`，`out_error=ErrInternalPanic`，宿主进程不崩溃（对应AC-SDK-002验收标准） |

---

## 4. 协议版本协商线格式

对应RGS-BAS-008§8时序图，落实为具体`SessionHandshake`消息字段编号（复用RGS-BAS-001§6.2既有字段设计，本文档只固定字段编号本身，不重新定义字段含义，同RGS-DTL-001§4.1既定记述原则）。

```protobuf
message SessionHandshakeRequest {
  uint32 protocol_version = 1;      // 高频字段，客户端当前SDK内置协议版本号N
}

message SessionHandshakeResponse {
  ResultCode result_code = 1;       // 复用RGS-DTL-001§4.4已定义的通用ResultCode枚举，不另建新枚举，
                                       // 新增取值PROTOCOL_VERSION_TOO_OLD（见下方扩展）
  int64  session_epoch    = 2;       // ARC-005核心字段，同RGS-DTL-001§4.2 SelectCharacterResponse同一语义
  uint32 accepted_version   = 3;      // 服务端实际采纳的协议版本(可能因N-1兼容策略而非等于请求的N)
}
```

`ResultCode`枚举扩展（在RGS-DTL-001§4.4已定义的`ResultCode`基础上新增一个取值，编号延续该枚举既有分配序列，不重新定义整个枚举，同RGS-DTL-001§1.3"编号一经分配不得变更/复用"纪律——`ResultCode`是跨全部服务复用的枚举，本文档在此新增取值须与RGS-DTL-001保持同一份定义源，不得另建重复枚举）：

```protobuf
enum ResultCode {
  // ... OK/UNKNOWN_ERROR/OCC_CONFLICT/STALE_SESSION_EPOCH/INVALID_REQUEST/
  //     ACCOUNT_BANNED/INSUFFICIENT_BALANCE/DUPLICATE_REQUEST_ID (0〜7, 定义于RGS-DTL-001§4.4)
  PROTOCOL_VERSION_TOO_OLD = 8;   // 新增：对应§8时序图"服务器不再接受该版本(超出N-1窗口)"分支
}
```

客户端侧收到`PROTOCOL_VERSION_TOO_OLD`后的处理伪代码：

```rust
fn on_handshake_response(resp: SessionHandshakeResponse) -> Result<Session, SdkError> {
    match resp.result_code {
        ResultCode::Ok => Ok(Session::new(resp.session_epoch, resp.accepted_version)),
        ResultCode::ProtocolVersionTooOld => {
            // 向引擎适配层抛出明确错误(RGS-BAS-008§8"提示需升级SDK")，
            // 不在核心SDK内部尝试自动降级重连——版本升级需要新的SDK构建产物，
            // 不是运行时可自愈的状态，核心SDK不应假装可以处理
            Err(SdkError::ProtocolVersionRejected {
                client_version: resp.accepted_version,
            })
        }
        other => Err(SdkError::HandshakeFailed(other)),
    }
}
```

---

## 5. 回归测试基础设施：基准测试与CI性能门禁配置

对应RGS-BAS-008§3.1"基准测试"要求与§9回归测试基础设施，落实为具体`cargo bench`骨架与CI阈值比对脚本。

### 5.1 基准测试骨架

```rust
// benches/codec_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_player_input_roundtrip(c: &mut Criterion) {
    let sample = build_sample_player_input_message();
    let mut buf = [0u8; SEND_BUF_CAPACITY];

    c.bench_function("codec_player_input_encode", |b| {
        b.iter(|| sample.encode_into(black_box(&mut buf)))
    });

    let encoded_len = sample.encode_into(&mut buf).unwrap();
    c.bench_function("codec_player_input_decode", |b| {
        b.iter(|| PlayerInputMessage::decode_borrowed(black_box(&buf[..encoded_len])))
    });
}

// 覆盖全部IF-001消息类型(RGS-BAS-008§3.1"必须覆盖全部IF-001消息类型"要求的字面落实)
criterion_group!(codec_benches,
    bench_player_input_roundtrip,
    bench_state_delta_snapshot_roundtrip,
    bench_input_ack_roundtrip,
);
criterion_main!(codec_benches);
```

### 5.2 CI性能门禁

```bash
# CI流水线新增阶段(复用RGS-BAS-002§4.2既有CI骨架的"性能门禁"同类思路，
# RGS-BAS-008§3.1原文已声明该复用关系，本文档补充具体命令)
cargo bench --bench codec_bench -- --output-format bencher | tee bench_output.txt

# 阈值比对：NFR-SDK-002要求p99<0.05ms，此处以criterion输出的均值近似把关
# (criterion默认输出含置信区间，CI脚本提取上界与阈值比对，严格p99校验
#  留待实现阶段视criterion版本支持情况决定是否切换--profile-time模式)
python3 scripts/check_bench_threshold.py \
    --input bench_output.txt \
    --threshold-ms 0.05 \
    --metric p99_upper_bound
# 回归超阈值时脚本以非零退出码结束，CI标红(RGS-BAS-008§3.1"回归超阈值时CI必须标红")
```

`check_bench_threshold.py`本身的实现细节（具体如何从criterion输出格式解析p99区间）不属于设计文档范围，留待实现阶段编写，此处只固定其调用契约（输入文件格式、阈值参数、退出码语义）。

---

## 6. 本文档的覆盖范围与后续计划

本文档覆盖：`ZeroCopyCodec` trait与预分配缓冲结构的具体签名、FFI边界连接生命周期/数据收发导出函数的可编译级C ABI原型（含panic捕获/参数校验/内存归属/错误传播四条约定的逐一落实）、崩溃回归测试集的具体用例清单、`SessionHandshake`协议版本协商的具体字段编号与`ResultCode`枚举扩展、编解码基准测试骨架与CI性能门禁阈值比对脚本。

本版本明确不覆盖、留待后续：

- TBD-SDK-001绑定生成工具链的最终选型评审结论——本文档§3给出的手写C ABI签名风格本身与`cbindgen`自动生成路径兼容（`cbindgen`可直接从本文档给出的`#[no_mangle] extern "C"`函数产出对应C头文件），但工具链选型本身（`cbindgen`+额外C#/C++绑定生成脚本的具体组合）仍需详细设计阶段或本文档后续版本与客户端团队联合确认。
- Unity C#侧/UE C++侧FFI边界**之外**的绑定代码与`MonoBehaviour`/`UActorComponent`具体实现——RGS-BAS-008§5〜§7已给出接口层面设计，其内部实现属各引擎适配层自身职责，不属于核心SDK详细设计范围。
- TBD-SDK-002 SDK分发渠道的具体方案——RGS-BAS-008原文已标注为留待详细设计阶段确定，本文档聚焦FFI边界与协议/性能设计，未涉及分发渠道（如私有npm/UPM包仓库、UE Marketplace等选型），留待后续版本或独立文档补充。
- `check_bench_threshold.py`脚本本身的实现——本文档只固定其调用契约，具体解析逻辑留待实现阶段编写。

后续详细设计建议顺序：本文档与RGS-DTL-027（客户端资源分发与热更新）同属04域，二者均运行于客户端侧但覆盖不同基本设计源（RGS-BAS-008 vs RGS-BAS-027），彼此独立，无直接依赖关系；建议04域下一步可考虑是否需要一份"客户端侧集成总览"文档说明SDK（本文档）与资源热更新（RGS-DTL-027）两个客户端组件在同一客户端进程内的启动时序/依赖关系，若确有必要另立，本文档不代为决定。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-008§2 整体架构 | 前提依赖，§3/§4边界划分不变 |
| RGS-BAS-008§3 核心SDK模块结构 | §2、§3 |
| RGS-BAS-008§3.1 编解码性能设计 | §2、§5 |
| RGS-BAS-008§4 FFI边界设计 | §3 |
| RGS-BAS-008§4.2 生成方式（TBD-SDK-001） | §6（明确排除最终选型结论，仅给出提案） |
| RGS-BAS-008§5 Bevy适配层设计 | §1.2（明确排除，不经FFI边界，不属本文档范围） |
| RGS-BAS-008§6 Unity适配层设计 | §3.2（错误传播落地）、§6（明确排除绑定代码本身） |
| RGS-BAS-008§7 Unreal Engine适配层设计 | §3.2（错误传播落地）、§6（明确排除绑定代码本身） |
| RGS-BAS-008§8 协议版本协商时序 | §4 |
| RGS-BAS-008§9 回归测试基础设施 | §3.3、§5 |
| RGS-BAS-008§11 AC-SDK-001〜004 | §3.3（AC-SDK-002）、§4（AC-SDK-003）、§5（AC-SDK-001基准部分） |
| RGS-DTL-001§4.4（通用`ResultCode`枚举） | 前提依赖，§4扩展而非重新定义 |
