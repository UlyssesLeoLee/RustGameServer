# H5 zsyz_client → RGS 完整迁移矩阵

**核心** (per 2026-09-09 15:25 JST Ulysses 拍板): H5 源码完整在 `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_client_h5\temp\quick-scripts\src\assets\Scripts\`。如果每个端口都跟 erlang 一致, 理论可完美迁移.

## 1. 真实规模 (per proto_mate.js)

| 维度 | 数量 | 来源 |
|------|------|------|
| H5 zsyz_client send (cli → srv) | **766 unique cmd** | `temp/quick-scripts/src/assets/Scripts/net/proto_mate.js` |
| H5 zsyz_client recv (srv → cli) | **875 unique cmd** | 同上 |
| shim 已实现真 zsyz cmd | **10** | 1.3% 业务覆盖 |
| shim-internal RGS 测试 | 2 (10400, 11001) | 不计业务覆盖 |
| zsyz_server proto_*.erl | 43 文件 | `zsyz_server/src/proto/proto_101.erl` ~ `proto_239.erl` |
| RGS 5 域 gRPC rpcs | 44 (11+10+5+8+7+3 admin) | `crates/{player,economy,match,social,admin}-service/proto/` |
| H5 源码 mod 模块 | 55 (login/mainui/mail/...) | `Scripts/mod/*` |
| zsyz_client 全资源 | 3025 文件 81.48 MB | `zsyz_client/res/resource/` (含 plist+png+mp3+mp4) |
| Cocos Creator 项目 | 2.3.2 (项目文件+settings+CusEngine) | `zsyz_client_h5/` |

## 2. 端点映射样例 (4 域真实 cmd)

### 2.1 player 域 (player.v1, port 50061)

| shim cmd | proto_mate.js 字段 | RGS RPC |
|----------|-------------------|---------|
| 10101 register | sex:u8 + name:str + career:i16 + playform:str | player.GetPlayer |
| 10300 ping | (empty) | (no RGS, client state) |
| 10301 role_info | (empty) → 25 字段 srv | player.GetPlayer |
| 10309 signature | signature:str | (no RGS, client state) |
| 10315 view_role | rid:u32 + srv_id:str | player + social 联合 |
| 11001 role_list (shim-internal) | (empty) | player.ListPlayers |

### 2.2 economy 域 (economy.v1, port 50062)

| shim cmd | proto_mate.js 字段 | RGS RPC |
|----------|-------------------|---------|
| 10302 assets | (empty) → 68 字节 srv (lev:u16 + 8 u32 + activity:u16 + 8 u32) | economy.GetAccount |
| 11000 partner_list | (empty) | economy.GetAccount |

### 2.3 match 域 (match.v1, port 50063)

| shim cmd | proto_mate.js 字段 | RGS RPC |
|----------|-------------------|---------|
| 10200 map_enter | battle_id:u32 + id:u32 + code:i16 | (match domain placeholder) |
| 10215 move | base_id:u32 + x:i16 + y:i16 + dir:u8 | match.SubmitMove |

### 2.4 social 域 (social.v1, port 50064)

| shim cmd | proto_mate.js 字段 | RGS RPC |
|----------|-------------------|---------|
| 10315 view_role 联合 | rid + srv_id | social.GetGuild |

### 2.5 admin 域 (admin.v1, port 50065)

| shim cmd | proto_mate.js 字段 | RGS RPC |
|----------|-------------------|---------|
| 11001 role_list (shim-internal) | (empty) | admin.QueryAuditLog |

## 3. 4 worker 派工计划 (per 9/9 13:45 JST 拍板 A + 15:25 JST 完整迁移)

按 proto_mate.js send cmd 域分布拆分, 4 worker 并行:

| worker | 域 | send cmd 范围 | 估计 cmd 数 | 工期 |
|--------|-----|--------------|------------|------|
| **w1** | player | 10000-10999 (角色/伙伴) | ~80 | 3-5 天 |
| **w2** | economy | 20000-29999 (背包/邮件/商城) | ~280 | 5-7 天 (最大) |
| **w3** | match | 30000-39999 (战斗/PVP) | ~50 | 2-3 天 |
| **w4** | social | 13000-14999 (工会/好友) | ~200 | 4-5 天 |
| **w5** (追加) | admin/GM | 10000-99999 杂项 GM | ~150 | 3-5 天 |

**总**: 4-5 worker × 758 cmd 缺口, 1-2 周达 100% 业务覆盖

## 4. 真实 H5 渲染 (per 15:25 JST Ulysses 拍板 "应该有完整 H5 源码")

H5 真能 build — 完整 Cocos Creator 2.3.2 project 在 `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_client_h5\`:

```
zsyz_client_h5/
├── project.json                     (Cocos Creator 2.3.2 manifest)
├── settings/
│   ├── project.json                 (设计分辨率 960x640, Native Socket 排除)
│   ├── builder.json
│   └── services.json
├── CusEngine/engine2.0.9/           (Cocos2d-html5 引擎源码)
├── temp/quick-scripts/src/
│   ├── assets/Scripts/              (50+ mod + net + sys, TypeScript 源码)
│   └── project/                     (Cocos Creator 临时项目)
├── templet/                         (engine + bootstrap)
└── wx_obj/                          (微信小游戏编译产物 3.1MB)
```

**Build H5 步骤** (1-2h 一次性):
1. 安装 Cocos Creator 2.3.2 (Cocos 官方下载, 需注册账号)
2. 导入 `zsyz_client_h5/` 作为 Cocos Creator project
3. 导入 `zsyz_client/res/resource/*` (3025 文件 81MB) 作为 assets
4. `cocos compile -p web -m release` → 输出 `build/web-mobile/`
5. 部署到 `D:\zsyz-h5-orig\public\zsyz-real\` 替换 HTML
6. rgs-shim-rust 8001 接收真 zsyz_client 766 cmd

**前置**: 必须先把 shim 100% 业务覆盖 (worker 派工), 否则 H5 启动后大部分 cmd 报 stub.

## 5. 协议字节级不变量 (worker 必守)

1. **字节序 BE** — 所有整数 big-endian (per `GameTcpClient.h`)
2. **字符串** — `len:u32 + bytes`, 无 null 终止 (per `protocol:pack(string, ...)`)
3. **帧** — `len:u32 + cmd:u16 + payload`, `len = 2 + payload.length`
4. **响应** — 严格按 `proto_*.erl pack(srv, Code, ...)` 字段顺序, 不可缺字段
5. **错误码** — `code:u8` 跟 `proto_*.erl` 定义一致 (0=OK, 1+=err)

## 6. 验证方法

### 6.1 协议级 (Node.js, 字节级)
- `bench/proto-test.js` — 17 case 真 zsyz_client 协议模拟, 17/17 passed

### 6.2 资源级 (Playwright, 真 zsyz 资源)
- `tests/05-zsyz-battle-flow.spec.ts` — 10 case 真 zsyz 资源 + 真实 SmartSocket 协议, 10/10 passed, 11 张截图

### 6.3 H5 渲染级 (待 build)
- 完整 Cocos Creator 2.3.2 build, 部署到 web 8083
- zsyz_client H5 启动 → 调 shim 9001 → 真实 RGS 5 域 gRPC → 真数据
- (待 shim 100% 业务覆盖后实施)

## 7. 已知约束

| 约束 | 说明 | 影响 |
|------|------|------|
| zsyz_client H5 binary 不在 | `project.9b183.js` 缺, settings.js 缺 | 需 Cocos Creator 2.3.2 build |
| zsyz_client/res vs H5 资源格式 | res/ 是 executable 格式 (plist/png/mp3), H5 需 Cocos Creator 导入 | 81MB 资源需转换 |
| wx_obj 是微信小游戏 | compileType:"game", libVersion 2.7.3 | 需 WeChat 运行时 |
| shim 业务覆盖 1.3% | 758 cmd 待实现 | 4-5 worker 1-2 周 |

## 8. 路线图

1. ✅ **Phase 1 (v0.3.0-3.2)**: 协议字节级验证 (10 cmd, proto-test 17/17)
2. ✅ **Phase 2 (v0.3.1)**: 10200 srv 5 字段修复 + 文档
3. ✅ **Phase 3 (v0.3.2)**: 战斗场景 6 cmd (10300/10215/10301/10302/10309/10315)
4. **Phase 4 (GAP-A)**: 4-5 worker 并行扩 758 cmd (1-2 周, 100% 业务覆盖)
5. **Phase 5 (H5 build)**: Cocos Creator 2.3.2 build zsyz_client H5, 部署 + 真实渲染 (1-2h build)
6. **Phase 6 (E2E)**: H5 启动 → shim → RGS → 真游戏数据, 拍板

代签: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
