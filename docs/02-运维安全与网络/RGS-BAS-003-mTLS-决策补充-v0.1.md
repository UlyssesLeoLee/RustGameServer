# RGS-BAS-003 mTLS 决策补充 v0.1 (per Ulysses 2026-08-29 05:28 JST 拍板)

> **目的**:明确 gm-backend → admin-service → 5 域 mTLS 范围(原 BAS-003 §2.1 + §4.4 仅写"gm→admin mTLS",5 域范围待定)
> **作者**:Mavis (接手 agent per DEC-008,2026-08-29 05:28 JST)
> **关联**:RGS-PLAN-WBS-token-bucket-v0.1 §7.2 拍板 2 / BAS-003 §2.1 / W21 mTLS 5 IT / W17 JWT propagation
> **覆盖关系**:本补充是 BAS-003 §2.1 的范围澄清,不修改 BAS-003 主体

---

## 1. 决策

| 路径 | mTLS | 备注 |
|---|---|---|
| GM 前端 → gm-backend (HTTP) | ❌ 不上 mTLS | HTTPS + JWT + RBAC(per BAS-003 §2 组件图 + RGS-IMPL-001 安全约定) |
| **gm-backend → admin-service (gRPC)** | ✅ **mTLS 双向认证** | 跨信任域(BAS-022 + ARC-019),强认证;W21 已实装 5 IT |
| admin-service → 5 域 (player/economy/match/social/cluster-ops) (gRPC) | ❌ 不上 mTLS | 同 k3s NetworkPolicy 信任域 + JWT(per BAS-022 §5.3) |
| 5 域内部 (e.g. match → player) (gRPC) | ❌ 不上 mTLS | 同信任域,NetworkPolicy 已隔离,JWT 鉴权足够 |
| cluster-ops → admin-service / gm-backend | ❌ 不上 mTLS | 同信任域 |

## 2. 理由

### 2.1 gm-backend → admin-service 上 mTLS 的理由

- **跨信任域**: gm-backend 接收前端 JWT(admin role),admin-service 在后端服务域,gm 不知道也不应该知道 admin 的 NetworkPolicy 策略细节
- **强认证需求**: GM 操作(封号/补偿/维护)是 high-impact,需要 client cert 双向认证作为 JWT 之外的第二因子
- **已有实装**: W21 (commit `ff62bdd`) 已 5 IT PASS,真实 k3s 证书从 rgs-secret-admin-tls 抽取
- **合规**: 部分监管要求 admin 操作双因素(client cert + JWT)

### 2.2 5 域内部不上 mTLS 的理由

- **同信任域**: 5 域都在 k3s namespace `rust-game-server` 内,NetworkPolicy 已隔离
- **JWT 已鉴权**: per W17 (commit `2acc222`),gRPC metadata 传播 JWT,服务间调用有 RBAC
- **mTLS 成本**: 每域 +1 套证书生命周期管理(签发/轮换/吊销/监控),估 +50% token(per RGS-TS-001 v0.6 §6.2 双算法估算)
- **复杂度**: 5 域 × 双向 mTLS = 10 套证书,与 5 域独立的 RACI(per DDD Review 决议 2)冲突

## 3. 实施范围(WBS 桶 4)

| 路径 | 状态 | 桶 4 工作 |
|---|---|---|
| gm-backend → admin-service | ✅ 已实装 (W21) | 仅需证书轮换策略 + 1 年有效期 + Vault 集成 |
| 5 域内部 (admin → 5 域, 5 域之间) | ❌ 不上 | 0 token 投入(决策记录即可) |

## 4. 决策留痕

- **决策日**: 2026-08-29 05:28 JST
- **决策方**: Ulysses (per ask_user 之外直接拍板, A 路径: 拍板 3 项)
- **落档文档**: RGS-PLAN-WBS-token-bucket-v0.1 §7.2 拍板 2 + 本补充 v0.1
- **覆盖关系**: 本补充是 BAS-003 §2.1 的范围澄清,不修改 BAS-003 主体
- **下游级联**: WBS 桶 4 范围缩小(gm→admin 5 IT 已实装, 5 域内部 0 token 投入)

## 5. 拒绝替代

- **A. 全 9 域 mTLS**(5 域 + cluster-ops + gm + admin + rgs-certgen 全部双向 mTLS): token 估 +50%, 增加 4 套证书生命周期管理, 与 5 域独立 RACI 冲突, 拒绝
- **B. 仅 gm 内部 mTLS**(gm → 自己的依赖): 5 域无依赖, 决策无意义, 拒绝
- **C. 完全不上 mTLS**(全 JWT): gm → admin 是 high-impact 跨域, JWT 单一因子不够, 拒绝

## 6. 关联文档

- BAS-003 §2.1 组件图(L74-89)+ §4.4 NetworkPolicy
- W21 mTLS 5 IT (commit `ff62bdd`)
- W17 JWT propagation gRPC metadata (commit `2acc222`)
- W9 mTLS to admin-service (commit `1333898`,gm-backend client cert via env)
- RGS-PLAN-WBS-token-bucket-v0.1 §7.2 拍板 2

---

> **mTLS 决策补充 v0.1**: gm-backend → admin-service 上 mTLS, 5 域内部不上 mTLS(NetworkPolicy + JWT 已足够)
> **节省 token**: 拒绝全 9 域 mTLS, 节省 ~12M tokens(per 桶 4 token 预算 25M 缩到 13M)
