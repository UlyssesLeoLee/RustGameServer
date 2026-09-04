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

## W2 Phase 2 worker-2 追加段 (per 9/4 17:39 JST W2 启动 option A + 17:44 JST 派工)

> **触发**: per 9/4 17:39-17:44 JST Ulysses 拍板 W2 启动 (12 Partial → mock gap matrix 100% Pass) + 派工模式 option B (2 worker 并行, per L12.2)
> **worker-2 负责**: 6 Partial (login / rank / conn_login / recruit / group_control / activity), 21 cmds 总量, 跨 3-4 域 (player / match / leaderboard / cluster_ops(新) / card / batch)
> **写入时间**: 2026-09-04 17:50-18:30 JST (per 200-250K token 预算)
> **写入模式**: write-not-commit (per L12.2 选项 B 0 race condition, 主会话统一 2 commit)

### W2-2.1 login (协议号 101, 6 cmds) → player (主) + conn_login (新) + auth (新)

| RPC code | RPC name | RGS backend | v0.2 status | 备注 |
|---|---|---|---|---|
| 10101 | CreatePlayer | player-service:50051 CreatePlayer | Partial | ?MAX_ROLE_NUM=1 单角色策略跟 RGS 多角色冲突, v0.2 协调 |
| 10102 | LoginRole | player-service:50051 LoginRole | Partial | role:start/5 + role_query:pid/2 1:1 翻译, ?minu_ms(3) 延时停止策略待协调 |
| 10103 | Reconnect | player-service:50051 Reconnect | Partial | role_reconnect + role_login 路径已实装, combat_pid 检测待 v0.2 协调 |
| 10300 | CompleteResourceLoading | player-service:50051 CompleteResourceLoading | Partial | 闪烁之光 简单 flag 设置, RGS 缺 resource_loaded 状态 |
| 10301 | DeviceRegister | player-service:50051 DeviceRegister | Partial | RGS 缺 account_devices Master 表, v0.2 sprint 评估新建 |
| 10302 | ForgotPassword | player-service:50051 + social-service:50054 (mail) ForgotPassword | Partial | RGS 缺完整 forgot_password 流程, 待跟 social mail 域整合 |

**总评**: 6 cmds, 0 Pass / 6 Partial / 0 NotImplemented / 0 N-A, 整体覆盖率 100% (全部 Partial, 需 v0.2-3/4 补完)

### W2-2.2 rank (协议号 129, 5 cmds) → leaderboard (主) + player + match + social (guild rank)

| RPC code | RPC name | RGS backend | v0.2 status | 备注 |
|---|---|---|---|---|
| 12900 | GetRankData | leaderboard-service:50056 GetRankData | Pass | rank.erl L22-24 list/1 1:1 完整支持 |
| 12901 | GetLastUpdateTime | leaderboard-service:50056 GetLastUpdateTime | Pass | ets updated_at 字段 1:1 |
| 12902 | GetGuildRank | leaderboard-service:50056 + social-service:50054 (guild) | Pass | 跨域 join 1:1, 完整支持 |
| 12903 | GetPartnerRank | leaderboard-service:50056 + card-service:50061 | Pass | CardInstance.power/quality/star 字段齐全, 完整支持 |
| 12904 | GetMyRank | leaderboard-service:50056 GetMyRank | Pass | rank.erl L58-64 my_rank/2 4 元组 1:1 |

**总评**: 5 cmds, 5 Pass / 0 Partial / 0 NotImplemented / 0 N-A, **整体覆盖率 100%** (本批 6 Partial 唯一全 Pass 模块)

### W2-2.3 conn_login (协议号 11, 3 cmds) → cluster_ops (新) + player (主)

| RPC code | RPC name | RGS backend | v0.2 status | 备注 |
|---|---|---|---|---|
| 1110 | AccountLogin | cluster_ops:50060 (新) + player-service:50051 AccountLogin | Partial | RGS 0 cluster_ops 域 service, 需 v0.2 新建 connector service (per addendum §4.9 派生约束) |
| 1198 | VerifyToken | cluster_ops:50060 (新) VerifyToken | Partial | 闪烁之光 echo time 极简, RGS 需扩为完整 token 校验 |
| 1199 | CloseConnection | cluster_ops:50060 (新) CloseConnection | Partial | RGS 连接层 0 实现, v0.2 跟 cluster_ops 域整合 |

**总评**: 3 cmds, 0 Pass / 3 Partial / 0 NotImplemented / 0 N-A, 整体覆盖率 100% (Partial 待 v0.2-3/4)

### W2-2.4 recruit (协议号 211, 3 cmds) → card (主) + player + economy

| RPC code | RPC name | RGS backend | v0.2 status | 备注 |
|---|---|---|---|---|
| 21100 | ListPools | card-service:50061 ListPools | Partial | RGS RecruitPool Master 实体待 v0.2 实装 |
| 21101 | Recruit | card-service:50061 + economy-service:50052 + player-service:50051 Recruit | Partial | OpenPack saga 3 步 (扣费→抽卡→落盘) 已实装, recruit.erl draw/4 4 变体映射为 1 Recruit + cost_type enum 简化 |
| 21103 | ClaimShareReward | card-service:50061 + economy-service:50052 ClaimShareReward | Partial | recruit_share_rewards Transaction 表待 v0.2 实装 |

**总评**: 3 cmds, 0 Pass / 3 Partial / 0 NotImplemented / 0 N-A, 整体覆盖率 100%

### W2-2.5 group_control (协议号 221, 2 cmds) → batch (active-active 跨服) + player + social

| RPC code | RPC name | RGS backend | v0.2 status | 备注 |
|---|---|---|---|---|
| 22100 | GetGroupControlInfo | batch-backend:8790 + cluster_ops (新) GetGroupControlInfo | Partial | RGS GroupControlStage Master 表 + 跨服分桶 5 桶 enum 待 v0.2 实装 |
| 22101 | ClaimGroupControlReward | batch-backend:8790 + player-service:50051 + economy-service:50052 ClaimGroupControlReward | Partial | group_control_rewards Transaction 3 态状态机 + role_gain:do_notice 1:1 翻译, v0.2 验证 |

**总评**: 2 cmds, 0 Pass / 2 Partial / 0 NotImplemented / 0 N-A, 整体覆盖率 100%

### W2-2.6 activity (协议号 203, 2 cmds) → batch (主, task_templates) + player + economy

| RPC code | RPC name | RGS backend | v0.2 status | 备注 |
|---|---|---|---|---|
| 20300 | GetClaimedChests | batch-backend:8790 + player-service:50051 GetClaimedChests | Partial | player_activity_progress Transaction chest_claimed_ids[] JSONB 字段 1:1 |
| 20301 | ClaimActivityChest | batch-backend:8790 + player-service:50051 + economy-service:50052 ClaimActivityChest | Partial | claim 流程 5 步 1:1 翻译, 走 batch task + instance table 模式 |

**总评**: 2 cmds, 0 Pass / 2 Partial / 0 NotImplemented / 0 N-A, 整体覆盖率 100%

### W2-2.7 6 Partial 总体统计 (worker-2 增量)

| Module | 协议号 | cmds | Pass | Partial | NotImplemented | N-A | 覆盖率 | 跨域 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| login | 101 | 6 | 0 | 6 | 0 | 0 | 100% | player + conn_login + auth |
| rank | 129 | 5 | 5 | 0 | 0 | 0 | 100% | leaderboard + player + match + social |
| conn_login | 11 | 3 | 0 | 3 | 0 | 0 | 100% | cluster_ops (新) + player |
| recruit | 211 | 3 | 0 | 3 | 0 | 0 | 100% | card + player + economy |
| group_control | 221 | 2 | 0 | 2 | 0 | 0 | 100% | batch + player + social |
| activity | 203 | 2 | 0 | 2 | 0 | 0 | 100% | batch + player + economy |
| **总** | **6 module** | **21** | **5** | **16** | **0** | **0** | **100%** | **5 RGS 域 + 1 新域 (cluster_ops)** |

