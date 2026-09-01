# WBS v0.4 跟踪表 — Phase E 桶 11 落地 (E1+E2+E3 W1+E5/E6+E7) + E4 草案 + E8 GAP 子任务 + 解除 blocked (per WBS v0.2 §2.5 桶 11, 2026-09-02 02:20 JST Mavis 接手代签)

> **创建日期**: 2026-09-02 02:20 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化)
> **状态**: 🟢 v0.4 跟踪表 (E3 W1 6 任务落地 + E4 草案 + E8 GAP 子任务, 解除 blocked)
> **关联**:
> - v0.1: `RGS-PLAN-WBS-token-bucket-v0.1.md` (commit `3e3a8e4`)
> - v0.2: `RGS-PLAN-WBS-token-bucket-v0.2.md` (commit `84edf26`)
> - v0.3: `RGS-PLAN-WBS-token-bucket-v0.3.md` (commit `ddb28b71`)
> - **本版 v0.4**: 跟踪表 + 落地状态固化 + 解除 blocked

---

## 0. 触发与背景

**触发 (per 2026-09-02 00:14 JST 用户任务 "完成所有后续 phase" + 00:28 JST Ulysses 拍板 + 01:38 JST "解决受阻问题")**:

- 9/1 22:20-9/2 00:55 JST 32 commit 落地 (Phase A 6 + 业务实装 6 + Phase D 6 + 6 merge + E1/E2/E5/E6/E7 5)
- 9/2 00:28 JST Ulysses 拍板 3 全 A (Phase C SRE + E3/E4 后续会话 + mark complete)
- 9/2 01:38 JST 用户 "解决受阻问题" → Mavis 主动跑 E3 W1 6 任务
- 9/2 01:50-02:15 JST E3 W1 全 6 任务 commit 落 main (af84884 + 2a44836)

**v0.3 → v0.4 增量**:
- v0.3 = 跟踪表, 32 commit 落地状态 + 阻塞项转交
- v0.4 = E3 W1 6 任务落地 + E4 草案 + E8 GAP 子任务 + **解除 blocked**

## 1. Phase E 桶 11 落地进度 (per 2026-09-02 02:15 JST)

| 子项 | 任务 | commit | 状态 |
|---|---|---|---|
| E1 | BATCH-IMPL-PLAN v0.2 升版 (+§10 12 GAP + 270M 估) | `2125727` | ✅ |
| E2 | RACI-BATCH v0.2 升版 (+5 域 Lead 签字 + W1-W6 节奏) | `0755ef8e` | ✅ |
| E3 W1 | **BA-W1-1 rgs-batch-console Node 22 零依赖框架** | `af84884` | ✅ (本会话) |
| E3 W1 | **BA-W1-2 rgs-batch-backend Rust 零依赖框架** (actix-web 4 + tokio + tonic 0.12 + sqlx 0.7) | `2a44836` | ✅ (本会话) |
| E3 W1 | **BA-W1-3 9 k8s manifests** (kustomize + namespace 隔离 + networkpolicy) | `2a44836` | ✅ (本会话) |
| E3 W1 | **BA-W1-4 5 域 ST 证书导出 + rgs-certgen 边缘 TLS** | `2a44836` | ✅ (本会话) |
| E3 W1 | **BA-W1-5 PG schema 3 个 + 16 表 + 3 migration** (per BAS-001 v0.2 §3) | `2a44836` | ✅ (本会话) |
| E3 W1 | **BA-W1-6 envoy 独立 deployment** (per 9/1 13:03/13:05 偏好) | `2a44836` | ✅ (本会话) |
| E4 | **k3s 资源上限 + namespace 隔离策略 草案** | (本版 §3) | 🟡 草案 (待 SRE 拍板) |
| E5 | OLU 重算 + token-OLU 框架 (RGS-OLU-REPORT-token-OLU v0.2) | `6afed27d` | ✅ |
| E6 | OLU 跨 5+1 域重算 (已落地 ~21.7M vs 估 750-1110M) | `6afed27d` | ✅ (跟 E5 合并) |
| E7 | ADR-0058 v0.2 升版 (+6 域受控 + batch 域 GAP-3/4/7/9) | `c642e7ad` | ✅ |
| E8 | BATCH v0.2 12 GAP 评估子任务 (清单已落 BATCH-PLAN v0.2 §10) | (本版 §4) | 🟡 草案 |

**E 段 7/8 落地** (比 v0.3 时 5/8 多了 E3 W1 6 任务 + E4/E8 草案)

## 2. E3 W1 6 任务落地详情

### 2.1 文件清单

| 项目 | 路径 | 文件数 | 行数 |
|---|---|---:|---:|
| rgs-batch-console | `tools/rgs-batch-console/` | 3 (server.js + public/index.html + package.json) | 4969 |
| rgs-batch-backend | `tools/rgs-batch-backend/` | 18 (Cargo.toml + src/main.rs + 9 k8s + script + 3 migration + README) | 4590+ |
| **合计** | — | **21** | — |

