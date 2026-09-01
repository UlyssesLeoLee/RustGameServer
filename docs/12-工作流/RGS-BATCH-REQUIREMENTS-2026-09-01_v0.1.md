# RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1

**综合 Batch 管理平台需求规约（rgs-batch-console + rgs-batch-backend）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BATCH-REQUIREMENTS-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 18:00 JST "batch 需要一个专门的管理界面和对其支持的前后端功能，应该是一个独立的项目，但可以按照其他功能的方式融入架构，从需求文档开始设计" + 18:25 JST "所有内容的批量，包括但不限于 log、数据整理"）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | 2026-09-01 18:00 JST Ulysses "batch 平台" 决策 + 18:25 JST 范围澄清（综合 batch 平台 / 覆盖 log + 数据整理 + 不限于）|
| 关联 | RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1（待起草）+ RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1（待起草）+ RGS-BATCH-PLAN-2026-09-01 v0.1（待起草）|
| 上游规范 | rgs-web 母规范 5 份（per `docs/12-工作流/RGS-WEB-*.md`）+ rgs-web Token 子系统 OLU-WEB 4 份（per `docs/12-工作流/RGS-OLU-WEB-*.md`）+ gm-backend 规范（per `crates/gm-backend/` + `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml`）+ 5 域 gRPC 协议（per `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`）+ shared-platform 横切关注点（per `crates/shared-platform/`，saga 在 `crates/function-plane/`）|
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 文档定位

本文档是 **rgs-batch 综合 Batch 管理平台**需求规约层，描述"为什么做"和"做什么"——不涉及"怎么做"。

按 RGS 项目三层文档规范（per RGS-DTL-001 设计模式：需求规约 / 基本设计 / 详细设计），本文档回答 5W1H 中的 **What + Why**。

**三层文档对应关系**：

| 层级 | 文档 | 回答 |
|---|---|---|
| 需求规约 | **本文档** | What + Why（用户痛点 + 业务目标 + 功能需求 + 非功能需求）|
| 基本设计 | RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1（待起草）| How 概要（架构 + 技术选型 + 模块划分 + 关键流程）|
| 详细设计 | RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1（待起草）| How 细节（API 签名 + 数据模型 + 部署 + 运维 + 安全）|

**项目形态**（per 2026-09-01 18:34 JST Ulysses 拍板 Q2 决策）：

| 项目 | 路径 | 技术栈 | 部署 |
|---|---|---|---|
| **rgs-batch-console**（前端）| `tools/rgs-batch-console/` | Node 22 + 原生 http + 0 依赖（沿用 rgs-web 母规范 v0.1）| envoy 独立 deployment + ClusterIP service（per 2026-09-01 13:03 / 13:05 JST 偏好）|
| **rgs-batch-backend**（后端）| `tools/rgs-batch-backend/` | Rust + actix-web + tokio + 5 域 gRPC client + shared-platform 复用 | envoy 独立 deployment + ClusterIP service（同上）|

> 跟 rgs-web + gm-backend 同模式（per RGS-WEB-REQUIREMENTS-2026-08-26 v0.1 + RGS-IMPL-PLAN-ADMIN-001 §gm-backend），但项目命名空间独立（`rgs-batch-*`），不与现有 tools 冲突。

**与上游规范的关系**：

- 复用 rgs-web 母规范的 0 依赖 Node + 127.0.0.1 only 约束（per RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1 §2）
- 复用 OLU-WEB Token 子系统的 data/ 目录 + lockfile + env value hard ban 范式（per RGS-OLU-WEB-BASIC-DESIGN §5.1.5 + §5.2.1 + NFR-28）
- 复用 gm-backend 的 actix-web APIGW 范式（per `crates/gm-backend/` + `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml`）+ 5 域 ST 业务级 mTLS 实践（per commit `401ac5c` st-11/13/14/15/16 Q8/Q9/Q10 完整 ST, per 9/1 15:50 JST 续 Q10）
- 走 5 域 gRPC 协议集成（per `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md` + 端口 50051-50055 k8s targetPort / 15051-15055 WSL2 port-forward）
- 共享 shared-platform 横切关注点（per `crates/shared-platform/`，saga 在 `crates/function-plane/gateway.rs`）
- 部署 = envoy 独立 deployment（per 2026-09-01 13:03 / 13:05 JST 偏好，边缘层默认 envoy 独立 deployment，**不**选 nginx，**不**选 istio sidecar）
- env value 硬 ban（per 2026-08-27 11:06 JST 硬约束）
- DB 三分类横展开（per 2026-09-01 18:30 JST 原则）

---

## 1. 背景与痛点

### 1.1 现状（per 2026-09-01 18:00 JST git 实证）

| 维度 | 现状 | 数据来源 |
|---|---|---|
| **GM 批量操作** | 5 域 admin 单条接口为主，无批量入口 | `grep -i batch crates/admin-service/src` → 0 match（仅 OTel `install_batch` + `outbox.batch_size` 等内部用法）|
| **定时任务调度** | 无统一调度器，每个域自己手写 `setInterval` 或 nohup | `grep -r "cron\|scheduled" crates/` 仅 OTel/tracing 内部用法 |
| **Log 批量处理** | 手工脚本 + PostgreSQL 直接查询 | `tools/db-seed/` + ad-hoc SQL（无平台化）|
| **数据整理** | 手工 SQL + Excel / CSV 临时导出 | ad-hoc，无版本控制无审计 |
| **rgs-web 现状** | v0.3 10 页面 dashboard，**5 域** gRPC 接入（player / economy / match / social / admin），127.0.0.1:8788 | `tools/rgs-web/public/index.html` + commit `625a3f0`（merge: wbs/WF-1-rgs-web-v0.3-pf 5 域 gRPC + 6 API + http2 + mTLS + port-forward, per 8/26 22:47 JST）|
| **gm-backend 现状** | actix-web APIGW 单服务，端口 8443 (HTTPS) / 8081 (HTTP health) / 9464 (metrics) | `crates/gm-backend/` + `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml`（per 9/1 13:03 JST 部署偏好）|
| **5 域 gRPC 端口** | player 50051 / economy 50052 / match 50053 / social 50054 / admin 50055（k8s targetPort）+ port-forward 15051-15055（WSL2 本地 dev）| `docs/deploy/01-k8s-manifests/0{1-5}-*-service.yaml` targetPort 字段 |
| **shared-platform** | outbox + tracing + span_helpers + retry + dlq + grpc_tracing + rbac + tls + metrics + messaging 等 20 模块已就位（**saga 不在此 crate**）| `crates/shared-platform/`（per 9/1 PT 派工 commit `ffbfb19` 8 worker 派工汇总）|
| **function-plane** | gateway + registry + wasm_host + contract（**saga 编排 runtime** 在此）| `crates/function-plane/` |
| **rgs-web Token 子系统** | v0.1 4 文档 9/1 落地，11 号 page-gantt + 4 选项卡 | `docs/12-工作流/RGS-OLU-WEB-*.md`（今日 9/1 三层 + PLAN）|

