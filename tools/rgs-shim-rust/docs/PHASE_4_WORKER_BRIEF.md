# Phase 4 Worker 派工 Brief (per 2026-09-09 16:30 JST Mavis 派工)

**目标**: 5 worker 并行扩 rgs-shim-rust 从 10/766 real cmd → 700+/766 real cmd (90%+ 业务覆盖)
**当前状态**: shim v0.4.0 已接受全 766 send cmd (real handler 10, stub 756)

## 1. 工作环境 (per worker)

```bash
# worktree (per worker)
cd D:\RustGameServer
git worktree add D:\rgs-shim-w<N> -b w<N>/shim main

# 编译
$env:CARGO_TARGET_DIR = "D:\RustGameServer\target\shim-w<N>"
cargo build --release
# → D:\RustGameServer\target\shim-w<N>\release\rgs-shim.exe (10-30s 增量)

# 启动
$env:SHIM_PORT = "900<N>"      # w1=9001, w2=9002, ..., w5=9005
$env:RGS_PROXY = "http://127.0.0.1:8084"
Start-Process D:\RustGameServer\target\shim-w<N>\release\rgs-shim.exe
```

## 2. 任务协议 (per worker)

### 2.1 cmd 来源
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_client_h5\temp\quick-scripts\src\assets\Scripts\net\proto_mate.js` (H5 客户端字段顺序)
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\proto\proto_<N>.erl` (Erlang 服务端字段定义, 43 文件)
- `D:\RustGameServer\tools\rgs-shim-rust\src\handlers.rs` (现有 10 real handler 作参考)

### 2.2 实现流程
1. **看 proto_<N>.erl** (per cmd 范围) — 字段类型 + 顺序
2. **看 proto_mate.js** (per cmd) — cli 字段顺序
3. **写 handler** 在 `handlers.rs` (仿 handle_register / handle_move 风格)
4. **registry.rs**: 把 stub-XXX 替换为新 handler (per 域命名)
5. **编译 + 测**: `cargo build --release` + 写 Node.js 字节级测试

### 2.3 字节级不变量 (必守)
1. **字节序 BE** (所有整数 big-endian, per `GameTcpClient.h`)
2. **字符串**: `len:u32 + bytes` (无 null 终止)
3. **帧格式**: `len:u32 + cmd:u16 + payload`, `len = 2 + payload.length`
4. **响应字段顺序**: 严格按 `proto_*.erl pack(srv, ...)`
5. **错误码**: `code:u8` 0=OK, 1+=err (per proto_*.erl)

## 3. 5 worker 派工 (按域 + cmd 范围)

### 3.1 w1: player 域 (65 cmd, 10000-10999)
```
来源: proto_101.erl (login) + proto_103.erl (role) + proto_104.erl (quest) + proto_105.erl (role_lev)
难度: ⭐⭐ (基础, 大部分有现成模板)
```
| cmd | 名称 | 备注 |
|-----|------|------|
| 10101-10103 | register/enter_server (已) | v0.3.0 |
| 10200/10215 | map_enter/move (已) | v0.3.1-3.2 |
| 10300-10399 | 角色属性/资源/签名/装备 (已部分) | v0.3.2 + 扩 |
| 10500-10599 | 角色成长/升级/转职 (新增 ~30) | |
| 10800-10899 | 英雄/伙伴相关 (新增 ~10) | |

### 3.2 w2: economy 域 (110 cmd, 20000-29999)
```
来源: proto_200.erl + proto_202.erl + proto_205.erl + proto_210.erl
难度: ⭐⭐⭐ (中等, 字段多)
```
| cmd | 名称 |
|-----|------|
| 10302 | assets (已) |
| 20000-20099 | 战斗/HP/能量 (新增 ~15) |
| 20200-20299 | 战斗详细 (回合/技能/伤害) (新增 ~30) |
| 20500-20599 | 副本/关卡 (新增 ~30) |
| 21000-21099 | 主城/活动 (新增 ~20) |

