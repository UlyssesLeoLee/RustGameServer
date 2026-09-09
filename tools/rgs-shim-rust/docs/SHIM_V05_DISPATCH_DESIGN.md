# shim-rust v0.5 Dispatch Table 设计 (per 9/9 19:38 JST Phase 4 派工后)

**代签**: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
**关联 commit**: (待 5 worker 业务 handler 落地后, commit b2744a6 + 后续 5 worker merge)
**时间**: 2026-09-09 19:50 JST

---

## 1. 现状 (shim v0.4.1 per 9/9 16:35 JST commit f8204f1)

| 指标 | 值 |
|---|---|
| 接受 send cmd | 766/766 (100%) |
| real handler | 12 (register/enter_server/map_enter/move/role_info/assets/signature/view_role/heartbeat/role_list/ping + 内部) |
| stub handler | 754 (auto-gen by `tools/rgs-shim-rust/bench/generate-stub-registry.js` from `E:\...\proto_mate.js`) |
| batch-stub-test | 766/766 passed, 1099 rps |
| shim-internal cmd | 2 (10400 quest_list / 11001 partner_list, 不在 erlang 766 范围) |

**问题**: stub handler 接受 cmd 但返回 0 数据, shim 业务覆盖 1.9%. RGS 真实业务无法跑通.

## 2. Phase 4 派工 (per 9/9 19:32 JST Ulysses 拍板选项 A)

5 worker 派工, 改 `tools/rgs-shim-rust/src/handlers.rs` + `registry.rs`:
- w1: cmd 10000-10999 (player 65 cmd) — bg_c7411c1e
- w2: cmd 20000-29999 (economy 110 cmd) — bg_c30c3477
- w3: cmd 19000-19999 + 25000-25999 (battle 103 cmd) — bg_d192ba2d
- w4: cmd 13000-14999 + 16000-16999 (social 93 cmd) — bg_d14dd97a
- w5: cmd 14000-14999 + 30000-30100 + 24000-24999 (admin 192 cmd) — bg_3b9f8f1d

5 worker 各自 worktree (`D:/rgs-shim-w1...w5`), branch `w<N>/shim`, baseline b3d4a55. 1-2 周并行 detached.