> **已知上游文档不一致**（per 2026-09-01 18:30 JST 缺标比错标原则，本表按 git 实证修正，不静默沿用错误引用）：`docs/12-工作流/RGS-OLU-WEB-PLAN-2026-09-01_v0.1.md` §0 引用 commit `23d447b` 标 rgs-web v0.3，但 `git log 23d447b` 实际 message 是 `feat(rgs-web): v0.2-gm GM 后台增强`；rgs-web v0.3 实为 commit `625a3f0`（merge wbs/WF-1-rgs-web-v0.3-pf 5 域 gRPC + 6 API + http2 + mTLS + port-forward, per 8/26 22:47 JST）。本表已修正，OLU-WEB 错误待 v0.2 修。|
| **envoy 部署偏好** | 独立 deployment 模式（per 9/1 13:03/13:05 JST）| 已用于 gm-console 静态资源 + gm-backend 反代 |
| **1 人 12 角色** | 1 人 = 架构师 + 5 域 Lead + SRE + DBA + 安全 + shared-platform + saga 召集人 + PM + ... | per DEC-008 |
| **5 域独立 Lead** | player / economy / match / social / admin 各 1 独立 Lead | per 2026-08-21 JST 拒绝兼任基线 |
| **DB 横展开原则** | Work / Transaction / Master 三分类 | per 2026-09-01 18:30 JST |

### 1.2 用户画像

**Ulysses**（一人公司 12 角色 per DEC-008）：

- 角色：1 人 12 角色（架构师 / 5 域 Lead / SRE / DBA / 安全 / shared-platform / saga 召集人 / PM / ...）
- 工作流：WBS v0.3 145 L4 任务 + 5 域 IMPL-PLAN v0.2 + RACI v1.1
- 环境：Windows 11 + WSL2 Ubuntu + k3s + Rust + node 22
- AI 协作偏好：**token 算 OLU**，1 人·天 ≈ 100K-300K tokens（per RGS-TS-001 v0.7 §6.2.2.1）
- 文档代签偏好：**Mavis 默认代签 Ulysses**（per 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化）
- 拍板决策偏好：**必须用 ask_user 给选项**（per 2026-09-01 14:58 JST）
- 反向代理偏好：**envoy 独立 deployment**（per 2026-09-01 13:03 / 13:05 JST）
- DB 横展开偏好：**Work / Transaction / Master 三分**（per 2026-09-01 18:30 JST）

### 1.3 当前痛点

| # | 痛点 | 影响 | 频率 |
|---|---|---|---|
| 1 | **GM 批量操作无 UI** | 批量发奖 / 封号 / 发邮件只能走 5 域 gRPC 单条接口 × N 次，手工拼接参数易错，事后无审计 | 每次运营活动（每周）|
| 2 | **定时任务无统一调度** | 夜间结算 / 排行榜重算 / 对账靠 nohup + 手工 crontab，任务状态不可见，失败无重试 | 每天 / 每周 |
| 3 | **Log 批量处理靠 ad-hoc SQL** | 玩家行为聚合 / 异常检测 / 日志归档都靠 DBA 临时写 SQL，无版本控制，无平台化 | 每次事故复盘 |
| 4 | **数据整理无平台** | 玩家数据导入 / 导出 / 迁移 / 整理靠 DBA 手工，无审计无回滚无模板复用 | 每次开新服 / 合服 / 备份 |
| 5 | **任务执行无统一监控** | 长跑任务（夜间结算可能 1h+）进度不可见，失败后无重试 + DLQ + 告警 | 每次失败 |
| 6 | **任务审计无统一入口** | 5 域 audit_log 各管各的，跨域 batch 操作无统一 trace + 不可按 player_id 检索 | 每次合规审查 |
| 7 | **5 域 gRPC 接口不直接面向 batch** | 批量场景下需手工分片 / 限流 / 错误聚合 / 进度汇总，5 域 Lead 各自实现 | 每次批量操作 |
| 8 | **DB 横展三分类无平台承载** | per 9/1 18:30 JST 横展开原则，Work / Transaction / Master 落地需 batch 平台支撑 | 设计阶段 + 实现阶段 |
| 9 | **跨 batch 任务依赖手工编排** | "夜间结算 = 排行榜重算 + 货币结算 + 邮件通知 + 数据归档"靠 shell 串行，无失败策略无 DAG 视图 | 每次复合任务 |
| 10 | **AI 协作 token 不可见** | batch 任务自身如需 AI 协助（SQL 生成 / 调试），token 流不入账（per OLU-WEB F-26 预留）| 每次 AI 协助 |

### 1.4 业务目标

