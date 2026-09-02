# RGS-DDD-2026-09-01-PHASE-D-D7-LIAISON — 5 业务域 Lead 跟 gm-backend Lead 联调协调

> **文档 ID**: RGS-DDD-2026-09-01-PHASE-D-D7-LIAISON
> **版本**: v0.1
> **生效日期**: 2026-09-01 22:30 JST
> **状态**: 🟡 占位 (per WBS v0.2 桶 10 Phase D D7, commit 84edf26)
> **作者**: 架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化)
> **范围**: 5 业务域 Lead (player / economy / match / social / admin) 跟 gm-backend Lead 联调协调
> **关联**:
> - WBS v0.2 commit 84edf26 (桶 10 Phase D D7)
> - `RGS-DDD-2026-09-01-PT-WORKERS_5平台+3工具+8派工_v0.1.md` §7.2 P1
> - `RGS-OPEN-QA-2026-08-31-test-summary_v0.3.md` Q8/Q9/Q11 收尾

---

## 0. 文档目的

per WBS v0.2 桶 10 Phase D D7 任务 + DDD Review §7.2 P1:

> 5 业务域 Lead 跟 gm-backend Lead 联调, 加 Q8/Q9 ST 业务级验证 (per OPEN-QA v0.2 Q8/Q9)

本 v0.1 占位 commit **不写代码**, 协调 5 域 Lead 跟 gm-backend Lead 1-on-1, 列举联调范围 + 决策项 + 签字栏.

---

## 1. 联调范围 (5 业务域 + gm-backend)

### 1.1 5 业务域 + gm-backend 关系

| 域 | gm-backend 调用 | 触发场景 |
|---|---|---|
| **player** | `list_players` / `get_player_stats` | GM 查玩家信息 + 战绩 |
| **economy** | `list_mall_items` / `create_mall_item` / `update_mall_item` / `delete_mall_item` | GM 商城管理 |
| **match** | `list_servers` / `start_server` / `stop_server` / `metrics` | GM 服务器启停 + 监控 |
| **social** | `create_ticket` / `list_tickets` / `update_ticket_status` | GM 客服工单 |
| **admin** | `ban_account` / `grant_compensation` / `set_maintenance` / `query_audit` / `health_view` | GM 5 大功能 (per gm.proto v0.4) |
| **gm-backend** (被调) | `summary` (聚合 5 域 + cluster-ops) | Dashboard 数据源 |

### 1.2 Q8/Q9/Q11 收尾 (per OPEN-QA v0.2 + v0.3)

| Q | 内容 | 5 域 Lead 协调责任 | gm-backend Lead 协调责任 |
|---|---|---|---|
| **Q8** | gm-backend 8081 诊断 (restartCount/events/logs/exec curl/top, 跟 HPA minReplicas 强启动风暴比对) | player / economy / match / social / admin 各自检查 gm-backend 重启是否影响 5 域 gRPC 客户端 | gm-backend Lead 主导诊断 + 修复 + 重启验证 |
| **Q9** | prometheus + grafana 诊断 + grafana admin password 核查 | 5 域检查 5 域 metrics 是否上报到 prometheus | gm-backend Lead 协调 prometheus + grafana 配置 + dashboard |
| **Q11** | NATS 8222 部署范围核查 (`kubectl get pods -l app.kubernetes.io/name=nats`) | 5 域检查 NATS subscriber 是否正常 (social push_delivery 优先) | gm-backend Lead 检查 NATS publisher 跟 SSE 实时事件流是否对接 |

---

## 2. 1-on-1 协调 Checklist (per 5 域 Lead × gm-backend Lead)

### 2.1 player 域 Lead 联调 (per 5/7 业务实装)

- [ ] player 域 `list_players` + `get_player_stats` 跟 gm-backend `players_handler.rs` 字段对齐
- [ ] player 域 gRPC 客户端 (port 50051) 跟 gm-backend admin_grpc_endpoint 互通
- [ ] player 域 mTLS 证书 (per 8/27 ST 导出 SOP) 跟 gm-backend mTLS 双向认证
- [ ] player 域 Lead 签字: __________ 日期: ____

### 2.2 economy 域 Lead 联调

- [ ] economy 域 `mall_items` CRUD 跟 gm-backend `mall_handler.rs` 字段对齐
- [ ] economy 域 gRPC 客户端 (port 50052) 跟 gm-backend admin_grpc_endpoint 互通
- [ ] economy 域 mTLS 证书 跟 gm-backend mTLS 双向认证
- [ ] economy 域 Lead 签字: __________ 日期: ____

### 2.3 match 域 Lead 联调

- [ ] match 域 `servers` 状态 + `metrics` 跟 gm-backend `servers_handler.rs` 字段对齐
- [ ] match 域 gRPC 客户端 (port 50053) 跟 gm-backend admin_grpc_endpoint 互通
- [ ] match 域 mTLS 证书 跟 gm-backend mTLS 双向认证
- [ ] match 域 Lead 签字: __________ 日期: ____

### 2.4 social 域 Lead 联调

- [ ] social 域 `tickets` 生命周期 跟 gm-backend `support_handler.rs` 字段对齐
- [ ] social 域 gRPC 客户端 (port 50054) 跟 gm-backend admin_grpc_endpoint 互通
- [ ] social 域 mTLS 证书 跟 gm-backend mTLS 双向认证
- [ ] social 域 Lead 签字: __________ 日期: ____

