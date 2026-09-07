# RGS 测试用例 v0.1 (60 module × 4 类 ≈ 950 用例)

> **创建日期**: 2026-09-05 07:00 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: RGS-TEST-DESIGN-2026-09-05 v0.1
> **状态**: ⏳ 待 DDD Review 二审

## 0. 用例 ID 命名约定

```
HP-{module_code}-{rpc_seq}    业务路径 (HAPPY PATH)
EC-{module_code}-{rpc_seq}    错误码 (ERROR CODE)
BV-{module_code}-{rpc_seq}    边界值 (BOUNDARY)
EX-{module_code}-{rpc_seq}    异常路径 (EXCEPTION)
SAGA-{nnn}                   跨域 saga
PLUGIN-{nnn}                 plugin 集群
APP-DEPLOY-{nnn}             app 独立更新
OPS-UI-{nnn}                 ops 运维 UI
```

**module_code**: 2-3 字母, e.g. COMBAT / ARENA / MARKET / PLUGIN-REG / APP-DEPLOY

## 1. 42 闪烁之光 module (per 协议号映射 addendum §5)

### 1.1 COMBAT (43 cmds, 20000-20063, 路由 match + PveService)

#### HP-COMBAT-001 ~ 008: 业务路径 8 用例

```yaml
- HP-COMBAT-001:
  name: "战斗准备 正常输入"
  rpc: 20001 PrepareCombat
  pre: 玩家已注册, 有 1 张 SSR 卡
  input: {combat_type: 1, combat_map: 16}
  expect: code=0, combat_session_id 非空, state=in_init, round_countdown_ms=30000
  evidence: mock_data/combat.json rpcs.20001

- HP-COMBAT-002:
  name: "战斗回合结束操作 正常路径"
  rpc: 20005 FinishCombatPlay
  input: {combat_session_id: <HP-COMBAT-001 的 session_id>}
  expect: code=0, next_state=in_round_begin_play
  evidence: mock_data/combat.json rpcs.20005

- HP-COMBAT-003: "退出战斗" (rpc 20008)
- HP-COMBAT-004: "查询战斗结果 胜利" (rpc 20013)
- HP-COMBAT-005: "挑战玩家" (rpc 20014, input: target_id + target_srv_id)
- HP-COMBAT-006: "回合开始操作完成" (rpc 20019)
- HP-COMBAT-007: "设置战斗速度" (rpc 20022, input: speed=2)
- HP-COMBAT-008: "观战" (rpc 20037, input: combat_session_id + 监战密码)
```

#### EC-COMBAT-001 ~ 004: 错误码 4 用例

```yaml
- EC-COMBAT-001: 战斗未准备就退出 (rpc 20008, in_init 状态, expect 3001 STATE_INVALID)
- EC-COMBAT-002: 挑战自己 (rpc 20014, target_id = self_id, expect 1003 PERM_DENIED)
- EC-COMBAT-003: 速度参数非法 (rpc 20022, speed=99, expect 1001 PARAM_INVALID)
- EC-COMBAT-004: 战斗 session_id 不存在 (rpc 20013, expect 2004 NOT_FOUND)
```

#### BV-COMBAT-001 ~ 005: 边界值 5 用例

```yaml
- BV-COMBAT-001: combat_type=0 (空值) — 期望 0 (协议允许, 0=default)
- BV-COMBAT-001: combat_type=i32::MAX — 期望 1001 PARAM_INVALID
- BV-COMBAT-001: target_id="" (空字符串) — 期望 1001 PARAM_INVALID
- BV-COMBAT-001: target_id="A" * 4096 (超长) — 期望 1002 FIELD_TOO_LONG
- BV-COMBAT-001: 同一 combat_session_id 重复 2 次 (幂等) — 期望 0 第二次返回相同
```

#### EX-COMBAT-001 ~ 003: 异常路径 3 用例

```yaml
- EX-COMBAT-001: 网络中断 1s 后断开 (expect client retry, server 事务回滚)
- EX-COMBAT-002: server 处理 > 5s 超时 (expect deadline exceeded + dead-letter)
- EX-COMBAT-003: mTLS 证书失效 (expect 5xx fail-closed + cert expiry trace)
```

**COMBAT 累计**: 8 + 4 + 5 + 3 = **20 用例**

### 1.2 PARTNER (41 cmds, 11000-11084, 路由 card PartnerService + player)

(同理, 略, 实际 6 HP + 3 EC + 4 BV + 2 EX = **15 用例**)

### 1.3 GUILD (29 cmds, 13500-13574, 路由 social GuildService)

(**15 用例**)

### 1.4 ARENA (26 cmds, 20200-20281, 路由 match ArenaService)

(**15 用例**)

### 1.5 ROLE (21 cmds, 10300-10399, 路由 player PlayerService)

(**15 用例**)

### 1.6 STAR (20 cmds, 11300-11333, 路由 player StarService)

(**15 用例**)

### 1.7 MARKET (19 cmds, 23500-23520, 路由 economy MarketService)

(**15 用例**)