| # | 目标 | 度量 |
|---|---|---|
| O-1 | **GM 批量操作可视化** | 选目标实体（player / guild / match）× 选动作（grant / ban / mail）× 预览 → 提交 → 实时进度 → 结果下载 |
| O-2 | **定时任务统一调度** | cron 表达式 / interval / 一次性 三种触发模式；任务执行历史 7 天可见 |
| O-3 | **Log 批量处理平台化** | 多源 log（5 域 gRPC interceptor / kubectl logs / 文件）→ 过滤 → 聚合 → 输出（DB / 文件 / Dashboard）|
| O-4 | **数据整理任务化** | SQL 模板 / 数据迁移 / CSV 导入导出 / 玩家数据归档；可保存为模板复用 |
| O-5 | **任务监控 + 审计** | 实时进度（已完成 / 总数 / 失败数 / ETA）+ 审计 log（操作人 / 时间 / 参数 / 结果 / trace_id）|
| O-6 | **失败重试 + DLQ** | 任务失败自动重试 N 次（指数退避）→ 进 DLQ → 人工干预；不丢任务 |
| O-7 | **跨 batch 编排**（v0.2 P2）| 多 batch 任务 DAG 编排，依赖关系显式 + 失败策略 |
| O-8 | **DB 横展三分类落地** | Master / Transaction / Work 三分清晰（per 9/1 18:30 JST 原则强制）|
| O-9 | **5 域 gRPC 业务级集成** | rgs-batch-backend 内置 5 域 gRPC client，mTLS 业务级（per gm-backend 范式 + 8/27 ST 实践）|
| O-10 | **token 算 OLU（v0.2 集成）**| batch 任务自身如调外部 LLM，token 流自动入账（per OLU-WEB IR-7）|

---

## 2. 用户故事（User Stories）

### 2.1 US-1: GM 批量发奖

> **作为** Ulysses（运营 = 1 人 12 角色）
> **我想要** 在 rgs-batch-console 选 player 列表（按条件筛选或上传 CSV），输入奖励内容（金币 + 道具 + 邮件），预览影响范围，确认后提交
> **以便于** 1 次操作完成 N 个 player 的奖励发放，实时看到进度，失败可重试

**验收标准**：

- [ ] player 列表支持多条件筛选（player_id 范围 / 注册时间 / 等级 / 段位 / 充值额 / 自定义 SQL）
- [ ] CSV 上传（player_id, optional 备注列）
- [ ] 奖励模板：金币 / 道具 / 邮件 / 任意 economy 域支持的操作
- [ ] 预览：影响 player 数 + 总量校验（不超 economy 风控上限）+ 抽样玩家显示
- [ ] 提交：异步执行，task_id 返回，进度 30s 轮询
- [ ] 完成：成功 / 失败 / 部分成功 三态；失败原因可下载 CSV
- [ ] 撤销：仅未执行 + 已执行未生效 的子任务可撤销

### 2.2 US-2: 定时任务调度

> **作为** Ulysses
> **我想要** 创建定时任务（cron 表达式或 interval），选目标（5 域 gRPC 接口 / 任意 SQL / batch 模板）
> **以便于** 无人值守执行夜间结算 / 排行榜重算 / 对账 / 日志归档

**验收标准**：

- [ ] 三种触发模式：cron 表达式（标准 5 段 + 秒级可选）/ interval（X 分钟 / 小时 / 天）/ 一次性（at YYYY-MM-DD HH:MM）
- [ ] 任务类型：5 域 gRPC 调用（pick from list）/ SQL 模板（带参数）/ 任意 batch 模板
- [ ] 执行历史：最近 7 天每次执行的状态 / 耗时 / 参数 / 结果
- [ ] 启停：可暂停 / 启用 / 删除（删除前 confirm + 审计）
- [ ] 超时：可设置 timeout（默认 1h），超时自动 kill + DLQ
- [ ] 告警：失败时通过 mavis cron 通知（v0.2 集成）

### 2.3 US-3: Log 批量处理

> **作为** Ulysses
> **我想要** 配置 log 源（5 域 stderr / file / syslog / kubectl logs），选过滤规则（level ≥ X / 含 pattern / 时间窗），选聚合方式（计数 / 求和 / 95 分位 / 自定义 SQL），选输出（DB / CSV / Dashboard）
> **以便于** 不写代码就能跑 log 分析任务

**验收标准**：

- [ ] log 源：5 域 gRPC interceptor（自动收集）+ 文件 glob + kubectl logs（WSL2 限定）
- [ ] 过滤：level / pattern / time range / 自定义 SQL where
- [ ] 聚合：count / sum / avg / p95 / p99 / group by + 自定义 SQL aggregate
- [ ] 输出：PostgreSQL 表 / CSV 文件 / rgs-web Dashboard embed
- [ ] 调度：可一次性 / 定时（按 cron）
- [ ] 任务结果：保存到 `data/` 目录，UI 可视化

### 2.4 US-4: 数据整理任务

> **作为** Ulysses
> **我想要** 选数据源（PostgreSQL 表 / 5 域 gRPC list 接口 / CSV 文件），选操作（聚合 / 迁移 / 转换 / 导入 / 导出 / 归档），保存为模板
> **以便于** 玩家数据导入导出 / 跨服迁移 / 备份归档可复用，审计可追溯

**验收标准**：

- [ ] 数据源：PostgreSQL 直接 query / 5 域 gRPC list 接口（带分页）/ CSV upload
- [ ] 操作：聚合（group by + 聚合函数）/ 迁移（源 → 目标，支持 dry-run）/ 转换（SQL 模板）/ 导入导出 / 归档（move to archive table）
- [ ] 模板：保存参数化的 SQL / 配置，可复用 + 版本化
- [ ] 审计：执行前必须填理由（≥ 20 字符），写入 `audit_event` 表
- [ ] 回滚：迁移类操作支持自动生成 rollback SQL（基于 before snapshot）

### 2.5 US-5: 任务监控 Dashboard

> **作为** Ulysses
> **我想要** 1 个 dashboard 看所有 batch 任务的状态（运行中 / 待执行 / 已成功 / 已失败 / DLQ），按类型筛选
> **以便于** 不开 5 个工具就能看全局

**验收标准**：

- [ ] 顶部 KPI：今日已完成 / 今日失败 / DLQ 数量 / 平均耗时
- [ ] 任务列表：按 type / status / created_at 筛选，按 type 颜色区分
- [ ] 任务详情：参数（脱敏）/ 进度 / 子任务列表 / 错误日志
- [ ] 实时进度：30s 轮询
- [ ] 任务操作：取消 / 重试 / DLQ 恢复 / 删除

### 2.6 US-6: 任务审计 + 回溯

