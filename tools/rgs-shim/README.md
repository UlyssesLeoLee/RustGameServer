# rgs-shim — zsyz SmartSocket → RGS gRPC shim v0.2.0

> **Mavis 接手, 2026-09-09 13:50 JST** — v0.2 框架扩展: cmdRegistry 模式 + 6 cmd 跑通.
> **作者**: Ulysses — Mavis 接手 (per DEC-008) | **审批**: 架构师(Mavis 接手 agent per DEC-008)+自审+2026-09-09 | **修订人**: Ulysses — Mavis 接手
> **依据**: Ulysses 2026-09-09 13:34 JST "A 写 SmartSocket shim 独立 proxy" + 13:45 JST "扩 cmd 覆盖"

---

## 1. 这是什么

zsyz 客户端 (zsyz_client C++ cocos2d-x + zsyz_client_h5) 通过 **SmartSocket TCP binary protocol** (默认 `localhost:9001`) 与 Erlang zsyz_server 通信。本项目 (v0.2) 让 zsyz 客户端**不改任何源码**就能连 RGS 5 域 gRPC server。

```
zsyz 客户端 (C++ / H5 / iOS / Android)
  ↓ TCP binary (SmartSocket 协议, 端口 9001)
rgs-shim (本项目, Node.js, 0 第三方依赖)
  ↓ HTTP/JSON (rgs-proxy 协议)
rgs-proxy (端口 8084)
  ↓ gRPC
5 binary (player 50061 / economy 50062 / match 50063 / social 50064 / admin 50065)
  ↓
Docker postgres 5433 (player_db / economy_db / ...)
```

## 2. v0.1 → v0.2 变化

| 项 | v0.1 | v0.2 |
|---|---|---|
| 总 cmd | 3 (10101/10102/10103) | **6** (+10200/10400/11001) |
| 框架 | 直接 dispatch 函数 | **cmdRegistry 模式** (worker 加 cmd 只需 +表项 + 写 handler) |
| 心跳 | 无 | **10400 heartbeat → RGS 5 域 HealthCheck 7ms** |
| 角色列表 | 无 | **11001 role_list → RGS player.GetPlayer/ListPlayers** |
| 地图 | 无 | **10200 map_enter** |
| 业务覆盖率 | 0.6% (3/514 cmd) | **1.2%** (6/514) — 框架就位, 业务待 worker 扩 |

## 3. 协议 (per GameTcpClient.h + SmartSocket.h)

- **字节序**: big-endian (网络字节序)
- **Frame**: `| len:32 BE | cmd:16 BE | payload |` (len = 2 + payload)
- **字段**: Int8/UInt8/Int16/UInt16/Int32/UInt32/Int64/UInt64 (BE) + String (u32 len + bytes) + Byte/Array

## 4. 已实现 cmd (6 个)

| Cmd | 方向 | Schema | RGS 映射 | 验证 (per 13:50 JST) |
|---|---|---|---|---|
| **10101** | cli | `{sex:u8, name:str, career:i16, playform:str}` | player.GetPlayer(MavisHero) | ✅ code=0, name="MavisHero" |
| **10101** | srv | `{code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32}` | rid=uuid前8hex, name=display_name | ✅ |
| **10102** | cli | `{rid:u32, srv_id:str}` | 直通 | ✅ |
| **10102** | srv | `{code:u8, msg:str, timestamp:u32, world_lev:u16}` | world_lev=52 | ✅ |
| **10103** | cli | 同 10102 (alias) | 同 10102 | ✅ |
| **10200** | cli | `{battle_id:u32, id:u32, code:i16}` (map_enter) | 直通 stub | ✅ code=0 |
| **10200** | srv | `{code:u8, msg:str}` | "OK (RGS map)" | ✅ |
| **10400** | cli | `(empty)` (heartbeat) | 5 域并发 HealthCheck | ✅ 5/5 in 7ms |
| **10400** | srv | `{code:u8, msg:str, ok_count:u8, total:u8}` | "OK N/5 RGS 域 in Xms" | ✅ |
| **11001** | cli | `(empty)` (role_list) | player.ListPlayers (降级 GetPlayer) | ✅ MavisHero(lv49) |
| **11001** | srv | `{code:u8, msg:str, count:u8, [name:str, level:u8]}` | RGS 真数据 | ✅ |
| **其它** | 任意 | stub 返空 payload | (TODO worker 扩) | |

## 5. 启动

