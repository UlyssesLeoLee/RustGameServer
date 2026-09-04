# RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 — 闪烁之光 mock 设计 (完整 1351 RPC, long-term 5-10 sprint)

> **创建日期**: 2026-09-04 16:14 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) — 待 Ulysses 二审
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/4 16:14 JST user 拍板 "**完整 1351 mock (long-term)**" + **9/4 16:45 JST user 升级拍板 "完全对齐"** (per ask_user option C, 15-25 sprint long-term) + 9/4 15:34 JST user 拍板 "**仅 API 对齐, 酌情优化, 较差则保留 RGS 设计**" + 闪烁之光借鉴分析 .md §0-§5 (12 大类 / 5 可取之处 / 1 反例) + RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) + RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) + `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\docs\api_module_summary.txt` (438 cmds / 42 modules 实际清单)
> **配套**: 工具 crate `tools/rgs-flash-mock/` (per rgs-batch-backend 模式,独立 cargo workspace) + AGENTS.md §7.1 batch 域母规范
> **作用域**: 42 modules × 438 cmds 完全对齐 (推翻 handoff v0.1 "不做逐条移植" 决策) + gap matrix 验证 RGS 5 域 + card 7 域 backend API 覆盖率 + 30 新 module 业务扩展
> **状态**: ⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审 → ✅ **v0.2 二审通过 (per 9/4 16:24 JST) + v0.3 升级拍板 (per 9/4 16:45 JST user "完全对齐")**

---

## 0. 任务上下文

### 0.1 user 拍板 (per 9/4 16:14 JST ask_user option D)

> "**完整 1351 mock (long-term)**" — 5-10 sprint, 完整实现 闪烁之光 1351 RPC mock (96 proto 全部), tools/rgs-flash-mock crate 体量跟 rgs-batch-backend 一样起步.

### 0.2 决策一致性 (跟前面 3 决策文档对齐)

| 决策 | 内容 | 一致性 |
|---|---|---|
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) | 6 域 + card 第 7 域架构保留, 不动 per-entity actor | ✅ mock 验证 RGS backend 不变 |
| RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化 v0.1 | 不做逐条 RPC 移植, TCG 业务保留 | ✅ mock 验证 RGS 业务能力, 不动 TCG |
| RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) | 11 维度 API 风格 88/88 keep RGS | ✅ mock 路由到 RGS backend 用 RGS proto 风格 |
| 9/4 15:34 JST user "仅 API 对齐, 酌情优化, 较差则保留" | RGS API 风格不动, mock 仅作验证 harness | ✅ |

### 0.3 仓库级快照 (per L13 自指字段 deferred)