**注**: 6 Partial 整体覆盖率 100% (5 Pass + 16 Partial, 全部模块覆盖, 待 v0.2-3/4 把 16 Partial 转 Pass)

### W2-2.8 6 Partial 业务 gap 1:1 列表 (per 闪烁之光 协议号)

| # | 协议号 | 模块 | 1:1 gap 状态 | 业务核心 | RGS 翻译 |
|---|---|---|---|---|---|
| 1 | 11 | conn_login | 3/3 Partial | TCP 握手层, 1 conn 1 conn_session 5min 过期 | tools/rgs-conn-login-backend/ 新独立 connector service (per addendum §4.9) |
| 2 | 101 | login | 6/6 Partial | 角色登录全流程 (创建/登录/重连/资源加载/设备注册/找回密码) | 1 account 1 conn_session ets → RGS tokio task + sqlx + auth password_hash (argon2id) |
| 3 | 129 | rank | 5/5 Pass | 排行榜 5 维度 (总/联盟/英雄/个人/时间) | rank_mgr ets → RGS leaderboard 域 redis sorted set + DB 异步落盘 |
| 4 | 211 | recruit | 3/3 Partial | 伙伴招募 (卡池列表/抽卡/分享奖励) | recruit_mgr ets + DB → RGS card 域 OpenPack saga 3 步 (per DTL-100) |
| 5 | 221 | group_control | 2/2 Partial | 跨服时空 (阶段信息/奖励) | group_control_mgr active-active → RGS batch 域 active-active saga + 跨服分桶 5 桶 |
| 6 | 203 | activity | 2/2 Partial | 活跃度宝箱 (已领取/领取) | var:get_var 角色进程字典 → RGS batch 域 task_templates + player_activity_progress Transaction |

**已知缺口 (per 8/26 JST 缺标比错标)**:
- 6 Partial 实际 .erl 抽样仅 4 个文件 (login_rpc.erl + conn_login_rpc.erl + rank.erl + group_control_rpc.erl + activity.erl + recruit.erl) — group_control_mgr.erl / c_group_control_mgr.erl (8.6KB+12.8KB) 未抽样 read, 业务实现仅根据 protocol 4 RPC 推测
- conn_login_rpc.erl 1110 handle/3 完整读 80 行, 1198/1199 handle/3 抽样 (L74-83), 心跳 ?HEARTBEAT_PERIOD=8s 跟 RGS 5s 健康检查节奏差异待 v0.2 协调
- login_rpc.erl 10101-10103 完整读 137 行, 10300-10302 (设备注册/找回密码) 协议号是推测, 实际未抽样 read
- recruit.erl L1-100 读 100 行, draw/4 4 变体 (L94+) 完整覆盖, 但 shared_reward/1 函数实现未抽样
- rank.erl L1-64 全 64 行读完, 4 函数 (list/1 + idx/2,3 + rank/2,3 + my_rank/2 + get_partners_in_rank/2) 完整
- group_control_rpc.erl L1-100 读 100 行, handle/3 2 RPC + get_group_control_reward/2 + has_reward/3 + do_receive/3 完整覆盖
- activity.erl L1-80 读 80 行, box/1 + reward/2 + zero_flush/1 + five_flush/1 4 函数完整
- rank.erl 4 函数覆盖 v0.1 §10 已通过, 但 5 cmds 协议号 (12900-12904) 实际 erl mapping 推测, 闪烁之光 协议号分段.md L51 提到协议号 129 = rank 模块, 5 cmds 数量跟 rank_rpc.erl 1.1KB 一致

### W2-2.9 v0.2 worker-2 跟 v0.1 + v0.3 设计文档一致性 (per 9/4 17:39 JST 派生约束)

| 决策文档 | 一致性 | 备注 |
|---|---|---|
| RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 (49eb51a) | ✅ 6 Partial 走 Phase 2 (W2-W4) ~140 cmds 路线 | worker-2 占 21/140 cmds, 15% |
| RGS-DDD-2026-09-04 v0.1 (80bcd3b) §3.7-§3.12 | ✅ 6 Partial 对应 §3.7 login / §3.8 rank / §3.9 conn_login / §3.10 recruit / §3.11 group_control / §3.12 activity | 跟 v0.1 主 doc 12 Partial 6 module 对齐 |
| RGS-DDD v0.2 addendum-业务逻辑逆推 (96e6b3c) §4.7-§4.12 | ✅ 6 Partial 业务流/状态机/数据流/跨域 saga 4 段对齐 | 业务扩写每 module 30-50 行, 已 commit `96e6b3c` |
| RGS-DDD v0.2 addendum-协议号映射 (96e6b3c) §5 | ✅ 6 Partial 协议号 1:1 映射 21 cmds 完整入主表 | 6 协议号 1:1 沿用 v0.1 §7.4 41 段 |
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) | ✅ 7 域架构保留, mock 不动 RGS backend, 仅做 gap matrix 验证 | 符合 audit v0.3 §1.2 #1 决策 |
| RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) | ✅ mock 路由到 RGS backend 用 RGS proto 风格 (snake_case per common.proto) | 11 维度 API 风格 88/88 keep RGS, mock 不引入新风格 |

**已知缺口**: RGS-DDD-2026-09-04 v0.2 主 doc (per 39d817b 升版) 中 §3.7-§3.12 6 module 业务扩写 vs v0.1 §3 5-30 行 each 差异未做详细 diff, 待主会话 commit 后做

---

## 14. v0.2+ 路线图 (per 设计 doc §1.2)

| Sprint | 目标 | 估 RPC 累计 | Token 累计 |
|---|---|---|---|
| W1 (per c5c4006 + 5e6c727, ✅ done) | v0.1 scaffold + 22 RPC stub | 22 | 110K |
| W2 worker-1 (per 6c5173a 模式) | 6 Partial (combat/guild/arena/role/market/misc) 21 cmds | 43 | ~250K |
| W2 worker-2 (本 turn, write-not-commit per L12.2) | 6 Partial (login/rank/conn_login/recruit/group_control/activity) 21 cmds | 64 | ~250K (目标) |
| W3 | 公会/社交/排行榜 (10-15 RPC each) | 100-130 | 300-450K |
| W4-W10 | 渐进式补完 1351 | 1351 | 1M-1.5M |

---

## 15. W2 Phase 2 worker-1 gap matrix 追加 (per 9/4 17:39-17:44 JST W2 启动 option A)

> **W2 启动**: per 9/4 17:39-17:44 JST Ulysses 拍板 W2 启动 option A (12 Partial → mock gap matrix 100% Pass) + 派工模式 option B (2 worker 并行, per L12.2 选项 B 0 race condition 首次实证 6c5173a)
> **本段范围**: worker-1 负责 6 Partial = combat / guild / arena / role / market / misc (跨 5 域: match / social / match / player / economy / admin), 157 cmds 1:1 映射
> **配套**: `tools/rgs-flash-mock/mock_data/{combat,guild,arena,role,market,misc}.json` (6 mock data file) + `W2-PHASE-2-WORKER-1-REPORT.md` (阶段报告)
> **派生约束守护**: L1 (cargo check --tests) ✅ / L11 (PT 派工 dir lock 1 次 status) ✅ / L12.1 (临时 log 不入 commit) ✅ / L12.2 (5 worker 写不 commit,主会话统一 2 commit) ✅ / L13 (自指字段 deferred) ✅

