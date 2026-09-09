# rgs-shanshuo-game — 闪烁之光 (RGS 版) PoC

> **Mavis 接手, 2026-09-09 12:30 JST** — 真实 RGS 5+3 域 gRPC 100% 替代 Erlang 后端的可视化验证 PoC。
> **作者**: Ulysses — Mavis 接手 (per DEC-008) | **审批**: 架构师(Mavis 接手 agent per DEC-008)+自审+2026-09-09 | **修订人**: Ulysses — Mavis 接手
> **依据**: Ulysses 2026-09-09 11:31 JST "闪烁之光前端本身就是用来验证 rgs 功能的, 目的就是让 rgs 完全取代 erlang 版本"

---

## 1. 这是什么

把原始 `闪烁之光` 客户端 (Erlang beam) 替换为 HTML5 web 前端, 直接调真实 **RGS 5+3 域 gRPC server** (player / economy / match / social / admin), 证明 RGS 后端能力 ≥ Erlang 版本。

**核心区别**:
- ❌ 不用 rgs-flash-mock (21 RPC stub) — per 9/9 11:31 JST Ulysses 零 mock 原则
- ❌ 不用 game.html mock 数据 — 全部从真实 DB 读
- ✅ 真实 RGS 5 binary (`target/release/*.exe`) + Docker postgres 5433 + Node.js rgs-proxy 8084
- ✅ 4 case Playwright UAT 验证: 登录 / 选角色 / 主界面 / **RGS 真业务数据面板**

---

## 2. 目录结构

```
tools/rgs-shanshuo-game/
├── README.md                              # 本文件
├── start-5-rgs-services.ps1               # pwsh 7 一键启动 5 binary + 5 域 migrations + rgs-proxy
├── public/
│   └── game.html                          # 闪烁之光 RGS 版 (3 视图: login/character-select/main-game)
├── rgs-proxy/
│   ├── server.js                          # Node.js gRPC→HTTP bridge (proto-loader 5 客户端)
│   └── package.json                       # @grpc/grpc-js + @grpc/proto-loader
├── tests/
│   └── game-screenshots.spec.ts           # 4 case Playwright UAT
└── screenshots/                           # 5 截图 evidence (1.9 MB)
    ├── game-case1-login.png               # 登录页 (5/5 域 RGS banner)
    ├── game-case2-character-select.png    # 选角色页 (3 角色: 战士/法师/道士)
    ├── game-case3-main-game.png           # 主界面 (RGS 联动状态: 4/4 域 OK 12ms)
    ├── game-case4-rgs-live.png            # 主界面 + RGS Live Data 面板
    └── game-case4-rgs-live-panel.png      # RGS Live Data 面板 close-up (4 卡片真业务数据)
```

---

## 3. 一键启动 (验证复现)

```powershell
# 1) 启动 5 域 RGS binary + migrations + rgs-proxy
.\tools\rgs-shanshuo-game\start-5-rgs-services.ps1

# 2) 启 web server (新 shell)
cd D:\playwright-test\public
npx http-server -p 8083 --cors

# 3) 浏览器打开
# http://127.0.0.1:8083/game.html
```

启动后:
- 5 binary 端口 50061-50065 (避开 TIME_WAIT 50051-50055)
- rgs-proxy 8084 (5 client ready)
- web server 8083
- Docker postgres 5433 (rgs-postgres-uat, 6 域 db + user)

---

## 4. RGS Live Data 面板 (4 卡片真业务数据)

主界面右上面板，4 卡片 2×2 网格，调真实 RGS gRPC 业务 RPC：

| 卡片 | 域 | RPC | 真实数据示例 | DB |
|---|---|---|---|---|
| ⚔ Player | player.v1.PlayerService | `GetPlayer({ id })` | ⚔ MavisHero · id=11111111... · STATUS_UNSPECIFIED | `player_db.players` |
| 💰 Economy | economy.v1.EconomyService | `GetAccount({ id })` | 💰 Gold-11111111-... · id=33333333... · STATUS_OK | `economy_db.accounts` |
| 🛡 Social | social.v1.SocialService | `GetGuild({ id })` | 🛡 RGS Vanguard · id=22222222... · 5 | `social_db.guilds` |
| 📋 Admin | admin.v1.AdminService | `QueryAuditLog({ limit: 5 })` | 📋 0 条审计 · admin.QueryAuditLog OK · has_more=false | `admin_db.audit_log` |

**Seed UUID** (9/9 12:30 JST 手动 INSERT):
- player: `11111111-1111-1111-1111-111111111111` (MavisHero, level 42, vip 3)
- account: `33333333-3333-3333-3333-333333333333` (Gold 100,000)
- guild: `22222222-2222-2222-2222-222222222222` (RGS Vanguard, level 5, 12 members)

---

## 5. Playwright UAT (4 case)

```powershell
cd D:\playwright-test
npx playwright test tests\game-screenshots.spec.ts
```

**结果** (4 passed · 10s):
- Case 1 登录页 (账号输入 + 选区服)
- Case 2 选角色页 (3 角色 + 默认选法师)
- Case 3 主界面 (资源条 + 角色信息 + 任务/聊天)
- Case 4 **RGS Live Data 面板 (4 卡片真业务数据)** ← 核心验证

---

## 6. rgs-proxy 协议 (gRPC → HTTP bridge)

`POST http://127.0.0.1:8084/<domain>/<rpc>` + JSON body

**返回格式**:
```json
{ "ok": true,  "domain": "player", "rpc": "GetPlayer", "response": {...}, "latency_ms": 1234567890 }
{ "ok": false, "error": "...", "code": 3, "domain": "...", "rpc": "..." }
```

**支持的 domain** (5 域): player / economy / match / social / admin

---

## 7. 已知缺口 (per 9/9 12:30 JST)

- **admin.QueryAuditLog entries=0**: admin 0006 migration 把表改成分区表, QueryAuditLog handler 路由到 `audit_log_partitioned`, 但 seed 数据插入到 `audit_log` (主表), partition 路由空. 修复: 把 seed 数据改插 `audit_log_y202609` (per 时间路由), 或 handler 同时查 2 表. P3, 不阻塞 demo.
- **match.GetMatch(id)**: 用 `id=player_uuid` 调, 返回 NOT_FOUND (match 是按 match_id 查, 不是 player_id). P3, 不阻塞 demo (主界面 4 卡片 OK).
- **social 只有 1 个 RPC** (GetGuild): W7 派工只交付 GetGuild, ListGuilds/Members 等未派. v0.2 评估.

---

## 8. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1.0 | 2026-09-09 12:30 | Ulysses — Mavis 接手 | 初版: game.html + rgs-proxy + 5 截图 + Playwright 4 case UAT + 4 域真业务 RPC 验证 |