> **作为** Ulysses
> **我想要** 所有 batch 任务执行前后都有 audit log（操作人 / 时间 / 参数 / 结果 / trace_id），且可按 player_id / 任务类型 / 时间范围 检索
> **以便于** 事故复盘 / 合规审查 / 跨 batch 任务关联

**验收标准**：

- [ ] audit_log 表：master.task_id + transaction.audit_event + work.audit_session 三分（per 9/1 18:30 JST 横展）
- [ ] 记录：操作人（默认 Ulysses）+ IP（per rgs-web 127.0.0.1 only 不实际需要）+ 时间 + 参数（脱敏）+ 结果 + trace_id
- [ ] 检索：按 task_id / player_id / 时间范围 / 操作类型
- [ ] 导出：CSV / JSON
- [ ] 永久保留（per NFR-29）

### 2.7 US-7: 跨 batch 编排（v0.2 可选）

> **作为** Ulysses
> **我想要** 把多个 batch 任务编排为 DAG（依赖关系显式），批量执行
> **以便于** "夜间结算 = 排行榜重算 + 货币结算 + 邮件通知 + 数据归档"的复合任务一次跑

**验收标准**（v0.2，per F-27 P2）：

- [ ] DAG 编辑器（拖拽 / JSON 配置）
- [ ] 任务依赖：上游成功 → 下游启动
- [ ] 失败策略：上游失败 → 下游跳过 / 仍执行 / 等待人工确认

---

## 3. 功能需求（Functional Requirements）

### 3.1 必备 v0.1（Must Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-1 | rgs-batch-console 基础框架 | 工具脚手架 + nav + 路由 + 127.0.0.1 only 监听 | P0 |
| F-2 | rgs-batch-backend 基础框架 | actix-web + mTLS 业务级 + shared-platform 集成 + 5 域 gRPC client | P0 |
| F-3 | **任务定义与提交** | 创建任务（GM 批量 / 定时 / 一次性 / SQL / 模板）+ 异步执行 + task_id 返回 | P0 |
| F-4 | **任务执行引擎** | worker 池（默认 5 worker，可配）+ 分片（每片 N 条 / 每片 1 player）+ 限流（per 域 RPM 配置）| P0 |
| F-5 | **5 域 gRPC 客户端** | rgs-batch-backend 内置 player / economy / match / social / admin 5 域 gRPC client（per `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`）| P0 |
| F-6 | **任务进度推送** | 30s 轮询 task 进度（已完成 / 总数 / 失败数 / ETA）+ 写时立即触发 | P0 |
| F-7 | **任务历史** | 最近 7 天执行历史 + 详情（参数 / 结果 / 审计）| P0 |
| F-8 | **失败重试 + DLQ** | 默认重试 3 次（指数退避 100ms / 200ms / 400ms）→ 进 DLQ → 人工干预 | P0 |
| F-9 | **定时调度** | cron 表达式（标准 5 段）/ interval / 一次性 三种触发模式 | P0 |
| F-10 | **审计 log** | master.task + transaction.audit_event + work.audit_session 三分（per 9/1 18:30 JST 横展）| P0 |
| F-11 | **envoy 独立部署** | rgs-batch-console + rgs-batch-backend + envoy 三个独立 deployment + ClusterIP service（per 9/1 13:03 / 13:05 JST 偏好）| P0 |
| F-12 | **127.0.0.1 only** | rgs-batch-console 监听 127.0.0.1（沿用 rgs-web 母规范 §2.1）| P0 |
| F-13 | **mTLS 业务级** | 5 域 gRPC 调用走 mTLS（per 5 域 ST 业务级 mTLS 实践 + commit `401ac5c` st-11/13/14/15/16 Q8/Q9/Q10 完整 ST, per 9/1 15:50 JST 续 Q10）+ 5 域证书复用 8/27 ST 导出 SOP | P0 |
| F-14 | **env value hard ban** | 凭据走 env var，永不打印值（per 2026-08-27 11:06 JST 硬约束 + NFR-30）| P0 |
| F-15 | **Work / Transaction / Master 横展** | DB 设计按 9/1 18:30 JST 横展原则，三分清晰（详见 §4）| P0 |
| F-16 | **5 域独立 Lead 不兼任** | batch 域不与 5 域 Lead 兼任（per 8/21 JST 拒绝兼任基线，新增独立 batch Lead，扩展为 6 域）| P0 |

### 3.2 重要 v0.1（Should Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-17 | Log 批量处理 | 多源 log + 过滤 + 聚合 + 输出（per US-3）| P1 |
| F-18 | 数据整理任务 | SQL 模板 + 迁移 / 转换 / 导入导出（per US-4）| P1 |
| F-19 | 任务模板复用 | SQL / gRPC 调用 / log 处理保存为模板，参数化 | P1 |
| F-20 | 任务结果下载 | CSV / JSON 导出执行结果（成功 + 失败列表）| P1 |
| F-21 | 任务撤销 | 未执行 + 已执行未生效 子任务可撤销 | P1 |
| F-22 | 任务超时 | 可配 timeout，超时自动 kill + DLQ | P1 |
| F-23 | 任务失败告警 | 失败时通过 mavis cron 通知（v0.2 集成，per OLU-WEB IR-8）| P1 |
| F-24 | Rollback SQL 生成 | 迁移类操作自动生成 before snapshot + rollback SQL | P1 |
| F-25 | 任务 KPI Dashboard | 顶部 KPI：今日完成 / 失败 / DLQ / 平均耗时 | P1 |
| F-26 | 任务审计检索 | 按 task_id / player_id / 时间范围 / 操作类型 | P1 |