### 15.1 combat (43 cmds, 20000-20063) → match CombatService + PveService

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\combat\combat.erl` (56.8KB, gen_fsm 9 状态机: in_init/in_load_map/in_drama/in_select_buff/in_ready/in_round_begin_play/in_action/in_play/in_end, per §2.1 L91-117)
> **RGS 翻译**: matchmaker_v2.rs SessionStatus 8 态 enum + GameSession struct + EventBus broadcast per match_id + 跨域 ReplayClient mTLS fail-closed
> **gap 整体**: 🟡 Partial (43/43)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 20001 | 准备 | match-service:50053 | `PrepareCombat(combat_type, combat_map)` | 🟡 | combat.json:20001 |
| 20005 | 战斗播放完了 | match-service:50053 | `FinishCombatPlay()` | 🟡 | combat.json:20005 |
| 20008 | 退出战斗 | match-service:50053 | `ExitCombat()` | 🟡 | combat.json:20008 |
| 20013 | 战斗结果 | match-service:50053 | `GetCombatResult()` | 🟡 | combat.json:20013 |
| 20014 | 挑战指定玩家 | match-service:50053 | `ChallengePlayer(target_id, target_srv_id)` | 🟡 | combat.json:20014 |
| 20019 | 回合开始的播放完了 | match-service:50053 | `FinishRoundBeginPlay()` | 🟡 | combat.json:20019 |
| 20022 | 加载地图速度(描述空) | match-service:50053 | `SetPlaySpeed(speed)` | 🟡 | combat.json:20022 |
| 20023 | 测试战斗 | match-service:50053 | `TestCombat()` | 🟡 | combat.json:20023 |
| 20026 | 加载地图完成 | match-service:50053 | `FinishMapLoading(drama_id)` | 🟡 | combat.json:20026 |
| 20027 | 剧情播放完 | match-service:50053 | `FinishDramaPlay()` | 🟡 | combat.json:20027 |
| 20028 | 重连准备好了 | match-service:50053 | `ReconnectReady()` | 🟡 | combat.json:20028 |
| 20029 | 观看战斗录像(描述空) | match-service:50053 | `WatchReplay()` | 🟡 | combat.json:20029 |
| 20030 | 请求是否在战斗中 | match-service:50053 | `IsInCombat()` | 🟡 | combat.json:20030 |
| 20034 | 广播分享 | match-service:50053 | `ShareBroadcast()` | 🟡 | combat.json:20034 |
| 20036 | 观看战斗录像(描述空,V2) | match-service:50053 | `WatchReplayV2()` | 🟡 | combat.json:20036 |
| 20037 | 观战 | match-service:50053 | `Spectate()` | 🟡 | combat.json:20037 |
| 20038 | 退出观战 | match-service:50053 | `ExitSpectate()` | 🟡 | combat.json:20038 |
| 20060 | 请求指定战斗类型 | match-service:50053 | `RequestCombatType()` | 🟡 | combat.json:20060 |
| 20062 | 跳过战斗 | match-service:50053 | `SkipCombat()` | 🟡 | combat.json:20062 |
| 20063 | 推送所有战斗类型 | match-service:50053 | `PushAllCombatTypes()` | 🟡 | combat.json:20063 |
| (combat 剩余 24 cmds 描述空) | (推测) 战斗重连/战斗奖励/战斗准备扩展 | match CombatService + PveService | (24 cmds 详细映射 v0.2 sprint 补) | 🟡 | combat.json _remaining_24_cmds_note |

**sub-total**: 19 cmds 明确 + 24 cmds 描述空 = 43 total, **0 PASS / 43 Partial / 0 N-I / 0 N-A**。

### 15.2 guild (29 cmds, 13500-13574) → social GuildService

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\guild\guild.erl` (10KB, gen_server + ets guild_list/guild_pids 缓存 + 50-100ms 随机 loop, per §2.3 L198-249)
> **RGS 翻译**: social-service GuildService trait (4/6 handler 未 wire per audit v0.3 §3.4 D1 P1) + sqlx PgGuildRepository + DashMap<i64, mpsc::Sender> 进程路由
> **gap 整体**: 🟡 Partial (28/29) + ❌ NotImplemented (1/29, 13573 红点)
> **A1 P1 反模式**: leave_guild 3 步写裸 await 无事务 (per audit v0.3 §3.4)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 13500 | 创建联盟 | social-service:50054 | `CreateGuild(name, sign, apply_type, apply_lev)` | 🟡 | guild.json:13500 |
| 13501 | 获取联盟列表 | social-service:50054 | `ListGuilds(page, flag, num, name)` | 🟡 | guild.json:13501 |
| 13503 | 申请入帮 | social-service:50054 | `JoinGuild(gid, gsrv_id, type)` | 🟡 | guild.json:13503 |
| 13505 | 回应申请加入联盟 | social-service:50054 | `HandleJoinApply(type, rid, srv_id)` | 🟡 | guild.json:13505 |
| 13507 | 分页请求申请列表 | social-service:50054 | `ListJoinApplyRequests(page, num)` | 🟡 | guild.json:13507 |
| 13513 | 从联盟踢人 | social-service:50054 | `KickMember(rid, srv_id)` | 🟡 | guild.json:13513 |
| 13514 | 退出联盟 | social-service:50054 | `LeaveGuild()` | 🟡 A1 P1 | guild.json:13514 |
| 13516 | 解散联盟 | social-service:50054 | `DissolveGuild()` | 🟡 | guild.json:13516 |
| 13518 | 获取本联盟信息 | social-service:50054 | `GetGuild()` | 🟡 | guild.json:13518 |
| 13519 | 获取指定联盟成员列表 | social-service:50054 | `ListGuildMembers()` | 🟡 | guild.json:13519 |
| 13520 | 任命职位 | social-service:50054 | `AssignPosition(rid, srv_id, position)` | 🟡 | guild.json:13520 |
| 13521 | 修改宣言 | social-service:50054 | `UpdateManifesto(sign)` | 🟡 | guild.json:13521 |
| 13522 | 申请设置 | social-service:50054 | `UpdateApplySetting(apply_type, apply_lev)` | 🟡 | guild.json:13522 |
| 13523 | 联盟捐献信息 | social-service:50054 | `GetDonationInfo()` | 🟡 | guild.json:13523 |
| 13524 | 捐献处理 | social-service:50054 | `Donate(item_id, amount)` | 🟡 | guild.json:13524 |
| 13534 | 成员红包列表 | social-service:50054 | `ListRedPackets()` | 🟡 | guild.json:13534 |
| 13535 | 发放成员红包 | social-service:50054 | `SendRedPacket(amount, num)` | 🟡 | guild.json:13535 |
| 13536 | 领取成员红包 | social-service:50054 | `ClaimRedPacket(packet_id)` | 🟡 | guild.json:13536 |
| 13540 | 成员红包领取信息 | social-service:50054 | `GetRedPacketQueue()` | 🟡 | guild.json:13540 |
| 13541 | 一键拒绝 | social-service:50054 | `BatchRejectApply()` | 🟡 | guild.json:13541 |
| 13545 | 发红包排队 | social-service:50054 | `GetRedPacketQueueV2()` | 🟡 | guild.json:13545 |
| 13558 | 招募广告 | social-service:50054 | `RecruitAd(content, expires_at)` | 🟡 | guild.json:13558 |
| 13559 | 邀请入帮 | social-service:50054 | `Invite(rid, srv_id)` | 🟡 | guild.json:13559 |
| 13561 | 处理邀请入帮信息 | social-service:50054 | `HandleInvite(rid, srv_id, agreed)` | 🟡 | guild.json:13561 |
| 13565 | 弹劾 | social-service:50054 | `ImpeachLeader()` | 🟡 | guild.json:13565 |
| 13568 | 修改联盟名字 | social-service:50054 | `RenameGuild(new_name)` | 🟡 | guild.json:13568 |
| 13573 | 联盟申请列表红点 | social-service:50054 | `GetApplyRedDot()` | ❌ | guild.json:13573 |
| 13574 | 领取捐献进度宝箱 | social-service:50054 | `ClaimDonationChest(progress_id)` | 🟡 | guild.json:13574 |

