# RGS-DDD-2026-09-10-bottest-from-erlang — erlang tester 优点迁移到 RGS 测试体系 DDD Review

| 字段 | 值 |
|---|---|
| 文档 ID | RGS-DDD-2026-09-10-bottest-from-erlang |
| 版本 | v0.3 |
| 创建日期 | 2026-09-10 JST |
| 创建者 | 架构师(Mavis 接手 agent per DEC-008) |
| 类型 | DDD Review 二审材料 (per DDD-REVIEW-TEMPLATE-v0.2) |
| 关联 | commit `32cff91` (基线) / `3131cd0` (v0.1) / `16a4b19` (v0.2) / `8979e3c` (v0.3 Phase B 落地 merge) / rgs-testkit 强约束段 (`crates/rgs-testkit/src/lib.rs:16-44`) / 闪烁之光 erlang 服务端 (`E:\shanshuo-src-winrar\zsyz_server\tester\src\*`) / AGENTS.md v0.6.13 |
| 基线 commit | `32cff91` (fix(deploy): 5 postgres manifest PLACEHOLDER 替换, per 9/10 WipeCluster 重建) |
| 当前 commit | `4157731` (fix(testkit) wave 3 MtlsConfig 兼容, 9/10 18:24 JST wave 3 全部 merge) |
| 范围 | 跨域(全 5 域 + 工具) — 借鉴 erlang tester 30 条优点, 设计 RGS bot 框架 + 5 域 BotAi 派生 + 9/10 15:14-15:30 JST WipeCluster 重建 k3s 拉起 5 域尝试 + 9/10 17:30-18:24 JST wave 3 mTLS 真实接入 |
| 阶段 | Phase B + Phase C mTLS 框架完成 (per D2 L1/L1.1/L1.2 三件套, L1+L1.1 ✅, L1.2 N/A 待 SRE 介入 k3s baseline 恢复) |
| 状态 | ✅ 二审通过 + Phase B 落地 + wave 3 mTLS 框架完成 (per 2026-09-10 16:36 JST Ulysses 拍板 "接受 baseline 0/12" + 选项 1 wave 3 启动) |

---

## 1. 执行摘要 (Executive Summary)

- **时间窗**: 2026-09-10 10:45 JST ~ 12:50 JST (≈ 2 小时调研 + 起草)
- **操作者**: Mavis 接手 (主会话, 调研 erlang tester 6 个源文件 + RGS 测试现状对比 + 12 条迁移建议)
- **触发**: 2026-09-10 10:41 JST Ulysses 问询"闪烁之光服务器有测试用例或者脚本吗" → 主会话定位 `E:\shanshuo-src-winrar\zsyz_server` (Erlang 闪烁之光服务端) → 读取 6 个测试源文件 → 提炼 30 条优点 → 对比 RGS 现状 (PG 真集成 / chaos / mTLS / e2e-smoke 已远超 erlang, 但缺应用层 bot 压测) → 12 条迁移建议
- **风格**: 主会话打头阵读 erlang 源码 + 看 RGS 现状 (per AGENTS.md §2.3 L4) → 出一份设计/计划文档 → 后续派 worker 实施 P0 6 条
- **拍板 (per 9/1 14:58 JST + 9/8 16:08 JST 守门, ask_user 推荐项 2026-09-10 12:45 JST)**:
  1. 起草 DDD Review 文档 + 落 commit (不立即派 worker, 文档先走二审)
  2. bot 框架代码落点 = `crates/rgs-testkit/src/bot/` (跟现有 `fixture / helper / mock / pg_test_db` 同级, 5 域共享)
- **最终产出**: 1 DDD Review 文档 (`v0.1`) + 1 commit (docs) + 12 条迁移建议清单 (P0 6 条 / P1 5 条 / P2 1 条, 估算 16 人·天)

---

## 2. 基线与分支拓扑

### 2.1 基线 commit SHA

- **基线**: `32cff91` (fix(deploy): 5 postgres manifest PLACEHOLDER 替换, per 9/10 WipeCluster 重建, 2026-09-10 09:55:25 +0900)
- **main HEAD**: 同上 (未 commit, 工作区有 .gitignore 杂项但不阻塞)
- **branch**: `main` (本会话直接在 main 操作, 无 worktree 派工, 因 v0.1 只起草文档)

### 2.2 分支策略 (后续派工阶段)

- **L12.2 选项 1 (per 8/31 W37 模式)**: 5 worker 独立 worktree, 各自 commit 到自己 branch, 主会话统一 merge
- **worktree 路径预定**: `D:/rgs-bottest-<scope>` 5 个 (scope = bot-core / bot-ai / bot-supervisor / bot-stats / bot-bridge)
- **per-worker `CARGO_TARGET_DIR=target-r1-bottest-<scope>`** (per 9/3 08:42 JST L11 dir lock 修复)
- **staggered 启动**: 5 worker 间隔 30s (per 9/3 11:08 JST L12.2)
- **DoD 简报明文**: "worker 不 commit, 报告即可" (避免 9/3 11:08 race condition 教训复发)

### 2.3 引用证据 (git 实证)

| 引用对象 | 位置 | 实证方式 |
|---|---|---|
| RGS main HEAD | `32cff91` | `git log -1` |
| rgs-testkit 强约束段 | `crates/rgs-testkit/src/lib.rs:16-44` | Read |
| e2e-smoke.ps1 12 probe baseline | `scripts/e2e-smoke.ps1:14-22` | Read |
| erlang tester 源码 6 个 | `E:\shanshuo-src-winrar\zsyz_server\tester\src\*` | Read |
| AGENTS.md L1-L14 派生约束 | `AGENTS.md` | Read (项目 instructions) |
| L-CANDIDATES 候选清单 | `docs/14-项目治理/L-CANDIDATES.md` | 后续登记新 L-CAND-010 |
| DDD Review 模板 | `docs/14-项目治理/DDD-REVIEW-TEMPLATE-v0.2.md` | Read (v0.2 二审流程) |
| 闪烁之光 RGS 借鉴 handoff | `docs/deploy/RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04` (L18 段) | 已存在 |

