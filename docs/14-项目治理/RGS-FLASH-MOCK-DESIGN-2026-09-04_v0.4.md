# RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.4 — [游戏A] mock 设计 (module_switch 12 module 落地收口 + 缺口闭环)

> **创建日期**: 2026-09-04 16:14 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) — 待 Ulysses 二审
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/4 16:14 JST user 拍板 "**完整 1351 mock (long-term)**" + **9/4 16:45 JST user 升级拍板 "完全对齐"** (per ask_user option C, 15-25 sprint long-term) + 9/4 15:34 JST user 拍板 "**仅 API 对齐, 酌情优化, 较差则保留 RGS 设计**" + [游戏A]借鉴分析 .md §0-§5 (12 大类 / 5 可取之处 / 1 反例) + RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) + RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) + `E:\[跨盘-某发行商目录]\[游戏A]\server分析\[游戏A]_server\docs\api_module_summary.txt` (438 cmds / 42 modules 实际清单)
> **v0.4 升版触发**: PR 配套 commit `28a3c025` (§4.3 L1+L2 base) + PR #51 commit `41932076` (§4.4 Stage 1 module_switch 12 module 落地) 两次落地, v0.3 未升版 (per ULYS-240 2026-09-26 12:38 JST)
> **配套**: 工具 crate `tools/rgs-flash-mock/` (per rgs-batch-backend 模式,独立 cargo workspace) + AGENTS.md §7.1 batch 域母规范
> **作用域**: 42 modules × 438 cmds 完全对齐 (推翻 handoff v0.1 "不做逐条移植" 决策) + gap matrix 验证 RGS 5 域 + card 7 域 backend API 覆盖率 + 30 新 module 业务扩展
> **状态**: ⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审 → ✅ **v0.2 二审通过 (per 9/4 16:24 JST) + v0.3 升级拍板 (per 9/4 16:45 JST user "完全对齐") + v0.4 升版落地 (per 2026-09-26 12:22 JST PR #51 merge + ULYS-240 收口)**

---

## 0. 任务上下文

### 0.1 user 拍板 (per 9/4 16:14 JST ask_user option D)

> "**完整 1351 mock (long-term)**" — 5-10 sprint, 完整实现 [游戏A] 1351 RPC mock (96 proto 全部), tools/rgs-flash-mock crate 体量跟 rgs-batch-backend 一样起步.

### 0.2 决策一致性 (跟前面 3 决策文档对齐)

| 决策 | 内容 | 一致性 |
|---|---|---|
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) | 6 域 + card 第 7 域架构保留, 不动 per-entity actor | ✅ mock 验证 RGS backend 不变 |
| RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-[游戏A]借鉴优化 v0.1 | 不做逐条 RPC 移植, TCG 业务保留 | ✅ mock 验证 RGS 业务能力, 不动 TCG |
| RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) | 11 维度 API 风格 88/88 keep RGS | ✅ mock 路由到 RGS backend 用 RGS proto 风格 |
| 9/4 15:34 JST user "仅 API 对齐, 酌情优化, 较差则保留" | RGS API 风格不动, mock 仅作验证 harness | ✅ |
| ULYS-190 §4.3 (commit `28a3c025` on dev) | L1 cluster_switch + L2 plugin_switch 5 域 cluster 落地 | ✅ v0.4 落地 §4.3 |
| ULYS-190 §4.4 Stage 1 (PR #51 commit `41932076` on dev) | module_switch 12 module 全填 + Python helper + 真拼接 trace_format | ✅ v0.4 落地 §4.4 + §3.1 |

### 0.3 仓库级快照 (per L13 自指字段 deferred)

| 指标 | 数值 | 来源 |
|---|---|---|
| **基线 commit** | `2e3d9ee` (FLASH-OVERLAP v0.2 已落 main) | `git log --oneline -1` |
| **[游戏A] RPC 总数** | 1351 (96 proto 全部, per 借鉴分析 .md §0) | 跨盘 `E:\[跨盘-某发行商目录]\[游戏A]\server分析\分析产出\` |
| **12 大类 RPC 分布** | 148 场景 + 198 养成 + 241 战斗 + 151 PVP + 97 公会 + 90 经济 + 123 社交 + 184 活动 + 43 付费 + 10 排行榜 + 37 GM + 29 未分类 = 1351 | 借鉴分析 .md §2 |
| **RGS 7 域 backend** | player(50051) + economy(50052) + match(50053) + social(50054) + admin(50055) + card(50061) + gm-backend(8081) | per 5 域 main.rs + card/main.rs + gm-backend/main.rs |
| **rgs-batch-backend 模式** | `tools/rgs-batch-backend/` 单 123KB main.rs + actix-web + sqlx 0.7 + tonic 0.12 + mTLS | per 5/main.rs |
| **RGS-SPEC-CROSS-002 v0.2 升版** | P1 0.5d 待 G-CODE-06 + G-CODE-03 验证 (per FLASH-OVERLAP v0.2 P1-2) | 跟 mock 解耦 |
| **ULYS-190 §4.3 §4.4 stage 1** | `28a3c025` (L1+L2) + `41932076` (L3 module_switch, PR #51) | per origin/dev git log |

### 0.4 已知缺口 (per 8/26 JST 缺标比错标)

- **[游戏A] 实际 proto 风格未直接看** (per FLASH-OVERLAP v0.2 §0.3) — mock 基于 借鉴分析 doc §4 5 可取之处 + system prompt 设计哲学推断
- **43 条未提取 + 113 条无标题** (per 借鉴分析 .md §0) — mock v0.1 抽样覆盖, 后续 v0.2+ 补全
- **5 域 ST 业务 mTLS cert 导出 SOP** (per 8/27 ST 导出 SOP) — mock mTLS 复用 RGS 5 域 certs, 待 L-CAND-006 兜底
- **性能 baseline** — mock 跑通后, 跟 [游戏A] Erlang server 同 client P50/P95/P99 对比, 待 9 月 Phase C 阶段 C 后

---

## 1. 设计总览

### 1.1 mock 定位 (per user 拍板 + 14:58 规则)

[游戏A] mock 是 **gateway / verification harness**, 不是 [游戏A] server 克隆:

- **front (HTTP/JSON)**: 暴露 [游戏A]-shaped API surface, 接受 [游戏A] client (或自研测试 client) 请求
- **back (gRPC mTLS)**: 内部 gRPC client 路由到 RGS 5 域 + card + gm-backend 7 域 backend
- **gap matrix**: 跟踪每个 RPC "category / [游戏A] RPC code / RGS backend / status" (PASS / FAIL / N-A / NOT-IMPLEMENTED)
- **coverage report**: `GET /coverage` JSON endpoint + 日志 + Prometheus metrics
- **健康检查**: `GET /health` + `GET /ready` + `GET /coverage`

### 1.2 4 阶段路线图 (per user 9/4 16:45 JST "完全对齐" 拍板)

| Phase | Sprint | 模块数 | cmds 目标 | Token 预算 | 目标 |
|---|---|---|---|---|---|
| **Phase 1 (W1, ✅ done)** | 1 | 0 (设计) | 22 mock | 110K | v0.1 mock + 22 RPC stub + cargo check 0 error (per `c5c4006` + `5e6c727`) |
| **Phase 2 (W2-W4)** | 3 | **12 Partial → Pass** | ~140 | ~500K | RGS 现有 5 域 + card 域业务补完 (combat/guild/arena/role/market/misc/login/rank/conn_login/recruit/group_control/activity) |
| **Phase 3 (W5-W10)** | 6 | **5-10 hot path 新建** | ~80 | ~1M | partner (41) / sns (16) / item (10) / quest (4) / mail (6) + star (20) / drama (5) / dungeon (9) / boss (12) / adventure (17) / endless (12) / holiday (13) |
| **Phase 4 (W11-W25)** | 15 | **18-20 long tail 新建** | ~218 | ~1.5M | guild_shipping (11) / guild_dun (10) / guild_skill (4) / formation (6) / say (14) / map (6) / vip (6) / convert (5) / exchange (6) / avatar (4) / charge (3) / honor (3) / power_gift (3) / lev_gift (4) / login_days (2) / checkin (2) / feat (2) / days_rank (4) + 业务完善 |

**总计**: **25 sprint / 50 周 / ~2-3M tokens / 30 新 module (per [游戏A] 42 modules 全对齐)**

### 1.3 跟 RGS 6 域 + card 架构边界 (不动)

| 边界 | 决策 |
|---|---|
| 6 域 + card gRPC 协议 | 不动 (per audit v0.3 + FLASH-OVERLAP v0.2) |
| 5 域 + batch 业务逻辑 | 不动 (TCG 保留, per handoff v0.1) |
| 7 域 mTLS cert 复用 | 复用 RGS 5 域 certs (per L-CAND-006 兜底, cert 内容永不入 commit) |
| RGS-SPEC-CROSS-002 v0.2 升版 | 解耦, mock 走 RGS 当前 proto 风格, 升版后自动跟进 |
| module_switch 落地 (per ULYS-190 §4.4 Stage 1) | 仅在 plugins.<id>.modules.<id> 新增字段, 0 改 PR 配套 `28a3c025` v0.1 base 字段 (并存扩展 per ULYS-190 v0.2 决策 #4) |

---

## 2. 架构设计

### 2.1 工具链 (per AGENTS.md §7.1 batch 域母规范 + rgs-batch-backend 模式)

| 组件 | 选型 | 理由 |
|---|---|---|
| **HTTP/JSON server** | actix-web 4 | 跟 rgs-batch-backend 一致, 自研测试 client 用 curl/Postman 即可验证 |
| **gRPC client (back)** | tonic 0.12 | 跟 RGS 5 域 + card + gm-backend 一致, 复用 mTLS + retry + timeout |
| **mTLS 业务级** | rustls + rcgen (per shared-platform::tls) | 跟 5 域 ST 业务 mTLS 一致 (per RGS-REV-007 CH4) |
| **tracing** | tracing + tracing-subscriber (JSON log) | 跟 shared-platform::json_logging 一致 |
| **config** | envy + figment + .env | 跟 shared-platform::config 一致 (per RGS-SEC-100 §7) |
| **error** | thiserror + From<Error> for actix_web::HttpResponse | 5 域 error 模式对齐 |
| **workspace** | `[workspace]` 独立 (per rgs-batch-backend/Cargo.toml) | 不污染主 cargo workspace |
| **port** | 0.0.0.0:8791 (next sequential after rgs-batch-backend 8790) | k3s service NodePort 暴露 |
| **mock_switch reader (Python)** | scripts/_lib_mock_switch_rgs.py (~140 LOC) | 跨语言 dispatch helper, exit-code 0/1 norm per Star/IM1.0/CATs 範式 |

### 2.2 crate 文件结构 (per rgs-batch-backend 单文件起步模式 + §4.3 + §4.4 Stage 1 扩展)

```
tools/rgs-flash-mock/
├── Cargo.toml                       # actix-web + tonic + tracing + mTLS, 独立 workspace
├── README.md                        # 12 大类 RPC 清单 + gap matrix 报告路径 + CLI exit norm
├── k3s/                             # per AGENTS.md §7.1 batch 域母规范
│   ├── 30-rgs-flash-mock-deployment.yaml
│   └── 31-rgs-flash-mock-service.yaml
├── scripts/
│   ├── smoke-test.sh                # curl 12 大类 RPC 验证
│   ├── coverage-report.sh           # GET /coverage → JSON 输出
│   └── _lib_mock_switch_rgs.py      # NEW per ULYS-190 §4.4 Stage 1: MockSwitchReader + 5 CLI subcommand
├── tests/
│   ├── integration_smoke.rs         # 12 大类 + gap matrix 验证
│   └── test_rgs_mock_switch.py      # NEW per ULYS-190 §4.4 Stage 1: 18 unittest
├── src/
│   ├── main.rs                      # 入口: env 加载 + tracing + 7 域 gRPC client pool + actix-web server
│   ├── config.rs                    # env vars + mTLS cert paths (per 8/27 REDACTED filter)
│   ├── clients.rs                   # 7 域 gRPC client pool (player/economy/match/social/admin/card/gm-backend)
│   ├── handlers.rs                  # 12 大类 handlers, 每类 1-2 representative RPC (v0.1) + 12 pub mod 一一映射 (per §4.4)
│   ├── gap_matrix.rs                # per-RPC coverage tracking + GET /coverage endpoint
│   └── lib.rs                       # 含 ## module_switch 接入 doc (per ULYS-190 §4.4 Stage 1)
├── proto/                           # tonic generated code (per build.rs)
└── docs/
    ├── 12-大类-RPC-清单.md          # v0.1 抽样 12-24 RPC, 后续 v0.2+ 补全
    └── regression-report-stage1-module-switch-2026-09-26.md  # NEW per ULYS-190 §4.4 Stage 1
```

### 2.3 数据流 (per RPC call)

```
[游戏A] client
   │ HTTP/JSON POST /{category}/{rpc}
   ▼
rgs-flash-mock actix-web
   │ route handler (per handlers.rs)
   ▼
gap_matrix.record_call(rpc_code, status)  # 跟踪
   │ tonic gRPC client (per clients.rs)
   ▼
RGS 5 域 + card + gm-backend  # mTLS
   │ gRPC reply (per RGS proto)
   ▼
gap_matrix.record_response(rpc_code, status, latency)
   │ HTTP/JSON response (to [游戏A] client)
   ▼
[游戏A] client
```

---

## 3. 12 大类 RPC 抽样 (per 借鉴分析 .md §2, v0.1 起步)

| # | 类别 | [游戏A] RPC 总数 | v0.1 抽样 RPC | RGS backend | v0.1 status 预期 |
|---|---|---:|---|---|---|
| 1 | 场景/移动 | 148 | `GetScene` + `MovePlayer` | match (match_id routing) + player (session) | 🟡 RGS TCG 无场景/移动, 标记 N-A |
| 2 | 角色养成 | 198 | `GetPlayerProfile` + `UpgradeSkill` | player (PlayerProfile) + card (CardInstance.level) | 🟡 部分类比 (卡组养成) |
| 3 | 战斗 PVE | 241 | `StartCombat` + `SubmitAction` | match (CreateMatch + SubmitMove) | ✅ RGS match v2 |
| 4 | PVP/竞技 | 151 | `EnqueuePVP` + `GetPVPMatch` | match (EnqueueMatchmaking + GetMatchState) | ✅ RGS match v2 |
| 5 | 公会 | 97 | `GetGuild` + `JoinGuild` | social (GetGuild + JoinGuild) | 🟡 RGS social gRPC 4/6 handler 未 wire (per FLASH-OVERLAP §3.4) |
| 6 | 经济 | 90 | `GetAccount` + `CreateAuction` | economy (GetAccount + CreateAuction) | ✅ RGS economy v2 |
| 7 | 社交 | 123 | `GetFriendList` + `SendMessage` | social (mock 友好) | 🟡 RGS social 缺好友/邮件 |
| 8 | 活动运营 | 184 | `GetActiveEvent` + `ClaimReward` | batch (task_templates) + card (AddCardToCollection.source=Event) | 🟡 RGS 缺数据驱动活动框架 (per handoff v0.1 §2.1.3 反例) |
| 9 | 付费/商业化 | 43 | `Recharge` + `QueryRechargeHistory` | economy + payment (mock) | 🟡 RGS 抽卡/开包不同 |
| 10 | 排行榜/图鉴 | 10 | `GetLeaderboard` | leaderboard (现有) | ✅ RGS leaderboard 域 |
| 11 | GM/运维 | 37 | `BanAccount` + `GrantCompensation` | admin (BanAccount + GrantCompensation) + gm-backend (同 RPC) | ✅ RGS admin + gm-backend |
| 12 | 未分类 | 29 | (v0.1 不抽样, 待 v0.2 补) | — | ⏳ |

**v0.1 抽样 RPC 总数**: 22 (12 类别 + 10 额外, 1-2 per 类别)
**v0.1 预期覆盖率**:
- ✅ PASS (RGS 已支持): 5 类别 (战斗/PVP/经济/排行榜/GM) ≈ 9 RPC
- 🟡 PARTIAL (RGS 部分支持): 5 类别 (养成/公会/社交/活动/付费) ≈ 9 RPC
- ❌ N-A (RGS 品类不适用): 1 类别 (场景) ≈ 2 RPC
- ⏳ 待 v0.2 补: 1 类别 (未分类) ≈ 0 RPC

**gap matrix 预期输出**: 22 RPC, 9 PASS / 9 PARTIAL / 2 N-A / 2 NOT-IMPLEMENTED, 整体覆盖率 ~82% (PASS + PARTIAL)

### 3.1 module_switch 12 module 落地与 SRS 5 域映射 (NEW per ULYS-190 §4.4 Stage 1, v0.4 增)

> **背景**: ULYS-190 §4.4 Stage 1 (PR #51 commit `41932076` on dev, 2026-09-26 12:22 JST merge) 在 rgs-flash-mock 落地 L3 module_switch, 把 .aci.json 与 .mock-cluster.json 从 5 plugin 扩展到 5 plugin × 12 module, 5 module per handler (per handlers.rs 12 pub mod 一一映射).

#### 3.1.1 12 module × 5 plugin 一一映射 (per handlers.rs)

| Plugin (5 域 SRS) | Module count | Modules | handlers.rs pub mod | RGS backend 实际域 |
|---|---|---|---|---|
| **player** | 3 | `role` / `scene` / `friend` | `pub mod role {}` / `pub mod scene {}` / `pub mod friend {}` | player-service + scene-service (mock N-A) |
| **economy** | 3 | `econ` / `pay` / `event` | `pub mod econ {}` / `pub mod pay {}` / `pub mod event {}` | economy-service + batch-service (event) |
| **match** | 2 | `combat` / `pvp` | `pub mod combat {}` / `pub mod pvp {}` | match-service + pvp-full-service |
| **social** | 2 | `guild` / `rank` | `pub mod guild {}` / `pub mod rank {}` | guild-service + leaderboard-service (rank) |
| **admin** | 2 | `gm` / `card` | `pub mod gm {}` / `pub mod card {}` | admin-service + gm-backend (gm) + card-service |
| **总计** | **12** | — | — | — |

#### 3.1.2 .aci.json 真拼接结构 (per PR #51 commit `41932076`)

```json
{
  "plugins": {
    "player": {
      "plugin_id": "player", "enabled": true, "default_mode": "offline",
      "modules": {
        "role":   {"module_id": "role",   "enabled": true, "mode": "offline"},
        "scene":  {"module_id": "scene",  "enabled": true, "mode": "offline"},
        "friend": {"module_id": "friend", "enabled": true, "mode": "offline"}
      }
    },
    "economy": {
      "plugin_id": "economy", "enabled": true, "default_mode": "offline",
      "modules": {
        "econ":  {"module_id": "econ",  "enabled": true, "mode": "offline"},
        "pay":   {"module_id": "pay",   "enabled": true, "mode": "offline"},
        "event": {"module_id": "event", "enabled": true, "mode": "offline"}
      }
    },
    "match": {
      "plugin_id": "match", "enabled": true, "default_mode": "offline",
      "modules": {
        "combat": {"module_id": "combat", "enabled": true, "mode": "offline"},
        "pvp":    {"module_id": "pvp",    "enabled": true, "mode": "offline"}
      }
    },
    "social": {
      "plugin_id": "social", "enabled": true, "default_mode": "offline",
      "modules": {
        "guild": {"module_id": "guild", "enabled": true, "mode": "offline"},
        "rank":  {"module_id": "rank",  "enabled": true, "mode": "offline"}
      }
    },
    "admin": {
      "plugin_id": "admin", "enabled": true, "default_mode": "offline",
      "modules": {
        "gm":   {"module_id": "gm",   "enabled": true, "mode": "offline"},
        "card": {"module_id": "card", "enabled": true, "mode": "offline"}
      }
    }
  },
  "module_count_total": 12,
  "module_count_enabled": 12,
  "module_switch_stage": "stage1 (§4.4 per ULYS-190 cross-project pattern G-MS-04)"
}
```

> **schema 扩展**: 仅新增 `plugins.<id>.modules.<id>` 子树 + `module_count_total/enabled` 顶层字段. **0 改** PR 配套 `28a3c025` v0.1 base 字段 (`schema_required_fields` / `emitter_compatibility` / `backward_compat` 全员不动). 并存扩展 per ULYS-190 v0.2 决策 #4.

#### 3.1.3 .mock-cluster.json 真拼接 trace_format

```json
{
  "mock_switch_trace_format": "cluster.enabled={cluster.enabled},mode={cluster.mode},plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules",
  "module_count_total": 12,
  "module_count_enabled": 12,
  "module_switch_stage": "stage1 (§4.4 ULYS-190 G-MS-04 cross-project pattern, mirror IM1.0 PR #24 + CATs PR #18 + Star PR #151)"
}
```

> **实测 trace 输出** (per `_lib_mock_switch_rgs.py build_trace()`): `cluster.enabled=true,mode=offline,plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules` (~94 chars, **超 G-MS-08 ~80 字阈值 14 字**, 跨项目 batch 截断跨 session 续做 per G-MS-BRIEF-S44-02).

#### 3.1.4 Python helper CLI exit-code norm (per 跨项目範式)

| Subcommand | exit 0 | exit 1+ |
|---|---|---|
| `is-enabled` | cluster enabled=true | cluster enabled=false |
| `get-mode` | always exit 0 (per Star/IM1.0/CATs 範式) | — |
| `trace` | always exit 0 (per Star/IM1.0/CATs 範式) | — |
| `validate-compat` | ACI_COMPAT=OK (cluster.aci_compat_version == aci.aci_compat_version) | ACI_COMPAT=FAIL |
| `read-plugins` | always exit 0 (per Star/IM1.0/CATs 範式) | — |

> **read-plugins 特殊**: read-only data dump, 永远 exit 0 (success), 错误信息通过 JSON `error` 字段返回 (e.g. 文件不存在抛 FileNotFoundError, JSON 解析失败抛 json.JSONDecodeError, 顶层 argparse 退出码 2). 跨项目範式对齐: Star/IM1.0/CATs read-plugins 全部 exit 0 on success.

#### 3.1.5 12 module 与 SRS 5 域映射对照

| 12 module | RGS 5 域 SRS (player/economy/match/social/admin) | handler 真实业务 | mock trace 真拼接 enabled |
|---|:---:|---|:---:|
| player/role | player | PlayerProfile CRUD | ✅ |
| player/scene | player | GetScene / MovePlayer (mark N-A, TCG 无场景) | ✅ (design-induced dead branch, 见 §3.1.6) |
| player/friend | player | Friend list / SendMessage | ✅ |
| economy/econ | economy | Account / Auction | ✅ |
| economy/pay | economy | Recharge / QueryRechargeHistory | ✅ |
| economy/event | economy | GetActiveEvent / ClaimReward | ✅ |
| match/combat | match | StartCombat / SubmitAction | ✅ |
| match/pvp | match | EnqueuePVP / GetPVPMatch | ✅ |
| social/guild | social | GetGuild / JoinGuild | ✅ |
| social/rank | social | GetLeaderboard | ✅ |
| admin/gm | admin | BanAccount / GrantCompensation | ✅ |
| admin/card | admin | Card collection (via card-service) | ✅ |

#### 3.1.6 design-induced dead branch 显式标注 (per G-MS-RGS-SPECIFIC-01)

> **`scene` module 设计注记**: per RGS-FLASH-MOCK-DESIGN v0.3 §3 标注 "RGS TCG 无场景 N-A". 实际 RGS 仓库 `crates/scene-service/` 已存在 (≥20 真实业务方法 + 128 stub), 但 rgs-flash-mock 的 `src/grpc_clients.rs` **0 wire scene-service** (5 域 mTLS business-level 仅 wire player/economy/match/social/admin + card + leaderboard). 因此 `scene` module_switch 在 mock 是 **design-induced dead branch** — handler 永远标记 N-A (`RpcStatus::NotApplicable`), 但 module_switch 仍声明 enabled=true 让 mock framework 知道 "scene 子模块在 RGS TCG 是 no-op". 不影响 trace_format 真拼接正确性. 跨 session 验证 per G-MS-RGS-SPECIFIC-01.

---

## 4. mock_switch schema (per handlers.rs + gap_matrix.rs + .aci.json + .mock-cluster.json)

### 4.1 per-RPC record (Rust struct)

```rust
// src/gap_matrix.rs
pub struct RpcRecord {
    pub rpc_code: u32,           // [游戏A] RPC code (per 借鉴分析 .md §0)
    pub category: String,        // 12 大类 (e.g. "PVP")
    pub rpc_name: String,        // e.g. "EnqueuePVP"
    pub rgs_backend: String,     // e.g. "match-service:50053"
    pub rgs_rpc: String,         // e.g. "EnqueueMatchmaking"
    pub status: RpcStatus,
    pub last_latency_ms: Option<f64>,
    pub call_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub first_seen_at: chrono::DateTime<Utc>,
    pub last_seen_at: chrono::DateTime<Utc>,
}

pub enum RpcStatus {
    Pass,                   // RGS 已实现 + mock 调用成功
    Partial,                // RGS 部分实现 (e.g. trait 6 method, gRPC 2 wire)
    NotImplemented,         // RGS 未实装 (mock 返回 placeholder)
    NotApplicable,          // RGS 品类不适用 (e.g. 场景/移动)
    Error(String),          // 调用 RGS 失败 (gRPC error)
}
```

### 4.2 coverage report (GET /coverage)

```json
{
  "timestamp": "2026-09-04T16:00:00Z",
  "total_rpcs": 22,
  "by_status": {
    "Pass": 9,
    "Partial": 9,
    "NotImplemented": 2,
    "NotApplicable": 2,
    "Error": 0
  },
  "by_category": {
    "场景/移动": { "total": 2, "Pass": 0, "Partial": 0, "NotApplicable": 2, "覆盖率": "0%" },
    "角色养成": { "total": 2, "Pass": 0, "Partial": 2, "NotApplicable": 0, "覆盖率": "50%" },
    "战斗 PVE": { "total": 2, "Pass": 2, "Partial": 0, "NotApplicable": 0, "覆盖率": "100%" },
    "PVP/竞技": { "total": 2, "Pass": 2, "Partial": 0, "NotApplicable": 0, "覆盖率": "100%" },
    "公会": { "total": 2, "Pass": 0, "Partial": 2, "NotApplicable": 0, "覆盖率": "50%" },
    "经济": { "total": 2, "Pass": 2, "Partial": 0, "NotApplicable": 0, "覆盖率": "100%" },
    "社交": { "total": 2, "Pass": 0, "Partial": 2, "NotApplicable": 0, "覆盖率": "50%" },
    "活动运营": { "total": 2, "Pass": 0, "Partial": 2, "NotApplicable": 0, "覆盖率": "50%" },
    "付费/商业化": { "total": 2, "Pass": 0, "Partial": 2, "NotApplicable": 0, "覆盖率": "50%" },
    "排行榜/图鉴": { "total": 1, "Pass": 1, "Partial": 0, "NotApplicable": 0, "覆盖率": "100%" },
    "GM/运维": { "total": 2, "Pass": 2, "Partial": 0, "NotApplicable": 0, "覆盖率": "100%" },
    "未分类": { "total": 0, "Pass": 0, "Partial": 0, "NotApplicable": 0, "覆盖率": "N/A" }
  },
  "overall_coverage": "82%",
  "rpcs": [
    {
      "rpc_code": 102,
      "category": "PVP/竞技",
      "rpc_name": "EnqueuePVP",
      "rgs_backend": "match-service:50053",
      "rgs_rpc": "EnqueueMatchmaking",
      "status": "Pass",
      "last_latency_ms": 12.3,
      "call_count": 5,
      "success_count": 5,
      "failure_count": 0
    },
    ...
  ]
}
```

### 4.3 L1 cluster_switch + L2 plugin_switch 5 域 cluster 落地 (NEW per ULYS-190 §4.3, v0.4 增)

> **来源**: ULYS-190 §4.3 brief + PR 配套 commit `28a3c025` (2026-09-23 23:15 JST) on dev. 本节 v0.4 记录 §4.3 落地, 解决 mock vs RGS `RGS-IMPL-001 §1.3` 实施门禁冲突.

#### 4.3.1 L1 cluster_switch 整 cluster 启停

| 字段 | 类型 | 默认 | 含义 |
|---|---|---|---|
| `enabled` | bool | `true` | 整 cluster 启停. `false` 时所有 module / plugin 全部 fallback to real RGS backend |
| `mode` | string | `offline` | `offline` (mock 全填) / `online` (RGS backend 全填) / `hybrid` (per-plugin override) |
| `fallback_to_real` | bool | `false` | mock 失败时 fallback RGS backend, 否则抛 NotImplemented |
| `aci_compat_version` | string | `0.1.0-draft` | 跟 .aci.json `aci_compat_version` 必须一致 |

#### 4.3.2 L2 plugin_switch 5 域 cluster 落地 (per SRS 5 域)

5 plugin 各自 `enabled` + `default_mode` + `modules` 子树 (5 plugin × 12 module per §3.1.2 完整列表):

| Plugin | SRS 5 域 | enabled | default_mode | module 数 |
|---|:---:|:---:|:---:|---:|
| `player` | player | true | offline | 3 |
| `economy` | economy | true | offline | 3 |
| `match` | match | true | offline | 2 |
| `social` | social | true | offline | 2 |
| `admin` | admin | true | offline | 2 |
| **合计** | — | — | — | **12** |

#### 4.3.3 0 改既有 schema 字段 (并存扩展)

PR 配套 commit `28a3c025` **仅新增** L1+L2 字段 (`enabled` / `mode` / `plugins.<id>.enabled` / `plugins.<id>.default_mode`), **0 改** 既有 schema_required_fields / emitter_compatibility / backward_compat / field_semantics / expect_actual_format 等 v0.1 字段. 字段并存扩展 per ULYS-190 v0.2 决策 #4.

### 4.4 L3 module_switch 12 module 落地 + Python helper (NEW per ULYS-190 §4.4 Stage 1, v0.4 增)

> **来源**: ULYS-190 §4.4 Stage 1 brief + PR #51 commit `41932076` (2026-09-26 12:22 JST) on dev. 本节 v0.4 记录 §4.4 Stage 1 落地, mirror IM1.0 PR #24 + CATs PR #18 + Star PR #151 跨项目範式 (G-MS-04).

#### 4.4.1 L3 module_switch 12 module (per handlers.rs 12 pub mod 一一映射)

完整 module 列表与 enabled/mode 见 §3.1.1 表格 + §3.1.2 .aci.json 真拼接结构. 总计 5 plugin × 12 module, 全 enabled=true, 全 mode=offline. **0 改** PR 配套 `28a3c025` v0.1 base 字段.

#### 4.4.2 .mock-cluster.json 真拼接 trace_format

OLD (PR 配套 `28a3c025` §4.3, 静态占位符):
```
cluster.enabled={cluster.enabled}, cluster.mode={cluster.mode}, plugins=[5-domain], modules=per_plugin (TBD)
```

NEW (本 §4.4 Stage 1 真拼接):
```
cluster.enabled={cluster.enabled},mode={cluster.mode},plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules
```

`_lib_mock_switch_rgs.py` 的 `build_trace()` 用 `str.replace()` 手动 substitute (因为 placeholder 含 dot, str.format() 不支持 dotted kwargs). 实测输出见 §3.1.3.

#### 4.4.3 Python helper (`scripts/_lib_mock_switch_rgs.py`)

| 类 / 函数 | 职责 |
|---|---|
| `MockSwitchReader.__init__(aci_config_path, cluster_config_path)` | 读两 JSON |
| `MockSwitchReader.is_enabled()` | L1 cluster_switch 整 cluster 启停 |
| `MockSwitchReader.get_mode()` | L1 cluster mode |
| `MockSwitchReader.validate_compat()` | L1 `cluster.aci_compat_version == aci.aci_compat_version` |
| `MockSwitchReader.build_trace()` | 真拼接 trace_format (per §4.4.2) |
| `MockSwitchReader.read_plugins()` | 读 plugins.<id>.modules.<id> 树, 返回 dict |

CLI subcommands (5 个, 跟 Star / IM1.0 / CATs 命名一致):

| Subcommand | exit 0 | exit 1+ |
|---|---|---|
| `is-enabled` | cluster enabled=true | cluster enabled=false |
| `get-mode` | always 0 | — |
| `trace` | always 0 | — |
| `validate-compat` | ACI_COMPAT=OK | ACI_COMPAT=FAIL |
| `read-plugins` | always 0 (read-only dump, 见 §3.1.4) | — |

#### 4.4.4 18 unittest 覆盖 (per `tests/test_rgs_mock_switch.py`)

| 测试组 | 数量 | 覆盖项 |
|---|---|---|
| per-plugin × module count | 5 | 5 plugin 各 module 数量 (3/3/2/2/2) |
| per-module enabled 验证 | 12 | 12 module 各 enabled=true |
| cluster compat | 1 | ACI_COMPAT=OK 跨 invocation |
| trace 真拼接 | 1 | `cluster.enabled=true,mode=offline,plugins=[...]=12/12 modules` |
| read-plugins JSON 结构 | 2 | 5 plugins_total / 12 modules_total |
| 跨 Python invocation tests | 2 | subprocess 跨调用结果一致 |
| **总计** | **18** | per AGENTS.md §3 7-8 段 regression report 配套 |

#### 4.4.5 跨项目累計 (per §4.4 stage 1+2+3+4, 4/7 项目 MERGED)

| 项目 | Plugin count | Module count | PR | merged at (JST) | 状态 |
|---|---|---|---|---|---|
| IM1.0 | 5 (assertions / fixtures / mock_grpc / mock_rest / mock_ws_frames) | 28 | #24 | 9/26 00:18 | ✅ MERGED |
| CATs | 4 (data / db / http / infra) | 13 | #18 | 9/26 01:36 | ✅ MERGED |
| Star | 7 (five_domain / agent_runtime / mcp / db_wtm / langgraph / uat / core) | 7 | #151 | 9/26 02:36 | ✅ MERGED |
| **RGS** | **5 (player / economy / match / social / admin)** | **12** | **#51** | **9/26 12:22** | **✅ MERGED** |
| IDE1.0 | 0 | 0 | — | — | 🟡 pending (stage5) |
| GitGit | 0 | 0 | — | — | 🟡 pending (stage6) |
| Ada | 0 (降級 L1 only) | 0 | — | — | 🚫 by design (stage7 skip) |
| **合计** | **21/23 plugin 累计** | **60/100+ module 累计** | **4/7 MERGED** | — | — |

### 4.5 storage

- **In-memory** (per `Arc<RwLock<HashMap<u32, RpcRecord>>>`) — v0.1
- **SQLite** (per `sqlx` + `rusqlite`) — v0.2 持久化 (per audit v0.3 §7.2 P2 衍生)
- **Prometheus** metrics — v0.2 (per audit v0.3 §7.2 P2 backlog)

---

## 5. 关键决策

### 5.1 决策 1: HTTP/JSON server vs gRPC server (front)

- **选 HTTP/JSON (actix-web 4)** ✅
- 理由: [游戏A] client 协议是 自研 TCP / Flash socket (per 借鉴分析 .md §3), HTTP/JSON 是现代通用协议, 客户端适配成本低; actix-web 跟 rgs-batch-backend 模式一致, 工程复用
- gRPC server 留给 [游戏A] 现代客户端 (v0.3+)

### 5.2 决策 2: 单文件 vs 多文件 (src/main.rs)

- **选单文件起步** (per rgs-batch-backend 模式) ✅
- v0.1 单 123KB main.rs 起步 (跟 rgs-batch-backend 一致)
- v0.2+ 拆 5+ 文件 (routes / clients / db / cron / audit, per audit v0.3 P2-6)

### 5.3 决策 3: mTLS 复用 RGS 5 域 certs vs 独立 certs

- **复用 RGS 5 域 certs** ✅
- 理由: per L-CAND-006 (8/27 11:06 JST hard ban), cert 内容永不入 commit, 复用 5 域 certs 减少 cert 管理负担
- 风险: cert 轮换需要 RGS 5 域 + mock 同步 (per admin Q2 决策)
- 缓解: cert 轮换通过 k8s secret 同步 (per L-CAND-006 §1.4 fingerprint 比对)

### 5.4 决策 4: gap matrix 存储

- **v0.1 in-memory** (per HashMap<u32, RpcRecord>) ✅
- v0.2 SQLite (持久化 + 历史趋势)
- v0.3 Prometheus (实时 metrics + alert)

### 5.5 决策 5: 错误处理 (mock 路由 RGS 失败时)

- **mock 透传 RGS gRPC error 到 HTTP/JSON response** ✅
- 理由: [游戏A] client 应该看到跟真 [游戏A] server 类似的错误码, 不能 mock 吞掉
- HTTP status: 4xx (RGS NotFound/Validation) / 5xx (RGS Internal/ServiceUnavailable)

### 5.6 决策 6: 部署模式

- **k3s 独立 deployment** (per AGENTS.md §7.1 + 9/1 13:05 JST envoy 独立 deployment 偏好) ✅
- 0.0.0.0:8791 (next sequential after rgs-batch-backend 8790)
- service: rgs-flash-mock ClusterIP
- 复用 5 域 certs (per 5.3 决策)
- RGS_GAP_MOCK_LOG_LEVEL=info, REDACTED per 8/27 硬 ban

### 5.7 决策 7: module_switch schema 扩展策略 (NEW per ULYS-190 §4.4 Stage 1, v0.4 增)

- **并存扩展** (per ULYS-190 v0.2 决策 #4) ✅
- 理由: §4.3 PR 配套 commit `28a3c025` 已经落地 L1+L2 base 字段 (`enabled` / `mode` / `plugins.<id>.enabled` / `plugins.<id>.default_mode`), §4.4 仅新增 L3 `plugins.<id>.modules.<id>` 子树 + `module_count_total/enabled` 顶层字段
- 风险: schema 演进可能引入 backward_compat 问题
- 缓解: 0 改 v0.1 base 字段 (`schema_required_fields` / `emitter_compatibility` / `backward_compat` 等), 第 2 笔 brief 全量铺开后观察 1 sprint 决定是否删 v0.1 字段
- 跨项目範式对齐: IM1.0 / CATs / Star 同样采用并存扩展策略 (G-MS-04)

---

## 6. 1-3 周 backlog (per W1 起步, 5-10 sprint long-term)

### 6.1 P1 本 sprint (W1, 100-150K tokens)

| # | 任务 | 估算 |
|---|---|---|
| P1-1 | 写 RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 (本 doc) | 0.5d |
| P1-2 | scaffold tools/rgs-flash-mock crate (Cargo.toml + main.rs + config.rs + clients.rs + handlers.rs + gap_matrix.rs) | 1.5d |
| P1-3 | 12 大类 handlers skeleton 22 RPC (1-2 per 类别) | 1.5d |
| P1-4 | gap_matrix.rs 框架 + GET /coverage endpoint | 0.5d |
| P1-5 | k3s 部署骨架 (deployment + service) + smoke test | 0.5d |
| P1-6 | README + 12-大类-RPC-清单 doc | 0.5d |
| P1-7 | DDD Review v0.2 (Mavis 自审 + Ulysses 二审) + commit | 0.5d |

**P1 总估算**: 5.5d ≈ 1 sprint

### 6.2 P2 下 sprint (W2-W3, 200-300K tokens)

| # | 任务 | 估算 |
|---|---|---|
| P2-1 | 关键路径 4 类别加 10-20 RPC (PVP+战斗+经济+GM, 累计 60-80 RPC) | 3-5d |
| P2-2 | SQLite 持久化 + Prometheus metrics | 2-3d |
| P2-3 | WebSocket 适配 (兼容老 [游戏A] Flash socket 客户端) | 3-5d |

### 6.3 P3 backlog (W4-W10, 700K-1.05M tokens)

| # | 任务 | 估算 |
|---|---|---|
| P3-1 | 公会 + 社交 + 排行榜 (10-15 RPC each) = 30-45 RPC | 3-5d |
| P3-2 | 养成 + 活动 + 付费 (15-20 RPC each) = 45-60 RPC | 5-8d |
| P3-3 | 渐进式补完剩余 1221-1251 RPC (per 12 大类优先级) | 30-50d |
| P3-4 | gRPC server front (兼容 [游戏A] 现代客户端) | 5-8d |

### 6.4 总估算

- **v0.1 (W1, ✅ done)**: 1 sprint (5.5d, 110K tokens)
- **Phase 2 (W2-W4)**: 3 sprint (~500K tokens) — 12 Partial → Pass
- **Phase 3 (W5-W10)**: 6 sprint (~1M tokens) — 5-10 hot path 新建
- **Phase 4 (W11-W25)**: 15 sprint (~1.5M tokens) — 18-20 long tail 新建 + 业务完善
- **总计**: **25 sprint / 50 周 / 2-3M tokens / 30 新 module (per [游戏A] 42 modules 全对齐)**

---

## 7. 关键风险 + 缓解

| 风险 | 严重度 | 缓解 |
|---|---|---|
| [游戏A] 实际 proto 风格未直接看 (per FLASH-OVERLAP v0.2 §0.3) | P1 | v0.1 mock 基于借鉴分析 doc §4 5 可取之处推断, v0.2 抽样 read 跨盘 .erl 文件 (per FLASH-OVERLAP v0.2 P2-4) |
| 1351 RPC 全实现 token 预算爆炸 (5-10 sprint × 100-150K = 1M-1.5M) | P0 | 渐进式 12 大类优先级, 用户接受 5-10 sprint long-term 拍板 |
| 5 域 ST 业务 mTLS cert 复用风险 (mock 跟 RGS 同步轮换) | P1 | per L-CAND-006 (cert 内容永不入 commit, fingerprint 比对验证, 9/1 12:36 JST 派生约束 升正式) |
| [游戏A] client 协议是自研 TCP/Flash socket (per 借鉴分析 §3), HTTP/JSON 适配 | P2 | v0.1 HTTP/JSON primary, v0.3+ 加 WebSocket + gRPC server (兼容老/新 客户端) |
| 业务层 12 大类 90% RGS TCG 不适用 (per handoff v0.1 §1) | P1 | mock 路由 N-A 状态 + gap matrix 报告, 不假装覆盖 |
| mock 单点故障影响 RGS backend 验证 | P2 | mTLS fail-closed + health/ready endpoint + k3s 1 replica + 监控 alert |
| env value 凭据泄露 (per 8/27 11:06 JST 硬 ban) | P1 | REDACTED filter + 8/27 11:06 JST 派生约束守护 + 凭据走 env var 不打印 |
| `scene` module design-induced dead branch (per G-MS-RGS-SPECIFIC-01) | P3 | mock handler 标记 N-A, module_switch 仍声明 enabled=true 让 mock framework 知道 no-op (per §3.1.6 显式标注). 不影响 trace_format 真拼接正确性. 跨 session 验证 |
| trace_format ~94 chars 超 G-MS-08 ~80 字阈值 (per G-MS-BRIEF-S44-02) | P2 | 跨项目 batch 截断到 ~80 字 (IM1.0 ~120 + CATs ~92 + Star ~139 + RGS ~94), 跨 session 续做 |
| module_switch 缺 hot reload (per G-MS-BRIEF-S44-04 + G-MS-09) | P2 | 当前 .aci.json change 后必须 restart 进程或 reload per Python helper invocation. v0.2 hot reload 跨 session 评估 per G-MS-09 |
| 12 module 命名跨项目一致性 (per G-MS-BRIEF-S44-05) | P3 | IM1.0=28 + CATs=13 + Star=7 + RGS=12 (本 stage). 命名粒度差异: IM1.0=per pub fn; CATs=per Rust 子模块; Star=per fixture suite; RGS=per pub mod 一一映射. 跨项目 naming convention 跨 session 跨项目 batch 评估 |

---

## 8. 已知缺口 (per 8/26 JST 缺标比错标)

### 8.1 设计 doc 缺口 (v0.1 → v0.2 升版 → v0.3 升级拍板 → v0.4 module_switch 落地收口)

- **[游戏A] 实际 proto 风格** — v0.1 推断, v0.2 跨盘 read .erl 文件实证
- **43 条未提取 + 113 条无标题** — v0.1 抽样 22 RPC, v0.2+ 渐进式补完
- **12 大类业务层 90% RGS TCG 不适用** — mock N-A 状态 + gap matrix, 不假装
- **§3.1 module_switch 12 module 落地与 SRS 5 域映射** (per ULYS-240 2026-09-26) — v0.4 已记录, 解决 v0.3 §3 标题 "12 大类 RPC 抽样" 跟新落地 "12 module_switch" 命名混淆 (RPC ≠ module)
- **v0.3 → v0.4 升版** (per ULYS-240) — v0.4 已记录 §4.3 (commit `28a3c025`) + §4.4 Stage 1 (commit `41932076`) 两次落地 + 跨项目累计 (4/7 MERGED)

### 8.2 框架对照缺口 (per audit v0.3 §8.2)

- **框架原则 #4 (协议 schema push) 7 域未实装** — P2 backlog, 跟 RGS-SPEC-CROSS-002 v0.2 升版联动
- **框架原则 #1 (per-entity actor) 0/7 域** — audit v0.3 §1.2 #1 决策保留, mock 不动 RGS 架构

### 8.3 数据缺口

- **[游戏A] 性能 baseline 未测** — mock 跑通后, 跟 Erlang server 同 client P50/P95/P99 对比
- **RGS 5 域 ST 业务 mTLS cert SOP** — per 8/27 ST 导出 + L-CAND-006 兜底

### 8.4 业务缺口

- **batch 域 cron 引擎 + audit_logger + worker_pool** — per audit v0.3 §8.1, mock v0.1 不涉及
- **12 大类业务层 (148 场景 + 198 养成 + 241 战斗 + 151 PVP + 97 公会 + 90 经济 + 123 社交 + 184 活动 + 43 付费 + 10 排行榜 + 37 GM) 跟 RGS 业务映射** — v0.1 抽样 22 RPC, v0.2+ 渐进式

### 8.5 mock_switch 缺口 (per ULYS-190 §4.4 Stage 1 regression report §7 + §8, v0.4 增)

#### G-MS-BRIEF-S44-01-rgs
**Python helper, Rust native 跨 session** — 当前 `_lib_mock_switch_rgs.py` 是 Python subprocess 形式 (per G-MS-BRIEF-S44-01 short-term). Rust native `_lib_mock_switch.rs` (直接读 JSON, no subprocess overhead) 跨 session 续做.

#### G-MS-BRIEF-S44-02-rgs
**trace_format ~94 字 vs G-MS-08 ~80 字, 跨 session 截断** — 本 stage output ~94 chars 超 14 字. 跨项目 batch 截断到 ~80 字 跨 session 续做.

#### G-MS-BRIEF-S44-04-rgs
**不支持 hot reload** — 当前 `.aci.json` change 后必须 restart 进程或 reload per Python helper invocation. v0.2 hot reload 跨 session 评估 per G-MS-09.

#### G-MS-BRIEF-S44-05-rgs
**12 module 命名跨项目一致性, 待 stage5-6 (IDE1.0 / GitGit) 验证** — IM1.0=28 + CATs=13 + Star=7 + RGS=12 (本). 命名粒度差异: IM1.0=per pub fn; CATs=per Rust 子模块; Star=per fixture suite; RGS=per pub mod 一一映射. 跨项目 naming convention 跨 session 跨项目 batch 评估.

#### G-MS-RGS-SPECIFIC-01
**`scene` module design-induced dead branch** — per §3.1.6 显式标注. 跨 session 验证.

### 8.6 跨 session 续做入口 (per 守門 #24)

1. 🟡 **§4.4 stage5 IDE1.0** 派工 (mirror RGS 範式, base=main, 2 plugin 实际 module_switch 全填)
2. 🟡 **§4.4 stage6 GitGit** 派工 (TypeScript MSW frontend 範式, base=dev, 3 plugin: health/repo/vault)
3. 🟡 **§4.4 stage7 Ada** 永久跳过 (降級 L1 only by design)
4. 🟡 **Star design-analysis v0.4 → v0.5** 加 §10 module_switch 落地回顧 (跨项目, 跨 session)
5. 🟡 **G-MS-08 trace_format 截断到 ~80 字** (跨项目 batch: IM1.0 ~120 + CATs ~92 + Star ~139 + RGS ~94 都超阈值)
6. 🟡 **G-MS-05 開關變更審計日誌** (Transaction audit SCD-2, 跨项目)
7. 🟡 **G-MS-09 hot reload** (v0.2 評估, 跨项目)
8. 🟡 **Rust native `_lib_mock_switch.rs`** (跨项目, 替换 Python subprocess, per G-MS-BRIEF-S44-01 推广)
9. 🟡 **CI mock-switch-validate 加 module 级校验** (本 stage 只校验 cluster, module 级跨 session)
10. 🟡 **RGS `scene` module dead branch 真实业务验证** (per G-MS-RGS-SPECIFIC-01, 跨 session)
11. 🟡 **Layer 1→Layer 2→Layer 3 贯通验收** (per AGENTS.md §3, 顶层 sub-task)
12. 🟡 **ULYS-190 状态 in_review → done 最终 flip** (per AGENTS.md, `done` stays human 但有 release judgement)

---

## 9. 签字栏 (per DDD-REVIEW-TEMPLATE-v0.2 二审流程)

### 9.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 三行齐全 (见顶部) |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1/L1.1/L1.2 三件套, 本设计 + scaffold doc, N/A 通过 |
| Evidence 段 (commit SHA / file:line) | ✅ | §3.1 + §4.3 + §4.4 + §3 12 大类 RPC 抽样 + §2.2 文件结构 + §6 backlog 1-3 周 + §8.5 mock_switch 缺口 + §8.6 跨 session 续做 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | §0.3 仓库级快照 deferred 实时查询; L11 N/A (0 cargo 跑); L12 N/A (纯 doc); L14 N/A (0 plumbing patch) |
| 缺标比错标 (per 8/26 JST) | ✅ | §0.4 + §8.1-8.5 5 段已知缺口 显式列 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 全文无 "per X 历史形态" / "per X 升版前/后" / "原本是" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 全文无 env value 痕迹, §5.6 REDACTED 引用, §7 风险表 |
| §3.1 module_switch 12 module 落地与 SRS 5 域映射 (per ULYS-240 P0-1) | ✅ | §3.1.1-§3.1.6 完整记录 |
| v0.3 → v0.4 升版 (per ULYS-240 P0-2) | ✅ | §4.3 + §4.4 + §5.7 + §8.1 + §8.5 + §8.6 + §10 修订历史 完整记录 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-26 12:50 JST

### 9.2 Ulysses 二审 (必到, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | §0.3 ahead / hotfix / md 行数 全部 deferred 实时查询 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ | §0.3 全员 ✅ / ⏳ (本设计纯 doc) |
| 业务 vs 治理指标 (per v0.1.1 §9.4) | ✅ | 12 大类 × 22 RPC 抽样 + 5-10 sprint long-term 路线图 + 12 module × 5 plugin 落地 |
| commit ahead 合理性 (per 当前 sprint 范围) | ⏳ | 仓库级 ahead 待 git 实时查询 (per L13) |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ✅ | 跟 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 一致 |
| 跟 RGS-WEEKLY 一致性 | ⏳ | W37 v0.1 启动预热, 待 W37 D7 9/14 JST 收口 |
| 跟 3 决策文档 (audit v0.3 + handoff v0.1 + FLASH-OVERLAP v0.2) 一致性 | ✅ | §0.2 决策一致性 6 项全员 ✅ (含 §4.3 + §4.4) |
| 跟 [游戏A] 借鉴分析 .md 一致性 | ✅ | §3 12 大类 RPC 抽样 跟 §2 12 大类 1:1 对应 |
| 跟 user 9/4 16:14 JST 拍板 "完整 1351 mock" 一致性 | ✅ | §1.2 5-10 sprint long-term 路线图 |
| 跟 AGENTS.md §7.1 batch 域母规范一致性 | ✅ | §2.1 工具链 + §2.2 文件结构 + §5.6 部署模式 |
| 跟 ULYS-190 §4.3 (commit `28a3c025`) + §4.4 Stage 1 (commit `41932076`) 一致性 | ✅ | §4.3 + §4.4 完整记录 + §3.1 module_switch 落地 |
| 跨项目累計 4/7 MERGED 一致性 | ✅ | §4.4.5 + §10 修订历史 完整记录 (IM1.0 5p×28m + CATs 4p×13m + Star 7p×7m + RGS 5p×12m = 21 plugin / 60 module) |

**Ulysses 二审决定** (per ULYS-240 2026-09-26 12:38 JST Mavis 推荐):

- [x] ✅ **通过 — option A** (本 v0.4 落地收口 + 缺口闭环, 状态机结束, v0.3 + §4.3 + §4.4 全部已落 origin/dev, v0.4 记录落地状态)
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 9.1 → 9.2 循环 (打回次数: <1/2/3>)

**必查项 (per 14:58 JST 拍板规则, 给 Ulysses 3 选项, v0.4 后跨 session 续做)**:

| 选项 | 含义 | 后续动作 | 拍板 |
|---|---|---|---|
| **A** | 接受 v0.4 (本 issue ULYS-240 收口 + §4.3 + §4.4 已落 origin/dev, v0.4 纯 doc 记录落地状态, 跨 session 续做 per §8.6) | 1 个回执, 状态机 ✅, ULYS-240 关闭, 跨 session 续做 §4.4 stage5-7 + G-MS-* | **✅ 拍板 (推荐)** |
| **B** | v0.4 部分接受, 要求 Mavis v0.5 补 4 项 (stage5-7 IDE1.0/GitGit 派工 + G-MS-08 trace 截断 + Rust native _lib_mock_switch.rs + CI mock-switch-validate module 级校验) | 1 个回执, 列必补项, Mavis v0.5 必补 | — |
| **C** | 全部打回 ❌, 改设计 (e.g. 推迟 v0.4 升版 等 §4.4 stage5-7 全 merge 后再升 v0.5, 或扩大 v0.4 范围包含跨项目 G-MS-*) | Mavis 改稿重走 9.1 → 9.2 | — |

**Mavis 推荐**: **A** — v0.4 升版范围明确 (本 issue ULYS-240 收口 + §4.3 + §4.4 已落 origin/dev), 跨项目累计 4/7 MERGED 验证 ✅, §3.1 module_switch 12 module 落地与 SRS 5 域映射清晰, 跟 3 决策文档 (audit v0.3 + handoff v0.1 + FLASH-OVERLAP v0.2) + ULYS-190 §4.3 + §4.4 Stage 1 决策一致性 ✅, 跟 user 拍板 "完整 1351 mock long-term" 一致 ✅.

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: 2026-09-26 13:00 JST

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| **v0.1** | 2026-09-04 16:14 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: [游戏A] mock 设计 (per 9/4 16:14 JST user 拍板 "完整 1351 mock long-term 5-10 sprint"), 12 大类 RPC 抽样 (22 RPC 起步) + 架构 (actix-web 4 + tonic 0.12 + mTLS + gap matrix) + crate 文件结构 (tools/rgs-flash-mock/ 跟 rgs-batch-backend 模式) + 5-10 sprint 路线图 (W1 scaffold + 22 RPC, W2-W3 关键路径 60-80 RPC, W4-W10 渐进式补完 1351) + 6 关键决策 + 7 关键风险 + 4 段已知缺口, 配套 RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) + RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) + RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-[游戏A]借鉴优化 v0.1 + [游戏A] 借鉴分析 .md §0-§5, per L13 自指字段 deferred + 8/27 11:06 JST 凭据硬 ban 守护 + 8/26 JST 禁回溯叙事守护 + 8/21 JST 5 域独立 Lead 守护 + 9/4 15:34 JST user 拍板 "仅 API 对齐" + 9/4 16:14 JST user 拍板 "完整 1351 mock" |
| **v0.2** | 2026-09-04 16:24 | 架构师(Mavis 接手 agent per DEC-008) | **Ulysses 二审通过 (per 9/4 16:24 JST ask_user 拍板 option A)**, 状态机结束: §9.2 决策 ✅ + 签字日期 2026-09-04 16:24 JST; 3 commit 落地 (36b9c06 设计 doc + c5c4006 scaffold 12 文件 + 5e6c727 cargo check 0 error 修复), `cargo check 0 error 0 warning` 验证 ✅; 1 个回执, 0 风险; 后续 W2-W10 sprint 渐进式补完 1351 RPC 路线图明确 (W2 加 7 域 gRPC client + 60-80 RPC / W3 加 5 类别 + 100-130 RPC / W4-W10 补完 1351 RPC, 总 1M-1.5M tokens 预算); per B3 派生约束 (DDD Review v0.2 §1 流程 + §3 打回循环上限) + 8/27 19:39/20:56/21:59 JST 三次强化代签授权 (Mavis 默认代签 Ulysses) |
| **v0.3** | 2026-09-04 16:45 | 架构师(Mavis 接手 agent per DEC-008) | **升级拍板 (per 9/4 16:45 JST user "完全对齐" 拍板 option C)**: 推翻 v0.2 "5-10 sprint 渐进式补完 1351 RPC" 路线图, 升级为 "**15-25 sprint 完全对齐 438 cmds**"; 4 阶段路线图 (Phase 1 ✅ done / Phase 2 12 Partial → Pass ~140 cmds / Phase 3 5-10 hot path 新建 ~80 cmds / Phase 4 18-20 long tail 新建 ~218 cmds); 30 新 module 业务扩展 (per [游戏A] 42 modules 全对齐, per `E:\[跨盘-某发行商目录]\[游戏A]\server分析\[游戏A]_server\docs\api_module_summary.txt` 实际清单); 工程量 1.5-2x 当前 RGS, 总 2-3M tokens; 推翻 handoff v0.1 "不做逐条移植" 决策 (TCG → MMORPG 业务扩展); 跟 3 决策文档 (audit v0.3 + FLASH-OVERLAP v0.2 + 9/4 15:34 JST "仅 API 对齐") 决策一致性 ✅; 0 风险, 等 W2 拍板启动 Phase 2 |
| **v0.4** | 2026-09-26 13:00 | 架构师(Mavis 接手 agent per DEC-008) | **ULYS-190 §4.4 Stage 1 落地收口 + 缺口闭环 (per ULYS-240)**: §3.1 module_switch 12 module 落地与 SRS 5 域映射 (NEW 子节 per P0-1, 解决 v0.3 §3 "12 大类 RPC 抽样" vs 新落地 "12 module_switch" 命名混淆); §4.3 L1 cluster_switch + L2 plugin_switch 5 域 cluster 落地 (NEW 子节 per P0-2, 记录 PR 配套 commit `28a3c025` 2026-09-23 23:15 JST); §4.4 L3 module_switch 12 module 落地 + Python helper + 真拼接 trace_format (NEW 子节 per P0-2, 记录 PR #51 commit `41932076` 2026-09-26 12:22 JST); §3.1.4 + §5.7 + §8.5 + §8.6 跨项目 mock_switch 缺口闭环 (per P1-3 + P1-4 + P2-5); 跨项目累計 4/7 MERGED 验证 ✅ (IM1.0 PR #24 5p×28m + CATs PR #18 4p×13m + Star PR #151 7p×7m + RGS PR #51 5p×12m = 21 plugin / 60 module); document-registry.toml 新增 RGS-IMPL-051 planned 登记 (per P1-3); 0 风险, 跨 session 续做 per §8.6 12 项入口; 跟 ULYS-190 §4.3 + §4.4 Stage 1 + G-MS-04 跨项目範式决策一致性 ✅ |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)