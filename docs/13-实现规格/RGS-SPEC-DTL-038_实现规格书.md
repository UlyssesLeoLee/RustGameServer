# RGS-DTL-038 实现规格书

**RGS-SPEC-DTL-038**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-038 |
| 版本 | 0.1 |
| 状态 | 规格草案，待 RGS-DTL-038 具名 DD Review |
| 源详细设计 | RGS-DTL-038 |
| 实现范围 | Match 域 App 与 match_db |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、Actix Web 4.14.1、PostgreSQL 18.4；环境需先核验 |
| 规格真源 | 源 DTL 的接口、字段、状态机、错误码、SQL/proto 和非目标 |

## 1. 使用规则

本规格把 RGS-DTL-038 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-038 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

实现匹配、队列、房间、结果、RT 绑定、超时、重试、结算和事件关联；match/room ID 不进 metric label。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 公共契约 | services/match；crates/match；migrations/match | crate/service 依赖边界必须显式登记 |
| API/event | proto、OpenAPI 或源 DTL 定义的接口 | 字段、枚举、错误码、版本兼容与 DTL 一致 |
| 数据 | migration、repository、Outbox 或源 DTL 指定存储 | 事务、幂等、索引、备份和回滚可验证 |
| 配置 | ConfigMap/Secret/版本化配置 | Secret 不进日志；配置变更有 owner、版本和审计 |
| 部署 | Deployment/StatefulSet/Service/NetworkPolicy/health probe | K3s/Kubernetes 适配前不得宣称已部署 |
| CI | fmt、clippy、test、deny、schema/cardinality/security checks | 负例必须阻断合并 |

## 3. 实现契约

- 入口统一经过认证、授权、限流、输入校验、幂等、脱敏、埋点和审计管道。
- 所有 timeout、retry、backoff、queue、mailbox、batch 和 buffer 有界；失败必须有明确降级、dead-letter 或人工介入出口。
- 跨域不得直连其他域数据库；使用既定 API、event、Outbox 或 workflow。
- migration 与 schema 使用 expand-contract；写操作保留 request_id、trace_id、operator_id，并按 DTL 要求携带 approval_ref、expected_version、OCC 或 fencing。
- 关闭和 drain 必须先停止新工作，再在 deadline 内 flush；超时有指标和日志。
- 若源 DTL 有动态配置或 plugin 生命周期，配置切换必须原子、可审计、可回滚；不得在 hot path 动态卸载有状态代码。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log、Grafana 或物理存储客户端。
- metrics 使用有限标签；本 DTL 的入口、依赖、状态、错误和恢复边界必须接入 RGS-DTL-004 的统一日志/指标/trace contract。
- player_id、session_id、scene_id、room_id、match_id、request_id、trace_id、event_id、workflow_id、operation_id 不得作为 metric label。
- 高频 tick、packet、entity 循环不得同步 telemetry I/O、逐项 span 或逐项日志；异常使用窗口聚合和诊断快照。
- 普通结构化日志与 OPERATION_AUDIT 分离；password、token、credential、payment secret 永不记录。
- 关键请求/事件必须能用 trace_id、request_id、event_id、workflow_id 反查 dashboard、trace、日志和审计。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 认证授权 | RBAC、operator/user scope、内部 mTLS/TLS、最小权限 |
| 幂等一致性 | request_id/idempotency key、OCC/fencing、重复请求和重试 |
| 故障 | timeout、依赖不可用、网络分区、DB/cache/event 故障、进程重启 |
| 背压 | bounded queue、drop/retry exhausted、降级、恢复和告警 |
| 发布 | manifest/version/health、灰度、drain、rollback、schema compatibility |
| 数据治理 | PII/secret redaction、审计、retention、备份恢复、删除/归档 |

## 6. 测试规格

- UT：字段/枚举/状态机/错误码、redaction、cardinality、幂等、retry/backoff、queue 上限。
- IT：API/event/DB/cache/Outbox/Collector/health probe 与跨服务 trace propagation。
- ST：K3s/Kubernetes workload、RBAC、NetworkPolicy、Ingress、migration、rolling update、dashboard/alert。
- Load：目标 CCU、峰值 3x、tick、ACK、登录、匹配、交易、重连，以及 telemetry on/off 对比。
- Chaos：runtime kill、PostgreSQL primary stop、cache loss、event restart、network partition、drain。
- Security：凭证/PII 泄露、越权、非法 label、未授权配置和审计完整性。
- Rollback：应用、plugin、配置、migration、Collector 和 dashboard 独立回滚。

测试必须回填 RGS-REQ-004 追踪矩阵和对应 DTL 的验收项；不能只证明“服务启动”。

## 7. Definition of Done

- RGS-DTL-038 的审批/风险条件已满足；源 DTL 的 TBD 已有批准处置。
- 代码、manifest、migration、proto/schema、配置和测试与 DTL 逐项对账。
- Cargo fmt、clippy、test、deny、schema、secret、high-cardinality 检查通过。
- health/readiness/liveness/degraded/draining 语义可由平台与 dashboard 一致识别。
- API/event/DB/trace/log/metric/security/rollback 证据归档。
- 对应 App/Plugin 有稳定 version、owner、dashboard、alert、runbook 和恢复路径。
- 当前无实现文件时保持“待实现/待评审”，不得标记生产完成。

## 8. Gate 证据与实测参数

RGS-IMPL-001 已固定 workspace、crate、协议、迁移、错误、Saga、CI、镜像与可观测性后端边界；本规格不再保留这些工程选择的平行候选。进入实现前必须取得：① 源 DTL 的具名 DD Review；② Rust 1.98 stable 的锁定依赖完整 CI、PostgreSQL 18.4 迁移演练、K3s 能力的核验证据；③ 针对本实现范围，以 PH 基线和测试结果确定的阈值、采样率、保留期、资源预算与 OLU 记录。上述第三项是实测参数和具名 Gate 证据，不是尚未选择的技术方案。
