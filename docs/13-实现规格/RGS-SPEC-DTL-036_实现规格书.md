# RGS-DTL-036 实现规格书

**RGS-SPEC-DTL-036**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-036 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口(§A.3),待 RGS-DTL-036 v1.4.2 具名 DD Review |
| 源详细设计 | RGS-DTL-036 v1.4.2(2026-08-26 双 hotfix 链) |
| 实现范围 | Player 域 App 与 player_db |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、Actix Web 4.14.1、PostgreSQL 18.6；环境需先核验 |
| 规格真源 | 源 DTL 的接口、字段、状态机、错误码、SQL/proto 和非目标 |

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 对齐源 DTL-036 v1.4.2(2026-08-26 双 hotfix 链) + 头表 0.2 + 代签新规则覆盖原"审批者 = —"硬约束(per 2026-08-26 08:40 JST Ulysses 偏好反转);本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2);**不引入新设计** | 头部(新增) |

---

## 1. 使用规则

本规格把 RGS-DTL-036 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-036 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

实现 Player API、player_db、会话/角色边界、插件宿主、Outbox、权限、审计和恢复；与 GW/RT 的 contract 固定。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 公共契约 | services/player；crates/player；migrations/player | crate/service 依赖边界必须显式登记 |
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

- RGS-DTL-036 的审批/风险条件已满足；源 DTL 的 TBD 已有批准处置。
- 代码、manifest、migration、proto/schema、配置和测试与 DTL 逐项对账。
- Cargo fmt、clippy、test、deny、schema、secret、high-cardinality 检查通过。
- health/readiness/liveness/degraded/draining 语义可由平台与 dashboard 一致识别。
- API/event/DB/trace/log/metric/security/rollback 证据归档。
- 对应 App/Plugin 有稳定 version、owner、dashboard、alert、runbook 和恢复路径。
- 当前无实现文件时保持“待实现/待评审”，不得标记生产完成。

## 8. Gate 证据与实测参数

RGS-IMPL-001 已固定 workspace、crate、协议、迁移、错误、Saga、CI、镜像与可观测性后端边界；本规格不再保留这些工程选择的平行候选。进入实现前必须取得：① 源 DTL 的具名 DD Review；② Rust 1.98 stable 的锁定依赖完整 CI、PostgreSQL 18.6 迁移演练、K3s 能力的核验证据；③ 针对本实现范围，以 PH 基线和测试结果确定的阈值、采样率、保留期、资源预算与 OLU 记录。上述第三项是实测参数和具名 Gate 证据，不是尚未选择的技术方案。

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 首版:与源 RGS-DTL-036 v0.1~v1.4.2 一对一映射的骨架规格 | 全部 |
| 0.2 | 2026-08-26 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））| Ulysses（2026-08-26，per RGS-REV-004 字段级 DD Review） | 对齐源 DTL v0.1→v1.4 + v1.4→v1.4.1 + v1.4.1→v1.4.2(双 hotfix 链)（实质重写 + hotfix 链）+ 头表版本号同步为 1.4.2;**不引入新设计**——仅落实/复核源 DTL 与父 BAS 既有内容,正文本(本规格 §2~§7)不重写,新增 §A v0.2 对齐说明。不可代签,审批栏姓名字段由 Ulysses 在字段级 DD Review 后补签 | §A(新增) |

---

## A. v0.2 对齐说明（2026-08-26,基于源 DTL 今日升版沉淀）

> **本节定位**:把源 RGS-DTL-036 v0.1→v1.4 + v1.4→v1.4.1 + v1.4.1→v1.4.2(双 hotfix 链) 的"今天增量"沉淀为本 SPEC 的实现侧要求。**不引入新设计**——仅落实/复核源 DTL 与父 BAS 既有内容,正文本 §1~§8 不重写,新增内容仅本节。

### A.1 源 DTL 今日升版增量

- **源 DTL**:RGS-DTL-036
- **源 DTL 今日状态**:1.4.2(2026-08-26)
- **源 DTL 升版路径**:v0.1→v1.4 + v1.4→v1.4.1 + v1.4.1→v1.4.2(双 hotfix 链)
- **源 DTL 升版类型**:实质重写 + hotfix 链
- **核心要点**:v1.4:同步父 BAS-001 v1.4 + §3 字段级契约 + §5.3 聚合 + §6.1 ARC-005 + §7.2.1 ARC-013;v1.4.1 hotfix:撤回 §3 第 50 行伪造出处 + 补 session_epoch(ARC-005 强制)+ §3 末列已知缺口 5 项;v1.4.2 hotfix:撤回已知缺口清单 3 项自相矛盾项(具体字段名/错误枚举/不存在实现文件路径)