---

## 3. 主题 1: Erlang tester 30 条优点全景 (per 6 个源文件)

### 3.1 源码清单 (`E:\shanshuo-src-winrar\zsyz_server\tester\src\`)

| 文件 | 大小 | 角色 |
|---|---|---|
| `tester.erl` | 17 KB | tester 主程序, gen_server + TCP socket 测试客户端, 导出 `start/8, start/1, stop/1, info/2, cmd/3,4, pack_send/2,3` |
| `tester_ai_base.erl` | 12 KB | AI 行为框架 (callback 协议 + act_list 序列 + AI 派生) |
| `test.erl` | 6.6 KB | tester 启动器, gen_server, 导出 `run/0, stop/0, t/2..7, count/0, offline/0, next_lev/0, max_lev/1` |
| `test_rpc.erl` | 6.4 KB | RPC 协议层 handler, 覆盖 5 域 (combat/guild/drama/arena/partner/dun) |
| `tester_ai_quest.erl` | 1.8 KB | 任务 AI 派生, 把所有 quest ID 序列化为 `gm_accept_quest → rand_map → gm_commit_quest` |
| `test_util.erl` | 954 B | 测试工具函数 (pos_info/1 阵位信息) |

### 3.2 6 大类 30 条优点

#### A. 进程/状态架构 (3 条)

| # | 优点 | erlang 实现 | file:line |
|---|---|---|---|
| **A1** | 每 bot 独立 gen_server (1 进程=1 虚拟玩家) | `gen_server:start({local, LocalName}, ?MODULE, [Tester], [])` | `tester.erl:7-216` |
| **A2** | ETS 全局表 `tester_online` (set, public) | `ets:new(tester_online, [set, named_table, public, {keypos, #tester.id}])` | `test.erl:112-113` |
| **A3** | 进程注册名 (本地命名空间, account→LocalName) | `whereis/1` 可查, 调试方便 | `tester.erl:66-70` |

#### B. AI 行为框架 (4 条)

| # | 优点 | erlang 实现 | file:line |
|---|---|---|---|
| **B1** | 可插拔 AI 模块 callback 协议 | `-callback init/1`, `-callback act_list/1`, `-callback handle/2` (3 个回调) | `tester_ai_base.erl:25-27` |
| **B2** | AI mod 派生机制 (set_ai_mod 路由) | `set_ai_mod(questgm) -> tester_ai_quest` | `tester_ai_base.erl:40-45` |
| **B3** | `act_list` 行为序列抽象 (行为是序列而非单点) | `act_list(robot) -> [init, rand_restart, rand_proto, guild, vip, dun, arena, boss, partner]` | `tester_ai_base.erl:54-58` |
| **B4** | AI act 队列 (process dict, 任务流) | `add_ai/1` push 进队列, `do/1` 取首项 dispatch | `tester_ai_base.erl:283-289` |

#### C. AI 行为实现细节 (8 条)

| # | 优点 | erlang 实现 | file:line |
|---|---|---|---|
| **C1** | 协议随机化触发 (`util:rand(1, N)` 概率) | `util:rand(1, 1000)` 概率触发 10800/10802/邮件/伙伴 | `tester_ai_base.erl:137-162` |
| **C2** | GM 命令注入 (中文指令, 协议 10399) | `pack_send(10399, {<<"加经验 ~w"/utf8>>})` | `tester_ai_base.erl:94, 126-134` |
| **C3** | 反应时间随机化 (500-3000ms 模拟人类) | `put(reaction_time, util:rand(500, 3000))` | `tester.erl:206, 247` |
| **C4** | 随机重登录 (`rand_restart` 1/(2000-acc/10)) | `do_ai({restart}, Tester)` | `tester_ai_base.erl:119-123` |
| **C5** | 随机化等级控制 (`rand_lev` 1/2000 概率设等级) | `pack_send(10399, {util:fbin("设等级 ~w", [N])})` | `tester_ai_base.erl:124-136` |
| **C6** | 业务场景 act (guild/dun/arena/boss/partner/vip) | 每业务域有专门 act, 触发对应协议号 | `tester_ai_base.erl:163-278` |
| **C7** | 战斗中状态保护 (`in_combat` break) | `handle(in_combat, ...) -> {break, Tester}` | `tester_ai_base.erl:109-113` |
| **C8** | quest 派生模式 (遍历 quest_data 序列化) | `gm_accept_quest, Id -> (rand_map) -> gm_commit_quest, Id` | `tester_ai_quest.erl:11-20` |

#### D. 协议层/网络 (4 条)

| # | 优点 | erlang 实现 | file:line |
|---|---|---|---|
| **D1** | 协议号→mod 路由 (`mapping:module`) | `mapping:module(tester, Code) -> {ok, Parser, Mod}` | `tester.erl:149-152, 324-339` |
| **D2** | 登录票据 (md5 防伪) | `md5(account + srv_id + now + server_key)` | `tester.erl:210-212` |
| **D3** | 异步 socket recv (4 字节头 + payload) | `prim_inet:async_recv(Socket, 4, -1)` | `tester.erl:316-339, 452-460` |
| **D4** | 调试开关 (编译期 `-ifdef(dbg_socket)`) | 切换 print 收发包 | `tester.erl:27-34` |

#### E. 运维/统计/可观察性 (7 条)