```bash
# 1) 前置: RGS 5 域 binary + rgs-proxy + 5 域 migrations 已就位
# 2) 启 shim
cd tools/rgs-shim
node shim.js
# 输出: [shim] RGS SmartSocket shim v0.2.0 listening on 0.0.0.0:9001
#       [shim] Registered cmds: 10101, 10102, 10103, 10200, 10400, 11001
#       [shim] Total: 6 cmds (514 unique in zsyz_server, 508 TODO)

# 3) zsyz 客户端 → tcp://localhost:9001 (原 zsyz_server 端口)
```

环境变量: `SHIM_PORT` (默认 9001), `RGS_PROXY` (默认 http://127.0.0.1:8084)

## 6. 测试 (6 cmd 端到端验证 OK)

```bash
node test-client.js
# 输出:
#   [test] → 10101 register
#   [test] ← 10101: code=0 msg="OK (via RGS player.GetPlayer)" rid=0x11111111 sid="rgs-uat-1" name="MavisHero" regTime=1788929022
#   [test] → 10102 enter_server
#   [test] ← 10102: code=0 msg="OK (RGS server ready)" ts=1788929022 worldLev=52
#   [test] → 10103 enter_server (alias)
#   [test] ← 10102: code=0 msg="OK (RGS server ready)" ts=1788929022 worldLev=52
#   [test] → 10200 map_enter
#   [test] ← 10200: code=0 msg="OK (RGS map)"
#   [test] → 10400 heartbeat
#   [test] ← 10400 heartbeat: code=0 msg="OK 5/5 RGS 域 in 7ms" ok=5/5
#   [test] → 11001 role_list
#   [test] ← 11001 role_list: code=0 msg="OK (RGS) 1 players" players=[MavisHero(lv49)]
#   [test] closed (6 frames)
```

## 7. 已知缺口 (per 缺标比错标, 5 减 1 = 4)

- ~~GAP-A: 只覆盖 3 cmd~~ → **v0.2 扩到 6 cmd (10101-10103 + 10200 + 10400 + 11001)**
- **GAP-A2 (新)**: 还差 508 个 cmd (514 unique - 6 done). 派 4 worker 并行扩:
  - player 域 worker: 20000-29999 (战斗/技能) ~150 cmd
  - economy 域 worker: 30000-39999 (聊天/邮件/好友) ~120 cmd
  - social 域 worker: 40000-49999 (工会/聊天) ~80 cmd
  - admin 域 worker: 50000+ (GM/审计) ~158 cmd
  - **预计**: 3-5 天/worker = 1-2 周 4 worker 并行
- **GAP-B**: RGS player.GetPlayer 硬编码 UUID, 没接 zsyz 账号体系. 修法: 解析 10101 name → ListPlayers 查 RGS player table.
- **GAP-C**: 没压测. 100+ 并发待验.
- **GAP-D**: 没加密. zsyz_server 真实有 XOR/TEA 加密, shim 当前明文.
- **GAP-E**: 没 session 管理. 短断重连丢登录态.

## 8. worker 接入指南

每个 worker 扩自己域 cmd:
```js
// 1) 在 shim.js 加 handler 函数
async function handleMyCmdCli(payload) {
  // 1. 解析 payload
  let o = 0;
  const myField = beU32(payload, o); o += 4;
  
  // 2. 调 RGS
  const rgs = await rgsCall('player', 'GetMyData', { id: '11111111-...' });
  
  // 3. 包回 srv payload
  return { cmd: 20001, payload: Buffer.concat([wU8(0), wStr('OK'), wU32(rgs.response.id)]) };
}

// 2) 在 CMD_REGISTRY 加表项
const CMD_REGISTRY = {
  ...
  20001: handleMyCmdCli,
  20002: handleMyCmdSrv, // 如果 cli 和 srv 不同
  ...
};
```

## 9. 协议参考

- Frame 格式: `zsyz_client_core/frameworks/game_core/thirdparty/Libnetwork/GameTcpClient.h`
- 字段类型: `zsyz_client_core/frameworks/game_core/net/SmartSocket.h`
- Cmd 定义: `zsyz_server/src/proto/proto_101-236.erl` (101 = login, 102 = 地图, 103 = 战斗, ...)
- 完整 cmd 目录: 991 pack defs / 514 unique cmd (10101-23911), 跑 `python zsyz-cmd-catalog.py` 看

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1.0 | 2026-09-09 13:35 | Ulysses — Mavis 接手 | PoC: 10101/10102/10103 · 2 帧验证 |
| v0.2.0 | 2026-09-09 13:50 | Ulysses — Mavis 接手 | 框架重构: cmdRegistry + +10200 (map) + 10400 (heartbeat → RGS 5 域 7ms) + 11001 (role_list) · 6 帧验证 |
