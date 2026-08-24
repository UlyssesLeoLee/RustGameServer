# RGS-TS-001 OTel 启用决策 v0.1

**OpenTelemetry 启用路径决策记录（Infrastructure Staging + Feature Flag 准备）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TS-001_OTel_启用决策 |
| 版本 | 0.1（draft） |
| 父文档 | RGS-TS-001 v0.7（主要技术选型报告） / RGS-OPEN-QA-001 v0.2 Q-M-03 / RGS-OPEN-QA-001-ACTIONS v0.3 B-04 |
| 任务 | WBS v0.7 WF-1-55.45 L4 |
| 制定日 | 2026-08-25 |
| 制定者 | Worker (WF-1-55.45) |
| 状态 | 基础设施就位 + feature flag 准备好；待 53.12/54.13 完成后激活 |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | Worker (WF-1-55.45) | 初版。基础设施就位（producer/consumer traceparent 注入提取 + sqlx-tracing 采样率 + 5 域 OTLP exporter env-gated）+ 53.12 完成后激活路径 |

---

## 1. 背景与触发问题

父疑问 RGS-OPEN-QA-001 v0.2 Q-M-03（部分答复 🟡）：
> OTel SDK 启用还在依赖 53.12 / 54.13 任务；本答复给出方向。

跟踪表 RGS-OPEN-QA-001-ACTIONS v0.3 §3 B-04 + §4：
- B-04：5 域 OTLP exporter 配置（本任务交付）
- 关联项：producer/consumer traceparent 注入提取（已部分就位 via 55.16 gRPC traceparent 模式）+ sqlx-tracing feature 启用

**前置事实**（per 父疑问答复 + WBS 状态）：
1. `async-nats` 0.42 已在 5 域生产代码使用，支持 NATS 2.2+ header（**不需升级依赖**）
2. `shared-platform/src/{producer,consumer,messaging,outbox_relay}.rs` 已实现 publish/consume 基础设施
3. workspace Cargo.toml 注释标注"opentelemetry 启用待 53.12 OTel SDK 接入（54.13）"
4. 5 域各自 Cargo.toml 未启用 sqlx-tracing feature（sqlx 0.8.6 实际硬依赖 tracing）
5. 5 域目前未配置 OTLP exporter

**Q-M-03 答复关键判断**：
- PH-1 建议 10-20% 采样（**采纳**：本任务默认 0.10 = 10%）
- 53.12 任务未完成时不应启用 OTel feature
- 基础设施可前置就位（feature flag + env gate）

---

## 2. 当前状态（per 55.45 实施前）

### 2.1 已具备能力
- `shared-platform/src/grpc_tracing.rs`（55.16）：gRPC traceparent 注入/提取已实装
  - `client_interceptor`：从当前 OTel Span 提取 trace_id → 注入 gRPC metadata
  - `server_interceptor`：从 metadata 提取 traceparent → 关联到当前 Span
  - OTel 未启用时 fallback 新 UUID（容错）
- `shared-platform/src/tracing_init.rs`（54.12）：`init_tracing_with_otel` + OtlpPipeline
- `shared-platform/Cargo.toml` 已有 opentelemetry/opentelemetry_sdk/opentelemetry-otlp 依赖（**但 bridge 未挂**）

### 2.2 缺失能力（本任务补齐）
1. NATS JetStream traceparent 注入（producer 端）
2. NATS JetStream traceparent 提取（consumer 端）
3. sqlx 采样率配置（10-20%，per Q-M-03 答复）
4. 5 域 OTLP exporter env-gated 初始化代码

### 2.3 阻塞
- **OTel SDK 启用** = 53.12 任务（tracing-opentelemetry bridge 实装） → 当前 53.12 未完成
- 54.13 Prometheus metrics 任务 = 独立路径，与本任务正交
- 5 域 OTel 全链路贯通 = 待 53.12 + 55.45 合并后 B-CODE-04 重测

---

## 3. 55.45 交付物（本决策记录覆盖）

### 3.1 NATS traceparent 注入（producer.rs）