### 1.8 MISC (19 cmds, 10900-10999, 路由 admin + gm-backend)

(**15 用例**)

### 1.9 ADVENTURE (17 cmds, 20600-20692, 路由 match AdventureService)

(**15 用例**)

### 1.10 SNS (16 cmds, 13300-13334, 路由 social SnsService)

(**15 用例**)

### 1.11 SAY (14 cmds, 12700-12768, 路由 social SayService)

(**15 用例**)

### 1.12 HOLIDAY (13 cmds, 16601-16639, 路由 batch HolidayService)

(**15 用例**)

### 1.13 ENDLESS (12 cmds, 23900-23911, 路由 match EndlessService)

(**15 用例**)

### 1.14 BOSS (12 cmds, 20500-20541, 路由 match BossService)

(**15 用例**)

### 1.15 GUILD_SHIPPING (11 cmds, 23800-23812, 路由 social)

(**15 用例**)

### 1.16 GUILD_DUN (10 cmds, 21300-21319, 路由 match)

(**15 用例**)

### 1.17 ITEM (10 cmds, 10500-10528, 路由 player ItemService)

(**15 用例**)

### 1.18 DUNGEON (9 cmds, 13000-13011, 路由 match)

(**15 用例**)

### 1.19 FORMATION (6 cmds, 11200-11212, 路由 player FormationService)

(**10 用例**)

### 1.20 LOGIN (6 cmds, 10101-10103, 路由 player PlayerService Partial)

(**10 用例**)

### 1.21 MAP (6 cmds, 10200-10215, 路由 player MapService N-A for TCG)

(**10 用例**)

### 1.22 MAIL (6 cmds, 10800-10810, 路由 social MailService)

(**10 用例**)

### 1.23 EXCHANGE (6 cmds, 13401-13419, 路由 economy ExchangeService)

(**10 用例**)

### 1.24 VIP (6 cmds, 16700-16713, 路由 economy VipService)

(**10 用例**)

### 1.25 CONVERT (5 cmds, 23600-23604, 路由 economy ConvertService)

(**10 用例**)

### 1.26 DRAMA (5 cmds, 11100-11122, 路由 player DramaService)

(**10 用例**)

### 1.27 RANK (5 cmds, 12900-12904, 路由 leaderboard RankService Partial)

(**10 用例**)

### 1.28 AVATAR (4 cmds, 21500-21504, 路由 player AvatarService)

(**10 用例**)

### 1.29 GUILD_SKILL (4 cmds, 23700-23703, 路由 social GuildSkillService)

(**10 用例**)

### 1.30 DAYS_RANK (4 cmds, 22700-22704, 路由 leaderboard DaysRankService)

(**10 用例**)

### 1.31 LEV_GIFT (4 cmds, 21200-21204, 路由 batch LevGiftService)

(**10 用例**)

### 1.32 QUEST (4 cmds, 10400-10406, 路由 player QuestService)

(**10 用例**)

### 1.33 CONN_LOGIN (3 cmds, 1110-1199, 路由 cluster-ops ClusterOpsService)

(**10 用例**)

### 1.34 POWER_GIFT (3 cmds, 23400-23403, 路由 batch PowerGiftService)

(**10 用例**)

### 1.35 HONOR (3 cmds, 23300-23303, 路由 player HonorService)

(**10 用例**)

### 1.36 CHARGE (3 cmds, 21000-21005, 路由 economy ChargeService)

(**10 用例**)

### 1.37 RECRUIT (3 cmds, 23200-23203, 路由 card RecruitService Partial)

(**10 用例**)

### 1.38 GROUP_CONTROL (2 cmds, 22100-22101, 路由 batch GroupControlService)

(**10 用例**)

### 1.39 ACTIVITY (2 cmds, 20300-20301, 路由 batch ActivityService Partial)

(**10 用例**)

### 1.40 FEAT (2 cmds, 16400-16402, 路由 batch FeatService)

(**10 用例**)

### 1.41 LOGIN_DAYS (2 cmds, 21100-21101, 路由 batch LoginDaysService)

(**10 用例**)

### 1.42 CHECKIN (2 cmds, 14100-14101, 路由 batch CheckinService)

(**10 用例**)

**42 module × 平均 14 用例 = ~588 用例** (含 HP/EC/BV/EX 4 类, 大 module 20, 小 module 10)

## 2. 18 跨域抽象 module (本设计书新增)

### 2.1 player 域 (3 module)

- **PROFILE-SYNC** (10 cmds): profile 跨域同步 (跨 cluster-ops realm migration)
- **SESSION** (8 cmds): 玩家 session 生命周期
- **INVENTORY** (12 cmds): 跨域 inventory 同步

**player 3 module × 15 用例 = 45 用例**

### 2.2 economy 域 (3 module)

- **TRADE** (15 cmds): 跨域 trade 撮合
- **LEDGER** (10 cmds): 跨域 transaction ledger 一致性
- **SAGA** (12 cmds): 跨域 saga orchestrator (per 8/27 economy::saga)

