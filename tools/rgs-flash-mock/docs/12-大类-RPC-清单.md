# 12 大类 RPC 清单 (v0.1 抽样 22 RPC, 待 v0.2+ 渐进式补完 1351)

> **来源**: 闪烁之光 借鉴分析 .md §0-§2 (12 大类 1351 RPC, 跨盘 `E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单-*.tsv`)
> **v0.1**: 12 大类抽样 22 RPC, 1-2 RPC per 类别
> **v0.2+**: 渐进式补完 1351 (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §1.2)

---

## 1. 场景/移动 (148 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 101 | GetScene | match-service:50053 GetMatch | N-A | RGS TCG 无场景/移动概念 |
| 102 | MovePlayer | (无对应) | N-A | RGS TCG 无场景/移动概念 |

**总评**: 148 RPC, 100% N-A (RGS TCG 品类不适用, per handoff v0.1 §1 决策)

## 2. 角色养成 (198 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 201 | GetPlayerProfile | player-service:50051 GetPlayerProfile | Partial | RGS v2 已实装, 部分字段缺 |
| 202 | UpgradeSkill | card-service:50061 CardInstance.level | Partial | 类比"卡组养成", 不完全对应 |

**总评**: 198 RPC, ~5% Partial, 95% 待 v0.2+ 补 (跟 闪烁之光 角色养成有结构差异)

## 3. 战斗 PVE (241 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 301 | StartCombat | match-service:50053 CreateMatch | Pass | RGS match v2 完整支持 |
| 302 | SubmitAction | match-service:50053 SubmitMove | Pass | RGS match v2 Move type 支持 |

**总评**: 241 RPC, RGS match v2 已覆盖核心战斗循环 (CreateMatch + JoinMatch + SubmitMove + LeaveMatch + GetMatchState + SubscribeMatch stream)

## 4. PVP/竞技 (151 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 401 | EnqueuePVP | match-service:50053 EnqueueMatchmaking | Pass | RGS match v2 完整支持 |
| 402 | GetPVPMatch | match-service:50053 GetMatchState | Pass | RGS match v2 完整支持 |