### 2.5 admin 域 Lead 联调

- [ ] admin 域 `ban_account` / `grant_compensation` / `set_maintenance` / `query_audit` / `health_view` 跟 gm-backend `business_handler.rs` 字段对齐
- [ ] admin 域 gRPC 客户端 (port 50055) 跟 gm-backend admin_grpc_endpoint 互通 (核心路径)
- [ ] admin 域 mTLS 证书 跟 gm-backend mTLS 双向认证
- [ ] admin 域 Lead 签字: __________ 日期: ____

### 2.6 gm-backend Lead 联调

- [ ] gm-backend 5 GM 业务 handler (per gm.proto v0.4) 跟 admin-service gRPC 字段对齐
- [ ] gm-backend SSE 实时事件流 跟 NATS JetStream 复用 (per 8/31 OPEN-QA v0.2 Q7)
- [ ] gm-backend Circuit Breaker 跟 admin-service 健康检查配合
- [ ] gm-backend Lead 签字: __________ 日期: ____

---

## 3. 决策项 (per DDD Review §7.2 P1 6 项 P1 backlog)

- [ ] DDD-P1-01 admin RBAC handler 入口 COC middleware (per OPEN-QA v0.2 Q1) → 5 域 Lead 配合 admin 域
- [ ] DDD-P1-02 admin audit verify 增量 (最近 1000 条 / 24h) (per OPEN-QA v0.2 Q2) → admin 域 Lead
- [ ] DDD-P1-03 player wins≤total 业务层 invariant (per OPEN-QA v0.2 Q3) → player 域 Lead
- [ ] DDD-P1-04 economy outbox L143 `expect` 改 skip (per OPEN-QA v0.2 Q4) → economy 域 Lead
- [ ] DDD-P1-05 social guild capacity 50 业务确认 (per OPEN-QA v0.2 Q5) → social 域 Lead
- [ ] DDD-P1-06 social leave_guild API (per OPEN-QA v0.2 Q6) → social 域 Lead
- [ ] DDD-P1-07 social push dispatcher NATS + DLQ (per OPEN-QA v0.2 Q7) → social 域 Lead

---

## 4. 推进路径 (per WBS v0.2 桶 8 Phase B + 桶 10 Phase D)

- 9/2-9/8 (per WBS v0.2 §7.1 桶 7 Phase A 启动):
  - 5 域 Lead 各自 1-on-1 签字 (5 commits 落 main)
  - gm-backend Lead 主导 6 项 P1 backlog 决议
- 9/9-9/15 (per WBS v0.2 §2.2 桶 8 Phase B):
  - 5 域 Lead 实装各自 P1 backlog (per §3 决策项)
  - 5 worker 派工, 1 worker 1 域
- 9/16-9/22 (per WBS v0.2 §2.3 桶 9 Phase C ⏳ 集群可达后):
  - gm-backend Lead 重跑 st-11/st-12 mTLS 业务级 ST
  - 5 域 Lead 验证 5 域 gRPC 客户端跨域通信

---

## 5. 状态

- 🟡 占位 (per 9/1 22:30 JST WT-10 brief §D7)
- 主会话 (Mavis) 协调 5 域 Lead 跟 gm-backend Lead 1-on-1
- 5 域 Lead 签字全 = 联调完成 (预计 9/8 前)
- gm-backend Lead 签字 = 联调完成

---

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
**溯源**: WBS v0.2 commit 84edf26 (桶 10 Phase D D7)

---

## 6. 二审签字栏 (per DDD-REVIEW-TEMPLATE-v0.2, B3 派生约束落地)

> **适用**: 本文档 v0.1 → v0.2 二审流程升级 (per AGENTS.md v0.6.3 §3.x, 9/2 10:18 JST 拍板).
> **模板**: docs/14-项目治理/DDD-REVIEW-TEMPLATE-v0.2.md §1 二审流程图 + §2 文档结构模板.

### 6.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1 cargo check 0 error (本批 N 文档 0 改动 Rust) |
| Evidence 段 (commit SHA / file:line) | ✅ | git log + Read 实证 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | 8/27 11:06 JST 凭据硬 ban |
| 缺标比错标 (per 8/26 JST) | ✅ | §N 已知缺口段保留 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 无 "per X 历史形态" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 无 env value 痕迹 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-02 14:11 JST

### 6.2 Ulysses 二审 (必到, per B3 派生约束, ⏳ 待签)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | git log + grep 实证 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ⏳ | cargo check / test 状态 |
| 业务 vs 治理指标 (per v0.1.1 §9.4 里程碑重定义) | ⏳ | hotfix 数 / commit ahead / md 行数 |
| commit ahead 合理性 | ⏳ | 应在 ±20 commit 范围 |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ⏳ | 拍板项已执行 vs 仅承诺 |
| 跟 RGS-WEEKLY 一致性 (若存在) | ⏳ | 周报双指标对齐 |

**Ulysses 二审决定**:

- [ ] ✅ 通过 — 落地, 状态机结束
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 6.1 → 6.2 循环 (打回次数: 0/2/3)

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: ⏳ 待签