### 3.3 可选 v0.2（Could Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-27 | 跨 batch DAG 编排 | 多任务依赖 + 失败策略（per US-7）| P2 |
| F-28 | WebSocket 实时推送 | 替代 30s 轮询 | P2 |
| F-29 | 流式处理 | Kafka / NATS 流式 batch | P2 |
| F-30 | 任务优先级 | 多 worker 池 + 优先级队列 | P2 |
| F-31 | mavis cron 失败告警 | 通过 mavis cron self-reminder 通知（per OLU-WEB IR-8 v0.2）| P2 |
| F-32 | provider token counter | 接入 AI 协作 token 真实值（per OLU-WEB F-25）| P2 |
| F-33 | batch 任务执行 AI 协作 | AI 协助生成 SQL / 调试 batch 任务 | P2 |
| F-34 | 深链 / 嵌入式 widget | 嵌入 rgs-web / gm-console 页面 | P2 |
| F-35 | 跨 batch 任务 chain | batch 任务完成后触发另一个 batch | P2 |

### 3.4 不做（Won't Have）

| # | 需求 | 不做理由 |
|---|---|---|
| F-W1 | 多用户 / RBAC | DEC-008 一人公司，127.0.0.1 only 足够安全 |
| F-W2 | 公网部署 | 一人公司本机工具，违反 127.0.0.1 only + 安全策略 |
| F-W3 | 5 域 binary 直接调外部 LLM | 5 域 Rust gRPC 当前不调外部 LLM（per OLU-WEB F-W5），batch 域预留 F-33 不实现 |
| F-W4 | 与 5 域 Lead 兼任 batch Lead | per 2026-08-21 JST 拒绝兼任基线，batch 域必须独立 Lead |
| F-W5 | Docker / k8s 强制部署 | 一人公司本机工具，envoy 部署到 k3s 但不强制（per rgs-web OLU-WEB §F-W2 同样基线）|
| F-W6 | 商用流式引擎（Spark / Flink）| 过度工程，v0.1 简单 worker 池够用 |
| F-W7 | 商业 ETL 工具 | 1 人 12 角色场景下自研够用 |
| F-W8 | 任务定义拖拽式 UI | v0.1 表单 / JSON 提交足够，v0.2 评估 |
| F-W9 | 跨 batch 任务并行多 worker 池 | v0.1 单 worker 池 + 限流足够 |

---

## 4. 数据需求（Data Requirements）

> **DB 三分类横展开原则**（per 2026-09-01 18:30 JST）：每张表必须归类 Work / Transaction / Master 之一，**不允许**只列一类合并；类似 X/Y/Z 多分类一律横展细化。

### 4.1 Master（参考数据，slowly changing，SCD-2）

| # | 表 | 字段（核心）| 备注 |
|---|---|---|---|
| M-1 | `task_def` | task_id (PK) / task_type / cron_expr / target / params / owner / status / created_at / updated_at | 任务定义，slowly changing（SCD-2，v0.2 加历史表）|
| M-2 | `task_template` | template_id (PK) / name / type / sql_template / params_schema / version / created_at | 可复用模板，version 字段支持 v0.2 模板回滚 |
| M-3 | `data_source` | source_id (PK) / type / conn_str_ref / credentials_ref | 数据源配置，**凭据只引用 env var 名，不存值**（per 2026-08-27 11:06 JST 硬 ban）|
| M-4 | `worker_pool` | pool_id (PK) / domain / max_concurrent / rpm_limit / enabled | 5 域 gRPC 限流配置，per 域 owner 拍板 |
| M-5 | `schedule` | schedule_id (PK) / task_id / cron_expr / next_run_at / enabled | 定时任务调度表，per 5 域独立 Lead + batch Lead 维护 |

### 4.2 Transaction（事件流水，append-only）

| # | 表 | 字段（核心）| 备注 |
|---|---|---|---|
| T-1 | `task_execution` | exec_id (PK) / task_id / started_at / finished_at / status / params_snapshot / result_summary / trace_id | 每次执行 1 条，append-only |
| T-2 | `sub_task` | sub_id (PK) / exec_id / target_id / status / retry_count / started_at / finished_at / error / result | 每个子任务 1 条（如 1 player），append-only |
| T-3 | `audit_event` | event_id (PK) / exec_id / operator / action / params_hash / result / created_at / trace_id | 审计事件，append-only，永久保留（per NFR-29）|
| T-4 | `dlq_event` | dlq_id (PK) / exec_id / sub_id / error / retry_count / first_failed_at / last_retried_at / resolved_at | DLQ 事件，append-only |
| T-5 | `log_event` | event_id (PK) / source / level / message / fields (JSONB) / ts | log 批量处理源事件，append-only，30 天后归档 |
| T-6 | `data_migration` | migration_id (PK) / source / target / before_snapshot / rollback_sql / applied_at | 数据迁移 + rollback 记录，append-only |

### 4.3 Work（作业中，session-bound，完成后清理或归档）

| # | 表 | 字段（核心）| 备注 |
|---|---|---|---|
| W-1 | `task_progress` | exec_id (PK) / completed / failed / total / eta_seconds / updated_at | 实时进度，30s 轮询写入，任务结束清理或转 T-1 |
| W-2 | `task_buffer` | exec_id (PK) / chunk_id / data (JSONB) / status | 子任务分片缓冲，session-bound，任务结束清理 |
| W-3 | `audit_session` | session_id (PK) / operator / ip / started_at / finished_at | 1 次操作 = 1 session，结束后归档到 T-3 |
| W-4 | `log_buffer` | session_id (PK) / chunk_id / raw_data (JSONB) / status | log 处理缓冲，session-bound，任务结束清理 |
| W-5 | `migration_buffer` | session_id (PK) / chunk_id / source_data (JSONB) / target_status | 迁移缓冲，session-bound，任务结束清理或转 T-6 |

### 4.4 Schema 划分

- **Master 表**：`batch_master` schema（task_def / task_template / data_source / worker_pool / schedule）
- **Transaction 表**：`batch_transaction` schema（task_execution / sub_task / audit_event / dlq_event / log_event / data_migration）
- **Work 表**：`batch_work` schema（task_progress / task_buffer / audit_session / log_buffer / migration_buffer）

### 4.5 存储位置

- Master / Transaction 表：**PostgreSQL 共享实例**（per 5 域共用 PG）+ 独立 schema `batch_*`（per IPA 横展）
- Work 表：内存（rgs-batch-backend 进程内）+ 时序落盘（per rgs-web ai-ledger.jsonl 模式，v0.2）
- 模板 / 凭据引用：env var 注入（per 2026-08-27 11:06 JST 硬 ban）
- 二进制文件：5 域 S3 / 文件系统 + 独立 bucket `rgs-batch-*`（v0.2）

