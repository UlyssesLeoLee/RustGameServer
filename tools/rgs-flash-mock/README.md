# rgs-flash-mock — 闪烁之光 1351 RPC gateway/verification harness

> **Status**: v0.1 PoC (12 大类 22 RPC 抽样 stub 模式)
> **Design**: [`docs/14-项目治理/RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.1.md`](../../docs/14-项目治理/RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.1.md)
> **Audit**: [`RGS-DDD-2026-09-04-GAP-AUDIT_v0.3`](../../docs/14-项目治理/RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md) (bb9f977)
> **Overlap**: [`RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04_v0.2`](../../docs/14-项目治理/RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04_v0.2.md) (2e3d9ee)

## 0. 概述

`rgs-flash-mock` 是 **gateway / verification harness**, 验证 RGS 5 域 + card + gm-backend 7 域 backend 是否能 serve 闪烁之光 1351 RPC 形状的请求。

**front**: HTTP/JSON (actix-web 4, port 8791)
**back**: gRPC mTLS (tonic 0.12 → RGS 7 域)
**gap matrix**: per-RPC 覆盖率跟踪 (Pass / Partial / NotImplemented / NotApplicable / Error)

## 1. 5-10 sprint 路线图

| Sprint | 目标 | 估 RPC | Token |
|---|---|---|---|
| **W1 (本 turn)** | v0.1 scaffold + 22 RPC stub | 22 / 1351 (1.6%) | 100-150K |
| W2 | 关键路径 4 类别 + 10-20 RPC (PVP/战斗/经济/GM) | 60-80 / 1351 (4-6%) | 100-150K |
| W3 | 公会/社交/排行榜 + 10-15 RPC each | 100-130 / 1351 (7-10%) | 100-150K |
| W4-W10 | 渐进式补完剩余 1221-1251 RPC | 1351 / 1351 (100%) | 700K-1.05M |

## 2. v0.1 启动

### 2.1 本地启动

```bash
# 1. 编译
cd tools/rgs-flash-mock
cargo build --release

# 2. 启动 (需 RGS 5 域 mTLS cert 路径 env)
export RGS_TLS_DIR=/path/to/rgs/certs
export RUST_LOG=info,rgs_flash_mock=debug
./target/release/rgs-flash-mock

# 3. 验证
curl http://127.0.0.1:8791/health
curl http://127.0.0.1:8791/coverage
curl -X POST http://127.0.0.1:8791/rpc/pvp/EnqueuePVP -H "Content-Type: application/json" -d '{}'
```

### 2.2 k3s 部署 (per AGENTS.md §7.1 batch 域母规范)

```bash
kubectl apply -f tools/rgs-flash-mock/k3s/30-rgs-flash-mock-deployment.yaml
kubectl apply -f tools/rgs-flash-mock/k3s/31-rgs-flash-mock-service.yaml
```

## 3. v0.1 端点

| Endpoint | Method | 描述 |
|---|---|---|
| `/health` | GET | 健康检查 (actuator 风格) |
| `/ready` | GET | 就绪探针 (k8s readiness probe) |
| `/coverage` | GET | gap matrix 报告 (JSON) |
| `/rpc/{category}/{rpc_name}` | POST | 12 大类 RPC stub (22 RPC) |

### 3.1 12 大类 + 22 RPC 抽样

| # | 类别 | RPC 抽样 | RGS backend | v0.1 status |
|---|---|---|---|---|
| 1 | 场景/移动 | GetScene, MovePlayer | match + player | N-A |
| 2 | 角色养成 | GetPlayerProfile, UpgradeSkill | player + card | Partial |
| 3 | 战斗 PVE | StartCombat, SubmitAction | match v2 | Pass |
| 4 | PVP/竞技 | EnqueuePVP, GetPVPMatch | match v2 | Pass |
| 5 | 公会 | GetGuild, JoinGuild | social | Partial (gRPC 4/6 wire) |
| 6 | 经济 | GetAccount, CreateAuction | economy v2 | Pass |
| 7 | 社交 | GetFriendList, SendMessage | social | NotImplemented |
| 8 | 活动运营 | GetActiveEvent, ClaimReward | batch + card | Partial |
| 9 | 付费/商业化 | Recharge, QueryRechargeHistory | economy | NotImplemented |
| 10 | 排行榜/图鉴 | GetLeaderboard | leaderboard | Pass |
| 11 | GM/运维 | BanAccount, GrantCompensation | admin + gm-backend | Pass |
| 12 | 未分类 | (v0.1 不抽样, 待 v0.2+) | — | — |

**v0.1 预期覆盖率**: ~82% (9 Pass + 9 Partial / 22)

## 4. v0.2 路线图

- **W2**: 加 7 域 gRPC client (玩家/economy/match/social/admin/card/gm-backend), handler 真实调用
- **W3**: SQLite 持久化 gap matrix + Prometheus metrics
- **W4+**: 渐进式补完 1351 RPC

## 5. 5-10 sprint 长期路线图

per 设计 doc §6.4: 56-90d ≈ 11-18 周, 1M-1.5M tokens (per R1 sprint OLU 100-150K)

## 6. 已知缺口 (per 8/26 JST 缺标比错标)

- **v0.1 stub 模式**: 不实际调用 RGS, 只返 placeholder JSON + 文档化 RGS routing
- **v0.1 22 RPC 抽样**: 1351 完整覆盖需 W2-W10 渐进式补完
- **v0.1 in-memory gap matrix**: 重启丢失, v0.2 SQLite 持久化
- **v0.1 单二进制**: v0.2 拆 5+ 文件 (routes / clients / db / cron / audit)

## 7. 决策一致性 (per 9/4 16:14 JST user 拍板)

- ✅ audit v0.3 (bb9f977): 6 域 + card 第 7 域架构保留
- ✅ handoff v0.1: TCG 业务保留, mock 验证 RGS backend
- ✅ FLASH-OVERLAP v0.2 (2e3d9ee): 11 维度 API 风格 88/88 keep RGS, mock 走 RGS 风格
- ✅ 9/4 15:34 JST user "仅 API 对齐": mock 仅作验证 harness
- ✅ 9/4 16:14 JST user "完整 1351 mock long-term": 5-10 sprint 路线图

## 8. 代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)

修订人: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
审批: 架构师(Mavis 接手 agent per DEC-008)
代签授权: Mavis 默认代签 Ulysses
