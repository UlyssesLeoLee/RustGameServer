# rgs-flash-mock DEPRECATED 通告

> **生效日期**: 2026-09-09 12:35 JST
> **拍板**: Ulysses (per 2026-09-09 11:31 JST "闪烁之光前端本身就是用来验证 rgs 功能的, 所以不需要多余的为此设置 mock, 目的就是让 rgs 完全取代 erlang 版本")
> **作者**: Ulysses — Mavis 接手 (per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008)+自审+2026-09-09
> **修订人**: Ulysses — Mavis 接手

---

## 1. 为什么 deprecate

per Ulysses 2026-09-09 11:31 JST 战略意图:

> "闪烁之光前端本身就是用来验证 rgs 功能的, 所以不需要多余的为此设置 mock, 目的就是让 rgs 完全取代 erlang 版本, 实现同等或更强的后端能力"

**核心**: 零 mock 原则 — 真实 RGS 5+3 域 gRPC server + 真实 DB (Docker postgres 5433) + Node.js rgs-proxy 8084 gRPC→HTTP bridge, 不需要 22 RPC stub 中转层。

**rgs-flash-mock 的 22 RPC stub** 是早期 (9/4 JST) 为了在 RGS 域未完工时, 用 stub 占位验证前端 UI。9/9 JST W7-W9 派工 110 RPC 完工, 5/5 域 RGS gRPC server 跑通 (player / economy / match / social / admin), stub 已无必要。

---

## 2. 替代方案 (per 9/9 12:30 JST 完工)

[`tools/rgs-shanshuo-game/`](../rgs-shanshuo-game/README.md) — 闪烁之光 RGS 版 PoC

**核心差异**:

| 维度 | rgs-flash-mock (旧, DEPRECATED) | rgs-shanshuo-game (新) |
|---|---|---|
| 后端 | 22 RPC stub (actix-web 4) | 真实 RGS 5+3 域 gRPC (5 binary release profile) |
| 端口 | 8791 (HTTP/JSON) | 8084 (rgs-proxy gRPC→HTTP) + 50061-50065 (gRPC) |
| DB | 无 (in-memory stub) | 真实 Docker postgres 5433, 6 域 db (player/economy/match/social/admin/cluster_ops) |
| 业务 RPC | 22 stub 返回 mock JSON | 4 域真业务 RPC 拿真 DB 数据 (player.GetPlayer / economy.GetAccount / social.GetGuild / admin.QueryAuditLog) |
| UAT 验证 | 6 阶段 41 case | 4 case Playwright + 5 截图 evidence |
| Commit | fdba686 (W3) | `ffe1778` (PoC) |
| 验证报告 | RGS-DDD-2026-09-04-GAP-AUDIT_v0.3 | [`tools/rgs-shanshuo-game/docs/VALIDATION-REPORT-v0.1.md`](../rgs-shanshuo-game/docs/VALIDATION-REPORT-v0.1.md) |

---

## 3. 保留原因 (不立即删)

1. **回归 baseline**: W1-W3 完工的 22 RPC stub 测试 (commit 5978e3f, 36f8470, 045328e, 01aee71, fdba686) 可作为 5 域 RGS 真实业务 RPC 的回归对比 baseline
2. **mTLS 业务级 ST 工具链**: rgs-flash-mock 用的 tonic 0.12 + mTLS 跟 RGS 5 域一致, v0.2 ST 阶段可复用
3. **5 域 mock fixture**: 5 域 mock_data/ (per 5978e3f) 短期保留作 v0.1 demo 演示

**保留不等于维护**: 不再修 bug, 不加新功能, 不升版。仅 v0.2 ST 阶段评估后决定是否彻底删 (per Ulysses 拍板)。

---

## 4. 端口 8791 状态

- **9/9 12:35 JST 验证**: `Test-NetConnection 127.0.0.1 -Port 8791` = DOWN
- **如需重启**: 需 Ulysses 显式拍板 (per 9/8 15:29 JST 守门: 涉及 host 状态改变需 Ulysses 决策)
- **重启命令** (仅作记录, 不主动跑):
  ```bash
  cd tools/rgs-flash-mock
  cargo build --release
  export RGS_TLS_DIR=/path/to/rgs/certs
  ./target/release/rgs-flash-mock
  ```

---

## 5. 替代工具链 (per 9/9 12:30 JST)

| 任务 | 旧工具 (rgs-flash-mock) | 新工具 (rgs-shanshuo-game) |
|---|---|---|
| 启动 RGS 联动 | `./target/release/rgs-flash-mock` (单进程 8791) | `.\tools\rgs-shanshuo-game\start-5-rgs-services.ps1` (5 binary + 5 域 migrations + rgs-proxy 8084) |
| 验证后端 | `curl http://127.0.0.1:8791/coverage` | `curl http://127.0.0.1:8084/health` (5 client ready) |
| 调 RPC | `POST /rpc/<domain>/<rpc>` (stub) | `POST /<domain>/<rpc>` (真实 gRPC 透传) |
| 业务数据 | stub JSON (mock) | 真实 DB (player_db / economy_db / social_db / admin_db) |
| UAT | 41 case (per 5978e3f) | 4 case (per ffe1778) + 5 截图 |
| 验证报告 | RGS-DDD-2026-09-04-GAP-AUDIT_v0.3 | VALIDATION-REPORT-v0.1.md |

---

## 6. 已知缺口 (per 9/9 12:30 JST)

- **rgs-shanshuo-game/ 也未完工**: GAP-1 ~ GAP-8 (per VALIDATION-REPORT-v0.1.md §7), v0.2 评估
- **mTLS 业务级 ST 未跑**: 现 `RGS_ALLOW_INSECURE_GRPC=1` bypass, v0.2 ST 阶段介入
- **5 域 binary 部署到 k3s cluster 未做**: 现 Windows 本地运行, v0.2 评估
- **真实闪烁之光客户端 (zsyz_client Erlang beam) 改 config 未做**: per VALIDATION-REPORT-v0.1.md GAP-6, Ulysses 拍板

---

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-09 12:35 | Ulysses — Mavis 接手 | 标记 deprecated: README 顶 banner + Cargo.toml description + DEPRECATED.md 迁移路径 → tools/rgs-shanshuo-game/ |
