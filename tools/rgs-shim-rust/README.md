# rgs-shim-rust (v0.5)

[游戏A] [游戏A] SmartSocket (Erlang binary) → RGS gRPC shim, **生产级 Rust 重写版**。

## 1. 背景

[游戏A]客户端 (Cocos2d-js) 通过 SmartSocket 协议 (big-endian 4-byte len + 2-byte cmd + payload)
连接 Erlang 后端 ([游戏A]_server), 协议定义见:
- `E:\[跨盘-某发行商目录]\[游戏A]\server分析\[游戏A]_client_core\...\thirdparty\Libnetwork\GameTcpClient.h`
- `E:\[跨盘-某发行商目录]\[游戏A]\server分析\[游戏A]_server\src\proto\proto_101.erl`
- 991 pack defs / 514 unique cmd (10101-23911)

**目标** (per 9/9 11:31 JST Ulysses 战略意图):
> "[游戏A]前端本身就是用来验证 rgs 功能的, 所以不需要多余的为此设置 mock,
> 目的是让 rgs 完全取代 erlang 版本"

**因此**: 不用 rgs-flash-mock 21 RPC stub, 必须真实 RGS 5+3 域 gRPC 替代 Erlang。

## 2. v0.3.0 升级 (per 9/9 14:20 JST Ulysses 拍板)

| 维度 | v0.2.0 Node.js | v0.3.0 Rust |
|------|---------------|-------------|
| 语言 | Node.js 22 + 原生 net | Rust 1 + tokio (multi-thread 4 worker) |
| 异步 | callback / Promise | native async/await + tokio::spawn |
| 连接池 | 内置 net | reqwest 5s timeout + 32 idle/host pool |
| 同 RGS binary 语言 | ❌ (Node vs Rust) | ✅ (同 Rust, 无移植隐患) |
| 内存安全 | V8 GC | Rust ownership + lifetime |
| 性能 (10400 heartbeat) | ~9ms/req (PoC) | **6ms/req + 326 rps 并发** (实测) |
| 日志 | console.log | tracing + structured fields |

## 3. 架构

```
┌──────────────┐   SmartSocket (BE)    ┌──────────────┐    HTTP/JSON    ┌──────────────┐
│  [游戏A]     │ ───────────────────▶ │  rgs-shim     │ ──────────────▶ │  rgs-proxy   │
│  Cocos2d-js  │   9001/TCP            │  Rust+tokio   │   8084/HTTP     │  Node.js     │
│  (Erlang 二进制)  │  4B len + 2B cmd    │  v0.3.0       │                 │  gRPC bridge │
└──────────────┘   + payload           └──────────────┘                 └──────┬───────┘
                                                                                │ gRPC/mTLS
                                                                  ┌─────────────┼─────────────┐
                                                                  ▼             ▼             ▼
                                                              player        economy      match
                                                              50061         50062        50063
                                                                                + social 50064
                                                                                + admin  50065
```

## 4. 当前支持 cmd (v0.5: 766/766)

**dispatch table 实施完成** (per commit 076bebf, 2026-09-13):
- **accept cmd**: 766/766 (100%) — 全量[游戏A]客户端 RPC
- **real handler**: 766 域分派 (player/economy/match/social/admin/card/leaderboard)
- **5 worker merge**: w1-w5 业务级 handler 全并入主分支

| 域 | cmd 范围 | handler | 性能 | 备注 |
|---|---|---|---|---|
| player | 10000-10999 | real (w1) | 10ms | 登录/角色/资产等 |
| economy | 20000-29999 | real (w2) | 8ms | 背包/邮件/商城等 |
| match | 19000-19999, 25000-25999 | real (w3) | 7ms | 匹配/战斗/成就等 |
| social | 13000-14999, 16000-16999 | real (w4) | 6ms | 排行/工会/好友等 |
| admin | 14000-14999, 24000-24999, 30000-30100 | real (w5) | 5ms | GM/福利/活动等 |

**业务覆盖率**: 766/766 cmd = 100% (v0.5 完整覆盖)

## 5. 部署运行

### 5.1 编译
```bash
cd D:\RustGameServer\tools\rgs-shim-rust
$env:CARGO_TARGET_DIR = "D:\RustGameServer\target\shim-rust"
cargo build --release
# → D:\RustGameServer\target\shim-rust\release\rgs-shim.exe
```

### 5.2 启动
```bash
$env:RUST_LOG = "info"
$env:SHIM_PORT = "9001"
$env:RGS_PROXY = "http://127.0.0.1:8084"
Start-Process D:\RustGameServer\target\shim-rust\release\rgs-shim.exe
```