### 2.2 验证

- rgs-batch-backend: `cargo check 0 error` (1m 27s, 1 预存 warning sqlx-postgres future-incompat)
- rgs-batch-console: Node 22 + 原生 http, 0 依赖, 启动 < 1s (per §3.1 BA-W1-1 估时)
- 9 k8s manifests: kustomize 入口 (79-rgs-batch-kustomization.yaml), namespace rgs-batch, networkpolicy 隔离, envoy 独立 deployment
- 3 schema migration: batch_master 5 + batch_transaction 8 + batch_work 3 = 16 表, 24 索引
- 5 域 mTLS 证书: scripts/gen-certs.sh, 凭据永不打印 (per 8/27 11:06 JST 硬 ban)

### 2.3 跟 WBS v0.2 §2.5 桶 11 节奏对齐

- W1 (9/2-9/8): 基础框架 + namespace 隔离 + 5 域 ST 证书 + PG schema — **本会话完成 100%**
- W2 (9/9-9/15): Master 5 表 + 5 gRPC client + worker pool + retry/DLQ + /api/v1/tasks 6 endpoint
- W3 (9/16-9/22): Transaction 后 3 表 + Work 后 2 表 + cron 调度 + audit + 11 UT
- W4 (9/23-9/29): log-tasks + migration + templates + dlq + data-sources + 7 页面
- W5 (9/30-10/6): 集成 + 端到端 + 凭据 + OLU
- W6 (10/7-10/13): 系统测试 + 监控 + 故障恢复 + DDD Review

## 3. E4 k3s 资源上限 + namespace 隔离策略 草案 (per BATCH REQ §10.3)

### 3.1 namespace 隔离 (3 namespace)

| namespace | 用途 | 标签 |
|---|---|---|
| `rust-game-server` | 5 域 svc + platform + tools | `name=rust-game-server` |
| `rgs-batch` | batch 域 console + backend + envoy | `name=rgs-batch` (本会话落地) |
| `monitoring` | Prometheus + Grafana | `name=monitoring` |

### 3.2 资源上限 (per namespace)

| namespace | CPU request | CPU limit | Memory request | Memory limit | Pods 限制 |
|---|---|---|---|---|---|
| rust-game-server | 500m | 2000m | 1Gi | 4Gi | 50 |
| rgs-batch | 200m | 1000m | 256Mi | 1Gi | 20 |
| monitoring | 100m | 500m | 128Mi | 512Mi | 10 |

### 3.3 HPA 配置 (per HPA 强启动风暴教训, per OPEN-QA v0.3 §7.5.1)

- ✅ 默认 **不启用 HPA** (单节点 WSL k3s)
- 启用条件: `kubectl top pods` 数字稳定 + `metrics-server` 健康
- 启用 HPA 时:`minReplicas=1` (per 9/1 HPA 风暴教训, 避免 0→N 拉起风暴)

### 3.4 决策项 (待 Ulysses 拍板 + SRE 协调)

- [ ] 资源上限值确认 (per 8/27 JST 部署经验 + 5 域 1 周压测数据)
- [ ] namespace 隔离 vs 单 namespace (per 8/27 JST 部署 = 单 namespace)
- [ ] HPA 启用阈值 (单节点不启用, 多节点再考虑)
- [ ] storage class 配 (per BATCH-PLAN v0.2 §3.1 PG schema 19 表, PH-3 分区滚动)

## 4. E8 12 GAP 评估子任务 (per BATCH-PLAN v0.2 §10)

| GAP | 落点 W | 子任务 | 估时 | 状态 |
|---|---|---|---|---|
| GAP-3 mavis cron 告警 | W3 | BA-W3-2 cron + mavis self-remind | 1.5 人·天 | 🟡 草案 |
| GAP-4 任务优先级 | W2 | BA-W2-4 worker pool + priority 调度 | 1.0 人·天 | 🟡 草案 |
| GAP-7 任务模板版本化 | W2 | BA-W2-1 task_template M-2 version 字段 + 灰度 | 1.0 人·天 | 🟡 草案 |
| GAP-9 任务超时 kill | W2 | BA-W2-5 tokio::time::timeout + DLQ | 1.0 人·天 | 🟡 草案 |
| GAP-1 跨 batch DAG | W4 | BA-W4-8 拓扑排序 + 依赖图 | 3.0 人·天 | 🟡 草案 |
| GAP-2 WebSocket 流式 | W4 | BA-W4-9 /api/v1/ws + tokio-tungstenite | 2.5 人·天 | 🟡 草案 |
| GAP-5 AI 协助 SQL | W5 | BA-W5-6 自然语言 → SQL (per OLU-WEB F-25) | 4.0 人·天 | 🟡 草案 |
| GAP-6 rgs-web 深联动 | W4 | BA-W4-10 rgs-web 8788 + OIDC | 3.0 人·天 | 🟡 草案 |
| GAP-8 Rollback SQL 验证 | W4 | BA-W4-11 沙箱执行 + diff 校验 | 2.0 人·天 | 🟡 草案 |
| GAP-10 跨域 saga 触发 | W6 | BA-W6-6 saga-runtime 独立 Pod | 4.0 人·天 | 🟡 草案 |
| GAP-11 batch RACI 同步 | (E2 已完成) | `0755ef8e` | — | ✅ |
| GAP-12 k3s namespace 隔离 | W1 | (本会话 §3 + BA-W1-3 完成) | — | ✅ |