| # | 优点 | erlang 实现 | file:line |
|---|---|---|---|
| **E1** | 实时在线统计 (`count/0` 按 mod 聚合) | `ets:foldl(F, [], tester_online)` | `test.erl:85-92` |
| **E2** | 掉线自愈 (`relogin` loop 10 min) | `erlang:send_after(1200000, self(), loop)` | `test.erl:171-192` |
| **E3** | 可中断 AI (`{do_ai_flag, stop}`) | `handle_info({ai, Flag}, State) -> put(do_ai_flag, Flag)` | `tester.erl:277-279` |
| **E4** | 心跳延迟检测 (60s 触发 restart) | `heartbeat` 每 25s 发 1199 | `tester.erl:224-244` |
| **E5** | 可观察运行报告 (terminate 输出) | "在线时长 / AI 量 / 发包数/秒 / 收包数/秒" | `tester.erl:406-422` |
| **E6** | 进程一致性自检 (`check_pid` 10 min) | 比对 ets pid 和 self pid | `tester.erl:380-390` |
| **E7** | 心跳包 GC 联动 (`rem 10 -> 1` 触发 gc) | `garbage_collect()` | `tester.erl:235-244` |

#### F. 启动/部署模式 (4 条)

| # | 优点 | erlang 实现 | file:line |
|---|---|---|---|
| **F1** | 多环境绑定 (`t/7` 多态: self/demo_1/dev_1..3/local_1..2) | 7 套环境路由, 5 套独立端口 | `test.erl:60-68` |
| **F2** | 统一测试入口 (`test:run/0` + `t/2..7` 5 套压测) | 一句话启动 1 bot, 5 套压测入口 | `test.erl:39-56` |
| **F3** | 错峰启动 (`util:sleep((I-N)*Time)`) | 每 bot 错峰 (I-N)*100ms, 避免峰值 | `test.erl:72` |
| **F4** | 登录随机化延迟 (2-30s 防同步) | `erlang:send_after(util:rand(2000, 30000), ...)` | `test.erl:128-131` |

---

## 4. 主题 2: RGS 现状对比 + 关键差异

### 4.1 RGS 现有测试资产清单 (基线 `32cff91`)

| 资产 | 位置 | 覆盖度 | 来源 |
|---|---|---|---|
| **rgs-testkit 强约束** | `crates/rgs-testkit/src/lib.rs:16-44` | 5 域 + cluster-ops + shared-platform 统一引用 | Read |
| **rgs-testkit 子模块** | `lib.rs:46-49` → `fixture.rs (8.4 KB)` / `helper.rs (2.0 KB)` / `mock.rs (10.1 KB, deprecated)` / `pg_test_db.rs (6.0 KB)` | fixture + helper + 真 PG 强制 | Read |
| **PG 真集成 (强约束)** | `rgs-testkit::pg_test` (per lib.rs:73-92) | 5 域 `integration_*.rs` 100+ 全部走真 PG | Read |
| **5 域集成测试** | `crates/{player,match,economy,social,admin,leaderboard,replay,i18n}-service/tests/integration_*.rs` (100+ 文件) | 业务级集成, 但不走真实协议 | glob |
| **chaos 测试** | `crates/{admin,economy,rgs-asset-download}-service/tests/chaos_*.rs` (8 文件) | 故障注入, 业务级 | grep chaos_ |
| **mTLS 业务级 ST** | `crates/{player,match,economy,social,admin}-service/tests/mtls_mock_phase_c.rs` | 5 域 mTLS Phase C 跑通 | glob |
| **e2e-smoke.ps1** | `scripts/e2e-smoke.ps1:14-22` (12 probe K8s 探活) | HTTP / gRPC / QUIC / TCP 端口可达 | Read |
| **ST 套件 (PowerShell)** | `scripts/st/st-01..16-*.ps1` (16 文件, 5 域 mTLS 业务级) | grpcurl 调用真实 5 域 | glob |
| **e2e-smoke.sh** | `scripts/e2e-smoke.sh` (WSL driver) | 12 probe bash 实现 | Read |
| **db seed** | `tools/db-seed/{seed-all,reset-and-seed,check-counts}.sh` | DB 初始化 + 校验 | glob |

### 4.2 erlang tester vs RGS 对比表

| 维度 | erlang tester | RGS 现状 | 评估 |
|---|---|---|---|
| 测试类型 | 应用层 bot 压测 (1 bot = 1 玩家) | K8s 探活 + 域级 integration + chaos | **RGS 缺应用层 bot 压测** |
| 测试入口 | `test:run/0` + `t/2..7` 5 套 | `cargo test -p <svc>` / `cargo test --test '*'` | **RGS 缺统一 bot 入口** |
| 进程模型 | 1 bot = 1 gen_server, 真实玩家协议 | 1 测试 = 1 tokio task, 不走协议层 | **RGS 缺 bot 框架** |
| AI 行为 | 可插拔 act_list 序列 | 断言式 (assert_eq! / assert!), 无 AI | **RGS 缺 AI 框架** |
| 协议随机化 | `util:rand(1, N)` 概率触发 | 无, 只 happy path | **RGS 缺随机化** |
| GM 注入 | 协议 10399 注入字符串 | `admin-service::issue_gm_command` (mTLS) 能力在, bot 层未对接 | **RGS 能力在, 未对接** |
| 多环境 | `t(dev_1, dev_2, dev_3, local_1, local_2)` 5 套 | `start-5-rgs-services.ps1` 起 5 域, 无多区服路由 | **RGS 缺多环境** |
| stat 实时 | `count/0` + `offline/0` | 无 | **RGS 缺** |
| 掉线自愈 | relogin loop 10 min | 无, 只 `fail_closed_start.rs` 单元 | **RGS 缺** |
| 心跳 watchdog | 60s 延迟触发 restart | 无业务级 | **RGS 缺** |
| 可观察报告 | terminate 时输出运行报告 | `tracing` 日志, 无 per-bot 报告 | **RGS 缺** |
| 调试开关 | `-ifdef(dbg_socket)` | `tracing::debug!` + feature flag | ✅ RGS 已有, 需对齐 |
| 错峰启动 | (I-N)*100ms | 无 | **RGS 缺** |
| PG 真集成 | 无 (erlang 用 emysql) | `rgs-testkit::pg_test` 强约束 | ✅ **RGS 远超 erlang** |
| chaos 测试 | 无 | 8 个 chaos_*.rs | ✅ **RGS 远超 erlang** |
| mTLS 业务级 | 无 (TCP 明文) | `mtls_mock_phase_c` 5 域都有 | ✅ **RGS 远超 erlang** |
| E2E smoke | 无 | `e2e-smoke.ps1` 12 probe | ✅ **RGS 远超 erlang** |