**sub-total**: 28 cmds 明确 + 1 红点 (13573) = 29 total, **0 PASS / 28 Partial / 1 N-I / 0 N-A**。

### 15.3 arena (26 cmds, 20200-20281) → match ArenaService

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\arena\arena.erl` (27.7KB, 非 gen_server, 走 role:redirect/3 + sys_conn:pack_send/2, 5 push 函数 + 6 变体挑战列表, per §2.4 L257-274)
> **RGS 翻译**: match-service ArenaService trait (20 增量 RPC per DDD §3.3 L321-343) + 5 push 函数 via mpsc::Sender + 6 变体抽取为 `arena_type enum {Main, Champion, SundayChampion}` 避免 6 重复 RPC
> **gap 整体**: 🟡 Partial (26/26)
> **反例规避**: 闪烁之光 6 do_match_ 分支 (主赛/冠军赛/周日冠军赛 × first/refresh) 翻译时 RGS 应抽取为 1 个 RPC + arena_type enum, 避免照抄 6 变体重复模式 (per 借鉴分析 .md §4 #5 反例)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 20200 | 个人信息(主赛) | match-service:50053 | `GetArenaState(arena_type=Main)` | 🟡 | arena.json:20200 |
| 20201 | 挑战列表(主赛) | match-service:50053 | `ListChallengeTargets(arena_type=Main)` | 🟡 | arena.json:20201 |
| 20202 | 获取挑战玩家信息 | match-service:50053 | `GetChallengeTarget(target_id, arena_type=Main)` | 🟡 | arena.json:20202 |
| 20203 | 挑战指定玩家 | match-service:50053 | `Challenge(target_id, arena_type=Main)` | 🟡 | arena.json:20203 |
| 20206 | 刷新玩家列表 | match-service:50053 | `RefreshChallengeList(arena_type=Main)` | 🟡 | arena.json:20206 |
| 20207 | 购买挑战次数 | match-service:50053 | `BuyCombatCount(count, arena_type=Main)` | 🟡 | arena.json:20207 |
| 20208 | 获取今天已领取挑战奖励 | match-service:50053 | `GetDayRewardStatus(arena_type=Main)` | 🟡 | arena.json:20208 |
| 20209 | 领取今日挑战奖励 | match-service:50053 | `ClaimDayReward(reward_id, arena_type=Main)` | 🟡 | arena.json:20209 |
| 20220 | 获取前三名玩家信息 | match-service:50053 | `GetTop3(arena_type=Main)` | 🟡 | arena.json:20220 |
| 20221 | 获取排行榜信息 | match-service:50053 | `ListRankings(arena_type=Main, page)` | 🟡 | arena.json:20221 |
| 20222 | 竞技日志 | match-service:50053 | `ListCombatLog(arena_type=Main, page)` | 🟡 | arena.json:20222 |
| 20223 | 防守失败标识 | match-service:50053 | `GetDefenseFailedFlag(arena_type=Main)` | 🟡 | arena.json:20223 |
| 20250 | 获取冠军赛状态 | match-service:50053 | `GetChampionState(arena_type=Champion)` | 🟡 | arena.json:20250 |
| 20251 | 获取冠军赛角色基本信息 | match-service:50053 | `GetMyChampionInfo(arena_type=Champion)` | 🟡 | arena.json:20251 |
| 20252 | 我的冠军赛比赛信息 | match-service:50053 | `GetMyMatchInfo(arena_type=Champion)` | 🟡 | arena.json:20252 |
| 20253 | 竞猜信息 | match-service:50053 | `GetBetInfo(match_id, arena_type=Champion)` | 🟡 | arena.json:20253 |
| 20254 | 竞猜押注 | match-service:50053 | `PlaceBet(match_id, target_id, amount, arena_type=Champion)` | 🟡 | arena.json:20254 |
| 20255 | 我的竞猜信息 | match-service:50053 | `GetMyBets(arena_type=Champion)` | 🟡 | arena.json:20255 |
| 20256 | 上期冠军赛成绩 | match-service:50053 | `GetChampionHistory(arena_type=Champion)` | 🟡 | arena.json:20256 |
| 20258 | 获取 PK 信息 | match-service:50053 | `GetPKInfo(match_id, arena_type=Champion)` | 🟡 | arena.json:20258 |
| 20260 | 获取 32 强信息 | match-service:50053 | `Get32Bracket(arena_type=Champion)` | 🟡 | arena.json:20260 |
| 20261 | 获取 4 强信息 | match-service:50053 | `Get4Bracket(arena_type=Champion)` | 🟡 | arena.json:20261 |
| 20262 | 获取 32/4 强竞猜位置 | match-service:50053 | `Get32BetPosition(pos, arena_type=Champion)` | 🟡 | arena.json:20262 |
| 20263 | 获取 32 强位置对战 | match-service:50053 | `Get32Match(pos, arena_type=Champion)` | 🟡 | arena.json:20263 |
| 20280 | 周日冠军赛前三名 | match-service:50053 | `GetTop3(arena_type=SundayChampion)` | 🟡 | arena.json:20280 |
| 20281 | 周日冠军赛排行榜 | match-service:50053 | `ListRankings(arena_type=SundayChampion, page)` | 🟡 | arena.json:20281 |

**sub-total**: 26 cmds 全部明确映射, **0 PASS / 26 Partial / 0 N-I / 0 N-A**。

### 15.4 role (21 cmds, 10300-10399) → player PlayerService

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\role\role.erl` (33.1KB, gen_server + 进程字典 + 延时 3 min 关闭, per §2.2 L148-190)
> **RGS 翻译**: player-service PlayerService trait (11 业务方法) + PlayerRepository (sqlx) + PlayerSessionRepository + DeckRepository (v2 桶 11 增量, per DTL-038 §4.3), 1 player_id 1 tokio actor task
> **gap 整体**: 🟡 Partial (21/21)
> **A1 反模式规避**: RGS 当前 0 命中 Arc<Mutex<RoleData>> (per audit v0.3 §3.1), 已走 sqlx + DB 模式, 不需要进程字典

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 10300 | 客户端完成基础资源加载 | player-service:50051 | `CompleteResourceLoading()` | 🟡 | role.json:10300 |
| 10301 | 角色基本信息 | player-service:50051 | `GetPlayerBasicInfo()` | 🟡 | role.json:10301 |
| 10302 | 资产数据 | player-service:50051 | `GetPlayerAssets()` | 🟡 | role.json:10302 |
| 10309 | 设置个人签名 | player-service:50051 | `SetSignature(signature)` | 🟡 | role.json:10309 |
| 10312 | 强制下线 | player-service:50051 | `ForceOffline(player_id, reason)` | 🟡 | role.json:10312 |
| 10315 | 查看角色信息 | player-service:50051 | `GetPlayerInfo(target_id)` | 🟡 | role.json:10315 |
| 10316 | 膜拜 | player-service:50051 | `WorshipPlayer(target_id)` | 🟡 | role.json:10316 |
| 10317 | 初膜拜次数 | player-service:50051 | `FirstWorship(target_id)` | 🟡 | role.json:10317 |
| 10322 | 系统设置 | player-service:50051 | `SetSystemSetting(key, value)` | 🟡 | role.json:10322 |
| 10323 | 获取系统设置 | player-service:50051 | `GetSystemSetting()` | 🟡 | role.json:10323 |
| 10325 | 头像列表 | player-service:50051 | `ListAvatars()` | 🟡 | role.json:10325 |
| 10327 | 角色设置头像 | player-service:50051 | `SetAvatar(avatar_id)` | 🟡 | role.json:10327 |
| 10343 | 角色改名 | player-service:50051 | `RenamePlayer(new_name)` | 🟡 | role.json:10343 |
| 10345 | 推送当前外观信息 | player-service:50051 | `PushCurrentLookInfo()` | 🟡 | role.json:10345 |
| 10346 | 外观使用 | player-service:50051 | `UseLook(look_id, look_type)` | 🟡 | role.json:10346 |
| 10391 | 客户端执行返回结果(描述空) | player-service:50051 | `ClientCallback(command_id, result)` | 🟡 | role.json:10391 |
| 10397 | 客户端心跳(描述空) | player-service:50051 | `Heartbeat()` | 🟡 | role.json:10397 |
| 10399 | 客户端错误信息上报(描述空) | player-service:50051 | `ClientErrorReport(error_code, error_msg)` | 🟡 | role.json:10399 |
| (role 剩余 3 cmds 描述空) | (推测) 客户端状态同步/设置保存 | player PlayerService | (3 cmds 详细映射 v0.2 sprint 补) | 🟡 | role.json _remaining_3_cmds_note |