### 4.6 锁文件

- `data/.lock` 原子锁（per rgs-web OLU-WEB §5.1.5 实践，多写者防御）
- 锁获取：`fs.openSync('.lock', 'wx')` 原子创建
- 锁失败：retry 3 次，指数退避 100ms / 200ms / 400ms
- 死锁防护：mtime > 1h 视为僵死，启动时删除

---

## 5. 集成需求（Integration Requirements）

| # | 集成 | 描述 | v0.1 | v0.2 |
|---|---|---|---|---|
| IR-1 | **5 域 gRPC 客户端** | rgs-batch-backend 内置 player / economy / match / social / admin 5 域 gRPC client，端口 50051-50055（k8s targetPort, per `docs/deploy/01-k8s-manifests/0{1-5}-*-service.yaml`）; 本地 dev 走 port-forward 15051-15055 | ✅ | ✅ |
| IR-2 | **shared-platform 复用** | outbox / tracing / span_helpers / retry / dlq / grpc_tracing / rbac / tls（per `crates/shared-platform/`，saga 编排 runtime 在 `crates/function-plane/gateway.rs`）| ✅ | ✅ |
| IR-3 | **PostgreSQL** | 用 5 域共享 PostgreSQL 实例，独立 schema `batch_master` / `batch_transaction` / `batch_work` | ✅ | ✅ |
| IR-4 | mavis runtime | mavis cron self-reminder 失败告警（per OLU-WEB IR-8 v0.2 实践）| ❌ | ✅ |
| IR-5 | rgs-web 联动 | rgs-web 加 page-batch，调用 rgs-batch-backend 代理（最小集成）| ✅ | ✅（深联动）|
| IR-6 | gm-backend 复用 | 部分 GM 操作（如 audit_log）走 gm-backend，**不重复实现** | ✅ | ✅ |
| IR-7 | **env 凭据** | K3S_TOKEN / BATCH_DB_PASSWORD / 5 域 gRPC 证书路径 / GitHub PAT 全部 env var 注入，**永不打**印值 | ✅ | ✅ |
| IR-8 | **envoy ingress** | 静态资源走 envoy file_system HTTP filter，mTLS termination（per 9/1 13:03/13:05 JST 偏好）| ✅ | ✅ |
| IR-9 | OLU 报表 | rgs-batch-console 自身运行 OLU token 入账（per OLU-WEB F-26 + IR-7）| ❌ | ✅ |
| IR-10 | GitHub 联动 | batch 任务审计事件 → issue 评论（per OLU-WEB 范式 + F-23 失败告警）| ❌ | ✅ |
| IR-11 | OTel tracing | 跨 batch + 5 域 gRPC 链路追踪（per shared-platform tracing_init）| ✅ | ✅ |
| IR-12 | k3s HPA | 长跑任务（夜间结算）自动扩缩（per 5 域 HPA 实践）| ❌ | ✅ |

---

## 6. 非功能需求（Non-Functional Requirements）

> **部分 NFR 继承 rgs-web 母规范 v0.1 NFR-1 至 NFR-21**（性能 / 可用性 / 安全性 / 可维护性 / 可移植性），不重复；本节只列 batch 平台**新增**的 NFR。

| # | 指标 | 目标 | 备注 |
|---|---|---|---|
| NFR-22 | 任务提交吞吐 | ≥ 100 子任务/秒 | 5 worker 池默认配置 |
| NFR-23 | 单 batch 任务容量 | ≤ 100K 子任务 | 超出分批提交 |
| NFR-24 | 长跑任务 | ≤ 24h 单任务 | 超时自动 kill + DLQ |
| NFR-25 | 任务不丢 | 强制 | 任务定义 + 执行历史全审计，进程崩溃后重启续跑 |
| NFR-26 | 5 域 gRPC 失败处理 | 重试 3 次（指数退避） + DLQ | 沿用 gm-backend mTLS 实践 + commit `401ac5c` |
| NFR-27 | 任务进度可见 | 30s 内 | 轮询 + 写时立即触发 |
| NFR-28 | 审计检索 | < 1s 返 100 条 | 索引：task_id / player_id / created_at |
| NFR-29 | 数据保留 | 任务定义永久 / 执行历史 90 天 / 审计永久 / log 30 天后归档 | 自动归档 |
| **NFR-30** | **env value 永不出现在日志 / 响应** | **强制** | per 2026-08-27 11:06 JST 硬 ban |
| **NFR-31** | **127.0.0.1 only** | **强制** | rgs-batch-console 跟 rgs-web 同模式（per rgs-web §NFR-10）|
| **NFR-32** | **mTLS 业务级** | **5 域 gRPC 调用强制** | per 5 域 ST 业务级 mTLS 实践（commit `401ac5c` st-11/13/14/15/16 Q8/Q9/Q10 完整 ST） |
| NFR-33 | 资源上限 | 单 batch 任务 ≤ 1GB 内存 / ≤ 4 CPU 核 | k3s resource limit |
| NFR-34 | 冷启动 | rgs-batch-backend ≤ 3s，rgs-batch-console ≤ 1s | 沿用 rgs-web 母规范 §NFR-5 |
| NFR-35 | 横展三分类 | DB 表 100% 归类 Work/Transaction/Master | per 9/1 18:30 JST 原则强制 |

---

## 7. 约束（Constraints）

### 7.1 治理约束

- per DEC-008：一公司 12 角色，无 RBAC，rgs-batch-console 是 1 人工具
- per DEC-005 + 2026-08-21 JST：5 域独立 Lead 兼任禁止，batch 域**新增独立 Lead**（扩展 5 域 → 6 域）
- per 2026-08-27 19:39 / 20:56 / 21:59 JST：Mavis 默认代签 Ulysses（修订历史"审批者"列）
- per 2026-08-27 11:06 JST：env value 打印硬 ban
- per 2026-08-26 04:30 JST：禁"per X 历史形态"回溯叙事，引用 RGS-* 必须 `git log -p --follow` 实证
- per 2026-09-01 14:58 JST：拍板决策必须用 ask_user 给选项
- per 2026-09-01 13:03 / 13:05 JST：边缘层 envoy 独立 deployment，**不**选 nginx，**不**选 istio sidecar
- per 2026-09-01 18:30 JST：DB 横展三分类（Work / Transaction / Master）