### 4.3 关键差异总结

- **RGS 在测试基础设施层面全面超越 erlang**: PG 真集成 / chaos / mTLS / K8s 探活 / Phase C 5 域 mTLS 业务级 ST
- **RGS 缺应用层 bot 压测**: 这是 erlang tester 30 条优点的核心价值 — **业务级真实玩家模拟**
- **能力缺口 (14 项)**: bot 框架 / AI 框架 / 协议随机化 / GM 注入对接 / 多环境 / stat / 掉线自愈 / 心跳 watchdog / 报告 / 错峰启动 / 调试开关对齐
- **机会**: erlang 30 条优点中, **18 条可直接迁移** (A1-A3, B1-B4, C1, C3-C8, D1, E1, E2, E4-E7, F1, F3, F4), **6 条需改造** (C2 GM 走 mTLS / D2 md5 票据走 5 域 gRPC 登录流 / D3 async socket 走 tonic gRPC streaming / D4 编译期开关走 feature flag / F2 入口需 Rust 重写), **6 条 RGS 已超越** (PG / chaos / mTLS / K8s / ST 16 脚本 / e2e-smoke 12 probe)

---

## 5. 主题 3: 12 条迁移建议 (优先级 + 落地路径)

### 5.1 12 条迁移项

| # | 建议 | 来源优点 | 优先级 | 工作量 | bot 框架落点 (per 9/10 12:45 JST 拍板) |
|---|---|---|---|---|---|
| **M1** | **rgs-testkit 下新建 `bot` 子模块**, bot = 1 tokio task, 1 虚拟玩家 | A1 | P0 | 3 天 | `crates/rgs-testkit/src/bot/mod.rs` |
| **M2** | **`Bot` trait 抽象 AI** (callback 协议 = B1 移植), 默认 `DefaultBotAi` 含 5 域 mod | B1-B4 | P0 | 2 天 | `crates/rgs-testkit/src/bot/ai.rs` |
| **M3** | **`act_list` 行为序列** (Rust 实现): 每 mod 一组协议号序列 + 概率触发 | C1, C6 | P0 | 2 天 | `crates/rgs-testkit/src/bot/act.rs` |
| **M4** | **GM 命令注入**走 `admin-service::issue_gm_command` (mTLS), 中文指令"加经验 100" | C2 | P0 | 1 天 | `crates/rgs-testkit/src/bot/gm.rs` |
| **M5** | **stat 实时统计** `Bot::count() -> HashMap<Mod, u32>` + `Bot::offline() -> Vec<BotId>` | E1 | P0 | 1 天 | `crates/rgs-testkit/src/bot/stats.rs` |
| **M6** | **掉线自愈 `BotSupervisor`** 周期 re-login, 白名单 4500 保护 | E2, A2 | P0 | 2 天 | `crates/rgs-testkit/src/bot/supervisor.rs` |
| **M7** | **心跳 watchdog**: 每 25s 发心跳, 延迟 > 60s 触发 restart | E4 | P1 | 1 天 | `crates/rgs-testkit/src/bot/heartbeat.rs` |
| **M8** | **反应时间随机化** `Bot::tick_with_jitter(min_ms, max_ms)`, 模拟人类 | C3 | P1 | 0.5 天 | `crates/rgs-testkit/src/bot/jitter.rs` |
| **M9** | **可观察运行报告** `Bot::terminate_report()`, JSON 输出, 聚合到 `scripts/bot-report.ps1` | E5 | P1 | 1 天 | `crates/rgs-testkit/src/bot/report.rs` |
| **M10** | **多环境路由** `BotConfig::for("dev_1" / "local_1" / "demo_1" / "staging")`, 5 套预置 | F1 | P1 | 1 天 | `crates/rgs-testkit/src/bot/config.rs` |
| **M11** | **错峰启动** `BotSupervisor::spawn_with_stagger(N, interval_ms)`, 避免峰值 | F3 | P1 | 0.5 天 | `crates/rgs-testkit/src/bot/supervisor.rs` (并入 M6) |
| **M12** | **quest 派生模式** `QuestBotAi: init() 遍历 quest_data, 序列化 gm_accept_quest / rand_map / gm_commit_quest` | C8 | P2 | 1 天 | `crates/rgs-testkit/src/bot/ai/quest.rs` |

**总工作量**: ≈ 16 人·天 (P0 6 条 11 天 + P1 5 条 4 天 + P2 1 条 1 天)
**5 worker 派工** 1 周可落地 P0 6 条 (per AGENTS.md §6.3 PT 派工模板 + L11/L12)

### 5.2 bot 框架架构图 (落点 = `crates/rgs-testkit/src/bot/`)

```
crates/rgs-testkit/src/
├── lib.rs                       (强约束段 L16-44 已有)
├── fixture.rs                   (已有)
├── helper.rs                    (已有)
├── mock.rs                      (已有, deprecated)
├── pg_test_db.rs                (已有, 强约束入口)
└── bot/                         (M1 新建)
    ├── mod.rs                   (Bot public API + Bot trait)
    ├── ai.rs                    (M2 Bot trait 抽象 + DefaultBotAi)
    ├── act.rs                   (M3 act_list 行为序列)
    ├── gm.rs                    (M4 GM 命令注入, 走 admin-service mTLS)
    ├── stats.rs                 (M5 stat 实时统计)
    ├── supervisor.rs            (M6 掉线自愈 + M11 错峰启动)
    ├── heartbeat.rs             (M7 心跳 watchdog)
    ├── jitter.rs                (M8 反应时间随机化)
    ├── report.rs                (M9 运行报告 JSON)
    ├── config.rs                (M10 多环境路由)
    └── ai/
        ├── mod.rs
        ├── player.rs            (DefaultBotAi for player 域)
        ├── economy.rs           (DefaultBotAi for economy 域)
        ├── match.rs             (DefaultBotAi for match 域)
        ├── social.rs            (DefaultBotAi for social 域)
        ├── admin.rs             (DefaultBotAi for admin 域)
        └── quest.rs             (M12 quest 派生 AI)
```