### 3.3 w3: battle 域 (103 cmd, 19000-19999 + 25000-25999)
```
来源: proto_108.erl (battle) + proto_110.erl (partner) + proto_127.erl (mall) + proto_129.erl (vip)
难度: ⭐⭐⭐⭐ (复杂, 战斗逻辑)
```
| cmd | 名称 |
|-----|------|
| 10215 | move (已) |
| 19000-19999 | 战斗前置/匹配 (新增 ~30) |
| 25000-25999 | 主城/任务/成就 (新增 ~50) |
| 19800-19899 | 战斗结果 (新增 ~10) |

### 3.4 w4: social 域 (93 cmd, 13000-14999 + 16000-16999)
```
来源: proto_130.erl + proto_133.erl + proto_134.erl + proto_135.erl + proto_166.erl + proto_168.erl
难度: ⭐⭐⭐ (公会逻辑)
```
| cmd | 名称 |
|-----|------|
| 13000-13099 | 邮件/聊天 (新增 ~20) |
| 13300-13399 | 排行榜 (新增 ~20) |
| 13400-13499 | 任务 (新增 ~20) |
| 16600-16699 | 公会 (新增 ~30) |
| 10315 | view_role (已, 联合 player+social) |

### 3.5 w5: admin/GM 域 (9 cmd + 73 cmd mail + 110 cmd welfare)
```
来源: proto_141.erl + proto_164.erl + proto_200.erl + proto_235.erl
难度: ⭐⭐ (简单, 多数返空)
```
| cmd | 名称 |
|-----|------|
| 14000-14999 | GM 命令 (9 cmd) |
| 30000-30100 | 杂项 (5 cmd) |
| 24000-24999 | 福利/活动 (~50 cmd) |

## 4. 测试方法 (per cmd)

### 4.1 字节级 Node.js 测试 (必跑)
```js
const net = require('net');
function makeFrame(cmd, payload) { /* BE frame */ }
function parseFrame(buf) { /* extract len, cmd, payload */ }
async function sendFrame(cmd, payload) { /* return parsed response */ }
const r = await sendFrame(10101, payload);
expect(r.cmd).toBe(10101);
expect(r.payload).toMatch(...);  // 字段检查
```

### 4.2 proto-test.js 模板 (per worker)
- 复制 `bench/proto-test.js` 改 cmd 列表
- 加新 cmd 的字段解析 (per proto_*.erl)
- 期望 17/17 passed (现有 17 + 新增 N)

### 4.3 性能测试 (并发 50)
- `concurrency = 50` 测每 cmd 50 帧, 验证 rps

## 5. DoD (Definition of Done)

| 维度 | 标准 |
|------|------|
| 代码 | `cargo build --release` 0 error 0 warning |
| 协议 | Node.js 字节级测试 100% 通过 (字段顺序 + 类型) |
| 性能 | 并发 50 rps >= 100 (单 cmd) |
| 文档 | proto_*.erl 字段表 + proto_mate.js 字段顺序 一致 |
| 提交 | per worker 1+ commit, 推送 origin |

## 6. 协调

- 每日 17:00 JST 同步进度 (Mavis 主持)
- cmd 编号冲突时优先真 handler, stub fallback
- 新 RGS gRPC RPC 需先在 `crates/<domain>-service/proto/` 加 proto + 实现
- shim 启动时 real handler 优先, 未注册的 cmd 自动 stub

## 7. 已知 gap (Mavis 2026-09-09 16:30 派工时)

| 缺 | 状态 | 备注 |
|----|------|------|
| 10400/11001 真 zsyz cmd | 不是真 zsyz cmd, 是 shim-internal | 10400=quest_list, 11001=partner_list per Erlang |
| 758 cmd stub | 756/758 已 stub, 2 跟 real 重叠 (10400/11001) | 后续 worker 扩时跳过 |
| H5 binary 缺 | 需 Cocos Creator 2.3.2 build (1-2h) | 阻塞 zsyz_client 真实渲染测试 |

代签: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