**业务 handler 落地方式** (5 worker 各自):
1. 读 E 盘 zsyz erlang proto_*.erl + mod/*.erl
2. 在 `handlers.rs` 写 `handle_<cmd>` 函数, 调 `crates/<domain>-service` gRPC (经 rgs-proxy 8084)
3. 字节级对齐 erlang (4B BE len + 2B cmd + payload, per `GameTcpClient.h` + `smartsocket.lua`)
4. `registry.rs` 把 `stub-XXX` 替换为新 handler
5. cargo build 0 error + Node.js 字节级测试 100% + 50 rps >= 100
6. 各自 commit 推 origin

## 3. shim v0.5 dispatch table 设计

### 3.1 目标

5 worker 业务 handler 落地后 (估 1-2 周), shim v0.5 整合:
- 766 send cmd → dispatch table → 5 域 RGS gRPC (经 rgs-proxy 8084)
- 5 域 RGS binary 已在 k3s 1/1 Running (per commit b3d4a55)
- shim 业务覆盖 1.9% → 100%

### 3.2 cmd → 域 service 映射

| cmd 范围 | 域 service | gRPC port | 备注 |
|---|---|---|---|
| 10000-10999 | player-service | 50061 | w1 负责 |
| 10400/11001 | shim-internal | - | 9/9 14:55 JST 拍板, 不调 RGS, shim 自己处理 |
| 19000-19999 | match-service | 50063 | w3 负责, 战前/匹配 |
| 20000-29999 | economy-service | 50062 | w2 负责, 战备/背包/邮件/商城 |
| 25000-25999 | match-service | 50063 | w3 负责, 战斗/成就 |
| 13000-14999 | social-service | 50064 | w4 负责, 邮件/排行 |
| 16000-16999 | social-service | 50064 | w4 负责, 工会/好友 |
| 14000-14999 | admin-service | 50065 | w5 负责, GM 命令 |
| 24000-24999 | admin-service | 50065 | w5 负责, 福利/活动 |
| 30000-30100 | admin-service | 50065 | w5 负责, 礼包 |

(具体 cmd 分配 per `tools/rgs-shim-rust/docs/H5_ZSYZ_CLIENT_MIGRATION_MATRIX.md` §3)

### 3.3 dispatch table Rust 伪代码

```rust
// tools/rgs-shim-rust/src/dispatch.rs
use crate::rgs_grpc::{PlayerClient, EconomyClient, MatchClient, SocialClient, AdminClient};

pub async fn dispatch(cmd: u16, payload: &[u8]) -> Result<Vec<u8>, DispatchError> {
    match cmd {
        // 内部 cmd (per 9/9 14:55 JST 拍板, 跟 erlang 10400/11001 冲突)
        10400 => crate::handlers::handle_quest_list(payload).await,
        11001 => crate::handlers::handle_partner_list(payload).await,

        // shim-internal 12 real handlers (v0.4.1)
        10101 | 10102 | 10103 | 10200 | 10215 | 10300..=10399 |
        10500..=10599 | 10800..=10899 => crate::handlers::handle_player_internal(cmd, payload).await,

        // 5 worker business handlers (v0.5+)
        10000..=10999 => dispatch_player(cmd, payload).await,
        13000..=14999 => dispatch_social(cmd, payload).await,
        16000..=16999 => dispatch_social(cmd, payload).await,
        19000..=19999 => dispatch_match(cmd, payload).await,
        20000..=29999 => dispatch_economy(cmd, payload).await,
        25000..=25999 => dispatch_match(cmd, payload).await,
        24000..=24999 => dispatch_admin(cmd, payload).await,
        30000..=30100 => dispatch_admin(cmd, payload).await,

        // unknown: stub fallback
        _ => crate::handlers::handle_stub(cmd, payload).await,
    }
}

async fn dispatch_player(cmd: u16, payload: &[u8]) -> Result<Vec<u8>, DispatchError> {
    let client = PlayerClient::connect("http://rgs-proxy:8084").await?;
    let resp = client.dispatch(cmd, payload).await?;
    Ok(resp)
}

// similar for economy / match / social / admin
```

### 3.4 关键设计决策

1. **dispatch table 自动生成**: 用 Rust macro + `proto_mate.js` cmd 列表 auto-gen `match` arm, 避免手写 766 case 出错
2. **cmd 分组 by domain**: 10000-10999 全 player, 20000-29999 全 economy, 避免 cmd 范围 split 跨 service
3. **shim-internal 保留**: 10400/11001 shim 内部, 不调 RGS, per 9/9 14:55 JST 拍板
4. **unknown cmd stub fallback**: shim 跑时, 未注册的 cmd 自动 stub (per PHASE_4 §6), 兼容 Phase 4 5 worker 进度差异
5. **gRPC client connection pool**: 5 域 5 client, 每个 client connection pool size 50 (per 9/3 09:00 JST + 8/31 rps 1099 估)
6. **timeout 5s**: gRPC call 5s timeout, 5 域 HealthCheck 5/5 OK (per commit b3d4a55)
7. **metric + trace**: OTel exporter (5 域 deployment env 已配 `OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317`), shim 也加 exporter

### 3.5 实施步骤

| 步骤 | 描述 | 估时 | 责任人 |
|---|---|---|---|
| 1. 5 worker 业务 handler 落地 | handlers.rs 5 worker 各自 commit | 1-2 周 | 5 worker w1-w5 (per Phase 4) |
| 2. shim dispatch table auto-gen | Rust macro from proto_mate.js → match arm | 1-2 天 | Mavis 主会话 |
| 3. 5 域 gRPC client 包装 | req/resp type + 5 client pool | 2-3 天 | Mavis 主会话 |
| 4. stub fallback 兼容 | 未注册 cmd 走 stub 0 数据 | 0.5 天 | Mavis 主会话 |
| 5. integration test | shim v0.5 vs 5 worker 业务 handler, Node.js 字节级 100% | 1-2 天 | Mavis 主会话 + 5 worker 协助 |
| 6. shim v0.5 commit + push | 5 worker merge + dispatch table + 5 client + stub | 0.5 天 | Mavis 主会话 |
| 7. e2e UAT (Phase 6) | zsyz H5 真机接入, 8 步战斗流/任务流/邮件流 | 3-5 天 | Mavis 主会话 |
| 8. perf 调优 (Phase 7) | shim tokio + 5 域 sqlx pool, 目标 < erlang p99 50ms | 2-3 天 | Mavis 主会话 |

### 3.6 风险 + 缓解

| 风险 | 缓解 |
|---|---|
| 5 worker 业务 handler 字节级不匹配 erlang | per worker Node.js 字节级测试 100% DoD, 5 worker 各自验证 |
| 5 域 gRPC client connection pool 满 | shim 单进程 + tokio multi-thread + pool size 50, monitor 5 域 HealthCheck |
| 5 worker 进度不同 (1-2 周) | shim v0.5 dispatch table 用 stub fallback, 未落地 cmd 仍 stub 0 数据, 兼容 |
| 跨域 saga 时序 | per 5 域 outbox + saga_init.sql (already in migration), shim 不直接参与 saga 状态机, 只 dispatch |
| 性能 | Phase 7 perf 调优, p99 50ms 目标 |

## 4. 后续 (Mavis 主会话)

1. **5 worker 监控** (5-30 min 一次, 看 commit progress, 1-2 周后 100% cmd done)
2. **postgres migration apply** (主会话负责, 已 kick off detached 跑, 5 worker 跑时 sqlx auto-apply 也行)
3. **shim v0.5 dispatch table 实施** (5 worker 完成后, Mavis 1-2 天实现)
4. **Phase 6 UAT 自动化** (zsyz H5 binary build 1-2h, Cocos Creator 2.3.2 装)
5. **AGENTS.md §8 持续加派生约束** (本轮 Phase 4 派工 + shim v0.5 + Phase 6 UAT 教训)
6. **ghcr.io visibility 拍板** (per 9/9 19:15 JST report, read 403, Ulysses 拍板 private + read:packages PAT 或 public)

代签: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