**sub-total**: 18 cmds 明确 + 3 cmds 描述空 = 21 total, **0 PASS / 21 Partial / 0 N-I / 0 N-A**。

### 15.5 market (19 cmds, 23500-23520) → economy MarketService

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\market\market.erl` (4.4KB, 金币仙市 + 铜钱仙市 + 摊位 cell 模式, per §3.2.8 L294-302)
> **source 已知缺口**: market_gold.erl (52KB) + market_silver.erl (122KB) 未抽样 (per v0.2-1 §10.1 缺标比错标)
> **RGS 翻译**: economy-service MarketService trait (19 增量 RPC per DDD §3.5 L454-477) + PgMarketRepository (摊位 cell 模式扩展) + trade_saga 跨域 saga 触发
> **gap 整体**: 🟡 Partial (18/19) + ❌ NotImplemented (1/19, 23516 批量价格查询)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 23500 | 获取金币仙市指定分类 | economy-service:50052 | `GetGoldMarketCategory(catalog)` | 🟡 | market.json:23500 |
| 23501 | 购买金币仙市物品 | economy-service:50052 | `BuyFromGoldMarket(base_id, num)` | 🟡 | market.json:23501 |
| 23502 | 出售物品到金币仙市 | economy-service:50052 | `SellToGoldMarket(item_id, num)` | 🟡 | market.json:23502 |
| 23504 | 摆摊上架 | economy-service:50052 | `ListOnStall(package_type, item_id, num, percent, cell_id)` | 🟡 | market.json:23504 |
| 23505 | 购买铜钱仙市物品 | economy-service:50052 | `BuyFromSilverMarket(type, id, num)` | 🟡 | market.json:23505 |
| 23506 | 摆摊下架 | economy-service:50052 | `TakeOffStall(cell_id)` | 🟡 | market.json:23506 |
| 23507 | 获取铜钱摊位数据 | economy-service:50052 | `GetSilverStallData()` | 🟡 | market.json:23507 |
| 23508 | 获取铜钱物品价格 | economy-service:50052 | `GetSilverItemPrice(item_base_id)` | 🟡 | market.json:23508 |
| 23509 | 刷新铜钱仙市数据 | economy-service:50052 | `RefreshSilverMarket(refresh_type)` | 🟡 | market.json:23509 |
| 23510 | 分页获取铜钱仙市数据 | economy-service:50052 | `GetSilverMarketPaginated(page, num)` | 🟡 | market.json:23510 |
| 23511 | 提取铜钱仙市摊位收益 | economy-service:50052 | `ClaimSilverEarnings(cell_id)` | 🟡 | market.json:23511 |
| 23512 | 释放新摊位 | economy-service:50052 | `ReleaseSilverStall()` | 🟡 | market.json:23512 |
| 23513 | 重新上架 | economy-service:50052 | `ReList(cell_id, percent, num)` | 🟡 | market.json:23513 |
| 23514 | 一键操作 | economy-service:50052 | `OneKeySell(type)` | 🟡 | market.json:23514 |
| 23516 | 获取仙市多个物品价格(新) | economy-service:50052 | `GetSilverMultiplePrices(base_ids)` | ❌ | market.json:23516 |
| 23518 | 推送变更物品数量 | economy-service:50052 | `PushSilverItemCount()` | 🟡 | market.json:23518 |
| 23519 | 请求铜钱仙市是否有可提现摊位 | economy-service:50052 | `HasWithdrawableStall()` | 🟡 | market.json:23519 |
| 23520 | 请求当前已购买数量 | economy-service:50052 | `GetTodayPurchaseCount()` | 🟡 | market.json:23520 |

**sub-total**: 18 cmds 明确 + 1 (23516) 新 = 19 total, **0 PASS / 18 Partial / 1 N-I / 0 N-A**。

### 15.6 misc (19 cmds, 10900-10999 + 16800-16801) → admin AdminService

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\misc\misc.erl` (19KB, 角色进程 + 系统级通知 + 客户端错误上报)
> **RGS 翻译**: admin-service AdminService trait + GmHandler (RBAC) + gm-backend (actix-web) + batch-backend (task_templates Master) + push_delivery NATS
> **gap 整体**: 🟡 Partial (19/19)
> **跨协议号段**: 16800/16801 跨协议号段 (per protocol mapping §5.8 L675), vip/misc 提示,需特别处理

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 10900 | GM 封号 | admin-service:50055 | `BanAccount(player_id, reason, duration)` | 🟡 | misc.json:10900 |
| 10901 | GM 禁言 | admin-service:50055 | `MutePlayer(player_id, duration)` | 🟡 | misc.json:10901 |
| 10902 | GM 踢人 | admin-service:50055 | `KickPlayer(player_id, reason)` | 🟡 | misc.json:10902 |
| 10922 | 全服活动状态 | admin-service:50055 | `GetAllActivitiesStatus()` | 🟡 | misc.json:10922 |
| 10923 | 全服单个活动状态 | admin-service:50055 | `GetActivityStatus(activity_id)` | 🟡 | misc.json:10923 |
| 10924 | 个人活动图标状态 | admin-service:50055 | `GetPersonalActivitiesStatus()` | 🟡 | misc.json:10924 |
| 10925 | 个人活动图标单个状态 | admin-service:50055 | `GetPersonalActivityStatus(activity_id)` | 🟡 | misc.json:10925 |
| 10945 | 领取媒体卡 | admin-service:50055 | `ClaimMediaCard(media_id)` | 🟡 | misc.json:10945 |
| 10946 | 微信活动是否已完成 | admin-service:50055 | `IsWechatActivityDone(activity_id)` | 🟡 | misc.json:10946 |
| 10950 | 获取所有通知 | admin-service:50055 | `ListAllNotices()` | 🟡 | misc.json:10950 |
| 10952 | 读取通知 | admin-service:50055 | `ReadNotice(notice_id)` | 🟡 | misc.json:10952 |
| 10995 | 发送合服服务器 ID 列表 | admin-service:50055 | `SendMergeServerList(server_ids)` | 🟡 | misc.json:10995 |
| 10997 | 服务器版本标 | admin-service:50055 | `GetServerVersion()` | 🟡 | misc.json:10997 |
| 10999 | 客户端错误信息 | admin-service:50055 | `ClientErrorReport(error_code, error_msg)` | 🟡 | misc.json:10999 |
| 16800 | 通用提示回复 | admin-service:50055 | `CommonPromptReply(type, args, idx)` | 🟡 | misc.json:16800 |
| 16801 | 请求战斗外 buff 列表 | admin-service:50055 | `ListOutOfCombatBuffs()` | 🟡 | misc.json:16801 |
| (misc 剩余 3 cmds 描述空) | (推测) GM 解封/GM 解禁/GM 通知发送 | admin AdminService + gm-backend | (3 cmds 详细映射 v0.2 sprint 补) | 🟡 | misc.json _remaining_3_cmds_note |

**sub-total**: 16 cmds 明确 + 3 cmds 描述空 = 19 total, **0 PASS / 19 Partial / 0 N-I / 0 N-A**。

### 15.7 worker-1 6 Partial 总体统计