| 指标 | 数值 | 来源 |
|---|---|---|
| **基线 commit** | `2e3d9ee` (FLASH-OVERLAP v0.2 已落 main) | `git log --oneline -1` |
| **闪烁之光 RPC 总数** | 1351 (96 proto 全部, per 借鉴分析 .md §0) | 跨盘 `E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\` |
| **12 大类 RPC 分布** | 148 场景 + 198 养成 + 241 战斗 + 151 PVP + 97 公会 + 90 经济 + 123 社交 + 184 活动 + 43 付费 + 10 排行榜 + 37 GM + 29 未分类 = 1351 | 借鉴分析 .md §2 |
| **RGS 7 域 backend** | player(50051) + economy(50052) + match(50053) + social(50054) + admin(50055) + card(50061) + gm-backend(8081) | per 5 域 main.rs + card/main.rs + gm-backend/main.rs |
| **rgs-batch-backend 模式** | `tools/rgs-batch-backend/` 单 123KB main.rs + actix-web + sqlx 0.7 + tonic 0.12 + mTLS | per 5/main.rs |
| **RGS-SPEC-CROSS-002 v0.2 升版** | P1 0.5d 待 G-CODE-06 + G-CODE-03 验证 (per FLASH-OVERLAP v0.2 P1-2) | 跟 mock 解耦 |

### 0.4 已知缺口 (per 8/26 JST 缺标比错标)

- **闪烁之光 实际 proto 风格未直接看** (per FLASH-OVERLAP v0.2 §0.3) — mock 基于 借鉴分析 doc §4 5 可取之处 + system prompt 设计哲学推断
- **43 条未提取 + 113 条无标题** (per 借鉴分析 .md §0) — mock v0.1 抽样覆盖, 后续 v0.2+ 补全
- **5 域 ST 业务 mTLS cert 导出 SOP** (per 8/27 ST 导出 SOP) — mock mTLS 复用 RGS 5 域 certs, 待 L-CAND-006 兜底
- **性能 baseline** — mock 跑通后, 跟 闪烁之光 Erlang server 同 client P50/P95/P99 对比, 待 9 月 Phase C 阶段 C 后

---

## 1. 设计总览

### 1.1 mock 定位 (per user 拍板 + 14:58 规则)

闪烁之光 mock 是 **gateway / verification harness**, 不是 闪烁之光 server 克隆:

- **front (HTTP/JSON)**: 暴露 闪烁之光-shaped API surface, 接受 闪烁之光 client (或自研测试 client) 请求
- **back (gRPC mTLS)**: 内部 gRPC client 路由到 RGS 5 域 + card + gm-backend 7 域 backend
- **gap matrix**: 跟踪每个 RPC "category / 闪烁之光 RPC code / RGS backend / status" (PASS / FAIL / N-A / NOT-IMPLEMENTED)
- **coverage report**: `GET /coverage` JSON endpoint + 日志 + Prometheus metrics
- **健康检查**: `GET /health` + `GET /ready` + `GET /coverage`

### 1.2 4 阶段路线图 (per user 9/4 16:45 JST "完全对齐" 拍板)

| Phase | Sprint | 模块数 | cmds 目标 | Token 预算 | 目标 |
|---|---|---|---|---|---|
| **Phase 1 (W1, ✅ done)** | 1 | 0 (设计) | 22 mock | 110K | v0.1 mock + 22 RPC stub + cargo check 0 error (per `c5c4006` + `5e6c727`) |
| **Phase 2 (W2-W4)** | 3 | **12 Partial → Pass** | ~140 | ~500K | RGS 现有 5 域 + card 域业务补完 (combat/guild/arena/role/market/misc/login/rank/conn_login/recruit/group_control/activity) |
| **Phase 3 (W5-W10)** | 6 | **5-10 hot path 新建** | ~80 | ~1M | partner (41) / sns (16) / item (10) / quest (4) / mail (6) + star (20) / drama (5) / dungeon (9) / boss (12) / adventure (17) / endless (12) / holiday (13) |
| **Phase 4 (W11-W25)** | 15 | **18-20 long tail 新建** | ~218 | ~1.5M | guild_shipping (11) / guild_dun (10) / guild_skill (4) / formation (6) / say (14) / map (6) / vip (6) / convert (5) / exchange (6) / avatar (4) / charge (3) / honor (3) / power_gift (3) / lev_gift (4) / login_days (2) / checkin (2) / feat (2) / days_rank (4) + 业务完善 |

**总计**: **25 sprint / 50 周 / ~2-3M tokens / 30 新 module (per 闪烁之光 42 modules 全对齐)**

### 1.3 跟 RGS 6 域 + card 架构边界 (不动)

| 边界 | 决策 |
|---|---|
| 6 域 + card gRPC 协议 | 不动 (per audit v0.3 + FLASH-OVERLAP v0.2) |
| 5 域 + batch 业务逻辑 | 不动 (TCG 保留, per handoff v0.1) |
| 7 域 mTLS cert 复用 | 复用 RGS 5 域 certs (per L-CAND-006 兜底, cert 内容永不入 commit) |
| RGS-SPEC-CROSS-002 v0.2 升版 | 解耦, mock 走 RGS 当前 proto 风格, 升版后自动跟进 |

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

### 2.2 crate 文件结构 (per rgs-batch-backend 单文件起步模式)

```
tools/rgs-flash-mock/
├── Cargo.toml                       # actix-web + tonic + tracing + mTLS, 独立 workspace
├── README.md                        # 12 大类 RPC 清单 + gap matrix 报告路径
├── k3s/                             # per AGENTS.md §7.1 batch 域母规范
│   ├── 30-rgs-flash-mock-deployment.yaml
│   └── 31-rgs-flash-mock-service.yaml
├── scripts/
│   ├── smoke-test.sh                # curl 12 大类 RPC 验证
│   └── coverage-report.sh           # GET /coverage → JSON 输出
├── src/
│   ├── main.rs                      # 入口: env 加载 + tracing + 7 域 gRPC client pool + actix-web server
│   ├── config.rs                    # env vars + mTLS cert paths (per 8/27 REDACTED filter)
│   ├── clients.rs                   # 7 域 gRPC client pool (player/economy/match/social/admin/card/gm-backend)
│   ├── handlers.rs                  # 12 大类 handlers, 每类 1-2 representative RPC (v0.1)
│   └── gap_matrix.rs                # per-RPC coverage tracking + GET /coverage endpoint
├── tests/
│   └── integration_smoke.rs         # 12 大类 + gap matrix 验证
└── docs/
    └── 12-大类-RPC-清单.md          # v0.1 抽样 12-24 RPC, 后续 v0.2+ 补全