### 7.2 技术约束

- 复用 rgs-web 母规范 v0.1 0 依赖 Node + 127.0.0.1 only 约束 — 仅适用 rgs-batch-console
- rgs-batch-backend 用 actix-web + tokio + 5 域 gRPC client（per `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`）
- 不引入 dhtmlx-gantt / chart.js / 任何 npm 依赖（per rgs-web OLU-WEB §3.2）
- 不引入 better-sqlite3（per rgs-web OLU-WEB 决策）
- 不引入 Kafka / NATS（v0.1 简单 worker 池够用，v0.2 评估）
- 数据存储：PostgreSQL 共享实例 + 独立 schema（per IPA 横展）
- 跨域调用：gRPC + mTLS（per 5 域 ST 业务级 mTLS 实践 commit `401ac5c` + 8/27 ST 导出 SOP）
- 反向代理：envoy 独立 deployment + ClusterIP service（per 9/1 13:03/13:05 JST 偏好）
- 凭据：env var 注入，永不打印（per 8/27 11:06 JST 硬 ban）
- 1 写者约束：rgs-batch-backend 单进程单线程（per rgs-web OLU-WEB §5.1.5）

### 7.3 时间约束

- v0.1 4-6 周落地（per 9/1 18:34 JST Ulysses 决策"独立双项目"，参考 rgs-web v0.3 已 4 周完成 + OLU-WEB v0.1 4 周计划）
- v0.2 DAG + 流式 + AI 协作 集成：6-10 周
- v0.3 商业化 / 多租户（如果需要）：待 v0.2 评估

### 7.4 资源约束（per token-OLU 框架）

- 1 人·天 ≈ 100K-300K tokens（per RGS-TS-001 v0.7 §6.2.2.1）
- 1 人·周 ≈ 1M tokens
- 5 域独立 Lead × 14-18 周 = 80-120M tokens（per RGS-TS-001 v0.4 §6.2 草案）
- batch 域 v0.1 估算 ≈ 4-6M tokens（参考 rgs-web + OLU-WEB v0.1 总和），需在 RGS-BATCH-PLAN-2026-09-01 细化

---

## 8. 验收标准（Acceptance Criteria）

### 8.1 v0.1 验收

#### 8.1.1 基础框架

- [ ] `node tools/rgs-batch-console/server.js` 启动 < 1s，127.0.0.1:8789 监听（区别于 rgs-web 的 8788，NFR-34）
- [ ] `cargo run -p rgs-batch-backend` 启动 < 3s，0.0.0.0:8790 内部监听（ClusterIP service，k3s 内部访问，不直接暴露 127.0.0.1，NFR-34）
- [ ] 三个独立 deployment：rgs-batch-console / rgs-batch-backend / envoy，ClusterIP service（per 9/1 13:03/13:05 JST 偏好）
- [ ] mTLS 业务级：5 域 gRPC 调用通过证书验证（per 5 域 ST 业务级 mTLS 实践 commit `401ac5c` + 8/27 ST 导出 SOP）
- [ ] 127.0.0.1 only：rgs-batch-console 不接受 0.0.0.0（per rgs-web §NFR-10 + NFR-31）

#### 8.1.2 任务定义与执行

- [ ] 创建 GM 批量发奖任务（10 个 player），异步执行
- [ ] 任务进度 30s 可见（NFR-27）
- [ ] 任务完成：成功 N / 失败 0
- [ ] 任务历史可在 console 查看（F-7）
- [ ] 审计 log 写入 master + transaction + work 三分（F-10 + NFR-35）

#### 8.1.3 定时调度

- [ ] 创建 cron 表达式任务（每 5 分钟）
- [ ] 等待 10 分钟，任务自动执行 2 次
- [ ] 任务执行历史可见（F-7）
- [ ] 暂停 / 启用 / 删除操作生效

#### 8.1.4 失败处理

- [ ] 模拟 5 域 gRPC 失败（如 admin service down）
- [ ] 任务自动重试 3 次（指数退避 100ms / 200ms / 400ms，per NFR-26）
- [ ] 进入 DLQ（F-8）
- [ ] 人工干预后从 DLQ 重启

#### 8.1.5 数据整理

- [ ] 创建 SQL 模板任务（聚合 player 表）
- [ ] 任务执行，结果保存到 PostgreSQL `batch_transaction` schema
- [ ] CSV 导出执行结果（F-20）
- [ ] Rollback SQL 自动生成（F-24，迁移类操作）

### 8.2 性能基准

- [ ] 任务提交吞吐 ≥ 100 子任务/秒（NFR-22）
- [ ] 5 worker 池跑 1K 子任务 ≤ 30s
- [ ] 5K 子任务 ≤ 5min
- [ ] 100K 子任务 ≤ 1h（分批提交，per NFR-23）
- [ ] 审计检索 100 条 < 1s（NFR-28）

### 8.3 验收者（per 2026-08-26 08:40 JST 代签新规则）

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | 架构师（**Mavis 接手 agent per DEC-008**）| 2026-09-01 |
| batch 域 Lead | _待 DDD Review 阶段补签_ | — |
| 5 域 Lead（player / economy / match / social / admin）| _待 DDD Review 阶段补签_ | — |
| shared-platform Lead | _待 DDD Review 阶段补签_ | — |
| cluster-ops Lead | _待 DDD Review 阶段补签_ | — |
| SRE Lead | _待 DDD Review 阶段补签_ | — |
| DBA | _待 DDD Review 阶段补签_ | — |
| 安全 | _待 DDD Review 阶段补签_ | — |
| PM | _待 DDD Review 阶段补签_ | — |

---

## 9. 已知缺口 + 风险（per 2026-09-01 18:30 JST 缺标比错标原则）

### 9.1 已知缺口（显式列出，不假装覆盖）