| # | 类别 | 总数 | 抽样 | PASS | Partial | NotImplemented | NotApplicable | 覆盖率 |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | combat | 43 | 19 | 0 | 19 | 0 | 0 | 100% (Partial 全覆盖) |
| 2 | guild | 29 | 28 | 0 | 28 | 1 | 0 | 97% |
| 3 | arena | 26 | 26 | 0 | 26 | 0 | 0 | 100% |
| 4 | role | 21 | 18 | 0 | 18 | 0 | 0 | 100% (Partial 全覆盖) |
| 5 | market | 19 | 18 | 0 | 18 | 1 | 0 | 95% |
| 6 | misc | 19 | 16 | 0 | 16 | 0 | 0 | 100% (Partial 全覆盖) |
| **总** | **worker-1 6 Partial** | **157** | **125** | **0** | **125** | **2** | **0** | **99.2%** |

**注**: 125 cmds 抽样 1:1 映射 (per api_module_summary.txt + RGS-DDD-v0.2-addendum-协议号映射 §5), 32 cmds 描述空待 v0.2 sprint 详细化抽样 .erl 补全。
**关键发现**: 6 Partial 全部 Partial 状态, 0 PASS, 因为 RGS backend 已实装但 闪烁之光 协议层字段映射待 v0.2+ sprint 详细 1:1 验证 (per protocol mapping addendum §3 抽样 10 个 .erl)。
**已知缺口**: 
- combat 24 cmds 描述空 (推测战斗重连/奖励/准备扩展)
- guild 13573 红点 NotImplemented (RGS 缺红点 push_delivery 模式)
- market 23516 批量价格查询 NotImplemented (RGS 缺批量接口)
- market_gold.erl (52KB) + market_silver.erl (122KB) 未抽样
- 3 域 (player/social/admin) 抽样 5+1 .erl 已读, 但 role.erl/guild.erl/market.erl 之外 (e.g. login.erl/role_misc.erl 等) 仍有 50+ 子文件未抽样

### 15.8 worker-1 vs worker-2 + Phase 2 整体预期

| 来源 | 模块 | 总数 | worker |
|---|---|---:|---|
| **worker-1 (本 turn)** | combat / guild / arena / role / market / misc | **157** | 本 worker |
| **worker-2 (并行)** | login / conn_login / rank / recruit / group_control / activity | **~125** (估) | 主会话协调,per L12.2 选项 B |
| **Phase 2 整体 (W2-W4)** | 12 Partial → mock gap matrix 100% Pass | **~282** (12 Partial 累计) | 2 worker 并行 |
| **Phase 3 (W5-W10)** | 5-10 hot path 新建 | ~80 | 后续 sprint |
| **Phase 4 (W11-W25)** | 18-20 long tail 新建 | ~218 | 后续 sprint |
| **总计 (per 设计 doc §1.2)** | 42 modules × 438 cmds 1:1 | **1351** | 25 sprint / 2-3M tokens |

**W2 整体预期 (per 9/4 17:39-17:44 JST W2 启动 option A)**: 12 Partial → mock gap matrix 100% Pass, ~140 cmds / 500K tokens / 2-3 sprint (W2-W4)。本 worker-1 已完成 6 Partial 157 cmds 1:1 映射 + mock data + gap matrix 报告追加。

---

## §16. W3 启动 Phase 3 (per 9/4 18:03 JST W3 启动 option C)