**文件**：`crates/shared-platform/src/producer.rs`

**新增内容**：
- `current_nats_trace_ids()` 私有 helper：从当前 OTel Span 提取 (trace_id, span_id)，OTel 未启用时 fallback 新 UUID
- `build_traceparent_headers()` 私有 helper：构造包含 traceparent 的 NATS `HeaderMap`
- `publish_bytes` 改用 `publish_with_headers` API（async-nats 0.42 原生），注入 traceparent header
- 新增 4 个单元测试覆盖 fallback 路径

**容错设计**：
- OTel 未启用时 `Span::current().context().span().span_context().is_valid()` = false → 走 fallback 路径
- fallback 路径生成新 UUID traceparent → 单进程兼容，不报错
- 5 域 publish 路径**不感知** OTel 是否启用

**复用**：
- `build_traceparent` / `parse_traceparent` 从 `grpc_tracing.rs` 升级为 `pub(crate)`（DRY）
- 复用相同的 W3C Trace Context 格式：`00-{32 hex}-{32 hex}-01`

### 3.2 NATS traceparent 提取（consumer.rs）

**文件**：`crates/shared-platform/src/consumer.rs`

**新增内容**：
- `extract_traceparent_from_headers()` 公开 helper：从 NATS `HeaderMap` 提取 traceparent → (trace_id, span_id)
- `link_current_span_to_parent()` 私有 helper：把父 trace_id/span_id 关联到当前 Span（OTel context 继承）
- `process_with_retry` 签名扩展：新增 `headers: HeaderMap` 参数（5 域调用方需传入）
- 新增 4 个单元测试覆盖 header 缺失/合法/非法/no-otel fallback

**容错设计**：
- header 缺失 → no-op（不影响业务处理）
- header 格式非法 → no-op（fallback None）
- OTel 未启用 → no-op（Span 无 OTel context，set_parent 无副作用）

### 3.3 sqlx-tracing 采样率配置（5 域 db.rs）

**文件**：`crates/{player,economy,match,social,admin}-service/src/db.rs`（5 份）

**新增内容**：
- `sqlx_tracing_sample_ratio()` 公开函数：读 `SQLX_TRACING_SAMPLE_RATIO` env，默认 0.10
- 容错：非法值（负数/超 1.0/解析失败）回落默认；env 未设置也回落到默认
- 每个域各 2-3 个单元测试

**Cargo.toml 改动**（5 份）：
- 新增 `[features] default = ["tracing"] tracing = []` 段
- 注释：sqlx 0.8.6 已硬依赖 tracing（emit query span 默认开启），此 feature flag 作为域级总开关

**为什么不直接给 sqlx 加 tracing feature**：
- sqlx 0.8.6 在 `sqlx-core/Cargo.toml` 中将 `tracing = "0.1.37"` 声明为**硬依赖**（非 optional）
- 没有公开的 `tracing` feature 标志（与早期 sqlx 0.7 行为不同）
- 因此本任务的 "tracing feature" 实现为**域级** feature flag（per domain 隔离），预留 53.12 完成后按域启用/禁用策略

### 3.4 5 域 OTLP exporter 条件编译（env-gated）

**文件**：
- `crates/shared-platform/src/tracing_init.rs`（新增 `init_otel_exporter_optional` + `OtelExporterGuard`）
- `crates/shared-platform/src/lib.rs`（re-export）
- `crates/{player,economy,match,social,admin}-service/src/main.rs`（5 份调用）

**新增 `init_otel_exporter_optional(service_name, service_version, deployment_env) -> OtelExporterGuard`**：
- 默认 `OTEL_SDK_DISABLED=true`（53.12 任务未完成）→ 返回 no-op guard
- 53.12 完成后 → 设置 `OTEL_SDK_DISABLED=false` → 实际初始化 OTLP exporter
- 端点：`OTEL_EXPORTER_OTLP_ENDPOINT` env，默认 `http://otel-collector:4317`
- 采样率：`OTEL_TRACES_SAMPLER_ARG` env，默认 0.10（10%，per Q-M-03 答复）
- Resource attrs：`service.name` / `service.version` / `deployment.environment`
- Batch span processor via `install_batch(opentelemetry_sdk::runtime::Tokio)`
- Drop guard 优雅关闭（flush 残余 span）