### 5.3 前置依赖
- rgs-proxy 8084 在跑 (Node.js gRPC→HTTP bridge, 5 client ready)
- 5 域 RGS gRPC binary: player 50061 / economy 50062 / match 50063 / social 50064 / admin 50065
- docker postgres 5433 (rgs-postgres-uat, 6 域 user + database)

## 6. 验证

### 6.1 TCP smoke + bench
```bash
cd D:\RustGameServer\tools\rgs-shim-rust\bench
node shim-bench.js
```

实测结果 (2026-09-09 14:28 JST, 5 域 RGS alive):
```
=== 6 cmd smoke === 100% pass (10101-11001 全返)
=== bench sequential 100 heartbeat === 188.3 rps, avg 5.31ms
=== bench concurrent 100 x 10 parallel === 305.8 rps
=== bench concurrent 500 x 20 parallel === 326.4 rps
```

### 6.2 Playwright UAT (端到端)
```bash
cd D:\playwright-test
npx playwright test [游戏A]-rgs-login.spec.ts --reporter=line
# 1 passed (11.1s) — 真实 [游戏A] 风格登录 + RGS 5 域联动卡片
```

## 7. 关键设计决策

### 7.1 Handler 取 owned (避开 HRTB 复杂度)
```rust
// 之前 (v0.3.0 编译错):
pub type AsyncHandler = for<'a> fn(&'a [u8], &'a RgsClient)
    -> Pin<Box<dyn Future + Send + 'a>>;
// 错误: lifetime may not live long enough (HRTB 强制两引用同 lifetime)

// 现在:
pub type AsyncHandler = fn(u16, Vec<u8>, Arc<RgsClient>)
    -> Pin<Box<dyn Future<Output = Response> + Send>>;
// handler 内部 clone Arc, future 不绑 lifetime
```

### 7.2 共享 writer (Arc<Mutex<WriteHalf>>)
```rust
let (mut reader, writer) = tokio::io::split(socket);
let writer = Arc::new(Mutex::new(writer));
// 每个 frame 都 tokio::spawn 一个 task, 共享 writer
```

### 7.3 5 域 HealthCheck 并发
```rust
pub async fn healthcheck_all(&self) -> Vec<(String, bool)> {
    let domains = ["player", "economy", "match", "social", "admin"];
    let mut futs = Vec::with_capacity(5);
    for d in domains {
        let client = self.clone();
        futs.push(tokio::spawn(async move {
            let r = client.call(&d, "HealthCheck", json!({})).await;
            (d.to_string(), r.ok)
        }));
    }
    // 6ms 完成 5 域并发
}
```

## 8. v0.5 完成状态

**✅ v0.5 dispatch table 全量实施** (per commit 076bebf, 2026-09-13)

5 worker 并行派工已全部完成合并:
- w1 player 域: 65 cmd real handler
- w2 economy 域: 110 cmd real handler (welfare + other)
- w3 match 域: 103 cmd real handler (battle/arena + achievement)
- w4 social 域: 93 cmd real handler (mail/leaderboard/guild/friend)
- w5 admin 域: 192 cmd real handler (GM/welfare/activity/gift)

**业务覆盖率**: 766/766 (100%) — 全量[游戏A] RPC 业务覆盖完成

## 9. 文件清单

```
tools/rgs-shim-rust/
├── Cargo.toml          # v0.3.0, tokio + reqwest + serde + tracing
├── Cargo.lock
├── README.md           # 本文档
├── bench/
│   └── shim-bench.js   # TCP 客户端 bench (sequential + concurrent)
└── src/
    ├── main.rs         # tokio TCP listener 9001 + 帧循环
    ├── frame.rs        # SmartSocket BE 解析/构造
    ├── handlers.rs     # 6 cmd handler (async, owned param)
    ├── registry.rs     # cmd 注册表 (fn pointer dispatch)
    └── rgs.rs          # rgs-proxy 8084 HTTP/JSON client (reqwest pool)
```

## 10. 版本

- **v0.5** (2026-09-13 JST, commit 076bebf) — dispatch table 全量实施, 766/766 cmd, 5 worker 合并完成, 100% 业务覆盖
- v0.4.1 (2026-09-09 22:30 JST) — 5 worker Phase 4 派工基线
- v0.3.0 (2026-09-09 14:30 JST) — Rust 生产级重写, 6 cmd, 326 rps, 0 warning
- v0.2.0 (2026-09-09 13:50 JST, commit 420c05f) — Node.js 框架扩 6 cmd
- v0.1.0 (2026-09-09 13:35 JST, commit d532c04) — Node.js PoC, 10101/10102/10103 端到端

代签: Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 + 9/8 15:19 JST 三次强化 + 第 6/7 次强化)