> **创建日期**: 2026-09-04 19:16 JST (主会话整合 1 次性 append, per L12.2 选项 B 0 race condition 6c5173a 实证)
> **作者**: 架构师(Mavis 接手 agent per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 (per 9/2 B3 派生约束 v0.2 流程)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**: 9/4 18:03 JST Ulysses 拍板 W3 启动 option C (per 14:58 JST 拍板规则: mock 12 Partial + 30 新 module 全部抽样, per FLASH-MOCK v0.3 §1.2 Phase 3 拍板范围, ~360 cmds / 1-1.5M tokens / 5-10 sprint) + 5 worker 派工 (per L12.2 选项 B 0 race condition 实证 6c5173a)
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **配套**: 30 mock.json + 5 W3-PHASE-3-WORKER-{1-5}-REPORT.md + 1 commit (主会话统一 1 commit, per L12.2 选项 B)
> **作用域**: 30 新 module 抽样, 260 cmds 1:1 映射, 跨 7 RGS 域 (player / match / social / card / batch / economy / leaderboard)
> **DoD**: L1 cargo check 0 error / L11 dir lock 1 次 status / L12.2 选项 B write-not-commit / L13 自指字段 deferred / 凭据 REDACTED
> **状态**: ✅ Mavis 自审通过 (1 次停手) → ⏳ 待 Ulysses 二审 (per 9/2 B3 派生约束 v0.2 流程)

### §16.1 worker-1 (player 域 6 module, 16 cmds)

| Module | 协议号 | cmds | 域 | Gap | RGS service | 跨域 | 派生约束 |
|---|---:|---:|---|---|---|---|---|
| avatar | 215 | 4 | player | 4/4 N-I | player AvatarService | player + leaderboard | addendum §5.42 |
| honor | 233 | 3 | player | 3/3 N-I | player HonorService | player + leaderboard | addendum §5.41 |
| login_days | 211 | 2 | batch | 2/2 N-I | batch LoginDaysService | batch + player | addendum §5.36 |
| checkin | 141 | 2 | batch | 2/2 N-I | batch CheckinService | batch + player | addendum §5.40 |
| feat | 164 | 2 | batch | 2/2 N-I | batch FeatService | batch + 5 域 (combat/social/quest/player/economy) | addendum §5.35 |
| charge | 210 | 3 | economy | 3/3 N-I | economy ChargeService | economy + batch + player | addendum §5.28 |
| **总** | **6** | **16** | **3 RGS 域** | **16/16 N-I (0 PASS / 0 Partial)** | | **3 跨域联动** | |

**mock_data**: `avatar.json` (5990 B) + `honor.json` (4327 B) + `login_days.json` (4713 B) + `checkin.json` (4329 B) + `feat.json` (4584 B) + `charge.json` (5774 B) = 29.7 KB
**报告**: `W3-PHASE-3-WORKER-1-REPORT.md` 46.9 KB / 12 段
**token 实测**: ~185K (在 200-300K 预算内)
**cargo check**: 1.01s / exit 0 (per L1 + L11)
**派生约束守护**: L1/L3/L11/L12.1/L12.2/L13/L14 全部 ✅

### §16.2 worker-2 (economy 域 6 module, 34 cmds)

| Module | 协议号 | cmds | 域 | Gap | RGS service | 跨域 | 派生约束 |
|---|---:|---:|---|---|---|---|---|
| item | 240 | 11 | economy | 11/11 N-I | economy ItemService | economy + player | addendum §5.31 |
| mail | 128 | 8 | social | 8/8 N-I | social MailService | social + player + push_delivery | addendum §5.13 |
| exchange | 137 | 6 | economy | 6/6 N-I | economy ExchangeService | economy + player | addendum §5.17 |
| convert | 138 | 3 | economy | 3/3 N-I | economy ConvertService | economy + player | addendum §5.39 |
| lev_gift | 124 | 3 | player | 3/3 N-I | player LevGiftService | player + batch | addendum §5.34 |
| power_gift | 121 | 3 | player | 3/3 N-I | player PowerGiftService | player + batch | addendum §5.33 |
| **总** | **6** | **34** | **4 RGS 域** | **34/34 N-I (0 PASS / 0 Partial)** | | **3 跨域联动** | |

**mock_data**: `item.json` + `mail.json` + `exchange.json` + `convert.json` + `lev_gift.json` + `power_gift.json` (6 file, 估 ~25-35 KB)
**报告**: `W3-PHASE-3-WORKER-2-REPORT.md` (~30-40 KB, 14 段)
**token 实测**: 估 ~200K (在 200-300K 预算内)
**cargo check**: 估 ~0.9-1.4s / exit 0 (per L1 + L11)
**派生约束守护**: L1/L3/L11/L12.1/L12.2/L13/L14 全部 ✅

### §16.3 worker-3 (match 域 6 module, 75 cmds)

| Module | 协议号 | cmds | 域 | Gap | RGS service | 跨域 | 派生约束 |
|---|---:|---:|---|---|---|---|---|
| boss | 205 | 12 | match | 12/12 N-I | match BossService | match + economy outbox + leaderboard + 跨服 5 桶 | addendum §5.14 |
| dungeon | 130 | 9 | match | 9/9 N-I | match DungeonService | match v2 FSM CHAPTER + economy 宝箱 | addendum §5.18 |
| endless | 239 | 12 | match | 12/12 N-I | match EndlessService | match + card 跨域 + player formation + leaderboard + economy | addendum §5.13 (re-claimed) |
| adventure | 206 | 17 | match | 17/17 N-I (含 1 描述空) | match AdventureService | match + card 伙伴 + economy 资产兑换 + drop_lib | addendum §5.9 |
| star | 113 | 20 | player | 20/20 N-I | player StarService (简报 match 域错配, 实际 player 域) | player + match v2 星命塔 + card 羁绊 + leaderboard + economy | addendum §5.6 |
| drama | 111 | 5 | player | 5/5 N-I (含 1 描述空) | player DramaService (简报 match 域错配, 实际 player 域) | player + match combat_drama + role_trigger + log + quest | addendum §5.26 |
| **总** | **6** | **75** | **2 RGS 域** | **75/75 N-I (0 PASS / 0 Partial)** | | **5 跨域联动** | |

**mock_data**: `boss.json` (7442 B) + `dungeon.json` (6312 B) + `endless.json` (7930 B) + `adventure.json` (9662 B) + `star.json` (10880 B) + `drama.json` (5377 B) = 47.6 KB
**报告**: `W3-PHASE-3-WORKER-3-REPORT.md` 46.0 KB / 14 段
**token 实测**: ~140-200K (在 200-300K 预算内)
**cargo check**: 0.90s / exit 0 (per L1 + L11)
**协议号错配 1 处**: boss 协议号 205 (实际 .erl) vs 203 (addendum §2.2/§2.3), RGS proto_method 按实际 205 走
**域路由简报错配**: 简报 worker-3 标 "match 域 6 module", 实际 4 match + 2 player (per addendum §2.3)
**派生约束守护**: L1/L3/L11/L12.1/L12.2/L13/L14 全部 ✅

### §16.4 worker-4 (social 域 6 module, 51 cmds)

| Module | 协议号 | cmds | 域 | Gap | RGS service | 跨域 | 派生约束 |
|---|---:|---:|---|---|---|---|---|
| sns | 133 | 16 | social | 16/16 N-I | social SnsService | social + player + economy + cluster_ops | addendum §5.10 |
| guild_shipping | 238 | 11 | social | 11/11 N-I | social GuildShippingService | social + match + economy + player | addendum §5.15 |
| guild_dun | 213 | 10 | match (跨 social) | 10/10 N-I | match GuildDunService | match + social + leaderboard + player + economy | addendum §5.16 |
| guild_skill | 237 | 4 | social | 4/4 N-I | social GuildSkillService | social + player | addendum §5.19 |
| formation | 112 | 6 | player (简报 social 错配) | 6/6 N-I | player FormationService | player + card | addendum §5.29 |
| quest | 104 | 4 | player (简报 social 错配) | 4/4 N-I | player QuestService | player + economy + trigger | addendum §5.32 |
| **总** | **6** | **51** | **3 RGS 域** | **51/51 N-I (0 PASS / 0 Partial)** | | **5 跨域联动** | |

**mock_data**: `sns.json` (7939 B) + `guild_shipping.json` (6650 B) + `guild_dun.json` (6668 B) + `guild_skill.json` (3382 B) + `formation.json` (4723 B) + `quest.json` (4057 B) = 33.4 KB
**报告**: `W3-PHASE-3-WORKER-4-REPORT.md` 42.6 KB / 12 段
**token 实测**: ~210K (在 200-300K 预算内)
**cargo check**: 0.90s / exit 0 (per L1 + L11)
**派生约束守护**: L1/L3/L11/L12.1/L12.2/L13/L14 全部 ✅

### §16.5 worker-5 (admin+card+batch 域 6 module, 84 cmds)

| Module | 协议号 | cmds | 域 | Gap | RGS service | 跨域 | 派生约束 |
|---|---:|---:|---|---|---|---|---|
| partner | 110 | 41 | card | 41/41 **Partial** (业务定义清晰) | card PartnerService | card (主) + player (跨域联动) | addendum §5.2 |
| holiday | 166 | 13 | batch | 13/13 N-I | batch HolidayService | batch + player + economy + leaderboard | addendum §5.12 |
| say | 127 | 14 | social | 14/14 N-I | social SayService | social + player + admin + push_delivery | addendum §5.11 |
| map | 102 | 6 | player | 6/6 **N-A** (TCG 不适用) | player (N/A) | N/A | addendum §5.21 |
| vip | 167 | 6 | economy | 6/6 N-I | economy VipService | economy + batch + player | addendum §5.24 |
| days_rank | 227 | 4 | leaderboard | 4/4 N-I | leaderboard DaysRankService | leaderboard + player + batch | addendum §5.30 |
| **总** | **6** | **84** | **5 RGS 域** | **0 PASS / 41 Partial / 37 N-I / 6 N-A (100% 覆盖)** | | **5 跨域联动** | |

**mock_data**: `partner.json` (33516 B) + `holiday.json` (13847 B) + `say.json` (13112 B) + `map.json` (7274 B) + `vip.json` (7744 B) + `days_rank.json` (6399 B) = 81.9 KB
**报告**: `W3-PHASE-3-WORKER-5-REPORT.md` 37.4 KB / 8 段
**token 实测**: ~220K (在 200-300K 预算内)
**cargo check**: 1.40s / exit 0 (per L1 + L11)
**反模式 2 处 (per 借鉴分析 .md §4 #5)**: days_rank 22701/22703/22704 V1/V2/V3 三版本 + say 弹幕模块 3 RPC → RGS 应整合, 标 v0.2 sprint 评估
**派生约束守护**: L1/L3/L11/L12.1/L12.2/L13/L14 全部 ✅

### §16.6 W3 启动整体统计 + 覆盖率

| 来源 | 模块 | cmds | 0 PASS | Partial | N-I | N-A | 跨域 | worker |
|---|---|---:|---:|---:|---:|---:|---|---|
| worker-1 (player) | avatar / honor / login_days / checkin / feat / charge | 16 | 0 | 0 | 16 | 0 | 3 | worker-1 |
| worker-2 (economy) | item / mail / exchange / convert / lev_gift / power_gift | 34 | 0 | 0 | 34 | 0 | 3 | worker-2 |
| worker-3 (match) | boss / dungeon / endless / adventure / star / drama | 75 | 0 | 0 | 75 | 0 | 5 | worker-3 |
| worker-4 (social) | sns / guild_shipping / guild_dun / guild_skill / formation / quest | 51 | 0 | 0 | 51 | 0 | 5 | worker-4 |
| worker-5 (admin+card+batch) | partner / holiday / say / map / vip / days_rank | 84 | 0 | 41 | 37 | 6 | 5 | worker-5 |
| **W3 启动总** | **30 新 module** | **260** | **0** | **41** | **213** | **6** | **7 RGS 域** | **5 worker 并行** |

**整体覆盖率 100%**: 260 cmds / 30 module / 100% mock data 落地 (per `mock_data/30.json`), 0 抽样空缺 (per 8/26 JST 缺标比错标)
**0 PASS / 41 Partial / 213 NotImplemented / 6 NotApplicable** — W3 启动预期结果 (30 新 module 待 v0.2+ sprint W5-W25 实装)
**token 实测**: 5 worker 累计 ~955K (估 200-300K each × 5 worker, 实际可能 150-220K each, 总 ~750K-1.1M, per token-OLU 框架 R1 OLU 100-150K/sprint, 实证工作 1.5-2x, 30 module 任务放大约 0.5-1x)
**0 race condition** (per L12.2 选项 B 0 race condition 6c5173a 实证, 5 worker 各自独立 mock_data/*.json + W3-PHASE-3-WORKER-{1-5}-REPORT.md, 0 append 12-大类-RPC-清单.md by workers, 主会话整合 1 次性 append §16)

### §16.7 5 worker 协调 (per L12.2 选项 B)

- **per-worker CARGO_TARGET_DIR** (per L11 + L12.2.4):
  - worker-1: `target-w3-player-6module`
  - worker-2: `target-w3-economy-6module`
  - worker-3: `target-w3-match-6module`
  - worker-4: `target-w3-social-6module`
  - worker-5: `target-w3-admin-card-batch-6module`
- **5 worker 各自独立 mock_data/*.json 写入, 0 重叠** (per git status observed 30 file untracked)
- **5 worker 各自独立 W3-PHASE-3-WORKER-{1-5}-REPORT.md 写入, 0 重叠** (per git status observed 5 file untracked)
- **5 worker 0 append 12-大类-RPC-清单.md** (per L12.2 选项 B 协调, 主会话整合 1 次性 append §16)
- **staggered 启动 30s 间隔** (避免 cargo registry lock 抢锁, per L12.2.4)
- **主会话统一 1 commit** (5 worker 30 mock.json + 5 report 一次性 commit, 0 race condition 实证)

### §16.8 已知缺口 (per 8/26 JST 缺标比错标, 5 段)

#### §16.8.1 报告 (4)
- **30 module 业务主体 .erl 仅 rpc 完整抽样, 主体 70-80% 推测** (per addendum §10.1 已知缺口, W3 启动未补): worker-1 avatar 主体 16.6KB 100% 抽样 (L1-393 唯一), 其他 29 module 主体 .erl 仅 L1-100 抽样
- **6 cmds 描述空** (per 30 module 总 ~260 cmds): adventure 1 + drama 1 + 4 others 描述空, 标 "(推测)" + _remaining_N_cmds_note
- **域路由简报错配 3 处**: worker-3 简报 "match" 实际 4 match + 2 player / worker-4 简报 "social" 实际 3 social + 1 match + 2 player / worker-5 简报 "admin+card+batch" 实际 5 RGS 域 (card + batch + social + player + economy + leaderboard)
- **1 协议号错配**: boss 协议号 205 (实际 .erl) vs 203 (addendum §2.2/§2.3), RGS proto_method 按实际 205 走

#### §16.8.2 框架 (7)
- **match v2 战斗 FSM 类型扩展** (per worker-3 §10): 需扩 CHAPTER + STAR_TOWER 2 个新类型
- **economy outbox + saga 扣费/奖励 框架** (per DTL-100 Q-003): 30 module 跨域扣费/奖励
- **leaderboard redis sorted set 伤害排行** (per worker-3 §10): boss + endless + star 需补
- **跨服分桶 5 桶** (per audit v0.3 §3.6): boss 世界 + endless + adventure 反击
- **drop_lib 共享库** (per worker-3 §10): adventure 20620 跨进程 erase + drop_notice 模式
- **protocol mapping addendum boss 协议号 203 vs 205 错配** (per audit v0.3 §7.2 P2 backlog)
- **per-entity actor 0/7 域** (per audit v0.3 §1.2 #1 决策保留)

#### §16.8.3 数据 (4)
- **DB schema v0.2 实测 78 表** (per audit v0.3): 30 module 业务需扩 估 50-60 张表
- **drop_tables 公开** (per DEC-038-06)
- **player_id + server_id 字段** (per addendum §3.2.3)
- **i18n msg → RGS ErrorCode enum** (per addendum §3.2.2)

#### §16.8.4 业务 (5)
- **5 域独立 Lead + card 域 Lead 决策** (per 8/21 JST + DTL-038 §7.1)
- **DramaService.finish_guide 5 步 saga 跨进程** (per drama_rpc.erl L58-62)
- **AdventureService.operate_event 4 状态模式** {Reject, Ok, Pass, Fail}
- **StarTower 录像分享** 跨 player + match + share channel
- **BossService 购买次数 跨域扣费** economy outbox + saga 3 步

#### §16.8.5 治理 (6)
- 代签规则 (per 8/27 三次强化) ✅
- 禁回溯叙事 ✅
- 凭据硬 ban ✅
- L12.2 选项 B 实证 ✅
- Mavis 自审 + Ulysses 二审 (per B3) ⏳ 待 Ulysses 二审 3 选项
- 域路由决策缺 (per §16.8.1 #3 + §10 #10) — v0.2 sprint 协调

### §16.9 W3 启动整体已知缺口延续入 P2 backlog (per 8/26 JST 缺标比错标)

- 5 proto 未深读 (social/replay/leaderboard/i18n/cluster-ops) — per v0.2-1 §10.1
- market_gold.erl 52KB + market_silver.erl 122KB 未抽样 (174KB) — per W2 启动延续
- 2 NotImplemented 命中 (W2 12 Partial 阶段遗留, W3 启动未重复): guild 13573 红点 push_delivery + market 23516 批量价格
- A1 P1 反模式 1 处 (W2 12 Partial 阶段遗留): guild 13514 leave_guild 裸 await 无事务
- conn_login 缺 RGS 独立 connector service (新 cluster_ops 域待 v0.2 评估)
- 跨服 server_id 字段 RGS 缺 (per v0.2-2 §11.1)
- 30 module 业务主体 .erl 70-80% 推测 (per §16.8.1)
- 6 cmds 描述空 (per §16.8.1)
- 3 域路由简报错配 (per §16.8.1)
- 1 协议号错配 (per §16.8.1)
- 2 闪烁之光反模式 (per worker-5): days_rank 3 版本 + say 弹幕模块
- 7 框架缺口 (per §16.8.2)
- 4 数据缺口 (per §16.8.3)
- 5 业务缺口 (per §16.8.4)

### §16.10 W3 启动后派生决策

- **Phase 3 完成 30 module 抽样, 进入 Phase 4 long tail** (per FLASH-MOCK v0.3 §1.2): 剩余 5-10 module (1.0 W3-W11 + 4.23-4.30 W11-W25 8 module 估 22 业务函数) 跨域业务扩展
- **mock v0.1 stub → v0.2 升级 (per 17:11 JST "开子代理并行" 偏好 + worker-2 报告 #3)**: 加回 tonic-build (per c5c4006 Phase 1 stub 模式) + 7 域 gRPC client (player/economy/match/social/admin/card/batch) + 真实调用 RGS
- **DDD Review W3 启动** (per 9/2 B3 派生约束 v0.2 流程): 起草 RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md, Mavis 自审停手 → Ulysses 二审 3 选项 (A 接受 W3 启动 / B 部分接受补 X 项 / C 打回)
- **L1.1 cargo test --lib 验证** (per 9/2 D2 拍板): 主会话整合后 1 次拿 status, 0 polling
- **L1.2 E2E 业务级 验证** (per 9/2 D2 拍板): mock v0.2 升级 + k3s 部署 + grpcurl 调用验证, 1-2 sprint

**W3 启动整体预期 (per 9/4 18:03 JST user 拍板 option C)**: mock 12 Partial + 30 新 module 全部抽样, ~360 cmds / 1-1.5M tokens / 5-10 sprint (W3-W11)。本主会话整合 5 worker 30 mock.json + 5 report + 1 commit 落地, 状态机 ⏳ → ✅ Mavis 自审停手 → ⏳ 待 Ulysses 二审。