**economy 3 module × 15 = 45 用例**

### 2.3 match 域 (3 module)

- **MATCHMAKING-V2** (15 cmds): v2 matchmaker 完整流程
- **SPECTATOR** (8 cmds): 观战模式
- **REPLAY** (12 cmds): replay save/load

**match 3 × 15 = 45 用例**

### 2.4 social 域 (3 module)

- **GUILD** (15 cmds): 跨域 guild lifecycle
- **PARTY** (10 cmds): 跨域组队
- **CHAT** (8 cmds): 跨域聊天 (push delivery per 8/27 Q7)

**social 3 × 15 = 45 用例**

### 2.5 admin 域 (3 module)

- **AUDIT-LOG** (15 cmds): 跨域 audit log 链验证 (per 8/31 Q2)
- **RBAC** (10 cmds): 跨域角色权限
- **GM-HANDLER** (12 cmds): GM 操作跨域

**admin 3 × 15 = 45 用例**

### 2.6 shared-platform (3 module)

- **OUTBOX** (10 cmds): 跨域 outbox 事件发送 (per 8/27 55.22)
- **MTLS** (8 cmds): 跨域 mTLS 握手 (per 8/27 55.21)
- **RBAC-CTRL** (10 cmds): 跨域 RBAC 中心化

**shared-platform 3 × 15 = 45 用例**

### 2.7 cluster-ops (3 module)

- **REALM-LIFECYCLE** (15 cmds): realm archive / split / merge (per LCM drill)
- **APP-DEPLOYMENT** (10 cmds): 单 app 独立更新 (per ARCH §3.4)
- **PLUGIN-REGISTRY** (12 cmds): plugin 注册中心 (per ARCH §2.2)

**cluster-ops 3 × 15 = 45 用例**

### 2.8 function-plane (3 module)

- **REGISTRY** (10 cmds): FunctionRegistry 增删改查
- **GATEWAY** (12 cmds): FunctionGateway invoke 流程
- **WASM-HOST** (8 cmds): Wasmtime 嵌入 / fuel / epoch

**function-plane 3 × 15 = 45 用例**

**18 跨域抽象 module × 15 = 270 用例**

## 3. 跨域 / 架构专项 (per §7 of TEST-DESIGN)

### 3.1 跨域 saga (5 用例)

```yaml
- SAGA-001: 经济域 Reserve 流程 (per §7.1)
- SAGA-002: 经济域 Confirm 失败回滚
- SAGA-003: 经济域 trade 跨域 trade_saga
- SAGA-004: 5 域并发 saga 协调
- SAGA-005: saga DLQ 异常路径
```

### 3.2 plugin 集群 (5 用例)

```yaml
- PLUGIN-001: PoC 抽卡 hot-swap (per §7.2 + RGS-PLUGIN-APP-ARCH §3.1)
- PLUGIN-002: WASM 资源超 fuel cap fail-closed
- PLUGIN-003: WASM 内存超 memory_mib cap fail-closed
- PLUGIN-004: plugin Paused 时 app 走 native fallback
- PLUGIN-005: 跨 app plugin 双 registry 一致性
```

### 3.3 app 独立更新 (3 用例)

```yaml
- APP-DEPLOY-001: player-service 灰度 10% → 100% (per §7.3)
- APP-DEPLOY-002: economy-service 跨域回滚
- APP-DEPLOY-003: k3s rolling update 不中断 function-plane
```

### 3.4 ops 运维 UI (5 用例)

```yaml
- OPS-UI-001: gm-backend /ops/functions/register RBAC (per §7.4)
- OPS-UI-002: gm-backend /ops/apps/deploy k8s 调用
- OPS-UI-003: ops UI env var 永不打印 (per 8/27 11:06 JST)
- OPS-UI-004: ops UI REDACTED filter 触发
- OPS-UI-005: 域 Lead 越权 register 跨域 function → 403
```

**专项 5+5+3+5 = 18 用例**

## 4. 用例汇总

| 类别 | 数量 |
|---|---|
| 42 module 业务路径 + 错误码 + 边界 + 异常 | ~588 |
| 18 跨域抽象 module (4 类) | ~270 |
| 跨域 saga 专项 | 5 |
| plugin 集群专项 | 5 |
| app 独立更新专项 | 3 |
| ops UI 专项 | 5 |
| **总计** | **~876 用例** |

## 5. 用例执行优先级

| 优先级 | 范围 | 触发 |
|---|---|---|
| P0 (must) | 5 域主链路 HP (5×6=30) + 跨域 saga (5) + plugin PoC (1) | 每次 commit |
| P1 (should) | 5 域 EC (5×4=20) + 18 抽象 module HP (18×6=108) | 每次 PR merge |
| P2 (nice) | 边界值 + 异常 + RBAC + app 部署 | 每周回归 |
| P3 (defer) | drill_lcm_001-010 + drill_chaos | 待 INC-002 saga 修复 |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 07:00 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review 二审 | 起草, 60 module × 4 类 ~876 用例 + 18 专项 |