### 5.3 跟现有 rgs-testkit 强约束段的关系

- `crates/rgs-testkit/src/lib.rs:16-44` 强约束段已有 `pg_pool()` + `pg_test` re-export (真 PG 强制)
- bot 框架**不替换** PG 强约束, 而**叠加**:
  - bot 调用 gRPC 走 tonic client (5 域 gRPC)
  - bot 状态变化如需持久化, 走 `pg_pool()` 强约束入口
  - bot 框架内**禁止**用 `mock::DbMock` / `mock::NoopMock` (跟强约束段 `#[deprecated]` 一致)
- 编译期 `compile_fail` doctest 锚定: bot 框架如有人误用 InMemory mock, 应产生编译错误 (跟 rgs-testkit lib.rs 已有模式一致)

### 5.4 落地 3 步路径 (per Mavis 自驱原则 9/8 15:29 JST 第 7 次强化)

**Step 1: 设计文档** (本文档, v0.1) — 1 commit (本 commit)
- Mavis 自审 1 次后停手 (per §3.x DDD Review 二审流程)
- 等 Ulysses 二审

**Step 2: 派 worker 实施 P0 6 条** (二审通过后, Mavis 自驱)
- 5 worker 并发 (M1-M3 / M4-M5 / M6-M11 / M2 DefaultBotAi 5 域 / 集成测试 + e2e 验证)
- worktree: `D:/rgs-bottest-<scope>` 5 个
- per-worker `CARGO_TARGET_DIR=target-r1-bottest-<scope>`
- staggered 30s
- DoD: `cargo check -p rgs-testkit --tests` 1 次拿 status, 不 polling

**Step 3: 落地到 5 域** (派工完成后, 主会话 merge + 5 域集成测试)
- 5 域 bot 模式在 `crates/<domain>-service/tests/bots/` (per worker 5 域)
- `scripts/bot-driver.ps1` (主入口, 跨 5 域启动)
- `tools/rgs-flash-mock/scripts/bot-smoke.sh` (per 9/4 17:47 JST "测试脚本归入 mock 项目" 守门, 备选)

---

## 6. 派生约束守护 (per AGENTS.md v0.6.13 §2.1 L1/L1.1/L1.2 + §2.6 L11/L12/L13/L14)

| 派生约束 | 状态 | 备注 |
|---|---|---|
| **L1** cargo check --tests 0 error | ✅ 0 error 0 warning 0.49s (wave 3) | 主会话跑 `cargo check -p rgs-testkit --tests` (per 9/10 18:24 JST), 5 wave 3 worker 各自 0.27-1.27s + 0.49s 主会话复验 + workspace 0 error (per 9/10 15:00 JST) |
| **L1.1** cargo test --lib 跑通 | ✅ 60 passed 0 failed 0.13s (wave 3) | 主会话跑 `cargo test -p rgs-testkit --lib` (per 9/10 18:24 JST), 41 wave 2 + 5 域 mTLS 真实 client + 4 admin GmClient unit = 60 total |
| **L1.2** E2E 业务级 | ⏳ 待 SRE 介入 k3s baseline 恢复 | bot 框架需 k3s 集群可达 (current 0/12 per §7.4), 跟 mTLS 业务级 ST 一起跑 (per §7.3 + L-CAND-016) |
| **L11** cargo build dir lock 不轮询 (per 8/31 PT 派工) | ✅ 计划已写 | 简报明文"1 次拿 status, 不 polling", per-worker `CARGO_TARGET_DIR=target-r1-bottest-<scope>` |
| **L12** 临时 log / .txt / .tmp_search* 不入 commit (pre-commit hook 兜底) | ✅ 计划已写 | pre-commit-tmp-check.ps1 (per 9/3 07:31 JST 拍板), 简报明文"不入 commit, 主会话 merge 后清理" |
| **L12.1** 临时 log 不入 commit | ✅ 同 L12 | |
| **L12.2** 5 worker 派工 3 选项 (per 9/3 11:08 race condition 教训) | ✅ 计划已写 | 选选项 1: 5 worker 独立 worktree, 主会话 merge |
| **L13** 自指字段 deferred 实时查询 (git log + grep 实证) | ✅ 文档已自查 | §2.3 引用证据段全 git 实证 (32cff91 / rgs-testkit L17-34 / e2e-smoke.ps1:14-22 / DDD-REVIEW-TEMPLATE-v0.2.md) |
| **L14** plumbing 节点字符串 brace 跟踪 (per 9/2 W2 BA-W2 patch) | ✅ | Phase B match + admin merge 触发 2 次 mod.rs conflict, 主会话手修用 `<<<<<<<` `=======` `>>>>>>>` 4 边界 brace 跟踪, 0 误判 |
| **守门 #5** env 安全 (per 8/27 11:06 JST hard ban) | ✅ | 文档无 env value 痕迹, 凭据引用标 "走 stdin pipe" 而非值 |
| **守门 #14 v2** Mavis 长期代签, 真人到位后追溯签字 | ✅ | per 9/5 10:43 JST 拍板 D, 9/8 15:19 JST 第 6 次强化 |

**L-CAND 入档计划** (per 9/3 12:36 JST L12.3):
- `L-CAND-010`: "rgs-testkit bot 框架 12 条迁移项 + 5 worker 派工 + per-worker CARGO_TARGET_DIR" (本 DDD Review v0.1 二审通过后入档)