| # | 缺口 | 影响 | 待补阶段 |
|---|---|---|---|
| GAP-1 | 跨 batch DAG 编排 F-27 推迟到 v0.2 | v0.1 不支持多任务依赖 | v0.2 |
| GAP-2 | WebSocket 实时推送 F-28 推迟到 v0.2 | v0.1 30s 轮询 | v0.2 |
| GAP-3 | 流式处理 F-29 推迟到 v0.2 | v0.1 不支持 Kafka / NATS | v0.2 |
| GAP-4 | mavis cron 失败告警 F-23 推迟到 v0.2 | v0.1 无告警通知 | v0.2 |
| GAP-5 | 任务优先级 F-30 推迟到 v0.2 | v0.1 FIFO 队列 | v0.2 |
| GAP-6 | AI 协助生成 SQL F-33 推迟到 v0.2 | v0.1 人工写 SQL | v0.2 |
| GAP-7 | rgs-web 深联动 F-34 推迟到 v0.2 | v0.1 rgs-web 最小集成 | v0.2 |
| GAP-8 | 任务模板版本化 v0.1 简化实现 | 仅最新版本可用，v0.2 加版本回滚 | v0.2 |
| GAP-9 | Rollback SQL 验证 v0.1 简化 | 仅生成，不自动验证，v0.2 加 dry-run | v0.2 |
| GAP-10 | 任务超时 kill v0.1 简化 | 仅标记超时，v0.2 加 worker 强制 kill | v0.2 |
| GAP-11 | 跨域 saga 触发 batch 任务 v0.1 不集成 | 5 域 saga 完成后触发 batch 任务（如自动补发）v0.2 评估 | v0.2 |
| GAP-12 | batch 域 Lead RACI 同步 | RACI v1.1 是 5 域 + 4 域 Lead，batch 域需 RACI v1.2 扩展 | RACI v1.2 DDD 阶段 |

### 9.2 风险

| # | 风险 | 影响 | 缓解 |
|---|---|---|---|
| R-1 | 5 域 gRPC 协议 v0.1 业务级集成深度 | batch 调用 5 域可能触发限流 | 5 域 RPM 配置（per worker_pool M-4，域 owner 拍板）|
| R-2 | 长跑任务资源消耗 | 夜间结算可能 1h+，占用 worker | k3s resource limit（NFR-33）+ 自动扩缩（v0.2 IR-12）|
| R-3 | 任务撤销原子性 | 部分子任务已执行难撤销 | 仅未执行 + 未生效可撤销（per F-21）|
| R-4 | DB 横展落地成本 | master / transaction / work 三分增加设计成本 | per 9/1 18:30 JST 原则强制（不妥协）|
| R-5 | batch 域 Lead 兼任 vs 独立 | 1 人 12 角色，新增独立 Lead 突破 5 域基线 | per 2026-08-21 JST 拒绝兼任，扩展为 6 域 + 申请额外 Lead 编制 |
| R-6 | AI 协作 token 估算不准 | 5 worker 池实际 token 流未知 | v0.1 估算（per OLU-WEB 公式 `message_count × 5K`），v0.2 校准 |
| R-7 | 5 域 gRPC 接口稳定性 | 5 域协议 v0.1 阶段可能 break | 5 域 gRPC client 包一层 retry + circuit breaker（per gm-backend 实践）|
| R-8 | k3s 资源争用 | 5 域 + batch + rgs-web + gm-backend 共享 k3s 集群 | HPA + resource limit + namespace 隔离（per 9/1 部署恢复实践）|

---

## 10. 后续

### 10.1 待起草文档

| 文档 | 状态 | 备注 |
|---|---|---|
| RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1 | ⏳ 待起草 | How 概要（架构 + 技术选型 + 模块划分 + 关键流程）|
| RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1 | ⏳ 待起草 | How 细节（API 签名 + 数据模型 + 部署 + 运维 + 安全）|
| RGS-BATCH-PLAN-2026-09-01 v0.1 | ⏳ 待起草 | 设计总览 + 4-6 周实施计划 |

### 10.2 待同步事项

- [ ] AGENTS.md v0.x 追加 §x.x "batch 域派生约束"（v0.1 落地后 commit）
- [ ] WBS v0.x 追加 batch 域 L4 任务（约 30-50 条，per 9/1 18:34 JST 范围）
- [ ] RACI v1.x 扩展为 6 域（5 域 + batch 域）+ 1 batch Lead 签字栏
- [ ] IMPL-PLAN-BATCH-001 v0.1 起草
- [ ] RACI-BATCH-V1 v0.1 起草（独立 RACI 文档，per 5 域独立 RACI 范式）

### 10.3 待协调事项

- [ ] 5 域 Lead 确认：5 域 gRPC 接口是否支持 batch 模式调用（分片 / 限流 / 错误聚合）
- [ ] DBA 确认：PostgreSQL 共享实例 + 独立 schema `batch_*` 是否可行
- [ ] SRE 确认：k3s 资源上限 + namespace 隔离策略
- [ ] 安全确认：mTLS 证书复用 5 域 ST 业务级 mTLS 实践（commit `401ac5c`）+ 8/27 ST 导出 SOP

---

## 11. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师（**Mavis 接手 agent per DEC-008**）| 首版：1 文档定位 + 2 背景与痛点（10 现状 + 12 痛点 + 10 业务目标）+ 3 用户故事（7 US）+ 4 功能需求（16 P0 + 10 P1 + 9 P2 + 9 Won't）+ 5 数据需求（Master 5 + Transaction 6 + Work 5 横展）+ 6 集成需求（12 IR）+ 7 NFR（14 NFR 新增）+ 8 约束（治理 8 + 技术 10 + 时间 4 + 资源 1）+ 9 验收标准（5 类 + 5 性能基准）+ 10 已知缺口（12 GAP）+ 11 风险（8 R）+ 12 后续（3 文档 + 5 同步 + 4 协调）+ 13 修订历史（v0.1）|

**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
**审批**：架构师（**Mavis 接手 agent per DEC-008**）
**代签授权**：2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化（Mavis 默认代签 Ulysses，无需再问）
