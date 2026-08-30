# WBS 桶 2b 5 域 axum-test 切 落档 v0.1 (per 2026-08-29 07:38 JST 拍板)

> **目的**:落档 WBS 桶 2b (5 域切 axum-test) 的实装状态 + 落档后续 W31 工作
> **作者**:Mavis (接手 agent per DEC-008,2026-08-29 07:38 JST)
> **关联**:RGS-PLAN-WBS-token-bucket-v0.3 §2.2.2 / 决议 6 / 7 域 wiremock + 8 域 axum-test 双工具并存

---

## 1. 现状盘点

### 1.1 5 域架构类型

| 域 | 协议 | HTTP 入口 | axum Router | 集成测试 |
|---|---|---|---|---|
| player-service | gRPC (tonic) | ❌ 无 | ❌ 无 | 0 IT |
| economy-service | gRPC (tonic) | ❌ 无 | ❌ 无 | 0 IT |
| match-service | gRPC (tonic) | ❌ 无 | ❌ 无 | 0 IT |
| social-service | gRPC (tonic) | ❌ 无 | ❌ 无 | 0 IT |
| admin-service | gRPC (tonic) | ❌ 无 | ❌ 无 | 0 IT (per W26 重构前) |
| cluster-ops | gRPC (tonic) | ❌ 无 | ❌ 无 | 1 IT (it_cross_domain_admin_health) |

**关键发现**:5 域 + cluster-ops + admin-service 全是 **gRPC (tonic)** 服务,没有 axum HTTP 入口。

### 1.2 决议 6 原描述

RGS-PLAN-WBS-token-bucket-v0.3 §2.2.2:
> 5 域(player/economy/match/social/cluster-ops)axum-test 切 + 业务 IT 骨架

### 1.3 决议 6 与现状的 gap

| 决议 6 假设 | 现状 | gap |
|---|---|---|
| 5 域有 HTTP 入口可测 | 5 域**无 HTTP 入口**,纯 gRPC | ❌ 不适用 axum-test |
| axum-test 是 HTTP 集成测试工具 | 5 域需要的是 gRPC 集成测试工具 (tonic-test) | ❌ 工具不匹配 |
| 5 域已有 IT 需切工具 | 5 域**0 IT** | ❌ 没有"切"的对象 |

**结论**:决议 6 描述"5 域切 axum-test"实际上是**"5 域加 axum HTTP 入口 + axum-test IT"** 或 **"5 域用 tonic-test 替代 axum-test"**。两者都是 5 域架构变更,不是工具切换。

## 2. 落档决策

### 2.1 W27 桶 2b 实际产出

- **决议 6 不做**(超桶 2b 20M 预算, 5 域 × IT 估 30-50M tokens)
- **落档后续 W31+**(5 域 IT 完整实装, 估 50-80M tokens, 需要 5 域架构升级 + 业务实装同步)

### 2.2 拒绝替代

- **A. W27 5 域 × 1 骨架 axum-test IT (5 IT)**: 5 域无 axum Router, "骨架 IT" = 5 个空 IT 文件, 无价值
- **B. W27 5 域 × 1 骨架 tonic-test IT (5 IT)**: 决议 6 明确"axum-test", 与决议不一致
- **C. W27 5 域加 axum HTTP 入口 + 1 IT/域 (5 IT)**: 5 域架构升级 (gRPC + HTTP 双协议), 估 30-50M, 超预算
- **D. W27 落档不做任何事 (本文档)**: 拒绝决议 6 与现状 gap, 推 W31+ 后续
- **采纳**: D 落档, 与 W28 链路 C 落档一致

### 2.3 W27 commit 包含

- 本落档文档 (RGS-BUCKET-2B-AXUM-TEST-v0.1.md)
- 不写新代码
- 不改 Cargo.toml (不引入 axum-test 依赖)

## 3. 落档后续 W31 工作范围

### 3.1 5 域 IT 完整实装 (估 50-80M tokens)

| 域 | 范围 | 估 token |
|---|---|---|
| player-service | 加 axum HTTP 入口 (per DTL-018) + 1 IT 业务冒烟 | 10-15M |
| economy-service | 加 axum HTTP 入口 (per DTL-037) + 1 IT 业务冒烟 | 10-15M |
| match-service | 加 axum HTTP 入口 (per DTL-026) + 1 IT 业务冒烟 | 10-15M |
| social-service | 加 axum HTTP 入口 (per DTL-019) + 1 IT 业务冒烟 | 10-15M |
| admin-service | 加 axum HTTP 入口 (per DTL-031) + 1 IT 业务冒烟 | 10-15M |
| **合计** | — | **50-80M** |

### 3.2 前置依赖

- gm.proto v0.3 业务实装已完成 (W26 commit `8ff7e0b`)
- 5 域 gRPC server 已实装 (W15+ W17+ 7 域 wire GM RPC)
- 5 域业务实装需要 (决议 6+7 暂缓项落档到桶 2b, 但需要 9 月 W7 业务实装基础)

### 3.3 决策依赖

W31 启动需 Ulysses 拍板:
- 5 域是否加 axum HTTP 入口(架构变更)
- 5 域 IT 用 axum-test 还是 tonic-test(工具选型)
- W31 token 预算 50-80M 是否批准(per WBS §2.2.2 桶 2b 20M 不够)

## 4. 决策留痕

- **决策日**: 2026-08-29 07:38 JST
- **决策方**: Ulysses (per ask_user 之外直接拍板, A 路径: 拍板 3 项 + 启动桶 2b+2c)
- **执行情况**:
  - W27 worktree 创建 (基于 main `ac18640` = v0.6 桶 2a)
  - 5 域 axum-test 工具切盘点: 5 域 0 IT + 0 axum Router
  - 拒绝 W27 强行做"5 域 × 1 骨架 IT"(无价值)
  - 落档后续 W31+ (50-80M tokens 估)
- **覆盖关系**: 本文档是 WBS 桶 2b 实际产出落档, 不写新代码
- **下游级联**: W31 启动时本节 §3 5 域 IT 完整实装范围 5 域作为 W31 任务清单输入

## 5. 关联文档

- RGS-PLAN-WBS-token-bucket-v0.3 §2.2.2 (桶 2b 范围)
- RGS-OPEN-QA-001 Q-TBD-08-06 (双工具决策: 7 域 wiremock + 8 域 axum-test)
- W26 commit `8ff7e0b` (gm-backend 5 endpoint 业务实装)
- W22 commit `c2abd12` (链路 B gm→admin→player 真链路 5/5 PASS)
- 决议 6 (5 域切 axum-test 推 W7 = 桶 2b, per 9-DECISIONS v0.3)
- TBD-08-06 工具决策 D (双工具并存)