---

## 7. 后续工作 (per WBS v0.2 / BATCH-PLAN)

### 7.1 Phase A — 设计 (本文档, ✅ 2026-09-10 12:50 JST 完成)

- [x] 调研 erlang tester 6 个源文件 (2026-09-10 10:45 JST)
- [x] 对比 RGS 现状 (per `32cff91` 基线)
- [x] 12 条迁移建议 + 优先级 + 工作量
- [x] 落地 3 步路径
- [x] 派生约束守护段 (L1/L1.1/L1.2 + L11/L12/L13/L14)
- [x] §8 已知缺口
- [x] §9 签字栏 2 段
- [x] §10 修订历史
- [x] 落 v0.1 commit `3131cd0` (12:47 JST)
- [x] 二审通过 v0.2 commit `16a4b19` (12:59 JST, per Ulysses "可以自驱了" 拍板)

### 7.2 Phase B — 派工实施 (✅ 2026-09-10 13:25 JST 全部完成)

- [x] 5 worker 派工 (per AGENTS.md §6.3 PT 派工模板)
  - [x] worker-core: M1-M6 bot 框架 (commit `16bfb95`, 9 files / 1117 lines, 28 tests)
  - [x] worker-economy: 5 域 BotAi 派生 (commit `90829d9`, 3 files / 153 lines, 3 tests)
  - [x] worker-social: 5 域 BotAi 派生 (commit `61872f5`, 3 files / 171 lines, 3 tests)
  - [x] worker-match: 5 域 BotAi 派生 (commit `91e64e7`, 3 files / 189 lines, 4 tests, **r#match 关键字转义**)
  - [x] worker-admin: 5 域 BotAi 派生 (commit `2f50f0f`, 3 files / 298 lines, 6 tests, **GmClient 集成**)
- [x] per-worker `CARGO_TARGET_DIR=target-r1-bottest-<scope>` (per L11)
- [x] 主会话统一 `--no-ff merge` (per L12.2 选项 1)
- [x] 4 merge commit (economy `540dd52` / social `d7a34b6` / match `5afe738` 含 conflict resolution / admin `8979e3c` 含 conflict resolution)
- [x] 跑 `cargo check -p rgs-testkit --tests` 1 次拿 status (per L11) — 0 error 0 warning 28.76s
- [x] 跑 `cargo test -p rgs-testkit --lib` (L1.1) — 41 passed 0 failed 0.12s
- [x] 派生约束 L1 + L1.1 + L11 + L12 + L14 全部 ✅ (per §6)

### 7.3 Phase C — 5 域集成 + 业务级 mTLS 验证 (⏳ 阻塞: 9/10 15:30 JST k3s 5 域 baseline 0/12 等 SRE 介入)

- [ ] 5 域真实 gRPC mTLS 接入 (替换 stub, 走 5 域 ST 业务级 mTLS 实践 commit `401ac5c` 证书导出 SOP)
- [ ] `scripts/bot-driver.ps1` 主入口 (跨 5 域启动)
- [x] k3s 集群可达验证 (per `32cff91` + RGS-OPEN-QA-2026-09-09-k3s-cc13-fix.md) — 15:14 JST 尝试拉起 5 域失败, 0/12 PASS (per §7.4 4 段历史)
- [ ] mTLS 业务级 ST (跟现有 `scripts/st/st-01..16-*.ps1` 集成, L1.2)
- [ ] `tools/rgs-flash-mock/scripts/bot-smoke.sh` (备选, per 9/4 17:47 JST 守门)

### 7.4 Phase D — 文档治理 + 季度评审 (⏳ 9/10 14:35-15:30 JST 4 段历史已记录)

**9/10 14:35-15:30 JST 4 段历史** (per 2026-09-10 15:14 JST Ulysses 拍板 "主会话用 kubectl apply 拉起 5 域" opt1 + 2026-09-10 16:36 JST 拍板 "接受 baseline 0/12 等 SRE 介入" 推荐项):

1. **14:35 JST 跨 session**: `ca493fe feat(rgs-flash-mock): v0.1 PoC HTTP+actix-web 骨架 + 12 类别 21 RPC stub` (per Ulysses 拍板) — rgs-flash-mock 重新启用 (9/9 12:35 JST deprecated 后 v0.1 重新拍板)
2. **15:14-15:17 JST 主会话**: `git status` baseline, k3s 1.36.4+k3s1 起来 (ulyssespc node Ready 19m), `kubectl apply` 全部 57 yaml (per docs/deploy/01-k8s-manifests/) — 5 域 + cluster-ops + gm-backend + postgres + nats + prometheus + grafana + otel + scene + battle + network-gateway
3. **15:18-15:25 JST pod rollout 失败 (per AGENTS.md §2.5 L6 ST FAIL 排查顺序)**:
   - **HPA minReplicas=2 强启动风暴**: `e4-02-hpa-templates.yaml` HPA minReplicas=2 + metrics-server 不可用 (FailedComputeMetricsReplicas 警告) → HPA 反复拉新 pod → CPU Insufficient + SandboxChanged 风暴
   - **gm-backend image tag 不存在**: `50-gm-backend-service.yaml` 旧占位 `0.1.0-gm-backend` + `imagePullPolicy: Never` → ErrImageNeverPull
   - **30+ pod 单节点资源耗尽**: k3s-server 主进程 crash 2 次 (15:22 + 15:25 area) → API server connection refused
4. **15:30 JST 修复 + 落 commit**: scale 5 域到 1 (无效, HPA 立即拉到 2), 删 3 不必要 deployment (scene/battle/network-gateway), 改 `50-gm-backend-service.yaml` image tag 0.1.0-cc13 + IfNotPresent (commit `85bfdf5`), e2e-smoke 12 probe 仍 0/11 PASS + 1 SKIP

**SRE 介入建议 (等 Ulysses 拍板)**:
- 修 HPA minReplicas=2 风暴 (删 HPA 或 scale 0 + 等资源回收)
- 装 metrics-server (k3s metrics-server 单独 deployment)
- 推 gm-backend 镜像到 ghcr.io (`0.1.0-gm-backend` tag)
- 单节点 → 多节点 (避免单点资源压力)

- [x] `L-CANDIDATES.md` 加 `L-CAND-013` (D 盘 0 free 防御) + `L-CAND-014` (mod.rs 4-way conflict 防御) + `L-CAND-015` (HPA 风暴 + 启动风暴防御, per 9/10 15:25 JST)
- [ ] 12/2 季度评审: 复盘 bot 框架落地, 评估 P1 5 条 + P2 1 条是否进入下季度 + 9/10 15:14-15:30 JST 4 段历史回顾
- [ ] `AGENTS.md` 派生约束守护段补 bot 框架相关 L-约束 (L19? 5 worker 派生模式强制 L12.2 选项 2) + L20? (HPA 风暴防御, per L-CAND-015)
- [ ] `RGS-CRITIQUE-IMPROVEMENT-2026-09-02` 升版 (v0.3?), 加 bot 框架章节 + 9/10 WipeCluster 重建反思章节

---

## 8. 已知缺口 (per 8/26 JST 缺标比错标)

| # | 缺口 | 影响 | 跟踪 |
|---|---|---|---|
| **G1** | erlang tester 6 个 .erl 是 GBK 编码, 部分中文注释乱码 | 阅读体验差, 不影响功能理解 (已 Read 全部) | N/A |
| **G2** | erlang tester 依赖 zsyz_server 主源码 (`test_rpc.erl` 引用 `combat.hrl / guild.hrl` 等) | 派工实施时 RGS 侧无对应 .hrl, 需 Rust 重写而非移植 | Phase B 派工需明确 |
| **G3** | erlang tester 协议号 (10101/10399/20001/...) 与 RGS proto RPC 不一一对应 | 12 条迁移项需在 RGS proto 重新映射, 不是字面翻译 | Phase B 派工时 worker-1 列出 RGS proto RPC 清单 |
| **G4** | erlang tester 的 quest_data 遍历在 RGS 无直接对等 | M12 quest 派生模式需要 RGS quest 域数据, 当前 RGS 无独立 quest 域 | Phase B worker-5 调研 RGS quest 实现, 如缺则降级到 "5 域 + skill 模拟" |
| **G5** | v0.1 文档未跑 L1 (cargo check) | 文档无代码改动, L1 N/A | Phase B 派工时跑 |
| **G6** | Ulysses 二审时间窗口不定 (per 9/2 v0.1.1 §6 风险) | 派工实施可能延后 | 配合 14:58 JST ask_user 拍板规则 (per 9/1 守门) |
| **G7** | k3s 集群当前状态 (per 9/10 WipeCluster 重建) | Phase C 业务级 mTLS 验证可能延后 | per `32cff91` 提交 + RGS-OPEN-QA-2026-09-09-k3s-cc13-fix.md 跟踪 |
| **G8** | 5 worker 并发派工的 dir lock 防御已写, 但实际未跑过 (per 9/3 08:42 JST 经验) | Phase B 首次跑可能遇 cargo registry lock | 简报明文 staggered 30s + 监控 |
| **G9** | v0.1 文档未在二审通过后, 在 `L-CANDIDATES.md` 加 L-CAND-010 | 二审通过后立即补 | Phase D |
| **G10** | D 盘磁盘空间耗尽 (0 bytes free, 6 worker target dirs + 30 历史 target-* 累计), `cargo test --workspace --tests` 失败 `os error 112 磁盘空间不足` | 主会话 L1.1 跑测试被阻塞 | 主会话 fallback `CARGO_TARGET_DIR=E:\DevCache\cargo\bottest-main` (E 盘 105GB free), L1.1 41 passed; 已入档 L-CAND-013 |
| **G11** | 5 worker wave 2 merge 时, `crates/rgs-testkit/src/bot/ai/mod.rs` 4 次 conflict (每个 worker 加 `pub mod <domain>;` 行, 顺序错乱) | match + admin 各 1 次 conflict, 主会话手修 2 次 (1 min 内) | 简报明文"mod.rs 加在 player 之后"; 已入档 L-CAND-014; L19 候选 (5 域派生强制 L12.2 选项 2) |
| **G12** | 9/10 15:14-15:30 JST WipeCluster 重建后 k3s 5 域拉起尝试 0/12 PASS (per §7.4 4 段历史): HPA minReplicas=2 强启动风暴 + gm-backend image tag 缺失 + 30+ pod 单节点资源耗尽 → k3s API server 反复 crash | L1.2 E2E 业务级 ST 阻塞 | SRE 介入 (删 HPA + metrics-server + gm-backend 镜像重推 + 多节点); 已入档 L-CAND-015 (HPA 风暴防御候选) + L-CAND-016 (mTLS stub 防御候选, per 9/10 18:24 JST) |
| **G13** | 9/10 18:24 JST wave 3 5 worker merge 阶段 Cargo.toml 3 次 conflict (per L-CAND-014 模式再现) + MtlsConfig skip_verify 字段 5 处缺失 (admin 加字段没通知 social/match) | 主会话 3 次手修 Cargo.toml + commit `4157731` 修 MtlsConfig | 简报明文 "worker 加公共 struct 字段时同步更新其他 worker 用例" (per 12/2 季度评审) |

---

## 9. 签字栏 (per DDD-REVIEW-TEMPLATE-v0.2 二审流程)

### 9.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008) / 修订人=Ulysses — Mavis 接手 |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | §6 派生约束守护段 L1/L1.1/L1.2 + L11/L12/L13/L14 |
| Evidence 段 (commit SHA / file:line) | ✅ | §2.3 引用证据段全 git 实证 (commit `32cff91` / rgs-testkit `lib.rs:16-44` / e2e-smoke.ps1:14-22 / DDD-REVIEW-TEMPLATE-v0.2.md) |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | §6 全 ✅ |
| 缺标比错标 (per 8/26 JST) | ✅ | §8 已知缺口段 9 条 (G1-G9) |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 无 "per X 历史形态" / "per X 升版前" / "原本是" 等 |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 无 env value 痕迹, §5.4 简报明文 "走 stdin pipe" 而非值 |
| 拍板依据 (per 9/1 14:58 JST + 9/8 16:08 JST 守门) | ✅ | §1 列出 2026-09-10 12:45 JST ask_user 推荐项 (2 项都选推荐项) |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-10 JST

### 9.2 Ulysses 二审 (必到, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ✅ | §2.3 引用证据段全 git 实证 (commit `32cff91` / rgs-testkit `lib.rs:16-44` / e2e-smoke.ps1:14-22 / 6 erlang .erl file:line) |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ | §6 全 ✅ (L1/L11/L14 N/A 因 v0.1 无代码, L12 ✅ 仅 add 1 .md, L13 ✅ Evidence 全实证) |
| 业务 vs 治理指标 (per v0.1.1 §9.4 里程碑重定义) | ✅ | 1 commit ahead (3131cd0 唯一 commit, 在 ±20 范围), md 401 行 + 28KB 详尽, hotfix 数 = 0 |
| commit ahead 合理性 (per 当前 sprint 范围) | ✅ | 1 commit ahead (3131cd0 唯一 commit), 在 ±20 范围 |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ✅ | 派工 + 落地 + 二审流程跟 v0.1.1 §5 一致 |
| 跟 RGS-WEEKLY 一致性 (若存在) | ✅ | W37 v0.1 + W38 周报对齐 (bot 框架不破坏现有周报双指标) |
| 12 条迁移项优先级 + 工作量合理性 | ✅ | P0 6 条 = 11 人·天, 5 worker 派工 1 周可落地, 跟 §5.4 3 步路径一致 |
| bot 框架落点 `crates/rgs-testkit/src/bot/` (per 2026-09-10 12:45 JST 拍板) | ✅ | 跟 5 域派工边界无冲突 (bot 框架在 rgs-testkit 工具 crate, 5 域用 `[dev-dependencies]` 引用, per §5.2 架构图) |

**Ulysses 二审决定** (per 2026-09-10 12:59 JST Ulysses "可以自驱了" 拍板):

- [x] ✅ 通过 — 落地, 状态机结束, Mavis 自驱进 Phase B 派工
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 9.1 → 9.2 循环 (打回次数: <1/2/3>)

签字: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 — 日期: 2026-09-10 12:59 JST

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-10 12:47 | 架构师(Mavis 接手 agent per DEC-008) | 初始 DDD Review 一审材料 (30 条 erlang 优点 + RGS 现状对比 + 12 条迁移建议 + 落地 3 步路径 + 派生约束守护段 + 9 条已知缺口), per 2026-09-10 12:45 JST ask_user 拍板 (起草 DDD 文档 + bot 落点 crates/rgs-testkit/src/bot/), commit `3131cd0` |
| v0.2 | 2026-09-10 12:59 | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 | 二审通过 (per Ulysses "可以自驱了" 拍板): 状态机 ⏳ → ✅, §9.2 8 项全 ✅, 二审决定勾选 ✅ 通过, 立即进 Phase B 派工 5 worker 实施 P0 6 条 (M1-M6 + 5 域 PoC) |
| v0.3 | 2026-09-10 13:25 | 架构师(Mavis 接手 agent per DEC-008) | Phase B 派工 5 worker 全部落地 + 主会话 4 --no-ff merge + L1.1 41 passed 0 failed 0.12s: 5 commits (`16bfb95` core / `90829d9` economy / `61872f5` social / `91e64e7` match / `2f50f0f` admin) + 4 merge (`540dd52` / `d7a34b6` / `5afe738` / `8979e3c`) + 9 条 → 11 条已知缺口 (G10 D 盘 0 free + G11 mod.rs 4-way conflict) + L-CAND-013/014 入档 + L1/L1.1/L11/L12/L14 全部 ✅; main HEAD `8979e3c` |
| v0.3.1 | 2026-09-10 16:38 | 架构师(Mavis 接手 agent per DEC-008) | Phase C k3s 5 域拉起尝试 + 阻塞报告 (per 2026-09-10 15:14 JST Ulysses 拍板 opt1 落地 + 2026-09-10 16:36 JST 拍板"接受 baseline 0/12, 落报告等 SRE 介入"推荐项): §7.4 加 Phase C 9/10 14:35-15:30 JST 4 段历史 (rgs-flash-mock ca493fe 启用 → kubectl apply 57 yaml → HPA 风暴 + image tag 缺失 + k3s API server crash → 改 gm-backend image + commit `85bfdf5`) + §8 G12 HPA 风暴 + L-CAND-015 入档 (per 9/10 15:25 JST); main HEAD `85bfdf5`, 12 commits ahead of `32cff91` |
| v0.3.2 | 2026-09-10 18:24 | 架构师(Mavis 接手 agent per DEC-008) | Wave 3 mTLS 真实接入 5 worker 全部落地 + 5 --no-ff merge + L1.1 60 passed 0 failed 0.13s: 5 commits (`b947c97` economy / `e62f79f` player / `037edf3` match / `a9ef3e5` social / `9788404` admin) + 5 merge (`3338ed3` / `db9c6b2` / `7a06f89` / `8c75a00` / `2a432bc`) + commit `4157731` 修 5 处 MtlsConfig skip_verify 字段兼容 (per L-CAND-014 模式再现) + L-CAND-016 入档 (mTLS stub 防御候选); main HEAD `4157731`, 19 commits ahead of `32cff91` |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
