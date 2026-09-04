# tracing_init.rs 拆分计划 v0.1 (P2-9 文档化划界)

> **创建日期**: 2026-09-05 JST
> **依据**: 2026-09-04 系统性扫描 P2-9 落地
> **状态**: ⏳ 待 DDD Review (留待 53.12 任务启动时一起实施)

## 0. 背景

`crates/shared-platform/src/tracing_init.rs` 当前 231 行, 混了 3 个职责:

1. **stdout subscriber 初始化** (EnvFilter + fmt layer, 100 行)
2. **OTel OTLP exporter 条件初始化** (init_otel_exporter_optional 100 行)
3. **公共类型** (OtelConfig, OtelExporterGuard, TracingError 30 行)

## 1. 拆分目标 (留待 53.12 实施)

```
crates/shared-platform/src/
├── tracing_init/
│   ├── mod.rs           # 公共类型 + 重导出 (30 行)
│   ├── stdout.rs        # stdout subscriber + EnvFilter (100 行)
│   └── otel.rs          # OTel OTLP exporter (100 行)
```

## 2. 不动 (per 8/27 JST 禁回溯叙事)

- 当前 `tracing_init.rs` 231 行 1 文件, **本任务不实施** — 仅写拆分计划文档
- 5 域 main.rs 调用 `init_otel_exporter_optional` 仍走原路径, 不破 API

## 3. 拆分触发条件

per RGS-OPEN-QA-001 Q-M-03 + WBS WF-1-53.12:
- 53.12 任务启动 (OTel SDK 完整接入)
- OTEL_SDK_DISABLED env 默认 true (per main.rs 默认值)
- 53.12 启动后, init_otel_exporter_optional 才真正启用

## 4. 已知缺口 (per AGENTS.md §1.1 缺标比错标安全)

- **不实施实际拆分** — 等 53.12 启动时一起做, 避免 OTel SDK 接入与拆分同时改 2 件事增加风险
- 53.12 启动时间未定, 留为下轮工单
- 5 域 main.rs 仍走 init_otel_optional 路径, 拆完后 0 改动

## 5. 维护原则 (per 8/27 19:39/20:56/21:59 JST 代签规则)

- **不动**当前 `tracing_init.rs` 实现
- **不动**5 域 main.rs 调用
- **不追溯改写** 9/4 已 commit 的 11 个 D 决策

## 6. 关联

- `crates/shared-platform/src/tracing_init.rs` (当前 231 行)
- AGENTS.md §2.1 (L1 派生约束) + §2.4 (L4 跨工具链场景主会话打头阵)
- 9/4 git log: `0ccd74a` P0-1 service_bootstrap 加 init_otel_optional 转发

## 7. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review | 起草, 划界留待 53.12 一起实施 |