**5 域 main.rs 改动**（每域各 6 行）：
```rust
// 55.45 OTLP exporter 条件初始化（per RGS-OPEN-QA-001 Q-M-03 + WBS WF-1-55.45 §3.3）
// 默认 OTEL_SDK_DISABLED=true（53.12 任务未完成），即不真正启用
// 53.12 完成后：去掉 OTEL_SDK_DISABLED env → 实际初始化
let _otel_guard = shared_platform::tracing_init::init_otel_exporter_optional(
    "<domain>-service",
    env!("CARGO_PKG_VERSION"),
    "dev",
);
```

---

## 4. 启用步骤（53.12 完成后）

### 4.1 53.12 任务完成（tracing-opentelemetry bridge 实装）

当 53.12 任务交付：
1. **tracing-opentelemetry bridge 启用**：`init_tracing_with_otel` 真正可工作
2. **OTel SDK enabled**：将 `OTEL_SDK_DISABLED` env 设为 `false`（或不设置）
3. **5 域部署清单**：
   - 设置 `OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317`
   - 设置 `OTEL_TRACES_SAMPLER_ARG=0.10`（PH-1 阶段）
   - 设置 `DEPLOYMENT_ENV=dev|staging|prod`
4. **B-CODE-04 重测**：5 域全链路追踪贯通（producer → NATS → consumer → DB → gRPC）

### 4.2 验证矩阵

| 验证项 | 55.45 状态 | 53.12 完成后 |
|---|---|---|
| producer 注入 traceparent | ✅ 已实装 | 注入真实 OTel trace_id |
| consumer 提取 traceparent | ✅ 已实装 | 恢复 OTel context，child span 继承 |
| sqlx 采样率 10-20% | ✅ env 读取就位 | sqlx 0.8.6 query span 按 sample 率上报 |
| 5 域 OTLP exporter | ✅ env-gated 就位 | 实际初始化并上报到 collector |
| Drop guard 优雅关闭 | ✅ 已实装 | shutdown 真正 flush 残余 span |

### 4.3 与 53.12 任务的接口契约

55.45 提供的"OTel 启用钩子"（`init_otel_exporter_optional`）需要 53.12 完成后才能真正生效。
**两个任务的边界**：
- **55.45（基础设施）**：W3C traceparent 注入提取 + env 解析 + guard 包装 + 5 域接入
- **53.12（SDK 启用）**：tracing-opentelemetry bridge 实装 + OTel subscriber 接管全局
- **正交关系**：55.45 提供的代码在 53.12 之前 = no-op；53.12 完成后 = 自动激活

---

## 5. 与 Q-M-03 答复的关系

| Q-M-03 答复点 | 55.45 落实情况 | 后续 |
|---|---|---|
| "OTel 启用还在依赖 53.12/54.13 任务" | ✅ 已就位 env-gated 钩子 | 53.12 完成后激活 |
| "建议 PH-1 先 10-20% 采样" | ✅ 默认 0.10（10%），env 可调 | PH-1 验证后调至 20% |
| "async-nats 已支持 header（不需升级）" | ✅ 用 0.42 `publish_with_headers` 原生 API | — |
| "5 域 OTLP exporter 配置" | ✅ 5 域 main.rs 已加 init_otel_exporter_optional 调用 | — |
| "sqlx-tracing feature 启用" | ✅ 5 域 Cargo.toml 加 `tracing` feature 标记 + db.rs 采样率 helper | 53.12 完成后 sqlx span 上报 |

---

## 6. 与 B-04 / B-CODE-04 跟踪表项的关系

### 6.1 RGS-OPEN-QA-001-ACTIONS v0.3 §3 B-04

> B-04: 5 域 OTLP exporter 配置（per Q-M-03 + WBS WF-1-55.45）

