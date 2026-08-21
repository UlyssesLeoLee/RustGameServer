# RGS-SPEC-CROSS-006 日志 trace_id 传播规范（Trace Propagation Spec）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-006 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（5 域 DTL §4 可观测性规格横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）/ 55（静态分析）/ 56（代码审查）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制并行 | RGS-SPEC-CROSS-003 事件 schema 字典 |

---

## 1. 文档目的

本文件是**跨服务 / 跨 DB / 跨事件 trace_id 传播**的横向规范，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**：5 域各自实现 OTel trace 上报 → trace_id 在跨域 RPC / DB / 事件中无法连贯追踪。

**解决方式**：建立 trace_id 传播规范，强制 5 域使用统一 OTel SDK + W3C Trace Context 头。

---

## 2. 规范范围

### §2.1 输入

- 5 域 DTL §4 可观测性规格
- RGS-GOBS-004 Observability 总体计划
- OpenTelemetry W3C Trace Context 规范
- RGS-TS-001 v0.6 §5.3 OpenTelemetry 选型

### §2.2 输出

- trace_id 格式（128-bit UUID，per W3C Trace Context）
- span_id 格式（64-bit，per W3C Trace Context）
- 跨 gRPC trace 传播（tonic interceptor + gRPC metadata `traceparent`）
- 跨 DB trace 传播（PG `SET LOCAL application_name = trace_id` + `pg_stat_activity`）
- 跨事件 trace 传播（CEM 事件 payload 必带 `ce_traceid` 字段，per CROSS-003）
- 跨 HTTP 传播（gateway → client W3C `traceparent` header）
- 跨异步传播（Tokio task_local + spawn instrument）
- 日志 trace_id 注入（tracing crate JSON formatter 必带 `trace_id` field）
- 错误 / 慢请求 / 重试自动 span 标记
- PII 脱敏（trace 中禁用 email / phone / token 字段）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| SRE | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **OpenTelemetry Rust SDK**（`opentelemetry` + `opentelemetry-otlp`）
- **tracing** + **tracing-subscriber**（结构化日志）
- **tracing-opentelemetry**（tracing → OTel 桥接）
- **tonic-tracing**（gRPC interceptor）
- **sqlx-tracing**（DB trace）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 trace_id / span_id 格式（W3C Trace Context）
# §2 跨 gRPC trace 传播（tonic interceptor）
# §3 跨 DB trace 传播（PG application_name）
# §4 跨事件 trace 传播（CEM ce_traceid）
# §5 跨 HTTP trace 传播（gateway W3C header）
# §6 跨异步 task trace 传播（Tokio task_local）
# §7 日志 trace_id 注入（tracing JSON formatter）
# §8 错误 / 慢请求 / 重试自动 span
# §9 PII 脱敏（trace 中禁用字段）
# §10 trace 采样率规范（默认 100%，可降采样）
# §11 trace 样例（player 域登录 → economy 域扣费全链路）
# §12 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际 trace 传播规则。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-003 事件 schema 字典
- 上游：RGS-WF-001 v0.5 §2 150 工程 54/55/56 + RGS-GOBS-004 + RGS-TS-001 v0.6 §5.3
- 5 域引用方：DTL-015/016/018/019/020/026/031 §4 + WF-1-54.13/54.14/54.15 OTel/Prom/tracing
- worktree：可单独 worktree 分支执行