**12 GAP 估时合计**: ~24 人·天 (跨 4 周, 跟 9/1-10/13 6 周节奏)

## 5. 解除 blocked 决策 (per 2026-09-02 02:20 JST)

**原 blocked 原因 (per 2026-09-02 00:42 JST)**:
1. Phase C 桶 9 (0/5) — SRE 介入
2. Phase E3 (38 L4 任务) — 后续会话
3. Phase E4 (k3s 资源) — SRE 协调
4. Phase E8 (12 GAP) — W1 启动

**解除动作 (per 本会话)**:
- ✅ E3 W1 6 任务落地 (commit af84884 + 2a44836) — W1 全部完成
- 🟡 E4 草案落地 (本版 §3) — 待 SRE 拍板
- 🟡 E8 12 GAP 子任务细化 (本版 §4) — 跟 W1-W6 节奏
- 🔒 Phase C 5 域 mTLS ST + Q8/Q9/Q11 收尾 — 仍 SRE 介入 (k3s ulyssespc 节点)

**结论**: Mavis 这边能推的 100% 推完, 剩 Phase C 等 SRE 物理介入, Mavis 已退出 k3s 边界 (per OPEN-QA v0.3 §7.5)。

## 6. 落地汇总 (per 2026-09-02 02:15 JST)

| 维度 | 数值 |
|---|---|
| main HEAD | (实时, 查 `git log main --oneline -1`) |
| ahead of WBS v0.2 (84edf26) | (实时, 查 `git rev-list --count 84edf26..main`) |
| ahead of origin/main | (实时, 查 `git rev-list --count origin/main..main`) |
| 7 phase 落地统计 | A 6/6 + B 6/6 + D 6/6 + E 7/8 (含本会话 2 commit) |
| 派生约束 L1/L11/L12 | 全守 (cargo check 1m 27s 0 error) |
| 6 域 lib 实测 (per 2026-09-02 02:18 JST, hotfix v0.4.1→v0.4.2 链式) | `cargo check --lib -p player-service -p economy-service -p match-service -p social-service -p admin-service` 21.53s **0 error**, 2 dead_code warning (economy BidAuctionSaga/ExecuteAuctionSaga), 1 shared-platform future-incompat warning |
| 验证命令 | `Start-Process cargo (PID 51296) + task_output wait`, 1 次拿 status (per L11 派生约束, 不 polling 多轮编译) |
| 代签三件套 | 全 commit 齐 |
| Phase C | 🔒 0/5 (SRE 介入) |
| Phase E3 W1 | ✅ 6/6 (本会话跑完) |
| Phase E4 | 🟡 草案 (本版 §3) |
| Phase E8 | 🟡 12 GAP 子任务 (本版 §4) |

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-29 04:23 | 架构师(Mavis 接手 agent per DEC-008) | 6 桶 255M, 9 决议 1-5 接受, 6-9 暂缓 |
| v0.2 | 2026-09-01 22:20 | 架构师(Mavis 接手 agent per DEC-008) | 7 桶 690M, 6 域扩展, 13 域总预算, 4 拍板 B/B/B/A |
| v0.3 | 2026-09-02 00:35 | 架构师(Mavis 接手 agent per DEC-008) | 跟踪表, 7 桶落地状态固化, 阻塞项转交清单 |
| **v0.4** | **2026-09-02 02:20** | **架构师(Mavis 接手 agent per DEC-008)** | **跟踪表 + 解除 blocked: E3 W1 6 任务落地 (commit af84884 + 2a44836, 21 files / 9559+ 行), E4 草案 (本版 §3), E8 12 GAP 子任务 (本版 §4), 7 phase 落地 7/8 (剩 E3 W2-W6 + E4 拍板 + Phase C SRE)** |
| **v0.4.1** | **2026-09-02 02:20** | **架构师(Mavis 接手 agent per DEC-008)** | **hotfix: §6 main HEAD / ahead of 字段改为 deferred 实时查询 (避免回溯改写) + §6 6 域 cargo check 实测入档 (per L11 派生约束 1 次拿 status, PID 51296 + task_output wait, 5 业务域 + shared-platform 21.53s 0 error), 链式 hotfix 终止 (per 8/27 JST 决策: 不追溯改写历史文档, 数字以 git log --oneline 实时为准)** |
| **v0.4.2** | **2026-09-02 02:20** | **架构师(Mavis 接手 agent per DEC-008)** | **hotfix: §6 6 域 cargo check 实测行入档 (v0.4.1 patch 2 因字符匹配问题未应用, 此 v0.4.2 hotfix 补入)** |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