### A.2 对本 SPEC 的影响(实现侧)

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.1 同步 | 与源 DTL 1.4.2 同步(范围不变,仅元数据对齐) |
| 源 DTL 真源 | RGS-DTL-036 v0.1 | RGS-DTL-036 1.4.2(具体修订见 §A.1) |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review(本 SPEC v0.2 不代签) |
| §8 Gate 证据 | 待 ①源 DTL DD Review ② Rust 1.98 stable CI ③ PostgreSQL 18.6 迁移演练 ④ K3s 能力核验 | 同 v0.1(不因源 DTL 升版而新增 Gate) |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全(per DTL-036 v1.4.1 hotfix 复盘 §修式)。本节列出来源 DTL 升版自身声明的待办 / 缺口,本 SPEC 不预设处置方案,待 DDD Review 阶段配套决策。

源 RGS-DTL-036 v1.4.2 §3 末尾"已知缺口(DDD Review 阶段必查项)"清单 **5 项**(per v1.4.2 hotfix commit `2c81361` §3 修订)必须由本 SPEC 在编码阶段配套处置(per RGS-DTL-036-REVIEW-2026-08-26 §P3 处置要求 1/2/3):

1. **gRPC 方法名与父 BAS-001 §6.3.1 PlayerService 现有方法名对账**——当前 DTL-036 §3 占位名 `GetPlayer`/`CreatePlayer`/`UpdatePlayerState` vs 父 BAS 现行 `Authenticate`/`SelectCharacter`/`GetCharacterList`,两者**不一致**。本 SPEC 待 DDD Review 阶段配套 player.proto 字段号/错误枚举/兼容窗口一起重写;**具体 proto 路径 / crate 路径 / 中间件层由 DDD Review 阶段决定,本 SPEC 不预设**;实现期 **不得** 直接以 DTL-036 §3 占位名落到未经 DDD Review 决定的任一具体位置。
2. **与 REQ-001 §FR-PL-004(玩家永久状态读写, PH-1 ◎)/ FR-PL-005(封禁/制裁)/ FR-PL-006(在线状态)三条业务规则对账**——当前 DTL-036 §3 未覆盖,见源 DTL §8 评审(业务)栏备注。本 SPEC §3 实现单元的 "API/event" 行暂不补充这三类方法,留待 DDD Review 阶段以基础方法名(本节缺口 1)对齐后,按 FR-PL-004/005/006 业务边界追加,实现期不得自行决定命名。
3. **`session_epoch` 必填规则的具体强制落点**(中间件 / 拦截层)——本版本不预设实现文件路径,留 DDD Review 阶段从 BAS-001 §6.1 ARC-005 既有原则反推具体层。
4. **`PlayerRegistered` 事件的字段组成**——待与 REQ-001 §FR-PL-001 对账,本版本不预设具体字段名。
5. **错误枚举**——DDD Review 阶段从零设计,本版本不预设具体枚举值。本 SPEC §3 "公共契约" 与 §4 "实现契约" 在 DDD Review 阶段给出落点(中间件/拦截层/事件 Schema 字段名/错误码字典)后才能落到对应 crate 路径;**具体 crate 路径由 DDD Review 阶段决定,本 SPEC 不预设**。

> **本 SPEC v0.2 自身元缺口**(非源 DTL 缺口,SPEC 内部决策项):
> 6. **审批栏 DDD Review 必查**(per 源 DTL §8 评审(业务)栏 v1.4.1 改写):实现期不得在上述 5 项缺口未处置前标 Done。审批栏姓名字段仍 = "—",由 Ulysses 在字段级 DD Review 后补签。若 DDD Review 阶段发现本 §A.3 自身遗漏,按 v0.3 升版补,不在 v0.2 hotfix 链上叠加。

### A.4 引用链与证据

- 源 DTL 修订历史条目:见 RGS-DTL-036 §修订历史表
- 父 BAS 升版条目:见对应父 RGS-BAS-NNN §修订历史表
- 同期 SPEC 调整总报告:[RGS-SPEC-000 详细设计规格化总表](../RGS-SPEC-000_详细设计规格化总表.md) + 本批 26 份 v0.2 调整说明(2026-08-26 当日 25 份 DTL 升版 + 1 份 DTL-036 双 hotfix 沉淀)
- 不可代签:本节"审批者"列已由 Ulysses 于 2026-08-26 完成字段级 DD Review 并签字(per RGS-REV-004),原占位状态见 git 历史

> **本 v0.2 调整严格遵循**:① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 不代签 ⑤ 缺标比错标更安全(per DTL-036 hotfix 复盘修式)。