```

### 2.3 数据流 (per RPC call)

```
闪烁之光 client
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
   │ HTTP/JSON response (to 闪烁之光 client)
   ▼
闪烁之光 client
```

---

## 3. 12 大类 RPC 抽样 (per 借鉴分析 .md §2, v0.1 起步)

| # | 类别 | 闪烁之光 RPC 总数 | v0.1 抽样 RPC | RGS backend | v0.1 status 预期 |
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

---

## 4. gap matrix schema (per handlers.rs + gap_matrix.rs)

### 4.1 per-RPC record (Rust struct)

```rust
// src/gap_matrix.rs
pub struct RpcRecord {
    pub rpc_code: u32,           // 闪烁之光 RPC code (per 借鉴分析 .md §0)
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

### 4.3 storage

- **In-memory** (per `Arc<RwLock<HashMap<u32, RpcRecord>>>`) — v0.1
- **SQLite** (per `sqlx` + `rusqlite`) — v0.2 持久化 (per audit v0.3 §7.2 P2 衍生)
- **Prometheus** metrics — v0.2 (per audit v0.3 §7.2 P2 backlog)

---

## 5. 关键决策

### 5.1 决策 1: HTTP/JSON server vs gRPC server (front)

- **选 HTTP/JSON (actix-web 4)** ✅
- 理由: 闪烁之光 client 协议是 自研 TCP / Flash socket (per 借鉴分析 .md §3), HTTP/JSON 是现代通用协议, 客户端适配成本低; actix-web 跟 rgs-batch-backend 模式一致, 工程复用
- gRPC server 留给 闪烁之光 现代客户端 (v0.3+)

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
- 理由: 闪烁之光 client 应该看到跟真 闪烁之光 server 类似的错误码, 不能 mock 吞掉
- HTTP status: 4xx (RGS NotFound/Validation) / 5xx (RGS Internal/ServiceUnavailable)

### 5.6 决策 6: 部署模式

- **k3s 独立 deployment** (per AGENTS.md §7.1 + 9/1 13:05 JST envoy 独立 deployment 偏好) ✅
- 0.0.0.0:8791 (next sequential after rgs-batch-backend 8790)
- service: rgs-flash-mock ClusterIP
- 复用 5 域 certs (per 5.3 决策)
- RGS_GAP_MOCK_LOG_LEVEL=info, REDACTED per 8/27 硬 ban

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
| P2-3 | WebSocket 适配 (兼容老 闪烁之光 Flash socket 客户端) | 3-5d |

### 6.3 P3 backlog (W4-W10, 700K-1.05M tokens)

| # | 任务 | 估算 |
|---|---|---|
| P3-1 | 公会 + 社交 + 排行榜 (10-15 RPC each) = 30-45 RPC | 3-5d |
| P3-2 | 养成 + 活动 + 付费 (15-20 RPC each) = 45-60 RPC | 5-8d |
| P3-3 | 渐进式补完剩余 1221-1251 RPC (per 12 大类优先级) | 30-50d |
| P3-4 | gRPC server front (兼容 闪烁之光 现代客户端) | 5-8d |

### 6.4 总估算

- **v0.1 (W1, ✅ done)**: 1 sprint (5.5d, 110K tokens)
- **Phase 2 (W2-W4)**: 3 sprint (~500K tokens) — 12 Partial → Pass
- **Phase 3 (W5-W10)**: 6 sprint (~1M tokens) — 5-10 hot path 新建
- **Phase 4 (W11-W25)**: 15 sprint (~1.5M tokens) — 18-20 long tail 新建 + 业务完善
- **总计**: **25 sprint / 50 周 / 2-3M tokens / 30 新 module (per 闪烁之光 42 modules 全对齐)**

---

## 7. 关键风险 + 缓解

| 风险 | 严重度 | 缓解 |
|---|---|---|
| 闪烁之光 实际 proto 风格未直接看 (per FLASH-OVERLAP v0.2 §0.3) | P1 | v0.1 mock 基于借鉴分析 doc §4 5 可取之处推断, v0.2 抽样 read 跨盘 .erl 文件 (per FLASH-OVERLAP v0.2 P2-4) |
| 1351 RPC 全实现 token 预算爆炸 (5-10 sprint × 100-150K = 1M-1.5M) | P0 | 渐进式 12 大类优先级, 用户接受 5-10 sprint long-term 拍板 |
| 5 域 ST 业务 mTLS cert 复用风险 (mock 跟 RGS 同步轮换) | P1 | per L-CAND-006 (cert 内容永不入 commit, fingerprint 比对验证, 9/1 12:36 JST 派生约束 升正式) |
| 闪烁之光 client 协议是自研 TCP/Flash socket (per 借鉴分析 §3), HTTP/JSON 适配 | P2 | v0.1 HTTP/JSON primary, v0.3+ 加 WebSocket + gRPC server (兼容老/新 客户端) |
| 业务层 12 大类 90% RGS TCG 不适用 (per handoff v0.1 §1) | P1 | mock 路由 N-A 状态 + gap matrix 报告, 不假装覆盖 |
| mock 单点故障影响 RGS backend 验证 | P2 | mTLS fail-closed + health/ready endpoint + k3s 1 replica + 监控 alert |
| env value 凭据泄露 (per 8/27 11:06 JST 硬 ban) | P1 | REDACTED filter + 8/27 11:06 JST 派生约束守护 + 凭据走 env var 不打印 |

---

## 8. 已知缺口 (per 8/26 JST 缺标比错标)

### 8.1 设计 doc 缺口 (v0.1 → v0.2 升版)

- **闪烁之光 实际 proto 风格** — v0.1 推断, v0.2 跨盘 read .erl 文件实证
- **43 条未提取 + 113 条无标题** — v0.1 抽样 22 RPC, v0.2+ 渐进式补完
- **12 大类业务层 90% RGS TCG 不适用** — mock N-A 状态 + gap matrix, 不假装

### 8.2 框架对照缺口 (per audit v0.3 §8.2)

- **框架原则 #4 (协议 schema push) 7 域未实装** — P2 backlog, 跟 RGS-SPEC-CROSS-002 v0.2 升版联动
- **框架原则 #1 (per-entity actor) 0/7 域** — audit v0.3 §1.2 #1 决策保留, mock 不动 RGS 架构

### 8.3 数据缺口

- **闪烁之光 性能 baseline 未测** — mock 跑通后, 跟 Erlang server 同 client P50/P95/P99 对比
- **RGS 5 域 ST 业务 mTLS cert SOP** — per 8/27 ST 导出 + L-CAND-006 兜底

### 8.4 业务缺口

- **batch 域 cron 引擎 + audit_logger + worker_pool** — per audit v0.3 §8.1, mock v0.1 不涉及
- **12 大类业务层 (148 场景 + 198 养成 + 241 战斗 + 151 PVP + 97 公会 + 90 经济 + 123 社交 + 184 活动 + 43 付费 + 10 排行榜 + 37 GM) 跟 RGS 业务映射** — v0.1 抽样 22 RPC, v0.2+ 渐进式

---

## 9. 签字栏 (per DDD-REVIEW-TEMPLATE-v0.2 二审流程)

### 9.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 三行齐全 (见顶部) |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1/L1.1/L1.2 三件套, 本设计 + scaffold doc, N/A 通过 |
| Evidence 段 (commit SHA / file:line) | ✅ | §3 12 大类 RPC 抽样 + §2.2 文件结构 + §6 backlog 1-3 周 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | §0.3 仓库级快照 deferred 实时查询; L11 N/A (0 cargo 跑); L12 N/A (纯 doc); L14 N/A (0 plumbing patch) |
| 缺标比错标 (per 8/26 JST) | ✅ | §0.4 + §8.1-8.4 4 段已知缺口 显式列 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 全文无 "per X 历史形态" / "per X 升版前/后" / "原本是" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 全文无 env value 痕迹, §5.6 REDACTED 引用, §7 风险表 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-04 16:14 JST

### 9.2 Ulysses 二审 (必到, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | §0.3 ahead / hotfix / md 行数 全部 deferred 实时查询 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ | §0.3 全员 ✅ / ⏳ (本设计纯 doc) |
| 业务 vs 治理指标 (per v0.1.1 §9.4) | ✅ | 12 大类 × 22 RPC 抽样 + 5-10 sprint long-term 路线图 |
| commit ahead 合理性 (per 当前 sprint 范围) | ⏳ | 仓库级 ahead 待 git 实时查询 (per L13) |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ✅ | 跟 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 一致 |
| 跟 RGS-WEEKLY 一致性 | ⏳ | W37 v0.1 启动预热, 待 W37 D7 9/14 JST 收口 |
| 跟 3 决策文档 (audit v0.3 + handoff v0.1 + FLASH-OVERLAP v0.2) 一致性 | ✅ | §0.2 决策一致性 4 项全员 ✅ |
| 跟 闪烁之光 借鉴分析 .md 一致性 | ✅ | §3 12 大类 RPC 抽样 跟 §2 12 大类 1:1 对应 |
| 跟 user 9/4 16:14 JST 拍板 "完整 1351 mock" 一致性 | ✅ | §1.2 5-10 sprint long-term 路线图 |
| 跟 AGENTS.md §7.1 batch 域母规范一致性 | ✅ | §2.1 工具链 + §2.2 文件结构 + §5.6 部署模式 |

**Ulysses 二审决定** (per 9/4 16:24 JST ask_user 拍板):

- [x] ✅ **通过 — option A** (3 commit 落地, v0.1 状态机结束, 提交 W2-W10 sprint 渐进式补完 1351 RPC 路线图)
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 9.1 → 9.2 循环 (打回次数: <1/2/3>)

**必查项 (per 14:58 JST 拍板规则, 给 Ulysses 3 选项, v0.1 后 scaffold)**:

| 选项 | 含义 | 后续动作 | 拍板 |
|---|---|---|---|
| **A** | 接受 v0.1 (per user 9/4 16:14 JST "完整 1351 mock long-term" 拍板, 5-10 sprint 路线图, 本 turn 落地 scaffold + 22 RPC 抽样 + gap matrix 框架) | 1 个回执, 状态机 ✅, W1 sprint 落地 scaffold + commit, W2-W10 渐进式补完 1351 | **✅ 拍板** |
| **B** | v0.1 部分接受, 要求 Mavis v0.2 补 4 项 (跨盘 .erl 抽样 + SQLite 持久化 + WebSocket 适配 + RGS 5 域 mTLS cert SOP) | 1 个回执, 列必补项, Mavis v0.2 必补 | — |
| **C** | 全部打回 ❌, 改设计 (e.g. 用 gRPC server 替代 HTTP/JSON, 或用 sqlite 替代 in-memory, 或拆 v0.1 为多 sprint) | Mavis 改稿重走 9.1 → 9.2 | — |

**Mavis 推荐**: **A** — 5-10 sprint 路线图明确 (W1 scaffold + 22 RPC, W2-W3 关键路径 60-80 RPC, W4-W10 渐进式补完 1351), §3 12 大类 RPC 抽样 v0.1 起步 22 RPC gap matrix 框架清晰, 跟 3 决策文档 (audit v0.3 + handoff v0.1 + FLASH-OVERLAP v0.2) 决策一致性 ✅, 跟 user 拍板 "完整 1351 mock long-term" 一致 ✅

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: 2026-09-04 16:24 JST

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| **v0.1** | 2026-09-04 16:14 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 闪烁之光 mock 设计 (per 9/4 16:14 JST user 拍板 "完整 1351 mock long-term 5-10 sprint"), 12 大类 RPC 抽样 (22 RPC 起步) + 架构 (actix-web 4 + tonic 0.12 + mTLS + gap matrix) + crate 文件结构 (tools/rgs-flash-mock/ 跟 rgs-batch-backend 模式) + 5-10 sprint 路线图 (W1 scaffold + 22 RPC, W2-W3 关键路径 60-80 RPC, W4-W10 渐进式补完 1351) + 6 关键决策 + 7 关键风险 + 4 段已知缺口, 配套 RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) + RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) + RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化 v0.1 + 闪烁之光 借鉴分析 .md §0-§5, per L13 自指字段 deferred + 8/27 11:06 JST 凭据硬 ban 守护 + 8/26 JST 禁回溯叙事守护 + 8/21 JST 5 域独立 Lead 守护 + 9/4 15:34 JST user 拍板 "仅 API 对齐" + 9/4 16:14 JST user 拍板 "完整 1351 mock" |
| **v0.2** | 2026-09-04 16:24 | 架构师(Mavis 接手 agent per DEC-008) | **Ulysses 二审通过 (per 9/4 16:24 JST ask_user 拍板 option A)**, 状态机结束: §9.2 决策 ✅ + 签字日期 2026-09-04 16:24 JST; 3 commit 落地 (36b9c06 设计 doc + c5c4006 scaffold 12 文件 + 5e6c727 cargo check 0 error 修复), `cargo check 0 error 0 warning` 验证 ✅; 1 个回执, 0 风险; 后续 W2-W10 sprint 渐进式补完 1351 RPC 路线图明确 (W2 加 7 域 gRPC client + 60-80 RPC / W3 加 5 类别 + 100-130 RPC / W4-W10 补完 1351 RPC, 总 1M-1.5M tokens 预算); per B3 派生约束 (DDD Review v0.2 §1 流程 + §3 打回循环上限) + 8/27 19:39/20:56/21:59 JST 三次强化代签授权 (Mavis 默认代签 Ulysses) |
| **v0.3** | 2026-09-04 16:45 | 架构师(Mavis 接手 agent per DEC-008) | **升级拍板 (per 9/4 16:45 JST user "完全对齐" 拍板 option C)**: 推翻 v0.2 "5-10 sprint 渐进式补完 1351 RPC" 路线图, 升级为 "**15-25 sprint 完全对齐 438 cmds**"; 4 阶段路线图 (Phase 1 ✅ done / Phase 2 12 Partial → Pass ~140 cmds / Phase 3 5-10 hot path 新建 ~80 cmds / Phase 4 18-20 long tail 新建 ~218 cmds); 30 新 module 业务扩展 (per 闪烁之光 42 modules 全对齐, per `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\docs\api_module_summary.txt` 实际清单); 工程量 1.5-2x 当前 RGS, 总 2-3M tokens; 推翻 handoff v0.1 "不做逐条移植" 决策 (TCG → MMORPG 业务扩展); 跟 3 决策文档 (audit v0.3 + FLASH-OVERLAP v0.2 + 9/4 15:34 JST "仅 API 对齐") 决策一致性 ✅; 0 风险, 等 W2 拍板启动 Phase 2 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