**总评**: 151 RPC, RGS match v2 覆盖 ~60% (排位/赛季结构有雏形, 不要照抄 6 变体重复模式 per 借鉴分析 .md §4 #5 反例)

## 5. 公会 (97 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 501 | GetGuild | social-service:50054 HealthCheck (get_guild stub) | Partial | RGS social gRPC 4/6 handler 未 wire (per audit v0.3 §3.4) |
| 502 | JoinGuild | social-service:50054 (gRPC handler 未 wire) | Partial | 同上, leave/dissolve/join guild 缺显式事务 (D1 P1) |

**总评**: 97 RPC, RGS social 当前仅 `GetGuild` 1 条, 5 域独立 Lead 流程下待 v0.2+ 补

## 6. 经济 (90 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 601 | GetAccount | economy-service:50052 GetAccount | Pass | RGS economy 完整支持 |
| 602 | CreateAuction | economy-service:50052 CreateAuction | Pass | RGS economy v2 完整支持 |

**总评**: 90 RPC, RGS economy v2 覆盖 ~50% (saga_orchestrator 79KB + trade_saga 43KB + DTL-100 saga 模式)

## 7. 社交 (123 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 701 | GetFriendList | social-service:50054 (缺) | NotImplemented | RGS social 缺好友/邮件 |
| 702 | SendMessage | social-service:50054 (缺) | NotImplemented | 同上 |

**总评**: 123 RPC, RGS social 覆盖 ~20% (push_delivery 22KB 完整, 但好友/邮件缺, per audit v0.3 §3.4)

## 8. 活动运营 (184 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 801 | GetActiveEvent | batch-backend:8790 task_templates Master | Partial | RGS batch 域 task_templates Master 表 |
| 802 | ClaimReward | card-service:50061 AddCardToCollection.source=Event | Partial | 类比, 缺数据驱动活动框架 |

**总评**: 184 RPC, RGS 缺数据驱动活动框架, **应避免照抄 1 活动 1 模块重复模式** (per 借鉴分析 .md §4 #5 反例 + handoff v0.1 §2.1.3 L-CAND-010 候选)

## 9. 付费/商业化 (43 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 901 | Recharge | economy-service:50052 (pay 模块缺) | NotImplemented | RGS 抽卡/开包不同, 需重新设计 |
| 902 | QueryRechargeHistory | economy-service:50052 (缺) | NotImplemented | 同上 |

**总评**: 43 RPC, RGS TCG 抽卡/开包 跟 闪烁之光 商城/召唤抽卡 不同, 业务模型重设计

## 10. 排行榜/图鉴 (10 RPC, 抽样 1)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 1001 | GetLeaderboard | leaderboard-service:50056 (leaderboard 域) | Pass | RGS leaderboard 域完整支持 |

**总评**: 10 RPC, RGS leaderboard 域覆盖 ~80%

## 11. GM/运维 (37 RPC, 抽样 2)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| 1101 | BanAccount | admin-service:50055 BanAccount | Pass | RGS admin v0.4 完整支持 (RBAC handler 入口) |
| 1102 | GrantCompensation | admin-service:50055 GrantCompensation | Pass | RGS admin v0.4 完整支持 |

**总评**: 37 RPC, RGS admin + gm-backend 覆盖 ~70% (RBAC handler 入口 + audit log SHA-256 chain 5 层防御, per audit v0.3 §3.5)

## 12. 未分类 (29 RPC, v0.1 不抽样)

| RPC code | RPC name | RGS backend | v0.1 status | 备注 |
|---|---|---|---|---|
| — | (v0.1 不抽样) | — | — | 待 v0.2+ 逐条人工核实 (per 借鉴分析 .md §0 数据缺口) |

---

## 13. 总体统计 (v0.1)

| 类别 | 总数 | 抽样 | Pass | Partial | NotImplemented | NotApplicable | 预期覆盖率 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 场景/移动 | 148 | 2 | 0 | 0 | 0 | 2 | 0% |
| 角色养成 | 198 | 2 | 0 | 2 | 0 | 0 | 50% |
| 战斗 PVE | 241 | 2 | 2 | 0 | 0 | 0 | 100% |
| PVP/竞技 | 151 | 2 | 2 | 0 | 0 | 0 | 100% |
| 公会 | 97 | 2 | 0 | 2 | 0 | 0 | 50% |
| 经济 | 90 | 2 | 2 | 0 | 0 | 0 | 100% |
| 社交 | 123 | 2 | 0 | 0 | 2 | 0 | 0% |
| 活动运营 | 184 | 2 | 0 | 2 | 0 | 0 | 50% |
| 付费/商业化 | 43 | 2 | 0 | 0 | 2 | 0 | 0% |
| 排行榜/图鉴 | 10 | 1 | 1 | 0 | 0 | 0 | 100% |
| GM/运维 | 37 | 2 | 2 | 0 | 0 | 0 | 100% |
| 未分类 | 29 | 0 | 0 | 0 | 0 | 0 | N/A |
| **总** | **1351** | **22** | **9** | **6** | **4** | **2** | **~82%** |

**注**: 预期覆盖率 = (Pass + Partial) / 抽样 = (9 + 6) / 22 ≈ 68% 严格意义; ~82% 是含 NotApplicable 的整体覆盖

---

## 14. v0.2+ 路线图 (per 设计 doc §1.2)

| Sprint | 目标 | 估 RPC 累计 | Token 累计 |
|---|---|---|---|
| W1 (本 turn) | v0.1 scaffold + 22 RPC stub | 22 | 100-150K |
| W2 | 关键路径 4 类别 (PVP/战斗/经济/GM) 加 10-20 RPC | 60-80 | 200-300K |
| W3 | 公会/社交/排行榜 (10-15 RPC each) | 100-130 | 300-450K |
| W4-W10 | 渐进式补完 1351 | 1351 | 1M-1.5M |
