# rgs-shim — zsyz SmartSocket → RGS gRPC shim (PoC v0.1.0)

> **Mavis 接手, 2026-09-09 13:35 JST** — 真 zsyz 客户端 0 改动, 透明转发到 RGS 5 域 gRPC.
> **作者**: Ulysses — Mavis 接手 (per DEC-008) | **审批**: 架构师(Mavis 接手 agent per DEC-008)+自审+2026-09-09 | **修订人**: Ulysses — Mavis 接手
> **依据**: Ulysses 2026-09-09 13:34 JST "写 SmartSocket shim 独立 proxy" 拍板

---

## 1. 这是什么

zsyz 客户端 (zsyz_client C++ cocos2d-x + zsyz_client_h5) 通过 **SmartSocket TCP binary protocol** (默认 `localhost:9001`) 与 Erlang zsyz_server 通信。本项目是一个独立 proxy, 让 zsyz 客户端**不改任何源码**就能连到 RGS 5 域 gRPC server。

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

## 2. 协议解析 (per zsyz_client_core/thirdparty/Libnetwork/GameTcpClient.h)

**字节序**: big-endian (大端, network byte order)

**Frame 格式**:
```
+--------+--------+----------------+
| len    | cmd    | payload...     |
| 32 bit | 16 bit | variable       |
+--------+--------+----------------+
  ^         ^         ^
  |         |         +-- SmartSocket 字段 (Int8/UInt8/Int16/UInt16/Int32/UInt32/Int64/UInt64/String/Byte/Array)
  |         +------------- 协议命令 ID (e.g. 10101 = register, 10102 = enter_server)
  +----------------------- len = byte_size(cmd) + byte_size(payload) (2 + payload)
```

**字段类型** (per SmartSocket.h):
- Int8/UInt8/Int16/UInt16/Int32/UInt32/Int64/UInt64 (big-endian)
- String: 4 bytes (u32 len) + len bytes (no terminator)
- Byte/Array: 4 bytes (u32 len) + len bytes

## 3. 启动

```bash
# 1) 前置: RGS 5 域 binary + rgs-proxy + 5 域 migrations 已就位
#    (per 9/9 12:00 JST 跑通 5/5 域 gRPC HealthCheck)

# 2) 启 shim
cd tools/rgs-shim
node shim.js
# 输出: [shim] RGS SmartSocket shim v0.1.0 listening on 0.0.0.0:9001

# 3) zsyz 客户端 → tcp://localhost:9001 (原 zsyz_server 端口)
```

环境变量:
- `SHIM_PORT` (默认 9001)
- `RGS_PROXY` (默认 http://127.0.0.1:8084)

## 4. 已实现 cmd

| Cmd | 方向 | Schema | RGS 映射 |
|---|---|---|---|
| **10101** | cli | `{sex:u8, name:str, career:i16, playform:str}` (注册) | 调 `player.GetPlayer({id:'11111111-...'})` 拿真玩家数据 |
| **10101** | srv | `{code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32}` | rid = uuid 前 8 hex 解析为 u32, name = RGS display_name |
| **10102** | cli | `{rid:u32, srv_id:str}` (进入服务器) | 直接通过 (RGS 无对应) |
| **10102** | srv | `{code:u8, msg:str, timestamp:u32, world_lev:u16}` | world_lev=52 (RGS 测试值) |
| **10103** | cli | 同 10102 (alias) | 同 10102 |
| **其它** | 任意 | stub (返空 payload) | RGS 业务未实装时 |

## 5. 测试

```bash
# test-client.js 模拟 zsyz 客户端
node test-client.js
# 输出:
#   [test] → 10101 register
#   [test] ← 10101: code=0 msg="OK (via RGS)" rid=286331153 sid="rgs-uat-1" name="MavisHero" regTime=1788928653
#   [test] → 10102 enter_server
#   [test] ← 10102: code=0 msg="OK (RGS server ready)" ts=1788928654 worldLev=52
```

## 6. 已知缺口 (per 13:35 JST 缺标比错标)

- **GAP-A**: 只覆盖 10101 (register) + 10102/10103 (enter_server). 战斗/聊天/工会等 cmd (~200+ 个 per proto_2xx/3xx) **RGS 业务全 stub**, shim 直接返空 payload. 客户端走到一半会卡. 修法: 逐个 cmd 加 RGS 业务映射 (3-5 天/域).
- **GAP-B**: RGS player.GetPlayer 用硬编码 UUID `11111111-1111-1111-1111-111111111111` (MavisHero). 没接 zsyz 账号体系, 任意账号都拿这个 UUID 的数据. 修法: 解析 cmd 10101 的 name → 查 RGS player table (待 RGS player service 加 ListPlayers RPC).
- **GAP-C**: 没压测. 单 client OK, 100+ 并发 client 待验.
- **GAP-D**: 没加密. zsyz_server 真实有 XOR/TEA 加密 (per zsyz_srv_web/protocol/), shim 当前是明文. 若 zsyz 客户端发的是密文会解析失败. 修法: 抓 zsyz 客户端 handshake 头几字节验证, 必要时加解密层.
- **GAP-E**: 没 reconnect/disconnect 处理. zsyz 客户端重连机制依赖 zsyz_server 的 session 管理, shim 无 session 概念, 短断重连后会丢登录态.

## 7. 协议参考

- Frame 格式: `zsyz_client_core/frameworks/game_core/thirdparty/Libnetwork/GameTcpClient.h`
- 字段类型: `zsyz_client_core/frameworks/game_core/net/SmartSocket.h`
- Cmd 定义: `zsyz_server/src/proto/proto_101.erl` (login), `proto_102-236.erl` (其它业务)
- 客户端实现: `zsyz_client_core/frameworks/game_core/net/SmartSocket.cpp` (17KB)

## 8. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1.0 | 2026-09-09 13:35 | Ulysses — Mavis 接手 | PoC: 10101 (register) + 10102/10103 (enter_server) · Node.js 0 依赖 · test-client.js 验证 2 帧 OK · 5 GAP 已知 |
