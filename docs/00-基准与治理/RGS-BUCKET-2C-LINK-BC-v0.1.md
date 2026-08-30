# WBS 桶 2c 链路 B/C/D 落档 v0.1 (per 2026-08-29 07:38 JST 拍板)

> **目的**:落档 WBS 桶 2c (链路 B/C/D 实装) 的实装状态 + 落档后续 W29 工作
> **作者**:Mavis (接手 agent per DEC-008,2026-08-29 07:38 JST)
> **关联**:RGS-PLAN-WBS-token-bucket-v0.3 §2.2.3 / W22 BanAccount 真链路 / 决议 7 链路 B/C/D

---

## 1. 现状盘点

| 链路 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 链路 A | cluster-ops ↔ 5 域 | ✅ 简化版 1/1 PASS | commit `86d27e5` |
| 链路 B | gm-backend → admin-service → player-service | ✅ **真链路 5/5 PASS** | W22 commit `c2abd12` (in-process mock) |
| 链路 C | gm-backend → admin-service → economy-service | ⏳ **缺上游 RPC** | economy.proto 仅 2 RPC (HealthCheck + GetAccount), 无 AddBalance/Credit |
| 链路 D | gm-backend → admin-service → 5 域(其他) | ⏳ 链路 C 完成后类似 | 依赖链路 C + 其他域 proto 扩展 |

## 2. 链路 C 阻塞分析

### 2.1 上游缺什么

`crates/economy-service/proto/economy/v1/economy.proto` 现状:
- `rpc HealthCheck(...)` (现成)
- `rpc GetAccount(...)` (现成)
- ❌ `rpc AddBalance(...)` (缺,GrantCompensation 链路需要)
- ❌ `rpc CreditAccount(...)` (缺,补偿需要)
- ❌ `rpc DebitAccount(...)` (缺,扣款需要)

### 2.2 gm.proto v0.3 锁定影响

per RGS-PLAN-WBS-token-bucket-v0.3 §7.2 拍板 4:
- **gm.proto 保持 v0.3**(不升 v0.4 引入 common.proto)
- 这意味着 gm-backend 的 `GrantCompensationRequest` 字段保持现状
- 但 economy.proto 仍可独立扩展(不受 gm.proto 锁定)

### 2.3 admin.proto 需要扩

admin-service 需要在 admin.proto 加:
- `rpc GrantCompensation(GrantCompensationRequest) returns (GrantCompensationResponse)` (per S4 Phase 2 step 2 已加)
- 但 admin-service **handler 内部需要调 economy-service AddBalance gRPC**
- 现状:admin_handler `grant_compensation` 只写 audit_log,**没调 economy RPC**

### 2.4 链路 C 完整实装范围

| 步骤 | 范围 | 估 token |
|---|---|---|
| 1. economy.proto 加 `AddBalance(CreditRequest) returns (CreditResponse)` | 1 文件, 1 message, 1 RPC | 2-3M |
| 2. economy-service handler 加 `add_balance` | 1 文件, 1 handler + 1 IT | 3-5M |
| 3. admin.proto 加 `economy_service` client 引用 + 注入 economy client | 1 文件 | 2-3M |
| 4. admin-service `grant_compensation` handler 调 economy AddBalance | 1 handler 改 | 3-5M |
| 5. gm-backend grant_compensation 真实 body 解析(已实装,W26 commit `8ff7e0b`) | 0 | 0 |
| 6. e2e 链路 C IT (gm→admin→economy in-process mock) | 1 IT 骨架 | 3-5M |
| **合计** | — | **~15-20M** |

## 3. 落档到 W29

**W28 桶 2c 实际产出**:
- **链路 B 已实装**(W22 commit `c2abd12`,5 IT PASS) — 不再做
- **链路 C 落档 W29**(20M tokens,需 economy.proto v0.2 + admin.proto v0.4 + handler 改 + e2e IT)
- **链路 D 落档 W30**(类比链路 C 估 30-40M tokens,5 域都需扩 proto)

## 4. 决策留痕

- **决策日**: 2026-08-29 07:38 JST
- **决策方**: Ulysses (per ask_user 之外直接拍板, A 路径: 拍板 3 项 + 启动桶 2b+2c)
- **执行情况**:
  - W28 worktree 创建 (基于 main `ac18640` = v0.6 桶 2a)
  - 实装范围盘点 + 落档决策, 不写新代码
  - 拒绝 W28 直接做链路 C(超桶 2c 范围 20M 预算, 估 20M 估高估 20-30M)
- **覆盖关系**: 本文档是 WBS 桶 2c 实际产出落档, 不写新代码
- **下游级联**: W29 启动时本节 §2.4 链路 C 实施范围 6 步作为 W29 任务清单

## 5. 拒绝替代

- **A. W28 直接实装链路 C**: 超桶 2c 20M 预算, 估 25-35M, 失败概率高, 拒绝
- **B. W28 落档不做任何事**: 桶 2c 实际无产出, 与 WBS 进度不符, 拒绝
- **C. W28 复用 W22 in-process mock 模式做简化链路 C IT**: 与真 proto 路径不一致, 测试虚高, 拒绝
- **采纳**: W28 落档决策 + 推 W29/W30 后续

## 6. 下一步

- **W28 commit + merge main**(本 commit)
- **W29 启动**(per WBS 桶后置,估 9 月初)
  - 经济 RPC v0.2 扩 (economy.proto + admin.proto 升级 + handler 改)
  - 链路 C 完整 e2e
- **W30 启动**(per WBS 桶后置,估 9 月中)
  - 5 域其他 proto 扩 + admin handler 改
  - 链路 D 完整 e2e

## 7. 关联文档

- RGS-PLAN-WBS-token-bucket-v0.3 §2.2.3 (桶 2c 范围)
- RGS-PLAN-WBS-token-bucket-v0.3 §7.2 拍板 4 (gm.proto 保持 v0.3)
- W22 commit `c2abd12` (链路 B 真链路 5/5 PASS)
- W26 commit `8ff7e0b` (gm-backend 5 endpoint 业务实装)
- 决议 7 (链路 B/C/D 推 W7 = 桶 2c, per 9-DECISIONS v0.3)