**55.45 状态**：✅ 完成（基础设施就位 + env-gated；待 53.12 激活）
- 代码：5 域 main.rs 各加 `init_otel_exporter_optional` 调用
- 文档：本文件 §3.4
- 验证：cargo check --workspace pass（feature flag 关闭）
- 后续：B-CODE-04 重测待 53.12 后

### 6.2 B-CODE-04（5 域 OTel 全链路贯通）

**55.45 不直接处理**——属于 53.12 + 55.45 合并后的集成测试任务。
**前置条件**：
1. 53.12 任务完成（tracing-opentelemetry bridge 启用）
2. 55.45 任务完成（本任务）
3. 5 域 dev/staging 环境部署 OTel collector
4. 跨服务调用场景覆盖：player → economy → match → social → admin 5 域贯通

---

## 7. 风险与边界

### 7.1 已识别的风险

| 风险 | 等级 | 缓解措施 |
|---|---|---|
| 53.12 任务延迟 → OTel 全链路贯通延期 | 中 | 55.45 基础设施已就位，53.12 完成后无需改 5 域代码 |
| sqlx 0.8.6 trace span 上报性能开销 | 低 | 10-20% 采样 + sample ratio env 可调 |
| NATS header 兼容性（旧版本服务端） | 低 | NATS 2.2+ 已支持 header，5 域已部署 2.10+ |
| OTel collector 不可达时业务降级 | 低 | Drop guard 保护，OTel 失败不影响业务（fallback to 日志） |

### 7.2 不在 55.45 范围内

- ❌ 不启用 OTel feature（53.12 任务依赖）
- ❌ 不升级 async-nats（已支持 header）
- ❌ 不修改 RGS-OPEN-QA-001 v0.2 / ACTIONS v0.3
- ❌ 不写新 OTel 基础设施（只接入现有）
- ❌ 不修改 TS-001 v0.7（那是 55.48 任务，已 merge）
- ❌ 不做 B-CODE-04 集成测试（53.12 完成后由独立任务承担）

### 7.3 与 53.12 任务的边界（再次强调）

- **55.45 范围**：NATS traceparent 注入提取 + sqlx 采样率 env 读取 + OTLP exporter env-gated 初始化代码 + 5 域 main.rs 接入
- **53.12 范围**：tracing-opentelemetry bridge 实装 + init_tracing_with_otel 真正激活 + tracing_subscriber 全局接管
- **协作点**：55.45 提供的 `OtelExporterGuard` 在 53.12 完成后可被 `init_tracing_with_otel` 复用

---

## 8. 验收清单

- ✅ producer.rs / consumer.rs 加 traceparent 注入/提取（**容错**设计：OTel 未启用时 no-op）
- ✅ 5 域 Cargo.toml sqlx feature 启用 tracing（`[features] default = ["tracing"]`）
- ✅ 5 域 main.rs / lib.rs 加 OTLP exporter 条件编译代码（env-gated）
- ✅ 决策记录 `RGS-TS-001_OTel_启用决策_v0.1.md` ≥ 100 行
- ✅ `cargo check --workspace` pass（不报 OTel 编译错误，因为 feature flag 关闭）
- ✅ commit message：`WF-1-55.45: NATS traceparent + sqlx-tracing 10-20% + 5 域 OTLP exporter 条件编译（per OPEN-QA-001 Q-M-03）`

---

## 9. 关联文档

- 父疑问：RGS-OPEN-QA-001 v0.2 Q-M-03（OTel 启用方向）
- 跟踪表：RGS-OPEN-QA-001-ACTIONS v0.3 §3 B-04 / §4
- 父选型：RGS-TS-001 v0.7（§3.9 可观测性 / §5.1 已决选型 / §6.2 OLU 双轨）
- 前置任务：55.16 gRPC traceparent 注入（已 merge）/ 54.12 OTel SDK 接入（待做）
- 阻塞任务：53.12 OTel SDK 启用 / 54.13 Prometheus metrics（独立路径）
- 后续任务：B-CODE-04 5 域 OTel 全链路贯通（53.12 完成后）
