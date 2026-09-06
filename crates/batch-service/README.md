# batch-service (Phase 0 scaffold)

> **创建日期**: 2026-09-05 12:30 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**: W25 任务 brief + RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 + AGENTS.md §7 + W11 42df673 RACI v1.1 (batch 域独立 Lead 不兼任 5 域) + 改进路线图 §1 Phase 0

## 0. 一句话当前状态

batch 域是 **6 域扩展** (per 9/1 18:00 JST 拍板) 中的**第 6 域**, **不与 5 域 Lead 兼任** (per 8/21 JST 5 域独立 Lead 基线). 本 crate 是 **Phase 0 占位**, 业务实装留 Phase 1+ 续.

## 1. 域边界

| 域 | Lead | 状态 |
|---|---|---|
| player / economy / match / social / admin | Ulysses 真实身份 (per 8/21 JST) | 🟢 5 域 |
| **batch (6 域)** | **Mavis 接手代签 (per WBS v0.2 §4.3 拍板 3 + RACI v1.1)** | **🟡 Phase 0 scaffold** |

## 2. DB 三分类 (per 9/1 18:30 JST 横展)

| Schema | 类型 | 说明 |
|---|---|---|
| `batch_master` | Master | 任务模板, slowly changing, SCD 策略 |
| `batch_transaction` | Transaction | 任务实例 + 流水, append-only |
| `batch_work` | Work | 流程中临时, session-bound, 完成后清理 |
| `batch_transaction_archive` | Transaction (归档) | 长期保留 (per NFR-29 audit_event T-3 永久) |

## 3. 13 proto + 3 子域 (Phase 1+ 实装 scope)

3 子域: batch_master / batch_transaction / batch_work
13 proto: BATCH 域 RPC (DAG 触发 + 任务模板 + audit + DLQ + retry 等)
完整实装: `tools/rgs-batch-backend` (per AGENTS §7.1), 本 crate 仅 workspace 注册占位

## 4. 派生约束 (per AGENTS §7.2 12 条)

1. **5 域独立 Lead → 6 域扩展**: batch 域不与 5 域 Lead 兼任
2. **DB 三分类横展**: Master / Transaction / Work 三分清晰
3. **env value 硬 ban**: 凭据走 env, 永不打印 (per 8/27 11:06 JST)
4. **mTLS 业务级**: 5 域 gRPC 调用走 mTLS (per 5 域 ST 业务级 mTLS 实践)
5. **envoy 独立 deployment**: 不选 nginx, 不选 istio sidecar (per 9/1 13:03/13:05 JST)
6. **127.0.0.1 only**: rgs-batch-console 监听 127.0.0.1:8789
7. **0 依赖 Node**: 沿用 rgs-web 母规范
8. **1 写者约束**: rgs-batch-backend 单进程 + tokio multi-thread
9. **rgs-testkit 禁 InMemory**: 用 NoOp + 真实 sqlx + 5 域 gRPC client mock
10. **audit 永久保留**: audit_event T-3 永久
11. **代签规则**: Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化)
12. **saga 集成**: v0.1 不集成, saga-runtime 独立 Pod (per RGS-BAS-100 v0.1)

## 5. 已知缺口 (per 缺标比错标 §1.1)

- [ ] 13 proto 业务实装 (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 §3)
- [ ] 3 子域 service 实现 (batch_master / batch_transaction / batch_work)
- [ ] DB schema migration (4 schema: batch_master / batch_transaction / batch_work / batch_transaction_archive)
- [ ] mTLS 业务级 wire-up (per §7.2 #4)
- [ ] audit_event T-3 永久表 + DLQ + retry (per §7.2 #10)
- [ ] rgs-testkit NoOp mock (per §7.2 #9)
- [ ] k3s 部署 yaml (per BATCH REQ §10.3 — k3s 资源上限 + namespace 隔离策略)
- [ ] batch 域 Lead 真实指派 (per RACI v1.1 §1 "待指派 per E2")

## 6. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 备注 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 12:30 JST | Ulysses — Mavis 接手 (per DEC-008) | 架构师(Mavis 接手 agent per DEC-008)+自审+2026-09-05 | Phase 0 scaffold 占位, W25 任务 brief 落 main workspace |

author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008)+自审+2026-09-05 / 修订人=Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)
